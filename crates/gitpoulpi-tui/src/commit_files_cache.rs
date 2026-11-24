//! The selected commit's changed files, fast.
//!
//! Two pieces of pure bookkeeping the daemon link's commit-files worker
//! (`daemon_link::commit_files_worker`) is built from, kept here so they
//! can be tested without a daemon:
//!
//! - [`CommitFilesCache`], an LRU of answers keyed by commit id. A commit is
//!   immutable, so what it changed never goes stale: coming back to a commit
//!   (the second j/k pass, closing a diff, a refresh that rebuilt the Log) is
//!   answered without a round trip.
//! - [`CommitFilesQueue`], what is still to fetch, with stale requests
//!   already thrown away: holding j over twenty rows keeps only the latest
//!   selection, so the row the key is let go on is answered first.

use std::collections::VecDeque;
use std::path::PathBuf;

use gitpoulpi_core::domain::{FileDiff, Oid};

/// How many commits the cache holds at most.
pub(crate) const CACHE_ENTRIES: usize = 128;

/// Roughly how many bytes of diff text the cache holds at most. One
/// enormous commit (a vendored dependency, a generated file) must not
/// pin hundreds of megabytes; a commit heavier than this on its own is
/// simply not cached, and costs what it cost before.
pub(crate) const CACHE_BYTES: usize = 48 * 1024 * 1024;

/// One cached answer: what the commit changed, and its whole tree once
/// something has asked for it (`View all files`).
#[derive(Debug, Clone, Default)]
struct Entry {
    files: Option<Vec<FileDiff>>,
    /// `files` is a list without hunks: the Diff view asks for
    /// the open file's hunks alone.
    lazy: bool,
    tree: Option<Vec<PathBuf>>,
    weight: usize,
}

/// A least-recently-used cache of commit-files answers, keyed by commit.
///
/// Only successes are stored: an error (a transient daemon hiccup, an oid a
/// refresh has since made unreachable) must be asked again next time.
#[derive(Debug)]
pub(crate) struct CommitFilesCache {
    /// Most recently used last.
    order: VecDeque<Oid>,
    entries: std::collections::HashMap<Oid, Entry>,
    bytes: usize,
    max_entries: usize,
    max_bytes: usize,
    /// The LFS paths of a revision's tree (`View all files`); small, and
    /// bounded by `max_entries` like the rest.
    lfs: std::collections::HashMap<Oid, std::sync::Arc<std::collections::HashSet<PathBuf>>>,
}

impl Default for CommitFilesCache {
    fn default() -> Self {
        Self::with_limits(CACHE_ENTRIES, CACHE_BYTES)
    }
}

/// A rough byte count for a list of file diffs - the text dominates.
fn files_weight(files: &[FileDiff]) -> usize {
    files
        .iter()
        .map(|f| {
            64 + f.path.as_os_str().len()
                + f.hunks
                    .iter()
                    .map(|h| 32 + h.lines.iter().map(|l| 24 + l.content.len()).sum::<usize>())
                    .sum::<usize>()
        })
        .sum()
}

fn tree_weight(tree: &[PathBuf]) -> usize {
    tree.iter().map(|p| 24 + p.as_os_str().len()).sum()
}

