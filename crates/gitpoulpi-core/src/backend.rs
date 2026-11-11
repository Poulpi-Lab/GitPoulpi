use std::path::{Path, PathBuf};

use crate::domain::rewrite::{Recipe, RewriteOutcome, RewritePlan, RewriteProgressSink, RunId};
use crate::domain::{
    AmOutcome, Blame, Blob, BlobSource, CheckoutTarget, CherryPickOptions, CherryPickOutcome,
    CloneOptions, CloneProgressSink, CommitAmend, CommitGraph, CommitMessages, CommitMessagesQuery,
    CommitOptions, CommitStat, ConflictEntry, ConflictResolution, ConflictSet, DiffFileList,
    DiffTarget, DiscardTarget, DiscoveredRepo, FastForwardOutcome, FetchOptions, FetchOutcome,
    FileDiff, FileHistoryPage, IndexLock, InitOptions, InteractiveRebase, LfsFetchOptions, LfsFile,
    LfsInstallScope, LfsLock, LfsPattern, LfsProgressSink, LfsPushOptions, LfsRepoStatus,
    LfsTrackOptions, LogOptions, LostCommitOptions, LostCommitsPage, MergeDocument, MergeOptions,
    MergeOutcome, NetworkAuth, Oid, OperationState, PatchApplied, PatchCreated, PatchSource,
    PathHash, PullOptions, PullOutcome, PushOptions, PushOutcome, RebaseOptions, RebaseOutcome,
    RebasePlanDraft, RebaseState, Ref, RefScope, RefSnapshot, RefUpdate, ReflogPage, Remote,
    RemoteAccessOptions, RemoteAccessReport, RepoRef, ResetMode, ResolveOptions, RevertOutcome,
    RewordOutcome, Snapshot, SparseState, SplitCommit, SplitOutcome, SplitSource, StageTarget,
    Stash, StashId, Submodule, SubmoduleUpdateOptions, WorkingTree, Worktree, WorktreeAddOptions,
};
use crate::Result;

/// A config section's name and its `(key, value)` pairs.
pub type ConfigSection = (String, Vec<(String, String)>);

/// The port every frontend (TUI/CLI/MCP, via the daemon) and every backend
/// adapter (git2, system git) implements against.
#[async_trait::async_trait]
pub trait GitBackend: Send + Sync {
    /// Short implementation name (`"git2"` or `"system"`), for diagnostics.
    fn name(&self) -> &'static str;

    // -- read --
    /// Finds the repository enclosing `path` and describes it.
    ///
    /// Behind the seam so both backends agree on what "the repository at
    /// this path" means, including a linked worktree (`.git` is a *file*),
    /// a bare repository and an external `$GIT_DIR`.
    ///
    /// `path` may be the root, any subdirectory, or a file inside it. Both
    /// fields of the returned [`RepoRef`] come from the root.
    ///
    /// A path that does not exist, and one inside no repository, are both
    /// [`crate::Error::RepoNotFound`] (also the answer for a recent entry
    /// whose directory has been deleted).
    fn discover(&self, path: &Path) -> Result<DiscoveredRepo>;

    /// Creates a new repository at `path` and returns the `RepoRef` every
    /// later request for it will be keyed on.
    ///
    /// Pinned by the contract suite:
    ///
    /// - **A missing directory is created**, parents included, like
    ///   `git init <path>`.
    /// - **The returned `RepoRef` is what `discover` would return** for the
    ///   same path.
    /// - **An existing repository root is an error**, not a silent
    ///   re-initialisation. A directory that merely contains files, or sits
    ///   inside another repository, is fine (a legitimate nested
    ///   repository).
    ///
    /// Synchronous: no network, and no repository to target yet.
    fn init(&self, path: &Path, opts: &InitOptions) -> Result<RepoRef>;

