//! The `ui` part of the config schema.

use serde::{Deserialize, Serialize};

use super::log::MIB_OR_GIB;
use super::quantity::{read_quantity, DisplayUnit};

config_enum! {
/// What decides a commit's color in the graph.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphColorization {
    /// One color per lane, cycling - what the graph does today.
    #[default]
    Lane => "lane",
    /// One color per author.
    Author => "author",
    /// One color per branch head the commit is reachable from.
    Branch => "branch",
    /// No colorization; every lane in the primary foreground.
    None => "none",
}
}

config_enum! {
/// How the Graph column reaches the screen.
///
/// On a terminal that implements an inline-image protocol the column is
/// drawn as **pixels**, the only rendering where a round commit marker
/// actually joins its lane (see `gitpoulpi_tui::graph::raster`); elsewhere
/// it is drawn as characters.
///
/// `auto` is **deny-by-default**: the terminal must identify itself as one
/// known to implement the protocol *and* there must be no sign of a
/// multiplexer in between. A multiplexer inherits its parent's environment
/// (`herdr`, `tmux`, `zellij` and `screen` still say "WezTerm") but does
/// not pass the escape through, so the base64 payload is echoed across the
/// pane as garbage. `image` forces pictures on for a multiplexer that does
/// pass them through; `text` forces them off for a terminal that lies the
/// other way.
///
/// `GITPOULPI_TERMINAL_IMAGE=off` is a separate, lower-level escape hatch
/// that still wins over everything here (the pty test harness sets it).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphRenderer {
    /// Draw pixels when, and only when, the environment says this terminal
    /// can display them and nothing is sitting between us and it.
    #[default]
    Auto => "auto",
    /// Always emit the inline image.
    Image => "image",
    /// Always draw characters.
    Text => "text",
}
}

config_enum! {
/// Whether to ask the terminal for the kitty keyboard protocol
/// (`ui.kitty_keyboard`). With it `Ctrl+,`, `Ctrl+Shift+N`, `Ctrl+Enter`
/// and `Ctrl+<digit>` arrive as distinct keys instead of colliding with
/// legacy control codes.
///
/// `auto` asks at start-up (a bounded query of about 100 ms at most) and
/// uses the protocol only when the terminal answers that it supports it;
/// anything that does not answer keeps the legacy keys, unchanged. `off`
/// never asks and never pushes anything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KittyKeyboard {
    /// Use it when the terminal supports it.
    #[default]
    Auto => "auto",
    /// Never touch the terminal's keyboard mode.
    Off => "off",
}
}

config_enum! {
/// How many colors the terminal is drawn with (`ui.color_depth`).
///
/// Themes are truecolor (`#rrggbb`). `auto` keeps truecolor except on
/// terminals known not to render it (Apple Terminal, the Linux console,
/// a `*-256color` terminal that sets no `COLORTERM`), where every RGB
/// color is mapped to the nearest xterm-256 (or 16) entry on the way out.
/// The other three force a depth.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorDepth {
    /// Detect it from the environment.
    #[default]
    Auto => "auto",
    /// Always 24-bit color.
    Truecolor => "truecolor",
    /// The xterm 256-color palette.
    Depth256 => "256",
    /// The 16 ANSI colors.
    Depth16 => "16",
}
}

config_enum! {
/// Which terminal image protocol pictures (the Graph column, author
/// avatars) are sent with. `ui.image_protocol`.
///
/// `auto` picks from the environment: the Kitty graphics protocol in
/// kitty and Ghostty, and inside herdr when herdr's own config turns its
/// `kitty_graphics` on; iTerm2's inline images (`OSC 1337`) in WezTerm,
/// iTerm2, mintty, Rio; nothing (the text rendering) anywhere else. The
/// three manual values exist for the case detection cannot see: a herdr
/// whose config file is somewhere else, a terminal that lies, or a user
/// comparing the two protocols in WezTerm, which speaks both.
///
/// `ui.graph_renderer = "text"` and `GITPOULPI_TERMINAL_IMAGE=off` still
/// win over an explicit protocol here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageProtocol {
    /// Detect it from the environment.
    #[default]
    Auto => "auto",
    /// iTerm2 inline images (`OSC 1337 ; File=`).
    Iterm2 => "iterm2",
    /// The Kitty graphics protocol, placed with Unicode placeholders.
    Kitty => "kitty",
    /// Sixel (`DCS q`): Windows Terminal, foot, Konsole, mlterm, a VS Code
    /// terminal with `terminal.integrated.enableImages`.
    Sixel => "sixel",
    /// No pictures at all: the text renderings everywhere.
    None => "none",
}
}

config_enum! {
/// Whether the Graph column's lane cells carry a background in the lane's
/// color, and how strongly.
///
/// **Off by default, deliberately.** A glyph is drawn inside side bearings
/// and cannot reach its cell's top and bottom edges, leaving a few pixels
/// of bare cell between a round commit marker and its lane; a background
/// closes that gap but reads as a wide tinted ribbon behind the commits.
/// It stays a setting because the complaint was partly about *intensity*,
/// and the real fix is elsewhere: on a terminal that displays images the
/// column is drawn as pixels (see `gitpoulpi_tui::graph::raster`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum GraphLaneBackground {
    /// No background anywhere: the Graph column is transparent, so the
    /// terminal's own background (and the table's selection / hover
    /// band) shows through behind the lanes.
    #[default]
    Off => "off",
    /// Each commit row gets a faint band of its commit's
    /// lane color inside the Graph column, from the node rightwards.
    RowTint => "row_tint",
}
}

impl GraphLaneBackground {
    /// The label the Settings combo shows (the stored value stays `as_str`).
    pub fn label(self) -> &'static str {
        match self {
            GraphLaneBackground::Off => "Off",
            GraphLaneBackground::RowTint => "Row tint",
        }
    }

    pub fn from_label(label: &str) -> Option<GraphLaneBackground> {
        Self::ALL.iter().copied().find(|b| b.label() == label)
    }
}

