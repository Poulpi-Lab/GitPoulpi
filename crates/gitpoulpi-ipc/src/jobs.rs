//! The daemon's job registry as clients see it: `JobList` answers a
//! `Vec<JobInfo>`, `JobCancel` the `JobInfo` it acted on, and
//! `Event::JobChanged` / `Event::JobRemoved` keep a client's copy current.
//!
//! Every network request is a job from the moment it is received (a job
//! waiting for a lock is `Queued`, and cancellable), see
//! the background-ops design`.

use std::path::PathBuf;

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_core::NotCancellableReason;
use serde::{Deserialize, Serialize};

/// The request-argument key that asks the daemon to cancel the job the
/// request starts if the connection that sent it drops before the job ends
/// (a CLI command killed with Ctrl+C). It only says who asked, so it never
/// takes part in the dedupe key; a request that joins a running job does not
/// own it and cancels nothing.
pub const CANCEL_ON_DISCONNECT: &str = "cancel_on_disconnect";

/// Where a job is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobState {
    /// Received, waiting for the repository's transfer or exclusive lock.
    Queued,
    Running,
    /// Cancel was requested; the process tree is being stopped.
    Cancelling,
    Done,
    Failed,
    Canceled,
}

impl JobState {
    /// Over: the job will not change again (only be dropped from the list).
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Done | Self::Failed | Self::Canceled)
    }
}

/// One job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobInfo {
    /// Unique within a daemon, never reused.
    pub id: u64,
    pub kind: OperationKind,
    /// The repository it works on; `None` for a clone (it produces one).
    pub repo: Option<RepoId>,
    /// A clone's destination.
    pub dest: Option<PathBuf>,
    /// A short English description for the CLI and MCP; the TUI words the
    /// job itself from `kind`, `repo` and `dest`.
    pub label: String,
    pub state: JobState,
    /// The part of the work it is in (`fetch`, `integrate`, a clone's
    /// phase), when it reports one.
    pub phase: Option<String>,
    /// 0-100, when the job reports it (a clone does; a fetch does not).
    pub progress: Option<u8>,
    /// Unix seconds when it began running; `None` while queued.
    pub started_at: Option<i64>,
    /// How many later identical requests joined this job instead of
    /// running again.
    pub joiners: u32,
    /// The job that holds the lock this one waits for, while `Queued`.
    pub queued_after: Option<u64>,
    /// Cancel would be refused right now, and why (`None`: it can be
    /// canceled).
    pub not_cancellable: Option<NotCancellableReason>,
    /// The error text of a `Failed` job.
    pub error: Option<String>,
}

impl JobInfo {
    pub fn is_cancellable(&self) -> bool {
        !self.state.is_finished()
            && self.state != JobState::Cancelling
            && self.not_cancellable.is_none()
    }
}
