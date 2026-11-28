use super::families::families;
use super::{sequences, ManagementCommand, PaletteCommand, WorkspaceCommand};
use crate::palette::vocab::repo_menu::RepoMenuAction;
use gitpoulpi_i18n::msg;

/// Everything the applicability rules need to know about the current UI
/// state, flattened to plain data at the call site.
///
/// All-`bool` on purpose - see the module doc.
#[derive(Debug, Clone, Copy, Default)]
pub struct PaletteContext {
    /// The Settings screen is open: only the commands that work over it
    /// (`PaletteCommand::stays_in_settings`) run, the rest are grayed out.
    pub settings_open: bool,
    /// The window is in a workspace.
    pub workspace_active: bool,
    /// A workspace exists (or one is active).
    pub workspaces_known: bool,
    /// The Home tab's Workspaces page has a row selected.
    pub workspace_selected: bool,
    /// The journal's top undo / redo entry (`undo_ui::avail`).
    pub undo: crate::palette::vocab::undo::Avail,
    pub redo: crate::palette::vocab::undo::Avail,
    /// A blocked undo entry offers a reflog position to restore.
    pub undo_offer: bool,
    /// The newest journal entries are one sequence run's.
    pub undo_sequence: bool,
    /// The view the Undo history / Reflog / Lost work panel is open on.
    pub safety_tab: Option<crate::palette::vocab::safety_panel::Tab>,
    /// What the open code review allows (`None`: no review is open).
    pub review: Option<crate::palette::vocab::review::ReviewAvail>,
    /// What the open Kanban board allows (`None`: no board is open).
    pub board: Option<crate::palette::vocab::board::BoardAvail>,
    /// What the open conversation allows (`None`: none is open).
    pub item: Option<crate::palette::vocab::item::ItemAvail>,
    /// What the open text box with the AI writing help allows (`None`: none
    /// is open).
    pub composer: Option<crate::palette::vocab::composer::ComposerAvail>,
    pub in_refs_pane: bool,
    pub in_diff_view: bool,
    /// File View / Blame / History is open in the central column.
    pub in_file_view: bool,
    /// The Blame view is open (`FileInsight::blame_lines`).
    pub blame_shown: bool,
    /// The File View is showing an image (`image_view::pane::active`).
    pub image_shown: bool,
    /// The Diff view is showing before / after pictures
    /// (`image_view::split::active`).
    pub image_diff_shown: bool,
    /// The File History split view is open (`history_view`).
    pub in_history_view: bool,
    /// A binary file with OS apps offers "Open in" (`open_with::view::shown`).
    pub binary_shown: bool,
    /// The open diff or File View has a mouse text selection.
    pub has_text_selection: bool,
    /// Text dragged over in the commit message block.
    pub commit_text_selection: bool,
    /// The open diff is the Unstaged half of the working tree, so its
    /// hunks and lines can be staged. `false` for the Staged half *and*
    /// for a commit's diff -- `in_stageable_diff` tells those two apart.
    pub diff_stages: bool,
    /// The open diff is one of the two working-tree halves, i.e. hunk ids
    /// in it mean something to `StageTarget::{Hunk,Line}` at all.
    pub in_stageable_diff: bool,
    /// A leaf ref row (not a folder) is selected in the Refs sidebar.
    pub selected_local_branch: bool,
    pub selected_remote_branch: bool,
    pub selected_tag: bool,
    /// A ref is marked as the source of a keyboard drop (`ref_drop_mark`).
    pub ref_marked: bool,
    /// The selected tag is one no remote is known to have - the same
    /// state the sidebar draws its marker from, so a grayed-out
    /// `Push Selected Tag` and an unmarked row always agree.
    pub selected_tag_is_unpushed: bool,
    /// The selected tag is on a remote only (not in this repository): the
    /// one state `Fetch Selected Tag` applies to.
    pub selected_tag_remote_only: bool,
    /// The selected tag is here and on a remote at another commit: the one
    /// state `Use the Remote's Version` and `Force Push` apply to.
    pub selected_tag_differs: bool,
    pub soloing: bool,
    /// At least one ref is hidden from the graph - gates
    /// `ShowAllHiddenRefs`, the same way `soloing` gates `StopSoloing`.
    pub has_hidden_refs: bool,
    /// A folder or remote-root row (not a leaf) is selected in the Refs
    /// sidebar - gates `ToggleHideSelectedFolder`/`ToggleSoloSelectedFolder`.
    pub selected_folder: bool,
    /// A row of the Stashes section is selected in the Refs sidebar.
    pub selected_stash: bool,
    /// A row of the Worktrees section is selected in the Refs sidebar.
    pub selected_worktree: bool,
    /// A row of the Submodules section is selected in the Refs sidebar.
    pub selected_submodule: bool,
    /// The repository has submodules at all.
    pub has_submodules: bool,
    pub has_refs_filter: bool,
    /// An agent's view change waits behind the footer chip.
    pub agent_pending: bool,
    /// An agent moved the view and the window can still go back.
    pub agent_back: bool,
    /// What the integration commands can do right now (`hosting_ui`).
    pub hosting: crate::palette::vocab::hosting_command::HostingAvailability,
    pub has_stash: bool,
    /// The repository has a pending auto-stash record (`autostash_banner`).
    pub auto_stash_pending: bool,
    pub unstaged_file_selected: bool,
    pub staged_file_selected: bool,
    /// A changed-file row is selected in the commit-detail pane.
    pub commit_file_selected: bool,
    /// The Log's selection is on a real commit. False on the pinned WIP
    /// row, which has no `Oid` and therefore nothing to cherry-pick,
    /// revert or reset onto.
    pub selected_commit: bool,
    /// The working tree has uncommitted changes, so the Log shows its WIP
    /// row - and with it the inline commit-message field.
    pub has_wip: bool,
    /// The sensitive-files question of a refused stage is up.
    pub stage_guard_open: bool,
    /// The AI commit composer's review is open behind the palette.
    pub compose_open: bool,
    /// ... and a file row is selected in it.
    pub compose_file: bool,
    /// `ai.enabled`: every AI command is hidden from the palette while
    /// this is false.
    pub ai_enabled: bool,
    /// An AI profile of kind local agent exists ("Tab: Open AI Agent Here"
    /// needs one).
    pub local_agent: bool,
    /// An AI request is already running.
    pub ai_running: bool,
    /// The repository is mid-merge/rebase/cherry-pick with unresolved
    /// paths - i.e. `GitBackend::conflicts` returned something.
    pub has_conflicts: bool,
    /// The Status pane's selected file is one of those conflicted paths.
    pub conflicted_file_selected: bool,
    /// The merge conflict editor is the central view.
    pub merge_open: bool,
    /// The built-in editor is the central view.
    pub editor_open: bool,
    /// ... and its Output came from the file on disk: picks are off.
    pub merge_picks_off: bool,
    /// ... and the file has no conflict blocks to pick from.
    pub merge_no_blocks: bool,
    /// The editor shows a card (a conflict that is not text).
    pub merge_card: bool,
    /// How many conflicted files there are (next / previous file).
    pub conflicted_files: usize,
    /// The strip the editor shows: the file was edited outside / was
    /// resolved elsewhere / changed under the editor.
    pub merge_strip_edited: bool,
    pub merge_strip_resolved: bool,
    pub merge_strip_changed: bool,
    /// A merge, cherry-pick or revert is in progress (a rebase is not:
    /// it has its own Abort).
    pub operation_abortable: bool,
    /// ... and it is a cherry-pick or a revert (a merge has nothing to
    /// skip).
    pub operation_skippable: bool,
    /// ... and it is a merge, which can be committed with nothing staged
    /// (its result may equal HEAD).
    pub merge_in_progress: bool,
    /// How many repositories are open as tabs in this session
    /// (ADR-0006). Zero means the window is showing a repository that
    /// is not tabbed - which happens after closing the last tab.
    pub open_repository_tabs: usize,
    /// The current repository tab's index (`Tab: Move Left/Right` need a
    /// neighbour on that side).
    pub current_tab_index: usize,
    /// The repository has at least one remote.
    pub has_remotes: bool,
    /// The grouped push or pull of this repository: a run to show, one
    /// paused on conflicts, its Continue on, something to retry.
    pub group: crate::palette::vocab::push_group::GroupState,
    /// The window is the Repository Management tab.
    pub in_repository_management: bool,
    /// ...and its selection is on a repository row (not a header).
    pub management_row_selected: bool,
    /// The Radar is showing over the Home tab.
    pub radar_open: bool,
    /// The Release notes tab is open.
    pub notes_open: bool,
    /// A newer release is known (the chip says "available").
    pub update_known: bool,
    /// Why the self-update commands cannot run now (`updates::whys`).
    pub update_whys: crate::palette::vocab::updates::Whys,
    /// ...and its selection is on an item (a request, an issue, a WIP row).
    pub radar_row: bool,
    /// The `git.lfs_enabled` preference, resolved for this repository.
    pub lfs_enabled: bool,
    /// The repository has files stored with LFS (the `LFS only` filter has
    /// something to narrow to).
    pub lfs_has_files: bool,
    /// `git-lfs` is usable (or not known to be missing): the LFS commands
    /// that run it are grayed out with the install link when it is not.
    pub lfs_program_ok: bool,
}

