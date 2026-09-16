//! The built-in presets (Dark and Light here, the rest in `themes.rs`) and
//! the named lane palettes. Dark is exactly the colors the TUI painted before themes
//! existed; the TUI's parity test pins it.

use super::spec::{
    ChromeColors, DiffColors, EditorColors, GraphColors, HostingColors, MergeColors, MinimapColors,
    RefColors, ScrollbarColors, StatusColors, ToolbarColors, UiColors,
};
use super::terminalcolors;
use super::themes;
use super::{ThemeColor, ThemeSpec};

include!(concat!(env!("OUT_DIR"), "/theme_selection.rs"));

/// Whether the theme `$slug` is compiled in (a `const`, so an unselected
/// family's palette code is removed from the binary).
macro_rules! sel {
    ($slug:literal) => {{
        const ON: bool = super::family::selected($slug, SELECTION);
        ON
    }};
}

macro_rules! lane {
    ($out:ident, $slug:literal, $label:literal, $colors:expr) => {{
        const ON: bool = super::family::lane_selected($slug, SELECTION);
        if ON {
            $out.push(LanePalette {
                slug: $slug.to_string(),
                label: $label.to_string(),
                colors: $colors,
            });
        }
    }};
}

/// The slug a missing or unknown theme falls back to.
pub const DEFAULT_SLUG: &str = "dark";

const fn c(r: u8, g: u8, b: u8) -> ThemeColor {
    ThemeColor::Rgb(r, g, b)
}

fn rgbs(list: &[(u8, u8, u8)]) -> Vec<ThemeColor> {
    list.iter().map(|&(r, g, b)| c(r, g, b)).collect()
}

/// A named set of lane colors, listed by the lane combobox next to
/// "Follow theme".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanePalette {
    pub slug: String,
    pub label: String,
    pub colors: Vec<ThemeColor>,
}

fn gitpoulpi_lanes() -> Vec<ThemeColor> {
    rgbs(&[
        (64, 156, 255),
        (54, 179, 126),
        (255, 171, 0),
        (101, 84, 192),
        (255, 92, 141),
        (0, 184, 217),
        (255, 139, 0),
        (107, 119, 140),
    ])
}

fn material_lanes() -> Vec<ThemeColor> {
    // Material Design 400 shades (Google, Apache-2.0), ordered by hue from blue around the wheel; amber is skipped as too close to orange.
    rgbs(&[
        (0x42, 0xa5, 0xf5),
        (0x5c, 0x6b, 0xc0),
        (0x7e, 0x57, 0xc2),
        (0xab, 0x47, 0xbc),
        (0xec, 0x40, 0x7a),
        (0xef, 0x53, 0x50),
        (0x8d, 0x6e, 0x63),
        (0xff, 0x70, 0x43),
        (0xff, 0xa7, 0x26),
        (0xff, 0xee, 0x58),
        (0xd4, 0xe1, 0x57),
        (0x9c, 0xcc, 0x65),
        (0x66, 0xbb, 0x6a),
        (0x26, 0xa6, 0x9a),
        (0x26, 0xc6, 0xda),
        (0x78, 0x90, 0x9c),
    ])
}

fn serie_lanes() -> Vec<ThemeColor> {
    // lusingander/serie default branch colors (commit 4ac4539).
    rgbs(&[
        (0xe0, 0x6c, 0x76),
        (0x98, 0xc3, 0x79),
        (0xe5, 0xc0, 0x7b),
        (0x61, 0xaf, 0xef),
        (0xc6, 0x78, 0xdd),
        (0x56, 0xb6, 0xc2),
    ])
}

fn light_lanes() -> Vec<ThemeColor> {
    // GitPoulpi's lanes, darker and more saturated for a light background.
    rgbs(&[
        (0, 102, 204),
        (26, 127, 55),
        (176, 112, 0),
        (101, 84, 192),
        (207, 34, 106),
        (0, 126, 148),
        (201, 95, 0),
        (90, 100, 120),
    ])
}

fn dracula_lanes() -> Vec<ThemeColor> {
    // Dracula accents.
    rgbs(&[
        (0xbd, 0x93, 0xf9),
        (0x50, 0xfa, 0x7b),
        (0xf1, 0xfa, 0x8c),
        (0xff, 0x79, 0xc6),
        (0x8b, 0xe9, 0xfd),
        (0xff, 0xb8, 0x6c),
        (0xff, 0x55, 0x55),
        (0x62, 0x72, 0xa4),
    ])
}

fn tokyo_night_lanes() -> Vec<ThemeColor> {
    // Tokyo Night accents.
    rgbs(&[
        (0x7a, 0xa2, 0xf7),
        (0x9e, 0xce, 0x6a),
        (0xe0, 0xaf, 0x68),
        (0xbb, 0x9a, 0xf7),
        (0xf7, 0x76, 0x8e),
        (0x7d, 0xcf, 0xff),
        (0xff, 0x9e, 0x64),
        (0x1a, 0xbc, 0x9c),
    ])
}

fn cyberpunk_lanes() -> Vec<ThemeColor> {
    // Cyberpunk (iTerm2-Color-Schemes) ANSI colors, bright where the normal one is dull.
    rgbs(&[
        (0x00, 0xbf, 0xff),
        (0x00, 0xfb, 0xac),
        (0xff, 0xfa, 0x6a),
        (0xdf, 0x95, 0xff),
        (0xff, 0x70, 0x92),
        (0x86, 0xcb, 0xfe),
        (0x21, 0xf6, 0xbc),
        (0xff, 0x8a, 0xa4),
    ])
}

