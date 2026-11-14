use std::path::PathBuf;

use gitpoulpi_core::domain::{Oid, OperationKind, RefScope, RepoRef};
use gitpoulpi_core::journal::{pin_name, JournalEntry, Pin, RemoteRefChange};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;
use serde_json::Value;

use super::after::{after_state, has_pending_history, After};
use super::capture::{capture_before, Before};
use super::labels::label_for;
use super::store::{git_dir, in_progress, journal_path, load, save, save_later, undo_depth};
use crate::dispatch::{repo_ref, Dispatched};
use gitpoulpi_core::time::unix_now as now;

/// How many "Save & Mark Resolved" entries one operation keeps.
const MAX_RESOLUTIONS: usize = 100;

pub(super) fn release_pins(backend: &dyn GitBackend, repo: &RepoRef, entries: &[JournalEntry]) {
    // The saved index and operation state of entries that fell off.
    if let Some(dir) = git_dir(&repo.root) {
        for os in entries.iter().filter_map(|e| e.opstate.as_ref()) {
            crate::opstate::remove(&dir, &os.before);
            crate::opstate::remove(&dir, &os.after);
        }
    }
    let pins: Vec<&Pin> = entries.iter().flat_map(|e| e.pins.iter()).collect();
    if pins.is_empty() {
        return;
    }
    let delete = |pin: &Pin| gitpoulpi_core::domain::RefUpdate::Delete {
        name: pin.name.clone(),
        old: pin.oid,
    };
    let all: Vec<_> = pins.iter().map(|p| delete(p)).collect();
    if backend
        .update_refs(repo, &all, "gitpoulpi: release undo pins")
        .is_err()
    {
        // One bad pin must not keep the others alive.
        for pin in pins {
            let _ = backend.update_refs(repo, &[delete(pin)], "gitpoulpi: release undo pins");
        }
    }
}

/// A write-ahead entry waiting for its operation to finish.
pub struct Recording {
    repo: RepoRef,
    path: PathBuf,
    id: u64,
    before: Before,
    /// Nothing is written until the operation has finished (staging).
    deferred: bool,
}

/// Runs `run` (the operation itself) with the journal around it:
/// write-ahead entry and pins first, the recorded outcome after. Falls
/// back to running the operation unjournaled when the journal cannot be
/// used (no git dir, a journal from a newer version).
pub fn record(
    backend: &dyn GitBackend,
    request: &Request,
    run: &dyn Fn() -> Dispatched,
) -> Dispatched {
    let recording = begin_record(backend, request);
    let mut dispatched = run();
    if let Some(recording) = recording {
        finish_record(backend, request, recording, &mut dispatched);
    }
    dispatched
}

/// The first half of [`record`], split out so an async operation can
/// await its network call between the two halves. `None`: the journal
/// cannot be used, run the operation unrecorded.
pub fn begin_record(backend: &dyn GitBackend, request: &Request) -> Option<Recording> {
    begin_record_with(backend, request, None)
}

/// What a remote deletion adds to its recording: the ref as the server held
/// it, and the local twin deleted by the same request, if any.
pub struct RemoteDelete {
    pub change: RemoteRefChange,
    pub local_request: Option<Request>,
}

/// [`begin_record`] for the deletion of a branch or tag on a remote.
pub fn begin_record_remote(
    backend: &dyn GitBackend,
    request: &Request,
    delete: RemoteDelete,
) -> Option<Recording> {
    begin_record_with(backend, request, Some(delete))
}

