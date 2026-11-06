//! The GitHub client (github.com and GitHub Enterprise Server).

use std::sync::Arc;

use crate::client::{
    encode_component, fetch_pages, fetch_pages_with, CreatedIssue, CreatedPullRequest,
    HostingClient, NewIssue, NewPullRequest, PAGE_SIZE,
};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::{
    best_pull_request, missing_scopes, ChecksState, HostedUser, IssueFilter, IssueState,
    IssueSummary, PrFilter, PullRequestState, PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::Result;

mod protection;

pub struct GitHubClient {
    pub(crate) integration: Integration,
    token: String,
    pub(crate) transport: std::sync::Arc<dyn HttpTransport>,
}

impl GitHubClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
        }
    }

    /// The headers every GitHub request carries. `Bearer` is what GitHub
    /// documents for fine-grained tokens and accepts for classic ones;
    /// `token <x>` works for neither fine-grained nor GitHub App tokens.
    /// Pinning the API version stops a future default from reshaping the
    /// JSON under a released GitPoulpi.
    pub(crate) fn authed(&self, request: HttpRequest) -> HttpRequest {
        request
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    pub(crate) fn delete(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::delete(url))
    }

    pub(crate) fn get(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::get(url))
    }

    pub(crate) fn post(&self, url: String, body: &serde_json::Value) -> HttpRequest {
        self.authed(HttpRequest::post_json(url, body))
    }
}

#[derive(serde::Deserialize)]
struct GitHubRepo {
    full_name: String,
    name: String,
    description: Option<String>,
    default_branch: Option<String>,
    ssh_url: Option<String>,
    clone_url: Option<String>,
    html_url: Option<String>,
    #[serde(default)]
    private: bool,
}

#[derive(serde::Deserialize)]
struct GitHubKey {
    id: i64,
    title: String,
    key: String,
}

#[derive(serde::Deserialize)]
struct GitHubPull {
    number: u64,
    html_url: String,
}

