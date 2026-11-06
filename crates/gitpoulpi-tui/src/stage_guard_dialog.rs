//! "Sensitive files": the question the staging guard asks when a stage
//! (the file, folder and all-files stage, from the keyboard, a menu, a
//! drag-and-drop or the palette: they all end in one `Action`) would add
//! files that look like secrets.
//!
//! The daemon refused with `TypedError::SensitiveStage` and staged nothing;
//! the link hands the refused action here (`ask`). The dialog lists every file
//! with its reason (a path pattern, or the kind of secret found with a masked
//! preview: never a value) in a scrollable pane with a draggable bar, and has
//! two numbered answers:
//!
//! 1. **Stage anyway**: the action again, wrapped in `Action::AllowSensitive`.
//! 2. **Cancel, stage nothing**.
//!
//! No answer is highlighted until the user moves with an arrow key or presses
//! a digit, so a stray Enter answers nothing. Esc, the close cross and a click
//! outside the dialog cancel. Every mouse action has a key: PageUp/PageDown/
//! Home/End scroll like the wheel and the bar, 1/2 or Up/Down + Enter answer
//! like a click. Process-wide state, like the other small modals, because the
//! window restarts on every tab switch and the refusal comes from the link's
//! own thread.

use std::path::PathBuf;
use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_i18n::msg::stage_guard as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::daemon_link::{Action, DaemonLink};
use crate::dialog_kit;
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;
use crate::ui_kit::display_width;

/// How many lines one wheel notch scrolls.
const WHEEL_LINES: usize = 3;

/// A refused stage the user answered "Stage anyway" to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Staged {
    /// The repository the refusal came from.
    pub(crate) root: PathBuf,
    pub(crate) action: Action,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Path,
    Reason,
}

struct Open {
    root: PathBuf,
    action: Action,
    /// The files, one path line then one line per reason.
    lines: Vec<(Kind, String)>,
    /// The highlighted answer; `None` until an arrow key or a digit.
    cursor: Option<usize>,
    hover: Option<usize>,
    top: usize,
    content: usize,
    viewport: usize,
    track: Option<Track>,
    drag: BarDrag,
    pane: Rect,
    frame: Rect,
    answers: [Rect; 2],
    close: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
/// Refusals that arrived while the dialog was open.
static WAITING: Mutex<Vec<Open>> = Mutex::new(Vec::new());

/// A control character is shown as `\x1b`, never sent to the terminal: a file
/// name must not be able to repaint the screen it is judged on.
fn visible(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\t' => out.push_str("    "),
            c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Cuts `text` into pieces of at most `width` columns, anywhere.
fn wrap_hard(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = vec![String::new()];
    let mut used = 0usize;
    for ch in text.chars() {
        let w = display_width(&ch.to_string());
        if used > 0 && used + w > width {
            out.push(String::new());
            used = 0;
        }
        if let Some(last) = out.last_mut() {
            last.push(ch);
        }
        used += w;
    }
    out
}

fn reason_text(file: &Sensitive) -> String {
    match &file.reason {
        gitpoulpi_core::privacy::Reason::Pattern(p) => msg::reason_pattern(p),
        gitpoulpi_core::privacy::Reason::Secret { rule, preview } => {
            msg::reason_secret(rule, preview)
        }
    }
}

/// The path line and the reason lines of `files`, a path once however many
/// reasons it has.
fn lines_of(files: &[Sensitive]) -> Vec<(Kind, String)> {
    let mut out: Vec<(Kind, String)> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for file in files {
        if !seen.contains(&file.path.as_str()) {
            seen.push(&file.path);
            out.push((Kind::Path, visible(&file.path)));
            for same in files.iter().filter(|f| f.path == file.path) {
                out.push((Kind::Reason, format!("  {}", visible(&reason_text(same)))));
            }
        }
    }
    out
}

/// A stage `action` of the repository at `root` was refused for `files`:
/// opens the question (or queues it behind the one that is open). Callable
/// from any thread.
pub(crate) fn ask(root: PathBuf, action: Action, files: Vec<Sensitive>) {
    let open = Open {
        root,
        action,
        lines: lines_of(&files),
        cursor: None,
        hover: None,
        top: 0,
        content: 0,
        viewport: 1,
        track: None,
        drag: BarDrag::default(),
        pane: Rect::default(),
        frame: Rect::default(),
        answers: [Rect::default(); 2],
        close: Default::default(),
    };
    let mut slot = POPUP.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(open);
    } else {
        WAITING.lock().unwrap_or_else(|e| e.into_inner()).push(open);
    }
    drop(slot);
    crate::redraw::poke();
}

pub(crate) fn is_open() -> bool {
    POPUP.lock().is_ok_and(|p| p.is_some())
}

/// Closes the open question and brings up the next waiting one.
fn finish(slot: &mut Option<Open>) {
    let mut waiting = WAITING.lock().unwrap_or_else(|e| e.into_inner());
    *slot = if waiting.is_empty() {
        None
    } else {
        Some(waiting.remove(0))
    };
}

impl Open {
    fn max_top(&self) -> usize {
        self.content.saturating_sub(self.viewport)
    }

