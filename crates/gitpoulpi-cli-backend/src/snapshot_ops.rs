//! The undo journal's index and file snapshots over the system `git`:
//! `index_tree`, `index_restore`, `snapshot_create`, `snapshot_restore`,
//! `path_hashes`, `delete_files`, `patch_files`.
//!
//! A snapshot is a **tree**: `index/<path>` holds the index entry and
//! `worktree/<path>` the file on disk, both as raw bytes (`hash-object
//! --no-filters`: CRLF and LFS pointers stay exactly as they are). It is
//! built in a throwaway index file, so the real index and the working tree
//! are never touched. Restoring reads blobs with `cat-file` (no smudge
//! filter either), so the round trip is byte-exact.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use gitpoulpi_core::domain::{FileHash, Oid, PathHash, RepoRef, Snapshot};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes_with_stdin, run_with_env_and_stdin, run_with_stdin};

pub const MODE_FILE: u32 = 0o100644;
#[cfg_attr(not(unix), allow(dead_code))]
pub const MODE_EXEC: u32 = 0o100755;
pub const MODE_LINK: u32 = 0o120000;
const ZERO_OID: &str = "0000000000000000000000000000000000000000";

/// Command lines are capped at about 32k characters on Windows.
const MAX_ARG_BYTES: usize = 20_000;

fn oid_of(text: &str) -> Result<Oid> {
    Oid::from_hex(text.trim())
        .ok_or_else(|| Error::Backend(format!("git printed `{}`, not an object id", text.trim())))
}

// --------------------------------------------------------------- the index

/// `git write-tree`. Reads the index and writes tree objects.
pub fn index_tree(repo: &RepoRef) -> Result<Oid> {
    oid_of(&run(&repo.root, &["write-tree"])?)
}

/// `git read-tree <tree>`: the index becomes the tree, the working tree is
/// left alone.
pub fn index_restore(repo: &RepoRef, tree: Oid) -> Result<()> {
    run(&repo.root, &["read-tree", &tree.to_hex()])?;
    Ok(())
}

// ------------------------------------------------------------- path walking

/// A repo-relative path in git's spelling, refused when it could leave the
/// repository or reach into `.git`.
pub(crate) fn normalize(path: &str) -> Result<String> {
    let norm = path.replace('\\', "/");
    let norm = norm.trim_start_matches("./").trim_end_matches('/');
    if norm.is_empty() {
        return Err(Error::Backend("an empty path".into()));
    }
    for part in Path::new(norm).components() {
        match part {
            Component::Normal(name) if name == ".git" => {
                return Err(Error::Backend(format!("{norm} is inside .git")))
            }
            Component::Normal(_) => {}
            _ => {
                return Err(Error::Backend(format!(
                    "{norm} is not a path inside the repository"
                )))
            }
        }
    }
    Ok(norm.to_string())
}

pub fn normalize_all(paths: &[String]) -> Result<Vec<String>> {
    let mut out = BTreeSet::new();
    for p in paths {
        out.insert(normalize(p)?);
    }
    Ok(out.into_iter().collect())
}

fn batches(specs: &[String]) -> Vec<&[String]> {
    let mut out = Vec::new();
    let (mut start, mut size) = (0, 0);
    for (i, s) in specs.iter().enumerate() {
        if size + s.len() + 1 > MAX_ARG_BYTES && i > start {
            out.push(&specs[start..i]);
            start = i;
            size = 0;
        }
        size += s.len() + 1;
    }
    if start < specs.len() {
        out.push(&specs[start..]);
    }
    out
}

/// What `git ls-files -s` says for every index entry under `paths`
/// (stage 0 only), by path. One spawn per batch.
fn index_entries(repo: &RepoRef, paths: &[String]) -> Result<BTreeMap<String, FileHash>> {
    index_entries_and_unmerged(repo, paths).map(|(entries, _)| entries)
}

