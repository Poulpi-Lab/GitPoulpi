use super::*;

/// Result of one status+log+refs refresh, sent back to the render loop.
pub struct RefreshOutcome {
    pub log_result: GitResult<CommitGraph>,
    /// How many commits `log_result` was asked for: the depth when the
    /// refresh was sent, which pages landing meanwhile do not change.
    pub log_limit: usize,
    /// The generation of the hidden-refs filter `log_result` was read
    /// under (`DaemonLink::log_filter_generation`): an outcome from an older
    /// one is a walk of other tips and its Log is not taken.
    pub log_filter_gen: u64,
    pub status_result: GitResult<WorkingTree>,
    pub refs_result: GitResult<Vec<Ref>>,
    pub stashes_result: GitResult<Vec<Stash>>,
    pub worktrees_result: GitResult<Vec<Worktree>>,
    /// The repository's submodules (the SUBMODULES sidebar section). Read
    /// only when the repository has a `.gitmodules` and the scope can have
    /// changed them (see `SUBMODULES_SCOPE`).
    pub submodules_result: GitResult<Vec<Submodule>>,
    /// The unmerged paths, i.e. whether the repo is sitting in the middle
    /// of a merge/rebase/cherry-pick. Part of the ordinary refresh rather
    /// than an on-demand read because a conflict can appear as the result
    /// of *another* frontend's mutation, and the pane that offers the
    /// resolution has to be right without anyone asking it to look.
    pub conflicts_result: GitResult<Vec<ConflictEntry>>,
    /// The configured remotes. Part of the ordinary refresh because the
    /// Refs sidebar's REMOTE tree needs them to know where a remote's
    /// name ends and a branch's begins (a remote may be named `grp/sub`),
    /// and because `remote add`/`remove` from any client must show up
    /// here without anyone asking.
    pub remotes_result: GitResult<Vec<Remote>>,
    /// The repository-local `core.sshCommand` pin (ADR-0008), so the pin
    /// menu shows what is set even when another frontend set it.
    pub ssh_command_result: GitResult<Option<String>>,
    /// Whether the working tree is a sparse checkout (the footer badge, the
    /// Sparse checkout dialog's starting point). Read only when
    /// `.git/info/sparse-checkout` exists (no process otherwise), so a plain
    /// repository costs the poll nothing.
    pub sparse_result: GitResult<gitpoulpi_core::domain::SparseState>,
    /// The repository-local `core.autocrlf` / `core.eol` (`None` = unset),
    /// read from git itself so Settings shows the real values, including
    /// ones another frontend or `git config` set.
    pub repo_config_result: GitResult<std::collections::BTreeMap<String, Option<String>>>,
    /// The rebase (interactive or plain) currently in progress, if any --
    /// what `rebase_banner` draws its Continue/Skip/Abort strip from. For
    /// the same reason as `conflicts_result`: a rebase can be left
    /// mid-flight by another frontend, or by this one's own last run, and
    /// the banner must be right without anyone asking it to look.
    pub rebase_state_result: GitResult<Option<RebaseState>>,
    /// The merge, cherry-pick, revert or rebase in progress, whichever it
    /// is: what the operation banner draws from.
    pub operation_state_result: GitResult<Option<OperationState>>,
    /// The repository's `index.lock`, if any: the stale-lock strip.
    pub index_lock_result: GitResult<Option<gitpoulpi_core::domain::IndexLock>>,
    /// Git LFS state, when this refresh took a snapshot (see
    /// `lfs_ui::LfsGate` for when that is). `None` means "not re-read,
    /// keep what you have" - not "LFS is off".
    pub lfs: Option<crate::lfs_ui::LfsSnapshot>,
    /// What this refresh re-read. Status is always re-read; every other
    /// `*_result` field the scope leaves out holds a placeholder `Err`
    /// that the receiver must not apply (`ChangeScope::ALL` for the poll,
    /// a manual refresh and the first load).
    pub scope: ChangeScope,
    /// The `Activity` generation this refresh started at. The render loop
    /// keeps its own (optimistic) status when a mutation was submitted
    /// since: the outcome describes the state from before it.
    pub generation: u64,
    /// A hash of what this refresh carries (see `refresh::fingerprint`): two
    /// polls that read the same state have the same one, so the link can
    /// deliver the second without waking the render loop.
    pub fingerprint: u64,
    /// Per-read hashes of the three results the Log layout is built from
    /// (see [`ReadHashes`]).
    pub read_hashes: ReadHashes,
}

