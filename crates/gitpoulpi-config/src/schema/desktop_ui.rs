//! The schema of `desktop.toml`: the interface settings of the native desktop
//! frontend (see `crate::split` for the three-file design).
//!
//! `[ui]` holds every key the classification table ([`crate::split::KEY_HOME`])
//! homes as [`Home::PerInterface`](crate::split::Home::PerInterface), with the
//! same name, type and default as the terminal UI's `[ui]` (the defaults are
//! taken from `UiConfig::default()`, so the two cannot drift), plus the
//! desktop-only keys in [`DESKTOP_ONLY_KEYS`]: window geometry, pane
//! visibility and widths, and column widths, all in logical pixels.
//! `[keybindings]` is the desktop's own action set. `[repos."<path>"]` holds
//! what is remembered per repository (column widths).
//!
//! The desktop is **not** seeded from `tui.toml`: a missing file is the
//! built-in defaults (language `auto`, theme `system`).
//!
//! **Validation.** Enumerations and booleans are checked by serde, like the
//! terminal UI's. Pixel values are *clamped* when loaded
//! ([`DesktopConfig::load`]): a hand-edited `left_pane_width = 5` shows the
//! minimum instead of breaking the window. The typed setters clamp too and
//! return what they stored. The generic writer behind
//! `gitpoulpi config set desktop.ui.<key>` instead *refuses* an out-of-range
//! value ([`DesktopConfig::validate`]), because a shell user wants to be told.
//! Floats (an older build stored pixels as floats) are accepted and rounded.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::quantity::DisplayUnit;
use super::ui::retired_slug;
use super::*;
use crate::keybindings::KeybindingsConfig;
use crate::{Error, Result};

/// The `[ui]` keys only the desktop has (everything else of
/// [`DesktopUiConfig`] is classified `PerInterface` in `KEY_HOME`).
pub const DESKTOP_ONLY_KEYS: &[&str] = &[
    "window_width",
    "window_height",
    "window_x",
    "window_y",
    "window_maximized",
    "left_pane_visible",
    "right_pane_visible",
    "left_pane_width",
    "right_pane_width",
    "column_widths_px",
];

/// Inclusive pixel range of a desktop-only numeric key, `None` for a key
/// without one. The single source of both the clamping and the error text.
pub fn pixel_range(key: &str) -> Option<(i64, i64)> {
    Some(match key {
        "window_width" => (900, 16_000),
        "window_height" => (560, 16_000),
        "window_x" | "window_y" => (-32_000, 32_000),
        "left_pane_width" => (180, 520),
        "right_pane_width" => (280, 760),
        _ => return None,
    })
}

/// The Log columns the desktop sizes, with their pixel limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopColumn {
    Ref,
    Graph,
    Author,
    Date,
    Sha,
}

impl DesktopColumn {
    pub const ALL: [DesktopColumn; 5] = [
        DesktopColumn::Ref,
        DesktopColumn::Graph,
        DesktopColumn::Author,
        DesktopColumn::Date,
        DesktopColumn::Sha,
    ];

    /// The key under `column_widths_px`.
    pub fn key(self) -> &'static str {
        match self {
            DesktopColumn::Ref => "ref_w",
            DesktopColumn::Graph => "graph_w",
            DesktopColumn::Author => "author_w",
            DesktopColumn::Date => "date_w",
            DesktopColumn::Sha => "sha_w",
        }
    }

    /// Smallest width, in logical pixels.
    pub const fn min(self) -> u32 {
        match self {
            DesktopColumn::Ref => 64,
            DesktopColumn::Graph => 48,
            DesktopColumn::Author => 72,
            DesktopColumn::Date => 124,
            DesktopColumn::Sha => 72,
        }
    }

    /// Largest width, in logical pixels.
    pub const fn max(self) -> u32 {
        match self {
            DesktopColumn::Ref => 420,
            DesktopColumn::Graph => 700,
            DesktopColumn::Author => 400,
            DesktopColumn::Date => 240,
            DesktopColumn::Sha => 160,
        }
    }
}