/// [`index_entries`], plus the paths the same listing shows unmerged
/// (stages 1 to 3), which have no stage 0 entry to find.
fn index_entries_and_unmerged(
    repo: &RepoRef,
    paths: &[String],
) -> Result<(BTreeMap<String, FileHash>, BTreeSet<String>)> {
    let mut unmerged = BTreeSet::new();
    let specs: Vec<String> = paths.iter().map(|p| format!(":(literal){p}")).collect();
    let mut out = BTreeMap::new();
    for batch in batches(&specs) {
        let mut args = vec!["ls-files", "-s", "-z", "--"];
        args.extend(batch.iter().map(String::as_str));
        let raw = run(&repo.root, &args)?;
        for record in raw.split('\0').filter(|r| !r.is_empty()) {
            // `<mode> <oid> <stage>\t<path>`
            let Some((meta, path)) = record.split_once('\t') else {
                continue;
            };
            let mut fields = meta.split(' ');
            let (Some(mode), Some(oid), Some(stage)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            if stage != "0" {
                unmerged.insert(path.to_string());
                continue;
            }
            let (Ok(mode), Some(oid)) = (u32::from_str_radix(mode, 8), Oid::from_hex(oid)) else {
                continue;
            };
            out.insert(path.to_string(), FileHash { mode, oid });
        }
    }
    Ok((out, unmerged))
}

/// A file or symlink found on disk.
pub struct OnDisk {
    pub mode: u32,
}

/// Every file (and, on Unix, symlink) at or under each of `paths` in the
/// working tree, by repo-relative path. `.git` is never entered.
pub fn walk(repo: &RepoRef, paths: &[String]) -> BTreeMap<String, OnDisk> {
    let mut out = BTreeMap::new();
    for path in paths {
        walk_one(&repo.root, path, &mut out);
    }
    out
}

fn walk_one(root: &Path, rel: &str, out: &mut BTreeMap<String, OnDisk>) {
    let full = root.join(rel);
    let Ok(meta) = std::fs::symlink_metadata(&full) else {
        return;
    };
    if meta.is_dir() {
        let Ok(entries) = std::fs::read_dir(&full) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name == ".git" {
                continue;
            }
            walk_one(root, &format!("{rel}/{}", name.to_string_lossy()), out);
        }
    } else if meta.file_type().is_symlink() {
        out.insert(rel.to_string(), OnDisk { mode: MODE_LINK });
    } else if meta.is_file() {
        out.insert(
            rel.to_string(),
            OnDisk {
                mode: file_mode(&meta),
            },
        );
    }
}

#[cfg(unix)]
fn file_mode(meta: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    if meta.permissions().mode() & 0o111 != 0 {
        MODE_EXEC
    } else {
        MODE_FILE
    }
}

#[cfg(not(unix))]
fn file_mode(_meta: &std::fs::Metadata) -> u32 {
    MODE_FILE
}

/// Blob ids of the raw bytes of `files` (regular files in one
/// `hash-object --stdin-paths`, symlinks by their target text). With
/// `write`, the blobs are stored.
fn hash_worktree(
    repo: &RepoRef,
    files: &BTreeMap<String, OnDisk>,
    write: bool,
) -> Result<BTreeMap<String, Oid>> {
    let mut out = BTreeMap::new();
    let regular: Vec<&String> = files
        .iter()
        .filter(|(_, f)| f.mode != MODE_LINK)
        .map(|(p, _)| p)
        .collect();
    if let Some(bad) = regular.iter().find(|p| p.contains('\n')) {
        return Err(Error::Backend(format!(
            "cannot snapshot {bad:?}: its name contains a newline"
        )));
    }
    for chunk in regular.chunks(500) {
        let mut args = vec!["hash-object", "--no-filters", "--stdin-paths"];
        if write {
            args.push("-w");
        }
        let input = chunk
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let printed = run_with_stdin(&repo.root, &args, &format!("{input}\n"))?;
        let ids: Vec<&str> = printed.lines().collect();
        if ids.len() != chunk.len() {
            return Err(Error::Backend(
                "git hash-object answered for a different number of files".into(),
            ));
        }
        for (path, id) in chunk.iter().zip(ids) {
            out.insert((*path).clone(), oid_of(id)?);
        }
    }
    for (path, file) in files {
        if file.mode != MODE_LINK {
            continue;
        }
        let target = std::fs::read_link(repo.root.join(path))
            .map_err(|e| Error::Backend(format!("cannot read the link {path}: {e}")))?;
        let bytes = target.to_string_lossy().replace('\\', "/").into_bytes();
        let mut args = vec!["hash-object", "-t", "blob", "--stdin"];
        if write {
            args.push("-w");
        }
        let printed = run_bytes_with_stdin(&repo.root, &args, &bytes)?;
        out.insert(path.clone(), oid_of(&String::from_utf8_lossy(&printed))?);
    }
    Ok(out)
}

