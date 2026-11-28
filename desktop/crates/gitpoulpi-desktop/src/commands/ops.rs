//! The dispatcher's table: every command the desktop implements, as an [`Op`].
//!
//! [`op_for`] is the only place that says "the desktop does this". A command it maps to `None` is not
//! implemented yet: running it answers "not available yet" in the status line (see `exec`). The parity
//! table derives its `done` set from this function ([`implemented_ids`]), so a command cannot be marked
//! done without being handled here.

use std::collections::BTreeSet;

use gitpoulpi_app::keymap::{Action, Ctx};
use gitpoulpi_app::palette::vocab::diff_view::DiffViewMode;
use gitpoulpi_app::palette::vocab::log_table::LogColumnCmd;
use gitpoulpi_app::palette::vocab::status_pane::{FileContextAction, StatusSection};
use gitpoulpi_app::palette::vocab::tree_fold::TreeFold;
use gitpoulpi_app::palette::{palette_entry, PaletteCommand};
use gitpoulpi_app::registry::{parity_rows, ParityKind};

use super::catalog::{command_for_id, Command};
use crate::daemon::transfer::Transfer;

/// What the desktop does for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    OpenPalette,
    Quit,
    /// Open the Log's find bar.
    FindInLog,
    /// Reload the refs and the Log.
    Refresh,
    /// The keybindings view.
    Keybindings,
    /// The first key of a two-key chord (Ctrl+G, Ctrl+N): the next key is looked up in that context.
    Prefix(Ctx),
    ToggleRefsPane,
    ToggleStatusPane,
    ChangeTheme,
    ChangeLanguage,
    ChangeGraphColors,
    FocusRefs,
    FocusRefsFilter,
    FocusLog,
    FocusStatus,
    /// The folder chooser of the operating system, then open that repository.
    BrowseRepository,
    FitColumns,
    ResetColumns,
    /// Fetch, Pull, Push, Undo or Redo through the daemon (the toolbar's five write buttons).
    Git(Transfer),
    /// Hide the selected Refs row from the graph, or show it again.
    RefsToggleHide,
    /// Solo the selected Refs row, or clear it.
    RefsToggleSolo,
    RefsClearSolo,
    RefsShowAllHidden,
    /// Fold or unfold the Refs folders (`ExpandAll` / `CollapseAll`; the "below" forms are not there yet).
    RefsFold(TreeFold),
    /// Fold or unfold the section the Refs selection is in.
    RefsToggleSection,
    /// Close the file diff and go back to the Log.
    DiffClose,
    /// The layout of the file diff: `None` toggles, `Some` sets it.
    DiffLayout(Option<DiffViewMode>),
    DiffToggleWrap,
    DiffToggleWhitespace,
    DiffStepHunk {
        forward: bool,
    },
    /// Staging and committing in the working-changes pane.
    Status(StatusOp),
    /// Stage, unstage or discard a hunk or lines of the open working-tree diff.
    DiffPart(DiffPartOp),
}

/// What the working-changes pane does (its lists and the commit form).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusOp {
    StageSelected,
    UnstageSelected,
    StageAll,
    DiscardAll,
    DiscardSelected,
    /// Stage what is selected in Unstaged, unstage what is selected in Staged (`Ctrl+Space`).
    ToggleStage,
    OpenDiff,
    TogglePathTree,
    ToggleAmend,
    ToggleSkipHooks,
    /// The keyboard goes to the commit summary.
    FocusMessage,
    /// The commit button.
    Commit,
    /// `Ctrl+G Enter`: commit when ready, else take the keyboard to the summary.
    CommitNow,
    SelectAll,
    ClearSelection,
    ToggleSelection,
    FocusSection(StatusSection),
    /// Fold or unfold the folders of the Tree lists (`=`, `-`, `*`, `_`).
    Fold(TreeFold),
    /// `z`: fold all folders, or unfold them when some are folded.
    ToggleFoldAll,
}

/// What the open diff does with the hunk under the cursor or the selected lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffPartOp {
    /// `Space`: stage or unstage the hunk, by the half the diff was opened from.
    ToggleHunk,
    /// `Enter`: stage or unstage the selected lines.
    ToggleLines,
    StageHunk,
    UnstageHunk,
    StageLines,
    UnstageLines,
    DiscardHunk,
    DiscardLines,
    /// `v`: start or drop a line selection.
    SelectLines,
}