    fn status(&self, repo: &RepoRef) -> Result<WorkingTree>;
    /// The commit graph. With `all_refs`, `opts.hidden` drops refs from the
    /// walk (the commits only they reach go with them; `HEAD` always stays
    /// while a filter is active); both backends take the tips from
    /// `domain::visible_tips` and page that one walk with `skip`/`limit`.
    fn log(&self, repo: &RepoRef, opts: &LogOptions) -> Result<CommitGraph>;
    /// How many commits [`log`](Self::log) would return for `opts` if it
    /// were not paged: the same refs (`all_refs`, `start_ref`, HEAD) and
    /// path filter and `hidden` filter, `skip` and `limit` ignored. Counts without loading any
    /// commit's text, so it stays cheap on a very large history. An unborn
    /// repository is `0`. Read-only; nothing to journal.
    fn count_commits(&self, repo: &RepoRef, opts: &LogOptions) -> Result<u64>;
    fn diff(&self, repo: &RepoRef, target: DiffTarget) -> Result<Vec<FileDiff>>;
    /// The files [`diff`](Self::diff) lists for the same `target` - same
    /// paths, same statuses, same order - as cheaply as the backend can
    /// produce them.
    ///
    /// The commit-detail pane shows the list from this and fills the hunks
    /// in from `diff` a moment later.
    ///
    /// A backend for which the list costs what the whole diff costs
    /// (system git: one `git diff` process either way) answers with the
    /// whole diff and says so in [`DiffFileList::hunks_included`], so the
    /// caller never pays twice.
    fn diff_files(&self, repo: &RepoRef, target: DiffTarget) -> Result<DiffFileList>;
    /// [`CommitStat`] for each of `ids`, in the order given, for the Log's
    /// "Changes" column. Each is the commit against its first parent (a
    /// root commit against nothing) without rename detection, so
    /// `files` equals the length of `diff(CommitVsParent(id))`. A batch
    /// so the system backend can answer it with one process instead of
    /// one per commit. An empty `ids` is an empty answer; an unknown id
    /// is an error.
    fn commit_stats(&self, repo: &RepoRef, ids: &[Oid]) -> Result<Vec<CommitStat>>;
    /// [`diff`](Self::diff) restricted to `paths`: the hunks of just those
    /// files, so the Diff view can load one file's hunks when it is opened.
    ///
    /// Paths are literal (no globbing, no pathspec magic), repo-relative.
    /// The answer holds the files of `diff(target)` whose path is in
    /// `paths`, in the same order, with identical statuses and identical
    /// hunks and hunk ids (ids number the hunks within one file, so
    /// restricting never renumbers them). An empty `paths` is an empty
    /// answer, not "everything". Untracked files are not part of any
    /// `diff` target and are not here either.
    fn diff_paths(
        &self,
        repo: &RepoRef,
        target: DiffTarget,
        paths: &[PathBuf],
    ) -> Result<Vec<FileDiff>>;
    fn refs(&self, repo: &RepoRef) -> Result<Vec<Ref>>;
    /// Raw ref values (tag objects not peeled) under `scope`, plus HEAD
    /// (symbolic, possibly unborn, or detached). A read; the undo
    /// journal's source of truth for "what did this operation change".
    /// The hidden `refs/gitpoulpi/` namespace is never listed.
    fn ref_snapshot(&self, repo: &RepoRef, scope: &RefScope) -> Result<RefSnapshot>;
    /// An atomic compare-and-swap ref transaction: every update applies
    /// or none does. A ref that does not hold the expected value is
    /// `Error::UndoRefused(UndoRefusal::RefMoved)`; a HEAD that is not
    /// where `SetHead` expects is `HeadMoved`. `SetHead` re-points HEAD
    /// without touching the index or any file. `reflog_message` is
    /// written for every ref that moves.
    fn update_refs(
        &self,
        repo: &RepoRef,
        updates: &[RefUpdate],
        reflog_message: &str,
    ) -> Result<()>;
    /// Moves the branch `branch` from `from` to `to` **and** carries the
    /// index and working tree along, with `git reset --keep` semantics:
    /// local changes survive unless the move would overwrite them, in
    /// which case it is `UndoRefusal::WouldOverwrite` and nothing changed.
    /// `branch` must be the checked-out branch (else `HeadMoved`) and must
    /// hold `from` (else `RefMoved`). The undo journal's primitive for
    /// fast-forwards of the checked-out branch.
    fn move_head_keep(&self, repo: &RepoRef, branch: &str, from: Oid, to: Oid) -> Result<()>;
    /// The `key = value` pairs of one repository-local config section,
    /// full keys (`branch.x.remote`), in file order. Only the sections
    /// `branch.<name>` and `remote.<name>` are accepted (an undo helper, not a
    /// general config reader).
    fn config_section(&self, repo: &RepoRef, section: &str) -> Result<Vec<(String, String)>>;
    /// [`GitBackend::config_section`] for several sections at once, in the
    /// order asked. The system-git backend answers with one process
    /// instead of one per section (the undo journal reads a remote's
    /// section and every dependent branch's together).
    fn config_sections(&self, repo: &RepoRef, sections: &[String]) -> Result<Vec<ConfigSection>> {
        sections
            .iter()
            .map(|s| Ok((s.clone(), self.config_section(repo, s)?)))
            .collect()
    }
    /// The `branch.<name>` sections whose `remote` is `remote`, sorted:
    /// what `git remote remove` deletes and `git remote rename` rewrites
    /// besides the remote's own section.
    fn remote_dependents(&self, repo: &RepoRef, remote: &str) -> Result<Vec<String>>;
    /// Replaces the whole section with `entries` (empty removes it). Same
    /// restriction as [`GitBackend::config_section`].
    fn set_config_section(
        &self,
        repo: &RepoRef,
        section: &str,
        entries: &[(String, String)],
    ) -> Result<()>;
    /// The tree id of the index as it is right now (`git write-tree`):
    /// what a staging operation's undo records before and after. Writes
    /// tree objects (never the working tree); an index with unresolved
    /// conflicts has no tree and is an error.
    fn index_tree(&self, repo: &RepoRef) -> Result<Oid>;
    /// Makes the index equal `tree` exactly (`git read-tree`). Never
    /// touches the working tree. The undo journal's primitive for
    /// stage/unstage; the caller has verified, under the repository's
    /// write lock, that the index is what it expects.
    fn index_restore(&self, repo: &RepoRef, tree: Oid) -> Result<()>;
    /// A safety snapshot of every file under `paths` (repo-relative, `/`
    /// separated; a directory means everything below it): the index entry
    /// and the working-tree file, **including untracked files**, as raw
    /// bytes (no clean filter), with the executable bit and symlinks. It
    /// writes objects only: neither the index nor the working tree is
    /// touched. Take it before a destructive operation and keep `tree`
    /// reachable with a pin ref. Snapshot only what the operation will
    /// touch, never the whole tree.
    fn snapshot_create(&self, repo: &RepoRef, paths: &[String]) -> Result<Snapshot>;
    /// Writes a snapshot's index and working-tree sides back for `paths`,
    /// exactly: a file the snapshot holds is rewritten (content, mode,
    /// symlink), and a file under `paths` that the snapshot does *not*
    /// hold is removed from the index and the disk. Index and working
    /// tree are restored independently of each other.
    fn snapshot_restore(&self, repo: &RepoRef, snapshot: Oid, paths: &[String]) -> Result<()>;
    /// The current [`PathHash`] of every file under `paths` that exists in
    /// the index or the working tree, sorted by path. Equal to
    /// `snapshot_create(paths).hashes` taken at the same instant. A read:
    /// the undo preconditions.
    fn path_hashes(&self, repo: &RepoRef, paths: &[String]) -> Result<Vec<PathHash>>;
    /// The raw bytes of the blob `oid` (no filter, no decoding): what a
    /// snapshot's [`FileHash`] names. "Restore only this" writes them next
    /// to a file that changed since. A missing object is an error.
    fn blob_bytes(&self, repo: &RepoRef, oid: Oid) -> Result<Vec<u8>>;
    /// One page of a reflog, newest entry first. `refname` is `HEAD` or a
    /// ref (`refs/heads/main`; a bare name is a local branch, see
    /// [`crate::domain::reflog_refname`]). `old`/`new` are the values
    /// git stored, `old` being all zeros for the first entry. A ref with
    /// no reflog (a deleted branch, a tag, `core.logAllRefUpdates` off)
    /// is an empty page, not an error. A read; shows moves made outside
    /// GitPoulpi too.
    fn reflog(
        &self,
        repo: &RepoRef,
        refname: &str,
        skip: usize,
        limit: usize,
    ) -> Result<ReflogPage>;
    /// Commits no branch, tag, remote-tracking ref or stash reaches (the
    /// reflogs and GitPoulpi's undo pins do not count as reaching), newest
    /// first: what a deleted branch left behind (named by a reflog or only
    /// by an undo pin), and with `opts.deep` the dangling commits of the
    /// whole object database (what `git fsck --unreachable --no-reflogs`
    /// lists). A read, bounded by `opts.budget_ms`.
    fn lost_commits(&self, repo: &RepoRef, opts: &LostCommitOptions) -> Result<LostCommitsPage>;
    /// Every commit's author, time, subject and body, for the workspace
    /// search index: the commits reachable from the branches, remote
    /// branches, tags and HEAD (not stashes, not GitPoulpi's own refs),
    /// newest first, minus those reachable from `query.known_tips` (so an
    /// index is brought up to date by asking for what is new). See
    /// [`CommitMessages`] for `tips`, `replace` and `truncated`. A read.
    fn commit_messages(
        &self,
        repo: &RepoRef,
        query: &CommitMessagesQuery,
    ) -> Result<CommitMessages>;
    /// Deletes `paths` (files, or directories with everything below) from
    /// the working tree; the index is not touched, so a tracked file shows
    /// up as a deleted change, as with `rm`. A path that does not exist is
    /// ignored; a path outside the repository or inside `.git` is an error. The daemon-side home of the
    /// TUI's "Delete file", so it can be journaled and undone.
    fn delete_files(&self, repo: &RepoRef, paths: &[String]) -> Result<()>;
    /// Rewrites the bytes of the files `paths` (working tree only; the index
    /// is not touched) as `op` says: another encoding, other line breaks or
    /// other indentation (`textconv::convert_bytes`, the editor's own
    /// functions). A binary file, a missing one, one the target encoding
    /// cannot hold, is left alone and reported, never damaged; a path
    /// outside the repository or inside `.git` is an error. The daemon-side
    /// home of the right pane's "Convert" menu, so one call is one journal
    /// entry.
    fn convert_files(
        &self,
        repo: &RepoRef,
        paths: &[String],
        op: crate::textconv::ConvertOp,
    ) -> Result<crate::textconv::ConvertReport>;
    /// Appends `pattern` as its own line to the `.gitignore` named by
    /// `file` (repo-relative, default `.gitignore`; its file name must be
    /// `.gitignore`), creating the file and keeping an existing last line
    /// intact. `Ok(false)` when the pattern is already a line of the file
    /// (nothing written). The daemon-side home of the TUI's "Ignore" menu,
    /// so it can be journaled and undone.
    fn gitignore_add(&self, repo: &RepoRef, pattern: &str, file: Option<&str>) -> Result<bool>;
    /// Writes the `.gitattributes` rule that makes git diff `paths` as
    /// `treat` says (see [`crate::domain::Treat::attributes`]): one pattern
    /// per path (`scope`: the file itself or its extension; `Folder`: the
    /// path is a folder and the rule covers everything below), relative to
    /// `file`'s folder. `file` is repo-relative: `None` is the root
    /// `.gitattributes`, `Some(".git/info/attributes")` the per-clone file,
    /// else a `.gitattributes` inside the repository (created when
    /// missing). A rule already naming the same pattern is rewritten rather
    /// than contradicted. The answer says what was written and whether git
    /// now really treats `paths` that way (a rule closer to the file, or
    /// in `.git/info/attributes`, can still override a root one). The
    /// daemon-side home of "Treat as Binary/Text", journaled like
    /// [`GitBackend::gitignore_add`], except for the per-clone file, which
    /// lives outside the working tree.
    fn gitattributes_treat(
        &self,
        repo: &RepoRef,
        paths: &[String],
        scope: crate::domain::RuleScope,
        file: Option<&str>,
        treat: crate::domain::Treat,
    ) -> Result<crate::domain::AttributesWrite>;
    /// Each of `idents` as the repository's `.mailmap` (and `mailmap.file`)
    /// shows it, in the same order: the canonical name and email, or the
    /// identity itself when nothing maps it. The system backend asks `git
    /// check-mailmap`, `Git2Backend` libgit2's `Mailmap`; the contract suite
    /// pins that they agree. A read.
    fn mailmap_resolve(
        &self,
        repo: &RepoRef,
        idents: &[crate::domain::rewrite::Ident],
    ) -> Result<Vec<crate::domain::rewrite::Ident>>;
    /// Every name and email that authored or committed a commit of `scope`,
    /// with the counts, most commits first, and what the `.mailmap` already
    /// maps each to. The Identities list of the history workbench. A read;
    /// one `git log`, whatever the size of the history.
    fn mailmap_identities(
        &self,
        repo: &RepoRef,
        scope: &crate::domain::rewrite::Scope,
        options: &crate::domain::rewrite::ScopeOptions,
    ) -> Result<crate::domain::rewrite::IdentityList>;
    /// Appends the lines that map each of `from` to `canonical` to the
    /// `.mailmap` (`file`: repo-relative, default `.mailmap`; created when
    /// missing), once each. Nothing is committed. Journaled like
    /// [`GitBackend::gitignore_add`]: undo puts the previous file back.
    fn mailmap_add(
        &self,
        repo: &RepoRef,
        canonical: &crate::domain::rewrite::Ident,
        from: &[crate::domain::rewrite::Ident],
        file: Option<&str>,
    ) -> Result<crate::domain::rewrite::MailmapAdded>;
    /// For each of `paths`, in order: does `git diff` treat it as binary?
    /// The `diff` attribute decides when a rule sets or unsets it (`binary`
    /// unsets it); otherwise the content does, with git's own sniff (a NUL
    /// in the first 8000 bytes of the working-tree file). A read.
    fn binary_paths(&self, repo: &RepoRef, paths: &[String]) -> Result<Vec<bool>>;
    /// The repo-relative paths a patch file would touch (every side of a
    /// rename), without applying it: what to snapshot before
    /// [`GitBackend::patch_apply`].
    fn patch_files(&self, repo: &RepoRef, patch: &Path) -> Result<Vec<String>>;
    fn stashes(&self, repo: &RepoRef) -> Result<Vec<Stash>>;
    fn remotes(&self, repo: &RepoRef) -> Result<Vec<Remote>>;
    /// Every worktree of `repo`, main one first (see [`Worktree`]).
    fn worktrees(&self, repo: &RepoRef) -> Result<Vec<Worktree>>;
    /// Every submodule `.gitmodules` declares, sorted by path, with its
    /// state against what the superproject records. Top level only: a
    /// submodule's own submodules are listed by that repository. An
    /// uninitialized submodule is listed (state `Uninitialized`, no
    /// `head`); a repository without `.gitmodules` answers an empty list
    /// without spawning anything.
    fn submodules(&self, repo: &RepoRef) -> Result<Vec<Submodule>>;
    /// Registers submodules in `.git/config` (`git submodule init`), without
    /// cloning anything. Empty `paths` is every submodule; an unknown path is
    /// an error.
    fn submodule_init(&self, repo: &RepoRef, paths: &[String]) -> Result<()>;
    /// `git submodule update [--init] [--recursive] [-- paths]`: checks each
    /// submodule out at the commit the superproject records, cloning it first
    /// with `init`. May reach the network (the submodule's remote), so a
    /// caller that cares about prompts applies `credentials.https_mode`
    /// first, as the daemon does.
    fn submodule_update(&self, repo: &RepoRef, opts: &SubmoduleUpdateOptions) -> Result<()>;
    /// Turns any revision spelling git accepts (`HEAD`, `HEAD~2`, a
    /// branch or tag name, a full or abbreviated sha) into the [`Oid`] of
    /// the *commit* it names, peeling annotated tags on the way.
    ///
    /// The only sanctioned bridge from the strings CLI/MCP callers type to
    /// the `Oid`s every mutating operation takes, so no frontend runs
    /// `git rev-parse` outside the seam.
    ///
    /// An unresolvable revision is an `Err`, never a fallback to HEAD.
    fn resolve(&self, repo: &RepoRef, rev: &str) -> Result<Oid>;

