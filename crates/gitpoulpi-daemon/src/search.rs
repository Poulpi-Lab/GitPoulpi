//! The workspace search's per-repository index (the workspaces design).
//!
//! Two halves, because they cost differently:
//!
//! - **Small items** (branches, remote branches, tags, worktrees, stashes):
//!   hundreds at most. Rebuilt from the existing reads whenever the ref
//!   fingerprint (`cache::fingerprint`) moved, in the request itself.
//! - **Commit messages**: tens of thousands, up to `search.commit_limit`.
//!   Built from `GitBackend::commit_messages` and kept in one arena;
//!   brought up to date *incrementally* (only commits not reachable from the
//!   tips already indexed), unless a tip vanished (a rewrite), which
//!   replaces everything.
//!
//! **Never in the way of a mutation.** The commit half is built by a
//! background thread that holds *no* repository lock (a read of the object
//! database is safe beside a ref update), so the repository's fair
//! reader-writer lock never queues a mutation behind a multi-second build.
//! A search that finds the commits stale answers at once from what is
//! indexed and says `partial`; `wait` (the one-shot CLI) builds inline
//! instead. A mutation's `ReadCache::invalidate` marks the index stale.
//!
//! **Warm queries** do no I/O: one stat walk for the fingerprint, then a
//! scan of the lower-cased subjects (`fuzzy::score_lower`), the best hits
//! selected without sorting the rest. Budget (perf-budgets.md): under 15 ms
//! per repository of 50k commits.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use gitpoulpi_core::domain::{
    parse_query, CommitMessagesQuery, Oid, RefKind, RepoId, RepoRef, SearchHit, SearchHits,
    SearchKind, SearchRequest, SearchTarget,
};
use gitpoulpi_core::fuzzy::{self, Anchor};
use gitpoulpi_core::GitBackend;

/// Hits returned when the request names no limit.
const DEFAULT_LIMIT: usize = 50;
/// Commit-body matches rank under every subject match of the three tiers.
const BODY_BASE: i32 = 5_000;
/// A body is only searched for a query this long: two letters are in every
/// body.
const BODY_MIN_QUERY: usize = 3;
/// Bytes of a body kept for matching (lower-cased).
const BODY_KEEP: usize = 512;
/// A secondary text (a worktree's path, behind its folder name) ranks lower.
const SECONDARY_PENALTY: i32 = 3_000;

/// `search.commit_limit` as the daemon last read it.
mod settings {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::SystemTime;

    type Stamp = (Option<PathBuf>, Option<(SystemTime, u64)>, Option<String>);
    static CACHE: Mutex<Option<(Stamp, u32)>> = Mutex::new(None);

    /// Reading the TOML per keystroke would cost more than the search: the
    /// answer is kept until the file's stamp or the environment variable
    /// moves (a change in Settings still applies at once).
    pub fn commit_limit() -> u32 {
        let file = gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_file());
        let stamp = file
            .as_ref()
            .and_then(|f| std::fs::metadata(f).ok())
            .and_then(|m| Some((m.modified().ok()?, m.len())));
        let key = (
            file,
            stamp,
            std::env::var("GITPOULPI_SEARCH_COMMIT_LIMIT").ok(),
        );
        let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((cached, limit)) = cache.as_ref() {
            if *cached == key {
                return *limit;
            }
        }
        let limit = gitpoulpi_config::Settings::resolve(
            gitpoulpi_config::ConfigPaths::discover().as_ref(),
            &gitpoulpi_config::Overrides::default(),
        )
        .map(|s| s.config().search.commit_limit)
        .unwrap_or(gitpoulpi_config::DEFAULT_SEARCH_COMMIT_LIMIT);
        *cache = Some((key, limit));
        limit
    }
}

/// One branch, tag, worktree or stash.
struct Small {
    kind: SearchKind,
    label: String,
    detail: String,
    target: SearchTarget,
    lower: String,
    /// A second text the item is also found by (empty: none).
    lower2: String,
}

#[derive(Default)]
struct SmallItems {
    /// The ref fingerprint these were built at (`None`: too fresh to trust,
    /// so rebuilt next time).
    fingerprint: Option<u64>,
    built: bool,
    items: Vec<Small>,
    /// A hash of the ref targets (what the commits index must cover).
    refs_sig: u64,
}

