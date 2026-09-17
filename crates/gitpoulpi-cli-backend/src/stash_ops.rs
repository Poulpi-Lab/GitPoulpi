use gitpoulpi_core::domain::{Oid, RepoRef, StashId};
use gitpoulpi_core::Result;

use crate::process::{run, run_in_english};

/// A freshly pushed stash is always `stash@{0}` - git's stash is a stack
/// and push always inserts at the top. `None`: git said "No local changes to
/// save" and made no stash.
pub fn stash_push(
    repo: &RepoRef,
    message: Option<&str>,
    include_untracked: bool,
) -> Result<Option<StashId>> {
    let mut args = vec!["stash".to_string(), "push".to_string()];
    if include_untracked {
        args.push("--include-untracked".to_string());
    }
    if let Some(message) = message {
        args.push("-m".to_string());
        args.push(message.to_string());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    pushed(&run_in_english(&repo.root, &args_ref)?)
}

/// `git stash push [-m msg] --include-untracked -- <paths>`: only the named
/// paths are stashed. Always `stash@{0}` afterwards, like `stash_push`.
pub fn stash_push_paths(
    repo: &RepoRef,
    message: Option<&str>,
    paths: &[String],
) -> Result<Option<StashId>> {
    let mut args = vec![
        "stash".to_string(),
        "push".to_string(),
        "--include-untracked".to_string(),
    ];
    if let Some(message) = message {
        args.push("-m".to_string());
        args.push(message.to_string());
    }
    if !paths.is_empty() {
        args.push("--".to_string());
        args.extend(paths.iter().cloned());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    pushed(&run_in_english(&repo.root, &args_ref)?)
}

fn pushed(stdout: &str) -> Result<Option<StashId>> {
    Ok(if stdout.contains("No local changes to save") {
        None
    } else {
        Some(StashId(0))
    })
}

pub fn stash_pop(repo: &RepoRef, id: StashId) -> Result<()> {
    run(&repo.root, &["stash", "pop", &stash_ref(id)])?;
    Ok(())
}

pub fn stash_apply(repo: &RepoRef, id: StashId) -> Result<()> {
    run(&repo.root, &["stash", "apply", &stash_ref(id)])?;
    Ok(())
}

pub fn stash_drop(repo: &RepoRef, id: StashId) -> Result<()> {
    run(&repo.root, &["stash", "drop", &stash_ref(id)])?;
    Ok(())
}

/// `git stash store -m <message> <commit>`: the commit goes back on top of
/// the stash list. Only a stash commit (a merge: its second parent is the
/// index) is accepted; newer git checks that itself, older git stores any
/// commit, so the check is made here for both.
pub fn stash_store(repo: &RepoRef, commit: Oid, message: &str) -> Result<()> {
    run(
        &repo.root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{}^2", commit.to_hex()),
        ],
    )
    .map_err(|_| {
        gitpoulpi_core::Error::Backend(format!("{} is not a stash commit", commit.to_hex()))
    })?;
    run(
        &repo.root,
        &["stash", "store", "-m", message, &commit.to_hex()],
    )?;
    Ok(())
}

/// The paths a stash commit holds changes for, untracked files included.
pub fn stash_paths(repo: &RepoRef, commit: Oid) -> Result<Vec<String>> {
    let raw = run(
        &repo.root,
        &[
            "stash",
            "show",
            "--name-only",
            "--include-untracked",
            "-z",
            &commit.to_hex(),
        ],
    )?;
    let mut paths: Vec<String> = raw
        .split('\0')
        .map(|p| p.trim_matches('\n'))
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn stash_ref(id: StashId) -> String {
    format!("stash@{{{}}}", id.0)
}
