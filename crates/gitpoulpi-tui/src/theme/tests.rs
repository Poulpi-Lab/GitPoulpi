//! Parity: the Dark theme is exactly the palette the UI painted before
//! themes existed, and the legacy role constants still equal it.

use super::*;
use ratatui::style::Color;

#[test]
fn dark_theme_is_exactly_the_old_palette() {
    let t = Theme::dark();
    let table: &[(&str, Color, Color)] = &[
        ("ui.bg", t.ui.bg, Color::Reset),
        ("ui.blend_bg", t.ui.blend_bg, Color::Rgb(29, 32, 34)),
        ("ui.panel", t.ui.panel, Color::Rgb(28, 30, 46)),
        ("ui.side_bg", t.ui.side_bg, Color::Reset),
        ("ui.toolbar_bg", t.ui.toolbar_bg, Color::Reset),
        ("ui.statusbar_bg", t.ui.statusbar_bg, Color::Reset),
        ("ui.statusbar_fg", t.ui.statusbar_fg, Color::Reset),
        ("ui.text", t.ui.text, Color::Rgb(226, 228, 235)),
        ("ui.muted", t.ui.muted, Color::Rgb(150, 154, 172)),
        ("ui.disabled", t.ui.disabled, Color::Rgb(136, 140, 158)),
        ("ui.accent", t.ui.accent, Color::Rgb(64, 156, 255)),
        ("ui.on_accent", t.ui.on_accent, Color::Rgb(16, 18, 28)),
        ("ui.selection", t.ui.selection, Color::Rgb(52, 58, 90)),
        ("ui.hover", t.ui.hover, Color::Rgb(34, 38, 56)),
        ("ui.button", t.ui.button, Color::Rgb(70, 75, 105)),
        (
            "ui.button_hover",
            t.ui.button_hover,
            Color::Rgb(88, 95, 130),
        ),
        ("ui.on_button", t.ui.on_button, Color::White),
        (
            "ui.border_focused",
            t.ui.border_focused,
            Color::Rgb(64, 156, 255),
        ),
        ("ui.border", t.ui.border, Color::Rgb(150, 154, 172)),
        ("ui.tooltip_bg", t.ui.tooltip_bg, Color::Rgb(28, 30, 46)),
        ("ui.popup_bg", t.ui.popup_bg, Color::Rgb(28, 30, 46)),
        ("ui.header_fg", t.ui.header_fg, Color::Rgb(226, 228, 235)),
        ("ui.input_rest", t.ui.input_rest, Color::Rgb(42, 42, 46)),
        ("ui.input_hover", t.ui.input_hover, Color::Rgb(52, 52, 56)),
        ("ui.input_focus", t.ui.input_focus, Color::Rgb(60, 60, 64)),
        (
            "ui.text_selection_bg",
            t.ui.text_selection_bg,
            Color::Rgb(64, 156, 255),
        ),
        (
            "ui.text_selection_fg",
            t.ui.text_selection_fg,
            Color::Rgb(16, 18, 28),
        ),
        ("status.success", t.status.success, Color::Rgb(54, 179, 126)),
        ("status.warning", t.status.warning, Color::Rgb(255, 171, 0)),
        ("status.danger", t.status.danger, Color::Rgb(255, 99, 99)),
        ("status.info", t.status.info, Color::Rgb(64, 156, 255)),
        ("status.ai", t.status.ai, Color::Rgb(186, 134, 255)),
        ("refs.local", t.refs.local, Color::Rgb(54, 179, 126)),
        ("refs.remote", t.refs.remote, Color::Rgb(64, 156, 255)),
        ("refs.tag", t.refs.tag, Color::Rgb(255, 171, 0)),
        ("scrollbar.track", t.scrollbar.track, Color::Rgb(40, 42, 60)),
        (
            "scrollbar.thumb",
            t.scrollbar.thumb,
            Color::Rgb(90, 96, 130),
        ),
        (
            "scrollbar.thumb_hover",
            t.scrollbar.thumb_hover,
            Color::Rgb(64, 156, 255),
        ),
        (
            "scrollbar.marks_add",
            t.scrollbar.marks_add,
            Color::Rgb(80, 200, 120),
        ),
        (
            "scrollbar.marks_del",
            t.scrollbar.marks_del,
            Color::Rgb(230, 90, 90),
        ),
        (
            "scrollbar.marks_cursor",
            t.scrollbar.marks_cursor,
            Color::Rgb(240, 240, 250),
        ),
        (
            "scrollbar.marks_occurrence",
            t.scrollbar.marks_occurrence,
            Color::Rgb(240, 200, 90),
        ),
        (
            "scrollbar.marks_modified",
            t.scrollbar.marks_modified,
            Color::Rgb(220, 170, 90),
        ),
        (
            "scrollbar.marks_conflict_open",
            t.scrollbar.marks_conflict_open,
            Color::Rgb(240, 150, 60),
        ),
        (
            "scrollbar.marks_conflict_current",
            t.scrollbar.marks_conflict_current,
            Color::Rgb(120, 180, 255),
        ),
        ("minimap.bg", t.minimap.bg, Color::Rgb(24, 26, 38)),
        ("minimap.view", t.minimap.view, Color::Rgb(56, 62, 92)),
        ("minimap.ink", t.minimap.ink, Color::Rgb(150, 156, 184)),
        ("diff.add_fg", t.diff.add_fg, Color::Rgb(126, 211, 158)),
        ("diff.del_fg", t.diff.del_fg, Color::Rgb(240, 138, 138)),
        ("diff.add_bg", t.diff.add_bg, Color::Rgb(24, 52, 38)),
        ("diff.del_bg", t.diff.del_bg, Color::Rgb(58, 30, 34)),
        ("diff.hunk_fg", t.diff.hunk_fg, Color::Rgb(140, 160, 210)),
        (
            "diff.gutter_fg",
            t.diff.gutter_fg,
            Color::Rgb(108, 114, 140),
        ),
        ("diff.range_bg", t.diff.range_bg, Color::Rgb(40, 45, 70)),
        (
            "diff.search_hit_bg",
            t.diff.search_hit_bg,
            Color::Rgb(92, 74, 18),
        ),
        ("merge.ours", t.merge.ours, Color::Rgb(110, 190, 255)),
        ("merge.theirs", t.merge.theirs, Color::Rgb(240, 170, 90)),
        ("merge.base", t.merge.base, Color::Rgb(160, 165, 185)),
        ("merge.edit", t.merge.edit, Color::Rgb(255, 196, 64)),
        ("merge.text", t.merge.text, Color::Rgb(192, 197, 206)),
        ("merge.ours_tint", t.merge.ours_tint, Color::Rgb(24, 40, 60)),
        (
            "merge.theirs_tint",
            t.merge.theirs_tint,
            Color::Rgb(56, 42, 24),
        ),
        ("merge.ours_pick", t.merge.ours_pick, Color::Rgb(28, 54, 84)),
        (
            "merge.theirs_pick",
            t.merge.theirs_pick,
            Color::Rgb(76, 54, 26),
        ),
        ("merge.base_pick", t.merge.base_pick, Color::Rgb(44, 46, 60)),
        (
            "merge.current_region",
            t.merge.current_region,
            Color::Rgb(26, 30, 46),
        ),
        ("merge.strip_bg", t.merge.strip_bg, Color::Rgb(56, 44, 20)),
        ("merge.strip_fg", t.merge.strip_fg, Color::Rgb(255, 196, 64)),
        ("editor.text", t.editor.text, Color::Rgb(192, 197, 206)),
        (
            "editor.current_line",
            t.editor.current_line,
            Color::Rgb(30, 34, 50),
        ),
        (
            "editor.find_hit",
            t.editor.find_hit,
            Color::Rgb(120, 100, 40),
        ),
        (
            "editor.find_hit_fg",
            t.editor.find_hit_fg,
            Color::Rgb(255, 255, 255),
        ),
        ("toolbar.pull", t.toolbar.pull, Color::Rgb(150, 195, 255)),
        ("toolbar.push", t.toolbar.push, Color::Rgb(150, 210, 180)),
        ("toolbar.stash", t.toolbar.stash, Color::Rgb(255, 205, 130)),
        (
            "toolbar.branch",
            t.toolbar.branch,
            Color::Rgb(175, 160, 220),
        ),
        ("toolbar.lfs", t.toolbar.lfs, Color::Rgb(185, 215, 150)),
        ("toolbar.tag", t.toolbar.tag, Color::Rgb(230, 190, 130)),
        (
            "toolbar.search",
            t.toolbar.search,
            Color::Rgb(200, 200, 215),
        ),
        ("toolbar.undo", t.toolbar.undo, Color::Rgb(140, 205, 225)),
        (
            "hosting.merged",
            t.hosting.merged,
            Color::Rgb(163, 113, 247),
        ),
        ("hosting.chip_fg", t.hosting.chip_fg, Color::Rgb(20, 22, 30)),
        ("chrome.stat_add", t.chrome.stat_add, Color::Green),
        ("chrome.stat_del", t.chrome.stat_del, Color::Red),
        ("chrome.notice", t.chrome.notice, Color::Red),
        ("chrome.marker", t.chrome.marker, Color::Magenta),
        ("chrome.on_bright", t.chrome.on_bright, Color::Black),
        ("chrome.wip_del", t.chrome.wip_del, Color::Rgb(255, 92, 141)),
        ("chrome.error_bg", t.chrome.error_bg, Color::Rgb(70, 24, 30)),
        (
            "chrome.banner_bg",
            t.chrome.banner_bg,
            Color::Rgb(56, 44, 20),
        ),
        (
            "chrome.banner_danger_bg",
            t.chrome.banner_danger_bg,
            Color::Rgb(120, 40, 40),
        ),
        (
            "chrome.toolbar_arrow_bg",
            t.chrome.toolbar_arrow_bg,
            Color::Rgb(58, 62, 90),
        ),
        (
            "chrome.pill_branch",
            t.chrome.pill_branch,
            Color::Rgb(90, 40, 120),
        ),
        (
            "chrome.pill_onto",
            t.chrome.pill_onto,
            Color::Rgb(20, 90, 100),
        ),
        (
            "chrome.action_edit",
            t.chrome.action_edit,
            Color::Rgb(120, 170, 255),
        ),
        (
            "chrome.dropdown_bg",
            t.chrome.dropdown_bg,
            Color::Rgb(30, 34, 52),
        ),
        (
            "chrome.dropdown_hit",
            t.chrome.dropdown_hit,
            Color::Rgb(150, 200, 255),
        ),
        (
            "chrome.action_reset",
            t.chrome.action_reset,
            Color::Rgb(90, 140, 255),
        ),
        (
            "chrome.tint_reset",
            t.chrome.tint_reset,
            Color::Rgb(28, 40, 78),
        ),
        (
            "chrome.tint_danger",
            t.chrome.tint_danger,
            Color::Rgb(56, 30, 40),
        ),
        (
            "chrome.tint_success",
            t.chrome.tint_success,
            Color::Rgb(24, 52, 40),
        ),
        (
            "chrome.toast_bg",
            t.chrome.toast_bg,
            Color::Rgb(70, 50, 110),
        ),
        (
            "chrome.proposal_bg",
            t.chrome.proposal_bg,
            Color::Rgb(110, 60, 170),
        ),
        (
            "chrome.confirm_danger",
            t.chrome.confirm_danger,
            Color::Rgb(165, 45, 45),
        ),
        (
            "chrome.disabled_button_bg",
            t.chrome.disabled_button_bg,
            Color::Rgb(40, 44, 58),
        ),
        (
            "chrome.toolbar_bg",
            t.chrome.toolbar_bg,
            Color::Rgb(70, 75, 105),
        ),
        (
            "chrome.toolbar_bg_hover",
            t.chrome.toolbar_bg_hover,
            Color::Rgb(88, 95, 130),
        ),
        ("chrome.toolbar_fg", t.chrome.toolbar_fg, Color::White),
        (
            "chrome.toolbar_disabled_fg",
            t.chrome.toolbar_disabled_fg,
            Color::Rgb(150, 154, 172),
        ),
        ("graph.wip", t.graph.wip, Color::Rgb(150, 154, 172)),
        ("graph.stash", t.graph.stash, Color::Rgb(150, 154, 172)),
    ];
    for (name, actual, expected) in table {
        assert_eq!(actual, expected, "{name}");
    }
    assert_eq!(table.len(), 112, "every role is in the table");
    let lanes: Vec<Color> = [
        (64, 156, 255),
        (54, 179, 126),
        (255, 171, 0),
        (101, 84, 192),
        (255, 92, 141),
        (0, 184, 217),
        (255, 139, 0),
        (107, 119, 140),
    ]
    .iter()
    .map(|&(r, g, b)| Color::Rgb(r, g, b))
    .collect();
    assert_eq!(t.graph.palette, lanes.as_slice());
    assert_eq!(t.syntax, "base16-ocean.dark");
    assert!(!t.light);
}

