use super::*;

/// One commit of [`Action::ComposeCommits`].
#[derive(Debug, Clone, PartialEq)]
pub struct ComposedCommit {
    pub message: String,
    pub paths: Vec<PathBuf>,
}

/// A checkout, merge, rebase or pull the daemon refused because local
/// changes are in the way and `general.auto_stash` is `ask`. Nothing was
/// changed; `action` is what was refused, to resend wrapped in
/// [`Action::AutoStashed`] if the user says yes.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoStashNeeded {
    /// `checkout`, `merge`, `rebase` or `pull`.
    pub operation: String,
    pub action: Action,
}

/// A mutation the render loop wants performed against the repo. Submitted
/// fire-and-forget: on success the daemon's own `RepoChanged` broadcast
/// triggers the next refresh, so the caller does not wait for a reply to see
/// the result reflected. Only failures are surfaced, via
/// `DaemonLink::action_error_rx`.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Undo / redo the latest journaled operation (the toolbar buttons,
    /// `Ctrl+Z` / `Ctrl+Y`, the palette). `expect` is the entry the user
    /// was shown; `confirmed` answers a `NeedsConfirmation`.
    Undo {
        expect: Option<u64>,
        confirmed: bool,
    },
    Redo {
        expect: Option<u64>,
        confirmed: bool,
    },
    /// Apply an AI commit-composer plan: unstage everything, then per
    /// commit stage its files and commit. Stops at the first failure and
    /// says how many commits were made.
    ComposeCommits {
        commits: Vec<ComposedCommit>,
    },
    StageFile(PathBuf),
    UnstageFile(PathBuf),
    StageAll,
    UnstageAll,
    /// Stage/unstage a whole directory at once - what clicking a folder
    /// row's button in the pane's Tree mode submits.
    StageFolder(PathBuf),
    UnstageFolder(PathBuf),
    /// One hunk of one file, identified by the `HunkId` the backend's own
    /// diff numbered it with. The daemon resolves that id against a diff
    /// it fetches itself (working-tree-vs-index when staging, index-vs-
    /// HEAD when unstaging), which is why the Diff view only offers this
    /// on those two halves - see `diff_view::DiffSource`.
    StageHunk {
        path: PathBuf,
        hunk: HunkId,
    },
    UnstageHunk {
        path: PathBuf,
        hunk: HunkId,
    },
    /// A line-level selection inside one hunk. `line_indices` are indices
    /// into that hunk's `lines`, the numbering `StageTarget::Line` is
    /// defined in.
    StageLines {
        path: PathBuf,
        selector: LineSelector,
    },
    UnstageLines {
        path: PathBuf,
        selector: LineSelector,
    },
    /// Throw away a file's uncommitted worktree changes by checking it
    /// back out of `source` (HEAD). Destructive and irreversible, so the
    /// render loop only submits this after the confirm modal.
    DiscardFile {
        path: PathBuf,
        source: Oid,
    },
    /// Throw away every uncommitted change to each of `paths` at once (a
    /// folder's worth, or "discard everything"): `GitBackend::discard`.
    /// Destructive and irreversible like `DiscardFile`, and submitted only
    /// after the confirm modal.
    DiscardPaths(Vec<PathBuf>),
    /// Revert one hunk of a file's unstaged changes in the working tree
    /// (`GitBackend::discard_part`), numbered like `StageHunk` against the
    /// working-tree-vs-index diff. Destructive: only submitted after the
    /// confirm modal.
    DiscardHunk {
        path: PathBuf,
        hunk: HunkId,
    },
    /// The line-level sibling of `DiscardHunk`, numbered like
    /// `StageLines`. Confirmed the same way.
    DiscardLines {
        path: PathBuf,
        selector: LineSelector,
    },
    Commit {
        message: String,
        amend: bool,
        /// `--no-verify`: skip the commit hooks (the composer's Skip hooks).
        no_verify: bool,
        /// The repository's push-after-commit setting: once the commit
        /// landed, push the branch to its upstream (none: say it was not).
        push: bool,
    },
    /// A single remote - pruned when `general.auto_prune` is on
    /// (`crate::live_settings::auto_prune`), off by default.
    Fetch(String),
    /// Every remote - the toolbar's Fetch, auto-fetch and the Pull
    /// button's "Fetch All" mode - pruned under the same rule as
    /// `Fetch`.
    FetchAll,
    /// Fetch a pull/merge request head (`refs/pull/<n>/head`, possibly a
    /// fork's commits) into the tracking local branch `branch` and check
    /// it out.
    CheckoutPullRequestHead {
        remote: String,
        remote_ref: String,
        branch: String,
    },
    /// Push `HEAD` to `refs/for/<branch>` of `remote` (a Gerrit change).
    /// Not undoable: the change lives on the server.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    PushForReview {
        remote: String,
        branch: String,
        topic: Option<String>,
    },
    /// Integrate as `PullOptions::mode` says, from the upstream or from
    /// `PullOptions::remote` for a branch that has none.
    Pull(PullOptions),
    /// Push a branch (`opts.branch`, or the checked-out one) to `remote`.
    /// A rejection because the remote diverged comes back typed on
    /// `DaemonLink::push_rejected_rx`, not as an error string.
    Push {
        remote: String,
        opts: PushOptions,
    },
    /// The diverged-push dialog's "Pull (merge) first, then push": one
    /// action, so the push only starts once the pull has succeeded.
    PullThenPush {
        remote: String,
        branch: String,
    },
    /// The diverged-push dialog's force push: fetch `remote`, then push
    /// `branch` with `--force-with-lease` against what was just fetched.
    /// Without the fetch the lease compares with a remote-tracking ref
    /// the rejected push proved stale, and is always refused.
    FetchThenForcePush {
        remote: String,
        branch: String,
    },
    /// The inner action again, answering the "stash your changes and
    /// continue?" dialog with yes: its checkout/merge/rebase/pull request
    /// carries `autostash: "always"`, so the daemon stashes what blocks it,
    /// runs it and pops the stash.
    AutoStashed(Box<Action>),
    /// The inner stage again, answering the sensitive-files question with
    /// "Stage anyway": its request carries `allow_sensitive: true`, so the
    /// daemon's staging guard lets it through.
    AllowSensitive(Box<Action>),
    CheckoutBranch(String),
    /// Detach HEAD onto a commit (`CheckoutTarget::Commit`). Separate from
    /// `CheckoutBranch` because two different menus offer them and they
    /// produce different status lines; the wire shape is the same
    /// `OperationKind::Checkout`.
    CheckoutCommit(Oid),
    StashPush,
    /// Remove this file from the working tree (a plain file delete; the
    /// working-tree poll then shows it as deleted/gone).
    DeleteFile(PathBuf),
    /// The multi-file sibling of `DeleteFile`: one daemon request, so one
    /// journal entry and one Ctrl+Z.
    DeleteFiles(Vec<PathBuf>),
    /// Overwrite these working-tree files with their version from `source`
    /// (`CheckoutTarget::Files`): one journal entry, the old bytes come back
    /// with Ctrl+Z. Submitted only after the confirm modal.
    RestoreFiles {
        paths: Vec<PathBuf>,
        source: Oid,
    },
    /// Append a pattern to a `.gitignore` or to `.git/info/exclude` (`file`,
    /// repo-relative; `None` is the root `.gitignore`). Daemon operation
    /// `AddToGitignore`, journaled except for the per-clone file: Ctrl+Z
    /// puts the file back.
    AddToGitignore {
        pattern: String,
        file: Option<String>,
    },
    /// Make git diff `paths` as binary or as text by writing a
    /// `.gitattributes` rule (`file` as for `AddToGitignore`). Daemon
    /// operation `GitattributesTreat`, journaled likewise. Handled by
    /// `run_action` itself: its answer says whether another rule overrides.
    TreatAs {
        paths: Vec<String>,
        scope: gitpoulpi_core::domain::RuleScope,
        treat: gitpoulpi_core::domain::Treat,
        file: Option<String>,
    },
    /// Rewrite the encoding, line endings or indentation of these working-tree
    /// files (repo-relative, `/`). Daemon operation `ConvertFiles`: one journal
    /// entry for the whole batch. Handled by `run_action` itself: its answer
    /// is the per-file report the status line summarises.
    ConvertFiles {
        paths: Vec<String>,
        op: gitpoulpi_core::textconv::ConvertOp,
    },
    /// Stash only this file (`git stash push -- <path>`).
    StashFile(PathBuf),
    /// Stash everything with this message (`Stash With Message...`).
    StashPushWith {
        message: String,
    },
    /// `submodule update` of these paths (all when empty), cloning the ones
    /// not yet cloned when `init`. May reach the network.
    SubmoduleUpdate {
        paths: Vec<String>,
        init: bool,
    },
    /// Register these submodules (all when empty) without cloning them.
    SubmoduleInit {
        paths: Vec<String>,
    },
    StashPop(StashId),
    StashApply(StashId),
    StashDrop(StashId),
    /// Pop / drop the pending auto-stash: the daemon resolves the index
    /// from the record's stash commit, so a stash that shifted (or
    /// vanished) outside is never confused with another.
    AutoStashPop(StashId, Option<Oid>),
    AutoStashDrop(StashId, Option<Oid>),
    /// *Start over* for the pending auto-stash (`confirmed` after the user
    /// agreed; the daemon would otherwise answer `NeedsConfirmation`).
    AutoStashStartOver {
        confirmed: bool,
    },
    /// Drop the blocked latest undo entry from the journal (the user
    /// confirmed; the repository is not touched).
    UndoForget {
        expect: Option<u64>,
    },
    /// Move a local branch (or HEAD) back to a reflog position - what a
    /// blocked undo offers. Journaled, so Undo takes it back.
    ReflogRestore {
        refname: String,
        oid: Oid,
    },
    BranchCreate {
        name: String,
        at: Oid,
    },
    /// Create `local_name` tracking the remote-tracking branch
    /// `upstream` and check it out, in one operation - what
    /// double-clicking a remote branch that has no local counterpart
    /// does. See `GitBackend::branch_create_tracking`.
    BranchCreateTracking {
        local_name: String,
        upstream: String,
    },
    BranchDelete(String),
    BranchRename {
        old_name: String,
        new_name: String,
    },
    TagCreate {
        name: String,
        at: Oid,
    },
    TagDelete(String),
    /// Create an annotated tag at a commit, in one operation.
    TagCreateAnnotated {
        name: String,
        at: Oid,
        message: String,
    },
    /// Delete a branch from a remote. With `and_local`, the local branch
    /// of that name goes too - only once the remote accepted the
    /// deletion, so a refused one leaves both in place.
    BranchDeleteRemote {
        remote: String,
        name: String,
        and_local: Option<String>,
    },
    /// Re-make an existing tag as an annotated one, where it already is.
    TagAnnotate {
        name: String,
        message: String,
    },
    /// Send tags to a remote. `selection` carries the whole
    /// `Named`/`AllUnpushed` choice, so "push every unpushed tag" is one
    /// action, one round trip and one result message - the UI never
    /// loops over tags.
    TagPush {
        remote: String,
        selection: TagSelection,
    },
    /// Delete a tag from a remote, keeping the local one.
    TagDeleteRemote {
        remote: String,
        name: String,
    },
    /// Delete a tag on `remote` and then locally ("Delete locally and on
    /// origin"). The remote goes first: if it is refused, nothing local was
    /// lost. Only the local half is journaled; Undo restores the local tag.
    TagDeleteBoth {
        remote: String,
        name: String,
    },
    /// Delete the local tag only, one that `remotes` still have: the message
    /// afterwards says the next fetch brings it back.
    TagDeleteKeepRemote {
        name: String,
        remotes: Vec<String>,
    },
    /// Fetch one tag from a remote ("Fetch this tag" of a remote-only tag).
    TagFetch {
        remote: String,
        name: String,
    },
    /// Check `branch` out and *then* run `then` - only if the checkout
    /// succeeded, so a refused checkout (a dirty tree in the way) can
    /// never leave a merge or rebase landing on the wrong branch. What a
    /// ref drag-and-drop needs: dropping `feature` on `main` merges into
    /// `main`, which must be checked out first.
    CheckoutThen {
        branch: String,
        then: Box<Action>,
    },
    Merge(Ref),
    /// Replay HEAD's commits onto `onto`, non-interactively.
    Rebase(Ref),
    /// Runs a pre-decided todo list (see `rebase_ui::RebaseForm`) --
    /// reorder/squash/fixup/drop/reword/edit - through
    /// `GitBackend::rebase_interactive`.
    RebaseInteractive(InteractiveRebase),
    /// Resumes a rebase stopped by a conflict or an `edit` step.
    RebaseContinue,
    /// Skips the commit a rebase is stopped at.
    RebaseSkip,
    /// Aborts a rebase in progress, restoring the branch and HEAD.
    RebaseAbort,
    /// Continues the `git am` stopped on a patch.
    AmContinue,
    /// Skips the patch `git am` stopped on.
    AmSkip,
    /// Aborts the `git am` in progress.
    AmAbort,
    /// Deletes a stale `.git/index.lock`.
    IndexLockRemove,
    /// Aborts the merge, cherry-pick or revert in progress (the operation
    /// banner's Abort), restoring HEAD, the index and the working tree.
    OperationAbort,
    /// Skips the cherry-pick or revert in progress (the operation banner's
    /// Skip): the way out of one that became empty.
    OperationSkip,
    /// One commit per action; the wire shape is still the list the daemon
    /// expects.
    CherryPick(Oid),
    /// `mainline`: the parent (1-based) that stays, required for a merge
    /// commit (`git revert -m`).
    Revert {
        commit: Oid,
        mainline: Option<u32>,
    },
    Reset {
        target: Oid,
        mode: ResetMode,
    },
    /// Give `commit` (HEAD or an ancestor of it) a new message; see
    /// `GitBackend::reword`.
    Reword {
        commit: Oid,
        message: String,
    },
    /// Change `commit`'s message, author or dates (HEAD or an ancestor of
    /// it); see `GitBackend::amend_commit`.
    AmendCommit {
        commit: Oid,
        amend: Box<gitpoulpi_core::domain::CommitAmend>,
    },
    /// Break one commit into several (`GitBackend::split_commit`); the
    /// request is what the split editor built and `validate_split` accepted.
    SplitCommit {
        req: gitpoulpi_core::domain::SplitCommit,
    },
    /// Take one side of an unmerged path whole. `ConflictResolution::UseContent`
    /// (hand-merged text) has no editor, so only `TakeSide` is reachable from
    /// here.
    ResolveConflict {
        path: PathBuf,
        side: ConflictSide,
    },
    RemoteAdd {
        name: String,
        url: String,
    },
    RemoteRemove(String),
    RemoteRename {
        old_name: String,
        new_name: String,
    },
    RemoteSetUrl {
        name: String,
        url: String,
    },
    /// Make the checkout sparse (cone mode) with exactly these directories
    /// (`OperationKind::SparseSet`). Not journaled: nothing is lost, `SparseDisable`
    /// brings every file back.
    SparseSet {
        dirs: Vec<String>,
    },
    /// Back to a full checkout (`OperationKind::SparseDisable`).
    SparseDisable,
    /// Set (`Some`) or clear (`None`) the repository-local `core.autocrlf` /
    /// `core.eol` (`OperationKind::SetRepoConfig`; journaled, so Undo
    /// restores the previous value).
    SetRepoConfig {
        key: String,
        value: Option<String>,
    },
    /// Replace the repository's hidden/solo ref filter with these full sets
    /// (`OperationKind::SetRefVisibility`, mode `replace`), so the daemon
    /// tells every other frontend.
    SetRefVisibility {
        hidden: Vec<String>,
        solo: Vec<String>,
        /// Remotes hidden as a whole.
        hidden_remotes: Vec<String>,
    },
    /// `None` removes the separate push URL.
    RemoteSetPushUrl {
        name: String,
        url: Option<String>,
    },
    /// `None` removes the upstream.
    BranchSetUpstream {
        branch: String,
        upstream: Option<String>,
    },
    /// The Remotes dialog's Apply (`OperationKind::BranchSetRemotes`): the
    /// extra push targets and pull sources of `branch` (config values,
    /// `<remote>` or `<remote>:<branch>`), and its upstream when it changed
    /// - one request, one undo entry.
    BranchSetRemotes {
        branch: String,
        upstream: Option<String>,
        push_to: Vec<String>,
        pull_from: Vec<String>,
    },
    /// A grouped push or pull over several remotes (`OperationKind::
    /// RemoteGroupRun`): the decisions the push-group dialog collected.
    /// The outcome goes to `push_group_ui::on_outcome`, never to a status
    /// line only.
    RemoteGroupRun(Box<gitpoulpi_core::domain::RemoteGroupRunRequest>),
    /// Go on with the saved grouped run once the conflicts it stopped on
    /// are resolved and concluded. `targets` are fresh decisions (a
    /// `needs_resolution` answer); never sent automatically.
    RemoteGroupContinue {
        expect_run_id: Option<String>,
        targets: Vec<gitpoulpi_core::domain::RunTarget>,
        prefer_rebase: bool,
    },
    /// Abort the merge or rebase the saved grouped run started and drop it.
    RemoteGroupAbort {
        expect_run_id: Option<String>,
    },
    /// Run the saved grouped run's failed targets again.
    RemoteGroupRetry {
        expect_run_id: Option<String>,
        prefer_rebase: bool,
    },
    /// Move the local branch `branch` forward to `target`, only if that
    /// is a fast-forward (`GitBackend::fast_forward`). `target_label` is
    /// what the status line calls the target (`main`, `origin/main`).
    FastForward {
        branch: String,
        target: Oid,
        target_label: String,
    },
    /// Add a linked worktree (`GitBackend::worktree_add`). Opening it as
    /// a tab is the render loop's business, once the refresh lists it.
    WorktreeAdd {
        path: PathBuf,
        opts: WorktreeAddOptions,
    },
    /// Write a patch file (`GitBackend::patch_create`).
    PatchCreate {
        source: PatchSource,
        dest: PathBuf,
    },
    /// Apply a patch file to the working tree (`GitBackend::patch_apply`).
    PatchApply {
        path: PathBuf,
    },
    /// `git lfs install --local` for this repository.
    LfsInstall,
    /// Route patterns through Git LFS (`.gitattributes` is left unstaged).
    LfsTrack {
        patterns: Vec<String>,
        opts: gitpoulpi_core::domain::LfsTrackOptions,
    },
    LfsUntrack(Vec<String>),
    /// Download LFS objects (working tree untouched) / download and
    /// check out / upload. Network; progress arrives as
    /// `Event::LfsProgress`, read back with `DaemonLink::lfs_progress`.
    LfsFetch(String),
    LfsPull(String),
    LfsPush(String),
    /// The toolbar dropdown's "Fetch all" / "Push all": every ref's
    /// objects (`--all`).
    LfsFetchAll(String),
    LfsPushAll(String),
    /// `git lfs checkout`: write real content over every pointer from the
    /// local store. Offline; progress as above.
    LfsCheckout,
    LfsPrune,
    /// Read the server's locks now (and keep them fresh).
    LfsLocks(String),
    LfsLock {
        remote: String,
        path: PathBuf,
    },
    LfsUnlock {
        remote: String,
        path: PathBuf,
        force: bool,
    },
}