#[async_trait::async_trait]
impl HostingClient for GitHubClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        // `/user/repos` rather than `/users/<login>/repos`: the former is
        // what the *token* can see, including private repositories and
        // ones owned by organisations; the latter shows only public ones.
        let url = format!(
            "{}/user/repos?sort=pushed&affiliation=owner,collaborator,organization_member",
            self.integration.api_base()
        );
        let repos: Vec<GitHubRepo> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub repositories",
        )
        .await?;
        Ok(repos
            .into_iter()
            .map(|r| HostedProject {
                integration_id: self.integration.id.clone(),
                id: r.full_name.clone(),
                name: r.name,
                full_name: r.full_name,
                description: r.description,
                default_branch: r.default_branch,
                ssh_url: r.ssh_url,
                https_url: r.clone_url,
                web_url: r.html_url,
                private: r.private,
            })
            .collect())
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        let url = format!(
            "{}/user/keys?per_page={PAGE_SIZE}",
            self.integration.api_base()
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("listing GitHub SSH keys")?;
        let keys: Vec<GitHubKey> = response.json()?;
        Ok(keys
            .into_iter()
            .map(|k| HostedSshKey {
                id: k.id.to_string(),
                title: k.title,
                key: k.key,
            })
            .collect())
    }

    async fn upload_ssh_key(&self, title: &str, public_key: &str) -> Result<HostedSshKey> {
        let url = format!("{}/user/keys", self.integration.api_base());
        let body = serde_json::json!({ "title": title, "key": public_key.trim() });
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("uploading an SSH key to GitHub")?;
        let key: GitHubKey = response.json()?;
        Ok(HostedSshKey {
            id: key.id.to_string(),
            title: key.title,
            key: key.key,
        })
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        // `{p}/pulls`, not `/repos/{p}/pulls`+something else: `project`
        // is already `owner/name`, exactly what this endpoint wants.
        let url = format!(
            "{}/repos/{}/pulls",
            self.integration.api_base(),
            req.project
        );
        // A fork's branch is addressed `owner:branch`.
        let head = match req.fork_source() {
            Some(fork) => format!(
                "{}:{}",
                fork.split('/').next().unwrap_or(fork),
                req.source_branch
            ),
            None => req.source_branch.clone(),
        };
        let body = serde_json::json!({
            "title": req.title,
            "head": head,
            "base": req.target_branch,
            "body": req.body,
            "draft": req.draft,
        });
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening a GitHub pull request")?;
        let pr: GitHubPull = response.json()?;
        // The pulls endpoint takes neither assignees, labels, a milestone
        // nor reviewers: they follow in a second round. A failure there
        // must not hide that the request exists.
        let warning = self.apply_new_pr_meta(req, pr.number).await.err().map(|e| {
            format!("the pull request was opened, but its metadata could not be set: {e}")
        });
        Ok(CreatedPullRequest {
            number: pr.number,
            web_url: pr.html_url,
            warning,
        })
    }

    async fn create_issue_branch(
        &self,
        req: &crate::issue_flow::NewIssueBranch,
    ) -> Result<crate::issue_flow::CreatedBranch> {
        self.flow_create_branch(req).await
    }

    async fn issue_links(
        &self,
        project: &str,
        number: u64,
    ) -> Result<crate::issue_flow::IssueLinks> {
        self.flow_issue_links(project, number).await
    }

    async fn branch_names(&self, project: &str, search: &str) -> Result<Vec<String>> {
        self.flow_branch_names(project, search).await
    }

    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        crate::account::github::fetch(self, query).await
    }

    async fn current_user(&self) -> Result<HostedUser> {
        Ok(self.fetch_user().await?.0)
    }

    async fn protected_branches(
        &self,
        project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        self.protection(project).await
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        let (user, response) = self.fetch_user().await?;
        // Classic tokens list their scopes in `X-OAuth-Scopes`. A
        // fine-grained token sends the header empty or not at all, which
        // means *unknown* -- reporting every scope as missing would tell
        // a working token it is broken.
        let scopes = header_value(&response, "x-oauth-scopes")
            .map(|raw| {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|scopes| !scopes.is_empty());
        let required = self
            .integration
            .kind
            .token_help()
            .map(|h| h.scopes)
            .unwrap_or(&[]);
        let missing = scopes
            .as_deref()
            .map(|granted| missing_scopes(required, granted))
            .unwrap_or_default();
        Ok(TokenCheck {
            user,
            scopes,
            missing_scopes: missing,
            expires_at: header_value(&response, "github-authentication-token-expiration"),
        })
    }

    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        let base = self.integration.api_base();
        let state = match filter.state {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        };
        let url = format!("{base}/repos/{project}/pulls?state={state}&sort=updated&direction=desc");
        let pulls: Vec<GitHubPullRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub pull requests",
        )
        .await?;

        // Which of them await the current user's review comes from the
        // search API: the pulls listing only carries the *requested
        // reviewers*, and matching those against a login would need
        // team expansion GitHub does server-side here.
        let awaiting: std::collections::HashSet<u64> = if filter.review_requested {
            self.review_requested_numbers(project).await?
        } else {
            Default::default()
        };
        let me = if filter.mine {
            Some(self.login().await?)
        } else {
            None
        };
        let mentioning = if filter.mentioned {
            Some(self.mentioned_numbers(project, true, filter.state).await?)
        } else {
            None
        };

        let mut rows: Vec<(PullRequestSummary, String)> = pulls
            .into_iter()
            .filter(|p| !filter.review_requested || awaiting.contains(&p.number))
            .filter(|p| {
                mentioning
                    .as_ref()
                    .map_or(true, |set| set.contains(&p.number))
            })
            .filter(|p| {
                me.as_deref().map_or(true, |me| {
                    p.user
                        .as_ref()
                        .is_some_and(|u| u.login.eq_ignore_ascii_case(me))
                })
            })
            .map(|p| {
                let review_requested = awaiting.contains(&p.number);
                let sha = p.head.sha.clone();
                (p.into_summary(review_requested), sha)
            })
            .collect();
        if filter.checks {
            self.fill_status(project, &mut rows).await;
        }
        Ok(rows.into_iter().map(|(summary, _)| summary).collect())
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let owner = project.split('/').next().unwrap_or(project);
        let url = format!(
            "{}/repos/{project}/pulls?state=all&head={}",
            self.integration.api_base(),
            encode_component(&format!("{owner}:{branch}"))
        );
        let pulls: Vec<GitHubPullRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "looking up the pull request for a branch",
        )
        .await?;
        Ok(best_pull_request(
            pulls.into_iter().map(|p| p.into_summary(false)).collect(),
        ))
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let state = match filter.state {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        };
        let mut url = format!(
            "{}/repos/{project}/issues?state={state}&sort=updated&direction=desc",
            self.integration.api_base()
        );
        if filter.assigned_to_me {
            url.push_str(&format!(
                "&assignee={}",
                encode_component(&self.login().await?)
            ));
        }
        let rows: Vec<GitHubIssueRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub issues",
        )
        .await?;
        let mentioning = if filter.mentioned {
            Some(self.mentioned_numbers(project, false, filter.state).await?)
        } else {
            None
        };
        // This endpoint returns pull requests too, flagged by a
        // `pull_request` key; they belong to the PR list, not here.
        Ok(rows
            .into_iter()
            .filter(|r| r.pull_request.is_none())
            .filter(|r| {
                mentioning
                    .as_ref()
                    .map_or(true, |set| set.contains(&r.number))
            })
            .map(|r| IssueSummary {
                number: r.number,
                id: r.number.to_string(),
                reference: format!("#{}", r.number),
                iid_label: format!("#{}", r.number),
                title: r.title,
                state: if r.state == "closed" {
                    IssueState::Closed
                } else {
                    IssueState::Open
                },
                author: r.user.map(|u| u.login).unwrap_or_default(),
                assignees: r.assignees.into_iter().map(|u| u.login).collect(),
                labels: r.labels.into_iter().map(|l| l.name).collect(),
                web_url: r.html_url,
                updated_at: r.updated_at,
                milestone: r.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
                ..IssueSummary::default()
            })
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let url = format!(
            "{}/repos/{}/issues",
            self.integration.api_base(),
            req.project
        );
        let mut body = serde_json::json!({ "title": req.title, "body": req.body });
        if !req.labels.is_empty() {
            body["labels"] = serde_json::json!(req.labels);
        }
        if !req.assignees.is_empty() {
            body["assignees"] = serde_json::json!(req.assignees);
        }
        if let Some(milestone) = req.milestone {
            body["milestone"] = serde_json::json!(milestone);
        }
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening a GitHub issue")?;
        let issue: GitHubCreated = response.json()?;
        Ok(CreatedIssue {
            number: issue.number,
            web_url: issue.html_url,
            reference: format!("#{}", issue.number),
        })
    }

    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        let url = format!("{}/user/keys/{id}", self.integration.api_base());
        let response = self.transport.send(self.delete(url)).await?;
        response.error_for_status("deleting a GitHub SSH key")
    }

    fn review_capabilities(&self) -> crate::review::ReviewCapabilities {
        crate::review::github::CAPABILITIES
    }

    async fn pull_request_detail(
        &self,
        project: &str,
        number: u64,
    ) -> Result<crate::review::PullRequestDetail> {
        self.review_detail(project, number).await
    }

    async fn pull_request_files(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ReviewFile>> {
        self.review_files(project, number).await
    }

    async fn pull_request_threads(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ReviewThread>> {
        self.review_threads(project, number).await
    }

    async fn pull_request_conversation(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ReviewComment>> {
        self.review_conversation(project, number).await
    }

    async fn pull_request_reviews(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::SubmittedReview>> {
        self.review_list_reviews(project, number).await
    }

    async fn add_comment(
        &self,
        project: &str,
        number: u64,
        body: &str,
    ) -> Result<crate::review::ReviewComment> {
        self.review_add_comment(project, number, body).await
    }

    async fn reply_to_thread(
        &self,
        _project: &str,
        _number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<crate::review::ReviewComment> {
        self.review_reply(thread_id, body).await
    }

    async fn resolve_thread(
        &self,
        _project: &str,
        _number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        self.review_resolve(thread_id, resolved).await
    }

    async fn submit_review(
        &self,
        project: &str,
        number: u64,
        review: &crate::review::ReviewSubmission,
    ) -> Result<crate::review::ReviewOutcome> {
        self.review_submit(project, number, review).await
    }

    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        self.meta_members(project, query).await
    }

    async fn project_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        self.meta_labels(project).await
    }

    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        self.meta_milestones(project).await
    }

    async fn item_meta(&self, project: &str, number: u64, kind: MetaKind) -> Result<ItemMeta> {
        self.meta_item(project, number, kind).await
    }

    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        self.meta_update_issue(project, number, edit).await
    }

    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::Labels
    }

    async fn set_issue_state(&self, project: &str, number: u64, closed: bool) -> Result<()> {
        self.meta_set_issue_state(project, number, closed).await
    }

    async fn update_pull_request(&self, project: &str, number: u64, edit: &PrEdit) -> Result<()> {
        self.meta_update_pull_request(project, number, edit).await
    }

    async fn pull_request_diff(
        &self,
        project: &str,
        number: u64,
        _patch_set: Option<u32>,
        _compare: Option<u32>,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.review_diff(project, number).await
    }

    async fn pull_request_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<crate::review::ApiCommit>> {
        self.review_commits(project, number).await
    }

    async fn commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        self.review_commit_diff(project, sha).await
    }

    async fn file_content(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        self.review_file(project, path, rev).await
    }
}

