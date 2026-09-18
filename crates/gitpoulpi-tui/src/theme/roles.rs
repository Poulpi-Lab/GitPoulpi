//! The resolved runtime [`Theme`]: every role of a `ThemeSpec` as a ratatui
//! [`Color`].

use gitpoulpi_config::theme::{ThemeColor, ThemeSpec};
use ratatui::style::Color;

/// One theme color as a ratatui color. `ansi:7` is `Color::White`, the
/// spelling the UI always used for its "white" (see the Dark preset).
pub(crate) fn color_of(c: ThemeColor) -> Color {
    match c {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        ThemeColor::Default => Color::Reset,
        ThemeColor::Ansi(0) => Color::Black,
        ThemeColor::Ansi(1) => Color::Red,
        ThemeColor::Ansi(2) => Color::Green,
        ThemeColor::Ansi(3) => Color::Yellow,
        ThemeColor::Ansi(4) => Color::Blue,
        ThemeColor::Ansi(5) => Color::Magenta,
        ThemeColor::Ansi(6) => Color::Cyan,
        ThemeColor::Ansi(7) | ThemeColor::Ansi(15) => Color::White,
        ThemeColor::Ansi(8) => Color::DarkGray,
        ThemeColor::Ansi(9) => Color::LightRed,
        ThemeColor::Ansi(10) => Color::LightGreen,
        ThemeColor::Ansi(11) => Color::LightYellow,
        ThemeColor::Ansi(12) => Color::LightBlue,
        ThemeColor::Ansi(13) => Color::LightMagenta,
        ThemeColor::Ansi(14) => Color::LightCyan,
        ThemeColor::Ansi(n) => Color::Indexed(n),
    }
}

macro_rules! role_groups {
    ($( $gname:ident : $gty:ident { $($f:ident),+ $(,)? } ),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub(crate) struct $gty { $(pub(crate) $f: Color),+ }

            impl $gty {
                /// Every role of the group, by name (the golden snapshots
                /// print colors as these names).
                #[cfg(test)]
                fn each(&self, mut f: impl FnMut(&'static str, Color)) {
                    $(f(stringify!($f), self.$f);)+
                }

                fn from_spec(s: &gitpoulpi_config::theme::$gty) -> Self {
                    Self { $($f: color_of(s.$f)),+ }
                }
            }
        )+
    };
}

macro_rules! theme_fields {
    ($( $gname:ident : $gty:ident ),+ $(,)?) => {
        /// A fully resolved theme (see the module doc).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) struct Theme {
            pub(crate) slug: &'static str,
            pub(crate) light: bool,
            /// The syntect theme name.
            pub(crate) syntax: &'static str,
            pub(crate) graph: GraphRoles,
            $(pub(crate) $gname: $gty),+
        }

        impl Theme {
            /// `group.role` and its color, for every role, in declaration order.
            #[cfg(test)]
            pub(crate) fn roles(&self) -> Vec<(String, Color)> {
                let mut out = vec![
                    ("graph.wip".to_string(), self.graph.wip),
                    ("graph.stash".to_string(), self.graph.stash),
                ];
                $(self.$gname.each(|n, c| out.push((format!("{}.{}", stringify!($gname), n), c)));)+
                out
            }
        }

        impl From<&ThemeSpec> for Theme {
            fn from(s: &ThemeSpec) -> Self {
                let leak = |t: &str| -> &'static str { Box::leak(t.to_string().into_boxed_str()) };
                let palette: Vec<Color> = s.graph_palette.iter().map(|c| color_of(*c)).collect();
                Theme {
                    slug: leak(&s.slug),
                    light: s.light,
                    syntax: leak(&s.syntax),
                    graph: GraphRoles {
                        wip: color_of(s.graph.wip),
                        stash: color_of(s.graph.stash),
                        palette: Box::leak(palette.into_boxed_slice()),
                    },
                    $($gname: $gty::from_spec(&s.$gname)),+
                }
            }
        }
    };
}

role_groups! {
    ui: UiColors {
        bg, blend_bg, panel, side_bg, toolbar_bg, statusbar_bg, statusbar_fg, text, muted, disabled, accent, on_accent, selection, hover,
        button, button_hover, on_button, border_focused, border, tooltip_bg, popup_bg,
        header_fg, input_rest, input_hover, input_focus, text_selection_bg, text_selection_fg
    },
    status: StatusColors { success, warning, danger, info, ai },
    refs: RefColors { local, remote, tag },
    scrollbar: ScrollbarColors {
        track, thumb, thumb_hover, marks_add, marks_del, marks_cursor, marks_occurrence,
        marks_modified, marks_conflict_open, marks_conflict_current
    },
    minimap: MinimapColors { bg, view, ink },
    diff: DiffColors {
        add_fg, del_fg, add_bg, del_bg, hunk_fg, gutter_fg, range_bg, search_hit_bg
    },
    merge: MergeColors {
        ours, theirs, base, edit, text, ours_tint, theirs_tint, ours_pick, theirs_pick,
        base_pick, current_region, strip_bg, strip_fg
    },
    editor: EditorColors { text, current_line, find_hit, find_hit_fg },
    toolbar: ToolbarColors { pull, push, stash, branch, lfs, tag, search, undo },
    hosting: HostingColors { merged, chip_fg },
    chrome: ChromeColors {
        stat_add, stat_del, notice, marker, on_bright, wip_del, error_bg, banner_bg, banner_danger_bg, toolbar_arrow_bg,
        pill_branch, pill_onto, action_edit, dropdown_bg, dropdown_hit, action_reset, tint_reset, tint_danger, tint_success, toast_bg, proposal_bg, confirm_danger, disabled_button_bg, toolbar_bg, toolbar_bg_hover, toolbar_fg, toolbar_disabled_fg
    },
}

/// The graph roles: two colors plus the lane palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GraphRoles {
    pub(crate) wip: Color,
    pub(crate) stash: Color,
    pub(crate) palette: &'static [Color],
}

theme_fields! {
    ui: UiColors, status: StatusColors, refs: RefColors, scrollbar: ScrollbarColors,
    minimap: MinimapColors, diff: DiffColors, merge: MergeColors, editor: EditorColors,
    toolbar: ToolbarColors, hosting: HostingColors, chrome: ChromeColors,
}

impl Theme {
    /// The built-in Dark theme: exactly the palette the UI had before themes.
    pub(crate) fn dark() -> Theme {
        let spec = gitpoulpi_config::theme::preset("dark").expect("dark is a built-in preset");
        Theme::from(&spec)
    }
}
