//! Maps a GitPoulpi theme onto the kit's own `Theme` (colours, fonts, diff syntax colours).

use std::sync::Arc;

use gitpoulpi_config::theme::{ThemeColor, ThemeSpec};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{hsla, App, Hsla};

use super::{hex, hsla_of, lift};
use crate::fonts;

pub(super) fn apply_kit_colors(spec: &ThemeSpec, fallback: Hsla, cx: &mut App) {
    let dark = !spec.light;
    Theme::change(
        if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );
    let c = |t: ThemeColor| hsla_of(t, fallback);

    let bg = c(spec.effective_bg());
    let side = c(spec.effective_side_bg());
    let panel = c(spec.ui.panel);
    let text = c(spec.ui.text);
    let muted = c(spec.ui.muted);
    let border = c(spec.ui.border);
    let accent = c(spec.ui.accent);
    let on_accent = c(spec.ui.on_accent);
    let selection = c(spec.ui.selection);
    let hover = c(spec.ui.hover);
    let success = c(spec.status.success);
    let warning = c(spec.status.warning);
    let danger = c(spec.status.danger);
    let info = c(spec.status.info);
    let toolbar = c(spec.effective_toolbar_bg());
    let popup = c(spec.ui.popup_bg);
    let button = c(spec.ui.button);
    let button_hover = c(spec.ui.button_hover);
    let on_button = c(spec.ui.on_button);
    let transparent = hsla(0., 0., 0., 0.);

    Theme::update(cx, |t| {
        t.font_family = fonts::UI_FAMILY.into();
        t.mono_font_family = fonts::MONO_FAMILY.into();
        t.font_size = gpui_kit::px(13.);
        t.mono_font_size = gpui_kit::px(12.);
        t.radius = gpui_kit::px(5.);
        t.colors.background = bg;
        t.colors.foreground = text;
        t.colors.muted = panel;
        t.colors.muted_foreground = muted;
        t.colors.border = border;
        t.colors.input = c(spec.ui.input_rest);
        t.colors.accent = hover;
        t.colors.accent_foreground = text;
        t.colors.primary = accent;
        t.colors.primary_foreground = on_accent;
        t.colors.primary_hover = lift(accent, 0.06, dark);
        t.colors.primary_active = lift(accent, 0.12, dark);
        t.colors.secondary = panel;
        t.colors.secondary_foreground = text;
        t.colors.secondary_hover = hover;
        t.colors.secondary_active = selection;
        t.colors.button = button;
        t.colors.button_hover = button_hover;
        t.colors.button_foreground = on_button;
        t.colors.ring = c(spec.ui.border_focused);
        t.colors.caret = text;
        t.colors.selection = c(spec.ui.text_selection_bg);
        t.colors.link = accent;
        t.colors.link_hover = lift(accent, 0.08, dark);
        t.colors.link_active = lift(accent, 0.14, dark);
        t.colors.popover = popup;
        t.colors.popover_foreground = text;
        t.colors.group_box = panel;
        t.colors.group_box_foreground = text;
        t.colors.overlay = hsla(0., 0., 0., if dark { 0.5 } else { 0.28 });
        t.colors.drag_border = accent;
        t.colors.drop_target = selection;

        t.colors.sidebar = side;
        t.colors.sidebar_foreground = text;
        t.colors.sidebar_border = border;
        t.colors.sidebar_accent = hover;
        t.colors.sidebar_accent_foreground = text;
        t.colors.sidebar_primary = accent;
        t.colors.sidebar_primary_foreground = on_accent;

        t.colors.list = bg;
        t.colors.list_even = bg;
        t.colors.list_head = panel;
        t.colors.list_active = selection;
        t.colors.list_active_border = accent;
        t.colors.list_hover = hover;
        t.colors.table = bg;
        t.colors.table_even = bg;
        t.colors.table_head = panel;
        t.colors.table_head_foreground = muted;
        t.colors.table_foot = panel;
        t.colors.table_foot_foreground = muted;
        t.colors.table_active = selection;
        t.colors.table_active_border = accent;
        t.colors.table_hover = hover;
        t.colors.table_row_border = transparent;

        t.colors.title_bar = toolbar;
        t.colors.title_bar_border = border;
        t.colors.status_bar = c(spec.ui.statusbar_bg);
        t.colors.status_bar_border = border;
        t.colors.tab = panel;
        t.colors.tab_bar = panel;
        t.colors.tab_bar_segmented = panel;
        t.colors.tab_active = bg;
        t.colors.tab_active_foreground = text;
        t.colors.tab_foreground = muted;
        t.colors.window_border = border;

        t.colors.scrollbar = transparent;
        t.colors.scrollbar_thumb = c(spec.scrollbar.thumb);
        t.colors.scrollbar_thumb_hover = c(spec.scrollbar.thumb_hover);

        t.colors.success = success;
        t.colors.success_hover = lift(success, 0.05, dark);
        t.colors.success_active = lift(success, 0.1, dark);
        t.colors.success_foreground = c(spec.chrome.on_bright);
        t.colors.warning = warning;
        t.colors.warning_hover = lift(warning, 0.05, dark);
        t.colors.warning_active = lift(warning, 0.1, dark);
        t.colors.warning_foreground = c(spec.chrome.on_bright);
        t.colors.danger = danger;
        t.colors.danger_hover = lift(danger, 0.05, dark);
        t.colors.danger_active = lift(danger, 0.1, dark);
        t.colors.danger_foreground = c(spec.chrome.on_bright);
        t.colors.info = info;
        t.colors.info_hover = lift(info, 0.05, dark);
        t.colors.info_active = lift(info, 0.1, dark);
        t.colors.info_foreground = c(spec.chrome.on_bright);
        t.colors.red = danger;
        t.colors.green = success;
        t.colors.blue = info;
        t.colors.yellow = warning;
        t.colors.magenta = c(spec.status.ai);
        t.colors.cyan = c(spec.refs.remote);
    });

    // Syntax colours for the diff: GitPoulpi themes carry a syntect theme *name*, not colours, so the
    // roles are derived from the palette the theme does define.
    let style = |col: Hsla| serde_json::json!({ "color": hex(col) });
    let json = serde_json::json!({
        "name": spec.name,
        "appearance": if dark { "dark" } else { "light" },
        "style": {
            "editor.background": hex(bg),
            "editor.foreground": hex(text),
            "syntax": {
                "keyword": style(c(spec.status.ai)),
                "string": style(success),
                "string.escape": style(warning),
                "number": style(warning),
                "boolean": style(warning),
                "constant": style(warning),
                "function": style(info),
                "type": style(c(spec.refs.tag)),
                "constructor": style(c(spec.refs.tag)),
                "comment": style(muted),
                "comment.doc": style(muted),
                "operator": style(c(spec.refs.remote)),
                "property": style(info),
                "attribute": style(c(spec.refs.local)),
                "variable": style(text),
                "tag": style(c(spec.status.ai)),
                "title": style(accent),
                "link_uri": style(accent),
            }
        }
    });
    if let Ok(h) = serde_json::from_value::<gpui_kit::component::highlighter::HighlightTheme>(json)
    {
        Theme::update(cx, |t| t.highlight_theme = Arc::new(h));
    }
}