fn begin_record_with(
    backend: &dyn GitBackend,
    request: &Request,
    remote: Option<RemoteDelete>,
) -> Option<Recording> {
    if !crate::undo_settings::enabled() {
        return None;
    }
    let repo = repo_ref(request).ok()?;
    let path = journal_path(&repo)?;
    let mut journal = match load(&path) {
        Ok(j) => j,
        Err(m) => {
            tracing::warn!("{m}; this operation is not recorded");
            return None;
        }
    };
    // The commit that concludes a conflicted merge or cherry-pick is the
    // end of that operation's own entry, not an operation of its own.
    if request.command == OperationKind::Commit
        && has_pending_history(&journal)
        && git_dir(&repo.root).is_some_and(|d| in_progress(&d).is_some())
    {
        return None;
    }
    let capture_request = remote
        .as_ref()
        .and_then(|r| r.local_request.as_ref())
        .unwrap_or(request);
    let mut before = match capture_before(backend, &repo, capture_request) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(
                "could not capture the state before {:?}: {e}",
                request.command
            );
            return None;
        }
    };
    if before.unrecordable {
        return None;
    }
    if let Some(remote) = remote {
        // The object the remote held must outlive the deletion: undo pushes
        // it back from here.
        before.pins.extend(remote.change.old);
        before.remote_ref = Some(remote.change);
        before.local_request = remote.local_request;
    }
    if crate::journal_snap::is_snapshot_op(request) {
        match crate::journal_snap::capture(backend, &repo, request, &journal) {
            Ok(Some(captured)) => {
                before.pins.extend(captured.pins());
                before.snap = Some(captured);
            }
            // Not recorded (conflicts, a merge in progress, no paths).
            Ok(None) => return None,
            Err(e) => {
                tracing::warn!(
                    "could not take the safety snapshot before {:?}: {e}",
                    request.command
                );
                return None;
            }
        }
    }

    // Staging only changes the index: record once, after the fact.
    if crate::journal_snap::is_snapshot_op(request)
        && matches!(
            before.snap,
            Some(crate::journal_snap::Captured::Index { .. })
        )
    {
        return Some(Recording {
            repo,
            path,
            id: 0,
            before,
            deferred: true,
        });
    }

    // Write-ahead: the entry and its pins exist before anything changes.
    let id = journal.begin(
        request.command,
        label_for(before.local_request.as_ref().unwrap_or(request)),
        now(),
        before.head.clone(),
        before.require_head,
        before.refs.clone(),
    );
    let mut pins = Vec::new();
    for oid in &before.pins {
        if pins.iter().any(|p: &Pin| p.oid == *oid) {
            continue;
        }
        // Numbered by the pins kept, not by the position in the list: a
        // skipped duplicate must not leave a gap, or the next pin an undo
        // adds (named after the count) would collide with a later slot.
        pins.push(Pin {
            name: pin_name(&journal.pin_ns, id, pins.len()),
            oid: *oid,
        });
    }
    if !pins.is_empty() {
        let creates: Vec<_> = pins
            .iter()
            .map(|p| gitpoulpi_core::domain::RefUpdate::Create {
                name: p.name.clone(),
                new: p.oid,
            })
            .collect();
        if let Err(e) = backend.update_refs(&repo, &creates, "gitpoulpi: pin for undo") {
            // Without the pin the operation could not be undone safely;
            // run it unjournaled rather than refuse the user's action.
            tracing::warn!("could not pin before {:?}: {e}", request.command);
            return None;
        }
    }
    if let Some(entry) = journal.entry_mut(id) {
        entry.pins = pins.clone();
        entry.effect = before.effect;
    }
    if let Err(e) = save(&path, &journal) {
        tracing::warn!("could not write the undo journal: {e}");
        let _ = backend.update_refs(
            &repo,
            &pins
                .iter()
                .map(|p| gitpoulpi_core::domain::RefUpdate::Delete {
                    name: p.name.clone(),
                    old: p.oid,
                })
                .collect::<Vec<_>>(),
            "gitpoulpi: release undo pins",
        );
        return None;
    }
    Some(Recording {
        repo,
        path,
        id,
        before,
        deferred: false,
    })
}

