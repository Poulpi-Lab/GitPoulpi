//! The community-palette presets (Solarized, Catppuccin, Gruvbox,
//! Nord) and the Terminal preset. Each is described by a small [`Pal`] (the
//! palette's own named colors) from which [`derive`] fills every role, so a
//! preset is ~30 lines of data and the contrast rules hold by construction
//! (`tests::presets_pass_the_contrast_rules` enforces them).

use super::presets::lane_colors;
use super::spec::{
    ChromeColors, DiffColors, EditorColors, GraphColors, HostingColors, MergeColors, MinimapColors,
    RefColors, ScrollbarColors, StatusColors, ToolbarColors, UiColors,
};
use super::{contrast_ratio, ThemeColor, ThemeSpec};

fn ratio(a: Rgb, b: Rgb) -> f64 {
    contrast_ratio(col(a), col(b)).unwrap_or(1.0)
}

pub(super) type Rgb = (u8, u8, u8);

pub(super) const fn hex(v: u32) -> Rgb {
    ((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub(super) fn col((r, g, b): Rgb) -> ThemeColor {
    ThemeColor::Rgb(r, g, b)
}

/// `a` towards `b` by `t` (0.0 = `a`, 1.0 = `b`).
pub(super) fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let m = |x: u8, y: u8| (f64::from(x) * (1.0 - t) + f64::from(y) * t).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

/// Whichever of `a` / `b` reads better on `on`.
pub(super) fn best_on(on: Rgb, a: Rgb, b: Rgb) -> Rgb {
    let r = |x: Rgb| contrast_ratio(col(x), col(on)).unwrap_or(1.0);
    if r(a) >= r(b) {
        a
    } else {
        b
    }
}

/// `spec` with a distinct background for the Refs pane and the right panel
/// (`ui.side_bg`), the way several popular Git clients and editors paint their sidebars.
fn with_side(mut spec: ThemeSpec, side: Rgb) -> ThemeSpec {
    spec.ui.side_bg = col(side);
    // The status bar matches the side panes; the toolbar sits a step between
    // the main background and them.
    spec.ui.statusbar_bg = col(side);
    if let Some(bg) = spec.effective_bg().approx_rgb() {
        spec.ui.toolbar_bg = col(mix(bg, side, 0.5));
    }
    spec
}

/// `spec` with its own toolbar (and selected tab) color, and a status bar
/// color and text, for presets whose app paints them distinctly.
fn with_chrome(mut spec: ThemeSpec, toolbar: Rgb, status_bg: Rgb, status_fg: Rgb) -> ThemeSpec {
    spec.ui.toolbar_bg = col(toolbar);
    spec.ui.statusbar_bg = col(status_bg);
    spec.ui.statusbar_fg = col(status_fg);
    spec
}

/// A palette's own named colors.
pub(super) struct Pal {
    pub(super) slug: &'static str,
    pub(super) name: &'static str,
    pub(super) light: bool,
    pub(super) syntax: &'static str,
    pub(super) lanes: &'static str,
    pub(super) bg: Rgb,
    pub(super) panel: Rgb,
    pub(super) text: Rgb,
    pub(super) muted: Rgb,
    pub(super) accent: Rgb,
    pub(super) selection: Rgb,
    pub(super) success: Rgb,
    pub(super) warning: Rgb,
    pub(super) danger: Rgb,
    pub(super) ai: Rgb,
    pub(super) cyan: Rgb,
}

pub(super) fn derive(p: &Pal) -> ThemeSpec {
    let Pal {
        bg,
        panel,
        text,
        muted,
        accent,
        selection,
        success,
        warning,
        danger,
        ai,
        cyan,
        light,
        ..
    } = *p;
    let dark_ink = hex(0x10121c);
    let white = (255, 255, 255);
    // Strong fills carrying `on_button` text: bright on dark themes, deep on
    // light ones.
    let fill = |c: Rgb| {
        if light {
            mix(c, text, 0.4)
        } else {
            mix(bg, c, 0.5)
        }
    };
    let icon = |c: Rgb| mix(c, white, if light { 0.55 } else { 0.2 });
    let ticon = |c: Rgb| if light { mix(c, text, 0.45) } else { icon(c) };
    let tint = |c: Rgb, t: f64| mix(bg, c, t);
    let button = if light {
        mix(text, dark_ink, 0.45)
    } else {
        mix(panel, text, 0.12)
    };
    let on_button = if light {
        mix(bg, white, 0.7)
    } else {
        mix(text, white, 0.6)
    };
    let on_accent = best_on(accent, dark_ink, white);
    // The hovered button fill: a step towards the text color, as far as its
    // label (`on_button`) still reads 4.5.
    let button_hover = [0.2, 0.15, 0.1, 0.05, 0.0]
        .into_iter()
        .map(|t| mix(button, text, t))
        .find(|h| ratio(on_button, *h) >= 4.5)
        .unwrap_or(button);
    // Disabled controls: muted, but readable (3.2) on the panel and on the
    // grayed button fill (resting and hovered).
    // The destructive confirm button's fill, deepened until `on_button` reads 4.5.
    let confirm_danger = {
        let toward = if light { text } else { bg };
        let mut d = fill(danger);
        let mut t = 0.0;
        while t < 1.0 && ratio(on_button, d) < 4.5 {
            t += 0.05;
            d = mix(fill(danger), toward, t);
        }
        d
    };
    let off_fill = mix(bg, text, 0.1);
    let off_hover = {
        let up = |c: u8| c.saturating_add(14);
        (up(off_fill.0), up(off_fill.1), up(off_fill.2))
    };
    let disabled = {
        let mut t = 0.0;
        let mut d = mix(muted, bg, 0.5);
        while t <= 1.0
            && [panel, off_fill, off_hover]
                .iter()
                .any(|b| ratio(d, *b) < 3.2)
        {
            t += 0.05;
            d = mix(mix(muted, bg, 0.5), text, t);
        }
        d
    };
    let hover = mix(bg, selection, 0.5);
    let (rest, input_hover, focus) = if light {
        (
            mix(bg, text, 0.07),
            mix(bg, text, 0.12),
            mix(bg, white, 0.7),
        )
    } else {
        (
            mix(bg, text, 0.07),
            mix(bg, text, 0.12),
            mix(bg, text, 0.17),
        )
    };
    let add_fg = mix(success, text, if light { 0.45 } else { 0.3 });
    let del_fg = mix(danger, text, if light { 0.45 } else { 0.3 });
    let lanes = lane_colors(p.lanes);
    ThemeSpec {
        slug: p.slug.into(),
        name: p.name.into(),
        light,
        syntax: p.syntax.into(),
        graph_palette: lanes,
        ui: UiColors {
            bg: col(bg),
            blend_bg: col(bg),
            panel: col(panel),
            // `default`: the side panes share `bg` (see `with_side`).
            side_bg: ThemeColor::Default,
            // A step off the background (`with_side` and `with_chrome` set
            // the presets that paint their own).
            toolbar_bg: col(mix(bg, text, if light { 0.06 } else { 0.07 })),
            statusbar_bg: col(mix(bg, text, 0.04)),
            statusbar_fg: col(text),
            text: col(text),
            muted: col(muted),
            disabled: col(disabled),
            accent: col(accent),
            on_accent: col(on_accent),
            selection: col(selection),
            hover: col(hover),
            button: col(button),
            button_hover: col(button_hover),
            on_button: col(on_button),
            border_focused: col(accent),
            border: col(muted),
            tooltip_bg: col(panel),
            popup_bg: col(panel),
            header_fg: col(text),
            input_rest: col(rest),
            input_hover: col(input_hover),
            input_focus: col(focus),
            text_selection_bg: col(accent),
            text_selection_fg: col(on_accent),
        },
        status: StatusColors {
            success: col(success),
            warning: col(warning),
            danger: col(danger),
            info: col(accent),
            ai: col(ai),
        },
        refs: RefColors {
            local: col(success),
            remote: col(accent),
            tag: col(warning),
        },
        scrollbar: ScrollbarColors {
            track: col(mix(bg, text, 0.08)),
            thumb: col(mix(bg, text, 0.3)),
            thumb_hover: col(accent),
            marks_add: col(success),
            marks_del: col(danger),
            marks_cursor: col(text),
            marks_occurrence: col(warning),
            marks_modified: col(mix(warning, danger, 0.3)),
            marks_conflict_open: col(mix(warning, danger, 0.15)),
            marks_conflict_current: col(accent),
        },
        minimap: MinimapColors {
            bg: col(mix(bg, panel, 0.6)),
            view: col(selection),
            ink: col(muted),
        },
        diff: DiffColors {
            add_fg: col(add_fg),
            del_fg: col(del_fg),
            add_bg: col(tint(success, if light { 0.2 } else { 0.18 })),
            del_bg: col(tint(danger, if light { 0.2 } else { 0.18 })),
            hunk_fg: col(mix(accent, text, 0.2)),
            gutter_fg: col(mix(muted, bg, 0.3)),
            range_bg: col(tint(accent, 0.15)),
            search_hit_bg: col(tint(warning, 0.35)),
        },
        merge: MergeColors {
            ours: col(accent),
            theirs: col(mix(warning, danger, 0.3)),
            base: col(muted),
            edit: col(warning),
            text: col(text),
            ours_tint: col(tint(accent, 0.12)),
            theirs_tint: col(tint(warning, 0.12)),
            ours_pick: col(tint(accent, 0.28)),
            theirs_pick: col(tint(warning, 0.28)),
            base_pick: col(tint(muted, 0.2)),
            current_region: col(mix(bg, text, 0.05)),
            strip_bg: col(tint(warning, 0.2)),
            strip_fg: col(if light {
                mix(warning, text, 0.35)
            } else {
                warning
            }),
        },
        editor: EditorColors {
            text: col(text),
            current_line: col(mix(bg, text, 0.05)),
            find_hit: col(tint(warning, 0.4)),
            find_hit_fg: col(text),
        },
        // Icons sit on `chrome.toolbar_bg`: pastels on a dark chip, deepened
        // tones on a light one.
        toolbar: ToolbarColors {
            pull: col(ticon(accent)),
            push: col(ticon(success)),
            stash: col(ticon(warning)),
            branch: col(ticon(ai)),
            lfs: col(ticon(mix(success, warning, 0.5))),
            tag: col(ticon(mix(warning, danger, 0.2))),
            search: col(ticon(muted)),
            undo: col(ticon(cyan)),
        },
        hosting: HostingColors {
            merged: col(ai),
            chip_fg: col(best_on(ai, dark_ink, white)),
        },
        chrome: ChromeColors {
            stat_add: col(success),
            stat_del: col(danger),
            notice: col(danger),
            marker: col(ai),
            on_bright: col(on_accent),
            wip_del: col(danger),
            error_bg: col(tint(danger, 0.2)),
            banner_bg: col(tint(warning, 0.2)),
            banner_danger_bg: col(fill(danger)),
            toolbar_arrow_bg: col(mix(bg, text, if light { 0.2 } else { 0.15 })),
            pill_branch: col(fill(ai)),
            pill_onto: col(fill(cyan)),
            action_edit: col(accent),
            dropdown_bg: col(panel),
            dropdown_hit: col(mix(accent, if light { bg } else { text }, 0.3)),
            action_reset: col(accent),
            tint_reset: col(tint(accent, 0.15)),
            tint_danger: col(tint(danger, 0.15)),
            tint_success: col(tint(success, 0.15)),
            toast_bg: col(fill(ai)),
            proposal_bg: col(fill(ai)),
            confirm_danger: col(confirm_danger),
            disabled_button_bg: col(mix(bg, text, 0.1)),
            toolbar_bg: col(if light { mix(bg, text, 0.12) } else { button }),
            toolbar_bg_hover: col(if light {
                mix(bg, text, 0.24)
            } else {
                mix(button, text, 0.2)
            }),
            toolbar_fg: col(if light { text } else { on_button }),
            toolbar_disabled_fg: col(muted),
        },
        graph: GraphColors {
            wip: col(muted),
            stash: col(muted),
        },
    }
}

pub(super) fn solarized_dark() -> ThemeSpec {
    derive(&Pal {
        slug: "solarized-dark",
        name: "Solarized Dark",
        light: false,
        syntax: "Solarized (dark)",
        lanes: "solarized",
        bg: hex(0x002b36),
        panel: hex(0x073642),
        text: hex(0xaab6b6),
        muted: hex(0x839496),
        accent: hex(0x268bd2),
        selection: hex(0x0c4452),
        success: hex(0x859900),
        warning: hex(0xb58900),
        danger: hex(0xdc322f),
        ai: hex(0x6c71c4),
        cyan: hex(0x2aa198),
    })
}

pub(super) fn solarized_light() -> ThemeSpec {
    derive(&Pal {
        slug: "solarized-light",
        name: "Solarized Light",
        light: true,
        syntax: "Solarized (light)",
        lanes: "solarized",
        bg: hex(0xfdf6e3),
        panel: hex(0xeee8d5),
        text: hex(0x4d6269),
        muted: hex(0x657b83),
        accent: hex(0x268bd2),
        selection: hex(0xe4ddc4),
        success: hex(0x859900),
        warning: hex(0xb58900),
        danger: hex(0xdc322f),
        ai: hex(0x6c71c4),
        cyan: hex(0x2aa198),
    })
}

pub(super) fn catppuccin_mocha() -> ThemeSpec {
    with_side(
        derive(&Pal {
            slug: "catppuccin-mocha",
            name: "Catppuccin Mocha",
            light: false,
            syntax: "base16-mocha.dark",
            lanes: "catppuccin-mocha",
            bg: hex(0x1e1e2e),
            panel: hex(0x28293b),
            text: hex(0xcdd6f4),
            muted: hex(0xa6adc8),
            accent: hex(0x89b4fa),
            selection: hex(0x45475a),
            success: hex(0xa6e3a1),
            warning: hex(0xf9e2af),
            danger: hex(0xf38ba8),
            ai: hex(0xcba6f7),
            cyan: hex(0x94e2d5),
        }),
        hex(0x181825),
    )
}

pub(super) fn catppuccin_latte() -> ThemeSpec {
    with_side(
        derive(&Pal {
            slug: "catppuccin-latte",
            name: "Catppuccin Latte",
            light: true,
            syntax: "InspiredGitHub",
            lanes: "catppuccin-latte",
            bg: hex(0xeff1f5),
            panel: hex(0xe6e9ef),
            text: hex(0x4c4f69),
            muted: hex(0x6c6f85),
            accent: hex(0x1e66f5),
            selection: hex(0xccd0da),
            success: hex(0x40a02b),
            warning: hex(0xdf8e1d),
            danger: hex(0xd20f39),
            ai: hex(0x8839ef),
            cyan: hex(0x179299),
        }),
        hex(0xe6e9ef),
    )
}

pub(super) fn gruvbox_dark() -> ThemeSpec {
    derive(&Pal {
        slug: "gruvbox-dark",
        name: "Gruvbox Dark",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "gruvbox",
        bg: hex(0x282828),
        panel: hex(0x3c3836),
        text: hex(0xebdbb2),
        muted: hex(0xa89984),
        accent: hex(0x83a598),
        selection: hex(0x504945),
        success: hex(0xb8bb26),
        warning: hex(0xfabd2f),
        danger: hex(0xfb4934),
        ai: hex(0xd3869b),
        cyan: hex(0x8ec07c),
    })
}

pub(super) fn nord() -> ThemeSpec {
    derive(&Pal {
        slug: "nord",
        name: "Nord",
        light: false,
        syntax: "base16-ocean.dark",
        lanes: "nord",
        bg: hex(0x2e3440),
        panel: hex(0x3b4252),
        text: hex(0xd8dee9),
        muted: hex(0xa3adbf),
        accent: hex(0x88c0d0),
        selection: hex(0x434c5e),
        success: hex(0xa3be8c),
        warning: hex(0xebcb8b),
        danger: hex(0xbf616a),
        ai: hex(0xb48ead),
        cyan: hex(0x8fbcbb),
    })
}

/// Dracula spec, https://draculatheme.com/contribute (Background 282a36, Current Line 44475a, Foreground f8f8f2, Comment 6272a4, Cyan 8be9fd, Green 50fa7b, Pink ff79c6, Purple bd93f9, Red ff5555, Yellow f1fa8c). `panel` is Dracula's darker 21222c; `muted` is the Comment color lightened to pass contrast.
pub(super) fn dracula() -> ThemeSpec {
    derive(&Pal {
        slug: "dracula",
        name: "Dracula",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "dracula",
        bg: hex(0x282a36),
        panel: hex(0x21222c),
        text: hex(0xf8f8f2),
        muted: hex(0x9aa5ce),
        accent: hex(0xbd93f9),
        selection: hex(0x44475a),
        success: hex(0x50fa7b),
        warning: hex(0xf1fa8c),
        danger: hex(0xff5555),
        ai: hex(0xff79c6),
        cyan: hex(0x8be9fd),
    })
}

/// Tokyo Night (night variant), https://github.com/folke/tokyonight.nvim/blob/main/extras/lua/tokyonight_night.lua (bg 1a1b26, bg_dark 16161e, fg c0caf5, dark5 737aa2, bg_visual 283457, blue 7aa2f7, green 9ece6a, yellow e0af68, red f7768e, magenta bb9af7, cyan 7dcfff).
pub(super) fn tokyo_night() -> ThemeSpec {
    derive(&Pal {
        slug: "tokyo-night",
        name: "Tokyo Night",
        light: false,
        syntax: "base16-mocha.dark",
        lanes: "tokyo-night",
        bg: hex(0x1a1b26),
        panel: hex(0x16161e),
        text: hex(0xc0caf5),
        muted: hex(0x737aa2),
        accent: hex(0x7aa2f7),
        selection: hex(0x283457),
        success: hex(0x9ece6a),
        warning: hex(0xe0af68),
        danger: hex(0xf7768e),
        ai: hex(0xbb9af7),
        cyan: hex(0x7dcfff),
    })
}

/// "Cyberpunk" scheme of iTerm2-Color-Schemes (also in Gogh), https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/windowsterminal/Cyberpunk.json (background 332a57, foreground e5e5e5, blue 00bfff, green 00fbac, yellow fffa6a, red ff7092, purple df95ff, cyan 86cbfe). `panel`, `selection` and `muted` are derived from the purple background.
pub(super) fn cyberpunk() -> ThemeSpec {
    derive(&Pal {
        slug: "cyberpunk",
        name: "Cyberpunk",
        light: false,
        syntax: "base16-mocha.dark",
        lanes: "cyberpunk",
        bg: hex(0x332a57),
        panel: hex(0x2a2248),
        text: hex(0xe5e5e5),
        muted: hex(0xb6add9),
        accent: hex(0x00bfff),
        selection: hex(0x4d4285),
        success: hex(0x00fbac),
        warning: hex(0xfffa6a),
        danger: hex(0xff7092),
        ai: hex(0xdf95ff),
        cyan: hex(0x86cbfe),
    })
}

/// "Matrix" scheme of iTerm2-Color-Schemes, https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/windowsterminal/Matrix.json (background 0f191c, selection 18282e, green 82d967, bright purple 11ff25, bright green 90d762, cyan 50b45a, yellow ffd700). The scheme has no red, so `danger` is a plain ff5555; `muted` is a lightened scheme green.
pub(super) fn matrix() -> ThemeSpec {
    derive(&Pal {
        slug: "matrix",
        name: "Matrix",
        light: false,
        syntax: "base16-ocean.dark",
        lanes: "matrix",
        bg: hex(0x0f191c),
        panel: hex(0x131f23),
        text: hex(0x82d967),
        muted: hex(0x6f9a68),
        accent: hex(0x11ff25),
        selection: hex(0x1d3a2b),
        success: hex(0x2fc079),
        warning: hex(0xffd700),
        danger: hex(0xff5555),
        ai: hex(0xc1ff8a),
        cyan: hex(0x50b45a),
    })
}

/// "Fire": an original GitPoulpi theme (no upstream palette). Embers: a near-black oxblood background, warm ivory text, flame-orange accent, ember-red selection; `success` is a yellow-green that still reads as success, `ai` a hot pink.
pub(super) fn fire() -> ThemeSpec {
    derive(&Pal {
        slug: "fire",
        name: "Fire",
        light: false,
        syntax: "base16-mocha.dark",
        lanes: "fire",
        bg: hex(0x3a1604),
        panel: hex(0x471c06),
        text: hex(0xfff0dc),
        muted: hex(0xd4a47c),
        accent: hex(0xffa02e),
        selection: hex(0x7a2e0a),
        success: hex(0xa8c93a),
        warning: hex(0xffc107),
        danger: hex(0xff3b30),
        ai: hex(0xff4fa3),
        cyan: hex(0x3fb8a8),
    })
}

/// "Fire Light": the light twin of "Fire", an original GitPoulpi theme. A real orange background, dark ember text, oxblood accent, pale flame selection; the lanes are deep reds, browns, plum, teal and navy so they read on orange.
pub(super) fn fire_light() -> ThemeSpec {
    derive(&Pal {
        slug: "fire-light",
        name: "Fire Light",
        light: true,
        syntax: "GitPoulpi Ember",
        lanes: "fire-light",
        bg: hex(0xffd0a2),
        panel: hex(0xffc28a),
        text: hex(0x2a0c00),
        muted: hex(0x6a2e08),
        accent: hex(0x8f0a12),
        selection: hex(0xfff07a),
        success: hex(0x245c00),
        warning: hex(0x6b3a00),
        danger: hex(0x9c0a0a),
        ai: hex(0x7a0a52),
        cyan: hex(0x004f52),
    })
}

/// "Sunset": an original GitPoulpi theme (no upstream palette). Dusk: deep indigo-purple background, pale peach text, coral accent, plum selection, soft gold-green success, lavender `ai`.
pub(super) fn sunset() -> ThemeSpec {
    derive(&Pal {
        slug: "sunset",
        name: "Sunset",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "sunset",
        bg: hex(0x1f1740),
        panel: hex(0x281e4d),
        text: hex(0xffe0cc),
        muted: hex(0xb9a3c9),
        accent: hex(0xff7f50),
        selection: hex(0x4a2a63),
        success: hex(0xb5cf6b),
        warning: hex(0xffc233),
        danger: hex(0xff5c7a),
        ai: hex(0xc9a0ff),
        cyan: hex(0x6fd0ff),
    })
}

/// Atom One Dark, https://github.com/atom/atom/blob/master/packages/one-dark-syntax/styles/colors.less (bg 282c34, fg abb2bf, blue 61afef, green 98c379, yellow e5c07b, red e06c75, purple c678dd, cyan 56b6c2; selection 3e4451, gutter 636d83).
pub(super) fn one_dark() -> ThemeSpec {
    derive(&Pal {
        slug: "one-dark",
        name: "One Dark",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "one-dark",
        bg: hex(0x282c34),
        panel: hex(0x21252b),
        text: hex(0xabb2bf),
        muted: hex(0x7f8794),
        accent: hex(0x61afef),
        selection: hex(0x3e4451),
        success: hex(0x98c379),
        warning: hex(0xe5c07b),
        danger: hex(0xe06c75),
        ai: hex(0xc678dd),
        cyan: hex(0x56b6c2),
    })
}

/// Rosé Pine, https://rosepinetheme.com/palette/ (base 191724, surface 1f1d2e, text e0def4, subtle 908caa, highlight med 403d52, foam 9ccfd8, iris c4a7e7, gold f6c177, love eb6f92; pine 3e8fb0 is the Moon variant). The palette has no green: `success` is foam.
pub(super) fn rose_pine() -> ThemeSpec {
    derive(&Pal {
        slug: "rose-pine",
        name: "Rosé Pine",
        light: false,
        syntax: "base16-mocha.dark",
        lanes: "rose-pine",
        bg: hex(0x191724),
        panel: hex(0x1f1d2e),
        text: hex(0xe0def4),
        muted: hex(0x908caa),
        accent: hex(0x9ccfd8),
        selection: hex(0x403d52),
        success: hex(0x9ccfd8),
        warning: hex(0xf6c177),
        danger: hex(0xeb6f92),
        ai: hex(0xc4a7e7),
        cyan: hex(0x3e8fb0),
    })
}

/// Kanagawa (wave), https://github.com/rebelot/kanagawa.nvim#default-colors (sumiInk3 1f1f28, sumiInk4 2a2a37, fujiWhite dcd7ba, springViolet1 938aa9, crystalBlue 7e9cd8, waveBlue2 2d4f67, springGreen 98bb6c, carpYellow e6c384, waveRed e46876, oniViolet 957fb8, waveAqua2 7aa89f).
pub(super) fn kanagawa() -> ThemeSpec {
    derive(&Pal {
        slug: "kanagawa",
        name: "Kanagawa",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "kanagawa",
        bg: hex(0x1f1f28),
        panel: hex(0x2a2a37),
        text: hex(0xdcd7ba),
        muted: hex(0x938aa9),
        accent: hex(0x7e9cd8),
        selection: hex(0x2d4f67),
        success: hex(0x98bb6c),
        warning: hex(0xe6c384),
        danger: hex(0xe46876),
        ai: hex(0x957fb8),
        cyan: hex(0x7aa89f),
    })
}

/// Everforest dark (medium), https://github.com/sainnhe/everforest/blob/master/palette.md (bg0 2d353b, bg1 343f44, bg3 475258, fg d3c6aa, gray1 859289 lightened to 9aa79d, red e67e80, yellow dbbc7f, green a7c080, aqua 83c092, blue 7fbbb3, purple d699b6).
pub(super) fn everforest_dark() -> ThemeSpec {
    derive(&Pal {
        slug: "everforest-dark",
        name: "Everforest Dark",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "everforest",
        bg: hex(0x2d353b),
        panel: hex(0x343f44),
        text: hex(0xd3c6aa),
        muted: hex(0x9aa79d),
        accent: hex(0x7fbbb3),
        selection: hex(0x475258),
        success: hex(0xa7c080),
        warning: hex(0xdbbc7f),
        danger: hex(0xe67e80),
        ai: hex(0xd699b6),
        cyan: hex(0x83c092),
    })
}

/// Monokai (Sublime Text original), https://monokai.pro / https://github.com/sickill/vim-monokai (bg 272822, fg f8f8f2, selection 49483e, yellow e6db74, red f92672, purple ae81ff, blue 66d9ef, green a6e22e). `muted` is the comment color 75715e lightened; `panel` is the darker 1e1f1c.
pub(super) fn monokai() -> ThemeSpec {
    derive(&Pal {
        slug: "monokai",
        name: "Monokai",
        light: false,
        syntax: "base16-eighties.dark",
        lanes: "monokai",
        bg: hex(0x272822),
        panel: hex(0x1e1f1c),
        text: hex(0xf8f8f2),
        muted: hex(0xa59f85),
        accent: hex(0x66d9ef),
        selection: hex(0x49483e),
        success: hex(0xa6e22e),
        warning: hex(0xe6db74),
        danger: hex(0xf92672),
        ai: hex(0xae81ff),
        cyan: hex(0x78dce8),
    })
}

/// Gruvbox light (medium), https://github.com/morhetz/gruvbox#palette (bg0 fbf1c7, bg1 ebdbb2, bg2 d5c4a1, fg 3c3836, gray 7c6f64, faded red 9d0006, green 79740e, yellow b57614, blue 076678, purple 8f3f71, aqua 427b58).
pub(super) fn gruvbox_light() -> ThemeSpec {
    derive(&Pal {
        slug: "gruvbox-light",
        name: "Gruvbox Light",
        light: true,
        syntax: "InspiredGitHub",
        lanes: "gruvbox-light",
        bg: hex(0xfbf1c7),
        panel: hex(0xebdbb2),
        text: hex(0x3c3836),
        muted: hex(0x7c6f64),
        accent: hex(0x076678),
        selection: hex(0xd5c4a1),
        success: hex(0x79740e),
        warning: hex(0xb57614),
        danger: hex(0x9d0006),
        ai: hex(0x8f3f71),
        cyan: hex(0x427b58),
    })
}

/// Atom One Light, https://github.com/atom/atom/tree/master/packages/one-light-syntax (bg fafafa, gutter/panel eaeaeb, fg 383a42, mono-2 696c77, blue 4078f2, green 50a14f, orange b76b01 (hue-6, computed from the less hsl values), red e45649, purple a626a4, cyan 0184bc; selection e5e5e6 derived).
pub(super) fn one_light() -> ThemeSpec {
    derive(&Pal {
        slug: "one-light",
        name: "One Light",
        light: true,
        syntax: "InspiredGitHub",
        lanes: "one-light",
        bg: hex(0xfafafa),
        panel: hex(0xeaeaeb),
        text: hex(0x383a42),
        muted: hex(0x696c77),
        accent: hex(0x4078f2),
        selection: hex(0xe5e5e6),
        success: hex(0x50a14f),
        warning: hex(0xb76b01),
        danger: hex(0xe45649),
        ai: hex(0xa626a4),
        cyan: hex(0x0184bc),
    })
}

/// Tokyo Night Day, https://github.com/folke/tokyonight.nvim (extras/lua/tokyonight_day.lua: bg e1e2e7, bg_dark d0d5e3, fg 3760bf darkened to 2b4a96 for contrast, dark5 68709a, blue 2e7de9, bg_visual b7c1e3, green 587539, yellow 8c6c3e, red f52a65, purple 9854f1, cyan 007197).
pub(super) fn tokyo_night_day() -> ThemeSpec {
    derive(&Pal {
        slug: "tokyo-night-day",
        name: "Tokyo Night Day",
        light: true,
        syntax: "InspiredGitHub",
        lanes: "tokyo-night-day",
        bg: hex(0xe1e2e7),
        panel: hex(0xd0d5e3),
        text: hex(0x2b4a96),
        muted: hex(0x68709a),
        accent: hex(0x2e7de9),
        selection: hex(0xb7c1e3),
        success: hex(0x587539),
        warning: hex(0x8c6c3e),
        danger: hex(0xf52a65),
        ai: hex(0x9854f1),
        cyan: hex(0x007197),
    })
}

/// Rosé Pine Moon, https://rosepinetheme.com/palette/ (base 232136, surface 2a273f, text e0def4, subtle 908caa, highlight med 44415a, foam 9ccfd8, iris c4a7e7, gold f6c177, love eb6f92, pine 3e8fb0). The palette has no green: `success` is foam.
pub(super) fn rose_pine_moon() -> ThemeSpec {
    derive(&Pal {
        slug: "rose-pine-moon",
        name: "Rosé Pine Moon",
        light: false,
        syntax: "base16-mocha.dark",
        lanes: "rose-pine-moon",
        bg: hex(0x232136),
        panel: hex(0x2a273f),
        text: hex(0xe0def4),
        muted: hex(0x908caa),
        accent: hex(0x9ccfd8),
        selection: hex(0x44415a),
        success: hex(0x9ccfd8),
        warning: hex(0xf6c177),
        danger: hex(0xeb6f92),
        ai: hex(0xc4a7e7),
        cyan: hex(0x3e8fb0),
    })
}

/// Rosé Pine Dawn, https://rosepinetheme.com/palette/ (base faf4ed, overlay f2e9e1, text 575279, subtle 797593, highlight med dfdad9, pine 286983, foam 56949f, iris 907aa9, gold ea9d34 darkened for contrast, love b4637a). The palette has no green: `success` is foam.
pub(super) fn rose_pine_dawn() -> ThemeSpec {
    derive(&Pal {
        slug: "rose-pine-dawn",
        name: "Rosé Pine Dawn",
        light: true,
        syntax: "InspiredGitHub",
        lanes: "rose-pine-dawn",
        bg: hex(0xfaf4ed),
        panel: hex(0xf2e9e1),
        text: hex(0x575279),
        muted: hex(0x797593),
        accent: hex(0x286983),
        selection: hex(0xdfdad9),
        success: hex(0x56949f),
        warning: hex(0xc07a12),
        danger: hex(0xb4637a),
        ai: hex(0x907aa9),
        cyan: hex(0x286983),
    })
}

/// Catppuccin Macchiato, https://github.com/catppuccin/palette (base 24273a, surface0 363a4f, text cad3f5, subtext0 a5adcb, blue 8aadf4, green a6da95, yellow eed49f, red ed8796, mauve c6a0f6, teal 8bd5ca); the panel sits between base and mantle like the Mocha one.
pub(super) fn catppuccin_macchiato() -> ThemeSpec {
    with_side(
        derive(&Pal {
            slug: "catppuccin-macchiato",
            name: "Catppuccin Macchiato",
            light: false,
            syntax: "base16-mocha.dark",
            lanes: "catppuccin-macchiato",
            bg: hex(0x24273a),
            panel: hex(0x272b3f),
            text: hex(0xcad3f5),
            muted: hex(0xa5adcb),
            accent: hex(0x8aadf4),
            selection: hex(0x494d64),
            success: hex(0xa6da95),
            warning: hex(0xeed49f),
            danger: hex(0xed8796),
            ai: hex(0xc6a0f6),
            cyan: hex(0x8bd5ca),
        }),
        hex(0x1e2030),
    )
}

/// Catppuccin Frappé, https://github.com/catppuccin/palette (base 303446, surface0 414559, text c6d0f5, subtext0 a5adce, blue 8caaee, green a6d189, yellow e5c890, red e78284, mauve ca9ee6, teal 81c8be); the panel sits between base and mantle like the Mocha one.
pub(super) fn catppuccin_frappe() -> ThemeSpec {
    with_side(
        derive(&Pal {
            slug: "catppuccin-frappe",
            name: "Catppuccin Frappé",
            light: false,
            syntax: "base16-mocha.dark",
            lanes: "catppuccin-frappe",
            bg: hex(0x303446),
            panel: hex(0x2d3143),
            text: hex(0xc6d0f5),
            muted: hex(0xa5adce),
            accent: hex(0x8caaee),
            selection: hex(0x51576d),
            success: hex(0xa6d189),
            warning: hex(0xe5c890),
            danger: hex(0xe78284),
            ai: hex(0xca9ee6),
            cyan: hex(0x81c8be),
        }),
        hex(0x292c3c),
    )
}

/// Kanagawa Lotus, https://github.com/rebelot/kanagawa.nvim (lotusWhite3 f2ecbc, lotusWhite2 e5ddb0, lotusWhite0 d5cea3, lotusInk1 545464, lotusGray3 716e61, lotusBlue4 4d699b, lotusGreen 6f894e, lotusYellow 77713f, lotusRed c84053, lotusViolet4 624c83, lotusAqua 597b75, lotusOrange cc6d00).
pub(super) fn kanagawa_lotus() -> ThemeSpec {
    derive(&Pal {
        slug: "kanagawa-lotus",
        name: "Kanagawa Lotus",
        light: true,
        syntax: "InspiredGitHub",
        lanes: "kanagawa-lotus",
        bg: hex(0xf2ecbc),
        panel: hex(0xe5ddb0),
        text: hex(0x545464),
        muted: hex(0x716e61),
        accent: hex(0x4d699b),
        selection: hex(0xd5cea3),
        success: hex(0x6f894e),
        warning: hex(0x836f4a),
        danger: hex(0xc84053),
        ai: hex(0x624c83),
        cyan: hex(0x597b75),
    })
}

/// Vesper, https://github.com/raunofreiberg/vesper (bg 101010, panel 161616, fg ffffff, comment a0a0a0, selection 343434 (the theme uses white at 15% alpha), peach ffc799, mint 99ffe4, red ff8080). The palette has no blue or purple: the accent is peach, `ai` a soft lilac and `cyan` a muted mint, both derived.
pub(super) fn vesper() -> ThemeSpec {
    derive(&Pal {
        slug: "vesper",
        name: "Vesper",
        light: false,
        syntax: "base16-ocean.dark",
        lanes: "vesper",
        bg: hex(0x101010),
        panel: hex(0x161616),
        text: hex(0xffffff),
        muted: hex(0xa0a0a0),
        accent: hex(0xffc799),
        selection: hex(0x343434),
        success: hex(0x99ffe4),
        warning: hex(0xffc799),
        danger: hex(0xff8080),
        ai: hex(0xb9a4ff),
        cyan: hex(0x7fd1c1),
    })
}

/// Every role is `ansi:N` or `default`, so the terminal's own palette decides
/// what it looks like. Only `blend_bg` (the color tints are mixed into) and
/// the lane palette (the graph picture needs RGB) are RGB.
pub(super) fn terminal() -> ThemeSpec {
    use ThemeColor::{Ansi, Default as D};
    let mut s = derive(&Pal {
        slug: "terminal",
        name: "Terminal (16 colors)",
        light: false,
        syntax: "base16-ocean.dark",
        lanes: "gitpoulpi",
        bg: hex(0x1d2022),
        panel: hex(0x1d2022),
        text: hex(0xe5e5e5),
        muted: hex(0x7f7f7f),
        accent: hex(0x5c5cff),
        selection: hex(0x0000ee),
        success: hex(0x00cd00),
        warning: hex(0xcdcd00),
        danger: hex(0xcd0000),
        ai: hex(0xcd00cd),
        cyan: hex(0x00cdcd),
    });
    let blend = s.ui.blend_bg;
    // The tint, pick and fill roles are all `default` or a 16-color entry.
    s.ui = UiColors {
        bg: D,
        blend_bg: blend,
        panel: D,
        side_bg: D,
        toolbar_bg: D,
        statusbar_bg: D,
        statusbar_fg: D,
        text: D,
        muted: Ansi(8),
        // Grayed toolbar and dialog buttons sit on Ansi(8) chips: the
        // disabled text must not be that same color.
        disabled: Ansi(7),
        accent: Ansi(12),
        on_accent: Ansi(0),
        selection: Ansi(4),
        hover: Ansi(8),
        button: Ansi(8),
        button_hover: Ansi(4),
        on_button: Ansi(15),
        border_focused: Ansi(12),
        border: Ansi(8),
        tooltip_bg: D,
        popup_bg: D,
        header_fg: D,
        input_rest: Ansi(0),
        input_hover: Ansi(8),
        input_focus: Ansi(8),
        text_selection_bg: Ansi(12),
        text_selection_fg: Ansi(0),
    };
    s.status = StatusColors {
        success: Ansi(10),
        warning: Ansi(11),
        danger: Ansi(9),
        info: Ansi(12),
        ai: Ansi(13),
    };
    s.refs = RefColors {
        local: Ansi(10),
        remote: Ansi(12),
        tag: Ansi(11),
    };
    s.scrollbar = ScrollbarColors {
        track: Ansi(0),
        thumb: Ansi(8),
        thumb_hover: Ansi(12),
        marks_add: Ansi(10),
        marks_del: Ansi(9),
        marks_cursor: Ansi(15),
        marks_occurrence: Ansi(11),
        marks_modified: Ansi(3),
        marks_conflict_open: Ansi(11),
        marks_conflict_current: Ansi(12),
    };
    s.minimap = MinimapColors {
        bg: Ansi(0),
        view: Ansi(8),
        ink: Ansi(7),
    };
    s.diff = DiffColors {
        add_fg: Ansi(10),
        del_fg: Ansi(9),
        add_bg: D,
        del_bg: D,
        hunk_fg: Ansi(12),
        gutter_fg: Ansi(8),
        range_bg: Ansi(8),
        search_hit_bg: Ansi(3),
    };
    s.merge = MergeColors {
        ours: Ansi(12),
        theirs: Ansi(11),
        base: Ansi(8),
        edit: Ansi(11),
        text: D,
        ours_tint: D,
        theirs_tint: D,
        ours_pick: Ansi(4),
        theirs_pick: Ansi(3),
        base_pick: Ansi(8),
        current_region: D,
        strip_bg: Ansi(3),
        strip_fg: Ansi(0),
    };
    s.editor = EditorColors {
        text: D,
        current_line: Ansi(0),
        find_hit: Ansi(3),
        find_hit_fg: Ansi(0),
    };
    s.toolbar = ToolbarColors {
        pull: Ansi(12),
        push: Ansi(10),
        stash: Ansi(11),
        branch: Ansi(13),
        lfs: Ansi(2),
        tag: Ansi(3),
        search: Ansi(7),
        undo: Ansi(14),
    };
    s.hosting = HostingColors {
        merged: Ansi(13),
        chip_fg: Ansi(0),
    };
    s.chrome = ChromeColors {
        stat_add: Ansi(2),
        stat_del: Ansi(1),
        notice: Ansi(1),
        marker: Ansi(5),
        on_bright: Ansi(0),
        wip_del: Ansi(9),
        error_bg: Ansi(1),
        banner_bg: Ansi(3),
        banner_danger_bg: Ansi(1),
        toolbar_arrow_bg: Ansi(8),
        pill_branch: Ansi(5),
        pill_onto: Ansi(6),
        action_edit: Ansi(12),
        dropdown_bg: D,
        dropdown_hit: Ansi(14),
        action_reset: Ansi(12),
        tint_reset: D,
        tint_danger: D,
        tint_success: D,
        toast_bg: Ansi(5),
        proposal_bg: Ansi(5),
        confirm_danger: Ansi(1),
        disabled_button_bg: Ansi(8),
        toolbar_bg: Ansi(8),
        toolbar_bg_hover: Ansi(4),
        toolbar_fg: Ansi(15),
        // Not `muted` (Ansi 8): the chips themselves are Ansi 8.
        toolbar_disabled_fg: Ansi(7),
    };
    s.graph = GraphColors {
        wip: Ansi(8),
        stash: Ansi(8),
    };
    s
}

/// GitHub Dark ("Dark default"), https://github.com/primer/github-vscode-theme with @primer/primitives colors/dark.json: canvas.default 0d1117, canvas.inset 010409 (the theme's `sideBar.background`, here `side_bg`), canvas.subtle 161b22 (panel), fg.default e6edf3, fg.muted 848d97, accent.fg 2f81f7, success 3fb950, attention d29922, danger f85149, done a371f7, ansi cyan 39c5cf. The selection is accent.muted (388bfd at 40%) over canvas.default. No color is adjusted.
pub(super) fn github_dark() -> ThemeSpec {
    with_chrome(
        with_side(
            derive(&Pal {
                slug: "github-dark",
                name: "GitHub Dark",
                light: false,
                syntax: "base16-ocean.dark",
                lanes: "github-dark",
                bg: hex(0x0d1117),
                panel: hex(0x161b22),
                text: hex(0xe6edf3),
                muted: hex(0x848d97),
                accent: hex(0x2f81f7),
                selection: hex(0x1e4273),
                success: hex(0x3fb950),
                warning: hex(0xd29922),
                danger: hex(0xf85149),
                ai: hex(0xa371f7),
                cyan: hex(0x39c5cf),
            }),
            hex(0x010409),
        ),
        hex(0x161b22),
        hex(0x010409),
        hex(0xe6edf3),
    )
}

/// GitHub Light ("Light default"), https://github.com/primer/github-vscode-theme with @primer/primitives colors/light.json: canvas.default ffffff, canvas.inset f6f8fa (`sideBar.background`, here `side_bg`), canvas.subtle f6f8fa (panel), fg.default 1f2328, fg.muted 656d76, accent.fg 0969da, success 1a7f37, attention 9a6700, danger d1242f, done 8250df, ansi cyan 1b7c83. The selection is accent.subtle ddf4ff (the theme's own neutral.muted selection is too faint on the sidebar). No color is adjusted.
pub(super) fn github_light() -> ThemeSpec {
    with_chrome(
        with_side(
            derive(&Pal {
                slug: "github-light",
                name: "GitHub Light",
                light: true,
                syntax: "InspiredGitHub",
                lanes: "github-light",
                bg: hex(0xffffff),
                panel: hex(0xf6f8fa),
                text: hex(0x1f2328),
                muted: hex(0x656d76),
                accent: hex(0x0969da),
                selection: hex(0xddf4ff),
                success: hex(0x1a7f37),
                warning: hex(0x9a6700),
                danger: hex(0xd1242f),
                ai: hex(0x8250df),
                cyan: hex(0x1b7c83),
            }),
            hex(0xf6f8fa),
        ),
        hex(0xeaeef2),
        hex(0xf6f8fa),
        hex(0x1f2328),
    )
}

/// VS Code "Dark Modern" (the default dark theme), https://github.com/microsoft/vscode/blob/main/extensions/theme-defaults/themes/dark_modern.json: editor.background 1f1f1f, sideBar.background 181818 (`side_bg`), foreground cccccc, descriptionForeground 9d9d9d, focusBorder 0078d4, editorGutter added 2ea043, errorForeground f85149. VS Code defaults (colorRegistry): list.activeSelectionBackground 04395e, editorWarning cca700, charts.purple b180d7, terminal.ansiBrightCyan 29b8db; the panel is the menu.background of Dark+ 252526. No color is adjusted.
pub(super) fn vscode_dark() -> ThemeSpec {
    with_chrome(
        with_side(
            derive(&Pal {
                slug: "vscode-dark",
                name: "VS Code Dark",
                light: false,
                syntax: "base16-ocean.dark",
                lanes: "vscode-dark",
                bg: hex(0x1f1f1f),
                panel: hex(0x252526),
                text: hex(0xcccccc),
                muted: hex(0x9d9d9d),
                accent: hex(0x0078d4),
                selection: hex(0x04395e),
                success: hex(0x2ea043),
                warning: hex(0xcca700),
                danger: hex(0xf85149),
                ai: hex(0xb180d7),
                cyan: hex(0x29b8db),
            }),
            hex(0x181818),
        ),
        hex(0x2b2b2b),
        hex(0x007acc),
        hex(0xffffff),
    )
}

/// VS Code "Light Modern" (the default light theme), https://github.com/microsoft/vscode/blob/main/extensions/theme-defaults/themes/light_modern.json: editor.background ffffff, sideBar.background f8f8f8 (`side_bg`, also the widget/panel color), foreground 3b3b3b, focusBorder 005fb8. VS Code defaults (colorRegistry): list.inactiveSelectionBackground e4e6f1 (Light Modern's own e8e8e8 is invisible on the sidebar), editorError e51400, charts.green 388a34, charts.yellow bf8803, charts.purple 652d90, terminal.ansiCyan 0598bc. The muted gray 616161 replaces descriptionForeground 3b3b3b (same as the text, so a muted line would not read as muted).
pub(super) fn vscode_light() -> ThemeSpec {
    with_chrome(
        with_side(
            derive(&Pal {
                slug: "vscode-light",
                name: "VS Code Light",
                light: true,
                syntax: "InspiredGitHub",
                lanes: "vscode-light",
                bg: hex(0xffffff),
                panel: hex(0xf8f8f8),
                text: hex(0x3b3b3b),
                muted: hex(0x616161),
                accent: hex(0x005fb8),
                selection: hex(0xe4e6f1),
                success: hex(0x388a34),
                warning: hex(0xbf8803),
                danger: hex(0xe51400),
                ai: hex(0x652d90),
                cyan: hex(0x0598bc),
            }),
            hex(0xf8f8f8),
        ),
        hex(0xdddddd),
        hex(0x007acc),
        hex(0xffffff),
    )
}
