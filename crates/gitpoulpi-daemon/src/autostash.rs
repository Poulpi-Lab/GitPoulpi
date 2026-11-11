//! `general.auto_stash`: when a checkout, merge, rebase or pull is blocked
//! by local changes, stash them, run the operation again, and pop the
//! stash afterwards.
//!
//! The operation always runs first; this module only acts on a *failure
//! that looks like local changes in the way* while the tree really has
//! something to stash, so a clean tree never sees a stash.
//!
//! * `never`: nothing here acts, the error stands.
//! * `ask`: the error becomes the typed [`Error::NeedsAutoStash`]; a UI
//!   asks and resends the request with `autostash: "always"`, a CLI/MCP
//!   caller is told to pass `--autostash`.
//! * `always`: stash, run again, pop.
//!
//! Each step is an ordinary request through `dispatch`, so the stash push,
//! the operation and the pop are three undoable journal entries. The
//! crash-safe record the session crate defines
//! ([`gitpoulpi_session::PendingAutoStash`]) is written as soon as the
//! stash exists and cleared on a clean pop; a pop that conflicts leaves it
//! as `Conflicted` (unmerged paths, the changes are in the tree) or
//! `PopFailed` (nothing applied, the stash is the only copy) and keeps
//! the stash, never retried by itself.
//!
//! A request's own `autostash` argument (`"always"`, `"ask"`, `"never"`)
//! overrides the config.

use gitpoulpi_config::AutoStashMode;
use gitpoulpi_core::domain::{
    CheckoutTarget, FileStatus, OperationKind, RebaseRefusal, RepoRef, StashId,
};
use gitpoulpi_core::{GitBackend, TypedError};
use gitpoulpi_ipc::Request;
use gitpoulpi_session::{PendingAutoStash, PendingAutoStashState, RepoState};
use serde_json::{json, Value};

use crate::dispatch::{err_kind, Dispatched};

/// The word for an operation that must not run while an auto-stash is
/// pending, `None` for everything that may.
///
/// Blocked: what moves HEAD or rewrites history under the stash's base
/// (checkout of a branch/commit, a branch created with tracking (it checks
/// the branch out), merge, rebase, interactive rebase, pull, fast-forward,
/// cherry-pick, revert, reword, compose, undo/redo).
///
/// Allowed: every read, staging and discarding, commit (so a conflicted pop
/// can be resolved and committed), branch/tag/remote bookkeeping, fetch
/// and push, rebase continue/skip/abort, conflict resolution, the
/// resolving operations `stash pop`/`apply`/`drop`, and - the way out of
/// an `Applied` record with a dirty tree - `stash push` (the record is
/// keyed by the stash's commit, so a new stash shifting the index changes
/// nothing; [`settle_manual`] follows it) and `reset` (the stash holds the
/// changes, so nothing it discards is lost).
fn blocked_word(request: &Request) -> Option<&'static str> {
    Some(match request.command {
        OperationKind::Checkout => operation_word(request)?,
        OperationKind::Merge => "merge",
        OperationKind::Rebase | OperationKind::RebaseInteractive => "rebase",
        OperationKind::Pull => "pull",
        OperationKind::FastForward => "fast-forward",
        OperationKind::CherryPick => "cherry-pick",
        OperationKind::Revert => "revert",
        OperationKind::Reword => "reword",
        OperationKind::AmendCommit => "commit amend",
        OperationKind::SplitCommit => "commit split",
        OperationKind::RewriteApply => "history rewrite",
        OperationKind::ComposeCommits => "commit composition",
        OperationKind::BranchCreateTracking => "branch checkout",
        OperationKind::Undo => "undo",
        OperationKind::Redo => "redo",
        OperationKind::FlowRun => flow_word(request)?,
        _ => return None,
    })
}

/// A workflow action is blocked while an auto-stash is pending, except
/// resuming or aborting one (the stash waits for exactly that to end).
fn flow_word(request: &Request) -> Option<&'static str> {
    request
        .args
        .get("resume")
        .is_none_or(Value::is_null)
        .then_some("workflow action")
}

