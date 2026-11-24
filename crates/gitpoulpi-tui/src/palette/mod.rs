//! The Ctrl+P command palette's catalogue: every command the app can run,
//! what it's called, which key runs it directly, and whether it's
//! applicable right now.
//!
//! The palette is the escape hatch that reaches 100% of the functionality
//! without memorizing shortcuts (architecture principle 5), so a feature
//! reachable only by a chord or a right-click is unfinished.
//!
//! Everything here is pure -- entries, a match-and-rank function, and an
//! applicability predicate over a plain-data context -- so it is testable
//! without a terminal, a daemon or a repo. The dispatch `match` needs the
//! render loop's locals and stays in `lib.rs`.
//!
//! `PaletteContext` is all `bool`s rather than `Focus`/`RefRowKind`/
//! `CentralView`, so this module cannot grow UI state of its own.
//!
//! - `catalog`: the entry table (one constant).
//! - `availability`: `PaletteContext` and `unavailable_reason`.
//! - `matching`: scoring and ranking.

use gitpoulpi_core::domain::{ConflictSide, ResetMode};

use crate::diff_view::DiffViewMode;
use crate::repo_menu::RepoMenuAction;
use crate::status_pane::FileContextAction;
use crate::{
    AmendScope, CommitField, HostingCommand, LogColumn, RefField, SidebarSection, StatusSection,
};
use gitpoulpi_config::PullButtonMode;
use gitpoulpi_i18n::msg;

impl crate::ToolbarButton {
    /// The palette command a toolbar button runs: the button is a mouse
    /// twin of it, and clicking one goes through the same `run_command`.
    pub(crate) fn command(self) -> PaletteCommand {
        use crate::ToolbarButton as B;
        match self {
            B::Undo => PaletteCommand::Undo,
            B::Redo => PaletteCommand::Redo,
            B::Pull => PaletteCommand::Pull,
            B::Push => PaletteCommand::Push,
            B::Branch => PaletteCommand::NewBranch,
            B::Tag => PaletteCommand::NewTag,
            B::Stash => PaletteCommand::StashPush,
            // The toolbar pops the latest stash; the palette entry the
            // selected one (see [`StashTarget`]).
            B::Pop => PaletteCommand::StashPop(StashTarget::Latest),
            B::Search => PaletteCommand::SearchLog,
            // The LFS button is the dropdown itself.
            B::Lfs => PaletteCommand::LfsMenu,
            B::Flow => PaletteCommand::Flow(crate::flow::FlowCmd::Menu),
        }
    }
}

/// Which stash [`PaletteCommand::StashPop`] pops, and whether it announces it.
///
/// The surfaces disagree today and the maintainer has not chosen one answer, so
/// each keeps its own behavior, spelled out here instead of buried in three
/// copies of the code (`pty_command_parity` pins the differences):
/// - `Selected`: the palette and the Refs menu. The Log's selected stash when
///   the Log has focus, else the STASHES pane's selection when it has focus,
///   else `stash@{0}`; silent.
/// - `LatestOrLogRow`: the `Ctrl+G o` leader key. The Log's selected stash
///   when the Log has focus, else the latest; announces which.
/// - `Latest`: the toolbar's Pop button. Always `stash@{0}`; announces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StashTarget {
    Selected,
    LatestOrLogRow,
    Latest,
}