/// Reads the current names and every older spelling: the booleans
/// (`true` = row tint, `false` = off) and the dropped `lane` level and its
/// earlier `subtle` / `strong` names (all migrate to `row_tint`).
impl<'de> Deserialize<'de> for GraphLaneBackground {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Name(String),
        }
        match Raw::deserialize(deserializer)? {
            Raw::Bool(true) => Ok(GraphLaneBackground::RowTint),
            Raw::Bool(false) => Ok(GraphLaneBackground::Off),
            Raw::Name(name) => match name.as_str() {
                "off" => Ok(GraphLaneBackground::Off),
                "lane" | "subtle" | "strong" | "row_tint" => Ok(GraphLaneBackground::RowTint),
                other => Err(serde::de::Error::unknown_variant(
                    other,
                    &["off", "row_tint"],
                )),
            },
        }
    }
}

config_enum! {
/// How strong the `row_tint` band is. `ui.graph_row_tint_strength`.
///
/// Only the band fades; the thin lane-color bar at the column's right
/// edge always stays at full color.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphRowTintStrength {
    /// The original blend: the most visible.
    Strong => "strong",
    /// A lighter band (the default).
    #[default]
    Medium => "medium",
    /// The faintest band.
    Subtle => "subtle",
}
}

impl GraphRowTintStrength {
    /// The label the Settings combo shows (the stored value stays `as_str`).
    pub fn label(self) -> &'static str {
        match self {
            GraphRowTintStrength::Strong => "Strong",
            GraphRowTintStrength::Medium => "Medium",
            GraphRowTintStrength::Subtle => "Subtle",
        }
    }

    pub fn from_label(label: &str) -> Option<GraphRowTintStrength> {
        Self::ALL.iter().copied().find(|b| b.label() == label)
    }
}

config_enum! {
/// Which build a GitPoulpi shortcut starts, `ui.launcher_build`. Only
/// honoured in dev mode (`general.dev_mode`); otherwise `Current`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LauncherBuild {
    /// The build that made the shortcut (the running exe). The default.
    #[default]
    Current => "current",
    /// The most recently built of `target/dist` and `target/release` of the
    /// checkout, decided when the shortcut is clicked.
    Newest => "newest",
}
}

config_enum! {
/// What the active tab's folder gets you when it is not the folder
/// GitPoulpi was launched from. `ui.tab_dir_follow`.
///
/// A program cannot change its parent shell's directory, so "following" a
/// tab means two things: the shell wrapper `gitpoulpi shell-init` prints
/// `cd`s there when GitPoulpi quits, and a terminal / multiplexer pane can
/// be opened in it (`ui.terminal_launch`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabDirFollow {
    /// Show the "Quit and cd here" / "Open terminal here" buttons in the tab
    /// bar while the active tab is not the launch folder. The default.
    #[default]
    Buttons => "buttons",
    /// No buttons: quitting always `cd`s to the active tab (when started
    /// through the wrapper) and a terminal opens by itself whenever the user
    /// opens a new tab (never for the tabs restored at start-up).
    Auto => "auto",
}
}

config_enum! {
/// Where the toolbar (Undo, Pull, Push...) sits: `ui.toolbar_position`.
/// `left` and `right` make it a vertical column beside the panes; a
/// terminal too narrow for the column draws it at the bottom instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarPosition {
    /// A row under the repository tabs, above the panes.
    Top => "top",
    /// A row under the panes, above the status line. The default.
    #[default]
    Bottom => "bottom",
    /// A column left of the panes.
    Left => "left",
    /// A column right of the panes.
    Right => "right",
}
}

config_enum! {
/// Where the toolbar's buttons sit along its length: `ui.toolbar_align`.
/// "Start" is the left of a row and the top of a column; "end" the right
/// and the bottom.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarAlign {
    /// Packed at the start. The default.
    #[default]
    Start => "start",
    /// Centered.
    Center => "center",
    /// Packed at the end.
    End => "end",
}
}

config_enum! {
/// Whether a dialog the user moved or resized (drag its title bar or its
/// edges, or `Ctrl+N m`) keeps that place: `ui.dialog_geometry`. One setting
/// for every dialog; each dialog keeps its own place and size under it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogGeometry {
    /// Kept while GitPoulpi runs, forgotten at the next launch. The default.
    #[default]
    Session => "session",
    /// Kept on disk and restored at the next launch.
    Saved => "saved",
    /// Every dialog opens at its default place and size.
    Never => "never",
}
}

config_enum! {
/// Where the status line (branch, counts, messages, key hints) sits:
/// `ui.statusbar_position`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusbarPosition {
    /// The first row of the window, above the repository tabs.
    Top => "top",
    /// The last row of the window. The default.
    #[default]
    Bottom => "bottom",
}
}

config_enum! {
/// Where the repository tab bar sits: `ui.tabs_position`. `left` makes it a
/// vertical list at the window's left edge; a terminal under 80 columns
/// draws it at the top instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabsPosition {
    /// A row under the window's title, above everything else. The default.
    #[default]
    Top => "top",
    /// A row at the very bottom of the window, below the status line.
    Bottom => "bottom",
    /// A column at the left edge, one tab per row.
    Left => "left",
}
}

impl ToolbarPosition {
    /// Whether the toolbar is a column rather than a row.
    pub fn is_vertical(self) -> bool {
        matches!(self, ToolbarPosition::Left | ToolbarPosition::Right)
    }
}

/// Reads a theme or lane palette slug, replacing a retired one
/// by its current name, so an old `config.toml` keeps its look and the next
/// save writes the new slug.
fn retired_slug<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let raw = String::deserialize(d)?;
    Ok(crate::theme::canonical_slug(&raw).to_string())
}

/// `ui.graph_color_theme` value meaning "use the active theme's own lane
/// palette" (the default). Any other value is a lane palette slug from
/// `gitpoulpi_config::theme::lane_palettes_with` (the historical names
/// `gitpoulpi`, `serie` and the renamed ones included, so old files keep
/// their look).
pub const GRAPH_COLOR_FOLLOW_THEME: &str = "theme";

/// `ui.theme` value meaning "follow the terminal" (Dark until terminal
/// detection lands). `"default"` is its legacy spelling.
pub const THEME_SYSTEM: &str = "system";

config_enum! {
/// The Diff view's layout: one column or two side by side (`m`, the
/// bar's Inline / Split buttons, or the palette). The other axis - the
/// whole file or only the hunks - is `UiConfig::diff_hunks_only`.
/// The config-side twin of `gitpoulpi_tui::diff_view::DiffViewMode` (which
/// stays in the TUI because it also drives rendering); the TUI converts
/// at one seam, `impl From<DiffLayout> for DiffViewMode`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLayout {
    /// `+`/`-` and context interleaved in one column.
    #[default]
    Inline => "inline",
    /// Before and after side by side.
    Split => "split",
}
}