impl PaletteContext {
    fn selected_any_ref(self) -> bool {
        self.selected_local_branch || self.selected_remote_branch || self.selected_tag
    }
}

/// Why a merge editor command can't run: the editor is closed, or the
/// file has nothing for it to act on.
fn merge_reason(
    cmd: crate::palette::vocab::merge_view::MergeCmd,
    ctx: PaletteContext,
) -> Option<&'static str> {
    use crate::palette::vocab::merge_view::MergeCmd as M;
    use gitpoulpi_i18n::msg::merge as tr;
    if !ctx.merge_open {
        return Some(tr::REASON_NO_EDITOR.get());
    }
    match cmd {
        M::ChunkA
        | M::ChunkB
        | M::TakeA
        | M::TakeB
        | M::TakeBoth
        | M::TakeBothReversed
        | M::TakeBase
        | M::Clear
        | M::Unpick
        | M::SwapOrder
        | M::TakeAllA
        | M::TakeAllB
        | M::EditInOutput => {
            if ctx.merge_picks_off {
                Some(tr::REASON_PICKS_OFF.get())
            } else if ctx.merge_no_blocks {
                Some(tr::REASON_NO_BLOCKS.get())
            } else {
                None
            }
        }
        M::NextConflict | M::PrevConflict => {
            ctx.merge_no_blocks.then(|| tr::REASON_NO_BLOCKS.get())
        }
        M::NextFile | M::PrevFile => (ctx.conflicted_files < 2).then(|| tr::REASON_ONE_FILE.get()),
        M::ContinueFromDisk | M::StartOver => {
            (!ctx.merge_strip_edited).then(|| tr::REASON_NO_STRIP.get())
        }
        M::Unresolve => (!ctx.merge_strip_resolved).then(|| tr::REASON_NO_STRIP.get()),
        M::Reload => (!ctx.merge_strip_changed).then(|| tr::REASON_NO_STRIP.get()),
        M::ToggleBase
        | M::TogglePanes
        | M::ScrollLeft
        | M::ScrollRight
        | M::ToggleFullFile
        | M::Undo
        | M::Redo
        | M::Save
        | M::SaveResolve
        | M::Reset
        | M::CycleHeight
        | M::WidenCurrent
        | M::WidenIncoming
        | M::ResetPaneSizes
        | M::Close
        | M::FocusNext
        | M::FocusPrev
        | M::FocusPane(_)
        | M::PaneLeft
        | M::PaneRight
        | M::UseFileA
        | M::UseFileB
        | M::UseFileBase => None,
        M::AiSuggest => {
            if !ctx.ai_enabled {
                Some(tr::MSG_AI_OFF.get())
            } else if ctx.ai_running {
                Some(tr::MSG_AI_BUSY.get())
            } else if ctx.merge_picks_off || ctx.merge_no_blocks || ctx.merge_card {
                Some(tr::MSG_AI_NO_TEXT.get())
            } else {
                None
            }
        }
        M::Keep | M::Delete | M::ExternalTool => {
            (!ctx.merge_card).then(|| tr::REASON_NOT_CARD.get())
        }
    }
}

fn undo_reason(
    avail: crate::palette::vocab::undo::Avail,
    verb: &'static str,
) -> Option<&'static str> {
    use crate::palette::vocab::undo::Avail;
    match (avail, verb) {
        (Avail::Ready, _) => None,
        (Avail::Off, _) => Some(msg::palette::WHY_UNDO_IS_TURNED_OFF_SETTINGS_GENERAL.get()),
        (Avail::Nothing, "undo") => Some((families().nothing_to_undo)()),
        (Avail::Nothing, _) => Some(msg::palette::WHY_NOTHING_TO_REDO.get()),
        (Avail::Blocked | Avail::Stuck, "undo") => {
            Some(msg::palette::WHY_CHANGED_OUTSIDE_GITPOULPI_SINCE_HOVER_THE_UNDO.get())
        }
        (Avail::Blocked | Avail::Stuck, _) => {
            Some(msg::palette::WHY_CHANGED_OUTSIDE_GITPOULPI_SINCE_HOVER_THE_REDO.get())
        }
    }
}

/// Why this command can't run right now, or `None` when it can.
///
/// Inapplicable commands stay in the list, grayed out, carrying their
/// reason - the same convention `DiffBarButton::pending_reason` already
/// set for the diff bar. Hiding them would make the palette's contents
/// depend on where the focus happens to be, which is exactly the
/// "memorize the app's modes" burden the palette exists to remove.
pub fn unavailable_reason(command: PaletteCommand, ctx: PaletteContext) -> Option<&'static str> {
    // Settings is a screen over the whole window: a command that acts on a
    // pane, tab or dialog behind it would act unseen. The app-wide ones
    // (themes, language, search, About, help, logs) work over it.
    if ctx.settings_open && !command.stays_in_settings() {
        return Some(msg::palette::WHY_CLOSE_SETTINGS_FIRST.get());
    }
    // A form or popup over the window owns it: only the help's own rows act.
    if ctx.composer.is_some_and(|c| c.modal) && !matches!(command, PaletteCommand::Composer(_)) {
        return Some(msg::md_editor::WHY_TEXT_BOX_OPEN.get());
    }
    base_reason(command, ctx).or_else(|| {
        (ctx.auto_stash_pending && blocked_by_auto_stash(command))
            .then_some(msg::palette::WHY_AN_AUTO_STASH_IS_PENDING_USE_THE_BANNER_S_POP_NOW.get())
    })
}

