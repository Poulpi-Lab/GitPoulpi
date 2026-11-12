//! The logo sprite and its colorways, from `assets/logo/colourways.json` --
//! the one table the About window, `scripts/gen-logo.py` (README images) and
//! `build-custom` read. `build.rs` copies it to `OUT_DIR` keeping only the
//! colorways a custom build selected (`GITPOULPI_LOGO_COLORS`) and setting
//! its default (`GITPOULPI_LOGO_DEFAULT`); a stored `ui.logo_color` that was
//! not compiled in falls back to that default.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::terminal_image::Bitmap;

const TABLE: &str = include_str!(concat!(env!("OUT_DIR"), "/colourways.json"));

#[derive(Debug, Deserialize)]
struct Table {
    default: String,
    grid: Vec<String>,
    ways: Vec<Way>,
}

/// One colorway: the octopus' palette.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Way {
    pub slug: String,
    body: String,
    light: String,
    ink: String,
    cheek: String,
    tips: [String; 4],
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| serde_json::from_str(TABLE).expect("assets/logo/colourways.json parses"))
}

/// Every colorway built in, in the table's order.
pub(crate) fn ways() -> &'static [Way] {
    &table().ways
}

/// The build's default colorway slug.
pub(crate) fn default_slug() -> &'static str {
    &table().default
}

/// The index of `slug`, else of the default.
pub(crate) fn index_of(slug: &str) -> usize {
    index_in(table(), slug)
}

fn index_in(t: &Table, slug: &str) -> usize {
    let find = |s: &str| t.ways.iter().position(|w| w.slug == s);
    find(slug).or_else(|| find(&t.default)).unwrap_or(0)
}

/// The translated name of a colorway.
pub(crate) fn name(slug: &str) -> &'static str {
    use gitpoulpi_i18n::msg::logo_colors as m;
    match slug {
        "violet" => m::VIOLET.get(),
        "orange" => m::ORANGE.get(),
        "teal" => m::TEAL.get(),
        "ocean" => m::OCEAN.get(),
        "magenta" => m::MAGENTA.get(),
        "candy" => m::CANDY.get(),
        "coral" => m::CORAL.get(),
        "green" => m::GREEN.get(),
        "cyan" => m::CYAN.get(),
        "indigo" => m::INDIGO.get(),
        "lavender" => m::LAVENDER.get(),
        "sun" => m::SUN.get(),
        "mint" => m::MINT.get(),
        "slate" => m::SLATE.get(),
        "snow" => m::SNOW.get(),
        "ivory" => m::IVORY.get(),
        "pearl" => m::PEARL.get(),
        "porcelain" => m::PORCELAIN.get(),
        "charcoal" => m::CHARCOAL.get(),
        "midnight" => m::MIDNIGHT.get(),
        _ => m::LAVENDER.get(),
    }
}