config_enum! {
/// What the TUI calls a hosted code-review request: `ui.pr_terminology`.
///
/// GitHub says "pull request", GitLab says "merge request". `auto`
/// follows the linked host; the other two force one word everywhere
/// (labels, menus, palette, messages).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrTerminology {
    /// "Pull Request" for GitHub, "Merge Request" for GitLab.
    #[default]
    Auto => "auto",
    PullRequest => "pull_request",
    MergeRequest => "merge_request",
}
}

impl PrTerminology {
    /// The label the Settings combo shows.
    pub fn label(self) -> &'static str {
        match self {
            PrTerminology::Auto => "Auto (per host)",
            PrTerminology::PullRequest => "Pull Request",
            PrTerminology::MergeRequest => "Merge Request",
        }
    }

    pub fn from_label(label: &str) -> Option<PrTerminology> {
        Self::ALL.iter().copied().find(|t| t.label() == label)
    }
}

config_enum! {
/// The display language of the TUI: `ui.language`. `auto` follows the OS
/// locale (`LC_ALL` / `LANG` on Unix, the user locale on Windows and macOS),
/// falling back to English. `GITPOULPI_LANG` beats this setting. The values
/// are BCP 47 tags; a language whose translation file is not complete yet
/// shows English until it ships.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiLanguage {
    /// The OS locale, else English.
    #[default]
    Auto => "auto",
    En => "en",
    Fr => "fr",
    Es => "es",
    PtBr => "pt-BR",
    PtPt => "pt-PT",
    It => "it",
    De => "de",
    ZhCn => "zh-CN",
    Ja => "ja",
    Ru => "ru",
    Ko => "ko",
    Nl => "nl",
    Pl => "pl",
    Uk => "uk",
    Cs => "cs",
    Sv => "sv",
    Tr => "tr",
}
}

impl UiLanguage {
    /// The language's own name (an endonym, so a user lost in a language
    /// they cannot read still finds theirs). `Auto` has no name of its own.
    pub fn endonym(self) -> &'static str {
        match self {
            UiLanguage::Auto => "Auto",
            UiLanguage::En => "English",
            UiLanguage::Fr => "Fran\u{e7}ais",
            UiLanguage::Es => "Espa\u{f1}ol",
            UiLanguage::PtBr => "Portugu\u{ea}s (Brasil)",
            UiLanguage::PtPt => "Portugu\u{ea}s (Portugal)",
            UiLanguage::It => "Italiano",
            UiLanguage::De => "Deutsch",
            UiLanguage::ZhCn => "\u{7b80}\u{4f53}\u{4e2d}\u{6587}",
            UiLanguage::Ja => "\u{65e5}\u{672c}\u{8a9e}",
            UiLanguage::Ru => "\u{420}\u{443}\u{441}\u{441}\u{43a}\u{438}\u{439}",
            UiLanguage::Ko => "\u{d55c}\u{ad6d}\u{c5b4}",
            UiLanguage::Nl => "Nederlands",
            UiLanguage::Pl => "Polski",
            UiLanguage::Uk => {
                "\u{423}\u{43a}\u{440}\u{430}\u{457}\u{43d}\u{441}\u{44c}\u{43a}\u{430}"
            }
            UiLanguage::Cs => "\u{10c}e\u{161}tina",
            UiLanguage::Sv => "Svenska",
            UiLanguage::Tr => "T\u{fc}rk\u{e7}e",
        }
    }
}

/// The legacy `ui.diff_view_mode` key, read only to migrate it.
///
/// Its three values all showed *only the hunks* (`hunk` trimmed context to
/// one line, `inline`/`split` used git's three), so every value maps to
/// `diff_hunks_only = true` and `hunk`/`inline` both become the one-column
/// layout. Never written back (`skip_serializing` on the field), so
/// `Config::save` drops it and `Config::get` does not list it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyDiffViewMode {
    Hunk,
    Inline,
    Split,
}

impl LegacyDiffViewMode {
    pub fn layout(self) -> DiffLayout {
        match self {
            LegacyDiffViewMode::Hunk | LegacyDiffViewMode::Inline => DiffLayout::Inline,
            LegacyDiffViewMode::Split => DiffLayout::Split,
        }
    }