/// A command the Ctrl+P palette can run.
///
/// Every variant must appear exactly once in [`PALETTE_COMMANDS`] - the
/// tests at the bottom of this module enforce that in both directions, so
/// a new variant can't be added without also being made reachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteCommand {
    // -- Navigation ------------------------------------------------
    FocusRefs,
    FocusLog,
    FocusStatus,
    /// Focus the Refs sidebar's filter field, as if `/` had been pressed
    /// while the sidebar had focus. One of the `Ctrl+N` badge targets
    /// see `crate::FocusTarget::RefsFilter`.
    FocusRefsFilter,
    /// Focus one Refs sidebar section directly (LOCAL, REMOTE, WORKTREES,
    /// STASHES, TAGS) - the fine-grained half of `Ctrl+N`'s badges that
    /// `FocusRefs` alone doesn't reach (`FocusRefs` lands on whichever
    /// section was already open).
    FocusRefsSection(SidebarSection),
    /// Focus one half of the third pane (Unstaged/Staged) directly, the
    /// Status equivalent of `FocusRefsSection`.
    FocusStatusSection(StatusSection),
    CycleFocus,
    CycleFocusBack,
    /// Opens the Log's incremental search bar. Same path as `f` and the
    /// toolbar's magnifier button.
    SearchLog,
    /// Collapses the Refs pane to a thin restore strip, or restores it --
    /// same action as `Ctrl+N Shift+H`, the chevron on the pane's own border,
    /// and the strip left behind once it's collapsed. Used to be a bare
    /// `Ctrl+K`; moved behind the `Ctrl+N` prefix because `Ctrl+K` clashes
    /// with terminal-reserved chords in real terminals.
    ToggleRefsPaneCollapsed,
    /// The Status pane's half of the same toggle (`Ctrl+N Shift+L`, formerly the
    /// bare `Ctrl+L`, moved for the same reason).
    ToggleStatusPaneCollapsed,
    /// A notch of the Refs or the Status pane's width: the keyboard twin of
    /// dragging a pane divider.
    ResizePane(crate::pane_layout::PaneResize),
    /// Grow / shrink the active Refs sidebar section by a row, or give
    /// every section its automatic height back: the keyboard (`Ctrl+N +`,
    /// `Ctrl+N -`) and palette twins of dragging a section's bottom border.
    GrowRefsSection,
    ShrinkRefsSection,
    ResetRefsSectionSizes,
    /// Every remembered size back to its default: the pane dividers, the
    /// Refs sections, the commit message block, the Log column widths, the
    /// merge editor and the folder picker.
    ResetLayout,
    /// The right panel's commit message block: resize it (twins of dragging
    /// its splitter), scroll it (twins of the wheel), and the file list's
    /// sort toggle (twin of the `A->Z` button).
    GrowCommitMessage,
    ShrinkCommitMessage,
    ResetCommitMessageSize,
    ScrollCommitMessageDown,
    ScrollCommitMessageUp,
    /// `Commit Details: Open Link in Message...`: the keyboard twin of a click
    /// on a web address of the message (asks first, `link_confirm`).
    OpenMessageLink,
    ToggleCommitFileSort,

    // -- Refs ------------------------------------------------------
    CheckoutSelectedRef,
    MergeSelectedRef,
    /// Replay the current branch's commits on top of the selected one.
    /// Sits in the Refs group beside Merge because it answers the same
    /// question - "integrate that branch into mine" - with the other of
    /// the two answers.
    RebaseOntoSelectedRef,
    /// The keyboard twin of dragging the selected ref onto the
    /// checked-out branch: opens the drop menu (merge / rebase /
    /// interactive rebase / fast-forward / push / pull, whichever the
    /// pair allows - see `ref_drop`) for it.
    DropSelectedRefOnCurrent,
    /// The keyboard twin of picking a ref up with the mouse: marks the
    /// selected ref as the source of a drop (again on the marked ref, or
    /// `Esc`, clears it).
    MarkRefAsDropSource,
    /// ... and of letting go: opens the drop menu (the same one a drag
    /// opens, `ref_drop::drop_menu_entries`) for the marked ref on the
    /// selected one.
    DropMarkedRefHere,
    RenameSelectedBranch,
    DeleteSelectedRef,
    /// Delete the selected branch from its remote (`origin/x`), keeping
    /// the local one - the branch menu's `Delete <remote>/<branch>`.
    DeleteSelectedRemoteBranch,
    /// Delete the selected branch locally and on its remote.
    DeleteSelectedBranchAndRemote,
    /// Copy the URL of the selected branch on its remote's host.
    CopySelectedBranchLink,
    /// Copy the URL of the Log's selected commit on the remote's host.
    CopySelectedCommitLink,
    /// Open Copy as Mermaid gitGraph on the Log's selected commits.
    CopySelectedAsMermaid,
    /// Create an annotated tag at the Log's selected commit.
    CreateAnnotatedTagAtSelectedCommit,
    /// Fast-forward the selected (local) branch to the checked-out one.
    FastForwardSelectedToCurrent,
    /// Fast-forward the checked-out branch to the selected branch (local
    /// or remote).
    FastForwardCurrentToSelected,
    /// Create worktree from the selected branch or tag.
    CreateWorktreeFromSelectedRef,
    /// Start a pull request from the selected local branch: the ref menu's own row, reachable
    /// without a selection made
    /// through the mouse.
    StartPullRequestFromSelectedBranch,
    /// The Reflog view of the selected local branch: the ref menu's own row.
    ReflogSelectedBranch,
    /// Open the Worktrees section's selected worktree as a repository tab
    /// (focusing its tab when it already has one) - the double-click's
    /// keyboard twin.
    OpenWorktreeInTab,
    /// Run an "Open in" entry on the selected worktree's folder (the entries
    /// marked "Can open a worktree"): a small menu at the row.
    WorktreeOpenInSelected,
    /// Merge the selected worktree's branch into another branch.
    WorktreeMergeSelected,
    /// Remove the selected worktree (a confirmation that also offers to save
    /// its changes).
    WorktreeRemoveSelected,
    /// The Worktrees header's `+`: from the current branch, from the
    /// selected commit, or choosing a branch.
    CreateWorktreeMenu,
    /// Create worktree from the current branch (the dialog proposes a new
    /// branch: the current one is checked out already).
    CreateWorktreeFromCurrentBranch,
    /// Create worktree, opening the dialog on its Ref list.
    CreateWorktreeChooseBranch,
    /// Show the Worktrees section, even in a repository with no worktree.
    ShowWorktreesSection,
    /// Update the Submodules section's selected submodule (cloning it when
    /// it is not cloned yet).
    SubmoduleUpdateSelected,
    /// Register the selected submodule in `.git/config`.
    SubmoduleInitSelected,
    /// Open the selected submodule as a repository tab.
    SubmoduleOpenSelected,
    /// Update every submodule (cloning the ones not cloned yet).
    SubmoduleUpdateAll,
    /// Create worktree at the Log's selected commit.
    CreateWorktreeAtSelectedCommit,
    /// Create patch from the Log's selected commit.
    CreatePatchAtSelectedCommit,
    /// Create patch from the Log's selected commit to HEAD.
    CreatePatchFromSelectedCommitToHead,
    /// Create patch from the working tree's current changes.
    CreatePatchFromWip,
    /// Apply a patch file to the working tree.
    ApplyPatch,
    // -- Git LFS ---------------------------------------------------
    /// `git lfs install --local` for this repository.
    LfsInitialize,
    /// Prompt for a pattern (`*.psd`) and route it through LFS.
    LfsTrackPattern,
    LfsUntrackPattern,
    LfsFetch,
    /// Open the toolbar's `LFS` dropdown (keyboard twin of clicking it).
    LfsMenu,
    /// `git lfs fetch --all`: every ref's objects.
    LfsFetchAll,
    /// `git lfs checkout`: real content over every pointer, offline.
    LfsCheckout,
    /// `git lfs push --all`: every ref's objects.
    LfsPushAll,
    LfsPull,
    LfsPush,
    LfsPrune,
    LfsShowLocks,
    /// Open https://git-lfs.com - the keyboard twin of the "not
    /// installed" banner.
    LfsOpenInstallPage,
    /// `r` in a Git LFS file's diff: card <-> raw pointer text.
    LfsToggleRawPointer,
    /// The file menu's LFS rows, on the Status pane's selected file.
    LfsTrackSelectedFile,
    LfsTrackSelectedExtension,
    LfsLockSelectedFile,
    LfsUnlockSelectedFile,
    LfsForceUnlockSelectedFile,
    /// Send the selected tag to `origin`. Grayed out on anything that is
    /// not a tag, and on a tag every remote already has - which is the
    /// same state the sidebar's marker reports, so the palette and the
    /// row can never tell different stories.
    PushSelectedTag,
    /// Delete the selected tag from `origin`, keeping the local one.
    /// Separate from `DeleteSelectedRef`, which is the local half: they
    /// are two different destructions, and conflating them is how a
    /// published tag disappears by accident.
    DeleteSelectedTagFromRemote,
    /// Fetch the selected tag from the remote that has it: for a tag that
    /// lives only on a remote (`tag_where_ui`).
    FetchSelectedTag,
    /// Use the remote's version of the selected tag that differs: the local
    /// tag moves to it (no network, undoable).
    TakeRemoteSelectedTag,
    /// Force-push the selected tag that differs to the remote that has it at
    /// another commit (confirmed, leased, undoable).
    ForcePushSelectedTag,
    /// Re-make the selected tag as an annotated one, where it is.
    AnnotateSelectedTag,
    /// Move the Refs pane's selected tag to the commit selected in the Log:
    /// the keyboard twin of dropping the tag on a commit. A fast-forward runs
    /// at once; a move that is not one asks first.
    MoveSelectedTagToSelectedCommit,
    ToggleHideInGraph,
    /// Scroll the Log to the selected ref's (or stash's) commit.
    RevealSelectedRefInGraph,
    ToggleSolo,
    StopSoloing,
    /// Unhides every hidden ref at once - the sidebar title's "Show All"
    /// button and `Shift+H`.
    ShowAllHiddenRefs,
    /// Hide/solo every ref under the selected Refs-sidebar folder or
    /// remote-root row at once - the folder's own eye gutter and context
    /// menu, reachable from the palette when a folder is selected.
    ToggleHideSelectedFolder,
    ToggleSoloSelectedFolder,
    FilterRefs,
    ClearRefsFilter,
    NextRefSection,
    PrevRefSection,
    ToggleSectionCollapse,
    /// Expand/collapse the folders of the active refs section, or the
    /// selected folder row and what is under it (`tree_fold`).
    RefsFold(crate::tree_fold::TreeFold),
    /// The same for the focused file list (Unstaged, Staged, the commit's
    /// files, View all files).
    FilesFold(crate::tree_fold::TreeFold),
    /// Opens the left pane's show/hide checklist of sections.
    SidebarSectionsMenu,
    /// Shows or hides one left-pane section (the last visible one stays).
    ToggleSidebarSection(SidebarSection),
    /// Shows every left-pane section again.
    ResetSidebarSections,
    /// Makes this repository's shown sections the default for all of them.
    UseSidebarSectionsForAll,
    ToggleHierarchy,

    // -- Git -------------------------------------------------------
    StageSelectedFile,
    UnstageSelectedFile,
    StageAll,
    /// The working-changes toolbar's global trash button: discard every Unstaged path, always
    /// confirmed.
    DiscardAllChanges,
    Commit,
    /// Commit the staged changes with the composer's message, from anywhere
    /// (`Ctrl+G Enter`).
    CommitNow,
    /// Select the Log's WIP row and put the caret in its inline message
    /// field - the commit composer's Summary, edited in the Log. The
    /// keyboard's `c` in the Log, and a click on the field.
    EditWipMessage,
    /// Fetch every remote, pruning - what the toolbar's Fetch does.
    Fetch,
    /// Whatever the Pull button is set to (its dropdown).
    Pull,
    /// One specific pull mode, whatever the button is set to.
    PullWithMode(PullButtonMode),
    /// Fetch, then bring every local branch up to its upstream; the
    /// checked-out one goes through the Pull button's mode.
    PullAllBranches,
    /// The same, with one specific pull mode for the checked-out branch.
    PullAllBranchesWithMode(PullButtonMode),
    /// Opens the Pull button's dropdown: choose what a click on Pull does.
    ChoosePullButtonAction,
    /// Push to the default remote (or ask, per `git.push_behavior`).
    Push,
    /// Opens the Push button's dropdown: push to a chosen remote, once.
    PushToRemote,
    /// Force push the checked-out branch, `--force-with-lease`, confirmed.
    ForcePushWithLease,
    /// The one-off grouped push: pick the remotes, see the plan, run
    /// (`push_group_ui`).
    PushToSelectedRemotes,
    /// Push to the upstream and every extra push target (the Push button's
    /// answer on a branch with targets).
    PushToAllTargets,
    /// Pull the upstream and every extra pull source.
    PullFromAllSources,
    /// Pull from one remote, once: a chooser of the remotes.
    PullFromOneRemote,
    /// Reopen the progress window of the last grouped push.
    ShowGroupedPush,
    /// The paused strip's Continue.
    ContinueGroupedPush,
    /// The paused strip's Abort group.
    AbortGroupedPush,
    /// The progress window's Retry failed.
    RetryFailedPushes,
    /// Choose, type or unset the upstream of the selected local branch,
    /// else of the checked-out one.
    SetUpstream,
    NewBranch,
    NewTag,
    /// Push every tag no remote is known to have, to `origin`, in one
    /// operation. Repository-wide, so unlike the three commands above it
    /// needs no selection - it is the answer to "I made five tags this
    /// afternoon and none of them are on the server".
    PushAllUnpushedTags,
    StashPush,
    /// Stash all changes, asking for a message first (`Ctrl+G m`).
    StashPushWithMessage,
    /// Pop a stash. Which one, and whether the status line says so, depends
    /// on the surface that asked: see [`StashTarget`].
    StashPop(StashTarget),
    StashApply,
    StashDrop,
    /// Apply the Log's selected commit on top of HEAD.
    CherryPickSelectedCommit,
    /// Commit the inverse of the Log's selected commit.
    RevertSelectedCommit,
    /// Open the Edit commit message popup on the Log's selected commit.
    EditSelectedCommitMessage,
    /// The Amend popup on the Log's selected commit, over `AmendScope`'s fields.
    AmendSelectedCommit(AmendScope),
    SplitSelectedCommit,
    /// Open the interactive rebase popup for the range just after the
    /// Log's selected commit up to HEAD.
    InteractiveRebaseFromSelectedCommit,
    /// Resume a rebase stopped by a conflict or an `edit` step.
    RebaseContinue,
    /// Skip the commit a rebase is stopped at.
    RebaseSkip,
    /// Abort a rebase in progress, restoring the branch and HEAD.
    RebaseAbort,
    /// Continue the `git am` stopped on a patch (the banner's Continue).
    AmContinue,
    /// Skip the patch `git am` stopped on.
    AmSkip,
    /// Abort the `git am` in progress (asks first).
    AmAbort,
    /// Delete a stale `.git/index.lock` (asks first).
    IndexLockRemove,
    /// Pop the pending auto-stash back onto the working tree.
    AutoStashPop,
    /// The pop conflicted and the conflicts are resolved: keep the working
    /// tree as it is (unstaged) and drop the now redundant auto-stash.
    AutoStashFinish,
    /// Select the pending auto-stash in the STASHES section.
    AutoStashShow,
    /// Open the popup of the predicted conflicts with the base branch (the
    /// badge's twin).
    ConflictsShow,
    /// Drop the pending auto-stash (asks first).
    AutoStashDrop,
    /// Start over: discard the stash's files' changes, undo the operation
    /// that auto-stashed, pop the stash again (asks first).
    AutoStashStartOver,
    /// Move the current branch to the Log's selected commit. One command
    /// per [`ResetMode`] rather than one command plus a mode prompt: the
    /// three differ in exactly how much work they throw away, which is
    /// the thing being chosen, so it belongs in the label the user reads
    /// rather than behind a second step.
    ResetToSelectedCommit(ResetMode),
    /// Create a branch/tag pointing at the Log's selected commit rather
    /// than at HEAD - the commit menu's `Create ... here...` rows.
    /// Separate commands from `NewBranch`/`NewTag` because the target is
    /// what differs, and a palette entry that silently used HEAD when a
    /// commit was selected would be the wrong kind of surprise.
    CreateBranchAtSelectedCommit,
    CreateTagAtSelectedCommit,
    /// Detach HEAD onto the Log's selected commit -
    /// `Checkout > this commit`. Distinct from `CheckoutSelectedRef`,
    /// which moves HEAD to a ref in the Refs pane.
    CheckoutSelectedCommit,
    /// Put one cell of the Log's selected commit on the system clipboard.
    /// One command per field, so each is findable by typing the field's
    /// own name into the palette.
    CopyCommitField(CommitField),
    /// Put the selected ref's name, fully-qualified name or target SHA on
    /// the system clipboard.
    CopyRefField(RefField),
    /// Resolve the selected conflicted file by taking one side whole.
    /// `ConflictResolution::UseContent` (hand-merged text) has no editor
    /// behind it yet, so only the three `TakeSide` variants appear.
    ResolveConflictWith(ConflictSide),
    /// Open the merge conflict editor on the Status pane's selected
    /// conflicted file (else the Diff view's, else the first one): the
    /// keyboard twin of Enter / double-click on a `!` row and of the
    /// operation banner's Resolve button.
    OpenMergeEditor,
    /// One action of the open merge editor (`merge_view::MergeCmd`): every
    /// key, button and menu row of the editor has an entry.
    Merge(crate::merge_view::MergeCmd),
    /// Abort the merge, cherry-pick or revert in progress (the operation
    /// banner's Abort; a rebase has its own).
    AbortOperation,
    /// Skip the cherry-pick or revert in progress (the operation banner's
    /// Skip), the way out of one that became empty.
    SkipOperation,
    /// The Repository Management tab with the Clone dialog open.
    CloneRepository,
    /// The Repository Management tab with the Init dialog open: `git
    /// init` into a chosen folder, then open it as a tab.
    CreateRepository,
    /// Goes to the Repository Management tab (the `⌂`).
    OpenRepository,
    /// The management tab's Browse: the folder picker, and the chosen
    /// repository opened as a tab.
    BrowseRepository,
    /// One of the management tab's own actions (its buttons, its row
    /// actions, its section actions).
    Management(ManagementCommand),
    /// A `Workspace: ...` command (`workspace_ui`).
    Workspace(WorkspaceCommand),
    /// The Radar (`radar`): opening it, and the twin of each of its
    /// keys and clicks.
    Radar(crate::radar::RadarCommand),
    /// Release notes and the update check (`updates`): the twin of the status
    /// bar's version chip and of the Release notes tab's buttons.
    Updates(crate::updates::UpdatesCommand),
    /// One row of the Status pane's file context menu, run on the
    /// selected file - the palette twin of right-click / the menu key.
    FileMenu(crate::status_pane::FileContextAction),
    /// Opens the context menu of the selected folder row in the Status
    /// pane (Tree mode) - the palette twin of right-click / the menu key.
    OpenFolderMenu,
    /// `Create File` in the repository root: the empty-area menu's row.
    CreateFile,
    /// A transient "New tab" - `Alt+T`, the tab bar's `+`.
    NewTab,
    /// Opens the current tab's menu - `Ctrl+T`, right-click on a tab.
    OpenTabMenu,
    /// Opens the `+` menu: New tab, Radar, Workspaces.
    OpenNewMenu,
    /// One row of that menu, run directly on the current repository: the
    /// active tab, or - on the management tab - the selected row.
    TabMenu(RepoMenuAction),
    /// The tab bar's "Quit & cd here" / "Terminal here" buttons, run on the
    /// active tab - also when the bar hides them (auto mode, the launch
    /// folder, a narrow window).
    TabDir(crate::tab_dir::TabDirButton),
    /// "Tab: Open in default": runs the default "Open in" entry on the active
    /// tab, no popup.
    TabOpenInDefault,
    /// `Repository: Change Profile...`: the toolbar's profile combobox,
    /// opened on the keyboard.
    ChangeProfile,
    /// `Help: About GitPoulpi`: the About window the title bar's octopus opens.
    AboutGitPoulpi,
    /// `Help: Licenses and Credits`: the About window on its Licenses view.
    AboutLicenses,
    /// `Integrations: Review security check`: the dialog that asks about an
    /// integration whose address, account or certificate changed.
    TrustReview,
    /// `View: Activity Log`: the window the status bar's Activity button opens.
    OpenActivityLog,
    /// `View: Global Activity Log`: the Activity Log's Global tab, reachable
    /// from the Home tab too.
    OpenGlobalLog,
    /// `Jobs: ...`: the background jobs panel and the Cancel of each job
    /// (`jobs_ui`), the keyboard twins of the status bar indicator.
    Jobs(crate::jobs_ui::JobsCmd),
    /// `View: Copy diagnostics`: version, system, the latest failure and the
    /// log tail, redacted, for pasting to an AI.
    CopyDiagnostics,
    /// `Activity Log: ...`: show or hide a log level, all or none; opens the
    /// window on its Application tab.
    ActivityLevel(crate::activity_log::LevelCmd),
    /// `Activity Log: Copy the Last Error` and `Activity Log: Run the
    /// Suggested Action`: the keyboard twins of the entry's Copy and remedy
    /// buttons, for the failure the status line shows.
    CopyLastError,
    RunLastErrorRemedy,
    /// `Activity Log: Open the Last Error`: the keyboard twin of a click on
    /// a failure in the status line.
    OpenLastError,
    /// Moves to the next/previous repository tab. Signed, so one
    /// command shape serves both and the wrap arithmetic lives in one
    /// place (`GlobalSession::cycle`).
    CycleRepositoryTab(bool),
    /// Moves the current repository tab one place right (`true`) or left,
    /// keeping the new order in the session (the keyboard twin of dragging
    /// a tab).
    MoveRepositoryTab(bool),
    /// Closes the current repository's tab. The repository itself is
    /// untouched - nothing on disk changes.
    CloseRepositoryTab,
    /// Opens the tab picker (`Ctrl+N Tab`): a popup listing every open
    /// repository tab, filtered by typing, that switches to the one
    /// chosen.
    SelectTab,
    /// Opens the workspace search (`Ctrl+Alt+F`): a fuzzy finder over the
    /// refs, worktrees, stashes, commits, pull/merge requests and issues of
    /// every repository of the active workspace (else the open tabs).
    WorkspaceSearch,
    /// Opens the Add remote popup (the REMOTE header's `+` button).
    AddRemote,
    /// Pick a remote, then confirm removing it.
    RemoveRemote,
    /// The rest of a remote name's right-click menu, each acting on the
    /// remote selected in the REMOTE section or, failing that, asking
    /// which one.
    EditRemote,
    SetDefaultRemote,
    CopyRemoteName,
    CopyRemoteLink,
    FetchRemote,
    /// Opens the selected remote's whole menu - `.` on a remote row.
    RemoteActions,
    /// Repository: Sparse Checkout... - the directories dialog.
    SparseCheckout,
    /// Repository: SSH & Credentials... - Settings on the page that picks how
    /// this repository signs in.
    RepoAuthOpen,
    /// Repository: Test Connection - the page's test of every remote.
    RepoAuthTest,
    /// Repository: Use Default Credentials - asks first, then clears the
    /// repository's sign-in settings.
    RepoAuthDefaults,
    /// Hide or show a whole remote in the graph (every branch of it, the ones
    /// fetched later too) - the remote-name row's menu entry and its eye.
    HideRemoteInGraph,
    ShowRemoteInGraph,
    Undo,
    /// Drop the blocked latest undo entry from the journal (confirmed).
    UndoForget,
    Redo,
    /// Edit: Undo History... - the panel of every action that can be taken
    /// back, one by one (`safety_panel`).
    UndoHistory,
    /// Edit: Reflog... - the past positions of HEAD.
    ReflogHead,
    /// Edit: Recover Lost Commits... - commits no branch reaches.
    LostCommits,
    /// Edit: Restore the branch to where the blocked action left it.
    UndoOffer,
    /// A row action of the open Undo history / Reflog / Lost work panel,
    /// on its selected row (`safety_panel::SafetyCmd`).
    Safety(crate::safety_panel::SafetyCmd),
    /// An action of the open in-app code review (`review_view`).
    Review(crate::review_cmd::ReviewCmd),
    /// An action of the open Kanban board (`board_view`).
    Board(crate::board_cmd::BoardCmd),
    /// An action of the open conversation of an issue or a request
    /// (`item_view`).
    Item(crate::item_cmd::ItemCmd),
    /// An action of the AI writing help of the open text box
    /// (`composer_cmd`).
    Composer(crate::composer_cmd::ComposerCmd),
    /// A branching-workflow command (`flow::cmd`): the `Flow ▾` menu, Start,
    /// Finish, Sync, Promote, Set parent, the grouping toggle, its Settings.
    Flow(crate::flow::FlowCmd),

    // -- View ------------------------------------------------------
    Refresh,
    /// F5: fetch every remote, re-check pull requests and issues, reload.
    RefreshAll,
    ToggleHelp,
    ToggleColumn(LogColumn),
    /// The Log's column keys by palette: pick, move, size (the header's drag twins).
    LogColumn(crate::log_table::LogColumnCmd),

    // -- Diff ------------------------------------------------------
    OpenDiff,
    CloseDiff,
    NextHunk,
    PrevHunk,
    /// Stage / unstage the hunk the Diff view's cursor is in. Two
    /// commands rather than one direction-inferring command, mirroring
    /// `StageSelectedFile`/`UnstageSelectedFile`: which half the diff was
    /// opened from decides which of the two applies, and the other one
    /// grays out with the reason, the way every other contextual command
    /// in this catalogue does.
    StageHunk,
    UnstageHunk,
    /// Begin (or clear) the visual-style line selection inside a hunk.
    SelectLines,
    /// Stage / unstage the selected lines - or, with no selection open,
    /// the one line under the cursor.
    StageLines,
    UnstageLines,
    /// Discard the hunk under the Diff view's cursor / the selected lines.
    /// Unstaged half only; both open the confirm modal.
    DiscardHunk,
    DiscardLines,
    DiffMode(DiffViewMode),
    /// Whole file <-> only the hunks.
    ToggleDiffHunksOnly,
    /// `Path` <-> `Tree` for the right pane's file list (`m`).
    FilesTogglePathTree,
    /// The `View all files` checkbox (`a`).
    FilesToggleViewAll,
    /// The commit composer's Skip hooks checkbox (`--no-verify` for the next
    /// commit): the palette twin of the click and `Shift+H`.
    CommitToggleSkipHooks,
    /// Opens the menu of the focused file list's cursor row, or of its
    /// multi-selection - the palette twin of right-click / the menu key.
    FilesOpenMenu,
    /// Selects every file of the focused file list (Ctrl+click's twin).
    FilesSelectAll,
    /// Clears the multi-selection of the file lists (Esc).
    FilesClearSelection,
    /// Flips the cursor row in the multi-selection (Ctrl+click / Space).
    FilesToggleSelection,
    /// Compare the Log's selected rows (`v`): the right pane lists what
    /// differs between them.
    CompareSelection,
    /// Swap the two ends of that comparison (`x`).
    CompareSwap,
    /// Move the Log cursor to the next row of the comparison.
    CompareGotoNext,
    ToggleWordWrap,
    ToggleWhitespace,
    /// The Diff view's encoding dropdown (`Shift+C`).
    DiffChooseEncoding,
    /// The minimap beside the vertical scrollbar of the diff and the editor
    /// (`ui.minimap`). Global: it flips a setting, whichever view is open.
    ToggleMinimap,
    /// Highlight the changed words of a changed line in the diff views
    /// (`ui.diff_intraline`). Global: it flips a setting, whichever view is
    /// open. Palette only: no key, the Diff view's keys are all taken.
    ToggleDiffIntraline,
    /// `editor.follow_selection`: with the editor open, selecting another
    /// file in the right pane opens it in the editor.
    EditorToggleFollowSelection,
    /// The commit column beside the Diff / File View / Blame / Edit views:
    /// show or hide it, give it the keyboard, resize it.
    CommitColumn(crate::commit_column::ColumnCmd),
    /// The image view's zoom (`+`, `-`, `0`, `1` while an image is shown in
    /// the File View).
    ImageZoomIn,
    ImageZoomOut,
    ImageFit,
    ImageActualSize,
    /// The image diff's `Tab` (the other side of an inline view) and the
    /// divider keys (`Alt+Right`, `Alt+Left`, `Alt+0`).
    ImageFlipSide,
    ImageWidenBefore,
    ImageWidenAfter,
    ImageResetSplit,
    ImageEditIn,
    ImageEditDefault,
    /// An action of the File History split view (`history_view`).
    HistoryView(crate::history_view::HistoryAction),
    /// Opens the history rewrite workbench (`Rewrite: ...`), optionally with
    /// one rule added or the Log's selection as the scope.
    Rewrite(crate::rewrite_view::RewriteEntry),
    /// The "Open in" list of a binary file shown in the Diff view or the File
    /// View (`open_with`), and its default application.
    OpenWith,
    OpenWithDefault,
    Blame,
    /// One of the Blame view's own actions (hunk navigation, the commit
    /// card and its actions, the annotation width).
    BlameOp(crate::blame_view::BlameOp),
    History,
    FileView,
    /// Edit in Working Directory: the built-in editor
    /// on the working-tree copy of the file the user is pointing at.
    EditInWorkingDirectory,
    /// One of the built-in editor's status menus (encoding, line breaks,
    /// indentation): the keyboard twin of clicking its chip.
    EditorMenu(crate::editor::StatusMenu),
    /// Copy text out of the Diff view / File View / Blame / History:
    /// the mouse text selection, the line under the
    /// cursor (the `v` range when one is open), the hunk under it.
    CopySelectedText,
    CopyLine,
    CopyHunk,

    // -- App -------------------------------------------------------
    /// Opens the Preferences screen. The gear in the toolbar's
    /// right-hand corner and `Ctrl+,` are the other two ways in --
    /// architecture principle 5 makes this entry the contract, not the
    /// convenience.
    OpenPreferences,
    /// Opens Preferences with the settings search popup (`/` or `Ctrl+F`
    /// inside Preferences, or a click on the sidebar search box).
    SearchSettings,
    /// Opens Settings on the Keyboard Shortcuts page.
    Keybindings,
    /// The same page with the "reset every shortcut" confirmation open.
    KeybindingsResetAll,
    /// Opens Settings on the Theme row (Appearance).
    Theme,
    /// Opens Settings > Conflict prevention (global): the
    /// default base branches and the named lists.
    ConflictLists,
    /// Opens the Conflict prevention page of this repository.
    ConflictRepoPage,
    /// Opens the theme picker: a second-level list of every theme that
    /// previews as the highlight moves and, on Enter, applies and saves it.
    ChangeTheme,
    /// Opens the language picker: a second-level list of the built-in
    /// languages; Enter applies and saves `ui.language`.
    ChangeLanguage,
    /// Opens Settings on the Launcher group and creates (or updates) the
    /// OS launcher shortcut: the app launcher, the Start menu, the macOS app.
    LauncherCreate,
    /// The same for the Desktop shortcut.
    LauncherCreateDesktop,
    /// Removes the shortcuts GitPoulpi made.
    LauncherRemove,
    /// Opens Settings on the Launcher group and creates the command alias
    /// (`ui.command_alias`).
    LauncherAliasCreate,
    /// Removes the command aliases GitPoulpi made.
    LauncherAliasRemove,
    /// Opens Settings > Agents & MCP and saves a free port as this
    /// installation's MCP bind.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    // only a build with the MCP server lists it
    McpSuggestPort,
    /// Adds GitPoulpi to the user PATH.
    LauncherPathAdd,
    /// Removes what GitPoulpi added to the user PATH.
    LauncherPathRemove,
    /// Adds "Open in GitPoulpi" to the file manager's right-click menu.
    LauncherFolderMenuInstall,
    /// Removes the "Open in GitPoulpi" menu entries GitPoulpi wrote.
    LauncherFolderMenuRemove,
    /// Makes the running installation the primary (asks first when another is).
    LauncherBecomePrimary,
    /// Removes the launchers an older build made under the plain name (asks first).
    LauncherRemoveLegacy,
    /// Forgets an installation whose folder is gone (asks first).
    LauncherForgetGone,
    /// Opens the graph color theme picker (`ui.graph_color_theme`).
    ChangeGraphColorTheme,
    /// Flips `ui.graph_lane_background` between `off` and `row_tint`.
    ToggleGraphRowTints,
    /// `Agent: Show Pending View`: apply the view an AI agent offered (the
    /// footer chip) - `Ctrl+N Enter`.
    AgentShowPending,
    /// `Agent: Dismiss Pending View`: drop the chip - `Esc`, `Ctrl+N
    /// Backspace`.
    AgentDismissPending,
    /// `Agent: Back`: undo the last view change an agent made.
    AgentBack,
    /// AI writes a commit message for the staged changes into the
    /// composer (same as the composer's Generate button and `Ctrl+G i`).
    /// Hidden while `ai.enabled` is off.
    AiGenerateCommitMessage,
    /// Open the stash-message prompt and let AI fill it in.
    AiGenerateStashMessage,
    /// Open Settings > AI's global instruction editor with the prompt
    /// variable dropdown ready (`${repo}`, `${uilang}`, ...).
    AiInsertPromptVariable,
    /// Open Settings > AI on its profile editor, the profile list open:
    /// the keyboard/palette twin of picking a profile with the mouse.
    AiEditProfile,
    /// Open the default profile's local agent once, visibly, in its working
    /// folder, so its first-run questions can be answered by hand.
    AiOpenAgent,
    /// AI explains the Log's selected commit in a read-only popup.
    AiExplainSelectedCommit,
    /// AI proposes a split of the Log's selected commit and opens the split
    /// editor prefilled with it.
    AiSplitSelectedCommit,
    /// AI explains the uncommitted working changes.
    AiExplainWip,
    /// AI proposes a split of the working changes into commits, opened
    /// as an interactive review.
    AiComposeCommits,
    /// The commit composer review's "Move file to commit..." dropdown.
    ComposerMoveFile,
    /// The sensitive-files question's "Show in file": opens the focused file
    /// in the File View at the line of the secret.
    StageGuardShowInFile,
    /// The integration commands: the linked pull request, the sidebar's
    /// pull-request / issue lists, the repository's profile
    /// (`hosting_ui`).
    Hosting(HostingCommand),
    Quit,
    /// A `Settings: ...` row, generated from the Preferences fields
    /// (`settings.rs`); not in [`PALETTE_COMMANDS`].
    Setting(settings::SettingOp),
}

