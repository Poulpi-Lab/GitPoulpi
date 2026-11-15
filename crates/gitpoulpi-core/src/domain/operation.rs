use serde::{Deserialize, Serialize};

use super::ChangeScope;

/// Fieldless kind used as the operation-log/audit-trail tag and the IPC
/// command key. Actual per-call arguments live in the
/// GitBackend method signatures, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    /// Path -> the repository enclosing it (see `GitBackend::discover`).
    /// A read, and with `Init` the only operation whose argument is a path
    /// rather than a `RepoId`, because it produces one.
    Discover,
    /// Path -> a brand-new repository there (see `GitBackend::init`).
    /// A mutation.
    Init,
    Status,
    Log,
    /// How many commits `Log` would return unpaged (see
    /// `GitBackend::count_commits`); the status line's "loaded / total". A
    /// read, cached by the daemon until the refs change.
    CountCommits,
    Diff,
    /// The files a `Diff` would list, without their hunks when the
    /// backend can list them more cheaply (see `GitBackend::diff_files`).
    /// A read.
    DiffFiles,
    /// Files changed and lines added/removed for a batch of commits (see
    /// `GitBackend::commit_stats`); the Log's "Changes" column. A read.
    CommitStats,
    /// The workspace search over one repository's index (see
    /// `domain::SearchRequest`): refs, tags, worktrees, stashes and commit
    /// messages. A read, answered by the daemon's search index, never by
    /// `GitBackend` directly.
    Search,
    Refs,
    Stashes,
    Remotes,
    /// The remote-tracking namespaces no remote is configured for (read; see
    /// `GitBackend::leftover_remotes`).
    LeftoverRemotes,
    Worktrees,
    /// The repository's submodules and where each stands (see
    /// `GitBackend::submodules`). A read.
    Submodules,
    /// Register submodules in `.git/config` (see
    /// `GitBackend::submodule_init`).
    SubmoduleInit,
    /// Clone/check out submodules at the commits the superproject records
    /// (see `GitBackend::submodule_update`).
    SubmoduleUpdate,
    /// Revision string -> commit `Oid` (see `GitBackend::resolve`). A read,
    /// so it never broadcasts a `RepoChanged` event.
    Resolve,
    /// Two commits -> their best common ancestor (see `GitBackend::merge_base`).
    MergeBase,
    /// Would merging HEAD into its base branch conflict, and on which files
    /// (see `conflict_prevention`); the daemon picks the base, debounces and
    /// caches. A read.
    PredictConflicts,
    /// A file's content at a revision, the index or the working tree
    /// (see `GitBackend::read_blob`). A read.
    ReadBlob,
    /// A file's exact bytes, base64 on the wire (see
    /// `GitBackend::read_blob_bytes`). A read.
    ReadBlobBytes,
    /// Per-line attribution for a file at a commit (see
    /// `GitBackend::blame`). A read.
    Blame,
    /// The commits that changed a file, following renames (see
    /// `GitBackend::file_history`). A read.
    FileHistory,
    /// Every path in the repository at a revision (see
    /// `GitBackend::list_tree`). A read: what the project *contained*, not
    /// what a commit changed.
    ListTree,
    /// The lines of a revision's files that contain given terms (see
    /// `GitBackend::grep_tree`). A read, never journaled.
    GrepTree,
    StageFile,
    StageHunk,
    StageLine,
    StageFolder,
    StageAll,
    UnstageFile,
    UnstageHunk,
    UnstageLine,
    UnstageFolder,
    UnstageAll,
    Commit,
    Checkout,
    /// Discard uncommitted changes to one or more paths at once (see
    /// `GitBackend::discard`), in one round trip.
    Discard,
    /// Revert one hunk of a file's unstaged changes in the working tree
    /// (see `GitBackend::discard_part`).
    DiscardHunk,
    /// Revert some changed lines of one hunk of a file's unstaged changes
    /// in the working tree (see `GitBackend::discard_part`).
    DiscardLine,
    BranchCreate,
    /// Create a local branch tracking a remote-tracking branch and check
    /// it out, in one operation. See
    /// [`crate::GitBackend::branch_create_tracking`].
    BranchCreateTracking,
    BranchDelete,
    /// Delete a branch from a remote, leaving the local one (see
    /// `GitBackend::branch_delete_remote`). Network. Journaled by the daemon
    /// (the commit the server held is recorded; undo pushes it back, see
    /// `RemoteRefChange`); with `and_local` the local branch goes in the same
    /// entry.
    BranchDeleteRemote,
    BranchRename,
    /// Set or clear a local branch's upstream (see
    /// `GitBackend::branch_set_upstream`).
    BranchSetUpstream,
    /// Replace a local branch's extra push targets and pull sources
    /// (`gitpoulpi-push-to` / `gitpoulpi-pull-from` in its config section,
    /// see `domain::remote_group`). Journaled like `BranchSetUpstream`.
    BranchSetRemotes,
    /// Read a branch's remotes (primary, push targets, pull sources) and
    /// where each stands, from local refs only: no network.
    RemoteGroupStatus,
    /// Plan a grouped push or pull of one branch over its remotes: asks each
    /// target (`ls-remote`), fetches only the remotes that need it, classifies
    /// each against the fresh state and suggests an action per row. Changes
    /// no branch; the fetch moves tracking refs (see `domain::remote_group_plan`).
    RemoteGroupPlan,
    /// Run a grouped push or pull of one branch over its remotes
    /// (`domain::remote_group_run`): integrations first, then the pushes
    /// one remote at a time, each with its own status. The local part (a
    /// fast-forward, merge or rebase) is one journaled entry like
    /// `FlowRun`; the pushes are never undone. A conflict leaves a saved
    /// run that `resume` continues (never automatically) or aborts.
    RemoteGroupRun,
    /// Move a local branch forward to a commit, only if that is a
    /// fast-forward (see `GitBackend::fast_forward`).
    FastForward,
    /// Add a linked worktree at a path, on a branch or a commit (see
    /// `GitBackend::worktree_add`).
    WorktreeAdd,
    /// The uncommitted changes of a linked worktree (see
    /// `GitBackend::worktree_changes`). A read.
    WorktreeChanges,
    /// Remove a linked worktree, optionally saving its changes first
    /// (see `GitBackend::worktree_remove`). Not undoable.
    WorktreeRemove,
    /// Write a patch file from one or more commits, a commit range, or
    /// the working tree (see `GitBackend::patch_create`). A read: it
    /// writes to an arbitrary destination file, never to the repository.
    PatchCreate,
    /// Apply a patch file to the working tree (see
    /// `GitBackend::patch_apply`).
    PatchApply,
    /// Replay one commit's changes onto the working tree, all or nothing
    /// (see `GitBackend::apply_commit_to_worktree`): "Restore changes" of
    /// a lost commit. Journaled as a file snapshot, like `PatchApply`.
    ApplyCommit,
    /// Delete files from the working tree; the index is not touched (see
    /// `GitBackend::delete_files`); journaled, so it can be undone.
    DeleteFile,
    /// Rewrite the encoding, line endings or indentation of files in the
    /// working tree (see `GitBackend::convert_files`); journaled as one
    /// entry, so one Ctrl+Z puts every file back.
    ConvertFiles,
    /// Append a pattern to a `.gitignore` (see
    /// `GitBackend::gitignore_add`); journaled, so it can be undone.
    AddToGitignore,
    /// Write the `.gitattributes` rule that makes git diff paths as text
    /// or as binary (see `GitBackend::gitattributes_treat`); journaled
    /// like [`OperationKind::AddToGitignore`] (undoable, except for the
    /// per-clone `.git/info/attributes`, which is outside the working tree).
    GitattributesTreat,
    /// Does git diff each path as binary (see `GitBackend::binary_paths`)?
    /// A read.
    BinaryPaths,
    /// Every name and email seen in a scope, with commit counts and what
    /// the `.mailmap` already maps (see `GitBackend::mailmap_identities`).
    /// A read.
    MailmapIdentities,
    /// Resolve identities through the repository's `.mailmap` (see
    /// `GitBackend::mailmap_resolve`). A read.
    MailmapResolve,
    /// Append the lines that merge identities into a canonical one to the
    /// `.mailmap` (see `GitBackend::mailmap_add`); journaled like
    /// [`OperationKind::AddToGitignore`], so undo puts the old file back.
    /// Nothing is committed.
    MailmapAdd,
    TagCreate,
    TagDelete,
    /// Re-make an existing tag as an annotated one, in place (see
    /// `GitBackend::tag_annotate`).
    TagAnnotate,
    /// Send one, several or every unpushed tag to a remote (see
    /// `GitBackend::tag_push`). Network, and a mutation of the *remote*
    /// as well as of the local mirror refs.
    TagPush,
    /// Delete a tag from a remote, keeping the local one (see
    /// `GitBackend::tag_delete_remote`). Network. Journaled like
    /// [`OperationKind::BranchDeleteRemote`] (an annotated tag's own object is
    /// what undo pushes back).
    TagDeleteRemote,
    /// Fetch one tag from a remote (see `GitBackend::tag_fetch`). Network.
    TagFetch,
    Merge,
    Rebase,
    RebaseInteractive,
    /// The range an interactive rebase from a given upstream would
    /// replay, oldest first (see `GitBackend::rebase_plan`). A read.
    RebasePlan,
    /// The state of a rebase currently in progress, if any (see
    /// `GitBackend::rebase_state`). A read.
    RebaseState,
    /// Resumes a stopped rebase (conflict resolved, or an `edit` step)
    /// (see `GitBackend::rebase_continue`).
    RebaseContinue,
    /// Skips the commit a rebase is stopped at (see
    /// `GitBackend::rebase_skip`).
    RebaseSkip,
    /// Aborts a rebase in progress, restoring the branch and HEAD (see
    /// `GitBackend::rebase_abort`).
    RebaseAbort,
    CherryPick,
    Revert,
    Reset,
    /// Give a commit on the current branch a new message, re-creating its
    /// descendants (see `GitBackend::reword`).
    Reword,
    /// Change a commit's message, author or dates, re-creating its descendants
    /// (see `GitBackend::amend_commit`).
    AmendCommit,
    /// Read the numbered pieces of a commit (see `GitBackend::split_source`).
    SplitSource,
    /// Break a commit on the current branch into several, re-creating its
    /// descendants (see `GitBackend::split_commit`).
    SplitCommit,
    /// Compute a history rewrite into a temporary pack and describe it
    /// without touching any ref (see `GitBackend::rewrite_preview`).
    RewritePreview,
    /// Move the refs to a previewed rewrite (see `GitBackend::rewrite_apply`).
    RewriteApply,
    /// Push the refs an applied rewrite moved, each with an explicit lease
    /// (see `GitBackend::push_rewritten`). A network job; not journaled and
    /// not undoable.
    RewritePush,
    /// The collaborator note of an applied rewrite (see
    /// `GitBackend::rewrite_record`).
    RewriteNote,
    /// The commit-map and ref-map of an applied rewrite, optionally copied
    /// to a directory (see `GitBackend::rewrite_maps`).
    RewriteMaps,
    /// What a purge of rewrite journal entries would do and lose, what
    /// blocks it, and the host-side checklist (a read; see
    /// `GitBackend::rewrite_purge` with `dry_run`).
    RewritePurgeCheck,
    /// Delete the old history of rewrite journal entries for good (see
    /// `GitBackend::rewrite_purge`). A job; not journaled and not undoable;
    /// marks the entries `Purged`.
    RewritePurge,
    /// Drop a preview's temporary objects (see `GitBackend::rewrite_discard`).
    RewriteDiscard,
    /// A page of the per-commit rows of the daemon's live preview (the
    /// plan stays in the daemon: it can hold 100 000 rows).
    RewriteRows,
    /// What a content rule did to one file of one commit of the live
    /// preview (see `GitBackend::rewrite_blob_diff`).
    RewriteBlobDiff,
    /// Compiles a history rewrite script (see `GitBackend::rewrite_script_check`).
    RewriteScriptCheck,
    /// Runs a script on some commits without writing anything (see
    /// `GitBackend::rewrite_script_test`).
    RewriteScriptTest,
    /// Trusts an external history rewrite script by its content hash, in the
    /// repository's state (see `gitpoulpi_ipc::rewrite::TrustArgs`). A human's
    /// act: the MCP server does not expose it.
    RewriteTrustScript,
    /// Trusts the committed workflow hooks as the file has them now (args
    /// `{ hash }`, which must equal the hash of the hooks on disk, see
    /// `gitpoulpi_core::workflow::hooks_hash`), in the repository's state. A
    /// human's act: the MCP server does not expose it.
    FlowTrustHooks,
    /// Scans the history for secrets, findings only (see
    /// `GitBackend::rewrite_scan_secrets`). A job.
    RewriteScanSecrets,
    /// The size analysis of the history (see `GitBackend::rewrite_analyze`).
    /// A job.
    RewriteAnalyze,
    /// Signs and verifies a dummy payload with the repository's signing
    /// setup (see `GitBackend::rewrite_check_signing`). Runs programs.
    RewriteCheckSigning,
    /// Which external programs a history rewrite could use here (see
    /// `GitBackend::rewrite_tools`).
    RewriteTools,
    StashPush,
    StashPop,
    StashApply,
    StashDrop,
    /// Put a stash holding one commit's changes on the stash list (see
    /// `GitBackend::stash_from_commit`). Journaled: undo drops it, redo
    /// stores it again.
    StashFromCommit,
    /// *Start over* for the pending auto-stash: discard the stash's paths,
    /// undo the operation that stashed, pop the stash at the original commit
    /// (see `gitpoulpi-daemon::autostash_start_over`).
    AutoStashStartOver,
    Fetch,
    /// Fetch every configured remote in one operation (see
    /// `GitBackend::fetch_all`). Network.
    FetchAll,
    /// Fetch a pull/merge request head into a tracking local branch (see
    /// `GitBackend::fetch_ref_as_branch`). Network.
    FetchRefAsBranch,
    /// Fetch one `refs/gitpoulpi/...` spec (a pull/merge request head for
    /// review; see `GitBackend::fetch_refspec`). Network, not journaled.
    FetchRefspec,
    /// Connection test of one remote: `ls-remote --heads` with the repository's
    /// network authentication applied (see `GitBackend::remote_access_check`).
    /// A network read: nothing changes, so nothing is broadcast.
    RemoteAccessCheck,
    /// The repository's authentication profile and what it resolves to, with
    /// no secret (read; see `gitpoulpi_core::domain::RepoAuthView`). Served by
    /// the daemon from its private `repo-auth.toml`; not a backend call.
    RepoAuthGet,
    /// Replace the repository's authentication profile (daemon-owned, private
    /// file). Not journaled and not undoable, like settings.
    RepoAuthSet,
    /// Remove the repository's authentication profile (back to the machine's
    /// defaults). Not journaled and not undoable, like settings.
    RepoAuthClear,
    /// Connection test of every remote (or one) with the profile applied: a
    /// network read, like `RemoteAccessCheck`.
    RepoAuthTest,
    Pull,
    Push,
    /// Push `HEAD` to `refs/for/<branch>` to open a Gerrit change (see
    /// `GitBackend::push_for_review`). Network, not journaled, not undoable.
    PushForReview,
    Clone,
    RemoteAdd,
    RemoteRemove,
    RemoteRename,
    RemoteSetUrl,
    RemoteSetPushUrl,
    /// The repository-local `core.sshCommand` (read; see
    /// `GitBackend::ssh_command`).
    SshCommand,
    /// Set or clear the repository-local `core.sshCommand` (see
    /// `GitBackend::set_ssh_command`).
    SetSshCommand,
    /// The repository-local `core.autocrlf` / `core.eol` (read; see
    /// `GitBackend::repo_config`). With no `key` argument, every allowed key.
    RepoConfig,
    /// The branching workflow of the repository: the effective definition,
    /// every branch classified with its parent and ahead/behind counts, and
    /// the templates detection ranks (read; see `gitpoulpi_core::workflow`).
    Workflow,
    /// What a branching-workflow action (start, finish, sync, promote,
    /// set-parent) would do: every step, blockers and warnings (read; args
    /// are a `gitpoulpi_core::workflow::FlowRequest`, the answer a
    /// `FlowPlan`). Changes nothing.
    FlowPlan,
    /// Runs a branching-workflow action (args a `FlowRequest`, the answer a
    /// `FlowOutcome`), journaled as one composite entry. A merge or rebase
    /// that stops on conflicts leaves a flow `resume` continues or aborts.
    FlowRun,
    /// Set or clear one allow-listed repository-local config key (see
    /// `GitBackend::set_repo_config`). Journaled: Undo restores the old value.
    SetRepoConfig,
    /// The sparse-checkout state (read; see
    /// `GitBackend::sparse_checkout_status`).
    SparseStatus,
    /// Sets the sparse-checkout directories (see
    /// `GitBackend::sparse_checkout_set`). Not journaled: nothing is lost,
    /// `SparseDisable` brings every file back.
    SparseSet,
    /// Turns sparse checkout off (see `GitBackend::sparse_checkout_disable`).
    /// Not journaled, for the same reason.
    SparseDisable,
    Conflicts,
    /// Every unmerged path with its stages and kind, no blob read (see
    /// `GitBackend::conflict_entries`). A read.
    ConflictEntries,
    /// The merge document of one unmerged path (see
    /// `GitBackend::merge_document`). A read; decodes with the fallback
    /// encoding.
    ConflictShow,
    ResolveConflict,
    /// Puts a resolved path back in conflict (see
    /// `GitBackend::unresolve_conflict`). The entries come from the
    /// request, or from the daemon's record of what it resolved.
    ConflictUnresolve,
    /// The merge, cherry-pick, revert or rebase in progress (see
    /// `GitBackend::operation_state`). A read.
    OperationState,
    /// Aborts the operation in progress (see
    /// `GitBackend::operation_abort`). Destructive; never journaled: the
    /// abort is the undo, and it settles the operation's pending entry.
    OperationAbort,
    /// `git am --continue` (see `GitBackend::am_continue`).
    AmContinue,
    /// `git am --skip` (see `GitBackend::am_skip`).
    AmSkip,
    /// `git am --abort` (see `GitBackend::am_abort`).
    AmAbort,
    /// The repository's `index.lock`, if any (see `GitBackend::index_lock`).
    /// A read.
    IndexLock,
    /// Deletes a stale `index.lock` (see `GitBackend::index_lock_remove`).
    IndexLockRemove,
    /// Skips the cherry-pick or revert in progress, or the commit a rebase
    /// is stopped at (see `GitBackend::operation_skip`). Not journaled and
    /// not undoable: the skipped pick leaves nothing to put back (it is
    /// empty, or its resolutions are given up on purpose).
    OperationSkip,
    /// Git LFS state of a repository (see `GitBackend::lfs_status`). A
    /// read that needs no `git-lfs` binary.
    LfsStatus,
    /// Pointer files at a revision or in the index (see
    /// `GitBackend::lfs_files`). A read.
    LfsFiles,
    /// `git lfs install` (see `GitBackend::lfs_install`).
    LfsInstall,
    /// Track patterns with LFS (see `GitBackend::lfs_track`).
    LfsTrack,
    /// Stop tracking patterns (see `GitBackend::lfs_untrack`).
    LfsUntrack,
    /// Download LFS objects for a remote (see `GitBackend::lfs_fetch`).
    /// Network.
    LfsFetch,
    /// Download and check out LFS objects (see `GitBackend::lfs_pull`).
    /// Network.
    LfsPull,
    /// Write real content over LFS pointer files from the local store (see
    /// `GitBackend::lfs_checkout`). Local, but runs on the async path for
    /// its progress reports.
    LfsCheckout,
    /// Upload LFS objects (see `GitBackend::lfs_push`). Network.
    LfsPush,
    /// Delete old local LFS objects (see `GitBackend::lfs_prune`).
    LfsPrune,
    /// List file locks on the LFS remote (see `GitBackend::lfs_locks`).
    /// Network read.
    LfsLocks,
    /// Lock a file (see `GitBackend::lfs_lock`). Network.
    LfsLock,
    /// Unlock a file (see `GitBackend::lfs_unlock`). Network.
    LfsUnlock,
    /// Daemon-control pseudo-operation: answers with a `DaemonInfo` (pid,
    /// session, backend, version, uptime source). Never reaches any
    /// `GitBackend`: `gitpoulpi-daemon/src/server.rs` intercepts it before
    /// `dispatch`. An `OperationKind` variant only so it rides the existing
    /// `Request`/`Response` envelope.
    Hello,
    /// Daemon-control pseudo-operation sent by the credential helper a
    /// GitPoulpi binary plays when git asks for a credential (see
    /// `gitpoulpi_ipc::git_credential`): args are a `CredentialGrantRequest`,
    /// the answer a `CredentialGrantAnswer`. Intercepted by the daemon like
    /// `Hello`: no repository lock, never journaled, never broadcast. Answers
    /// only for a live, host-bound grant of a running network operation.
    CredentialGrant,
    /// The AI commit composer's plan as one operation: unstage everything,
    /// then per commit stage its files and commit, under one lock and
    /// journaled as one entry. Args `{ commits: [{ message, paths }] }`.
    ComposeCommits,
    /// Undo the latest journaled operation (see the undo-redo design).
    /// Handled by the daemon's journal, never by a `GitBackend` call of
    /// its own; a mutation.
    Undo,
    /// Redo the latest undone operation. A mutation.
    Redo,
    /// The journal's top undo/redo entries and why they are blocked, if
    /// they are. A read.
    JournalState,
    /// Every journal entry (`undo --list`). A read.
    JournalList,
    /// "Restore only this" (selective restore, later actions kept): a new
    /// journaled operation planned by `journal::restore`. Args `{ id, name?,
    /// dry_run?, confirmed? }`. A mutation, handled by the daemon.
    JournalRestore,
    /// A change made outside GitPoulpi (a shell, an AI agent), found by the
    /// daemon's ref snapshot diff and recorded as a journal entry
    /// (the outside-operations design). Only ever the kind of
    /// an entry, never a request.
    Outside,
    /// A page of a ref's reflog (`GitBackend::reflog`). Args `{ ref?, skip?,
    /// limit? }`. A read.
    Reflog,
    /// Move a ref to a reflog position, journaled. Args `{ ref, oid, mode?,
    /// dry_run?, confirmed? }`. A mutation, handled by the daemon.
    ReflogRestore,
    /// Commits no ref reaches (`GitBackend::lost_commits`). Args are
    /// `LostCommitOptions`. A read, scanned off the render thread.
    LostCommits,
    /// Daemon-control pseudo-operation: tells the daemon to answer OK,
    /// broadcast `Event::ShuttingDown` (only when args carry a `reason`, as
    /// the updater's `{"reason":"update"}` does) and `Event::DaemonStopping`, flush its discovery files and
    /// exit the process. Never reaches any `GitBackend`, for the same
    /// reason as `Hello`.
    Shutdown,
    /// Hide, un-hide or solo refs in the repository's saved display filter
    /// (`RepoState::ref_visibility`), or hide whole remotes
    /// (`RepoState::hidden_remotes`). Args `{ mode: "hide" | "show" |
    /// "solo" | "hide-remote" | "show-remote" | "replace", names: [..],
    /// clear: bool }` (`replace`: `{ hidden, solo, hidden_remotes? }`, the
    /// whole sets; an absent `hidden_remotes` leaves them untouched; `names`
    /// of the remote modes are remote names; `clear` on `hide`/`show` also
    /// empties the hidden remotes); answers `{ hidden, solo, hidden_remotes }`
    /// after the change. Handled by the daemon itself (never a
    /// `GitBackend` call) so it can broadcast `Event::RefVisibilityChanged`.
    /// A mutation of GitPoulpi state, not of the repository.
    SetRefVisibility,
    /// A client (the CLI or the MCP server) posted, edited or deleted a
    /// comment on an issue or a pull request in its own process: tells the
    /// daemon so it can broadcast `Event::HostingItemChanged` and an open
    /// item view refreshes. Args `{ integration_id, project, item, change }`.
    /// No repository effect, not journaled, not undoable (the host's
    /// comment is outward).
    NotifyHostingChange,
    /// Daemon-control pseudo-operation sent by a TUI window: register or
    /// refresh this connection as a *viewer* (the window an agent's "show
    /// it to the user" intents are routed to). Args `{ viewer, info }`.
    /// Intercepted by the daemon like `Hello`: no repository lock, never
    /// journaled, never broadcast. See `gitpoulpi_ipc::ui`.
    UiAttach,
    /// Daemon-control, TUI to daemon: focus gained/lost or the tabs
    /// changed. Args `{ viewer, info }`. Same handling as `UiAttach`.
    UiPresence,
    /// Daemon-control, TUI to daemon: the outcome of an intent, or the
    /// state a `UiStateRequest` asked for. Args `{ id, body }`.
    UiReport,
    /// Daemon-control, CLI/MCP to daemon: route one view intent to a
    /// viewer and answer with what the viewer reports. View-only: the
    /// intent type carries no action, and the daemon never treats this as
    /// a mutation. Args `{ viewer?, intent, origin }`.
    UiShow,
    /// Daemon-control, CLI/MCP to daemon: list the attached viewers, or
    /// read one viewer's state. Args `{ what: "viewers" | "state", .. }`.
    UiQuery,
    /// Daemon-control: the jobs the daemon is running or has just finished
    /// (every network request is one). Answered before admission, so it never
    /// waits behind a running operation. No args.
    JobList,
    /// Daemon-control: cancel one job. Args `{ id }`. Answers
    /// `Error::NotCancellable` for a job past its point of no return.
    JobCancel,
}