/// Content hashes of the Log, Refs, Stashes and LFS reads of one refresh, `None`
/// for a read the scope left out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadHashes {
    pub log: Option<u64>,
    pub refs: Option<u64>,
    pub stashes: Option<u64>,
    /// The LFS snapshot, `None` when this refresh took none. Kept out of
    /// `RefreshOutcome::fingerprint`: the gate takes a snapshot every so
    /// often, and a poll with one must not look different from the poll
    /// without (the link compares it with the last snapshot it saw).
    pub lfs: Option<u64>,
}

/// Which results an outcome really replaces (see [`ReadHashes::apply`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadChanges {
    pub log: bool,
    pub refs: bool,
    pub stashes: bool,
}

impl ReadHashes {
    /// Compares `new` with what the window last applied (`self`), records
    /// it, and says which reads differ. A read whose hash is unchanged keeps
    /// the held result and its generation counter, so the Log layout cache
    /// (keyed on those counters) does not rebuild on an idle poll. A read
    /// the scope left out is never "changed" and leaves its hash alone.
    pub fn apply(&mut self, new: ReadHashes) -> ReadChanges {
        fn step(held: &mut Option<u64>, new: Option<u64>) -> bool {
            match new {
                None => false,
                Some(h) if *held == Some(h) => false,
                Some(h) => {
                    *held = Some(h);
                    true
                }
            }
        }
        ReadChanges {
            log: step(&mut self.log, new.log),
            refs: step(&mut self.refs, new.refs),
            stashes: step(&mut self.stashes, new.stashes),
        }
    }

    /// These hashes as the window may record them: a Log read under another
    /// hidden-refs filter generation than the one in use (`read_gen` against
    /// `current_gen`) is a walk of other tips, so its hash is dropped and
    /// `apply` reports no Log change (the window then keeps the held Log and
    /// the lazy loader's bookkeeping untouched).
    pub fn for_filter_generation(mut self, read_gen: u64, current_gen: u64) -> ReadHashes {
        if read_gen != current_gen {
            self.log = None;
        }
        self
    }

    /// The window changed its held result behind the link's back (a lazily
    /// loaded page was appended): the next Log read must replace it.
    pub fn forget_log(&mut self) {
        self.log = None;
    }
}

/// Which files a selected commit touched, fetched on demand (see
/// `DaemonLink::request_commit_files`) rather than as part of every
/// refresh, since it's only needed while the Log pane's selection is on
/// that commit. Tagged with the requesting `commit` so a slow response
/// for a since-abandoned selection can be told apart from the latest one.
pub struct CommitFilesOutcome {
    pub commit: Oid,
    pub files: GitResult<Vec<FileDiff>>,
    /// The commit's *whole* tree, present only when it was asked for.
    ///
    /// `files` is what the commit changed, `tree` is what the repository
    /// contained; the commit-detail pane's `View all files` toggle switches
    /// between them, and carrying both in one outcome stops the pane
    /// flickering through an empty list while a second request is in flight.
    /// `None` means "not asked for", the common case: listing a large
    /// repository's tree on every selection change would be wasted work.
    pub tree: Option<GitResult<Vec<std::path::PathBuf>>>,
    /// `true` on the first of two answers for one selection: `files` is the
    /// list alone, every `hunks` still empty, and a second outcome with the
    /// hunks follows. The list is shown at once; the Diff view waits for the
    /// second answer (it shows "loading" rather than an empty diff).
    pub hunks_pending: bool,
}

/// What travels from `DaemonLink` to the link's commit-files worker
/// (`commit_files_worker`).
#[derive(Debug, Clone)]
pub enum CommitFilesRequest {
    /// Fetch one commit's files for someone waiting on the answer.
    Fetch(crate::commit_files_cache::QueuedFetch),
    /// Warm the cache for these commits (the selection's neighbors),
    /// replacing any prefetch still queued.
    Prefetch(Vec<Oid>),
    /// The files changed between the two ends of a comparison, published
    /// as the `CommitFilesOutcome` of the synthetic key `id`.
    Compare {
        id: Oid,
        target: gitpoulpi_core::domain::DiffTarget,
    },
}

