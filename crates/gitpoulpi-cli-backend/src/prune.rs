//! Removing the folders that a deletion of untracked files left empty.
//!
//! Shared by both backends' `discard` and by `snapshot_restore` (redo of a
//! discard), so they all apply one rule.

use std::path::{Component, Path};

/// After `removed` files (paths relative to `root`) were deleted from disk,
/// removes each directory that is now completely empty, walking up from
/// every file's parent and stopping at the first directory that is not.
///
/// A directory is only ever removed when it is a real directory (never a
/// symlink, which is not followed), has no entry at all (so no hidden or
/// ignored file, no sub-folder, no `.git` of a submodule or worktree), and
/// `tracked_under(dir)` says the index holds nothing beneath it. The root
/// itself is never touched, and a directory that no removed file lived in is
/// never visited, so a folder that was already empty stays. Returns the
/// removed directories (forward slashes, relative to `root`).
pub fn prune_empty_parents(
    root: &Path,
    removed: &[String],
    tracked_under: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut gone: Vec<String> = Vec::new();
    for file in removed {
        let rel = file.replace('\\', "/");
        let mut parts: Vec<&str> = rel
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        if parts.iter().any(|p| *p == ".." || *p == ".git") {
            continue;
        }
        parts.pop();
        while !parts.is_empty() {
            let dir_rel = parts.join("/");
            let full = root.join(&dir_rel);
            if full.components().any(|c| matches!(c, Component::ParentDir)) {
                break;
            }
            let Ok(meta) = std::fs::symlink_metadata(&full) else {
                // Already gone (another file's walk removed it): go on up.
                parts.pop();
                continue;
            };
            if !meta.is_dir() {
                break;
            }
            let empty = std::fs::read_dir(&full).is_ok_and(|mut d| d.next().is_none());
            if !empty || tracked_under(&dir_rel) {
                break;
            }
            if std::fs::remove_dir(&full).is_err() {
                break;
            }
            gone.push(dir_rel);
            parts.pop();
        }
    }
    gone.sort();
    gone.dedup();
    gone
}
