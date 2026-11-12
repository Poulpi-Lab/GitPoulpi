//! `ui.color_depth`: draws a truecolor theme on a terminal that cannot show
//! it. One post-pass over the finished frame buffer maps every `Color::Rgb`
//! (and, at 16 colors, every `Color::Indexed`) to the nearest entry of the
//! xterm-256 palette or of the 16 ANSI colors, so it catches every color
//! source (themes, syntect, avatar initials) without each caller knowing.
//!
//! Truecolor installs nothing: [`apply`] is then one atomic load per frame.
//! The pass is W x H cells and runs only when a reduced depth is active.
//! Kitty's `U+10EEEE` placeholder cells are skipped: their foreground
//! *encodes the image id*, so quantising it would show another picture.

use std::sync::atomic::{AtomicU8, Ordering};

use gitpoulpi_config::ColorDepth;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

/// The kitty graphics protocol's Unicode placeholder.
const PLACEHOLDER: char = '\u{10EEEE}';

/// The depth actually drawn with (`auto` resolved).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Depth {
    True,
    X256,
    X16,
}

static ACTIVE: AtomicU8 = AtomicU8::new(0);

/// Resolves `auto` from the environment (`env` is injected for tests).
/// Explicit `COLORTERM=truecolor|24bit` always wins over the TERM guesses.
pub(crate) fn resolve_with(setting: ColorDepth, env: impl Fn(&str) -> Option<String>) -> Depth {
    match setting {
        ColorDepth::Truecolor => return Depth::True,
        ColorDepth::Depth256 => return Depth::X256,
        ColorDepth::Depth16 => return Depth::X16,
        ColorDepth::Auto => {}
    }
    let get = |k: &str| env(k).filter(|v| !v.is_empty());
    let colorterm = get("COLORTERM");
    if matches!(colorterm.as_deref(), Some("truecolor" | "24bit")) {
        return Depth::True;
    }
    let term = get("TERM").unwrap_or_default();
    if get("TERM_PROGRAM").as_deref() == Some("Apple_Terminal") {
        return Depth::X256;
    }
    if term == "linux" {
        return Depth::X16;
    }
    let truecolor_terminal = get("TERM_PROGRAM").as_deref() == Some("WezTerm")
        || term.contains("kitty")
        || get("KITTY_WINDOW_ID").is_some()
        || get("WT_SESSION").is_some();
    if term.ends_with("-256color") && colorterm.is_none() && !truecolor_terminal {
        return Depth::X256;
    }
    Depth::True
}

/// Installs the depth for the setting, from the process environment.
pub(crate) fn install(setting: ColorDepth) {
    let depth = resolve_with(setting, |k| std::env::var(k).ok());
    ACTIVE.store(depth as u8, Ordering::Relaxed);
}

pub(crate) fn active() -> Depth {
    match ACTIVE.load(Ordering::Relaxed) {
        1 => Depth::X256,
        2 => Depth::X16,
        _ => Depth::True,
    }
}

/// Maps the finished frame to the active depth (nothing at truecolor).
pub(crate) fn apply(buf: &mut Buffer) {
    let depth = active();
    if depth != Depth::True {
        apply_depth(buf, depth);
    }
}

/// [`apply`] for one cell, for cells written outside the frame's diff.
pub(crate) fn apply_cell(cell: &mut ratatui::buffer::Cell) {
    let depth = active();
    if depth != Depth::True && !cell.symbol().contains(PLACEHOLDER) {
        cell.fg = reduce(cell.fg, depth);
        cell.bg = reduce(cell.bg, depth);
        cell.underline_color = reduce(cell.underline_color, depth);
    }
}

pub(crate) fn apply_depth(buf: &mut Buffer, depth: Depth) {
    for cell in &mut buf.content {
        if cell.symbol().contains(PLACEHOLDER) {
            continue;
        }
        cell.fg = reduce(cell.fg, depth);
        cell.bg = reduce(cell.bg, depth);
        cell.underline_color = reduce(cell.underline_color, depth);
    }
}

fn reduce(color: Color, depth: Depth) -> Color {
    match (color, depth) {
        (Color::Rgb(r, g, b), Depth::X256) => Color::Indexed(nearest_256(r, g, b)),
        (Color::Rgb(r, g, b), Depth::X16) => nearest_16(r, g, b),
        (Color::Indexed(i), Depth::X16) if i >= 16 => {
            let (r, g, b) = indexed_rgb(i);
            nearest_16(r, g, b)
        }
        _ => color,
    }
}

const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

fn dist(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let d = |x: u8, y: u8| u32::from(x.abs_diff(y)).pow(2);
    d(a.0, b.0) + d(a.1, b.1) + d(a.2, b.2)
}

fn cube_level(v: u8) -> usize {
    (0..6)
        .min_by_key(|&i| u32::from(CUBE[i].abs_diff(v)))
        .unwrap_or(0)
}

/// The nearest entry of the xterm-256 color cube (16..=231) or gray ramp
/// (232..=255). The first 16 are skipped: terminals theme them.
pub(crate) fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let (ri, gi, bi) = (cube_level(r), cube_level(g), cube_level(b));
    let cube = (CUBE[ri], CUBE[gi], CUBE[bi]);
    let cube_index = 16 + 36 * ri + 6 * gi + bi;
    let avg = ((u32::from(r) + u32::from(g) + u32::from(b)) / 3) as u8;
    // Gray ramp: 8, 18, ... 238.
    let step = ((i32::from(avg) - 8 + 5) / 10).clamp(0, 23) as u8;
    let gray_v = 8 + 10 * step;
    let want = (r, g, b);
    if dist(want, (gray_v, gray_v, gray_v)) < dist(want, cube) {
        232 + step
    } else {
        cube_index as u8
    }
}

