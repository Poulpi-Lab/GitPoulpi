use gitpoulpi_core::domain::{
    CheckoutTarget, Direction, HeadState, RepoRef, UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::journal::Step;
use gitpoulpi_core::{GitBackend, Result, TypedError};
use gitpoulpi_ipc::Request;
use serde_json::Value;

use super::risk::risk;
use super::state::{gather, ok_json, state_with_facts};
use super::store::{journal_path, load, save};
use crate::dispatch::{err, err_kind, repo_ref, Dispatched};

pub(super) fn refusal(request_id: u64, refusal: UndoRefusal) -> Dispatched {
    err_kind(
        request_id,
        refusal.message(),
        false,
        Some(TypedError::UndoRefused(refusal)),
    )
}

// ------------------------------------------------------- undo and redo

/// `Undo` / `Redo`, under the exclusive repo lock.
pub fn undo_redo(backend: &dyn GitBackend, request: &Request, direction: Direction) -> Dispatched {
    if !crate::undo_settings::enabled() {
        return refusal(request.id, UndoRefusal::Disabled);
    }
    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let expect = request.args.get("expect").and_then(Value::as_u64);
    let dry_run = request
        .args
        .get("dry_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let confirmed = request
        .args
        .get("confirmed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Some(path) = journal_path(&repo) else {
        return refusal(request.id, nothing(direction));
    };
    let mut journal = match load(&path) {
        Ok(j) => j,
        Err(m) => return err(request.id, m, false),
    };
    if journal.top(direction).is_none() {
        return refusal(request.id, nothing(direction));
    }
    let facts = match gather(backend, &repo, &journal, direction) {
        Ok(f) => f,
        Err(e) => return err(request.id, e.to_string(), false),
    };
    if request.args.get("forget").and_then(Value::as_bool) == Some(true) {
        return forget(
            backend,
            request,
            &repo,
            &path,
            journal,
            direction,
            &facts,
            (dry_run, confirmed),
        );
    }
    let plan = match journal.plan(direction, &facts, expect) {
        Ok(p) => p,
        Err(r) => {
            let r = super::restore::with_restore_hint(backend, &repo, r);
            return refusal(request.id, r);
        }
    };
    let risky = risk(backend, &repo, &journal, direction);
    let outcome = |applied| UndoOutcome {
        direction,
        entry: plan.entry.clone(),
        steps: plan.describe(),
        applied,
        needs_confirmation: if applied { None } else { risky.clone() },
    };
    if dry_run {
        return ok_json(request.id, &outcome(false));
    }
    if let (Some(prompt), false) = (&risky, confirmed) {
        return err_kind(
            request.id,
            format!("confirmation needed: {prompt}"),
            false,
            Some(TypedError::NeedsConfirmation(prompt.clone())),
        );
    }

    let message = format!("gitpoulpi: {} {}", direction.verb(), plan.entry.label);
    let mut done: Vec<&Step> = Vec::new();
    for step in &plan.steps {
        if let Err(e) = run_step(backend, &repo, step, &message, &request.session) {
            // Put back what already moved, from the states the plan
            // recorded; the entry stays where it was.
            for earlier in done.iter().rev() {
                if let Err(rollback) = run_step(
                    backend,
                    &repo,
                    &earlier.inverse(),
                    &message,
                    &request.session,
                ) {
                    tracing::warn!("undo rollback step failed: {rollback}");
                }
            }
            return match e.typed() {
                Some(kind) => err_kind(request.id, e.to_string(), false, Some(kind)),
                None => err(request.id, e.to_string(), false),
            };
        }
        done.push(step);
    }
    journal.commit(&plan);
    {
        let dest = match direction {
            Direction::Undo => journal.redo.last_mut(),
            Direction::Redo => journal.undo.last_mut(),
        };
        crate::journal_snap::note_restored(backend, &repo, dest, &plan.steps);
    }
    if let Err(e) = save(&path, &journal) {
        return err(
            request.id,
            format!(
                "the {} was applied but the journal could not be saved: {e}",
                direction.verb()
            ),
            false,
        );
    }
    let state = state_with_facts(backend, &repo, &journal);
    let mut dispatched = ok_json(request.id, &outcome(true));
    dispatched.mutated = true;
    dispatched.journal = Some(state);
    dispatched
}

pub(super) fn nothing(direction: Direction) -> UndoRefusal {
    match direction {
        Direction::Undo => UndoRefusal::NothingToUndo,
        Direction::Redo => UndoRefusal::NothingToRedo,
    }
}

pub(super) fn run_step(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    step: &Step,
    message: &str,
    session: &str,
) -> Result<()> {
    match step {
        Step::UpdateRefs(updates) => backend.update_refs(repo, updates, message),
        Step::Checkout { to, .. } => backend.checkout(
            repo,
            &match to {
                HeadState::Branch(name) => CheckoutTarget::Branch(name.clone()),
                HeadState::Detached(oid) => CheckoutTarget::Commit(*oid),
            },
        ),
        Step::SetConfig { section, to, .. } => backend.set_config_section(repo, section, to),
        Step::MoveHeadKeep { branch, from, to } => backend.move_head_keep(repo, branch, *from, *to),
        Step::RestoreIndex { to, .. } => backend.index_restore(repo, *to),
        Step::RestorePaths { paths, to, .. } => backend.snapshot_restore(repo, *to, paths),
        Step::RestoreOpState { to, .. } => {
            let dir = super::store::git_dir(&repo.root).ok_or_else(|| {
                gitpoulpi_core::Error::Backend("cannot find the git directory".into())
            })?;
            crate::opstate::restore(&dir, to).map_err(|e| {
                gitpoulpi_core::Error::Backend(format!("cannot restore the saved index: {e}"))
            })
        }
        Step::Unresolve { path, entry, .. } => {
            backend.unresolve_conflict(repo, std::path::Path::new(path), entry)
        }
        Step::StashStore { commit, message } => {
            crate::journal_stashes_remotes::run_stash_step(backend, repo, *commit, message, true)
        }
        Step::StashDrop { commit, message } => {
            crate::journal_stashes_remotes::run_stash_step(backend, repo, *commit, message, false)
        }
        Step::RemoteRef {
            remote,
            name,
            to,
            from,
        } => run_remote_step(backend, repo, (remote, name), *to, *from, session),
    }
}

/// Runs a future to its end from the blocking thread a request runs on.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(future),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a tokio runtime")
            .block_on(future),
    }
}

