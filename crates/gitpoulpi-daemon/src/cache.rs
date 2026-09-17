//! A per-repo cache of the two expensive idle reads, `Log` and `Refs`.
//!
//! The TUI re-reads both on every 2 s poll; on a 5,000-commit repo the log
//! alone is 0.3-2 s of libgit2 work and 1.5 MB of JSON. Both answers are a
//! pure function of the refs plus, for `Refs`, the repo config, so each
//! entry is keyed by a *fingerprint* of `HEAD`, `packed-refs`, `config` and
//! every loose ref under `refs/`, hashed from their `(path, mtime, len)`
//! (a directory walk and one `stat` per loose ref, no reads).
//!
//! Two safety nets keep a stale answer from living long:
//! * a mutation whose scope touches refs, the log or the config calls
//!   [`ReadCache::invalidate`] (see `server.rs`), so the daemon's own
//!   writes never depend on timestamp granularity;
//! * the git "racy" rule: when any fingerprinted file changed within the
//!   last [`RACY_WINDOW`], a result is neither served from nor stored in
//!   the cache, because a second write in the same clock tick would leave
//!   the fingerprint unchanged (ref files are always the same length).
//!
//! A repo whose git dir cannot be located is never cached.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use gitpoulpi_core::domain::{CommitStat, Oid, OperationKind, RepoId};
use gitpoulpi_core::paths;
use serde_json::Value;

#[cfg(test)]
mod tests;

/// Files modified more recently than this are not trusted as a cache key.
pub const RACY_WINDOW: Duration = Duration::from_secs(1);

/// Entries kept in all; the cache is cleared when it grows past this.
const MAX_ENTRIES: usize = 64;

/// The answer is kept as its serialized JSON, shared: a hit hands out the
/// same bytes without cloning a `Value` tree (1.5 MB for a 5,000-commit
/// log) and the server writes them to the wire without serializing again.
struct Entry {
    fingerprint: u64,
    json: Arc<[u8]>,
    /// `gitpoulpi_ipc::etag::of(json)`, for conditional reads.
    etag: u64,
}

/// What a conditional read produced.
pub enum Cached {
    /// The serialized answer.
    Json(Arc<[u8]>),
    /// The answer is what the client's etag names: send nothing more.
    NotModified,
}

/// Which read this is: the command plus its (compact) arguments.
pub type ReadKey = (OperationKind, String);

type Key = (RepoId, ReadKey);

pub struct ReadCache {
    entries: Mutex<HashMap<Key, Entry>>,
    racy_window: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
    /// Per-commit change counts (the Log "Changes" column). Commits are
    /// immutable, so these never need invalidating; only memory is bounded.
    stats: Mutex<HashMap<(RepoId, Oid), CommitStat>>,
    /// The workspace search's per-repository indexes (`search.rs`).
    pub(crate) search: crate::search::SearchIndexes,
}

/// Commit stats kept in all (~40 bytes each); the map is cleared past this.
const MAX_STATS: usize = 50_000;

impl Default for ReadCache {
    fn default() -> Self {
        Self::with_racy_window(RACY_WINDOW)
    }
}

