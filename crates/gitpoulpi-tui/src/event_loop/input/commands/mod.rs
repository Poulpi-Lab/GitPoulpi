//! The one command layer: every palette command, leader key and toolbar
//! button runs through [`run_command`] on a [`CommandCtx`], so the surfaces
//! cannot drift and an agent's "show" intent is the same call as a click.
//!
//! The commands are grouped by theme, one file each. A palette command must
//! be listed under exactly one of them in `run_command`'s exhaustive match;
//! one whose category has no arm for it reports itself.

mod app;
mod blame;
pub(in crate::event_loop) mod commit_column;
mod conflicts;
mod ctx;
mod diff;
mod folds;
mod git;
mod guides;
pub(in crate::event_loop) mod history;
mod hscroll;
pub(in crate::event_loop) mod log_columns;
mod nav;
mod pane;
mod refs;
mod shared;
mod sidebar_sections;
pub(in crate::event_loop) mod worktrees;

use crate::event_loop::*;
pub(in crate::event_loop) use blame::{run as run_blame_op, save_width as save_blame_width};
pub(in crate::event_loop) use conflicts::{apply_merge_outcome, open_path, open_selected};
pub(in crate::event_loop) use ctx::CommandCtx;
pub(in crate::event_loop) use folds::{
    apply_ref_fold_action, fold_files, fold_key, is_ref_fold_action,
};
use gitpoulpi_i18n::msg;
pub(in crate::event_loop) use guides::{
    block_on_conflicts, guard_off_branch, guide_key, guide_mouse, open_limit_setting, start_ai,
};
pub(in crate::event_loop) use hscroll::{hscroll_key, scroll_focused_pane};
pub(in crate::event_loop) use nav::clear_author_filter;
pub(in crate::event_loop) use pane::*;
pub(in crate::event_loop) use shared::*;
#[allow(unused_imports)]
pub(in crate::event_loop) use sidebar_sections::{
    apply as apply_sidebar_sections_action, open_menu as open_sidebar_sections_menu, show_worktrees,
};

/// What the loop does after a command ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::event_loop) enum Flow {
    /// Nothing consumed the event yet (or: keep the window running).
    Continue,
    /// The window should close (a quit, or a switch of repository).
    Quit,
    /// A key handler consumed the key; later handlers must not see it.
    Handled,
}

/// "Close Settings first", where Settings shows it (its bottom border).
pub(in crate::event_loop) fn refuse_over_settings(cx: &mut CommandCtx<'_>) {
    let text = msg::palette::WHY_CLOSE_SETTINGS_FIRST.get();
    if let Some(prefs) = cx.preferences.as_mut() {
        prefs.set_notice(text);
    }
    *cx.last_message = Some(text.into());
}

