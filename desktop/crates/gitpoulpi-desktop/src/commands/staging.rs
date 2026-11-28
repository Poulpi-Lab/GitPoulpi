//! Running the staging and commit ops: each one is a method of the working-changes pane or of the diff
//! pane, so a key, a menu item, a palette row, a toolbar button and a click all end in the same code.

use gpui_kit::{Context, Window};

use super::ops::{DiffPartOp, StatusOp};
use crate::daemon::staging::PartOp;
use crate::panels::diff::DiffPane;
use crate::panels::right::RightPanel;

/// Does what `op` says on the working-changes pane.
pub fn run_status(
    right: &mut RightPanel,
    op: StatusOp,
    window: &mut Window,
    cx: &mut Context<RightPanel>,
) {
    match op {
        StatusOp::StageSelected => right.stage_selected(cx),
        StatusOp::UnstageSelected => right.unstage_selected(cx),
        StatusOp::StageAll => right.stage_all(cx),
        StatusOp::DiscardAll => right.discard_all(window, cx),
        StatusOp::DiscardSelected => right.discard_selected(window, cx),
        StatusOp::ToggleStage => right.toggle_stage_row(cx),
        StatusOp::OpenDiff => right.open_selected_diff(cx),
        StatusOp::TogglePathTree => right.toggle_tree(cx),
        StatusOp::ToggleAmend => right.toggle_amend(cx),
        StatusOp::ToggleSkipHooks => right.toggle_skip_hooks(cx),
        StatusOp::FocusMessage => right.focus_summary(window, cx),
        StatusOp::Commit => right.commit(window, cx),
        StatusOp::CommitNow => right.commit_now(window, cx),
        StatusOp::SelectAll => right.select_all(cx),
        StatusOp::ClearSelection => right.clear_selection(cx),
        StatusOp::ToggleSelection => right.toggle_selection(cx),
        StatusOp::FocusSection(section) => right.focus_section(section, window, cx),
        StatusOp::Fold(fold) => right.fold(fold, cx),
        StatusOp::ToggleFoldAll => right.toggle_fold_all(cx),
    }
}

/// Does what `op` says on the open diff.
pub fn run_part(diff: &mut DiffPane, op: DiffPartOp, cx: &mut Context<DiffPane>) {
    match op {
        DiffPartOp::ToggleHunk => diff.request_toggle_hunk(cx),
        DiffPartOp::ToggleLines => diff.request_toggle_lines(cx),
        DiffPartOp::StageHunk => diff.request_hunk(PartOp::Stage, cx),
        DiffPartOp::UnstageHunk => diff.request_hunk(PartOp::Unstage, cx),
        DiffPartOp::StageLines => diff.request_lines(PartOp::Stage, cx),
        DiffPartOp::UnstageLines => diff.request_lines(PartOp::Unstage, cx),
        DiffPartOp::DiscardHunk => diff.request_hunk(PartOp::Discard, cx),
        DiffPartOp::DiscardLines => diff.request_lines(PartOp::Discard, cx),
        DiffPartOp::SelectLines => diff.toggle_selection(cx),
    }
}