/// The preferred width of each fixed Log column, in logical pixels (Message
/// takes what the others leave).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopColumnWidths {
    #[serde(deserialize_with = "lenient_u32")]
    pub ref_w: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub graph_w: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub author_w: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub date_w: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub sha_w: u32,
}

impl Default for DesktopColumnWidths {
    fn default() -> Self {
        Self {
            ref_w: 190,
            graph_w: 180,
            author_w: 130,
            date_w: 132,
            sha_w: 72,
        }
    }
}

impl DesktopColumnWidths {
    pub fn get(&self, col: DesktopColumn) -> u32 {
        match col {
            DesktopColumn::Ref => self.ref_w,
            DesktopColumn::Graph => self.graph_w,
            DesktopColumn::Author => self.author_w,
            DesktopColumn::Date => self.date_w,
            DesktopColumn::Sha => self.sha_w,
        }
    }

    /// Sets one width, kept inside the column's limits.
    pub fn set(&mut self, col: DesktopColumn, width: u32) {
        let w = width.clamp(col.min(), col.max());
        match col {
            DesktopColumn::Ref => self.ref_w = w,
            DesktopColumn::Graph => self.graph_w = w,
            DesktopColumn::Author => self.author_w = w,
            DesktopColumn::Date => self.date_w = w,
            DesktopColumn::Sha => self.sha_w = w,
        }
    }

    /// A copy with every width inside its column's limits.
    pub fn sanitized(self) -> Self {
        let mut out = self;
        for col in DesktopColumn::ALL {
            out.set(col, self.get(col));
        }
        out
    }
}

/// A pixel count a file may spell as an integer or a float.
fn lenient_u32<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Int(i64),
        Float(f64),
    }
    match Number::deserialize(d)? {
        Number::Int(i) => Ok(i.clamp(0, i64::from(u32::MAX)) as u32),
        Number::Float(f) if f.is_finite() => Ok(f.round().clamp(0.0, f64::from(u32::MAX)) as u32),
        Number::Float(_) => Err(serde::de::Error::custom("not a finite number")),
    }
}

/// [`lenient_u32`] for an optional signed coordinate.
fn lenient_opt_i32<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<i32>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Int(i64),
        Float(f64),
    }
    Ok(match Option::<Number>::deserialize(d)? {
        None => None,
        Some(Number::Int(i)) => Some(i.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32),
        Some(Number::Float(f)) if f.is_finite() => {
            Some(f.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32)
        }
        Some(Number::Float(_)) => return Err(serde::de::Error::custom("not a finite number")),
    })
}