/// The Repository Management tab's actions, as palette commands. Each
/// is also a key on the tab and (for most) a button or a row control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagementCommand {
    OpenSelected,
    ToggleSelectedFavorite,
    CloseOrRemoveSelected,
    SelectedMenu,
    CopySelectedPath,
    OpenSelectedIn,
    Search,
    CollapseAll,
    ExpandAll,
    CloseAllTabs,
    RemoveAllRecent,
    RemoveSelectedFromList,
}

/// The workspace commands: the Home page's controls, the chip's combo and the
/// tab menu's membership rows, as palette commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceCommand {
    New,
    FromTabs,
    Switch,
    /// Leave the workspace: "No workspace".
    Close,
    Manage,
    /// Close the Workspaces tab.
    CloseTab,
    /// Locate the selected missing repository of a workspace.
    Locate,
    Edit,
    Delete,
    FetchAll,
    PullAll,
    CloneMissing,
    AddTab,
    RemoveTab,
}

/// One row of the palette: what it's called, the key that runs it
/// directly (empty when there isn't one), and the command itself.
///
/// The shortcut is shown right-aligned in the popup on purpose - it turns
/// the palette from a fallback menu into the place you *learn* the
/// keymap, which is the only way a keyboard-first app stops depending on
/// its own help overlay.
pub(crate) struct PaletteEntry {
    /// The `Category: Action` label as a message id: resolved at draw and
    /// match time, so a language switch relabels the next frame.
    pub(crate) label: gitpoulpi_i18n::M,
    pub(crate) shortcut: &'static str,
    /// For a command with no key: how to reach it ("double-click",
    /// "Enter on the search row"); replaces `shortcut` when set.
    pub(crate) hint: Option<gitpoulpi_i18n::M>,
    pub(crate) command: PaletteCommand,
    /// The cargo feature this command belongs to (`.requires(..)` in the
    /// catalogue); the palette leaves it out when the build lacks it.
    pub(crate) requires: Option<crate::build_features::Feature>,
    /// A generated row's text (a `Settings: ...` row): its label is built
    /// from a Preferences field, not from the catalogue.
    pub(crate) text: Option<&'static str>,
    /// That text in English, for a query typed in English.
    pub(crate) text_en: Option<&'static str>,
}

