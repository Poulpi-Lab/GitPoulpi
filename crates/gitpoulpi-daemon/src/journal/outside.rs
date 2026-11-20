//! Changes made outside GitPoulpi, recorded as journal entries
//! (the outside-operations design).
//!
//! The daemon keeps the refs it last saw (`<git-dir>/gitpoulpi/refs-snapshot.json`,
//! next to `journal.json`; both under `env/<env>/` in the environment layout,
//! so one environment records another's operations as outside changes). [`sync`] diffs the repository against it and
//! records one `OperationKind::Outside` entry per coherent change; [`adopt`]
//! refreshes the snapshot after one of the daemon's own mutations, so those
//! are never recorded twice.
//!
//! Order matters: a mutation first syncs (what happened outside before it is
//! recorded before its own entry), then runs, then adopts.
//!
//! Detection cost: a read of the refs, HEAD, the stash list and the worktree
//! list, skipped entirely while a cheap fingerprint of the ref storage (HEAD,
//! `packed-refs`, the directories of `refs/`, the stash reflog) says nothing
//! moved. The fingerprint is not trusted for a few seconds after the newest
//! file time it saw (the file system's clock is coarse), like git's racy
//! index.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use gitpoulpi_core::domain::{JournalState, RepoRef};
use gitpoulpi_core::journal::{
    diff_outside, pin_name, CaptureScope, Journal, OutsideSnapshot, Pin, OUTSIDE_SNAPSHOT_VERSION,
};
use gitpoulpi_core::GitBackend;

use super::state::state_with_facts;
use super::store::{git_dir, in_progress, journal_path, load, save, undo_depth};
use gitpoulpi_core::time::unix_now as now;

/// One sync or adopt at a time, process wide: they read and write the same
/// snapshot and journal files from reads that share the repository lock.
static GUARD: Mutex<()> = Mutex::new(());
/// Network transfers running now: a fetch is not an outside change, and its
/// half-written state must not be diffed.
static TRANSFERS: AtomicUsize = AtomicUsize::new(0);
/// Snapshot files this process has already compared once: the first
/// comparison covers whatever happened while no daemon was running.
static SEEN: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);
/// The fingerprint of the ref storage when the snapshot file was last
/// written or confirmed, and whether it is too fresh to be trusted.
static PRINTS: Mutex<Option<HashMap<PathBuf, (u64, bool)>>> = Mutex::new(None);

/// A transfer starts (`Fetch`, `Push`, `Pull`...): outside detection waits.
pub fn transfer_started() {
    TRANSFERS.fetch_add(1, Ordering::SeqCst);
}

pub fn transfer_finished() {
    TRANSFERS.fetch_sub(1, Ordering::SeqCst);
}

/// Repositories with a request in flight that owns outside detection: a
/// request that calls `dispatch` again for a step of its own (auto-stash
/// pops, start over) must not sync in the middle of itself.
static BUSY: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

/// Held by the outermost request on a repository.
pub struct Busy(PathBuf);

impl Busy {
    /// `None` when another request on `root` already owns detection.
    pub fn enter(root: &Path) -> Option<Busy> {
        let fresh = BUSY
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_or_insert_with(HashSet::new)
            .insert(root.to_path_buf());
        fresh.then(|| Busy(root.to_path_buf()))
    }
}

impl Drop for Busy {
    fn drop(&mut self) {
        BUSY.lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_or_insert_with(HashSet::new)
            .remove(&self.0);
    }
}

fn snapshot_path(repo: &RepoRef) -> Option<PathBuf> {
    // Per environment, like the journal: each environment records the other's
    // operations as outside changes.
    git_dir(&repo.root).map(|d| {
        super::store::env_file(
            &d,
            super::store::current_env().as_deref(),
            "refs-snapshot.json",
        )
    })
}

