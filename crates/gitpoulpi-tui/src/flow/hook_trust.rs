//! The question the Flow dialog asks before a repository's committed
//! workflow hooks run: "This repository's workflow runs these commands on
//! your machine. Trust them?", with the commands shown verbatim in a
//! scrollable pane (draggable scrollbar) and two numbered answers.
//!
//! The pane never decides anything: it reports an [`Event`] and the dialog
//! asks the daemon (`FlowTrustHooks`, which re-reads the file and trusts only
//! the hash it finds on disk). No answer is highlighted until the user moves
//! with an arrow key or presses a digit, so a stray Enter answers nothing.
//! Every mouse action has a key: PgUp/PgDn/Home/End scroll like the wheel and
//! the scrollbar, 1/2 or Up/Down + Enter answer like a click, Esc goes back.

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::workflow::HookInfo;
use gitpoulpi_i18n::msg::flow_hook_trust as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit;
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;
use crate::ui_kit::display_width;

/// How many lines one wheel notch scrolls.
const WHEEL_LINES: usize = 3;

/// What the user chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    /// "Trust and run".
    Trust,
    /// "Run without hooks".
    Skip,
}

/// What an input did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Event {
    /// Not for the pane.
    Ignored,
    /// Used (scrolled, moved the highlight).
    Handled,
    /// Esc: back to the preview, nothing answered.
    Cancel,
    Pick(Pick),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Heading,
    Phase,
    Command,
    Blank,
}

pub(crate) struct HookTrust {
    /// The hash the daemon gave for the file's hooks (what gets trusted).
    pub(crate) hash: String,
    /// The list trusted before, when the hooks changed since.
    changed: bool,
    lines: Vec<(Kind, String)>,
    /// The highlighted answer; `None` until an arrow key or a digit.
    cursor: Option<usize>,
    hover: Option<usize>,
    top: usize,
    content: usize,
    viewport: usize,
    track: Option<Track>,
    drag: BarDrag,
    /// The scroll pane's outer rect and the answers, from the last paint.
    pane: Rect,
    answers: [Rect; 2],
}

/// A control character is shown as `\x1b`, never sent to the terminal:
/// a command must not be able to repaint the screen it is judged on.
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

fn push_hooks(lines: &mut Vec<(Kind, String)>, heading: &str, hooks: &[(String, String)]) {
    lines.push((Kind::Heading, heading.to_string()));
    for (phase, command) in hooks {
        lines.push((Kind::Phase, visible(phase)));
        for (i, line) in command.split('\n').enumerate() {
            let prefix = if i == 0 { "$ " } else { "  " };
            lines.push((Kind::Command, format!("{prefix}{}", visible(line))));
        }
    }
}

/// Cuts `text` into pieces of at most `width` columns, anywhere (a command
/// is shown whole, never truncated).
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

impl HookTrust {
    /// `now`: the commands the repository wants to run; `previous`: the list
    /// trusted before when the file changed since.
    pub(crate) fn new(hash: String, now: &[(String, String)], previous: Option<&[HookInfo]>) -> Self {
        let mut lines = Vec::new();
        push_hooks(&mut lines, msg::COMMANDS_HEADING.get(), now);
        if let Some(old) = previous {
            lines.push((Kind::Blank, String::new()));
            let old: Vec<(String, String)> = old
                .iter()
                .map(|h| (h.phase.clone(), h.command.clone()))
                .collect();
            push_hooks(&mut lines, msg::PREVIOUS_HEADING.get(), &old);
        }
        Self {
            hash,
            changed: previous.is_some(),
            lines,
            cursor: None,
            hover: None,
            top: 0,
            content: 0,
            viewport: 1,
            track: None,
            drag: BarDrag::default(),
            pane: Rect::default(),
            answers: [Rect::default(); 2],
        }
    }