    /// The best common ancestor of two commits (`git merge-base a b`), or
    /// `None` when their histories share nothing. A read; the code review
    /// view diffs `merge_base(base, head)..head`, which is what a host shows.
    fn merge_base(&self, repo: &RepoRef, a: Oid, b: Oid) -> Result<Option<Oid>>;

    /// `(ahead, behind)` of `tip` against `base`: the commits reachable from
    /// `tip` but not from `base`, and the reverse (`rev-list --left-right
    /// --count tip...base`). A read; the branching workflow's "behind its
    /// parent by n".
    fn ahead_behind(&self, repo: &RepoRef, tip: Oid, base: Oid) -> Result<(u32, u32)>;

    /// The paths a merge of `theirs` into `ours` would leave conflicted,
    /// sorted and without duplicates, computed **without touching the work
    /// tree, the index, HEAD or any ref** (proactive conflict detection).
    /// Empty when the merge would go through, including when one commit
    /// contains the other.
    ///
    /// `SystemGitBackend` runs `git merge-tree --write-tree --name-only -z`
    /// (git 2.38 or later; an older git is `Error::Unsupported`) and
    /// `Git2Backend` merges the two commits in memory. Both detect renames,
    /// so rename/modify merges cleanly on both and the sets agree; a merge
    /// can write the merged blobs and trees into the object database as
    /// unreferenced objects, which `git gc` collects.
    fn predict_merge_conflicts(
        &self,
        repo: &RepoRef,
        ours: Oid,
        theirs: Oid,
    ) -> Result<Vec<String>>;

    /// A file's *content* - as opposed to a diff of it - at a commit,
    /// at the index, or on disk (see [`BlobSource`]).
    ///
    /// `path` is always relative to the repository root, with `/`
    /// separators, matching every other path in the domain.
    ///
    /// A binary or oversized file is a successful [`Blob`] whose
    /// [`crate::domain::BlobContent`] says so, not an `Err`: "this is a
    /// 4 GiB PNG" is something a file viewer renders. Both backends learn
    /// the size before reading anything, so an oversized file is never
    /// pulled into memory just to be rejected. A path that does not
    /// exist at `source` *is* an `Err`.
    fn read_blob(&self, repo: &RepoRef, path: &Path, source: &BlobSource) -> Result<Blob>;

    /// A file's **exact bytes** at `source` (same sources and path rules as
    /// [`GitBackend::read_blob`]): no smudge filter, no decoding. What an
    /// image viewer needs, since `read_blob` drops the bytes of a binary
    /// file.
    ///
    /// The size is learned first; above `max` bytes the answer is
    /// [`crate::domain::BlobBody::TooLarge`] and nothing is read. An LFS
    /// pointer comes back as `BlobBody::Lfs`, carrying the real object when
    /// it is in the local LFS store and within `max`. A path that does not
    /// exist at `source` is an `Err`.
    fn read_blob_bytes(
        &self,
        repo: &RepoRef,
        path: &Path,
        source: &BlobSource,
        max: u64,
    ) -> Result<crate::domain::BlobBytes>;

    /// Every path present in the repository **at** `at`, sorted, with
    /// `/` separators - the whole tree, not the subset a commit changed
    /// (what the `View all files` toggle needs).
    ///
    /// Directories are not listed; a caller wanting a folder view derives
    /// it from the path list.
    fn list_tree(&self, repo: &RepoRef, at: Oid) -> Result<Vec<PathBuf>>;

    /// Per-line attribution for `path` as of the commit `at`.
    ///
    /// Always a committed revision: blaming a dirty working tree would
    /// attribute half the lines to git's "Not Committed Yet" pseudo-commit,
    /// whose zero oid nothing downstream can click through to. Callers
    /// that want "blame what I'm looking at" resolve a commit first.
    ///
    /// The returned lines are the same lines [`GitBackend::read_blob`]
    /// would produce for `BlobSource::Commit(at)`, in the same order --
    /// they share [`crate::domain::BlobContent::from_bytes`] - so a
    /// blame gutter can be drawn beside file content without the two
    /// disagreeing about where a line is.
    ///
    /// Whole-file renames are followed (blame's default): a line born
    /// under an old name carries that name in `BlameLine::orig_path`.
    /// Lines are not chased across files (no `-M`/`-C` line movement).
    fn blame(&self, repo: &RepoRef, path: &Path, at: Oid) -> Result<Blame>;

    /// The commits that changed `path`, newest first, **following renames**
    /// (`git log --follow`), starting from `start` and returning at most
    /// `limit` entries after skipping `skip`.
    ///
    /// - History is the default simplified history, not first-parent: a
    ///   merge appears only when the file differs from every parent (a
    ///   conflict resolution), and both sides of a kept merge are walked.
    ///   A kept merge reports its change and line counts against its
    ///   **first** parent.
    /// - A file deleted then re-added shows the re-adding commit as
    ///   `Added`, the deleting one as `Deleted`, and keeps going back.
    /// - Lazy: the walk stops once `skip + limit + 1` entries are known
    ///   (the extra one sets `FileHistoryPage::more`), so the first page
    ///   of a huge history is cheap; deep pages cost their depth.
    /// - A path that never existed at or before `start` is an empty page.
    fn file_history(
        &self,
        repo: &RepoRef,
        path: &Path,
        start: Oid,
        skip: usize,
        limit: usize,
    ) -> Result<FileHistoryPage>;

    // -- staging (every granularity funnels through StageTarget) --
    fn stage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()>;
    fn unstage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()>;

    // -- history mutation --
    fn commit(&self, repo: &RepoRef, message: &str, opts: &CommitOptions) -> Result<Oid>;
    fn checkout(&self, repo: &RepoRef, target: &CheckoutTarget) -> Result<()>;
    /// Discards every uncommitted change to each of `paths` (relative to
    /// the repository root, `/`-separated): a tracked path's index entry
    /// and working-tree copy are both reset back to `HEAD`, and a path with
    /// no `HEAD` entry (untracked, or staged but never committed) is
    /// removed from the index if present and deleted from the working tree.
    ///
    /// Unlike [`CheckoutTarget::File`] (one already-tracked path from a
    /// caller-supplied commit), this takes several paths and handles
    /// untracked ones, which are not a `checkout` source at all.
    ///
    /// Empty `paths` is a no-op success. A missing path, or one with
    /// nothing to discard, is not an error: discarding is idempotent, so a
    /// caller racing another client's discard sees success.
    fn discard(&self, repo: &RepoRef, paths: &[String]) -> Result<()>;
    /// Reverts one hunk, or some changed lines of one hunk, of a file's
    /// **unstaged** changes in the working tree - the
    /// working-tree half of what [`GitBackend::unstage`] does to the index
    /// with the same selection, and the "Discard Hunk" / "Discard
    /// this line" actions.
    ///
    /// The target is resolved against a working-tree-vs-index diff the
    /// backend computes itself, numbered as [`StageTarget::Hunk`] /
    /// [`StageTarget::Line`] are when staging. The index is never touched:
    /// a line that is staged *and* then edited again goes back to its
    /// staged content, not to `HEAD`'s. Only the named lines change on
    /// disk; every other change in the file, in that hunk or another,
    /// survives.
    ///
    /// Selecting an added line removes it from the file; selecting a
    /// removed line puts it back. An unknown hunk, or a selection whose
    /// patch no longer applies (the file moved underneath the caller), is
    /// an error and leaves the file exactly as it was.
    fn discard_part(&self, repo: &RepoRef, target: &DiscardTarget) -> Result<()>;
    fn branch_create(&self, repo: &RepoRef, name: &str, at: Oid) -> Result<()>;
    /// Create the local branch `local_name` tracking the remote-tracking
    /// branch `upstream` (spelled the way `RefKind::RemoteBranch` spells
    /// it, `origin/feature/login`) and check it out - what
    /// `git checkout -b <local_name> --track <upstream>` does, in one
    /// operation.
    ///
    /// One call because the three steps are only ever wanted together, and
    /// separate daemon round trips can interleave with another client's
    /// mutation, leaving a branch created but not checked out.
    ///
    /// An existing `local_name` is an **error**, never a silent checkout of
    /// that branch; callers wanting that call [`GitBackend::checkout`].
    fn branch_create_tracking(
        &self,
        repo: &RepoRef,
        local_name: &str,
        upstream: &str,
    ) -> Result<()>;
    fn branch_delete(&self, repo: &RepoRef, name: &str) -> Result<()>;
    fn branch_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()>;
    /// Sets the local branch `branch`'s upstream to the remote-tracking
    /// branch `upstream` (spelled `origin/feature`, as
    /// `RefKind::RemoteBranch` joins it), or removes it with `None` --
    /// `git branch --set-upstream-to` / `--unset-upstream`.
    ///
    /// The upstream must exist: pointing a branch at a remote branch
    /// nobody has fetched is an error rather than config that silently
    /// names nothing. Unsetting an upstream that is not set is a
    /// success - the state asked for is the state reached.
    fn branch_set_upstream(
        &self,
        repo: &RepoRef,
        branch: &str,
        upstream: Option<&str>,
    ) -> Result<()>;

