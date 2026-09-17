//! What a canceled (dropped) network command leaves behind, and its removal.
//!
//! git removes its lock files on SIGINT/SIGTERM, but a hard kill (Windows
//! `TerminateProcess`, SIGKILL) skips that. The guards here run when the
//! future awaiting git is dropped before git finished:
//!
//! - [`RepoCleanup`] (fetch, push, LFS transfers): removes the ref locks,
//!   `packed-refs.lock`, `shallow.lock`, half-written `tmp_pack_*` and
//!   `lfs/tmp` files **created after the command started** (mtime >= start),
//!   so a lock an unrelated git holds (older) is never touched. Never
//!   `index.lock`: the existing `IndexLockRemove` offer is the way out of that
//!   one. Residual risk, accepted: another git that started *after* ours on the
//!   same ref and still runs.
//! - [`CloneCleanup`]: deletes the clone destination when the clone created
//!   it, empties it when it existed empty, and never touches a folder that
//!   had content before.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// A removal retried briefly: the killed process may still hold the file for
/// a few milliseconds (Windows).
fn retry(mut attempt: impl FnMut() -> std::io::Result<()>) -> bool {
    for i in 0..40 {
        match attempt() {
            Ok(()) => return true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) => {
                if i == 39 {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    false
}

/// The directory holding `refs/`, `objects/` and the lock files of `root`
/// (`.git`, a worktree's common dir, or `root` itself for a bare repository).
fn common_git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    let mut git_dir = if dot_git.is_dir() {
        dot_git
    } else if dot_git.is_file() {
        let text = std::fs::read_to_string(&dot_git).ok()?;
        let target = text.trim().strip_prefix("gitdir:")?.trim();
        let target = Path::new(target);
        if target.is_absolute() {
            target.to_path_buf()
        } else {
            root.join(target)
        }
    } else if root.join("HEAD").is_file() && root.join("objects").is_dir() {
        root.to_path_buf()
    } else {
        return None;
    };
    if let Ok(common) = std::fs::read_to_string(git_dir.join("commondir")) {
        let common = Path::new(common.trim());
        git_dir = if common.is_absolute() {
            common.to_path_buf()
        } else {
            git_dir.join(common)
        };
    }
    Some(git_dir)
}

fn created_since(path: &Path, started: SystemTime) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .is_ok_and(|m| m >= started)
}

fn walk_locks(dir: &Path, started: SystemTime, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_locks(&path, started, out);
        } else if path.extension().is_some_and(|e| e == "lock") && created_since(&path, started) {
            out.push(path);
        }
    }
}

/// Removes what a hard-killed fetch / push / LFS transfer started at
/// `started` left in the repository at `root`; returns what it removed.
pub fn remove_stale_transfer_files(root: &Path, started: SystemTime) -> Vec<PathBuf> {
    let Some(git) = common_git_dir(root) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    walk_locks(&git.join("refs"), started, &mut candidates);
    for name in ["packed-refs.lock", "shallow.lock"] {
        let path = git.join(name);
        if created_since(&path, started) {
            candidates.push(path);
        }
    }
    let pack = git.join("objects").join("pack");
    let lfs_tmp = git.join("lfs").join("tmp");
    for dir in [&pack, &lfs_tmp] {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let ours = if dir == &pack {
                name.starts_with("tmp_pack_")
                    || name.starts_with("tmp_idx_")
                    || name.starts_with("tmp_rev_")
            } else {
                path.is_file()
            };
            if ours && created_since(&path, started) {
                candidates.push(path);
            }
        }
    }
    candidates
        .into_iter()
        .filter(|path| retry(|| std::fs::remove_file(path)))
        .collect()
}

/// Runs [`remove_stale_transfer_files`] when dropped while armed.
pub(crate) struct RepoCleanup {
    root: PathBuf,
    started: SystemTime,
    armed: bool,
}

impl RepoCleanup {
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            started: SystemTime::now(),
            armed: true,
        }
    }

    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for RepoCleanup {
    fn drop(&mut self) {
        if self.armed {
            remove_stale_transfer_files(&self.root, self.started);
        }
    }
}

/// What the clone destination was before the clone started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Before {
    Missing,
    Empty,
    HadContent,
}

/// Puts the clone destination back as it was when dropped while armed.
pub(crate) struct CloneCleanup {
    dest: PathBuf,
    before: Before,
    armed: bool,
}

impl CloneCleanup {
    pub(crate) fn new(dest: &Path) -> Self {
        let before = match std::fs::read_dir(dest) {
            Err(_) => Before::Missing,
            Ok(mut entries) => {
                if entries.next().is_none() {
                    Before::Empty
                } else {
                    Before::HadContent
                }
            }
        };
        Self {
            dest: dest.to_path_buf(),
            before,
            armed: true,
        }
    }

    /// The clone ran to its end (git cleans a failed clone up itself).
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for CloneCleanup {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        match self.before {
            Before::Missing => {
                retry(|| std::fs::remove_dir_all(&self.dest));
            }
            Before::Empty => {
                if let Ok(entries) = std::fs::read_dir(&self.dest) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        retry(|| {
                            if path.is_dir() {
                                std::fs::remove_dir_all(&path)
                            } else {
                                std::fs::remove_file(&path)
                            }
                        });
                    }
                }
            }
            Before::HadContent => {}
        }
    }
}

/// The git subcommand of an argument list, skipping the leading `-c k=v`
/// pairs and flags (`clone`, `fetch`, `push`, `lfs`, ...).
pub(crate) fn subcommand<'a>(args: &[&'a str]) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if *arg == "-c" {
            iter.next();
        } else if !arg.starts_with('-') {
            return Some(arg);
        }
    }
    None
}

/// The cleanup a git command run in `root` needs when canceled, if any.
pub(crate) fn for_command(root: &Path, args: &[&str]) -> Option<RepoCleanup> {
    matches!(subcommand(args), Some("fetch" | "push" | "lfs")).then(|| RepoCleanup::new(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_subcommand_skips_config_pairs_and_flags() {
        assert_eq!(subcommand(&["-c", "a=b", "fetch", "--all"]), Some("fetch"));
        assert_eq!(
            subcommand(&["-c", "filter.lfs.required=false", "clone", "x"]),
            Some("clone")
        );
        assert_eq!(subcommand(&["--no-pager", "push"]), Some("push"));
        assert_eq!(subcommand(&[]), None);
    }
}