/// Commit messages in one arena: the search scans `low` only.
#[derive(Default)]
struct CommitIndex {
    ids: Vec<Oid>,
    times: Vec<i64>,
    authors: Vec<u32>,
    /// `(start, len)` into `text` (the subject as written) and into `low`
    /// (lower-cased) and `body_low`.
    subject: Vec<(u32, u32)>,
    low_span: Vec<(u32, u32)>,
    body_span: Vec<(u32, u32)>,
    text: String,
    low: String,
    body_low: String,
    author_names: Vec<String>,
    author_ids: HashMap<String, u32>,
    /// The tips the commits are complete for (`commit_messages().tips`).
    tips: Vec<Oid>,
    /// The fingerprint and ref hash they were built at.
    built_fp: Option<u64>,
    built_sig: u64,
    /// The first build or the last update happened (false: never).
    built: bool,
    /// The limit cut the walk short.
    truncated: bool,
}

impl CommitIndex {
    fn len(&self) -> usize {
        self.ids.len()
    }

    fn push(&mut self, c: gitpoulpi_core::domain::CommitMessage) {
        let author = match self.author_ids.get(&c.author) {
            Some(i) => *i,
            None => {
                let i = self.author_names.len() as u32;
                self.author_names.push(c.author.clone());
                self.author_ids.insert(c.author, i);
                i
            }
        };
        self.ids.push(c.id);
        self.times.push(c.time);
        self.authors.push(author);
        let at = self.text.len() as u32;
        self.text.push_str(&c.subject);
        self.subject.push((at, c.subject.len() as u32));
        let at = self.low.len() as u32;
        let lower = c.subject.to_lowercase();
        self.low.push_str(&lower);
        self.low_span.push((at, lower.len() as u32));
        let at = self.body_low.len() as u32;
        let body = c.body.to_lowercase();
        let keep = floor_char_boundary(&body, BODY_KEEP);
        self.body_low.push_str(&body[..keep]);
        self.body_span.push((at, keep as u32));
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn low_of(&self, i: usize) -> &str {
        let (at, len) = self.low_span[i];
        &self.low[at as usize..(at + len) as usize]
    }

    fn subject_of(&self, i: usize) -> &str {
        let (at, len) = self.subject[i];
        &self.text[at as usize..(at + len) as usize]
    }

    fn body_of(&self, i: usize) -> &str {
        let (at, len) = self.body_span[i];
        &self.body_low[at as usize..(at + len) as usize]
    }
}

fn floor_char_boundary(s: &str, max: usize) -> usize {
    let mut i = max.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// One repository's index.
pub struct RepoIndex {
    small: Mutex<SmallItems>,
    commits: RwLock<CommitIndex>,
    /// Serializes commit builds (one at a time per repository).
    build: Mutex<()>,
    /// A background build is queued or running.
    building: AtomicBool,
    /// A mutation may have moved refs: recheck before trusting the index.
    stale: AtomicBool,
    /// The last build failed (the search says `partial`).
    failed: AtomicBool,
}

impl RepoIndex {
    fn new() -> Self {
        Self {
            small: Mutex::new(SmallItems::default()),
            commits: RwLock::new(CommitIndex::default()),
            build: Mutex::new(()),
            building: AtomicBool::new(false),
            stale: AtomicBool::new(true),
            failed: AtomicBool::new(false),
        }
    }
}

/// Every repository's index, kept in the daemon's `ReadCache`.
#[derive(Default)]
pub struct SearchIndexes {
    map: Mutex<HashMap<RepoId, Arc<RepoIndex>>>,
}

/// Indexes kept in all; the oldest are dropped past this (a window rarely
/// searches more repositories than it has tabs).
const MAX_REPOS: usize = 64;

impl SearchIndexes {
    fn get(&self, repo: &RepoId) -> Arc<RepoIndex> {
        let mut map = self.map.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() >= MAX_REPOS && !map.contains_key(repo) {
            map.clear();
        }
        Arc::clone(
            map.entry(repo.clone())
                .or_insert_with(|| Arc::new(RepoIndex::new())),
        )
    }

    /// A mutation may have moved refs, HEAD or the stash: recheck next time.
    pub fn invalidate(&self, repo: &RepoId) {
        let map = self.map.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(idx) = map.get(repo) {
            idx.stale.store(true, Ordering::Release);
        }
    }
}

fn sig_of(oids: impl Iterator<Item = Oid>) -> u64 {
    let mut sorted: Vec<Oid> = oids.collect();
    sorted.sort_by_key(|o| o.0);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for o in sorted {
        o.0.hash(&mut h);
    }
    h.finish()
}

/// Rebuilds the small items when the refs moved. Returns the fingerprint it
/// is current for.
fn refresh_small(idx: &RepoIndex, backend: &dyn GitBackend, repo: &RepoRef) -> (Option<u64>, u64) {
    let fingerprint = crate::cache::fingerprint_of(&repo.root);
    let mut small = idx.small.lock().unwrap_or_else(|e| e.into_inner());
    let stale = idx.stale.swap(false, Ordering::AcqRel);
    if small.built && !stale && fingerprint.is_some() && small.fingerprint == fingerprint {
        return (fingerprint, small.refs_sig);
    }
    let mut items: Vec<Small> = Vec::new();
    let mut targets: Vec<Oid> = Vec::new();
    // A failed read leaves its kind out: a repository that cannot list
    // stashes still has branches to find.
    if let Ok(refs) = backend.refs(repo) {
        for r in refs {
            targets.push(r.target);
            let short = r.target.to_hex()[..7].to_string();
            let (kind, label, full, detail) = match &r.kind {
                RefKind::LocalBranch { name } => (
                    SearchKind::Branch,
                    name.clone(),
                    format!("refs/heads/{name}"),
                    match (&r.upstream, r.is_head) {
                        (Some(up), true) => format!("HEAD, tracks {up}"),
                        (Some(up), false) => format!("tracks {up}"),
                        (None, true) => "HEAD".to_string(),
                        (None, false) => short,
                    },
                ),
                RefKind::RemoteBranch { remote, name } => (
                    SearchKind::Branch,
                    format!("{remote}/{name}"),
                    format!("refs/remotes/{remote}/{name}"),
                    "remote".to_string(),
                ),
                RefKind::Tag { name } => (
                    SearchKind::Tag,
                    name.clone(),
                    format!("refs/tags/{name}"),
                    short,
                ),
            };
            items.push(Small {
                kind,
                lower: label.to_lowercase(),
                lower2: String::new(),
                label,
                detail,
                target: SearchTarget::Ref { name: full },
            });
        }
    }
    if let Ok(stashes) = backend.stashes(repo) {
        for s in stashes {
            let label = if s.message.trim().is_empty() {
                format!("stash@{{{}}}", s.id.0)
            } else {
                s.message.clone()
            };
            items.push(Small {
                kind: SearchKind::Stash,
                lower: label.to_lowercase(),
                lower2: String::new(),
                detail: format!("stash@{{{}}} on {}", s.id.0, s.branch_at_creation),
                target: SearchTarget::Stash {
                    index: s.id.0,
                    id: s.commit,
                },
                label,
            });
        }
    }
    if let Ok(worktrees) = backend.worktrees(repo) {
        for w in worktrees.iter().filter(|w| !w.is_main) {
            let path = w.path.to_string_lossy().into_owned();
            let name = w
                .path
                .file_name()
                .map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
            let branch = w.branch.clone().unwrap_or_default();
            items.push(Small {
                kind: SearchKind::Worktree,
                lower: name.to_lowercase(),
                lower2: format!("{path} {branch}").to_lowercase(),
                detail: if branch.is_empty() {
                    format!("{path} (detached)")
                } else {
                    format!("{path} ({branch})")
                },
                target: SearchTarget::Worktree { path },
                label: name,
            });
        }
    }
    let refs_sig = sig_of(targets.into_iter());
    *small = SmallItems {
        fingerprint,
        built: true,
        items,
        refs_sig,
    };
    (fingerprint, refs_sig)
}

/// Brings the commit index up to date. Serialized per repository; the
/// backend is read without any lock on the index, which is only written for
/// the moment it takes to append.
fn update_commits(
    idx: &RepoIndex,
    backend: &dyn GitBackend,
    repo: &RepoRef,
    limit: usize,
    fingerprint: Option<u64>,
    sig: u64,
) {
    let _one_at_a_time = idx.build.lock().unwrap_or_else(|e| e.into_inner());
    let known = {
        let c = idx.commits.read().unwrap_or_else(|e| e.into_inner());
        if c.built && is_fresh(&c, fingerprint, sig) {
            return;
        }
        c.tips.clone()
    };
    let mut query = CommitMessagesQuery {
        known_tips: known.clone(),
        limit,
    };
    let mut answer = backend.commit_messages(repo, &query);
    // More than the limit would be kept: start over, newest `limit` only.
    if let Ok(a) = &answer {
        let have = idx.commits.read().unwrap_or_else(|e| e.into_inner()).len();
        if !a.replace && !known.is_empty() && have + a.commits.len() > limit {
            query.known_tips = Vec::new();
            answer = backend.commit_messages(repo, &query);
        }
    }
    match answer {
        Ok(a) => {
            let mut c = idx.commits.write().unwrap_or_else(|e| e.into_inner());
            if a.replace || query.known_tips.is_empty() {
                c.clear();
            }
            for commit in a.commits {
                c.push(commit);
            }
            c.tips = a.tips;
            c.built = true;
            c.built_fp = fingerprint;
            c.built_sig = sig;
            if a.replace || query.known_tips.is_empty() {
                c.truncated = a.truncated;
            }
            idx.failed.store(false, Ordering::Release);
        }
        Err(_) => idx.failed.store(true, Ordering::Release),
    }
}

/// Whether the commits cover what the refs are now.
fn is_fresh(c: &CommitIndex, fingerprint: Option<u64>, sig: u64) -> bool {
    match fingerprint {
        Some(fp) => c.built_fp == Some(fp),
        None => c.built_fp.is_none() && c.built_sig == sig,
    }
}

/// The reply for a `Search` request: the hits as the response value, or the
/// error of a malformed request. Called from the read path with the daemon's
/// backend handle (the commit half is built on a thread that outlives the
/// request).
pub(crate) fn reply(
    cache: &crate::cache::ReadCache,
    backend: &Arc<dyn GitBackend>,
    request: &gitpoulpi_ipc::Request,
) -> crate::dispatch::CachedReply {
    use crate::dispatch::{err, ok, CachedReply};
    let dispatched = match (
        crate::dispatch::repo_ref(request),
        serde_json::from_value::<SearchRequest>(request.args.clone()),
    ) {
        (Err(e), _) => err(request.id, e, false),
        (_, Err(e)) => err(request.id, format!("invalid search request: {e}"), false),
        (Ok(repo), Ok(search)) => {
            let hits = answer(
                &cache.search,
                backend,
                &repo,
                &search,
                settings::commit_limit() as usize,
            );
            match serde_json::to_value(hits) {
                Ok(v) => ok(request.id, v, false),
                Err(e) => err(request.id, format!("failed to encode result: {e}"), false),
            }
        }
    };
    CachedReply::Dispatched(Box::new(dispatched))
}

/// Answers a `Search` request.
pub fn answer(
    indexes: &SearchIndexes,
    backend: &Arc<dyn GitBackend>,
    repo: &RepoRef,
    request: &SearchRequest,
    commit_limit: usize,
) -> SearchHits {
    let idx = indexes.get(&repo.id);
    let limit = if request.limit == 0 {
        DEFAULT_LIMIT
    } else {
        request.limit
    };
    let parsed = parse_query(&request.query);
    // A daemon answers repository data only: `#` and the hosting kinds are
    // the caller's (it reads the hosting caches).
    let kinds: Vec<SearchKind> = match parsed.kind {
        Some(k) => vec![k],
        None if parsed.hosting_only => Vec::new(),
        None if request.kinds.is_empty() => SearchKind::ALL.to_vec(),
        None => request.kinds.clone(),
    };
    let query = parsed.text.to_lowercase();

    let (fingerprint, sig) = refresh_small(&idx, backend.as_ref(), repo);

    // The commit half: inline for a caller that waits, else in the background.

    let wants_commits = kinds.contains(&SearchKind::Commit) && commit_limit > 0;
    let mut partial = false;
    if wants_commits {
        let fresh = {
            let c = idx.commits.read().unwrap_or_else(|e| e.into_inner());
            c.built && is_fresh(&c, fingerprint, sig)
        };
        if !fresh {
            if request.wait {
                update_commits(&idx, backend.as_ref(), repo, commit_limit, fingerprint, sig);
            } else if !idx.building.swap(true, Ordering::AcqRel) {
                let (idx, backend, repo) = (Arc::clone(&idx), Arc::clone(backend), repo.clone());
                std::thread::spawn(move || {
                    update_commits(
                        &idx,
                        backend.as_ref(),
                        &repo,
                        commit_limit,
                        fingerprint,
                        sig,
                    );
                    idx.building.store(false, Ordering::Release);
                });
            }
            // Built before but behind the refs: answered from what is indexed,
            // topped up in the background; not "partial" (that is: never
            // built, cut at the limit, or the last build failed).
            partial = idx.failed.load(Ordering::Acquire);
        }
        let c = idx.commits.read().unwrap_or_else(|e| e.into_inner());
        partial |= c.truncated || !c.built;
    }

    let mut hits: Vec<SearchHit> = Vec::new();
    {
        let small = idx.small.lock().unwrap_or_else(|e| e.into_inner());
        for item in small.items.iter().filter(|i| kinds.contains(&i.kind)) {
            let primary = fuzzy::matched(&item.lower, &query, Anchor::Whole);
            let secondary = (!item.lower2.is_empty())
                .then(|| fuzzy::score_lower(&item.lower2, &query, Anchor::Whole))
                .flatten()
                .map(|s| s - SECONDARY_PENALTY);
            let (score, positions) = match (primary, secondary) {
                (Some(m), Some(s)) if m.score >= s => (m.score, m.positions),
                (Some(_), Some(s)) | (None, Some(s)) => (s, Vec::new()),
                (Some(m), None) => (m.score, m.positions),
                (None, None) => continue,
            };
            hits.push(SearchHit {
                kind: item.kind,
                label: item.label.clone(),
                detail: item.detail.clone(),
                target: item.target.clone(),
                score,
                positions,
                repo: None,
            });
        }
    }
    if wants_commits {
        let c = idx.commits.read().unwrap_or_else(|e| e.into_inner());
        hits.extend(commit_hits(&c, &query, limit));
    }

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.label.cmp(&b.label))
    });
    hits.truncate(limit);
    SearchHits { hits, partial }
}