impl GitHubClient {
    /// The metadata a new pull request was asked to carry.
    async fn apply_new_pr_meta(&self, req: &NewPullRequest, number: u64) -> Result<()> {
        let assignees = (!req.assignees.is_empty()).then_some(req.assignees.as_slice());
        let labels = (!req.labels.is_empty()).then_some(req.labels.as_slice());
        self.patch_issue_fields(
            &req.project,
            number,
            assignees,
            labels,
            req.milestone.map(Some),
        )
        .await?;
        self.change_reviewers(&req.project, number, &req.reviewers, &[])
            .await
    }

    async fn fetch_user(&self) -> Result<(HostedUser, HttpResponse)> {
        let url = format!("{}/user", self.integration.api_base());
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("checking the GitHub token")?;
        let row: GitHubUser = response.json()?;
        let user = HostedUser {
            login: row.login,
            name: row.name.unwrap_or_default(),
            id: row.id.to_string(),
            web_url: row.html_url,
        };
        Ok((user, response))
    }

    /// The cached login when the profile has one, else a `/user` call.
    pub(crate) async fn login(&self) -> Result<String> {
        if let Some(user) = &self.integration.user {
            if !user.login.is_empty() {
                return Ok(user.login.clone());
            }
        }
        Ok(self.fetch_user().await?.0.login)
    }

    /// The numbers of the open (per `state`) pull requests or issues of
    /// `project` that mention the current user: the search API's
    /// `mentions:` qualifier, which the list endpoints have no twin for.
    async fn mentioned_numbers(
        &self,
        project: &str,
        pull_requests: bool,
        state: StateFilter,
    ) -> Result<std::collections::HashSet<u64>> {
        #[derive(serde::Deserialize)]
        struct Search {
            #[serde(default)]
            items: Vec<Item>,
        }
        #[derive(serde::Deserialize)]
        struct Item {
            number: u64,
        }
        let kind = if pull_requests { "is:pr" } else { "is:issue" };
        let state = match state {
            StateFilter::Open => " is:open",
            StateFilter::Closed => " is:closed",
            StateFilter::All => "",
        };
        let login = self.login().await?;
        let query = encode_component(&format!("{kind}{state} mentions:{login} repo:{project}"));
        let url = format!("{}/search/issues?q={query}", self.integration.api_base());
        let items: Vec<Item> = fetch_pages_with(
            &*self.transport,
            |u| self.get(u),
            &url,
            "searching mentions on GitHub",
            |r| Ok(r.json::<Search>()?.items),
        )
        .await?;
        Ok(items.into_iter().map(|i| i.number).collect())
    }