impl PaletteEntry {
    /// The row's name in the current language.
    pub(crate) fn name(&self) -> &'static str {
        self.text.unwrap_or_else(|| self.label.get())
    }

    /// The row's name in English.
    pub(crate) fn english(&self) -> &'static str {
        self.text_en.unwrap_or_else(|| self.label.english())
    }

    /// Tags the entry with the feature it needs.
    #[allow(dead_code)] // first used by the slices that gate each feature
    pub(crate) const fn requires(mut self, feature: crate::build_features::Feature) -> Self {
        self.requires = Some(feature);
        self
    }

    /// The label as drawn: an AI command carries the purple sparkle, which
    /// is a glyph (kept in code, outside the messages).
    pub(crate) fn display_label(&self) -> std::borrow::Cow<'static, str> {
        if self.command.is_ai() {
            std::borrow::Cow::Owned(format!("{} {}", crate::theme::AI_SPARK, self.label.get()))
        } else {
            std::borrow::Cow::Borrowed(self.name())
        }
    }
}

/// The feature a command needs without the catalogue saying so: every
/// integration command (and starting a pull request) needs a provider.
pub(crate) const fn command_requires(
    command: &PaletteCommand,
) -> Option<crate::build_features::Feature> {
    match command {
        // The features carved out of `hosting` (each needs a provider too, which
        // `build_features` folds into its own flag).
        // Reviewing a request from the Radar needs the review view too.
        PaletteCommand::Radar(crate::radar::RadarCommand::Act(
            crate::radar::Act::Review
            | crate::radar::Act::ReviewApi
            | crate::radar::Act::CloneReview,
        )) => Some(crate::build_features::Feature::Review),
        PaletteCommand::Radar(crate::radar::RadarCommand::Act(crate::radar::Act::Conversation)) => {
            Some(crate::build_features::Feature::Conversation)
        }
        PaletteCommand::Radar(_) => Some(crate::build_features::Feature::Radar),
        PaletteCommand::Review(_) | PaletteCommand::Hosting(HostingCommand::ReviewPullRequest) => {
            Some(crate::build_features::Feature::Review)
        }
        PaletteCommand::Board(_) | PaletteCommand::Hosting(HostingCommand::OpenBoard) => {
            Some(crate::build_features::Feature::Board)
        }
        PaletteCommand::Item(_)
        | PaletteCommand::Hosting(HostingCommand::OpenIssueConversation) => {
            Some(crate::build_features::Feature::Conversation)
        }
        PaletteCommand::Composer(_) => Some(crate::build_features::Feature::Ai),
        PaletteCommand::CopySelectedAsMermaid => Some(crate::build_features::Feature::Mermaid),
        PaletteCommand::WorkspaceSearch => Some(crate::build_features::Feature::GlobalSearch),
        PaletteCommand::Workspace(_) => Some(crate::build_features::Feature::Workspaces),
        PaletteCommand::Rewrite(_) => Some(crate::build_features::Feature::HistoryRewrite),
        PaletteCommand::Hosting(_)
        | PaletteCommand::TrustReview
        | PaletteCommand::StartPullRequestFromSelectedBranch => {
            Some(crate::build_features::Feature::Hosting)
        }
        PaletteCommand::CreateWorktreeFromSelectedRef
        | PaletteCommand::OpenWorktreeInTab
        | PaletteCommand::WorktreeOpenInSelected
        | PaletteCommand::WorktreeMergeSelected
        | PaletteCommand::WorktreeRemoveSelected
        | PaletteCommand::CreateWorktreeMenu
        | PaletteCommand::CreateWorktreeFromCurrentBranch
        | PaletteCommand::CreateWorktreeChooseBranch
        | PaletteCommand::ShowWorktreesSection
        | PaletteCommand::CreateWorktreeAtSelectedCommit
        | PaletteCommand::FocusRefsSection(SidebarSection::Worktrees) => {
            Some(crate::build_features::Feature::Worktree)
        }
        PaletteCommand::ToggleSidebarSection(SidebarSection::Worktrees) => {
            Some(crate::build_features::Feature::Worktree)
        }
        PaletteCommand::ToggleSidebarSection(SidebarSection::Submodules) => {
            Some(crate::build_features::Feature::Submodule)
        }
        PaletteCommand::ToggleSidebarSection(
            SidebarSection::PullRequests | SidebarSection::Issues,
        ) => Some(crate::build_features::Feature::Hosting),
        PaletteCommand::SubmoduleUpdateSelected
        | PaletteCommand::SubmoduleInitSelected
        | PaletteCommand::SubmoduleOpenSelected
        | PaletteCommand::SubmoduleUpdateAll
        | PaletteCommand::FocusRefsSection(SidebarSection::Submodules) => {
            Some(crate::build_features::Feature::Submodule)
        }
        PaletteCommand::ImageZoomIn
        | PaletteCommand::ImageZoomOut
        | PaletteCommand::ImageFit
        | PaletteCommand::ImageActualSize
        | PaletteCommand::ImageFlipSide
        | PaletteCommand::ImageWidenBefore
        | PaletteCommand::ImageWidenAfter
        | PaletteCommand::ImageResetSplit
        | PaletteCommand::ImageEditIn
        | PaletteCommand::ImageEditDefault => Some(crate::build_features::Feature::ImageView),
        _ => None,
    }
}