    fn scroll_to(&mut self, top: usize) {
        self.top = top.min(self.max_top());
    }

    fn scroll_by(&mut self, delta: isize) {
        self.scroll_to((self.top as isize + delta).max(0) as usize);
    }

    fn page(&self) -> isize {
        self.viewport.saturating_sub(1).max(1) as isize
    }

    fn answer_at(&self, col: u16, row: u16) -> Option<usize> {
        self.answers
            .iter()
            .position(|r| r.width > 0 && r.contains((col, row).into()))
    }

    fn staged(&self) -> Staged {
        Staged {
            root: self.root.clone(),
            action: self.action.clone(),
        }
    }
}

/// A key while the question is open (it swallows them all). `Some` is the
/// "Stage anyway" answer.
pub(crate) fn on_key(key: KeyEvent) -> Option<Staged> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    match key.code {
        KeyCode::Esc | KeyCode::Char('2') => {
            finish(&mut slot);
            None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            open.cursor = Some(open.cursor.map_or(0, |c| c.saturating_sub(1)));
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            open.cursor = Some(open.cursor.map_or(0, |c| (c + 1).min(1)));
            None
        }
        KeyCode::Char('1') => {
            let staged = open.staged();
            finish(&mut slot);
            Some(staged)
        }
        KeyCode::Enter => match open.cursor {
            // No default: Enter answers nothing until the user chose.
            None => None,
            Some(0) => {
                let staged = open.staged();
                finish(&mut slot);
                Some(staged)
            }
            Some(_) => {
                finish(&mut slot);
                None
            }
        },
        KeyCode::PageUp => {
            let page = open.page();
            open.scroll_by(-page);
            None
        }
        KeyCode::PageDown => {
            let page = open.page();
            open.scroll_by(page);
            None
        }
        KeyCode::Home => {
            open.scroll_to(0);
            None
        }
        KeyCode::End => {
            open.scroll_to(usize::MAX);
            None
        }
        _ => None,
    }
}

/// A mouse event while the question is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Staged> {
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    // The scrollbar first: its press, its drag (even off the track) and its
    // release.
    if let Some(top) = open
        .drag
        .on_mouse(open.track, mouse, open.content, open.top)
    {
        open.scroll_to(top);
        return None;
    }
    let at = ratatui::layout::Position::new(mouse.column, mouse.row);
    match mouse.kind {
        MouseEventKind::ScrollUp if open.pane.contains(at) => {
            open.scroll_by(-(WHEEL_LINES as isize));
            None
        }
        MouseEventKind::ScrollDown if open.pane.contains(at) => {
            open.scroll_by(WHEEL_LINES as isize);
            None
        }
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.close.set_hover(mouse.column, mouse.row);
            open.hover = open.answer_at(mouse.column, mouse.row);
            None
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if open.close.hit(mouse.column, mouse.row) {
                finish(&mut slot);
                return None;
            }
            match open.answer_at(mouse.column, mouse.row) {
                Some(0) => {
                    let staged = open.staged();
                    finish(&mut slot);
                    Some(staged)
                }
                Some(_) => {
                    finish(&mut slot);
                    None
                }
                None => {
                    // A click away cancels and is consumed.
                    if !open.frame.contains(at) {
                        finish(&mut slot);
                    }
                    None
                }
            }
        }
        _ => None,
    }
}

/// Sends the stage again, allowing sensitive files, when the answer came from
/// this window's repository.
pub(crate) fn apply(staged: Staged, link: &DaemonLink, repo_root: &std::path::Path) {
    if staged.root == repo_root {
        link.submit_action(Action::AllowSensitive(Box::new(staged.action)));
    }
}

