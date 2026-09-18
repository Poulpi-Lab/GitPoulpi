//! How the multi-line editors inside Settings (hook, paths, AI instructions)
//! close: the shared cross, "Discard changes?" when the text was edited,
//! and a swallowed outside click on edited text (the cross flashes).

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::dialog_kit::{DiscardGuard, DismissKind, Verdict};

/// What the editor does with an event this guard looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Act {
    /// Not the guard's: the editor handles it.
    Pass,
    /// Spent (a confirmation is up or answered, a click was swallowed).
    Spent,
    /// Close the editor and drop the text.
    Close,
}

/// The close state of one editor: the text it opened with.
#[derive(Debug, Default)]
pub(super) struct EditorClose {
    guard: DiscardGuard,
    original: String,
}

impl EditorClose {
    pub(super) fn new(original: &str) -> Self {
        Self {
            guard: DiscardGuard::default(),
            original: original.to_string(),
        }
    }

    /// The text differs from what the editor opened with.
    pub(super) fn dirty(&self, text: &str) -> bool {
        text != self.original
    }

    /// Paints the closable frame.
    pub(super) fn panel(&mut self, buf: &mut Buffer, rect: Rect, title: &str) {
        self.guard.panel(buf, rect, title);
    }

    /// Paints the confirmation when it is open (call last).
    pub(super) fn render(&mut self, buf: &mut Buffer) {
        self.guard.render(buf);
    }

    /// The cross as last painted.
    #[cfg(test)]
    pub(super) fn cross(&self) -> Option<Rect> {
        self.guard.close.rect()
    }

    /// Whether "Discard changes?" is up.
    pub(super) fn asking(&self) -> bool {
        self.guard.is_open()
    }

    /// The decision for a close request on `text`.
    pub(super) fn request(&mut self, kind: DismissKind, text: &str) -> Act {
        if !self.dirty(text) {
            return Act::Close;
        }
        if kind == DismissKind::Outside {
            self.guard.close.request_flash();
        } else {
            self.guard.ask();
        }
        Act::Spent
    }

    /// A key while the confirmation is up, else Esc as a close request.
    pub(super) fn key(&mut self, key: KeyEvent, text: &str) -> Act {
        if let Some(verdict) = self.guard.key(key) {
            return settle(verdict);
        }
        if key.code == KeyCode::Esc {
            return self.request(DismissKind::Esc, text);
        }
        Act::Pass
    }

    /// The confirmation's mouse, the cross and the click outside.
    pub(super) fn mouse(&mut self, mouse: MouseEvent, text: &str) -> Act {
        if let Some(verdict) = self.guard.mouse(mouse) {
            return settle(verdict);
        }
        match self.guard.pointer(mouse) {
            Some(kind) => self.request(kind, text),
            None => Act::Pass,
        }
    }

    /// The Cancel button.
    pub(super) fn cancel_button(&mut self, text: &str) -> Act {
        self.request(DismissKind::Cancel, text)
    }
}

fn settle(verdict: Verdict) -> Act {
    match verdict {
        Verdict::Discard => Act::Close,
        Verdict::Stay => Act::Spent,
    }
}

/// Paints `text` right-aligned on the frame's bottom border.
pub(super) fn footer(buf: &mut Buffer, rect: Rect, text: &str) {
    let label = format!(" {text} ");
    let width = crate::display_width(&label) as u16;
    if rect.width < width + 4 || rect.height < 2 {
        return;
    }
    let style = ratatui::style::Style::default()
        .fg(crate::th().ui.text)
        .bg(crate::th().ui.panel);
    crate::dialog_kit::put(
        buf,
        rect.right() - 2 - width,
        rect.bottom() - 1,
        &label,
        width,
        style,
    );
}