const fn entry(
    label: gitpoulpi_i18n::M,
    shortcut: &'static str,
    command: PaletteCommand,
) -> PaletteEntry {
    PaletteEntry {
        label,
        shortcut,
        hint: None,
        requires: command_requires(&command),
        command,
        text: None,
        text_en: None,
    }
}

const fn hint_entry(
    label: gitpoulpi_i18n::M,
    hint: gitpoulpi_i18n::M,
    command: PaletteCommand,
) -> PaletteEntry {
    PaletteEntry {
        label,
        shortcut: "",
        hint: Some(hint),
        requires: command_requires(&command),
        command,
        text: None,
        text_en: None,
    }
}

/// The shipped on-screen label of every action whose dispatch reads the
/// keymap and whose catalogue spelling is a chord: palette shortcut texts
/// are looked up here to find the action they stand for.
pub(crate) fn default_shortcut_actions() -> &'static [(String, crate::keymap::Action)] {
    use crate::keymap::{Ctx, Keymap, STATIC_ACTIONS};
    static MAP: std::sync::OnceLock<Vec<(String, crate::keymap::Action)>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let km = Keymap::defaults_of(STATIC_ACTIONS);
        STATIC_ACTIONS
            .iter()
            .filter(|d| matches!(d.ctx, Ctx::Global | Ctx::GitPrefix | Ctx::NavPrefix) && d.built())
            .flat_map(|d| {
                km.shown_labels(d.action)
                    .into_iter()
                    .map(move |l| (l, d.action))
            })
            .filter(|(l, _)| l.contains("Ctrl+"))
            .collect()
    })
}