    /// Moves the local branch `branch` forward to the commit `target`,
    /// **only** when that is a fast-forward - the
    /// `Fast-forward <branch> to <other>` action.
    ///
    /// - `branch`'s tip an ancestor of `target`: the branch moves there.
    ///   If it is the branch checked out in *this* worktree, the index and
    ///   working tree follow, as `git merge --ff-only` does; local changes
    ///   that the move would overwrite refuse it, with nothing changed.
    ///   Any other branch just has its ref moved (`git fetch . a:b`'s
    ///   semantics) and no file is touched.
    /// - `target` already contained in `branch` (the same commit, or
    ///   behind it): [`FastForwardOutcome::UpToDate`], nothing moves.
    /// - The two diverged: [`crate::Error::NotFastForward`], nothing moves.
    ///
    /// A branch checked out in *another* linked worktree is refused:
    /// moving it would leave that worktree's files describing the old
    /// commit, which is what `git fetch . a:b` refuses too.
    fn fast_forward(&self, repo: &RepoRef, branch: &str, target: Oid)
        -> Result<FastForwardOutcome>;

    /// Adds a linked worktree at `path` (`git worktree add`), on the
    /// branch or commit [`WorktreeAddOptions`] names, optionally on a
    /// new branch created there, and returns its entry as
    /// [`GitBackend::worktrees`] would list it.
    ///
    /// `path` must not exist, or be an empty directory - git's own rule.
    /// Missing parent directories are created. A branch already checked
    /// out in another worktree, a `new_branch` name already taken, and a
    /// `WorktreeStart::Branch` that is not a local branch are errors, and
    /// leave no worktree behind.
    fn worktree_add(
        &self,
        repo: &RepoRef,
        path: &Path,
        opts: &WorktreeAddOptions,
    ) -> Result<Worktree>;

    /// Writes `source` out as a patch file at `dest` (`git format-patch`
    /// for commits/ranges, `git diff HEAD --binary` for the working
    /// tree).
    ///
    /// Refuses, writing nothing, when: `dest` already exists; `dest`'s
    /// parent directory does not exist; the resulting patch would be
    /// empty; a [`PatchSource::Commits`] entry is a merge commit or
    /// names an unknown commit; or the working tree's `HEAD` is unborn
    /// (`PatchSource::WorkingTree` on a repository with no commits).
    fn patch_create(
        &self,
        repo: &RepoRef,
        source: &PatchSource,
        dest: &Path,
    ) -> Result<PatchCreated>;

    /// Applies `patch` (a file on disk, in `git apply`'s format) to the
    /// working tree only - never the index - via `git apply`.
    ///
    /// Atomic: a patch that does not apply cleanly changes nothing.
    fn patch_apply(&self, repo: &RepoRef, patch: &Path) -> Result<PatchApplied>;

    /// Replays the changes `commit` made (its diff against its parent, or
    /// against the empty tree for a root commit) onto the working tree
    /// only - never the index, never HEAD. Binary files and line endings
    /// come out as in the commit.
    ///
    /// All or nothing: when any file does not apply cleanly, nothing is
    /// changed and the error says why. A merge commit is refused with
    /// [`crate::Error::ApplyRefused`]. Returns the touched paths.
    fn apply_commit_to_worktree(&self, repo: &RepoRef, commit: Oid) -> Result<PatchApplied>;

    fn tag_create(
        &self,
        repo: &RepoRef,
        name: &str,
        at: Oid,
        annotated_message: Option<&str>,
    ) -> Result<()>;
    fn tag_delete(&self, repo: &RepoRef, name: &str) -> Result<()>;

    /// Replaces the existing tag `name` with an *annotated* tag carrying
    /// `message`, at the same commit.
    ///
    /// Separate from `tag_create(.., force: true)` because it must *keep*
    /// the tag where it is: both backends read the tag's own peeled commit
    /// and put it back there (pinned by the contract). Annotating a tag
    /// that does not exist is an error; annotating an already-annotated
    /// one replaces its message.
    fn tag_annotate(&self, repo: &RepoRef, name: &str, message: &str) -> Result<()>;