/// One half of the working tree's diff -- `WorkingTreeVsIndex` (the
/// Unstaged half) or `IndexVsHead` (the Staged one) - fetched on demand
/// while the Diff view is showing a file from that half.
///
/// A separate round trip from `CommitFilesOutcome` because it answers a
/// different question and goes stale on a different schedule: every
/// mutation changes it, and hunk-level staging is *made of* mutations, so
/// the view re-requests it on every refresh rather than caching it.
/// Tagged with the target it was asked for, so a reply for the half the
/// view has since left can be told from the current one.
pub struct WorkingDiffOutcome {
    pub target: DiffTarget,
    /// The one file this answers for, or `None` for the whole
    /// diff. The Diff view only ever shows one file, so it asks for one.
    pub path: Option<PathBuf>,
    pub files: GitResult<Vec<FileDiff>>,
}

/// A request for a diff: a whole `target`, or just one file's
/// hunks of it, so opening a file in a 2,000-file working tree or a
/// 500-file commit costs that file's diff, not every file's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiffRequest {
    pub target: DiffTarget,
    pub path: Option<PathBuf>,
    /// A second path to include: the other name of a renamed file (the
    /// History view). The answer then holds the rename as one file.
    pub also: Option<PathBuf>,
}

impl FileDiffRequest {
    #[cfg(test)]
    pub fn whole(target: DiffTarget) -> Self {
        Self {
            target,
            path: None,
            also: None,
        }
    }

    pub fn file(target: DiffTarget, path: PathBuf) -> Self {
        Self {
            target,
            path: Some(path),
            also: None,
        }
    }

    pub fn with_also(mut self, also: Option<PathBuf>) -> Self {
        self.also = also;
        self
    }
}

/// Which of the three file-centric reads the File Insight overlay wants
/// for a path. One request type rather than three channels: they are
/// opened from the same places, land in the same overlay, and are all
/// slow enough to need the same "answer arrives later, possibly after the
/// user moved on" handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInsightKind {
    /// The file's contents at the revision (the "File View").
    Content,
    Blame,
    /// Open the file's History split view (`history_view`). Never sent to
    /// the daemon: every site that opens a History creates this request, and
    /// the event loop promotes it into the split view at its next tick.
    History,
    /// One page of the file's history (`FileHistory`, following renames),
    /// `skip` entries in, walking from the request's `at`. Routed to the
    /// History view, never to the File Insight panel.
    HistoryPage(u32),
    /// The new side of an open diff, for the Diff view's whole-file
    /// layout (`diff_content.rs`). Not a File View: the
    /// answer is routed to the Diff view, never to the File Insight panel.
    DiffContent(DiffContentSide),
    /// A file's diff recomputed from the exact bytes of its two sides read
    /// in one chosen encoding (`diff_recode.rs`): `None` is an empty side
    /// (an added or deleted file). Routed to `diff_recode::offer`, never to
    /// the File Insight panel.
    DiffRecoded {
        old: Option<DiffContentSide>,
        new: Option<DiffContentSide>,
        charset: gitpoulpi_core::textconv::Charset,
    },
    /// The exact bytes of an image (`read_blob_bytes`), for the image view
    /// (`image_view/pane.rs`). Routed to the image pane, never to the File
    /// Insight panel's text rows. `DiffContentSide` names where they live.
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    ImageBytes(DiffContentSide),
    /// Does git diff the path as binary (`BinaryPaths`)? Routed to
    /// `binary_hint`, never to a panel.
    BinaryState,
}

