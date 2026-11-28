//! The branch, tag and stash commands: which registry ids the desktop implements as a [`RefOp`] and how
//! one is run. Following `staging.rs`: the ids map to an op here, `ops.rs` carries one dispatch line per
//! direction, and `exec.rs` one line to run it. The work itself is `Workspace::run_ref_op`
//! (`app/workspace/refs.rs`), the same code a menu row, a toolbar button, a click and a key reach.

use gitpoulpi_app::keymap::Action;
use gitpoulpi_app::palette::vocab::ref_menu::RefField;
use gitpoulpi_app::palette::{palette_entry, PaletteCommand, StashTarget};
use gpui_kit::{App, Window};

use crate::app::workspace_of;
use crate::ref_target::RefTarget;

/// What a ref command does. The target is the Refs sidebar's selection (else the stash or branch the
/// command names), unless a menu hands one over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefOp {
    /// New branch at HEAD, or at the Log's selected commit.
    NewBranch,
    NewBranchAtCommit,
    NewTag,
    NewTagAtCommit,
    NewAnnotatedTagAtCommit,
    /// Check out the selected ref (a remote branch through its local twin).
    Checkout,
    /// Detach HEAD onto the Log's selected commit.
    CheckoutCommit,
    /// Merge the selected branch into the checked-out one.
    Merge,
    /// Rebase the checked-out branch onto the selected one.
    Rebase,
    Rename,
    SetUpstream,
    FastForwardToCurrent,
    FastForwardCurrentToThis,
    /// `d`: delete the selected branch or tag, drop the selected stash.
    DeleteSelected,
    DeleteRemoteBranch,
    DeleteBranchAndRemote,
    DeleteTagFromRemote,
    PushTag,
    PushAllTags,
    AnnotateTag,
    Copy(RefField),
    StashPush,
    StashMessage,
    /// The selected stash when the Stashes section has the selection, else the latest.
    StashPop,
    StashApply,
    StashDrop,
    /// Show the stash's changes in the diff pane.
    StashShow,
    /// Show the ref's commit in the Log.
    Reveal,
    ToggleHide,
    ToggleSolo,
    /// The keyboard twin of dragging the selected ref onto the checked-out branch.
    DropOnCurrent,
    /// Open the context menu of the selected row (the menu key, Shift+F10).
    OpenMenu,
}

/// The op of a key action, if the desktop implements it as a ref op.
pub fn for_action(action: Action) -> Option<RefOp> {
    Some(match action {
        Action::GitNewBranch => RefOp::NewBranch,
        Action::GitNewTag => RefOp::NewTag,
        Action::GitStashPush => RefOp::StashPush,
        Action::GitStashMessage => RefOp::StashMessage,
        Action::GitStashPop => RefOp::StashPop,
        Action::GitDeleteRef => RefOp::DeleteSelected,
        Action::GitRebaseOnto => RefOp::Rebase,
        Action::GitPushTags => RefOp::PushAllTags,
        Action::RefsReveal => RefOp::Reveal,
        Action::GlobalContextMenu => RefOp::OpenMenu,
        // A palette command bound to a key of its own.
        Action::Palette(_) => return palette_entry(action).and_then(|e| for_palette(e.command)),
        _ => return None,
    })
}

/// The op of a palette command, if the desktop implements it as a ref op.
pub fn for_palette(command: PaletteCommand) -> Option<RefOp> {
    use PaletteCommand as C;
    Some(match command {
        C::NewBranch => RefOp::NewBranch,
        C::NewTag => RefOp::NewTag,
        C::CreateBranchAtSelectedCommit => RefOp::NewBranchAtCommit,
        C::CreateTagAtSelectedCommit => RefOp::NewTagAtCommit,
        C::CreateAnnotatedTagAtSelectedCommit => RefOp::NewAnnotatedTagAtCommit,
        C::CheckoutSelectedRef => RefOp::Checkout,
        C::CheckoutSelectedCommit => RefOp::CheckoutCommit,
        C::MergeSelectedRef => RefOp::Merge,
        C::RebaseOntoSelectedRef => RefOp::Rebase,
        C::RenameSelectedBranch => RefOp::Rename,
        C::SetUpstream => RefOp::SetUpstream,
        C::FastForwardSelectedToCurrent => RefOp::FastForwardToCurrent,
        C::FastForwardCurrentToSelected => RefOp::FastForwardCurrentToThis,
        C::DeleteSelectedRef => RefOp::DeleteSelected,
        C::DeleteSelectedRemoteBranch => RefOp::DeleteRemoteBranch,
        C::DeleteSelectedBranchAndRemote => RefOp::DeleteBranchAndRemote,
        C::DeleteSelectedTagFromRemote => RefOp::DeleteTagFromRemote,
        C::PushSelectedTag => RefOp::PushTag,
        C::PushAllUnpushedTags => RefOp::PushAllTags,
        C::RevealSelectedRefInGraph => RefOp::Reveal,
        C::AnnotateSelectedTag => RefOp::AnnotateTag,
        C::CopyRefField(f) => RefOp::Copy(f),
        C::StashPush => RefOp::StashPush,
        C::StashPushWithMessage => RefOp::StashMessage,
        C::StashPop(StashTarget::Selected | StashTarget::LatestOrLogRow | StashTarget::Latest) => {
            RefOp::StashPop
        }
        C::StashApply => RefOp::StashApply,
        C::StashDrop => RefOp::StashDrop,
        C::DropSelectedRefOnCurrent => RefOp::DropOnCurrent,
        _ => return None,
    })
}

/// Runs `op` on the Refs sidebar's selection.
pub fn run(op: RefOp, window: &mut Window, cx: &mut App) {
    run_on(op, None, window, cx);
}

/// Runs `op` on `target` (what a menu row or a drop names), or on the selection when `None`.
pub fn run_on(op: RefOp, target: Option<RefTarget>, window: &mut Window, cx: &mut App) {
    if let Some(ws) = workspace_of(cx) {
        ws.update(cx, |ws, cx| ws.run_ref_op(op, target, window, cx));
    }
}