impl Action {
    /// The single daemon request this action amounts to, when that is all it
    /// does: no outcome to turn into a status line and no follow-up request.
    /// `None` for the actions `run_action` has to handle itself.
    pub(super) fn request(&self) -> Option<(OperationKind, serde_json::Value)> {
        use serde_json::json;
        let slash = |p: &Path| p.to_string_lossy().replace('\\', "/");
        Some(match self {
            Action::StageFile(path) => (
                OperationKind::StageFile,
                json!({ "target": StageTarget::File(path.clone()) }),
            ),
            Action::UnstageFile(path) => (
                OperationKind::UnstageFile,
                json!({ "target": StageTarget::File(path.clone()) }),
            ),
            Action::StageAll => (
                OperationKind::StageAll,
                json!({ "target": StageTarget::All }),
            ),
            Action::UnstageAll => (
                OperationKind::UnstageAll,
                json!({ "target": StageTarget::All }),
            ),
            Action::StageFolder(path) => (
                OperationKind::StageFolder,
                json!({ "target": StageTarget::Folder(path.clone()) }),
            ),
            Action::UnstageFolder(path) => (
                OperationKind::UnstageFolder,
                json!({ "target": StageTarget::Folder(path.clone()) }),
            ),
            // Hunk- and line-level staging live in the backend behind
            // `StageTarget::{Hunk,Line}`; these requests have the shape
            // `gitpoulpi stage --hunk` / `--lines` sends.
            Action::StageHunk { path, hunk } => (
                OperationKind::StageHunk,
                json!({ "target": StageTarget::Hunk(path.clone(), *hunk) }),
            ),
            Action::UnstageHunk { path, hunk } => (
                OperationKind::UnstageHunk,
                json!({ "target": StageTarget::Hunk(path.clone(), *hunk) }),
            ),
            Action::StageLines { path, selector } => (
                OperationKind::StageLine,
                json!({ "target": StageTarget::Line(path.clone(), selector.clone()) }),
            ),
            Action::UnstageLines { path, selector } => (
                OperationKind::UnstageLine,
                json!({ "target": StageTarget::Line(path.clone(), selector.clone()) }),
            ),
            // `git checkout <source> -- <path>`: git2 reports this variant
            // as unsupported and the decorator falls through to the CLI
            // backend, which does it correctly.
            Action::DiscardFile { path, source } => (
                OperationKind::Checkout,
                json!({
                    "target": CheckoutTarget::File {
                        path: path.to_string_lossy().into_owned(),
                        source: *source,
                    }
                }),
            ),
            // The multi-path sibling of `DiscardFile`: several paths, and
            // untracked ones, need a call of their own rather than one
            // `Checkout` per path.
            Action::DiscardPaths(paths) => {
                let paths: Vec<String> = paths
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                (OperationKind::Discard, json!({ "paths": paths }))
            }
            Action::DiscardHunk { path, hunk } => (
                OperationKind::DiscardHunk,
                json!({ "target": DiscardTarget::Hunk(path.clone(), *hunk) }),
            ),
            Action::DiscardLines { path, selector } => (
                OperationKind::DiscardLine,
                json!({ "target": DiscardTarget::Line(path.clone(), selector.clone()) }),
            ),
            Action::Commit {
                message,
                amend,
                no_verify,
                ..
            } => (
                OperationKind::Commit,
                json!({
                    "message": message,
                    // `general.active_profile`, read live.
                    "opts": CommitOptions {
                        amend: *amend,
                        sign: false,
                        no_verify: *no_verify,
                        author: crate::live_settings::active_profile_author(),
                    },
                }),
            ),
            Action::CheckoutBranch(name) => (
                OperationKind::Checkout,
                json!({ "target": CheckoutTarget::Branch(name.clone()) }),
            ),
            Action::CheckoutCommit(oid) => (
                OperationKind::Checkout,
                json!({ "target": CheckoutTarget::Commit(*oid) }),
            ),
            Action::DeleteFile(path) => {
                (OperationKind::DeleteFile, json!({ "paths": [slash(path)] }))
            }
            Action::DeleteFiles(paths) => {
                let paths: Vec<String> = paths.iter().map(|p| slash(p)).collect();
                (OperationKind::DeleteFile, json!({ "paths": paths }))
            }
            Action::RestoreFiles { paths, source } => (
                OperationKind::Checkout,
                json!({
                    "target": CheckoutTarget::Files {
                        paths: paths.iter().map(|p| slash(p)).collect(),
                        source: *source,
                    }
                }),
            ),
            Action::AddToGitignore { pattern, file } => (
                OperationKind::AddToGitignore,
                json!({ "pattern": pattern, "file": file }),
            ),
            Action::StashFile(path) => (
                OperationKind::StashPush,
                json!({
                    "message": Option::<String>::None,
                    "include_untracked": true,
                    "paths": [slash(path)],
                }),
            ),
            Action::StashPushWith { message } => (
                OperationKind::StashPush,
                json!({ "message": Some(message), "include_untracked": true }),
            ),
            // Untracked files included: with git's
            // own default a new file stays behind and "stash everything"
            // leaves the Log's WIP row in place. Pinned by
            // `stashing_through_the_link_leaves_a_clean_working_tree`.
            Action::StashPush => (
                OperationKind::StashPush,
                json!({ "message": Option::<String>::None, "include_untracked": true }),
            ),
            Action::SubmoduleUpdate { paths, init } => (
                OperationKind::SubmoduleUpdate,
                json!({ "opts": SubmoduleUpdateOptions {
                    paths: paths.clone(),
                    init: *init,
                    recursive: true,
                } }),
            ),
            Action::SubmoduleInit { paths } => {
                (OperationKind::SubmoduleInit, json!({ "paths": paths }))
            }
            Action::StashPop(id) => (OperationKind::StashPop, json!({ "id": id })),
            Action::StashApply(id) => (OperationKind::StashApply, json!({ "id": id })),
            Action::StashDrop(id) => (OperationKind::StashDrop, json!({ "id": id })),
            Action::AutoStashPop(id, commit) => (
                OperationKind::StashPop,
                json!({ "id": id, "commit": commit }),
            ),
            Action::AutoStashDrop(id, commit) => (
                OperationKind::StashDrop,
                json!({ "id": id, "commit": commit }),
            ),
            Action::BranchCreateTracking {
                local_name,
                upstream,
            } => (
                OperationKind::BranchCreateTracking,
                json!({ "local_name": local_name, "upstream": upstream }),
            ),
            Action::BranchDelete(name) => (OperationKind::BranchDelete, json!({ "name": name })),
            Action::BranchRename { old_name, new_name } => (
                OperationKind::BranchRename,
                json!({ "old_name": old_name, "new_name": new_name }),
            ),
            Action::TagCreate { name, at } => (
                OperationKind::TagCreate,
                json!({ "name": name, "at": at, "annotated_message": Option::<String>::None }),
            ),
            Action::TagCreateAnnotated { name, at, message } => (
                OperationKind::TagCreate,
                json!({ "name": name, "at": at, "annotated_message": message }),
            ),
            Action::TagDelete(name) => (OperationKind::TagDelete, json!({ "name": name })),
            Action::SetRefVisibility {
                hidden,
                solo,
                hidden_remotes,
            } => (
                OperationKind::SetRefVisibility,
                json!({
                    "mode": "replace",
                    "hidden": hidden,
                    "solo": solo,
                    "hidden_remotes": hidden_remotes,
                }),
            ),
            _ => return None,
        })
    }

