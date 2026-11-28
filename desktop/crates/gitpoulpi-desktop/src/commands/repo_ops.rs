//! The worktree, submodule, remote and LFS commands: which registry ids the desktop implements as a
//! [`RepoOp`] and how one is run. Following `refs_ops.rs`: the ids map to an op here, `ops.rs` carries one
//! dispatch line per direction and `exec.rs` one line to run it. The work itself is
//! `Workspace::run_repo_op` (`app/workspace/repo.rs`), the same code a menu row, a toolbar button, a click
//! and a key reach. The file commands are in `file_ops.rs`.

use gitpoulpi_app::keymap::Action;
use gitpoulpi_app::palette::{palette_entry, PaletteCommand};
use gpui_kit::{App, Window};

use super::file_ops::FileOp;
use crate::app::workspace_of;
use crate::ref_target::RefTarget;

/// What a worktree, submodule, remote or LFS command does. The target (a Refs row, a clicked file) comes
/// with the command, else it is the Refs sidebar's selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoOp {
    // -- worktrees
    /// The WORKTREES header's `+`: from the current branch, from the selected commit, choosing a branch.
    WorktreeMenu,
    WorktreeFromHead,
    WorktreeChoose,
    /// From the selected branch or tag.
    WorktreeFromRef,
    WorktreeAtCommit,
    /// Open the selected worktree as the window's repository.
    WorktreeOpen,
    /// The selected worktree's "Open in" entries.
    WorktreeOpenInMenu,
    /// Run the `n`th "Open in" entry on the selected worktree's folder.
    WorktreeOpenIn(u8),
    WorktreeRemove,
    // -- submodules
    SubmoduleUpdate,
    SubmoduleInit,
    SubmoduleOpen,
    SubmoduleUpdateAll,
    // -- remotes
    RemoteAdd,
    RemoteEdit,
    RemoteRemove,
    RemoteFetch,
    RemoteCopyName,
    /// The selected remote's whole menu.
    RemoteActions,
    /// The branch's remotes form (its upstream).
    BranchRemotes,
    /// Push, pull or fetch the remote of the selected branch.
    BranchPush,
    BranchPull,
    BranchFetch,
    // -- LFS
    LfsMenu,
    LfsInit,
    LfsTrackPattern,
    LfsUntrackPattern,
    LfsFetch,
    LfsFetchAll,
    LfsPull,
    LfsPush,
    LfsPushAll,
    LfsCheckout,
    LfsPrune,
    LfsShowLocks,
    LfsInstallPage,
    /// `status.lfs_only` / `log.lfs_only`: the working tree lists only the LFS files.
    LfsOnly,
    LfsTrackFile,
    LfsTrackExtension,
    LfsLock,
    LfsUnlock,
    LfsForceUnlock,
    // -- files
    File(FileOp),
}

/// The op of a key action, if the desktop implements it as a repo op.
pub fn for_action(action: Action) -> Option<RepoOp> {
    Some(match action {
        Action::GitLfsMenu => RepoOp::LfsMenu,
        Action::GitLfsFetchAll => RepoOp::LfsFetchAll,
        Action::GitLfsCheckout => RepoOp::LfsCheckout,
        Action::GitLfsPull => RepoOp::LfsPull,
        Action::GitLfsPushAll => RepoOp::LfsPushAll,
        Action::GitLfsPrune => RepoOp::LfsPrune,
        Action::StatusLfsOnly | Action::LogLfsOnly => RepoOp::LfsOnly,
        Action::RefsAddRemote => RepoOp::RemoteAdd,
        Action::RefsRemotes => RepoOp::BranchRemotes,
        // A palette command bound to a key of its own.
        Action::Palette(_) => return palette_entry(action).and_then(|e| for_palette(e.command)),
        _ => return None,
    })
}

/// The op of a palette command, if the desktop implements it as a repo op.
pub fn for_palette(command: PaletteCommand) -> Option<RepoOp> {
    use PaletteCommand as C;
    Some(match command {
        C::CreateWorktreeMenu => RepoOp::WorktreeMenu,
        C::CreateWorktreeFromCurrentBranch => RepoOp::WorktreeFromHead,
        C::CreateWorktreeChooseBranch => RepoOp::WorktreeChoose,
        C::CreateWorktreeFromSelectedRef => RepoOp::WorktreeFromRef,
        C::CreateWorktreeAtSelectedCommit => RepoOp::WorktreeAtCommit,
        C::OpenWorktreeInTab => RepoOp::WorktreeOpen,
        C::WorktreeOpenInSelected => RepoOp::WorktreeOpenInMenu,
        C::WorktreeRemoveSelected => RepoOp::WorktreeRemove,
        C::SubmoduleUpdateSelected => RepoOp::SubmoduleUpdate,
        C::SubmoduleInitSelected => RepoOp::SubmoduleInit,
        C::SubmoduleOpenSelected => RepoOp::SubmoduleOpen,
        C::SubmoduleUpdateAll => RepoOp::SubmoduleUpdateAll,
        C::AddRemote => RepoOp::RemoteAdd,
        C::EditRemote => RepoOp::RemoteEdit,
        C::RemoveRemote => RepoOp::RemoteRemove,
        C::FetchRemote => RepoOp::RemoteFetch,
        C::CopyRemoteName => RepoOp::RemoteCopyName,
        C::RemoteActions => RepoOp::RemoteActions,
        C::LfsMenu => RepoOp::LfsMenu,
        C::FilesToggleLfsOnly => RepoOp::LfsOnly,
        C::LfsInitialize => RepoOp::LfsInit,
        C::LfsTrackPattern => RepoOp::LfsTrackPattern,
        C::LfsUntrackPattern => RepoOp::LfsUntrackPattern,
        C::LfsFetch => RepoOp::LfsFetch,
        C::LfsFetchAll => RepoOp::LfsFetchAll,
        C::LfsPull => RepoOp::LfsPull,
        C::LfsPush => RepoOp::LfsPush,
        C::LfsPushAll => RepoOp::LfsPushAll,
        C::LfsCheckout => RepoOp::LfsCheckout,
        C::LfsPrune => RepoOp::LfsPrune,
        C::LfsShowLocks => RepoOp::LfsShowLocks,
        C::LfsOpenInstallPage => RepoOp::LfsInstallPage,
        C::LfsTrackSelectedFile => RepoOp::LfsTrackFile,
        C::LfsTrackSelectedExtension => RepoOp::LfsTrackExtension,
        C::LfsLockSelectedFile => RepoOp::LfsLock,
        C::LfsUnlockSelectedFile => RepoOp::LfsUnlock,
        C::LfsForceUnlockSelectedFile => RepoOp::LfsForceUnlock,
        C::FileMenu(action) => return super::file_ops::for_file_action(action).map(RepoOp::File),
        _ => return None,
    })
}

/// Runs `op` on the Refs sidebar's selection (or the selected file).
pub fn run(op: RepoOp, window: &mut Window, cx: &mut App) {
    run_on(op, None, window, cx);
}

/// Runs `op` on `target` (what a menu row names), or on the selection when `None`.
pub fn run_on(op: RepoOp, target: Option<RefTarget>, window: &mut Window, cx: &mut App) {
    if let Some(ws) = workspace_of(cx) {
        ws.update(cx, |ws, cx| ws.run_repo_op(op, target, window, cx));
    }
}