/// The state of every file under `paths`, in one pass (`write` stores the
/// worktree blobs, for a snapshot).
fn gather(repo: &RepoRef, paths: &[String], write: bool) -> Result<Vec<PathHash>> {
    let paths = normalize_all(paths)?;
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let index = index_entries(repo, &paths)?;
    let disk = walk(repo, &paths);
    let hashes = hash_worktree(repo, &disk, write)?;
    let all: BTreeSet<&String> = index.keys().chain(disk.keys()).collect();
    Ok(all
        .into_iter()
        .map(|path| PathHash {
            path: path.clone(),
            worktree: disk.get(path).and_then(|f| {
                hashes.get(path).map(|oid| FileHash {
                    mode: f.mode,
                    oid: *oid,
                })
            }),
            index: index.get(path).copied(),
        })
        .collect())
}

pub fn path_hashes(repo: &RepoRef, paths: &[String]) -> Result<Vec<PathHash>> {
    gather(repo, paths, false)
}

// ---------------------------------------------------------------- snapshot

static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn temp_index() -> PathBuf {
    let n = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "gitpoulpi-snapshot-{}-{n}.index",
        std::process::id()
    ))
}

pub fn snapshot_create(repo: &RepoRef, paths: &[String]) -> Result<Snapshot> {
    let hashes = gather(repo, paths, true)?;
    let mut input = String::new();
    for h in &hashes {
        for (side, file) in [("index", &h.index), ("worktree", &h.worktree)] {
            if let Some(file) = file {
                input.push_str(&format!(
                    "{:o} {}\t{side}/{}\0",
                    file.mode, file.oid, h.path
                ));
            }
        }
    }
    let tmp = temp_index();
    let tmp_str = tmp.to_string_lossy().to_string();
    let env = [("GIT_INDEX_FILE", tmp_str.as_str())];
    let built = (|| {
        if !input.is_empty() {
            run_with_env_and_stdin(
                &repo.root,
                &["update-index", "-z", "--index-info"],
                &env,
                &input,
            )?;
        }
        run_with_env_and_stdin(&repo.root, &["write-tree"], &env, "")
    })();
    let _ = std::fs::remove_file(&tmp);
    Ok(Snapshot {
        tree: oid_of(&built?)?,
        hashes,
    })
}

