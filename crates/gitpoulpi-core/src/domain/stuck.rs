//! States a repository can be left in that only plain git used to clear:
//! a `git am` stopped on a patch, and a stale `.git/index.lock`
//! (plan `stuck-states.md`, slice S8).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How old an `index.lock` must be, in seconds, before it counts as stale.
/// Every git command that takes the lock holds it for milliseconds to a few
/// seconds (a huge `git add` on a slow disk is the long case), so ten
/// seconds separates "a git command is running right now" from "one died
/// and left its lock" without ever asking the user to wait long. The check
/// is age only: whether a git process is alive is not reliably knowable on
/// every OS, so the user confirms before the file is removed.
pub const STALE_INDEX_LOCK_SECS: u64 = 10;

/// The `index.lock` of a repository (of the linked worktree's own git
/// directory in a linked worktree), see [`crate::GitBackend::index_lock`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexLock {
    /// The lock file.
    pub path: PathBuf,
    /// Seconds since it was last written.
    pub age_secs: u64,
    /// `age_secs >= STALE_INDEX_LOCK_SECS`: old enough to offer removing it.
    pub stale: bool,
}

/// What `git am --continue` / `--skip` left behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmOutcome {
    /// Every patch applied (or was skipped): no `am` is in progress.
    Completed,
    /// Another patch stopped it: the `am` is still in progress.
    Stopped,
}