impl CommitFilesCache {
    pub(crate) fn with_limits(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            order: VecDeque::new(),
            entries: Default::default(),
            bytes: 0,
            max_entries,
            max_bytes,
            lfs: Default::default(),
        }
    }

    /// The LFS paths held for `commit`'s tree.
    pub(crate) fn lfs(
        &self,
        commit: Oid,
    ) -> Option<std::sync::Arc<std::collections::HashSet<PathBuf>>> {
        self.lfs.get(&commit).cloned()
    }

    pub(crate) fn insert_lfs(
        &mut self,
        commit: Oid,
        set: std::sync::Arc<std::collections::HashSet<PathBuf>>,
    ) {
        if self.lfs.len() >= self.max_entries.max(1) {
            self.lfs.clear();
        }
        self.lfs.insert(commit, set);
    }

    /// The changed files of `commit` (and its tree when `with_tree`), if
    /// everything asked for is held. Marks the entry recently used.
    pub(crate) fn get(
        &mut self,
        commit: Oid,
        with_tree: bool,
    ) -> Option<(Vec<FileDiff>, Option<Vec<PathBuf>>)> {
        let entry = self.entries.get(&commit)?;
        let files = entry.files.clone()?;
        let tree = match (with_tree, &entry.tree) {
            (false, _) => None,
            (true, Some(tree)) => Some(tree.clone()),
            (true, None) => return None,
        };
        self.touch(commit);
        Some((files, tree))
    }

    /// Whether the held files of `commit` are a list without hunks.
    pub(crate) fn is_lazy(&self, commit: Oid) -> bool {
        self.entries
            .get(&commit)
            .is_some_and(|entry| entry.files.is_some() && entry.lazy)
    }

    /// Stores a hunk-less list for `commit`, unless the whole
    /// diff is already held.
    pub(crate) fn insert_list(&mut self, commit: Oid, files: Vec<FileDiff>) {
        if self.has_files(commit) && !self.is_lazy(commit) {
            return;
        }
        self.insert(commit, Some(files), None);
        if let Some(entry) = self.entries.get_mut(&commit) {
            entry.lazy = true;
        }
    }

    /// Whether `commit`'s changed files are held (its tree may not be).
    pub(crate) fn has_files(&self, commit: Oid) -> bool {
        self.entries
            .get(&commit)
            .is_some_and(|entry| entry.files.is_some())
    }

    /// The changed files alone, when held - what a request that also
    /// wants the tree can reuse while it fetches only the tree.
    pub(crate) fn files(&mut self, commit: Oid) -> Option<Vec<FileDiff>> {
        let files = self.entries.get(&commit)?.files.clone()?;
        self.touch(commit);
        Some(files)
    }

    /// Stores what was fetched for `commit`. `None` for either half keeps
    /// what is already held for it.
    pub(crate) fn insert(
        &mut self,
        commit: Oid,
        files: Option<Vec<FileDiff>>,
        tree: Option<Vec<PathBuf>>,
    ) {
        let mut entry = self.remove(commit).unwrap_or_default();
        if let Some(files) = files {
            entry.files = Some(files);
            entry.lazy = false;
        }
        if let Some(tree) = tree {
            entry.tree = Some(tree);
        }
        entry.weight = entry.files.as_deref().map(files_weight).unwrap_or(0)
            + entry.tree.as_deref().map(tree_weight).unwrap_or(0);
        if entry.weight > self.max_bytes || self.max_entries == 0 {
            return;
        }
        self.bytes += entry.weight;
        self.entries.insert(commit, entry);
        self.order.push_back(commit);
        while self.order.len() > self.max_entries || self.bytes > self.max_bytes {
            let Some(oldest) = self.order.front().copied() else {
                break;
            };
            self.remove(oldest);
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    fn touch(&mut self, commit: Oid) {
        if let Some(at) = self.order.iter().position(|c| *c == commit) {
            self.order.remove(at);
            self.order.push_back(commit);
        }
    }

    fn remove(&mut self, commit: Oid) -> Option<Entry> {
        let entry = self.entries.remove(&commit)?;
        self.bytes -= entry.weight;
        if let Some(at) = self.order.iter().position(|c| *c == commit) {
            self.order.remove(at);
        }
        Some(entry)
    }
}

/// Who asked for a commit's files, which decides what a newer request
/// does to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitFilesPurpose {
    /// The Log selection (or a stash row). Only the newest one matters:
    /// a newer selection replaces it while it is still queued.
    Selection,
    /// The WIP row's `View all files` asking for HEAD's tree. Independent
    /// of the selection, so never dropped for one.
    Wip,
    /// Warming the cache for the selection's neighbors. Answered to
    /// nobody, and replaced wholesale by the next prefetch.
    Prefetch,
}

/// One commit-files fetch still to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuedFetch {
    pub commit: Oid,
    pub with_tree: bool,
    pub purpose: CommitFilesPurpose,
    /// When the render loop asked, for the `GITPOULPI_TIMING` log: the
    /// latency that matters is from the keypress, not from when the
    /// worker got round to it.
    pub sent: std::time::Instant,
}

impl QueuedFetch {
    fn same_work(&self, other: &QueuedFetch) -> bool {
        self.commit == other.commit && self.with_tree == other.with_tree
    }
}

/// The commit-files work still to do, stale requests already dropped.
///
/// Order of service: the selection first (it is what the user is looking
/// at), then the WIP tree, then prefetches.
#[derive(Debug, Default)]
pub(crate) struct CommitFilesQueue {
    selection: Option<QueuedFetch>,
    wip: VecDeque<QueuedFetch>,
    prefetch: VecDeque<QueuedFetch>,
    /// The comparison the Log's multi-select stands for: only the newest
    /// request matters, like the selection.
    compare: Option<(Oid, gitpoulpi_core::domain::DiffTarget)>,
}