fn matrix_lanes() -> Vec<ThemeColor> {
    // Matrix (iTerm2-Color-Schemes) greens plus its yellows.
    rgbs(&[
        (0x11, 0xff, 0x25),
        (0x82, 0xd9, 0x67),
        (0x50, 0xb4, 0x5a),
        (0x40, 0x99, 0x31),
        (0x2f, 0xc0, 0x79),
        (0xc1, 0xff, 0x8a),
        (0xff, 0xd7, 0x00),
        (0x4f, 0x7e, 0x7e),
        (0x90, 0xd7, 0x62),
        (0x23, 0x75, 0x5a),
    ])
}

fn fire_lanes() -> Vec<ThemeColor> {
    // Original GitPoulpi palette: embers, reds through yellows, with a lime and a teal to tell lanes apart.
    rgbs(&[
        (0xff, 0x3b, 0x30),
        (0xff, 0x7a, 0x1a),
        (0xff, 0xc1, 0x07),
        (0xe0, 0x4a, 0x24),
        (0xff, 0xd9, 0x3d),
        (0xff, 0x8f, 0x6b),
        (0xa8, 0xc9, 0x3a),
        (0xff, 0x4f, 0xa3),
        (0x3f, 0xb8, 0xa8),
        (0xe8, 0xa8, 0x7c),
    ])
}

fn fire_light_lanes() -> Vec<ThemeColor> {
    // Original GitPoulpi palette for an orange background: dark embers and cool contrasts.
    rgbs(&[
        (0x9c, 0x0a, 0x0a),
        (0x4a, 0x1a, 0x00),
        (0x7a, 0x0a, 0x52),
        (0x00, 0x4f, 0x52),
        (0x1f, 0x2a, 0x6b),
        (0x24, 0x5c, 0x00),
        (0x6b, 0x3a, 0x00),
        (0x5a, 0x00, 0x24),
        (0x3a, 0x3a, 0x3a),
        (0x2e, 0x16, 0x4a),
    ])
}

fn sunset_lanes() -> Vec<ThemeColor> {
    // Original GitPoulpi palette: gold, orange, coral, pink, magenta, purple, indigo, sky.
    rgbs(&[
        (0xff, 0xc2, 0x33),
        (0xff, 0x9a, 0x3c),
        (0xff, 0x7f, 0x50),
        (0xff, 0x6f, 0xa0),
        (0xe0, 0x4f, 0xc0),
        (0xa0, 0x70, 0xf0),
        (0x7a, 0x7e, 0xf0),
        (0x6f, 0xd0, 0xff),
    ])
}

fn one_dark_lanes() -> Vec<ThemeColor> {
    // Atom One Dark accents.
    rgbs(&[
        (0x61, 0xaf, 0xef),
        (0x98, 0xc3, 0x79),
        (0xe5, 0xc0, 0x7b),
        (0xc6, 0x78, 0xdd),
        (0xe0, 0x6c, 0x75),
        (0x56, 0xb6, 0xc2),
        (0xd1, 0x9a, 0x66),
        (0xab, 0xb2, 0xbf),
    ])
}

fn rose_pine_lanes() -> Vec<ThemeColor> {
    // Rosé Pine accents.
    rgbs(&[
        (0x9c, 0xcf, 0xd8),
        (0xc4, 0xa7, 0xe7),
        (0xf6, 0xc1, 0x77),
        (0xeb, 0x6f, 0x92),
        (0xeb, 0xbc, 0xba),
        (0x3e, 0x8f, 0xb0),
        (0x90, 0x8c, 0xaa),
        (0x6e, 0x6a, 0x86),
    ])
}

fn kanagawa_lanes() -> Vec<ThemeColor> {
    // Kanagawa accents.
    rgbs(&[
        (0x7e, 0x9c, 0xd8),
        (0x98, 0xbb, 0x6c),
        (0xe6, 0xc3, 0x84),
        (0x95, 0x7f, 0xb8),
        (0xe4, 0x68, 0x76),
        (0x7a, 0xa8, 0x9f),
        (0xff, 0xa0, 0x66),
        (0xd2, 0x7e, 0x99),
    ])
}

fn everforest_lanes() -> Vec<ThemeColor> {
    // Everforest accents.
    rgbs(&[
        (0x7f, 0xbb, 0xb3),
        (0xa7, 0xc0, 0x80),
        (0xdb, 0xbc, 0x7f),
        (0xd6, 0x99, 0xb6),
        (0xe6, 0x7e, 0x80),
        (0x83, 0xc0, 0x92),
        (0xe6, 0x98, 0x75),
        (0x85, 0x92, 0x89),
    ])
}

fn monokai_lanes() -> Vec<ThemeColor> {
    // Monokai accents.
    rgbs(&[
        (0x66, 0xd9, 0xef),
        (0xa6, 0xe2, 0x2e),
        (0xe6, 0xdb, 0x74),
        (0xae, 0x81, 0xff),
        (0xf9, 0x26, 0x72),
        (0x78, 0xdc, 0xe8),
        (0xfd, 0x97, 0x1f),
        (0x90, 0x90, 0x8a),
    ])
}

