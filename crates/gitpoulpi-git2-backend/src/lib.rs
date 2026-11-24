//! GitBackend implementation over libgit2 (via the `git2` crate).
//!
//! `Git2Backend` is a decorator: per-operation it either calls into libgit2
//! directly or dispatches to an inner `gitpoulpi_cli_backend::SystemGitBackend`
//! for operations git2 doesn't support well (see
//! `docs/architecture/git-backend-trait.md` for the capability matrix this
//! dispatch follows and which operations are currently Native vs.
//! CliFallback in this implementation).

mod journal_ops;
#[cfg(feature = "lfs")]
mod lfs;
#[cfg(not(feature = "lfs"))]
#[path = "lfs_stub.rs"]
mod lfs;
mod mutate;
mod predict;
mod reads;
mod reflog_ops;
mod repo;
mod reword;
mod snapshot_ops;
mod staging;

use std::path::{Path, PathBuf};

use gitpoulpi_cli_backend::SystemGitBackend;

/// The commit total counted by an in-process libgit2 revwalk, whichever
/// backend the session runs (`log.commit_count = "libgit2"`). Spawns no
/// process, except for a path-filtered count (see `reads::count_commits`).
pub fn count_commits_in_process(repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
    reads::count_commits(repo, opts)
}
use gitpoulpi_core::domain::rewrite::{
    Recipe, RewriteOutcome, RewritePlan, RewriteProgressSink, RunId,
};
use gitpoulpi_core::domain::*;
use gitpoulpi_core::{GitBackend, Result};

/// The tree of the index on disk, written to the object database, with no
/// process. The undo journal calls it for every backend: a tree id is the
/// same whoever computes it, and it keeps system-git staging at one
/// process. Errors (conflicts, a split or sparse index libgit2 cannot
/// read) are the caller's cue to ask the backend instead.
pub fn native_index_tree(repo: &RepoRef) -> Result<Oid> {
    journal_ops::index_tree(repo)
}

/// A fingerprint of what the index *holds*: every entry's mode, blob id,
/// stage (so unmerged entries count) and path, hashed. Unlike the index
/// file's checksum it does not change when a `git status` merely
/// refreshes the stat data.
pub fn native_index_fingerprint(repo: &RepoRef) -> Result<String> {
    let git = repo::open(repo)?;
    let mut index = git.index().map_err(repo::map_err)?;
    // Read from disk: another process may have written it.
    index.read(true).map_err(repo::map_err)?;
    let mut text: Vec<u8> = Vec::new();
    for entry in index.iter() {
        let stage = (entry.flags >> 12) & 0x3;
        text.extend_from_slice(format!("{:o} {} {stage}\t", entry.mode, entry.id).as_bytes());
        text.extend_from_slice(&entry.path);
        text.push(b'\n');
    }
    let id = git2::Oid::hash_object(git2::ObjectType::Blob, &text).map_err(repo::map_err)?;
    Ok(id.to_string())
}

/// `GitBackend::snapshot_create` / `path_hashes` over libgit2, for the
/// undo journal to use whichever backend serves the session (the contract
/// suite holds the results identical): no process.
pub fn native_snapshot_create(repo: &RepoRef, paths: &[String]) -> Result<Snapshot> {
    snapshot_ops::snapshot_create(repo, paths)
}

/// `GitBackend::ref_snapshot` and `stashes` over libgit2 (the contract
/// suite holds them identical to the system git's): the journal reads them
/// around every recorded operation, so serving them in process saves the
/// system backend a spawn or two per read.
pub fn native_ref_snapshot(repo: &RepoRef, scope: &RefScope) -> Result<RefSnapshot> {
    journal_ops::ref_snapshot(repo, scope)
}

pub fn native_stashes(repo: &RepoRef) -> Result<Vec<Stash>> {
    reads::stashes(repo)
}

pub fn native_path_hashes(repo: &RepoRef, paths: &[String]) -> Result<Vec<PathHash>> {
    snapshot_ops::path_hashes(repo, paths)
}

pub struct Git2Backend {
    fallback: SystemGitBackend,
}

impl Git2Backend {
    pub fn new() -> Self {
        Self {
            fallback: SystemGitBackend::new(),
        }
    }
}

/// Diff targets that involve the working tree or the index, the only ones
/// LFS filters can change the answer for.
fn touches_worktree_or_index(target: &DiffTarget) -> bool {
    matches!(
        target,
        DiffTarget::WorkingTreeVsIndex | DiffTarget::IndexVsHead
    )
}

impl Git2Backend {
    /// A backend that behaves as if `git-lfs` is not installed (see
    /// `SystemGitBackend::without_lfs`).
    pub fn without_lfs() -> Self {
        Self {
            fallback: SystemGitBackend::without_lfs(),
        }
    }
}

impl Default for Git2Backend {
    fn default() -> Self {
        Self::new()
    }
}

