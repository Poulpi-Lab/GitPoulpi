//! Host-driven flows (slice W4): what the host can make from an issue and
//! what it already made for one. The types are provider-agnostic; the
//! providers fill them (`gitlab_flow.rs`, `github_flow.rs`, `gitea/flow.rs`).

use crate::model::PullRequestSummary;

/// A branch to create on the host for an issue.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewIssueBranch {
    /// `owner/name`, `group/sub/proj`.
    pub project: String,
    /// The issue the branch is made for (GitHub links it to the issue).
    pub issue_number: u64,
    /// The new branch's name.
    pub name: String,
    /// The existing branch (name, not a remote-tracking spelling) it starts
    /// from.
    pub from: String,
}

/// What the host answers once the branch exists.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct CreatedBranch {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
}

/// What the host links to an issue itself (not by a naming convention).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IssueLinks {
    /// Branches the host links to the issue (GitHub's linked branches).
    pub branches: Vec<String>,
    /// Merge / pull requests that mention or close the issue.
    pub requests: Vec<PullRequestSummary>,
}
