//! *Start over* for a pending auto-stash: the way out of a pop that
//! conflicted or failed, without keeping the half-applied result.
//!
//! In order: save the stash's files as they are now (a backup ref), discard
//! the working-tree and index changes on the stash's paths (only those), undo
//! the journal entry of the operation that auto-stashed (pull, checkout,
//! merge, rebase), then pop the stash at the commit it was made on and clear
//! the record. The stash is only ever popped, never dropped, before the
//! last step: any failure stops with the stash intact, the files put back
//! from the backup, and the record saying what is true.
//!
//! `Undo` routes here too while a record is pending and the newest journal
//! entry is the operation that auto-stashed (`route_undo`).

use gitpoulpi_core::domain::{
    short, Direction, EntryState, EntrySummary, FileStatus, Oid, OperationKind, RefUpdate, RepoRef,
    ResetMode, UndoOutcome,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_ipc::Request;
use gitpoulpi_session::{PendingAutoStashState, RepoState};
use serde_json::{json, Value};

use crate::autostash::{pending_record, reconcile, state_path, write_pending, Reconciled};
use crate::dispatch::{err, err_kind, ok, Dispatched};

/// Everything checked before anything is touched.
struct Plan {
    commit: Oid,
    paths: Vec<String>,
    entry: EntrySummary,
    staging_above: usize,
}

fn covered(paths: &[String], file: &str) -> bool {
    paths
        .iter()
        .any(|p| p == file || file.starts_with(&format!("{p}/")))
}

fn plan(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Plan, String> {
    if let Reconciled::Cleared(was) = reconcile(backend, repo) {
        return Err(format!(
            "the auto-stash (stash@{{{}}}) is no longer in the stash list; the record was cleared",
            was.0
        ));
    }
    let pending = pending_record(repo).ok_or("there is no pending auto-stash")?;
    let commit = pending
        .stash_commit
        .ok_or("the pending auto-stash is not identified by a commit; use Pop or Drop instead")?;
    if let Some(op) = crate::journal::git_dir(&repo.root)
        .as_deref()
        .and_then(crate::journal::in_progress)
    {
        return Err(format!(
            "a {op} is in progress; finish or abort it first, then start over"
        ));
    }
    if !crate::undo_settings::enabled() {
        return Err(
            "the undo journal is off (general.undo_enabled), so the operation that \
                    stashed cannot be undone; pop or drop the stash instead"
                .into(),
        );
    }
    let (entry, above) = crate::journal::start_over_target(repo)?
        .ok_or("nothing in the undo journal to take back; pop or drop the stash instead")?;
    if entry.kind != pending.reason || entry.state != EntryState::Done {
        return Err(format!(
            "the newest operation is `{}`, not the {:?} that stashed your changes; start over \
             would have to undo it first (use Undo, then try again), or pop or drop the stash",
            entry.label, pending.reason
        ));
    }
    // The undo itself, asked about before anything is touched. With staging
    // steps stacked on the entry the question cannot be put yet (they go
    // first); a failure then is rolled back instead.
    if above.is_empty() {
        let probe = crate::journal::undo_redo(
            backend,
            &Request {
                id: 0,
                session: String::new(),
                repo: Some(repo.id.clone()),
                command: OperationKind::Undo,
                args: json!({ "expect": entry.id, "dry_run": true }),
            },
            Direction::Undo,
        );
        if let Err(e) = probe.response.result {
            return Err(format!("{} cannot be undone: {}", entry.label, e.message));
        }
    }
    let paths = backend
        .stash_paths(repo, commit)
        .map_err(|e| format!("cannot read the stash: {e}"))?;
    let tree = backend
        .status(repo)
        .map_err(|e| format!("cannot read the status: {e}"))?;
    let others: Vec<String> = tree
        .files
        .iter()
        .filter(|f| f.worktree != Some(FileStatus::Untracked))
        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
        .filter(|p| !covered(&paths, p))
        .collect();
    if !others.is_empty() {
        let shown: Vec<&str> = others.iter().take(5).map(String::as_str).collect();
        return Err(format!(
            "{} file(s) have local changes the stash does not hold ({}{}); undoing the {:?} \
             could lose them. Commit, stash or discard them first",
            others.len(),
            shown.join(", "),
            if others.len() > shown.len() {
                ", ..."
            } else {
                ""
            },
            pending.reason
        ));
    }
    Ok(Plan {
        commit,
        paths,
        entry,
        staging_above: above.len(),
    })
}

fn steps(plan: &Plan, backup: &str) -> Vec<String> {
    let mut out = vec![
        format!(
            "keep a copy of the {} file(s) as they are now under {backup}",
            plan.paths.len()
        ),
        format!(
            "discard the changes to those {} file(s) in the working tree and the index",
            plan.paths.len()
        ),
    ];
    if plan.staging_above > 0 {
        out.push(format!(
            "forget {} staging step(s) in the undo history",
            plan.staging_above
        ));
    }
    out.push(format!("undo {}", plan.entry.label));
    out.push(format!(
        "put the stash back with a pop on the original commit ({})",
        short(plan.commit)
    ));
    out
}

fn prompt(plan: &Plan, backup: &str) -> String {
    format!("Start over: {}.", steps(plan, backup).join("; then "))
}

fn outcome(plan: &Plan, backup: &str, applied: bool) -> UndoOutcome {
    UndoOutcome {
        direction: Direction::Undo,
        entry: plan.entry.clone(),
        steps: steps(plan, backup),
        applied,
        needs_confirmation: (!applied).then(|| prompt(plan, backup)),
    }
}

/// `OperationKind::AutoStashStartOver`.
pub fn start_over(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    let repo = match crate::dispatch::repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let mut dispatched = match plan(backend, &repo) {
        Ok(plan) => run(backend, request, &repo, plan),
        Err(m) => err(request.id, m, false),
    };
    // The record may have been cleared or rewritten on the way.
    dispatched.pending_auto_stash_changed = true;
    dispatched
}

/// `Undo` while a record is pending: `Some` when Start over is what the
/// user means (the newest entry is the operation that auto-stashed), else
/// `None` and the usual refusal stands.
pub fn route_undo(backend: &dyn GitBackend, request: &Request) -> Option<Dispatched> {
    let repo = crate::dispatch::repo_ref(request).ok()?;
    pending_record(&repo)?;
    plan(backend, &repo).ok()?;
    Some(start_over(backend, request))
}

fn flag(request: &Request, name: &str) -> bool {
    request
        .args
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn run(backend: &dyn GitBackend, request: &Request, repo: &RepoRef, plan: Plan) -> Dispatched {
    let backup = format!(
        "refs/gitpoulpi-backup/autostash-start-over-{}",
        gitpoulpi_core::time::unix_now()
    );
    if flag(request, "dry_run") {
        return json_ok(request.id, &outcome(&plan, &backup, false), false);
    }
    if !flag(request, "confirmed") {
        let p = prompt(&plan, &backup);
        return err_kind(
            request.id,
            format!("confirmation needed: {p}"),
            false,
            Some(TypedError::NeedsConfirmation(p)),
        );
    }

    // 1. A copy of what is about to be discarded.
    let snapshot = if plan.paths.is_empty() {
        None
    } else {
        let taken = backend.snapshot_create(repo, &plan.paths).and_then(|snap| {
            backend.update_refs(
                repo,
                &[RefUpdate::Create {
                    name: backup.clone(),
                    new: snap.tree,
                }],
                "gitpoulpi: start over backup",
            )?;
            Ok(snap)
        });
        match taken {
            Ok(snap) => Some(snap),
            Err(e) => {
                return err(
                    request.id,
                    format!("could not keep a copy of your files, nothing was changed: {e}"),
                    false,
                )
            }
        }
    };
    let put_back = |why: &str| -> String {
        match &snapshot {
            Some(snap) => match backend.snapshot_restore(repo, snap.tree, &plan.paths) {
                Ok(()) => format!("{why}; your files were put back as they were"),
                Err(e) => format!(
                    "{why}; putting your files back failed ({e}), they are saved under {backup}"
                ),
            },
            None => why.to_string(),
        }
    };

    // 2. Discard on the stash's paths only. A conflicted pop leaves unmerged
    // index entries that `discard` does not clear and that block the undo's
    // `reset --keep`; nothing else is staged (checked in `plan`), so the
    // index goes back to HEAD first.
    let unmerged = backend.status(repo).is_ok_and(|t| {
        t.files.iter().any(|f| {
            f.index == Some(FileStatus::Conflicted) || f.worktree == Some(FileStatus::Conflicted)
        })
    });
    if unmerged {
        let reset = backend
            .resolve(repo, "HEAD")
            .and_then(|head| backend.reset(repo, head, ResetMode::Mixed));
        if let Err(e) = reset {
            let msg = put_back(&format!("could not clear the conflicts: {e}"));
            return err(request.id, format!("{msg}; the stash is intact"), false);
        }
    }
    if let Err(e) = backend.discard(repo, &plan.paths) {
        let msg = put_back(&format!("could not discard the changes: {e}"));
        return err(request.id, format!("{msg}; the stash is intact"), false);
    }
    // 3. Staging steps stacked on the operation described an index that is gone.
    if let Err(e) = crate::journal::forget_staging_above(backend, repo, plan.entry.id) {
        let msg = put_back(&format!("could not tidy the undo history: {e}"));
        return err(request.id, format!("{msg}; the stash is intact"), false);
    }
    // 4. Take the operation back.
    let undo = crate::journal::undo_redo(
        backend,
        &Request {
            command: OperationKind::Undo,
            args: json!({ "expect": plan.entry.id, "confirmed": true }),
            ..request.clone()
        },
        Direction::Undo,
    );
    let mut journal = undo.journal.clone();
    if let Err(e) = &undo.response.result {
        let msg = put_back(&format!(
            "could not undo {}: {}",
            plan.entry.label, e.message
        ));
        let mut failed = err(request.id, format!("{msg}; the stash is intact"), false);
        failed.mutated = true;
        return failed;
    }
    // 5. Pop at the original commit.
    let id = crate::journal_snap::stashes(backend, repo)
        .ok()
        .and_then(|list| list.into_iter().find(|s| s.commit == Some(plan.commit)))
        .map(|s| s.id);
    let Some(id) = id else {
        write_pending(&state_path(repo), None);
        let mut failed = err(
            request.id,
            format!(
                "undid {}, but the stash is no longer in the stash list",
                plan.entry.label
            ),
            false,
        );
        failed.mutated = true;
        failed.journal = journal;
        return failed;
    };
    let popped = crate::dispatch::dispatch(
        backend,
        &Request {
            command: OperationKind::StashPop,
            args: json!({ "id": id }),
            ..request.clone()
        },
    );
    journal = popped.journal.clone().or(journal);
    match &popped.response.result {
        Ok(_) => {
            write_pending(&state_path(repo), None);
            let mut done = json_ok(request.id, &outcome(&plan, &backup, true), true);
            done.journal = journal;
            done
        }
        Err(e) => {
            // `dispatch` already recorded what the failed pop left (Conflicted
            // or PopFailed); the stash is still in the list.
            let state = match RepoState::load(&state_path(repo))
                .ok()
                .and_then(|s| s.pending_auto_stash)
                .map(|p| p.state)
            {
                Some(PendingAutoStashState::Conflicted) => "with conflicts",
                _ => "nothing was applied",
            };
            let mut failed = err(
                request.id,
                format!(
                    "undid {}, but putting your stashed changes back failed ({}; {state}); they \
                     are still in the stash list (stash@{{{}}})",
                    plan.entry.label, e.message, id.0
                ),
                false,
            );
            failed.mutated = true;
            failed.journal = journal;
            failed
        }
    }
}

fn json_ok<T: serde::Serialize>(id: u64, value: &T, mutated: bool) -> Dispatched {
    match serde_json::to_value(value) {
        Ok(v) => ok(id, v, mutated),
        Err(e) => err(id, format!("failed to encode result: {e}"), false),
    }
}