/// Methods answered by the system-git backend: libgit2 either lacks the
/// operation or cannot match git's result exactly. (Async methods cannot
/// be generated here: `async_trait` expands before this macro does.)
macro_rules! forward_to_fallback {
    ($(
        $(#[$meta:meta])*
        fn $name:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) -> $ret:ty;
    )*) => {
        $(
            $(#[$meta])*
            fn $name(&self $(, $arg: $ty)*) -> $ret {
                self.fallback.$name($($arg),*)
            }
        )*
    };
}

#[async_trait::async_trait]
impl GitBackend for Git2Backend {
    fn name(&self) -> &'static str {
        "git2"
    }
    fn discover(&self, path: &Path) -> Result<DiscoveredRepo> {
        repo::discover(path)
    }
    fn init(&self, path: &Path, opts: &InitOptions) -> Result<RepoRef> {
        repo::init(path, opts)
    }
    /// Routed to the system git in an LFS repository: libgit2 has no
    /// clean filter, so it compares a smudged file with the pointer blob
    /// and reports every stat-dirty LFS file as modified (ADR 0009).
    fn status(&self, repo: &RepoRef) -> Result<WorkingTree> {
        if lfs::is_active(repo) {
            self.fallback.status(repo)
        } else {
            reads::status(repo)
        }
    }
    /// Native, *except* when a path filter is asked for.
    ///
    /// `git log -- <path>` applies history simplification (a merge whose
    /// tree equals a parent's along that path is pruned and only that
    /// parent followed); libgit2's revwalk has no equivalent, so a
    /// hand-rolled filter would return a different commit list from
    /// `SystemGitBackend` on merge-heavy history. Dispatched per variant,
    /// like `checkout(File)` (see `mutate::checkout`).
    fn log(&self, repo: &RepoRef, opts: &LogOptions) -> Result<CommitGraph> {
        if opts.paths.is_empty() {
            reads::log(repo, opts)
        } else {
            self.fallback.log(repo, opts)
        }
    }
    fn count_commits(&self, repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
        // System git by default (`log.commit_count`; the daemon calls
        // `count_commits_in_process` for "libgit2"): `rev-list --count` over 50,000 commits takes
        // about 12 ms where a libgit2 revwalk (oids only, no commit object
        // loaded by us, still parsed by libgit2 to follow parents) takes
        // about 210 ms, because git reads the commit-graph file when there
        // is one and libgit2 does not. One spawn, off the request path.
        // Path filters need history simplification, which git does too.
        self.fallback.count_commits(repo, opts)
    }
    /// Working-tree and index diffs go to the system git in an LFS
    /// repository (same reason as `status`); commit-to-commit diffs
    /// compare trees, never the working tree, so libgit2 keeps them.
    fn diff(&self, repo: &RepoRef, target: DiffTarget) -> Result<Vec<FileDiff>> {
        if touches_worktree_or_index(&target) && lfs::is_active(repo) {
            self.fallback.diff(repo, target)
        } else {
            reads::diff(repo, target)
        }
    }
    fn diff_files(&self, repo: &RepoRef, target: DiffTarget) -> Result<DiffFileList> {
        if touches_worktree_or_index(&target) && lfs::is_active(repo) {
            self.fallback.diff_files(repo, target)
        } else {
            reads::diff_files(repo, target)
        }
    }
    fn commit_stats(&self, repo: &RepoRef, ids: &[Oid]) -> Result<Vec<CommitStat>> {
        reads::commit_stats(repo, ids)
    }
    fn commit_messages(
        &self,
        repo: &RepoRef,
        query: &gitpoulpi_core::domain::CommitMessagesQuery,
    ) -> Result<gitpoulpi_core::domain::CommitMessages> {
        reads::commit_messages(repo, query)
    }
    fn diff_paths(
        &self,
        repo: &RepoRef,
        target: DiffTarget,
        paths: &[PathBuf],
    ) -> Result<Vec<FileDiff>> {
        if touches_worktree_or_index(&target) && lfs::is_active(repo) {
            self.fallback.diff_paths(repo, target, paths)
        } else {
            reads::diff_paths(repo, target, paths)
        }
    }
    fn refs(&self, repo: &RepoRef) -> Result<Vec<Ref>> {
        reads::refs(repo)
    }
    fn ref_snapshot(&self, repo: &RepoRef, scope: &RefScope) -> Result<RefSnapshot> {
        journal_ops::ref_snapshot(repo, scope)
    }
    fn update_refs(
        &self,
        repo: &RepoRef,
        updates: &[RefUpdate],
        reflog_message: &str,
    ) -> Result<()> {
        journal_ops::update_refs(repo, updates, reflog_message)
    }
    /// Native: `index_tree` / `index_restore` are what every staging
    /// operation's undo records, so they must cost no process.
    fn index_tree(&self, repo: &RepoRef) -> Result<Oid> {
        journal_ops::index_tree(repo)
    }
    fn index_restore(&self, repo: &RepoRef, tree: Oid) -> Result<()> {
        journal_ops::index_restore(repo, tree)
    }
    /// Native: the snapshot builder and restorer (raw bytes, no filters,
    /// symlinks, the executable bit) cost no process.
    fn snapshot_create(&self, repo: &RepoRef, paths: &[String]) -> Result<Snapshot> {
        snapshot_ops::snapshot_create(repo, paths)
    }
    fn snapshot_restore(&self, repo: &RepoRef, snapshot: Oid, paths: &[String]) -> Result<()> {
        snapshot_ops::snapshot_restore(repo, snapshot, paths)
    }
    fn path_hashes(&self, repo: &RepoRef, paths: &[String]) -> Result<Vec<PathHash>> {
        snapshot_ops::path_hashes(repo, paths)
    }
    fn blob_bytes(&self, repo: &RepoRef, oid: Oid) -> Result<Vec<u8>> {
        reflog_ops::blob_bytes(repo, oid)
    }
    fn reflog(
        &self,
        repo: &RepoRef,
        refname: &str,
        skip: usize,
        limit: usize,
    ) -> Result<gitpoulpi_core::domain::ReflogPage> {
        reflog_ops::reflog(repo, refname, skip, limit)
    }
    fn config_section(&self, repo: &RepoRef, section: &str) -> Result<Vec<(String, String)>> {
        journal_ops::config_section(repo, section)
    }
    fn remote_dependents(&self, repo: &RepoRef, remote: &str) -> Result<Vec<String>> {
        journal_ops::remote_dependents(repo, remote)
    }
    fn set_config_section(
        &self,
        repo: &RepoRef,
        section: &str,
        entries: &[(String, String)],
    ) -> Result<()> {
        journal_ops::set_config_section(repo, section, entries)
    }
    fn stashes(&self, repo: &RepoRef) -> Result<Vec<Stash>> {
        reads::stashes(repo)
    }
    fn remotes(&self, repo: &RepoRef) -> Result<Vec<Remote>> {
        reads::remotes(repo)
    }
    fn worktrees(&self, repo: &RepoRef) -> Result<Vec<Worktree>> {
        if cfg!(feature = "worktree") {
            reads::worktrees(repo)
        } else {
            Err(gitpoulpi_core::Error::NotBuilt {
                feature: "worktree",
            })
        }
    }
    fn submodules(&self, repo: &RepoRef) -> Result<Vec<Submodule>> {
        #[cfg(feature = "submodule")]
        {
            reads::submodules(repo)
        }
        #[cfg(not(feature = "submodule"))]
        {
            let _ = repo;
            Err(gitpoulpi_core::Error::NotBuilt {
                feature: "submodule",
            })
        }
    }
    fn resolve(&self, repo: &RepoRef, rev: &str) -> Result<Oid> {
        reads::resolve(repo, rev)
    }
    fn merge_base(&self, repo: &RepoRef, a: Oid, b: Oid) -> Result<Option<Oid>> {
        reads::merge_base(repo, a, b)
    }
    fn ahead_behind(&self, repo: &RepoRef, tip: Oid, base: Oid) -> Result<(u32, u32)> {
        reads::ahead_behind(repo, tip, base)
    }
    // libgit2 has no patch-id: the system git answers.
    fn cherry_counts(&self, repo: &RepoRef, local: Oid, remote: Oid) -> Result<(u32, u32)> {
        self.fallback.cherry_counts(repo, local, remote)
    }
    fn predict_merge_conflicts(
        &self,
        repo: &RepoRef,
        ours: Oid,
        theirs: Oid,
    ) -> Result<Vec<String>> {
        predict::predict_merge_conflicts(repo, ours, theirs)
    }
    fn read_blob(&self, repo: &RepoRef, path: &Path, source: &BlobSource) -> Result<Blob> {
        reads::read_blob(repo, path, source)
    }
    fn read_blob_bytes(
        &self,
        repo: &RepoRef,
        path: &Path,
        source: &BlobSource,
        max: u64,
    ) -> Result<gitpoulpi_core::domain::BlobBytes> {
        reads::read_blob_bytes(repo, path, source, max)
    }
    fn list_tree(&self, repo: &RepoRef, at: Oid) -> Result<Vec<PathBuf>> {
        reads::list_tree(repo, at)
    }
    fn grep_tree(
        &self,
        repo: &RepoRef,
        at: Oid,
        terms: &[String],
        max_hits: usize,
    ) -> Result<Vec<gitpoulpi_core::domain::GrepHit>> {
        reads::grep_tree(repo, at, terms, max_hits)
    }
    fn mailmap_resolve(
        &self,
        repo: &RepoRef,
        idents: &[gitpoulpi_core::domain::rewrite::Ident],
    ) -> Result<Vec<gitpoulpi_core::domain::rewrite::Ident>> {
        reads::mailmap_resolve(repo, idents)
    }
    fn blame(&self, repo: &RepoRef, path: &Path, at: Oid) -> Result<Blame> {
        reads::blame(repo, path, at)
    }
    /// libgit2 has no `--follow`; re-implementing git's rename-chasing walk
    /// would make the backends disagree exactly where this feature matters,
    /// so it is one `git log` process either way.
    fn file_history(
        &self,
        repo: &RepoRef,
        path: &Path,
        start: Oid,
        skip: usize,
        limit: usize,
    ) -> Result<gitpoulpi_core::domain::FileHistoryPage> {
        self.fallback.file_history(repo, path, start, skip, limit)
    }
    /// File / Folder / All run on libgit2's index (no spawn, one index
    /// write); Hunk / Line and every operation in an LFS repository (no
    /// LFS clean filter in libgit2, ADR 0009) go to the system git.
    fn stage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()> {
        if !staging::handles(target) || lfs::is_active(repo) {
            return self.fallback.stage(repo, target);
        }
        match staging::stage(repo, target) {
            Err(gitpoulpi_core::Error::Unsupported { .. }) => self.fallback.stage(repo, target),
            other => other,
        }
    }
    fn unstage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()> {
        if !staging::handles(target) || lfs::is_active(repo) {
            return self.fallback.unstage(repo, target);
        }
        match staging::unstage(repo, target) {
            Err(gitpoulpi_core::Error::Unsupported { .. }) => self.fallback.unstage(repo, target),
            other => other,
        }
    }
    fn commit(&self, repo: &RepoRef, message: &str, opts: &CommitOptions) -> Result<Oid> {
        // A commit that concludes a merge / cherry-pick / revert has to
        // read MERGE_HEAD / CHERRY_PICK_HEAD / REVERT_HEAD (second parent,
        // picked author) and clear that state afterwards; the system git
        // does exactly that, so these (rare) commits are delegated to it,
        // the same way merge, cherry-pick and revert themselves are.
        // The same goes for a repository with commit hooks: libgit2 runs
        // none, so the system git commits (and honours `no_verify`).
        if mutate::has_pending_sequencer_commit(repo) || mutate::has_commit_hooks(repo) {
            return self.fallback.commit(repo, message, opts);
        }
        mutate::commit(repo, message, opts)
    }
    /// Writes the working tree, so in an LFS repository it goes to the
    /// system git (which runs the smudge filter); libgit2 would write
    /// pointer text (ADR 0009).
    fn checkout(&self, repo: &RepoRef, target: &CheckoutTarget) -> Result<()> {
        if lfs::is_active(repo) {
            return self.fallback.checkout(repo, target);
        }
        match mutate::checkout(repo, target) {
            Err(gitpoulpi_core::Error::Unsupported { .. }) => self.fallback.checkout(repo, target),
            other => other,
        }
    }
    // Native on the index (`staging::discard`), except in an LFS repository
    // where the checkout needs the smudge filter.
    fn discard(&self, repo: &RepoRef, paths: &[String]) -> Result<()> {
        if lfs::is_active(repo) {
            return self.fallback.discard(repo, paths);
        }
        staging::discard(repo, paths)
    }
    fn branch_create(&self, repo: &RepoRef, name: &str, at: Oid) -> Result<()> {
        mutate::branch_create(repo, name, at)
    }
    fn branch_create_tracking(
        &self,
        repo: &RepoRef,
        local_name: &str,
        upstream: &str,
    ) -> Result<()> {
        if lfs::is_active(repo) {
            return self
                .fallback
                .branch_create_tracking(repo, local_name, upstream);
        }
        mutate::branch_create_tracking(repo, local_name, upstream)
    }
    fn branch_delete(&self, repo: &RepoRef, name: &str) -> Result<()> {
        mutate::branch_delete(repo, name)
    }
    fn branch_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
        mutate::branch_rename(repo, old_name, new_name)
    }
    fn branch_set_upstream(
        &self,
        repo: &RepoRef,
        branch: &str,
        upstream: Option<&str>,
    ) -> Result<()> {
        mutate::branch_set_upstream(repo, branch, upstream)
    }
    fn fast_forward(
        &self,
        repo: &RepoRef,
        branch: &str,
        target: Oid,
    ) -> Result<FastForwardOutcome> {
        if lfs::is_active(repo) {
            return self.fallback.fast_forward(repo, branch, target);
        }
        mutate::fast_forward(repo, branch, target)
    }
    fn tag_create(
        &self,
        repo: &RepoRef,
        name: &str,
        at: Oid,
        annotated_message: Option<&str>,
    ) -> Result<()> {
        mutate::tag_create(repo, name, at, annotated_message)
    }
    fn tag_delete(&self, repo: &RepoRef, name: &str) -> Result<()> {
        mutate::tag_delete(repo, name)
    }
    fn tag_annotate(&self, repo: &RepoRef, name: &str, message: &str) -> Result<()> {
        mutate::tag_annotate(repo, name, message)
    }
    fn tag_move(&self, repo: &RepoRef, name: &str, to: Oid, force: bool) -> Result<()> {
        mutate::tag_move(repo, name, to, force)
    }
    fn tag_merge_branch(
        &self,
        repo: &RepoRef,
        name: &str,
        branch: &str,
        message: Option<&str>,
    ) -> Result<Oid> {
        mutate::tag_merge_branch(repo, name, branch, message)
    }
    /// Forwarded, like every other network operation here.
    ///
    /// Pushing a tag also has to *record* the push in the remote-tag
    /// mirror, and that bookkeeping is what makes the backends agree about
    /// which tags are unpushed. The mirror therefore has exactly one
    /// writer, in `gitpoulpi_cli_backend::tags`; this backend only *reads*
    /// it (natively, in `reads::refs`).
    async fn tag_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        selection: &TagSelection,
    ) -> Result<PushOutcome> {
        self.fallback.tag_push(repo, remote, selection).await
    }
    fn tag_take_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        self.fallback.tag_take_remote(repo, remote, name)
    }
    async fn tag_force_push(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        self.fallback.tag_force_push(repo, remote, name).await
    }
    async fn tag_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        self.fallback.tag_delete_remote(repo, remote, name).await
    }
    async fn tag_fetch(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        self.fallback.tag_fetch(repo, remote, name).await
    }
    async fn branch_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        self.fallback.branch_delete_remote(repo, remote, name).await
    }
    async fn remote_ref_value(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
    ) -> Result<Option<Oid>> {
        self.fallback.remote_ref_value(repo, remote, full_ref).await
    }
    async fn remote_ref_update(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
        new: Option<Oid>,
        expect: Option<Oid>,
    ) -> Result<()> {
        self.fallback
            .remote_ref_update(repo, remote, full_ref, new, expect)
            .await
    }
    fn rebase_state(&self, repo: &RepoRef) -> Result<Option<RebaseState>> {
        // Part of every refresh: answered natively in the (overwhelmingly
        // common) no-rebase case; the system backend reads the state
        // files when one is in progress.
        if reads::provably_no_rebase(repo) {
            return Ok(None);
        }
        self.fallback.rebase_state(repo)
    }
    /// `Soft` only moves the branch, so it stays native; `Mixed` and
    /// `Hard` rewrite the index / working tree and go to the system git in
    /// an LFS repository (ADR 0009).
    fn reset(&self, repo: &RepoRef, target: Oid, mode: ResetMode) -> Result<()> {
        if mode != ResetMode::Soft && lfs::is_active(repo) {
            return self.fallback.reset(repo, target, mode);
        }
        mutate::reset(repo, target, mode)
    }
    fn reword(&self, repo: &RepoRef, commit: Oid, message: &str) -> Result<RewordOutcome> {
        reword::reword(repo, commit, message)
    }
    fn amend_commit(
        &self,
        repo: &RepoRef,
        commit: Oid,
        amend: &gitpoulpi_core::domain::CommitAmend,
    ) -> Result<RewordOutcome> {
        reword::amend_commit(repo, commit, amend, "amend")
    }
    /// System git: the byte-faithful partial patches live in the CLI
    /// backend's `patch.rs`, as for hunk and line staging.
    fn split_source(&self, repo: &RepoRef, commit: Oid) -> Result<SplitSource> {
        self.fallback.split_source(repo, commit)
    }
    fn split_commit(&self, repo: &RepoRef, req: &SplitCommit) -> Result<SplitOutcome> {
        self.fallback.split_commit(repo, req)
    }
    // One engine for both backends (the split-commit precedent).
    fn rewrite_preview(
        &self,
        repo: &RepoRef,
        recipe: &Recipe,
        progress: RewriteProgressSink<'_>,
    ) -> Result<RewritePlan> {
        self.fallback.rewrite_preview(repo, recipe, progress)
    }
    fn rewrite_apply(&self, repo: &RepoRef, plan: &RewritePlan) -> Result<RewriteOutcome> {
        self.fallback.rewrite_apply(repo, plan)
    }
    fn rewrite_extract(
        &self,
        repo: &RepoRef,
        plan: &RewritePlan,
        target: &std::path::Path,
    ) -> Result<RewriteOutcome> {
        self.fallback.rewrite_extract(repo, plan, target)
    }
    fn rewrite_load(&self, repo: &RepoRef, run: &RunId) -> Result<Option<RewritePlan>> {
        self.fallback.rewrite_load(repo, run)
    }
    fn rewrite_record(
        &self,
        repo: &RepoRef,
        run: &RunId,
    ) -> Result<gitpoulpi_core::domain::rewrite::RewriteRecord> {
        self.fallback.rewrite_record(repo, run)
    }
    fn rewrite_maps(
        &self,
        repo: &RepoRef,
        run: &RunId,
        out: Option<&std::path::Path>,
    ) -> Result<gitpoulpi_core::domain::rewrite::RewriteMaps> {
        self.fallback.rewrite_maps(repo, run, out)
    }
    async fn push_rewritten(
        &self,
        repo: &RepoRef,
        req: &gitpoulpi_core::domain::rewrite::PushRewritten,
    ) -> Result<gitpoulpi_core::domain::rewrite::PushRewrittenOutcome> {
        self.fallback.push_rewritten(repo, req).await
    }
    fn rewrite_purge(
        &self,
        repo: &RepoRef,
        req: &gitpoulpi_core::domain::rewrite::PurgeRequest,
        progress: gitpoulpi_core::domain::rewrite::PurgeProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::PurgeOutcome> {
        self.fallback.rewrite_purge(repo, req, progress)
    }
    fn rewrite_blob_diff(
        &self,
        repo: &RepoRef,
        run: &RunId,
        commit: Oid,
        path: &str,
    ) -> Result<gitpoulpi_core::domain::rewrite::BlobChange> {
        self.fallback.rewrite_blob_diff(repo, run, commit, path)
    }
    fn rewrite_script_check(
        &self,
        repo: &RepoRef,
        script: &gitpoulpi_core::domain::rewrite::ScriptRef,
        policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
    ) -> Result<gitpoulpi_core::domain::rewrite::script::ScriptCheck> {
        self.fallback.rewrite_script_check(repo, script, policy)
    }
    fn rewrite_script_test(
        &self,
        repo: &RepoRef,
        script: &gitpoulpi_core::domain::rewrite::ScriptRef,
        policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
        commits: &[Oid],
    ) -> Result<Vec<gitpoulpi_core::domain::rewrite::script::ScriptTestRow>> {
        self.fallback
            .rewrite_script_test(repo, script, policy, commits)
    }
    fn rewrite_analyze(
        &self,
        repo: &RepoRef,
        scope: &gitpoulpi_core::domain::rewrite::Scope,
        top: u32,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::SizeAnalysis> {
        self.fallback.rewrite_analyze(repo, scope, top, progress)
    }
    fn rewrite_scan_secrets(
        &self,
        repo: &RepoRef,
        scope: &gitpoulpi_core::domain::rewrite::Scope,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::SecretScan> {
        self.fallback.rewrite_scan_secrets(repo, scope, progress)
    }
    fn rewrite_discard(&self, repo: &RepoRef, run: &RunId) -> Result<()> {
        self.fallback.rewrite_discard(repo, run)
    }
    /// Networking is CliFallback, like `fetch`: the system-git layer is where
    /// the entry is applied.
    fn set_network_auth(&self, repo: &RepoRef, auth: Option<NetworkAuth>) {
        self.fallback.set_network_auth(repo, auth);
    }
    async fn remote_access_check(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &RemoteAccessOptions,
    ) -> Result<RemoteAccessReport> {
        self.fallback.remote_access_check(repo, remote, opts).await
    }
    async fn fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &FetchOptions,
    ) -> Result<FetchOutcome> {
        self.fallback.fetch(repo, remote, opts).await
    }
    async fn fetch_all(&self, repo: &RepoRef, opts: &FetchOptions) -> Result<FetchOutcome> {
        self.fallback.fetch_all(repo, opts).await
    }
    async fn fetch_ref_as_branch(
        &self,
        repo: &RepoRef,
        remote: &str,
        remote_ref: &str,
        branch: &str,
    ) -> Result<FetchOutcome> {
        self.fallback
            .fetch_ref_as_branch(repo, remote, remote_ref, branch)
            .await
    }
    async fn fetch_refspec(
        &self,
        repo: &RepoRef,
        remote: &str,
        refspec: &str,
    ) -> Result<FetchOutcome> {
        self.fallback.fetch_refspec(repo, remote, refspec).await
    }
    async fn pull(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
        self.fallback.pull(repo, opts).await
    }
    async fn pull_fetch(&self, repo: &RepoRef, opts: &PullOptions) -> Result<()> {
        self.fallback.pull_fetch(repo, opts).await
    }
    async fn pull_integrate(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
        self.fallback.pull_integrate(repo, opts).await
    }
    async fn push(&self, repo: &RepoRef, remote: &str, opts: &PushOptions) -> Result<PushOutcome> {
        self.fallback.push(repo, remote, opts).await
    }
    async fn push_for_review(
        &self,
        repo: &RepoRef,
        remote: &str,
        branch: &str,
        topic: Option<&str>,
    ) -> Result<PushOutcome> {
        self.fallback
            .push_for_review(repo, remote, branch, topic)
            .await
    }
    async fn clone(&self, url: &str, dest: &Path, opts: &CloneOptions) -> Result<RepoRef> {
        self.fallback.clone(url, dest, opts).await
    }
    /// Forwarded rather than reimplemented on libgit2's own
    /// `RemoteCallbacks::transfer_progress`: `clone` is already
    /// `CliFallback`, and narrating through libgit2 would give the same
    /// operation two progress vocabularies depending on the backend. If
    /// `clone` ever becomes Native, the progress mapping moves with it.
    async fn clone_with_progress(
        &self,
        url: &str,
        dest: &Path,
        opts: &CloneOptions,
        progress: CloneProgressSink<'_>,
    ) -> Result<RepoRef> {
        self.fallback
            .clone_with_progress(url, dest, opts, progress)
            .await
    }
    fn remote_set_url(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()> {
        if !mutate::remote_has_url(repo, name)? {
            return self.fallback.remote_set_url(repo, name, url);
        }
        mutate::remote_set_url(repo, name, url)
    }
    fn remote_set_push_url(&self, repo: &RepoRef, name: &str, url: Option<&str>) -> Result<()> {
        if !mutate::remote_has_url(repo, name)? {
            return self.fallback.remote_set_push_url(repo, name, url);
        }
        mutate::remote_set_push_url(repo, name, url)
    }
    fn ssh_command(&self, repo: &RepoRef) -> Result<Option<String>> {
        mutate::ssh_command(repo)
    }
    fn set_ssh_command(&self, repo: &RepoRef, command: Option<&str>) -> Result<()> {
        mutate::set_ssh_command(repo, command)
    }
    fn repo_config(&self, repo: &RepoRef, key: &str) -> Result<Option<String>> {
        mutate::repo_config(repo, key)
    }
    // libgit2 has no sparse-checkout API (it cannot even write the skip-worktree
    // bits back reliably), so all three are `git sparse-checkout`.
    fn sparse_checkout_status(&self, repo: &RepoRef) -> Result<SparseState> {
        self.fallback.sparse_checkout_status(repo)
    }
    fn sparse_checkout_set(&self, repo: &RepoRef, dirs: &[String]) -> Result<()> {
        self.fallback.sparse_checkout_set(repo, dirs)
    }
    fn sparse_checkout_disable(&self, repo: &RepoRef) -> Result<()> {
        self.fallback.sparse_checkout_disable(repo)
    }
    fn set_repo_config(&self, repo: &RepoRef, key: &str, value: Option<&str>) -> Result<()> {
        mutate::set_repo_config(repo, key, value)
    }
    fn config_entries(&self, repo: &RepoRef, prefixes: &[&str]) -> Result<Vec<(String, String)>> {
        mutate::config_entries(repo, prefixes)
    }
    fn lfs_files(&self, repo: &RepoRef, at: Option<Oid>) -> Result<Vec<LfsFile>> {
        lfs::lfs_files(repo, at)
    }
    async fn lfs_locks(&self, repo: &RepoRef, remote: &str) -> Result<Vec<LfsLock>> {
        self.fallback.lfs_locks(repo, remote).await
    }
    async fn lfs_lock(&self, repo: &RepoRef, remote: &str, path: &Path) -> Result<LfsLock> {
        self.fallback.lfs_lock(repo, remote, path).await
    }
    async fn lfs_unlock(
        &self,
        repo: &RepoRef,
        remote: &str,
        path: &Path,
        force: bool,
    ) -> Result<()> {
        self.fallback.lfs_unlock(repo, remote, path, force).await
    }
    async fn lfs_fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsFetchOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        self.fallback.lfs_fetch(repo, remote, opts, progress).await
    }
    async fn lfs_pull(
        &self,
        repo: &RepoRef,
        remote: &str,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        self.fallback.lfs_pull(repo, remote, progress).await
    }
    async fn lfs_checkout(
        &self,
        repo: &RepoRef,
        paths: &[PathBuf],
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        self.fallback.lfs_checkout(repo, paths, progress).await
    }
    async fn lfs_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsPushOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        self.fallback.lfs_push(repo, remote, opts, progress).await
    }
    fn conflicts(&self, repo: &RepoRef) -> Result<Vec<ConflictSet>> {
        // Part of every refresh: an index with no unmerged entries has
        // nothing for `git diff --diff-filter=U` to report, so skip the
        // process spawn; anything else goes to git as before.
        if reads::provably_no_conflicts(repo) {
            return Ok(Vec::new());
        }
        self.fallback.conflicts(repo)
    }
    /// Native: libgit2 lists the index's unmerged entries without a
    /// process or a blob read.
    fn conflict_entries(&self, repo: &RepoRef) -> Result<Vec<ConflictEntry>> {
        reads::conflict_entries(repo)
    }
    /// Native stage reads; the 3-way merge of them is `CliFallback`
    /// (see `reads::merge_document`).
    fn merge_document(&self, repo: &RepoRef, path: &Path) -> Result<MergeDocument> {
        reads::merge_document(repo, path)
    }

    forward_to_fallback! {
        /// `CliFallback`: the reachability walk is the CLI's own, one implementation.
        fn lost_commits(
            &self,
            repo: &RepoRef,
            opts: &gitpoulpi_core::domain::LostCommitOptions,
        ) -> Result<gitpoulpi_core::domain::LostCommitsPage>;
        /// `CliFallback`: libgit2 has no `reset --keep`; one implementation,
        /// the CLI's.
        fn move_head_keep(&self, repo: &RepoRef, branch: &str, from: Oid, to: Oid) -> Result<()>;
        fn delete_files(&self, repo: &RepoRef, paths: &[String]) -> Result<()>;
        /// `CliFallback`: plain file I/O over the shared `textconv` functions,
        /// one implementation so both backends write the same bytes.
        fn convert_files(
            &self,
            repo: &RepoRef,
            paths: &[String],
            op: gitpoulpi_core::textconv::ConvertOp,
        ) -> Result<gitpoulpi_core::textconv::ConvertReport>;
        fn gitignore_add(&self, repo: &RepoRef, pattern: &str, file: Option<&str>) -> Result<bool>;
        /// `CliFallback`: the one implementation writes the attribute files and
        /// asks `git check-attr`, so both backends answer alike.
        fn gitattributes_treat(
            &self,
            repo: &RepoRef,
            paths: &[String],
            scope: gitpoulpi_core::domain::RuleScope,
            file: Option<&str>,
            treat: gitpoulpi_core::domain::Treat,
        ) -> Result<gitpoulpi_core::domain::AttributesWrite>;
        fn binary_paths(&self, repo: &RepoRef, paths: &[String]) -> Result<Vec<bool>>;
        /// `CliFallback`: one `git log` counts the identities of a scope, and
        /// the `.mailmap` edit is plain file I/O; one implementation each.
        fn mailmap_identities(
            &self,
            repo: &RepoRef,
            scope: &gitpoulpi_core::domain::rewrite::Scope,
            options: &gitpoulpi_core::domain::rewrite::ScopeOptions,
        ) -> Result<gitpoulpi_core::domain::rewrite::IdentityList>;
        fn mailmap_add(
            &self,
            repo: &RepoRef,
            canonical: &gitpoulpi_core::domain::rewrite::Ident,
            from: &[gitpoulpi_core::domain::rewrite::Ident],
            file: Option<&str>,
        ) -> Result<gitpoulpi_core::domain::rewrite::MailmapAdded>;
        fn patch_files(&self, repo: &RepoRef, patch: &Path) -> Result<Vec<String>>;
        // `CliFallback` for the same reason hunk- and line-level staging are:
        // it is a partial patch reverse-applied with `git apply`, and the
        // patch builder that gets CRLF and a missing final newline right
        // lives in the CLI backend (`patch.rs`) -- one builder, not two.
        fn discard_part(&self, repo: &RepoRef, target: &DiscardTarget) -> Result<()>;
        /// `CliFallback`: libgit2's `worktree_add` cannot start a worktree on
        /// a new branch and a commit in one step, does not create missing
        /// parent folders, and names the admin entry differently from `git
        /// worktree add` -- three ways for the two backends to disagree about
        /// the same worktree. One implementation, the CLI's.
        /// `CliFallback`: `submodule init`/`update` clone and fetch through
        /// git's own transports and credentials; libgit2's submodule update
        /// needs callbacks for all of it. One implementation, the CLI's.
        fn submodule_init(&self, repo: &RepoRef, paths: &[String]) -> Result<()>;
        fn submodule_update(&self, repo: &RepoRef, opts: &SubmoduleUpdateOptions) -> Result<()>;
        fn worktree_add(
            &self,
            repo: &RepoRef,
            path: &Path,
            opts: &WorktreeAddOptions,
        ) -> Result<Worktree>;
        fn worktree_changes(&self, repo: &RepoRef, path: &Path) -> Result<Vec<WorktreeChange>>;
        fn worktree_remove(
            &self,
            repo: &RepoRef,
            path: &Path,
            opts: &WorktreeRemoveOptions,
        ) -> Result<WorktreeRemoved>;
        /// `CliFallback`: `git format-patch`/`git diff --binary`/`git apply`
        /// are the sources of truth for a patch's exact bytes (base85 binary
        /// hunks, CRLF preservation); one implementation, the CLI's.
        fn patch_create(
            &self,
            repo: &RepoRef,
            source: &PatchSource,
            dest: &Path,
        ) -> Result<PatchCreated>;
        fn patch_apply(&self, repo: &RepoRef, patch: &Path) -> Result<PatchApplied>;
        fn apply_commit_to_worktree(&self, repo: &RepoRef, commit: Oid) -> Result<PatchApplied>;
        fn merge(&self, repo: &RepoRef, from: &Ref, opts: &MergeOptions) -> Result<MergeOutcome>;
        fn rebase(&self, repo: &RepoRef, onto: &Ref, opts: &RebaseOptions) -> Result<RebaseOutcome>;
        fn rebase_plan(&self, repo: &RepoRef, upstream: Option<Oid>) -> Result<RebasePlanDraft>;
        fn rebase_interactive(
            &self,
            repo: &RepoRef,
            req: &InteractiveRebase,
        ) -> Result<RebaseOutcome>;
        fn rebase_continue(&self, repo: &RepoRef) -> Result<RebaseOutcome>;
        fn rebase_skip(&self, repo: &RepoRef) -> Result<RebaseOutcome>;
        fn rebase_abort(&self, repo: &RepoRef) -> Result<()>;
        fn cherry_pick(
            &self,
            repo: &RepoRef,
            commits: &[Oid],
            opts: &CherryPickOptions,
        ) -> Result<CherryPickOutcome>;
        fn revert(&self, repo: &RepoRef, commit: Oid, mainline: Option<u32>) -> Result<RevertOutcome>;
        fn stash_push(
            &self,
            repo: &RepoRef,
            message: Option<&str>,
            include_untracked: bool,
        ) -> Result<Option<StashId>>;
        fn stash_push_paths(
            &self,
            repo: &RepoRef,
            message: Option<&str>,
            paths: &[String],
        ) -> Result<Option<StashId>>;
        fn stash_pop(&self, repo: &RepoRef, id: StashId) -> Result<()>;
        fn stash_apply(&self, repo: &RepoRef, id: StashId) -> Result<()>;
        fn stash_drop(&self, repo: &RepoRef, id: StashId) -> Result<()>;
        fn stash_store(&self, repo: &RepoRef, commit: Oid, message: &str) -> Result<()>;
        fn stash_from_commit(
            &self,
            repo: &RepoRef,
            commit: Oid,
            message: Option<&str>,
        ) -> Result<Oid>;
        fn stash_paths(&self, repo: &RepoRef, commit: Oid) -> Result<Vec<String>>;
        fn remote_add(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()>;
        fn remote_remove(&self, repo: &RepoRef, name: &str) -> Result<()>;
        fn leftover_remotes(&self, repo: &RepoRef) -> Result<Vec<String>>;
        /// Forwarded: a rename also has to move the remote-tag mirror, and
        /// the mirror has exactly one writer (see `tag_push`).
        fn remote_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()>;
        fn lfs_status(&self, repo: &RepoRef) -> Result<LfsRepoStatus>;
        fn rewrite_tools(&self, repo: &RepoRef, fresh: bool) -> Result<gitpoulpi_core::domain::rewrite::RewriteTools>;
        fn rewrite_check_signing(&self, repo: &RepoRef) -> Result<gitpoulpi_core::domain::rewrite::SigningCheck>;
        fn lfs_install(&self, repo: &RepoRef, scope: LfsInstallScope) -> Result<()>;
        fn lfs_track(
            &self,
            repo: &RepoRef,
            patterns: &[String],
            opts: &LfsTrackOptions,
        ) -> Result<Vec<LfsPattern>>;
        fn lfs_untrack(&self, repo: &RepoRef, patterns: &[String]) -> Result<Vec<LfsPattern>>;
        fn lfs_prune(&self, repo: &RepoRef) -> Result<()>;
        /// `CliFallback`: the resolution writes files, restores stages and
        /// shells the same `git` plumbing either way; one implementation
        /// is the identity guarantee.
        fn resolve_conflict(
            &self,
            repo: &RepoRef,
            path: &Path,
            resolution: ConflictResolution,
            opts: &ResolveOptions,
        ) -> Result<()>;
        fn unresolve_conflict(&self, repo: &RepoRef, path: &Path, entry: &ConflictEntry)
            -> Result<()>;
        fn operation_state(&self, repo: &RepoRef) -> Result<Option<OperationState>>;
        fn operation_abort(&self, repo: &RepoRef) -> Result<()>;
        fn am_continue(&self, repo: &RepoRef) -> Result<AmOutcome>;
        fn am_skip(&self, repo: &RepoRef) -> Result<AmOutcome>;
        fn am_abort(&self, repo: &RepoRef) -> Result<()>;
        fn index_lock(&self, repo: &RepoRef) -> Result<Option<IndexLock>>;
        fn index_lock_remove(&self, repo: &RepoRef) -> Result<()>;
        fn operation_skip(&self, repo: &RepoRef) -> Result<()>;
    }
}
