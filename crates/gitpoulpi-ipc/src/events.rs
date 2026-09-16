use std::path::PathBuf;

use crate::scope::ChangeScope;
use gitpoulpi_core::domain::{
    CloneProgress, JournalState, LfsProgress, Operation, OperationKind, RepoId,
};
use serde::{Deserialize, Serialize};

/// Pushed by the daemon to every subscribed client so a change made via one
/// frontend (e.g. an MCP-driven agent commit) renders live in another
/// (e.g. an already-open TUI) without restarting anything (§1.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    RepoChanged {
        repo: RepoId,
        /// What the operation can have changed; absent on the wire from an
        /// older daemon, which means everything.
        #[serde(default)]
        scope: ChangeScope,
    },
    OperationCompleted {
        repo: RepoId,
        operation: Operation,
    },
    ConflictDetected {
        repo: RepoId,
        paths: Vec<String>,
    },
    /// The repository's hidden/solo ref filter was changed through the
    /// daemon (`OperationKind::SetRefVisibility`, e.g. `gitpoulpi refs
    /// hide`). Re-read `repo-state.toml`'s `ref_visibility` to follow.
    RefVisibilityChanged {
        repo: RepoId,
    },
    /// The repository's pending auto-stash record (`repo-state.toml`'s
    /// `pending_auto_stash`) was written, changed or cleared by an
    /// auto-stash. Re-read it to follow.
    PendingAutoStashChanged {
        repo: RepoId,
    },
    /// The pending auto-stash record was cleared because its stash is no
    /// longer in the stash list (dropped or popped outside GitPoulpi).
    /// Sent next to [`Event::PendingAutoStashChanged`] so a UI can say so.
    /// `stash` is the index the record last had.
    PendingAutoStashGone {
        repo: RepoId,
        stash: gitpoulpi_core::domain::StashId,
    },
    /// The undo journal changed: a journaled operation was recorded, or
    /// an undo/redo ran. Carries the new top entries so every frontend
    /// can light its buttons without polling.
    JournalChanged {
        repo: RepoId,
        state: JournalState,
    },
    /// A clone is in flight. Keyed by its *destination*, because there is
    /// no `RepoId` until it finishes.
    ///
    /// Broadcast like every other event (ADR-0006); a client that did not
    /// ask for this clone matches on `dest` and ignores it.
    CloneProgress {
        url: String,
        dest: PathBuf,
        progress: CloneProgress,
    },
    /// A Git LFS fetch, pull or push is in flight. Broadcast like
    /// [`Event::CloneProgress`]; `operation` says which (`LfsFetch`,
    /// `LfsPull` or `LfsPush`). The operation's own reply, followed by a
    /// `RepoChanged`, is what says it ended; the last progress report has
    /// `done: true`.
    LfsProgress {
        repo: RepoId,
        operation: OperationKind,
        progress: LfsProgress,
    },
    /// Before a pull or push, the daemon installed the missing
    /// repository-local LFS filters / pre-push hook of `repo` (see
    /// `git.lfs_autofix_filters`). Frontends say so on their status line.
    LfsFiltersPosed {
        repo: RepoId,
    },
    /// A clone finished. Carries the `RepoRef` the daemon will accept
    /// for it from now on, so a client can open the new repository as a
    /// tab without a round trip to `Discover`.
    CloneFinished {
        url: String,
        dest: PathBuf,
        result: std::result::Result<RepoId, String>,
    },
    /// The daemon received `OperationKind::Shutdown` and is about to exit.
    /// Broadcast to every subscriber, so a TUI on this session can show
    /// "daemon stopped" instead of hanging on its next request; every
    /// connection is about to be dropped.
    DaemonStopping {
        session: String,
    },
    /// A history rewrite preview is running: the phase it is in and how far
    /// it got (`done` of `total`, `total` 0 while unknown). Throttled to
    /// about 10 per second. The preview's job (`JobChanged`) carries the same
    /// percentage for the jobs indicator.
    RewriteProgress {
        repo: RepoId,
        job: u64,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgress,
    },
    /// A job was created or changed (state, phase, progress, joiners).
    /// Broadcast to every client; throttled to about 10 per second per job
    /// while only the progress moves. A client keeps a map of them.
    JobChanged {
        job: crate::jobs::JobInfo,
    },
    /// A finished job was dropped from the daemon's list (finished jobs
    /// stay listed for 30 seconds).
    JobRemoved {
        id: u64,
    },
    /// An agent asks this window to move its view (`gitpoulpi ui show ...`,
    /// the MCP `ui_*` tools). **Never broadcast**: the daemon sends it only
    /// to the connection of the chosen viewer (`viewer`), because a view
    /// change is addressed to one window and is not repository state.
    /// Answer with `OperationKind::UiReport` carrying `intent_id`.
    UiIntent {
        viewer: crate::ui::ViewerId,
        intent_id: u64,
        intent: crate::ui::UiIntent,
        origin: crate::ui::UiOrigin,
    },
    /// An agent asks this window for its state (`gitpoulpi ui state`).
    /// Like [`Event::UiIntent`], sent to one connection only. Answer with
    /// `UiReport` (a `State` body) carrying `query_id`.
    UiStateRequest {
        viewer: crate::ui::ViewerId,
        query_id: u64,
        include_text: bool,
    },
    /// A launcher click (`gitpoulpi launch`) asks this window to bring its
    /// terminal window to the front. Sent to one connection only, answered
    /// with `UiReport` (an `Outcome` body: `Applied` when it did what it
    /// could, `Refused` with the reason when nothing worked) carrying
    /// `raise_id`. Not an agent intent: it moves no view.
    RaiseWindow {
        viewer: crate::ui::ViewerId,
        raise_id: u64,
    },
}
