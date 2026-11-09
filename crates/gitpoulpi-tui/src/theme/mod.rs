//! The TUI's palette and glyphs: the fixed truecolor RGB colors every pane
//! draws with, the ref-kind icons (local/remote/tag/worktree/stash, the
//! hide-in-graph eye) and the small style helpers built on them.

use super::*;

#[cfg(test)]
#[path = "tests.rs"]
mod parity_tests;
mod roles;
mod runtime;
#[cfg(test)]
mod snapshot_tests;

#[allow(unused_imports)]
pub(crate) use roles::color_of;
#[allow(unused_imports)]
pub(crate) use roles::Theme;
#[cfg(test)]
pub(crate) use runtime::override_theme;
#[allow(unused_imports)]
pub(crate) use runtime::set_system_is_light;
#[allow(unused_imports)]
pub(crate) use runtime::{install_theme, install_theme_from_config, th, theme_generation};

/// The sparkle every AI control carries (button, menu entry, palette
/// command), text presentation, one column. Painted in `status.ai`.
pub(crate) const AI_SPARK: &str = "\u{2726}";
/// The named presets of `ui.input_color`, in the order the Settings
/// combo lists them. `theme` (the default) follows the active theme;
/// `custom` is the last: it reads `ui.input_color_custom`.
pub(crate) const INPUT_PRESETS: &[&str] = &["theme", "gray", "blue", "slate", "dark", "custom"];

/// Resting input background of a preset (the `custom` entry has none).
pub(crate) fn input_preset_rgb(name: &str) -> Option<(u8, u8, u8)> {
    match name.trim().to_ascii_lowercase().as_str() {
        "gray" | "grey" => Some((42, 42, 46)),
        "blue" => Some((28, 34, 52)),
        "slate" => Some((34, 42, 50)),
        "dark" => Some((20, 20, 24)),
        _ => None,
    }
}