#[test]
fn the_role_constants_equal_the_dark_theme() {
    let t = Theme::dark();
    assert_eq!(th().ui.muted, t.ui.muted);
    assert_eq!(th().ui.text, t.ui.text);
    assert_eq!(th().ui.accent, t.ui.accent);
    assert_eq!(th().status.warning, t.status.warning);
    assert_eq!(th().status.danger, t.status.danger);
    assert_eq!(th().status.ai, t.status.ai);
    assert_eq!(th().ui.selection, t.ui.selection);
    assert_eq!(th().ui.hover, t.ui.hover);
    assert_eq!(th().refs.local, t.refs.local);
    assert_eq!(th().refs.remote, t.refs.remote);
    assert_eq!(th().refs.tag, t.refs.tag);
    assert_eq!(th().ui.button, t.ui.button);
    assert_eq!(th().ui.button_hover, t.ui.button_hover);
    assert_eq!(th().ui.panel, t.ui.panel);
    assert_eq!(th().ui.button_hover, t.ui.button_hover);
}

#[test]
fn the_themes_input_colors_are_the_legacy_lighten_steps() {
    let t = Theme::dark();
    assert_eq!(lighten(t.ui.input_rest, 18), t.ui.input_focus);
    assert_eq!(lighten(t.ui.input_rest, 10), t.ui.input_hover);
    assert_eq!(Some((42, 42, 46)), input_preset_rgb("gray"));
}

