//! The source the Clone dialog showed when it was last closed.
//!
//! State, not configuration: the user never typed it, so it is not in
//! `config.toml` (`repos.clone_start_source = "last"` merely says to use
//! it). One small file in the config directory, shared by every session on
//! the machine, like `recent.toml`.
//!
//! Keyed by the integration profile's id, never by an index: profiles can
//! be reordered or removed. "Clone with URL" is the absence of a profile
//! (`last_profile = None`), so no profile id can collide with a marker. A
//! missing or corrupt file means URL: a convenience file must never make
//! Clone unusable.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneSource {
    /// The integration profile id the dialog last showed; `None` = URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_profile: Option<String>,
}

impl CloneSource {
    /// `<config dir>/clone-source.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("clone-source.toml")
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Temp file + rename, so a crash never leaves half a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, toml::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_profile_round_trips_and_url_is_the_absence_of_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = CloneSource::file_in(dir.path());
        assert_eq!(CloneSource::load(&path), CloneSource::default());
        let saved = CloneSource {
            last_profile: Some("work-gitlab".into()),
        };
        saved.save(&path).unwrap();
        assert_eq!(CloneSource::load(&path), saved);
        CloneSource::default().save(&path).unwrap();
        assert_eq!(CloneSource::load(&path).last_profile, None);
    }

    #[test]
    fn a_corrupt_file_is_url() {
        let dir = tempfile::tempdir().unwrap();
        let path = CloneSource::file_in(dir.path());
        std::fs::write(&path, "last_profile = [").unwrap();
        assert_eq!(CloneSource::load(&path), CloneSource::default());
    }
}
