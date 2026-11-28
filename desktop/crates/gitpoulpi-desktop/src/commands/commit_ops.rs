//! The Log's commit operations as commands: which registry ids the desktop runs as a [`CommitOp`] and how
//! they run. Every key, menu row, palette row and context-menu row of these ends here, so they cannot
//! disagree. The work itself is the Log's (`panels::log::ops`) and the window's (`app::workspace::commit`).

use gitpoulpi_app::keymap::Action;
use gitpoulpi_app::palette::vocab::log_menu::CommitField;
use gitpoulpi_app::palette::PaletteCommand;
use gitpoulpi_core::domain::ResetMode;
use gpui_kit::{App, Window};

use crate::app::workspace_of;

/// What a Log command does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitOp {
    /// Apply the selected commit (every marked commit, oldest first) on top of HEAD.
    CherryPick,
    /// Commit the inverse of the selected commit (asks first).
    Revert,
    /// Move the current branch to the selected commit (`--hard` asks first).
    Reset(ResetMode),
    /// Detach HEAD onto the selected commit.
    Checkout,
    /// Put one field of the selected commit on the clipboard.
    Copy(CommitField),
    /// The context menu of the focused pane (the commit menu in the Log).
    OpenMenu,
    /// Compare the selected rows: the file list of the comparison takes the keyboard.
    CompareSelection,
    /// Trade the two ends of a two-row comparison.
    CompareSwap,
    /// The Log cursor goes to the next compared row.
    CompareNext,
    /// Abort the merge, cherry-pick or revert in progress (asks first).
    Abort,
    /// Skip the cherry-pick or revert in progress (asks first).
    Skip,
}

/// The op of a key action.
pub fn op_for_action(action: Action) -> Option<CommitOp> {
    Some(match action {
        Action::GitCherryPick => CommitOp::CherryPick,
        Action::GitRevert => CommitOp::Revert,
        Action::GitResetMixed => CommitOp::Reset(ResetMode::Mixed),
        Action::GlobalContextMenu => CommitOp::OpenMenu,
        Action::LogCompareSelection => CommitOp::CompareSelection,
        Action::LogCompareSwap => CommitOp::CompareSwap,
        Action::LogCompareNext => CommitOp::CompareNext,
        _ => return None,
    })
}

/// The op of a palette command.
pub fn op_for_palette(command: PaletteCommand) -> Option<CommitOp> {
    use PaletteCommand as C;
    Some(match command {
        C::CherryPickSelectedCommit => CommitOp::CherryPick,
        C::RevertSelectedCommit => CommitOp::Revert,
        C::ResetToSelectedCommit(mode) => CommitOp::Reset(mode),
        C::CheckoutSelectedCommit => CommitOp::Checkout,
        C::CopyCommitField(field) => CommitOp::Copy(field),
        C::CompareSelection => CommitOp::CompareSelection,
        C::CompareSwap => CommitOp::CompareSwap,
        C::CompareGotoNext => CommitOp::CompareNext,
        C::AbortOperation => CommitOp::Abort,
        C::SkipOperation => CommitOp::Skip,
        _ => return None,
    })
}

/// Does what `op` says on the main window.
pub fn run(op: CommitOp, window: &mut Window, cx: &mut App) {
    if let Some(ws) = workspace_of(cx) {
        ws.update(cx, |ws, cx| ws.run_commit_op(op, window, cx));
    }
}