/// The directory refs, `packed-refs` and the stash live in: the common git
/// dir of a linked worktree, else the git dir itself.
fn common_dir(git_dir: &Path) -> PathBuf {
    match std::fs::read_to_string(git_dir.join("commondir")) {
        Ok(rel) => git_dir.join(rel.trim()),
        Err(_) => git_dir.to_path_buf(),
    }
}

fn read_snapshot(path: &Path) -> Option<OutsideSnapshot> {
    let text = std::fs::read_to_string(path).ok()?;
    let snap: OutsideSnapshot = serde_json::from_str(&text).ok()?;
    (snap.version == OUTSIDE_SNAPSHOT_VERSION).then_some(snap)
}

fn write_snapshot(path: &Path, snap: &OutsideSnapshot) {
    let write = || -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension(format!("json.tmp{}", std::process::id()));
        std::fs::write(
            &tmp,
            serde_json::to_vec(snap).map_err(std::io::Error::other)?,
        )?;
        std::fs::rename(&tmp, path)
    };
    if let Err(e) = write() {
        tracing::warn!("could not write the refs snapshot {}: {e}", path.display());
    }
}

/// A hash of the ref storage's modification times, and whether the newest
/// of them is too recent to tell two states apart.
fn fingerprint(git_dir: &Path) -> (u64, bool) {
    let common = common_dir(git_dir);
    let mut hasher = DefaultHasher::new();
    let mut newest = SystemTime::UNIX_EPOCH;
    let mut stat = |path: &Path, hasher: &mut DefaultHasher| {
        path.hash(hasher);
        match std::fs::metadata(path) {
            Ok(meta) => {
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                modified.hash(hasher);
                meta.len().hash(hasher);
                newest = newest.max(modified);
            }
            Err(_) => 0u8.hash(hasher),
        }
    };
    stat(&git_dir.join("HEAD"), &mut hasher);
    stat(&git_dir.join("logs").join("HEAD"), &mut hasher);
    stat(&common.join("packed-refs"), &mut hasher);
    stat(&common.join("refs").join("stash"), &mut hasher);
    stat(&common.join("logs").join("refs").join("stash"), &mut hasher);
    stat(&common.join("worktrees"), &mut hasher);
    // Reftable keeps every ref and reflog in `reftable/`: each change writes
    // a table and rewrites `tables.list`, and nothing under `refs/` moves.
    stat(&common.join("reftable"), &mut hasher);
    stat(&common.join("reftable").join("tables.list"), &mut hasher);
    let mut dirs = vec![common.join("refs")];
    while let Some(dir) = dirs.pop() {
        stat(&dir, &mut hasher);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut subdirs: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.path())
            // Our own pins churn this tree and are not outside changes.
            .filter(|p| !p.ends_with("gitpoulpi"))
            .collect();
        subdirs.sort();
        dirs.extend(subdirs);
    }
    // Branch reflogs grow without touching the ref's directory.
    stat(&common.join("logs").join("refs").join("heads"), &mut hasher);
    let racy = newest
        .elapsed()
        .map(|age| age < Duration::from_secs(3))
        .unwrap_or(true);
    (hasher.finish(), racy)
}

fn remember_print(path: &Path, print: (u64, bool)) {
    PRINTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashMap::new)
        .insert(path.to_path_buf(), print);
}

/// The fingerprint says nothing moved since the snapshot was confirmed.
fn unchanged_since(path: &Path, now_print: (u64, bool)) -> bool {
    let guard = PRINTS.lock().unwrap_or_else(|e| e.into_inner());
    matches!(
        guard.as_ref().and_then(|m| m.get(path)),
        Some((hash, false)) if *hash == now_print.0
    )
}

/// Snapshot files left behind while an operation was stopped (a conflicted
/// merge, a pending entry): what moved since is that operation's own, so the
/// next comparison adopts it instead of recording it. Anything an outside
/// process did meanwhile is folded into it: the price of not guessing in the
/// middle of an operation.
static TAINTED: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