/// The refusal for a blocked operation while the repository has a pending
/// auto-stash record, `None` when the request may go on. The record is
/// read from `repo-state.toml` (one small file); the auto-stash sequence
/// itself never trips this, because its own steps are a stash push that
/// runs before the record exists, a pop, and a re-run that bypasses it.
pub fn refuse_while_pending(request: &Request) -> Option<Dispatched> {
    let operation = blocked_word(request)?;
    let repo = crate::dispatch::repo_ref(request).ok()?;
    let pending = pending_record(&repo)?;
    let error = gitpoulpi_core::Error::PendingAutoStash {
        operation: operation.to_string(),
        stash: pending.stash_id,
        hint: "pop it (`gitpoulpi stash pop`), look at it (`gitpoulpi stash list`), drop it \
               (`gitpoulpi stash drop`) or start over (`gitpoulpi autostash start-over`) first"
            .to_string(),
    };
    Some(err_kind(
        request.id,
        error.to_string(),
        false,
        error.typed(),
    ))
}

pub(crate) fn state_path(repo: &RepoRef) -> std::path::PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    RepoState::path_for(&repo.root, &state_dir)
}

pub(crate) fn pending_record(repo: &RepoRef) -> Option<PendingAutoStash> {
    RepoState::load(&state_path(repo)).ok()?.pending_auto_stash
}

/// The message every auto-stash carries, used to recognise the stash of an
/// old index-only record.
const MARK: &str = "GitPoulpi auto-stash";

/// What [`reconcile`] did to the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reconciled {
    /// No record, or it already matches the stash list.
    Untouched,
    /// The stash is still there, at another index (or an old record got
    /// its commit): the record was rewritten.
    Updated,
    /// The stash is no longer in the list: the record was cleared. Carries
    /// the index it last had.
    Cleared(StashId),
}

/// Brings the pending record in line with the stash list: re-resolves the
/// stash's index from its commit id (an index shifts whenever another stash
/// is pushed or dropped), and clears the record when the commit is gone
/// (dropped or popped with plain git) - otherwise every history operation
/// would stay refused for a stash that no longer exists.
///
/// A record written before commit ids were stored is resolved once: the
/// stash at its index when that is an auto-stash, else the one auto-stash
/// in the list; none -> cleared. The commit is then stored.
pub fn reconcile(backend: &dyn GitBackend, repo: &RepoRef) -> Reconciled {
    let path = state_path(repo);
    let Some(pending) = RepoState::load(&path)
        .ok()
        .and_then(|s| s.pending_auto_stash)
    else {
        return Reconciled::Untouched;
    };
    let Ok(stashes) = crate::journal_snap::stashes(backend, repo) else {
        return Reconciled::Untouched;
    };
    let found = match &pending.stash_commit {
        Some(commit) => stashes.iter().find(|s| s.commit.as_ref() == Some(commit)),
        None => stashes
            .iter()
            .find(|s| s.id == pending.stash_id && s.message.contains(MARK))
            .or_else(|| stashes.iter().find(|s| s.message.contains(MARK))),
    };
    let Some(found) = found else {
        write_pending(&path, None);
        return Reconciled::Cleared(pending.stash_id);
    };
    let commit = found.commit.or(pending.stash_commit);
    if found.id == pending.stash_id && commit == pending.stash_commit {
        return Reconciled::Untouched;
    }
    write_pending(
        &path,
        Some(PendingAutoStash {
            stash_id: found.id,
            stash_commit: commit,
            ..pending
        }),
    );
    Reconciled::Updated
}

/// What [`before`] found out before the request runs.
#[derive(Default)]
pub struct Before {
    /// The request with its stash index resolved from a `commit` argument.
    pub request: Option<Request>,
    /// The request cannot run (its stash is gone).
    pub refused: Option<Dispatched>,
    changed: bool,
    gone: Option<StashId>,
    /// A read popped the stash of an operation that ended outside the app.
    popped: Option<Option<gitpoulpi_core::domain::JournalState>>,
}

