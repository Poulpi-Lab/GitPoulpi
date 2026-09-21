use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{RepoRef, Worktree, WorktreeAddOptions, WorktreeStart};
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
