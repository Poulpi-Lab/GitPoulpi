//! Running a grouped push or pull: the request (decisions per target, as a
//! plan suggested them or the user changed them), the per-target results,
//! the saved state of a run that stopped, and the outcome.
//!
//! Pure data, no I/O. The engine lives in the daemon
//! (`remote_group/exec.rs`); the CLI and the TUI read these types.
//!
//! A run integrates first (every fast-forward, merge or rebase the
//! decisions ask for, in target order) and then pushes to the targets one
//! at a time, primary first. Every target ends with its own
//! [`TargetStatus`]; there is no atomicity across remotes, and the
//! outcome says so.

use serde::{Deserialize, Serialize};

use super::remote_group::RemoteRelation;
use super::remote_group_plan::{GroupAction, GroupMode, PlannedTarget};
use super::{Oid, PushRejectReason};
use crate::failure::FailureKind;

/// The version written into the saved state.
pub const REMOTE_GROUP_STATE_VERSION: u32 = 1;

/// What the caller decided for one target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTarget {
    pub remote: String,
    /// The branch name on that remote.
    pub branch: String,
    pub action: GroupAction,
    /// What the remote held when the plan looked: the oid a
    /// `ForceWithLease` push expects (never the bare lease).
    #[serde(default)]
    pub lease: Option<Oid>,
    /// The plan saw no such branch on the remote: it is created with the
    /// `Absent` lease, so a branch someone created meanwhile is never
    /// overwritten.
    #[serde(default)]
    pub absent: bool,
}

impl RunTarget {
    /// The decision for `planned`, taking `action` (usually its
    /// `suggested` one) and the lease the plan saw.
    pub fn from_planned(planned: &PlannedTarget, action: GroupAction) -> RunTarget {
        RunTarget {
            remote: planned.remote.clone(),
            branch: planned.branch.clone(),
            action,
            lease: planned.lease,
            absent: planned.relation == RemoteRelation::Absent,
        }
    }
}

/// Continue or abort the saved run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupResume {
    /// Never automatic: the caller asks for it, once the conflict is
    /// resolved and concluded.
    Continue,
    /// Aborts the merge or rebase the run started and drops the saved run.
    /// Pushes already done stay.
    Abort,
}

/// Arguments of `RemoteGroupRun`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGroupRunRequest {
    /// The local branch. Not needed to resume or retry.
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub mode: GroupMode,
    /// The decisions, in run order (the plan's order: primary first). On
    /// `resume: continue` they replace the saved decision of the same
    /// remote and branch (how a `needs_resolution` answer is answered).
    #[serde(default)]
    pub targets: Vec<RunTarget>,
    /// Also push the unpushed tags to every target that took the branch.
    #[serde(default)]
    pub tags: bool,
    #[serde(default)]
    pub resume: Option<GroupResume>,
    /// With `resume: continue` (or alone, which means continue): also try
    /// the targets that failed again, not only the ones never attempted or
    /// left unknown.
    #[serde(default)]
    pub retry: bool,
    /// The saved run the caller means (a second frontend may have
    /// replaced it).
    #[serde(default)]
    pub expect_run_id: Option<String>,
    /// `git.pull_mode` is rebase. Pull mode only: a fast-forward that an
    /// earlier integration of the run made impossible is then a rebase
    /// instead of a merge (extra sources follow the primary's pull mode).
    #[serde(default)]
    pub prefer_rebase: bool,
}

/// How a target's integration went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Integration {
    FastForwarded,
    Merged,
    Rebased,
    UpToDate,
}

/// Where one target stands. `Integrating` and `Pushing` are transient
/// (progress events); a result never ends in them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetStatus {
    /// Not started; also what a run that was stopped leaves behind.
    NotAttempted,
    Integrating,
    Pushing,
    /// Pull mode: what the remote had was taken in (see `integrated`).
    Integrated,
    Pushed {
        /// The branch did not exist on the remote.
        created: bool,
    },
    UpToDate,
    Skipped,
    Rejected {
        reason: PushRejectReason,
    },
    Failed {
        kind: FailureKind,
    },
    /// The remote is not configured (renamed or removed with plain git).
    RemoteMissing,
    /// Canceled in flight: the push may or may not have arrived.
    Unknown,
}

impl TargetStatus {
    /// What a retry runs again: never attempted, unknown, and with
    /// `failed` the failed ones.
    pub fn is_retryable(&self, failed: bool) -> bool {
        match self {
            TargetStatus::NotAttempted
            | TargetStatus::Unknown
            | TargetStatus::Integrating
            | TargetStatus::Pushing => true,
            TargetStatus::Failed { .. } => failed,
            _ => false,
        }
    }