fn gruvbox_light_lanes() -> Vec<ThemeColor> {
    // Gruvbox's faded accents, for a light background.
    rgbs(&[
        (0x07, 0x66, 0x78),
        (0x79, 0x74, 0x0e),
        (0xb5, 0x76, 0x14),
        (0x8f, 0x3f, 0x71),
        (0x9d, 0x00, 0x06),
        (0x42, 0x7b, 0x58),
        (0xaf, 0x3a, 0x03),
        (0x7c, 0x6f, 0x64),
    ])
}

fn one_light_lanes() -> Vec<ThemeColor> {
    // One Light accents.
    rgbs(&[
        (0x40, 0x78, 0xf2),
        (0x50, 0xa1, 0x4f),
        (0xc1, 0x84, 0x01),
        (0xa6, 0x26, 0xa4),
        (0xe4, 0x56, 0x49),
        (0x01, 0x84, 0xbc),
        (0x98, 0x68, 0x01),
        (0x69, 0x6c, 0x77),
    ])
}

fn tokyo_night_day_lanes() -> Vec<ThemeColor> {
    // Tokyo Night Day accents.
    rgbs(&[
        (0x2e, 0x7d, 0xe9),
        (0x58, 0x75, 0x39),
        (0x8c, 0x6c, 0x3e),
        (0x98, 0x54, 0xf1),
        (0xf5, 0x2a, 0x65),
        (0x00, 0x71, 0x97),
        (0xb1, 0x5c, 0x00),
        (0x68, 0x70, 0x9a),
    ])
}

fn rose_pine_moon_lanes() -> Vec<ThemeColor> {
    // Rosé Pine Moon accents.
    rgbs(&[
        (0x9c, 0xcf, 0xd8),
        (0xc4, 0xa7, 0xe7),
        (0xf6, 0xc1, 0x77),
        (0xeb, 0x6f, 0x92),
        (0xea, 0x9a, 0x97),
        (0x3e, 0x8f, 0xb0),
        (0x90, 0x8c, 0xaa),
        (0x6e, 0x6a, 0x86),
    ])
}

fn rose_pine_dawn_lanes() -> Vec<ThemeColor> {
    // Rosé Pine Dawn accents.
    rgbs(&[
        (0x28, 0x69, 0x83),
        (0x90, 0x7a, 0xa9),
        (0xc0, 0x7a, 0x12),
        (0xb4, 0x63, 0x7a),
        (0xd7, 0x82, 0x7e),
        (0x56, 0x94, 0x9f),
        (0x79, 0x75, 0x93),
        (0x98, 0x93, 0xa5),
    ])
}

fn catppuccin_macchiato_lanes() -> Vec<ThemeColor> {
    // Catppuccin Macchiato accents.
    rgbs(&[
        (0x8a, 0xad, 0xf4),
        (0xa6, 0xda, 0x95),
        (0xee, 0xd4, 0x9f),
        (0xc6, 0xa0, 0xf6),
        (0xf5, 0xbd, 0xe6),
        (0x8b, 0xd5, 0xca),
        (0xf5, 0xa9, 0x7f),
        (0xed, 0x87, 0x96),
        (0xb7, 0xbd, 0xf8),
        (0x91, 0xd7, 0xe3),
    ])
}

fn catppuccin_frappe_lanes() -> Vec<ThemeColor> {
    // Catppuccin Frappé accents.
    rgbs(&[
        (0x8c, 0xaa, 0xee),
        (0xa6, 0xd1, 0x89),
        (0xe5, 0xc8, 0x90),
        (0xca, 0x9e, 0xe6),
        (0xf4, 0xb8, 0xe4),
        (0x81, 0xc8, 0xbe),
        (0xef, 0x9f, 0x76),
        (0xe7, 0x82, 0x84),
        (0xba, 0xbb, 0xf1),
        (0x99, 0xd1, 0xdb),
    ])
}

fn kanagawa_lotus_lanes() -> Vec<ThemeColor> {
    // Kanagawa Lotus accents.
    rgbs(&[
        (0x4d, 0x69, 0x9b),
        (0x6f, 0x89, 0x4e),
        (0x83, 0x6f, 0x4a),
        (0x62, 0x4c, 0x83),
        (0xc8, 0x40, 0x53),
        (0x59, 0x7b, 0x75),
        (0xcc, 0x6d, 0x00),
        (0x71, 0x6e, 0x61),
    ])
}

fn github_dark_lanes() -> Vec<ThemeColor> {
    // GitHub Dark (Primer) accents: blue, green, yellow, purple, red, cyan, orange, pink.
    rgbs(&[
        (0x2f, 0x81, 0xf7),
        (0x3f, 0xb9, 0x50),
        (0xd2, 0x99, 0x22),
        (0xa3, 0x71, 0xf7),
        (0xf8, 0x51, 0x49),
        (0x39, 0xc5, 0xcf),
        (0xdb, 0x6d, 0x28),
        (0xdb, 0x61, 0xa2),
    ])
}

fn github_light_lanes() -> Vec<ThemeColor> {
    // GitHub Light (Primer) accents: blue, green, yellow, purple, red, cyan, orange, pink.
    rgbs(&[
        (0x09, 0x69, 0xda),
        (0x1a, 0x7f, 0x37),
        (0x9a, 0x67, 0x00),
        (0x82, 0x50, 0xdf),
        (0xd1, 0x24, 0x2f),
        (0x1b, 0x7c, 0x83),
        (0xbc, 0x4c, 0x00),
        (0xbf, 0x39, 0x89),
    ])
}

