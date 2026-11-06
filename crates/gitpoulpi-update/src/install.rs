//! Staging, swapping, reverting and cleaning up the files of an install.
//!
//! For each managed file `name` in the install folder:
//! - `name.new`: the downloaded replacement, staged (nothing running touched);
//! - `name.old`: the previous version's file, kept after [`apply`] so
//!   [`revert`] can bring it back, deleted by [`cleanup`] once the new
//!   version has run well.
//!
//! [`Replace`] is the platform's way of swapping a file that may be running.
//! It is a parameter, not a `cfg`, so the Windows path is tested on Linux.

use std::path::{Path, PathBuf};

use crate::archive::ArchiveFile;
use crate::fsops::FileOps;
use crate::{io_err, Result, UpdateError};
use gitpoulpi_config::release::{InstallChannel, Marker, MARKER_FILE};

pub const NEW: &str = ".new";
pub const OLD: &str = ".old";
const TMP: &str = ".swap";

/// How a file that a process may be running is replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replace {
    /// Unix: keep a second name for the old file, then rename the new one
    /// over it atomically. A running process keeps its inode.
    Atomic,
    /// Windows: a running exe cannot be overwritten or deleted, but it can be
    /// renamed. The old file moves to `.old`, then the new one moves in.
    MoveAside,
}

impl Replace {
    pub fn for_target(target: &str) -> Self {
        if gitpoulpi_config::release::is_windows(target) {
            Self::MoveAside
        } else {
            Self::Atomic
        }
    }
}

fn with(dir: &Path, name: &str, suffix: &str) -> PathBuf {
    dir.join(format!("{name}{suffix}"))
}

/// Writes each of `files` that has a counterpart in `dir` as `<name>.new`.
/// A file the install does not have (a binary the user removed) is not
/// added. Returns the staged names; on a write failure what was staged is
/// removed again.
pub fn stage(fs: &dyn FileOps, dir: &Path, files: &[ArchiveFile]) -> Result<Vec<String>> {
    let mut staged: Vec<String> = Vec::new();
    for file in files {
        if !fs.exists(&dir.join(&file.name)) {
            continue;
        }
        let target = with(dir, &file.name, NEW);
        if let Err(e) = fs.write(&target, &file.data, file.executable) {
            discard(fs, dir, &staged);
            return Err(io_err(format!("could not write {}", target.display()), e));
        }
        staged.push(file.name.clone());
    }
    Ok(staged)
}

/// Unpacks `archive` (a release archive for `version` / `target`) and stages
/// its files into `dir` as [`stage`] does. The elevated worker uses this with
/// the archive it just re-verified: it never trusts files a user-level process
/// staged in a folder only administrators may write.
pub fn stage_from_archive_into(
    fs: &dyn FileOps,
    dir: &Path,
    archive: &[u8],
    version: &str,
    target: &str,
) -> Result<Vec<String>> {
    let files = crate::archive::extract(archive, version, target)?;
    stage(fs, dir, &files)
}

/// Removes the staged `.new` files of `names` (best effort).
pub fn discard(fs: &dyn FileOps, dir: &Path, names: &[String]) {
    for name in names {
        let _ = fs.remove_file(&with(dir, name, NEW));
    }
}

/// The names (of `all`) that have a staged `.new` file.
pub fn staged_names(fs: &dyn FileOps, dir: &Path, all: &[String]) -> Vec<String> {
    all.iter()
        .filter(|n| fs.exists(&with(dir, n, NEW)))
        .cloned()
        .collect()
}

fn marker_of(fs: &dyn FileOps, dir: &Path, suffix: &str) -> Option<Marker> {
    Marker::parse(&fs.read_to_string(&with(dir, MARKER_FILE, suffix)).ok()?)
}

/// The version staged and waiting for a restart.
pub fn staged_version(fs: &dyn FileOps, dir: &Path) -> Option<String> {
    marker_of(fs, dir, NEW).map(|m| m.version)
}

/// The version `.old` holds: what [`revert`] would bring back.
pub fn previous_version(fs: &dyn FileOps, dir: &Path) -> Option<String> {
    marker_of(fs, dir, OLD).map(|m| m.version)
}

/// One step done, remembered so a failure can walk back.
enum Done {
    Backup(String),
    Install(String),
}

/// Fails with [`UpdateError::Locked`] naming the first of `names` (other than
/// `own`, which this process is and which may be renamed) that a program holds
/// open; only the Windows strategy cares.
fn check_locks(
    fs: &dyn FileOps,
    dir: &Path,
    names: &[String],
    replace: Replace,
    own: &str,
) -> Result<()> {
    if replace != Replace::MoveAside {
        return Ok(());
    }
    for name in names.iter().filter(|n| n.as_str() != own) {
        let path = dir.join(name);
        if fs.exists(&path) && fs.is_locked(&path) {
            return Err(UpdateError::Locked(name.clone()));
        }
    }
    Ok(())
}

/// Replaces every staged file by its `.new`, keeping the previous one as
/// `.old`. All or nothing: a failure (a locked file, a full disk) puts
/// everything back, leaving the `.new` files staged, and says which file.
///
/// The marker that lands is the archive's (`channel=archive`, or nothing):
/// the installed marker's `channel=` and `feed=` are carried over, because a
/// swap must never turn a Chocolatey install into an archive one.
pub fn apply(
    fs: &dyn FileOps,
    dir: &Path,
    all: &[String],
    replace: Replace,
    own: &str,
) -> Result<()> {
    apply_with_channel(fs, dir, all, replace, own, None)
}

