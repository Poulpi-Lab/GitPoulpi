use super::*;
#[allow(unused_imports)]
use crate::safety_panel::SafetyCmdExt;

/// Every entry in the catalogue is reachable and unique.
#[test]
fn the_catalogue_has_no_duplicate_commands_or_labels() {
    let mut commands: Vec<PaletteCommand> = PALETTE_COMMANDS.iter().map(|e| e.command).collect();
    let before = commands.len();
    commands.dedup_by(|a, b| a == b);
    // dedup only removes *consecutive* duplicates, so compare against a
    // full pairwise check instead.
    assert_eq!(before, commands.len(), "consecutive duplicate command");
    for (i, a) in PALETTE_COMMANDS.iter().enumerate() {
        for b in PALETTE_COMMANDS.iter().skip(i + 1) {
            assert_ne!(a.command, b.command, "duplicate command: {}", a.label.get());
            assert_ne!(
                a.label.get(),
                b.label.get(),
                "duplicate label: {}",
                a.label.get()
            );
        }
    }
}

/// Labels carry their `Category: ` prefix - the ranking in `score`
/// depends on it, and so does the popup's readability.
#[test]
fn every_label_is_categorised() {
    for e in PALETTE_COMMANDS {
        assert!(
            e.label.get().contains(": "),
            "label without a category prefix: {}",
            e.label.get()
        );
    }
}