fn vscode_dark_lanes() -> Vec<ThemeColor> {
    // VS Code Dark Modern: link blue, green, yellow, purple, red, cyan, charts.orange, gray.
    rgbs(&[
        (0x4d, 0xaa, 0xfc),
        (0x2e, 0xa0, 0x43),
        (0xcc, 0xa7, 0x00),
        (0xb1, 0x80, 0xd7),
        (0xf8, 0x51, 0x49),
        (0x29, 0xb8, 0xdb),
        (0xd1, 0x86, 0x16),
        (0x9d, 0x9d, 0x9d),
    ])
}

fn vscode_light_lanes() -> Vec<ThemeColor> {
    // VS Code Light Modern: blue, charts green/yellow/purple, red, cyan, charts.orange, gray.
    rgbs(&[
        (0x00, 0x5f, 0xb8),
        (0x38, 0x8a, 0x34),
        (0xbf, 0x88, 0x03),
        (0x65, 0x2d, 0x90),
        (0xe5, 0x14, 0x00),
        (0x05, 0x98, 0xbc),
        (0xd1, 0x86, 0x16),
        (0x61, 0x61, 0x61),
    ])
}

fn vesper_lanes() -> Vec<ThemeColor> {
    // Vesper accents.
    rgbs(&[
        (0xff, 0xc7, 0x99),
        (0x99, 0xff, 0xe4),
        (0xff, 0x80, 0x80),
        (0xb9, 0xa4, 0xff),
        (0xa0, 0xa0, 0xa0),
        (0x7f, 0xd1, 0xc1),
        (0xe6, 0xa3, 0xa3),
        (0xff, 0xe0, 0xc2),
    ])
}

fn solarized_lanes() -> Vec<ThemeColor> {
    // The Solarized accents, shared by the dark and light variants.
    rgbs(&[
        (0x26, 0x8b, 0xd2),
        (0x85, 0x99, 0x00),
        (0xb5, 0x89, 0x00),
        (0x6c, 0x71, 0xc4),
        (0xd3, 0x36, 0x82),
        (0x2a, 0xa1, 0x98),
        (0xcb, 0x4b, 0x16),
        (0xdc, 0x32, 0x2f),
    ])
}

fn catppuccin_mocha_lanes() -> Vec<ThemeColor> {
    rgbs(&[
        (0x89, 0xb4, 0xfa),
        (0xa6, 0xe3, 0xa1),
        (0xf9, 0xe2, 0xaf),
        (0xcb, 0xa6, 0xf7),
        (0xf5, 0xc2, 0xe7),
        (0x94, 0xe2, 0xd5),
        (0xfa, 0xb3, 0x87),
        (0xf3, 0x8b, 0xa8),
        (0xb4, 0xbe, 0xfe),
        (0x89, 0xdc, 0xeb),
    ])
}

fn catppuccin_latte_lanes() -> Vec<ThemeColor> {
    rgbs(&[
        (0x1e, 0x66, 0xf5),
        (0x40, 0xa0, 0x2b),
        (0xdf, 0x8e, 0x1d),
        (0x88, 0x39, 0xef),
        (0xea, 0x76, 0xcb),
        (0x17, 0x92, 0x99),
        (0xfe, 0x64, 0x0b),
        (0xd2, 0x0f, 0x39),
        (0x72, 0x87, 0xfd),
        (0x04, 0xa5, 0xe5),
    ])
}

fn gruvbox_lanes() -> Vec<ThemeColor> {
    rgbs(&[
        (0x83, 0xa5, 0x98),
        (0xb8, 0xbb, 0x26),
        (0xfa, 0xbd, 0x2f),
        (0xd3, 0x86, 0x9b),
        (0xfb, 0x49, 0x34),
        (0x8e, 0xc0, 0x7c),
        (0xfe, 0x80, 0x19),
        (0x92, 0x83, 0x74),
    ])
}

fn nord_lanes() -> Vec<ThemeColor> {
    // Frost and Aurora.
    rgbs(&[
        (0x88, 0xc0, 0xd0),
        (0xa3, 0xbe, 0x8c),
        (0xeb, 0xcb, 0x8b),
        (0xb4, 0x8e, 0xad),
        (0xbf, 0x61, 0x6a),
        (0x8f, 0xbc, 0xbb),
        (0xd0, 0x87, 0x70),
        (0x81, 0xa1, 0xc1),
    ])
}

/// A built-in lane palette's colors by registry slug (empty if unknown).
pub(super) fn lane_colors(slug: &str) -> Vec<ThemeColor> {
    lane_cache()
        .iter()
        .find(|p| p.slug == slug)
        .map(|p| p.colors.clone())
        .unwrap_or_default()
}

/// Every built-in lane palette. The first four are the former
/// `ui.graph_color_theme` choices; each preset's own palette is listed too
/// (the Terminal preset keeps GitPoulpi's: graph pixels need RGB).
pub fn lane_palettes() -> Vec<LanePalette> {
    lane_cache().clone()
}

fn lane_cache() -> &'static Vec<LanePalette> {
    static CACHE: std::sync::OnceLock<Vec<LanePalette>> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_lane_palettes)
}

