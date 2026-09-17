//! The record of what the daemon resolved while an operation runs
//! (`<git-dir>/gitpoulpi/resolved.json`, plan 3.7): the stages a path had
//! before "Save & Mark Resolved", so `ConflictUnresolve` (the editor's
//! "Unresolve and reopen", the CLI's `conflicts unresolve`) can put the
//! conflict back exactly, long after git forgot it. It outlives a TUI
//! restart and is deleted as soon as no operation is in progress.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{ConflictEntry, RepoRef};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;

/// One resolved path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolved {
    pub entry: ConflictEntry,
    /// Why the daemon resolved it by itself (`whitespace only`), or `None`
    /// for a person's resolution.
    #[serde(default)]
    pub auto: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRecord {
    pub version: u32,
    /// Slash-separated path to what it was.
    pub entries: BTreeMap<String, Resolved>,
}

impl Default for ResolvedRecord {
    fn default() -> Self {
        Self {
            version: VERSION,
            entries: BTreeMap::new(),
        }
    }
}

fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl ResolvedRecord {
    /// Remembers `entry` as resolved. The first record of a path wins: it
    /// is the original conflict, where a later one (resolved, unresolved by
    /// hand, resolved again) would be a copy of a copy.
    pub fn note(&mut self, entry: ConflictEntry, auto: Option<String>) {
        self.entries
            .entry(key(&entry.path))
            .or_insert(Resolved { entry, auto });
    }

    /// Takes the record of `path` out.
    pub fn take(&mut self, path: &Path) -> Option<Resolved> {
        self.entries.remove(&key(path))
    }

    pub fn get(&self, path: &Path) -> Option<&Resolved> {
        self.entries.get(&key(path))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn record_path(repo: &RepoRef) -> Option<PathBuf> {
    crate::journal::git_dir(&repo.root).map(|dir| dir.join("gitpoulpi").join("resolved.json"))
}

fn load(path: &Path) -> ResolvedRecord {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<ResolvedRecord>(&text).ok())
        .filter(|r| r.version == VERSION)
        .unwrap_or_default()
}

fn store(path: &Path, record: &ResolvedRecord) {
    if record.is_empty() {
        let _ = std::fs::remove_file(path);
        return;
    }
    let write = || -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(record).unwrap_or_default())?;
        std::fs::rename(&tmp, path)
    };
    if let Err(e) = write() {
        tracing::warn!("could not write the resolved-conflict record: {e}");
    }
}

/// Records `entry` as resolved (best effort: a record that cannot be
/// written only costs the "unresolve" after a restart).
pub fn note(repo: &RepoRef, entry: ConflictEntry, auto: Option<String>) {
    let Some(path) = record_path(repo) else {
        return;
    };
    let mut record = load(&path);
    record.note(entry, auto);
    store(&path, &record);
}

/// The record of `path`, left in place.
pub fn get(repo: &RepoRef, path: &Path) -> Option<Resolved> {
    let file = record_path(repo)?;
    load(&file).get(path).cloned()
}

/// Forgets `path` (it is conflicted again).
pub fn forget(repo: &RepoRef, path: &Path) {
    let Some(file) = record_path(repo) else {
        return;
    };
    let mut record = load(&file);
    if record.take(path).is_some() {
        store(&file, &record);
    }
}

/// Deletes the whole record once no operation is in progress.
pub fn prune_when_idle(repo: &RepoRef) {
    let Some(dir) = crate::journal::git_dir(&repo.root) else {
        return;
    };
    let file = dir.join("gitpoulpi").join("resolved.json");
    if file.exists() && crate::journal::in_progress(&dir).is_none() {
        let _ = std::fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, StageEntry};

    fn entry(path: &str, n: u8) -> ConflictEntry {
        let stage = |m: u8| {
            Some(StageEntry {
                mode: 0o100644,
                oid: Oid([m; 20]),
            })
        };
        ConflictEntry::from_stages(path.into(), stage(n), stage(n + 1), stage(n + 2)).unwrap()
    }

    #[test]
    fn the_first_record_of_a_path_wins_and_taking_removes_it() {
        let mut record = ResolvedRecord::default();
        record.note(entry("a/b.txt", 1), None);
        record.note(entry("a/b.txt", 7), Some("whitespace only".into()));
        assert_eq!(
            record.get(Path::new("a/b.txt")).unwrap().entry,
            entry("a/b.txt", 1)
        );
        assert!(record.get(Path::new("a/b.txt")).unwrap().auto.is_none());
        assert!(record.take(Path::new("a\\b.txt")).is_some(), "either slash");
        assert!(record.is_empty());
        assert!(record.take(Path::new("a/b.txt")).is_none());
    }

    #[test]
    fn it_round_trips_through_json_and_an_empty_record_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("gitpoulpi").join("resolved.json");
        let mut record = ResolvedRecord::default();
        record.note(entry("x", 1), None);
        store(&file, &record);
        assert_eq!(load(&file), record);
        record.take(Path::new("x"));
        store(&file, &record);
        assert!(!file.exists());
        // Garbage or a newer version reads as empty rather than failing.
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "{ nonsense").unwrap();
        assert!(load(&file).is_empty());
    }
}