/// What `ls-tree -r` says about a snapshot: the two sides, by path.
fn read_snapshot(
    repo: &RepoRef,
    tree: Oid,
) -> Result<(BTreeMap<String, FileHash>, BTreeMap<String, FileHash>)> {
    let raw = run(&repo.root, &["ls-tree", "-r", "-z", &tree.to_hex()])?;
    let (mut index, mut worktree) = (BTreeMap::new(), BTreeMap::new());
    for record in raw.split('\0').filter(|r| !r.is_empty()) {
        // `<mode> blob <oid>\t<path>`
        let Some((meta, path)) = record.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split(' ');
        let (Some(mode), Some(_kind), Some(oid)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let (Ok(mode), Some(oid)) = (u32::from_str_radix(mode, 8), Oid::from_hex(oid)) else {
            continue;
        };
        let file = FileHash { mode, oid };
        if let Some(p) = path.strip_prefix("index/") {
            index.insert(p.to_string(), file);
        } else if let Some(p) = path.strip_prefix("worktree/") {
            worktree.insert(p.to_string(), file);
        }
    }
    Ok((index, worktree))
}

/// The bytes of each blob, one `cat-file --batch` for all of them.
fn read_blobs(repo: &RepoRef, oids: &HashSet<Oid>) -> Result<HashMap<Oid, Vec<u8>>> {
    let mut out = HashMap::new();
    if oids.is_empty() {
        return Ok(out);
    }
    let input: String = oids.iter().map(|o| format!("{o}\n")).collect();
    let raw = run_bytes_with_stdin(&repo.root, &["cat-file", "--batch"], input.as_bytes())?;
    let mut at = 0usize;
    for _ in 0..oids.len() {
        let Some(nl) = raw[at..].iter().position(|b| *b == b'\n') else {
            return Err(Error::Backend("truncated git cat-file output".into()));
        };
        let header = String::from_utf8_lossy(&raw[at..at + nl]).to_string();
        at += nl + 1;
        let mut parts = header.split(' ');
        let (Some(id), Some(kind), Some(size)) = (parts.next(), parts.next(), parts.next()) else {
            return Err(Error::Backend(format!(
                "a snapshot object is missing ({header})"
            )));
        };
        if kind != "blob" {
            return Err(Error::Backend(format!(
                "a snapshot object is missing ({header})"
            )));
        }
        let size: usize = size
            .parse()
            .map_err(|_| Error::Backend("bad git cat-file size".into()))?;
        let id = oid_of(id)?;
        if raw.len() < at + size + 1 {
            return Err(Error::Backend("truncated git cat-file output".into()));
        }
        out.insert(id, raw[at..at + size].to_vec());
        at += size + 1;
    }
    Ok(out)
}

#[cfg(unix)]
pub fn write_file(full: &Path, file: &FileHash, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    remove_any(full)?;
    if file.mode == MODE_LINK {
        let target = String::from_utf8_lossy(bytes).to_string();
        return std::os::unix::fs::symlink(target, full);
    }
    std::fs::write(full, bytes)?;
    let mode = if file.mode == MODE_EXEC { 0o755 } else { 0o644 };
    std::fs::set_permissions(full, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
pub fn write_file(full: &Path, _file: &FileHash, bytes: &[u8]) -> std::io::Result<()> {
    remove_any(full)?;
    std::fs::write(full, bytes)
}

/// Removes whatever is at `full` (file, symlink or directory), if anything.
pub fn remove_any(full: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(full) {
        Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(full),
        Ok(_) => std::fs::remove_file(full),
        Err(_) => Ok(()),
    }
}

pub fn snapshot_restore(repo: &RepoRef, snapshot: Oid, paths: &[String]) -> Result<()> {
    let paths = normalize_all(paths)?;
    if paths.is_empty() {
        return Ok(());
    }
    let (want_index, want_worktree) = read_snapshot(repo, snapshot)?;
    // Read everything first: a missing object fails before any change.
    let blobs = read_blobs(
        repo,
        &want_worktree
            .values()
            .map(|f| f.oid)
            .collect::<HashSet<_>>(),
    )?;

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
    // A conflicted path has no stage 0 to find: every stage goes, then
    // the wanted entry (a resolution being redone) comes in as stage 0.
    let (have, unmerged) = index_entries_and_unmerged(repo, &paths)?;
    let mut input = String::new();
    for path in have.keys() {
        if !want_index.contains_key(path) {
            input.push_str(&format!("0 {ZERO_OID}\t{path}\0"));
        }
    }
    for path in &unmerged {
        if !have.contains_key(path) {
            input.push_str(&format!("0 {ZERO_OID}\t{path}\0"));
        }
    }
    for (path, file) in &want_index {
        input.push_str(&format!("{:o} {}\t{path}\0", file.mode, file.oid));
    }
    if !input.is_empty() {
        run_with_stdin(&repo.root, &["update-index", "-z", "--index-info"], &input)?;
    }
    Ok(())
}

// ------------------------------------------------------------ delete, patch

pub fn delete_files(repo: &RepoRef, paths: &[String]) -> Result<()> {
    let paths = normalize_all(paths)?;
    for path in &paths {
        let full = repo.root.join(path);
        remove_any(&full).map_err(|e| Error::Backend(format!("cannot delete {path}: {e}")))?;
    }
    Ok(())
}

/// `git apply --numstat -z` names every file a patch touches; a rename is
/// an empty path followed by the old and the new name.
pub fn patch_files(repo: &RepoRef, patch: &Path) -> Result<Vec<String>> {
    if !patch.is_file() {
        return Err(Error::Backend(format!("{} is not a file", patch.display())));
    }
    let arg = patch.to_string_lossy().to_string();
    let raw = run(&repo.root, &["apply", "--numstat", "-z", &arg])?;
    let mut tokens = raw.split('\0').filter(|t| !t.is_empty());
    let mut out = BTreeSet::new();
    while let Some(token) = tokens.next() {
        let mut parts = token.splitn(3, '\t');
        let (_, _, path) = (parts.next(), parts.next(), parts.next());
        match path {
            Some("") => {
                for _ in 0..2 {
                    if let Some(name) = tokens.next() {
                        out.insert(name.to_string());
                    }
                }
            }
            Some(path) => {
                out.insert(path.to_string());
            }
            None => {}
        }
    }
    // numstat names only the new side of a rename; the old one is a header.
    if let Ok(bytes) = std::fs::read(patch) {
        for line in String::from_utf8_lossy(&bytes).lines() {
            if let Some(name) = line.strip_prefix("rename from ") {
                out.insert(name.to_string());
            }
        }
    }
    Ok(out.into_iter().collect())
}
