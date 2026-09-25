//! Half-block (`▀`) rendering: the text-terminal path of the image view
//! (`Protocol::None`). One cell shows two vertically stacked pixels, the
//! top as foreground and the bottom as background, so a pane of
//! `cols x rows` cells is a `cols x rows*2` pixel grid.
//!
//! Colors are written as truecolor and reduced to the 256-color cube or
//! the 16 ANSI colors when the terminal cannot do better, through the
//! same `color_depth` mapping the rest of the UI uses.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::color_depth::{nearest_16, nearest_256, Depth};
use crate::terminal_image::Bitmap;

pub(crate) const UPPER_HALF: &str = "\u{2580}";

/// `rgb` as a ratatui color at `depth`.
pub(crate) fn color(rgb: (u8, u8, u8), depth: Depth) -> Color {
    match depth {
        Depth::True => Color::Rgb(rgb.0, rgb.1, rgb.2), // color-data
        Depth::X256 => Color::Indexed(nearest_256(rgb.0, rgb.1, rgb.2)), // color-data
        Depth::X16 => nearest_16(rgb.0, rgb.1, rgb.2),
    }
}

/// The pixel size of a cell area: `cols x rows*2`.
pub(crate) fn pixel_size(area: Rect) -> (u32, u32) {
    (u32::from(area.width), u32::from(area.height) * 2)
}

/// Composites an alpha pixel over `under` (the checkerboard is applied by
/// the rasteriser; this covers a bitmap that still carries alpha).
fn flatten(p: (u8, u8, u8, u8), under: (u8, u8, u8)) -> (u8, u8, u8) {
    if p.3 == 255 {
        return (p.0, p.1, p.2);
    }
    let a = f32::from(p.3) / 255.0;
    let m = |c: u8, u: u8| (f32::from(c) * a + f32::from(u) * (1.0 - a)).round() as u8;
    (m(p.0, under.0), m(p.1, under.1), m(p.2, under.2))
}

/// Paints `bitmap` into `buf` at `area`, one cell per two pixel rows. The
/// bitmap is expected to be [`pixel_size`] of the area; any other size is
/// sampled nearest-neighbor onto it (never panics, never reads outside).
/// An odd bitmap height leaves the last cell's bottom half on `under`.
pub(crate) fn render(
    buf: &mut Buffer,
    area: Rect,
    bitmap: &Bitmap,
    depth: Depth,
    under: (u8, u8, u8),
) {
    let area = area.intersection(buf.area);
    if area.is_empty() || bitmap.width == 0 || bitmap.height == 0 {
        return;
    }
    let (pw, ph) = (bitmap.width, bitmap.height);
    let (gw, gh) = pixel_size(area);
    let at = |gx: u32, gy: u32| -> (u8, u8, u8) {
        let x =
            (u64::from(gx) * u64::from(pw) / u64::from(gw.max(1))).min(u64::from(pw) - 1) as u32;
        let y =
            (u64::from(gy) * u64::from(ph) / u64::from(gh.max(1))).min(u64::from(ph) - 1) as u32;
        flatten(bitmap.rgba(x, y).unwrap_or((0, 0, 0, 255)), under)
    };
    for cy in 0..area.height {
        for cx in 0..area.width {
            let top = at(u32::from(cx), u32::from(cy) * 2);
            let bottom = at(u32::from(cx), u32::from(cy) * 2 + 1);
            if let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) {
                cell.set_symbol(UPPER_HALF)
                    .set_fg(color(top, depth))
                    .set_bg(color(bottom, depth));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bmp(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Bitmap {
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                data.extend(f(x, y));
            }
        }
        Bitmap {
            width: w,
            height: h,
            data,
        }
    }

    #[test]
    fn a_cell_is_two_stacked_pixels_top_as_foreground() {
        let img = bmp(2, 4, |x, y| match (x, y) {
            (0, 0) => [255, 0, 0, 255],
            (0, 1) => [0, 255, 0, 255],
            (1, 0) => [0, 0, 255, 255],
            (1, 1) => [9, 9, 9, 255],
            (0, 2) => [1, 2, 3, 255],
            _ => [0, 0, 0, 255],
        });
        let area = Rect::new(1, 1, 2, 2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 4));
        render(&mut buf, area, &img, Depth::True, (0, 0, 0));
        let c = &buf[(1, 1)];
        assert_eq!(c.symbol(), UPPER_HALF);
        assert_eq!((c.fg, c.bg), (Color::Rgb(255, 0, 0), Color::Rgb(0, 255, 0)));
        let c = &buf[(2, 1)];
        assert_eq!((c.fg, c.bg), (Color::Rgb(0, 0, 255), Color::Rgb(9, 9, 9)));
        assert_eq!(buf[(1, 2)].fg, Color::Rgb(1, 2, 3));
        // Outside the area nothing is written.
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }

    #[test]
    fn colors_follow_the_terminal_depth() {
        let rgb = (200, 30, 40);
        assert_eq!(color(rgb, Depth::True), Color::Rgb(200, 30, 40));
        assert_eq!(
            color(rgb, Depth::X256),
            Color::Indexed(nearest_256(200, 30, 40))
        );
        assert_eq!(color(rgb, Depth::X16), nearest_16(200, 30, 40));
        assert!(matches!(color(rgb, Depth::X256), Color::Indexed(i) if i >= 16));
    }

    #[test]
    fn alpha_is_flattened_over_the_given_color() {
        let img = bmp(1, 2, |_, y| {
            if y == 0 {
                [255, 0, 0, 0]
            } else {
                [255, 0, 0, 255]
            }
        });
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        render(
            &mut buf,
            Rect::new(0, 0, 1, 1),
            &img,
            Depth::True,
            (10, 20, 30),
        );
        assert_eq!(buf[(0, 0)].fg, Color::Rgb(10, 20, 30));
        assert_eq!(buf[(0, 0)].bg, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn a_bitmap_of_another_size_is_sampled_onto_the_area_without_panicking() {
        let img = bmp(3, 3, |x, y| [x as u8 * 10, y as u8 * 10, 0, 255]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 5));
        render(
            &mut buf,
            Rect::new(0, 0, 5, 5),
            &img,
            Depth::True,
            (0, 0, 0),
        );
        // An area larger than the buffer is clipped, an empty bitmap is a no-op.
        render(
            &mut buf,
            Rect::new(3, 3, 9, 9),
            &img,
            Depth::True,
            (0, 0, 0),
        );
        render(
            &mut buf,
            Rect::new(0, 0, 2, 2),
            &Bitmap::transparent(0, 0),
            Depth::True,
            (0, 0, 0),
        );
        assert_eq!(pixel_size(Rect::new(0, 0, 5, 3)), (5, 6));
    }
}