    /// Every old mode showed only the hunks (see the type's doc comment).
    pub fn hunks_only(self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Live: the display language. See [`UiLanguage`]; `auto` is the
    /// default. `GITPOULPI_LANG` wins over it.
    pub language: UiLanguage,
    /// Live: git's own words (commit, push, pull, stash, tag...) stay
    /// English by default in every language; on, they use the translated
    /// vocabulary of the chosen language (fr "remiser", "\u{e9}tiquette",
    /// "valider"). Off by default. Right below `language` in Settings.
    pub translate_git_terms: bool,
    /// Live: only for ja, ko and zh-CN. On, the git words that those languages
    /// would transliterate (stage/unstage...) show the English git word
    /// instead. No effect while `translate_git_terms` is on. Off by default.
    pub git_terms_english: bool,
    /// Live: `false` leaves the terminal's own selection/copy behavior
    /// alone by never entering mouse-capture mode (see `gitpoulpi_tui::run`).
    pub mouse_enabled: bool,
    /// Live: the color theme, as a slug: `system` (default; follows the
    /// terminal, Dark until detection lands), a preset (`dark`, `light`,
    /// `catppuccin-mocha`, `nord`...) or a user theme file under `<config>/themes/`.
    /// The legacy spelling `default` means `system`.
    #[serde(deserialize_with = "retired_slug")]
    pub theme: String,
    pub graph_colorization: GraphColorization,
    /// Live: read by the Log's graph renderer. See
    /// [`GraphLaneBackground`]; `off` is the default.
    pub graph_lane_background: GraphLaneBackground,
    /// Live: where the toolbar sits. See [`ToolbarPosition`]; `bottom` is
    /// the default.
    pub toolbar_position: ToolbarPosition,
    /// Live: where the toolbar's buttons sit along it. See [`ToolbarAlign`];
    /// `start` is the default.
    pub toolbar_align: ToolbarAlign,
    /// Live: whether a moved or resized dialog keeps its place. See
    /// [`DialogGeometry`]; `session` is the default.
    pub dialog_geometry: DialogGeometry,
    /// Live: the linked pull request (merge request) button is pinned to the
    /// toolbar's far end (right, or bottom in a column) instead of flowing
    /// with the other buttons, and **on by default**.
    pub toolbar_pr_pinned: bool,
    /// Live: where the status line sits. See [`StatusbarPosition`]; `bottom`
    /// is the default.
    pub statusbar_position: StatusbarPosition,
    /// Live: where the repository tab bar sits. See [`TabsPosition`]; `top`
    /// is the default.
    pub tabs_position: TabsPosition,
    /// Live: how faded the `row_tint` band is. See [`GraphRowTintStrength`];
    /// `medium` is the default. Ignored when the lane background is `off`.
    pub graph_row_tint_strength: GraphRowTintStrength,
    /// Live: the Log graph's lane color palette: `theme` (default, the
    /// active theme's own palette) or any lane palette slug. See
    /// [`GRAPH_COLOR_FOLLOW_THEME`].
    #[serde(deserialize_with = "retired_slug")]
    pub graph_color_theme: String,
    /// Live: the Refs sidebar's LOCAL/REMOTE sections show branch names
    /// as a folder tree, splitting on `/`, with each remote as its own
    /// top-level node, and **on by default**.
    /// `false` gives one flat row per branch. `y` in the Refs pane toggles it in the
    /// running TUI and writes the choice back here.
    pub ref_tree: bool,
    /// Live: read once at start-up by `gitpoulpi_tui::run`. See
    /// [`GraphRenderer`]; `auto` is the default.
    pub graph_renderer: GraphRenderer,
    /// Live: read once at start-up by `gitpoulpi_tui::run`, next to
    /// `graph_renderer`. See [`ImageProtocol`]; `auto` is the default.
    pub image_protocol: ImageProtocol,
    /// Live: read once at start-up by `gitpoulpi_tui::run`. See
    /// [`KittyKeyboard`]; `auto` is the default.
    pub kitty_keyboard: KittyKeyboard,
    /// Live: how many colors to draw with. See [`ColorDepth`]; `auto` is
    /// the default.
    pub color_depth: ColorDepth,
    /// Live: the Log draws each stash as a row of its own -
    /// a small box on a lane beside the commit it was made on, joined
    /// to it by a dashed line. **On by default.** Off, stashes are only in
    /// the Refs sidebar's STASHES section.
    pub graph_show_stashes: bool,
    /// Live: the Log draws a thin line from each ref pill in the Ref
    /// column to its commit's node in the Graph column -
    /// `─` over blank cells in the text graph, a
    /// one-pixel line under the lanes in the image graph. **On by
    /// default.** Only drawn while the Ref column sits immediately left of
    /// the Graph column.
    pub ref_connector: bool,
    /// Live: while the Log's Changes column is hidden, the WIP row shows its
    /// change counts (`✎ 3  + 2  − 1`) in its message. **On by default.**
    /// With the Changes column visible the counts always go in that column
    /// instead and this setting has no effect.
    pub log_wip_counts_in_message: bool,
    /// Live: a single click on a branch, tag, remote branch or stash in the
    /// Refs pane scrolls the Log to that ref's commit and flashes its row,
    /// without moving the commit selection. **On by default.** The
    /// `Reveal in graph` menu item, palette command and key work either way.
    pub reveal_on_ref_click: bool,
    /// Live: the Log's Ref column shows a faded "ghost" pill naming the ref a
    /// hovered commit belongs to (deduced from the history) when the commit
    /// has no ref of its own; hovering the pill shows its kind. It is not a
    /// button. On by default.
    pub graph_ghost_ref: bool,
    /// Live: the master switch "Allow AI agents to drive the view". On by
    /// default. Off, this TUI does not attach to the daemon's viewer
    /// registry, so `gitpoulpi ui show ...` and the MCP `ui_*` tools find
    /// no window to move. Read when a window opens.
    pub allow_agent_view_control: bool,
    /// How to open a GitPoulpi window when an agent asks to show something
    /// (`gitpoulpi ui show ... --launch`) and none is open anywhere:
    /// `"off"` (the default: never launch), a preset id from
    /// [`crate::launch::PRESETS`] (`kitty`, `tmux`, ...) or `"custom"`,
    /// which uses [`UiConfig::launch_command`].
    pub launch: String,
    /// The custom launch template (argv-split, no shell): placeholders
    /// `{repo}`, `{session}` and `{gitpoulpi}`. Used when `launch =
    /// "custom"`.
    pub launch_command: String,
    /// Live: what the active tab's folder gets you when it is not the
    /// launch folder. See [`TabDirFollow`]. `"buttons"` by default.
    pub tab_dir_follow: TabDirFollow,
    /// Live: how "Open terminal here" opens a terminal or multiplexer pane
    /// in a tab's folder, running the user's own shell: `"auto"` (the
    /// default: the preset of `ui.launch` when one is set, else `$TERMINAL`,
    /// `xdg-terminal-exec` or `x-terminal-emulator` on Linux, Terminal on
    /// macOS, Windows Terminal or a console window on Windows), a preset id
    /// from [`crate::launch::PRESETS`], or `"custom"`, which uses
    /// [`UiConfig::terminal_launch_command`].
    pub terminal_launch: String,
    /// The custom template for `terminal_launch = "custom"` (argv-split, no
    /// shell): the placeholder is `{repo}`, the tab's folder. The program is
    /// also started with that folder as its working directory.
    pub terminal_launch_command: String,
    /// Live: the "Open in" launcher, `[[ui.open_in]]` (label + command, see
    /// [`crate::open_in`]). `None` - the key is not in the file - means "not
    /// configured yet": the list is computed from the legacy
    /// `terminal_launch*` keys and what is detected, which is what makes the
    /// migration lossless. The first save from Settings writes the list.
    /// `Some(vec![])` is a deliberately empty list. `None` serialises as
    /// `null` in JSON and is absent from TOML.
    pub open_in: Option<Vec<crate::open_in::OpenInEntry>>,
    /// Live: the label of the entry "Open in default" and `tab_dir_follow =
    /// "auto"` run. Empty (or a label no entry has): the first entry.
    pub open_in_default: String,
    /// Live: the terminal GitPoulpi's shortcuts (launcher entry, Start menu,
    /// Desktop) and the Windows double-click relaunch start it in. `auto`
    /// (the default) picks from what is installed, `default` the OS's own
    /// terminal, else a terminal id (`windows-terminal`, `kitty`, ...; see
    /// `gitpoulpi_config::desktop::terminals`).
    pub launch_terminal: String,
    /// Live, dev mode only (`general.dev_mode`): which build the shortcuts
    /// and the command alias start, see [`LauncherBuild`]. Not offered in
    /// Settings outside dev mode, where it has no effect.
    pub launcher_build: LauncherBuild,
    /// Live, Windows only: started by a double-click into a console of its
    /// own, relaunch inside `launch_terminal` and exit. On by default.
    pub relaunch_in_terminal: bool,
    /// Live: a click on a GitPoulpi shortcut (`gitpoulpi launch`) shows the
    /// window of a GitPoulpi that is already running instead of starting
    /// another one. On by default; off: every click starts a new window.
    pub reuse_window: bool,
    /// Live: the name of the command that starts GitPoulpi (`gp`, or any
    /// name without spaces or path separators). Empty (the default): none.
    /// Settings > Launcher creates and removes it; see
    /// `gitpoulpi_config::desktop::alias`.
    pub command_alias: String,
    /// Live: the name of the second command, which opens the current folder
    /// as a new tab of the GitPoulpi that is already running (`gpa`, running
    /// `gitpoulpi --add`). Empty (the default): none. Created and removed
    /// with `command_alias`; see `gitpoulpi_config::desktop::alias::Kind`.
    pub command_alias_add: String,
    /// Live: whether a binary file that is an image shows as a picture in the
    /// Diff view and the File View instead of the "binary" notice. On by
    /// default.
    pub image_view: bool,
    /// Live: images bigger than this many MiB are not decoded for the
    /// preview (an info card shows instead). `32` by default.
    pub image_view_max_mib: u32,
    /// Live: the unit Settings shows `image_view_max_mib` in (`auto`, `mib`,
    /// `gib`). Display only. See [`DisplayUnit`].
    pub image_view_max_unit: DisplayUnit,
    /// Live: what the image view draws on a terminal without an inline-image
    /// protocol. See [`ImageTextFallback`].
    pub image_view_text_fallback: ImageTextFallback,
    /// Live: the zoom from which the image view stops smoothing and shows
    /// square pixels. See [`ImagePixelatedFrom`].
    pub image_view_pixelated_from: ImagePixelatedFrom,
    /// Live: whether the before and after panes of the image diff share one
    /// zoom and scroll position.
    pub image_view_sync: bool,
    /// Live: the "Edit in" image editors, `[[ui.image_editors]]` (label +
    /// command with `${file}`, see [`crate::image_editors`]). `None` - the
    /// key is not in the file - means "not configured yet": the list is the
    /// editors detected on this machine, then the OS default app.
    /// `Some(vec![])` is a deliberately empty list.
    pub image_editors: Option<Vec<crate::open_in::OpenInEntry>>,
    /// Live: the label of the editor "Edit in default" runs. Empty (or a
    /// label no entry has): the first entry.
    pub image_editor_default: String,
    /// Live: the About window's logo colorway, a slug of
    /// `assets/logo/colourways.json`. Empty (or an unknown slug): the build's
    /// default colorway. Written when the logo is clicked or a color picked.
    pub logo_color: String,
    /// Live: whether the Log row and the commit-detail pane show the
    /// author's Gravatar avatar. **Off by default, deliberately**: turning
    /// it on sends a hash of each author's email to gravatar.com over
    /// HTTPS, which discloses *who is looking at which commit author* (a
    /// hash of a guessable email is not anonymous).
    ///
    /// The hash is **SHA-256**, not MD5: Gravatar's current documentation
    /// requires SHA-256 of the trimmed, lowercased email (API v3) and says
    /// not to use MD5. See `gitpoulpi_tui::avatar::gravatar` for the URL
    /// and `docs/user/config-reference.md` for the same warning in prose.
    /// When off, when the terminal cannot draw images, or when a fetch
    /// fails, a colored initials badge is shown and no request is made.
    ///
    /// **Legacy**: superseded by [`UiConfig::avatars`] and still read when
    /// that key is absent (`true` -> `author`, `false` -> `off`). Read
    /// [`UiConfig::avatar_placement`], never this field directly.
    pub show_avatars: bool,
    /// Live: where author avatars are drawn - `graph` (inside the commit
    /// circle of the image graph renderer), `author`
    /// (a round picture at the start of the Author column) or `off`.
    /// The commit-detail pane shows the picture in both `graph` and
    /// `author`. Same privacy note as `show_avatars` above, plus: with a
    /// github.com remote, one GitHub REST call per author (never per
    /// commit, cached on disk) resolves the author's GitHub avatar.
    ///
    /// `None` - the key is not in the file - means "fall back to the
    /// legacy `show_avatars`", which is what makes the migration
    /// lossless: an old `show_avatars = true` keeps working, and the
    /// first time the Preferences screen writes this key, it wins.
    /// `None` serialises as `null` in JSON (so `config set ui.avatars
    /// graph` finds a settable leaf) and is simply absent from TOML.
    pub avatars: Option<AvatarPlacement>,
    /// Live: which avatar sources are asked, and in which order. See
    /// [`AvatarSources`]. `ui.avatar_sources`.
    pub avatar_sources: AvatarSources,
    /// Live: what an author with no picture anywhere gets. See
    /// [`AvatarFallback`]. `ui.avatar_fallback`.
    pub avatar_fallback: AvatarFallback,
    /// Live: whether the commit composer's proposals open with their files
    /// listed under them (`expanded`, the default) or only the commit rows
    /// (`collapsed`). See [`ComposerFiles`]. `ui.composer_files`.
    pub composer_files: ComposerFiles,
    /// Live: which side of the Blame view the annotation column (avatar,
    /// author, age, summary) sits on. See [`BlameSide`]. `ui.blame_side`.
    pub blame_side: BlameSide,
    /// Live: how the Log, the Radar and the commit column let a column be
    /// resized. See [`ColumnResize`]. `ui.column_resize`.
    pub column_resize: ColumnResize,
    /// Live: how the Log lays its columns out: `fit` (every column fits the
    /// pane, only Message flexes) or `scroll` (columns keep their widths, the
    /// table can be wider than the pane and scrolls sideways). See
    /// [`LogColumnsLayout`]. `ui.log_columns_layout`.
    pub log_columns_layout: LogColumnsLayout,
    /// Live: the annotation column's width in cells, set by dragging its
    /// border (or `<` / `>` in the view). Clamped by
    /// [`UiConfig::blame_width_clamped`]. `ui.blame_width`.
    pub blame_width: u16,
    /// Live: how many rows one mouse-wheel notch moves the commit list.
    /// Three, like the diff view and the File Insight overlay: a one-row
    /// notch is slow and repaints the commit graph on every step. Clamped
    /// by [`UiConfig::mouse_scroll_lines_clamped`]; `0` would make the
    /// wheel do nothing, which looks like a broken mouse.
    pub mouse_scroll_lines: u16,
    /// Live: `system` (default) writes month and weekday names in the
    /// interface language (`ui.language`); `english` keeps them English
    /// whatever the language. Any other value behaves as `system`.
    pub date_locale: String,
    /// The four date/time patterns offered, in the usual token
    /// spelling (`DD`, `MM`, `YYYY`, `D`, `MMMM`, `HH`, `mm`). See
    /// `gitpoulpi_tui::live_settings::format_date`, which is the one place
    /// that interprets them.
    pub date_format: String,
    /// **Legacy**: still parsed so an old `config.toml` loads, read by nothing
    /// and absent from Settings.
    pub date_word_format: String,
    pub date_time_format: String,
    pub date_verbose_format: String,
    /// Live: the Diff view's last-used layout (Inline/Split), hunks-only
    /// toggle, whitespace-only-changes visibility and word wrap - written
    /// back whenever the user changes one, from the keyboard, the mouse or
    /// the palette (`gitpoulpi_tui::lib`'s `persist_diff_view_toggle` is the
    /// one path all three funnel through). Global rather than
    /// per-repository: it is about how the user likes to read a diff,
    /// unlike `RepoPreferences`' encoding/EOL settings, which are
    /// per-repository facts about the files.
    pub diff_layout: DiffLayout,
    /// Live: the word for a hosted review request, see [`PrTerminology`].
    pub pr_terminology: PrTerminology,
    /// Live: hide the toolbar chip of the current branch's latest pull or
    /// merge request when it is closed or merged (the dim "new request" chip
    /// then shows, as if there were none). On by default.
    pub hide_finished_pr_chip: bool,
    /// `true`: only the hunks, with git's context lines around them.
    /// `false`: the whole file, the changes highlighted in place.
    pub diff_hunks_only: bool,
    /// Legacy, read only: migrated into
    /// `diff_layout` / `diff_hunks_only` by [`UiConfig::migrate_legacy`]
    /// when the file does not say those two itself. Never written back.
    #[serde(skip_serializing)]
    pub diff_view_mode: Option<LegacyDiffViewMode>,
    pub diff_show_whitespace: bool,
    pub diff_word_wrap: bool,
    /// Live: a minimap of the whole file beside the vertical scrollbar of
    /// the Diff view and the editor (a pixel picture where the terminal
    /// draws images, braille cells elsewhere). Off by default.
    pub minimap: bool,
    /// Live: what the Log's "Copy as Mermaid gitGraph" popup last used
    /// (`structure`, `refs`, `subjects`, `full`), written back when it
    /// changes. The CLI's `log --format mermaid --detail` defaults to it.
    pub mermaid_detail: gitpoulpi_core::domain::MermaidDetail,
    /// Live: the same popup's direction, `lr` or `tb`.
    pub mermaid_orientation: gitpoulpi_core::domain::MermaidOrientation,
    /// Live: the same popup's "include the mermaid fence" checkbox.
    pub mermaid_fence: bool,
    /// Live: commit subjects are cut to this many characters in the
    /// Mermaid ids (`subjects` and `full` detail).
    pub mermaid_subject_width: u32,
    /// Live: the background of text inputs - `theme` (default: the active
    /// theme's input colors), `gray`, `blue`, `slate`, `dark`, or `custom`
    /// (then `input_color_custom` is used). The named ones are dark colors.
    pub input_color: String,
    /// Live: the `#rrggbb` used when `input_color` is `custom`.
    pub input_color_custom: String,
    /// Live: Log columns the user hid (header right-click menu or palette),
    /// by stable name - `ref`, `graph`, `author`, `date`, `sha`. Global:
    /// applied to every repository tab and kept across restarts. Unknown
    /// names are ignored on load.
    pub log_columns_hidden: Vec<String>,
    /// Live: the Log columns' display order (drag-to-reorder), by name.
    /// Empty means the default order; unknown names are ignored and
    /// columns left out are appended in default order.
    pub log_columns_order: Vec<String>,
    /// Live: manual Log column widths (drag-to-resize), name -> cells.
    pub log_column_widths: std::collections::BTreeMap<String, u16>,
    /// Live: the Log column widths of the `scroll` layout, name -> cells
    /// (`message` included: it has a width of its own there). Apart from
    /// `log_column_widths`, which stays the `fit` layout's.
    pub log_column_widths_scroll: std::collections::BTreeMap<String, u16>,
    /// Live: the parts of the commit column beside a diff (the column's
    /// `File | All` selector) that are shown, by the Log's stable column
    /// names - `ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`.
    /// Empty means the default (lanes, message, author, date). Apart from
    /// `log_columns_*`: neither list changes the other. Global, kept across
    /// restarts.
    pub commit_column_columns_visible: Vec<String>,
    /// Live: the order of the commit column's parts, left to right, by the
    /// same names. Empty means the default order (`ref`, `graph`, `message`,
    /// `author`, `date`, `changes`, `sha`); a part the list leaves out follows
    /// in default order. Apart from `log_columns_order`.
    pub commit_column_columns_order: Vec<String>,
    /// Live: manual widths of the commit column's parts, name -> cells (the
    /// same names; a part left out is automatic). Apart from
    /// `log_column_widths`.
    pub commit_column_columns_widths: std::collections::BTreeMap<String, u16>,
    /// Live: left-pane sections the user hid (pane right-click menu or
    /// palette), by stable lowercase name - `local`, `remote`, `worktrees`,
    /// `stashes`, `tags`, `pull_requests`, `issues`, `submodules`. Global:
    /// every repository tab and every restart. Unknown names are ignored on
    /// load, and at least one available section always stays shown.
    pub sidebar_sections_hidden: Vec<String>,
    /// Live: whose choice the left-pane section set is. `repository`
    /// (default): each repository keeps its own, falling back to
    /// `sidebar_sections_hidden`; `global`: every repository shows
    /// `sidebar_sections_hidden` and the per-repository choices are kept
    /// (not used) for when it is switched back. See [`SidebarSectionsScope`].
    /// `ui.sidebar_sections_scope`.
    pub sidebar_sections_scope: SidebarSectionsScope,
}

/// Rows per mouse-wheel notch in the commit list, when nothing says
/// otherwise. Matches the diff view and the File Insight overlay.
pub const DEFAULT_MOUSE_SCROLL_LINES: u16 = 3;

/// A notch that moves nothing looks like a broken mouse, and one that
/// moves a screenful looks like a bug.
pub const MIN_MOUSE_SCROLL_LINES: u16 = 1;

pub const MAX_MOUSE_SCROLL_LINES: u16 = 20;

/// The most MiB an image the viewer decodes may weigh (Settings), one GiB.
pub const MAX_IMAGE_VIEW_MAX_MIB: u32 = 1024;

impl UiConfig {
    /// `image_view_max_mib` as Settings reads it back (1 to
    /// [`MAX_IMAGE_VIEW_MAX_MIB`]): a count and `mib` or `gib`.
    pub fn image_view_max_quantity(&self) -> (u64, DisplayUnit) {
        read_quantity(
            u64::from(self.image_view_max_mib.clamp(1, MAX_IMAGE_VIEW_MAX_MIB)),
            DisplayUnit::Mib,
            self.image_view_max_unit,
            MIB_OR_GIB,
        )
    }

