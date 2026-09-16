//! The reflog, read (the undo-safety-net design): where a ref
//! was before, including moves made outside GitPoulpi, and the commits no
//! ref reaches any more.

use serde::{Deserialize, Serialize};

use super::Oid;

/// The all-zero object id git writes for "no object" (`old` of a ref's
/// first reflog entry).
pub const ZERO_OID: Oid = Oid([0; 20]);

/// One line of a reflog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflogEntry {
    /// The value before the move ([`ZERO_OID`] for the first entry).
    pub old: Oid,
    /// The value after the move: the position "restore to" returns to.
    pub new: Oid,
    /// `Name <email>` of whoever moved the ref.
    pub committer: String,
    /// Unix seconds.
    pub time: i64,
    /// Git's reason: `commit: fix typo`, `checkout: moving from a to b`,
    /// `reset: moving to HEAD~1`, ...
    pub message: String,
}

/// A page of a reflog, newest entry first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflogPage {
    /// The full ref name read (`HEAD`, `refs/heads/main`).
    pub refname: String,
    pub entries: Vec<ReflogEntry>,
    /// More entries exist past this page.
    pub more: bool,
}

/// Full reflog name for what a user typed: `HEAD`, `refs/...` as is, any
/// other name is a local branch.
pub fn reflog_refname(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() || name == "HEAD" {
        "HEAD".to_string()
    } else if name.starts_with("refs/") {
        name.to_string()
    } else {
        format!("refs/heads/{name}")
    }
}

/// Where in a reflog a ref held a given value: what a blocked undo offers
/// ("restore main to where Commit left it").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflogPosition {
    /// Full ref name.
    pub refname: String,
    /// The value to restore the ref to.
    pub oid: Oid,
    /// 0 is the newest entry.
    pub index: usize,
    pub time: i64,
    pub message: String,
}

/// Why a lost commit is still around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LostSource {
    /// A reflog (`HEAD`'s, or a branch's) still names it.
    Reflog,
    /// Only a GitPoulpi undo pin keeps it (an old or expired journal entry).
    Pin,
    /// Nothing names it: a dangling object, as `git fsck --lost-found` finds.
    Dangling,
}

/// A commit no branch, tag, remote-tracking ref or stash reaches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LostCommit {
    pub oid: Oid,
    pub summary: String,
    pub author: String,
    /// Author date, unix seconds.
    pub time: i64,
    pub parents: Vec<Oid>,
    pub source: LostSource,
}

/// What `GitBackend::lost_commits` scans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LostCommitOptions {
    pub skip: usize,
    pub limit: usize,
    /// Also look for dangling objects in the whole object database (the
    /// part `git fsck --unreachable` would find). Bounded by `budget_ms`.
    pub deep: bool,
    /// The scan stops after this long (0: no limit) and says so in
    /// `LostCommitsPage::partial`.
    pub budget_ms: u64,
}

impl Default for LostCommitOptions {
    fn default() -> Self {
        Self {
            skip: 0,
            limit: 50,
            deep: true,
            budget_ms: 8000,
        }
    }
}

/// A page of lost commits, newest first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LostCommitsPage {
    pub commits: Vec<LostCommit>,
    pub more: bool,
    /// The time budget ran out before the scan finished: the list is
    /// correct but may be missing older commits.
    pub partial: bool,
}
