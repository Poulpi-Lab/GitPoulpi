//! The undo journal's index and file snapshots over libgit2, with no
//! process (the system-git twin is
//! `gitpoulpi_cli_backend::snapshot_ops`, and the contract suite holds the
//! two identical). A snapshot is a tree: `index/<path>` is the index entry
//! and `worktree/<path>` the file on disk, raw bytes (no clean/smudge
//! filter, so CRLF and LFS pointers stay as they are).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use git2::{ObjectType, Oid as GitOid};
use gitpoulpi_cli_backend::{
    snapshot_normalize_all as normalize_all, snapshot_remove_any as remove_any,
    snapshot_walk as walk, snapshot_write_file as write_file, MODE_LINK,
};
use gitpoulpi_core::domain::{FileHash, Oid, PathHash, RepoRef, Snapshot};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

fn file_hash(mode: u32, oid: GitOid) -> Result<FileHash> {
    Ok(FileHash {
        mode,
        oid: to_domain_oid(oid)?,
    })
}

/// Stage-0 index entries at or under `paths`, by path.
fn index_entries(index: &git2::Index, paths: &[String]) -> Result<BTreeMap<String, FileHash>> {
    let wanted: HashSet<&str> = paths.iter().map(String::as_str).collect();
    let mut out = BTreeMap::new();
    for entry in index.iter() {
        if (entry.flags & 0x3000) != 0 {
            continue;
        }
        let path = String::from_utf8_lossy(&entry.path).to_string();
        let mut hit = wanted.contains(path.as_str());
        if !hit {
            let mut end = path.len();
            while let Some(slash) = path[..end].rfind('/') {
                if wanted.contains(&path[..slash]) {
                    hit = true;
                    break;
                }
                end = slash;
            }
        }
        if hit {
            out.insert(path, file_hash(entry.mode, entry.id)?);
        }
    }
    Ok(out)
}

/// The blob id of a file's raw bytes (a symlink: its target text),
/// stored when `write`.
fn hash_disk(
    git: &git2::Repository,
    repo: &RepoRef,
    path: &str,
    mode: u32,
    write: bool,
) -> Result<GitOid> {
    let full = repo.root.join(path);
    let bytes = if mode == MODE_LINK {
        std::fs::read_link(&full)
            .map_err(|e| Error::Backend(format!("cannot read the link {path}: {e}")))?
            .to_string_lossy()
            .replace('\\', "/")
            .into_bytes()
    } else {
        std::fs::read(&full).map_err(|e| Error::Backend(format!("cannot read {path}: {e}")))?
    };
    if write {
        git.odb()
            .and_then(|odb| odb.write(ObjectType::Blob, &bytes))
            .map_err(map_err)
    } else {
        GitOid::hash_object(ObjectType::Blob, &bytes).map_err(map_err)
    }
}

fn gather(
    git: &git2::Repository,
    repo: &RepoRef,
    paths: &[String],
    write: bool,
) -> Result<Vec<PathHash>> {
    let paths = normalize_all(paths)?;
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let index = index_entries(&git.index().map_err(map_err)?, &paths)?;
    let disk = walk(repo, &paths);
    let mut worktree: BTreeMap<&String, FileHash> = BTreeMap::new();
    for (path, file) in &disk {
        let oid = hash_disk(git, repo, path, file.mode, write)?;
        worktree.insert(path, file_hash(file.mode, oid)?);
    }
    let all: BTreeSet<&String> = index.keys().chain(disk.keys()).collect();
    Ok(all
        .into_iter()
        .map(|path| PathHash {
            path: path.clone(),
            worktree: worktree.get(path).copied(),
            index: index.get(path).copied(),
        })
        .collect())
}

pub fn path_hashes(repo: &RepoRef, paths: &[String]) -> Result<Vec<PathHash>> {
    let git = open(repo)?;
    gather(&git, repo, paths, false)
}

fn entry(path: String, file: &FileHash) -> Result<git2::IndexEntry> {
    Ok(git2::IndexEntry {
        ctime: git2::IndexTime::new(0, 0),
        mtime: git2::IndexTime::new(0, 0),
        dev: 0,
        ino: 0,
        mode: file.mode,
        uid: 0,
        gid: 0,
        file_size: 0,
        id: to_git2_oid(file.oid)?,
        flags: path.len().min(0xFFF) as u16,
        flags_extended: 0,
        path: path.into_bytes(),
    })
}

