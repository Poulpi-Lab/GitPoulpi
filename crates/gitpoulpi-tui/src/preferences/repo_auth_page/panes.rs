//! The two panels the SSH & credentials page opens over itself: the
//! confirmation before a helper or token command is saved (the hook-trust
//! pattern of `flow/hook_trust.rs`, whose `visible` and `wrap_hard` it
//! shares) and the connection test's results. Both hold a scrollable text
//! pane with a draggable scrollbar, a close cross and a key for every mouse
//! action (PgUp/PgDn/Home/End scroll, Esc closes).

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit::{self, CloseHit};
use crate::flow::hook_trust::{visible, wrap_hard, wrap_words};
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;

/// How many lines one wheel notch scrolls.
const WHEEL_LINES: usize = 3;

/// How a line of a pane is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Heading,
    Normal,
    Muted,
    Good,
    Bad,
    /// A command, shown verbatim.
    Command,
}

/// A bordered, scrollable list of lines.
#[derive(Default)]
pub(super) struct TextPane {
    lines: Vec<(Kind, String)>,
    top: usize,
    content: usize,
    viewport: usize,
    track: Option<Track>,
    drag: BarDrag,
    rect: Rect,
}

impl TextPane {
    pub(super) fn new(lines: Vec<(Kind, String)>) -> Self {
        Self {
            lines,
            viewport: 1,
            ..Self::default()
        }
    }

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

    #[cfg(test)]
    pub(super) fn top(&self) -> usize {
        self.top
    }

    #[cfg(test)]
    pub(super) fn track(&self) -> Option<Track> {
        self.track
    }

    /// Scroll keys. `lines`: the arrows scroll by one line too (the results
    /// panel; the confirmation keeps the arrows for its answers).
    pub(super) fn key(&mut self, key: KeyEvent, lines: bool) -> bool {
        match key.code {
            KeyCode::PageUp => self.scroll_by(-self.page()),
            KeyCode::PageDown => self.scroll_by(self.page()),
            KeyCode::Home => self.scroll_to(0),
            KeyCode::End => self.scroll_to(usize::MAX),
            KeyCode::Up if lines => self.scroll_by(-1),
            KeyCode::Down if lines => self.scroll_by(1),
            _ => return false,
        }
        true
    }

    /// The scrollbar (press, drag, release) and the wheel.
    pub(super) fn mouse(&mut self, mouse: MouseEvent) -> bool {
        if let Some(top) = self
            .drag
            .on_mouse(self.track, mouse, self.content, self.top)
        {
            self.scroll_to(top);
            return true;
        }
        let over = self.rect.contains((mouse.column, mouse.row).into());
        match mouse.kind {
            MouseEventKind::ScrollUp if over => {
                self.scroll_by(-(WHEEL_LINES as isize));
                true
            }
            MouseEventKind::ScrollDown if over => {
                self.scroll_by(WHEEL_LINES as isize);
                true
            }
            _ => false,
        }
    }