impl CommitFilesQueue {
    pub(crate) fn push_compare(&mut self, id: Oid, target: gitpoulpi_core::domain::DiffTarget) {
        self.compare = Some((id, target));
    }

    pub(crate) fn pop_compare(&mut self) -> Option<(Oid, gitpoulpi_core::domain::DiffTarget)> {
        self.compare.take()
    }

    pub(crate) fn push(&mut self, fetch: QueuedFetch) {
        match fetch.purpose {
            // The one that makes scrolling never queue up work: an older
            // selection still waiting is simply forgotten.
            CommitFilesPurpose::Selection => self.selection = Some(fetch),
            CommitFilesPurpose::Wip => {
                if !self.wip.iter().any(|queued| queued.same_work(&fetch)) {
                    self.wip.push_back(fetch);
                }
            }
            CommitFilesPurpose::Prefetch => self.prefetch.push_back(fetch),
        }
    }

    /// Replaces every queued prefetch with `commits`: the old ones were the
    /// previous selection's neighbors, and the selection has moved.
    pub(crate) fn replace_prefetch(&mut self, commits: impl IntoIterator<Item = Oid>) {
        self.prefetch.clear();
        let now = std::time::Instant::now();
        for commit in commits {
            self.push(QueuedFetch {
                commit,
                with_tree: false,
                purpose: CommitFilesPurpose::Prefetch,
                sent: now,
            });
        }
    }

    /// The next fetch someone is waiting on: the selection, then the WIP
    /// tree. Prefetches are taken separately ([`Self::pop_prefetch`]),
    /// because the worker runs them beside the urgent ones rather than
    /// in front of them.
    pub(crate) fn pop_urgent(&mut self) -> Option<QueuedFetch> {
        self.selection.take().or_else(|| self.wip.pop_front())
    }

    pub(crate) fn pop_prefetch(&mut self) -> Option<QueuedFetch> {
        self.prefetch.pop_front()
    }

    /// Everything, most urgent first.
    #[cfg(test)]
    pub(crate) fn pop(&mut self) -> Option<QueuedFetch> {
        self.pop_urgent().or_else(|| self.pop_prefetch())
    }

