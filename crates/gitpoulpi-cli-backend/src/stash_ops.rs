use gitpoulpi_core::domain::{Oid, RepoRef, StashId};
use gitpoulpi_core::Result;

use crate::process::{run, run_in_english, run_with_env_and_stdin, run_with_stdin};

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

/// A stash commit holding what `commit` changed: its worktree commit has
/// `commit`'s tree and the parents (base, index) - the base is the first
/// parent of `commit`, or an empty commit for a root one, so applying the
/// stash replays exactly that change. Then `stash_store` puts it on top.
pub fn stash_from_commit(repo: &RepoRef, commit: Oid, message: Option<&str>) -> Result<Oid> {
    let hex = commit.to_hex();
    let known = |_| gitpoulpi_core::Error::Backend(format!("{hex} is not a known commit"));
    let tree = run(
        &repo.root,
        &["rev-parse", "--verify", &format!("{hex}^{{tree}}")],
    )
    .map_err(known)?
    .trim()
    .to_string();
    let subject = run(&repo.root, &["log", "-1", "--format=%s", &hex])?
        .trim()
        .to_string();
    let short = &hex[..hex.len().min(8)];
    let message = match message {
        Some(m) if !m.trim().is_empty() => m.trim().to_string(),
        _ => format!("restored changes of {short} {subject}")
            .trim_end()
            .to_string(),
    };

    let parents = run(&repo.root, &["rev-list", "--parents", "-n", "1", &hex])?;
    let base = match parents.split_whitespace().nth(1) {
        Some(p) => p.to_string(),
        None => {
            let empty = run_with_stdin(
                &repo.root,
                &["hash-object", "-w", "-t", "tree", "--stdin"],
                "",
            )?;
            commit_tree(repo, empty.trim(), &[], "root of the restored changes")?
        }
    };
    let index = commit_tree(
        repo,
        &tree,
        &[&base],
        &format!("index on (restored): {short} {subject}"),
    )?;
    let work = commit_tree(
        repo,
        &tree,
        &[&base, &index],
        &format!("On (restored): {message}"),
    )?;
    let work_oid = Oid::from_hex(&work)
        .ok_or_else(|| gitpoulpi_core::Error::Backend(format!("odd commit id {work}")))?;
    stash_store(repo, work_oid, &message)?;
    Ok(work_oid)
}

fn commit_tree(repo: &RepoRef, tree: &str, parents: &[&str], message: &str) -> Result<String> {
    let mut args = vec!["commit-tree", "--no-gpg-sign", tree];
    for p in parents {
        args.push("-p");
        args.push(p);
    }
    let out = run_with_env_and_stdin(&repo.root, &args, &[], message)?;
    Ok(out.trim().to_string())
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