/// How the daemon treats one operation. The single table behind its lock
/// mode, executor choice, change events, conflict probe and journal rules:
/// every variant is listed in [`OperationKind::traits`], so a new one
/// cannot be left unclassified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpTraits {
    /// An async network operation: awaited on the connection's task rather
    /// than run in `spawn_blocking` (`GitBackend` declares fetch, pull,
    /// push and clone as `async fn`).
    pub network: bool,
    /// Never changes the repository. Reads take the shared repo lock and
    /// run concurrently, and never broadcast a change. Anything not a read
    /// is a mutation: exclusive and broadcast, the safe default for an
    /// operation added later.
    pub read: bool,
    /// A network operation that reads and writes only refs and objects, so
    /// it can share the repo lock with reads. `Pull` (merges into the
    /// worktree and index) and `Clone` stay exclusive.
    pub shared_network: bool,
    /// Can leave unmerged paths, so the daemon probes `conflicts` after it
    /// (a full index read, hence restricted to these operations).
    /// `ResolveConflict` is included: resolving one file of three leaves
    /// the others.
    pub can_conflict: bool,
    /// Always recorded by the undo journal, whatever its arguments. The
    /// snapshot, stash and remote operations are recorded through their own
    /// paths and are not flagged here.
    pub journaled: bool,
    /// History-writing operation whose undo can rewrite commits already
    /// pushed to the branch's upstream.
    pub pushed_risk: bool,
    /// Its journal entry can stay `Pending` across a conflict.
    pub history: bool,
    /// What a mutation can have changed, so a client re-reads only that.
    /// A wrong guess must be too wide, never too narrow: the default is
    /// everything.
    pub scope: ChangeScope,
}

