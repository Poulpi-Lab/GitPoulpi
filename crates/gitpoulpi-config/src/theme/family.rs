//! Theme families: the unit a custom build selects themes by
//! (`GITPOULPI_THEMES=catppuccin,nord`). Shared by the crate and its
//! `build.rs` (`#[path]`), so the build-time filter and the run-time lookup
//! cannot disagree. Everything here is `const`-evaluable on purpose: the
//! presets and lane palettes of an unselected family are removed at compile
//! time, not just hidden.

#![allow(dead_code)] // `build.rs` includes this file and uses only part of it.

/// Themes that every build has: the fallbacks (`system` needs both Dark and
/// Light) and the terminal-colors preset.
pub const ALWAYS: [&str; 3] = ["dark", "light", "terminal"];

/// Lane palettes that are not tied to a theme family and always exist.
pub const ALWAYS_LANES: [&str; 4] = ["gitpoulpi", "light", "material", "serie"];

/// Multi-word family names; every other family is the slug's first word.
const MULTI_WORD: [&str; 4] = ["rose-pine", "tokyo-night", "night-owl", "shades-of-purple"];

const fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Whether `slug` is `family` or `family-<variant>`.
pub const fn in_family(slug: &str, family: &str) -> bool {
    let (s, f) = (slug.as_bytes(), family.as_bytes());
    if s.len() < f.len() {
        return false;
    }
    let mut i = 0;
    while i < f.len() {
        if s[i] != f[i] {
            return false;
        }
        i += 1;
    }
    s.len() == f.len() || s[f.len()] == b'-'
}

/// Whether `slug` is built when only `families` were selected (`None` = all).
pub const fn selected(slug: &str, families: Option<&[&str]>) -> bool {
    let Some(list) = families else {
        return true;
    };
    let mut i = 0;
    while i < ALWAYS.len() {
        if bytes_eq(slug.as_bytes(), ALWAYS[i].as_bytes()) {
            return true;
        }
        i += 1;
    }
    let mut i = 0;
    while i < list.len() {
        if in_family(slug, list[i]) {
            return true;
        }
        i += 1;
    }
    false
}

/// Whether the lane palette `slug` is built (see [`ALWAYS_LANES`]).
pub const fn lane_selected(slug: &str, families: Option<&[&str]>) -> bool {
    let mut i = 0;
    while i < ALWAYS_LANES.len() {
        if bytes_eq(slug.as_bytes(), ALWAYS_LANES[i].as_bytes()) {
            return true;
        }
        i += 1;
    }
    selected(slug, families)
}

/// The family of a theme slug (`catppuccin-latte` -> `catppuccin`,
/// `rose-pine-moon` -> `rose-pine`).
pub fn family_of(slug: &str) -> &str {
    for m in MULTI_WORD {
        if in_family(slug, m) {
            return m;
        }
    }
    slug.split('-').next().unwrap_or(slug)
}

/// Parses a `GITPOULPI_THEMES` value: `None` for "all" (or empty), otherwise
/// the sorted, deduplicated family list (`none` is the empty list).
pub fn parse_selection(value: &str) -> Option<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for raw in value.split(',').map(|s| s.trim().to_ascii_lowercase()) {
        match raw.as_str() {
            "" | "none" => {}
            "all" => return None,
            _ => out.push(raw),
        }
    }
    if value.trim().is_empty() {
        return None;
    }
    out.sort();
    out.dedup();
    Some(out)
}