    pub(super) fn render(&mut self, buf: &mut Buffer, pane: Rect) {
        use ratatui::widgets::{Block, BorderType, Borders, Widget};
        self.rect = pane;
        self.track = None;
        if pane.width < 6 || pane.height < 3 {
            return;
        }
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
        for (kind, text) in &self.lines {
            let pieces = if *kind == Kind::Command {
                wrap_hard(text, text_w)
            } else if text.is_empty() {
                vec![String::new()]
            } else {
                wrap_hard_words(text, text_w)
            };
            for piece in pieces {
                wrapped.push((*kind, piece));
            }
        }
        self.content = wrapped.len();
        self.viewport = rows.max(1);
        self.top = self.top.min(self.max_top());
        for (i, (kind, text)) in wrapped.iter().skip(self.top).take(rows).enumerate() {
            let base = Style::default().fg(th().ui.text).bg(panel);
            let style = match kind {
                Kind::Heading => Style::default()
                    .fg(th().ui.accent)
                    .bg(panel)
                    .add_modifier(Modifier::BOLD),
                Kind::Muted => Style::default().fg(th().ui.muted).bg(panel),
                Kind::Good => Style::default().fg(th().status.success).bg(panel),
                Kind::Bad => Style::default().fg(th().status.warning).bg(panel),
                Kind::Normal | Kind::Command => base,
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
        self.track = Track::vertical(pane, self.content, self.top);
        if let Some(track) = self.track {
            scrollbar::render_buf(buf, track, true, self.drag.held());
        }
    }
}

/// Word wrap that still cuts a word longer than the line.
fn wrap_hard_words(text: &str, width: usize) -> Vec<String> {
    // Keep leading blanks (an indented line stays indented).
    let indent: String = text.chars().take_while(|c| *c == ' ').collect();
    let body = text.trim_start_matches(' ');
    let room = width.saturating_sub(indent.chars().count()).max(1);
    let mut out = Vec::new();
    for line in wrap_words(body, room) {
        for piece in wrap_hard(&line, room) {
            out.push(format!("{indent}{piece}"));
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// A centred, closable panel: where the frame is, and the cross.
fn frame_rect(area: Rect, want_w: u16, want_h: u16) -> Rect {
    dialog_kit::centered(area, want_w.min(area.width), want_h.min(area.height))
}

// ----------------------------------------------------------------- results

/// What the results panel did with an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultsEvent {
    Ignored,
    Handled,
    Close,
}

/// The connection test's results.
pub(super) struct ResultsPane {
    pane: TextPane,
    close: CloseHit,
    frame: Rect,
}

impl ResultsPane {
    pub(super) fn new(lines: Vec<(Kind, String)>) -> Self {
        Self {
            pane: TextPane::new(lines),
            close: CloseHit::default(),
            frame: Rect::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn pane(&self) -> &TextPane {
        &self.pane
    }

    pub(super) fn key(&mut self, key: KeyEvent) -> ResultsEvent {
        if key.code == KeyCode::Esc {
            return ResultsEvent::Close;
        }
        if self.pane.key(key, true) {
            ResultsEvent::Handled
        } else {
            ResultsEvent::Ignored
        }
    }

    pub(super) fn mouse(&mut self, mouse: MouseEvent) -> ResultsEvent {
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) if self.close.hit(mouse.column, mouse.row) => {
                return ResultsEvent::Close;
            }
            _ => {}
        }
        // The panel is modal: it takes every event.
        self.pane.mouse(mouse);
        ResultsEvent::Handled
    }

    pub(super) fn render(&mut self, buf: &mut Buffer, area: Rect) {
        use gitpoulpi_i18n::msg::repo_auth as t;
        let rect = frame_rect(area, 96, area.height.saturating_sub(4).max(8));
        self.frame = rect;
        dialog_kit::fill(buf, rect, Style::default().bg(th().ui.panel));
        self.close.panel(buf, rect, t::RESULTS_TITLE.get());
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(3),
        );
        self.pane.render(buf, inner);
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        dialog_kit::put(
            buf,
            rect.x + 2,
            rect.bottom().saturating_sub(2),
            t::RESULTS_HINT.get(),
            rect.width.saturating_sub(4),
            muted,
        );
    }
}

// ----------------------------------------------------------------- confirm

/// What the confirmation answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfirmEvent {
    /// Used (scrolled, moved the highlight), or not for the panel.
    Handled,
    /// Save and allow.
    Save,
    /// Cancel, Esc or the close cross: nothing is saved.
    Cancel,
}

/// "Allow this command?": the command verbatim and its hash, two numbered
/// answers and no default until an arrow key or a digit.
pub(super) struct CommandConfirm {
    pane: TextPane,
    close: CloseHit,
    cursor: Option<usize>,
    hover: Option<usize>,
    answers: [Rect; 2],
    frame: Rect,
}

impl CommandConfirm {
    /// `helper`: whether it is a credential helper (else a token command);
    /// `shown`: the command, one entry per line; `hash`: its SHA-256.
    pub(super) fn new(helper: bool, shown: &str, hash: &str) -> Self {
        use gitpoulpi_i18n::msg::repo_auth as t;
        let mut lines = vec![
            (
                Kind::Heading,
                if helper {
                    t::CONFIRM_HELPER_HEADING.get()
                } else {
                    t::CONFIRM_COMMAND_HEADING.get()
                }
                .to_string(),
            ),
        ];
        for (i, line) in shown.split('\n').enumerate() {
            let prefix = if i == 0 { "$ " } else { "  " };
            lines.push((Kind::Command, format!("{prefix}{}", visible(line))));
        }
        lines.push((Kind::Normal, String::new()));
        lines.push((Kind::Muted, t::confirm_hash(hash)));
        Self {
            pane: TextPane::new(lines),
            close: CloseHit::default(),
            cursor: None,
            hover: None,
            answers: [Rect::default(); 2],
            frame: Rect::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    #[cfg(test)]
    pub(super) fn pane(&self) -> &TextPane {
        &self.pane
    }

    #[cfg(test)]
    pub(super) fn close_rect(&self) -> Option<Rect> {
        self.close.rect()
    }

    pub(super) fn key(&mut self, key: KeyEvent) -> ConfirmEvent {
        match key.code {
            KeyCode::Esc => ConfirmEvent::Cancel,
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = Some(self.cursor.map_or(0, |c| c.saturating_sub(1)));
                ConfirmEvent::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = Some(self.cursor.map_or(0, |c| (c + 1).min(1)));
                ConfirmEvent::Handled
            }
            KeyCode::Enter => match self.cursor {
                Some(0) => ConfirmEvent::Save,
                Some(_) => ConfirmEvent::Cancel,
                // No default: Enter answers nothing until the user chose.
                None => ConfirmEvent::Handled,
            },
            KeyCode::Char('1') => ConfirmEvent::Save,
            KeyCode::Char('2') => ConfirmEvent::Cancel,
            _ => {
                self.pane.key(key, false);
                ConfirmEvent::Handled
            }
        }
    }

    fn answer_at(&self, col: u16, row: u16) -> Option<usize> {
        self.answers
            .iter()
            .position(|r| r.width > 0 && r.contains((col, row).into()))
    }

    pub(super) fn mouse(&mut self, mouse: MouseEvent) -> ConfirmEvent {
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = self.answer_at(mouse.column, mouse.row);
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(mouse.column, mouse.row) {
                    return ConfirmEvent::Cancel;
                }
                match self.answer_at(mouse.column, mouse.row) {
                    Some(0) => return ConfirmEvent::Save,
                    Some(_) => return ConfirmEvent::Cancel,
                    None => {}
                }
            }
            _ => {}
        }
        self.pane.mouse(mouse);
        ConfirmEvent::Handled
    }

    pub(super) fn render(&mut self, buf: &mut Buffer, area: Rect) {
        use gitpoulpi_i18n::msg::repo_auth as t;
        let rect = frame_rect(area, 90, 20);
        self.frame = rect;
        dialog_kit::fill(buf, rect, Style::default().bg(th().ui.panel));
        self.close.panel(buf, rect, t::CONFIRM_TITLE.get());
        let body = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let normal = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = body.y;
        for line in wrap_words(t::CONFIRM_QUESTION.get(), body.width as usize)
            .into_iter()
            .take(3)
        {
            dialog_kit::put(buf, body.x, y, &line, body.width, normal);
            y += 1;
        }
        let hint_y = body.bottom().saturating_sub(1);
        let a2_y = hint_y.saturating_sub(1);
        let a1_y = a2_y.saturating_sub(1);
        let pane = Rect::new(
            body.x - 1,
            y,
            body.width + 2,
            a1_y.saturating_sub(y),
        );
        self.pane.render(buf, pane);
        self.answers = [Rect::default(); 2];
        for (i, (label, row)) in [
            (t::ANSWER_SAVE.get(), a1_y),
            (t::ANSWER_CANCEL.get(), a2_y),
        ]
        .into_iter()
        .enumerate()
        {
            if row < y || row >= body.bottom() {
                continue;
            }
            let selected = self.cursor == Some(i);
            let bg = dialog_kit::row_bg(selected, self.hover == Some(i));
            let band = Rect::new(body.x, row, body.width, 1);
            dialog_kit::row_band(buf, band, bg);
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
                body.x + 1,
                row,
                &format!("{marker}{}. {label}", i + 1),
                body.width.saturating_sub(1),
                style,
            );
            self.answers[i] = band;
        }
        if hint_y >= y {
            dialog_kit::put(buf, body.x, hint_y, t::CONFIRM_HINT.get(), body.width, muted);
        }
    }
}