fn hex(c: &str) -> (u8, u8, u8) {
    let v = u32::from_str_radix(c.trim_start_matches('#'), 16).unwrap_or(0);
    ((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// Side of the bitmap [`bitmap`] draws, in pixels (the sprite is 16 wide).
const SIZE: u32 = 128;

/// First sprite row of the tentacle tips, which wiggle during the spin.
const LEGS_FROM: usize = 12;

/// How far (-1, 0 or +1 sprite columns) the tentacle under column `col` of
/// sprite row `row` sways at `wiggle` (in turns; 0 = at rest). Each of the
/// four tentacles and each row is out of phase, so they wave.
fn leg_shift(row: usize, col: usize, wiggle: f32) -> isize {
    if row < LEGS_FROM || wiggle == 0.0 {
        return 0;
    }
    let leg = (col / 4) as f32;
    let phase = std::f32::consts::TAU * wiggle
        + leg * std::f32::consts::FRAC_PI_2
        + (row - LEGS_FROM) as f32 * 0.9;
    phase.sin().round() as isize
}

/// Where the octopus looks: a step (-1, 0 or +1) on each axis, (0, 0) straight
/// ahead. The eyes are 2x2 sprite pixels (white, with one dark pupil pixel), so
/// the pupil has four places; `look` picks the one on that side of the eye and
/// (0, 0) is the sprite's own (bottom-right) one.
pub(crate) type Look = (i8, i8);

/// The middle of the pair of eyes, as fractions of the sprite's width and
/// height: the eyes are the 2x2 boxes at columns 4-5 and 10-11, rows 5-6 of
/// the 16 x 16 grid.
pub(crate) const EYES: (f32, f32) = (8.0 / 16.0, 6.0 / 16.0);

/// The sprite grid with its pupils moved to `look`.
fn grid_looking(look: Look) -> Vec<Vec<char>> {
    let mut rows: Vec<Vec<char>> = table().grid.iter().map(|r| r.chars().collect()).collect();
    if look == (0, 0) {
        return rows;
    }
    let at = |rows: &[Vec<char>], r: usize, c: usize| rows.get(r).and_then(|l| l.get(c)).copied();
    let pupils: Vec<(usize, usize)> = (1..rows.len())
        .flat_map(|r| (1..rows[r].len()).map(move |c| (r, c)))
        .filter(|&(r, c)| {
            at(&rows, r, c) == Some('K')
                && at(&rows, r - 1, c - 1) == Some('W')
                && at(&rows, r - 1, c) == Some('W')
                && at(&rows, r, c - 1) == Some('W')
        })
        .collect();
    for (r, c) in pupils {
        // The eye is the 2x2 box whose bottom-right pixel is the pupil.
        for (dr, dc) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            rows[r - 1 + dr][c - 1 + dc] = 'W';
        }
        let dc = usize::from(look.0 >= 0);
        let dr = usize::from(look.1 >= 0);
        rows[r - 1 + dr][c - 1 + dc] = 'K';
    }
    rows
}

/// The sprite in `way`, squashed horizontally to `scale` (1.0 = full width)
/// about its center and optionally mirrored, its tentacle tips swayed by
/// `wiggle` (see [`leg_shift`]): one frame of the spin. Transparent where the
/// sprite is empty. The pupils look towards `look`.
pub(crate) fn bitmap(way: &Way, scale: f32, mirrored: bool, wiggle: f32, look: Look) -> Bitmap {
    let pal = |ch: char| -> Option<(u8, u8, u8)> {
        Some(match ch {
            'O' | 'K' => hex(&way.ink),
            'P' => hex(&way.body),
            'L' => hex(&way.light),
            'W' => (255, 255, 255),
            'C' => hex(&way.cheek),
            'T' => hex(&way.tips[0]),
            'R' => hex(&way.tips[1]),
            'S' => hex(&way.tips[2]),
            'B' => hex(&way.tips[3]),
            _ => return None,
        })
    };
    let rows = grid_looking(look);
    let mut out = Bitmap::transparent(SIZE, SIZE);
    let half = SIZE as f32 / 2.0;
    let scale = scale.max(0.0);
    for y in 0..SIZE {
        let row = &rows[(y as usize * 16 / SIZE as usize).min(rows.len() - 1)];
        for x in 0..SIZE {
            if scale <= 0.0 {
                break;
            }
            let u = (x as f32 + 0.5 - half) / (half * scale);
            if u.abs() > 1.0 {
                continue;
            }
            let mut col = (((u + 1.0) / 2.0) * 16.0) as usize;
            col = col.min(15);
            if mirrored {
                col = 15 - col;
            }
            let src = col as isize - leg_shift(y as usize * 16 / SIZE as usize, col, wiggle);
            if !(0..16).contains(&src) {
                continue;
            }
            if let Some(c) = row.get(src as usize).copied().and_then(pal) {
                out.set_pixel(x, y, c);
            }
        }
    }
    out
}

/// The 16x16 sprite in `way` at rest, each sprite pixel drawn `px / 16`
/// device pixels wide (`px` a multiple of 16, at least 16). Transparent where
/// the sprite is empty. Small on purpose: the title bar's icon and launcher
/// files want it at its own size, not the About window's 128.
pub(crate) fn sprite_bitmap(way: &Way, px: u32) -> Bitmap {
    let px = (px.max(16) / 16) * 16;
    let big = bitmap(way, 1.0, false, 0.0, (0, 0));
    let mut out = Bitmap::transparent(px, px);
    for y in 0..px {
        for x in 0..px {
            if let Some((r, g, b, a)) = big.rgba(x * SIZE / px, y * SIZE / px) {
                if a > 0 {
                    out.set_pixel(x, y, (r, g, b));
                }
            }
        }
    }
    out
}

/// The build-time selection logic, also exercised here (it is shared with
/// `build.rs` and `xtask` by `#[path]`).
#[cfg(test)]
#[path = "../../../assets/logo/logo_build.rs"]
mod logo_build;

#[cfg(test)]
mod tests {
    use super::*;

    /// Under a custom build's environment the About default is the chosen
    /// one, and the table holds exactly the chosen colorways.
    #[test]
    fn the_build_environment_decides_the_compiled_table() {
        if let Some(d) = option_env!("GITPOULPI_LOGO_DEFAULT").filter(|d| !d.is_empty()) {
            assert_eq!(default_slug(), d);
        } else {
            assert_eq!(default_slug(), "lavender");
        }
        if let Some(list) = option_env!("GITPOULPI_LOGO_COLORS").filter(|l| *l != "all") {
            let have: Vec<&str> = ways().iter().map(|w| w.slug.as_str()).collect();
            let mut want: Vec<&str> = list.split(',').collect();
            want.sort_by_key(|w| have.iter().position(|h| h == w));
            assert_eq!(have, want);
        }
    }

    #[test]
    fn a_stale_slug_falls_back_to_the_compiled_default() {
        // Only violet and teal compiled, teal the default: a stored "sun"
        // (not compiled) or "lavender" resolves to teal.
        let full: serde_json::Value =
            serde_json::from_str(include_str!("../../../assets/logo/colourways.json")).unwrap();
        let all = logo_build::slugs(&full);
        let sel = logo_build::parse_colors("violet,teal", &all).unwrap();
        let t: Table =
            serde_json::from_value(logo_build::filter_table(&full, &sel, "teal")).unwrap();
        assert_eq!(t.ways.len(), 2);
        assert_eq!(t.ways[index_in(&t, "sun")].slug, "teal");
        assert_eq!(t.ways[index_in(&t, "")].slug, "teal");
        assert_eq!(t.ways[index_in(&t, "violet")].slug, "violet");
    }

    #[test]
    fn the_retired_obsidian_slug_resolves_to_the_default() {
        assert!(ways().iter().all(|w| w.slug != "obsidian"));
        assert_eq!(index_of("obsidian"), index_of(default_slug()));
    }

    #[test]
    fn the_table_has_twenty_ways_and_a_default_that_exists() {
        if option_env!("GITPOULPI_LOGO_COLORS").is_none() {
            assert_eq!(ways().len(), 20);
        }
        assert_eq!(ways()[index_of(default_slug())].slug, default_slug());
        assert_eq!(index_of("no-such"), index_of(default_slug()));
        for w in ways() {
            assert!(!name(&w.slug).is_empty(), "{}", w.slug);
        }
    }

    #[test]
    fn a_squashed_frame_is_narrower_and_a_mirrored_one_swaps_the_tips() {
        let w = &ways()[index_of("lavender")];
        let full = bitmap(w, 1.0, false, 0.0, (0, 0));
        let thin = bitmap(w, 0.25, false, 0.0, (0, 0));
        let count = |b: &Bitmap| {
            (0..b.width)
                .filter(|x| b.rgba(*x, 64).is_some_and(|p| p.3 > 0))
                .count()
        };
        assert!(count(&thin) < count(&full) / 2);
        assert_eq!(
            bitmap(w, 0.0, false, 0.0, (0, 0)).rgba(64, 64).unwrap().3,
            0
        );
        // The bottom row has four tips, left teal and right blue; mirrored it is the reverse.
        let y = 116;
        assert_eq!(
            full.rgba(8 + 4, y).map(|p| (p.0, p.1, p.2)),
            Some((0x14, 0xB8, 0xA6))
        );
        let flipped = bitmap(w, 1.0, true, 0.0, (0, 0));
        assert_eq!(
            flipped.rgba(128 - 8 - 4, y).map(|p| (p.0, p.1, p.2)),
            Some((0x14, 0xB8, 0xA6))
        );
    }

    /// The dark pixel of each eye, as sprite (column, row) pairs.
    fn pupils(look: Look) -> Vec<(usize, usize)> {
        let g = grid_looking(look);
        let mut out = Vec::new();
        for (r, row) in g.iter().enumerate() {
            for (c, ch) in row.iter().enumerate() {
                if *ch == 'K' {
                    out.push((c, r));
                }
            }
        }
        out
    }

    #[test]
    fn the_pupils_move_inside_their_eyes() {
        assert_eq!(pupils((0, 0)), vec![(5, 6), (11, 6)]);
        assert_eq!(pupils((-1, -1)), vec![(4, 5), (10, 5)]);
        assert_eq!(pupils((1, -1)), vec![(5, 5), (11, 5)]);
        assert_eq!(pupils((-1, 1)), vec![(4, 6), (10, 6)]);
        // The eye stays white around the pupil.
        let g = grid_looking((-1, -1));
        assert_eq!(g[6][5], 'W');
        // And the picture follows: the pixel the pupil left is white now.
        let w = &ways()[0];
        let rest = bitmap(w, 1.0, false, 0.0, (0, 0));
        let up = bitmap(w, 1.0, false, 0.0, (-1, -1));
        assert_ne!(
            rest.rgba(5 * 8 + 4, 6 * 8 + 4),
            up.rgba(5 * 8 + 4, 6 * 8 + 4)
        );
    }

    #[test]
    fn the_tentacles_sway_while_the_head_stays_put() {
        let w = &ways()[0];
        let rest = bitmap(w, 1.0, false, 0.0, (0, 0));
        let mut legs_moved = false;
        for wiggle in [0.1, 0.25, 0.4, 0.6, 0.75, 0.9] {
            let moved = bitmap(w, 1.0, false, wiggle, (0, 0));
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let differs = rest.rgba(x, y) != moved.rgba(x, y);
                    if (y as usize * 16 / SIZE as usize) < LEGS_FROM {
                        assert!(!differs, "the head moved at {x},{y} (wiggle {wiggle})");
                    } else if differs {
                        legs_moved = true;
                    }
                }
            }
        }
        assert!(legs_moved, "the tentacles never moved");
    }
}
