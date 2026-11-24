//! `lfs_files` over libgit2: the one LFS operation that needs no
//! filter, no network and no `git-lfs` program (ADR 0009), so it is
//! native here. Everything else about LFS goes to the system-git
//! fallback.

use std::path::PathBuf;

use gitpoulpi_core::domain::{
    attributes_mention_lfs, LfsFile, LfsPointer, Oid, RepoRef, LFS_POINTER_MAX_BYTES,
};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_git2_oid};

/// Blobs shorter than this cannot hold the three pointer lines.
const MIN_POINTER_BYTES: usize = 40;

/// Does this repository route files through LFS: the root
/// `.gitattributes` or `info/attributes` mention `filter=lfs`, or an LFS
/// object store exists (which also catches patterns that live only in a
/// nested `.gitattributes`, once LFS has been used).
fn uses_lfs(repo: &RepoRef, common: &std::path::Path) -> bool {
    if common.join("lfs").is_dir() {
        return true;
    }
    [
        repo.root.join(".gitattributes"),
        common.join("info").join("attributes"),
    ]
    .iter()
    .any(|file| std::fs::read_to_string(file).is_ok_and(|t| attributes_mention_lfs(&t)))
}

/// Whether the operations libgit2 cannot do correctly here (anything that
/// writes the working tree or compares it with the index: libgit2 has no
/// LFS smudge / clean filter) must go to the system git. See ADR 0009.
pub fn is_active(repo: &RepoRef) -> bool {
    match open(repo) {
        Ok(git) => uses_lfs(repo, &gitpoulpi_core::paths::common_dir(git.path())),
        Err(_) => false,
    }
}

/// See `GitBackend::lfs_files`. Same rules as the system-git twin: an
/// empty index answer unless the root `.gitattributes` / `info/attributes`
/// mention `filter=lfs` or an LFS object store exists; a commit is always
/// scanned.
pub fn lfs_files(repo: &RepoRef, at: Option<Oid>) -> Result<Vec<LfsFile>> {
    let git = open(repo)?;
    let common = gitpoulpi_core::paths::common_dir(git.path());
    // The gate only spares the index scan: a revision is asked about for
    // what it holds (its `.gitattributes` may differ from the checked-out
    // one), and the scan is bounded by the pointer size.
    if at.is_none() && !uses_lfs(repo, &common) {
        return Ok(Vec::new());
    }

    let storage = match git
        .config()
        .ok()
        .and_then(|c| c.get_string("lfs.storage").ok())
        .filter(|s| !s.trim().is_empty())
    {
        Some(p) => {
            let p = PathBuf::from(p.trim());
            if p.is_absolute() {
                p
            } else {
                common.join(p)
            }
        }
        None => common.join("lfs"),
    };

    // (path, blob id) of every regular file at the source.
    let mut entries: Vec<(String, git2::Oid)> = Vec::new();
    match at {
        Some(oid) => {
            let commit = git
                .find_commit(to_git2_oid(oid)?)
                .map_err(|e| Error::Backend(format!("no commit {}: {e}", oid.to_hex())))?;
            let tree = commit.tree().map_err(map_err)?;
            tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
                let regular = matches!(entry.filemode(), 0o100644 | 0o100755);
                if regular && entry.kind() == Some(git2::ObjectType::Blob) {
                    if let Ok(name) = entry.name() {
                        entries.push((format!("{dir}{name}"), entry.id()));
                    }
                }
                git2::TreeWalkResult::Ok
            })
            .map_err(map_err)?;
        }
        None => {
            let index = git.index().map_err(map_err)?;
            for entry in index.iter() {
                let stage = (entry.flags >> 12) & 0x3;
                if stage == 0 && matches!(entry.mode, 0o100644 | 0o100755) {
                    entries.push((String::from_utf8_lossy(&entry.path).into_owned(), entry.id));
                }
            }
        }
    }

    let odb = git.odb().map_err(map_err)?;
    let mut files = Vec::new();
    for (path, id) in entries {
        // Header first: a large blob is never read to find out it is not
        // a pointer.
        let Ok((size, _)) = odb.read_header(id) else {
            continue;
        };
        if !(MIN_POINTER_BYTES..LFS_POINTER_MAX_BYTES).contains(&size) {
            continue;
        }
        let Ok(blob) = git.find_blob(id) else {
            continue;
        };
        let Some(pointer) = LfsPointer::parse(blob.content()) else {
            continue;
        };
        let local = storage.join(pointer.storage_rel_path()).is_file();
        files.push(LfsFile {
            path: PathBuf::from(path),
            pointer,
            local,
        });
    }
    Ok(files)
}
