//! The "Discard changes?" guard of the multi-line forms (new issue, new
//! pull / merge request): the close cross, the frame (for the click
//! outside) and the confirmation, in one field the form owns. The form
//! implements [`super::Dismiss`] with `Guard::ConfirmWhenDirty` and calls
//! [`DiscardGuard::ask`] when [`super::dismiss`] answers `Confirm`.

use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::discard_changes as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{CloseHit, Confirm, ConfirmEvent, DismissKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Keep,
    Discard,
}

/// What the open confirmation decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Still asking, or back to the form.
    Stay,
    /// Throw the typed work away and close.
    Discard,
}

#[derive(Debug, Default)]
pub(crate) struct DiscardGuard {
    pub(crate) close: CloseHit,
    frame: Rect,
    confirm: Option<Confirm<Choice>>,
}

impl DiscardGuard {
    /// Paints the closable frame of a form that moves, and remembers it.
    pub(crate) fn framed(&mut self, buf: &mut Buffer, rect: Rect, title: &str, win: &super::Win) {
        self.frame = rect;
        let enabled = self.confirm.is_none();
        self.close.framed(buf, rect, title, win, enabled);
    }

    /// Paints the confirmation over the form when it is open (call last).
    pub(crate) fn render(&mut self, buf: &mut Buffer) {
        let area = buf.area;
        if let Some(c) = self.confirm.as_mut() {
            c.render(buf, area);
        }
    }

    /// Whether the confirmation is up.
    pub(crate) fn is_open(&self) -> bool {
        self.confirm.is_some()
    }

    /// Opens "Discard changes?" (Keep editing is the safe default button).
    pub(crate) fn ask(&mut self) {
        self.confirm = Some(
            Confirm::new(
                msg::TITLE.get(),
                msg::BODY.get(),
                vec![
                    (msg::KEEP.get().to_string(), Choice::Keep),
                    (msg::DISCARD.get().to_string(), Choice::Discard),
                ],
                Choice::Keep,
            )
            .primary(Choice::Discard),
        );
    }

    fn settle(&mut self, event: ConfirmEvent<Choice>) -> Verdict {
        match event {
            ConfirmEvent::Stay => Verdict::Stay,
            ConfirmEvent::Chose(choice) => {
                self.confirm = None;
                match choice {
                    Choice::Keep => Verdict::Stay,
                    Choice::Discard => Verdict::Discard,
                }
            }
        }
    }

    /// A key while the confirmation is open; `None` when it is not.
    pub(crate) fn key(&mut self, key: KeyEvent) -> Option<Verdict> {
        let event = self.confirm.as_mut()?.on_key(key);
        Some(self.settle(event))
    }

    /// A mouse event while the confirmation is open; `None` when it is not.
    pub(crate) fn mouse(&mut self, mouse: MouseEvent) -> Option<Verdict> {
        let event = self.confirm.as_mut()?.on_mouse(mouse);
        Some(self.settle(event))
    }

    /// Hover on the cross, and which close request (cross or outside) a
    /// left press is, if any.
    pub(crate) fn pointer(&mut self, mouse: MouseEvent) -> Option<DismissKind> {
        let (col, row) = (mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.close.set_hover(col, row);
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(col, row) {
                    Some(DismissKind::Cross)
                } else if !self.frame.contains((col, row).into()) {
                    Some(DismissKind::Outside)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