/// The real deliverable of this module: these three `match`es are
/// exhaustive, so adding a variant to any of the three enums that
/// define the app's mouse surface stops the *test build* compiling
/// until that new thing is also given a palette command. That's a
/// stronger guarantee than a runtime assertion - it can't be
/// forgotten, and it fires at the moment the feature is written.
#[test]
fn every_mouse_affordance_maps_to_a_palette_command() {
    use crate::diff_view::{DiffBarButton, DiffStageAction};
    use crate::status_pane::FileContextAction;
    use crate::{CommitField, LogContextAction, RefContextAction, RefField, ToolbarButton};

    fn for_ref_menu(a: RefContextAction) -> PaletteCommand {
        match a {
            RefContextAction::ToggleHideInGraph => PaletteCommand::ToggleHideInGraph,
            RefContextAction::ToggleSolo => PaletteCommand::ToggleSolo,
            RefContextAction::RevealInGraph => PaletteCommand::RevealSelectedRefInGraph,
            RefContextAction::ToggleHideFolder => PaletteCommand::ToggleHideSelectedFolder,
            RefContextAction::ToggleSoloFolder => PaletteCommand::ToggleSoloSelectedFolder,
            RefContextAction::MergeIntoCurrent => PaletteCommand::MergeSelectedRef,
            RefContextAction::RebaseOntoThis => PaletteCommand::RebaseOntoSelectedRef,
            RefContextAction::RenameBranch => PaletteCommand::RenameSelectedBranch,
            RefContextAction::DeleteBranch | RefContextAction::DeleteTag => {
                PaletteCommand::DeleteSelectedRef
            }
            RefContextAction::DeleteRemoteBranch => PaletteCommand::DeleteSelectedRemoteBranch,
            RefContextAction::DeleteBranchAndRemote => {
                PaletteCommand::DeleteSelectedBranchAndRemote
            }
            RefContextAction::CopyBranchLink => PaletteCommand::CopySelectedBranchLink,
            RefContextAction::PushTag => PaletteCommand::PushSelectedTag,
            RefContextAction::FetchTag => PaletteCommand::FetchSelectedTag,
            RefContextAction::TagTakeRemote(_) => PaletteCommand::TakeRemoteSelectedTag,
            RefContextAction::TagForcePush(_) => PaletteCommand::ForcePushSelectedTag,
            RefContextAction::DeleteTagFromRemote => PaletteCommand::DeleteSelectedTagFromRemote,
            RefContextAction::AnnotateTag => PaletteCommand::AnnotateSelectedTag,
            RefContextAction::PushAllUnpushedTags => PaletteCommand::PushAllUnpushedTags,
            RefContextAction::StashPop => PaletteCommand::StashPop(StashTarget::Selected),
            RefContextAction::SubmoduleUpdate => PaletteCommand::SubmoduleUpdateSelected,
            RefContextAction::SubmoduleInit => PaletteCommand::SubmoduleInitSelected,
            RefContextAction::SubmoduleOpen => PaletteCommand::SubmoduleOpenSelected,
            RefContextAction::StashApply => PaletteCommand::StashApply,
            RefContextAction::StashDrop => PaletteCommand::StashDrop,
            RefContextAction::Checkout => PaletteCommand::CheckoutSelectedRef,
            RefContextAction::Copy(f) => PaletteCommand::CopyRefField(f),
            RefContextAction::SetUpstream => PaletteCommand::SetUpstream,
            RefContextAction::PushBranch => PaletteCommand::Push,
            RefContextAction::PushToTarget(_) => PaletteCommand::PushToRemote,
            RefContextAction::PullFromTarget(_) => PaletteCommand::PullFromOneRemote,
            RefContextAction::PushToSelected => PaletteCommand::PushToSelectedRemotes,
            RefContextAction::PullBranch => PaletteCommand::Pull,
            RefContextAction::FetchBranchRemote => PaletteCommand::FetchRemote,
            RefContextAction::FastForwardToCurrent => PaletteCommand::FastForwardSelectedToCurrent,
            RefContextAction::FastForwardCurrentToThis => {
                PaletteCommand::FastForwardCurrentToSelected
            }
            RefContextAction::CreateWorktree => PaletteCommand::CreateWorktreeFromSelectedRef,
            RefContextAction::WorktreeOpenTab => PaletteCommand::OpenWorktreeInTab,
            RefContextAction::WorktreeOpenWith(_) => PaletteCommand::WorktreeOpenInSelected,
            RefContextAction::WorktreeMerge => PaletteCommand::WorktreeMergeSelected,
            RefContextAction::WorktreeRemove => PaletteCommand::WorktreeRemoveSelected,
            RefContextAction::WorktreeNewHead => PaletteCommand::CreateWorktreeFromCurrentBranch,
            RefContextAction::WorktreeNewCommit => PaletteCommand::CreateWorktreeAtSelectedCommit,
            RefContextAction::WorktreeNewChoose => PaletteCommand::CreateWorktreeChooseBranch,
            RefContextAction::Reflog => PaletteCommand::ReflogSelectedBranch,
            RefContextAction::Drop(_) => PaletteCommand::DropSelectedRefOnCurrent,
            RefContextAction::ToggleSidebarSection(s) => PaletteCommand::ToggleSidebarSection(s),
            RefContextAction::ResetSidebarSections => PaletteCommand::ResetSidebarSections,
            RefContextAction::Fold(op) => PaletteCommand::RefsFold(op),
            RefContextAction::OpenSectionsChecklist => PaletteCommand::SidebarSectionsMenu,
            RefContextAction::UseSidebarSectionsForAll => PaletteCommand::UseSidebarSectionsForAll,
            RefContextAction::StartPullRequest => {
                PaletteCommand::StartPullRequestFromSelectedBranch
            }
        }
    }
    fn for_diff_bar(b: DiffBarButton) -> PaletteCommand {
        match b {
            DiffBarButton::FileView => PaletteCommand::FileView,
            DiffBarButton::DiffView => PaletteCommand::OpenDiff,
            DiffBarButton::Blame => PaletteCommand::Blame,
            // The column's selector: the segments pick a list (palette:
            // `View: Commit Column: ...`; the selected one hides), the item itself
            // toggles the column.
            DiffBarButton::History => {
                PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Toggle)
            }
            DiffBarButton::ColumnFile => PaletteCommand::CommitColumn(
                crate::commit_column::ColumnCmd::Set(crate::commit_column::ColumnState::File),
            ),
            DiffBarButton::ColumnAll => PaletteCommand::CommitColumn(
                crate::commit_column::ColumnCmd::Set(crate::commit_column::ColumnState::Graph),
            ),
            DiffBarButton::PrevHunk => PaletteCommand::PrevHunk,
            DiffBarButton::NextHunk => PaletteCommand::NextHunk,
            DiffBarButton::Mode(m) => PaletteCommand::DiffMode(m),
            DiffBarButton::ToggleHunksOnly => PaletteCommand::ToggleDiffHunksOnly,
            DiffBarButton::ToggleWhitespace => PaletteCommand::ToggleWhitespace,
            DiffBarButton::ToggleWordWrap => PaletteCommand::ToggleWordWrap,
            DiffBarButton::ToggleMinimap => PaletteCommand::ToggleMinimap,
            DiffBarButton::EditWorkingFile => PaletteCommand::EditInWorkingDirectory,
            DiffBarButton::Encoding => PaletteCommand::DiffChooseEncoding,
        }
    }

    // The toolbar's Pop is the catalogue's `Stash Pop` entry with a
    // different `StashTarget`.
    let in_catalogue = |c: PaletteCommand| {
        PALETTE_COMMANDS.iter().any(|e| {
            e.command == c
                || matches!(
                    (e.command, c),
                    (PaletteCommand::StashPop(_), PaletteCommand::StashPop(_))
                )
        })
    };

    for b in [
        ToolbarButton::Undo,
        ToolbarButton::Redo,
        ToolbarButton::Pull,
        ToolbarButton::Push,
        ToolbarButton::Branch,
        ToolbarButton::Tag,
        ToolbarButton::Stash,
        ToolbarButton::Pop,
        ToolbarButton::Search,
    ] {
        assert!(
            in_catalogue(ToolbarButton::command(b)),
            "toolbar {b:?} unreachable"
        );
    }
    for a in [
        RefContextAction::ToggleHideInGraph,
        RefContextAction::ToggleSolo,
        RefContextAction::ToggleHideFolder,
        RefContextAction::ToggleSoloFolder,
        RefContextAction::MergeIntoCurrent,
        RefContextAction::RebaseOntoThis,
        RefContextAction::RenameBranch,
        RefContextAction::DeleteBranch,
        RefContextAction::DeleteRemoteBranch,
        RefContextAction::DeleteBranchAndRemote,
        RefContextAction::CopyBranchLink,
        RefContextAction::DeleteTag,
        RefContextAction::PushTag,
        RefContextAction::DeleteTagFromRemote,
        RefContextAction::TagTakeRemote(0),
        RefContextAction::TagForcePush(0),
        RefContextAction::AnnotateTag,
        RefContextAction::StashPop,
        RefContextAction::StashApply,
        RefContextAction::StashDrop,
        RefContextAction::Checkout,
        RefContextAction::Copy(RefField::Name),
        RefContextAction::Copy(RefField::FullRef),
        RefContextAction::Copy(RefField::TargetSha),
        RefContextAction::SetUpstream,
        RefContextAction::PushBranch,
        RefContextAction::PushToTarget(0),
        RefContextAction::PullFromTarget(0),
        RefContextAction::PushToSelected,
        RefContextAction::PullBranch,
        RefContextAction::FetchBranchRemote,
        RefContextAction::FastForwardToCurrent,
        RefContextAction::FastForwardCurrentToThis,
        RefContextAction::CreateWorktree,
        RefContextAction::StartPullRequest,
    ] {
        assert!(in_catalogue(for_ref_menu(a)), "ref menu {a:?} unreachable");
    }

    // The remote menus (a remote's own menu, the toolbar dropdowns,
    // Set upstream, the diverged-push dialog). Exhaustive for the
    // same reason as the ones above. The diverged dialog's answers
    // map to the pull and force-push commands the palette runs from
    // any state; Cancel is the menu's own Esc.
    fn for_remote_ui(a: &crate::RemoteUiAction) -> Option<PaletteCommand> {
        use crate::RemoteUiAction as R;
        Some(match a {
            R::CopyRemoteName(_) => PaletteCommand::CopyRemoteName,
            R::CopyRemoteLink(_) => PaletteCommand::CopyRemoteLink,
            R::SetDefaultRemote(_) => PaletteCommand::SetDefaultRemote,
            R::EditRemote(_) => PaletteCommand::EditRemote,
            R::RemoveRemote(_) => PaletteCommand::RemoveRemote,
            // A leftover namespace (branches, no remote): the palette's
            // Remove / Edit choosers list it, Remove deletes its branches
            // and Edit opens the Add form with its name filled in.
            R::RemoveLeftover(_) => PaletteCommand::RemoveRemote,
            R::AddAsRemote(_) => PaletteCommand::EditRemote,
            R::FetchRemote(_) => PaletteCommand::FetchRemote,
            R::AddRemote => PaletteCommand::AddRemote,
            R::FetchAllRemotes => PaletteCommand::Fetch,
            R::RunPull(mode) => PaletteCommand::PullWithMode(*mode),
            R::RunPullAllBranches(None) => PaletteCommand::PullAllBranches,
            R::RunPullAllBranches(Some(mode)) => PaletteCommand::PullAllBranchesWithMode(*mode),
            R::PushTo(_) | R::PushOne { .. } => PaletteCommand::PushToRemote,
            R::PushAll { .. } => PaletteCommand::PushToAllTargets,
            R::PushSelected { .. } => PaletteCommand::PushToSelectedRemotes,
            R::RemotesOf { .. } => PaletteCommand::SetUpstream,
            R::PullAll { .. } => PaletteCommand::PullFromAllSources,
            R::PullOne { .. } => PaletteCommand::PullFromOneRemote,
            R::Lfs(item) => item.command(),
            // Checked against the generated `Flow: Start <Type>...` rows by
            // `flow::tests::every_menu_row_has_a_palette_twin`.
            R::Flow(_) => return None,
            R::PullThenPush { .. } => PaletteCommand::PullWithMode(PullButtonMode::Merge),
            R::ForcePush { .. } => PaletteCommand::ForcePushWithLease,
            R::HideRemote(_) => PaletteCommand::HideRemoteInGraph,
            R::ShowRemote(_) => PaletteCommand::ShowRemoteInGraph,
            R::OpenRepoAuth => PaletteCommand::RepoAuthOpen,
            // The auto-stash dialog is the answer to an operation the palette
            // already reaches (checkout, pull, merge, rebase); its yes is
            // Enter on the dialog, and `general.auto_stash = always` skips it.
            R::AutoStashContinue | R::AutoStashAlways | R::Cancel => return None,
        })
    }
    let rejection = gitpoulpi_core::domain::PushRejection {
        remote: "o".into(),
        branch: "b".into(),
        reason: gitpoulpi_core::domain::PushRejectReason::NonFastForward,
        detail: None,
    };
    let every_remote_row = crate::remote_menu_entries("o", false, false)
        .into_iter()
        .chain(crate::remote_menu_entries("o", false, true))
        .chain(crate::remotes_ui::leftover_menu_entries("o", false))
        .chain(crate::remotes_ui::leftover_menu_entries("o", true))
        .chain(crate::pull_mode_entries())
        .chain(crate::remotes_ui::lfs_menu_entries())
        .chain(crate::push_to_entries(&["o".to_string()], None))
        .chain(crate::diverged_entries(&rejection));
    for e in every_remote_row {
        if let crate::context_menu::Entry::Action { action, .. } = e {
            if let Some(c) = for_remote_ui(&action) {
                assert!(in_catalogue(c), "remote menu {action:?} unreachable");
            }
        }
    }
    assert!(in_catalogue(
        for_remote_ui(&crate::RemoteUiAction::AddRemote).unwrap()
    ));
    // The Diff view's staging affordances aren't bar buttons, so the
    // exhaustive `match` that keeps them reachable is over their own
    // enum. Adding a variant to it stops this test compiling until
    // the new affordance has a palette command too.
    fn for_diff_stage(a: DiffStageAction, staging: bool) -> PaletteCommand {
        match (a, staging) {
            (DiffStageAction::Hunk, true) => PaletteCommand::StageHunk,
            (DiffStageAction::Hunk, false) => PaletteCommand::UnstageHunk,
            (DiffStageAction::Lines, true) => PaletteCommand::StageLines,
            (DiffStageAction::Lines, false) => PaletteCommand::UnstageLines,
            (DiffStageAction::SelectLines, _) => PaletteCommand::SelectLines,
            (DiffStageAction::DiscardHunk, _) => PaletteCommand::DiscardHunk,
            (DiffStageAction::DiscardLines, _) => PaletteCommand::DiscardLines,
        }
    }

    for b in [
        DiffBarButton::FileView,
        DiffBarButton::DiffView,
        DiffBarButton::Blame,
        DiffBarButton::History,
        DiffBarButton::ColumnFile,
        DiffBarButton::ColumnAll,
        DiffBarButton::PrevHunk,
        DiffBarButton::NextHunk,
        DiffBarButton::Mode(DiffViewMode::Inline),
        DiffBarButton::Mode(DiffViewMode::Split),
        DiffBarButton::ToggleHunksOnly,
        DiffBarButton::ToggleWhitespace,
        DiffBarButton::ToggleWordWrap,
        DiffBarButton::ToggleMinimap,
        DiffBarButton::EditWorkingFile,
        DiffBarButton::Encoding,
    ] {
        assert!(in_catalogue(for_diff_bar(b)), "diff bar {b:?} unreachable");
    }
    for a in [
        DiffStageAction::Hunk,
        DiffStageAction::Lines,
        DiffStageAction::SelectLines,
        DiffStageAction::DiscardHunk,
        DiffStageAction::DiscardLines,
    ] {
        for staging in [true, false] {
            assert!(
                in_catalogue(for_diff_stage(a, staging)),
                "diff staging {a:?} unreachable"
            );
        }
    }

    // The Log's own right-click menu. Same exhaustive-match guard as
    // the ref menu above: a sixth row added to `log_context_actions`
    // stops this compiling until it has a palette command too.
    fn for_log_menu(a: LogContextAction) -> PaletteCommand {
        match a {
            LogContextAction::CherryPick => PaletteCommand::CherryPickSelectedCommit,
            LogContextAction::Revert => PaletteCommand::RevertSelectedCommit,
            LogContextAction::Reset(mode) => PaletteCommand::ResetToSelectedCommit(mode),
            LogContextAction::CreateBranchHere => PaletteCommand::CreateBranchAtSelectedCommit,
            LogContextAction::CreateTagHere => PaletteCommand::CreateTagAtSelectedCommit,
            LogContextAction::CheckoutCommit => PaletteCommand::CheckoutSelectedCommit,
            LogContextAction::EditMessage => PaletteCommand::EditSelectedCommitMessage,
            LogContextAction::Amend(scope) => PaletteCommand::AmendSelectedCommit(scope),
            LogContextAction::SplitCommit => PaletteCommand::SplitSelectedCommit,
            LogContextAction::InteractiveRebase => {
                PaletteCommand::InteractiveRebaseFromSelectedCommit
            }
            LogContextAction::CreateWorktree => PaletteCommand::CreateWorktreeAtSelectedCommit,
            LogContextAction::CreatePatch => PaletteCommand::CreatePatchAtSelectedCommit,
            LogContextAction::CreatePatchToHead => {
                PaletteCommand::CreatePatchFromSelectedCommitToHead
            }
            LogContextAction::Copy(f) => PaletteCommand::CopyCommitField(f),
            LogContextAction::Column(op) => {
                PaletteCommand::CommitColumn(crate::commit_column::ColumnCmd::Show(op))
            }
            LogContextAction::AiExplain => PaletteCommand::AiExplainSelectedCommit,
            LogContextAction::AiSplitCommit => PaletteCommand::AiSplitSelectedCommit,
            LogContextAction::CopyCommitLink => PaletteCommand::CopySelectedCommitLink,
            LogContextAction::CopyAsMermaid => PaletteCommand::CopySelectedAsMermaid,
            LogContextAction::CreateAnnotatedTagHere => {
                PaletteCommand::CreateAnnotatedTagAtSelectedCommit
            }
            LogContextAction::ApplyPatch => PaletteCommand::ApplyPatch,
            LogContextAction::RecoverLost => PaletteCommand::LostCommits,
            LogContextAction::RewriteHistory(from) => {
                use crate::log_menu::RewriteFrom;
                use crate::rewrite_view::RewriteEntry;
                PaletteCommand::Rewrite(match from {
                    RewriteFrom::Workbench => RewriteEntry::Workbench,
                    RewriteFrom::Selected => RewriteEntry::SelectedCommits,
                    RewriteFrom::FromHere => RewriteEntry::FromSelectedCommit,
                })
            }
            // A ref's own row inside a merged ref+commit menu: the
            // same palette command the sidebar's menu maps it to.
            LogContextAction::Ref(a, _) => for_ref_menu(a),
        }
    }
    for a in [
        LogContextAction::CherryPick,
        LogContextAction::Revert,
        LogContextAction::EditMessage,
        LogContextAction::Amend(AmendScope::All),
        LogContextAction::Amend(AmendScope::Author),
        LogContextAction::Amend(AmendScope::Summary),
        LogContextAction::Amend(AmendScope::Description),
        LogContextAction::SplitCommit,
        LogContextAction::InteractiveRebase,
        LogContextAction::Reset(ResetMode::Soft),
        LogContextAction::Reset(ResetMode::Mixed),
        LogContextAction::Reset(ResetMode::Hard),
        LogContextAction::CreateBranchHere,
        LogContextAction::CreateTagHere,
        LogContextAction::CreateAnnotatedTagHere,
        LogContextAction::CopyCommitLink,
        LogContextAction::CopyAsMermaid,
        LogContextAction::ApplyPatch,
        LogContextAction::RecoverLost,
        LogContextAction::Ref(RefContextAction::Checkout, 0),
        LogContextAction::CheckoutCommit,
        LogContextAction::CreateWorktree,
        LogContextAction::CreatePatch,
        LogContextAction::CreatePatchToHead,
        LogContextAction::Copy(CommitField::Message),
        LogContextAction::Copy(CommitField::AuthorName),
        LogContextAction::Copy(CommitField::AuthorEmail),
        LogContextAction::Copy(CommitField::Date),
        LogContextAction::Copy(CommitField::ShortSha),
        LogContextAction::Copy(CommitField::LongSha),
        LogContextAction::AiExplain,
        LogContextAction::AiSplitCommit,
        LogContextAction::Column(crate::log_menu::ColumnOp::ShowInLog),
        LogContextAction::Column(crate::log_menu::ColumnOp::Diff),
        LogContextAction::Column(crate::log_menu::ColumnOp::File),
        LogContextAction::Column(crate::log_menu::ColumnOp::Blame),
        LogContextAction::RewriteHistory(crate::log_menu::RewriteFrom::Workbench),
        LogContextAction::RewriteHistory(crate::log_menu::RewriteFrom::Selected),
        LogContextAction::RewriteHistory(crate::log_menu::RewriteFrom::FromHere),
    ] {
        assert!(in_catalogue(for_log_menu(a)), "log menu {a:?} unreachable");
    }

    // The WIP row's own right-click menu. Same exhaustive-match guard
    // as the Log's commit menu just above: a sixth action added to
    // `WipContextAction` stops this compiling until it too has a
    // palette command.
    fn for_wip_menu(a: crate::log_menu::WipContextAction) -> PaletteCommand {
        use crate::log_menu::WipContextAction as W;
        match a {
            W::CreatePatch => PaletteCommand::CreatePatchFromWip,
            W::ApplyPatch => PaletteCommand::ApplyPatch,
            W::StashAll => PaletteCommand::StashPush,
            W::StageAll => PaletteCommand::StageAll,
            W::DiscardAll => PaletteCommand::DiscardAllChanges,
            W::AiExplain => PaletteCommand::AiExplainWip,
            W::AiCompose => PaletteCommand::AiComposeCommits,
        }
    }
    for a in [
        crate::log_menu::WipContextAction::CreatePatch,
        crate::log_menu::WipContextAction::ApplyPatch,
        crate::log_menu::WipContextAction::StashAll,
        crate::log_menu::WipContextAction::StageAll,
        crate::log_menu::WipContextAction::DiscardAll,
        crate::log_menu::WipContextAction::AiExplain,
        crate::log_menu::WipContextAction::AiCompose,
    ] {
        assert!(in_catalogue(for_wip_menu(a)), "WIP menu {a:?} unreachable");
    }

    // The conflict resolutions the third pane's file menu grew. The
    // rest of `FileContextAction` predates this guard and is covered
    // by `every_disabled_context_entry_explains_itself` in
    // `status_pane.rs`; what matters here is that the three rows that
    // really mutate the repo are also in the catalogue.
    fn for_file_menu(side: ConflictSide) -> PaletteCommand {
        match FileContextAction::ResolveUsing(side) {
            FileContextAction::ResolveUsing(s) => PaletteCommand::ResolveConflictWith(s),
            _ => unreachable!("constructed one line above"),
        }
    }
    assert!(
        in_catalogue(PaletteCommand::OpenMergeEditor),
        "the file menu's Open in merge editor is unreachable"
    );
    for side in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
        assert!(
            in_catalogue(for_file_menu(side)),
            "file menu resolution {side:?} unreachable"
        );
    }

    // The Git LFS rows of the same menu. Exhaustive over the LFS
    // variants, so a sixth LFS row stops this compiling until it has
    // a palette command as well.
    fn for_lfs_file_menu(a: FileContextAction) -> Option<PaletteCommand> {
        match a {
            FileContextAction::LfsTrackFile => Some(PaletteCommand::LfsTrackSelectedFile),
            FileContextAction::LfsTrackExtension => Some(PaletteCommand::LfsTrackSelectedExtension),
            FileContextAction::LfsLock => Some(PaletteCommand::LfsLockSelectedFile),
            FileContextAction::LfsUnlock => Some(PaletteCommand::LfsUnlockSelectedFile),
            FileContextAction::LfsForceUnlock => Some(PaletteCommand::LfsForceUnlockSelectedFile),
            _ => None,
        }
    }
    for a in [
        FileContextAction::LfsTrackFile,
        FileContextAction::LfsTrackExtension,
        FileContextAction::LfsLock,
        FileContextAction::LfsUnlock,
        FileContextAction::LfsForceUnlock,
    ] {
        let command = for_lfs_file_menu(a).expect("every LFS file-menu row has a command");
        assert!(in_catalogue(command), "file menu {a:?} unreachable");
    }
}