/// The desktop's `[ui]` table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopUiConfig {
    // ---- Per-interface keys: same names, types and defaults as `[ui]`. ----
    pub language: UiLanguage,
    pub translate_git_terms: bool,
    pub git_terms_english: bool,
    #[serde(deserialize_with = "retired_slug")]
    pub theme: String,
    pub graph_colorization: GraphColorization,
    pub graph_lane_background: GraphLaneBackground,
    pub toolbar_position: ToolbarPosition,
    pub toolbar_align: ToolbarAlign,
    pub dialog_geometry: DialogGeometry,
    pub toolbar_pr_pinned: bool,
    pub statusbar_position: StatusbarPosition,
    pub tabs_position: TabsPosition,
    pub graph_row_tint_strength: GraphRowTintStrength,
    #[serde(deserialize_with = "retired_slug")]
    pub graph_color_theme: String,
    pub ref_tree: bool,
    pub graph_show_stashes: bool,
    pub ref_connector: bool,
    pub log_wip_counts_in_message: bool,
    pub reveal_on_ref_click: bool,
    pub tree_folders_first: bool,
    pub graph_ghost_ref: bool,
    pub allow_agent_view_control: bool,
    pub tab_dir_follow: TabDirFollow,
    pub reuse_window: bool,
    pub image_view: bool,
    pub image_view_max_mib: u32,
    pub image_view_max_unit: DisplayUnit,
    pub image_view_pixelated_from: ImagePixelatedFrom,
    pub image_view_sync: bool,
    pub show_avatars: bool,
    pub avatars: Option<AvatarPlacement>,
    pub avatar_fallback: AvatarFallback,
    pub composer_files: ComposerFiles,
    pub blame_side: BlameSide,
    pub column_resize: ColumnResize,
    pub log_columns_layout: LogColumnsLayout,
    pub date_locale: String,
    pub date_format: String,
    pub date_time_format: String,
    pub date_verbose_format: String,
    pub diff_layout: DiffLayout,
    pub pr_terminology: PrTerminology,
    pub hide_finished_pr_chip: bool,
    pub diff_hunks_only: bool,
    pub diff_show_whitespace: bool,
    pub diff_word_wrap: bool,
    pub diff_intraline: bool,
    pub minimap: bool,
    pub log_columns_hidden: Vec<String>,
    pub log_columns_order: Vec<String>,
    pub commit_column_columns_visible: Vec<String>,
    pub commit_column_columns_order: Vec<String>,
    pub sidebar_sections_hidden: Vec<String>,
    pub sidebar_sections_scope: SidebarSectionsScope,

    // ---- Desktop-only keys (see `DESKTOP_ONLY_KEYS`), logical pixels. ----
    /// Window size, 900x560 up to 16000x16000. Default 1500x900.
    #[serde(deserialize_with = "lenient_u32")]
    pub window_width: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub window_height: u32,
    /// Window position (top-left, -32000..32000); absent: the window manager
    /// places it.
    #[serde(deserialize_with = "lenient_opt_i32")]
    pub window_x: Option<i32>,
    #[serde(deserialize_with = "lenient_opt_i32")]
    pub window_y: Option<i32>,
    pub window_maximized: bool,
    /// The refs sidebar and the right pane (both shown by default).
    pub left_pane_visible: bool,
    pub right_pane_visible: bool,
    /// Pane widths: left 180..520 (default 260), right 280..760 (default 380).
    #[serde(deserialize_with = "lenient_u32")]
    pub left_pane_width: u32,
    #[serde(deserialize_with = "lenient_u32")]
    pub right_pane_width: u32,
    /// Log column widths for repositories with none of their own.
    pub column_widths_px: DesktopColumnWidths,
}