impl OpTraits {
    /// A mutation with no special handling.
    const NONE: OpTraits = OpTraits {
        network: false,
        read: false,
        shared_network: false,
        can_conflict: false,
        journaled: false,
        pushed_risk: false,
        history: false,
        scope: ChangeScope::ALL,
    };

    const fn network(mut self) -> Self {
        self.network = true;
        self
    }
    const fn read(mut self) -> Self {
        self.read = true;
        self
    }
    const fn shared_network(mut self) -> Self {
        self.shared_network = true;
        self
    }
    const fn can_conflict(mut self) -> Self {
        self.can_conflict = true;
        self
    }
    const fn journaled(mut self) -> Self {
        self.journaled = true;
        self
    }
    const fn pushed_risk(mut self) -> Self {
        self.pushed_risk = true;
        self
    }
    const fn history(mut self) -> Self {
        self.history = true;
        self
    }
    fn scope(mut self, scope: ChangeScope) -> Self {
        self.scope = scope;
        self
    }
}

impl OperationKind {
    /// How the daemon treats this operation. No wildcard arm: adding a
    /// variant fails to compile until it is classified here.
    pub fn traits(self) -> OpTraits {
        use OperationKind as K;
        match self {
            K::Discover
            | K::Init
            | K::Checkout
            | K::PatchApply
            | K::ApplyCommit
            | K::RebaseAbort
            | K::Hello
            | K::CredentialGrant
            | K::Undo
            | K::JournalRestore
            | K::Outside
            | K::ReflogRestore
            | K::AutoStashStartOver
            | K::Redo
            | K::Shutdown => OpTraits::NONE,
            // View control: no repository effect at all, so nothing for a
            // client to re-read (`ChangeScope::NONE`), and never journaled.
            K::UiAttach
            | K::UiPresence
            | K::UiReport
            | K::UiShow
            | K::UiQuery
            | K::JobList
            | K::JobCancel => OpTraits::NONE.scope(ChangeScope::NONE),
            K::Status
            | K::Log
            | K::CountCommits
            | K::Diff
            | K::DiffFiles
            | K::CommitStats
            | K::Search
            | K::Refs
            | K::RemoteGroupStatus
            | K::Stashes
            | K::Remotes
            | K::LeftoverRemotes
            | K::Worktrees
            | K::WorktreeChanges
            | K::Submodules
            | K::Resolve
            | K::MergeBase
            | K::PredictConflicts
            | K::ReadBlob
            | K::ReadBlobBytes
            | K::Blame
            | K::FileHistory
            | K::ListTree
            | K::GrepTree
            | K::BinaryPaths
            | K::MailmapIdentities
            | K::MailmapResolve
            | K::PatchCreate
            | K::RebasePlan
            | K::RebaseState
            | K::SplitSource
            | K::RewritePreview
            | K::RewriteDiscard
            | K::RewriteNote
            | K::RewriteMaps
            | K::RewritePurgeCheck
            | K::RewriteRows
            | K::RewriteBlobDiff
            | K::RewriteScriptCheck
            | K::RewriteScriptTest
            | K::RewriteTrustScript
            | K::FlowTrustHooks
            | K::RepoAuthGet
            | K::RewriteScanSecrets
            | K::RewriteAnalyze
            | K::RewriteCheckSigning
            | K::RewriteTools
            | K::SshCommand
            | K::RepoConfig
            | K::Workflow
            | K::FlowPlan
            | K::SparseStatus
            | K::Conflicts
            | K::ConflictEntries
            | K::ConflictShow
            | K::OperationState
            | K::IndexLock
            | K::LfsStatus
            | K::LfsFiles
            | K::JournalState
            | K::JournalList
            | K::Reflog
            | K::LostCommits => OpTraits::NONE.read(),
            K::StageFile
            | K::StageHunk
            | K::StageLine
            | K::StageFolder
            | K::StageAll
            | K::UnstageFile
            | K::UnstageHunk
            | K::UnstageLine
            | K::UnstageFolder
            | K::UnstageAll
            | K::Discard
            | K::DiscardHunk
            | K::DiscardLine
            | K::DeleteFile
            | K::ConvertFiles
            | K::AddToGitignore
            | K::GitattributesTreat => OpTraits::NONE
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFLICTS),
            // The log shows identities through the `.mailmap`, so it re-reads.
            K::MailmapAdd => OpTraits::NONE.scope(
                ChangeScope::INDEX
                    | ChangeScope::WORKTREE
                    | ChangeScope::CONFLICTS
                    | ChangeScope::LOG,
            ),
            K::Commit => OpTraits::NONE.journaled().pushed_risk().scope(
                ChangeScope::INDEX | ChangeScope::REFS | ChangeScope::LOG | ChangeScope::CONFLICTS,
            ),
            K::BranchCreate
            | K::BranchDelete
            | K::BranchRename
            | K::BranchSetUpstream
            | K::BranchSetRemotes
            | K::TagCreate
            | K::TagDelete
            | K::TagAnnotate => OpTraits::NONE
                .journaled()
                .scope(ChangeScope::REFS | ChangeScope::LOG),
            K::BranchCreateTracking | K::FastForward | K::Reset => OpTraits::NONE.journaled(),
            K::BranchDeleteRemote | K::TagDeleteRemote | K::TagFetch => OpTraits::NONE
                .network()
                .scope(ChangeScope::REFS | ChangeScope::LOG),
            // A mutation that can move any submodule's checkout (and, through
            // them, what status shows): the default scope, everything.
            K::SubmoduleInit | K::SubmoduleUpdate => OpTraits::NONE,
            K::WorktreeAdd => {
                OpTraits::NONE.scope(ChangeScope::REFS | ChangeScope::LOG | ChangeScope::WORKTREES)
            }
            K::WorktreeRemove => OpTraits::NONE.scope(
                ChangeScope::REFS
                    | ChangeScope::LOG
                    | ChangeScope::STASHES
                    | ChangeScope::WORKTREES,
            ),
            K::TagPush => OpTraits::NONE
                .network()
                .shared_network()
                .scope(ChangeScope::REFS | ChangeScope::LOG),
            K::Merge | K::Rebase | K::RebaseInteractive | K::CherryPick | K::Revert => {
                OpTraits::NONE
                    .can_conflict()
                    .journaled()
                    .pushed_risk()
                    .history()
            }
            K::RebaseContinue | K::RebaseSkip | K::AmContinue | K::AmSkip => {
                OpTraits::NONE.can_conflict()
            }
            K::AmAbort | K::IndexLockRemove => OpTraits::NONE,
            K::Reword | K::AmendCommit | K::SplitCommit => OpTraits::NONE.journaled().scope(
                ChangeScope::INDEX
                    | ChangeScope::WORKTREE
                    | ChangeScope::REFS
                    | ChangeScope::LOG
                    | ChangeScope::CONFLICTS,
            ),
            // Moves refs (and, for content rules, the index and working
            // tree) of the current repository; undo restores every ref.
            K::RewriteApply => OpTraits::NONE.journaled().pushed_risk().scope(
                ChangeScope::INDEX
                    | ChangeScope::WORKTREE
                    | ChangeScope::REFS
                    | ChangeScope::LOG
                    | ChangeScope::CONFLICTS,
            ),
            // Deletes pins and reflogs and prunes objects: refs and the log
            // change, no working tree file does. Not journaled.
            K::RewritePurge => OpTraits::NONE.scope(ChangeScope::REFS | ChangeScope::LOG),
            K::StashFromCommit => OpTraits::NONE.scope(ChangeScope::LOG | ChangeScope::STASHES),
            K::StashPush | K::StashDrop => OpTraits::NONE.scope(
                ChangeScope::INDEX
                    | ChangeScope::WORKTREE
                    | ChangeScope::LOG
                    | ChangeScope::STASHES
                    | ChangeScope::CONFLICTS,
            ),
            K::StashPop | K::StashApply => OpTraits::NONE.can_conflict().scope(
                ChangeScope::INDEX
                    | ChangeScope::WORKTREE
                    | ChangeScope::LOG
                    | ChangeScope::STASHES
                    | ChangeScope::CONFLICTS,
            ),
            K::Fetch
            | K::FetchAll
            | K::FetchRefspec
            | K::RemoteGroupPlan
            | K::Push
            | K::PushForReview
            | K::RewritePush => OpTraits::NONE.network().shared_network(),
            K::FetchRefAsBranch => OpTraits::NONE.network().journaled(),
            K::Pull => OpTraits::NONE
                .network()
                .can_conflict()
                .journaled()
                .history(),
            K::Clone | K::LfsPull | K::LfsLocks => OpTraits::NONE.network(),
            // Reads a remote with the repository's credentials; changes nothing.
            K::RemoteAccessCheck | K::RepoAuthTest => {
                OpTraits::NONE.network().scope(ChangeScope::NONE)
            }
            K::RepoAuthSet | K::RepoAuthClear => OpTraits::NONE.scope(ChangeScope::NONE),
            K::RemoteAdd
            | K::RemoteRemove
            | K::RemoteRename
            | K::RemoteSetUrl
            | K::RemoteSetPushUrl => {
                OpTraits::NONE.scope(ChangeScope::REFS | ChangeScope::LOG | ChangeScope::CONFIG)
            }
            K::SetSshCommand => OpTraits::NONE.scope(ChangeScope::CONFIG),
            K::SetRepoConfig => OpTraits::NONE.journaled().scope(ChangeScope::CONFIG),
            K::SparseSet | K::SparseDisable => OpTraits::NONE
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFIG),
            // A display filter only: no git state moved.
            K::SetRefVisibility | K::NotifyHostingChange => OpTraits::NONE.scope(ChangeScope::NONE),
            K::ResolveConflict | K::ConflictUnresolve => OpTraits::NONE
                .can_conflict()
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFLICTS),
            K::OperationAbort | K::OperationSkip => OpTraits::NONE.can_conflict(),
            K::LfsInstall | K::LfsTrack | K::LfsUntrack | K::LfsPrune => {
                OpTraits::NONE.scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::LFS)
            }
            K::LfsCheckout => OpTraits::NONE
                .network()
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::LFS),
            K::LfsFetch | K::LfsPush => OpTraits::NONE
                .network()
                .shared_network()
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::LFS),
            K::LfsLock | K::LfsUnlock => OpTraits::NONE
                .network()
                .scope(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::LFS),
            K::ComposeCommits => OpTraits::NONE.journaled().pushed_risk(),
            // Pushes, deletes remote branches and merges: network (async),
            // journaled by hand around the run, can stop on a conflict.
            K::FlowRun | K::RemoteGroupRun => OpTraits::NONE
                .network()
                .can_conflict()
                .journaled()
                .pushed_risk(),
        }
    }
}