    /// Sends tags to `remote` - one named tag, several, or every tag no
    /// remote is known to have (see [`crate::domain::TagSelection`]).
    ///
    /// One operation so the frontends get one result, one broadcast and one
    /// credential prompt for what the user did as one gesture.
    ///
    /// Both backends must also **record the push locally**, by writing the
    /// mirror refs described at
    /// [`crate::domain::REMOTE_TAG_MIRROR_PREFIX`]; otherwise the tag keeps
    /// its "unpushed" marker until the next fetch.
    ///
    /// The returned [`PushOutcome::pushed_refs`] holds the tag names
    /// actually sent, in the order they were sent. Pushing an empty
    /// selection is a success reporting nothing.
    async fn tag_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        selection: &crate::domain::TagSelection,
    ) -> Result<PushOutcome>;

    /// Deletes tag `name` from `remote` (`git push --delete`), leaving
    /// the local tag alone.
    ///
    /// [`GitBackend::tag_delete`] is the other half and must not touch the
    /// remote either.
    ///
    /// Drops the tag's mirror ref on success, as `tag_push` writes one.
    async fn tag_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()>;

    /// Fetches the single tag `name` from `remote` into `refs/tags/` - the
    /// "Fetch this tag" of a remote-only tag. Never overwrites: a local
    /// tag of that name at another commit is an error, not a silent loss.
    /// Network. The remote tag mirror already knows the tag, so it is
    /// not rewritten.
    async fn tag_fetch(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()>;

    /// Deletes branch `name` from `remote` (`push --delete`), leaving
    /// any local branch of that name alone; the remote-tracking ref goes
    /// with it. The remote twin of [`GitBackend::branch_delete`], which
    /// must not touch the remote.
    async fn branch_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()>;

    /// What `remote` holds at the fully qualified `full_ref` (`refs/heads/x`
    /// or `refs/tags/v1`) right now, asked of the server (`ls-remote`);
    /// `None` when it has no such ref. For an annotated tag, the tag object.
    /// A network read, never answered from local tracking refs. It is the
    /// value an undo of a remote deletion records and later checks.
    async fn remote_ref_value(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
    ) -> Result<Option<Oid>>;

    /// Moves the branch or tag `full_ref` on `remote` from `expect` to `new`,
    /// never forced: `Some(oid)` pushes that object there (it re-creates a
    /// deleted ref when `expect` is `None`), `None` deletes the ref. The
    /// remote must hold `expect` at that moment (a lease); if it moved on,
    /// [`crate::Error::PushRejected`] with `StaleLease` comes back and
    /// nothing changed. An object this repository no longer has is
    /// [`crate::Error::UndoRefused`] with `RemoteObjectMissing`. Network.
    /// The remote-tracking ref (branch) or remote-tag mirror (tag) follows.
    async fn remote_ref_update(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
        new: Option<Oid>,
        expect: Option<Oid>,
    ) -> Result<()>;
    fn merge(&self, repo: &RepoRef, from: &Ref, opts: &MergeOptions) -> Result<MergeOutcome>;
    fn rebase(&self, repo: &RepoRef, onto: &Ref, opts: &RebaseOptions) -> Result<RebaseOutcome>;

    /// The range an interactive rebase from `upstream` would replay,
    /// oldest first - what a frontend builds an [`InteractiveRebase`]
    /// plan's `steps` from. `upstream: None` means the whole history
    /// (`--root`). A read.
    fn rebase_plan(&self, repo: &RepoRef, upstream: Option<Oid>) -> Result<RebasePlanDraft>;

    /// Runs an interactive rebase from an explicit, pre-decided todo list
    /// rather than an interactively edited one - the plan a frontend
    /// built (possibly from [`GitBackend::rebase_plan`]), validated
    /// again here regardless.
    ///
    /// Refuses (leaving the repository untouched) with
    /// [`crate::Error::RebaseRefused`] when: another operation is in
    /// progress, the working tree is dirty, `req.upstream` is not an
    /// ancestor of HEAD, the range contains a merge commit, `req.steps`
    /// is not exactly a permutation of the range, a `Squash`/`Fixup` step
    /// is first, or a `Reword` step has no message.
    ///
    /// Never consults the user's `sequence.editor`/`core.editor`/
    /// `rebase.autoSquash` config - the plan given here is authoritative.
    fn rebase_interactive(&self, repo: &RepoRef, req: &InteractiveRebase) -> Result<RebaseOutcome>;

    /// The state of a rebase (interactive or plain) currently in
    /// progress, or `None` if none is. A read.
    fn rebase_state(&self, repo: &RepoRef) -> Result<Option<RebaseState>>;

    /// Resumes a rebase stopped by a conflict or an `edit` step
    /// (`git rebase --continue`). [`crate::Error::RebaseRefused`] with
    /// [`crate::domain::RebaseRefusal::NoRebaseInProgress`] if none is
    /// running.
    fn rebase_continue(&self, repo: &RepoRef) -> Result<RebaseOutcome>;

    /// Skips the commit a rebase is stopped at (`git rebase --skip`).
    fn rebase_skip(&self, repo: &RepoRef) -> Result<RebaseOutcome>;

    /// Aborts a rebase in progress, restoring the branch and HEAD to
    /// where they were before it started (`git rebase --abort`).
    fn rebase_abort(&self, repo: &RepoRef) -> Result<()>;

    fn cherry_pick(
        &self,
        repo: &RepoRef,
        commits: &[Oid],
        opts: &CherryPickOptions,
    ) -> Result<CherryPickOutcome>;
    /// Reverts `commit` with a new commit. A merge commit needs
    /// `mainline`: the 1-based number of the parent whose side is kept
    /// (`git revert -m`); `None` on a merge commit is an error.
    fn revert(&self, repo: &RepoRef, commit: Oid, mainline: Option<u32>) -> Result<RevertOutcome>;
    fn reset(&self, repo: &RepoRef, target: Oid, mode: ResetMode) -> Result<()>;

    /// Gives `commit` the message `message`, keeping everything else.
    ///
    /// `commit` must be HEAD or an ancestor of HEAD. The commit is
    /// re-created with the same tree, parents and author (name, email
    /// *and date*); every commit between it and HEAD - the descendants
    /// that are ancestors of HEAD - is then re-created on top of it,
    /// again with its own tree, author and message, so merges among them
    /// are kept as merges with their other parents untouched. Committer
    /// is the current identity at the current time, as with `git rebase`
    /// or `git commit --amend`. Finally the checked-out branch (or a
    /// detached HEAD) moves to the new tip, with `ORIG_HEAD` and a reflog
    /// entry recording the old one.
    ///
    /// Only messages change, so the rewrite cannot conflict and **the index
    /// and working tree are never touched**: on HEAD this is an amend of
    /// the message alone (staged changes stay staged, *not* folded in as
    /// `git commit --amend` would), and a dirty working tree is no reason
    /// to refuse. No rebase runs, so the repository is never left
    /// mid-operation: either the branch moved or nothing did.
    ///
    /// The message is stored as [`crate::domain::normalize_reword_message`]
    /// leaves it. Refusals are [`crate::Error::RewordRefused`]: the
    /// commit is not on the current branch, a merge/rebase/... is in
    /// progress, or the message is empty. Other branches or tags pointing
    /// into the rewritten range are left where they are, as `git rebase`
    /// leaves them.
    fn reword(&self, repo: &RepoRef, commit: Oid, message: &str) -> Result<RewordOutcome>;

    /// [`GitBackend::reword`] generalised to the commit's other
    /// properties: whichever of `amend`'s message, author name/email/date
    /// and committer name/email/date are `Some` replace the commit's; the
    /// rest (tree, parents, every `None`) is kept. Same walk, same
    /// descendants re-created on top (their own authors kept), same
    /// guarantees (index and working tree untouched, no rebase) and the
    /// same typed refusals, plus `InvalidIdentity` and `InvalidDate`;
    /// [`crate::domain::CommitAmend::resolve`] runs first. Unless a
    /// committer field says otherwise the committer is the current
    /// identity, now. The reflog entry reads `amend: <oid> <summary>`.
    fn amend_commit(
        &self,
        repo: &RepoRef,
        commit: Oid,
        amend: &CommitAmend,
    ) -> Result<RewordOutcome>;

    /// The pieces `commit` can be split into, numbered with `--no-renames`
    /// (never from `diff(CommitVsParent)`, which honours `diff.renames`).
    /// Text-modified files list hunks; binary, mode-only, added, deleted,
    /// submodule and LFS files are whole-only. A read.
    fn split_source(&self, repo: &RepoRef, commit: Oid) -> Result<SplitSource>;

    /// Breaks `req.commit` into `req.parts` (oldest first) by plumbing, as
    /// `reword` does: the last part carries the original tree, the
    /// descendants are re-created on top, the index and working tree are
    /// never touched. Refusals are [`crate::Error::SplitRefused`] and
    /// [`crate::domain::validate_split`] runs first.
    fn split_commit(&self, repo: &RepoRef, req: &SplitCommit) -> Result<SplitOutcome>;

    // -- history rewrite (the history-rewrite design) --
    /// Computes `recipe` into a temporary pack under the repository's
    /// `gitpoulpi/rewrite/<run>/` directory and describes the result; no ref
    /// and no object store moves. A read. The preview is the real run:
    /// [`GitBackend::rewrite_apply`] indexes exactly that pack. `progress`
    /// returning `Break` cancels and leaves nothing behind. Refusals are
    /// [`crate::Error::RewriteRefused`];
    /// [`crate::domain::rewrite::validate_recipe`] runs first.
    fn rewrite_preview(
        &self,
        repo: &RepoRef,
        recipe: &Recipe,
        progress: RewriteProgressSink<'_>,
    ) -> Result<RewritePlan>;

    /// Indexes the previewed pack and moves every ref in one transaction with
    /// compare-and-swap on the old values (a moved ref fails the whole run,
    /// `RewriteRefusal::Stale`). Journaled by the daemon as one entry.
    fn rewrite_apply(&self, repo: &RepoRef, plan: &RewritePlan) -> Result<RewriteOutcome>;

    /// Applies a preview whose recipe extracts a subdirectory as a new
    /// repository (`ExtractMode::Clone` or `Submodule`) instead of moving
    /// this repository's refs: a `git clone --no-local` of this repository
    /// into `target` (a folder that does not exist or is empty) is rewritten
    /// there with the recipe, then cleaned of the old history. `Clone` leaves
    /// this repository untouched and the outcome moves no ref here;
    /// `Submodule` also adds one commit to the current branch that replaces
    /// the folder by a submodule of `target` (the outcome's one ref change),
    /// the old history stays. Journaled by the daemon (a submodule commit is
    /// undone by moving the branch back). `Error::RewriteRefused` otherwise.
    fn rewrite_extract(
        &self,
        _repo: &RepoRef,
        _plan: &RewritePlan,
        _target: &Path,
    ) -> Result<RewriteOutcome> {
        Err(crate::Error::Unsupported {
            operation: "rewrite extract",
        })
    }

    /// Deletes a preview's temporary objects. Idempotent.
    fn rewrite_discard(&self, repo: &RepoRef, run: &RunId) -> Result<()>;

    /// A previewed plan, without its rows, rebuilt from the run directory
    /// the preview left (so a preview outlives its daemon). `None` when the
    /// run is gone or its stored plan does not hash to its digest.
    fn rewrite_load(&self, _repo: &RepoRef, _run: &RunId) -> Result<Option<RewritePlan>> {
        Ok(None)
    }

    /// What an applied run moved, from the record its apply left in the run
    /// directory (`moved.json`), with the directory the maps are in. This is
    /// what the collaborator note and the push are built from; it outlives
    /// the daemon. `RewriteRefusal::Stale` when the run was not applied here
    /// or its directory was swept.
    fn rewrite_record(
        &self,
        _repo: &RepoRef,
        _run: &RunId,
    ) -> Result<crate::domain::rewrite::RewriteRecord> {
        Err(crate::Error::Unsupported {
            operation: "rewrite record",
        })
    }

    /// The run's `commit-map` and `ref-map` (git-filter-repo's format),
    /// copied to `out` when given. A read.
    fn rewrite_maps(
        &self,
        _repo: &RepoRef,
        _run: &RunId,
        _out: Option<&Path>,
    ) -> Result<crate::domain::rewrite::RewriteMaps> {
        Err(crate::Error::Unsupported {
            operation: "rewrite maps",
        })
    }

    /// Pushes the refs an applied run moved that were on a remote, each with
    /// `--force-with-lease=<ref>:<the remote's id when the preview ran>`
    /// (never a plain force), one `--atomic` push per remote where the
    /// server allows it. Protected refs are left out unless asked for. A
    /// rejection is a per-ref status in the answer, not an error. Not
    /// journaled and not undoable; the record of what was pushed lets a later
    /// undo of the rewrite warn.
    async fn push_rewritten(
        &self,
        _repo: &RepoRef,
        _req: &crate::domain::rewrite::PushRewritten,
    ) -> Result<crate::domain::rewrite::PushRewrittenOutcome> {
        Err(crate::Error::Unsupported {
            operation: "push rewritten",
        })
    }

    /// Deletes the old history of rewrite journal entries from this
    /// repository for good: drops the entries' undo pins, expires the
    /// reflogs that name the old commits, removes the applied runs' folders,
    /// prunes the object store (`gc --prune=now`), then checks that every
    /// object reachable only from the old tips is gone. Anything that still
    /// holds the old history (a branch outside the rewrite's scope, a stash,
    /// another worktree's HEAD, a remote-tracking ref, a pin of an entry that
    /// is not part of the request, ...) is a typed
    /// [`crate::domain::rewrite::PurgeBlocker`] and the call changes
    /// **nothing** (`PurgeStatus::Blocked`). With `dry_run` it only reports
    /// (`Ready` or `Blocked`). Stashes are never expired. Not journaled and
    /// not undoable; the daemon marks the entries `Purged`. `progress` is told
    /// each step; there is no cancel.
    fn rewrite_purge(
        &self,
        _repo: &RepoRef,
        _req: &crate::domain::rewrite::PurgeRequest,
        _progress: crate::domain::rewrite::PurgeProgressSink<'_>,
    ) -> Result<crate::domain::rewrite::PurgeOutcome> {
        Err(crate::Error::Unsupported {
            operation: "rewrite purge",
        })
    }

    /// What a content rule did to `path` in the preview's commit `commit`:
    /// the old and the new file, diffed (the new one is read back from the
    /// preview's temporary pack). Lines a secret rule removed are hidden. A
    /// read. `RewriteRefusal::Stale` when the run is gone.
    fn rewrite_blob_diff(
        &self,
        _repo: &RepoRef,
        _run: &RunId,
        _commit: Oid,
        _path: &str,
    ) -> Result<crate::domain::rewrite::BlobChange> {
        Err(crate::Error::Unsupported {
            operation: "rewrite blob diff",
        })
    }

    /// Compiles a script (an inline text, or a file of the repository) and
    /// reports its header, the functions it defines and every problem. A
    /// syntax error is a problem in the answer, not an error. A read.
    /// `Error::Unsupported` without the `rewrite-scripts` feature.
    ///
    /// An external script (shell, Python, external Lua) is checked against
    /// `policy` first: its language switched off is
    /// `RewriteRefusal::ScriptsDisabled`, its interpreter missing
    /// `RewriteRefusal::ToolMissing`; the answer carries the script's content
    /// hash and whether `policy` trusts it (an untrusted script is an answer,
    /// not an error, and is not run); a trusted one gets a dry handshake.
    fn rewrite_script_check(
        &self,
        _repo: &RepoRef,
        _script: &crate::domain::rewrite::ScriptRef,
        _policy: &crate::domain::rewrite::ScriptPolicy,
    ) -> Result<crate::domain::rewrite::script::ScriptCheck> {
        Err(crate::Error::Unsupported {
            operation: "rewrite script check",
        })
    }

    /// Runs a script on `commits` (their original data, in history order,
    /// state carried across them) and returns what it would change. Nothing
    /// is written. `RewriteRefusal::ScriptFailed` names the commit and the
    /// reason. A read. An external script that `policy` does not allow, whose
    /// interpreter is missing or that is not trusted is refused before it
    /// runs (`ScriptsDisabled`, `ToolMissing`, `ScriptNotTrusted`).
    fn rewrite_script_test(
        &self,
        _repo: &RepoRef,
        _script: &crate::domain::rewrite::ScriptRef,
        _policy: &crate::domain::rewrite::ScriptPolicy,
        _commits: &[Oid],
    ) -> Result<Vec<crate::domain::rewrite::script::ScriptTestRow>> {
        Err(crate::Error::Unsupported {
            operation: "rewrite script test",
        })
    }

    /// Scans the history of `scope` for secrets (token prefixes, private
    /// keys, passwords in URLs, `password = value` with enough entropy):
    /// every unique blob is read once, text only. The answer holds each
    /// finding's rule, path, first and last commit, counts and a redacted
    /// preview, never its value; a finding's id is stable, and a content rule
    /// with `scan: true` names findings by id (the backend resolves them at
    /// preview time). `progress` reports and can cancel. A read.
    fn rewrite_scan_secrets(
        &self,
        _repo: &RepoRef,
        _scope: &crate::domain::rewrite::Scope,
        _progress: crate::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<crate::domain::rewrite::SecretScan> {
        Err(crate::Error::Unsupported {
            operation: "rewrite scan secrets",
        })
    }

    /// The size analysis of the history `scope` walks (slice S18): the
    /// `top` largest blobs with their paths, first and last commit and
    /// whether they are still in HEAD, the biggest folders (unique blob
    /// bytes), the paths deleted from HEAD but kept by the history, and the
    /// totals with what removing the history-only data would free (an
    /// estimate: after a purge and a `gc` only). One `log --raw` pass plus
    /// one `cat-file --batch-check`. `progress` reports and can cancel. A read.
    fn rewrite_analyze(
        &self,
        _repo: &RepoRef,
        _scope: &crate::domain::rewrite::Scope,
        _top: u32,
        _progress: crate::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<crate::domain::rewrite::SizeAnalysis> {
        Err(crate::Error::Unsupported {
            operation: "rewrite analyze",
        })
    }

    // -- stash --
    /// `None`: there was nothing to stash ("No local changes to save"), no
    /// stash was made and nothing changed - callers record no undo entry.
    fn stash_push(
        &self,
        repo: &RepoRef,
        message: Option<&str>,
        include_untracked: bool,
    ) -> Result<Option<StashId>>;
    /// Stashes only `paths` (`git stash push -- <paths>`), leaving every
    /// other change where it is. Untracked files among `paths` are
    /// stashed too (they were asked for by name). An empty `paths` stashes
    /// everything, untracked files included.
    fn stash_push_paths(
        &self,
        repo: &RepoRef,
        message: Option<&str>,
        paths: &[String],
    ) -> Result<Option<StashId>>;
    fn stash_pop(&self, repo: &RepoRef, id: StashId) -> Result<()>;
    fn stash_apply(&self, repo: &RepoRef, id: StashId) -> Result<()>;
    fn stash_drop(&self, repo: &RepoRef, id: StashId) -> Result<()>;
    /// Puts the stash commit `commit` back on the stash list, at
    /// `stash@{0}` (`git stash store -m <message>`): how an undone drop or
    /// pop gets its stash back, with the very same commit. `commit` must
    /// be a stash commit (a merge of the base, the index and, maybe, the
    /// untracked files); anything else is an error and changes nothing.
    fn stash_store(&self, repo: &RepoRef, commit: Oid, message: &str) -> Result<()>;
    /// Builds a stash holding exactly the changes `commit` made (against
    /// its first parent, or the empty tree for a root commit) and puts it
    /// on top of the stash list, as `git stash store` would. The working
    /// tree, index and HEAD are not touched. `message` defaults to a text
    /// naming the commit. Returns the stash commit.
    fn stash_from_commit(&self, repo: &RepoRef, commit: Oid, message: Option<&str>) -> Result<Oid>;
    /// The repo-relative paths the stash commit `commit` holds changes
    /// for, untracked files included, sorted: what applying or popping it
    /// can touch. A read.
    fn stash_paths(&self, repo: &RepoRef, commit: Oid) -> Result<Vec<String>>;

    // -- network (async: fetch/pull/push/clone hit the network) --
    /// Sets (or, with `None`, removes) how the network git processes of
    /// `repo` authenticate: they get `auth` instead of the process-wide
    /// credential mode and SSH key (ADR-0010). Held in memory, per
    /// repository, never persisted; no entry means the unchanged behaviour.
    /// Not a repository mutation: not journaled, not undoable. `Git2Backend`
    /// forwards it to its system-git fallback, which does all networking.
    fn set_network_auth(&self, repo: &RepoRef, auth: Option<NetworkAuth>);
    /// Connection test: `git ls-remote --heads` against `remote`'s fetch URL
    /// (its push URL with [`RemoteAccessOptions::push`]), with the
    /// repository's [`NetworkAuth`] applied like for any transfer. A failed
    /// connection is **not** an `Err`: it is a report with `ok: false` and a
    /// classified [`crate::ErrorDetail`]. `Err` is for a remote that does not
    /// exist or a git that cannot start. A read: nothing in the repository
    /// changes.
    async fn remote_access_check(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &RemoteAccessOptions,
    ) -> Result<RemoteAccessReport>;
    async fn fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &FetchOptions,
    ) -> Result<FetchOutcome>;
    /// Fetches **every** configured remote (`git fetch --all`), with
    /// `--prune` when asked, and refreshes each remote's tag mirror the
    /// way [`GitBackend::fetch`] does for one.
    ///
    /// One operation rather than a frontend looping over `remotes()`:
    /// one result, one broadcast, and a remote added by another client a
    /// moment ago is included. `updated_refs` covers all remotes.
    async fn fetch_all(&self, repo: &RepoRef, opts: &FetchOptions) -> Result<FetchOutcome>;
    /// Fetches one `remote_ref` of `remote` that a normal fetch does not
    /// bring - a pull/merge request head such as `refs/pull/<n>/head` or
    /// `refs/merge-requests/<n>/head`, which is where a fork's commits
    /// live - and makes the local branch `branch` track it.
    ///
    /// The ref lands in `refs/remotes/<remote>/<branch>`; `branch` is
    /// created from it (with that as its upstream) or, when it already
    /// exists and is a strict ancestor, fast-forwarded. A branch with
    /// local commits of its own is left alone. Nothing is checked out
    /// (a *checked-out* `branch` is fast-forwarded in place); the caller
    /// checks it out. `remote_ref` must start with `refs/`.
    async fn fetch_ref_as_branch(
        &self,
        repo: &RepoRef,
        remote: &str,
        remote_ref: &str,
        branch: &str,
    ) -> Result<FetchOutcome>;
    /// Fetches one `[+]refs/<src>:refs/gitpoulpi/<dst>` spec from
    /// `remote` (a pull/merge request's head into `refs/gitpoulpi/pr/<n>`,
    /// for in-app review). Never a branch, tag or remote-tracking ref:
    /// the destination must sit strictly inside
    /// [`crate::domain::FETCH_REFSPEC_NAMESPACE`] and anything else is
    /// refused before git runs (see [`crate::domain::parse_fetch_refspec`]).
    /// Nothing is checked out, and the destination is force-updated (a
    /// request may be force-pushed). Not journaled and not undoable: it
    /// only refreshes a read-only cache ref no user workflow points at.
    /// Returns the destination ref in `updated_refs`.
    async fn fetch_refspec(
        &self,
        repo: &RepoRef,
        remote: &str,
        refspec: &str,
    ) -> Result<FetchOutcome>;
    /// Pulls into the checked-out branch, integrating as
    /// [`crate::domain::PullMode`] says - always explicitly, so the
    /// user's `pull.rebase` / `pull.ff` cannot change what was asked.
    ///
    /// A fast-forward-only pull of a diverged branch is
    /// [`crate::Error::NotFastForward`], with nothing changed.
    async fn pull(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome>;
    /// The download half of [`GitBackend::pull`]: fetches what the pull
    /// would integrate and touches nothing else (no index, working tree or
    /// `HEAD`), so the daemon can cancel it by dropping the future. The
    /// default does nothing: a backend without a separate download phase
    /// does it all in [`GitBackend::pull_integrate`].
    async fn pull_fetch(&self, _repo: &RepoRef, _opts: &PullOptions) -> Result<()> {
        Ok(())
    }
    /// The integrate half: merges, fast-forwards or rebases what
    /// [`GitBackend::pull_fetch`] downloaded, without touching the network,
    /// and gives `pull`'s outcome and errors. This is the point of no return
    /// of a pull: it rewrites the index and the working tree, so it must not
    /// be interrupted. The default is the whole `pull`.
    async fn pull_integrate(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
        self.pull(repo, opts).await
    }
    /// Pushes a local branch (`opts.branch`, or the checked-out one) to
    /// the same-named branch on `remote`.
    ///
    /// A force push is only ever `--force-with-lease`. A rejection the
    /// user can act on - the remote diverged, or the lease found it
    /// moved - is the typed [`crate::Error::PushRejected`], never a
    /// string to parse.
    async fn push(&self, repo: &RepoRef, remote: &str, opts: &PushOptions) -> Result<PushOutcome>;
    /// Pushes `HEAD` to `refs/for/<branch>` on `remote` - how a Gerrit
    /// change is created - with the optional `%topic=<topic>` option.
    /// `branch` is the Gerrit target branch (`main`), not a local one. Never
    /// forced. Not journaled and not undoable: the change lives on the
    /// server (abandon it there). The default is `Unsupported`.
    async fn push_for_review(
        &self,
        _repo: &RepoRef,
        _remote: &str,
        _branch: &str,
        _topic: Option<&str>,
    ) -> Result<PushOutcome> {
        Err(crate::Error::Unsupported {
            operation: "push_for_review",
        })
    }
    async fn clone(&self, url: &str, dest: &Path, opts: &CloneOptions) -> Result<RepoRef>;

    /// `clone`, reporting progress to `progress` as it goes.
    ///
    /// The default runs the plain `clone` and reports a single
    /// [`crate::domain::CloneProgress::done`] at the end, so a backend that
    /// cannot stream progress is still correct.
    ///
    /// Pinned by the contract suite: the last report is always `Done`/100,
    /// and `percent` never decreases within a phase.
    async fn clone_with_progress(
        &self,
        url: &str,
        dest: &Path,
        opts: &CloneOptions,
        progress: CloneProgressSink<'_>,
    ) -> Result<RepoRef> {
        let repo = self.clone(url, dest, opts).await?;
        progress(crate::domain::CloneProgress::done());
        Ok(repo)
    }
    fn remote_add(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()>;
    fn remote_remove(&self, repo: &RepoRef, name: &str) -> Result<()>;
    /// Renames a remote (`git remote rename`): its config section, its
    /// remote-tracking branches, every local branch's upstream that named
    /// it, *and* its tag mirror under
    /// [`crate::domain::REMOTE_TAG_MIRROR_PREFIX`] - or every tag it had
    /// would read as unpushed until the next fetch.
    fn remote_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()>;
    /// Sets a remote's fetch URL (`remote.<name>.url`).
    fn remote_set_url(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()>;
    /// Sets a remote's separate push URL (`remote.<name>.pushurl`), or
    /// removes it with `None` so pushes go to the fetch URL again.
    /// [`crate::domain::Remote::push_url`] always reports the URL a push
    /// would actually use, so after `None` it equals the fetch URL.
    fn remote_set_push_url(&self, repo: &RepoRef, name: &str, url: Option<&str>) -> Result<()>;

    /// The repository-local `core.sshCommand`, if one is set. Read-only
    /// companion of [`Self::set_ssh_command`].
    fn ssh_command(&self, repo: &RepoRef) -> Result<Option<String>>;
    /// Sets (or, with `None`, removes) the repository-local
    /// `core.sshCommand` - the opt-in way to make one repository use a
    /// profile's SSH key (ADR-0008). Clearing an unset value is not an
    /// error. Only `core.sshCommand` is reachable: this is deliberately
    /// not a general config write.
    fn set_ssh_command(&self, repo: &RepoRef, command: Option<&str>) -> Result<()>;

    /// The repository-local value of `key` (the repo's own config file, not
    /// the global or system one), or `None` when unset. `key` must be on
    /// [`crate::repo_config::REPO_CONFIG_KEYS`] (`core.autocrlf`, `core.eol`).
    fn repo_config(&self, repo: &RepoRef, key: &str) -> Result<Option<String>>;

    /// The entries of the repository-local config (`.git/config` only, not
    /// the global or system files) whose key starts, case-insensitively, with
    /// one of `prefixes` (`"gitflow."`, `"branch."`), as `(key, value)` in
    /// file order, keys spelled as `config --list` does (section and
    /// variable lower-cased, subsection as written). Not limited to the
    /// [`crate::repo_config::REPO_CONFIG_KEYS`] allow-list: it is read-only and
    /// meant for other tools' settings (git-flow, Git Town) and the recorded
    /// parent of a branch (`branch.<b>.gitpoulpi-parent`). An entry without a
    /// value is skipped.
    fn config_entries(&self, repo: &RepoRef, prefixes: &[&str]) -> Result<Vec<(String, String)>>;
    /// Sets (or, with `None`, removes) the repository-local `key`, so the
    /// repository inherits the global value again. Same allow-list as
    /// [`Self::repo_config`], and the value must be one the key accepts.
    /// Clearing an unset key is not an error.
    fn set_repo_config(&self, repo: &RepoRef, key: &str, value: Option<&str>) -> Result<()>;

    // -- Sparse checkout (cone mode) --
    /// Whether the working tree is a sparse checkout, in which mode, and
    /// which directories are checked out. A repository that never enabled
    /// it reports `SparseState::default()` (disabled), never an error.
    fn sparse_checkout_status(&self, repo: &RepoRef) -> Result<SparseState>;
    /// Makes the checkout sparse (cone mode) showing exactly `dirs` plus
    /// the files at the root; replaces any earlier set, so directories left
    /// out disappear from the working tree and the new ones appear. `dirs`
    /// go through [`crate::domain::normalize_sparse_dirs`] (at least one,
    /// relative, no patterns). Not journaled: nothing is lost, "disable"
    /// brings every file back.
    fn sparse_checkout_set(&self, repo: &RepoRef, dirs: &[String]) -> Result<()>;
    /// Turns sparse checkout off: every file of the tree is checked out
    /// again. Disabling a repository that is not sparse is not an error.
    fn sparse_checkout_disable(&self, repo: &RepoRef) -> Result<()>;

    // -- Git LFS (ADR 0009) --
    /// What is known about Git LFS in this repository: the `git-lfs`
    /// program's availability, whether the repository routes files
    /// through it, the tracked patterns, and whether its filters and hook
    /// are installed.
    ///
    /// A read that **never fails because `git-lfs` is missing**: that is
    /// `binary: LfsBinary::Missing` in the answer, so a frontend can
    /// explain it. Always re-probes the program (no stale cache).
    fn lfs_status(&self, repo: &RepoRef) -> Result<LfsRepoStatus>;
    /// Which external programs a history rewrite could use here (`gpg`,
    /// `ssh-keygen`, `gpgsm`, `git-lfs`, `python`, shells) and whether each
    /// works, with the install hint key for this OS. **The core rewrite
    /// actions need none of them** (see `RuleKind::required_tools`) and the
    /// embedded Lua is always there. Cached for a minute; `fresh` re-asks
    /// (the "Check again" button). Runs programs: never call it on a render
    /// thread. A read that never fails because a tool is missing.
    fn rewrite_tools(
        &self,
        repo: &RepoRef,
        fresh: bool,
    ) -> Result<crate::domain::rewrite::RewriteTools>;
    /// Signs a dummy payload once with the repository's signing setup
    /// (`gpg.format`, `user.signingkey`, `gpg.*.program`) and verifies it:
    /// the "Check signing" smoke test before a run that re-signs. May open a
    /// passphrase prompt. A broken setup is an `Ok` check with a `reason`,
    /// not an error. Runs programs: never on a render thread.
    fn rewrite_check_signing(&self, repo: &RepoRef)
        -> Result<crate::domain::rewrite::SigningCheck>;
    /// The LFS pointer files at a commit (`Some`) or in the index
    /// (`None`), each with whether its real object is in the local LFS
    /// store. Needs no `git-lfs` program. Empty for a repository that does
    /// not use LFS.
    fn lfs_files(&self, repo: &RepoRef, at: Option<Oid>) -> Result<Vec<LfsFile>>;

    /// `git lfs install`: the clean / smudge filters and the LFS hooks,
    /// for this repository only (`Repository`, the default: `--local`) or
    /// the user's global git config (`User`). Needs the `git-lfs`
    /// program: [`crate::Error::LfsUnavailable`] otherwise, with nothing
    /// changed.
    fn lfs_install(&self, repo: &RepoRef, scope: LfsInstallScope) -> Result<()>;
    /// Routes `patterns` through LFS (`git lfs track`). Writes
    /// `.gitattributes` and leaves it **unstaged**, a visible working
    /// change. Returns every LFS pattern of the repository afterwards.
    /// With `opts.renormalize` the already-committed files that match are
    /// re-staged so their next commit stores pointers.
    fn lfs_track(
        &self,
        repo: &RepoRef,
        patterns: &[String],
        opts: &LfsTrackOptions,
    ) -> Result<Vec<LfsPattern>>;
    /// Stops routing `patterns` through LFS (`git lfs untrack`). Returns
    /// the LFS patterns left.
    fn lfs_untrack(&self, repo: &RepoRef, patterns: &[String]) -> Result<Vec<LfsPattern>>;
    /// `git lfs fetch <remote>`: download the LFS objects the current ref
    /// (or every ref with `opts.all_refs`) points at into the local LFS
    /// store, leaving the working tree alone. Network. Reports progress to
    /// `progress`; the last report is always `done: true`, and no
    /// direction's counters ever decrease (the contract suite pins both).
    /// Needs the `git-lfs` program ([`crate::Error::LfsUnavailable`]).
    async fn lfs_fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsFetchOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()>;
    /// `git lfs pull <remote>`: fetch, then replace the pointer files in
    /// the working tree with the real content. Network.
    async fn lfs_pull(
        &self,
        repo: &RepoRef,
        remote: &str,
        progress: LfsProgressSink<'_>,
    ) -> Result<()>;
    /// `git lfs checkout [paths]`: write the real content of every LFS
    /// pointer in the working tree (or only `paths`, relative to the
    /// root; empty means all) from the objects already in the local LFS
    /// store. Offline: it never downloads (that is `lfs_pull`). Installs
    /// the repository-local LFS filters first when they are missing, like
    /// `lfs_pull`. Not a journaled state change: it only expands pointers.
    /// Needs the `git-lfs` program ([`crate::Error::LfsUnavailable`]).
    async fn lfs_checkout(
        &self,
        repo: &RepoRef,
        paths: &[PathBuf],
        progress: LfsProgressSink<'_>,
    ) -> Result<()>;
    /// `git lfs push <remote>`: upload the LFS objects of the current
    /// branch (or of every ref with `opts.all_refs`). Network; a
    /// mutation of the remote.
    async fn lfs_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsPushOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()>;
    /// The file locks held on the LFS server behind `remote`, each marked
    /// `ours` when the current user holds it. Network; a read (never
    /// broadcasts `RepoChanged`). Locking is a server feature reached with
    /// the remote's own credentials. A server without locking answers
    /// git-lfs's `locks` with an empty list, so that is what this returns;
    /// [`crate::Error::LfsLockingUnsupported`] comes from `lfs_lock`.
    async fn lfs_locks(&self, repo: &RepoRef, remote: &str) -> Result<Vec<LfsLock>>;
    /// Locks `path` on the server, so nobody else can lock or push
    /// changes to it. Network. A server without the locking API is
    /// [`crate::Error::LfsLockingUnsupported`].
    async fn lfs_lock(&self, repo: &RepoRef, remote: &str, path: &Path) -> Result<LfsLock>;
    /// Releases the lock on `path`. Somebody else's lock is refused unless
    /// `force` (which breaks it: destructive). Network.
    async fn lfs_unlock(
        &self,
        repo: &RepoRef,
        remote: &str,
        path: &Path,
        force: bool,
    ) -> Result<()>;
    /// `git lfs prune`: delete old, already-pushed objects from the local
    /// LFS store.
    fn lfs_prune(&self, repo: &RepoRef) -> Result<()>;

    // -- conflicts --
    fn conflicts(&self, repo: &RepoRef) -> Result<Vec<ConflictSet>>;
    /// Every unmerged path with its stages' modes and oids and the kind of
    /// conflict they make, sorted by path. Reads no blob (the cheap form of
    /// [`GitBackend::conflicts`], for the refresh): 1 spawn on system git,
    /// none under git2. Empty when nothing is unmerged.
    fn conflict_entries(&self, repo: &RepoRef) -> Result<Vec<ConflictEntry>>;
    /// Everything the merge conflict editor needs for one unmerged `path`:
    /// the three stages read, git's own 3-way merge of them parsed into
    /// regions (see [`MergeDocument`]), the pane labels and the state of
    /// the file on disk. Text is decoded as `read_blob` does (the fallback
    /// encoding applies). A path that is not unmerged is an error.
    fn merge_document(&self, repo: &RepoRef, path: &Path) -> Result<MergeDocument>;
    /// Resolves the unmerged `path` as `resolution` says and, with
    /// `opts.stage`, stages the result. Refused (nothing written) with
    /// [`crate::domain::ConflictRefusal::Stale`] when `opts.expected` is
    /// not what the index holds, and
    /// [`crate::domain::ConflictRefusal::Unencodable`] for text a
    /// `UseEncoded` encoding cannot hold. A path that is not conflicted is
    /// written and staged like any file unless `expected` is given (then
    /// it is stale).
    fn resolve_conflict(
        &self,
        repo: &RepoRef,
        path: &Path,
        resolution: ConflictResolution,
        opts: &ResolveOptions,
    ) -> Result<()>;
    /// Puts a resolved path back in conflict: the stage 1/2/3 entries of
    /// `entry` return to the index (stage 0 goes) and the working tree
    /// file is rewritten the way git leaves it (the merge with conflict
    /// markers; the surviving side for a non-text conflict). Refused with
    /// [`crate::domain::ConflictRefusal::AlreadyConflicted`] when the path
    /// is still unmerged.
    fn unresolve_conflict(&self, repo: &RepoRef, path: &Path, entry: &ConflictEntry) -> Result<()>;
    /// The merge, cherry-pick, revert or rebase in progress, if any
    /// (rebase wins when several markers exist).
    fn operation_state(&self, repo: &RepoRef) -> Result<Option<OperationState>>;
    /// Abandons whatever [`GitBackend::operation_state`] reports
    /// (`merge`/`cherry-pick`/`revert`/`rebase --abort`): HEAD, the index
    /// and the working tree return to before the operation. Destructive
    /// for work done since. Without an operation: refused with
    /// [`crate::domain::ConflictRefusal::NoOperation`].
    fn operation_abort(&self, repo: &RepoRef) -> Result<()>;
    /// `git am --continue`: applies the patch the `am` stopped on (the
    /// caller resolved and staged it) and the rest. Without an `am` in
    /// progress: refused with [`crate::domain::ConflictRefusal::NoOperation`].
    fn am_continue(&self, repo: &RepoRef) -> Result<AmOutcome>;
    /// `git am --skip`: drops the patch the `am` stopped on and goes on.
    /// Same refusal as [`GitBackend::am_continue`].
    fn am_skip(&self, repo: &RepoRef) -> Result<AmOutcome>;
    /// `git am --abort`: back to the branch tip from before the `am`.
    /// Same refusal as [`GitBackend::am_continue`].
    fn am_abort(&self, repo: &RepoRef) -> Result<()>;
    /// The repository's `index.lock`, if the file exists. In a linked
    /// worktree it is the worktree's own (`.git/worktrees/<name>`).
    fn index_lock(&self, repo: &RepoRef) -> Result<Option<IndexLock>>;
    /// Deletes a stale `index.lock` (and nothing else). Refused with an
    /// error when there is none, or when it is younger than
    /// [`crate::domain::STALE_INDEX_LOCK_SECS`].
    fn index_lock_remove(&self, repo: &RepoRef) -> Result<()>;
    /// Skips what [`GitBackend::operation_state`] reports
    /// (`cherry-pick --skip`, `revert --skip`, `rebase --skip`): the
    /// stopped commit is given up on and HEAD, the index and the working
    /// tree return to HEAD. The way out of a pick or revert that became
    /// empty (the change is already applied; git refuses to commit it). A
    /// merge has nothing to skip: an `Error::Backend`, nothing changed.
    /// Without an operation: [`crate::domain::ConflictRefusal::NoOperation`].
    fn operation_skip(&self, repo: &RepoRef) -> Result<()>;
}
