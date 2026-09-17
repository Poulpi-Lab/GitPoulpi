use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use gitpoulpi_core::domain::{RefScope, RepoRef};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;

use super::store::{git_dir, journal_path, RECOVERED};
use crate::dispatch::repo_ref;

/// Repositories this daemon has served, and whether their undo history
/// was already cleared since undo was turned off.
pub(super) static SERVED: Mutex<Option<std::collections::HashMap<PathBuf, (RepoRef, bool)>>> =
    Mutex::new(None);

/// Called with every request. While `general.undo_enabled` is off, the
/// undo history of every repository this daemon knows (and of any other
/// the first time it is opened) is erased once: the journal file, its
/// in-memory copy and every `refs/gitpoulpi/undo/` pin. Turning undo back
/// on forgets what was cleared, so a later "off" clears again. Costs a
/// config stamp check and a lock when nothing is to be done.
pub fn housekeeping(backend: &dyn GitBackend, request: &Request) {
    let Ok(repo) = repo_ref(request) else {
        return;
    };
    let on = crate::undo_settings::enabled();
    let todo: Vec<RepoRef> = {
        let mut guard = SERVED.lock().unwrap_or_else(|e| e.into_inner());
        let served = guard.get_or_insert_with(Default::default);
        served
            .entry(repo.root.clone())
            .or_insert_with(|| (repo.clone(), false));
        if on {
            for (_, cleared) in served.values_mut() {
                *cleared = false;
            }
            return;
        }
        served
            .values_mut()
            .filter(|(_, cleared)| !*cleared)
            .map(|(repo, cleared)| {
                *cleared = true;
                repo.clone()
            })
            .collect()
    };
    for repo in todo {
        erase_history(backend, &repo);
    }
}

/// Deletes the journal and every undo pin of `repo`.
pub(super) fn erase_history(backend: &dyn GitBackend, repo: &RepoRef) {
    super::outside::erase(repo);
    if let Some(path) = journal_path(repo) {
        crate::journal_store::forget(&path);
        match std::fs::remove_file(&path) {
            Ok(()) => {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::remove_dir(dir);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!("could not delete the undo journal: {e}"),
        }
        RECOVERED
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_or_insert_with(HashSet::new)
            .remove(&path);
    }
    let scope = RefScope {
        prefixes: vec![gitpoulpi_core::domain::UNDO_PIN_PREFIX.to_string()],
    };
    let Ok(snapshot) = crate::journal_snap::ref_snapshot(backend, repo, &scope) else {
        return;
    };
    // Only this worktree's pins: a linked worktree's journal lives apart
    // and keeps its own (`pin_name`).
    let ns = git_dir(&repo.root)
        .map(|d| gitpoulpi_core::journal::pin_namespace(&d))
        .unwrap_or_default();
    let all: Vec<_> = snapshot
        .refs
        .iter()
        .filter(|(name, _)| {
            let rest = name
                .strip_prefix(gitpoulpi_core::domain::UNDO_PIN_PREFIX)
                .unwrap_or("");
            if ns.is_empty() {
                rest.starts_with(|c: char| c.is_ascii_digit())
            } else {
                rest.starts_with(&ns)
            }
        })
        .map(|(name, oid)| gitpoulpi_core::domain::RefUpdate::Delete {
            name: name.clone(),
            old: *oid,
        })
        .collect();
    if all.is_empty() {
        return;
    }
    if backend
        .update_refs(repo, &all, "gitpoulpi: erase the undo history")
        .is_err()
    {
        for update in all {
            let _ = backend.update_refs(repo, &[update], "gitpoulpi: erase the undo history");
        }
    }
}

/// Writes any journal still waiting behind a staging operation (daemon
/// shutdown, tests that read the file).
pub fn flush_pending() {
    crate::journal_store::flush();
}
