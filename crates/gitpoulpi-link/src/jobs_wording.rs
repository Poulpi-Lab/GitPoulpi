//! How a background job is worded: the daemon sends facts (`kind`, `repo`,
//! `dest`, a state), a frontend words them in its language with these.

use std::path::Path;

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::NotCancellableReason;
use gitpoulpi_i18n::msg::jobs as m;
use gitpoulpi_ipc::{JobInfo, JobState};

/// The short name of what a job does.
pub fn kind(kind: OperationKind) -> &'static str {
    use OperationKind as K;
    match kind {
        K::Fetch | K::FetchAll | K::FetchRefspec | K::FetchRefAsBranch => m::KIND_FETCH.get(),
        K::Push | K::PushForReview => m::KIND_PUSH.get(),
        K::TagPush => m::KIND_PUSH_TAGS.get(),
        K::Pull => m::KIND_PULL.get(),
        K::Clone => m::KIND_CLONE.get(),
        K::LfsFetch
        | K::LfsPull
        | K::LfsPush
        | K::LfsCheckout
        | K::LfsLocks
        | K::LfsLock
        | K::LfsUnlock => m::KIND_LFS.get(),
        _ => m::KIND_REMOTE.get(),
    }
}

fn folder_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
}

/// The repository (or clone destination) folder a job works in.
pub fn name(job: &JobInfo) -> Option<String> {
    job.repo
        .as_ref()
        .and_then(|r| folder_name(&r.0))
        .or_else(|| {
            job.dest
                .as_ref()
                .and_then(|d| folder_name(&d.to_string_lossy()))
        })
}

/// `Push: tool`: what a job does and where. A frontend's own local jobs
/// (ones the daemon never heard of) carry their wording in `label` and skip
/// this.
pub fn label(job: &JobInfo) -> String {
    match name(job) {
        Some(name) => m::label(kind(job.kind), name),
        None => kind(job.kind).to_string(),
    }
}

/// The word for a job's state.
pub fn state(state: JobState) -> &'static str {
    match state {
        JobState::Queued => m::STATE_QUEUED.get(),
        JobState::Running => m::STATE_RUNNING.get(),
        JobState::Cancelling => m::STATE_CANCELLING.get(),
        JobState::Done => m::STATE_DONE.get(),
        JobState::Failed => m::STATE_FAILED.get(),
        JobState::Canceled => m::STATE_CANCELED.get(),
    }
}

/// Why a job cannot be canceled.
pub fn reason(reason: NotCancellableReason) -> &'static str {
    match reason {
        NotCancellableReason::Integrating => m::WHY_INTEGRATING.get(),
        NotCancellableReason::RewritingFiles => m::WHY_REWRITING.get(),
        NotCancellableReason::InSteps => m::WHY_STEPS.get(),
        NotCancellableReason::Sent => m::WHY_SENT.get(),
        NotCancellableReason::Purging => m::WHY_PURGING.get(),
        NotCancellableReason::Finished => m::WHY_FINISHED.get(),
        NotCancellableReason::UnknownJob => m::WHY_UNKNOWN.get(),
    }
}

/// Why a Cancel on this job would do nothing right now, for the disabled
/// button's reason line (`None`: it can be canceled, or it has ended).
pub fn why(job: &JobInfo) -> Option<&'static str> {
    if job.state.is_finished() {
        return None;
    }
    if job.state == JobState::Cancelling {
        return Some(m::WHY_CANCELLING.get());
    }
    job.not_cancellable.map(reason)
}

/// The status line for a Cancel a frontend will not even send.
pub fn refusal(job: &JobInfo) -> Option<String> {
    if job.state.is_finished() {
        return Some(m::not_cancellable(m::WHY_FINISHED.get()));
    }
    why(job).map(m::not_cancellable)
}