impl OperationKind {
    /// Every variant, for tests and tools that must visit each one.
    pub const ALL: [OperationKind; 187] = [
        OperationKind::Discover,
        OperationKind::Init,
        OperationKind::Status,
        OperationKind::Log,
        OperationKind::CountCommits,
        OperationKind::Diff,
        OperationKind::DiffFiles,
        OperationKind::CommitStats,
        OperationKind::Search,
        OperationKind::Refs,
        OperationKind::Stashes,
        OperationKind::Remotes,
        OperationKind::LeftoverRemotes,
        OperationKind::Worktrees,
        OperationKind::Submodules,
        OperationKind::SubmoduleInit,
        OperationKind::SubmoduleUpdate,
        OperationKind::Resolve,
        OperationKind::MergeBase,
        OperationKind::PredictConflicts,
        OperationKind::ReadBlob,
        OperationKind::ReadBlobBytes,
        OperationKind::Blame,
        OperationKind::FileHistory,
        OperationKind::ListTree,
        OperationKind::GrepTree,
        OperationKind::StageFile,
        OperationKind::StageHunk,
        OperationKind::StageLine,
        OperationKind::StageFolder,
        OperationKind::StageAll,
        OperationKind::UnstageFile,
        OperationKind::UnstageHunk,
        OperationKind::UnstageLine,
        OperationKind::UnstageFolder,
        OperationKind::UnstageAll,
        OperationKind::Commit,
        OperationKind::Checkout,
        OperationKind::Discard,
        OperationKind::DiscardHunk,
        OperationKind::DiscardLine,
        OperationKind::BranchCreate,
        OperationKind::BranchCreateTracking,
        OperationKind::BranchDelete,
        OperationKind::BranchDeleteRemote,
        OperationKind::BranchRename,
        OperationKind::BranchSetUpstream,
        OperationKind::BranchSetRemotes,
        OperationKind::RemoteGroupStatus,
        OperationKind::RemoteGroupPlan,
        OperationKind::RemoteGroupRun,
        OperationKind::FastForward,
        OperationKind::WorktreeAdd,
        OperationKind::WorktreeChanges,
        OperationKind::WorktreeRemove,
        OperationKind::PatchCreate,
        OperationKind::PatchApply,
        OperationKind::ApplyCommit,
        OperationKind::DeleteFile,
        OperationKind::ConvertFiles,
        OperationKind::AddToGitignore,
        OperationKind::GitattributesTreat,
        OperationKind::BinaryPaths,
        OperationKind::MailmapIdentities,
        OperationKind::MailmapResolve,
        OperationKind::MailmapAdd,
        OperationKind::TagCreate,
        OperationKind::TagDelete,
        OperationKind::TagAnnotate,
        OperationKind::TagPush,
        OperationKind::TagDeleteRemote,
        OperationKind::TagFetch,
        OperationKind::Merge,
        OperationKind::Rebase,
        OperationKind::RebaseInteractive,
        OperationKind::RebasePlan,
        OperationKind::RebaseState,
        OperationKind::RebaseContinue,
        OperationKind::RebaseSkip,
        OperationKind::RebaseAbort,
        OperationKind::CherryPick,
        OperationKind::Revert,
        OperationKind::Reset,
        OperationKind::Reword,
        OperationKind::AmendCommit,
        OperationKind::SplitSource,
        OperationKind::SplitCommit,
        OperationKind::RewritePreview,
        OperationKind::RewriteApply,
        OperationKind::RewritePush,
        OperationKind::RewriteNote,
        OperationKind::RewriteMaps,
        OperationKind::RewritePurgeCheck,
        OperationKind::RewritePurge,
        OperationKind::RewriteDiscard,
        OperationKind::RewriteRows,
        OperationKind::RewriteBlobDiff,
        OperationKind::RewriteScriptCheck,
        OperationKind::RewriteScriptTest,
        OperationKind::RewriteTrustScript,
        OperationKind::FlowTrustHooks,
        OperationKind::RewriteScanSecrets,
        OperationKind::RewriteAnalyze,
        OperationKind::RewriteCheckSigning,
        OperationKind::RewriteTools,
        OperationKind::StashPush,
        OperationKind::StashPop,
        OperationKind::StashApply,
        OperationKind::StashDrop,
        OperationKind::StashFromCommit,
        OperationKind::AutoStashStartOver,
        OperationKind::Fetch,
        OperationKind::FetchAll,
        OperationKind::FetchRefAsBranch,
        OperationKind::FetchRefspec,
        OperationKind::RemoteAccessCheck,
        OperationKind::RepoAuthGet,
        OperationKind::RepoAuthSet,
        OperationKind::RepoAuthClear,
        OperationKind::RepoAuthTest,
        OperationKind::Pull,
        OperationKind::Push,
        OperationKind::PushForReview,
        OperationKind::Clone,
        OperationKind::RemoteAdd,
        OperationKind::RemoteRemove,
        OperationKind::RemoteRename,
        OperationKind::RemoteSetUrl,
        OperationKind::RemoteSetPushUrl,
        OperationKind::SshCommand,
        OperationKind::SetSshCommand,
        OperationKind::RepoConfig,
        OperationKind::Workflow,
        OperationKind::FlowPlan,
        OperationKind::FlowRun,
        OperationKind::SetRepoConfig,
        OperationKind::SparseStatus,
        OperationKind::SparseSet,
        OperationKind::SparseDisable,
        OperationKind::Conflicts,
        OperationKind::ConflictEntries,
        OperationKind::ConflictShow,
        OperationKind::ResolveConflict,
        OperationKind::ConflictUnresolve,
        OperationKind::OperationState,
        OperationKind::OperationAbort,
        OperationKind::AmContinue,
        OperationKind::AmSkip,
        OperationKind::AmAbort,
        OperationKind::IndexLock,
        OperationKind::IndexLockRemove,
        OperationKind::OperationSkip,
        OperationKind::LfsStatus,
        OperationKind::LfsFiles,
        OperationKind::LfsInstall,
        OperationKind::LfsTrack,
        OperationKind::LfsUntrack,
        OperationKind::LfsFetch,
        OperationKind::LfsPull,
        OperationKind::LfsCheckout,
        OperationKind::LfsPush,
        OperationKind::LfsPrune,
        OperationKind::LfsLocks,
        OperationKind::LfsLock,
        OperationKind::LfsUnlock,
        OperationKind::Hello,
        OperationKind::CredentialGrant,
        OperationKind::ComposeCommits,
        OperationKind::Undo,
        OperationKind::Redo,
        OperationKind::JournalState,
        OperationKind::JournalList,
        OperationKind::JournalRestore,
        OperationKind::Outside,
        OperationKind::Reflog,
        OperationKind::ReflogRestore,
        OperationKind::LostCommits,
        OperationKind::Shutdown,
        OperationKind::SetRefVisibility,
        OperationKind::NotifyHostingChange,
        OperationKind::UiAttach,
        OperationKind::UiPresence,
        OperationKind::UiReport,
        OperationKind::UiShow,
        OperationKind::UiQuery,
        OperationKind::JobList,
        OperationKind::JobCancel,
    ];

