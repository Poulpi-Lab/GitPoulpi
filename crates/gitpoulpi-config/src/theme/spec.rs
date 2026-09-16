//! [`ThemeColor`] and [`ThemeSpec`]: every role a theme paints, grouped.

use std::fmt;
use std::str::FromStr;

/// One color of a theme. TOML spellings: `"#rrggbb"`, `"ansi:N"`,
/// `"default"` (the terminal's own color).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    Rgb(u8, u8, u8),
    Ansi(u8),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a color: {0:?} (use \"#rrggbb\", \"ansi:N\" or \"default\")")]
pub struct ThemeColorError(pub String);

impl ThemeColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        ThemeColor::Rgb(r, g, b)
    }

    /// The color as RGB for contrast maths: exact for `Rgb`, the standard
    /// xterm palette for `Ansi`, `None` for `Default` (unknown).
    pub fn approx_rgb(self) -> Option<(u8, u8, u8)> {
        match self {
            ThemeColor::Rgb(r, g, b) => Some((r, g, b)),
            ThemeColor::Ansi(n) => Some(xterm_rgb(n)),
            ThemeColor::Default => None,
        }
    }
}

fn xterm_rgb(n: u8) -> (u8, u8, u8) {
    const BASE: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 0, 0),
        (0, 205, 0),
        (205, 205, 0),
        (0, 0, 238),
        (205, 0, 205),
        (0, 205, 205),
        (229, 229, 229),
        (127, 127, 127),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (92, 92, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    match n {
        0..=15 => BASE[n as usize],
        16..=231 => {
            let i = n - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            (level(i / 36), level((i / 6) % 6), level(i % 6))
        }
        232..=255 => {
            let v = 8 + (n - 232) * 10;
            (v, v, v)
        }
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeColor::Rgb(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
            ThemeColor::Ansi(n) => write!(f, "ansi:{n}"),
            ThemeColor::Default => f.write_str("default"),
        }
    }
}

impl FromStr for ThemeColor {
    type Err = ThemeColorError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bad = || ThemeColorError(text.to_string());
        let t = text.trim();
        if t.eq_ignore_ascii_case("default") {
            return Ok(ThemeColor::Default);
        }
        if let Some(hex) = t.strip_prefix('#') {
            if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(bad());
            }
            let v = u32::from_str_radix(hex, 16).map_err(|_| bad())?;
            return Ok(ThemeColor::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8));
        }
        if let Some(n) = t.strip_prefix("ansi:") {
            return n
                .trim()
                .parse::<u8>()
                .map(ThemeColor::Ansi)
                .map_err(|_| bad());
        }
        Err(bad())
    }
}

/// Declares the color groups once: the structs, name-based `set`, the
/// ordered `entries` (for export and the parity table) and the whole
/// [`ThemeSpec`].
macro_rules! theme_groups {
    ($( $gname:ident : $gty:ident { $($f:ident),+ $(,)? } ),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $gty { $(pub $f: ThemeColor),+ }

            impl $gty {
                pub const KEYS: &'static [&'static str] = &[$(stringify!($f)),+];

                fn set(&mut self, key: &str, color: ThemeColor) -> bool {
                    match key {
                        $(stringify!($f) => self.$f = color,)+
                        _ => return false,
                    }
                    true
                }

                fn entries(&self) -> Vec<(&'static str, ThemeColor)> {
                    vec![$((stringify!($f), self.$f)),+]
                }
            }
        )+

        /// A complete theme: every role, plus `light`, the syntect theme
        /// name and the lane palette.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct ThemeSpec {
            pub slug: String,
            /// The label shown in the Theme combo.
            pub name: String,
            /// Whether the theme is a light one (a dark/light hint).
            pub light: bool,
            /// A syntect theme name from the defaults compiled into the TUI.
            pub syntax: String,
            /// This theme's graph lane colors.
            pub graph_palette: Vec<ThemeColor>,
            $(pub $gname: $gty),+
        }

        impl ThemeSpec {
            /// The names of the color groups, in order.
            pub const GROUPS: &'static [&'static str] = &[$(stringify!($gname)),+];

            /// Sets `group.key`; `false` when either name is unknown.
            pub fn set(&mut self, group: &str, key: &str, color: ThemeColor) -> bool {
                match group {
                    $(stringify!($gname) => self.$gname.set(key, color),)+
                    _ => false,
                }
            }

            /// Every color role as `(group, key, color)`, in order
            /// (the palette is not included: see `graph_palette`).
            pub fn entries(&self) -> Vec<(&'static str, &'static str, ThemeColor)> {
                let mut out = Vec::new();
                $(for (k, c) in self.$gname.entries() { out.push((stringify!($gname), k, c)); })+
                out
            }
        }
    };
}

theme_groups! {
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
    graph: GraphColors { wip, stash },
}

impl ThemeSpec {
    /// The color the contrast and blend maths treat as the background:
    /// `ui.bg`, or `ui.blend_bg` while `bg` is the terminal's own.
    pub fn effective_bg(&self) -> ThemeColor {
        match self.ui.bg {
            ThemeColor::Default => self.ui.blend_bg,
            bg => bg,
        }
    }

    /// The background of the Refs pane and the right panel: `ui.side_bg`, or
    /// [`effective_bg`](Self::effective_bg) while it is `default`.
    pub fn effective_side_bg(&self) -> ThemeColor {
        match self.ui.side_bg {
            ThemeColor::Default => self.effective_bg(),
            side => side,
        }
    }

    /// The toolbar row's background, which the selected repository tab shares:
    /// `ui.toolbar_bg`, or [`effective_bg`](Self::effective_bg) while it is `default`.
    pub fn effective_toolbar_bg(&self) -> ThemeColor {
        match self.ui.toolbar_bg {
            ThemeColor::Default => self.effective_bg(),
            c => c,
        }
    }

    /// The status bar's background: `ui.statusbar_bg`, or
    /// [`effective_side_bg`](Self::effective_side_bg) while it is `default`.
    pub fn effective_statusbar_bg(&self) -> ThemeColor {
        match self.ui.statusbar_bg {
            ThemeColor::Default => self.effective_side_bg(),
            c => c,
        }
    }

    /// The status bar's text: `ui.statusbar_fg`, or `ui.text` while it is `default`.
    pub fn effective_statusbar_fg(&self) -> ThemeColor {
        match self.ui.statusbar_fg {
            ThemeColor::Default => self.ui.text,
            c => c,
        }
    }

    /// A complete theme file (what `theme export` prints); it parses back
    /// to the same theme with this one as the preset it overrides.
    pub fn to_toml(&self) -> String {
        let mut out = format!(
            "name = {:?}\nlight = {}\nsyntax = {:?}\ngraph_palette = [{}]\n",
            self.name,
            self.light,
            self.syntax,
            self.graph_palette
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut group = "";
        for (g, k, c) in self.entries() {
            if g != group {
                out.push_str(&format!("\n[{g}]\n"));
                group = g;
            }
            out.push_str(&format!("{k} = \"{c}\"\n"));
        }
        out
    }
}