    /// `ui.theme` as a theme slug: the legacy `"default"` spelling (and an
    /// empty value) read as `system`.
    pub fn theme_slug(&self) -> &str {
        match self.theme.trim() {
            "" | "default" => THEME_SYSTEM,
            other => other,
        }
    }

    /// `ui.graph_color_theme` with an empty value read as "follow the theme".
    pub fn graph_color_slug(&self) -> &str {
        match self.graph_color_theme.trim() {
            "" => GRAPH_COLOR_FOLLOW_THEME,
            other => other,
        }
    }

    /// `blame_width`, clamped into something usable.
    pub fn blame_width_clamped(&self) -> u16 {
        self.blame_width.clamp(MIN_BLAME_WIDTH, MAX_BLAME_WIDTH)
    }

    /// `mouse_scroll_lines`, clamped into something usable. Call this
    /// rather than the field; `0` from a config file would silently
    /// disable the wheel.
    pub fn mouse_scroll_lines_clamped(&self) -> u16 {
        self.mouse_scroll_lines
            .clamp(MIN_MOUSE_SCROLL_LINES, MAX_MOUSE_SCROLL_LINES)
    }

    /// Where avatars go, with the legacy key folded in: `ui.avatars` when
    /// the file says it, else `ui.show_avatars` (`true` -> `author`).
    pub fn avatar_placement(&self) -> AvatarPlacement {
        match self.avatars {
            Some(placement) => placement,
            None if self.show_avatars => AvatarPlacement::Author,
            None => AvatarPlacement::Off,
        }
    }