/// The Repository Management tab and the tab menu, under the same
/// exhaustive-match guard as every other mouse surface: a new button,
/// row control or menu row stops this compiling until it has a
/// palette command too.
#[test]
fn every_management_control_and_tab_menu_row_maps_to_a_palette_command() {
    use crate::repo_manager::{ManagerButton, ManagerHit, Section};
    use ManagementCommand as M;
    fn for_button(b: ManagerButton) -> PaletteCommand {
        match b {
            ManagerButton::Browse => PaletteCommand::BrowseRepository,
            ManagerButton::Clone => PaletteCommand::CloneRepository,
            ManagerButton::Init => PaletteCommand::CreateRepository,
            ManagerButton::CollapseAll => PaletteCommand::Management(M::CollapseAll),
            ManagerButton::ExpandAll => PaletteCommand::Management(M::ExpandAll),
        }
    }
    fn for_hit(h: ManagerHit) -> PaletteCommand {
        match h {
            ManagerHit::Button(b) => for_button(b),
            ManagerHit::Search => PaletteCommand::Management(M::Search),
            // Collapsing one section is Space / Left on its header;
            // the palette reaches all of them at once.
            ManagerHit::Header(_) => PaletteCommand::Management(M::CollapseAll),
            ManagerHit::SectionAction(Section::Recent) => {
                PaletteCommand::Management(M::RemoveAllRecent)
            }
            ManagerHit::SectionAction(_) => PaletteCommand::Management(M::CloseAllTabs),
            ManagerHit::Row(_) | ManagerHit::RowOpen(_) => {
                PaletteCommand::Management(M::OpenSelected)
            }
            ManagerHit::Star(_) => PaletteCommand::Management(M::ToggleSelectedFavorite),
            ManagerHit::RowClose(_) => PaletteCommand::Management(M::CloseOrRemoveSelected),
            ManagerHit::RowMenu(_) => PaletteCommand::Management(M::SelectedMenu),
            ManagerHit::Ws(h) => for_ws_hit(h),
        }
    }
    fn for_ws_hit(h: crate::repo_manager::WsHit) -> PaletteCommand {
        use crate::repo_manager::WsHit as W;
        use WorkspaceCommand as C;
        match h {
            W::New => PaletteCommand::Workspace(C::New),
            W::FromTabs => PaletteCommand::Workspace(C::FromTabs),
            // Enter on a row opens the workspace, or opens / locates the repository.
            W::Row(_) | W::Open(_) | W::Locate(_) => PaletteCommand::Management(M::OpenSelected),
            W::Edit(_) => PaletteCommand::Workspace(C::Edit),
            W::Menu(_) => PaletteCommand::Management(M::SelectedMenu),
            W::Clone(_) => PaletteCommand::Workspace(C::CloneMissing),
            W::Remove(_) => PaletteCommand::Management(M::CloseOrRemoveSelected),
        }
    }
    fn for_menu(a: RepoMenuAction) -> PaletteCommand {
        match a {
            RepoMenuAction::Open => PaletteCommand::Management(M::OpenSelected),
            RepoMenuAction::Close => PaletteCommand::CloseRepositoryTab,
            RepoMenuAction::RemoveFromRecent => {
                PaletteCommand::Management(M::CloseOrRemoveSelected)
            }
            RepoMenuAction::RemoveFromList => PaletteCommand::Management(M::RemoveSelectedFromList),
            RepoMenuAction::CopyPath => PaletteCommand::Management(M::CopySelectedPath),
            RepoMenuAction::SshKey => PaletteCommand::RepoAuthOpen,
            RepoMenuAction::Sparse => PaletteCommand::SparseCheckout,
            RepoMenuAction::NewTab => PaletteCommand::NewTab,
            other => PaletteCommand::TabMenu(other),
        }
    }
    let in_catalogue = |c: PaletteCommand| PALETTE_COMMANDS.iter().any(|e| e.command == c);
    for b in [
        ManagerButton::Browse,
        ManagerButton::Clone,
        ManagerButton::Init,
        ManagerButton::CollapseAll,
        ManagerButton::ExpandAll,
    ] {
        assert!(in_catalogue(for_hit(ManagerHit::Button(b))), "{b:?}");
    }
    for h in [
        ManagerHit::Search,
        ManagerHit::Header(Section::Open),
        ManagerHit::SectionAction(Section::Open),
        ManagerHit::SectionAction(Section::Recent),
        ManagerHit::Row(0),
        ManagerHit::Star(0),
        ManagerHit::RowOpen(0),
        ManagerHit::RowClose(0),
        ManagerHit::RowMenu(0),
    ] {
        assert!(in_catalogue(for_hit(h)), "{h:?} unreachable");
    }
    for a in [
        RepoMenuAction::Open,
        RepoMenuAction::Close,
        RepoMenuAction::CloseOthers,
        RepoMenuAction::CloseToTheRight,
        RepoMenuAction::ToggleFavorite,
        RepoMenuAction::RemoveFromRecent,
        RepoMenuAction::RemoveFromList,
        RepoMenuAction::CopyPath,
        RepoMenuAction::OpenInFileManager,
        RepoMenuAction::OpenInTerminal,
        RepoMenuAction::SshKey,
        RepoMenuAction::Sparse,
        RepoMenuAction::NewTab,
    ] {
        assert!(in_catalogue(for_menu(a)), "menu row {a:?} unreachable");
    }
    assert!(in_catalogue(PaletteCommand::OpenTabMenu));
}