/// Where the new side of a diff lives: a commit's blob (`at`), the index
/// (the Staged half's new side) or the file on disk (the Unstaged half's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffContentSide {
    Commit,
    Index,
    WorkingTree,
    /// The first parent of the request's commit (the image view's "before"
    /// side of a commit's diff; a root commit has none, which is an error).
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    Parent,
    /// The `HEAD` commit (the "before" side of the Staged half).
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    Head,
    /// One stage of an unmerged path (1 base, 2 ours, 3 theirs): the merge
    /// editor's picture previews.
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    Stage(u8),
    /// This commit's blob, whatever the request's `at` (a comparison's ends).
    CommitAt(Oid),
    /// The first parent of this commit (a comparison span's start).
    ParentOf(Oid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInsightRequest {
    pub path: PathBuf,
    /// Revision to read/blame at, and the ceiling for a history walk.
    pub at: Oid,
    pub kind: FileInsightKind,
    /// The file is looked at as part of the WIP (uncommitted) state, not
    /// as a committed revision. File View reads the bytes on disk (or the
    /// index, see `staged`), Blame marks the lines HEAD does not own as
    /// "Not committed yet", History walks from `at` (HEAD) and is empty
    /// for a file HEAD never had. What View all files on the WIP row, a
    /// working diff's bar and the Changed files menus open.
    pub worktree: bool,
    /// With `worktree`: the file was opened from the Staged list, so the
    /// content is the index's rather than the working tree's.
    pub staged: bool,
}

impl FileInsightRequest {
    /// `at` is the zero oid while HEAD is unborn; no read ever sends it
    /// to the backend.
    pub fn is_unborn(&self) -> bool {
        self.at == Oid([0u8; 20])
    }
}

/// The answer to one [`FileInsightRequest`], shaped by its `kind`.
pub enum FileInsightPayload {
    /// Nothing to render for this file, with the reason in plain words
    /// (a deleted file has no working-copy content).
    Notice(String),
    Content(Blob),
    Blame(Blame),
    /// The answer to a [`FileInsightKind::HistoryPage`] request.
    HistoryPage(gitpoulpi_core::domain::FileHistoryPage),
    /// The answer to a [`FileInsightKind::ImageBytes`] request.
    #[cfg_attr(not(feature = "image-view"), allow(dead_code))]
    ImageBytes(gitpoulpi_core::domain::BlobBytes),
    /// The answer to a [`FileInsightKind::BinaryState`] request.
    BinaryState(bool),
    /// The answer to a [`FileInsightKind::DiffRecoded`] request: the diff
    /// and the new side's whole text (the whole-file layout's content).
    Recoded {
        diff: FileDiff,
        new_lines: Vec<String>,
    },
}

/// Tagged with the `request` it answers, for the same reason
/// [`CommitFilesOutcome`] carries its commit: blame on a big file can
/// take long enough that the user has already asked for something else,
/// and a stale answer must be discardable rather than overwriting the
/// newer one.
pub struct FileInsightOutcome {
    pub request: FileInsightRequest,
    pub result: GitResult<FileInsightPayload>,
}

/// One lazily-loaded page of older history, tagged with the
/// [`LogPageRequest`](crate::graph::LogPageRequest) it answers.
///
/// Tagged for a sharper reason than [`CommitFilesOutcome`]: a page is
/// *appended* to the node list, so one stitched in at the wrong offset
/// corrupts the graph's lane bookkeeping for every row below it.
/// `graph::LazyLog::on_page` checks the request's `skip` against what is
/// held and drops the page if a refresh moved the offsets.
pub struct LogPageOutcome {
    pub request: crate::graph::LogPageRequest,
    pub result: GitResult<CommitGraph>,
    /// The generation of the hidden-refs filter the page was read under; a
    /// page of an older one belongs to another walk and is dropped.
    pub log_filter_gen: u64,
}

/// One batch of commit stats.
pub struct CommitStatsOutcome {
    pub result: GitResult<Vec<CommitStat>>,
}

/// The answer to one `request_split_source`: the commit asked for and its
/// numbered changes (or why the daemon could not read them).
pub struct SplitSourceOutcome {
    pub commit: Oid,
    pub result: GitResult<gitpoulpi_core::domain::SplitSource>,
}

/// The link-side half of the split-source channel pair (see
/// [`split_source_worker`]).
pub(crate) struct SplitSourceWorker {
    pub(super) rx: tokio::sync::mpsc::UnboundedReceiver<Oid>,
    pub(super) out_tx: std_mpsc::Sender<SplitSourceOutcome>,
}

/// The link-side half of the commit-stats channel pair (see
/// [`commit_stats_worker`]).
pub(crate) struct StatsWorker {
    pub(super) rx: tokio::sync::mpsc::UnboundedReceiver<Vec<Oid>>,
    pub(super) out_tx: std_mpsc::Sender<CommitStatsOutcome>,
}
