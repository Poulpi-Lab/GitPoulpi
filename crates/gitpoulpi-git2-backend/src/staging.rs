//! Whole-file / folder / everything staging, unstaging and discard over
//! libgit2's index: no process spawn, one index write per operation.
//!
//! Hunk and line staging stay on the system-git fallback (`patch.rs` owns
//! the one patch builder), and so does everything in an LFS repository
//! (libgit2 has no clean/smudge filter for LFS, ADR 0009) -- both are the
//! caller's dispatch, not this module's.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{RepoRef, StageTarget};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open};

/// Forward-slash, no leading `./`, no trailing separator: how the index
/// spells a path.
fn index_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let s = s.trim_start_matches("./").trim_end_matches('/');
    s.to_string()
}

/// Whether `target` is something this module handles (File, Folder, All).
pub fn handles(target: &StageTarget) -> bool {
    matches!(
        target,
        StageTarget::File(_) | StageTarget::Folder(_) | StageTarget::All
    )
}

pub fn stage(repo: &RepoRef, target: &StageTarget) -> Result<()> {
    let git = open(repo)?;
    let mut index = git.index().map_err(map_err)?;
    match target {
        StageTarget::File(path) | StageTarget::Folder(path) => {
            let rel = index_path(path);
            let full = repo.root.join(&rel);
            let on_disk = std::fs::symlink_metadata(&full).ok();
            match on_disk {
                Some(meta) if meta.is_dir() => {
                    // A folder (or a file target that is one): `git add -A <dir>`.
                    add_dirty(&git, &mut index, Some(&rel))?;
                }
                Some(_) => {
                    index.add_path(Path::new(&rel)).map_err(map_err)?;
                }
                None => {
                    // Gone from disk: stage the deletion if the index (or,
                    // for a folder, anything under it) holds it.
                    let known = index.get_path(Path::new(&rel), 0).is_some()
                        || index.iter().any(|e| {
                            let p = String::from_utf8_lossy(&e.path);
                            p.starts_with(&format!("{rel}/"))
                        });
                    if !known {
                        return Err(Error::Backend(format!(
                            "git2: pathspec '{rel}' did not match any files"
                        )));
                    }
                    index.update_all([rel.as_str()], None).map_err(map_err)?;
                }
            }
        }
        StageTarget::All => {
            add_dirty(&git, &mut index, None)?;
        }
        StageTarget::Hunk(..) | StageTarget::Line(..) => {
            return Err(Error::Unsupported {
                operation: "Git2Backend::stage(Hunk/Line) - dispatched via decorator fallback",
            });
        }
    }
    index.write().map_err(map_err)
}

/// Above this many dirty paths one `git add` process (which hashes the
/// files with a warm stat cache and a single attribute lookup) beats
/// libgit2's per-file filter and attribute checks, which cost about 10 ms a
/// file on a Windows box with endpoint scanning. Below it the spawn
/// (100-200 ms) is the bigger cost. The caller falls back to system git
/// on `Error::Unsupported`.
const NATIVE_ADD_LIMIT: usize = 30;

/// `git add -A [<dir>]`: stages exactly what `status` reports as dirty in
/// the worktree (ignored files are not reported, so they are never
/// added). Driving it from the status list rather than from libgit2's
/// `add_all` walk matters on a slow filesystem: `add_all` re-checks
/// ignore and attribute files for every entry it visits, several times
/// slower than reading only the files that changed.
fn add_dirty(git: &git2::Repository, index: &mut git2::Index, dir: Option<&str>) -> Result<()> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .renames_head_to_index(false)
        .renames_index_to_workdir(false);
    if let Some(dir) = dir {
        opts.pathspec(dir);
    }
    let statuses = git.statuses(Some(&mut opts)).map_err(map_err)?;
    if statuses.len() > NATIVE_ADD_LIMIT {
        return Err(Error::Unsupported {
            operation: "Git2Backend::stage(many files) - dispatched via decorator fallback",
        });
    }
    let mut nested: Vec<String> = Vec::new();
    for entry in statuses.iter() {
        let st = entry.status();
        let Some(path) = entry.path().ok() else { continue };
        if path.ends_with('/') {
            // An untracked nested repository: let libgit2 add it as one.
            nested.push(path.trim_end_matches('/').to_string());
        } else if st.contains(git2::Status::WT_DELETED) {
            let _ = index.remove_path(Path::new(path));
        } else if st.intersects(
            git2::Status::WT_NEW
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::CONFLICTED,
        ) {
            index.add_path(Path::new(path)).map_err(map_err)?;
        }
    }
    if !nested.is_empty() {
        index
            .add_all(&nested, git2::IndexAddOption::DEFAULT, None)
            .map_err(map_err)?;
    }
    Ok(())
}

