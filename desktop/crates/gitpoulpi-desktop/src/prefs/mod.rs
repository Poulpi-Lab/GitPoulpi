//! Desktop-only interface preferences, kept in `desktop.toml` inside the GitPoulpi config directory.
//!
//! These are the settings only this frontend has (theme, language, layout, column widths, window
//! size). They are deliberately not part of the shared `config.toml` schema. A missing, unreadable
//! or half-valid file never stops the app: every field falls back to its default.

mod columns;
mod store;

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use columns::{resolve, Col, ColumnLayout, ColumnWidths, MESSAGE_FLOOR, MESSAGE_MIN};
pub use store::PrefsStore;

/// File name inside the config directory.
pub const FILE_NAME: &str = "desktop.toml";

/// How many repositories the "recent" list remembers.
const RECENT_MAX: usize = 8;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct UiPrefs {
    /// Slug of a theme preset.
    pub theme: String,
    /// Slug of a graph lane palette; absent follows the theme's own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane_theme: Option<String>,
    /// UI language code (`en`, `fr`, ...).
    pub language: String,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            theme: crate::theme::DEFAULT_THEME.into(),
            lane_theme: None,
            language: "en".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct LayoutPrefs {
    pub show_left: bool,
    pub show_right: bool,
    /// Refs sidebar width in logical pixels.
    pub left_width: f32,
    /// Right pane width in logical pixels.
    pub right_width: f32,
}

impl Default for LayoutPrefs {
    fn default() -> Self {
        Self {
            show_left: true,
            show_right: true,
            left_width: 260.0,
            right_width: 380.0,
        }
    }
}

impl LayoutPrefs {
    pub const LEFT_RANGE: (f32, f32) = (180.0, 520.0);
    pub const RIGHT_RANGE: (f32, f32) = (280.0, 760.0);

    fn sanitized(mut self) -> Self {
        self.left_width = clamp_or(self.left_width, Self::LEFT_RANGE, 260.0);
        self.right_width = clamp_or(self.right_width, Self::RIGHT_RANGE, 380.0);
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub struct WindowPrefs {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowPrefs {
    fn default() -> Self {
        Self {
            width: 1500.0,
            height: 900.0,
        }
    }
}

impl WindowPrefs {
    /// Smallest window the layout is designed for; also the floor of a restored size.
    pub const MIN: (f32, f32) = (900.0, 560.0);
    pub const MAX: (f32, f32) = (16_000.0, 16_000.0);

    /// The size pulled into the supported range.
    pub fn clamped(self) -> Self {
        Self {
            width: clamp_or(self.width, (Self::MIN.0, Self::MAX.0), 1500.0),
            height: clamp_or(self.height, (Self::MIN.1, Self::MAX.1), 900.0),
        }
    }
}

/// What is remembered for one repository (keyed by its path).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct RepoPrefs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<ColumnWidths>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct DesktopPrefs {
    pub ui: UiPrefs,
    pub layout: LayoutPrefs,
    pub window: WindowPrefs,
    /// Log column widths for repositories that have none of their own.
    pub columns: ColumnWidths,
    pub repos: BTreeMap<String, RepoPrefs>,
    /// Most recently opened repositories, newest first.
    pub recent: Vec<String>,
}

fn clamp_or(v: f32, (lo, hi): (f32, f32), fallback: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        fallback
    }
}

impl DesktopPrefs {
    /// The file's path inside a config directory.
    pub fn path_in(dir: &Path) -> PathBuf {
        dir.join(FILE_NAME)
    }

    /// Reads the preferences from `dir`; a missing or unparsable file yields the defaults.
    pub fn load(dir: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(Self::path_in(dir)) else {
            return Self::default();
        };
        Self::parse(&text)
    }

    /// Parses TOML text; invalid text yields the defaults.
    pub fn parse(text: &str) -> Self {
        let mut prefs: Self = toml::from_str(text).unwrap_or_default();
        prefs.layout = prefs.layout.sanitized();
        prefs.window = prefs.window.clamped();
        prefs.columns = prefs.columns.sanitized();
        for repo in prefs.repos.values_mut() {
            repo.columns = repo.columns.map(ColumnWidths::sanitized);
        }
        prefs.recent.truncate(RECENT_MAX);
        prefs
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    /// Writes the preferences into `dir` (created when missing), through a temporary file so a crash
    /// cannot leave half a file behind.
    pub fn save(&self, dir: &Path) -> io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = Self::path_in(dir);
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, self.to_toml())?;
        std::fs::rename(&tmp, &path)
    }

    /// Column widths for a repository: its own, else the shared default.
    pub fn columns_for(&self, repo: &str) -> ColumnWidths {
        self.repos
            .get(repo)
            .and_then(|r| r.columns)
            .unwrap_or(self.columns)
    }

    pub fn set_columns_for(&mut self, repo: &str, widths: ColumnWidths) {
        self.repos.entry(repo.to_string()).or_default().columns = Some(widths.sanitized());
    }

    /// Forgets a repository's own widths: it follows the shared default again.
    pub fn reset_columns_for(&mut self, repo: &str) {
        if let Some(r) = self.repos.get_mut(repo) {
            r.columns = None;
        }
    }

    /// Moves `repo` to the front of the recent list.
    pub fn remember_recent(&mut self, repo: &str) {
        self.recent.retain(|r| r != repo);
        self.recent.insert(0, repo.to_string());
        self.recent.truncate(RECENT_MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_through_a_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = DesktopPrefs::default();
        prefs.ui.theme = "nord".into();
        prefs.ui.lane_theme = Some("gitpoulpi".into());
        prefs.ui.language = "fr".into();
        prefs.layout.show_right = false;
        prefs.layout.left_width = 300.0;
        prefs.window = WindowPrefs {
            width: 1280.0,
            height: 720.0,
        };
        prefs.columns.ref_w = 210.0;
        prefs.set_columns_for(
            "/some/repo",
            ColumnWidths {
                sha_w: 90.0,
                ..ColumnWidths::default()
            },
        );
        prefs.remember_recent("/some/repo");
        prefs.save(dir.path()).unwrap();

        assert!(DesktopPrefs::path_in(dir.path()).is_file());
        assert_eq!(DesktopPrefs::load(dir.path()), prefs);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(DesktopPrefs::load(dir.path()), DesktopPrefs::default());
    }

    #[test]
    fn garbage_gives_defaults() {
        assert_eq!(
            DesktopPrefs::parse("this is [not toml"),
            DesktopPrefs::default()
        );
    }

    #[test]
    fn partial_file_keeps_the_rest_at_defaults() {
        let prefs = DesktopPrefs::parse("[ui]\ntheme = \"nord\"\n");
        assert_eq!(prefs.ui.theme, "nord");
        assert_eq!(prefs.ui.language, "en");
        assert_eq!(prefs.layout, LayoutPrefs::default());
    }

    #[test]
    fn out_of_range_values_are_pulled_in() {
        let prefs = DesktopPrefs::parse(
            "[window]\nwidth = 10.0\nheight = 99999.0\n[layout]\nleft_width = 1.0\n[columns]\nref_w = 5.0\n",
        );
        assert_eq!(prefs.window.width, WindowPrefs::MIN.0);
        assert_eq!(prefs.window.height, WindowPrefs::MAX.1);
        assert_eq!(prefs.layout.left_width, LayoutPrefs::LEFT_RANGE.0);
        assert_eq!(prefs.columns.ref_w, Col::Ref.min());
    }

    #[test]
    fn per_repo_columns_override_the_default_and_can_be_reset() {
        let mut prefs = DesktopPrefs::default();
        let own = ColumnWidths {
            author_w: 200.0,
            ..ColumnWidths::default()
        };
        prefs.set_columns_for("/a", own);
        assert_eq!(prefs.columns_for("/a"), own);
        assert_eq!(prefs.columns_for("/b"), prefs.columns);
        prefs.reset_columns_for("/a");
        assert_eq!(prefs.columns_for("/a"), prefs.columns);
    }

    #[test]
    fn recent_list_is_deduplicated_newest_first_and_capped() {
        let mut prefs = DesktopPrefs::default();
        for i in 0..12 {
            prefs.remember_recent(&format!("/r{i}"));
        }
        prefs.remember_recent("/r5");
        assert_eq!(prefs.recent.len(), RECENT_MAX);
        assert_eq!(prefs.recent[0], "/r5");
        assert_eq!(prefs.recent.iter().filter(|r| *r == "/r5").count(), 1);
    }

    #[test]
    fn save_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        DesktopPrefs::default().save(dir.path()).unwrap();
        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from(FILE_NAME)]);
    }
}
