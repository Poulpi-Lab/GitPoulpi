//! Repository discovery: "is this path in a git repository, and where is
//! its root?"
//!
//! One `git rev-parse` invocation answers all of it, because each spawn is
//! a process and the *open repository* flow runs this per typed path.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{DiscoveredRepo, InitOptions, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::run;

/// The directory `git` should be run in for `path`.
///
/// `git -C <file>` fails; `git -C <missing dir>` fails too. A path the
/// user typed can be either, and the fix for the first (use the parent)
/// must not be applied to the second, or a nonexistent path inside a
/// repository would silently discover that repository.
fn start_dir(path: &Path) -> Result<PathBuf> {
    let meta = std::fs::metadata(path).map_err(|_| Error::RepoNotFound(path.to_path_buf()))?;
    if meta.is_dir() {
        Ok(path.to_path_buf())
    } else {
        // A file inside a repository discovers the repository containing
        // it; a file that is nowhere near one still fails below.
        Ok(path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf()))
    }
}

pub fn discover(path: &Path) -> Result<DiscoveredRepo> {
    let start = start_dir(path)?;

    // `--show-toplevel` is empty for a bare repository, so the bare case is
    // answered by `--is-bare-repository` and the root taken from
    // `--absolute-git-dir`. One command keeps the answers from different
    // repositories if the tree changes underneath us.
    let raw = run(
        &start,
        &[
            "rev-parse",
            "--is-bare-repository",
            "--is-inside-work-tree",
            "--absolute-git-dir",
            "--show-toplevel",
        ],
    )
    .map_err(|_| Error::RepoNotFound(path.to_path_buf()))?;

    let mut lines = raw.lines();
    let is_bare = lines.next().unwrap_or("false").trim() == "true";
    let _is_inside_work_tree = lines.next().unwrap_or("false").trim() == "true";
    let git_dir = PathBuf::from(lines.next().unwrap_or("").trim());
    let toplevel = lines.next().unwrap_or("").trim().to_string();

    let root = if is_bare || toplevel.is_empty() {
        git_dir.clone()
    } else {
        PathBuf::from(toplevel)
    };
    if root.as_os_str().is_empty() {
        return Err(Error::RepoNotFound(path.to_path_buf()));
    }

    // A linked worktree's git dir is `<main>/.git/worktrees/<name>`,
    // never the repository's own `.git`. `--git-common-dir` names the
    // shared one, so the two differing *is* the definition of "this is a
    // linked worktree" -- more reliable than testing whether `.git` is a
    // file, which is also true of a submodule.
    let common = run(&root, &["rev-parse", "--git-common-dir"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| absolutize(&root, Path::new(&s)));
    let is_worktree = match common {
        Some(common) => !same_path(&common, &git_dir),
        None => false,
    };

    // `--abbrev-ref HEAD` prints the literal `HEAD` on a detached head,
    // and fails outright in a repository with no commits. Both mean "no
    // branch", and both have to come out as `None` rather than as a
    // branch name nothing can check out.
    let head_branch = run(&root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "HEAD");

    Ok(DiscoveredRepo {
        repo: RepoRef::from_root(&root),
        name: DiscoveredRepo::name_for_root(&root),
        is_bare,
        is_worktree,
        head_branch,
    })
}

/// git may answer `--git-common-dir` relatively (`.git`), so it is
/// resolved against the directory the question was asked in before being
/// compared with the absolute `--absolute-git-dir`.
fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// Compares two paths the way the filesystem would.
///
/// Canonicalisation matters on Windows in both directions: `C:\Users\…`
/// vs `\\?\C:\Users\…`, and a temp dir reached through a short (8.3) name
/// vs its long form. The raw-comparison fallback keeps a deleted path from
/// being reported as "not a worktree" because canonicalize failed.
fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// `GitBackend::init` over `git init`.
///
/// Beyond spawning git it creates the directory chain, refuses a path that
/// is already a repository root, and reports the root by re-`discover`ing
/// the result, so the `RepoRef` the daemon keys on equals the one a later
/// `discover` of the same directory produces.
pub fn init(path: &Path, opts: &InitOptions) -> Result<RepoRef> {
    if already_a_repository_root(path) {
        return Err(Error::Backend(format!(
            "{} is already a git repository",
            path.display()
        )));
    }
    // `git init <path>` would create it too, but the cwd the child is
    // spawned in must exist regardless, and `process::git` spawns in the
    // path it is given.
    std::fs::create_dir_all(path)
        .map_err(|e| Error::Backend(format!("could not create {}: {e}", path.display())))?;

    let mut args: Vec<String> = vec!["init".to_string()];
    if opts.bare {
        args.push("--bare".to_string());
    }
    if let Some(branch) = &opts.initial_branch {
        args.push("--initial-branch".to_string());
        args.push(branch.clone());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run(path, &args_ref)?;

    Ok(discover(path)?.repo)
}

/// Whether `path` *is* a repository root, as opposed to being inside one:
/// re-initializing a repository is refused, creating a nested one is not.
///
/// Duplicated in the git2 backend rather than hoisted into
/// `gitpoulpi-core`, which must not know how either backend discovers
/// anything.
fn already_a_repository_root(path: &Path) -> bool {
    match discover(path) {
        Ok(found) => found.repo.root == RepoRef::from_root(path).root,
        Err(_) => false,
    }
}