/// The second half of [`record`]: turns the pending entry into a done
/// one, or removes it (and its pins) when the operation failed or
/// changed nothing.
pub fn finish_record(
    backend: &dyn GitBackend,
    request: &Request,
    recording: Recording,
    dispatched: &mut Dispatched,
) {
    let Recording {
        repo,
        path,
        id,
        before,
        deferred,
    } = recording;
    let succeeded = dispatched.response.result.is_ok() && dispatched.mutated;
    // A composer that stopped half-way still made commits: record them.
    let partial_compose =
        request.command == OperationKind::ComposeCommits && dispatched.response.result.is_err();
    let snap_done = match (&before.snap, succeeded || partial_compose) {
        (Some(captured), true) => crate::journal_snap::after(backend, &repo, captured),
        _ => None,
    };
    // A deferred (staging) recording has written nothing yet: when the
    // operation failed or changed nothing, there is nothing to undo.
    if deferred && snap_done.is_none() {
        return;
    }
    // The journal as it is on disk now, not as we left it.
    let mut journal = match load(&path) {
        Ok(j) => j,
        Err(m) => {
            tracing::warn!("{m}; this operation's entry is left pending");
            return;
        }
    };
    let pin_ns = journal.pin_ns.clone();
    // Staging writes its entry only now, in one go (no write-ahead, no
    // second journal write): the index is not data at risk, and both trees
    // are pinned below.
    let id = if deferred {
        journal.begin(
            request.command,
            label_for(request),
            now(),
            None,
            false,
            vec![],
        )
    } else {
        id
    };

    // A merge, rebase or cherry-pick that stopped on a conflict leaves
    // its entry `Pending`; the operation that ends it (continue, skip,
    // the concluding commit, an abort) settles it, see [`settle`].
    if request.command.traits().history
        && git_dir(&repo.root).is_some_and(|d| in_progress(&d).is_some())
    {
        return;
    }
    let ref_based = matches!(
        request.command,
        OperationKind::Reset | OperationKind::ComposeCommits
    );
    // Stashes, remotes and LFS tracking: the module reads what changed.
    let stash_remote_done = match (&before.stash_remote, succeeded) {
        (Some(capture), true) => {
            crate::journal_stashes_remotes::after(backend, &repo, request, capture)
        }
        _ => None,
    };
    // A remote deletion with a local twin is recorded as the local operation
    // would be, with the remote ref added to the same entry.
    let effective = before.local_request.as_ref().unwrap_or(request);
    // A workflow action that deleted branches on a remote reports them.
    let flow_remote: Vec<RemoteRefChange> = match (&dispatched.response.result, request.command) {
        (Ok(value), OperationKind::FlowRun) => value
            .get("remote_deleted")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let own_remote = before.remote_ref.is_some() && succeeded;
    let remote_done = (before.remote_ref.is_some() || !flow_remote.is_empty()) && succeeded;
    let recorded = match &dispatched.response.result {
        Ok(_) if own_remote && before.local_request.is_none() => Some(After {
            head: None,
            refs: vec![],
            config: vec![],
            label: None,
        }),
        Ok(value) if own_remote => {
            after_state(backend, &repo, effective, &before, value).map(|mut after| {
                let remote = before.remote_ref.as_ref().map_or("", |r| r.remote.as_str());
                let base = after.label.take().unwrap_or_else(|| label_for(effective));
                after.label = Some(format!("{base} and on {remote}"));
                after
            })
        }
        Ok(_) if before.stash_remote.is_some() => stash_remote_done.as_ref().map(|d| After {
            head: None,
            refs: d.refs.clone(),
            config: d.config.clone(),
            label: d.label.clone(),
        }),
        Ok(_) if before.snap.is_some() && !ref_based => snap_done.is_some().then(|| After {
            head: None,
            refs: vec![],
            config: vec![],
            label: snap_done.as_ref().and_then(|d| d.label.clone()),
        }),
        Ok(value) if dispatched.mutated => after_state(backend, &repo, request, &before, value),
        Err(_) if partial_compose => after_state(backend, &repo, request, &before, &Value::Null),
        _ => None,
    };
    match recorded {
        Some(after)
            if !after.refs.is_empty()
                || !after.config.is_empty()
                || after.head_moved(&before)
                || snap_done.is_some()
                || remote_done
                || stash_remote_done
                    .as_ref()
                    .is_some_and(|d| d.stash.is_some() || d.index.is_some()) =>
        {
            if let Some(entry) = journal.entry_mut(id) {
                entry.label = after.label.clone().unwrap_or_else(|| entry.label.clone());
                if let (true, Some(change)) = (remote_done, &before.remote_ref) {
                    entry.remote_refs = vec![RemoteRefChange {
                        new: None,
                        ..change.clone()
                    }];
                }
                if !flow_remote.is_empty() {
                    // Pinned now (the delete already dropped the tracking
                    // ref that kept them alive): undo pushes them back.
                    let olds: Vec<Oid> = flow_remote.iter().filter_map(|c| c.old).collect();
                    crate::journal_stashes_remotes::pin_extra(
                        backend, &repo, &pin_ns, entry, &olds,
                    );
                    entry.remote_refs = flow_remote.clone();
                }
                if matches!(
                    request.command,
                    OperationKind::FlowRun | OperationKind::RemoteGroupRun
                ) {
                    entry.effect =
                        flow_effect(before.head.as_ref(), after.head.as_ref(), &after.refs);
                }
                if let Some(done) = &snap_done {
                    entry.files = done.files.clone();
                    entry.index = done.index.clone();
                    entry.conflict = done.conflict.clone();
                    entry.opstate = done.opstate.clone();
                    // The "after" objects are pinned too: redo needs them.
                    let base = entry.pins.len();
                    let wanted: Vec<Oid> = if deferred {
                        before.pins.iter().chain(&done.pins).copied().collect()
                    } else {
                        done.pins.clone()
                    };
                    let mut unique: Vec<Oid> = Vec::new();
                    for oid in wanted {
                        if !unique.contains(&oid) {
                            unique.push(oid);
                        }
                    }
                    let extra: Vec<Pin> = unique
                        .iter()
                        .enumerate()
                        .map(|(n, oid)| Pin {
                            name: pin_name(&pin_ns, id, base + n),
                            oid: *oid,
                        })
                        .collect();
                    let creates: Vec<_> = extra
                        .iter()
                        .map(|p| gitpoulpi_core::domain::RefUpdate::Create {
                            name: p.name.clone(),
                            new: p.oid,
                        })
                        .collect();
                    let mut pinned =
                        backend.update_refs(&repo, &creates, "gitpoulpi: pin for redo");
                    if pinned.is_err() && deferred {
                        // A crash before the write-behind left pins named
                        // for this id: nothing in the journal owns them.
                        pin_after_crash(backend, &repo, &pin_ns, id);
                        pinned = backend.update_refs(&repo, &creates, "gitpoulpi: pin for redo");
                    }
                    match pinned {
                        Ok(()) => entry.pins.extend(extra),
                        Err(e) => {
                            tracing::warn!("could not pin the state after the operation: {e}")
                        }
                    }
                }
            }
            if let (Some(done), Some(entry)) = (&stash_remote_done, journal.entry_mut(id)) {
                entry.stash = done.stash.clone();
                entry.symrefs = done.symrefs.clone();
                entry.index = done.index.clone();
                crate::journal_stashes_remotes::pin_extra(
                    backend, &repo, &pin_ns, entry, &done.pins,
                );
            }
            // A long merge's resolutions must not push the merge's own
            // pending entry off the bounded history: they have their own
            // (generous) cap.
            let resolving = request.command == OperationKind::ResolveConflict
                || snap_done
                    .as_ref()
                    .and_then(|d| d.opstate.as_ref())
                    .is_some_and(|o| !o.aborts);
            let depth = if resolving { usize::MAX } else { undo_depth() };
            let mut dropped =
                journal.finish(id, after.head.clone(), after.refs, after.config, depth);
            if resolving {
                dropped.extend(journal.drop_resolutions(Some(MAX_RESOLUTIONS)));
            }
            release_pins(backend, &repo, &dropped);
            if deferred {
                save_later(&path, &journal);
            } else if let Err(e) = save(&path, &journal) {
                tracing::warn!("could not write the undo journal: {e}");
            }
            dispatched.journal = Some(journal.state(None));
            dispatched.mutated |= partial_compose;
        }
        _ => {
            // Failed, or changed nothing: no entry, no pins.
            if let Some(entry) = journal.abort(id) {
                release_pins(backend, &repo, std::slice::from_ref(&entry));
            }
            if let Err(e) = save(&path, &journal) {
                tracing::warn!("could not write the undo journal: {e}");
            }
        }
    }
}

/// What undoing a workflow action must do to the files: they follow the
/// checked-out branch only when HEAD stayed on a branch that itself moved
/// (a sync, a finish run from the target); when HEAD moved, undo checks the
/// old branch out, which carries the files.
fn flow_effect(
    before: Option<&gitpoulpi_core::domain::HeadState>,
    after: Option<&gitpoulpi_core::domain::HeadState>,
    refs: &[gitpoulpi_core::journal::RefChange],
) -> gitpoulpi_core::journal::Effect {
    use gitpoulpi_core::domain::HeadState;
    use gitpoulpi_core::journal::Effect;
    match (before, after) {
        (Some(a), Some(b)) if a == b => match a {
            HeadState::Branch(name) => {
                let full = format!("refs/heads/{name}");
                if refs
                    .iter()
                    .any(|c| c.name == full && c.old.is_some() && c.new.is_some())
                {
                    Effect::Files
                } else {
                    Effect::Refs
                }
            }
            HeadState::Detached(_) => Effect::Refs,
        },
        _ => Effect::Refs,
    }
}

/// Deletes the pins named for entry `id`, left behind when the daemon
/// stopped before a staging entry's journal write reached the disk (so
/// that id was handed out again).
pub(super) fn pin_after_crash(backend: &dyn GitBackend, repo: &RepoRef, ns: &str, id: u64) {
    let scope = RefScope {
        prefixes: vec![format!(
            "{}{ns}{id}/",
            gitpoulpi_core::domain::UNDO_PIN_PREFIX
        )],
    };
    let Ok(snapshot) = crate::journal_snap::ref_snapshot(backend, repo, &scope) else {
        return;
    };
    let stale: Vec<_> = snapshot
        .refs
        .iter()
        .map(|(name, oid)| gitpoulpi_core::domain::RefUpdate::Delete {
            name: name.clone(),
            old: *oid,
        })
        .collect();
    if !stale.is_empty() {
        let _ = backend.update_refs(repo, &stale, "gitpoulpi: drop pins of a lost entry");
    }
}