    /// Folds the legacy `ui.diff_view_mode` into `diff_layout` /
    /// `diff_hunks_only`, each only when the file's own `[ui]` table
    /// (`file_ui`, the raw TOML) does not already set it - a new key the
    /// user (or the Diff view) wrote always wins over the old one still
    /// sitting beside it. Called by every loader right after parsing
    /// (`Config::from_toml_str`), so no consumer ever sees the old key.
    pub fn migrate_legacy(&mut self, file_ui: Option<&toml::Value>) {
        let Some(legacy) = self.diff_view_mode else {
            return;
        };
        let file_says = |key: &str| file_ui.and_then(|ui| ui.get(key)).is_some();
        if !file_says("diff_layout") {
            self.diff_layout = legacy.layout();
        }
        if !file_says("diff_hunks_only") {
            self.diff_hunks_only = legacy.hunks_only();
        }
    }
}

config_enum! {
/// `ui.avatars`: where the author's avatar is drawn in the Log.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarPlacement {
    /// Inside the commit circle of the image graph. On a
    /// terminal drawing the graph as text, behaves like `author` there.
    Graph => "graph",
    /// A round picture at the start of the Author column.
    Author => "author",
    /// No avatars and no avatar network traffic at all.
    #[default]
    Off => "off",
}
}