    /// The `RemoteGroupRun` request of a grouped-run action.
    pub(super) fn group_request(&self) -> Option<gitpoulpi_core::domain::RemoteGroupRunRequest> {
        use gitpoulpi_core::domain::{GroupResume, RemoteGroupRunRequest};
        Some(match self {
            Action::RemoteGroupRun(req) => (**req).clone(),
            Action::RemoteGroupContinue {
                expect_run_id,
                targets,
                prefer_rebase,
            } => RemoteGroupRunRequest {
                targets: targets.clone(),
                resume: Some(GroupResume::Continue),
                expect_run_id: expect_run_id.clone(),
                prefer_rebase: *prefer_rebase,
                ..Default::default()
            },
            Action::RemoteGroupAbort { expect_run_id } => RemoteGroupRunRequest {
                resume: Some(GroupResume::Abort),
                expect_run_id: expect_run_id.clone(),
                ..Default::default()
            },
            Action::RemoteGroupRetry {
                expect_run_id,
                prefer_rebase,
            } => RemoteGroupRunRequest {
                retry: true,
                resume: Some(GroupResume::Continue),
                expect_run_id: expect_run_id.clone(),
                prefer_rebase: *prefer_rebase,
                ..Default::default()
            },
            _ => return None,
        })
    }

    /// What this action is, for an error that reached the window without the
    /// daemon's own account of the operation (the link was lost, an older
    /// daemon): the operation it sends, or the action's own name.
    pub(crate) fn op_context(&self) -> gitpoulpi_core::OpContext {
        match self {
            Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner.op_context(),
            _ => match self.request() {
                Some((kind, args)) => gitpoulpi_ipc::op_context::context_of(kind, &args, None, 0),
                None => {
                    let debug = format!("{self:?}");
                    let name: String = debug
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .collect();
                    gitpoulpi_core::OpContext::named(
                        gitpoulpi_core::op_context::words_of_key(&name),
                        "",
                    )
                }
            },
        }
    }