fn build_lane_palettes() -> Vec<LanePalette> {
    let mut out = Vec::new();
    lane!(out, "gitpoulpi", "GitPoulpi Dark", gitpoulpi_lanes());
    lane!(out, "material", "Material", material_lanes());
    lane!(out, "serie", "Serie", serie_lanes());
    lane!(out, "light", "GitPoulpi Light", light_lanes());
    lane!(out, "solarized", "Solarized", solarized_lanes());
    lane!(
        out,
        "catppuccin-mocha",
        "Catppuccin Mocha",
        catppuccin_mocha_lanes()
    );
    lane!(
        out,
        "catppuccin-latte",
        "Catppuccin Latte",
        catppuccin_latte_lanes()
    );
    lane!(out, "gruvbox", "Gruvbox", gruvbox_lanes());
    lane!(out, "nord", "Nord", nord_lanes());
    lane!(out, "dracula", "Dracula", dracula_lanes());
    lane!(out, "tokyo-night", "Tokyo Night", tokyo_night_lanes());
    lane!(out, "cyberpunk", "Cyberpunk", cyberpunk_lanes());
    lane!(out, "matrix", "Matrix", matrix_lanes());
    lane!(out, "fire", "Fire", fire_lanes());
    lane!(out, "fire-light", "Fire Light", fire_light_lanes());
    lane!(out, "sunset", "Sunset", sunset_lanes());
    lane!(out, "one-dark", "One Dark", one_dark_lanes());
    lane!(out, "rose-pine", "Rosé Pine", rose_pine_lanes());
    lane!(out, "kanagawa", "Kanagawa", kanagawa_lanes());
    lane!(out, "everforest", "Everforest", everforest_lanes());
    lane!(out, "monokai", "Monokai", monokai_lanes());
    lane!(out, "gruvbox-light", "Gruvbox Light", gruvbox_light_lanes());
    lane!(out, "one-light", "One Light", one_light_lanes());
    lane!(
        out,
        "tokyo-night-day",
        "Tokyo Night Day",
        tokyo_night_day_lanes()
    );
    lane!(
        out,
        "rose-pine-moon",
        "Rosé Pine Moon",
        rose_pine_moon_lanes()
    );
    lane!(
        out,
        "rose-pine-dawn",
        "Rosé Pine Dawn",
        rose_pine_dawn_lanes()
    );
    lane!(
        out,
        "catppuccin-macchiato",
        "Catppuccin Macchiato",
        catppuccin_macchiato_lanes()
    );
    lane!(
        out,
        "catppuccin-frappe",
        "Catppuccin Frappé",
        catppuccin_frappe_lanes()
    );
    lane!(
        out,
        "kanagawa-lotus",
        "Kanagawa Lotus",
        kanagawa_lotus_lanes()
    );
    lane!(out, "vesper", "Vesper", vesper_lanes());
    lane!(out, "github-dark", "GitHub Dark", github_dark_lanes());
    lane!(out, "github-light", "GitHub Light", github_light_lanes());
    lane!(out, "vscode-dark", "VS Code Dark", vscode_dark_lanes());
    lane!(out, "vscode-light", "VS Code Light", vscode_light_lanes());
    for t in terminalcolors::TERMINAL_COLORS {
        out.push(LanePalette {
            slug: t.slug.to_string(),
            label: t.label.to_string(),
            colors: terminalcolors::lanes(t),
        });
    }
    // The order rule: GitPoulpi Dark and GitPoulpi Light first, the rest
    // alphabetical by label. `sort_by_key` is stable.
    out.sort_by_key(|p| {
        let rank = match p.slug.as_str() {
            "gitpoulpi" => 0,
            "light" => 1,
            _ => 2,
        };
        (rank, p.label.to_lowercase())
    });
    out
}

/// The built-in palettes plus the `graph_palette` of every given theme not
/// already listed under that slug (user themes, later presets).
pub fn lane_palettes_with(themes: &[ThemeSpec]) -> Vec<LanePalette> {
    let mut out = lane_palettes();
    for t in themes {
        if t.graph_palette.is_empty() || out.iter().any(|p| p.slug == t.slug) {
            continue;
        }
        out.push(LanePalette {
            slug: t.slug.clone(),
            label: t.name.clone(),
            colors: t.graph_palette.clone(),
        });
    }
    out
}

/// A built-in lane palette by slug.
pub fn lane_palette(slug: &str) -> Option<LanePalette> {
    let slug = canonical_slug(slug);
    lane_cache().iter().find(|p| p.slug == slug).cloned()
}

