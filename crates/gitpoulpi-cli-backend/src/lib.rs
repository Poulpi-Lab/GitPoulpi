//! GitBackend implementation driving an installed system `git` binary.
//!
//! Used both standalone (user-selectable `default_backend = "system"`) and
//! as the CLI-fallback target inside `Git2Backend` for operations git2
//! doesn't support well. Parses only machine-readable git output
//! (`--porcelain=v2 -z`, NUL-delimited log formats) -- never human text.

mod branch;
mod cancel_cleanup;
mod checkout;
mod commit;
mod conflicts;
mod content;
mod convert_ops;
mod count;
mod diff;
mod discard;
mod discover;
mod file_history;
mod history;
mod history_ops;
mod index_scan;
mod journal_ops;
#[cfg(feature = "lfs")]
mod lfs;
#[cfg(not(feature = "lfs"))]
#[path = "lfs_stub.rs"]
mod lfs;
#[cfg(feature = "lfs")]
mod lfs_locks;
#[cfg(not(feature = "lfs"))]
#[path = "lfs_locks_stub.rs"]
mod lfs_locks;
#[cfg(feature = "lfs")]
mod lfs_network;
#[cfg(not(feature = "lfs"))]
#[path = "lfs_network_stub.rs"]
mod lfs_network;
mod log;
mod messages;
mod network;
mod operation;
mod patch;
mod patch_file;
mod predict;
mod process;
mod rebase_interactive;
mod reflog_ops;
mod refs;
mod remote_ref;
mod remotes;
mod reset;
mod reword;
mod rewrite;
mod rule_files;
mod snapshot_ops;
mod sparse;
mod split;
mod staging;
mod stash_ops;
mod stashes;
mod status;
mod stream;
mod stuck;
#[cfg(feature = "submodule")]
mod submodules;
#[cfg(not(feature = "submodule"))]
#[path = "submodules_stub.rs"]
mod submodules;
mod tags;
mod tools;
mod tree_kill;
mod worktree_list;
#[cfg(feature = "worktree")]
mod worktrees;
#[cfg(not(feature = "worktree"))]
#[path = "worktrees_stub.rs"]
mod worktrees;

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::rewrite::{
    Recipe, RewriteOutcome, RewritePlan, RewriteProgressSink, RunId,
};
use gitpoulpi_core::domain::*;
use gitpoulpi_core::{GitBackend, Result};

/// Which remotes are known to hold a tag at a given commit, given the
/// remote-tag mirror.
///
/// Re-exported because `Git2Backend` reads the same mirror refs through
/// libgit2 and must reach the identical verdict - the rule that a
/// differing oid does *not* count as pushed lives in exactly one place,
/// not once per backend.
pub use cancel_cleanup::remove_stale_transfer_files;
pub use content::{body_of, lfs_local_object};
pub use reflog_ops::reflog_via_git;
pub use refs::{apply_tag_sync, tag_sync_for};

/// The config-section rules of the undo journal, shared with
/// `Git2Backend` so the two accept exactly the same sections.
pub use journal_ops::{
    check_section as check_config_section, dependents_in as remote_dependents_in,
    in_section as config_key_in_section, is_key_section as is_config_key_section,
};

/// How network commands get HTTPS credentials; the daemon sets it from
/// `credentials.https_mode` before each network operation.
pub use process::{credential_mode, git_program, set_credential_mode, set_ssh_key, CredentialMode};

/// The part of `merge_document` after the stages are read, and the label
/// reader, shared with `Git2Backend` (which reads the stages natively).
pub use conflicts::{document_from_stages, git_dir};
/// Process-wide spawn counter (see `process`), the perf guard's currency.
pub use process::total_spawn_count;
pub use snapshot_ops::{
    normalize_all as snapshot_normalize_all, remove_any as snapshot_remove_any,
    walk as snapshot_walk, write_file as snapshot_write_file, MODE_LINK,
};

pub struct SystemGitBackend {
    /// Cached `git lfs version` (see `lfs::LfsProbe`).
    lfs_probe: lfs::LfsProbe,
    /// Cached probes of the programs a history rewrite may use.
    tool_probe: tools::ToolProbe,
}

