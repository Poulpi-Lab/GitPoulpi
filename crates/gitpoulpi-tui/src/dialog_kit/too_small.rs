//! The minimal frame a dialog paints when the window is too small for its
//! body: a title (clipped), the close cross and one line saying why
//! ("Window too small: enlarge it"). Returning without painting would leave an
//! invisible modal that still eats keys; this way the user sees what is wrong
//! and can close the dialog (the cross, Esc).
//!
//! Two shapes: [`note`] for a dialog that already painted its own frame (the
//! `framed` ones) and only lacks the message, [`paint`] / [`paint_closable`]
//! for a dialog that bails out before painting anything.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::{centered, panel_closable, put, CloseHit};
use crate::display_width;
use crate::th;
use gitpoulpi_i18n::msg;

/// The one-line message.
fn text() -> &'static str {
    msg::dialog::TOO_SMALL.get()
}

/// Writes the message inside an already painted frame `rect` (one row under
/// the top border). Nothing when the frame has no row or column to spare.
pub(crate) fn note(buf: &mut Buffer, rect: Rect) {
    if rect.height < 3 || rect.width < 6 {
        return;
    }
    put(
        buf,
        rect.x + 2,
        rect.y + 1,
        text(),
        rect.width.saturating_sub(4),
        Style::default().fg(th().ui.muted).bg(th().ui.panel),
    );
}

/// The frame's rect: as wide as the message needs (never wider than `area`),
/// three rows tall, centered.
pub(crate) fn rect_in(area: Rect) -> Rect {
    let want = (display_width(text()) as u16).saturating_add(4);
    centered(area, want, 3)
}

/// Paints the minimal frame in `area` and returns the close cross's hit rect
/// (`None` when even the cross does not fit).
pub(crate) fn paint(buf: &mut Buffer, area: Rect, title: &str, hovered: bool) -> Option<Rect> {
    let rect = rect_in(area);
    let cross = panel_closable(buf, rect, title, hovered);
    note(buf, rect);
    cross
}

/// [`paint`] for a dialog that keeps its cross in a [`CloseHit`]; returns the
/// frame's rect (what an outside-click test compares with).
pub(crate) fn paint_closable(
    buf: &mut Buffer,
    area: Rect,
    title: &str,
    close: &mut CloseHit,
) -> Rect {
    let rect = rect_in(area);
    close.panel(buf, rect, title);
    note(buf, rect);
    rect
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog_kit::buffer_text;

    #[test]
    fn paints_title_cross_and_message_in_a_three_row_frame() {
        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);
        let mut close = CloseHit::default();
        let rect = paint_closable(&mut buf, area, "Tags", &mut close);
        let text = buffer_text(&buf);
        assert!(text.contains("Tags"), "{text}");
        assert!(text.contains(msg::dialog::TOO_SMALL.get()), "{text}");
        assert_eq!(rect.height, 3);
        let cross = close.rect().expect("the cross is painted and hit-testable");
        assert!(close.hit(cross.x, cross.y));
        assert!(rect.contains((cross.x, cross.y).into()));
    }

    #[test]
    fn clips_inside_a_narrow_window_instead_of_overflowing() {
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        let cross = paint(&mut buf, area, "A very long dialog title", false);
        assert!(cross.is_some());
        let text = buffer_text(&buf);
        assert!(text.lines().all(|l| display_width(l) <= 20), "{text}");
        assert!(text.contains('\u{2026}'), "{text}");
    }

    #[test]
    fn a_window_with_no_room_for_a_frame_does_not_panic() {
        let area = Rect::new(0, 0, 4, 2);
        let mut buf = Buffer::empty(area);
        let _ = paint(&mut buf, area, "T", false);
    }
}