    /// Finished well: nothing left to do for this target.
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            TargetStatus::Integrated
                | TargetStatus::Pushed { .. }
                | TargetStatus::UpToDate
                | TargetStatus::Skipped
        )
    }
}

/// One target's result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetResult {
    pub remote: String,
    pub branch: String,
    pub action: GroupAction,
    pub status: TargetStatus,
    /// What the integration did, when the decision had one.
    #[serde(default)]
    pub integrated: Option<Integration>,
    /// The failure's or rejection's own words, a twin's note, ...
    #[serde(default)]
    pub detail: Option<String>,
    /// A force push replaced this commit on the remote.
    #[serde(default)]
    pub replaced: Option<Oid>,
    /// The command that puts `replaced` back.
    #[serde(default)]
    pub recovery: Option<String>,
    /// The tag push, when asked for: what it sent or why it failed.
    #[serde(default)]
    pub tags: Option<String>,
}

/// A target in the saved state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTarget {
    pub decision: RunTarget,
    /// The integration this decision asks for is done.
    #[serde(default)]
    pub integrated: bool,
    pub result: TargetResult,
}

/// What a merge or rebase the run started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoppedOp {
    Merge,
    Rebase,
}

/// The integration a run is stopped in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoppedAt {
    pub remote: String,
    pub branch: String,
    pub op: StoppedOp,
    /// The local tip before the integration started.
    pub pre_tip: Oid,
    /// The remote's tip being integrated: the finished resolution must
    /// contain it.
    pub remote_tip: Oid,
}

/// How a run ended (or where it stopped).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRunStatus {
    /// Every target is pushed, up to date, integrated or skipped.
    Done,
    /// Some target failed, was rejected, is unknown or was not attempted.
    Problems,
    /// A merge or rebase stopped on conflicts: resolve, conclude, then ask
    /// to continue (or abort).
    Conflicted,
    /// A continue found that a push now needs a decision (a rebase left a
    /// target holding the old commits, so it needs a force): the answer
    /// carries a fresh plan; nothing was pushed.
    NeedsResolution,
    /// Canceled; the in-flight target is `Unknown`, the rest
    /// `NotAttempted`.
    Canceled,
    /// The saved run was aborted.
    Aborted,
}

/// Answer of `RemoteGroupRun`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteGroupOutcome {
    pub run_id: String,
    pub branch: String,
    pub mode: GroupMode,
    pub status: GroupRunStatus,
    /// The local branch's tip when the run ended.
    #[serde(default)]
    pub local_tip: Option<Oid>,
    pub targets: Vec<TargetResult>,
    /// A saved run waits (stopped, canceled or with failures to retry).
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub stopped: Option<StoppedAt>,
    /// The unmerged paths of a stop on conflicts.
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// `NeedsResolution`: the plan to decide from.
    #[serde(default)]
    pub plan: Option<super::RemoteGroupPlan>,
    pub message: String,
}

/// The saved run as a read shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingGroup {
    pub run_id: String,
    pub branch: String,
    pub mode: GroupMode,
    pub started_tip: Oid,
    pub targets: Vec<TargetResult>,
    #[serde(default)]
    pub stopped: Option<StoppedAt>,
    /// Computed on every read, never acted on: nothing in progress,
    /// nothing unmerged, the branch checked out and its tip containing the
    /// stopped remote's tip. Only turns a Continue button on.
    pub ready_to_continue: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_statuses() {
        assert!(TargetStatus::NotAttempted.is_retryable(false));
        assert!(TargetStatus::Unknown.is_retryable(false));
        assert!(!TargetStatus::Failed {
            kind: FailureKind::Other
        }
        .is_retryable(false));
        assert!(TargetStatus::Failed {
            kind: FailureKind::Other
        }
        .is_retryable(true));
        assert!(!TargetStatus::Pushed { created: false }.is_retryable(true));
        assert!(!TargetStatus::Skipped.is_retryable(true));
    }

    #[test]
    fn a_request_needs_only_its_decisions() {
        let r: RemoteGroupRunRequest = serde_json::from_str(r#"{"branch":"b"}"#).unwrap();
        assert!(r.targets.is_empty() && r.resume.is_none() && !r.retry && !r.tags);
        let r: RemoteGroupRunRequest =
            serde_json::from_str(r#"{"resume":"continue","expect_run_id":"x"}"#).unwrap();
        assert_eq!(r.resume, Some(GroupResume::Continue));
    }
}
