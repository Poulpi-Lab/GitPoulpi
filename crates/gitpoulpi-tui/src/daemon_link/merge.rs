//! The merge conflict editor's round trips: opening a file (`ConflictShow`),
//! saving it (`ResolveConflict` with the stale guard), re-opening a
//! resolved one (`ConflictUnresolve`). They have a channel of their own, so the typed
//! refusal (`ConflictRefusal::Stale` ...) reaches the editor instead of
//! becoming a status-line string.

use super::*;
use crate::redraw::NotifySend as _;
use gitpoulpi_core::domain::{MergeDocument, ResolveOptions, WorktreeFileState};

/// What the editor asks the link for.
#[derive(Debug, Clone, PartialEq)]
pub enum MergeRequest {
    /// Read the three stages and git's own merge of them.
    Show { path: PathBuf },
    /// Write the resolution; `stage` also marks the path resolved.
    /// `expected` is the stage oids the editor opened on.
    Resolve {
        path: PathBuf,
        resolution: ConflictResolution,
        stage: bool,
        expected: [Option<Oid>; 3],
    },
    /// Put a resolved path's conflict back (the daemon remembers the
    /// stages it resolved).
    Unresolve { path: PathBuf },
}

impl MergeRequest {
    /// Whether the request changes the repository (the in-flight tracking
    /// and the refresh that follows).
    fn mutates(&self) -> bool {
        !matches!(self, MergeRequest::Show { .. })
    }

    fn label(&self) -> String {
        use gitpoulpi_i18n::msg::merge as tr;
        match self {
            MergeRequest::Show { path } => tr::msg_opening(path.display()),
            MergeRequest::Resolve { path, .. } => tr::msg_saving(path.display()),
            MergeRequest::Unresolve { path } => tr::msg_unresolving(path.display()),
        }
    }
}

/// The answer to one [`MergeRequest`].
pub enum MergeReply {
    Opened {
        path: PathBuf,
        result: GitResult<Box<MergeDocument>>,
        /// The working-tree file's text, read when it was edited outside
        /// (no markers, differs from git's merge): the editor offers to
        /// continue from it.
        disk: Option<String>,
    },
    Saved {
        path: PathBuf,
        stage: bool,
        result: GitResult<()>,
    },
    Unresolved {
        path: PathBuf,
        result: GitResult<()>,
    },
}

/// Runs one request against the daemon.
pub(super) async fn run_merge(conn: &LinkConn, request: MergeRequest) -> MergeReply {
    match request {
        MergeRequest::Show { path } => {
            let result = conn
                .call::<MergeDocument>(
                    OperationKind::ConflictShow,
                    serde_json::json!({ "path": path }),
                )
                .await
                .map(Box::new);
            let edited_outside = matches!(
                result.as_ref().map(|d| &d.worktree),
                Ok(WorktreeFileState::EditedNoMarkers { .. })
            );
            let disk = if edited_outside {
                let abs = conn.repo.root.join(&path);
                std::fs::read(&abs).ok().map(|bytes| {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    text.strip_prefix('\u{feff}')
                        .map(str::to_string)
                        .unwrap_or(text)
                })
            } else {
                None
            };
            MergeReply::Opened { path, result, disk }
        }
        MergeRequest::Resolve {
            path,
            resolution,
            stage,
            expected,
        } => {
            let opts = ResolveOptions {
                stage,
                expected: Some(expected),
            };
            let result = conn
                .call::<()>(
                    OperationKind::ResolveConflict,
                    serde_json::json!({
                        "path": path,
                        "resolution": resolution,
                        "opts": opts,
                    }),
                )
                .await;
            MergeReply::Saved {
                path,
                stage,
                result,
            }
        }
        MergeRequest::Unresolve { path } => {
            let result = conn
                .call::<()>(
                    OperationKind::ConflictUnresolve,
                    serde_json::json!({ "path": path }),
                )
                .await;
            MergeReply::Unresolved { path, result }
        }
    }
}

/// What the link loop does for one request: tracked as an operation when it
/// changes the repository, so the status line's in-flight text, the
/// generation counter and the refresh that follows work as for any action.
pub(super) fn spawn_merge(
    conn: &LinkConn,
    activity: &Arc<crate::activity::Activity>,
    out_tx: &std_mpsc::Sender<MergeReply>,
    action_done_tx: &tokio::sync::mpsc::UnboundedSender<(u64, bool)>,
    request: MergeRequest,
) {
    let op = request
        .mutates()
        .then(|| activity.begin(request.label(), true));
    let conn = conn.clone();
    let out_tx = out_tx.clone();
    let action_done_tx = action_done_tx.clone();
    tokio::spawn(async move {
        let reply = run_merge(&conn, request).await;
        let ok = match &reply {
            MergeReply::Saved { result, .. } | MergeReply::Unresolved { result, .. } => {
                result.is_ok()
            }
            MergeReply::Opened { result, .. } => result.is_ok(),
        };
        if let Some(op) = op {
            let _ = action_done_tx.send((op, ok));
        }
        let _ = out_tx.send_notify(reply);
    });
}

impl DaemonLink {
    /// Asks for one editor round trip; the answer arrives on `merge_rx`.
    pub fn request_merge(&self, request: MergeRequest) {
        let _ = self.merge_tx.send(request);
    }
}