impl PaletteCommand {
    /// Runs while the Settings screen is open, without closing it: the
    /// commands that do not act on what Settings covers. Everything else is
    /// grayed out with "close Settings first" (and `run_command` refuses it),
    /// so a command never works on a pane the user cannot see and Settings'
    /// unsaved edits are never lost behind their back.
    pub fn stays_in_settings(self) -> bool {
        use PaletteCommand as C;
        matches!(
            self,
            C::ChangeTheme
                | C::ChangeLanguage
                | C::ChangeGraphColorTheme
                | C::ToggleGraphRowTints
                | C::WorkspaceSearch
                | C::AboutGitPoulpi
                | C::AboutLicenses
                | C::TrustReview
                | C::ToggleHelp
                | C::Setting(..)
        )
    }
}

/// The commands whose operation the daemon refuses while an auto-stash is
/// pending (`gitpoulpi-daemon::autostash::blocked_word`). Not Undo: the
/// daemon turns it into *Start over* when it applies.
fn blocked_by_auto_stash(command: PaletteCommand) -> bool {
    use PaletteCommand as C;
    matches!(
        command,
        C::CheckoutSelectedRef
            | C::CheckoutSelectedCommit
            | C::MergeSelectedRef
            | C::RebaseOntoSelectedRef
            | C::InteractiveRebaseFromSelectedCommit
            | C::DropSelectedRefOnCurrent
            | C::DropMarkedRefHere
            | C::FastForwardSelectedToCurrent
            | C::FastForwardCurrentToSelected
            | C::Pull
            | C::PullWithMode(_)
            | C::PullAllBranches
            | C::PullAllBranchesWithMode(_)
            | C::CherryPickSelectedCommit
            | C::RevertSelectedCommit
            | C::EditSelectedCommitMessage
            | C::AmendSelectedCommit(_)
            | C::SplitSelectedCommit
            | C::Redo
    )
}