/// Parses a strict `#rrggbb` (the leading `#` is required).
pub(crate) fn parse_hex_color(text: &str) -> Option<(u8, u8, u8)> {
    let hex = text.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(hex, 16).ok()?;
    Some(((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

/// The resting input color for a `ui.input_color` / `ui.input_color_custom`
/// pair; anything unreadable falls back to gray.
pub(crate) fn resolve_input_color(preset: &str, custom: &str) -> (u8, u8, u8) {
    let picked = if preset.trim().eq_ignore_ascii_case("custom") {
        parse_hex_color(custom)
    } else {
        input_preset_rgb(preset)
    };
    picked
        .or_else(|| input_preset_rgb("gray"))
        .unwrap_or((42, 42, 46))
}

/// `ui.input_color`'s explicit override (`0xRRGGBB`), or [`NO_INPUT_OVERRIDE`]
/// to follow the theme's input colors.
static INPUT_RGB: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(NO_INPUT_OVERRIDE);
const NO_INPUT_OVERRIDE: u32 = u32::MAX;

/// Publishes the configured input color (live: read on the next frame).
/// The preset `theme` (the future default) follows the theme instead.
pub(crate) fn install_input_color(preset: &str, custom: &str) {
    if preset.trim().eq_ignore_ascii_case("theme") {
        INPUT_RGB.store(NO_INPUT_OVERRIDE, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    let (r, g, b) = resolve_input_color(preset, custom);
    INPUT_RGB.store(
        (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
        std::sync::atomic::Ordering::Relaxed,
    );
}

fn input_override() -> Option<Color> {
    let v = INPUT_RGB.load(std::sync::atomic::Ordering::Relaxed);
    (v != NO_INPUT_OVERRIDE).then_some(Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

/// Background of a single-line text input at rest, so a field reads as
/// a field. The theme's, unless `ui.input_color` overrides it; distinct
/// from `SELECTION_BG` (a selected row) and `HOVER_BG`.
pub(crate) fn input_rest_bg() -> Color {
    input_override().unwrap_or_else(|| th().ui.input_rest)
}

/// Background of the text input that has the keyboard focus: the
/// resting color, lighter.
pub(crate) fn input_focus_bg() -> Color {
    match input_override() {
        Some(rest) => crate::lighten(rest, 18),
        None => th().ui.input_focus,
    }
}

/// The background of a text input: resting, hovered or focused.
pub(crate) fn input_bg(focused: bool, hovered: bool) -> Color {
    if focused {
        input_focus_bg()
    } else if hovered {
        match input_override() {
            Some(rest) => crate::lighten(rest, 10),
            None => th().ui.input_hover,
        }
    } else {
        input_rest_bg()
    }
}

/// Color + icon for a given ref kind, shared by the Log column's pills
/// and the Refs sidebar's rows so both read as the same design system.
pub(crate) fn ref_style(kind: RefRowKind) -> (Color, &'static str) {
    match kind {
        RefRowKind::Local => (th().refs.local, LOCAL_ICON),
        RefRowKind::Remote => (th().refs.remote, REMOTE_ICON),
        RefRowKind::Tag => (th().refs.tag, TAG_ICON),
    }
}

/// Style for one half of a segmented control (`Path`/`Tree`): the
/// selected half is filled, the other is plain. A segmented control shows
/// which option is active, unlike a toolbar button that only reacts to
/// hover. Under the pointer, the inactive half fills like a plain button
/// and the active half brightens.
pub(crate) fn segmented_style(active: bool, hovered: bool) -> Style {
    if active {
        Style::default()
            .bg(if hovered {
                hover_fill(th().ui.accent, th().ui.on_accent, 40)
            } else {
                th().ui.accent
            })
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD)
    } else if hovered {
        plain_button_style(true)
    } else {
        Style::default().fg(th().ui.muted)
    }
}

/// The status colors: added green, modified amber, deleted red,
/// renamed blue, conflicted red.
pub(crate) fn file_status_color(status: gitpoulpi_core::domain::FileStatus) -> Color {
    use gitpoulpi_core::domain::FileStatus as F;
    match status {
        F::Added | F::Untracked => th().refs.local,
        F::Modified | F::Staged => th().status.warning,
        F::Deleted | F::Conflicted => th().status.danger,
        F::Renamed => th().refs.remote,
    }
}

/// See `hide_in_graph_icon!` (defined at the top of `lib.rs`, where
/// `macro_rules!` has to be to reach `ref_context_actions`).
///
/// `\u{1F441}` (EYE) is a *text*-default emoji (Emoji_Presentation=No),
/// which this project otherwise avoids: WezTerm on Windows drew it
/// monochrome on a plain row but in color on the selected row, because the
/// selection's BOLD sent the lookup down the bold font's fallback chain.
/// The fix is at the cause: **the cell holding this glyph is never bold.**
/// The Refs lists' highlight carries no BOLD (the selected row bolds its
/// *name* span itself), and the context menu draws no highlight at all.
/// Pinned by `the_hide_eye_is_never_bold_even_on_the_selected_row`.
pub(crate) const HIDE_IN_GRAPH_ICON: &str = hide_in_graph_icon!();

/// What a ref row shows while it **is** hidden from the graph: the circled
/// division slash, a single well-supported BMP glyph that reads as "off".
/// Distinct on purpose from `HIDE_IN_GRAPH_ICON`, the *action* (hover
/// affordance and menu entry); an eye with a combining slash renders as
/// tofu in WezTerm.
pub(crate) const HIDDEN_IN_GRAPH_MARK: &str = "\u{2298}";

/// The Refs sidebar leaf row's "this is the checked-out branch" mark, in the
/// fixed gutter left of the name: CHECK MARK, the same glyph the Log's ref
/// pills use. Not BLACK CIRCLE: a wide-glyph shift in WezTerm was traced to
/// it. If the gutter ever shifts again, fix the alignment, not the glyph.
pub(crate) const CHECKED_OUT_MARK: &str = "\u{2713}";

/// The Refs sidebar's solo-mode gutter icon for "soloed" (or, on a folder
/// row, "every descendant soloed"): BULLSEYE, one of the three circle glyphs
/// measured in `graph/part.rs`'s font-coverage table, unlike FISHEYE. The
/// "not (fully) soloed" glyph next to it, `\u{25CB}` (WHITE CIRCLE), is
/// another of the measured three.
pub(crate) const SOLO_FULL_MARK: &str = "\u{25CE}";

pub(crate) const LOCAL_ICON: &str = "\u{1F4BB}"; // 💻 computer glyph - a
                                                 // computer icon for "local", not a
                                                 // branch glyph (redundant with the
                                                 // LOCAL section header/color already
                                                 // saying what kind of ref this is).
                                                 //
                                                 // `REMOTE_ICON`/`TAG_ICON` deliberately pick codepoints that are
                                                 // "Emoji" *default* presentation per Unicode's emoji-data (render in
                                                 // full color on their own), NOT "Text" default ones needing a trailing
                                                 // `\u{FE0F}` (VS16) selector to force color -- `\u{2601}` (☁, text-
                                                 // default) and `\u{1F3F7}` (🏷, text-default) were tried with VS16 first
                                                 // to fix low-contrast monochrome fallback, but on this WezTerm/font
                                                 // combination VS16 doesn't reliably merge into the preceding glyph: it
                                                 // paints as its own stray fallback glyph in whatever cell comes next,
                                                 // "un trou dans la colorisation" -- confirmed live (real screenshot)
                                                 // that widening the reserved column for it just gave that stray glyph
                                                 // a cell to render *into* instead of bleeding past, not a real fix.
                                                 // Picking emoji-default codepoints sidesteps the whole VS16 question.
pub(crate) const REMOTE_ICON: &str = "\u{1F310}"; // 🌐 globe - "remote/network", full color without VS16.
pub(crate) const TAG_ICON: &str = "\u{1F516}"; // 🔖 bookmark - "tag", full color without VS16.
                                               // Same emoji-default-presentation rule as above: both are
                                               // Emoji_Presentation=Yes, so neither needs (or tolerates) a VS16.
pub(crate) const WORKTREE_ICON: &str = "\u{1F33F}"; // 🌿 herb - a worktree is a branch checked out elsewhere.
pub(crate) const SUBMODULE_ICON: &str = "\u{1F9E9}"; // 🧩 puzzle piece - a repository nested in this one.
pub(crate) const STASH_ICON: &str = "\u{1F4E6}"; // 📦 package - "put away for later".
pub(crate) const PULL_REQUEST_ICON: &str = "\u{1F500}"; // shuffle arrows - Emoji_Presentation=Yes, no VS16 needed.
pub(crate) const ISSUE_ICON: &str = "\u{1F41B}"; // bug - Emoji_Presentation=Yes, no VS16 needed.

/// A `/`-derived intermediate folder row in `ui.ref_tree` mode (e.g. the
/// `feature` and `ui` rows above `feature/ui/login`), reflecting its own
/// fold state: closed while collapsed, open while expanded. Both
/// `\u{1F4C1}`/`\u{1F4C2}` are Emoji_Presentation=Yes and fully qualified,
/// so, like `REMOTE_ICON`/`TAG_ICON`, they need no VS16 and leave no stray
/// fallback glyph on a nearby cell. A remote root (`origin`, `gh`) keeps
/// `REMOTE_ICON`; only the folders underneath it (or a plain LOCAL/TAGS
/// folder) get this pair.
pub(crate) const FOLDER_CLOSED_ICON: &str = "\u{1F4C1}"; // 📁 file folder
pub(crate) const FOLDER_OPEN_ICON: &str = "\u{1F4C2}"; // 📂 open file folder

/// A lane color, blended toward the app's dark panel background, used as a
/// ref pill's own background chip: the raw `LANE_PALETTE` value stays
/// full-strength for graph lines/nodes, while a full-saturation block would
/// make the pill look like a block rather than a badge.
pub(crate) fn muted_pill_bg(color: Color) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };
    let toward = pill_blend_target();
    let blend = |c: u8, t: u8| ((u16::from(c) * 45 + u16::from(t) * 55) / 100) as u8;
    Color::Rgb(blend(r, toward.0), blend(g, toward.1), blend(b, toward.2))
}

/// An RGB color's channels (`None` for `Reset` and the palette colors,
/// whose real value the terminal decides).
pub(crate) fn rgb_of(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

/// Gives the cells of a side pane (Refs, the right panel) the theme's
/// `ui.side_bg`: every cell still on the theme's main background (the root
/// fill, or one a `Clear` left unpainted, which the end-of-frame pass would
/// give `ui.bg`) takes it; selection,
/// hover, input and button fills keep theirs. No-op while `side_bg` is
/// `default` or equals `bg`. The kitty placeholder cells keep their cell.
pub(crate) fn paint_side_bg(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect) {
    let t = th();
    if t.ui.side_bg == Color::Reset || t.ui.side_bg == t.ui.bg {
        return;
    }
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if (cell.bg == t.ui.bg || cell.bg == Color::Reset) && cell.symbol() != "\u{10EEEE}" {
                cell.bg = t.ui.side_bg;
            }
        }
    }
}

/// Gives the toolbar row `ui.toolbar_bg` (the selected repository tab uses the
/// same color, so it reads as attached): the cells still on the main
/// background take it, the button chips and hover fills keep theirs. No-op
/// while the role is `default` or equals `bg`. Color only, no layout.
pub(crate) fn paint_toolbar_bg(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect) {
    let t = th();
    if t.ui.toolbar_bg == Color::Reset || t.ui.toolbar_bg == t.ui.bg {
        return;
    }
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if cell.bg == t.ui.bg || cell.bg == Color::Reset {
                cell.bg = t.ui.toolbar_bg;
            }
        }
    }
}

/// The background of the selected repository tab: the toolbar's, falling back
/// to the selection color while `ui.toolbar_bg` is `default`.
pub(crate) fn selected_tab_bg() -> Color {
    let t = th();
    if t.ui.toolbar_bg == Color::Reset {
        t.ui.selection
    } else {
        t.ui.toolbar_bg
    }
}

/// Paints the status bar (footer) with `ui.statusbar_bg` / `ui.statusbar_fg`:
/// cells on the main background take the fill (an input keeps its own), and
/// text in the plain colors, or in one that would not read on the new fill
/// (below 3:1), takes `statusbar_fg`; warning / danger text that stays
/// readable keeps its color. `default` roles fall back to `side_bg` / `text`
/// and an unchanged fill leaves the bar alone.
pub(crate) fn paint_statusbar(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect) {
    use gitpoulpi_config::theme::{contrast_ratio, ThemeColor};
    let t = th();
    let fill = match t.ui.statusbar_bg {
        Color::Reset => t.ui.side_bg,
        c => c,
    };
    let fg = match t.ui.statusbar_fg {
        Color::Reset => t.ui.text,
        c => c,
    };
    if (fill == Color::Reset || fill == t.ui.bg) && t.ui.statusbar_fg == Color::Reset {
        return;
    }
    let readable = |c: Color| match (rgb_of(c), rgb_of(fill)) {
        (Some((r, g, b)), Some((fr, fg_, fb))) => {
            contrast_ratio(ThemeColor::Rgb(r, g, b), ThemeColor::Rgb(fr, fg_, fb))
                .is_none_or(|x| x >= 3.0)
        }
        _ => true,
    };
    let plain = [Color::Reset, t.ui.text, t.ui.muted, t.ui.accent];
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if cell.bg != t.ui.bg && cell.bg != Color::Reset {
                continue;
            }
            cell.bg = fill;
            if plain.contains(&cell.fg) || !readable(cell.fg) {
                cell.fg = fg;
            }
        }
    }
}

/// What tints (`graph::part::row_tint_bg`) are mixed into: `ui.blend_bg`.
/// Dark: the terminal background GitPoulpi assumes, `Rgb(29, 32, 34)`.
pub(crate) fn blend_rgb() -> (u8, u8, u8) {
    rgb_of(th().ui.blend_bg).unwrap_or((29, 32, 34))
}

/// What ref pills and the non-alpha graph picture are blended into: the
/// theme's own `bg` when it paints one, else (Dark, Terminal) the nominal
/// dark panel color `Rgb(34, 36, 46)`.
pub(crate) fn pill_blend_target() -> (u8, u8, u8) {
    rgb_of(th().ui.bg).unwrap_or((34, 36, 46))
}

/// The selection color baked into a non-alpha graph picture.
pub(crate) fn backdrop_selection() -> (u8, u8, u8) {
    rgb_of(th().ui.selection).unwrap_or((52, 58, 90))
}

/// `ui.button_hover` when hovered, else `ui.button`.
pub(crate) fn button_bg(hovered: bool) -> Color {
    if hovered {
        th().ui.button_hover
    } else {
        th().ui.button
    }
}

/// The one hover treatment for a "plain glyph" chrome button: at rest it is
/// *just the glyph*, in the app's muted chrome color, with no filled
/// background; hovered, it gets a filled background like a toolbar button's
/// hover state, so resting on it reads as "this does something" and resting
/// one cell away does not. Every plain-glyph button (the `+` Add remote, the
/// top-right `\u{D7}`) goes through this one function, so none can fill its
/// background at rest.
pub(crate) fn plain_button_style(hovered: bool) -> Style {
    if hovered {
        Style::default()
            .bg(th().ui.button_hover)
            .fg(th().ui.on_button)
    } else {
        Style::default().fg(th().ui.muted)
    }
}

/// Hover for a glyph button that carries its own color (the red trash, the
/// purple AI sparkle): it fills with that color and the glyph goes dark, so
/// the change is unmissable where `hover_style`'s subtle row tint was not.
/// A button with no color of its own gets [`plain_button_style`].
pub(crate) fn filled_hover_style(base: Style) -> Style {
    match base.fg {
        Some(fg) if fg != th().ui.muted => base
            .bg(fg)
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD),
        _ => plain_button_style(true),
    }
}

