//! What a dragged column header shows: a ghost of its title that follows the
//! pointer (styled like the ref drag's), the header dimmed where it came from
//! and an accent insertion bar where it would land. Shared by every table
//! whose columns reorder by dragging a header (the Log, the commit column).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::theme::th;

/// The insertion bar's glyph.
pub(crate) const MARKER: &str = "\u{2503}";

/// Dims the cells of `rect` (the header being carried).
pub(crate) fn dim(buf: &mut Buffer, rect: Rect) {
    let area = buf.area;
    for y in rect.y..rect.y.saturating_add(rect.height) {
        for x in rect.x..rect.x.saturating_add(rect.width) {
            if x < area.x + area.width && y < area.y + area.height {
                buf[(x, y)].set_style(Style::default().add_modifier(Modifier::DIM));
            }
        }
    }
}

/// The accent insertion bar at `(x, y)`.
pub(crate) fn paint_marker(buf: &mut Buffer, x: u16, y: u16) {
    let area = buf.area;
    if x < area.x + area.width && y < area.y + area.height {
        buf.set_string(
            x,
            y,
            MARKER,
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD),
        );
    }
}

/// The ghost: ` Title ` on the accent, just below-right of the pointer so it
/// never hides the cell being aimed at, kept inside the frame.
pub(crate) fn paint_ghost(buf: &mut Buffer, title: &str, at: (u16, u16)) {
    let area = buf.area;
    if area.width == 0 || area.height == 0 {
        return;
    }
    let label = format!(" {title} ");
    let width = Line::from(label.as_str()).width() as u16;
    let (px, py) = at;
    let mut x = px.saturating_add(2);
    if x.saturating_add(width) > area.x + area.width {
        x = (area.x + area.width).saturating_sub(width).max(area.x);
    }
    let y = if py + 1 < area.y + area.height {
        py + 1
    } else {
        py.saturating_sub(1).max(area.y)
    };
    let max = (area.x + area.width).saturating_sub(x) as usize;
    buf.set_stringn(
        x,
        y,
        label,
        max,
        Style::default()
            .fg(th().chrome.on_bright)
            .bg(th().ui.accent)
            .add_modifier(Modifier::BOLD),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    }

    /// Red when the ghost is not drawn below-right of the pointer with the
    /// title, when it leaves the frame, or when the bar is not at its cell.
    #[test]
    fn the_ghost_follows_the_pointer_inside_the_frame_and_the_bar_sits_on_its_cell() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        paint_ghost(&mut buf, "Author", (10, 3));
        assert_eq!(&row(&buf, 4)[12..20], " Author ");
        assert!(row(&buf, 3).trim().is_empty(), "nothing on the pointer row");
        // At the right edge it is pulled back in.
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        paint_ghost(&mut buf, "Author", (39, 9));
        assert!(row(&buf, 8).contains(" Author "), "{}", row(&buf, 8));
        paint_marker(&mut buf, 5, 0);
        assert_eq!(buf[(5, 0)].symbol(), MARKER);
        // Off the buffer: nothing, no panic.
        paint_marker(&mut buf, 99, 0);
    }
}