/// RGB of an xterm-256 index (the 16 first use the common xterm values).
fn indexed_rgb(i: u8) -> (u8, u8, u8) {
    match i {
        0..=15 => ANSI16[usize::from(i)].1,
        16..=231 => {
            let n = usize::from(i - 16);
            (CUBE[n / 36], CUBE[n / 6 % 6], CUBE[n % 6])
        }
        _ => {
            let v = 8 + 10 * (i - 232);
            (v, v, v)
        }
    }
}

const ANSI16: [(Color, (u8, u8, u8)); 16] = [
    (Color::Black, (0, 0, 0)),
    (Color::Red, (205, 0, 0)),
    (Color::Green, (0, 205, 0)),
    (Color::Yellow, (205, 205, 0)),
    (Color::Blue, (0, 0, 238)),
    (Color::Magenta, (205, 0, 205)),
    (Color::Cyan, (0, 205, 205)),
    (Color::Gray, (229, 229, 229)),
    (Color::DarkGray, (127, 127, 127)),
    (Color::LightRed, (255, 0, 0)),
    (Color::LightGreen, (0, 255, 0)),
    (Color::LightYellow, (255, 255, 0)),
    (Color::LightBlue, (92, 92, 255)),
    (Color::LightMagenta, (255, 0, 255)),
    (Color::LightCyan, (0, 255, 255)),
    (Color::White, (255, 255, 255)),
];

/// The nearest of the 16 ANSI colors, as named colors (so the terminal's
/// own palette decides the exact shade, which is the point of 16 colors).
pub(crate) fn nearest_16(r: u8, g: u8, b: u8) -> Color {
    ANSI16
        .iter()
        .min_by_key(|(_, rgb)| dist((r, g, b), *rgb))
        .map(|(c, _)| *c)
        .unwrap_or(Color::Reset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn nearest_256_hits_the_cube_and_the_gray_ramp() {
        assert_eq!(nearest_256(0, 0, 0), 16);
        assert_eq!(nearest_256(255, 255, 255), 231);
        assert_eq!(nearest_256(255, 0, 0), 196);
        assert_eq!(nearest_256(95, 135, 175), 16 + 36 + 6 * 2 + 3);
        // A mid gray is nearer the ramp than any cube entry.
        assert_eq!(nearest_256(128, 128, 128), 244);
        assert_eq!(nearest_256(46, 52, 64), 237);
    }

    #[test]
    fn nearest_16_picks_the_closest_ansi_color() {
        assert_eq!(nearest_16(0, 0, 0), Color::Black);
        assert_eq!(nearest_16(250, 250, 250), Color::White);
        assert_eq!(nearest_16(200, 10, 10), Color::Red);
        assert_eq!(nearest_16(120, 120, 120), Color::DarkGray);
    }

    #[test]
    fn the_pass_maps_rgb_and_leaves_placeholders_and_named_colors() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        let rgb = Color::Rgb(255, 0, 0);
        for x in 0..3 {
            buf[(x, 0)].set_fg(rgb).set_bg(rgb);
        }
        buf[(1, 0)].set_symbol("\u{10EEEE}");
        buf[(2, 0)].set_fg(Color::Green);
        apply_depth(&mut buf, Depth::X256);
        assert_eq!(buf[(0, 0)].fg, Color::Indexed(196));
        assert_eq!(buf[(0, 0)].bg, Color::Indexed(196));
        assert_eq!(buf[(1, 0)].fg, rgb, "placeholder fg encodes the image id");
        assert_eq!(buf[(2, 0)].fg, Color::Green);
        apply_depth(&mut buf, Depth::X16);
        assert_eq!(buf[(0, 0)].fg, Color::LightRed, "256 entries reduce to 16");
        assert_eq!(buf[(1, 0)].fg, rgb);
    }

    #[test]
    fn explicit_settings_ignore_the_environment() {
        let e = env(&[("TERM", "linux")]);
        assert_eq!(resolve_with(ColorDepth::Truecolor, &e), Depth::True);
        assert_eq!(resolve_with(ColorDepth::Depth256, &e), Depth::X256);
        assert_eq!(resolve_with(ColorDepth::Depth16, &e), Depth::X16);
    }

    #[test]
    fn auto_follows_the_documented_rules() {
        let auto = |pairs: &[(&str, &str)]| resolve_with(ColorDepth::Auto, env(pairs));
        assert_eq!(auto(&[]), Depth::True);
        assert_eq!(auto(&[("TERM_PROGRAM", "Apple_Terminal")]), Depth::X256);
        assert_eq!(auto(&[("TERM", "linux")]), Depth::X16);
        assert_eq!(auto(&[("TERM", "xterm-256color")]), Depth::X256);
        assert_eq!(
            auto(&[("TERM", "xterm-256color"), ("COLORTERM", "truecolor")]),
            Depth::True
        );
        assert_eq!(
            auto(&[("TERM", "xterm-256color"), ("WT_SESSION", "abc")]),
            Depth::True
        );
        assert_eq!(
            auto(&[("TERM", "xterm-256color"), ("TERM_PROGRAM", "WezTerm")]),
            Depth::True
        );
        assert_eq!(auto(&[("TERM", "xterm-kitty")]), Depth::True);
        assert_eq!(
            auto(&[("TERM", "xterm-256color"), ("KITTY_WINDOW_ID", "1")]),
            Depth::True
        );
        // Apple Terminal lies with COLORTERM-less TERM but may be inherited
        // through ssh: the program name wins over a truecolor claim only
        // when COLORTERM does not say so.
        assert_eq!(
            auto(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("COLORTERM", "truecolor")
            ]),
            Depth::True
        );
    }
}