/// The hover fill of a button: `base` moved by `amt` away from its `label`
/// color (see `gitpoulpi_config::theme::hover_fill`), so hovering never
/// costs contrast. Non-RGB colors are returned unchanged.
pub(crate) fn hover_fill(base: Color, label: Color, amt: i16) -> Color {
    use gitpoulpi_config::theme::{hover_fill, ThemeColor};
    let tc = |c: Color| rgb_of(c).map(|(r, g, b)| ThemeColor::Rgb(r, g, b));
    match (tc(base), tc(label)) {
        (Some(b), Some(l)) => match hover_fill(b, l, amt) {
            ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
            _ => base,
        },
        _ => base,
    }
}

/// Lightens (or darkens, for a negative `amt`) an RGB color by `amt` per
/// channel, clamped: the hover affordance for a solid filled band (the
/// commit composer's Commit button, a modal's Confirm/Cancel row), which
/// cannot use `hover_style`'s background *lift* because it already is a
/// background.
pub(crate) fn lighten(color: Color, amt: i16) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let bump = |c: u8| (i16::from(c) + amt).clamp(0, 255) as u8;
            Color::Rgb(bump(r), bump(g), bump(b))
        }
        other => other,
    }
}

/// The one hover treatment for an element that already carries its own base
/// style (a menu row, a title-bar text button, a field label): lifts the
/// background by `HOVER_BG` without disturbing the foreground/modifiers the
/// caller computed, so hovering never fights selection/label styling. Every
/// hoverable element that is not a plain glyph button should go through
/// this rather than inventing its own `if hovered { ... }` bg.
pub(crate) fn hover_style(base: Style) -> Style {
    base.bg(th().ui.hover)
}