    /// For an action that destroys something and is journaled, the status
    /// line after it succeeded: "Discard done - Ctrl+Z to undo", with the
    /// key as the user has it bound.
    pub(super) fn undoable_done_text(&self) -> Option<String> {
        use gitpoulpi_i18n::msg::undo_safety as s;
        let op = match self {
            Action::AutoStashed(inner) | Action::AllowSensitive(inner) => {
                return inner.undoable_done_text()
            }
            Action::DiscardFile { .. }
            | Action::DiscardPaths(_)
            | Action::DiscardHunk { .. }
            | Action::DiscardLines { .. } => s::OP_DISCARD.get().to_string(),
            Action::DeleteFile(_) | Action::DeleteFiles(_) => s::OP_DELETE_FILE.get().to_string(),
            Action::BranchDelete(_) => s::OP_DELETE_BRANCH.get().to_string(),
            Action::TagDelete(_) => s::OP_DELETE_TAG.get().to_string(),
            Action::Reset {
                mode: ResetMode::Hard,
                ..
            } => s::OP_RESET_HARD.get().to_string(),
            Action::StashDrop(_) => s::OP_STASH_DROP.get().to_string(),
            Action::RemoteRemove(_) => s::OP_REMOVE_REMOTE.get().to_string(),
            _ => return None,
        };
        let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndo);
        Some(if key.is_empty() {
            s::done_no_key(op)
        } else {
            s::done_with_undo(op, key)
        })
    }

    /// What the status line says while this action runs (one mechanism for
    /// every operation: see `activity`).
    pub(super) fn activity_label(&self) -> String {
        use gitpoulpi_i18n::msg::link as l;
        fn name(path: &std::path::Path) -> String {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string())
        }
        let plain = |m: gitpoulpi_i18n::M| m.get().to_string();
        match self {
            Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner.activity_label(),
            Action::StageFile(p) => l::act_staging_file(name(p)),
            Action::UnstageFile(p) => l::act_unstaging_file(name(p)),
            Action::StageFolder(p) => l::act_staging_folder(name(p)),
            Action::UnstageFolder(p) => l::act_unstaging_folder(name(p)),
            Action::StageAll => plain(l::ACT_STAGING_ALL),
            Action::UnstageAll => plain(l::ACT_UNSTAGING_ALL),
            Action::StageHunk { .. } | Action::StageLines { .. } => plain(l::ACT_STAGING),
            Action::UnstageHunk { .. } | Action::UnstageLines { .. } => plain(l::ACT_UNSTAGING),
            Action::DiscardFile { .. }
            | Action::DiscardHunk { .. }
            | Action::DiscardLines { .. } => plain(l::ACT_DISCARDING),
            Action::DiscardPaths(paths) => l::act_discarding_paths(paths.len() as u64),
            Action::Commit { amend: true, .. } => plain(l::ACT_AMENDING),
            Action::Commit { push: true, .. } => plain(l::ACT_COMMITTING_PUSHING),
            Action::Commit { .. } => plain(l::ACT_COMMITTING),
            Action::ComposeCommits { commits } => l::act_creating_commits(commits.len() as u64),
            Action::Fetch(_) | Action::FetchAll | Action::CheckoutPullRequestHead { .. } => {
                plain(l::ACT_FETCHING)
            }
            Action::Pull(_) => plain(l::ACT_PULLING),
            Action::Push { .. }
            | Action::PushForReview { .. }
            | Action::FetchThenForcePush { .. } => plain(l::ACT_PUSHING),
            Action::PullThenPush { .. } => plain(l::ACT_PULLING_PUSHING),
            Action::RemoteGroupRun(_)
            | Action::RemoteGroupContinue { .. }
            | Action::RemoteGroupAbort { .. }
            | Action::RemoteGroupRetry { .. } => plain(gitpoulpi_i18n::msg::push_group::RUNNING),
            Action::CheckoutBranch(_) | Action::CheckoutCommit(_) | Action::CheckoutThen { .. } => {
                plain(l::ACT_CHECKING_OUT)
            }
            Action::StashPush | Action::StashFile(_) | Action::StashPushWith { .. } => {
                plain(l::ACT_STASHING)
            }
            Action::StashPop(_) | Action::StashApply(_) | Action::AutoStashPop(..) => {
                plain(l::ACT_APPLYING_STASH)
            }
            Action::SubmoduleUpdate { .. } => plain(l::ACT_UPDATING_SUBMODULES),
            Action::SubmoduleInit { .. } => plain(l::ACT_REGISTERING_SUBMODULES),
            Action::StashDrop(_) | Action::AutoStashDrop(..) => plain(l::ACT_DROPPING_STASH),
            Action::DeleteFile(p) => l::act_deleting(name(p)),
            Action::DeleteFiles(paths) => l::act_deleting_many(paths.len() as u64),
            Action::RestoreFiles { paths, .. } => l::act_restoring(paths.len() as u64),
            Action::AddToGitignore { .. } => plain(l::ACT_ADDING_GITIGNORE),
            Action::TreatAs { .. } => plain(l::ACT_TREATING),
            Action::ConvertFiles { paths, .. } => l::act_converting(paths.len() as u64),
            Action::Merge(_) => plain(l::ACT_MERGING),
            Action::Rebase(_)
            | Action::RebaseInteractive(_)
            | Action::RebaseContinue
            | Action::RebaseSkip => plain(l::ACT_REBASING),
            Action::RebaseAbort => plain(l::ACT_ABORTING_REBASE),
            Action::OperationAbort => plain(l::ACT_ABORTING_OPERATION),
            Action::AmContinue | Action::AmSkip => {
                plain(gitpoulpi_i18n::msg::stuck_states::ACT_APPLYING_PATCHES)
            }
            Action::AmAbort => plain(gitpoulpi_i18n::msg::stuck_states::AM_ABORTING),
            Action::IndexLockRemove => plain(gitpoulpi_i18n::msg::stuck_states::LOCK_REMOVING),
            Action::SparseSet { .. } => plain(gitpoulpi_i18n::msg::sparse::APPLYING),
            Action::SparseDisable => plain(gitpoulpi_i18n::msg::sparse::DISABLING),
            Action::OperationSkip => plain(l::ACT_SKIPPING_OPERATION),
            Action::CherryPick(_) => plain(l::ACT_CHERRY_PICKING),
            Action::Revert { .. } => plain(l::ACT_REVERTING),
            Action::Reset { .. } => plain(l::ACT_RESETTING),
            Action::AutoStashStartOver { .. } => plain(l::ACT_STARTING_OVER),
            Action::UndoForget { .. } => plain(gitpoulpi_i18n::msg::undo_journal::FORGETTING),
            Action::ReflogRestore { .. } => plain(gitpoulpi_i18n::msg::undo_safety::RUNNING),
            Action::Undo { .. } => plain(gitpoulpi_i18n::msg::undo_ui::UNDOING),
            Action::Redo { .. } => plain(gitpoulpi_i18n::msg::undo_ui::REDOING),
            Action::Reword { .. } => plain(l::ACT_REWORDING),
            Action::AmendCommit { .. } => plain(l::ACT_AMENDING),
            Action::SplitCommit { .. } => plain(gitpoulpi_i18n::msg::split_ui::SPLITTING),
            Action::LfsFetch(_)
            | Action::LfsPull(_)
            | Action::LfsPush(_)
            | Action::LfsFetchAll(_)
            | Action::LfsPushAll(_)
            | Action::LfsCheckout => plain(l::ACT_LFS_TRANSFER),
            _ => plain(l::ACT_WORKING),
        }
    }

    /// Actions the daemon runs as a tracked job (the jobs indicator shows
    /// them): every network action but the submodule update.
    pub(super) fn runs_as_job(&self) -> bool {
        match self {
            Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner.runs_as_job(),
            Action::SubmoduleUpdate { .. } => false,
            other => other.is_network(),
        }
    }

    /// Actions that reach the network, and so must be spawned rather
    /// than awaited inline - the input loop never waits on one.
    pub(super) fn is_network(&self) -> bool {
        if let Action::AutoStashed(inner) | Action::AllowSensitive(inner) = self {
            return inner.is_network();
        }
        matches!(
            self,
            Action::Commit { push: true, .. }
                | Action::Fetch(_)
                | Action::FetchAll
                | Action::SubmoduleUpdate { .. }
                | Action::CheckoutPullRequestHead { .. }
                | Action::Pull(_)
                | Action::Push { .. }
                | Action::PushForReview { .. }
                | Action::PullThenPush { .. }
                | Action::FetchThenForcePush { .. }
                | Action::RemoteGroupRun(_)
                | Action::RemoteGroupContinue { .. }
                | Action::RemoteGroupAbort { .. }
                | Action::RemoteGroupRetry { .. }
                | Action::TagPush { .. }
                | Action::TagDeleteRemote { .. }
                | Action::TagDeleteBoth { .. }
                | Action::TagFetch { .. }
                | Action::LfsFetch(_)
                | Action::LfsPull(_)
                | Action::LfsPush(_)
                | Action::LfsFetchAll(_)
                | Action::LfsPushAll(_)
                | Action::LfsCheckout
                | Action::LfsLocks(_)
                | Action::LfsLock { .. }
                | Action::LfsUnlock { .. }
                | Action::BranchDeleteRemote { .. }
        )
    }

    /// The remote whose locks this action reads or changes, so the link
    /// can keep that remote's lock list fresh afterwards.
    pub(super) fn lfs_locks_remote(&self) -> Option<&str> {
        match self {
            Action::LfsLocks(remote)
            | Action::LfsLock { remote, .. }
            | Action::LfsUnlock { remote, .. } => Some(remote),
            _ => None,
        }
    }
}
