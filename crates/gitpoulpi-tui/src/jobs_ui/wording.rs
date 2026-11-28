//! How a job is worded: the daemon sends facts (`kind`, `repo`, `dest`), the
//! window words them in its language. The wording itself is
//! `gitpoulpi_link::jobs_wording`; what stays here is the window's own jobs.

use gitpoulpi_ipc::JobInfo;

pub(crate) use gitpoulpi_link::jobs_wording::reason;
pub(super) use gitpoulpi_link::jobs_wording::{refusal, state, why};

/// `Push: tool`.
pub(crate) fn label(job: &JobInfo) -> String {
    if super::is_local(job.id) {
        return job.label.clone();
    }
    gitpoulpi_link::jobs_wording::label(job)
}

/// The label of job `id`, or `#id` when the window never heard of it.
pub(crate) fn label_of_id(id: u64) -> String {
    super::find(id).map_or_else(|| format!("#{id}"), |j| label(&j))
}
