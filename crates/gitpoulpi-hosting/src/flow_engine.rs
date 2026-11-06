//! Host-driven flows, the shared engine of the CLI, the MCP server and the
//! TUI: create the issue, make the branch on the host, find what the host or a
//! bot made for an issue, wait for it. Everything here is a function over a
//! [`HostingClient`], so a fake transport tests it and the three frontends
//! cannot drift apart. The loops (sleeping, cancelling, drawing a spinner) are
//! the frontend's: it asks [`find_for_issue`] again after [`backoff`].

use std::time::Duration;

use gitpoulpi_core::workflow::{
    branches_for_issue, issue_branch_pattern, slugify, BranchPattern, Fields, IssueDraft,
    IssueFlow, WaitFor, DEFAULT_SLUG_MAX,
};

use crate::client::{CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest};
use crate::issue_flow::{CreatedBranch, NewIssueBranch};
use crate::model::{PrFilter, PullRequestSummary, StateFilter};
use crate::{Error, Result};

/// What the host (or a bot) made for an issue, as far as it could be told.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Found {
    /// Branch names (without a remote), the host's links first.
    pub branches: Vec<String>,
    /// Merge / pull requests tied to the issue by the host or by name.
    pub requests: Vec<PullRequestSummary>,
    /// Sources that failed (a host without the endpoint, a 403): the rest
    /// still answered.
    pub notes: Vec<String>,
}

impl Found {
    /// Whether what was found is what `wait` asked for.
    pub fn satisfies(&self, wait: WaitFor) -> bool {
        (wait.wants_branch() && !self.branches.is_empty())
            || (wait.wants_request() && !self.requests.is_empty())
    }

    /// The branch to pick up: the one of the newest live request, else the
    /// first branch found.
    pub fn best_branch(&self) -> Option<&str> {
        let live = self.requests.iter().find(|r| {
            matches!(
                r.state,
                crate::model::PullRequestState::Open | crate::model::PullRequestState::Draft
            )
        });
        live.map(|r| r.source_branch.as_str())
            .or_else(|| self.requests.first().map(|r| r.source_branch.as_str()))
            .or_else(|| self.branches.first().map(String::as_str))
            .filter(|b| !b.is_empty())
    }

    /// The request that has `branch` as its source: its target is the
    /// branch's parent.
    pub fn request_of(&self, branch: &str) -> Option<&PullRequestSummary> {
        self.requests.iter().find(|r| r.source_branch == branch)
    }
}

/// How long to wait before asking the host again after `attempt` tries
/// (0-based): 1 s, 2 s, 3 s, 5 s, 8 s, then every 10 s. Bounded so a bot that
/// takes minutes costs a few dozen cheap calls, never a hammered host.
/// `GITPOULPI_FLOW_POLL_MS` fixes the pause (tests and benches).
pub fn backoff(attempt: u32) -> Duration {
    if let Some(ms) = std::env::var("GITPOULPI_FLOW_POLL_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        return Duration::from_millis(ms.max(10));
    }
    const STEPS: [u64; 5] = [1, 2, 3, 5, 8];
    Duration::from_secs(STEPS.get(attempt as usize).copied().unwrap_or(10))
}