config_enum! {
/// `ui.avatar_sources`: which places an author's picture is looked up, in
/// which order. A GitHub noreply address always names its account first
/// (no request needed); the presets order what comes after it.
///
/// "Hosting" is the connected GitHub/GitLab integration whose host matches
/// the repository's remote (GitHub commit author / user search, GitLab
/// `/avatar?email=` / users API). `hosting_gravatar` asks the hosting API
/// first only when a matching integration is connected; without one it
/// behaves like `gravatar_hosting` (Gravatar, then the unauthenticated
/// GitHub API for a github.com remote), the pre-integration behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarSources {
    /// The connected hosting API, then Gravatar.
    HostingGravatar => "hosting_gravatar",
    /// Gravatar, then the hosting API (the default).
    #[default]
    GravatarHosting => "gravatar_hosting",
    /// Gravatar only.
    Gravatar => "gravatar",
    /// The hosting API only.
    Hosting => "hosting",
}
}

impl AvatarSources {
    /// Whether the hosting API is asked at all.
    pub fn uses_hosting(self) -> bool {
        !matches!(self, AvatarSources::Gravatar)
    }

    /// Whether Gravatar is asked at all.
    pub fn uses_gravatar(self) -> bool {
        !matches!(self, AvatarSources::Hosting)
    }

    /// Whether the hosting API comes before Gravatar.
    pub fn hosting_first(self) -> bool {
        matches!(
            self,
            AvatarSources::HostingGravatar | AvatarSources::Hosting
        )
    }
}

config_enum! {
/// `ui.avatar_fallback`: the last resort for an author no source knows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvatarFallback {
    /// A GitHub-style identicon: a 5x5 mirrored pattern derived locally
    /// from the email (no network), as a picture, or as block characters
    /// on a terminal without an image protocol.
    #[default]
    Identicon => "identicon",
    /// A colored badge with the author's initials.
    Initials => "initials",
}
}

