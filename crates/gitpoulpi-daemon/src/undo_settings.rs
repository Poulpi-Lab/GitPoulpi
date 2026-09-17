//! `general.undo_enabled` and `general.undo_depth`, read for every
//! operation the journal might record. Reading the TOML each time costs
//! more than the staging operation it guards, so the answer is kept until
//! the config file's stamp or one of the two environment variables moves
//! (a change in Settings still applies at once).

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UndoSettings {
    pub enabled: bool,
    /// Entry count; `usize::MAX` is unlimited.
    pub depth: usize,
    /// `general.journal_outside`: record what happens outside GitPoulpi.
    pub outside: bool,
}

impl Default for UndoSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            depth: gitpoulpi_core::journal::DEFAULT_MAX_ENTRIES,
            outside: true,
        }
    }
}

type Stamp = (Option<PathBuf>, Option<(SystemTime, u64)>);

#[derive(PartialEq, Eq)]
struct Key {
    file: Stamp,
    env_depth: Option<String>,
    env_enabled: Option<String>,
    env_outside: Option<String>,
}

static CACHE: Mutex<Option<(Key, UndoSettings)>> = Mutex::new(None);

fn key() -> Key {
    let file = gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_file());
    let stamp = file
        .as_ref()
        .and_then(|f| std::fs::metadata(f).ok())
        .and_then(|m| Some((m.modified().ok()?, m.len())));
    Key {
        file: (file, stamp),
        env_depth: std::env::var("GITPOULPI_GENERAL_UNDO_DEPTH").ok(),
        env_enabled: std::env::var("GITPOULPI_GENERAL_UNDO_ENABLED").ok(),
        env_outside: std::env::var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE").ok(),
    }
}

/// The settings now. Anything unreadable is the default (on, depth 100).
pub fn current() -> UndoSettings {
    let key = key();
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached, settings)) = cache.as_ref() {
        if *cached == key {
            return *settings;
        }
    }
    let settings = gitpoulpi_config::Settings::resolve(
        gitpoulpi_config::ConfigPaths::discover().as_ref(),
        &gitpoulpi_config::Overrides::default(),
    )
    .map(|s| UndoSettings {
        enabled: s.config().general.undo_enabled,
        depth: s.config().general.undo_depth_limit(),
        outside: s.config().general.journal_outside,
    })
    .unwrap_or_default();
    *cache = Some((key, settings));
    settings
}

pub fn enabled() -> bool {
    current().enabled
}

/// Undo is on and `general.journal_outside` is on.
pub fn outside_enabled() -> bool {
    let s = current();
    s.enabled && s.outside
}