pub(super) fn dark() -> ThemeSpec {
    let panel = c(28, 30, 46);
    let text = c(226, 228, 235);
    let muted = c(150, 154, 172);
    let accent = c(64, 156, 255);
    let amber = c(255, 171, 0);
    ThemeSpec {
        slug: "dark".into(),
        name: "GitPoulpi Dark".into(),
        light: false,
        syntax: "base16-ocean.dark".into(),
        graph_palette: gitpoulpi_lanes(),
        ui: UiColors {
            bg: ThemeColor::Default,
            blend_bg: c(29, 32, 34),
            panel,
            side_bg: ThemeColor::Default,
            // The terminal's own background: nothing to tint.
            toolbar_bg: ThemeColor::Default,
            statusbar_bg: ThemeColor::Default,
            statusbar_fg: ThemeColor::Default,
            text,
            muted,
            disabled: c(136, 140, 158),
            accent,
            on_accent: c(16, 18, 28),
            selection: c(52, 58, 90),
            hover: c(34, 38, 56),
            button: c(70, 75, 105),
            button_hover: c(88, 95, 130),
            // `Color::White` today: the terminal's ANSI 7.
            on_button: ThemeColor::Ansi(7),
            border_focused: accent,
            border: muted,
            tooltip_bg: panel,
            popup_bg: panel,
            header_fg: text,
            input_rest: c(42, 42, 46),
            input_hover: c(52, 52, 56),
            input_focus: c(60, 60, 64),
            text_selection_bg: accent,
            text_selection_fg: c(16, 18, 28),
        },
        status: StatusColors {
            success: c(54, 179, 126),
            warning: amber,
            danger: c(255, 99, 99),
            info: accent,
            ai: c(186, 134, 255),
        },
        refs: RefColors {
            local: c(54, 179, 126),
            remote: accent,
            tag: amber,
        },
        scrollbar: ScrollbarColors {
            track: c(40, 42, 60),
            thumb: c(90, 96, 130),
            thumb_hover: accent,
            marks_add: c(80, 200, 120),
            marks_del: c(230, 90, 90),
            marks_cursor: c(240, 240, 250),
            marks_occurrence: c(240, 200, 90),
            marks_modified: c(220, 170, 90),
            marks_conflict_open: c(240, 150, 60),
            marks_conflict_current: c(120, 180, 255),
        },
        minimap: MinimapColors {
            bg: c(24, 26, 38),
            view: c(56, 62, 92),
            ink: c(150, 156, 184),
        },
        diff: DiffColors {
            add_fg: c(126, 211, 158),
            del_fg: c(240, 138, 138),
            add_bg: c(24, 52, 38),
            del_bg: c(58, 30, 34),
            hunk_fg: c(140, 160, 210),
            gutter_fg: c(108, 114, 140),
            range_bg: c(40, 45, 70),
            search_hit_bg: c(92, 74, 18),
        },
        merge: MergeColors {
            ours: c(110, 190, 255),
            theirs: c(240, 170, 90),
            base: c(160, 165, 185),
            edit: c(255, 196, 64),
            text: c(192, 197, 206),
            ours_tint: c(24, 40, 60),
            theirs_tint: c(56, 42, 24),
            ours_pick: c(28, 54, 84),
            theirs_pick: c(76, 54, 26),
            base_pick: c(44, 46, 60),
            current_region: c(26, 30, 46),
            strip_bg: c(56, 44, 20),
            strip_fg: c(255, 196, 64),
        },
        editor: EditorColors {
            text: c(192, 197, 206),
            current_line: c(30, 34, 50),
            find_hit: c(120, 100, 40),
            find_hit_fg: c(255, 255, 255),
        },
        toolbar: ToolbarColors {
            pull: c(150, 195, 255),
            push: c(150, 210, 180),
            stash: c(255, 205, 130),
            branch: c(175, 160, 220),
            lfs: c(185, 215, 150),
            tag: c(230, 190, 130),
            search: c(200, 200, 215),
            undo: c(140, 205, 225),
        },
        hosting: HostingColors {
            merged: c(163, 113, 247),
            chip_fg: c(20, 22, 30),
        },
        chrome: ChromeColors {
            // Named terminal-palette colors today (they follow the terminal's
            // own 16 colors), kept as ANSI so Dark renders identically.
            stat_add: ThemeColor::Ansi(2),
            stat_del: ThemeColor::Ansi(1),
            notice: ThemeColor::Ansi(1),
            marker: ThemeColor::Ansi(5),
            on_bright: ThemeColor::Ansi(0),
            wip_del: c(255, 92, 141),
            error_bg: c(70, 24, 30),
            banner_bg: c(56, 44, 20),
            banner_danger_bg: c(120, 40, 40),
            toolbar_arrow_bg: c(58, 62, 90),
            pill_branch: c(90, 40, 120),
            pill_onto: c(20, 90, 100),
            action_edit: c(120, 170, 255),
            dropdown_bg: c(30, 34, 52),
            dropdown_hit: c(150, 200, 255),
            action_reset: c(90, 140, 255),
            tint_reset: c(28, 40, 78),
            tint_danger: c(56, 30, 40),
            tint_success: c(24, 52, 40),
            toast_bg: c(70, 50, 110),
            proposal_bg: c(110, 60, 170),
            confirm_danger: c(165, 45, 45),
            disabled_button_bg: c(40, 44, 58),
            toolbar_bg: c(70, 75, 105),
            toolbar_bg_hover: c(88, 95, 130),
            toolbar_fg: ThemeColor::Ansi(7),
            toolbar_disabled_fg: muted,
        },
        graph: GraphColors {
            wip: muted,
            stash: muted,
        },
    }
}

