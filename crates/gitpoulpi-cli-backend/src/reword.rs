//! `GitBackend::reword` with plumbing: `git commit-tree` re-creates the
//! commit with its new message, then every commit between it and HEAD on
//! top of it, and one `git update-ref` transaction moves HEAD.
//!
//! Deliberately not `git rebase -i` with a scripted editor: a rebase
//! checks commits out one by one, so it needs a clean tree, can stop
//! halfway, flattens merges unless asked not to, and runs hooks. Only
//! messages change here, so every tree is reused as is -- nothing can
//! conflict and the index and working tree are never touched. This is
//! the same algorithm `Git2Backend` runs over libgit2; the contract suite
//! pins that both produce the same history. The shared steps live in
//! `rewrite.rs`.

use std::collections::HashMap;

use gitpoulpi_core::domain::{CommitAmend, Oid, RepoRef, RewordOutcome, RewordRefusal};
use gitpoulpi_core::{Error, Result};

use crate::rewrite::{
    descendants, move_head, operation_in_progress, probe, read_commits, recreate_as, replay,
    CommitterOverride,
};

pub fn reword(repo: &RepoRef, commit: Oid, message: &str) -> Result<RewordOutcome> {
    amend_commit(repo, commit, &CommitAmend::message_only(message), "reword")
}

/// `verb` heads the reflog entry (`reword` or `amend`).
pub fn amend_commit(
    repo: &RepoRef,
    commit: Oid,
    amend: &CommitAmend,
    verb: &str,
) -> Result<RewordOutcome> {
    let (git_dir, head) = probe(&repo.root)?;
    if operation_in_progress(&git_dir) {
        return Err(Error::RewordRefused(RewordRefusal::OperationInProgress));
    }
    let resolved = amend.resolve().map_err(Error::RewordRefused)?;

    let target = commit.to_hex();
    let to_replay = descendants(&repo.root, commit, head)?
        .ok_or(Error::RewordRefused(RewordRefusal::NotOnCurrentBranch))?;

    let mut wanted = vec![target.clone()];
    wanted.extend(to_replay.iter().cloned());
    let objects = read_commits(&repo.root, &wanted)?;

    let mut rewritten: HashMap<String, String> = HashMap::new();
    let mut amended = objects[0].clone_with_author(
        resolved.author_name.as_deref(),
        resolved.author_email.as_deref(),
        resolved.author_date.map(|d| d.raw()).as_deref(),
    );
    let message = resolved
        .message
        .clone()
        .unwrap_or_else(|| std::mem::take(&mut amended.message));
    let committer = CommitterOverride {
        name: resolved.committer_name.clone(),
        email: resolved.committer_email.clone(),
        date: resolved.committer_date.map(|d| d.raw()),
    };
    let new_target = recreate_as(
        &repo.root, &amended, &message, &rewritten, false, &committer,
    )?;
    rewritten.insert(target.clone(), new_target.clone());
    replay(&repo.root, &to_replay, &objects[1..], &mut rewritten, false)?;
    let head_hex = head.to_hex();
    let new_head = rewritten[&head_hex].clone();

    let log_message = format!(
        "{verb}: {} {}",
        &target[..7],
        message.lines().next().unwrap_or_default()
    );
    move_head(&repo.root, &new_head, &head_hex, &log_message)?;

    let to_oid =
        |hex: &str| Oid::from_hex(hex).ok_or_else(|| Error::Backend(format!("not an oid: {hex}")));
    Ok(RewordOutcome {
        reworded: to_oid(&new_target)?,
        previous_head: head,
        new_head: to_oid(&new_head)?,
        replayed: to_replay.len(),
    })
}