fn base_reason(command: PaletteCommand, ctx: PaletteContext) -> Option<&'static str> {
    use PaletteCommand as C;
    match command {
        // The journal's top entry (`undo_ui`); the full reason, naming
        // what moved, is in the Undo button's tooltip and the status line.
        C::Undo => undo_reason(ctx.undo, "undo"),
        C::Redo => undo_reason(ctx.redo, "redo"),
        C::UndoForget => (ctx.undo != crate::palette::vocab::undo::Avail::Stuck)
            .then_some(msg::undo_journal::NOT_BLOCKED.get()),
        C::UndoHistory | C::ReflogHead | C::LostCommits => (ctx.undo
            == crate::palette::vocab::undo::Avail::Off)
            .then_some(msg::undo_ui::DISABLED.get()),
        C::Safety(cmd) => cmd.unavailable(ctx.safety_tab),
        C::Review(cmd) => cmd.unavailable(ctx.review),
        C::Board(cmd) => cmd.unavailable(ctx.board),
        C::Item(cmd) => cmd.unavailable(ctx.item),
        C::Composer(cmd) => cmd.unavailable(ctx.composer),
        C::Flow(cmd) => (families().flow_unavailable)(cmd),
        C::UndoSequence => match undo_reason(ctx.undo, "undo") {
            Some(why) => Some(why),
            None => (!ctx.undo_sequence).then_some(msg::undo_ui::WHY_NO_SEQUENCE.get()),
        },
        C::UndoOffer => (!ctx.undo_offer).then_some(msg::undo_safety::WHY_NO_OFFER.get()),

        // The pending auto-stash banner's actions.
        C::AutoStashPop
        | C::AutoStashFinish
        | C::AutoStashShow
        | C::AutoStashDrop
        | C::AutoStashStartOver => (!ctx.auto_stash_pending)
            .then_some(msg::palette::WHY_NO_AUTO_STASH_IS_PENDING_IN_THIS_REPOSITORY.get()),

        // Copying text.
        C::CopySelectedText => {
            if ctx.commit_text_selection {
                None
            } else if !(ctx.in_diff_view || ctx.in_file_view) {
                Some(msg::palette::WHY_OPEN_A_DIFF_FILE_VIEW_BLAME_OR_HISTORY_FIRST.get())
            } else if !ctx.has_text_selection {
                Some(msg::palette::WHY_NO_TEXT_SELECTED_DRAG_OVER_THE_TEXT_FIRST.get())
            } else {
                None
            }
        }
        C::ImageZoomIn
        | C::ImageZoomOut
        | C::ImageFit
        | C::ImageActualSize
        | C::ImageEditIn
        | C::ImageEditDefault => (!ctx.image_shown).then_some(msg::palette::WHY_NO_IMAGE.get()),
        C::Rewrite(
            crate::palette::vocab::rewrite_view::RewriteEntry::SelectedCommits
            | crate::palette::vocab::rewrite_view::RewriteEntry::FromSelectedCommit,
        ) => (!ctx.selected_commit)
            .then_some(msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN.get()),
        C::Rewrite(_) => None,
        C::HistoryView(_) => (!ctx.in_history_view).then_some(msg::palette::WHY_NO_HISTORY_VIEW.get()),
        C::OpenWith | C::OpenWithDefault => {
            (!ctx.binary_shown).then_some(msg::palette::WHY_NO_BINARY.get())
        }
        C::ImageFlipSide | C::ImageWidenBefore | C::ImageWidenAfter | C::ImageResetSplit => {
            (!ctx.image_diff_shown).then_some(msg::palette::WHY_NO_IMAGE_DIFF.get())
        }
        C::BlameOp(_) => (!ctx.blame_shown).then_some(gitpoulpi_i18n::msg::blame_view::WHY_NO_BLAME.get()),
        C::CopyLine => (!(ctx.in_diff_view || ctx.in_file_view))
            .then_some(msg::palette::WHY_OPEN_A_DIFF_FILE_VIEW_BLAME_OR_HISTORY_FIRST.get()),
        C::CopyHunk => {
            (!ctx.in_diff_view).then_some(msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get())
        }

        // File View / Blame / File History open the File Insight overlay on
        // the file the user is pointing at: the open Diff view's file first,
        // then the third pane's selection, then the selected commit's
        // changed-file list. `in_diff_view` belongs here because the Diff
        // view can be opened on a working-tree half whose file no
        // third-pane selection need still match.
        C::Blame | C::History | C::FileView => (!(ctx.in_diff_view
            || ctx.unstaged_file_selected
            || ctx.staged_file_selected
            || ctx.commit_file_selected))
            .then_some(msg::palette::WHY_OPEN_A_FILE_FIRST_IN_THE_DIFF_VIEW_THE_STATUS.get()),
        // The editor opens the working-tree copy of the same file those
        // three would show -- or of the File View's file.
        C::EditInWorkingDirectory => (!(ctx.in_diff_view
            || ctx.in_file_view
            || ctx.unstaged_file_selected
            || ctx.staged_file_selected
            || ctx.commit_file_selected))
            .then_some(msg::palette::WHY_OPEN_A_FILE_FIRST_IN_THE_DIFF_VIEW_THE_STATUS.get()),

        // The editor's status menus need the editor.
        C::EditorMenu(_) => {
            (!ctx.editor_open).then(|| gitpoulpi_i18n::msg::editor::REASON_CLOSED.get())
        }

        // Refs-pane commands acting on the selected row.
        C::CheckoutSelectedRef => {
            if !ctx.selected_any_ref() {
                Some(msg::palette::WHY_SELECT_A_BRANCH_OR_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if ctx.selected_remote_branch {
                Some(msg::palette::WHY_CHECKING_OUT_A_REMOTE_BRANCH_ISN_T_SUPPORTED_YET.get())
            } else {
                None
            }
        }
        C::MergeSelectedRef | C::RebaseOntoSelectedRef => (!ctx.selected_local_branch)
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_BRANCH_IN_THE_REFS_PANE_FIRST.get()),
        C::RenameSelectedBranch | C::FastForwardSelectedToCurrent => (!ctx.selected_local_branch)
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_BRANCH_IN_THE_REFS_PANE_FIRST.get()),
        C::MarkRefAsDropSource => (!ctx.selected_any_ref())
            .then_some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get()),
        C::DropMarkedRefHere => {
            if !ctx.ref_marked {
                Some(msg::palette::WHY_MARK_A_REF_AS_A_DROP_SOURCE_FIRST.get())
            } else if !ctx.selected_any_ref() {
                Some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get())
            } else {
                None
            }
        }
        C::DropSelectedRefOnCurrent => (!(ctx.selected_local_branch
            || ctx.selected_remote_branch
            || ctx.selected_tag))
            .then_some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get()),
        C::FastForwardCurrentToSelected => (!(ctx.selected_local_branch
            || ctx.selected_remote_branch))
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_OR_REMOTE_BRANCH_IN_THE_REFS_PANE.get()),
        C::OpenWorktreeInTab
        | C::WorktreeOpenInSelected
        | C::WorktreeMergeSelected
        | C::WorktreeRemoveSelected => (!ctx.selected_worktree)
            .then_some(msg::palette::WHY_SELECT_A_WORKTREE_IN_THE_REFS_PANE_S_WORKTREES.get()),
        C::SubmoduleUpdateSelected | C::SubmoduleInitSelected | C::SubmoduleOpenSelected => (!ctx
            .selected_submodule)
            .then_some(msg::palette::WHY_SELECT_A_SUBMODULE_IN_THE_REFS_PANE_S_SUBMODULES.get()),
        C::SubmoduleUpdateAll => (!ctx.has_submodules)
            .then_some(msg::palette::WHY_THIS_REPOSITORY_HAS_NO_SUBMODULES.get()),
        C::CreateWorktreeFromSelectedRef => (!ctx.selected_any_ref())
            .then_some(msg::palette::WHY_SELECT_A_BRANCH_OR_A_TAG_IN_THE_REFS_PANE_FIRST.get()),
        // The finer-grained reasons ("push it first", "remote host not
        // recognized") are computed with live remote/upstream/integration
        // data the palette's `PaletteContext` does not carry (the ref
        // menu computes them itself, from state the event loop already
        // has); here, as with every other ref-menu command mirrored into
        // the palette, only "is there a selection to act on" is checked.
        C::StartPullRequestFromSelectedBranch => (!ctx.selected_local_branch)
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_BRANCH_IN_THE_REFS_PANE_FIRST.get()),
        C::ReflogSelectedBranch => (!ctx.selected_local_branch)
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_BRANCH_IN_THE_REFS_PANE_FIRST.get()),
        C::DeleteSelectedRef => (!(ctx.selected_local_branch || ctx.selected_tag))
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_BRANCH_OR_A_TAG_IN_THE_REFS_PANE.get()),
        C::DeleteSelectedRemoteBranch
        | C::DeleteSelectedBranchAndRemote
        | C::CopySelectedBranchLink => (!(ctx.selected_local_branch || ctx.selected_remote_branch))
            .then_some(msg::palette::WHY_SELECT_A_LOCAL_OR_REMOTE_BRANCH_IN_THE_REFS_PANE.get()),
        // Two different reasons, deliberately: "that is not a tag" and
        // "that tag is already on the remote" are different things to
        // fix, and a single "unavailable" would leave the user guessing
        // which.
        C::PushSelectedTag => {
            if !ctx.selected_tag {
                Some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if !ctx.selected_tag_is_unpushed {
                Some(msg::palette::WHY_ORIGIN_ALREADY_HAS_THIS_TAG.get())
            } else {
                None
            }
        }
        // Offered only for a tag the remote *has*: deleting a tag from a
        // remote that never had it is a no-op dressed as a destructive
        // action.
        C::DeleteSelectedTagFromRemote => {
            if !ctx.selected_tag {
                Some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if ctx.selected_tag_is_unpushed {
                Some(msg::palette::WHY_ORIGIN_DOES_NOT_HAVE_THIS_TAG.get())
            } else {
                None
            }
        }
        C::FetchSelectedTag => {
            if !ctx.selected_tag {
                Some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if !ctx.selected_tag_remote_only {
                Some(msg::tag_where::WHY_NOT_REMOTE_ONLY.get())
            } else {
                None
            }
        }
        C::TakeRemoteSelectedTag | C::ForcePushSelectedTag => {
            if !ctx.selected_tag {
                Some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if !ctx.selected_tag_differs {
                Some(msg::tag_where::WHY_NOT_DIFFERING.get())
            } else {
                None
            }
        }
        C::AnnotateSelectedTag => (!ctx.selected_tag)
            .then_some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get()),
        C::MoveSelectedTagToSelectedCommit => {
            if !ctx.selected_tag {
                Some(msg::palette::WHY_SELECT_A_TAG_IN_THE_REFS_PANE_FIRST.get())
            } else if !ctx.selected_commit {
                Some(msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN.get())
            } else {
                None
            }
        }
        C::RevealSelectedRefInGraph => (!ctx.selected_any_ref() && !ctx.selected_stash)
            .then_some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get()),
        C::ToggleHideInGraph | C::ToggleSolo => (!ctx.selected_any_ref())
            .then_some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get()),
        C::StopSoloing => (!ctx.soloing).then_some(msg::palette::WHY_NOTHING_IS_SOLOED.get()),
        C::ShowAllHiddenRefs => {
            (!ctx.has_hidden_refs).then_some(msg::palette::WHY_NOTHING_IS_HIDDEN.get())
        }
        C::ToggleHideSelectedFolder | C::ToggleSoloSelectedFolder => (!ctx.selected_folder)
            .then_some(msg::palette::WHY_SELECT_A_FOLDER_OR_REMOTE_IN_THE_REFS_PANE_FIRST.get()),
        C::HideRemoteInGraph | C::ShowRemoteInGraph => {
            (!ctx.has_remotes).then_some(msg::palette::WHY_THIS_REPOSITORY_HAS_NO_REMOTES.get())
        }
        C::ClearRefsFilter => {
            (!ctx.has_refs_filter).then_some(msg::palette::WHY_NO_NAME_FILTER_IS_ACTIVE.get())
        }
        C::RefsFold(_) => (!ctx.in_refs_pane)
            .then_some(msg::palette::WHY_FOCUS_THE_REFS_PANE_FIRST_PALETTE_FOCUS_REFS_PANE.get()),
        C::FilterRefs
        | C::NextRefSection
        | C::PrevRefSection
        | C::ToggleSectionCollapse
        | C::ToggleHierarchy => (!ctx.in_refs_pane)
            .then_some(msg::palette::WHY_FOCUS_THE_REFS_PANE_FIRST_PALETTE_FOCUS_REFS_PANE.get()),

        // Status-pane commands acting on the selected file.
        C::StageSelectedFile => (!ctx.unstaged_file_selected)
            .then_some(msg::palette::WHY_SELECT_AN_UNSTAGED_FILE_IN_THE_STATUS_PANE_FIRST.get()),
        C::UnstageSelectedFile => (!ctx.staged_file_selected)
            .then_some(msg::palette::WHY_SELECT_A_STAGED_FILE_IN_THE_STATUS_PANE_FIRST.get()),

        // The file context menu's rows, on the Status pane's selection.
        C::FileMenu(action) => {
            use crate::palette::vocab::status_pane::FileContextAction as F;
            if !(ctx.unstaged_file_selected || ctx.staged_file_selected) {
                Some(msg::palette::WHY_SELECT_A_CHANGED_FILE_IN_THE_STATUS_PANE_FIRST.get())
            } else if matches!(action, F::Discard | F::IgnoreFile | F::IgnoreExtension)
                && !ctx.unstaged_file_selected
            {
                Some(msg::palette::WHY_SELECT_AN_UNSTAGED_FILE_IN_THE_STATUS_PANE_FIRST.get())
            } else {
                None
            }
        }

        // Commit-targeted history rewrites, all acting on the Log's
        // selection. The WIP row is deliberately excluded: it is a
        // rendering of the working tree, not a commit, so `selected_commit`
        // is what these ask about rather than "something is selected".
        C::CherryPickSelectedCommit
        | C::RevertSelectedCommit
        | C::EditSelectedCommitMessage
        | C::AmendSelectedCommit(_)
        | C::SplitSelectedCommit
        | C::InteractiveRebaseFromSelectedCommit
        | C::ResetToSelectedCommit(_)
        | C::CreateBranchAtSelectedCommit
        | C::CreateTagAtSelectedCommit
        | C::CreateAnnotatedTagAtSelectedCommit
        | C::CopySelectedCommitLink
        | C::CopySelectedAsMermaid
        | C::CheckoutSelectedCommit
        | C::CreateWorktreeAtSelectedCommit
        | C::CreatePatchAtSelectedCommit
        | C::CreatePatchFromSelectedCommitToHead
        | C::CopyCommitField(_) => (!ctx.selected_commit)
            .then_some(msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN.get()),

        // Neither reads the Log's selection: one patches whatever is
        // currently changed, the other just opens a file picker.
        C::CreatePatchFromWip | C::ApplyPatch => None,

        // Git LFS. The install page is always reachable; everything else
        // needs the preference on, and whatever runs the program needs it
        // installed (the reason carries the link, so the grayed row is
        // itself the "how do I fix this").
        C::FilesToggleLfsOnly => (!ctx.lfs_has_files)
            .then(|| msg::palette::WHY_THIS_REPOSITORY_HAS_NO_LFS_FILES.get()),
        C::LfsOpenInstallPage => None,
        C::LfsToggleRawPointer => {
            (!ctx.in_diff_view).then_some(msg::palette::WHY_OPEN_A_DIFF_FIRST.get())
        }
        // The dropdown itself only needs the preference: with the program
        // missing, its rows lead to the install dialog.
        C::LfsMenu => (!ctx.lfs_enabled)
            .then(|| msg::palette::WHY_GIT_LFS_IS_TURNED_OFF_PREFERENCES_GIT_GIT_LFS.get()),
        C::LfsInitialize
        | C::LfsFetchAll
        | C::LfsCheckout
        | C::LfsPushAll
        | C::LfsTrackPattern
        | C::LfsUntrackPattern
        | C::LfsPrune
        | C::LfsFetch
        | C::LfsPull
        | C::LfsPush
        | C::LfsShowLocks
        | C::LfsTrackSelectedFile
        | C::LfsTrackSelectedExtension
        | C::LfsLockSelectedFile
        | C::LfsUnlockSelectedFile
        | C::LfsForceUnlockSelectedFile => {
            if !ctx.lfs_enabled {
                Some(msg::palette::WHY_GIT_LFS_IS_TURNED_OFF_PREFERENCES_GIT_GIT_LFS.get())
            } else if !ctx.lfs_program_ok {
                Some(msg::palette::WHY_GIT_LFS_IS_NOT_INSTALLED_INSTALL_IT_FROM_HTTPS.get())
            } else if matches!(
                command,
                C::LfsFetch
                    | C::LfsFetchAll
                    | C::LfsPull
                    | C::LfsPush
                    | C::LfsPushAll
                    | C::LfsShowLocks
            ) && !ctx.has_remotes
            {
                Some(msg::palette::WHY_THIS_REPOSITORY_HAS_NO_REMOTE.get())
            } else if matches!(
                command,
                C::LfsTrackSelectedFile
                    | C::LfsTrackSelectedExtension
                    | C::LfsLockSelectedFile
                    | C::LfsUnlockSelectedFile
                    | C::LfsForceUnlockSelectedFile
            ) && !(ctx.unstaged_file_selected || ctx.staged_file_selected)
            {
                Some(msg::palette::WHY_SELECT_A_CHANGED_FILE_IN_THE_STATUS_PANE_FIRST.get())
            } else {
                None
            }
        }

        // Copying a ref works for every kind, remote branches included --
        // nothing is checked out, so the one restriction `CheckoutSelectedRef`
        // carries does not apply here.
        C::CopyRefField(_) => (!ctx.selected_any_ref())
            .then_some(msg::palette::WHY_SELECT_A_REF_IN_THE_REFS_PANE_FIRST.get()),

        // Conflict resolution. Two different reasons, because "there are
        // no conflicts" and "there are, but you haven't pointed at one"
        // send the user to two different places.
        C::ResolveConflictWith(_) => {
            if !ctx.has_conflicts {
                Some(msg::palette::WHY_THE_REPOSITORY_HAS_NO_UNRESOLVED_CONFLICTS.get())
            } else if !ctx.conflicted_file_selected {
                Some(msg::palette::WHY_SELECT_A_CONFLICTED_FILE_IN_THE_STATUS_PANE_FIRST.get())
            } else {
                None
            }
        }

        C::OpenMergeEditor => {
            (!ctx.has_conflicts).then(|| gitpoulpi_i18n::msg::merge::REASON_NO_CONFLICTS.get())
        }
        C::AbortOperation => (!ctx.operation_abortable)
            .then(|| gitpoulpi_i18n::msg::merge::REASON_NO_OPERATION.get()),
        C::SkipOperation => (!ctx.operation_skippable).then(|| {
            if ctx.operation_abortable {
                gitpoulpi_i18n::msg::merge::REASON_NOT_SKIPPABLE.get()
            } else {
                gitpoulpi_i18n::msg::merge::REASON_NO_OPERATION.get()
            }
        }),
        C::Merge(cmd) => merge_reason(cmd, ctx),

        C::EditWipMessage => (!ctx.has_wip && !ctx.merge_in_progress)
            .then_some(msg::palette::WHY_NO_UNCOMMITTED_CHANGES_THE_WIP_ROW_APPEARS_ONCE.get()),

        // Stash commands acting on the most recent entry.
        C::StashPop(_) | C::StashApply | C::StashDrop => {
            (!ctx.has_stash).then_some(msg::palette::WHY_THERE_ARE_NO_STASHES.get())
        }

        // Diff-view commands.
        C::OpenDiff => (!ctx.commit_file_selected)
            .then_some(msg::palette::WHY_SELECT_A_CHANGED_FILE_IN_THE_COMMIT_DETAIL_PANE.get()),
        C::CloseDiff
        | C::NextHunk
        | C::PrevHunk
        | C::DiffMode(_)
        | C::ToggleDiffHunksOnly
        | C::ToggleWordWrap
        | C::DiffChooseEncoding
        | C::ToggleWhitespace
        | C::SelectLines => {
            (!ctx.in_diff_view).then_some(msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get())
        }

        // Partial staging. A hunk id only means something against the
        // working tree -- the backend resolves it against its own
        // working-tree-vs-index (or index-vs-HEAD) diff -- so a commit's
        // diff grays these out with that as the reason rather than
        // sending an id that would name a different hunk entirely.
        // Everything that needs a remote to act on. Add Remote is the one
        // remote command that does not, and stays below with the rest.
        C::Fetch
        | C::Pull
        | C::PullWithMode(_)
        | C::PullAllBranches
        | C::PullAllBranchesWithMode(_)
        | C::Push
        | C::PushToRemote
        | C::ForcePushWithLease
        | C::PushToSelectedRemotes
        | C::PushToAllTargets
        | C::PullFromAllSources
        | C::PullFromOneRemote
        | C::SetUpstream
        | C::PushAllUnpushedTags
        | C::RemoveRemote
        | C::EditRemote
        | C::SetDefaultRemote
        | C::CopyRemoteName
        | C::CopyRemoteLink
        | C::FetchRemote
        | C::RemoteActions => (!ctx.has_remotes)
            .then_some(msg::palette::WHY_THIS_REPOSITORY_HAS_NO_REMOTE_ADD_ONE_FIRST.get()),

        C::ShowGroupedPush => (!ctx.group.run).then_some(msg::palette::WHY_NO_GROUPED_PUSH.get()),
        C::ContinueGroupedPush => {
            if !ctx.group.paused {
                Some(msg::palette::WHY_GROUP_NOT_PAUSED.get())
            } else if !ctx.group.ready {
                Some(msg::palette::WHY_GROUP_NOT_READY.get())
            } else {
                None
            }
        }
        C::AbortGroupedPush => (!ctx.group.paused).then_some(msg::palette::WHY_GROUP_NOT_PAUSED.get()),
        C::RetryFailedPushes => (!ctx.group.retry).then_some(msg::palette::WHY_NOTHING_TO_RETRY.get()),

        C::StageHunk | C::StageLines => {
            if !ctx.in_diff_view {
                Some(msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get())
            } else if !ctx.in_stageable_diff {
                Some(msg::palette::WHY_OPEN_A_FILE_FROM_THE_THIRD_PANE_S_UNSTAGED_HALF_A.get())
            } else if !ctx.diff_stages {
                Some(msg::palette::WHY_THIS_DIFF_IS_THE_STAGED_HALF_USE_THE_UNSTAGE.get())
            } else {
                None
            }
        }
        C::DiscardHunk | C::DiscardLines => {
            if !ctx.in_diff_view {
                Some(msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get())
            } else if !ctx.in_stageable_diff {
                Some(msg::palette::WHY_OPEN_A_FILE_FROM_THE_THIRD_PANE_S_UNSTAGED_HALF_A_2.get())
            } else if !ctx.diff_stages {
                Some(msg::palette::WHY_THIS_DIFF_IS_THE_STAGED_HALF_UNSTAGE_FIRST_THEN.get())
            } else {
                None
            }
        }
        C::UnstageHunk | C::UnstageLines => {
            if !ctx.in_diff_view {
                Some(msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get())
            } else if !ctx.in_stageable_diff {
                Some(msg::palette::WHY_OPEN_A_FILE_FROM_THE_THIRD_PANE_S_STAGED_HALF_A.get())
            } else if ctx.diff_stages {
                Some(msg::palette::WHY_THIS_DIFF_IS_THE_UNSTAGED_HALF_USE_THE_STAGE.get())
            } else {
                None
            }
        }

        // Tab navigation needs somewhere else to go, and closing needs
        // something to close. Both reasons name the state rather than
        // the command, so the user learns what to do about it.
        C::CycleRepositoryTab(_) => (ctx.open_repository_tabs < 2)
            .then_some(msg::palette::WHY_ONLY_ONE_REPOSITORY_IS_OPEN.get()),
        // Moving a tab needs a neighbour on that side, and a tab to move:
        // not on Home, which has none selected.
        C::MoveRepositoryTab(forward) => {
            if ctx.in_repository_management || ctx.open_repository_tabs == 0 {
                Some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            } else if forward && ctx.current_tab_index + 1 >= ctx.open_repository_tabs {
                Some(msg::palette::WHY_TAB_ALREADY_LAST.get())
            } else if !forward && ctx.current_tab_index == 0 {
                Some(msg::palette::WHY_TAB_ALREADY_FIRST.get())
            } else {
                None
            }
        }
        C::AgentShowPending | C::AgentDismissPending => {
            (!ctx.agent_pending).then_some(msg::palette::WHY_NO_AGENT_VIEW_IS_WAITING.get())
        }
        C::SequenceCancel => (!(families().sequence_running)())
            .then_some(msg::palette::WHY_NO_SEQUENCE_RUNNING.get()),
        C::SequenceContinue => (!(families().sequence_can_continue)())
            .then_some(msg::palette::WHY_SEQUENCE_NOT_WAITING.get()),
        C::SequenceRepoReview | C::SequenceRepoTrust | C::SequenceRepoIgnore => {
            // The strip may be hidden (ignored, or no free border row): the
            // commands answer while the file still waits for an answer.
            (!(families().repo_sequences_pending)())
                .then_some(msg::palette::WHY_NO_REPO_SEQUENCES.get())
        }
        C::AgentBack => (!ctx.agent_back).then_some(msg::palette::WHY_NOTHING_TO_GO_BACK_TO.get()),
        C::CloseRepositoryTab | C::SelectTab => (ctx.open_repository_tabs == 0)
            .then_some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get()),
        C::SparseCheckout | C::RepoAuthOpen | C::RepoAuthTest | C::RepoAuthDefaults => {
            if ctx.in_repository_management {
                Some(msg::palette::WHY_OPEN_A_REPOSITORY_TAB_FIRST_THE_PIN_IS_PER.get())
            } else {
                (ctx.open_repository_tabs == 0)
                    .then_some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            }
        }

        // Workspaces.
        C::Workspace(cmd) => {
            use WorkspaceCommand as W;
            let wm = gitpoulpi_i18n::msg::workspaces::WHY_NONE_YET;
            match cmd {
                W::New | W::Manage | W::CloseTab => None,
                W::FromTabs => (ctx.open_repository_tabs == 0)
                    .then_some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get()),
                W::Switch => (!ctx.workspaces_known).then_some(wm.get()),
                W::Close => (!ctx.workspace_active)
                    .then_some(gitpoulpi_i18n::msg::workspaces::WHY_NONE_ACTIVE.get()),
                W::Locate => (!ctx.workspace_selected)
                    .then_some(gitpoulpi_i18n::msg::workspaces::SELECT_REPOSITORY_FIRST.get()),
                W::Edit | W::Delete | W::FetchAll | W::PullAll | W::CloneMissing => {
                    if !ctx.workspaces_known {
                        Some(wm.get())
                    } else if !ctx.workspace_active && !ctx.workspace_selected {
                        Some(gitpoulpi_i18n::msg::workspaces::WHY_NO_TARGET.get())
                    } else {
                        None
                    }
                }
                W::AddTab | W::RemoveTab => {
                    if !ctx.workspace_active {
                        Some(gitpoulpi_i18n::msg::workspaces::WHY_NONE_ACTIVE.get())
                    } else if ctx.open_repository_tabs == 0 && !ctx.management_row_selected {
                        Some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
                    } else {
                        None
                    }
                }
            }
        }

        // The management tab's own actions need the management tab, and
        // the row actions a selected repository.
        C::Management(command) => {
            use ManagementCommand as M;
            if !ctx.in_repository_management {
                Some(msg::palette::WHY_GO_TO_THE_REPOSITORY_MANAGEMENT_TAB_FIRST_THE.get())
            } else if matches!(
                command,
                M::OpenSelected
                    | M::ToggleSelectedFavorite
                    | M::CloseOrRemoveSelected
                    | M::SelectedMenu
                    | M::CopySelectedPath
                    | M::OpenSelectedIn
            ) && !ctx.management_row_selected
            {
                Some(msg::palette::WHY_SELECT_A_REPOSITORY_IN_THE_LIST_FIRST.get())
            } else if command == M::CloseAllTabs && ctx.open_repository_tabs == 0 {
                Some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            } else {
                None
            }
        }
        // The Radar's own actions need the Radar open, and the row
        // actions an item selected.
        C::Radar(command) => {
            use crate::palette::vocab::radar::{Act, RadarCommand as L};
            match command {
                L::Open | L::CloseTab => None,
                _ if !ctx.radar_open => Some(msg::palette::WHY_RADAR_CLOSED.get()),
                L::Act(
                    Act::OpenBrowser
                    | Act::OpenRepo
                    | Act::Checkout
                    | Act::Worktree
                    | Act::Show
                    | Act::StartWork
                    | Act::Pin
                    | Act::Snooze
                    | Act::Unsnooze
                    | Act::EditAssignees
                    | Act::EditReviewers
                    | Act::EditLabels
                    | Act::SetMilestone
                    | Act::Review
                    | Act::Conversation
                    | Act::ReviewApi
                    | Act::CloneReview
                    | Act::Merge
                    | Act::CloseItem
                    | Act::Ready
                    | Act::CloneRepo,
                )
                | L::Menu
                    if !ctx.radar_row =>
                {
                    Some(msg::palette::WHY_RADAR_NO_ROW.get())
                }
                _ => None,
            }
        }
        // Download and Skip need a newer release; Close needs the tab.
        C::Updates(command) => {
            use crate::palette::vocab::updates::UpdatesCommand as U;
            match command {
                U::ReleaseNotes | U::CheckNow | U::ShowPanel => None,
                U::CopyCommand => ctx.update_whys.copy,
                U::RunInTerminal => ctx.update_whys.terminal,
                U::SecurityNotice => ctx.update_whys.notice,
                U::Download | U::Skip => {
                    (!ctx.update_known).then_some(msg::updates::WHY_NO_UPDATE.get())
                }
                U::CloseTab => (!ctx.notes_open).then_some(msg::updates::WHY_NOTES_CLOSED.get()),
                U::UpdateNow => ctx.update_whys.update_now,
                U::Restart => ctx.update_whys.restart,
                U::Revert => ctx.update_whys.revert,
            }
        }
        // The tab menu acts on the current repository: the active tab,
        // or the management tab's selected row.
        C::OpenTabMenu => {
            if ctx.in_repository_management {
                Some(msg::palette::WHY_THE_REPOSITORY_MANAGEMENT_TAB_HAS_NO_TAB_MENU_USE.get())
            } else {
                (ctx.open_repository_tabs == 0)
                    .then_some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            }
        }
        C::TabOpenAgentHere => {
            if !ctx.ai_enabled {
                Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
            } else if ctx.in_repository_management {
                Some(msg::palette::WHY_THE_REPOSITORY_MANAGEMENT_TAB_HAS_NO_FOLDER.get())
            } else if ctx.open_repository_tabs == 0 {
                Some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            } else {
                (!ctx.local_agent).then_some(msg::palette::WHY_NO_LOCAL_AI_AGENT.get())
            }
        }
        C::TabDir(_) | C::TabOpenIn | C::TabOpenInDefault => {
            if ctx.in_repository_management {
                Some(msg::palette::WHY_THE_REPOSITORY_MANAGEMENT_TAB_HAS_NO_FOLDER.get())
            } else {
                (ctx.open_repository_tabs == 0)
                    .then_some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            }
        }
        C::TabMenu(action) => {
            if ctx.in_repository_management && !ctx.management_row_selected {
                Some(msg::palette::WHY_SELECT_A_REPOSITORY_IN_THE_LIST_FIRST.get())
            } else if !ctx.in_repository_management && ctx.open_repository_tabs == 0 {
                Some(msg::palette::WHY_NO_REPOSITORY_TAB_IS_OPEN.get())
            } else if action == RepoMenuAction::CloseOthers && ctx.open_repository_tabs < 2 {
                Some(msg::palette::WHY_ONLY_ONE_REPOSITORY_IS_OPEN.get())
            } else {
                None
            }
        }

        C::Hosting(h) => super::vocab::hosting_command::command_unavailable_from(h, ctx.hosting),

        C::AiGenerateStashMessage => {
            if !ctx.ai_enabled {
                Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
            } else if ctx.ai_running {
                Some(msg::palette::WHY_AN_AI_REQUEST_IS_ALREADY_RUNNING_ESC_CANCELS_IT.get())
            } else if !ctx.has_wip {
                Some(msg::palette::WHY_NO_UNCOMMITTED_CHANGES.get())
            } else {
                None
            }
        }
        C::ComposerMoveFile => {
            if !ctx.compose_open {
                Some(msg::palette::WHY_OPEN_THE_COMMIT_COMPOSER_REVIEW_FIRST_AI_SPLIT.get())
            } else if !ctx.compose_file {
                Some(msg::palette::WHY_SELECT_A_FILE_IN_THE_REVIEW_FIRST.get())
            } else {
                None
            }
        }
        // Git refuses to commit unmerged files: grayed with the reason (the
        // action itself opens the guiding dialog if something still runs it).
        C::Commit | C::CommitNow if ctx.has_conflicts => {
            Some(gitpoulpi_i18n::msg::conflict_guard::WHY_COMMIT.get())
        }
        C::StageGuardShowInFile => {
            (!ctx.stage_guard_open).then_some(msg::palette::WHY_NO_SENSITIVE_FILES_QUESTION.get())
        }
        C::AiComposeCommits => {
            if !ctx.ai_enabled {
                Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
            } else if ctx.ai_running {
                Some(msg::palette::WHY_AN_AI_REQUEST_IS_ALREADY_RUNNING_ESC_CANCELS_IT.get())
            } else if !ctx.has_wip {
                Some(msg::palette::WHY_NO_UNCOMMITTED_CHANGES.get())
            } else {
                None
            }
        }
        C::AiInsertPromptVariable | C::AiEditProfile | C::AiOpenAgent if !ctx.ai_enabled => {
            Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
        }
        C::AiInsertPromptVariable | C::AiEditProfile | C::AiOpenAgent => None,
        C::AiExplainSelectedCommit | C::AiSplitSelectedCommit | C::AiExplainWip
            if !ctx.ai_enabled =>
        {
            Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
        }
        C::AiExplainSelectedCommit | C::AiSplitSelectedCommit => (!ctx.selected_commit)
            .then_some(msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN.get()),
        C::AiExplainWip => (!ctx.has_wip).then_some(msg::palette::WHY_NO_UNCOMMITTED_CHANGES.get()),
        C::AiGenerateCommitMessage => {
            if !ctx.ai_enabled {
                Some(msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI.get())
            } else if ctx.ai_running {
                Some(msg::palette::WHY_AN_AI_REQUEST_IS_ALREADY_RUNNING_ESC_CANCELS_IT.get())
            } else {
                None
            }
        }

        C::Jobs(crate::palette::vocab::jobs_ui::JobsCmd::CancelAll) => (!(families().any_job_cancellable)())
            .then_some(gitpoulpi_i18n::msg::jobs::PALETTE_NONE.get()),

        C::OpenLastError | C::CopyLastError | C::RunLastErrorRemedy
            if !(families().status_error_present)() =>
        {
            Some(gitpoulpi_i18n::msg::activity_log::WHY_NO_LAST_ERROR.get())
        }
        C::OpenLastError | C::CopyLastError => None,
        C::RunLastErrorRemedy => (!(families().last_error_has_remedy)())
            .then_some(gitpoulpi_i18n::msg::activity_log::WHY_NO_REMEDY.get()),

        // Always available.
        C::OpenRepository
        | C::NewTab
        | C::OpenNewMenu
        | C::BrowseRepository
        | C::CloneRepository
        | C::CreateRepository
        | C::FocusRefs
        | C::FocusLog
        | C::FocusStatus
        | C::FocusRefsFilter
        | C::FocusRefsSection(_)
        | C::SidebarSectionsMenu
        | C::ShowWorktreesSection
        | C::CreateWorktreeMenu
        | C::CreateWorktreeFromCurrentBranch
        | C::CreateWorktreeChooseBranch
        | C::ToggleSidebarSection(_)
        | C::ResetSidebarSections
        | C::UseSidebarSectionsForAll
        | C::FocusStatusSection(_)
        | C::CycleFocus
        | C::CycleFocusBack
        | C::SearchLog
        | C::FilterLogByAuthor
        | C::ClearAuthorFilter
        | C::ToggleRefsPaneCollapsed
        | C::ToggleStatusPaneCollapsed
        | C::ResizePane(_)
        | C::GrowRefsSection
        | C::ShrinkRefsSection
        | C::ResetRefsSectionSizes
        | C::ResetLayout
        | C::GrowCommitMessage
        | C::ShrinkCommitMessage
        | C::ResetCommitMessageSize
        | C::ScrollCommitMessageDown
        | C::ScrollCommitMessageUp
        | C::OpenMessageLink
        | C::ToggleCommitFileSort
        | C::StageAll
        | C::DiscardAllChanges
        | C::CreateFile
        | C::OpenFolderMenu
        | C::Commit
        | C::CommitNow
        | C::NewBranch
        | C::NewTag
        | C::StashPush
        | C::StashPushWithMessage
        | C::AddRemote
        | C::ChoosePullButtonAction
        | C::Refresh
        | C::RefreshAll
        | C::ToggleHelp
        | C::AboutGitPoulpi
        | C::AboutLicenses
        | C::TrustReview
        | C::WorkspaceSearch
        | C::OpenActivityLog
        | C::OpenGlobalLog
        | C::Jobs(crate::palette::vocab::jobs_ui::JobsCmd::Show | crate::palette::vocab::jobs_ui::JobsCmd::Cancel(_))
        | C::CopyDiagnostics
        | C::ActivityLevel(_)
        | C::ChangeProfile
        | C::ToggleColumn(_)
        | C::LogColumn(_)
        | C::OpenPreferences
        | C::SequenceCreateAi
        | C::SearchSettings
        | C::Keybindings
        | C::KeybindingsResetAll
        | C::Theme
        | C::ConflictLists
        | C::ConflictRepoPage
        | C::ChangeTheme
        | C::ChangeLanguage
        | C::LauncherCreate
        | C::LauncherCreateDesktop
        | C::LauncherRemove
        | C::LauncherAliasCreate
        | C::LauncherAliasRemove
        | C::McpSuggestPort
        | C::LauncherPathAdd
        | C::LauncherPathRemove
        | C::LauncherFolderMenuInstall
        | C::LauncherFolderMenuRemove
        | C::LauncherBecomePrimary
        | C::LauncherRemoveLegacy
        | C::LauncherForgetGone
        | C::ChangeGraphColorTheme
        | C::ToggleGraphRowTints
        | C::RebaseContinue
        | C::RebaseSkip
        | C::RebaseAbort
        | C::AmContinue
        | C::AmSkip
        | C::AmAbort
        | C::IndexLockRemove
        | C::FilesTogglePathTree
        | C::FilesToggleViewAll
        | C::CommitToggleSkipHooks
        | C::ConflictsShow
        | C::FilesOpenMenu
        | C::FilesFold(_)
        | C::PaneScroll(_)
        | C::FilesSelectAll
        | C::FilesClearSelection
        | C::FilesToggleSelection
        | C::CompareSelection
        | C::CompareSwap
        | C::CompareGotoNext
        | C::ToggleMinimap
        | C::ToggleDiffIntraline
        | C::EditorToggleFollowSelection
        | C::CommitColumn(_)
        | C::Quit
        // A setting is editable from anywhere (a Home tab included).
        | C::Setting(..) => None,
        // A sequence is gated by its own validation only.
        C::Sequence(op) => sequences::unavailable(op),
    }
}