pub(super) fn light() -> ThemeSpec {
    let bg = c(250, 250, 252);
    let panel = c(240, 241, 246);
    let text = c(28, 30, 40);
    let muted = c(96, 100, 120);
    let accent = c(0, 102, 204);
    let amber = c(176, 112, 0);
    ThemeSpec {
        slug: "light".into(),
        name: "GitPoulpi Light".into(),
        light: true,
        syntax: "InspiredGitHub".into(),
        graph_palette: light_lanes(),
        ui: UiColors {
            bg,
            blend_bg: bg,
            panel,
            side_bg: ThemeColor::Default,
            toolbar_bg: c(234, 236, 244),
            statusbar_bg: c(244, 245, 250),
            statusbar_fg: text,
            text,
            muted,
            disabled: c(108, 112, 130),
            accent,
            on_accent: c(255, 255, 255),
            selection: c(200, 214, 240),
            hover: c(228, 232, 244),
            button: c(58, 64, 96),
            button_hover: c(88, 96, 140),
            on_button: c(255, 255, 255),
            border_focused: accent,
            border: c(150, 154, 170),
            tooltip_bg: panel,
            popup_bg: panel,
            header_fg: text,
            input_rest: c(232, 234, 240),
            input_hover: c(222, 225, 234),
            input_focus: c(255, 255, 255),
            text_selection_bg: accent,
            text_selection_fg: c(255, 255, 255),
        },
        status: StatusColors {
            success: c(26, 127, 55),
            warning: amber,
            danger: c(200, 30, 30),
            info: accent,
            ai: c(130, 60, 200),
        },
        refs: RefColors {
            local: c(26, 127, 55),
            remote: accent,
            tag: amber,
        },
        scrollbar: ScrollbarColors {
            track: c(226, 228, 236),
            thumb: c(170, 176, 196),
            thumb_hover: accent,
            marks_add: c(40, 150, 80),
            marks_del: c(200, 60, 60),
            marks_cursor: c(40, 44, 60),
            marks_occurrence: c(200, 150, 20),
            marks_modified: c(190, 130, 40),
            marks_conflict_open: c(200, 110, 20),
            marks_conflict_current: c(40, 110, 220),
        },
        minimap: MinimapColors {
            bg: c(238, 240, 246),
            view: c(206, 214, 236),
            ink: c(110, 116, 140),
        },
        diff: DiffColors {
            add_fg: c(20, 100, 50),
            del_fg: c(160, 30, 30),
            add_bg: c(204, 240, 214),
            del_bg: c(255, 215, 215),
            hunk_fg: c(60, 80, 150),
            gutter_fg: c(120, 126, 146),
            range_bg: c(220, 228, 246),
            search_hit_bg: c(255, 230, 140),
        },
        merge: MergeColors {
            ours: c(0, 102, 204),
            theirs: c(176, 96, 0),
            base: c(100, 106, 130),
            edit: c(160, 110, 0),
            text: c(40, 44, 56),
            ours_tint: c(226, 238, 252),
            theirs_tint: c(252, 238, 220),
            ours_pick: c(196, 220, 248),
            theirs_pick: c(250, 218, 180),
            base_pick: c(228, 230, 238),
            current_region: c(236, 240, 250),
            strip_bg: c(252, 238, 200),
            strip_fg: c(130, 86, 0),
        },
        editor: EditorColors {
            text: c(40, 44, 56),
            current_line: c(236, 238, 246),
            find_hit: c(255, 224, 120),
            find_hit_fg: c(16, 18, 28),
        },
        // The toolbar's chips are light on Light (`chrome.toolbar_bg`), so
        // its icons are deep, saturated tones.
        toolbar: ToolbarColors {
            pull: c(20, 90, 190),
            push: c(20, 110, 50),
            stash: c(150, 90, 0),
            branch: c(110, 60, 180),
            lfs: c(70, 110, 20),
            tag: c(150, 90, 20),
            search: c(80, 86, 110),
            undo: c(10, 100, 130),
        },
        hosting: HostingColors {
            merged: c(130, 70, 220),
            chip_fg: c(255, 255, 255),
        },
        chrome: ChromeColors {
            stat_add: ThemeColor::Ansi(2),
            stat_del: ThemeColor::Ansi(1),
            notice: ThemeColor::Ansi(1),
            marker: ThemeColor::Ansi(5),
            on_bright: c(255, 255, 255),
            wip_del: c(200, 40, 100),
            error_bg: c(252, 226, 228),
            banner_bg: c(255, 240, 205),
            banner_danger_bg: c(200, 50, 50),
            toolbar_arrow_bg: c(196, 203, 226),
            pill_branch: c(120, 70, 160),
            pill_onto: c(20, 110, 124),
            action_edit: c(0, 102, 204),
            dropdown_bg: c(228, 232, 244),
            dropdown_hit: c(40, 110, 200),
            action_reset: c(0, 102, 204),
            tint_reset: c(220, 232, 250),
            tint_danger: c(252, 226, 228),
            tint_success: c(220, 242, 228),
            toast_bg: c(110, 80, 170),
            proposal_bg: c(110, 60, 170),
            confirm_danger: c(200, 50, 50),
            disabled_button_bg: c(224, 227, 236),
            toolbar_bg: c(214, 219, 234),
            toolbar_bg_hover: c(194, 202, 226),
            toolbar_fg: c(40, 44, 56),
            toolbar_disabled_fg: muted,
        },
        graph: GraphColors {
            wip: muted,
            stash: muted,
        },
    }
}

/// Every built-in preset, in combo order (built once: deriving the imported
/// palettes runs the contrast fix, too slow to repeat on every redraw).
pub fn presets() -> Vec<ThemeSpec> {
    all().clone()
}

fn all() -> &'static Vec<ThemeSpec> {
    static ALL: std::sync::OnceLock<Vec<ThemeSpec>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        const REST: &[&str] = &[
            "solarized-dark",
            "solarized-light",
            "catppuccin-mocha",
            "catppuccin-latte",
            "gruvbox-dark",
            "nord",
            "terminal",
            "dracula",
            "tokyo-night",
            "cyberpunk",
            "matrix",
            "fire",
            "fire-light",
            "sunset",
            "one-dark",
            "rose-pine",
            "kanagawa",
            "everforest-dark",
            "monokai",
            "gruvbox-light",
            "one-light",
            "tokyo-night-day",
            "rose-pine-moon",
            "rose-pine-dawn",
            "catppuccin-macchiato",
            "catppuccin-frappe",
            "kanagawa-lotus",
            "vesper",
            "github-dark",
            "github-light",
            "vscode-dark",
            "vscode-light",
        ];
        let mut rest: Vec<(String, ThemeSpec)> = REST
            .iter()
            .copied()
            .chain(terminalcolors::slugs())
            .filter_map(|s| build_preset(s).map(|t| (t.name.to_lowercase(), t)))
            .collect();
        rest.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.slug.cmp(&b.1.slug)));
        let mut out = vec![dark(), light()];
        out.extend(rest.into_iter().map(|(_, t)| t));
        out
    })
}