/// Paints the question, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let width = 80u16.min(area.width.saturating_sub(4));
    if width < 30 || area.height < 12 {
        return;
    }
    // Title, blank, question (2 rows), the pane, the two answers and the hint.
    let fixed = 2 + 1 + 2 + 2 + 1 + 2;
    let want_pane = (open.lines.len() as u16 + 2).max(5);
    let height = (fixed + want_pane).min(area.height.saturating_sub(2));
    let outer = dialog_kit::centered(area, width, height);
    open.frame = outer;
    let buf = frame.buffer_mut();
    open.close.panel(buf, outer, msg::TITLE.get());
    let inner = Rect::new(
        outer.x + 2,
        outer.y + 1,
        outer.width.saturating_sub(4),
        outer.height.saturating_sub(2),
    );
    paint_body(open, buf, inner);
}

fn paint_body(open: &mut Open, buf: &mut Buffer, inner: Rect) {
    let panel = th().ui.panel;
    let normal = Style::default().fg(th().ui.text).bg(panel);
    let muted = Style::default().fg(th().ui.muted).bg(panel);
    let mut y = inner.y;
    for line in wrap_words(msg::QUESTION.get(), inner.width as usize)
        .into_iter()
        .take(2)
    {
        dialog_kit::put(buf, inner.x, y, &line, inner.width, normal);
        y += 1;
    }
    y += 1;
    let bottom = inner.bottom();
    let hint_y = bottom.saturating_sub(1);
    let a2_y = hint_y.saturating_sub(1);
    let a1_y = a2_y.saturating_sub(1);
    let pane = Rect::new(inner.x, y, inner.width, a1_y.saturating_sub(y));
    open.pane = pane;
    open.answers = [Rect::default(); 2];
    open.track = None;
    if pane.width >= 6 && pane.height >= 3 {
        paint_pane(open, buf, pane);
    }
    let answers = [
        (msg::ANSWER_STAGE.get(), a1_y),
        (msg::ANSWER_CANCEL.get(), a2_y),
    ];
    for (i, (label, row)) in answers.into_iter().enumerate() {
        if row < y || row >= bottom {
            continue;
        }
        let selected = open.cursor == Some(i);
        let hovered = open.hover == Some(i);
        let bg = dialog_kit::row_bg(selected, hovered);
        let rect = Rect::new(inner.x, row, inner.width, 1);
        dialog_kit::row_band(buf, rect, bg);
        let style = if selected {
            Style::default()
                .fg(th().ui.accent)
                .bg(bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th().ui.text).bg(bg)
        };
        let marker = if selected { "> " } else { "  " };
        dialog_kit::put(
            buf,
            inner.x + 1,
            row,
            &format!("{marker}{}. {label}", i + 1),
            inner.width.saturating_sub(1),
            style,
        );
        open.answers[i] = rect;
    }
    if hint_y >= y {
        dialog_kit::put(buf, inner.x, hint_y, msg::HINT.get(), inner.width, muted);
    }
}

fn paint_pane(open: &mut Open, buf: &mut Buffer, pane: Rect) {
    use ratatui::widgets::{Block, BorderType, Borders, Widget};
    let panel = th().ui.panel;
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(th().ui.muted).bg(panel))
        .style(Style::default().bg(panel))
        .render(pane, buf);
    let text_x = pane.x + 2;
    let text_w = pane.width.saturating_sub(4) as usize;
    let rows = pane.height.saturating_sub(2) as usize;
    let mut wrapped: Vec<(Kind, String)> = Vec::new();
    for (kind, text) in &open.lines {
        for piece in wrap_hard(text, text_w) {
            wrapped.push((*kind, piece));
        }
    }
    open.content = wrapped.len();
    open.viewport = rows.max(1);
    open.top = open.top.min(open.max_top());
    for (i, (kind, text)) in wrapped.iter().skip(open.top).take(rows).enumerate() {
        let style = match kind {
            Kind::Path => Style::default()
                .fg(th().ui.text)
                .bg(panel)
                .add_modifier(Modifier::BOLD),
            Kind::Reason => Style::default().fg(th().status.warning).bg(panel),
        };
        dialog_kit::put(
            buf,
            text_x,
            pane.y + 1 + i as u16,
            text,
            text_w as u16,
            style,
        );
    }
    open.track = Track::vertical(pane, open.content, open.top);
    if let Some(track) = open.track {
        scrollbar::render_buf(buf, track, true, open.drag.held());
    }
}

/// Greedy word wrap on spaces, by display width.
fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && display_width(&line) + 1 + display_width(word) > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

#[cfg(test)]
mod tests;
