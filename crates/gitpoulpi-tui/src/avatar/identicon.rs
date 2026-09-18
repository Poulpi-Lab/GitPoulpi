//! A GitHub-style identicon: the picture an author gets when no source
//! knows them. Generated locally from the email -- no network, no
//! account -- and deterministic: the same address always draws the same
//! picture, across runs and machines.
//!
//! - The hash is the SHA-256 of the trimmed, lowercased email (the same
//!   normalisation as the Gravatar hash, so "Ada@X.com " and "ada@x.com"
//!   are one author).
//! - A 5x5 grid, mirrored horizontally: three columns are computed from
//!   the hash bits (15 bits), the fourth and fifth mirror the second and
//!   first.
//! - The foreground is an HSL color taken from the hash: a hue anywhere
//!   on the wheel, saturation 55-70 % and lightness 42-52 %, so it is
//!   vivid but never neon or muddy, on a light background with a margin.
//!
//! Two renderings: [`bitmap`] (a pixel picture, fed through the same path
//! as a fetched avatar) and [`blocks`] (quadrant block characters, for a
//! terminal that cannot draw images).

use ratatui::style::Color;
use sha2::{Digest, Sha256};

/// Cells per side.
pub(crate) const GRID: usize = 5;

/// The light background every identicon sits on.
pub(crate) const BACKGROUND: (u8, u8, u8) = (0xF0, 0xF0, 0xF0);

fn digest(email: &str) -> [u8; 32] {
    let normalized = email.trim().to_lowercase();
    Sha256::digest(normalized.as_bytes()).into()
}

/// The filled cells, `grid[row][col]`, mirrored: `col 4 == col 0`,
/// `col 3 == col 1`. Never empty: a hash that fills nothing gets its
/// center cell, so the picture is never a blank square.
pub(crate) fn grid(email: &str) -> [[bool; GRID]; GRID] {
    grid_of(&digest(email))
}

fn grid_of(hash: &[u8; 32]) -> [[bool; GRID]; GRID] {
    let mut cells = [[false; GRID]; GRID];
    let mut any = false;
    for (row, line) in cells.iter_mut().enumerate() {
        for col in 0..3 {
            // 15 bits out of the first two bytes.
            let bit = row * 3 + col;
            let on = (hash[bit / 8] >> (bit % 8)) & 1 == 1;
            line[col] = on;
            line[GRID - 1 - col] = on;
            any |= on;
        }
    }
    if !any {
        cells[GRID / 2][GRID / 2] = true;
    }
    cells
}

/// The foreground as `(hue in degrees, saturation, lightness)`.
pub(crate) fn hsl(email: &str) -> (f32, f32, f32) {
    hsl_of(&digest(email))
}

fn hsl_of(hash: &[u8; 32]) -> (f32, f32, f32) {
    let hue = f32::from(u16::from_le_bytes([hash[2], hash[3]]) % 360);
    let saturation = 0.55 + f32::from(hash[4]) / 255.0 * 0.15;
    let lightness = 0.42 + f32::from(hash[5]) / 255.0 * 0.10;
    (hue, saturation, lightness)
}

/// The foreground color, as RGB.
pub(crate) fn rgb(email: &str) -> (u8, u8, u8) {
    let (h, s, l) = hsl(email);
    hsl_to_rgb(h, s, l)
}

/// The foreground color, as a terminal color (the block rendering).
pub(crate) fn color(email: &str) -> Color {
    let (r, g, b) = rgb(email);
    Color::Rgb(r, g, b)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to_u8 = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(r), to_u8(g), to_u8(b))
}

/// The identicon as a `side`x`side` picture: the grid in the middle, a
/// margin of `side / 10` pixels all round.
pub(crate) fn bitmap(email: &str, side: u32) -> crate::terminal_image::Bitmap {
    let side = side.max(GRID as u32);
    let cells = grid(email);
    let fg = rgb(email);
    let mut out = crate::terminal_image::Bitmap::new(side, side, BACKGROUND);
    let margin = side / 10;
    let inner = side - 2 * margin;
    for y in margin..side - margin {
        let row = ((y - margin) * GRID as u32 / inner) as usize;
        for x in margin..side - margin {
            let col = ((x - margin) * GRID as u32 / inner) as usize;
            if cells[row.min(GRID - 1)][col.min(GRID - 1)] {
                out.set_pixel(x, y, fg);
            }
        }
    }
    out
}

/// The quadrant character for `(top-left, top-right, bottom-left,
/// bottom-right)`.
fn quadrant(tl: bool, tr: bool, bl: bool, br: bool) -> char {
    const TABLE: [char; 16] = [
        ' ', '\u{2598}', '\u{259D}', '\u{2580}', '\u{2596}', '\u{258C}', '\u{259E}', '\u{259B}',
        '\u{2597}', '\u{259A}', '\u{2590}', '\u{259C}', '\u{2584}', '\u{2599}', '\u{259F}',
        '\u{2588}',
    ];
    TABLE[usize::from(tl) | usize::from(tr) << 1 | usize::from(bl) << 2 | usize::from(br) << 3]
}

