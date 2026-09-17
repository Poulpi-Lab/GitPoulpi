//! Journaling an operation the daemon itself composes ("restore only this",
//! "restore a ref to a reflog position"): the same write-ahead entry and
//! pins as `record`, but the operation is a closure and not a request.

use gitpoulpi_core::domain::{EntrySummary, HeadState, Oid, OperationKind, RefUpdate, RepoRef};
use gitpoulpi_core::journal::{
    pin_name, ConfigChange, Effect, PathChange, Pin, RefChange, StashChange, SymrefChange,
};
use gitpoulpi_core::time::unix_now;
use gitpoulpi_core::{Error, GitBackend, Result};

use super::record::release_pins;
use super::store::{journal_path, load, save, undo_depth};

/// What the entry is, before the operation runs.
pub(super) struct Spec {
    pub kind: OperationKind,
    pub label: String,
    /// HEAD before, when undo must find it where the entry left it.
    pub head: Option<HeadState>,
    pub require_head: bool,
    pub effect: Effect,
    /// The refs the operation may change, with their current values.
    pub refs_before: Vec<(String, Option<Oid>)>,
    /// Oids to keep alive before anything changes.
    pub pins: Vec<Oid>,
    /// Files the operation writes: a safety snapshot is taken before and
    /// after, so undo can put them back.
    pub file_paths: Vec<String>,
}

/// What the operation changed.
#[derive(Default)]
pub(super) struct Done {
    pub head_after: Option<HeadState>,
    pub refs: Vec<RefChange>,
    pub config: Vec<ConfigChange>,
    pub stash: Option<StashChange>,
    pub symrefs: Vec<SymrefChange>,
    /// Further oids to pin (stash commits the operation put back).
    pub pins: Vec<Oid>,
}

/// Runs `run` with a write-ahead journal entry around it. On failure the
/// entry and its pins are removed again and the error is returned.
pub(super) fn run_recorded(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    spec: Spec,
    run: &dyn Fn() -> Result<Done>,
) -> Result<EntrySummary> {
    let path =
        journal_path(repo).ok_or_else(|| Error::Backend("cannot find the git directory".into()))?;
    let mut journal = load(&path).map_err(Error::Backend)?;
    let ns = journal.pin_ns.clone();
    let id = journal.begin(
        spec.kind,
        spec.label.clone(),
        unix_now(),
        spec.head.clone(),
        spec.require_head,
        spec.refs_before.clone(),
    );

    // Pins first: the objects must outlive anything the operation does.
    let mut wanted: Vec<Oid> = Vec::new();
    for oid in &spec.pins {
        if !wanted.contains(oid) {
            wanted.push(*oid);
        }
    }
    let before_snapshot = if spec.file_paths.is_empty() {
        None
    } else {
        let snapshot = backend.snapshot_create(repo, &spec.file_paths)?;
        if !wanted.contains(&snapshot.tree) {
            wanted.push(snapshot.tree);
        }
        Some(snapshot)
    };
    let pins: Vec<Pin> = wanted
        .iter()
        .enumerate()
        .map(|(n, oid)| Pin {
            name: pin_name(&ns, id, n),
            oid: *oid,
        })
        .collect();
    if !pins.is_empty() {
        let creates: Vec<RefUpdate> = pins
            .iter()
            .map(|p| RefUpdate::Create {
                name: p.name.clone(),
                new: p.oid,
            })
            .collect();
        backend.update_refs(repo, &creates, "gitpoulpi: pin for undo")?;
    }
    if let Some(entry) = journal.entry_mut(id) {
        entry.pins = pins;
        entry.effect = spec.effect;
    }
    if let Err(e) = save(&path, &journal) {
        if let Some(entry) = journal.abort(id) {
            release_pins(backend, repo, std::slice::from_ref(&entry));
        }
        return Err(Error::Backend(format!(
            "cannot write the undo journal: {e}"
        )));
    }

    let done = match run() {
        Ok(done) => done,
        Err(e) => {
            // The closure rolled its own changes back.
            let mut journal = load(&path).map_err(Error::Backend)?;
            if let Some(entry) = journal.abort(id) {
                release_pins(backend, repo, std::slice::from_ref(&entry));
            }
            let _ = save(&path, &journal);
            return Err(e);
        }
    };

    // The journal as it is on disk now.
    let mut journal = load(&path).map_err(Error::Backend)?;
    let files = match before_snapshot {
        Some(before) => match backend.snapshot_create(repo, &spec.file_paths) {
            Ok(after) => Some(PathChange {
                paths: spec.file_paths.clone(),
                before,
                after,
            }),
            Err(e) => {
                tracing::warn!("could not snapshot the restored files: {e}");
                None
            }
        },
        None => None,
    };
    let mut extra: Vec<Oid> = done.pins.clone();
    if let Some(f) = &files {
        extra.push(f.after.tree);
    }
    let dropped = journal.finish(id, done.head_after, done.refs, done.config, undo_depth());
    if let Some(entry) = journal.entry_mut(id) {
        entry.stash = done.stash;
        entry.symrefs = done.symrefs;
        entry.files = files;
        crate::journal_stashes_remotes::pin_extra(backend, repo, &ns, entry, &extra);
    }
    release_pins(backend, repo, &dropped);
    let summary = journal
        .undo
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.summary())
        .ok_or_else(|| Error::Backend("the restore's journal entry fell off the history".into()))?;
    save(&path, &journal).map_err(|e| {
        Error::Backend(format!(
            "the restore was applied but the journal could not be saved: {e}"
        ))
    })?;
    Ok(summary)
}