    /// The highlighted answer (a test seam).
    #[cfg(test)]
    pub(crate) fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// The first visible line of the pane (a test seam).
    #[cfg(test)]
    pub(crate) fn top(&self) -> usize {
        self.top
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

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Event {
        match key.code {
            KeyCode::Esc => Event::Cancel,
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = Some(self.cursor.map_or(0, |c| c.saturating_sub(1)));
                Event::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = Some(self.cursor.map_or(0, |c| (c + 1).min(1)));
                Event::Handled
            }
            KeyCode::Enter => match self.cursor {
                Some(0) => Event::Pick(Pick::Trust),
                Some(_) => Event::Pick(Pick::Skip),
                // No default: Enter answers nothing until the user chose.
                None => Event::Handled,
            },
            KeyCode::Char('1') => Event::Pick(Pick::Trust),
            KeyCode::Char('2') => Event::Pick(Pick::Skip),
            KeyCode::PageUp => {
                self.scroll_by(-self.page());
                Event::Handled
            }
            KeyCode::PageDown => {
                self.scroll_by(self.page());
                Event::Handled
            }
            KeyCode::Home => {
                self.scroll_to(0);
                Event::Handled
            }
            KeyCode::End => {
                self.scroll_to(usize::MAX);
                Event::Handled
            }
            // Tab and the rest belong to the dialog (its buttons).
            _ => Event::Ignored,
        }
    }

    fn answer_at(&self, col: u16, row: u16) -> Option<usize> {
        self.answers
            .iter()
            .position(|r| r.width > 0 && r.contains((col, row).into()))
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Event {
        // The scrollbar first: its press, its drag (even off the track) and
        // its release.
        if let Some(top) = self
            .drag
            .on_mouse(self.track, mouse, self.content, self.top)
        {
            self.scroll_to(top);
            return Event::Handled;
        }
        let over_pane = self.pane.contains((mouse.column, mouse.row).into());
        match mouse.kind {
            MouseEventKind::ScrollUp if over_pane => {
                self.scroll_by(-(WHEEL_LINES as isize));
                Event::Handled
            }
            MouseEventKind::ScrollDown if over_pane => {
                self.scroll_by(WHEEL_LINES as isize);
                Event::Handled
            }
            MouseEventKind::Moved => {
                self.hover = self.answer_at(mouse.column, mouse.row);
                Event::Ignored
            }
            MouseEventKind::Down(MouseButton::Left) => {
                match self.answer_at(mouse.column, mouse.row) {
                    Some(0) => Event::Pick(Pick::Trust),
                    Some(_) => Event::Pick(Pick::Skip),
                    None => Event::Ignored,
                }
            }
            _ => Event::Ignored,
        }
    }

    /// Paints the question into `area` (the dialog's body). Hit rects are
    /// taken from this walk.
    pub(crate) fn render(&mut self, buf: &mut Buffer, area: Rect) {
        let panel = th().ui.panel;
        let bold_warn = Style::default()
            .fg(th().status.warning)
            .bg(panel)
            .add_modifier(Modifier::BOLD);
        let normal = Style::default().fg(th().ui.text).bg(panel);
        let muted = Style::default().fg(th().ui.muted).bg(panel);
        let mut y = area.y;
        let bottom = area.bottom();
        if self.changed {
            dialog_kit::put(
                buf,
                area.x,
                y,
                msg::HEADING_CHANGED.get(),
                area.width,
                bold_warn,
            );
            y += 1;
        }
        for line in wrap_words(msg::QUESTION.get(), area.width as usize)
            .into_iter()
            .take(2)
        {
            dialog_kit::put(buf, area.x, y, &line, area.width, normal);
            y += 1;
        }
        // Below the pane: the two answers and the hint.
        let hint_y = bottom.saturating_sub(1);
        let a2_y = hint_y.saturating_sub(1);
        let a1_y = a2_y.saturating_sub(1);
        let pane = Rect::new(area.x, y, area.width, a1_y.saturating_sub(y));
        self.pane = pane;
        self.answers = [Rect::default(); 2];
        self.track = None;
        if pane.width >= 6 && pane.height >= 3 {
            self.render_pane(buf, pane);
        }
        let answers = [
            (msg::ANSWER_TRUST.get(), a1_y),
            (msg::ANSWER_SKIP.get(), a2_y),
        ];
        for (i, (label, row)) in answers.into_iter().enumerate() {
            if row < y || row >= bottom {
                continue;
            }
            let selected = self.cursor == Some(i);
            let hovered = self.hover == Some(i);
            let bg = dialog_kit::row_bg(selected, hovered);
            let rect = Rect::new(area.x, row, area.width, 1);
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
                area.x + 1,
                row,
                &format!("{marker}{}. {label}", i + 1),
                area.width.saturating_sub(1),
                style,
            );
            self.answers[i] = rect;
        }
        if hint_y >= y {
            dialog_kit::put(buf, area.x, hint_y, msg::HINT.get(), area.width, muted);
        }
    }

