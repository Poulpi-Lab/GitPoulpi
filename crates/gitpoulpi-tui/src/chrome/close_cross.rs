//! The one close cross every closable framed surface paints.
//!
//! ` ✕ ` (3 cells) on the block's top border, `block.right() - 5`, with the
//! plain-glyph hover. There is never an "Esc" text beside it: Esc keeps
//! working, it just is not advertised on the frame. The hit rect is what
//! [`paint`] returns, so a click target exists exactly when a cross was
//! drawn (paint walk), never from a second guess at the layout.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::theme::{plain_button_style, th};

const CROSS: &str = " \u{2715} ";
const WIDTH: u16 = 3;
/// A block narrower than this is too small to carry the cross.
const MIN_BLOCK_WIDTH: u16 = 8;

/// Where the cross sits for `block`, or `None` when it cannot be drawn
/// whole (block too narrow, or off the buffer).
fn spot(buf: &Buffer, block: Rect) -> Option<Rect> {
    if block.width < MIN_BLOCK_WIDTH || block.height == 0 {
        return None;
    }
    let rect = Rect::new(block.right() - 5, block.y, WIDTH, 1);
    (buf.area.intersection(rect) == rect).then_some(rect)
}

fn put(buf: &mut Buffer, rect: Rect, style: Style) {
    buf.set_stringn(rect.x, rect.y, CROSS, WIDTH as usize, style);
}

/// Paint the cross on `block`'s top border (after the block itself) and
/// return its rect, the hit target. `None` when nothing was drawn.
pub(crate) fn paint(buf: &mut Buffer, block: Rect, hovered: bool) -> Option<Rect> {
    let rect = spot(buf, block)?;
    put(buf, rect, plain_button_style(hovered));
    crate::pointer_map::hand(rect);
    Some(rect)
}

/// Make the cross blink: used when a click outside a dirty form was
/// swallowed, to point at the way out.
#[allow(dead_code)] // first caller: the dirty-form outside click (slice C2)
pub(crate) fn flash(buf: &mut Buffer, block: Rect) -> Option<Rect> {
    let rect = spot(buf, block)?;
    put(
        buf,
        rect,
        Style::default()
            .bg(th().ui.accent)
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD),
    );
    Some(rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::{Block, BorderType, Borders, Widget};

    fn framed(width: u16) -> (Buffer, Rect) {
        let area = Rect::new(0, 0, width, 5);
        let mut buf = Buffer::empty(area);
        Block::default()
            .title(" a view ")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .render(area, &mut buf);
        (buf, area)
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    #[test]
    fn the_cross_is_painted_on_the_top_border_without_esc_and_the_rect_is_its_cells() {
        let (mut buf, area) = framed(40);
        let rect = paint(&mut buf, area, false).expect("drawn");
        let top = row(&buf, 0);
        assert!(top.contains('\u{2715}'), "{top:?}");
        assert!(!top.contains("Esc"), "{top:?}");
        assert_eq!(rect, Rect::new(area.right() - 5, 0, 3, 1));
        let covered: String = (rect.x..rect.right())
            .map(|x| buf[(x, 0)].symbol())
            .collect();
        assert_eq!(covered, " \u{2715} ");
    }

    #[test]
    fn a_block_too_narrow_registers_nothing() {
        let (mut buf, area) = framed(7);
        assert_eq!(paint(&mut buf, area, false), None);
        assert!(!row(&buf, 0).contains('\u{2715}'));
    }

    #[test]
    fn hover_changes_the_style_and_flash_draws_the_same_cells() {
        let (mut buf, area) = framed(40);
        let rect = paint(&mut buf, area, false).unwrap();
        let idle = buf[(rect.x + 1, 0)].style();
        paint(&mut buf, area, true);
        assert_ne!(buf[(rect.x + 1, 0)].style(), idle);
        assert_eq!(flash(&mut buf, area), Some(rect));
    }
}
