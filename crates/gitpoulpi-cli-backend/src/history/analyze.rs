//! The size analysis (slice S18): one `log --raw` pass over the scope for
//! every blob that was ever added or removed (with its paths and commits),
//! one `cat-file --batch-check` for their sizes, and HEAD's file list to say
//! what is still there.
//!
//! The work is proportional to the changes in the history, not to commits
//! times files. A merge is read against each parent (`-m`) so a blob that
//! only a conflict resolution introduced is seen; identical content is one
//! blob however many paths and commits hold it.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Stdio};

use gitpoulpi_core::domain::rewrite::{
    BlobRow, BlobState, DeletedRow, FolderRow, RewritePhase, RewriteProgress, RewriteProgressSink,
    RewriteRefusal, Scope, ScopeOptions, SizeAnalysis, MAX_TOP,
};
use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use super::refs;
use super::walk::{self, read_git, refuse};
use crate::process::{failed_with, spawn_failed};

type Tick<'a> = &'a mut dyn FnMut(RewritePhase, u64, u64) -> Result<()>;

/// Paths remembered per blob: past this a blob is "in many places".
const PATHS_PER_BLOB: usize = 64;
/// Blobs asked about in one `cat-file --batch-check` exchange.
const SIZE_CHUNK: usize = 4096;

/// Kills a child process on drop (a canceled or failed pass).
struct Reaper(Child);

impl Drop for Reaper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[derive(Default)]
struct BlobAcc {
    path: String,
    paths: HashSet<String>,
    first: usize,
    last: usize,
}

struct PathAcc {
    blobs: HashSet<Oid>,
    first: usize,
    last: usize,
}

fn io(e: std::io::Error) -> Error {
    Error::Backend(format!("git: {e}"))
}

fn canceled() -> Error {
    // The daemon puts its job number on it.
    Error::Canceled { job: 0 }
}

fn is_zero(hex: &str) -> bool {
    hex.bytes().all(|b| b == b'0')
}

/// Regular files and symlinks hold blobs; a submodule's id is no object here.
fn holds_blob(mode: &str) -> bool {
    matches!(mode, "100644" | "100755" | "120000")
}