impl Before {
    /// Folds what reconciling did into the answer, so the server tells
    /// every client.
    pub fn apply(&self, dispatched: &mut Dispatched) {
        dispatched.pending_auto_stash_changed |= self.changed;
        if let Some(journal) = &self.popped {
            dispatched.mutated = true;
            dispatched.journal = journal.clone().or(dispatched.journal.take());
        }
        if self.gone.is_some() {
            dispatched.pending_auto_stash_gone = self.gone;
        }
    }
}

/// Runs before a request: reconciles the record for the requests that read
/// it (the refresh reads, the stash resolvers, every blocked operation), and
/// resolves a `commit` argument of `stash pop|apply|drop` to the stash's
/// current index, so a client that knows the stash by its commit (the
/// banner) never acts on whichever stash moved into that slot.
pub fn before(backend: &dyn GitBackend, request: &Request) -> Before {
    let mut out = Before::default();
    let by_commit = matches!(
        request.command,
        OperationKind::StashPop | OperationKind::StashApply | OperationKind::StashDrop
    )
    .then(|| request.args.get("commit").cloned())
    .flatten()
    .filter(|v| !v.is_null());
    let reads = matches!(
        request.command,
        OperationKind::Stashes | OperationKind::Status | OperationKind::Refs
    ) || by_commit.is_some()
        || matches!(
            request.command,
            OperationKind::StashPop | OperationKind::StashApply | OperationKind::StashDrop
        )
        || blocked_word(request).is_some();
    if !reads {
        return out;
    }
    let Ok(repo) = crate::dispatch::repo_ref(request) else {
        return out;
    };
    match reconcile(backend, &repo) {
        Reconciled::Untouched => {}
        Reconciled::Updated => out.changed = true,
        Reconciled::Cleared(was) => {
            out.changed = true;
            out.gone = Some(was);
        }
    }
    // A merge or rebase finished with plain git: its stash goes back now.
    if matches!(
        request.command,
        OperationKind::Status | OperationKind::Stashes
    ) {
        if let Some(popped) = pop_when_ended(backend, request) {
            out.changed = true;
            out.popped = Some(popped.journal);
        }
    }
    if let Some(commit) = by_commit {
        let commit: Option<gitpoulpi_core::domain::Oid> = serde_json::from_value(commit).ok();
        let index = commit.and_then(|c| {
            crate::journal_snap::stashes(backend, &repo)
                .ok()?
                .into_iter()
                .find(|s| s.commit.as_ref() == Some(&c))
                .map(|s| s.id)
        });
        match index {
            Some(id) => {
                let mut rewritten = request.clone();
                rewritten.args["id"] = json!(id);
                out.request = Some(rewritten);
            }
            None => {
                let mut refused = err_kind(
                    request.id,
                    "that stash is no longer in the stash list (it was dropped or popped \
                     outside GitPoulpi)"
                        .to_string(),
                    false,
                    None,
                );
                out.apply(&mut refused);
                out.refused = Some(refused);
            }
        }
    }
    out
}

/// A stash this module made and still has to pop.
pub struct Armed {
    id: StashId,
    state_path: std::path::PathBuf,
}

/// The word for the operation in messages, if auto-stash covers it.
fn operation_word(request: &Request) -> Option<&'static str> {
    match request.command {
        OperationKind::Checkout => {
            let target: CheckoutTarget =
                serde_json::from_value(request.args.get("target")?.clone()).ok()?;
            // Restoring one file from a commit is not blocked by anything.
            (!matches!(
                target,
                CheckoutTarget::File { .. } | CheckoutTarget::Files { .. }
            ))
            .then_some("checkout")
        }
        OperationKind::Merge => Some("merge"),
        OperationKind::Rebase => Some("rebase"),
        OperationKind::Pull => Some("pull"),
        OperationKind::FlowRun => Some("workflow action"),
        _ => None,
    }
}