/// The text a branch search for `number` starts with, for each type's
/// pattern (`feature/12` for `feature/{issue}-{slug}`), without duplicates.
fn searches(flow: &IssueFlow, type_ids: &[String], number: u64) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in type_ids {
        if let Ok(pattern) = issue_branch_pattern(flow, t) {
            let s = format!("{}{number}", pattern.prefix());
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    if out.is_empty() {
        out.push(number.to_string());
    }
    out
}

/// Asks the host what exists for issue `number`: its own links, the branches
/// matching the workflow's `branch_from_issue` and the requests whose source
/// branch does. A source that fails is noted, not fatal; when every source
/// fails the error of the first is returned.
pub async fn find_for_issue(
    client: &dyn HostingClient,
    project: &str,
    flow: &IssueFlow,
    type_ids: &[String],
    number: u64,
) -> Result<Found> {
    let mut found = Found::default();
    let mut answered = false;
    let mut first_error: Option<Error> = None;
    let note = |what: &str, e: Error, found: &mut Found, first: &mut Option<Error>| {
        found.notes.push(format!("{what}: {e}"));
        if first.is_none() {
            *first = Some(e);
        }
    };
    let ids: Vec<&str> = type_ids.iter().map(String::as_str).collect();

    match client.issue_links(project, number).await {
        Ok(links) => {
            answered = true;
            found.branches.extend(links.branches);
            found.requests.extend(links.requests);
        }
        Err(e) => note("the issue's links", e, &mut found, &mut first_error),
    }
    let mut names: Vec<String> = Vec::new();
    for search in searches(flow, type_ids, number) {
        match client.branch_names(project, &search).await {
            Ok(rows) => {
                answered = true;
                names.extend(rows);
            }
            Err(e) => {
                note("the branch search", e, &mut found, &mut first_error);
                break;
            }
        }
    }
    for b in branches_for_issue(flow, &ids, &names, number) {
        if !found.branches.contains(&b) {
            found.branches.push(b);
        }
    }
    // Requests the host did not link (Gitea, a bot that does not close the
    // issue): their source branch carries the issue number.
    if found.requests.is_empty() {
        let filter = PrFilter {
            state: StateFilter::Open,
            ..PrFilter::default()
        };
        match client.list_pull_requests(project, &filter).await {
            Ok(rows) => {
                answered = true;
                let sources: Vec<String> = rows.iter().map(|r| r.source_branch.clone()).collect();
                let matching = branches_for_issue(flow, &ids, &sources, number);
                found.requests.extend(
                    rows.into_iter()
                        .filter(|r| matching.contains(&r.source_branch)),
                );
            }
            Err(e) => note("the request list", e, &mut found, &mut first_error),
        }
    }
    for r in &found.requests {
        if !r.source_branch.is_empty() && !found.branches.contains(&r.source_branch) {
            found.branches.push(r.source_branch.clone());
        }
    }
    if !answered {
        if let Some(e) = first_error {
            return Err(e);
        }
    }
    Ok(found)
}

/// Creates the issue of `draft` on `project`.
pub async fn create_issue(
    client: &dyn HostingClient,
    project: &str,
    draft: &IssueDraft,
    milestone: Option<u64>,
) -> Result<CreatedIssue> {
    client
        .create_issue(&NewIssue {
            project: project.to_string(),
            title: draft.title.clone(),
            body: draft.body.clone(),
            labels: draft.labels.clone(),
            assignees: draft.assignees.clone(),
            milestone,
            ..NewIssue::default()
        })
        .await
}

/// The name the host-made branch of issue `number` gets: the workflow's
/// `branch_from_issue` with the type, the issue number and the title's slug.
pub fn issue_branch_name(
    flow: &IssueFlow,
    type_id: &str,
    number: u64,
    title: &str,
) -> std::result::Result<String, String> {
    let pattern: BranchPattern =
        issue_branch_pattern(flow, type_id).map_err(|e| format!("branch_from_issue: {e}"))?;
    let mut fields = Fields::new();
    fields.insert("issue".into(), number.to_string());
    fields.insert("issue_key".into(), format!("{type_id}-{number}"));
    let mut slug = slugify(title, DEFAULT_SLUG_MAX);
    if slug.is_empty() {
        slug = "issue".into();
    }
    fields.insert("slug".into(), slug);
    pattern.render(&fields).map_err(|e| e.to_string())
}

/// What `host-creates-branch` made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostMade {
    pub branch: CreatedBranch,
    pub request: Option<CreatedPullRequest>,
    /// The merge request could not be opened (a host refuses one with no
    /// commits of its own); the branch exists.
    pub warning: Option<String>,
}