/// The catalogue spelling of "the prefix, then whichever badge this frame
/// handed the sub-pane".
pub(crate) const BADGE_SUFFIX: &str = " <badge>";

/// What the palette shows right-aligned for a catalogue shortcut `text`
/// under `km`: each ` / `-separated alternative that names a keymap
/// action is replaced by that action's effective label (an unbound one
/// is dropped); descriptions like `double-click` or `c in Log` stay as
/// written.
pub(crate) fn effective_shortcut(km: &crate::keymap::Keymap, text: &str) -> String {
    let defaults = default_shortcut_actions();
    let mut parts: Vec<String> = Vec::new();
    for alt in text.split(" / ") {
        let alt = alt.trim();
        // `Ctrl+N <badge>`: the prefix follows a rebinding, the badge
        // character is handed out per frame so it stays a placeholder.
        if let Some(head) = alt.strip_suffix(BADGE_SUFFIX) {
            if let Some((_, action)) = defaults.iter().find(|(l, _)| l == head) {
                if let Some(label) = km.shown_labels(*action).into_iter().next() {
                    parts.push(format!("{label}{BADGE_SUFFIX}"));
                }
                continue;
            }
        }
        match defaults.iter().find(|(l, _)| l == alt) {
            Some((_, action)) => {
                if let Some(label) = km.shown_labels(*action).into_iter().next() {
                    parts.push(label);
                }
            }
            None => parts.push(alt.to_string()),
        }
    }
    parts.join(" / ")
}