impl SystemGitBackend {
    pub fn new() -> Self {
        Self {
            lfs_probe: lfs::LfsProbe::default(),
            tool_probe: tools::ToolProbe::default(),
        }
    }
}

impl SystemGitBackend {
    /// A backend that behaves as if `git-lfs` is not installed, whatever
    /// the machine has: every LFS operation that needs the program fails
    /// with `Error::LfsUnavailable`. A constructor rather than the
    /// `GITPOULPI_GIT_LFS` variable so tests running in parallel cannot
    /// affect each other.
    pub fn without_lfs() -> Self {
        Self {
            lfs_probe: lfs::LfsProbe::forced(LfsBinary::Missing),
            tool_probe: tools::ToolProbe::forced(
                gitpoulpi_core::domain::rewrite::Tool::GitLfs,
                gitpoulpi_core::domain::rewrite::ToolStatus::Missing,
            ),
        }
    }
}

impl Default for SystemGitBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitBackend for SystemGitBackend {
    fn name(&self) -> &'static str {
        "system"
    }

    fn discover(&self, path: &Path) -> Result<DiscoveredRepo> {
        discover::discover(path)
    }

    fn init(&self, path: &Path, opts: &InitOptions) -> Result<RepoRef> {
        discover::init(path, opts)
    }

    fn status(&self, repo: &RepoRef) -> Result<WorkingTree> {
        status::status(repo)
    }
    fn log(&self, repo: &RepoRef, opts: &LogOptions) -> Result<CommitGraph> {
        log::log(repo, opts)
    }
    fn count_commits(&self, repo: &RepoRef, opts: &LogOptions) -> Result<u64> {
        count::count_commits(repo, opts)
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
    fn move_head_keep(&self, repo: &RepoRef, branch: &str, from: Oid, to: Oid) -> Result<()> {
        journal_ops::move_head_keep(repo, branch, from, to)
    }
    fn index_tree(&self, repo: &RepoRef) -> Result<Oid> {
        snapshot_ops::index_tree(repo)
    }
    fn index_restore(&self, repo: &RepoRef, tree: Oid) -> Result<()> {
        snapshot_ops::index_restore(repo, tree)
    }
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
    fn lost_commits(
        &self,
        repo: &RepoRef,
        opts: &gitpoulpi_core::domain::LostCommitOptions,
    ) -> Result<gitpoulpi_core::domain::LostCommitsPage> {
        reflog_ops::lost_commits(repo, opts)
    }
    fn delete_files(&self, repo: &RepoRef, paths: &[String]) -> Result<()> {
        snapshot_ops::delete_files(repo, paths)
    }
    fn convert_files(
        &self,
        repo: &RepoRef,
        paths: &[String],
        op: gitpoulpi_core::textconv::ConvertOp,
    ) -> Result<gitpoulpi_core::textconv::ConvertReport> {
        convert_ops::convert_files(repo, paths, op)
    }
    fn gitignore_add(&self, repo: &RepoRef, pattern: &str, file: Option<&str>) -> Result<bool> {
        rule_files::gitignore_add(repo, pattern, file)
    }
    fn gitattributes_treat(
        &self,
        repo: &RepoRef,
        paths: &[String],
        scope: gitpoulpi_core::domain::RuleScope,
        file: Option<&str>,
        treat: gitpoulpi_core::domain::Treat,
    ) -> Result<gitpoulpi_core::domain::AttributesWrite> {
        rule_files::gitattributes_treat(repo, paths, scope, file, treat)
    }
    fn mailmap_resolve(
        &self,
        repo: &RepoRef,
        idents: &[gitpoulpi_core::domain::rewrite::Ident],
    ) -> Result<Vec<gitpoulpi_core::domain::rewrite::Ident>> {
        history::mailmap_resolve(repo, idents)
    }
    fn mailmap_identities(
        &self,
        repo: &RepoRef,
        scope: &gitpoulpi_core::domain::rewrite::Scope,
        options: &gitpoulpi_core::domain::rewrite::ScopeOptions,
    ) -> Result<gitpoulpi_core::domain::rewrite::IdentityList> {
        history::mailmap_identities(repo, scope, options)
    }
    fn mailmap_add(
        &self,
        repo: &RepoRef,
        canonical: &gitpoulpi_core::domain::rewrite::Ident,
        from: &[gitpoulpi_core::domain::rewrite::Ident],
        file: Option<&str>,
    ) -> Result<gitpoulpi_core::domain::rewrite::MailmapAdded> {
        history::mailmap_add(repo, canonical, from, file)
    }
    fn binary_paths(&self, repo: &RepoRef, paths: &[String]) -> Result<Vec<bool>> {
        rule_files::binary_paths(repo, paths)
    }
    fn patch_files(&self, repo: &RepoRef, patch: &Path) -> Result<Vec<String>> {
        snapshot_ops::patch_files(repo, patch)
    }
    fn config_section(&self, repo: &RepoRef, section: &str) -> Result<Vec<(String, String)>> {
        journal_ops::config_section(repo, section)
    }
    fn config_sections(
        &self,
        repo: &RepoRef,
        sections: &[String],
    ) -> Result<Vec<gitpoulpi_core::backend::ConfigSection>> {
        journal_ops::config_sections(repo, sections)
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
    fn diff(&self, repo: &RepoRef, target: DiffTarget) -> Result<Vec<FileDiff>> {
        diff::diff(repo, target)
    }
    fn diff_files(&self, repo: &RepoRef, target: DiffTarget) -> Result<DiffFileList> {
        diff::diff_files(repo, target)
    }
    fn commit_stats(&self, repo: &RepoRef, ids: &[Oid]) -> Result<Vec<CommitStat>> {
        diff::commit_stats(repo, ids)
    }
    fn commit_messages(
        &self,
        repo: &RepoRef,
        query: &gitpoulpi_core::domain::CommitMessagesQuery,
    ) -> Result<gitpoulpi_core::domain::CommitMessages> {
        messages::commit_messages(repo, query)
    }
    fn diff_paths(
        &self,
        repo: &RepoRef,
        target: DiffTarget,
        paths: &[PathBuf],
    ) -> Result<Vec<FileDiff>> {
        diff::diff_paths(repo, target, paths)
    }
    fn refs(&self, repo: &RepoRef) -> Result<Vec<Ref>> {
        refs::refs(repo)
    }
    fn stashes(&self, repo: &RepoRef) -> Result<Vec<Stash>> {
        stashes::stashes(repo)
    }
    fn remotes(&self, repo: &RepoRef) -> Result<Vec<Remote>> {
        remotes::remotes(repo)
    }
    fn worktrees(&self, repo: &RepoRef) -> Result<Vec<Worktree>> {
        worktrees::worktrees(repo)
    }
    fn submodules(&self, repo: &RepoRef) -> Result<Vec<Submodule>> {
        submodules::submodules(repo)
    }
    fn submodule_init(&self, repo: &RepoRef, paths: &[String]) -> Result<()> {
        submodules::submodule_init(repo, paths)
    }
    fn submodule_update(&self, repo: &RepoRef, opts: &SubmoduleUpdateOptions) -> Result<()> {
        submodules::submodule_update(repo, opts)
    }
    fn resolve(&self, repo: &RepoRef, rev: &str) -> Result<Oid> {
        refs::resolve(repo, rev)
    }
    fn merge_base(&self, repo: &RepoRef, a: Oid, b: Oid) -> Result<Option<Oid>> {
        refs::merge_base(repo, a, b)
    }
    fn ahead_behind(&self, repo: &RepoRef, tip: Oid, base: Oid) -> Result<(u32, u32)> {
        refs::ahead_behind(repo, tip, base)
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
        content::read_blob(repo, path, source)
    }
    fn read_blob_bytes(
        &self,
        repo: &RepoRef,
        path: &Path,
        source: &BlobSource,
        max: u64,
    ) -> Result<gitpoulpi_core::domain::BlobBytes> {
        content::read_blob_bytes(repo, path, source, max)
    }
    fn list_tree(&self, repo: &RepoRef, at: Oid) -> Result<Vec<PathBuf>> {
        content::list_tree(repo, at)
    }

    fn blame(&self, repo: &RepoRef, path: &Path, at: Oid) -> Result<Blame> {
        content::blame(repo, path, at)
    }
    fn file_history(
        &self,
        repo: &RepoRef,
        path: &Path,
        start: Oid,
        skip: usize,
        limit: usize,
    ) -> Result<gitpoulpi_core::domain::FileHistoryPage> {
        file_history::file_history(repo, path, start, skip, limit)
    }
    fn stage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()> {
        staging::stage(repo, target)
    }
    fn unstage(&self, repo: &RepoRef, target: &StageTarget) -> Result<()> {
        staging::unstage(repo, target)
    }
    fn commit(&self, repo: &RepoRef, message: &str, opts: &CommitOptions) -> Result<Oid> {
        commit::commit(repo, message, opts)
    }
    fn checkout(&self, repo: &RepoRef, target: &CheckoutTarget) -> Result<()> {
        checkout::checkout(repo, target)
    }
    fn discard(&self, repo: &RepoRef, paths: &[String]) -> Result<()> {
        discard::discard(repo, paths)
    }
    fn discard_part(&self, repo: &RepoRef, target: &DiscardTarget) -> Result<()> {
        staging::discard_part(repo, target)
    }
    fn branch_create(&self, repo: &RepoRef, name: &str, at: Oid) -> Result<()> {
        branch::branch_create(repo, name, at)
    }
    fn branch_create_tracking(
        &self,
        repo: &RepoRef,
        local_name: &str,
        upstream: &str,
    ) -> Result<()> {
        branch::branch_create_tracking(repo, local_name, upstream)
    }
    fn branch_delete(&self, repo: &RepoRef, name: &str) -> Result<()> {
        branch::branch_delete(repo, name)
    }
    fn branch_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
        branch::branch_rename(repo, old_name, new_name)
    }
    fn branch_set_upstream(
        &self,
        repo: &RepoRef,
        branch: &str,
        upstream: Option<&str>,
    ) -> Result<()> {
        branch::branch_set_upstream(repo, branch, upstream)
    }
    fn fast_forward(
        &self,
        repo: &RepoRef,
        branch: &str,
        target: Oid,
    ) -> Result<FastForwardOutcome> {
        branch::fast_forward(repo, branch, target)
    }
    fn worktree_add(
        &self,
        repo: &RepoRef,
        path: &Path,
        opts: &WorktreeAddOptions,
    ) -> Result<Worktree> {
        worktrees::worktree_add(repo, path, opts)
    }
    fn patch_create(
        &self,
        repo: &RepoRef,
        source: &PatchSource,
        dest: &Path,
    ) -> Result<PatchCreated> {
        patch_file::patch_create(repo, source, dest)
    }
    fn patch_apply(&self, repo: &RepoRef, patch: &Path) -> Result<PatchApplied> {
        patch_file::patch_apply(repo, patch)
    }
    fn tag_create(
        &self,
        repo: &RepoRef,
        name: &str,
        at: Oid,
        annotated_message: Option<&str>,
    ) -> Result<()> {
        branch::tag_create(repo, name, at, annotated_message)
    }
    fn tag_delete(&self, repo: &RepoRef, name: &str) -> Result<()> {
        branch::tag_delete(repo, name)
    }
    fn tag_annotate(&self, repo: &RepoRef, name: &str, message: &str) -> Result<()> {
        tags::tag_annotate(repo, name, message)
    }
    async fn tag_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        selection: &TagSelection,
    ) -> Result<PushOutcome> {
        tags::tag_push(repo, remote, selection).await
    }
    async fn tag_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        tags::tag_delete_remote(repo, remote, name).await
    }
    async fn tag_fetch(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        tags::tag_fetch(repo, remote, name).await
    }
    async fn branch_delete_remote(&self, repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
        branch::branch_delete_remote(repo, remote, name).await
    }
    async fn remote_ref_value(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
    ) -> Result<Option<Oid>> {
        remote_ref::remote_ref_value(repo, remote, full_ref).await
    }
    async fn remote_ref_update(
        &self,
        repo: &RepoRef,
        remote: &str,
        full_ref: &str,
        new: Option<Oid>,
        expect: Option<Oid>,
    ) -> Result<()> {
        remote_ref::remote_ref_update(repo, remote, full_ref, new, expect).await
    }
    fn merge(&self, repo: &RepoRef, from: &Ref, opts: &MergeOptions) -> Result<MergeOutcome> {
        history_ops::merge(repo, from, opts)
    }
    fn rebase(&self, repo: &RepoRef, onto: &Ref, opts: &RebaseOptions) -> Result<RebaseOutcome> {
        history_ops::rebase(repo, onto, opts)
    }
    fn rebase_plan(&self, repo: &RepoRef, upstream: Option<Oid>) -> Result<RebasePlanDraft> {
        rebase_interactive::rebase_plan(repo, upstream)
    }
    fn rebase_interactive(&self, repo: &RepoRef, req: &InteractiveRebase) -> Result<RebaseOutcome> {
        rebase_interactive::rebase_interactive(repo, req)
    }
    fn rebase_state(&self, repo: &RepoRef) -> Result<Option<RebaseState>> {
        rebase_interactive::rebase_state(repo)
    }
    fn rebase_continue(&self, repo: &RepoRef) -> Result<RebaseOutcome> {
        rebase_interactive::rebase_continue(repo)
    }
    fn rebase_skip(&self, repo: &RepoRef) -> Result<RebaseOutcome> {
        rebase_interactive::rebase_skip(repo)
    }
    fn rebase_abort(&self, repo: &RepoRef) -> Result<()> {
        rebase_interactive::rebase_abort(repo)
    }
    fn cherry_pick(
        &self,
        repo: &RepoRef,
        commits: &[Oid],
        opts: &CherryPickOptions,
    ) -> Result<CherryPickOutcome> {
        history_ops::cherry_pick(repo, commits, opts)
    }
    fn revert(&self, repo: &RepoRef, commit: Oid, mainline: Option<u32>) -> Result<RevertOutcome> {
        history_ops::revert(repo, commit, mainline)
    }
    fn reset(&self, repo: &RepoRef, target: Oid, mode: ResetMode) -> Result<()> {
        reset::reset(repo, target, mode)
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
    fn split_source(&self, repo: &RepoRef, commit: Oid) -> Result<SplitSource> {
        split::split_source(repo, commit)
    }
    fn split_commit(&self, repo: &RepoRef, req: &SplitCommit) -> Result<SplitOutcome> {
        split::split_commit(repo, req)
    }
    // History rewrite (`history/`): the engine is the system git's, the
    // git2 backend delegates here.
    fn rewrite_preview(
        &self,
        repo: &RepoRef,
        recipe: &Recipe,
        progress: RewriteProgressSink<'_>,
    ) -> Result<RewritePlan> {
        history::preview(repo, recipe, &self.tool_probe, progress)
    }
    fn rewrite_apply(&self, repo: &RepoRef, plan: &RewritePlan) -> Result<RewriteOutcome> {
        history::apply(repo, plan)
    }
    fn rewrite_extract(
        &self,
        repo: &RepoRef,
        plan: &RewritePlan,
        target: &std::path::Path,
    ) -> Result<RewriteOutcome> {
        history::extract_to(repo, plan, target, &self.tool_probe)
    }
    fn rewrite_load(&self, repo: &RepoRef, run: &RunId) -> Result<Option<RewritePlan>> {
        history::load(repo, run)
    }
    fn rewrite_record(
        &self,
        repo: &RepoRef,
        run: &RunId,
    ) -> Result<gitpoulpi_core::domain::rewrite::RewriteRecord> {
        history::record(repo, run)
    }
    fn rewrite_maps(
        &self,
        repo: &RepoRef,
        run: &RunId,
        out: Option<&Path>,
    ) -> Result<gitpoulpi_core::domain::rewrite::RewriteMaps> {
        history::maps(repo, run, out)
    }
    async fn push_rewritten(
        &self,
        repo: &RepoRef,
        req: &gitpoulpi_core::domain::rewrite::PushRewritten,
    ) -> Result<gitpoulpi_core::domain::rewrite::PushRewrittenOutcome> {
        history::push_rewritten(repo, req).await
    }
    fn rewrite_purge(
        &self,
        repo: &RepoRef,
        req: &gitpoulpi_core::domain::rewrite::PurgeRequest,
        progress: gitpoulpi_core::domain::rewrite::PurgeProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::PurgeOutcome> {
        history::purge(repo, req, progress)
    }
    fn rewrite_blob_diff(
        &self,
        repo: &RepoRef,
        run: &RunId,
        commit: Oid,
        path: &str,
    ) -> Result<gitpoulpi_core::domain::rewrite::BlobChange> {
        history::blob_diff(repo, run, commit, path)
    }
    fn rewrite_script_check(
        &self,
        repo: &RepoRef,
        script: &gitpoulpi_core::domain::rewrite::ScriptRef,
        policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
    ) -> Result<gitpoulpi_core::domain::rewrite::script::ScriptCheck> {
        history::script_check(repo, script, policy, &self.tool_probe)
    }
    fn rewrite_script_test(
        &self,
        repo: &RepoRef,
        script: &gitpoulpi_core::domain::rewrite::ScriptRef,
        policy: &gitpoulpi_core::domain::rewrite::ScriptPolicy,
        commits: &[Oid],
    ) -> Result<Vec<gitpoulpi_core::domain::rewrite::script::ScriptTestRow>> {
        history::script_test(repo, script, policy, &self.tool_probe, commits)
    }
    fn rewrite_scan_secrets(
        &self,
        repo: &RepoRef,
        scope: &gitpoulpi_core::domain::rewrite::Scope,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::SecretScan> {
        history::scan_secrets(repo, scope, progress)
    }
    fn rewrite_analyze(
        &self,
        repo: &RepoRef,
        scope: &gitpoulpi_core::domain::rewrite::Scope,
        top: u32,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgressSink<'_>,
    ) -> Result<gitpoulpi_core::domain::rewrite::SizeAnalysis> {
        history::analyze(repo, scope, top, progress)
    }
    fn rewrite_discard(&self, repo: &RepoRef, run: &RunId) -> Result<()> {
        history::discard(repo, run)
    }
    fn stash_push(
        &self,
        repo: &RepoRef,
        message: Option<&str>,
        include_untracked: bool,
    ) -> Result<Option<StashId>> {
        stash_ops::stash_push(repo, message, include_untracked)
    }
    fn stash_push_paths(
        &self,
        repo: &RepoRef,
        message: Option<&str>,
        paths: &[String],
    ) -> Result<Option<StashId>> {
        stash_ops::stash_push_paths(repo, message, paths)
    }
    fn stash_pop(&self, repo: &RepoRef, id: StashId) -> Result<()> {
        stash_ops::stash_pop(repo, id)
    }
    fn stash_apply(&self, repo: &RepoRef, id: StashId) -> Result<()> {
        stash_ops::stash_apply(repo, id)
    }
    fn stash_drop(&self, repo: &RepoRef, id: StashId) -> Result<()> {
        stash_ops::stash_drop(repo, id)
    }
    fn stash_store(&self, repo: &RepoRef, commit: Oid, message: &str) -> Result<()> {
        stash_ops::stash_store(repo, commit, message)
    }
    fn stash_paths(&self, repo: &RepoRef, commit: Oid) -> Result<Vec<String>> {
        stash_ops::stash_paths(repo, commit)
    }
    fn set_network_auth(&self, repo: &RepoRef, auth: Option<NetworkAuth>) {
        process::set_network_auth(&repo.root, auth);
    }
    async fn fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &FetchOptions,
    ) -> Result<FetchOutcome> {
        network::fetch(repo, remote, opts).await
    }
    async fn fetch_all(&self, repo: &RepoRef, opts: &FetchOptions) -> Result<FetchOutcome> {
        network::fetch_all(repo, opts).await
    }
    async fn fetch_ref_as_branch(
        &self,
        repo: &RepoRef,
        remote: &str,
        remote_ref: &str,
        branch: &str,
    ) -> Result<FetchOutcome> {
        network::fetch_ref_as_branch(repo, remote, remote_ref, branch).await
    }
    async fn fetch_refspec(
        &self,
        repo: &RepoRef,
        remote: &str,
        refspec: &str,
    ) -> Result<FetchOutcome> {
        network::fetch_refspec(repo, remote, refspec).await
    }
    async fn pull(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
        network::pull(repo, opts).await
    }
    async fn pull_fetch(&self, repo: &RepoRef, opts: &PullOptions) -> Result<()> {
        network::pull_fetch(repo, opts).await
    }
    async fn pull_integrate(&self, repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
        network::pull_integrate(repo, opts).await
    }
    async fn push(&self, repo: &RepoRef, remote: &str, opts: &PushOptions) -> Result<PushOutcome> {
        network::push(repo, remote, opts).await
    }
    async fn push_for_review(
        &self,
        repo: &RepoRef,
        remote: &str,
        branch: &str,
        topic: Option<&str>,
    ) -> Result<PushOutcome> {
        network::push_for_review(repo, remote, branch, topic).await
    }
    async fn clone(&self, url: &str, dest: &Path, opts: &CloneOptions) -> Result<RepoRef> {
        network::clone(url, dest, opts, &self.lfs_probe).await
    }
    async fn clone_with_progress(
        &self,
        url: &str,
        dest: &Path,
        opts: &CloneOptions,
        progress: CloneProgressSink<'_>,
    ) -> Result<RepoRef> {
        network::clone_with_progress(url, dest, opts, &self.lfs_probe, progress).await
    }
    fn remote_add(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()> {
        network::remote_add(repo, name, url)
    }
    fn remote_remove(&self, repo: &RepoRef, name: &str) -> Result<()> {
        remotes::remote_remove(repo, name)
    }
    fn remote_rename(&self, repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
        remotes::remote_rename(repo, old_name, new_name)
    }
    fn remote_set_url(&self, repo: &RepoRef, name: &str, url: &str) -> Result<()> {
        remotes::remote_set_url(repo, name, url)
    }
    fn remote_set_push_url(&self, repo: &RepoRef, name: &str, url: Option<&str>) -> Result<()> {
        remotes::remote_set_push_url(repo, name, url)
    }
    fn ssh_command(&self, repo: &RepoRef) -> Result<Option<String>> {
        remotes::ssh_command(repo)
    }
    fn set_ssh_command(&self, repo: &RepoRef, command: Option<&str>) -> Result<()> {
        remotes::set_ssh_command(repo, command)
    }
    fn repo_config(&self, repo: &RepoRef, key: &str) -> Result<Option<String>> {
        remotes::repo_config(repo, key)
    }
    fn sparse_checkout_status(&self, repo: &RepoRef) -> Result<SparseState> {
        sparse::status(repo)
    }
    fn sparse_checkout_set(&self, repo: &RepoRef, dirs: &[String]) -> Result<()> {
        sparse::set(repo, dirs)
    }
    fn sparse_checkout_disable(&self, repo: &RepoRef) -> Result<()> {
        sparse::disable(repo)
    }
    fn set_repo_config(&self, repo: &RepoRef, key: &str, value: Option<&str>) -> Result<()> {
        remotes::set_repo_config(repo, key, value)
    }
    fn config_entries(&self, repo: &RepoRef, prefixes: &[&str]) -> Result<Vec<(String, String)>> {
        remotes::config_entries(repo, prefixes)
    }
    fn rewrite_tools(
        &self,
        repo: &RepoRef,
        fresh: bool,
    ) -> Result<gitpoulpi_core::domain::rewrite::RewriteTools> {
        tools::rewrite_tools(repo, &self.tool_probe, fresh)
    }
    fn rewrite_check_signing(
        &self,
        repo: &RepoRef,
    ) -> Result<gitpoulpi_core::domain::rewrite::SigningCheck> {
        history::check_signing(repo, &self.tool_probe)
    }
    fn lfs_status(&self, repo: &RepoRef) -> Result<LfsRepoStatus> {
        lfs::lfs_status(repo, &self.lfs_probe)
    }
    fn lfs_files(&self, repo: &RepoRef, at: Option<Oid>) -> Result<Vec<LfsFile>> {
        lfs::lfs_files(repo, at)
    }
    fn lfs_install(&self, repo: &RepoRef, scope: LfsInstallScope) -> Result<()> {
        lfs::lfs_install(repo, &self.lfs_probe, scope)
    }
    fn lfs_track(
        &self,
        repo: &RepoRef,
        patterns: &[String],
        opts: &LfsTrackOptions,
    ) -> Result<Vec<LfsPattern>> {
        lfs::lfs_track(repo, &self.lfs_probe, patterns, opts)
    }
    fn lfs_untrack(&self, repo: &RepoRef, patterns: &[String]) -> Result<Vec<LfsPattern>> {
        lfs::lfs_untrack(repo, &self.lfs_probe, patterns)
    }
    fn lfs_prune(&self, repo: &RepoRef) -> Result<()> {
        lfs::lfs_prune(repo, &self.lfs_probe)
    }
    async fn lfs_locks(&self, repo: &RepoRef, remote: &str) -> Result<Vec<LfsLock>> {
        lfs_locks::lfs_locks(repo, &self.lfs_probe, remote).await
    }
    async fn lfs_lock(&self, repo: &RepoRef, remote: &str, path: &Path) -> Result<LfsLock> {
        lfs_locks::lfs_lock(repo, &self.lfs_probe, remote, path).await
    }
    async fn lfs_unlock(
        &self,
        repo: &RepoRef,
        remote: &str,
        path: &Path,
        force: bool,
    ) -> Result<()> {
        lfs_locks::lfs_unlock(repo, &self.lfs_probe, remote, path, force).await
    }
    async fn lfs_fetch(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsFetchOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        lfs_network::lfs_fetch(repo, &self.lfs_probe, remote, opts, progress).await
    }
    async fn lfs_pull(
        &self,
        repo: &RepoRef,
        remote: &str,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        lfs_network::lfs_pull(repo, &self.lfs_probe, remote, progress).await
    }
    async fn lfs_checkout(
        &self,
        repo: &RepoRef,
        paths: &[PathBuf],
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        lfs_network::lfs_checkout(repo, &self.lfs_probe, paths, progress).await
    }
    async fn lfs_push(
        &self,
        repo: &RepoRef,
        remote: &str,
        opts: &LfsPushOptions,
        progress: LfsProgressSink<'_>,
    ) -> Result<()> {
        lfs_network::lfs_push(repo, &self.lfs_probe, remote, opts, progress).await
    }
    fn conflicts(&self, repo: &RepoRef) -> Result<Vec<ConflictSet>> {
        conflicts::conflicts(repo)
    }
    fn conflict_entries(&self, repo: &RepoRef) -> Result<Vec<ConflictEntry>> {
        conflicts::conflict_entries(repo)
    }
    fn merge_document(&self, repo: &RepoRef, path: &Path) -> Result<MergeDocument> {
        conflicts::merge_document(repo, path)
    }
    fn resolve_conflict(
        &self,
        repo: &RepoRef,
        path: &Path,
        resolution: ConflictResolution,
        opts: &ResolveOptions,
    ) -> Result<()> {
        conflicts::resolve_conflict(repo, path, resolution, opts)
    }
    fn unresolve_conflict(&self, repo: &RepoRef, path: &Path, entry: &ConflictEntry) -> Result<()> {
        conflicts::unresolve_conflict(repo, path, entry)
    }
    fn operation_state(&self, repo: &RepoRef) -> Result<Option<OperationState>> {
        operation::operation_state(repo)
    }
    fn operation_abort(&self, repo: &RepoRef) -> Result<()> {
        operation::operation_abort(repo)
    }
    fn am_continue(&self, repo: &RepoRef) -> Result<AmOutcome> {
        stuck::am_continue(repo)
    }
    fn am_skip(&self, repo: &RepoRef) -> Result<AmOutcome> {
        stuck::am_skip(repo)
    }
    fn am_abort(&self, repo: &RepoRef) -> Result<()> {
        stuck::am_abort(repo)
    }
    fn index_lock(&self, repo: &RepoRef) -> Result<Option<IndexLock>> {
        stuck::index_lock(repo)
    }
    fn index_lock_remove(&self, repo: &RepoRef) -> Result<()> {
        stuck::index_lock_remove(repo)
    }
    fn operation_skip(&self, repo: &RepoRef) -> Result<()> {
        operation::operation_skip(repo)
    }
}