pub fn snapshot_create(repo: &RepoRef, paths: &[String]) -> Result<Snapshot> {
    let git = open(repo)?;
    let hashes = gather(&git, repo, paths, true)?;
    let mut tree_index = git2::Index::new().map_err(map_err)?;
    for h in &hashes {
        for (side, file) in [("index", &h.index), ("worktree", &h.worktree)] {
            if let Some(file) = file {
                tree_index
                    .add(&entry(format!("{side}/{}", h.path), file)?)
                    .map_err(map_err)?;
            }
        }
    }
    let tree = tree_index.write_tree_to(&git).map_err(map_err)?;
    Ok(Snapshot {
        tree: to_domain_oid(tree)?,
        hashes,
    })
}

/// One side of a snapshot tree: path -> (mode, blob).
fn read_side(
    git: &git2::Repository,
    tree: &git2::Tree,
    side: &str,
) -> Result<BTreeMap<String, FileHash>> {
    let mut out = BTreeMap::new();
    let Some(sub) = tree.get_name(side) else {
        return Ok(out);
    };
    let sub = sub
        .to_object(git)
        .and_then(|o| o.peel_to_tree())
        .map_err(map_err)?;
    let mut failure = None;
    sub.walk(git2::TreeWalkMode::PreOrder, |dir, e| {
        if matches!(e.kind(), Some(ObjectType::Blob | ObjectType::Commit)) {
            match file_hash(e.filemode() as u32, e.id()) {
                Ok(f) => {
                    out.insert(
                        format!("{dir}{}", String::from_utf8_lossy(e.name_bytes())),
                        f,
                    );
                }
                Err(err) => {
                    failure = Some(err);
                    return git2::TreeWalkResult::Abort;
                }
            }
        }
        git2::TreeWalkResult::Ok
    })
    .map_err(map_err)?;
    failure.map_or(Ok(out), Err)
}

pub fn snapshot_restore(repo: &RepoRef, snapshot: Oid, paths: &[String]) -> Result<()> {
    let paths = normalize_all(paths)?;
    if paths.is_empty() {
        return Ok(());
    }
    let git = open(repo)?;
    let tree = git.find_tree(to_git2_oid(snapshot)?).map_err(map_err)?;
    let want_index = read_side(&git, &tree, "index")?;
    let want_worktree = read_side(&git, &tree, "worktree")?;
    // Read everything first: a missing object fails before any change.
    let mut blobs: HashMap<Oid, Vec<u8>> = HashMap::new();
    for file in want_worktree.values() {
        if blobs.contains_key(&file.oid) {
            continue;
        }
        let blob = git
            .find_blob(to_git2_oid(file.oid)?)
            .map_err(|e| Error::Backend(format!("a snapshot object is missing ({e})")))?;
        blobs.insert(file.oid, blob.content().to_vec());
    }

    // Working tree: drop what is there and the snapshot lacks, write the rest.
    let current = walk(repo, &paths);
    for path in current.keys() {
        if !want_worktree.contains_key(path) {
            remove_any(&repo.root.join(path))
                .map_err(|e| Error::Backend(format!("cannot remove {path}: {e}")))?;
        }
    }
    for (path, file) in &want_worktree {
        let full = repo.root.join(path);
        if let Some(parent) = full.parent() {
            // A file where a directory has to be (or the reverse) goes first.
            let mut up = parent.to_path_buf();
            while up != repo.root && !up.exists() {
                match up.parent() {
                    Some(p) => up = p.to_path_buf(),
                    None => break,
                }
            }
            if up.is_file() {
                let _ = std::fs::remove_file(&up);
            }
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Backend(format!("cannot create {}: {e}", parent.display())))?;
        }
        let bytes = blobs.get(&file.oid).expect("every wanted blob was read");
        write_file(&full, file, bytes)
            .map_err(|e| Error::Backend(format!("cannot restore {path}: {e}")))?;
    }

    // Index: remove what is there and the snapshot lacks, set the rest.
    let mut index = git.index().map_err(map_err)?;
    let have = index_entries(&index, &paths)?;
    for path in have.keys() {
        if !want_index.contains_key(path) {
            index.remove(Path::new(path), 0).map_err(map_err)?;
        }
    }
    // A conflicted path (a resolution being redone) has no stage 0: its
    // stages go, and the wanted entry comes in as stage 0.
    let unmerged: Vec<PathBuf> = index
        .conflicts()
        .map_err(map_err)?
        .flatten()
        .filter_map(|c| c.our.or(c.their).or(c.ancestor))
        .map(|e| PathBuf::from(String::from_utf8_lossy(&e.path).into_owned()))
        .filter(|p| {
            let p = p.to_string_lossy().replace('\\', "/");
            paths
                .iter()
                .any(|root| p == *root || p.starts_with(&format!("{root}/")))
        })
        .collect();
    for path in unmerged {
        index.remove_path(&path).map_err(map_err)?;
    }
    for (path, file) in &want_index {
        index.add(&entry(path.clone(), file)?).map_err(map_err)?;
    }
    index.write().map_err(map_err)
}