fn requested_mode(request: &Request) -> Option<AutoStashMode> {
    match request.args.get("autostash")? {
        Value::Bool(true) => Some(AutoStashMode::Always),
        Value::Bool(false) => None,
        Value::String(s) => match s.as_str() {
            "always" => Some(AutoStashMode::Always),
            "ask" => Some(AutoStashMode::Ask),
            "never" => Some(AutoStashMode::Never),
            _ => None,
        },
        _ => None,
    }
}

/// Whether a failure is git (or libgit2) refusing because local changes
/// are in the way, as opposed to any other reason the operation failed.
fn blocked_by_local_changes(dispatched: &Dispatched) -> bool {
    let Err(error) = &dispatched.response.result else {
        return false;
    };
    if matches!(
        error.kind,
        Some(TypedError::RebaseRefused(RebaseRefusal::UncommittedChanges))
    ) {
        return true;
    }
    if error.kind.is_some() {
        return false;
    }
    let message = error.message.to_lowercase();
    [
        "would be overwritten",
        "local changes",
        "prevent checkout",
        "prevents checkout",
        "uncommitted changes",
        "working tree is not clean",
        "unstaged changes",
        "cannot pull with rebase",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

/// What `git stash` would take (repo-relative, `/`-separated), `None` when
/// there is nothing or the tree has unmerged paths (which cannot be
/// stashed at all). The journal's capture of the stash push reuses this
/// list instead of scanning the tree a second time (`paths_hint`).
fn stashable_paths(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    include_untracked: bool,
) -> Option<Vec<String>> {
    let tree = backend.status(repo).ok()?;
    if tree.files.iter().any(|f| {
        f.index == Some(FileStatus::Conflicted) || f.worktree == Some(FileStatus::Conflicted)
    }) {
        return None;
    }
    let paths: Vec<String> = tree
        .files
        .iter()
        .filter(|f| {
            let untracked = f.worktree == Some(FileStatus::Untracked);
            if untracked {
                include_untracked
            } else {
                f.index.is_some() || f.worktree.is_some()
            }
        })
        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
        .collect();
    (!paths.is_empty()).then_some(paths)
}

/// Called with the result of the operation's first attempt. `None`: leave
/// `first` as it is. `Some`: the changes are stashed; run the operation
/// again and hand its result to [`finish`].
pub fn after_first(
    backend: &dyn GitBackend,
    request: &Request,
    first: &mut Dispatched,
) -> Option<Armed> {
    let operation = operation_word(request)?;
    if !blocked_by_local_changes(first) {
        return None;
    }
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
    let general = settings.as_ref().map(|s| s.config().general.clone());
    let mode = requested_mode(request)
        .or_else(|| general.as_ref().map(|g| g.auto_stash))
        .unwrap_or_default();
    let include_untracked = general
        .as_ref()
        .is_some_and(|g| g.include_untracked_in_autostash);
    if mode == AutoStashMode::Never {
        return None;
    }
    let repo = crate::dispatch::repo_ref(request).ok()?;
    let paths = stashable_paths(backend, &repo, include_untracked)?;
    if mode == AutoStashMode::Ask {
        let error = gitpoulpi_core::Error::NeedsAutoStash {
            operation: operation.to_string(),
        };
        *first = err_kind(request.id, error.to_string(), false, error.typed());
        return None;
    }

    let stash = crate::dispatch::dispatch(
        backend,
        &Request {
            command: OperationKind::StashPush,
            args: json!({
                "message": format!("GitPoulpi auto-stash before {operation}"),
                "include_untracked": include_untracked,
                "paths_hint": paths,
            }),
            ..request.clone()
        },
    );
    let id: StashId = serde_json::from_value(stash.response.result.ok()?).ok()?;

    let commit = crate::journal_snap::stashes(backend, &repo)
        .ok()
        .and_then(|list| list.into_iter().find(|s| s.id == id))
        .and_then(|s| s.commit);
    let state_path = state_path(&repo);
    write_pending(
        &state_path,
        Some(PendingAutoStash {
            stash_id: id,
            stash_commit: commit,
            reason: request.command,
            created_at: gitpoulpi_core::time::unix_now(),
            state: PendingAutoStashState::Applied,
            error: None,
            after_operation: false,
        }),
    );
    Some(Armed { id, state_path })
}

/// After a stash pop, apply or drop (the banner's "Pop now" / "Drop", the
/// keyboard, or `gitpoulpi stash ...`): reconciles the record - cleared
/// when the stash it pointed at is gone, its index following any shift.
/// Either way clients are told (`pending_auto_stash_changed`), so the
/// banner goes live. The user did this themselves, so no "gone" notice.
pub fn settle_manual(backend: &dyn GitBackend, request: &Request, dispatched: &mut Dispatched) {
    if !matches!(
        request.command,
        OperationKind::StashPop
            | OperationKind::StashApply
            | OperationKind::StashDrop
            // A new stash shifts the pending one's index.
            | OperationKind::StashPush
            | OperationKind::StashFromCommit
    ) {
        return;
    }
    let Ok(repo) = crate::dispatch::repo_ref(request) else {
        return;
    };
    let mut changed = reconcile(backend, &repo) != Reconciled::Untouched;
    if request.command == OperationKind::StashPop {
        if let Err(e) = &dispatched.response.result {
            changed |= record_failed_pop(backend, &repo, request, &e.message);
        }
    }
    if changed {
        dispatched.pending_auto_stash_changed = true;
    }
}

/// Whether the working tree has unmerged paths.
fn has_unmerged(backend: &dyn GitBackend, repo: &RepoRef) -> bool {
    backend.status(repo).is_ok_and(|tree| {
        tree.files.iter().any(|f| {
            f.index == Some(FileStatus::Conflicted) || f.worktree == Some(FileStatus::Conflicted)
        })
    })
}

/// The state a failed pop leaves: unmerged paths mean the changes are in
/// the tree (`Conflicted`), none mean nothing was applied (`PopFailed`).
fn failed_pop_state(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    message: &str,
) -> (PendingAutoStashState, Option<String>) {
    if has_unmerged(backend, repo) {
        (PendingAutoStashState::Conflicted, None)
    } else {
        (PendingAutoStashState::PopFailed, Some(message.to_string()))
    }
}

/// A user's own pop of the pending stash failed: record what it left.
/// Returns whether the record changed.
fn record_failed_pop(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
    message: &str,
) -> bool {
    let path = state_path(repo);
    let mut state = RepoState::load(&path).unwrap_or_default();
    let Some(pending) = state.pending_auto_stash.as_mut() else {
        return false;
    };
    let popped = request.args.get("id").and_then(Value::as_u64);
    if popped != Some(u64::from(pending.stash_id.0)) {
        return false;
    }
    let (next, error) = failed_pop_state(backend, repo, message);
    // Conflicted stays Conflicted: Keep result is still the way out.
    let next = if pending.state == PendingAutoStashState::Conflicted {
        PendingAutoStashState::Conflicted
    } else {
        next
    };
    let error = if next == PendingAutoStashState::PopFailed {
        error
    } else {
        None
    };
    if pending.state == next && pending.error == error {
        return false;
    }
    pending.state = next;
    pending.error = error;
    if let Err(e) = state.save(&path) {
        tracing::warn!("could not record the failed pop: {e}");
    }
    true
}

pub(crate) fn write_pending(path: &std::path::Path, pending: Option<PendingAutoStash>) {
    let mut state = RepoState::load(path).unwrap_or_default();
    state.pending_auto_stash = pending;
    if let Err(e) = state.save(path) {
        tracing::warn!("could not record the pending auto-stash: {e}");
    }
}

fn pop(backend: &dyn GitBackend, request: &Request, armed: &Armed) -> Dispatched {
    crate::dispatch::dispatch(
        backend,
        &Request {
            command: OperationKind::StashPop,
            args: json!({ "id": armed.id }),
            ..request.clone()
        },
    )
}

/// Called with the result of the second attempt: pops the stash (unless
/// the operation stopped half-way, or failed and the changes just go
/// back) and answers the client.
pub fn finish(
    backend: &dyn GitBackend,
    request: &Request,
    armed: Armed,
    mut second: Dispatched,
) -> Dispatched {
    let in_progress = crate::dispatch::repo_ref(request)
        .ok()
        .and_then(|repo| crate::journal::git_dir(&repo.root))
        .is_some_and(|dir| crate::journal::in_progress(&dir).is_some());
    // `after_first` wrote the pending record: whatever happens below, the
    // state a client reads differs from before the request.
    second.pending_auto_stash_changed = true;
    if second.response.result.is_ok() && in_progress {
        // A conflicted merge or rebase: popping onto it would pile a
        // second conflict on the first. The stash stays, recorded as
        // pending, until the operation is finished: `pop_when_ended` pops
        // it then, however the operation ends.
        let mut state = RepoState::load(&armed.state_path).unwrap_or_default();
        if let Some(pending) = state.pending_auto_stash.as_mut() {
            pending.after_operation = true;
            let _ = state.save(&armed.state_path);
        }
        return second;
    }
    let popped = pop(backend, request, &armed);
    if second.response.result.is_err() {
        // Still refused: the changes are back, nothing else happened.
        if popped.response.result.is_ok() {
            write_pending(&armed.state_path, None);
        }
        return second;
    }
    match popped.response.result {
        Ok(_) => {
            write_pending(&armed.state_path, None);
            second.journal = popped.journal.or(second.journal);
            second.mutated = true;
            second
        }
        Err(e) => {
            let mut state = RepoState::load(&armed.state_path).unwrap_or_default();
            if let Some(pending) = state.pending_auto_stash.as_mut() {
                if let Ok(repo) = crate::dispatch::repo_ref(request) {
                    let (next, error) = failed_pop_state(backend, &repo, &e.message);
                    pending.state = next;
                    pending.error = error;
                }
            }
            let _ = state.save(&armed.state_path);
            let mut failed = err_kind(
                request.id,
                format!(
                    "the operation succeeded, but putting your stashed changes back failed ({}); \
                     they are still in the stash list (stash@{{{}}})",
                    e.message, armed.id.0
                ),
                false,
                None,
            );
            failed.mutated = true;
            failed.journal = popped.journal;
            failed.pending_auto_stash_changed = true;
            failed
        }
    }
}

/// After a request that may have ended the merge or rebase an auto-stash
/// is waiting for (continue, skip, abort, the concluding commit): pops it.
/// See [`pop_when_ended`].
pub fn settle_ended(backend: &dyn GitBackend, request: &Request, dispatched: &mut Dispatched) {
    if !dispatched.mutated {
        return;
    }
    if let Some(popped) = pop_when_ended(backend, request) {
        dispatched.mutated = true;
        dispatched.pending_auto_stash_changed = true;
        dispatched.journal = popped.journal.or(dispatched.journal.take());
    }
}

/// Pops the pending stash when the operation it waited for (the record's
/// `after_operation`) has ended - in the app or with plain git - and the
/// tree has no unmerged path left. A pop that fails leaves the record
/// `Conflicted` / `PopFailed` like any other (`settle_manual`), never
/// retried by itself. Returns the pop's answer when one was attempted.
fn pop_when_ended(backend: &dyn GitBackend, request: &Request) -> Option<Dispatched> {
    if matches!(
        request.command,
        OperationKind::StashPop
            | OperationKind::StashApply
            | OperationKind::StashDrop
            | OperationKind::StashPush
            | OperationKind::StashFromCommit
            | OperationKind::AutoStashStartOver
    ) {
        return None;
    }
    let repo = crate::dispatch::repo_ref(request).ok()?;
    let pending = pending_record(&repo)?;
    if !pending.after_operation || pending.state != PendingAutoStashState::Applied {
        return None;
    }
    let git_dir = crate::journal::git_dir(&repo.root)?;
    if crate::journal::in_progress(&git_dir).is_some() || has_unmerged(backend, &repo) {
        return None;
    }
    let armed = Armed {
        id: pending.stash_id,
        state_path: state_path(&repo),
    };
    Some(pop(backend, request, &armed))
}