#[test]
fn override_theme_is_per_thread_and_restores_on_drop() {
    let mut other = Theme::dark();
    other.ui.accent = Color::Rgb(1, 2, 3);
    let before = th().ui.accent;
    let gen_before = theme_generation();
    {
        let _g = override_theme(&other);
        assert_eq!(th().ui.accent, Color::Rgb(1, 2, 3));
        assert_ne!(theme_generation(), gen_before);
        std::thread::spawn(move || assert_eq!(th().ui.accent, before))
            .join()
            .unwrap();
    }
    assert_eq!(th().ui.accent, before);
    assert_eq!(theme_generation(), gen_before);
}

#[test]
fn themes_resolve_by_slug_are_interned_and_fall_back_to_dark() {
    use runtime::resolve_theme;
    let light = resolve_theme("light", &[]);
    assert!(light.light);
    assert!(std::ptr::eq(light, resolve_theme("light", &[])), "interned");
    // A config naming a theme that does not exist (a removed one, a typo)
    // falls back to Dark.
    for unknown in [
        "system",
        "nope",
        "",
        "poulpi-big-bro",
        "poulpi-big-bro-light",
    ] {
        assert_eq!(resolve_theme(unknown, &[]).slug, "dark", "{unknown:?}");
    }
    let mut mine = gitpoulpi_config::theme::preset("dark").unwrap();
    mine.slug = "mine".into();
    mine.ui.accent = gitpoulpi_config::theme::ThemeColor::Rgb(9, 8, 7);
    let t = resolve_theme("mine", &[mine]);
    assert_eq!(t.ui.accent, Color::Rgb(9, 8, 7));
}