/// [`apply`], recording `resolved` as the channel when the installed marker
/// has none (a Legacy install resolved to a channel: its first update
/// rewrites the marker).
pub fn apply_with_channel(
    fs: &dyn FileOps,
    dir: &Path,
    all: &[String],
    replace: Replace,
    own: &str,
    resolved: Option<InstallChannel>,
) -> Result<()> {
    let names = staged_names(fs, dir, all);
    if names.is_empty() {
        return Err(UpdateError::NothingStaged);
    }
    check_locks(fs, dir, &names, replace, own)?;
    let mut done: Vec<Done> = Vec::new();
    let result = (|| {
        for name in &names {
            let (live, old) = (dir.join(name), with(dir, name, OLD));
            if fs.exists(&old) {
                fs.remove_file(&old)
                    .map_err(|_| UpdateError::Locked(format!("{name}{OLD}")))?;
            }
            match replace {
                Replace::Atomic => fs
                    .link_or_copy(&live, &old)
                    .map_err(|e| io_err(format!("could not keep {name}"), e))?,
                Replace::MoveAside => fs
                    .rename(&live, &old)
                    .map_err(|_| UpdateError::Locked(name.clone()))?,
            }
            done.push(Done::Backup(name.clone()));
        }
        for name in &names {
            fs.rename(&with(dir, name, NEW), &dir.join(name))
                .map_err(|e| io_err(format!("could not install {name}"), e))?;
            done.push(Done::Install(name.clone()));
        }
        carry_channel(fs, dir, resolved)
    })();
    if result.is_err() {
        walk_back(fs, dir, done);
    }
    result
}

/// Rewrites the marker that was just moved in so it keeps what the previous
/// one recorded about the install: `channel=` (else `resolved`, else the new
/// marker's own) and `feed=`.
fn carry_channel(fs: &dyn FileOps, dir: &Path, resolved: Option<InstallChannel>) -> Result<()> {
    let (Some(old), Some(new)) = (marker_of(fs, dir, OLD), marker_of(fs, dir, "")) else {
        return Ok(());
    };
    let merged = Marker {
        channel: old.channel.or(resolved).or(new.channel),
        feed: old.feed.or_else(|| new.feed.clone()),
        ..new.clone()
    };
    if merged == new {
        return Ok(());
    }
    let path = dir.join(MARKER_FILE);
    fs.write(&path, merged.render().as_bytes(), false)
        .map_err(|e| io_err(format!("could not update {MARKER_FILE}"), e))
}

fn walk_back(fs: &dyn FileOps, dir: &Path, done: Vec<Done>) {
    for step in done.into_iter().rev() {
        match step {
            Done::Install(name) => {
                let _ = fs.rename(&dir.join(&name), &with(dir, &name, NEW));
            }
            Done::Backup(name) => {
                let (live, old) = (dir.join(&name), with(dir, &name, OLD));
                if fs.exists(&live) {
                    let _ = fs.remove_file(&old);
                } else {
                    let _ = fs.rename(&old, &live);
                }
            }
        }
    }
}

/// Brings the `.old` files back and keeps the current ones as `.old` (a swap,
/// so the same call goes forward again). All or nothing like [`apply`].
pub fn revert(
    fs: &dyn FileOps,
    dir: &Path,
    all: &[String],
    replace: Replace,
    own: &str,
) -> Result<()> {
    let names: Vec<String> = all
        .iter()
        .filter(|n| fs.exists(&with(dir, n, OLD)))
        .cloned()
        .collect();
    if names.is_empty() {
        return Err(UpdateError::NothingToRevert);
    }
    check_locks(fs, dir, &names, replace, own)?;
    // (from, to) renames done, undone in reverse.
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
    let result = (|| {
        for name in &names {
            let (live, old, tmp) = (dir.join(name), with(dir, name, OLD), with(dir, name, TMP));
            for (from, to) in [(&live, &tmp), (&old, &live), (&tmp, &old)] {
                fs.rename(from, to)
                    .map_err(|_| UpdateError::Locked(name.clone()))?;
                done.push((from.clone(), to.clone()));
            }
        }
        Ok(())
    })();
    if result.is_err() {
        for (from, to) in done.into_iter().rev() {
            let _ = fs.rename(&to, &from);
        }
    }
    result
}

/// What [`cleanup`] did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Cleanup {
    pub removed: Vec<String>,
    /// Still in use (Windows, the process that was replaced is running): the
    /// next start tries again.
    pub kept: Vec<String>,
}

/// Deletes the `.old` (and any stray `.swap`) files of `all`: the new version
/// has run well, there is nothing to go back to.
pub fn cleanup(fs: &dyn FileOps, dir: &Path, all: &[String]) -> Cleanup {
    let mut out = Cleanup::default();
    for name in all {
        for suffix in [OLD, TMP] {
            let path = with(dir, name, suffix);
            if !fs.exists(&path) {
                continue;
            }
            let label = format!("{name}{suffix}");
            match fs.remove_file(&path) {
                Ok(()) => out.removed.push(label),
                Err(_) => out.kept.push(label),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