/// The op of a key action, if the desktop implements it.
pub fn op_for_action(action: Action) -> Option<Op> {
    use Action as A;
    Some(match action {
        A::GlobalPalette => Op::OpenPalette,
        A::GlobalQuit => Op::Quit,
        A::GlobalFindInLog => Op::FindInLog,
        A::GlobalRefresh => Op::Refresh,
        A::GlobalHelp | A::NavToggleHelp => Op::Keybindings,
        A::GlobalGitPrefix => Op::Prefix(Ctx::GitPrefix),
        A::GlobalNavPrefix => Op::Prefix(Ctx::NavPrefix),
        A::NavCollapseLeft => Op::ToggleRefsPane,
        A::NavCollapseRight => Op::ToggleStatusPane,
        A::NavChangeTheme => Op::ChangeTheme,
        A::NavChangeLanguage => Op::ChangeLanguage,
        A::NavChangeGraphColorTheme => Op::ChangeGraphColors,
        A::LogColumnsFitAll => Op::FitColumns,
        A::LogColumnsResetWidth => Op::ResetColumns,
        A::GlobalUndo => Op::Git(Transfer::Undo),
        A::GlobalRedo => Op::Git(Transfer::Redo),
        A::GitFetchAll => Op::Git(Transfer::Fetch),
        A::GitPull => Op::Git(Transfer::Pull),
        A::GitPush => Op::Git(Transfer::Push),
        A::RefsSolo => Op::RefsToggleSolo,
        A::RefsClearSolo => Op::RefsClearSolo,
        A::RefsShowAll => Op::RefsShowAllHidden,
        A::RefsExpandAll => Op::RefsFold(TreeFold::ExpandAll),
        A::RefsCollapseAll => Op::RefsFold(TreeFold::CollapseAll),
        A::DiffClose => Op::DiffClose,
        A::DiffNextHunk => Op::DiffStepHunk { forward: true },
        A::DiffPrevHunk => Op::DiffStepHunk { forward: false },
        A::DiffToggleLayout => Op::DiffLayout(None),
        A::DiffToggleWrap => Op::DiffToggleWrap,
        A::DiffToggleWhitespace => Op::DiffToggleWhitespace,
        A::GitStageFile => Op::Status(StatusOp::StageSelected),
        A::GitUnstageFile => Op::Status(StatusOp::UnstageSelected),
        A::GitStageAll => Op::Status(StatusOp::StageAll),
        A::GitDiscardAll => Op::Status(StatusOp::DiscardAll),
        A::GitCommitNow | A::GitCompose => Op::Status(StatusOp::CommitNow),
        A::StatusCompose => Op::Status(StatusOp::FocusMessage),
        A::StatusToggleFileMode => Op::Status(StatusOp::TogglePathTree),
        A::StatusAmend => Op::Status(StatusOp::ToggleAmend),
        A::StatusSkipHooks => Op::Status(StatusOp::ToggleSkipHooks),
        A::StatusStageToggle => Op::Status(StatusOp::ToggleStage),
        A::StatusOpenDiff => Op::Status(StatusOp::OpenDiff),
        A::ComposerCommit => Op::Status(StatusOp::Commit),
        A::StatusCollapseAll => Op::Status(StatusOp::ToggleFoldAll),
        A::StatusExpandAll => Op::Status(StatusOp::Fold(TreeFold::ExpandAll)),
        A::StatusCollapseAllFolders => Op::Status(StatusOp::Fold(TreeFold::CollapseAll)),
        A::StatusExpandBelow => Op::Status(StatusOp::Fold(TreeFold::ExpandBelow)),
        A::StatusCollapseBelow => Op::Status(StatusOp::Fold(TreeFold::CollapseBelow)),
        A::DiffStageHunk => Op::DiffPart(DiffPartOp::ToggleHunk),
        A::DiffStageLines => Op::DiffPart(DiffPartOp::ToggleLines),
        A::DiffDiscardHunk => Op::DiffPart(DiffPartOp::DiscardHunk),
        A::DiffDiscardLines => Op::DiffPart(DiffPartOp::DiscardLines),
        A::DiffSelectLines => Op::DiffPart(DiffPartOp::SelectLines),
        // A palette command bound to a key of its own.
        A::Palette(_) => return palette_entry(action).and_then(|e| op_for_palette(e.command)),
        _ => return None,
    })
}