/// The identicon in `cols` terminal cells on one row, each a quadrant
/// character (2x2 sub-cells): the 5x5 grid resampled to `2*cols` by 2,
/// a sub-cell being lit when at least half of it is. `cols` 1 is the
/// Log's one-cell glyph, 2 its picture slot.
pub(crate) fn blocks(email: &str, cols: usize) -> String {
    let cells = grid(email);
    let sub_cols = cols * 2;
    // Whether the sub-cell (`sx` of `sub_cols`, `sy` of 2) is lit: the
    // share of grid cells it overlaps that are filled, by area.
    let lit = |sx: usize, sy: usize| -> bool {
        let (x0, x1) = (sx * GRID, (sx + 1) * GRID); // in 1/sub_cols units
        let (y0, y1) = (sy * GRID, (sy + 1) * GRID); // in 1/2 units
        let (mut filled, mut total) = (0usize, 0usize);
        for (row, cells_row) in cells.iter().enumerate() {
            let oy = overlap(y0, y1, row * 2, (row + 1) * 2);
            if oy == 0 {
                continue;
            }
            for (col, on) in cells_row.iter().enumerate() {
                let ox = overlap(x0, x1, col * sub_cols, (col + 1) * sub_cols);
                let area = ox * oy;
                total += area;
                if *on {
                    filled += area;
                }
            }
        }
        total > 0 && filled * 2 >= total
    };
    (0..cols)
        .map(|c| {
            quadrant(
                lit(c * 2, 0),
                lit(c * 2 + 1, 0),
                lit(c * 2, 1),
                lit(c * 2 + 1, 1),
            )
        })
        .collect()
}

fn overlap(a0: usize, a1: usize, b0: usize, b1: usize) -> usize {
    a1.min(b1).saturating_sub(a0.max(b0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emails() -> Vec<String> {
        (0..500).map(|i| format!("user{i}@example.com")).collect()
    }

    /// Falsified: seeded the hash with the wall clock, watched this fail.
    #[test]
    fn the_same_email_always_draws_the_same_picture() {
        assert_eq!(grid("a@x.com"), grid("a@x.com"));
        assert_eq!(rgb("a@x.com"), rgb("a@x.com"));
        assert_eq!(bitmap("a@x.com", 60).data, bitmap("a@x.com", 60).data);
        assert_eq!(grid("Ada@X.com "), grid("ada@x.com"));
        assert_eq!(rgb("Ada@X.com "), rgb("ada@x.com"));
    }

    #[test]
    fn different_emails_mostly_draw_different_pictures() {
        let distinct: std::collections::HashSet<_> =
            emails().iter().map(|e| (grid(e), rgb(e))).collect();
        assert!(distinct.len() > 450, "{}", distinct.len());
    }

    /// Falsified: mirrored column 4 from column 1 instead of column 0.
    #[test]
    fn the_grid_is_mirrored_left_to_right_and_never_blank() {
        for email in emails() {
            let cells = grid(&email);
            let mut any = false;
            for row in cells {
                for col in 0..GRID {
                    assert_eq!(row[col], row[GRID - 1 - col], "{email}");
                    any |= row[col];
                }
            }
            assert!(any, "{email}");
        }
        // The blank-hash guard.
        let blank = grid_of(&[0; 32]);
        assert!(blank[2][2]);
    }

    #[test]
    fn the_bitmap_is_mirrored_too() {
        let b = bitmap("someone@example.com", 100);
        for y in 0..100u32 {
            for x in 0..50u32 {
                let left = (y * 100 + x) as usize * 4;
                let right = (y * 100 + (99 - x)) as usize * 4;
                assert_eq!(b.data[left..left + 4], b.data[right..right + 4], "{x},{y}");
            }
        }
        // The margin is background, the picture has some foreground.
        assert_eq!(
            b.data[0..4],
            [BACKGROUND.0, BACKGROUND.1, BACKGROUND.2, 255]
        );
        let fg = rgb("someone@example.com");
        assert!(b.data.chunks(4).any(|p| p == [fg.0, fg.1, fg.2, 255]));
    }

    /// Saturated but not garish: every hue, 55-70 % saturation, 42-52 %
    /// lightness; and the RGB it converts to is not near the background.
    #[test]
    fn the_color_is_saturated_but_not_garish() {
        let mut hues = std::collections::HashSet::new();
        for email in emails() {
            let (h, s, l) = hsl(&email);
            assert!((0.0..360.0).contains(&h));
            assert!((0.55..=0.7001).contains(&s), "{s}");
            assert!((0.42..=0.5201).contains(&l), "{l}");
            hues.insert((h / 30.0) as u32);
            let (r, g, b) = rgb(&email);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            assert!(max < 235 && min > 15, "{email}: {r},{g},{b}");
            assert!(max - min > 60, "{email}: {r},{g},{b}");
        }
        assert_eq!(hues.len(), 12, "every 30-degree hue band is reachable");
    }

    #[test]
    fn hsl_conversion_hits_known_colors() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0));
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255));
    }

    #[test]
    fn the_block_rendering_has_the_asked_width_and_is_deterministic() {
        for cols in [1, 2, 4] {
            for email in emails().iter().take(50) {
                let text = blocks(email, cols);
                assert_eq!(text.chars().count(), cols, "{email}");
                assert_eq!(text, blocks(email, cols));
            }
        }
        assert!(emails().iter().take(50).any(|e| blocks(e, 1) != " "));
    }

    #[test]
    fn the_quadrant_table_matches_the_unicode_names() {
        assert_eq!(quadrant(true, true, true, true), '\u{2588}');
        assert_eq!(quadrant(false, false, false, false), ' ');
        assert_eq!(quadrant(true, false, false, false), '\u{2598}');
        assert_eq!(quadrant(false, false, true, true), '\u{2584}');
        assert_eq!(quadrant(true, false, false, true), '\u{259A}');
    }
}
