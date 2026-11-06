//! `GitBackend::reword` over libgit2: re-create the commit with a new
//! message, then re-create every commit between it and HEAD on top of it.
//!
//! Only messages change, so every tree is reused as is: nothing can
//! conflict, and neither the index nor the working tree is touched. The
//! system-git backend (`gitpoulpi_cli_backend::reword`) does the same
//! with `git commit-tree`; the contract suite pins that the two agree.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{
    CommitAmend, GitDate, Oid, RepoRef, ResolvedAmend, RewordOutcome, RewordRefusal,
};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

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
    let git_repo = open(repo)?;
    // Same order of checks as the system-git backend, so a request that
    // is wrong in two ways gets the same answer from both.
    if git_repo.state() != git2::RepositoryState::Clean {
        return Err(Error::RewordRefused(RewordRefusal::OperationInProgress));
    }
    let resolved = amend.resolve().map_err(Error::RewordRefused)?;

    let head_ref = git_repo.head().map_err(map_err)?;
    let head = head_ref.peel_to_commit().map_err(map_err)?.id();
    let target = to_git2_oid(commit)?;
    let target_commit = git_repo.find_commit(target).map_err(map_err)?;

    let to_replay = if target == head {
        Vec::new()
    } else {
        if !git_repo
            .graph_descendant_of(head, target)
            .map_err(map_err)?
        {
            return Err(Error::RewordRefused(RewordRefusal::NotOnCurrentBranch));
        }
        descendants_up_to(&git_repo, target, head)?
    };

    let committer = git_repo.signature().map_err(map_err)?;
    let mut rewritten: HashMap<git2::Oid, git2::Oid> = HashMap::new();
    let (author, target_committer, message) = amended_parts(&target_commit, &committer, &resolved)?;
    let new_target = recreate(
        &git_repo,
        &target_commit,
        &message,
        &author,
        &target_committer,
        &rewritten,
    )?;
    rewritten.insert(target, new_target);
    for old in &to_replay {
        let old_commit = git_repo.find_commit(*old).map_err(map_err)?;
        let old_message = old_commit.message_raw().ok().ok_or_else(|| {
            Error::Backend(format!("commit {old} has a message that is not UTF-8"))
        })?;
        let new = recreate(
            &git_repo,
            &old_commit,
            old_message,
            &old_commit.author(),
            &committer,
            &rewritten,
        )?;
        rewritten.insert(*old, new);
    }
    let new_head = rewritten[&head];

    let log_message = format!(
        "{verb}: {} {}",
        &target.to_string()[..7],
        message.lines().next().unwrap_or_default()
    );
    if head_ref.is_branch() {
        let name = head_ref
            .name()
            .ok()
            .ok_or_else(|| Error::Backend("the checked-out branch has no name".into()))?;
        git_repo
            .reference_matching(name, new_head, true, head, &log_message)
            .map_err(map_err)?;
    } else {
        git_repo.set_head_detached(new_head).map_err(map_err)?;
    }
    git_repo
        .reference("ORIG_HEAD", head, true, &log_message)
        .map_err(map_err)?;

    Ok(RewordOutcome {
        reworded: to_domain_oid(new_target)?,
        previous_head: to_domain_oid(head)?,
        new_head: to_domain_oid(new_head)?,
        replayed: to_replay.len(),
    })
}

/// The commits that descend from `target` and are ancestors of (or equal
/// to) `head`, parents before children - `git rev-list --ancestry-path
/// --topo-order --reverse target..head`.
fn descendants_up_to(
    git_repo: &git2::Repository,
    target: git2::Oid,
    head: git2::Oid,
) -> Result<Vec<git2::Oid>> {
    let mut walk = git_repo.revwalk().map_err(map_err)?;
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)
        .map_err(map_err)?;
    walk.push(head).map_err(map_err)?;
    walk.hide(target).map_err(map_err)?;
    let mut on_path: HashSet<git2::Oid> = HashSet::new();
    let mut ordered = Vec::new();
    for oid in walk {
        let oid = oid.map_err(map_err)?;
        let commit = git_repo.find_commit(oid).map_err(map_err)?;
        if commit
            .parent_ids()
            .any(|p| p == target || on_path.contains(&p))
        {
            on_path.insert(oid);
            ordered.push(oid);
        }
    }
    Ok(ordered)
}

/// `old` again, with `message`, `author`, `committer`, and each parent
/// replaced by its rewritten version when it has one.
fn recreate(
    git_repo: &git2::Repository,
    old: &git2::Commit<'_>,
    message: &str,
    author: &git2::Signature<'_>,
    committer: &git2::Signature<'_>,
    rewritten: &HashMap<git2::Oid, git2::Oid>,
) -> Result<git2::Oid> {
    let parents = old
        .parent_ids()
        .map(|p| git_repo.find_commit(*rewritten.get(&p).unwrap_or(&p)))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(map_err)?;
    let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
    let tree = old.tree().map_err(map_err)?;
    git_repo
        .commit(None, author, committer, message, &tree, &parent_refs)
        .map_err(map_err)
}

/// The author, committer and message the amended commit gets: its own,
/// with whatever `resolved` replaces; the committer is `now` unless a
/// committer field says otherwise.
fn amended_parts(
    old: &git2::Commit<'_>,
    now: &git2::Signature<'static>,
    resolved: &ResolvedAmend,
) -> Result<(git2::Signature<'static>, git2::Signature<'static>, String)> {
    let old_author = old.author();
    let author = signature(
        resolved
            .author_name
            .as_deref()
            .unwrap_or_else(|| old_author.name().unwrap_or_default()),
        resolved
            .author_email
            .as_deref()
            .unwrap_or_else(|| old_author.email().unwrap_or_default()),
        resolved.author_date.map_or(old_author.when(), time),
    )?;
    let committer = signature(
        resolved
            .committer_name
            .as_deref()
            .unwrap_or_else(|| now.name().unwrap_or_default()),
        resolved
            .committer_email
            .as_deref()
            .unwrap_or_else(|| now.email().unwrap_or_default()),
        resolved.committer_date.map_or(now.when(), time),
    )?;
    let message = match &resolved.message {
        Some(m) => m.clone(),
        None => old
            .message_raw()
            .ok()
            .ok_or_else(|| {
                Error::Backend(format!(
                    "commit {} has a message that is not UTF-8",
                    old.id()
                ))
            })?
            .to_string(),
    };
    Ok((author, committer, message))
}

fn time(date: GitDate) -> git2::Time {
    git2::Time::new(date.seconds, date.offset_minutes)
}

fn signature(name: &str, email: &str, when: git2::Time) -> Result<git2::Signature<'static>> {
    git2::Signature::new(name, email, &when).map_err(map_err)
}
