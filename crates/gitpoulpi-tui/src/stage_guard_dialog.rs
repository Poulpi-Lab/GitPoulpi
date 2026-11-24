//! "Sensitive files": the question the staging guard asks when a stage
//! (the file, folder and all-files stage, from the keyboard, a menu, a
//! drag-and-drop or the palette: they all end in one `Action`) would add
//! files that look like secrets.
//!
//! The daemon refused with `TypedError::SensitiveStage` and staged nothing;
//! the link hands the refused action here (`ask`). The dialog lists every file
//! with its reason (a path pattern, or the kind of secret found with a masked
//! preview: never a value) in a scrollable pane with a draggable bar, and has
//! two numbered answers (and a "Show in file" button, below):
//!
//! 1. **Stage anyway**: the action again, wrapped in `Action::AllowSensitive`.
//! 2. **Cancel, stage nothing**.
//!
//! No answer is highlighted until the user moves with an arrow key or presses
//! a digit, so a stray Enter answers nothing. Esc, the close cross and a click
//! outside the dialog cancel. Every mouse action has a key: PageUp/PageDown/
//! Home/End scroll like the wheel and the bar, 1/2 or Up/Down + Enter answer
//! like a click.
//!
//! A finding inside a file names its line (`path:line`). One finding is
//! focused (a click on its row, `Tab` / `Shift+Tab` / `Ctrl+J` / `Ctrl+K`);
//! **Show in file** (the button, `S`, and the palette's "Sensitive Files: Show
//! in File") opens that file in the File View, scrolled to the line and
//! highlighting it. While the File View is up the dialog steps aside (it is
//! not "open", so the File View owns the keys and the mouse); closing the
//! File View brings the question back, untouched. The window drives that
//! round trip through [`peek_request`] and [`peek_line`]. Process-wide state, like the other small modals, because the
//! window restarts on every tab switch and the refusal comes from the link's
//! own thread.

use std::path::PathBuf;
use std::sync::Mutex;

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_i18n::msg::stage_guard as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::daemon_link::{Action, DaemonLink, Ticket, TicketGuard};
use crate::dialog_kit;

const WIN: dialog_kit::Win = dialog_kit::Win::movable("stage_guard");
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;
use crate::ui_kit::display_width;

/// The key of "Show in file" (a fixed key of this modal, like 1 and 2).
const SHOW_KEY: char = 'S';

/// How many lines one wheel notch scrolls.
const WHEEL_LINES: usize = 3;

/// A refused stage the user answered "Stage anyway" to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Staged {
    /// The repository the refusal came from.
    pub(crate) root: PathBuf,
    pub(crate) action: Action,
    /// The ticket the refused stage ran under: "Stage anyway" resubmits
    /// under it.
    pub(crate) ticket: Option<Ticket>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Path,
    Reason,
}

/// One finding the dialog can show in its file: a path, and the line of the
/// secret in it (none for a path-pattern finding).
#[derive(Clone, PartialEq, Eq, Debug)]
struct Finding {
    path: String,
    line: Option<u32>,
}

/// One row of the list and the finding it belongs to.
struct Row {
    kind: Kind,
    text: String,
    finding: usize,
}

struct Open {
    root: PathBuf,
    action: Action,
    /// Cancels the ticket when the dialog is dropped unanswered (Cancel, Esc,
    /// the close cross, a click away); the "Stage anyway" answer takes it.
    ticket: Option<TicketGuard>,
    /// What the list is about, in list order.
    findings: Vec<Finding>,
    /// The focused finding (what "Show in file" opens).
    focus: usize,
    /// The next paint scrolls the focused finding into view.
    reveal: bool,
    /// The files, one path line then one line per reason.
    lines: Vec<Row>,
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
    /// Where the finding rows were painted, for a click.
    row_hits: Vec<(Rect, usize)>,
    show_button: Rect,
    show_hover: bool,
    close: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);

/// The round trip to the File View, owned by the window's tick.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Peek {
    /// The dialog is up.
    Off,
    /// "Show in file" was asked: the window opens the file.
    Wanted {
        root: PathBuf,
        path: String,
        line: Option<u32>,
    },
    /// The File View is up for the dialog; `line` is still to be scrolled to.
    Open { line: Option<u32> },
}

static PEEK: Mutex<Peek> = Mutex::new(Peek::Off);

fn peek() -> std::sync::MutexGuard<'static, Peek> {
    PEEK.lock().unwrap_or_else(|e| e.into_inner())
}
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