/// Theme and lane palette slugs that were renamed: an old config file or
/// command line that still says the old one keeps its look. Silent on
/// purpose (never listed, never warned about). None today.
const SLUG_ALIASES: [(&str, &str); 0] = [];

/// The current slug for `slug`, which may be a retired alias.
pub fn canonical_slug(slug: &str) -> &str {
    SLUG_ALIASES
        .iter()
        .find(|(old, _)| *old == slug)
        .map_or(slug, |(_, new)| new)
}

/// A built-in preset by slug (a retired alias works too).
pub fn preset(slug: &str) -> Option<ThemeSpec> {
    let slug = canonical_slug(slug);
    all().iter().find(|t| t.slug == slug).cloned()
}

fn build_preset(slug: &str) -> Option<ThemeSpec> {
    Some(match slug {
        "dark" => dark(),
        "light" => light(),
        "solarized-dark" if sel!("solarized-dark") => themes::solarized_dark(),
        "solarized-light" if sel!("solarized-light") => themes::solarized_light(),
        "catppuccin-mocha" if sel!("catppuccin-mocha") => themes::catppuccin_mocha(),
        "catppuccin-latte" if sel!("catppuccin-latte") => themes::catppuccin_latte(),
        "gruvbox-dark" if sel!("gruvbox-dark") => themes::gruvbox_dark(),
        "nord" if sel!("nord") => themes::nord(),
        "terminal" if sel!("terminal") => themes::terminal(),
        "dracula" if sel!("dracula") => themes::dracula(),
        "tokyo-night" if sel!("tokyo-night") => themes::tokyo_night(),
        "cyberpunk" if sel!("cyberpunk") => themes::cyberpunk(),
        "matrix" if sel!("matrix") => themes::matrix(),
        "fire" if sel!("fire") => themes::fire(),
        "fire-light" if sel!("fire-light") => themes::fire_light(),
        "sunset" if sel!("sunset") => themes::sunset(),
        "one-dark" if sel!("one-dark") => themes::one_dark(),
        "rose-pine" if sel!("rose-pine") => themes::rose_pine(),
        "kanagawa" if sel!("kanagawa") => themes::kanagawa(),
        "everforest-dark" if sel!("everforest-dark") => themes::everforest_dark(),
        "monokai" if sel!("monokai") => themes::monokai(),
        "gruvbox-light" if sel!("gruvbox-light") => themes::gruvbox_light(),
        "one-light" if sel!("one-light") => themes::one_light(),
        "tokyo-night-day" if sel!("tokyo-night-day") => themes::tokyo_night_day(),
        "rose-pine-moon" if sel!("rose-pine-moon") => themes::rose_pine_moon(),
        "rose-pine-dawn" if sel!("rose-pine-dawn") => themes::rose_pine_dawn(),
        "catppuccin-macchiato" if sel!("catppuccin-macchiato") => themes::catppuccin_macchiato(),
        "catppuccin-frappe" if sel!("catppuccin-frappe") => themes::catppuccin_frappe(),
        "kanagawa-lotus" if sel!("kanagawa-lotus") => themes::kanagawa_lotus(),
        "vesper" if sel!("vesper") => themes::vesper(),
        "github-dark" if sel!("github-dark") => themes::github_dark(),
        "github-light" if sel!("github-light") => themes::github_light(),
        "vscode-dark" if sel!("vscode-dark") => themes::vscode_dark(),
        "vscode-light" if sel!("vscode-light") => themes::vscode_light(),
        other => return terminalcolors::find(other).map(terminalcolors::derive_tc),
    })
}

/// Every built-in slug in combo order, the single source of truth for every
/// theme list: `dark` and `light` (GitPoulpi Dark / GitPoulpi Light) first,
/// then all the others sorted alphabetically by displayed name,
/// case-insensitive.
pub fn preset_slugs() -> Vec<&'static str> {
    all().iter().map(|t| t.slug.as_str()).collect()
}

/// Whether `slug` is one of the presets imported from terminalcolors.com.
pub fn is_imported(slug: &str) -> bool {
    terminalcolors::find(slug).is_some()
}

/// What this build's theme set is, for `--version`: `all`, or the fixed
/// themes plus the selected families.
pub fn theme_set() -> String {
    match SELECTION {
        None => "all".to_string(),
        Some(list) => {
            let mut parts: Vec<&str> = super::family::ALWAYS.to_vec();
            parts.extend(list.iter().copied());
            parts.join(" ")
        }
    }
}

/// Adds the `themes: ...` line to a `--version` text, before its
/// `custom build:` line when it has one.
pub fn with_theme_line(mut text: String) -> String {
    let line = format!("themes: {}", theme_set());
    match text.find("\ncustom build:") {
        Some(at) => text.insert_str(at, &format!("\n{line}")),
        None => {
            text.push('\n');
            text.push_str(&line);
        }
    }
    text
}