/// Picks black or white text for readable contrast against `bg`, via the
/// relative-luminance heuristic (perceptual weights: green reads brighter
/// than blue at the same numeric value).
pub(crate) fn contrasting_fg(bg: Color) -> Color {
    let t = th();
    // Dark theme: Black / White (`chrome.on_bright` / `ui.on_button`). A light
    // theme reads its own text color on a light chip.
    let on_light = if t.light {
        t.ui.text
    } else {
        t.chrome.on_bright
    };
    let on_dark = t.ui.on_button;
    let Color::Rgb(r, g, b) = bg else {
        return on_dark;
    };
    let luminance = 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
    if luminance > 140.0 {
        on_light
    } else {
        on_dark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In a real WezTerm window, a soloed checked-out branch drew
    /// `\u{2713}`/`\u{25C9}` wider than the single column the Refs sidebar's
    /// gutter math assumes, shifting the branch name over (and, stacked,
    /// corrupting the title-bar repaint next to it). Neither replacement glyph
    /// needs a VS16 selector, and both measure exactly one display cell, the
    /// same guarantee `graph/part.rs` measured for the Geometric Shapes trio.
    #[test]
    fn checked_out_and_solo_marks_are_one_cell_and_need_no_variation_selector() {
        for mark in [CHECKED_OUT_MARK, SOLO_FULL_MARK] {
            assert!(!mark.contains('\u{FE0F}'), "{mark:?} should need no VS16");
            assert!(!mark.contains('\u{FE0E}'), "{mark:?} should need no VS15");
            assert_eq!(display_width(mark), 1, "{mark:?} should measure 1 cell");
        }
        // Pin the exact codepoints so an edit cannot silently bring back the
        // wide glyphs this test keeps out.
        assert_eq!(
            CHECKED_OUT_MARK, "\u{2713}",
            "the Log pills' CHECK MARK, kept for uniformity (maintainer request)"
        );
        assert_eq!(SOLO_FULL_MARK, "\u{25CE}", "not \\u{{25C9}} FISHEYE");
        assert_ne!(CHECKED_OUT_MARK, SOLO_FULL_MARK);
    }

    /// The REMOTE section's `+` and the top-right `\u{D7}` must not have a
    /// filled background when NOT hovered. Both go through
    /// `plain_button_style`, so pinning its contract protects every button
    /// built on it, including ones (like the Refs title's Show All/Stop
    /// Soloing button, which uses `hover_style` because it was never filled
    /// at rest) that do not call it directly.
    ///
    /// Red if `plain_button_style` always sets a background.
    #[test]
    fn plain_button_style_has_no_background_at_rest_and_one_on_hover() {
        assert_eq!(plain_button_style(false).bg, None, "plain at rest");
        assert_eq!(
            plain_button_style(true).bg,
            Some(th().ui.button_hover),
            "filled on hover"
        );
        assert_ne!(
            plain_button_style(false),
            plain_button_style(true),
            "hovering a plain-glyph button must change its rendered style"
        );
    }

    /// `hover_style` is the other half of the shared hover style: for an
    /// element that already carries its own foreground, it lifts the
    /// background without touching what the caller set.
    #[test]
    fn hover_style_lifts_the_background_without_touching_the_foreground() {
        let base = Style::default().fg(th().status.warning);
        let hovered = hover_style(base);
        assert_eq!(
            hovered.fg,
            Some(th().status.warning),
            "foreground must survive"
        );
        assert_eq!(hovered.bg, Some(th().ui.hover));
        assert_ne!(base, hovered);
    }

    /// `lighten` is the third hover idiom (a solid filled band, like the
    /// commit composer's Commit button and the Confirm/Cancel dialog), for
    /// elements with no background to lift.
    #[test]
    fn lighten_moves_every_channel_toward_white_and_clamps() {
        assert_eq!(lighten(Color::Rgb(10, 20, 30), 5), Color::Rgb(15, 25, 35));
        assert_eq!(lighten(Color::Rgb(250, 0, 0), 20), Color::Rgb(255, 20, 20));
        assert_eq!(lighten(Color::Rgb(10, 10, 10), -20), Color::Rgb(0, 0, 0));
        // A non-RGB color (never used by this app's palette, but the
        // function must not panic on it) passes through unchanged.
        assert_eq!(lighten(Color::Reset, 10), Color::Reset);
    }

    #[test]
    fn hex_colors_parse_strictly() {
        assert_eq!(parse_hex_color("#2a2a2e"), Some((42, 42, 46)));
        assert_eq!(parse_hex_color(" #FFA000 "), Some((255, 160, 0)));
        for bad in [
            "", "2a2a2e", "#fff", "#12345", "#1234567", "#gggggg", "#+12345",
        ] {
            assert_eq!(parse_hex_color(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn presets_map_and_unreadable_values_fall_back_to_gray() {
        let gray = resolve_input_color("gray", "");
        assert_eq!(resolve_input_color("blue", ""), (28, 34, 52));
        assert_eq!(resolve_input_color("Slate", ""), (34, 42, 50));
        assert_eq!(resolve_input_color("custom", "#102030"), (16, 32, 48));
        assert_eq!(resolve_input_color("custom", "nope"), gray);
        assert_eq!(resolve_input_color("mauve", ""), gray);
        for name in INPUT_PRESETS
            .iter()
            .filter(|n| !matches!(**n, "custom" | "theme"))
        {
            assert!(input_preset_rgb(name).is_some(), "{name}");
        }
        // Distinct from the row highlights.
        assert_ne!(Color::Rgb(gray.0, gray.1, gray.2), th().ui.selection);
        assert_ne!(Color::Rgb(gray.0, gray.1, gray.2), th().ui.hover);
    }
}