fn taint(path: &Path) {
    TAINTED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashSet::new)
        .insert(path.to_path_buf());
}

fn untaint(path: &Path) -> bool {
    TAINTED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashSet::new)
        .remove(path)
}

fn first_time(path: &Path) -> bool {
    SEEN.lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(HashSet::new)
        .insert(path.to_path_buf())
}

/// Which optional reads are worth making: a stash list only when the repo
/// has (or had) stashes, the worktree list only when it has (or had) linked
/// worktrees. On the system backend each read is a process.
fn capture_scope(git_dir: &Path, before: Option<&OutsideSnapshot>) -> CaptureScope {
    let common = common_dir(git_dir);
    let stashes = common.join("refs").join("stash").exists()
        || std::fs::read_to_string(common.join("packed-refs"))
            .is_ok_and(|p| p.lines().any(|l| l.ends_with(" refs/stash")))
        || before.is_some_and(|b| !b.stashes.is_empty());
    let worktrees = common.join("worktrees").is_dir()
        || before.is_some_and(|b| b.worktrees.iter().any(|w| !w.is_main));
    CaptureScope { stashes, worktrees }
}

fn locked(git_dir: &Path) -> bool {
    let common = common_dir(git_dir);
    [
        git_dir.join("HEAD.lock"),
        common.join("packed-refs.lock"),
        common.join("refs").join("stash.lock"),
    ]
    .iter()
    .any(|p| p.exists())
}

/// How a request relates to the journal: a mutation holds the repository
/// exclusively, a read shares it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Caller {
    Mutation,
    Read,
}

/// Records what changed outside GitPoulpi since the snapshot, as entries.
/// `Some` with the journal state when it recorded any. Never blocks on a
/// repository another process is changing (a rebase, a lock): it waits for
/// the next call and records the final state only.
pub fn sync(backend: &dyn GitBackend, repo: &RepoRef, caller: Caller) -> Option<JournalState> {
    if !crate::undo_settings::outside_enabled() {
        forget_snapshot(repo);
        return None;
    }
    let dir = git_dir(&repo.root)?;
    let snap_path = snapshot_path(repo)?;
    let _guard = GUARD.lock().unwrap_or_else(|e| e.into_inner());
    // A rebase or merge somebody else is running: wait for its end, then
    // record the final state only.
    if in_progress(&dir).is_some() {
        return None;
    }
    if caller == Caller::Read && (TRANSFERS.load(Ordering::SeqCst) > 0 || locked(&dir)) {
        return None;
    }
    let path = journal_path(repo)?;
    let mut journal = match load(&path) {
        Ok(j) => j,
        Err(m) => {
            tracing::warn!("{m}; outside changes are not recorded");
            return None;
        }
    };
    // Our own operation is running (a conflicted merge keeps its entry
    // pending): what it moves is its own, not an outside change.
    if super::after::has_pending_history(&journal) {
        taint(&snap_path);
        return None;
    }
    let print = fingerprint(&dir);
    let Some(before) = read_snapshot(&snap_path) else {
        // The first look at this repository: a baseline, nothing to report.
        let snap =
            OutsideSnapshot::capture_scoped(backend, repo, capture_scope(&dir, None)).ok()?;
        write_snapshot(&snap_path, &snap);
        first_time(&snap_path);
        remember_print(&snap_path, print);
        return None;
    };
    let adopting = untaint(&snap_path);
    if !adopting && unchanged_since(&snap_path, print) {
        return None;
    }
    let after =
        OutsideSnapshot::capture_scoped(backend, repo, capture_scope(&dir, Some(&before))).ok()?;
    if adopting {
        write_snapshot(&snap_path, &after);
        remember_print(&snap_path, fingerprint(&dir));
        return None;
    }
    if before == after {
        remember_print(&snap_path, print);
        return None;
    }
    let closed = first_time(&snap_path);
    let diff = match diff_outside(backend, repo, &before, &after) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("could not diff the refs snapshot: {e}");
            return None;
        }
    };
    let recorded = !diff.is_quiet();
    let mut released = Vec::new();
    for change in &diff.changes {
        let mut label = change.label();
        if closed {
            label.push_str(" (while GitPoulpi was closed)");
        }
        let pins = pin_all(backend, repo, &journal, &change.pin_oids());
        let (_, dropped) = journal.record_outside(
            change,
            &after.head,
            label,
            change.time.unwrap_or_else(now),
            pins,
            undo_depth(),
        );
        released.extend(dropped);
    }
    if recorded {
        super::record::release_pins(backend, repo, &released);
        if let Err(e) = save(&path, &journal) {
            tracing::warn!("could not write the undo journal: {e}");
            return None;
        }
    }
    write_snapshot(&snap_path, &after);
    // The pins just written changed the ref storage: take the print after.
    remember_print(&snap_path, fingerprint(&dir));
    recorded.then(|| state_with_facts(backend, repo, &journal))
}

