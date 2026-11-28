//! Builds the [`Skin`] from a theme spec.

use std::sync::Arc;

use gitpoulpi_config::theme::{self, ThemeColor, ThemeSpec};
use gpui_kit::Hsla;

use super::{hsla_of, Skin};

pub(super) fn build(spec: &ThemeSpec, fallback: Hsla, lane_choice: Option<&str>) -> Skin {
    let c = |t: ThemeColor| hsla_of(t, fallback);
    let palette: Vec<ThemeColor> = match lane_choice.and_then(theme::lane_palette) {
        Some(p) => p.colors,
        None if spec.graph_palette.is_empty() => theme::lane_palette("gitpoulpi")
            .map(|p| p.colors)
            .unwrap_or_default(),
        None => spec.graph_palette.clone(),
    };
    let lanes: Vec<Hsla> = palette.iter().map(|t| c(*t)).collect();
    let on_light = if spec.light {
        c(spec.ui.text)
    } else {
        c(spec.chrome.on_bright)
    };
    Skin {
        lanes: Arc::new(lanes),
        wip: c(spec.graph.wip),
        stash: c(spec.graph.stash),
        local: c(spec.refs.local),
        remote: c(spec.refs.remote),
        tag: c(spec.refs.tag),
        bg: c(spec.effective_bg()),
        side_bg: c(spec.effective_side_bg()),
        text: c(spec.ui.text),
        muted: c(spec.ui.muted),
        border: c(spec.ui.border),
        border_focused: c(spec.ui.border_focused),
        selection: c(spec.ui.selection),
        hover: c(spec.ui.hover),
        accent: c(spec.ui.accent),
        success: c(spec.status.success),
        warning: c(spec.status.warning),
        danger: c(spec.status.danger),
        info: c(spec.status.info),
        on_light,
        on_dark: c(spec.ui.on_button),
        on_button: c(spec.ui.on_button),
        header_fg: c(spec.ui.header_fg),
        input_bg: c(spec.ui.input_rest),
        tb_bg: c(spec.chrome.toolbar_bg),
        tb_bg_hover: c(spec.chrome.toolbar_bg_hover),
        tb_fg: c(spec.chrome.toolbar_fg),
        tb_disabled_bg: c(spec.chrome.disabled_button_bg),
        tb_disabled_fg: c(spec.chrome.toolbar_disabled_fg),
        toolbar: [
            c(spec.toolbar.undo),
            c(spec.toolbar.pull),
            c(spec.toolbar.push),
            c(spec.toolbar.branch),
            c(spec.toolbar.tag),
            c(spec.toolbar.stash),
            c(spec.toolbar.lfs),
            c(spec.toolbar.search),
        ],
        rgb_bg: spec.effective_bg().approx_rgb().unwrap_or((34, 36, 46)),
    }
}