    /// Whether a selection newer than the one being served is waiting: the
    /// worker's cue to stop spending time on the old one.
    #[cfg(test)]
    pub(crate) fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.selection.is_none() && self.wip.is_empty() && self.prefetch.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{DiffLine, FileStatus, Hunk, HunkId, LineOrigin};

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn files(path: &str, text_len: usize) -> Vec<FileDiff> {
        vec![FileDiff {
            renamed_from: None,
            similarity: None,
            path: PathBuf::from(path),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 1,
                lines: vec![DiffLine {
                    origin: LineOrigin::Add,
                    content: "x".repeat(text_len),
                    staged: false,
                }],
            }],
        }]
    }

    #[test]
    fn a_cached_commit_is_answered_with_exactly_what_was_stored() {
        let mut cache = CommitFilesCache::default();
        assert!(cache.get(oid(1), false).is_none());
        cache.insert(oid(1), Some(files("a.txt", 3)), None);
        let (got, tree) = cache.get(oid(1), false).expect("cached");
        assert_eq!(got, files("a.txt", 3));
        assert_eq!(tree, None);
    }

    #[test]
    fn a_request_for_the_tree_misses_until_the_tree_is_stored_but_keeps_the_files() {
        let mut cache = CommitFilesCache::default();
        cache.insert(oid(1), Some(files("a.txt", 3)), None);
        assert!(cache.get(oid(1), true).is_none(), "tree never fetched");
        assert_eq!(cache.files(oid(1)), Some(files("a.txt", 3)));
        cache.insert(oid(1), None, Some(vec![PathBuf::from("a.txt")]));
        let (got, tree) = cache.get(oid(1), true).expect("both halves held");
        assert_eq!(got, files("a.txt", 3), "adding the tree kept the files");
        assert_eq!(tree, Some(vec![PathBuf::from("a.txt")]));
    }

    #[test]
    fn the_least_recently_used_commit_is_evicted_first() {
        let mut cache = CommitFilesCache::with_limits(2, usize::MAX);
        cache.insert(oid(1), Some(files("a", 1)), None);
        cache.insert(oid(2), Some(files("b", 1)), None);
        // Reading 1 makes 2 the oldest.
        assert!(cache.get(oid(1), false).is_some());
        cache.insert(oid(3), Some(files("c", 1)), None);
        assert!(cache.has_files(oid(1)));
        assert!(!cache.has_files(oid(2)), "2 was least recently used");
        assert!(cache.has_files(oid(3)));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn the_byte_budget_evicts_and_a_commit_heavier_than_it_is_not_cached() {
        let one = files_weight(&files("a", 1000));
        let mut cache = CommitFilesCache::with_limits(100, one * 2);
        cache.insert(oid(1), Some(files("a", 1000)), None);
        cache.insert(oid(2), Some(files("a", 1000)), None);
        cache.insert(oid(3), Some(files("a", 1000)), None);
        assert!(!cache.has_files(oid(1)), "over budget: oldest goes");
        assert!(cache.has_files(oid(2)) && cache.has_files(oid(3)));

        cache.insert(oid(4), Some(files("a", 10_000)), None);
        assert!(!cache.has_files(oid(4)), "alone heavier than the budget");
        assert!(
            cache.has_files(oid(2)) && cache.has_files(oid(3)),
            "and it evicted nothing on the way"
        );
    }

    #[test]
    fn reinserting_a_commit_does_not_count_its_weight_twice() {
        let one = files_weight(&files("a", 100));
        let mut cache = CommitFilesCache::with_limits(100, one * 2);
        for _ in 0..10 {
            cache.insert(oid(1), Some(files("a", 100)), None);
        }
        cache.insert(oid(2), Some(files("a", 100)), None);
        assert!(cache.has_files(oid(1)) && cache.has_files(oid(2)));
    }

    /// One instant for every fetch a test builds, so they compare equal.
    fn t0() -> std::time::Instant {
        static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        *T0.get_or_init(std::time::Instant::now)
    }

    fn fetch(n: u8, purpose: CommitFilesPurpose) -> QueuedFetch {
        QueuedFetch {
            commit: oid(n),
            with_tree: false,
            purpose,
            sent: t0(),
        }
    }

    #[test]
    fn a_newer_selection_replaces_a_queued_one_so_scrolling_never_piles_up_work() {
        let mut queue = CommitFilesQueue::default();
        for n in 1..=20 {
            queue.push(fetch(n, CommitFilesPurpose::Selection));
        }
        assert_eq!(queue.pop(), Some(fetch(20, CommitFilesPurpose::Selection)));
        assert_eq!(queue.pop(), None, "the nineteen stale ones are gone");
        assert!(queue.is_empty());
    }

    #[test]
    fn the_selection_is_served_before_the_wip_tree_and_both_before_prefetches() {
        let mut queue = CommitFilesQueue::default();
        queue.replace_prefetch([oid(8), oid(9)]);
        queue.push(QueuedFetch {
            with_tree: true,
            ..fetch(5, CommitFilesPurpose::Wip)
        });
        queue.push(fetch(1, CommitFilesPurpose::Selection));
        assert_eq!(queue.pop().unwrap().commit, oid(1));
        let wip = queue.pop().unwrap();
        assert_eq!((wip.commit, wip.with_tree), (oid(5), true));
        assert_eq!(queue.pop().unwrap().commit, oid(8));
        assert_eq!(queue.pop().unwrap().commit, oid(9));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn a_selection_never_drops_the_wip_tree_request() {
        let mut queue = CommitFilesQueue::default();
        queue.push(fetch(5, CommitFilesPurpose::Wip));
        queue.push(fetch(5, CommitFilesPurpose::Wip));
        queue.push(fetch(1, CommitFilesPurpose::Selection));
        queue.push(fetch(2, CommitFilesPurpose::Selection));
        assert_eq!(queue.pop().unwrap().commit, oid(2));
        assert_eq!(queue.pop().unwrap().commit, oid(5));
        assert_eq!(queue.pop(), None, "the duplicate WIP request was merged");
    }

    #[test]
    fn urgent_work_never_hands_out_a_prefetch() {
        let mut queue = CommitFilesQueue::default();
        queue.replace_prefetch([oid(8)]);
        assert_eq!(queue.pop_urgent(), None, "a prefetch is not urgent");
        assert!(!queue.has_selection());
        queue.push(fetch(1, CommitFilesPurpose::Selection));
        assert!(queue.has_selection());
        assert_eq!(queue.pop_urgent().unwrap().commit, oid(1));
        assert_eq!(queue.pop_prefetch().unwrap().commit, oid(8));
        assert!(queue.is_empty());
    }

    #[test]
    fn a_new_prefetch_set_replaces_the_old_neighbors() {
        let mut queue = CommitFilesQueue::default();
        queue.replace_prefetch([oid(1), oid(2)]);
        queue.replace_prefetch([oid(3)]);
        assert_eq!(queue.pop().unwrap().commit, oid(3));
        assert_eq!(queue.pop(), None);
    }
}