/// The files HEAD has now, with their blob ids. Empty when HEAD names no
/// commit (the scope walk refuses that case on its own).
fn head_files(root: &Path) -> HashMap<String, Oid> {
    let args = ["ls-tree", "-r", "-z", "HEAD"];
    let Ok(out) = read_git(root, &args).output() else {
        return HashMap::new();
    };
    if !out.status.success() {
        return HashMap::new();
    }
    let mut files = HashMap::new();
    for entry in out.stdout.split(|b| *b == 0) {
        let text = String::from_utf8_lossy(entry);
        let Some((meta, path)) = text.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split_whitespace();
        let (Some(mode), Some("blob"), Some(id)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if !holds_blob(mode) {
            continue;
        }
        if let Some(oid) = Oid::from_hex(id) {
            files.insert(path.to_string(), oid);
        }
    }
    files
}

/// Every directory above `path`: `a/b/c.bin` gives `a` and `a/b`.
fn dirs_of(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/').map(move |(i, _)| &path[..i])
}

/// The sizes of `ids`: object size and size in the database; an object that
/// is missing (a partial clone) is left out.
fn sizes(root: &Path, ids: &[Oid], tick: Tick<'_>) -> Result<HashMap<Oid, (u64, u64)>> {
    let mut sizes = HashMap::with_capacity(ids.len());
    if ids.is_empty() {
        return Ok(sizes);
    }
    let args = [
        "cat-file",
        "--batch-check=%(objectname) %(objectsize) %(objectsize:disk)",
    ];
    let mut child = Reaper(
        read_git(root, &args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(spawn_failed)?,
    );
    let mut stdin = child.0.stdin.take().expect("stdin was piped");
    let mut out = BufReader::new(child.0.stdout.take().expect("stdout was piped"));
    let total = ids.len() as u64;
    let mut done = 0u64;
    for chunk in ids.chunks(SIZE_CHUNK) {
        tick(RewritePhase::Rewriting, done, total)?;
        let mut input = String::with_capacity(chunk.len() * 41);
        for id in chunk {
            input.push_str(&id.to_hex());
            input.push('\n');
        }
        // Written from its own thread: git answers while it is still reading.
        let result = std::thread::scope(|scope| {
            let writer = scope.spawn(|| {
                stdin
                    .write_all(input.as_bytes())
                    .and_then(|()| stdin.flush())
            });
            let result = (|| -> Result<()> {
                let mut line = String::new();
                for _ in chunk {
                    line.clear();
                    if out.read_line(&mut line).map_err(io)? == 0 {
                        return Err(Error::Backend("cat-file stopped early".into()));
                    }
                    let mut fields = line.split_whitespace();
                    let (Some(id), Some(size), Some(disk)) =
                        (fields.next(), fields.next(), fields.next())
                    else {
                        continue; // `<id> missing`
                    };
                    if let (Some(id), Ok(size), Ok(disk)) =
                        (Oid::from_hex(id), size.parse::<u64>(), disk.parse::<u64>())
                    {
                        sizes.insert(id, (size, disk));
                    }
                }
                Ok(())
            })();
            if result.is_err() {
                let _ = child.0.kill();
            }
            let _ = writer.join();
            result
        });
        result?;
        done += chunk.len() as u64;
    }
    tick(RewritePhase::Rewriting, total, total)?;
    Ok(sizes)
}

/// What `log --raw` says about the scope: blobs and paths with their commits.
struct Changes {
    commits: Vec<Oid>,
    blobs: HashMap<Oid, BlobAcc>,
    paths: HashMap<String, PathAcc>,
}

fn changes(
    root: &Path,
    tips: &[String],
    not: &[String],
    total: u64,
    tick: Tick<'_>,
) -> Result<Changes> {
    let mut args = vec![
        "log",
        "--raw",
        "-r",
        "-z",
        "--root",
        "--no-renames",
        "--no-abbrev",
        "-m",
        "--no-color",
        "--format=%x01%H",
        "--topo-order",
        "--reverse",
    ];
    args.extend(tips.iter().map(String::as_str));
    if !not.is_empty() {
        args.push("--not");
        args.extend(not.iter().map(String::as_str));
    }
    let mut child = Reaper(
        read_git(root, &args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(spawn_failed)?,
    );
    let mut out = BufReader::with_capacity(1 << 16, child.0.stdout.take().expect("piped"));
    let mut found = Changes {
        commits: Vec::new(),
        blobs: HashMap::new(),
        paths: HashMap::new(),
    };
    let mut token = Vec::new();
    // The meta line waiting for its path token.
    let mut pending: Option<(String, String, String, String)> = None;
    loop {
        token.clear();
        if out.read_until(0, &mut token).map_err(io)? == 0 {
            break;
        }
        if token.last() == Some(&0) {
            token.pop();
        }
        let text = String::from_utf8_lossy(&token);
        let text = text.trim_start_matches('\n');
        if let Some((mo, mn, old, new)) = pending.take() {
            let idx = found.commits.len().saturating_sub(1);
            let path = text.to_string();
            for (mode, id) in [(&mo, &old), (&mn, &new)] {
                if is_zero(id) || !holds_blob(mode) {
                    continue;
                }
                let Some(oid) = Oid::from_hex(id) else {
                    continue;
                };
                let blob = found.blobs.entry(oid).or_insert_with(|| BlobAcc {
                    first: idx,
                    ..BlobAcc::default()
                });
                if blob.path.is_empty() {
                    blob.path = path.clone();
                }
                if blob.paths.len() < PATHS_PER_BLOB {
                    blob.paths.insert(path.clone());
                }
                blob.first = blob.first.min(idx);
                blob.last = blob.last.max(idx);
                let acc = found.paths.entry(path.clone()).or_insert_with(|| PathAcc {
                    blobs: HashSet::new(),
                    first: idx,
                    last: idx,
                });
                acc.blobs.insert(oid);
                acc.first = acc.first.min(idx);
                acc.last = acc.last.max(idx);
            }
        } else if let Some(hex) = text.strip_prefix('\u{1}') {
            if let Some(oid) = Oid::from_hex(hex.trim()) {
                // `-m` prints a merge once per parent.
                if found.commits.last() != Some(&oid) {
                    found.commits.push(oid);
                    let n = found.commits.len() as u64;
                    if n.is_multiple_of(256) {
                        tick(RewritePhase::Walking, n, total)?;
                    }
                }
            }
        } else if let Some(meta) = text.strip_prefix(':') {
            let mut f = meta.split_whitespace();
            if let (Some(mo), Some(mn), Some(old), Some(new)) =
                (f.next(), f.next(), f.next(), f.next())
            {
                pending = Some((
                    mo.to_string(),
                    mn.to_string(),
                    old.to_string(),
                    new.to_string(),
                ));
            }
        }
    }
    let status = child.0.wait().map_err(io)?;
    if !status.success() {
        return Err(failed_with(&args, status.code(), "git log failed"));
    }
    tick(RewritePhase::Walking, total, total)?;
    Ok(found)
}

fn run(root: &Path, scope: &Scope, top: u32, tick: Tick<'_>) -> Result<SizeAnalysis> {
    let top = top.clamp(1, MAX_TOP) as usize;
    tick(RewritePhase::Walking, 0, 0)?;
    let head = walk::head_of(root)?;
    let all_refs = refs::list(root)?;
    let resolved = refs::resolve(scope, &ScopeOptions::default(), &all_refs, &head)?;
    let order = walk::rev_list(root, &resolved.tips, &resolved.not)?;
    if order.is_empty() {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            "the scope has no commit",
        ));
    }
    let total = order.len() as u64;
    let in_head = head_files(root);
    let found = changes(root, &resolved.tips, &resolved.not, total, tick)?;

    let ids: Vec<Oid> = found.blobs.keys().copied().collect();
    let size_of = sizes(root, &ids, tick)?;

    let head_blobs: HashSet<Oid> = in_head.values().copied().collect();
    let mut head_dirs: HashSet<&str> = HashSet::new();
    for path in in_head.keys() {
        head_dirs.extend(dirs_of(path));
    }
    let commit_at = |i: usize| found.commits.get(i).copied().unwrap_or(order[0]);

    let mut analysis = SizeAnalysis {
        top: top as u32,
        commits: total,
        ..SizeAnalysis::default()
    };
    let mut rows: Vec<BlobRow> = Vec::new();
    let mut folders: HashMap<String, (u64, u64)> = HashMap::new();
    for (oid, acc) in &found.blobs {
        let Some(&(size, disk)) = size_of.get(oid) else {
            continue;
        };
        let state = if head_blobs.contains(oid) {
            BlobState::InHead
        } else if acc.paths.iter().any(|p| in_head.contains_key(p)) {
            BlobState::Replaced
        } else {
            BlobState::Deleted
        };
        analysis.blobs_total += 1;
        analysis.bytes_total += size;
        analysis.disk_total += disk;
        if state == BlobState::InHead {
            analysis.bytes_in_head += size;
        } else {
            analysis.bytes_history_only += size;
        }
        let mut dirs: HashSet<&str> = HashSet::new();
        for p in &acc.paths {
            dirs.extend(dirs_of(p));
        }
        for d in dirs {
            let e = folders.entry(d.to_string()).or_default();
            e.0 += size;
            e.1 += 1;
        }
        rows.push(BlobRow {
            path: acc.path.clone(),
            more_paths: acc.paths.len().saturating_sub(1) as u32,
            blob: *oid,
            size,
            disk_size: disk,
            first_commit: commit_at(acc.first),
            last_commit: commit_at(acc.last),
            state,
        });
    }
    rows.sort_by(|a, b| (b.size, &a.path, a.blob.0).cmp(&(a.size, &b.path, b.blob.0)));
    analysis.blobs_cut = rows.len().saturating_sub(top) as u64;
    rows.truncate(top);
    analysis.blobs = rows;

    let mut folder_rows: Vec<FolderRow> = folders
        .into_iter()
        .map(|(path, (size, blobs))| FolderRow {
            in_head: head_dirs.contains(path.as_str()),
            path,
            size,
            blobs,
        })
        .collect();
    folder_rows.sort_by(|a, b| (b.size, &a.path).cmp(&(a.size, &b.path)));
    analysis.folders_cut = folder_rows.len().saturating_sub(top) as u64;
    folder_rows.truncate(top);
    analysis.folders = folder_rows;

    let mut deleted: Vec<DeletedRow> = Vec::new();
    for (path, acc) in &found.paths {
        if in_head.contains_key(path) {
            continue;
        }
        let known: Vec<u64> = acc
            .blobs
            .iter()
            .filter_map(|b| size_of.get(b).map(|s| s.0))
            .collect();
        deleted.push(DeletedRow {
            path: path.clone(),
            versions: known.len() as u64,
            size: known.iter().sum(),
            first_commit: commit_at(acc.first),
            last_commit: commit_at(acc.last),
        });
    }
    deleted.retain(|d| d.versions > 0);
    deleted.sort_by(|a, b| (b.size, &a.path).cmp(&(a.size, &b.path)));
    analysis.deleted_paths = deleted.len() as u64;
    analysis.deleted_cut = deleted.len().saturating_sub(top) as u64;
    deleted.truncate(top);
    analysis.deleted = deleted;
    Ok(analysis)
}

/// `rewrite_analyze`.
pub(crate) fn analyze(
    repo: &RepoRef,
    scope: &Scope,
    top: u32,
    sink: RewriteProgressSink<'_>,
) -> Result<SizeAnalysis> {
    let mut tick = |phase, done, total| match sink(RewriteProgress { phase, done, total }) {
        std::ops::ControlFlow::Continue(()) => Ok(()),
        std::ops::ControlFlow::Break(()) => Err(canceled()),
    };
    run(&repo.root, scope, top, &mut tick)
}