config_enum! {
/// `ui.image_view_text_fallback`: what the image view draws when the
/// terminal has no inline-image protocol.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageTextFallback {
    /// The picture as colored half-block characters.
    #[default]
    Halfblock => "halfblock",
    /// Only the info card (format, size, dimensions).
    Card => "card",
}
}

config_enum! {
/// `ui.image_view_pixelated_from`: the zoom factor from which the image
/// view enlarges with nearest-neighbor (square pixels) instead of smoothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImagePixelatedFrom {
    /// From 100% up.
    X1 => "1x",
    /// From 200% up.
    #[default]
    X2 => "2x",
    /// Always smooth.
    Never => "never",
}
}

config_enum! {
/// `ui.column_resize`: how a column boundary is grabbed in the Log, the
/// Radar and the commit column of the diff, file and blame views.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnResize {
    /// A one-cell gap between columns: a small tick in the header, and a
    /// thin line over the whole height of the table while the pointer is on
    /// the boundary. Dragging anywhere along the line resizes.
    #[default]
    Line => "line",
    /// No gap and no tick: the first character of the header cell after the
    /// boundary lights up and is the only place to drag from.
    Header => "header",
}
}

config_enum! {
/// `ui.log_columns_layout`: how the Log lays its columns out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogColumnsLayout {
    /// Every visible column fits the pane exactly; only Message flexes.
    #[default]
    Fit => "fit",
    /// Columns keep their widths (Message too) and the table scrolls sideways
    /// when it is wider than the pane.
    Scroll => "scroll",
}
}

config_enum! {
/// `ui.blame_side`: where the Blame view puts its annotation column.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlameSide {
    /// To the right of the code.
    #[default]
    Right => "right",
    /// To the left of the code.
    Left => "left",
}
}

/// The Blame annotation column's default width, in cells.
pub const DEFAULT_BLAME_WIDTH: u16 = 46;
/// Its narrowest and widest.
pub const MIN_BLAME_WIDTH: u16 = 24;
pub const MAX_BLAME_WIDTH: u16 = 90;

config_enum! {
/// `ui.sidebar_sections_scope`: whose choice the left pane's shown/hidden
/// sections are.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarSectionsScope {
    /// Each repository has its own set (the global one is its default).
    #[default]
    Repository => "repository",
    /// One set for every repository.
    Global => "global",
}
}

config_enum! {
/// `ui.composer_files`: how the AI commit composer's review lists its
/// proposals when it opens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComposerFiles {
    /// Every proposal shows its files under it.
    #[default]
    Expanded => "expanded",
    /// Only the commit rows; a proposal's files open on demand.
    Collapsed => "collapsed",
}
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            language: UiLanguage::default(),
            translate_git_terms: false,
            git_terms_english: false,
            mouse_enabled: true,
            theme: THEME_SYSTEM.into(),
            graph_colorization: GraphColorization::default(),
            graph_lane_background: GraphLaneBackground::default(),
            graph_row_tint_strength: GraphRowTintStrength::default(),
            toolbar_position: ToolbarPosition::default(),
            toolbar_align: ToolbarAlign::default(),
            dialog_geometry: DialogGeometry::default(),
            toolbar_pr_pinned: true,
            statusbar_position: StatusbarPosition::default(),
            tabs_position: TabsPosition::default(),
            graph_color_theme: GRAPH_COLOR_FOLLOW_THEME.into(),
            ref_tree: true,
            graph_renderer: GraphRenderer::default(),
            image_protocol: ImageProtocol::default(),
            kitty_keyboard: KittyKeyboard::default(),
            color_depth: ColorDepth::default(),
            graph_show_stashes: true,
            ref_connector: true,
            log_wip_counts_in_message: true,
            reveal_on_ref_click: true,
            graph_ghost_ref: true,
            allow_agent_view_control: true,
            launch: "off".into(),
            launch_command: String::new(),
            tab_dir_follow: TabDirFollow::default(),
            terminal_launch: "auto".into(),
            terminal_launch_command: String::new(),
            open_in: None,
            open_in_default: String::new(),
            launch_terminal: "auto".into(),
            launcher_build: LauncherBuild::default(),
            relaunch_in_terminal: true,
            reuse_window: true,
            command_alias: String::new(),
            command_alias_add: String::new(),
            image_view: true,
            image_view_max_mib: 32,
            image_view_max_unit: DisplayUnit::default(),
            image_view_text_fallback: ImageTextFallback::default(),
            image_view_pixelated_from: ImagePixelatedFrom::default(),
            image_view_sync: true,
            image_editors: None,
            image_editor_default: String::new(),
            logo_color: String::new(),
            show_avatars: false,
            avatars: None,
            avatar_sources: AvatarSources::default(),
            avatar_fallback: AvatarFallback::default(),
            composer_files: ComposerFiles::default(),
            blame_side: BlameSide::default(),
            column_resize: ColumnResize::default(),
            log_columns_layout: LogColumnsLayout::default(),
            blame_width: DEFAULT_BLAME_WIDTH,
            mouse_scroll_lines: DEFAULT_MOUSE_SCROLL_LINES,
            date_locale: "system".into(),
            date_format: "YYYY-MM-DD".into(),
            date_word_format: "D MMMM YYYY".into(),
            date_time_format: "YYYY-MM-DD HH:mm".into(),
            date_verbose_format: "D MMMM YYYY HH:mm".into(),
            diff_layout: DiffLayout::default(),
            pr_terminology: PrTerminology::default(),
            hide_finished_pr_chip: true,
            diff_hunks_only: true,
            diff_view_mode: None,
            diff_show_whitespace: true,
            diff_word_wrap: false,
            minimap: false,
            mermaid_detail: gitpoulpi_core::domain::MermaidDetail::default(),
            mermaid_orientation: gitpoulpi_core::domain::MermaidOrientation::default(),
            mermaid_fence: false,
            mermaid_subject_width: 40,
            input_color: "theme".into(),
            input_color_custom: "#2a2a2a".into(),
            log_columns_hidden: Vec::new(),
            log_columns_order: Vec::new(),
            log_column_widths: Default::default(),
            log_column_widths_scroll: Default::default(),
            commit_column_columns_visible: Vec::new(),
            commit_column_columns_order: Vec::new(),
            commit_column_columns_widths: Default::default(),
            sidebar_sections_hidden: Vec::new(),
            sidebar_sections_scope: SidebarSectionsScope::default(),
        }
    }
}
