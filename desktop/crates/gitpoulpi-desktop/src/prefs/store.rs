//! The preferences as an application global: read anywhere, saved when changed.

use std::path::PathBuf;

use gpui_kit::{App, Global};

use super::DesktopPrefs;

/// Holds the live preferences and where they are saved. With no directory (no config directory could
/// be found) the preferences live in memory only.
pub struct PrefsStore {
    dir: Option<PathBuf>,
    prefs: DesktopPrefs,
}

impl Global for PrefsStore {}

impl PrefsStore {
    /// Loads the preferences stored in `dir` (the defaults when there is nothing yet).
    pub fn open(dir: Option<PathBuf>) -> Self {
        let prefs = dir.as_deref().map(DesktopPrefs::load).unwrap_or_default();
        Self { dir, prefs }
    }

    /// The GitPoulpi config directory, through the shared path helpers (honours `GITPOULPI_CONFIG_DIR`).
    pub fn default_dir() -> Option<PathBuf> {
        gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().to_path_buf())
    }

    pub fn get(cx: &App) -> &DesktopPrefs {
        &cx.global::<PrefsStore>().prefs
    }

    /// Applies `f` to the preferences and writes the file. A failed write is reported once on stderr and
    /// never interrupts the user.
    pub fn update<R>(cx: &mut App, f: impl FnOnce(&mut DesktopPrefs) -> R) -> R {
        let store = cx.global_mut::<PrefsStore>();
        let out = f(&mut store.prefs);
        if let Some(dir) = &store.dir {
            if let Err(e) = store.prefs.save(dir) {
                eprintln!("desktop preferences not saved: {e}");
            }
        }
        out
    }
}