pub(in crate::event_loop) fn run_command(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> Flow {
    // Settings covers the window: a command that acts on what it hides is
    // refused (the palette shows it grayed out with the same reason).
    if cx.preferences.is_some() && !cmd.stays_in_settings() {
        refuse_over_settings(cx);
        return Flow::Handled;
    }
    // Resolved once here rather than per-arm -- several ref commands need
    // it and the walk isn't free.
    let sel_ref = selected_ref_leaf(
        &*cx.refs_result,
        &*cx.refs_filter,
        *cx.active_ref_section,
        cx.sidebar_states
            .state_mut(*cx.active_ref_section)
            .selected(),
        *cx.hierarchical_refs,
        &*cx.collapsed_ref_folders,
        &*cx.remote_names,
    );
    // One exhaustive match: a new `PaletteCommand` variant does not compile
    // until it is routed to the category file that handles it.
    let handled = match cmd {
        PaletteCommand::FocusRefs
        | PaletteCommand::FocusLog
        | PaletteCommand::FocusStatus
        | PaletteCommand::FocusRefsFilter
        | PaletteCommand::FocusRefsSection(..)
        | PaletteCommand::FocusStatusSection(..)
        | PaletteCommand::SearchLog
        | PaletteCommand::FilterLogByAuthor
        | PaletteCommand::ClearAuthorFilter
        | PaletteCommand::ToggleRefsPaneCollapsed
        | PaletteCommand::GrowCommitMessage
        | PaletteCommand::ShrinkCommitMessage
        | PaletteCommand::ResetCommitMessageSize
        | PaletteCommand::ScrollCommitMessageDown
        | PaletteCommand::ScrollCommitMessageUp
        | PaletteCommand::OpenMessageLink
        | PaletteCommand::ToggleCommitFileSort
        | PaletteCommand::GrowRefsSection
        | PaletteCommand::ShrinkRefsSection
        | PaletteCommand::ResetRefsSectionSizes
        | PaletteCommand::ResetLayout
        | PaletteCommand::ToggleStatusPaneCollapsed
        | PaletteCommand::ResizePane(_)
        | PaletteCommand::CycleFocus
        | PaletteCommand::CycleFocusBack => nav::run(cx, cmd),
        PaletteCommand::CheckoutSelectedRef
        | PaletteCommand::CopyRefField(..)
        | PaletteCommand::DeleteSelectedRemoteBranch
        | PaletteCommand::DeleteSelectedBranchAndRemote
        | PaletteCommand::CopySelectedBranchLink
        | PaletteCommand::FastForwardSelectedToCurrent
        | PaletteCommand::FastForwardCurrentToSelected
        | PaletteCommand::CreateWorktreeFromSelectedRef
        | PaletteCommand::StartPullRequestFromSelectedBranch
        | PaletteCommand::ReflogSelectedBranch
        | PaletteCommand::SubmoduleOpenSelected
        | PaletteCommand::SubmoduleUpdateSelected
        | PaletteCommand::SubmoduleInitSelected
        | PaletteCommand::SubmoduleUpdateAll
        | PaletteCommand::OpenWorktreeInTab
        | PaletteCommand::DropSelectedRefOnCurrent
        | PaletteCommand::MarkRefAsDropSource
        | PaletteCommand::DropMarkedRefHere
        | PaletteCommand::MergeSelectedRef
        | PaletteCommand::RebaseOntoSelectedRef
        | PaletteCommand::RenameSelectedBranch
        | PaletteCommand::DeleteSelectedRef
        | PaletteCommand::PushSelectedTag
        | PaletteCommand::DeleteSelectedTagFromRemote
        | PaletteCommand::FetchSelectedTag
        | PaletteCommand::TakeRemoteSelectedTag
        | PaletteCommand::ForcePushSelectedTag
        | PaletteCommand::AnnotateSelectedTag
        | PaletteCommand::MoveSelectedTagToSelectedCommit
        | PaletteCommand::PushAllUnpushedTags
        | PaletteCommand::ToggleHideInGraph
        | PaletteCommand::RevealSelectedRefInGraph
        | PaletteCommand::ToggleSolo
        | PaletteCommand::StopSoloing
        | PaletteCommand::ShowAllHiddenRefs
        | PaletteCommand::ToggleHideSelectedFolder
        | PaletteCommand::ToggleSoloSelectedFolder
        | PaletteCommand::FilterRefs
        | PaletteCommand::ClearRefsFilter
        | PaletteCommand::NextRefSection
        | PaletteCommand::PrevRefSection
        | PaletteCommand::ToggleSectionCollapse
        | PaletteCommand::ToggleHierarchy => refs::run(cx, cmd, &sel_ref),
        PaletteCommand::RefsFold(..) | PaletteCommand::FilesFold(..) => folds::run(cx, cmd),
        PaletteCommand::PaneScroll(dir) => {
            scroll_focused_pane(cx, dir);
            true
        }
        PaletteCommand::WorktreeOpenInSelected
        | PaletteCommand::WorktreeMergeSelected
        | PaletteCommand::WorktreeRemoveSelected
        | PaletteCommand::CreateWorktreeMenu
        | PaletteCommand::CreateWorktreeFromCurrentBranch
        | PaletteCommand::CreateWorktreeChooseBranch => worktrees::run(cx, cmd),
        PaletteCommand::ShowWorktreesSection
        | PaletteCommand::SidebarSectionsMenu
        | PaletteCommand::ToggleSidebarSection(..)
        | PaletteCommand::ResetSidebarSections
        | PaletteCommand::UseSidebarSectionsForAll => sidebar_sections::run(cx, cmd),
        PaletteCommand::FileMenu(..)
        | PaletteCommand::FilesOpenMenu
        | PaletteCommand::FilesSelectAll
        | PaletteCommand::FilesClearSelection
        | PaletteCommand::FilesToggleSelection
        | PaletteCommand::CreateFile
        | PaletteCommand::OpenFolderMenu
        | PaletteCommand::StageSelectedFile
        | PaletteCommand::UnstageSelectedFile
        | PaletteCommand::StageAll
        | PaletteCommand::DiscardAllChanges
        | PaletteCommand::Commit
        | PaletteCommand::CommitNow
        | PaletteCommand::EditWipMessage
        | PaletteCommand::Fetch
        | PaletteCommand::Pull
        | PaletteCommand::PullWithMode(..)
        | PaletteCommand::PullAllBranches
        | PaletteCommand::PullAllBranchesWithMode(..)
        | PaletteCommand::ChoosePullButtonAction
        | PaletteCommand::LfsMenu
        | PaletteCommand::LfsFetchAll
        | PaletteCommand::LfsCheckout
        | PaletteCommand::LfsPushAll
        | PaletteCommand::Push
        | PaletteCommand::PushToRemote
        | PaletteCommand::ForcePushWithLease
        | PaletteCommand::PushToSelectedRemotes
        | PaletteCommand::PushToAllTargets
        | PaletteCommand::PullFromAllSources
        | PaletteCommand::PullFromOneRemote
        | PaletteCommand::ShowGroupedPush
        | PaletteCommand::ContinueGroupedPush
        | PaletteCommand::AbortGroupedPush
        | PaletteCommand::RetryFailedPushes
        | PaletteCommand::SetUpstream
        | PaletteCommand::AddRemote
        | PaletteCommand::RemoveRemote
        | PaletteCommand::EditRemote
        | PaletteCommand::SetDefaultRemote
        | PaletteCommand::CopyRemoteName
        | PaletteCommand::CopyRemoteLink
        | PaletteCommand::FetchRemote
        | PaletteCommand::RemoteActions
        | PaletteCommand::SparseCheckout
        | PaletteCommand::HideRemoteInGraph
        | PaletteCommand::ShowRemoteInGraph
        | PaletteCommand::NewBranch
        | PaletteCommand::NewTag
        | PaletteCommand::StashPush
        | PaletteCommand::StashPop(..)
        | PaletteCommand::StashApply
        | PaletteCommand::StashDrop
        | PaletteCommand::CopySelectedText
        | PaletteCommand::CopyLine
        | PaletteCommand::CopyHunk
        | PaletteCommand::EditInWorkingDirectory
        | PaletteCommand::EditorMenu(..)
        | PaletteCommand::Blame
        | PaletteCommand::History
        | PaletteCommand::FileView
        | PaletteCommand::CherryPickSelectedCommit
        | PaletteCommand::RevertSelectedCommit
        | PaletteCommand::EditSelectedCommitMessage
        | PaletteCommand::AmendSelectedCommit(_)
        | PaletteCommand::SplitSelectedCommit
        | PaletteCommand::InteractiveRebaseFromSelectedCommit
        | PaletteCommand::ResetToSelectedCommit(..)
        | PaletteCommand::CreateBranchAtSelectedCommit
        | PaletteCommand::CreateTagAtSelectedCommit
        | PaletteCommand::CheckoutSelectedCommit
        | PaletteCommand::CreateWorktreeAtSelectedCommit
        | PaletteCommand::CreatePatchAtSelectedCommit
        | PaletteCommand::CreatePatchFromSelectedCommitToHead
        | PaletteCommand::CreateAnnotatedTagAtSelectedCommit
        | PaletteCommand::CopySelectedCommitLink
        | PaletteCommand::CopySelectedAsMermaid
        | PaletteCommand::CopyCommitField(..)
        | PaletteCommand::RebaseContinue
        | PaletteCommand::RebaseSkip
        | PaletteCommand::RebaseAbort
        | PaletteCommand::AmContinue
        | PaletteCommand::AmSkip
        | PaletteCommand::AmAbort
        | PaletteCommand::IndexLockRemove
        | PaletteCommand::AutoStashPop
        | PaletteCommand::AutoStashFinish
        | PaletteCommand::AutoStashShow
        | PaletteCommand::ConflictsShow
        | PaletteCommand::AutoStashDrop
        | PaletteCommand::AutoStashStartOver
        | PaletteCommand::CreatePatchFromWip
        | PaletteCommand::ApplyPatch
        | PaletteCommand::LfsInitialize
        | PaletteCommand::LfsTrackPattern
        | PaletteCommand::LfsUntrackPattern
        | PaletteCommand::LfsPrune
        | PaletteCommand::LfsFetch
        | PaletteCommand::LfsPull
        | PaletteCommand::LfsPush
        | PaletteCommand::LfsShowLocks
        | PaletteCommand::LfsOpenInstallPage
        | PaletteCommand::LfsToggleRawPointer
        | PaletteCommand::LfsTrackSelectedFile
        | PaletteCommand::LfsTrackSelectedExtension
        | PaletteCommand::LfsLockSelectedFile
        | PaletteCommand::LfsUnlockSelectedFile
        | PaletteCommand::LfsForceUnlockSelectedFile
        | PaletteCommand::ResolveConflictWith(..) => git::run(cx, cmd, &sel_ref),
        PaletteCommand::OpenMergeEditor
        | PaletteCommand::Merge(..)
        | PaletteCommand::AbortOperation
        | PaletteCommand::SkipOperation => conflicts::run(cx, cmd),
        PaletteCommand::Setting(..)
        | PaletteCommand::Sequence(..)
        | PaletteCommand::OpenRepository
        | PaletteCommand::BrowseRepository
        | PaletteCommand::CreateRepository
        | PaletteCommand::CloneRepository
        | PaletteCommand::Management(..)
        | PaletteCommand::Workspace(..)
        | PaletteCommand::Radar(..)
        | PaletteCommand::Updates(..)
        | PaletteCommand::OpenTabMenu
        | PaletteCommand::OpenNewMenu
        | PaletteCommand::NewTab
        | PaletteCommand::TabMenu(..)
        | PaletteCommand::TabDir(..)
        | PaletteCommand::TabOpenIn
        | PaletteCommand::TabOpenAgentHere
        | PaletteCommand::TabOpenInDefault
        | PaletteCommand::ChangeProfile
        | PaletteCommand::AboutGitPoulpi
        | PaletteCommand::AboutLicenses
        | PaletteCommand::TrustReview
        | PaletteCommand::OpenActivityLog
        | PaletteCommand::OpenGlobalLog
        | PaletteCommand::Jobs(_)
        | PaletteCommand::CopyDiagnostics
        | PaletteCommand::ActivityLevel(..)
        | PaletteCommand::CopyLastError
        | PaletteCommand::RunLastErrorRemedy
        | PaletteCommand::OpenLastError
        | PaletteCommand::CycleRepositoryTab(..)
        | PaletteCommand::MoveRepositoryTab(..)
        | PaletteCommand::CloseRepositoryTab
        | PaletteCommand::SelectTab
        | PaletteCommand::WorkspaceSearch
        | PaletteCommand::Undo
        | PaletteCommand::UndoForget
        | PaletteCommand::UndoHistory
        | PaletteCommand::ReflogHead
        | PaletteCommand::LostCommits
        | PaletteCommand::UndoOffer
        | PaletteCommand::Safety(_)
        | PaletteCommand::Review(_)
        | PaletteCommand::Board(_)
        | PaletteCommand::Item(_)
        | PaletteCommand::Composer(_)
        | PaletteCommand::Flow(_)
        | PaletteCommand::Redo
        | PaletteCommand::Refresh
        | PaletteCommand::RefreshAll
        | PaletteCommand::ToggleHelp
        | PaletteCommand::OpenPreferences
        | PaletteCommand::SequenceCreateAi
        | PaletteCommand::SearchSettings
        | PaletteCommand::Keybindings
        | PaletteCommand::KeybindingsResetAll
        | PaletteCommand::RepoAuthOpen
        | PaletteCommand::RepoAuthTest
        | PaletteCommand::RepoAuthDefaults
        | PaletteCommand::Theme
        | PaletteCommand::ConflictLists
        | PaletteCommand::ConflictRepoPage
        | PaletteCommand::ChangeTheme
        | PaletteCommand::ChangeLanguage
        | PaletteCommand::LauncherCreate
        | PaletteCommand::LauncherCreateDesktop
        | PaletteCommand::LauncherRemove
        | PaletteCommand::LauncherAliasCreate
        | PaletteCommand::LauncherAliasRemove
        | PaletteCommand::McpSuggestPort
        | PaletteCommand::LauncherPathAdd
        | PaletteCommand::LauncherPathRemove
        | PaletteCommand::LauncherFolderMenuInstall
        | PaletteCommand::LauncherFolderMenuRemove
        | PaletteCommand::LauncherBecomePrimary
        | PaletteCommand::LauncherRemoveLegacy
        | PaletteCommand::LauncherForgetGone
        | PaletteCommand::ChangeGraphColorTheme
        | PaletteCommand::ToggleGraphRowTints
        | PaletteCommand::AgentShowPending
        | PaletteCommand::AgentDismissPending
        | PaletteCommand::AgentBack
        | PaletteCommand::SequenceCancel
        | PaletteCommand::SequenceContinue
        | PaletteCommand::AiExplainSelectedCommit
        | PaletteCommand::AiSplitSelectedCommit
        | PaletteCommand::AiExplainWip
        | PaletteCommand::ComposerMoveFile
        | PaletteCommand::AiComposeCommits
        | PaletteCommand::StageGuardShowInFile
        | PaletteCommand::StashPushWithMessage
        | PaletteCommand::AiGenerateStashMessage
        | PaletteCommand::AiInsertPromptVariable
        | PaletteCommand::AiEditProfile
        | PaletteCommand::AiOpenAgent
        | PaletteCommand::AiGenerateCommitMessage
        | PaletteCommand::Hosting(..)
        | PaletteCommand::ToggleColumn(..)
        | PaletteCommand::LogColumn(..)
        | PaletteCommand::Quit => app::run(cx, cmd),
        PaletteCommand::BlameOp(..) => blame::run_palette(cx, cmd),
        PaletteCommand::Rewrite(entry) => {
            super::keys_prefix::open_rewrite(cx, entry);
            true
        }
        PaletteCommand::OpenDiff
        | PaletteCommand::CloseDiff
        | PaletteCommand::NextHunk
        | PaletteCommand::PrevHunk
        | PaletteCommand::DiffMode(..)
        | PaletteCommand::FilesTogglePathTree
        | PaletteCommand::FilesToggleViewAll
        | PaletteCommand::CommitToggleSkipHooks
        | PaletteCommand::CompareSelection
        | PaletteCommand::CompareSwap
        | PaletteCommand::CompareGotoNext
        | PaletteCommand::ToggleDiffHunksOnly
        | PaletteCommand::ToggleWordWrap
        | PaletteCommand::DiffChooseEncoding
        | PaletteCommand::ToggleMinimap
        | PaletteCommand::ToggleDiffIntraline
        | PaletteCommand::EditorToggleFollowSelection
        | PaletteCommand::ImageZoomIn
        | PaletteCommand::ImageZoomOut
        | PaletteCommand::ImageFit
        | PaletteCommand::ImageActualSize
        | PaletteCommand::ImageFlipSide
        | PaletteCommand::ImageWidenBefore
        | PaletteCommand::ImageWidenAfter
        | PaletteCommand::ImageResetSplit
        | PaletteCommand::ImageEditIn
        | PaletteCommand::ImageEditDefault
        | PaletteCommand::HistoryView(_)
        | PaletteCommand::CommitColumn(_)
        | PaletteCommand::OpenWith
        | PaletteCommand::OpenWithDefault
        | PaletteCommand::ToggleWhitespace
        | PaletteCommand::StageHunk
        | PaletteCommand::UnstageHunk
        | PaletteCommand::StageLines
        | PaletteCommand::UnstageLines
        | PaletteCommand::SelectLines
        | PaletteCommand::DiscardHunk
        | PaletteCommand::DiscardLines => diff::run(cx, cmd),
    };
    if !handled {
        debug_assert!(false, "palette command {cmd:?} has no handler");
        *cx.last_message = Some(msg::input::not_wired_up(format!("{cmd:?}")));
    }
    if *cx.should_quit {
        Flow::Quit
    } else {
        Flow::Continue
    }
}