/// Creates `name` from `from` on the host for the issue and, when `open_request`,
/// a draft merge request into `target` that closes the issue.
#[allow(clippy::too_many_arguments)]
pub async fn host_creates_branch(
    client: &dyn HostingClient,
    project: &str,
    number: u64,
    name: &str,
    from: &str,
    target: &str,
    title: &str,
    open_request: bool,
) -> Result<HostMade> {
    let branch = client
        .create_issue_branch(&NewIssueBranch {
            project: project.to_string(),
            issue_number: number,
            name: name.to_string(),
            from: from.to_string(),
        })
        .await?;
    let mut made = HostMade {
        branch,
        request: None,
        warning: None,
    };
    if open_request {
        let result = client
            .create_pull_request(&NewPullRequest {
                project: project.to_string(),
                source_branch: made.branch.name.clone(),
                target_branch: target.to_string(),
                title: title.to_string(),
                body: format!("Closes #{number}"),
                draft: true,
                ..NewPullRequest::default()
            })
            .await;
        match result {
            Ok(created) => made.request = Some(created),
            Err(e) => {
                made.warning = Some(format!(
                    "the branch exists but the merge request could not be opened: {e}"
                ))
            }
        }
    }
    Ok(made)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::IntegrationKind;

    use crate::client::client_for;
    use crate::http::RecordedTransport;
    use crate::integration::Integration;

    use super::*;

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn gitlab(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
        client_for(
            Integration::new(
                "gl",
                IntegrationKind::GitlabSelfManaged,
                "https://gl.example.com",
            ),
            "t".into(),
            t,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap()
    }

    fn mr(iid: u64, state: &str, source: &str) -> String {
        format!(
            r#"{{"iid":{iid},"title":"t","state":"{state}","source_branch":"{source}","target_branch":"develop",
            "source_project_id":1,"target_project_id":1,"web_url":"u","updated_at":""}}"#
        )
    }

    #[test]
    fn backoff_grows_then_stays_at_ten_seconds() {
        if std::env::var("GITPOULPI_FLOW_POLL_MS").is_ok() {
            return;
        }
        let secs: Vec<u64> = (0..8).map(|i| backoff(i).as_secs()).collect();
        assert_eq!(secs, [1, 2, 3, 5, 8, 10, 10, 10]);
    }

    #[test]
    fn nothing_yet_is_an_empty_answer_not_an_error() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/related_merge_requests", 200, "[]")
                .on("/closed_by", 200, "[]")
                .on("/repository/branches", 200, "[]")
                .on("/merge_requests", 200, "[]"),
        );
        let found = block_on(find_for_issue(
            &*gitlab(t),
            "g/p",
            &IssueFlow::default(),
            &["feature".to_string()],
            12,
        ))
        .unwrap();
        assert!(!found.satisfies(WaitFor::BranchOrMergeRequest));
        assert!(found.best_branch().is_none());
    }

    #[test]
    fn a_bot_branch_and_its_request_are_found_by_number_and_by_the_hosts_link() {
        let t = Arc::new(
            RecordedTransport::new()
                .on(
                    "/related_merge_requests",
                    200,
                    &format!("[{}]", mr(5, "opened", "feature/12-login")),
                )
                .on("/closed_by", 200, "[]")
                .on(
                    "/repository/branches",
                    200,
                    r#"[{"name":"feature/12-login"},{"name":"feature/120-x"}]"#,
                ),
        );
        let found = block_on(find_for_issue(
            &*gitlab(t),
            "g/p",
            &IssueFlow::default(),
            &["feature".to_string(), "bugfix".to_string()],
            12,
        ))
        .unwrap();
        assert_eq!(found.branches, ["feature/12-login"]);
        assert_eq!(found.best_branch(), Some("feature/12-login"));
        assert_eq!(
            found.request_of("feature/12-login").unwrap().target_branch,
            "develop"
        );
        assert!(found.satisfies(WaitFor::Branch));
        assert!(found.satisfies(WaitFor::MergeRequest));
    }

    #[test]
    fn a_request_the_host_did_not_link_is_found_by_its_source_branch() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/related_merge_requests", 200, "[]")
                .on("/closed_by", 200, "[]")
                .on("/repository/branches", 200, "[]")
                .on(
                    "/merge_requests",
                    200,
                    &format!(
                        "[{},{}]",
                        mr(6, "opened", "feature/12-a"),
                        mr(7, "opened", "feature/13-b")
                    ),
                ),
        );
        let found = block_on(find_for_issue(
            &*gitlab(t),
            "g/p",
            &IssueFlow::default(),
            &["feature".to_string()],
            12,
        ))
        .unwrap();
        assert_eq!(found.requests.len(), 1);
        assert_eq!(found.requests[0].number, 6);
        // The request's branch is a branch of the issue too.
        assert_eq!(found.branches, ["feature/12-a"]);
    }

    #[test]
    fn when_every_source_fails_the_error_comes_back() {
        let t = Arc::new(RecordedTransport::new().on("/projects/", 500, "{}"));
        let r = block_on(find_for_issue(
            &*gitlab(t),
            "g/p",
            &IssueFlow::default(),
            &["feature".to_string()],
            1,
        ));
        assert!(r.is_err());
    }

    #[test]
    fn the_branch_name_follows_the_pattern() {
        let flow = IssueFlow::default();
        assert_eq!(
            issue_branch_name(&flow, "feature", 12, "Login page!").unwrap(),
            "feature/12-login-page"
        );
        assert_eq!(
            issue_branch_name(&flow, "bugfix", 3, "???").unwrap(),
            "bugfix/3-issue"
        );
    }

    #[test]
    fn host_creates_the_branch_then_a_draft_request_that_closes_the_issue() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/repository/branches", 201, r#"{"name":"feature/12-x"}"#)
                .on(
                    "/merge_requests",
                    201,
                    r#"{"iid":9,"web_url":"https://gl.example.com/mr/9"}"#,
                ),
        );
        let made = block_on(host_creates_branch(
            &*gitlab(t.clone()),
            "g/p",
            12,
            "feature/12-x",
            "develop",
            "develop",
            "Login",
            true,
        ))
        .unwrap();
        assert_eq!(made.request.unwrap().number, 9);
        let mr_request = &t.requests()[1];
        let body: serde_json::Value =
            serde_json::from_str(mr_request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["description"], "Closes #12");
        assert_eq!(body["source_branch"], "feature/12-x");
        assert!(body["title"].as_str().unwrap().starts_with("Draft: "));
    }

    #[test]
    fn a_refused_request_leaves_the_branch_and_a_warning() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/repository/branches", 201, r#"{"name":"b"}"#)
                .on("/merge_requests", 422, r#"{"message":"no commits"}"#),
        );
        let made = block_on(host_creates_branch(
            &*gitlab(t),
            "g/p",
            1,
            "b",
            "main",
            "main",
            "t",
            true,
        ))
        .unwrap();
        assert!(made.request.is_none());
        assert!(made.warning.unwrap().contains("could not be opened"));
    }
}