/// Moves a ref on a remote (the undo of a remote deletion pushes the branch
/// or tag back, the redo deletes it again). The same credentials as every
/// other transfer. The server is asked first, and the move is leased on what
/// it said, so a ref changed on the remote since is refused (typed, nothing
/// changed), never overwritten.
fn run_remote_step(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    (remote, name): (&str, &str),
    to: Option<gitpoulpi_core::domain::Oid>,
    from: Option<gitpoulpi_core::domain::Oid>,
    session: &str,
) -> Result<()> {
    use gitpoulpi_core::domain::UndoRefusal;
    use gitpoulpi_core::Error;
    // The grant (an account source) lives until the step is over.
    let _grant = crate::credentials::apply_for(backend, repo, session)?;
    let moved = |actual| {
        Error::UndoRefused(UndoRefusal::RemoteRefMoved(Box::new(
            gitpoulpi_core::domain::RemoteRefMoved {
                remote: remote.to_string(),
                name: name.to_string(),
                expected: from,
                actual,
            },
        )))
    };
    block_on(async {
        let actual = backend.remote_ref_value(repo, remote, name).await?;
        if actual != from {
            return Err(moved(actual));
        }
        match backend
            .remote_ref_update(repo, remote, name, to, from)
            .await
        {
            Err(Error::PushRejected(rejected))
                if rejected.reason == gitpoulpi_core::domain::PushRejectReason::StaleLease =>
            {
                Err(moved(backend.remote_ref_value(repo, remote, name).await?))
            }
            other => other,
        }
    })
}

/// `Undo`/`Redo` with `forget`: drops the top entry of `direction` from the
/// journal when it is blocked for good. Never touches the repository.
#[allow(clippy::too_many_arguments)]
fn forget(
    backend: &dyn GitBackend,
    request: &Request,
    repo: &RepoRef,
    path: &std::path::Path,
    mut journal: gitpoulpi_core::journal::Journal,
    direction: Direction,
    facts: &gitpoulpi_core::journal::RepoFacts,
    (dry_run, confirmed): (bool, bool),
) -> Dispatched {
    let expect = request.args.get("expect").and_then(Value::as_u64);
    match journal.plan(direction, facts, expect) {
        Ok(_) => {
            return err(
                request.id,
                "the top entry is not blocked: undo it instead of forgetting it".into(),
                false,
            )
        }
        Err(r) if !r.forgettable() => return refusal(request.id, r),
        Err(_) => {}
    }
    let Some(top) = journal.top(direction) else {
        return refusal(request.id, nothing(direction));
    };
    let summary = top.summary();
    let outcome = |applied| UndoOutcome {
        direction,
        entry: summary.clone(),
        steps: vec![format!(
            "drop \"{}\" from the journal (the repository is not touched)",
            summary.label
        )],
        applied,
        needs_confirmation: None,
    };
    if dry_run {
        return ok_json(request.id, &outcome(false));
    }
    if !confirmed {
        let prompt = format!(
            "Forget \"{}\"? It leaves the undo history for good; the repository is not touched.",
            summary.label
        );
        return err_kind(
            request.id,
            format!("confirmation needed: {prompt}"),
            false,
            Some(TypedError::NeedsConfirmation(prompt)),
        );
    }
    let gone = match direction {
        Direction::Undo => journal.undo.pop(),
        Direction::Redo => journal.redo.pop(),
    };
    if let Some(entry) = gone {
        super::record::release_pins(backend, repo, &[entry]);
    }
    if let Err(e) = save(path, &journal) {
        return err(
            request.id,
            format!("could not save the journal: {e}"),
            false,
        );
    }
    let state = state_with_facts(backend, repo, &journal);
    let mut dispatched = ok_json(request.id, &outcome(true));
    dispatched.mutated = true;
    dispatched.journal = Some(state);
    dispatched
}
