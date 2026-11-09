//! `operation_state` and `operation_abort`: which merge, cherry-pick,
//! revert or rebase is stopped, and abandoning it.

use gitpoulpi_core::domain::{ConflictRefusal, Oid, OperationState, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::conflicts::{git_dir, read_trimmed, short, side_labels};
use crate::process::{abort_failed, run, run_abort};

/// See `GitBackend::operation_state`. Rebase wins over the others (a
/// rebase replays commits through cherry-pick machinery and can leave
/// `CHERRY_PICK_HEAD`).
pub fn operation_state(repo: &RepoRef) -> Result<Option<OperationState>> {
    if let Some(am) = crate::stuck::am_state(repo) {
        return Ok(Some(am));
    }
    if let Some(state) = crate::rebase_interactive::rebase_state(repo)? {
        return Ok(Some(OperationState::Rebase(state)));
    }
    let dir = git_dir(&repo.root);
    let commit_in = |marker: &str| {
        read_trimmed(&dir.join(marker))
            .and_then(|text| Oid::from_hex(text.lines().next().unwrap_or_default().trim()))
    };
    if let Some(commit) = commit_in("CHERRY_PICK_HEAD") {
        return Ok(Some(OperationState::CherryPick { commit }));
    }
    if let Some(commit) = commit_in("REVERT_HEAD") {
        return Ok(Some(OperationState::Revert { commit }));
    }
    if let Some(sha) = read_trimmed(&dir.join("MERGE_HEAD")) {
        let head = read_trimmed(&dir.join("HEAD")).unwrap_or_default();
        let head_name = head
            .strip_prefix("ref: ")
            .map(|r| r.strip_prefix("refs/heads/").unwrap_or(r).to_string());
        let their_label = side_labels(repo).git_theirs_label;
        let their_label = if their_label.is_empty() {
            short(sha.lines().next().unwrap_or_default()).to_string()
        } else {
            their_label
        };
        return Ok(Some(OperationState::Merge {
            head_name,
            their_label,
        }));
    }
    Ok(None)
}

/// See `GitBackend::operation_skip`.
pub fn operation_skip(repo: &RepoRef) -> Result<()> {
    let Some(state) = operation_state(repo)? else {
        return Err(Error::ConflictRefused(ConflictRefusal::NoOperation));
    };
    match state {
        OperationState::Rebase(_) => crate::rebase_interactive::rebase_skip(repo).map(|_| ()),
        OperationState::Merge { .. } => Err(Error::Backend(
            "a merge has nothing to skip: commit it or abort it".into(),
        )),
        OperationState::CherryPick { .. } => {
            run(&repo.root, &["cherry-pick", "--skip"]).map(|_| ())
        }
        OperationState::Revert { .. } => run(&repo.root, &["revert", "--skip"]).map(|_| ()),
        OperationState::Am { .. } => crate::stuck::am_skip(repo).map(|_| ()),
    }
}

/// See `GitBackend::operation_abort`.
pub fn operation_abort(repo: &RepoRef) -> Result<()> {
    let Some(state) = operation_state(repo)? else {
        return Err(Error::ConflictRefused(ConflictRefusal::NoOperation));
    };
    match state {
        OperationState::Rebase(_) => crate::rebase_interactive::rebase_abort(repo),
        OperationState::Merge { .. } => abort_with(repo, &["merge", "--abort"]),
        OperationState::CherryPick { .. } => abort_with(repo, &["cherry-pick", "--abort"]),
        OperationState::Revert { .. } => abort_with(repo, &["revert", "--abort"]),
        OperationState::Am { .. } => crate::stuck::am_abort(repo),
    }
}

/// `git <args>` as an abort: a stale index is refreshed and the abort retried
/// once; files that really have local changes the abort would overwrite are
/// a typed [`ConflictRefusal::LocalChanges`].
pub(crate) fn abort_with(repo: &RepoRef, args: &[&str]) -> Result<()> {
    let out = run_abort(&repo.root, args)?;
    if out.success {
        Ok(())
    } else {
        Err(abort_failed(args, &out))
    }
}