    fn render_pane(&mut self, buf: &mut Buffer, pane: Rect) {
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
        // The wrapped pane: every logical line cut to the width.
        let mut wrapped: Vec<(Kind, String)> = Vec::new();
        for (kind, text) in &self.lines {
            if *kind == Kind::Blank {
                wrapped.push((Kind::Blank, String::new()));
                continue;
            }
            for piece in wrap_hard(text, text_w) {
                wrapped.push((*kind, piece));
            }
        }
        self.content = wrapped.len();
        self.viewport = rows.max(1);
        self.top = self.top.min(self.max_top());
        for (i, (kind, text)) in wrapped.iter().skip(self.top).take(rows).enumerate() {
            let style = match kind {
                Kind::Heading => Style::default()
                    .fg(th().ui.accent)
                    .bg(panel)
                    .add_modifier(Modifier::BOLD),
                Kind::Phase => Style::default().fg(th().ui.muted).bg(panel),
                Kind::Command | Kind::Blank => Style::default().fg(th().ui.text).bg(panel),
            };
            dialog_kit::put(buf, text_x, pane.y + 1 + i as u16, text, text_w as u16, style);
        }
        self.track = Track::vertical(pane, self.content, self.top);
        if let Some(track) = self.track {
            scrollbar::render_buf(buf, track, true, self.drag.held());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseEventKind};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn many(n: usize) -> Vec<(String, String)> {
        (0..n)
            .map(|i| ("after_start".to_string(), format!("echo command-{i}")))
            .collect()
    }

    fn painted(pane: &mut HookTrust) -> (Buffer, Rect) {
        let area = Rect::new(0, 0, 80, 22);
        let mut buf = Buffer::empty(area);
        pane.render(&mut buf, area);
        (buf, area)
    }

    #[test]
    fn no_answer_is_chosen_until_an_arrow_or_a_digit() {
        let mut pane = HookTrust::new("h".into(), &many(1), None);
        painted(&mut pane);
        // Enter with nothing highlighted answers nothing.
        assert_eq!(pane.on_key(key(KeyCode::Enter)), Event::Handled);
        assert_eq!(pane.cursor(), None);
        assert_eq!(pane.on_key(key(KeyCode::Down)), Event::Handled);
        assert_eq!(pane.cursor(), Some(0), "the first arrow only starts the highlight");
        assert_eq!(pane.on_key(key(KeyCode::Enter)), Event::Pick(Pick::Trust));
        assert_eq!(pane.on_key(key(KeyCode::Down)), Event::Handled);
        assert_eq!(pane.on_key(key(KeyCode::Enter)), Event::Pick(Pick::Skip));
        assert_eq!(pane.on_key(key(KeyCode::Char('1'))), Event::Pick(Pick::Trust));
        assert_eq!(pane.on_key(key(KeyCode::Char('2'))), Event::Pick(Pick::Skip));
        assert_eq!(pane.on_key(key(KeyCode::Esc)), Event::Cancel);
    }

    #[test]
    fn the_commands_are_shown_whole_and_a_control_character_is_spelled_out() {
        let long = "Q".repeat(150);
        let hooks = vec![
            ("after_start".to_string(), long.clone()),
            ("before_finish".to_string(), "echo \u{1b}[2J\tdone".to_string()),
        ];
        let mut pane = HookTrust::new("h".into(), &hooks, None);
        let (buf, area) = painted(&mut pane);
        let text = dialog_kit::buffer_text(&buf);
        assert!(!text.contains('\u{1b}'), "an escape never reaches the screen");
        assert!(text.contains("\\x1b[2J    done"), "{text}");
        // The 150-char command wraps over several rows, none lost.
        let xs: usize = (0..area.height)
            .map(|y| {
                (0..area.width)
                    .filter(|x| buf[(*x, y)].symbol() == "Q")
                    .count()
            })
            .sum();
        assert!(xs > 0 && xs <= 150);
        // Scroll to the end: every x has been on screen at some point.
        let mut seen = xs;
        while pane.top() < pane.max_top() {
            pane.on_key(key(KeyCode::PageDown));
            let (b, _) = painted(&mut pane);
            seen += (0..area.height)
                .map(|y| {
                    (0..area.width)
                        .filter(|x| b[(*x, y)].symbol() == "Q")
                        .count()
                })
                .sum::<usize>();
        }
        assert!(seen >= 150, "{seen}");
    }

    #[test]
    fn keys_wheel_and_the_scrollbar_scroll_the_same_pane() {
        let mut pane = HookTrust::new("h".into(), &many(40), None);
        painted(&mut pane);
        assert!(pane.track.is_some(), "a long list shows its scrollbar");
        assert_eq!(pane.top(), 0);
        pane.on_key(key(KeyCode::PageDown));
        let paged = pane.top();
        assert!(paged > 0);
        pane.on_key(key(KeyCode::Home));
        assert_eq!(pane.top(), 0);
        // The wheel over the pane.
        let inside = (pane.pane.x + 3, pane.pane.y + 2);
        pane.on_mouse(mouse(MouseEventKind::ScrollDown, inside.0, inside.1));
        assert_eq!(pane.top(), WHEEL_LINES);
        // The thumb is dragged: press on it, move to the bottom of the track.
        let track = pane.track.unwrap();
        let thumb_row = track.rect.y + track.thumb_start;
        let col = track.rect.x;
        assert_eq!(
            pane.on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), col, thumb_row)),
            Event::Handled
        );
        pane.on_mouse(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            col,
            track.rect.y + track.track_len,
        ));
        assert_eq!(pane.top(), pane.max_top(), "dragged to the end");
        pane.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), col, track.rect.y));
        pane.on_key(key(KeyCode::End));
        assert_eq!(pane.top(), pane.max_top());
    }

    #[test]
    fn a_click_on_an_answer_picks_it_and_the_old_list_shows_when_the_hooks_changed() {
        let old = vec![HookInfo {
            phase: "after_start".into(),
            command: "echo the-old-one".into(),
            personal: false,
        }];
        let mut pane = HookTrust::new("h".into(), &many(1), Some(&old));
        let (buf, _) = painted(&mut pane);
        let text = dialog_kit::buffer_text(&buf);
        assert!(text.contains("The hooks changed since you trusted them"));
        assert!(text.contains("echo command-0") && text.contains("echo the-old-one"));
        let skip = pane.answers[1];
        assert_eq!(
            pane.on_mouse(mouse(
                MouseEventKind::Down(MouseButton::Left),
                skip.x + 2,
                skip.y
            )),
            Event::Pick(Pick::Skip)
        );
        let trust = pane.answers[0];
        assert_eq!(
            pane.on_mouse(mouse(
                MouseEventKind::Down(MouseButton::Left),
                trust.x + 2,
                trust.y
            )),
            Event::Pick(Pick::Trust)
        );
    }
}