#[test]
fn paint_side_bg_recolors_only_cells_on_the_main_background() {
    use ratatui::{buffer::Buffer, layout::Rect};
    let spec = gitpoulpi_config::theme::preset("catppuccin-mocha").unwrap();
    let theme = Theme::from(&spec);
    let _guard = override_theme(&theme);
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 1));
    for x in 0..4 {
        buf[(x, 0)].bg = theme.ui.bg;
    }
    buf[(1, 0)].bg = theme.ui.selection;
    paint_side_bg(&mut buf, Rect::new(0, 0, 3, 1));
    assert_eq!(buf[(0, 0)].bg, theme.ui.side_bg);
    assert_eq!(
        buf[(1, 0)].bg,
        theme.ui.selection,
        "selection keeps its fill"
    );
    assert_eq!(buf[(2, 0)].bg, theme.ui.side_bg);
    assert_eq!(buf[(3, 0)].bg, theme.ui.bg, "outside the area");
    assert_ne!(theme.ui.side_bg, theme.ui.bg);
}

fn buf_on(bg: Color, fg: Color, w: u16) -> ratatui::buffer::Buffer {
    use ratatui::{buffer::Buffer, layout::Rect};
    let mut buf = Buffer::empty(Rect::new(0, 0, w, 2));
    for y in 0..2 {
        for x in 0..w {
            buf[(x, y)].bg = bg;
            buf[(x, y)].fg = fg;
        }
    }
    buf
}