impl Default for DesktopUiConfig {
    /// Every per-interface default comes from `UiConfig::default()`.
    fn default() -> Self {
        let u = UiConfig::default();
        Self {
            language: u.language,
            translate_git_terms: u.translate_git_terms,
            git_terms_english: u.git_terms_english,
            theme: u.theme,
            graph_colorization: u.graph_colorization,
            graph_lane_background: u.graph_lane_background,
            toolbar_position: u.toolbar_position,
            toolbar_align: u.toolbar_align,
            dialog_geometry: u.dialog_geometry,
            toolbar_pr_pinned: u.toolbar_pr_pinned,
            statusbar_position: u.statusbar_position,
            tabs_position: u.tabs_position,
            graph_row_tint_strength: u.graph_row_tint_strength,
            graph_color_theme: u.graph_color_theme,
            ref_tree: u.ref_tree,
            graph_show_stashes: u.graph_show_stashes,
            ref_connector: u.ref_connector,
            log_wip_counts_in_message: u.log_wip_counts_in_message,
            reveal_on_ref_click: u.reveal_on_ref_click,
            tree_folders_first: u.tree_folders_first,
            graph_ghost_ref: u.graph_ghost_ref,
            allow_agent_view_control: u.allow_agent_view_control,
            tab_dir_follow: u.tab_dir_follow,
            reuse_window: u.reuse_window,
            image_view: u.image_view,
            image_view_max_mib: u.image_view_max_mib,
            image_view_max_unit: u.image_view_max_unit,
            image_view_pixelated_from: u.image_view_pixelated_from,
            image_view_sync: u.image_view_sync,
            show_avatars: u.show_avatars,
            avatars: u.avatars,
            avatar_fallback: u.avatar_fallback,
            composer_files: u.composer_files,
            blame_side: u.blame_side,
            column_resize: u.column_resize,
            log_columns_layout: u.log_columns_layout,
            date_locale: u.date_locale,
            date_format: u.date_format,
            date_time_format: u.date_time_format,
            date_verbose_format: u.date_verbose_format,
            diff_layout: u.diff_layout,
            pr_terminology: u.pr_terminology,
            hide_finished_pr_chip: u.hide_finished_pr_chip,
            diff_hunks_only: u.diff_hunks_only,
            diff_show_whitespace: u.diff_show_whitespace,
            diff_word_wrap: u.diff_word_wrap,
            diff_intraline: u.diff_intraline,
            minimap: u.minimap,
            log_columns_hidden: u.log_columns_hidden,
            log_columns_order: u.log_columns_order,
            commit_column_columns_visible: u.commit_column_columns_visible,
            commit_column_columns_order: u.commit_column_columns_order,
            sidebar_sections_hidden: u.sidebar_sections_hidden,
            sidebar_sections_scope: u.sidebar_sections_scope,
            window_width: 1500,
            window_height: 900,
            window_x: None,
            window_y: None,
            window_maximized: false,
            left_pane_visible: true,
            right_pane_visible: true,
            left_pane_width: 260,
            right_pane_width: 380,
            column_widths_px: DesktopColumnWidths::default(),
        }
    }
}

impl DesktopUiConfig {
    /// Pulls every pixel value into its supported range.
    pub fn sanitize(&mut self) {
        let clamp = |key: &str, v: i64| {
            let (lo, hi) = pixel_range(key).expect("a ranged key");
            v.clamp(lo, hi)
        };
        self.window_width = clamp("window_width", i64::from(self.window_width)) as u32;
        self.window_height = clamp("window_height", i64::from(self.window_height)) as u32;
        self.window_x = self
            .window_x
            .map(|v| clamp("window_x", i64::from(v)) as i32);
        self.window_y = self
            .window_y
            .map(|v| clamp("window_y", i64::from(v)) as i32);
        self.left_pane_width = clamp("left_pane_width", i64::from(self.left_pane_width)) as u32;
        self.right_pane_width = clamp("right_pane_width", i64::from(self.right_pane_width)) as u32;
        self.column_widths_px = self.column_widths_px.sanitized();
    }
}

/// What is remembered for one repository (keyed by its path).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopRepoConfig {
    /// This repository's own Log column widths; absent follows
    /// `[ui] column_widths_px`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_widths_px: Option<DesktopColumnWidths>,
}

/// The whole of `desktop.toml`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopConfig {
    pub ui: DesktopUiConfig,
    pub keybindings: KeybindingsConfig,
    /// Per-repository state, by repository path (never reachable from
    /// `config get|set`: a path holds dots).
    pub repos: BTreeMap<String, DesktopRepoConfig>,
}

impl DesktopConfig {
    /// Parses `desktop.toml` text and clamps the pixel values.
    pub fn from_toml_str(raw: &str) -> Result<Self> {
        let mut config: Self = toml::from_str(raw)?;
        config.sanitize();
        Ok(config)
    }

    /// Reads `desktop.toml` from the config directory. A missing or blank
    /// file is the defaults and is **never created** by a read. A file that
    /// does not parse is an error (see [`DesktopConfig::load_or_default`]).
    pub fn load(paths: &crate::ConfigPaths) -> Result<Self> {
        Self::load_file(&paths.desktop_file())
    }