    /// Fills `checks` and `has_conflicts` of the live requests: the list
    /// endpoint carries neither, so each costs three calls (the request for
    /// `mergeable_state`, the head's combined status, its check runs). At most
    /// [`STATUS_FANOUT`] requests are asked, the user's own first; a failing
    /// call (a rate limit, a token without the scope) ends the fan-out and
    /// leaves the rest unknown rather than failing the list.
    async fn fill_status(&self, project: &str, rows: &mut [(PullRequestSummary, String)]) {
        let me = self.login().await.unwrap_or_default();
        let mut order: Vec<usize> = (0..rows.len())
            .filter(|&i| {
                matches!(
                    rows[i].0.state,
                    PullRequestState::Open | PullRequestState::Draft
                ) && !rows[i].1.is_empty()
            })
            .collect();
        order.sort_by_key(|&i| !rows[i].0.author.eq_ignore_ascii_case(&me));
        order.truncate(STATUS_FANOUT);
        for i in order {
            let Ok((checks, conflicts)) = self
                .pull_request_status(project, rows[i].0.number, &rows[i].1)
                .await
            else {
                return;
            };
            rows[i].0.checks = checks;
            rows[i].0.has_conflicts = conflicts;
        }
    }

    async fn pull_request_status(
        &self,
        project: &str,
        number: u64,
        sha: &str,
    ) -> Result<(Option<ChecksState>, Option<bool>)> {
        #[derive(serde::Deserialize)]
        struct Detail {
            mergeable_state: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct Combined {
            #[serde(default)]
            state: String,
            #[serde(default)]
            total_count: u64,
        }
        #[derive(serde::Deserialize)]
        struct Runs {
            #[serde(default)]
            check_runs: Vec<Run>,
        }
        #[derive(serde::Deserialize)]
        struct Run {
            #[serde(default)]
            status: String,
            conclusion: Option<String>,
        }
        let base = self.integration.api_base();
        let detail: Detail = self
            .get_json(
                format!("{base}/repos/{project}/pulls/{number}"),
                "reading a GitHub pull request",
            )
            .await?;
        let combined: Combined = self
            .get_json(
                format!("{base}/repos/{project}/commits/{sha}/status"),
                "reading GitHub commit statuses",
            )
            .await?;
        let runs: Runs = self
            .get_json(
                format!("{base}/repos/{project}/commits/{sha}/check-runs?per_page=100"),
                "reading GitHub check runs",
            )
            .await?;
        let runs: Vec<(&str, Option<&str>)> = runs
            .check_runs
            .iter()
            .map(|r| (r.status.as_str(), r.conclusion.as_deref()))
            .collect();
        let conflicts = match detail.mergeable_state.as_deref() {
            None | Some("unknown") => None,
            Some(state) => Some(state == "dirty"),
        };
        Ok((
            roll_up_checks(&combined.state, combined.total_count, &runs),
            conflicts,
        ))
    }

    pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: String,
        what: &str,
    ) -> Result<T> {
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status(what)?;
        response.json()
    }

    async fn review_requested_numbers(
        &self,
        project: &str,
    ) -> Result<std::collections::HashSet<u64>> {
        #[derive(serde::Deserialize)]
        struct Search {
            #[serde(default)]
            items: Vec<Item>,
        }
        #[derive(serde::Deserialize)]
        struct Item {
            number: u64,
        }
        let query = encode_component(&format!(
            "is:pr is:open review-requested:@me repo:{project}"
        ));
        let url = format!("{}/search/issues?q={query}", self.integration.api_base());
        let items: Vec<Item> = fetch_pages_with(
            &*self.transport,
            |u| self.get(u),
            &url,
            "searching review requests on GitHub",
            |r| Ok(r.json::<Search>()?.items),
        )
        .await?;
        Ok(items.into_iter().map(|i| i.number).collect())
    }
}

/// How many requests of one list get their checks and conflict state asked.
const STATUS_FANOUT: usize = 15;

/// One state out of a commit's combined status (`state`, with how many
/// statuses it combines) and its check runs (`status`, `conclusion`): any
/// failure wins, then anything still running, else passing; no status and no
/// run means no CI, which is unknown, not green.
fn roll_up_checks(
    combined: &str,
    combined_total: u64,
    runs: &[(&str, Option<&str>)],
) -> Option<ChecksState> {
    let failed_status = matches!(combined, "failure" | "error") && combined_total > 0;
    let failed_run = runs.iter().any(|(_, conclusion)| {
        matches!(
            conclusion,
            Some("failure" | "timed_out" | "cancelled" | "action_required" | "startup_failure")
        )
    });
    if failed_status || failed_run {
        return Some(ChecksState::Failing);
    }
    let pending_status = combined == "pending" && combined_total > 0;
    let pending_run = runs.iter().any(|(status, _)| *status != "completed");
    if pending_status || pending_run {
        return Some(ChecksState::Pending);
    }
    (combined_total > 0 || !runs.is_empty()).then_some(ChecksState::Passing)
}

