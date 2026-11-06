//! The recent-repositories list: what an "open a repository" flow offers
//! before the user has typed anything.
//!
//! **Why this is not in the daemon.** ADR-0006 draws the line at
//! *repository state*: what a `GitBackend` could tell you is the
//! daemon's; a property of a window is not. Which repositories this user
//! opened last is the second kind, so it is an ordinary file in the
//! config directory, shared by every session on the machine, which also
//! lets `gitpoulpi repo recent` answer without starting a daemon.
//!
//! **Not validated on load, by this module.** `load` hands back what the
//! file holds. The Home tab calls `dedupe` and `prune_missing` on it and
//! writes the result back, so a folder deleted from disk leaves the
//! recent list; favorites, which are a deliberate choice, are never
//! pruned.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{same_repo_path, Result};

/// How many entries are kept. Past this, the least recently opened is
/// dropped. Twenty is roughly a screenful in the open-repository
/// overlay; the list is a convenience, not an archive.
pub const MAX_RECENT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentRepo {
    /// The repository *root*, as `GitBackend::discover` reported it --
    /// never a subdirectory: recording the cwd would reopen a different
    /// `RepoId` than the one the daemon keys on.
    pub path: PathBuf,
    /// The label shown in the list. Stored rather than recomputed so an
    /// entry whose directory is gone still has a name to show.
    pub name: String,
    /// Unix seconds. Newest first is the list's whole ordering.
    pub last_opened: i64,
    /// The branch that was out when it was last opened, if any. Shown as
    /// a subtitle; never acted on.
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentRepos {
    #[serde(default)]
    pub repos: Vec<RecentRepo>,
}

impl RecentRepos {
    /// `<config dir>/recent.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("recent.toml")
    }

    /// A missing file is an empty list, not an error: the first run of
    /// the app has no recents and that is not a failure.
    ///
    /// A *corrupt* file is also an empty list rather than an error - the
    /// one lenient decision in this module: a truncated convenience file
    /// must not make the repository picker unusable.
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

    /// Moves `path` to the front, deduplicating by path, and truncates
    /// to [`MAX_RECENT`].
    ///
    /// Deduplication compares normalized paths ([`same_repo_path`]), since
    /// callers do not always pass `discover`'s spelling.
    pub fn record(&mut self, path: &Path, name: &str, branch: Option<String>, now: i64) {
        self.repos.retain(|r| !same_repo_path(&r.path, path));
        self.repos.insert(
            0,
            RecentRepo {
                path: path.to_path_buf(),
                name: name.to_string(),
                last_opened: now,
                branch,
            },
        );
        self.repos.truncate(MAX_RECENT);
    }

    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.repos.len();
        self.repos.retain(|r| !same_repo_path(&r.path, path));
        self.repos.len() != before
    }

    /// Collapses entries that are the same directory under different
    /// spellings (trailing separator, `.`, symlink, Windows case or
    /// `\\?\` prefix), keeping the first - the newest. Returns how many
    /// were dropped. Migrates files written before `record` compared
    /// normalized paths.
    pub fn dedupe(&mut self) -> usize {
        let before = self.repos.len();
        let mut kept: Vec<RecentRepo> = Vec::with_capacity(before);
        for repo in self.repos.drain(..) {
            if !kept.iter().any(|k| same_repo_path(&k.path, &repo.path)) {
                kept.push(repo);
            }
        }
        self.repos = kept;
        before - self.repos.len()
    }

    /// Drops every entry whose directory is gone from disk. Returns how
    /// many were dropped, so a caller can tell the user.
    pub fn prune_missing(&mut self) -> usize {
        let before = self.repos.len();
        self.repos.retain(|r| r.path.exists());
        before - self.repos.len()
    }

    /// Newest first, which is the order they are stored in.
    pub fn iter(&self) -> impl Iterator<Item = &RecentRepo> {
        self.repos.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.repos.is_empty()
    }
}