impl ReadCache {
    pub fn with_racy_window(racy_window: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            racy_window,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            stats: Mutex::new(HashMap::new()),
            search: crate::search::SearchIndexes::default(),
        }
    }

    /// Answers served from the cache so far.
    #[cfg(test)]
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Cacheable reads that had to run the backend.
    #[cfg(test)]
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Splits `ids` into the stats already known and the ids still to ask
    /// the backend for (de-duplicated, in first-seen order).
    pub fn stats_lookup(&self, repo: &RepoId, ids: &[Oid]) -> (Vec<(Oid, CommitStat)>, Vec<Oid>) {
        let stats = self.stats.lock().unwrap();
        let mut known = Vec::new();
        let mut missing: Vec<Oid> = Vec::new();
        for id in ids {
            match stats.get(&(repo.clone(), *id)) {
                Some(s) => known.push((*id, *s)),
                None if !missing.contains(id) => missing.push(*id),
                None => {}
            }
        }
        (known, missing)
    }

    /// Remembers freshly computed stats.
    pub fn stats_store(&self, repo: &RepoId, fresh: &[CommitStat]) {
        let mut stats = self.stats.lock().unwrap();
        if stats.len() + fresh.len() > MAX_STATS {
            stats.clear();
        }
        for s in fresh {
            stats.insert((repo.clone(), s.id), *s);
        }
    }

    /// Drops everything cached for `repo`.
    pub fn invalidate(&self, repo: &RepoId) {
        self.entries.lock().unwrap().retain(|(r, _), _| r != repo);
        self.search.invalidate(repo);
    }

    /// Returns the cached JSON for `key` when the repo's fingerprint is
    /// unchanged, else runs `compute` and caches an `Ok` result. For a
    /// client that holds the answer whose etag is `if_none_match`: when the
    /// current answer has that etag (cached or freshly computed) the bytes
    /// are not handed out again.
    pub fn get_or_compute_if_none_match<E>(
        &self,
        repo: &RepoId,
        key: ReadKey,
        if_none_match: Option<u64>,
        compute: impl FnOnce() -> Result<Value, E>,
    ) -> Result<Cached, E> {
        let fingerprint = fingerprint(Path::new(&repo.0), self.racy_window);
        let key = (repo.clone(), key);
        if let Some(fp) = fingerprint {
            if let Some(entry) = self.entries.lock().unwrap().get(&key) {
                if entry.fingerprint == fp {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    if if_none_match == Some(entry.etag) {
                        return Ok(Cached::NotModified);
                    }
                    return Ok(Cached::Json(Arc::clone(&entry.json)));
                }
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let value = compute()?;
        // A `Value` always serializes; `null` is the unreachable fallback.
        let json: Arc<[u8]> = serde_json::to_vec(&value)
            .unwrap_or_else(|_| b"null".to_vec())
            .into();
        let etag = gitpoulpi_ipc::etag::of(&json);
        if let Some(fp) = fingerprint {
            let mut entries = self.entries.lock().unwrap();
            if entries.len() >= MAX_ENTRIES {
                entries.clear();
            }
            entries.insert(
                key,
                Entry {
                    fingerprint: fp,
                    json: Arc::clone(&json),
                    etag,
                },
            );
        }
        if if_none_match == Some(etag) {
            return Ok(Cached::NotModified);
        }
        Ok(Cached::Json(json))
    }
}

/// `(git dir, common dir)` of the repo at `root`: they differ for a linked
/// worktree, where `HEAD` is per worktree and the refs are shared.
fn git_dirs(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let git_dir = paths::git_dir(root).or_else(|| {
        (root.join("HEAD").is_file() && root.join("refs").is_dir()).then(|| root.to_path_buf())
    })?;
    let common = paths::common_dir(&git_dir);
    Some((git_dir, common))
}

fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

fn walk_refs(dir: &Path, rel: &str, out: &mut Vec<(String, SystemTime, u64)>) -> Option<()> {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(()),
        Err(_) => return None,
    };
    for entry in read {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".lock") {
            continue;
        }
        let rel = format!("{rel}/{name}");
        let Ok(meta) = entry.metadata() else {
            // Deleted between listing and stat: the next call sees it gone.
            continue;
        };
        if meta.is_dir() {
            walk_refs(&entry.path(), &rel, out)?;
        } else {
            out.push((rel, meta.modified().ok()?, meta.len()));
        }
    }
    Some(())
}

/// [`fingerprint`] with the standard racy window: what the search index
/// keys on.
pub(crate) fn fingerprint_of(root: &Path) -> Option<u64> {
    fingerprint(root, RACY_WINDOW)
}

/// A hash of the ref and config files of the repo at `root`, or `None`
/// when it cannot be computed or is too fresh to trust (see the module
/// docs).
fn fingerprint(root: &Path, racy_window: Duration) -> Option<u64> {
    let (git_dir, common) = git_dirs(root)?;
    let mut files: Vec<(String, SystemTime, u64)> = Vec::new();
    let (t, l) = stamp(&git_dir.join("HEAD"))?;
    files.push(("HEAD".into(), t, l));
    for (name, base) in [
        ("packed-refs", &common),
        ("config", &common),
        ("config.worktree", &git_dir),
    ] {
        if let Some((t, l)) = stamp(&base.join(name)) {
            files.push((name.into(), t, l));
        }
    }
    walk_refs(&common.join("refs"), "refs", &mut files)?;
    // Reftable: the refs live in these table files (see the outside journal).
    walk_refs(&common.join("reftable"), "reftable", &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let now = SystemTime::now();
    let mut hasher = DefaultHasher::new();
    for (name, time, len) in &files {
        if now
            .duration_since(*time)
            .map_or(true, |age| age < racy_window)
        {
            return None;
        }
        name.hash(&mut hasher);
        time.hash(&mut hasher);
        len.hash(&mut hasher);
    }
    Some(hasher.finish())
}
