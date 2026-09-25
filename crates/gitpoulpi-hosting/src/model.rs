//! Host-agnostic shapes for the account, pull/merge request and issue
//! queries. Normalised here, once, so the TUI/CLI/MCP never see whether a
//! row came from GitHub or GitLab.

use serde::{Deserialize, Serialize};

/// Who a token belongs to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedUser {
    /// The handle (`octocat`, GitLab's `username`).
    pub login: String,
    /// Display name; empty when the account has none.
    pub name: String,
    /// The host's own numeric id, as a string.
    pub id: String,
    pub web_url: Option<String>,
}

/// The outcome of checking a token against its host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenCheck {
    pub user: HostedUser,
    /// The scopes the host reports. `None` means *unknown* (a GitHub
    /// fine-grained token, an old GitLab without
    /// `/personal_access_tokens/self`), which is deliberately not the same
    /// as "none": it never produces a missing-scope warning.
    pub scopes: Option<Vec<String>>,
    /// Scopes GitPoulpi wants (see `IntegrationKind::token_help`) that the
    /// host said are absent. Always empty when `scopes` is `None`.
    pub missing_scopes: Vec<String>,
    /// When the token expires, as the host spelled it.
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    #[default]
    Open,
    Draft,
    Merged,
    Closed,
}

impl PullRequestState {
    /// Lower is better when several requests exist for one branch: a live
    /// non-draft request beats a draft, which beats a merged one, which
    /// beats one closed unmerged.
    fn rank(self) -> u8 {
        match self {
            PullRequestState::Open => 0,
            PullRequestState::Draft => 1,
            PullRequestState::Merged => 2,
            PullRequestState::Closed => 3,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestSummary {
    /// GitHub number / GitLab `iid`.
    pub number: u64,
    /// What the host's own UI shows: `#12` (GitHub) or `!53` (GitLab).
    pub iid_label: String,
    pub title: String,
    pub state: PullRequestState,
    pub source_branch: String,
    pub target_branch: String,
    /// The fork the source branch lives in, when it is not the target
    /// project itself.
    pub source_project: Option<String>,
    pub author: String,
    pub web_url: String,
    /// ISO-8601 as the host sent it.
    pub updated_at: String,
    /// The current user is asked to review it.
    pub review_requested: bool,
    /// Logins of the people it is assigned to.
    #[serde(default)]
    pub assignees: Vec<String>,
    /// Logins of the people asked to review it.
    #[serde(default)]
    pub reviewers: Vec<String>,
    /// The state of the head commit's checks (CI); `None` = unknown or not
    /// asked (GitHub only answers it on request, see [`PrFilter::checks`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checks: Option<ChecksState>,
    /// Whether the request conflicts with its target; `None` = unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_conflicts: Option<bool>,
}

/// The rolled-up state of a request's checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksState {
    /// Queued or running.
    Pending,
    Passing,
    Failing,
}

/// Picks the request that best answers "which PR is this branch's?":
/// open, then draft, then merged, then closed; newest `updated_at` breaks
/// a tie (ISO-8601 strings sort chronologically).
pub fn best_pull_request(mut all: Vec<PullRequestSummary>) -> Option<PullRequestSummary> {
    all.sort_by(|a, b| {
        a.state
            .rank()
            .cmp(&b.state.rank())
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    all.into_iter().next()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    #[default]
    Open,
    Closed,
}

/// A milestone as an issue row carries it (`"milestone": {"title": ..}`).
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[allow(dead_code)] // only the hosts compiled in read it
pub(crate) struct MilestoneTitle {
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueSummary {
    /// GitHub number / GitLab `iid`; Jira's numeric key suffix; Trello's
    /// `idShort`. For sorting and old paths: humans use `reference`.
    pub number: u64,
    /// The provider's own id (Trello card id, Jira id); the number as a
    /// string on GitHub and GitLab.
    #[serde(default)]
    pub id: String,
    /// What a human types to reference it: `#12` (GitHub / GitLab),
    /// `ABC-123` (Jira), `AB#123` (Azure), a Trello `shortUrl`.
    #[serde(default)]
    pub reference: String,
    /// Work item type, Jira issue type, Trello list name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind_label: Option<String>,
    /// The workflow state as the provider spells it (Jira, Azure).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_label: Option<String>,
    /// `#12` on both hosts.
    pub iid_label: String,
    pub title: String,
    pub state: IssueState,
    pub author: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub web_url: String,
    pub updated_at: String,
    /// The branch name the tracker proposes for the issue (Linear's
    /// `branchName`); `None`: the app derives one from the reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    /// The milestone's title (GitHub, GitLab, Gitea, local notes); the
    /// Kanban board filters on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<String>,
}

/// Which states a list query asks for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateFilter {
    #[default]
    Open,
    /// For pull requests this includes merged ones.
    Closed,
    All,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrFilter {
    pub state: StateFilter,
    /// Only those the current user opened.
    pub mine: bool,
    /// Only those awaiting the current user's review.
    pub review_requested: bool,
    /// Only those that mention the current user (GitHub: the search API's
    /// `mentions:`; GitLab: the to-do list's `mentioned` items). Providers
    /// without the notion return nothing.
    pub mentioned: bool,
    /// Also fill `checks` and `has_conflicts` where the list endpoint does
    /// not carry them (GitHub: a bounded number of extra requests per
    /// request shown). GitLab always carries them.
    pub checks: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IssueFilter {
    pub state: StateFilter,
    /// Only those assigned to the current user.
    pub assigned_to_me: bool,
    /// Only those that mention the current user (see [`PrFilter::mentioned`]).
    pub mentioned: bool,
    /// The provider-native extra clause (`IssuesSource::filter`): a Jira
    /// JQL fragment ANDed with `project = KEY`, an Azure WIQL condition,
    /// Trello list names (comma-separated). Other providers ignore it.
    pub query: Option<String>,
}

/// One entry of a create-form choice list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
}

/// What the New-issue form can offer for a project: issue types (Jira,
/// Azure) and lists (Trello). Empty for GitHub and GitLab.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueCreateOptions {
    pub types: Vec<Choice>,
    pub lists: Vec<Choice>,
    pub default_type: Option<String>,
    pub default_list: Option<String>,
}

/// Whether a granted scope satisfies a required one, including the
/// umbrella scopes (`api` covers everything GitLab-side, `user` covers
/// `read:user`, ...).
fn scope_satisfies(granted: &str, required: &str) -> bool {
    granted == required
        || matches!(
            (granted, required),
            (
                "api",
                "read_user" | "read_api" | "read_repository" | "write_repository"
            ) | ("user", "read:user" | "user:email")
                | ("admin:public_key", "write:public_key" | "read:public_key")
                | ("write:public_key", "read:public_key")
        )
}

/// The required scopes nothing in `granted` satisfies.
pub fn missing_scopes(required: &[&str], granted: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|r| !granted.iter().any(|g| scope_satisfies(g, r)))
        .map(|r| r.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(n: u64, state: PullRequestState, updated: &str) -> PullRequestSummary {
        PullRequestSummary {
            number: n,
            iid_label: format!("#{n}"),
            title: String::new(),
            state,
            source_branch: "b".into(),
            target_branch: "main".into(),
            source_project: None,
            author: String::new(),
            web_url: String::new(),
            updated_at: updated.into(),
            review_requested: false,
            assignees: Vec::new(),
            reviewers: Vec::new(),
            checks: None,
            has_conflicts: None,
        }
    }

    #[test]
    fn best_pull_request_prefers_open_then_draft_then_merged_then_newest() {
        use PullRequestState::*;
        let best = best_pull_request(vec![
            pr(1, Merged, "2026-01-05"),
            pr(2, Draft, "2026-01-04"),
            pr(3, Open, "2026-01-01"),
            pr(4, Open, "2026-01-02"),
            pr(5, Closed, "2026-01-09"),
        ])
        .unwrap();
        assert_eq!(best.number, 4, "newest of the open ones");
        let best = best_pull_request(vec![pr(1, Closed, "x"), pr(2, Merged, "a")]).unwrap();
        assert_eq!(best.number, 2);
        assert!(best_pull_request(vec![]).is_none());
    }

    #[test]
    fn the_issue_model_has_defaults_and_additive_fields() {
        let issue = IssueSummary::default();
        assert_eq!(issue.state, IssueState::Open);
        assert_eq!(issue.reference, "");
        assert_eq!(issue.kind_label, None);
        // Optional labels stay out of the JSON when absent.
        let json = serde_json::to_value(&issue).unwrap();
        assert!(json.get("kind_label").is_none() && json.get("status_label").is_none());
        // An older payload without the new fields still deserializes.
        let old = r##"{"number":1,"iid_label":"#1","title":"t","state":"open","author":"a",
            "assignees":[],"labels":[],"web_url":"u","updated_at":"x"}"##;
        let parsed: IssueSummary = serde_json::from_str(old).unwrap();
        assert_eq!(parsed.number, 1);
        assert_eq!(parsed.id, "");
        assert_eq!(IssueFilter::default().query, None);
        let options = IssueCreateOptions::default();
        assert!(options.types.is_empty() && options.lists.is_empty());
    }

    #[test]
    fn umbrella_scopes_satisfy_narrower_ones() {
        let granted = vec!["api".to_string()];
        assert!(missing_scopes(&["api", "read_user"], &granted).is_empty());
        let granted = vec!["repo".to_string()];
        assert_eq!(
            missing_scopes(&["repo", "read:user", "admin:public_key"], &granted),
            vec!["read:user", "admin:public_key"]
        );
        let granted = vec!["user".to_string()];
        assert!(missing_scopes(&["read:user"], &granted).is_empty());
    }
}