pub fn unstage(repo: &RepoRef, target: &StageTarget) -> Result<()> {
    let spec = match target {
        StageTarget::File(path) | StageTarget::Folder(path) => index_path(path),
        StageTarget::All => "*".to_string(),
        StageTarget::Hunk(..) | StageTarget::Line(..) => {
            return Err(Error::Unsupported {
                operation: "Git2Backend::unstage(Hunk/Line) - dispatched via decorator fallback",
            });
        }
    };
    let git = open(repo)?;
    let head = git.head();
    match head {
        Ok(head) => {
            let commit = head.peel(git2::ObjectType::Commit).map_err(map_err)?;
            let specs = if matches!(target, StageTarget::File(_)) {
                // An explicit path is reset like `git restore --staged` does.
                vec![spec.clone()]
            } else {
                unmerged_free_specs(&git, &spec)?
            };
            if specs.is_empty() {
                return Ok(());
            }
            git.reset_default(Some(&commit), specs.iter().map(String::as_str))
                .map_err(map_err)
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
            // No HEAD to restore from: unstaging empties the entries.
            let mut index = git.index().map_err(map_err)?;
            index.remove_all([spec.as_str()], None).map_err(map_err)?;
            index.write().map_err(map_err)
        }
        Err(e) => Err(map_err(e)),
    }
}

/// `GitBackend::discard` for a repository without LFS. Paths `HEAD`
/// has get their index entry reset to `HEAD`'s and their worktree copy
/// rewritten from it (one `checkout_index` for all of them); every other
/// path is dropped from the index and deleted from disk. One index write.
pub fn discard(repo: &RepoRef, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let git = open(repo)?;
    let head_tree = match git.head() {
        Ok(head) => Some(head.peel_to_tree().map_err(map_err)?),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(map_err(e)),
    };
    let mut index = git.index().map_err(map_err)?;

    let mut restore: Vec<String> = Vec::new();
    let mut remove: Vec<(String, PathBuf)> = Vec::new();
    for raw in paths {
        let rel = index_path(Path::new(raw));
        if rel.is_empty() {
            continue;
        }
        let entry = head_tree
            .as_ref()
            .and_then(|t| t.get_path(Path::new(&rel)).ok())
            .filter(|e| e.kind() == Some(git2::ObjectType::Blob));
        match entry {
            Some(e) => {
                let bytes = rel.clone().into_bytes();
                let idx_entry = git2::IndexEntry {
                    ctime: git2::IndexTime::new(0, 0),
                    mtime: git2::IndexTime::new(0, 0),
                    dev: 0,
                    ino: 0,
                    mode: e.filemode() as u32,
                    uid: 0,
                    gid: 0,
                    file_size: 0,
                    id: e.id(),
                    flags: bytes.len().min(0xFFF) as u16,
                    flags_extended: 0,
                    path: bytes,
                };
                index.add(&idx_entry).map_err(map_err)?;
                restore.push(rel);
            }
            None => {
                let full = repo.root.join(&rel);
                remove.push((rel, full));
            }
        }
    }

    if !restore.is_empty() {
        let mut cb = git2::build::CheckoutBuilder::new();
        cb.force().update_index(true);
        for p in &restore {
            cb.path(p);
        }
        git.checkout_index(Some(&mut index), Some(&mut cb))
            .map_err(map_err)?;
    }
    for (rel, full) in &remove {
        let _ = index.remove_path(Path::new(rel));
        if full.is_file() || full.is_symlink() {
            let _ = std::fs::remove_file(full);
        }
    }
    index.write().map_err(map_err)
}

/// The pathspecs an unstage of `spec` may reset. An unmerged path has no
/// staged change to take back (its row belongs to the Unstaged half), and
/// resetting it to `HEAD` would silently drop its conflict stages; so while
/// the index holds conflicts the spec is replaced by the staged,
/// non-conflicted paths it matches. Without conflicts it is `spec` itself.
fn unmerged_free_specs(git: &git2::Repository, spec: &str) -> Result<Vec<String>> {
    let has_conflicts = git.index().map_err(map_err)?.has_conflicts();
    if !has_conflicts {
        return Ok(vec![spec.to_string()]);
    }
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(false)
        .include_ignored(false)
        .renames_head_to_index(false)
        .renames_index_to_workdir(false)
        .pathspec(spec);
    let statuses = git.statuses(Some(&mut opts)).map_err(map_err)?;
    let staged = git2::Status::INDEX_NEW
        | git2::Status::INDEX_MODIFIED
        | git2::Status::INDEX_DELETED
        | git2::Status::INDEX_RENAMED
        | git2::Status::INDEX_TYPECHANGE;
    Ok(statuses
        .iter()
        .filter(|e| e.status().intersects(staged) && !e.status().contains(git2::Status::CONFLICTED))
        .filter_map(|e| e.path().ok().map(str::to_string))
        .collect())
}
