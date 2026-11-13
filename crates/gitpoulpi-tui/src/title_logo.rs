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
use ratatui::style::{Color, Style};

use crate::avatar::DetailAvatarImage;
use crate::logo_colors;
use crate::terminal_image::{Bitmap, Capability, CellSize};

type SpriteKey = (usize, u32, Option<(u8, u8, u8)>);

struct State {
    image: DetailAvatarImage,
    cell: CellSize,
    /// The sprite for `(way, px, backdrop)`, built once.
    sprite: Option<(SpriteKey, Bitmap)>,
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
///
/// `hover` is the hover background of the button, `None` at rest. Image
/// protocols do not reliably show the cell background through transparent
/// pixels (sixel and iTerm paint them opaque or default), so when hovered the
/// color is baked under the sprite and the reserved cells get it too. A
/// non-RGB color (indexed, reset) cannot be baked: the sprite stays
/// transparent and only the cell background carries it. The backdrop is part
/// of the sprite key, so a steady frame writes nothing and hover in/out
/// re-sends once.
pub(crate) fn render(buf: &mut Buffer, area: Rect, way: usize, hover: Option<Color>) -> bool {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let Some(state) = s.as_mut() else {
            return false;
        };
        if !state.image.can_draw() {
            return false;
        }
        let px = sprite_px(state.cell);
        let backdrop = hover.and_then(crate::theme::rgb_of);
        let key = (way, px, backdrop);
        if state.sprite.as_ref().map(|(k, _)| *k) != Some(key) {
            let ways = logo_colors::ways();
            let mut sprite = logo_colors::sprite_bitmap(&ways[way.min(ways.len() - 1)], px);
            if let Some(bg) = backdrop {
                under_backdrop(&mut sprite, bg);
            }
            state.sprite = Some((key, sprite));
        }
        let Some((_, sprite)) = &state.sprite else {
            return false;
        };
        let drew = state.image.render(buf, area, sprite);
        if drew {
            if let Some(bg) = hover {
                buf.set_style(area, Style::default().bg(bg));
            }
        }
        state.expected = drew.then(|| (area, cells_of(buf, area)));
        drew
    })
}

/// Fills every fully transparent pixel with `bg`.
fn under_backdrop(sprite: &mut Bitmap, bg: (u8, u8, u8)) {
    for y in 0..sprite.height {
        for x in 0..sprite.width {
            if matches!(sprite.rgba(x, y), Some((_, _, _, 0))) {
                sprite.set_pixel(x, y, bg);
            }
        }
    }
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
        frame_hover(vt, prev, way, popup, None)
    }

    fn frame_hover(
        vt: &mut Vt,
        prev: &mut Buffer,
        way: usize,
        popup: Option<Rect>,
        hover: Option<Color>,
    ) -> (bool, String) {
        let mut buf = Buffer::empty(SCREEN);
        let drew = render(&mut buf, LOGO, way, hover);
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
            assert_eq!(render(&mut buf, LOGO, 0, None), expect, "{protocol:?}");
        }
        // Never initialized: no picture either.
        STATE.with(|s| *s.borrow_mut() = None);
        assert!(!render(&mut Buffer::empty(SCREEN), LOGO, 0, None));
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

    #[test]
    fn hover_bakes_the_background_under_the_sprite_and_resends_once() {
        init(capability(Protocol::Iterm2Alpha));
        let hover = Color::Rgb(10, 20, 30);
        let mut vt = Vt::new(SCREEN);
        let mut prev = Buffer::empty(SCREEN);
        let (_, rest) = frame(&mut vt, &mut prev, 0, None);
        assert!(rest.contains("]1337;File="));
        let (_, steady) = frame(&mut vt, &mut prev, 0, None);
        assert_eq!(steady, "");
        // Hover in: re-sent once, steady hover writes nothing.
        let (_, entered) = frame_hover(&mut vt, &mut prev, 0, None, Some(hover));
        assert!(entered.contains("]1337;File="), "{entered:?}");
        let (_, held) = frame_hover(&mut vt, &mut prev, 0, None, Some(hover));
        assert_eq!(held, "");
        // The reserved cells carry the hover bg too.
        let mut buf = Buffer::empty(SCREEN);
        assert!(render(&mut buf, LOGO, 0, Some(hover)));
        for x in LOGO.left()..LOGO.right() {
            assert_eq!(buf[(x, 0)].bg, hover);
        }
        // Hover out: re-sent once more.
        let (_, left) = frame(&mut vt, &mut prev, 0, None);
        assert!(left.contains("]1337;File="));
        // Bitmaps: transparent at rest, backdrop when hovered.
        let ways = logo_colors::ways();
        let mut sprite = logo_colors::sprite_bitmap(&ways[0], 16);
        let transparent = |b: &Bitmap| {
            (0..b.height)
                .flat_map(|y| (0..b.width).map(move |x| (x, y)))
                .filter(|&(x, y)| matches!(b.rgba(x, y), Some((_, _, _, 0))))
                .count()
        };
        let empty = transparent(&sprite);
        assert!(empty > 0, "the sprite has empty pixels at rest");
        under_backdrop(&mut sprite, (10, 20, 30));
        assert_eq!(transparent(&sprite), 0);
        assert_eq!(sprite.rgba(0, 0), Some((10, 20, 30, 255)));
    }
}