/// The op of a palette command, if the desktop implements it.
pub fn op_for_palette(command: PaletteCommand) -> Option<Op> {
    use PaletteCommand as C;
    Some(match command {
        C::Quit => Op::Quit,
        C::SearchLog => Op::FindInLog,
        C::Refresh => Op::Refresh,
        C::ToggleHelp | C::Keybindings => Op::Keybindings,
        C::ToggleRefsPaneCollapsed => Op::ToggleRefsPane,
        C::ToggleStatusPaneCollapsed => Op::ToggleStatusPane,
        C::ChangeTheme | C::Theme => Op::ChangeTheme,
        C::ChangeLanguage => Op::ChangeLanguage,
        C::ChangeGraphColorTheme => Op::ChangeGraphColors,
        C::FocusRefs => Op::FocusRefs,
        C::FocusRefsFilter => Op::FocusRefsFilter,
        C::FocusLog => Op::FocusLog,
        C::FocusStatus => Op::FocusStatus,
        C::BrowseRepository => Op::BrowseRepository,
        C::Undo => Op::Git(Transfer::Undo),
        C::Redo => Op::Git(Transfer::Redo),
        C::Fetch => Op::Git(Transfer::Fetch),
        C::Pull => Op::Git(Transfer::Pull),
        C::Push => Op::Git(Transfer::Push),
        C::ToggleHideInGraph | C::ToggleHideSelectedFolder => Op::RefsToggleHide,
        C::ToggleSolo | C::ToggleSoloSelectedFolder => Op::RefsToggleSolo,
        C::StopSoloing => Op::RefsClearSolo,
        C::ShowAllHiddenRefs => Op::RefsShowAllHidden,
        C::RefsFold(f @ (TreeFold::ExpandAll | TreeFold::CollapseAll)) => Op::RefsFold(f),
        C::ToggleSectionCollapse => Op::RefsToggleSection,
        C::CloseDiff => Op::DiffClose,
        C::NextHunk => Op::DiffStepHunk { forward: true },
        C::PrevHunk => Op::DiffStepHunk { forward: false },
        C::DiffMode(mode) => Op::DiffLayout(Some(mode)),
        C::ToggleWordWrap => Op::DiffToggleWrap,
        C::ToggleWhitespace => Op::DiffToggleWhitespace,
        C::StageSelectedFile | C::FileMenu(FileContextAction::Stage) => {
            Op::Status(StatusOp::StageSelected)
        }
        C::UnstageSelectedFile | C::FileMenu(FileContextAction::Unstage) => {
            Op::Status(StatusOp::UnstageSelected)
        }
        C::FileMenu(FileContextAction::Discard) => Op::Status(StatusOp::DiscardSelected),
        C::StageAll => Op::Status(StatusOp::StageAll),
        C::DiscardAllChanges => Op::Status(StatusOp::DiscardAll),
        C::Commit | C::CommitNow => Op::Status(StatusOp::CommitNow),
        C::FilesTogglePathTree => Op::Status(StatusOp::TogglePathTree),
        C::CommitToggleSkipHooks => Op::Status(StatusOp::ToggleSkipHooks),
        C::FilesSelectAll => Op::Status(StatusOp::SelectAll),
        C::FilesClearSelection => Op::Status(StatusOp::ClearSelection),
        C::FilesToggleSelection => Op::Status(StatusOp::ToggleSelection),
        C::FocusStatusSection(section) => Op::Status(StatusOp::FocusSection(section)),
        C::FilesFold(f) => Op::Status(StatusOp::Fold(f)),
        C::StageHunk => Op::DiffPart(DiffPartOp::StageHunk),
        C::UnstageHunk => Op::DiffPart(DiffPartOp::UnstageHunk),
        C::StageLines => Op::DiffPart(DiffPartOp::StageLines),
        C::UnstageLines => Op::DiffPart(DiffPartOp::UnstageLines),
        C::DiscardHunk => Op::DiffPart(DiffPartOp::DiscardHunk),
        C::DiscardLines => Op::DiffPart(DiffPartOp::DiscardLines),
        C::SelectLines => Op::DiffPart(DiffPartOp::SelectLines),
        C::LogColumn(LogColumnCmd::FitAllToContent) => Op::FitColumns,
        C::LogColumn(LogColumnCmd::ResetWidth) => Op::ResetColumns,
        _ => return None,
    })
}

/// The op of a command, `None` when the desktop does not implement it yet.
pub fn op_for(command: Command) -> Option<Op> {
    match command {
        Command::Action(a) => op_for_action(a),
        Command::Palette(c) => op_for_palette(c),
    }
}

/// The ids (of the registry's actions and palette commands) the dispatcher implements.
pub fn implemented_ids() -> BTreeSet<String> {
    parity_rows()
        .into_iter()
        .filter(|r| matches!(r.kind, ParityKind::Action | ParityKind::Palette))
        .filter(|r| command_for_id(&r.id).and_then(op_for).is_some())
        .map(|r| r.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_app::palette::PALETTE_COMMANDS;

    /// A palette command and the key action it is the only twin of must do the same thing, or the merged
    /// parity row would say "done" for one and not the other.
    #[test]
    fn a_command_and_its_key_action_agree() {
        use gitpoulpi_app::palette::bindings::static_bind_action;
        for entry in PALETTE_COMMANDS {
            let Some(action) = static_bind_action(entry) else {
                continue;
            };
            let twins = PALETTE_COMMANDS
                .iter()
                .filter(|e| static_bind_action(e) == Some(action))
                .count();
            if twins != 1 {
                continue;
            }
            assert_eq!(
                op_for_action(action),
                op_for_palette(entry.command),
                "{:?}",
                entry.command
            );
        }
    }

    #[test]
    fn the_known_commands_are_implemented() {
        let ids = implemented_ids();
        for id in [
            "global.palette",
            "global.quit",
            "global.find_in_log",
            "nav_prefix.collapse_refs",
            "nav_prefix.collapse_status",
            "nav_prefix.change_theme",
            "nav_prefix.change_language",
            "nav_prefix.change_graph_color_theme",
            "palette.focus_log",
        ] {
            assert!(ids.contains(id), "{id}");
        }
        for id in [
            "git_prefix.fetch_all",
            "git_prefix.pull",
            "git_prefix.push",
            "global.undo",
            "global.redo",
        ] {
            assert!(ids.contains(id), "{id}");
        }
        assert!(!ids.contains("git_prefix.new_branch"));
    }
}
