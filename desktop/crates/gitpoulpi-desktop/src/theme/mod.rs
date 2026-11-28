//! GitPoulpi `ThemeSpec` -> GPUI Kit `Theme` plus the app's own [`Skin`].
//!
//! The kit has its own semantic palette (`ThemeColor`, ~200 fields). [`apply`] switches the kit's
//! light/dark mode first (that loads the kit's own colours), then overwrites the roles that have an
//! obvious GitPoulpi counterpart ([`kit`]). What the kit has no word for (graph lanes, ref colours,
//! toolbar chips) is kept in the [`Skin`] global ([`skin`]).

mod kit;
mod skin;

use std::sync::Arc;

use gitpoulpi_config::theme::{self, ThemeColor, ThemeSpec};
use gpui_kit::{hsla, rgb, App, Global, Hsla};

/// Slug of the theme used when the preferences name none (or an unknown one).
pub const DEFAULT_THEME: &str = "dark";

/// Everything the GitPoulpi-specific widgets (graph, pills, toolbar, refs) read from the theme.
#[derive(Clone)]
pub struct Skin {
    pub lanes: Arc<Vec<Hsla>>,
    pub wip: Hsla,
    pub stash: Hsla,
    pub local: Hsla,
    pub remote: Hsla,
    pub tag: Hsla,
    pub bg: Hsla,
    pub side_bg: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub border: Hsla,
    pub border_focused: Hsla,
    pub selection: Hsla,
    pub hover: Hsla,
    pub accent: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
    pub info: Hsla,
    pub on_light: Hsla,
    pub on_dark: Hsla,
    pub on_button: Hsla,
    pub header_fg: Hsla,
    pub input_bg: Hsla,
    pub tb_bg: Hsla,
    pub tb_bg_hover: Hsla,
    pub tb_fg: Hsla,
    pub tb_disabled_bg: Hsla,
    pub tb_disabled_fg: Hsla,
    /// undo, pull, push, branch, tag, stash, lfs, search
    pub toolbar: [Hsla; 8],
    pub rgb_bg: (u8, u8, u8),
    /// What the diff pane paints with (the theme's `diff` colours).
    pub diff: DiffColors,
}

/// The theme's diff colours: added and removed lines, the stronger background of a changed word, the
/// gutter and the hunk header, with the plain text colours the other roles use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffColors {
    pub text: Hsla,
    pub muted: Hsla,
    pub add_fg: Hsla,
    pub add_bg: Hsla,
    pub add_emph_bg: Hsla,
    pub del_fg: Hsla,
    pub del_bg: Hsla,
    pub del_emph_bg: Hsla,
    pub gutter_fg: Hsla,
    pub hunk_fg: Hsla,
}

impl Global for Skin {}

/// `ui.graph_color_theme`: `None` follows the theme's own lane palette.
pub struct LaneTheme(pub Option<String>);
impl Global for LaneTheme {}

/// Slug of the theme currently applied.
pub struct CurrentTheme(pub String);
impl Global for CurrentTheme {}

pub fn rgb_of(c: Hsla) -> (u8, u8, u8) {
    let r: gpui_kit::Rgba = c.into();
    (
        (r.r * 255.0).round() as u8,
        (r.g * 255.0).round() as u8,
        (r.b * 255.0).round() as u8,
    )
}

fn rgb_hsla(c: (u8, u8, u8)) -> Hsla {
    rgb(((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32).into()
}

impl Skin {
    /// `muted_pill_bg`: the lane colour blended 45/55 toward the app background.
    pub fn pill_bg(&self, lane: Hsla) -> Hsla {
        let (r, g, b) = rgb_of(lane);
        let t = self.rgb_bg;
        let m = |c: u8, t: u8| ((u16::from(c) * 45 + u16::from(t) * 55) / 100) as u8;
        rgb_hsla((m(r, t.0), m(g, t.1), m(b, t.2)))
    }

    /// `contrasting_fg`.
    pub fn contrasting_fg(&self, bg: Hsla) -> Hsla {
        let (r, g, b) = rgb_of(bg);
        let l = 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
        if l > 140.0 {
            self.on_light
        } else {
            self.on_dark
        }
    }
}

pub(crate) fn hsla_of(c: ThemeColor, fallback: Hsla) -> Hsla {
    match c.approx_rgb() {
        Some((r, g, b)) => rgb_hsla((r, g, b)),
        None => fallback,
    }
}

pub(crate) fn hex(c: Hsla) -> String {
    let (r, g, b) = rgb_of(c);
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub(crate) fn lift(c: Hsla, amount: f32, dark: bool) -> Hsla {
    let mut c = c;
    c.l = if dark {
        (c.l + amount).min(1.0)
    } else {
        (c.l - amount).max(0.0)
    };
    c
}

pub fn all_presets() -> Vec<ThemeSpec> {
    theme::presets()
}

/// The preset with this slug, else the first preset.
pub fn find_preset(slug: &str) -> ThemeSpec {
    let mut all = all_presets();
    match all.iter().position(|t| t.slug == slug) {
        Some(ix) => all.swap_remove(ix),
        None => all.remove(0),
    }
}

/// Makes `spec` the active theme: the kit's colours, the diff highlighter and the [`Skin`].
pub fn apply(spec: &ThemeSpec, cx: &mut App) {
    let dark = !spec.light;
    let fallback = if dark {
        hsla(0., 0., 0.1, 1.)
    } else {
        hsla(0., 0., 0.98, 1.)
    };
    kit::apply_kit_colors(spec, fallback, cx);
    let lane_choice = cx.try_global::<LaneTheme>().and_then(|l| l.0.clone());
    let skin = skin::build(spec, fallback, lane_choice.as_deref());
    cx.set_global(CurrentTheme(spec.slug.clone()));
    cx.set_global(skin);
}