#[test]
fn paint_toolbar_bg_recolors_only_main_background_cells_inside_the_area() {
    use ratatui::layout::Rect;
    let spec = gitpoulpi_config::theme::preset("catppuccin-mocha").unwrap();
    let theme = Theme::from(&spec);
    let _guard = override_theme(&theme);
    let mut buf = buf_on(theme.ui.bg, theme.ui.text, 4);
    buf[(1, 0)].bg = theme.chrome.toolbar_bg;
    paint_toolbar_bg(&mut buf, Rect::new(0, 0, 3, 1));
    assert_eq!(buf[(0, 0)].bg, theme.ui.toolbar_bg);
    assert_eq!(
        buf[(1, 0)].bg,
        theme.chrome.toolbar_bg,
        "a button chip keeps its fill"
    );
    assert_eq!(buf[(3, 0)].bg, theme.ui.bg, "outside the area");
    assert_eq!(buf[(0, 1)].bg, theme.ui.bg, "other row untouched");
    assert_ne!(theme.ui.toolbar_bg, theme.ui.bg);
}

#[test]
fn paint_statusbar_uses_the_status_colors_and_keeps_readable_accents() {
    use ratatui::layout::Rect;
    let spec = gitpoulpi_config::theme::preset("vscode-dark").unwrap();
    let theme = Theme::from(&spec);
    let _guard = override_theme(&theme);
    let mut buf = buf_on(theme.ui.bg, theme.ui.muted, 4);
    // Unreadable on the blue bar: remapped. Readable: kept.
    buf[(1, 0)].fg = theme.status.warning;
    buf[(2, 0)].fg = Color::Rgb(255, 255, 255);
    buf[(3, 0)].bg = theme.ui.input_focus;
    paint_statusbar(&mut buf, Rect::new(0, 0, 4, 1));
    assert_eq!(buf[(0, 0)].bg, Color::Rgb(0, 0x7a, 0xcc));
    assert_eq!(
        buf[(0, 0)].fg,
        theme.ui.statusbar_fg,
        "muted text takes the status color"
    );
    assert_eq!(
        buf[(1, 0)].fg,
        theme.ui.statusbar_fg,
        "unreadable warning remapped"
    );
    assert_eq!(
        buf[(3, 0)].bg,
        theme.ui.input_focus,
        "an input keeps its fill"
    );
    assert_eq!(buf[(0, 1)].bg, theme.ui.bg, "other row untouched");
}

#[test]
fn paint_chrome_is_a_no_op_while_the_roles_are_default() {
    use ratatui::layout::Rect;
    let theme = Theme::dark();
    let _guard = override_theme(&theme);
    let mut buf = buf_on(Color::Reset, Color::Reset, 3);
    let before = buf.clone();
    paint_toolbar_bg(&mut buf, Rect::new(0, 0, 3, 2));
    paint_statusbar(&mut buf, Rect::new(0, 0, 3, 2));
    assert_eq!(buf, before);
}
