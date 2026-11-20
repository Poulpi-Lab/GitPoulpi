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

/// The environment (the installations design, 1.5) this process keeps its
/// undo journal in: `None` (the flat layout, where the journal and its pins
/// stay exactly where builds without environments put them) until the real
/// program calls [`set_env`], so in-process tests never depend on the
/// machine's own installation.
pub(crate) fn current_env() -> Option<String> {
    ENV.read().unwrap_or_else(|e| e.into_inner()).clone()
}

static ENV: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// See `crate::set_journal_env`.
pub fn set_env(env: Option<&str>) {
    *ENV.write().unwrap_or_else(|e| e.into_inner()) = env.map(str::to_owned);
}

/// `<git-dir>/gitpoulpi/<what>` in the flat layout, and
/// `<git-dir>/gitpoulpi/env/<env>/<what>` in the environment layout.
pub(super) fn env_file(git_dir: &Path, env: Option<&str>, what: &str) -> PathBuf {
    let base = git_dir.join("gitpoulpi");
    match env {
        Some(env) => base.join("env").join(env).join(what),
        None => base.join(what),
    }
}

pub(super) fn journal_path(repo: &RepoRef) -> Option<PathBuf> {
    git_dir(&repo.root).map(|dir| env_file(&dir, current_env().as_deref(), "journal.json"))
}

/// A journal file's git directory and environment (`None` for the flat
/// `<git-dir>/gitpoulpi/journal.json`).
fn locate(path: &Path) -> Option<(PathBuf, Option<String>)> {
    let dir = path.parent()?;
    let up = dir.parent()?;
    if up.file_name().is_some_and(|n| n == "env") {
        let env = dir.file_name()?.to_string_lossy().into_owned();
        Some((up.parent()?.parent()?.to_path_buf(), Some(env)))
    } else {
        Some((up.to_path_buf(), None))
    }
}

/// Marks that the legacy journal of a repository has been adopted.
const ADOPTED_MARKER: &str = "legacy-journal-adopted";

/// The first environment that opens a repository adopts the legacy journal
/// (`<git-dir>/gitpoulpi/journal.json`, written by builds without
/// environments) BY COPY: the file stays for old builds, and a marker
/// (`legacy-journal-adopted`, beside it) makes this happen once per
/// repository, however many environments come later.
///
/// The copied entries keep their legacy pin names
/// (`refs/gitpoulpi/undo/<id>/<n>`), referenced read-only. Re-pointing them
/// into the environment's namespace would need a backend (this runs while
/// the journal is read) and could leave half of the copies behind. Instead
/// `release_pins` never deletes a pin outside the environment's own
/// namespace, so an old build that is still running keeps the pins of its
/// journal, and the history-rewrite purge, which scans the whole
/// `refs/gitpoulpi/undo/` prefix, still finds every pin (`purge::pin_owner`
/// resolves a legacy pin through the entry that lists it). If the old build
/// later prunes an entry, the adopted copy's undo of it refuses (its
/// snapshot is gone); nothing is corrupted.
fn adopt_legacy(path: &Path) {
    let Some((root, Some(env))) = locate(path) else {
        return;
    };
    if path.exists() {
        return;
    }
    let base = root.join("gitpoulpi");
    let marker = base.join(ADOPTED_MARKER);
    let legacy = base.join("journal.json");
    if marker.exists() || !legacy.is_file() {
        return;
    }
    // Another environment already has its own history: this one is not the
    // first to open the repository.
    for sibling in std::fs::read_dir(base.join("env"))
        .into_iter()
        .flatten()
        .flatten()
    {
        if sibling.file_name().to_string_lossy() != env
            && sibling.path().join("journal.json").exists()
        {
            return;
        }
    }
    let Ok(text) = std::fs::read_to_string(&legacy) else {
        return;
    };
    // Unreadable or of another version: left alone, nothing adopted.
    if !serde_json::from_str::<Journal>(&text).is_ok_and(|j| j.version == JOURNAL_VERSION) {
        return;
    }
    let copy = || -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension(format!("json.adopt{}", std::process::id()));
        std::fs::write(&tmp, text.as_bytes())?;
        std::fs::rename(&tmp, path)?;
        std::fs::write(&marker, env.as_bytes())
    };
    if let Err(e) = copy() {
        tracing::warn!("could not adopt the legacy undo journal: {e}");
    }
}

/// Reads the journal; a missing file is an empty one. A version this
/// build does not know is an error and the file is left untouched.
pub(super) fn load(path: &Path) -> std::result::Result<Journal, String> {
    if let Some(journal) = crate::journal_store::cached(path) {
        return Ok(journal);
    }
    adopt_legacy(path);
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
    let located = locate(path);
    journal.pin_ns = located
        .as_ref()
        .map(|(dir, env)| gitpoulpi_core::journal::pin_namespace_in(dir, env.as_deref()))
        .unwrap_or_default();
    let first_time = RECOVERED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashSet::new)
        .insert(path.to_path_buf());
    // A conflicted merge or rebase keeps its `Pending` entry across a
    // restart: it is finished (or dropped) when the operation ends.
    let running = located
        .as_ref()
        .is_some_and(|(git_dir, _)| in_progress(git_dir).is_some());
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