/// Unix seconds, for callers that have no clock of their own.
pub use gitpoulpi_core::time::unix_now as now_seconds;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn a_saved_session_file_is_user_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sessions").join("recent.toml");
        RecentRepos::default().save(&file).unwrap();
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&file), 0o600);
        assert_eq!(mode(file.parent().unwrap()), 0o700);
    }

    fn at(n: usize) -> PathBuf {
        PathBuf::from(format!("/repos/r{n}"))
    }

    #[test]
    fn the_most_recently_opened_repository_is_first() {
        let mut recents = RecentRepos::default();
        recents.record(&at(1), "r1", None, 100);
        recents.record(&at(2), "r2", Some("main".into()), 200);

        let names: Vec<_> = recents.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["r2", "r1"]);
        assert_eq!(recents.repos[0].branch.as_deref(), Some("main"));
    }

    /// Reopening a repository must move it, not duplicate it.
    #[test]
    fn reopening_moves_an_entry_instead_of_duplicating_it() {
        let mut recents = RecentRepos::default();
        recents.record(&at(1), "r1", None, 100);
        recents.record(&at(2), "r2", None, 200);
        recents.record(&at(1), "r1", Some("side".into()), 300);

        assert_eq!(recents.repos.len(), 2);
        assert_eq!(recents.repos[0].path, at(1));
        assert_eq!(recents.repos[0].last_opened, 300);
        assert_eq!(recents.repos[0].branch.as_deref(), Some("side"));
    }

    #[test]
    fn the_list_is_capped_and_drops_the_oldest() {
        let mut recents = RecentRepos::default();
        for n in 0..MAX_RECENT + 5 {
            recents.record(&at(n), &format!("r{n}"), None, n as i64);
        }
        assert_eq!(recents.repos.len(), MAX_RECENT);
        assert_eq!(recents.repos[0].path, at(MAX_RECENT + 4));
        assert!(!recents.repos.iter().any(|r| r.path == at(0)));
    }

    #[test]
    fn a_round_trip_through_the_file_preserves_the_order() {
        let dir = tempfile::tempdir().unwrap();
        let file = RecentRepos::file_in(dir.path());

        let mut recents = RecentRepos::default();
        recents.record(&at(1), "r1", None, 100);
        recents.record(&at(2), "r2", Some("main".into()), 200);
        recents.save(&file).unwrap();

        let loaded = RecentRepos::load(&file);
        assert_eq!(loaded.repos, recents.repos);
    }

    /// A missing file and a corrupt one both have to be survivable: the
    /// picker opening empty beats the picker not opening.
    #[test]
    fn a_missing_or_corrupt_file_loads_as_an_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        assert!(RecentRepos::load(&RecentRepos::file_in(dir.path())).is_empty());

        let file = dir.path().join("broken.toml");
        std::fs::write(&file, "this is not toml {{{").unwrap();
        assert!(RecentRepos::load(&file).is_empty());
    }

    /// An entry whose directory is gone stays until something asks for
    /// it to go, so the user can see *why* the list shrank.
    #[test]
    fn pruning_is_opt_in_and_reports_what_it_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let live = dir.path().join("live");
        std::fs::create_dir_all(&live).unwrap();

        let mut recents = RecentRepos::default();
        recents.record(&live, "live", None, 100);
        recents.record(&dir.path().join("gone"), "gone", None, 200);
        assert_eq!(recents.repos.len(), 2, "loading must not prune");

        assert_eq!(recents.prune_missing(), 1);
        assert_eq!(recents.repos.len(), 1);
        assert_eq!(recents.repos[0].path, live);
    }

    /// Spellings of one directory: a trailing separator, a `.` segment, a
    /// symlink. The list must hold it once.
    #[cfg(unix)]
    #[test]
    fn two_spellings_of_one_repository_are_one_entry() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let mut recents = RecentRepos::default();
        recents.record(&real, "real", None, 100);
        recents.record(&link, "link", None, 200);
        recents.record(&dir.path().join("real/"), "real", None, 300);
        recents.record(&dir.path().join("./real"), "real", None, 400);
        assert_eq!(recents.repos.len(), 1);
        assert_eq!(recents.repos[0].last_opened, 400);
        assert!(recents.remove(&link));
        assert!(recents.is_empty());
    }

    /// A file saved by an older build can already hold the duplicates:
    /// `dedupe` keeps the newest spelling and reports how many it dropped.
    #[test]
    fn dedupe_migrates_stored_duplicates_keeping_the_newest() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        let mut recents = RecentRepos::default();
        for (p, t) in [
            (real.join("."), 300),
            (real.clone(), 200),
            (real.join("x/.."), 100),
        ] {
            recents.repos.push(RecentRepo {
                path: p,
                name: "real".into(),
                last_opened: t,
                branch: None,
            });
        }
        assert_eq!(recents.dedupe(), 2);
        assert_eq!(recents.repos.len(), 1);
        assert_eq!(recents.repos[0].last_opened, 300);
    }

    #[test]
    fn removing_reports_whether_anything_was_there() {
        let mut recents = RecentRepos::default();
        recents.record(&at(1), "r1", None, 100);
        assert!(recents.remove(&at(1)));
        assert!(!recents.remove(&at(1)));
        assert!(recents.is_empty());
    }
}
