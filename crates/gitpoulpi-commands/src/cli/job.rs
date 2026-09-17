//! `gitpoulpi job ...`: the daemon's background jobs. Every network
//! operation (fetch, push, pull, clone, LFS transfers, workflows) is a job
//! from the moment the daemon receives it, whichever client sent it, so a
//! script or an agent can see what is running, stop it, or wait for it.

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum JobAction {
    /// The jobs of this session's daemon, oldest first: running, queued
    /// behind another, and the ones that ended in the last 30 seconds.
    List,
    /// Stop a job. Refused (exit code 1, nothing changes) once it is past
    /// the point where stopping is safe, for instance a pull that has begun
    /// merging. A canceled push may already have reached the remote.
    Cancel {
        /// The job's id, from `job list`.
        id: u64,
    },
    /// Block until a job ends and print its final state. Exit code 0 when it
    /// is `Done`, 1 when it failed or was canceled.
    Wait {
        /// The job's id, from `job list`.
        id: u64,
        /// Give up after this many seconds (exit code 1, the job keeps
        /// running). Omitted: wait as long as it takes.
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,
    },
}