/// The best `limit` commits for `query` (lower-cased), best first, newer
/// first among equals. No allocation per non-matching commit.
fn commit_hits(c: &CommitIndex, query: &str, limit: usize) -> Vec<SearchHit> {
    let n = c.len();
    let mut found: Vec<(i32, u32)> = Vec::new();
    if query.is_empty() {
        // The newest commits, in the order they were indexed.
        found.extend((0..n.min(limit)).map(|i| (0, i as u32)));
    } else {
        let body_too = query.len() >= BODY_MIN_QUERY;
        for i in 0..n {
            if let Some(score) = fuzzy::score_lower(c.low_of(i), query, Anchor::Whole) {
                found.push((score, i as u32));
            } else if body_too {
                if let Some(pos) = c.body_of(i).find(query) {
                    found.push((BODY_BASE - pos.min(1_000) as i32, i as u32));
                }
            }
        }
    }
    let order = |a: &(i32, u32), b: &(i32, u32)| {
        b.0.cmp(&a.0)
            .then_with(|| c.times[b.1 as usize].cmp(&c.times[a.1 as usize]))
    };
    if found.len() > limit {
        found.select_nth_unstable_by(limit, order);
        found.truncate(limit);
    }
    found.sort_by(order);
    found
        .into_iter()
        .map(|(score, i)| {
            let i = i as usize;
            let subject = c.subject_of(i);
            // Positions only when the subject itself matched.
            let positions = if query.is_empty() {
                Vec::new()
            } else {
                fuzzy::matched(c.low_of(i), query, Anchor::Whole)
                    .map_or_else(Vec::new, |m| m.positions)
            };
            let id = c.ids[i];
            SearchHit {
                kind: SearchKind::Commit,
                label: subject.to_string(),
                detail: format!(
                    "{} {}",
                    &id.to_hex()[..7],
                    c.author_names[c.authors[i] as usize]
                ),
                target: SearchTarget::Commit { id },
                score,
                positions,
                repo: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