    /// The job-control pseudo-operations: answered by the daemon's job
    /// registry on the connection's reader, ahead of the request queue, so a
    /// running pull never delays them.
    pub const fn is_job_control(self) -> bool {
        matches!(self, OperationKind::JobList | OperationKind::JobCancel)
    }

    /// The view-control pseudo-operations (`Ui*`): answered by the daemon's
    /// viewer registry, never dispatched to a `GitBackend`, never journaled
    /// and never a mutation of the repository.
    pub const fn is_ui_control(self) -> bool {
        matches!(
            self,
            OperationKind::UiAttach
                | OperationKind::UiPresence
                | OperationKind::UiReport
                | OperationKind::UiShow
                | OperationKind::UiQuery
        )
    }
}

/// Audit-log entry recorded by the daemon after each mutation, surfaced in the TUI's recent-activity panel and in CLI/MCP audit output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub kind: OperationKind,
    pub description: String,
    pub timestamp: i64,
    pub auto_stashed: bool,
    pub succeeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The view-control operations can never be journaled, never touch the
    /// repository (no scope, no conflict probe) and are not network work.
    #[test]
    fn ui_control_operations_are_view_only() {
        let ui: Vec<OperationKind> = OperationKind::ALL
            .iter()
            .copied()
            .filter(|k| k.is_ui_control())
            .collect();
        assert_eq!(ui.len(), 5);
        for kind in ui {
            let t = kind.traits();
            assert!(!t.journaled && !t.pushed_risk && !t.history, "{kind:?}");
            assert!(
                !t.can_conflict && !t.network && !t.shared_network,
                "{kind:?}"
            );
            assert_eq!(t.scope, ChangeScope::NONE, "{kind:?}");
        }
        assert!(!OperationKind::Commit.is_ui_control());
    }

    /// `ALL` names every variant exactly once: the variants are counted in
    /// this file's own source, so adding one without listing it fails here.
    #[test]
    fn all_lists_every_variant_once() {
        let source = include_str!("operation.rs");
        let start = source.find("pub enum OperationKind {").unwrap();
        let body = &source[start..];
        let body = &body[..body.find("\n}\n").unwrap()];
        let declared: Vec<&str> = body
            .lines()
            .filter_map(|l| l.strip_prefix("    ")?.strip_suffix(','))
            .filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric()))
            .collect();
        let listed: Vec<String> = OperationKind::ALL
            .iter()
            .map(|k| format!("{k:?}"))
            .collect();
        assert_eq!(declared, listed);
    }
}