/// The findings and the rows of `files`: a path (with `:line` when the
/// secret has one) once however many reasons it has.
fn lines_of(files: &[Sensitive]) -> (Vec<Finding>, Vec<Row>) {
    let mut findings: Vec<Finding> = Vec::new();
    let mut rows: Vec<Row> = Vec::new();
    for file in files {
        let finding = Finding {
            path: file.path.clone(),
            line: file.line,
        };
        if findings.contains(&finding) {
            continue;
        }
        let at = findings.len();
        let shown = match finding.line {
            Some(line) => format!("{}:{line}", file.path),
            None => file.path.clone(),
        };
        rows.push(Row {
            kind: Kind::Path,
            text: visible(&shown),
            finding: at,
        });
        for same in files
            .iter()
            .filter(|f| f.path == file.path && f.line == file.line)
        {
            rows.push(Row {
                kind: Kind::Reason,
                text: format!("  {}", visible(&reason_text(same))),
                finding: at,
            });
        }
        findings.push(finding);
    }
    (findings, rows)
}

/// A stage `action` of the repository at `root` was refused for `files`:
/// opens the question (or queues it behind the one that is open). Callable
/// from any thread.
pub(crate) fn ask(root: PathBuf, action: Action, files: Vec<Sensitive>, ticket: Option<Ticket>) {
    let (findings, lines) = lines_of(&files);
    let open = Open {
        root,
        action,
        ticket: TicketGuard::new(ticket),
        findings,
        focus: 0,
        reveal: false,
        lines,
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
        row_hits: Vec::new(),
        show_button: Rect::default(),
        show_hover: false,
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

/// Whether the question is up and owns the keys and the mouse: not while its
/// file is being looked at in the File View.
pub(crate) fn is_open() -> bool {
    POPUP.lock().is_ok_and(|p| p.is_some()) && *peek() == Peek::Off
}

/// The window is showing a file for the question (so the File View's close
/// brings the question back): the tick polls [`peek_closed`] with it.
pub(crate) fn peeking() -> bool {
    *peek() != Peek::Off
}

/// "Show in file" on the focused finding, from the palette.
pub(crate) fn show_in_file() {
    if let Ok(mut slot) = POPUP.lock() {
        if let Some(open) = slot.as_mut() {
            open.begin_peek();
        }
    }
}

/// The file the window must open for "Show in file" in the repository at
/// `root` (and the line to scroll to), once: the dialog steps aside from then
/// on. `None` when nothing was asked, or the ask was for another repository.
pub(crate) fn peek_request(root: &std::path::Path) -> Option<(String, Option<u32>)> {
    let mut state = peek();
    let Peek::Wanted {
        root: wanted,
        path,
        line,
    } = &*state
    else {
        return None;
    };
    if wanted != root {
        return None;
    }
    let out = (path.clone(), *line);
    *state = Peek::Open { line: out.1 };
    Some(out)
}

/// The line still to be scrolled to once the file has loaded; [`peek_applied`]
/// takes it.
pub(crate) fn peek_line() -> Option<u32> {
    match &*peek() {
        Peek::Open { line } => *line,
        _ => None,
    }
}

/// The File View was scrolled to the line (or cannot be).
pub(crate) fn peek_applied() {
    let mut state = peek();
    if matches!(&*state, Peek::Open { .. }) {
        *state = Peek::Open { line: None };
    }
}

/// The File View closed (or never opened): the question is back.
pub(crate) fn peek_closed() {
    let mut state = peek();
    if matches!(&*state, Peek::Open { .. }) {
        *state = Peek::Off;
        drop(state);
        crate::redraw::poke();
    }
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

    /// Moves the focus over the findings, wrapping.
    fn focus_by(&mut self, delta: isize) {
        let n = self.findings.len() as isize;
        if n > 0 {
            self.focus = (self.focus as isize + delta).rem_euclid(n) as usize;
            self.reveal = true;
        }
    }

    /// Asks the window to open the focused finding's file; the question steps
    /// aside until the File View closes.
    fn begin_peek(&self) {
        let Some(finding) = self.findings.get(self.focus) else {
            return;
        };
        *peek() = Peek::Wanted {
            root: self.root.clone(),
            path: finding.path.clone(),
            line: finding.line,
        };
        crate::redraw::poke();
    }

    fn staged(&mut self) -> Staged {
        Staged {
            root: self.root.clone(),
            action: self.action.clone(),
            ticket: self.ticket.take().map(TicketGuard::take),
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
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Tab => {
            open.focus_by(1);
            None
        }
        KeyCode::BackTab => {
            open.focus_by(-1);
            None
        }
        KeyCode::Char('j') if ctrl => {
            open.focus_by(1);
            None
        }
        KeyCode::Char('k') if ctrl => {
            open.focus_by(-1);
            None
        }
        KeyCode::Char('s' | 'S') if !ctrl => {
            open.begin_peek();
            None
        }
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
            open.show_hover = open.show_button.width > 0 && open.show_button.contains(at);
            open.hover = open.answer_at(mouse.column, mouse.row);
            None
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if open.close.hit(mouse.column, mouse.row) {
                finish(&mut slot);
                return None;
            }
            if open.show_button.width > 0 && open.show_button.contains(at) {
                open.begin_peek();
                return None;
            }
            if let Some(finding) = open
                .row_hits
                .iter()
                .find(|(r, _)| r.contains(at))
                .map(|(_, f)| *f)
            {
                open.focus = finding;
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
        link.submit_as(
            staged.ticket,
            Action::AllowSensitive(Box::new(staged.action)),
        );
    } else if let Some(ticket) = staged.ticket {
        // Answered from another repository's window: nothing was staged.
        crate::daemon_link::cancel_ticket(ticket);
    }
}

/// Paints the question, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    if peeking() {
        return;
    }
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let width = 80u16.min(area.width.saturating_sub(4));
    if width < 30 || area.height < 12 {
        open.row_hits.clear();
        open.frame = dialog_kit::too_small::paint_closable(
            frame.buffer_mut(),
            area,
            msg::TITLE.get(),
            &mut open.close,
        );
        return;
    }
    // Title, blank, question (2 rows), the pane, the button row, the two
    // answers and the hint.
    let fixed = 2 + 1 + 2 + 2 + 1 + 1 + 2;
    let want_pane = (open.lines.len() as u16 + 2).max(5);
    let height = (fixed + want_pane).min(area.height.saturating_sub(2));
    let outer = WIN.place(area, dialog_kit::centered(area, width, height));
    open.frame = outer;
    let buf = frame.buffer_mut();
    open.close.framed(buf, outer, msg::TITLE.get(), &WIN, true);
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
    let button_y = a1_y.saturating_sub(1);
    let pane = Rect::new(inner.x, y, inner.width, button_y.saturating_sub(y));
    open.pane = pane;
    open.answers = [Rect::default(); 2];
    open.track = None;
    open.row_hits.clear();
    open.show_button = Rect::default();
    if button_y > y + 3 {
        let label = format!("{} [{}]", msg::SHOW_IN_FILE.get(), SHOW_KEY);
        let state = dialog_kit::ButtonState::Normal {
            hovered: open.show_hover,
            focused: false,
        };
        open.show_button = dialog_kit::button(buf, inner.x, button_y, &label, state);
    }
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
    let mut wrapped: Vec<(Kind, String, usize)> = Vec::new();
    for row in &open.lines {
        for piece in wrap_hard(&row.text, text_w) {
            wrapped.push((row.kind, piece, row.finding));
        }
    }
    open.content = wrapped.len();
    open.viewport = rows.max(1);
    open.top = open.top.min(open.max_top());
    if std::mem::take(&mut open.reveal) {
        if let Some(first) = wrapped.iter().position(|(_, _, f)| *f == open.focus) {
            let last = wrapped
                .iter()
                .rposition(|(_, _, f)| *f == open.focus)
                .unwrap_or(first);
            if first < open.top {
                open.top = first;
            } else if last >= open.top + rows {
                open.top = (last + 1).saturating_sub(rows).min(first);
            }
            open.top = open.top.min(open.max_top());
        }
    }
    for (i, (kind, text, finding)) in wrapped.iter().skip(open.top).take(rows).enumerate() {
        let y = pane.y + 1 + i as u16;
        let band = Rect::new(pane.x + 1, y, pane.width.saturating_sub(2), 1);
        let focused = *finding == open.focus && open.findings.len() > 1;
        if focused {
            dialog_kit::fill(buf, band, Style::default().bg(th().ui.selection));
        }
        open.row_hits.push((band, *finding));
        let row_bg = if focused { th().ui.selection } else { panel };
        let style = match kind {
            Kind::Path => Style::default()
                .fg(th().ui.text)
                .bg(row_bg)
                .add_modifier(Modifier::BOLD),
            Kind::Reason => Style::default().fg(th().status.warning).bg(row_bg),
        };
        dialog_kit::put(buf, text_x, y, text, text_w as u16, style);
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
