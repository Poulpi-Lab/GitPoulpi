use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::journal::{Journal, JOURNAL_VERSION};

/// Journals this process has already recovered (`Pending` -> `Interrupted`):
/// a `Pending` entry found later belongs to an operation that is running
/// right now, possibly in another process.
pub(super) static RECOVERED: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

/// The per-worktree git directory: HEAD and the index are per worktree,
/// so each linked worktree has its own journal.
pub fn git_dir(root: &Path) -> Option<PathBuf> {
    gitpoulpi_core::paths::git_dir(root)
}

pub(super) fn journal_path(repo: &RepoRef) -> Option<PathBuf> {
    git_dir(&repo.root).map(|dir| dir.join("gitpoulpi").join("journal.json"))
}

/// Reads the journal; a missing file is an empty one. A version this
/// build does not know is an error and the file is left untouched.
pub(super) fn load(path: &Path) -> std::result::Result<Journal, String> {
    if let Some(journal) = crate::journal_store::cached(path) {
        return Ok(journal);
    }
    // The stamp is taken before the read: a write landing in between
    // makes the copy look stale next time, never fresh.
    let stamp = crate::journal_store::stamp(path);
    let mut journal = match std::fs::read_to_string(path) {
        Ok(text) => {
            // Peek at the version first (a file from a newer GitPoulpi may
            // not parse as a `Journal`), then parse once.
            #[derive(serde::Deserialize)]
            struct Peek {
                #[serde(default)]
                version: u64,
            }
            let version = serde_json::from_str::<Peek>(&text)
                .map_err(|e| format!("the undo journal {} is unreadable: {e}", path.display()))?
                .version;
            if version != u64::from(JOURNAL_VERSION) {
                return Err(format!(
                    "the undo journal {} has version {version}, written by another GitPoulpi; \
                     it is left untouched",
                    path.display()
                ));
            }
            serde_json::from_str::<Journal>(&text)
                .map_err(|e| format!("the undo journal {} is unreadable: {e}", path.display()))?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Journal::default(),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };
    journal.pin_ns = path
        .parent()
        .and_then(Path::parent)
        .map(gitpoulpi_core::journal::pin_namespace)
        .unwrap_or_default();
    let first_time = RECOVERED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashSet::new)
        .insert(path.to_path_buf());
    // A conflicted merge or rebase keeps its `Pending` entry across a
    // restart: it is finished (or dropped) when the operation ends.
    let running = path
        .parent()
        .and_then(Path::parent)
        .is_some_and(|git_dir| in_progress(git_dir).is_some());
    if first_time && !running && journal.recover() {
        let _ = save(path, &journal);
    } else {
        crate::journal_store::put_clean(path, &journal, stamp);
    }
    Ok(journal)
}

/// Temporary file, then rename: a crash leaves the old journal or the
/// new one, never half of one.
pub(super) fn save(path: &Path, journal: &Journal) -> std::io::Result<()> {
    crate::journal_store::write_through(path, journal)
}

/// [`save`] for a staging entry, whose bytes can follow the response: see
/// `journal_store`.
pub(super) fn save_later(path: &Path, journal: &Journal) {
    crate::journal_store::put_later(path, journal);
}

/// A merge, rebase, cherry-pick or revert that has not finished.
pub(crate) fn in_progress(git_dir: &Path) -> Option<String> {
    // `rebase-apply/applying` is a `git am`, which is never journaled.
    if git_dir.join("rebase-apply").join("applying").exists() {
        return Some("am".to_string());
    }
    for (marker, op) in [
        ("rebase-merge", "rebase"),
        ("rebase-apply", "rebase"),
        ("MERGE_HEAD", "merge"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("REVERT_HEAD", "revert"),
    ] {
        if git_dir.join(marker).exists() {
            return Some(op.to_string());
        }
    }
    None
}

pub(super) fn branch_section(name: &str) -> String {
    format!("branch.{name}")
}

pub(super) fn head_ref(name: &str) -> String {
    format!("refs/heads/{name}")
}

/// `general.undo_depth` as an entry count (`usize::MAX` for unlimited),
/// read from the ambient config each time an operation is recorded so a
/// change in Settings applies at once. Anything unreadable is the default.
pub(super) fn undo_depth() -> usize {
    crate::undo_settings::current().depth
}