/// What the palette shows right-aligned for `entry` under `km`: the
/// effective binding of the action the command stands for (plain keys
/// included: `r`, `o`, `m`, ...), else [`effective_shortcut`] of its
/// catalogue text.
pub(crate) fn entry_shortcut(km: &crate::keymap::Keymap, entry: &PaletteEntry) -> String {
    // A `Settings: ...` row shows the setting's value where a key would be.
    if let PaletteCommand::Setting(op) = entry.command {
        return settings::value_text(op);
    }
    match command_action(entry) {
        // The tab-switch rows list every binding (joined like the help
        // overlay does): Ctrl+Tab is as much "the" key as Ctrl+PageDown.
        // Other rows keep showing the primary one.
        Some(action) if matches!(entry.command, PaletteCommand::CycleRepositoryTab(_)) => {
            km.shown_labels(action).join(" / ")
        }
        Some(action) => km.shown(action),
        None => {
            // A key the user gave the command shows in place of the hint.
            if let Some(own) = bindings::palette_action(entry) {
                let bound = km.shown(own);
                if !bound.is_empty() {
                    return bound;
                }
            }
            match entry.hint {
                Some(hint) => hint.get().to_string(),
                None => effective_shortcut(km, entry.shortcut),
            }
        }
    }
}

/// The keymap action whose shortcut this row shows, if any: what the row's
/// "configure the shortcut" button (and Alt+K) opens Settings > Keyboard &
/// mouse filtered on.
pub(crate) fn bind_action(entry: &PaletteEntry) -> Option<crate::keymap::Action> {
    bindings::static_bind_action(entry).or_else(|| bindings::palette_action(entry))
}

/// The "configure the shortcut" button drawn at the end of a palette row
/// that has one ([`bind_action`]), and the cells it takes.
///
/// Plain ASCII, so every terminal and font draws it one cell per char. It was
/// the keyboard emoji `U+2328 U+FE0F`: WezTerm (Windows) drew it off its two
/// cells, so the button showed as a bar and the cell ratatui skips after a
/// wide glyph kept what was behind the popup (see where-we-are.md, "Emoji
/// presentation").
pub(crate) const KEY_BUTTON: &str = "[k] "; // i18n-english
pub(crate) const KEY_BUTTON_W: u16 = 4;

/// The registry action a palette command is the twin of, when its key is
/// a plain one that the catalogue text cannot look up.
pub(crate) fn command_action(entry: &PaletteEntry) -> Option<crate::keymap::Action> {
    use crate::keymap::Action as A;
    match entry.command {
        PaletteCommand::ToggleHelp => Some(A::GlobalHelp),
        PaletteCommand::CycleRepositoryTab(true) => Some(A::GlobalNextTab),
        PaletteCommand::CycleRepositoryTab(false) => Some(A::GlobalPrevTab),
        PaletteCommand::Safety(crate::safety_panel::SafetyCmd::ArrangeDialog) => {
            Some(A::NavArrangeDialog)
        }
        PaletteCommand::AboutGitPoulpi => Some(A::NavOpenAbout),
        PaletteCommand::OpenActivityLog => Some(A::NavOpenActivityLog),
        PaletteCommand::OpenGlobalLog => Some(A::NavOpenGlobalLog),
        PaletteCommand::Jobs(crate::jobs_ui::JobsCmd::Show) => Some(A::NavShowJobs),
        PaletteCommand::Updates(crate::updates::UpdatesCommand::ShowPanel) => {
            Some(A::NavShowUpdates)
        }
        PaletteCommand::Refresh => Some(A::GlobalRefresh),
        PaletteCommand::RefreshAll => Some(A::GlobalRefreshAll),
        PaletteCommand::Quit => Some(A::GlobalQuit),
        PaletteCommand::SearchLog => Some(A::GlobalFindInLog),
        PaletteCommand::SelectTab => Some(A::GlobalSelectTab),
        PaletteCommand::WorkspaceSearch => Some(A::GlobalWorkspaceSearch),
        PaletteCommand::ChangeTheme => Some(A::NavChangeTheme),
        PaletteCommand::ChangeLanguage => Some(A::NavChangeLanguage),
        PaletteCommand::ChangeGraphColorTheme => Some(A::NavChangeGraphColorTheme),
        PaletteCommand::ToggleGraphRowTints => Some(A::NavToggleGraphRowTints),
        PaletteCommand::ToggleSolo | PaletteCommand::ToggleSoloSelectedFolder => Some(A::RefsSolo),
        PaletteCommand::StopSoloing => Some(A::RefsClearSolo),
        PaletteCommand::RevealSelectedRefInGraph => Some(A::RefsReveal),
        PaletteCommand::MarkRefAsDropSource => Some(A::RefsMarkDrop),
        PaletteCommand::DropMarkedRefHere => Some(A::RefsDropMarked),
        PaletteCommand::ToggleHierarchy => Some(A::RefsToggleHierarchy),
        PaletteCommand::RefsFold(op) => Some(op.key(crate::tree_fold::FoldCtx::Refs)),
        PaletteCommand::FilesFold(op) => Some(op.key(crate::tree_fold::FoldCtx::Status)),
        PaletteCommand::NewTab => Some(A::GlobalNewTab),
        PaletteCommand::Workspace(cmd) => match cmd {
            WorkspaceCommand::Switch => Some(A::GlobalWorkspaceSwitch),
            WorkspaceCommand::New => Some(A::HomeWsNew),
            WorkspaceCommand::FromTabs => Some(A::HomeWsFromTabs),
            WorkspaceCommand::Edit => Some(A::HomeWsEdit),
            WorkspaceCommand::FetchAll => Some(A::HomeWsFetch),
            WorkspaceCommand::PullAll => Some(A::HomeWsPull),
            WorkspaceCommand::CloneMissing => Some(A::HomeWsCloneMissing),
            WorkspaceCommand::Locate => Some(A::HomeWsLocate),
            WorkspaceCommand::Manage => Some(A::GlobalWorkspaces),
            _ => None,
        },
        PaletteCommand::Radar(crate::radar::RadarCommand::Open) => Some(A::GlobalRadar),
        PaletteCommand::Radar(crate::radar::RadarCommand::Refresh) => Some(A::GlobalRefreshAll),
        PaletteCommand::TabDir(crate::tab_dir::TabDirButton::Terminal) => Some(A::TabOpenInPopup),
        PaletteCommand::TabOpenInDefault => Some(A::TabOpenInDefault),
        PaletteCommand::AgentBack => Some(A::NavAgentBack),
        PaletteCommand::LfsMenu => Some(A::GitLfsMenu),
        PaletteCommand::Flow(crate::flow::FlowCmd::Menu) => Some(A::GitFlowMenu),
        PaletteCommand::LfsFetchAll => Some(A::GitLfsFetchAll),
        PaletteCommand::LfsCheckout => Some(A::GitLfsCheckout),
        PaletteCommand::LfsPull => Some(A::GitLfsPull),
        PaletteCommand::LfsPushAll => Some(A::GitLfsPushAll),
        PaletteCommand::LfsPrune => Some(A::GitLfsPrune),
        PaletteCommand::ActivityLevel(crate::activity_log::LevelCmd::ScrollLeft) => {
            Some(A::ActivityLogScrollLeft)
        }
        PaletteCommand::ActivityLevel(crate::activity_log::LevelCmd::ScrollRight) => {
            Some(A::ActivityLogScrollRight)
        }
        PaletteCommand::Undo => Some(A::GlobalUndo),
        PaletteCommand::Redo => Some(A::GlobalRedo),
        PaletteCommand::UndoHistory => Some(A::GlobalUndoHistory),
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Toggle) => {
            Some(A::GlobalCommitColumn)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::ScrollLeft) => {
            Some(A::CommitColumnScrollLeft)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::ScrollRight) => {
            Some(A::CommitColumnScrollRight)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::NextPart) => {
            Some(A::CommitColumnNextPart)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::MovePart(
            crate::commit_column::Side::Left,
        )) => Some(A::CommitColumnMoveLeft),
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::MovePart(
            crate::commit_column::Side::Right,
        )) => Some(A::CommitColumnMoveRight),
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::PartWider) => {
            Some(A::CommitColumnWiderPart)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::PartNarrower) => {
            Some(A::CommitColumnNarrowerPart)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::ResetWidths) => {
            Some(A::CommitColumnResetWidths)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Fit) => {
            Some(A::CommitColumnFit)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::FitAll) => {
            Some(A::CommitColumnFitAll)
        }
        PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Menu) => {
            Some(A::GlobalContextMenu)
        }
        PaletteCommand::LogColumn(cmd) => {
            use crate::commit_column::Side;
            use crate::log_table::LogColumnCmd as L;
            Some(match cmd {
                L::Pick => A::LogPickColumn,
                L::NextColumn => A::LogColumnsNext,
                L::PrevColumn => A::LogColumnsPrev,
                L::Move(Side::Left) => A::LogColumnsMoveLeft,
                L::Move(Side::Right) => A::LogColumnsMoveRight,
                L::Wider => A::LogColumnsWider,
                L::Narrower => A::LogColumnsNarrower,
                L::ResetWidth => A::LogColumnsResetWidth,
                L::FitToContent => A::LogColumnsFit,
                L::FitAllToContent => A::LogColumnsFitAll,
                L::ScrollLeft => A::LogScrollLeft,
                L::ScrollRight => A::LogScrollRight,
                L::ToggleLayout => A::LogColumnsLayout,
                L::Chooser => return None,
            })
        }
        PaletteCommand::UndoOffer => Some(A::GlobalUndoOffer),
        PaletteCommand::Merge(cmd) => merge_action(cmd),
        PaletteCommand::ShowAllHiddenRefs => Some(A::RefsShowAll),
        PaletteCommand::FilesTogglePathTree => Some(A::StatusToggleFileMode),
        PaletteCommand::BlameOp(op) => Some(op.action()),
        PaletteCommand::ImageZoomIn => Some(A::ImageZoomIn),
        PaletteCommand::ImageZoomOut => Some(A::ImageZoomOut),
        PaletteCommand::ImageFit => Some(A::ImageFit),
        PaletteCommand::ImageActualSize => Some(A::ImageActualSize),
        PaletteCommand::ImageFlipSide => Some(A::ImageFlipSide),
        PaletteCommand::ImageWidenBefore => Some(A::ImageWidenBefore),
        PaletteCommand::ImageWidenAfter => Some(A::ImageWidenAfter),
        PaletteCommand::ImageResetSplit => Some(A::ImageResetSplit),
        PaletteCommand::ImageEditIn => Some(A::ImageEditIn),
        PaletteCommand::ImageEditDefault => Some(A::ImageEditDefault),
        PaletteCommand::HistoryView(action) => Some(action.bind()),
        PaletteCommand::Rewrite(crate::rewrite_view::RewriteEntry::Workbench) => {
            Some(A::GitRewriteWorkbench)
        }
        // The binary file's "Open in" shares the picture's keys.
        PaletteCommand::OpenWith => Some(A::ImageEditIn),
        PaletteCommand::OpenWithDefault => Some(A::ImageEditDefault),
        PaletteCommand::FilesToggleViewAll => Some(A::StatusViewAllFiles),
        PaletteCommand::CommitToggleSkipHooks => Some(A::StatusSkipHooks),
        PaletteCommand::CompareSelection => Some(A::LogCompareSelection),
        PaletteCommand::CopySelectedAsMermaid => Some(A::LogCopyMermaid),
        PaletteCommand::CompareSwap => Some(A::LogCompareSwap),
        PaletteCommand::CompareGotoNext => Some(A::LogCompareNext),
        PaletteCommand::Management(ManagementCommand::ToggleSelectedFavorite) => {
            Some(A::HomeFavorite)
        }
        PaletteCommand::Management(ManagementCommand::CloseOrRemoveSelected) => {
            Some(A::HomeCloseRepo)
        }
        PaletteCommand::Management(ManagementCommand::SelectedMenu) => Some(A::GlobalContextMenu),
        PaletteCommand::Management(ManagementCommand::CopySelectedPath) => Some(A::HomeCopyPath),
        PaletteCommand::Management(ManagementCommand::OpenSelectedIn) => Some(A::HomeOpenIn),
        PaletteCommand::Management(ManagementCommand::CollapseAll) => Some(A::HomeCollapseAll),
        PaletteCommand::Management(ManagementCommand::ExpandAll) => Some(A::HomeExpandAll),
        _ => None,
    }
}

