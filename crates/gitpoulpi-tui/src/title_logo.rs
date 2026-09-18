//! The title bar's octopus as a picture: the About button's 🐙 becomes the
//! 16x16 sprite (`logo_colors`) in the chosen colorway, two cells by one row,
//! when the terminal draws images. Everywhere else the caller keeps the emoji.
//!
//! Same machinery as the commit-detail avatar (`DetailAvatarImage`): reserve
//! the cells while drawing, see what a popup drew over them in `finish_frame`
//! (a covered picture is hidden, an uncovered one restored), send after
//! `Terminal::draw`. The sprite is cached by colorway and cell pixel size,
//! and the encoded picture by its content, so a steady frame writes nothing.

use std::cell::RefCell;

use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;

use crate::avatar::DetailAvatarImage;
use crate::logo_colors;
use crate::terminal_image::{Bitmap, Capability, CellSize};

struct State {
    image: DetailAvatarImage,
    cell: CellSize,
    /// The sprite for `(way, px)`, built once.
    sprite: Option<((usize, u32), Bitmap)>,
    /// The reserved cells as `render` left them: anything different in
    /// `finish_frame` means a popup or page was drawn over the picture.
    expected: Option<(Rect, Vec<Cell>)>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Tells the pixel path what the terminal can draw. Never called (or a
/// terminal with no image protocol): [`render`] answers `false`.
pub(crate) fn init(capability: Capability) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            image: DetailAvatarImage::new(capability),
            cell: capability.cell,
            sprite: None,
            expected: None,
        });
    });
}

/// Sprite pixels per side for this cell shape: a multiple of 16 close to
/// the 2-column box's shorter side.
fn sprite_px(cell: CellSize) -> u32 {
    let side = (2 * cell.width).min(cell.height);
    ((side + 8) / 16).max(1) * 16
}

/// Reserves `area` (2x1 cells) for the octopus of colorway `way`. `false`
/// when this terminal cannot draw it: nothing was touched, paint the emoji.
pub(crate) fn render(buf: &mut Buffer, area: Rect, way: usize) -> bool {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let Some(state) = s.as_mut() else {
            return false;
        };
        if !state.image.can_draw() {
            return false;
        }
        let px = sprite_px(state.cell);
        if state.sprite.as_ref().map(|(k, _)| *k) != Some((way, px)) {
            let ways = logo_colors::ways();
            let sprite = logo_colors::sprite_bitmap(&ways[way.min(ways.len() - 1)], px);
            state.sprite = Some(((way, px), sprite));
        }
        let Some((_, sprite)) = &state.sprite else {
            return false;
        };
        let drew = state.image.render(buf, area, sprite);
        state.expected = drew.then(|| (area, cells_of(buf, area)));
        drew
    })
}

fn cells_of(buf: &Buffer, area: Rect) -> Vec<Cell> {
    let mut cells = Vec::new();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            cells.push(buf[(x, y)].clone());
        }
    }
    cells
}

/// Call once per frame after every popup is drawn. A popup, menu or page
/// that touches any cell of the picture hides all of it (never half an
/// octopus beside a menu); it comes back when the cells are free again.
pub(crate) fn finish_frame(buf: &Buffer) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            if let Some((area, expected)) = state.expected.take() {
                if cells_of(buf, area) != expected {
                    state.image.cancel();
                }
            }
            state.image.finish_frame(buf);
        }
    });
}

/// Call after `Terminal::draw` returns.
pub(crate) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    STATE.with(|s| match s.borrow_mut().as_mut() {
        Some(state) => state.image.write_pending(out),
        None => Ok(()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_image::vt_sim::Vt;
    use crate::terminal_image::Protocol;
    use crate::wide_edge::Clear;
    use ratatui::widgets::{Paragraph, Widget};

    const SCREEN: Rect = Rect::new(0, 0, 30, 6);
    const LOGO: Rect = Rect::new(1, 0, 2, 1);

    fn capability(protocol: Protocol) -> Capability {
        Capability {
            protocol,
            cell: CellSize {
                width: 9,
                height: 21,
            },
        }
    }

    /// One whole frame as the event loop runs it: reserve, optionally a
    /// popup over `popup`, `finish_frame`, `write_pending`, then the model.
    fn frame(vt: &mut Vt, prev: &mut Buffer, way: usize, popup: Option<Rect>) -> (bool, String) {
        let mut buf = Buffer::empty(SCREEN);
        let drew = render(&mut buf, LOGO, way);
        if let Some(p) = popup {
            Clear.render(p, &mut buf);
            Paragraph::new("x").render(p, &mut buf);
        }
        finish_frame(&buf);
        let mut out = Vec::new();
        write_pending(&mut out).unwrap();
        vt.draw(prev, &buf);
        vt.feed(&out);
        *prev = buf;
        (drew, String::from_utf8(out).unwrap())
    }

    #[test]
    fn the_picture_is_chosen_by_protocol() {
        for (protocol, expect) in [
            (Protocol::None, false),
            (Protocol::Iterm2, true),
            (Protocol::Iterm2Alpha, true),
        ] {
            init(capability(protocol));
            let mut buf = Buffer::empty(SCREEN);
            assert_eq!(render(&mut buf, LOGO, 0), expect, "{protocol:?}");
        }
        // Never initialized: no picture either.
        STATE.with(|s| *s.borrow_mut() = None);
        assert!(!render(&mut Buffer::empty(SCREEN), LOGO, 0));
    }

    #[test]
    fn the_sprite_is_a_multiple_of_sixteen_near_the_box() {
        let px = |w, h| {
            sprite_px(CellSize {
                width: w,
                height: h,
            })
        };
        assert_eq!(px(9, 21), 16);
        assert_eq!(px(8, 16), 16);
        assert_eq!(px(12, 24), 32);
        assert_eq!(px(1, 1), 16);
    }

    #[test]
    fn sent_once_hidden_under_a_popup_and_restored() {
        init(capability(Protocol::Iterm2Alpha));
        let mut vt = Vt::new(SCREEN);
        let mut prev = Buffer::empty(SCREEN);
        let (drew, first) = frame(&mut vt, &mut prev, 0, None);
        assert!(drew && first.contains("]1337;File="), "{first:?}");
        assert_eq!(vt.tiles_in(LOGO).len(), 2, "the picture is on both cells");
        // Steady frames cost nothing.
        let (_, second) = frame(&mut vt, &mut prev, 0, None);
        assert_eq!(second, "");
        // A popup over one cell: no picture tile survives under it.
        let (_, under) = frame(&mut vt, &mut prev, 0, Some(Rect::new(2, 0, 4, 3)));
        assert!(!under.contains("]1337;File="), "no picture under a popup");
        assert!(vt.tiles_in(LOGO).is_empty(), "all of it is hidden");
        // Popup gone: the picture is sent again.
        let (_, back) = frame(&mut vt, &mut prev, 0, None);
        assert!(back.contains("]1337;File="), "{back:?}");
        assert_eq!(vt.tiles_in(LOGO).len(), 2);
        // A new color is re-sent; the same color is not.
        assert!(logo_colors::ways().len() > 1);
        let (_, recolored) = frame(&mut vt, &mut prev, 1, None);
        assert!(recolored.contains("]1337;File="));
        let (_, steady) = frame(&mut vt, &mut prev, 1, None);
        assert_eq!(steady, "");
    }
}