/// Pins every object an outside entry may need back, one ref each, named for
/// the entry about to be written. A pin that cannot be made (the object is
/// gone) is left out: the entry is still recorded.
fn pin_all(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
    oids: &[gitpoulpi_core::domain::Oid],
) -> Vec<Pin> {
    let id = journal.next_id;
    let wanted: Vec<Pin> = oids
        .iter()
        .enumerate()
        .map(|(n, oid)| Pin {
            name: pin_name(&journal.pin_ns, id, n),
            oid: *oid,
        })
        .collect();
    let create = |pin: &Pin| gitpoulpi_core::domain::RefUpdate::Create {
        name: pin.name.clone(),
        new: pin.oid,
    };
    let all: Vec<_> = wanted.iter().map(create).collect();
    if backend
        .update_refs(repo, &all, "gitpoulpi: pin an outside change")
        .is_ok()
    {
        return wanted;
    }
    wanted
        .into_iter()
        .filter(|pin| {
            backend
                .update_refs(repo, &[create(pin)], "gitpoulpi: pin an outside change")
                .is_ok()
        })
        .collect()
}

/// Makes the snapshot what the repository is now, without recording
/// anything: called after the daemon's own mutation, so that its moves are
/// not mistaken for outside ones. Does nothing while a merge, rebase or
/// cherry-pick is stopped (the snapshot is then taken when it ends).
pub fn adopt(backend: &dyn GitBackend, repo: &RepoRef) {
    if !crate::undo_settings::outside_enabled() {
        return;
    }
    let (Some(dir), Some(snap_path)) = (git_dir(&repo.root), snapshot_path(repo)) else {
        return;
    };
    let _guard = GUARD.lock().unwrap_or_else(|e| e.into_inner());
    // A merge, rebase or cherry-pick that stopped: HEAD and the refs are
    // mid-way; the snapshot is taken when it ends.
    if in_progress(&dir).is_some() {
        taint(&snap_path);
        return;
    }
    // Nothing to adopt into: the baseline is made by the first sync.
    if !snap_path.exists() {
        return;
    }
    let before = read_snapshot(&snap_path);
    let Ok(snap) =
        OutsideSnapshot::capture_scoped(backend, repo, capture_scope(&dir, before.as_ref()))
    else {
        return;
    };
    write_snapshot(&snap_path, &snap);
    remember_print(&snap_path, fingerprint(&dir));
}

/// Drops the snapshot (outside recording was turned off): turning it back
/// on starts from a fresh baseline instead of reporting what happened while
/// it was off as one old change.
fn forget_snapshot(repo: &RepoRef) {
    if let Some(path) = snapshot_path(repo) {
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        PRINTS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_or_insert_with(HashMap::new)
            .remove(&path);
    }
}

/// Removes the snapshot with the rest of the history (`erase_history`).
pub(super) fn erase(repo: &RepoRef) {
    forget_snapshot(repo);
}