fn header_value(response: &HttpResponse, name: &str) -> Option<String> {
    response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[derive(serde::Deserialize)]
struct GitHubUser {
    login: String,
    id: i64,
    name: Option<String>,
    html_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitHubLogin {
    login: String,
}

#[derive(serde::Deserialize)]
struct GitHubRef {
    #[serde(rename = "ref")]
    name: String,
    #[serde(default)]
    sha: String,
    repo: Option<GitHubRefRepo>,
}

#[derive(serde::Deserialize)]
struct GitHubRefRepo {
    full_name: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct GitHubPullRow {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    merged_at: Option<String>,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    user: Option<GitHubLogin>,
    head: GitHubRef,
    base: GitHubRef,
    #[serde(default)]
    assignees: Vec<GitHubLogin>,
    #[serde(default)]
    requested_reviewers: Vec<GitHubLogin>,
}

impl GitHubPullRow {
    pub(crate) fn into_summary(self, review_requested: bool) -> PullRequestSummary {
        let state = if self.merged_at.is_some() {
            PullRequestState::Merged
        } else if self.state == "closed" {
            PullRequestState::Closed
        } else if self.draft {
            PullRequestState::Draft
        } else {
            PullRequestState::Open
        };
        let base_repo = self.base.repo.as_ref().map(|r| r.full_name.as_str());
        let source_project = self
            .head
            .repo
            .as_ref()
            .map(|r| r.full_name.clone())
            .filter(|head| Some(head.as_str()) != base_repo);
        PullRequestSummary {
            number: self.number,
            iid_label: format!("#{}", self.number),
            title: self.title,
            state,
            source_branch: self.head.name,
            target_branch: self.base.name,
            source_project,
            author: self.user.map(|u| u.login).unwrap_or_default(),
            web_url: self.html_url,
            updated_at: self.updated_at,
            review_requested,
            assignees: self.assignees.into_iter().map(|u| u.login).collect(),
            reviewers: self
                .requested_reviewers
                .into_iter()
                .map(|u| u.login)
                .collect(),
            checks: None,
            has_conflicts: None,
        }
    }
}

#[derive(serde::Deserialize)]
struct GitHubLabel {
    name: String,
}

#[derive(serde::Deserialize)]
struct GitHubIssueRow {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    user: Option<GitHubLogin>,
    #[serde(default)]
    assignees: Vec<GitHubLogin>,
    #[serde(default)]
    labels: Vec<GitHubLabel>,
    pull_request: Option<serde_json::Value>,
    milestone: Option<crate::model::MilestoneTitle>,
}

#[derive(serde::Deserialize)]
struct GitHubCreated {
    number: u64,
    html_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_for;
    use crate::http::RecordedTransport;
    use crate::Error;
    use gitpoulpi_config::IntegrationKind;

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn client(
        kind: IntegrationKind,
        url: &str,
        transport: Arc<RecordedTransport>,
    ) -> Box<dyn HostingClient> {
        client_for(
            Integration::new("gh", kind, url),
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap()
    }

    const USER: &str =
        r#"{"login":"octo","id":9,"name":"Octo Cat","html_url":"https://github.com/octo"}"#;

    fn pull(n: u64, state: &str, extra: &str) -> String {
        format!(
            r#"{{"number":{n},"title":"t{n}","state":"{state}","html_url":"https://github.com/me/thing/pull/{n}",
            "updated_at":"2026-01-0{n}T00:00:00Z","user":{{"login":"octo"}},
            "head":{{"ref":"feature","repo":{{"full_name":"me/thing"}}}},
            "base":{{"ref":"main","repo":{{"full_name":"me/thing"}}}}{extra}}}"#
        )
    }

    #[test]
    fn check_token_reads_scopes_and_reports_missing_ones() {
        let transport = Arc::new(RecordedTransport::new().on_with_headers(
            "/user",
            200,
            USER,
            &[
                ("X-OAuth-Scopes", "repo, read:user"),
                (
                    "github-authentication-token-expiration",
                    "2026-12-01 10:00:00 UTC",
                ),
            ],
        ));
        let c = client(IntegrationKind::Github, "https://github.com", transport);
        let check = block_on(c.check_token()).unwrap();
        assert_eq!(check.user.login, "octo");
        assert_eq!(check.user.name, "Octo Cat");
        assert_eq!(check.user.id, "9");
        assert_eq!(check.missing_scopes, vec!["admin:public_key"]);
        assert_eq!(check.expires_at.as_deref(), Some("2026-12-01 10:00:00 UTC"));
    }

    #[test]
    fn a_fine_grained_token_has_unknown_scopes_not_missing_ones() {
        let transport = Arc::new(RecordedTransport::new().on("/user", 200, USER));
        let c = client(IntegrationKind::Github, "https://github.com", transport);
        let check = block_on(c.check_token()).unwrap();
        assert_eq!(check.scopes, None);
        assert!(check.missing_scopes.is_empty());
    }

    #[test]
    fn enterprise_calls_go_to_the_api_v3_path() {
        let transport = Arc::new(RecordedTransport::new().on("/user", 200, USER));
        let c = client(
            IntegrationKind::GithubEnterprise,
            "https://ghe.example.com",
            transport.clone(),
        );
        block_on(c.current_user()).unwrap();
        assert_eq!(
            transport.requests()[0].url,
            "https://ghe.example.com/api/v3/user"
        );
    }

    #[test]
    fn a_rejected_token_and_an_underscoped_one_are_distinct_errors() {
        for (status, unauthorized) in [(401, true), (403, false)] {
            let transport =
                Arc::new(RecordedTransport::new().on("/user", status, r#"{"message":"no"}"#));
            let c = client(IntegrationKind::Github, "https://github.com", transport);
            let err = block_on(c.check_token()).unwrap_err();
            assert_eq!(matches!(err, Error::Unauthorized { .. }), unauthorized);
        }
    }

    #[test]
    fn pull_requests_normalise_state_labels_and_forks() {
        let merged = pull(3, "closed", r#","merged_at":"2026-01-03T00:00:00Z""#);
        let draft = pull(2, "open", r#","draft":true"#);
        let fork = pull(1, "open", "").replace(
            r#""head":{"ref":"feature","repo":{"full_name":"me/thing"}}"#,
            r#""head":{"ref":"feature","repo":{"full_name":"fork/thing"}}"#,
        );
        let body = format!("[{merged},{draft},{fork}]");
        let transport = Arc::new(RecordedTransport::new().on("/pulls?", 200, &body));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let prs = block_on(c.list_pull_requests("me/thing", &PrFilter::default())).unwrap();
        assert_eq!(prs[0].state, PullRequestState::Merged);
        assert_eq!(prs[0].iid_label, "#3");
        assert_eq!(prs[1].state, PullRequestState::Draft);
        assert_eq!(prs[2].source_project.as_deref(), Some("fork/thing"));
        assert_eq!(prs[0].source_project, None);
        assert_eq!(prs[0].source_branch, "feature");
        assert_eq!(prs[0].target_branch, "main");
        assert!(transport.requests()[0].url.contains("state=open"));
    }

    #[test]
    fn assignees_and_requested_reviewers_are_carried_into_the_summary() {
        let row = pull(4, "open", "").replace(
            r#""user":{"login":"octo"}"#,
            r#""user":{"login":"octo"},"assignees":[{"login":"me"}],"requested_reviewers":[{"login":"rev1"},{"login":"me"}]"#,
        );
        let transport = Arc::new(RecordedTransport::new().on("/pulls?", 200, &format!("[{row}]")));
        let c = client(IntegrationKind::Github, "https://github.com", transport);
        let prs = block_on(c.list_pull_requests("me/thing", &PrFilter::default())).unwrap();
        assert_eq!(prs[0].assignees, vec!["me"]);
        assert_eq!(prs[0].reviewers, vec!["rev1", "me"]);
    }

    #[test]
    fn pull_request_lists_follow_full_pages() {
        let full = format!(
            "[{}]",
            (0..PAGE_SIZE)
                .map(|n| pull(n as u64 + 1, "open", ""))
                .collect::<Vec<_>>()
                .join(",")
        );
        let transport = Arc::new(RecordedTransport::new().on("&page=1", 200, &full).on(
            "&page=2",
            200,
            &format!("[{}]", pull(500, "open", "")),
        ));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let prs = block_on(c.list_pull_requests("me/thing", &PrFilter::default())).unwrap();
        assert_eq!(prs.len(), PAGE_SIZE as usize + 1);
        assert_eq!(transport.requests().len(), 2);
    }

    #[test]
    fn mine_filters_by_the_users_login_and_review_requested_uses_search() {
        let mut other = pull(2, "open", "");
        other = other.replace(
            r#""user":{"login":"octo"}"#,
            r#""user":{"login":"someone"}"#,
        );
        let body = format!("[{},{other}]", pull(1, "open", ""));
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/search/issues", 200, r#"{"items":[{"number":2}]}"#)
                .on("/pulls?", 200, &body),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );

        let mine = PrFilter {
            mine: true,
            ..Default::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &mine)).unwrap();
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 1);

        let review = PrFilter {
            review_requested: true,
            ..Default::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &review)).unwrap();
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 2);
        assert!(prs[0].review_requested);
        let search = transport
            .requests()
            .into_iter()
            .find(|r| r.url.contains("/search/issues"))
            .unwrap();
        assert!(
            search.url.contains("review-requested%3A%40me"),
            "{}",
            search.url
        );
    }

    #[test]
    fn the_pull_request_for_a_branch_prefers_open_over_merged() {
        let merged = pull(5, "closed", r#","merged_at":"2026-01-05T00:00:00Z""#);
        let open = pull(2, "open", "");
        let transport =
            Arc::new(RecordedTransport::new().on("/pulls?", 200, &format!("[{merged},{open}]")));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let found = block_on(c.pull_request_for_branch("me/thing", "feature/x"))
            .unwrap()
            .unwrap();
        assert_eq!(found.number, 2);
        let url = &transport.requests()[0].url;
        assert!(url.contains("head=me%3Afeature%2Fx"), "{url}");

        let empty = Arc::new(RecordedTransport::new().on("/pulls?", 200, "[]"));
        let c = client(IntegrationKind::Github, "https://github.com", empty);
        assert!(block_on(c.pull_request_for_branch("me/thing", "b"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn issues_drop_pull_request_rows_and_map_fields() {
        let body = r#"[
            {"number":1,"title":"bug","state":"open","html_url":"https://github.com/me/thing/issues/1",
             "updated_at":"2026-01-01T00:00:00Z","user":{"login":"octo"},
             "assignees":[{"login":"octo"}],"labels":[{"name":"bug"}]},
            {"number":2,"title":"a pr","state":"open","html_url":"x","pull_request":{"url":"y"}}
        ]"#;
        let transport = Arc::new(RecordedTransport::new().on("/issues?", 200, body));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let issues = block_on(c.list_issues("me/thing", &IssueFilter::default())).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].iid_label, "#1");
        assert_eq!(issues[0].reference, "#1");
        assert_eq!(issues[0].id, "1");
        assert_eq!(issues[0].kind_label, None);
        assert_eq!(issues[0].labels, vec!["bug"]);
        assert_eq!(issues[0].assignees, vec!["octo"]);
        assert_eq!(issues[0].state, IssueState::Open);
    }

    #[test]
    fn assigned_to_me_passes_the_login() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/issues?", 200, "[]"),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let filter = IssueFilter {
            assigned_to_me: true,
            ..Default::default()
        };
        block_on(c.list_issues("me/thing", &filter)).unwrap();
        assert!(transport
            .requests()
            .iter()
            .any(|r| r.url.contains("assignee=octo")));
    }

    #[test]
    fn creating_an_issue_posts_title_body_and_labels() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/repos/me/thing/issues",
            201,
            r#"{"number":12,"html_url":"https://github.com/me/thing/issues/12"}"#,
        ));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let created = block_on(c.create_issue(&NewIssue {
            project: "me/thing".into(),
            title: "It breaks".into(),
            body: "Steps".into(),
            labels: vec!["bug".into()],
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(created.number, 12);
        assert_eq!(created.reference, "#12");
        assert_eq!(created.web_url, "https://github.com/me/thing/issues/12");
        let request = &transport.requests()[0];
        assert_eq!(request.method, crate::http::Method::Post);
        assert_eq!(request.url, "https://api.github.com/repos/me/thing/issues");
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["title"], "It breaks");
        assert_eq!(body["body"], "Steps");
        assert_eq!(body["labels"], serde_json::json!(["bug"]));
    }

    #[test]
    fn creating_an_issue_without_labels_omits_them() {
        let transport =
            Arc::new(RecordedTransport::new().on("/issues", 201, r#"{"number":1,"html_url":"u"}"#));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        block_on(c.create_issue(&NewIssue {
            project: "me/thing".into(),
            title: "t".into(),
            ..Default::default()
        }))
        .unwrap();
        let body: serde_json::Value =
            serde_json::from_str(transport.requests()[0].body.as_deref().unwrap()).unwrap();
        assert!(body.get("labels").is_none());
    }

    #[test]
    fn creating_an_issue_surfaces_each_failure_clearly() {
        for (status, body) in [
            (401, r#"{"message":"Bad credentials"}"#),
            (
                403,
                r#"{"message":"Resource not accessible by personal access token"}"#,
            ),
            (404, r#"{"message":"Not Found"}"#),
            (
                422,
                r#"{"message":"Validation Failed","errors":[{"resource":"Issue","field":"title","code":"missing_field"}]}"#,
            ),
        ] {
            let transport = Arc::new(RecordedTransport::new().on("/issues", status, body));
            let c = client(IntegrationKind::Github, "https://github.com", transport);
            let err = block_on(c.create_issue(&NewIssue {
                project: "me/thing".into(),
                title: "t".into(),
                ..Default::default()
            }))
            .unwrap_err();
            match status {
                401 => assert!(matches!(err, Error::Unauthorized { .. })),
                403 => assert!(matches!(err, Error::Forbidden { .. })),
                _ => assert!(matches!(err, Error::Api { status: s, .. } if s == status)),
            }
            let text = err.to_string();
            assert!(text.contains("opening a GitHub issue"), "{text}");
            if status == 422 {
                assert!(
                    text.contains("Validation Failed") && text.contains("title"),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn deleting_a_key_sends_delete_to_the_key_id() {
        let transport = Arc::new(RecordedTransport::new().on("/user/keys/7", 204, ""));
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        block_on(c.delete_ssh_key("7")).unwrap();
        let request = &transport.requests()[0];
        assert_eq!(request.method, crate::http::Method::Delete);
        assert_eq!(request.url, "https://api.github.com/user/keys/7");
    }

    fn pull_with_sha(n: u64, login: &str, sha: &str) -> String {
        pull(n, "open", "")
            .replace(
                r#""user":{"login":"octo"}"#,
                &format!(r#""user":{{"login":"{login}"}}"#),
            )
            .replace(
                r#""head":{"ref":"feature","#,
                &format!(r#""head":{{"sha":"{sha}","ref":"feature","#),
            )
    }

    #[test]
    fn checks_and_conflicts_are_asked_per_request_only_on_demand() {
        let list = format!(
            "[{},{}]",
            pull_with_sha(1, "someone", "aaa"),
            pull_with_sha(2, "octo", "bbb")
        );
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/pulls?", 200, &list)
                .on("/pulls/1", 200, r#"{"mergeable_state":"dirty"}"#)
                .on("/pulls/2", 200, r#"{"mergeable_state":"clean"}"#)
                .on(
                    "/commits/aaa/status",
                    200,
                    r#"{"state":"pending","total_count":0}"#,
                )
                .on(
                    "/commits/aaa/check-runs",
                    200,
                    r#"{"check_runs":[{"status":"completed","conclusion":"failure"}]}"#,
                )
                .on(
                    "/commits/bbb/status",
                    200,
                    r#"{"state":"success","total_count":1}"#,
                )
                .on(
                    "/commits/bbb/check-runs",
                    200,
                    r#"{"check_runs":[{"status":"completed","conclusion":"success"}]}"#,
                ),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        // Not asked: nothing beyond the list, nothing filled.
        let plain = block_on(c.list_pull_requests("me/thing", &PrFilter::default())).unwrap();
        assert_eq!(transport.requests().len(), 1);
        assert!(plain
            .iter()
            .all(|p| p.checks.is_none() && p.has_conflicts.is_none()));

        let filter = PrFilter {
            checks: true,
            ..PrFilter::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &filter)).unwrap();
        assert_eq!(prs[0].checks, Some(ChecksState::Failing));
        assert_eq!(prs[0].has_conflicts, Some(true));
        assert_eq!(prs[1].checks, Some(ChecksState::Passing));
        assert_eq!(prs[1].has_conflicts, Some(false));
        // The user's own request (#2) is asked first.
        let urls: Vec<String> = transport.requests().into_iter().map(|r| r.url).collect();
        let first_detail = urls.iter().position(|u| u.contains("/pulls/")).unwrap();
        assert!(urls[first_detail].ends_with("/pulls/2"), "{urls:?}");
    }

    #[test]
    fn the_fan_out_is_bounded_and_a_failing_call_ends_it_quietly() {
        let list = format!(
            "[{}]",
            (1..=(STATUS_FANOUT as u64 + 10))
                .map(|n| pull_with_sha(n, "octo", &format!("s{n}")))
                .collect::<Vec<_>>()
                .join(",")
        );
        // Every detail call is a rate limit: the list still succeeds.
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/pulls?", 200, &list)
                .on("/pulls/", 403, r#"{"message":"rate limit"}"#),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let filter = PrFilter {
            checks: true,
            ..PrFilter::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &filter)).unwrap();
        assert_eq!(prs.len(), STATUS_FANOUT + 10);
        assert!(prs.iter().all(|p| p.checks.is_none()));
        let details = transport
            .requests()
            .iter()
            .filter(|r| r.url.contains("/pulls/"))
            .count();
        assert_eq!(details, 1, "the first refusal ends the fan-out");
    }

    #[test]
    fn the_fan_out_asks_at_most_the_bound() {
        let list = format!(
            "[{}]",
            (1..=(STATUS_FANOUT as u64 + 10))
                .map(|n| pull_with_sha(n, "octo", &format!("s{n}")))
                .collect::<Vec<_>>()
                .join(",")
        );
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/pulls?", 200, &list)
                .on("/pulls/", 200, r#"{"mergeable_state":"clean"}"#)
                .on("/status", 200, r#"{"state":"success","total_count":1}"#)
                .on("/check-runs", 200, r#"{"check_runs":[]}"#),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let filter = PrFilter {
            checks: true,
            ..PrFilter::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &filter)).unwrap();
        assert_eq!(
            prs.iter().filter(|p| p.checks.is_some()).count(),
            STATUS_FANOUT
        );
        let details = transport
            .requests()
            .iter()
            .filter(|r| r.url.contains("/pulls/"))
            .count();
        assert_eq!(details, STATUS_FANOUT);
    }

    #[test]
    fn checks_roll_up_failure_then_pending_then_passing() {
        use ChecksState::*;
        assert_eq!(
            roll_up_checks("success", 2, &[("completed", Some("success"))]),
            Some(Passing)
        );
        assert_eq!(roll_up_checks("failure", 1, &[]), Some(Failing));
        assert_eq!(
            roll_up_checks(
                "success",
                1,
                &[("in_progress", None), ("completed", Some("failure"))]
            ),
            Some(Failing)
        );
        assert_eq!(
            roll_up_checks("success", 1, &[("queued", None)]),
            Some(Pending)
        );
        assert_eq!(roll_up_checks("pending", 2, &[]), Some(Pending));
        // GitHub says "pending" for a commit with no statuses at all: no CI.
        assert_eq!(roll_up_checks("pending", 0, &[]), None);
        assert_eq!(
            roll_up_checks("pending", 0, &[("completed", Some("skipped"))]),
            Some(Passing)
        );
    }

    #[test]
    fn mentioned_asks_the_search_api_and_keeps_only_those_requests() {
        let list = format!("[{},{}]", pull(1, "open", ""), pull(2, "open", ""));
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/pulls?", 200, &list)
                .on("/search/issues", 200, r#"{"items":[{"number":2}]}"#),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let filter = PrFilter {
            mentioned: true,
            ..PrFilter::default()
        };
        let prs = block_on(c.list_pull_requests("me/thing", &filter)).unwrap();
        assert_eq!(prs.iter().map(|p| p.number).collect::<Vec<_>>(), vec![2]);
        let search = transport
            .requests()
            .into_iter()
            .find(|r| r.url.contains("/search/issues"))
            .unwrap();
        assert!(search.url.contains("is%3Apr"), "{}", search.url);
        assert!(search.url.contains("mentions%3Aocto"), "{}", search.url);
        assert!(search.url.contains("repo%3Ame%2Fthing"), "{}", search.url);
    }

    #[test]
    fn mentioned_issues_search_issues_not_requests() {
        let issues = r#"[
            {"number":5,"title":"a","state":"open","html_url":"u5","updated_at":"x","user":{"login":"o"}},
            {"number":6,"title":"b","state":"open","html_url":"u6","updated_at":"x","user":{"login":"o"}}
        ]"#;
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/thing/issues?", 200, issues)
                .on("/search/issues", 200, r#"{"items":[{"number":6}]}"#),
        );
        let c = client(
            IntegrationKind::Github,
            "https://github.com",
            transport.clone(),
        );
        let filter = IssueFilter {
            mentioned: true,
            ..IssueFilter::default()
        };
        let found = block_on(c.list_issues("me/thing", &filter)).unwrap();
        assert_eq!(found.iter().map(|i| i.number).collect::<Vec<_>>(), vec![6]);
        let search = transport
            .requests()
            .into_iter()
            .find(|r| r.url.contains("/search/issues"))
            .unwrap();
        assert!(search.url.contains("is%3Aissue"), "{}", search.url);
    }
}