/// The registry action a merge editor command is the twin of.
fn merge_action(cmd: crate::merge_view::MergeCmd) -> Option<crate::keymap::Action> {
    use crate::keymap::Action as A;
    use crate::merge_view::MergeCmd as M;
    Some(match cmd {
        M::NextConflict => A::MergeNextConflict,
        M::PrevConflict => A::MergePrevConflict,
        M::NextFile => A::MergeNextFile,
        M::PrevFile => A::MergePrevFile,
        M::TakeA => A::MergeTakeA,
        M::TakeB => A::MergeTakeB,
        M::TakeBoth => A::MergeTakeBoth,
        M::TakeBothReversed => A::MergeTakeBothReversed,
        M::Clear => A::MergeClear,
        M::Unpick => A::MergeUnpick,
        M::Reset => A::MergeResetAll,
        M::FocusPane(crate::merge_view::MergePane::A) => A::MergeFocusA,
        M::FocusPane(crate::merge_view::MergePane::B) => A::MergeFocusB,
        M::FocusPane(crate::merge_view::MergePane::Output) => A::MergeFocusOutput,
        M::FocusPane(crate::merge_view::MergePane::Base) => A::MergeFocusBase,
        M::SwapOrder => A::MergeSwapOrder,
        M::TakeAllA => A::MergeTakeAllA,
        M::TakeAllB => A::MergeTakeAllB,
        M::ToggleBase => A::MergeToggleBase,
        M::TogglePanes => A::MergeTogglePanes,
        M::ScrollLeft => A::MergeScrollLeft,
        M::ScrollRight => A::MergeScrollRight,
        M::ToggleFullFile => A::MergeToggleFullFile,
        M::Undo => A::MergeUndo,
        M::Redo => A::MergeRedo,
        M::Save => A::MergeSave,
        M::SaveResolve => A::MergeSaveResolve,
        M::CycleHeight => A::MergeCycleHeight,
        M::WidenCurrent => A::MergeWidenCurrent,
        M::WidenIncoming => A::MergeWidenIncoming,
        M::ResetPaneSizes => A::MergeResetPanes,
        M::UseFileA => A::MergeCardA,
        M::UseFileB => A::MergeCardB,
        M::UseFileBase => A::MergeCardBase,
        M::Keep => A::MergeCardKeep,
        M::Delete => A::MergeCardDelete,
        M::ExternalTool => A::MergeCardExternal,
        M::AiSuggest => A::MergeAiSuggest,
        M::FocusNext => A::MergeNextPane,
        M::FocusPrev => A::MergePrevPane,
        _ => return None,
    })
}

impl PaletteCommand {
    /// Commands that only exist while AI is on: the palette hides them
    /// (rather than graying them out) when `ai.enabled` is false.
    pub(crate) fn is_ai(self) -> bool {
        matches!(
            self,
            PaletteCommand::AiGenerateCommitMessage
                | PaletteCommand::AiGenerateStashMessage
                | PaletteCommand::AiInsertPromptVariable
                | PaletteCommand::AiEditProfile
                | PaletteCommand::AiOpenAgent
                | PaletteCommand::AiExplainSelectedCommit
                | PaletteCommand::AiSplitSelectedCommit
                | PaletteCommand::AiExplainWip
                | PaletteCommand::AiComposeCommits
                | PaletteCommand::Merge(crate::merge_view::MergeCmd::AiSuggest)
                | PaletteCommand::Rewrite(crate::rewrite_view::RewriteEntry::ScriptAi)
                | PaletteCommand::Rewrite(crate::rewrite_view::RewriteEntry::RulesAi)
        )
    }

    /// Commands the palette leaves out while AI is off: the `is_ai` ones and
    /// the review's ✦ rows, whose labels carry their own sparkle after the
    /// "Review: " prefix.
    pub(crate) fn needs_ai(self) -> bool {
        self.is_ai() || matches!(self, PaletteCommand::Review(cmd) if cmd.is_ai())
    }
}

mod availability;
mod bindings;
mod catalog;
mod flow;
mod jobs;
mod matching;
mod settings;

pub(crate) use availability::{unavailable_reason, PaletteContext};
#[cfg(test)]
pub(crate) use bindings::command_id;
pub(crate) use bindings::{palette_action_defs, palette_entry};
pub(crate) use catalog::PALETTE_COMMANDS;
#[cfg(test)]
pub(crate) use flow::entries as flow_entries;
#[cfg(test)]
pub(crate) use jobs::entries as jobs_entries;
pub(crate) use matching::{palette_matches, score};
pub(crate) use settings::{
    listed_count as settings_listed_count, run as run_setting, sub_query as setting_sub_query,
    Outcome as SettingOutcome,
};

#[cfg(test)]
mod tests;