/// Maintainer rule: every palette command that triggers AI shows the
/// sparkle (and only those do).
#[test]
fn every_ai_palette_command_carries_the_sparkle() {
    let mut ai = 0;
    for e in PALETTE_COMMANDS {
        assert_eq!(
            e.display_label().starts_with(crate::theme::AI_SPARK),
            e.command.is_ai(),
            "{}",
            e.label.get()
        );
        if e.command.is_ai() {
            // "AI: ..." rows, and the Rewrite row that names its action.
            assert!(
                e.label.get().contains("AI: ")
                    || e.label.get().ends_with("with AI...")
                    || e.command == PaletteCommand::TabOpenAgentHere,
                "{}",
                e.label.get()
            );
            ai += 1;
        }
    }
    assert_eq!(ai, 14);
}

/// Every row action of the Undo history / Reflog / Lost work panel has a
/// palette row. Exhaustive over `Act`: a new action stops this compiling
/// until it is given one.
#[test]
fn every_safety_panel_action_has_a_palette_command() {
    use crate::safety_panel::{Act, SafetyCmd, Tab};
    fn cmds_for(act: Act) -> Vec<SafetyCmd> {
        match act {
            Act::UndoTo => vec![SafetyCmd::UndoTo],
            Act::RedoTo => vec![SafetyCmd::RedoTo],
            Act::UndoSequence => vec![SafetyCmd::UndoSequence],
            Act::RestoreOnly => vec![SafetyCmd::RestoreOnly],
            Act::Offer => vec![SafetyCmd::Offer],
            Act::ShowInLog => vec![SafetyCmd::ReflogShow],
            Act::CreateBranch => vec![SafetyCmd::ReflogBranch, SafetyCmd::LostBranch],
            Act::CreateTag => vec![SafetyCmd::ReflogTag, SafetyCmd::LostTag],
            Act::ApplyToWorktree => vec![SafetyCmd::ReflogApply, SafetyCmd::LostApply],
            Act::SaveAsStash => vec![SafetyCmd::ReflogStash, SafetyCmd::LostStash],
            Act::RestoreRef => vec![SafetyCmd::ReflogRestore],
            Act::LoadMore => vec![SafetyCmd::ReflogMore],
            Act::CherryPick => vec![SafetyCmd::LostPick],
        }
    }
    for act in [
        Act::UndoTo,
        Act::RedoTo,
        Act::UndoSequence,
        Act::RestoreOnly,
        Act::Offer,
        Act::ShowInLog,
        Act::CreateBranch,
        Act::CreateTag,
        Act::ApplyToWorktree,
        Act::SaveAsStash,
        Act::RestoreRef,
        Act::CherryPick,
        Act::LoadMore,
    ] {
        for cmd in cmds_for(act) {
            assert_eq!(cmd.act(), Some(act));
            assert!(
                PALETTE_COMMANDS
                    .iter()
                    .any(|e| e.command == PaletteCommand::Safety(cmd)),
                "{cmd:?} is not in the catalogue"
            );
        }
    }
    // Refresh of each view, and Close.
    for (tab, cmd) in [
        (Tab::History, SafetyCmd::HistoryRefresh),
        (Tab::Reflog, SafetyCmd::ReflogRefresh),
        (Tab::Lost, SafetyCmd::LostRefresh),
    ] {
        assert_eq!(cmd.tab(), Some(tab));
        assert!(PALETTE_COMMANDS
            .iter()
            .any(|e| e.command == PaletteCommand::Safety(cmd)));
    }
    assert!(PALETTE_COMMANDS
        .iter()
        .any(|e| e.command == PaletteCommand::Safety(SafetyCmd::Close)));
}

/// The panel's rows are available only while their view is open.
#[test]
fn safety_rows_are_available_only_while_their_view_is_open() {
    use crate::safety_panel::{SafetyCmd, Tab};
    let row = PaletteCommand::Safety;
    let ctx = |tab| PaletteContext {
        safety_tab: tab,
        ..Default::default()
    };
    assert!(unavailable_reason(row(SafetyCmd::UndoTo), ctx(None)).is_some());
    assert!(unavailable_reason(row(SafetyCmd::Close), ctx(None)).is_some());
    assert!(unavailable_reason(row(SafetyCmd::UndoTo), ctx(Some(Tab::History))).is_none());
    assert!(unavailable_reason(row(SafetyCmd::UndoTo), ctx(Some(Tab::Reflog))).is_some());
    assert!(unavailable_reason(row(SafetyCmd::ReflogRestore), ctx(Some(Tab::Reflog))).is_none());
    assert!(unavailable_reason(row(SafetyCmd::LostPick), ctx(Some(Tab::Lost))).is_none());
    assert!(unavailable_reason(row(SafetyCmd::LostPick), ctx(Some(Tab::History))).is_some());
    assert!(unavailable_reason(row(SafetyCmd::Close), ctx(Some(Tab::Lost))).is_none());
}
