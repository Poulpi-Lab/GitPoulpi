//! Favorite repositories: the ones the user starred in the Repository
//! Management tab.
//!
//! **Beside the recent list, for the same reason.** Which repositories a
//! user cares about is a property of the *user*, not of a repository and
//! not of a window: two frontends "disagreeing" about it is not a
//! disagreement (ADR-0006), so it is not the daemon's. It is an ordinary
//! file in the config directory, `favorites.toml`, shared by every
//! session on the machine exactly as `recent.toml` is.
//!
//! Unlike the recent list, favorites are **never capped and never
//! reordered by use**: a favorite is a deliberate choice, and one that
//! quietly fell off the end because twenty other repositories were
//! opened since would be a lie about what the user chose. The order is
//! the order they were starred in.
//!
//! Like the recent list, an entry whose directory is gone is kept and
//! handed back: the UI shows it grayed, and un-starring it is one click.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{same_repo_path, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteRepo {
    /// The repository root, as the recent list and the tab list record
    /// it - the same spelling, so the three lists can be joined by
    /// path.
    pub path: PathBuf,
    /// The label shown. Stored so a favorite whose directory is gone
    /// still has a name.
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteRepos {
    #[serde(default)]
    pub repos: Vec<FavoriteRepo>,
}

impl FavoriteRepos {
    /// `<config dir>/favorites.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("favorites.toml")
    }

    /// A missing or corrupt file is an empty list, for the reason
    /// `RecentRepos::load` gives: a convenience file must never make the
    /// repository picker unusable.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
        }
        gitpoulpi_config::secure_fs::write_private(path, toml::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.repos.iter().any(|r| same_repo_path(&r.path, path))
    }

    /// Stars `path` (appended, so the list keeps the order things were
    /// starred in), or un-stars it if it already is. Returns whether it
    /// is a favorite *afterwards*, which is what a caller reports.
    pub fn toggle(&mut self, path: &Path, name: &str) -> bool {
        if self.remove(path) {
            return false;
        }
        self.repos.push(FavoriteRepo {
            path: path.to_path_buf(),
            name: name.to_string(),
        });
        true
    }

    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.repos.len();
        self.repos.retain(|r| !same_repo_path(&r.path, path));
        self.repos.len() != before
    }

    /// Collapses entries naming the same directory under different
    /// spellings, keeping the first (the earliest starred). Returns how
    /// many were dropped.
    pub fn dedupe(&mut self) -> usize {
        let before = self.repos.len();
        let mut kept: Vec<FavoriteRepo> = Vec::with_capacity(before);
        for repo in self.repos.drain(..) {
            if !kept.iter().any(|k| same_repo_path(&k.path, &repo.path)) {
                kept.push(repo);
            }
        }
        self.repos = kept;
        before - self.repos.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FavoriteRepo> {
        self.repos.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(n: usize) -> PathBuf {
        PathBuf::from(format!("/repos/r{n}"))
    }

    #[test]
    fn dedupe_collapses_spellings_of_one_directory() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        let mut favs = FavoriteRepos::default();
        favs.toggle(&real, "real");
        favs.repos.push(FavoriteRepo {
            path: real.join("."),
            name: "real".into(),
        });
        assert!(favs.contains(&dir.path().join("real/")));
        assert_eq!(favs.dedupe(), 1);
        assert_eq!(favs.repos.len(), 1);
    }

    #[test]
    fn toggling_stars_then_unstars_and_reports_the_state_after() {
        let mut favs = FavoriteRepos::default();
        assert!(favs.toggle(&at(1), "r1"));
        assert!(favs.contains(&at(1)));
        assert!(!favs.toggle(&at(1), "r1"));
        assert!(!favs.contains(&at(1)));
        assert!(favs.repos.is_empty());
    }

    /// Starring keeps the order things were starred in; it is not a
    /// most-recently-used list, so starring a second one must not move
    /// the first.
    #[test]
    fn favorites_keep_the_order_they_were_starred_in() {
        let mut favs = FavoriteRepos::default();
        favs.toggle(&at(2), "r2");
        favs.toggle(&at(1), "r1");
        favs.toggle(&at(3), "r3");
        let names: Vec<_> = favs.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["r2", "r1", "r3"]);
    }

    #[test]
    fn a_round_trip_through_the_file_preserves_the_list() {
        let dir = tempfile::tempdir().unwrap();
        let file = FavoriteRepos::file_in(dir.path());
        let mut favs = FavoriteRepos::default();
        favs.toggle(&at(1), "r1");
        favs.toggle(&at(2), "r2");
        favs.save(&file).unwrap();
        assert_eq!(FavoriteRepos::load(&file), favs);
    }

    #[test]
    fn a_missing_or_corrupt_file_is_an_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        let file = FavoriteRepos::file_in(dir.path());
        assert!(FavoriteRepos::load(&file).repos.is_empty());
        std::fs::write(&file, "this is [not toml").unwrap();
        assert!(FavoriteRepos::load(&file).repos.is_empty());
    }
}
