use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    RepoRef, Worktree, WorktreeAddOptions, WorktreeChange, WorktreeRemoveOptions, WorktreeRemoved,
    WorktreeStart,
};
use gitpoulpi_core::{Error, Result};

use crate::process::run;

pub use crate::worktree_list::worktrees;

/// `git worktree add`, in the one of its four shapes `opts` asks for:
///
/// | start          | new branch | command                               |
/// |----------------|------------|---------------------------------------|
/// | `Branch(b)`    | none       | `worktree add <path> <b>`             |
/// | `Branch(b)`    | `n`        | `worktree add -b <n> <path> refs/heads/<b>` |
/// | `Commit(c)`    | none       | `worktree add --detach <path> <c>`    |
/// | `Commit(c)`    | `n`        | `worktree add -b <n> <path> <c>`      |
///
/// `Branch(b)` is checked to be a local branch first: handed a name that
/// is not one, git would guess - create a branch tracking `origin/<b>`,
/// or detach at a tag of that name - and the caller asked for neither.
/// git itself refuses a non-empty folder, a taken `-b` name and a branch
/// checked out elsewhere, and creates missing parent folders.
///
/// The entry returned is the one `git worktree list` then shows for the
/// path, so it is spelled exactly as [`worktrees`] spells it.
pub fn worktree_add(repo: &RepoRef, path: &Path, opts: &WorktreeAddOptions) -> Result<Worktree> {
    let path_arg = path.to_string_lossy().to_string();
    let start = match &opts.start {
        WorktreeStart::Branch(branch) => {
            let full = format!("refs/heads/{branch}");
            run(&repo.root, &["show-ref", "--verify", "--quiet", &full])
                .map_err(|_| Error::Backend(format!("there is no local branch {branch}")))?;
            if opts.new_branch.is_some() {
                full
            } else {
                branch.clone()
            }
        }
        WorktreeStart::Commit(oid) => oid.to_hex(),
    };
    let mut args: Vec<&str> = vec!["worktree", "add"];
    match (&opts.new_branch, &opts.start) {
        (Some(name), _) => args.extend(["-b", name.as_str()]),
        (None, WorktreeStart::Commit(_)) => args.push("--detach"),
        (None, WorktreeStart::Branch(_)) => {}
    }
    args.extend([path_arg.as_str(), start.as_str()]);
    run(&repo.root, &args)?;

    let wanted = canonical(path);
    worktrees(repo)?
        .into_iter()
        .find(|wt| canonical(&wt.path) == wanted)
        .ok_or_else(|| {
            Error::Backend(format!(
                "git worktree add reported success but {} is not listed",
                path.display()
            ))
        })
}

/// Paths from `git worktree list` (forward slashes on Windows) and from
/// the caller (backslashes, maybe an 8.3 name) only compare once
/// canonicalized.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// `git status --porcelain=v1 -z -uall` of the worktree at `path`: every
/// tracked change and every untracked, not ignored file (one per file,
/// never a collapsed folder).
pub fn worktree_changes(repo: &RepoRef, path: &Path) -> Result<Vec<WorktreeChange>> {
    ensure_listed(repo, path)?;
    let raw = run(
        path,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    let mut out = Vec::new();
    let mut parts = raw.split('\0').filter(|p| !p.is_empty());
    while let Some(entry) = parts.next() {
        if entry.len() < 4 {
            continue;
        }
        let (code, file) = entry.split_at(3);
        // A rename lists its origin as a second field.
        if code.starts_with('R') || code.starts_with('C') || code[1..2] == *"R" {
            parts.next();
        }
        out.push(WorktreeChange {
            path: file.to_string(),
            untracked: code.starts_with("??"),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// The listed worktree for `path`, or an error: the guard of both
/// operations, so a path that is not a worktree of this repository is
/// never handed to git.
fn ensure_listed(repo: &RepoRef, path: &Path) -> Result<Worktree> {
    let wanted = canonical(path);
    worktrees(repo)?
        .into_iter()
        .find(|wt| canonical(&wt.path) == wanted)
        .ok_or_else(|| {
            Error::Backend(format!(
                "{} is not a worktree of this repository",
                path.display()
            ))
        })
}

/// `git worktree remove`, after saving the changes the options ask to save.
///
/// Order: refuse (main, locked, dirty without a way to save them), commit
/// or stash, then remove. A dirty worktree with neither `force`, `commit`
/// nor `stash` is `DirtyWorkingTree` and nothing moves.
pub fn worktree_remove(
    repo: &RepoRef,
    path: &Path,
    opts: &WorktreeRemoveOptions,
) -> Result<WorktreeRemoved> {
    let wt = ensure_listed(repo, path)?;
    if wt.is_main {
        return Err(Error::Backend(
            "the main worktree cannot be removed".to_string(),
        ));
    }
    if wt.is_locked {
        return Err(Error::Backend(format!(
            "{} is locked; unlock it first",
            path.display()
        )));
    }
    if opts.commit.is_some() && opts.stash.is_some() {
        return Err(Error::Backend(
            "commit and stash are alternatives, not both".to_string(),
        ));
    }
    let changes = worktree_changes(repo, path)?;
    let dirty = !changes.is_empty();
    if dirty && !opts.force && opts.commit.is_none() && opts.stash.is_none() {
        return Err(Error::DirtyWorkingTree {
            operation: "remove worktree",
        });
    }
    let mut commit = None;
    let mut stashed = false;
    if dirty && !opts.force {
        if let Some(c) = &opts.commit {
            if c.message.trim().is_empty() {
                return Err(Error::Backend("the commit message is empty".to_string()));
            }
            if wt.branch.is_none() {
                let Some(branch) = c.branch.as_deref().filter(|b| !b.trim().is_empty()) else {
                    return Err(Error::Backend(
                        "the worktree is detached: name a branch for the commit".to_string(),
                    ));
                };
                run(path, &["switch", "-c", branch])?;
            }
            run(path, &["add", "--all"])?;
            run(path, &["commit", "-m", c.message.as_str()])?;
            let head = run(path, &["rev-parse", "HEAD"])?;
            commit = gitpoulpi_core::domain::Oid::from_hex(head.trim());
        } else if let Some(message) = &opts.stash {
            run(
                path,
                &[
                    "stash",
                    "push",
                    "--include-untracked",
                    "-m",
                    message.as_str(),
                ],
            )?;
            stashed = true;
        }
    }
    let path_arg = path.to_string_lossy().to_string();
    let mut args = vec!["worktree", "remove"];
    if opts.force {
        args.push("--force");
    }
    args.push(path_arg.as_str());
    run(&repo.root, &args)?;
    Ok(WorktreeRemoved {
        path: path.to_path_buf(),
        commit,
        stashed,
    })
}