    /// [`DesktopConfig::load`] for an explicit file.
    pub fn load_file(path: &std::path::Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(raw) if raw.trim().is_empty() => Ok(Self::default()),
            Ok(raw) => Self::from_toml_str(&raw),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    /// For a window that must start whatever the file says: the defaults
    /// when the file cannot be read or parsed.
    pub fn load_or_default(paths: &crate::ConfigPaths) -> Self {
        Self::load(paths).unwrap_or_default()
    }

    pub fn sanitize(&mut self) {
        self.ui.sanitize();
        for repo in self.repos.values_mut() {
            repo.column_widths_px = repo.column_widths_px.map(DesktopColumnWidths::sanitized);
        }
    }

    /// Refuses a pixel value outside its range (the file is not sanitized
    /// first). Used by the writers behind `config set`.
    pub fn validate(&self) -> Result<()> {
        let mut clamped = self.clone();
        clamped.sanitize();
        let (now, fit) = (
            serde_json::to_value(&self.ui)?,
            serde_json::to_value(&clamped.ui)?,
        );
        for key in DESKTOP_ONLY_KEYS {
            if now[key] != fit[key] {
                let expected = match pixel_range(key) {
                    Some((lo, hi)) => format!("a whole number from {lo} to {hi}"),
                    None => "a width inside the column's limits".to_string(),
                };
                return Err(Error::BadValue {
                    key: format!("ui.{key}"),
                    value: now[key].to_string(),
                    expected,
                });
            }
        }
        Ok(())
    }

    /// Own widths of `repo`, else the shared default.
    pub fn column_widths_for(&self, repo: &str) -> DesktopColumnWidths {
        self.repos
            .get(repo)
            .and_then(|r| r.column_widths_px)
            .unwrap_or(self.ui.column_widths_px)
    }

    /// Reads a dotted key relative to `desktop.toml` (`ui.theme`) as JSON.
    pub fn get(&self, key: &str) -> Result<serde_json::Value> {
        let root = serde_json::to_value(self)?;
        let mut cursor = &root;
        for segment in key.split('.') {
            cursor = cursor
                .get(segment)
                .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        }
        Ok(cursor.clone())
    }

    /// Sets a dotted key (`ui.theme`) from text, typed by the schema and
    /// validated by deserializing back; `self` is untouched on error.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let mut root = serde_json::to_value(&*self)?;
        let (last, parents) = split_key(key)?;
        if parents.first() == Some(&"repos") || parents.is_empty() {
            return Err(Error::UnknownKey(key.to_string()));
        }
        let mut cursor = &mut root;
        for segment in parents {
            cursor = cursor
                .get_mut(segment)
                .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        }
        let leaf = cursor
            .get_mut(last)
            .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        *leaf = coerce_desktop(key, leaf, value)?;
        let next: Self = serde_json::from_value(root).map_err(|err| Error::BadValue {
            key: key.to_string(),
            value: value.to_string(),
            expected: err.to_string(),
        })?;
        next.validate()?;
        *self = next;
        Ok(())
    }
}

fn split_key(key: &str) -> Result<(&str, Vec<&str>)> {
    let mut segments: Vec<&str> = key.split('.').collect();
    let last = segments.pop().filter(|s| !s.is_empty());
    last.map(|l| (l, segments))
        .ok_or_else(|| Error::UnknownKey(key.to_string()))
}

/// Like the shared coercion, plus an optional integer (a window coordinate
/// that is not set yet reads as `null`).
pub(crate) fn coerce_desktop(
    key: &str,
    current: &serde_json::Value,
    value: &str,
) -> Result<serde_json::Value> {
    if current.is_null() && DESKTOP_ONLY_KEYS.iter().any(|k| key.ends_with(k)) {
        return value
            .parse::<i64>()
            .map(serde_json::Value::from)
            .map_err(|_| Error::BadValue {
                key: key.to_string(),
                value: value.to_string(),
                expected: "expected a whole number".to_string(),
            });
    }
    crate::coerce(key, current, value)
}
