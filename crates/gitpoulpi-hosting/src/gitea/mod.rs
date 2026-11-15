//! Gitea and Forgejo (incl. Codeberg) hosting client (integrations wave 2,
//! slice G). One client for both kinds: Forgejo is a Gitea fork whose REST
//! API (`/api/v1`) stayed compatible.
//!
//! Verified 2026-10-14 against a Gitea (swagger 28.1.0) and a Forgejo 11
//! (`11.0.16+gitea-1.22.0`) container, plus their `swagger.v1.json`:
//!
//! - Auth `Authorization: token <t>`. `GET /user` has no scopes header, so
//!   a token check reports the scopes as unknown, never as missing.
//! - Paging is `limit` (at most 50 by default) and `page`; `X-HasMore` and
//!   `X-Total-Count` say whether to go on.
//! - A pull request is an issue: assignees, labels and the milestone are
//!   edited through `/issues/{n}` (`PATCH` for assignees and the milestone
//!   *id*, `PUT .../labels` with names), reviewers through
//!   `/pulls/{n}/requested_reviewers` (same body as GitHub).
//! - Create takes label *ids*, so names are resolved through the labels
//!   listing first.
//! - No draft flag on create: the title prefix `WIP:` is the draft.
//! - Review threads, replies and resolution live in [`review`]: no thread
//!   ids exist in the API, so a thread is the comments of one line.

#[cfg(test)]
mod code_tests;
mod conversation;
#[cfg(test)]
mod conversation_tests;
mod flow;
mod meta;
mod review;
mod review_code;
pub mod spec;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::client::{
    CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest, MAX_PAGES,
};
use crate::conversation::{
    Comment, ConversationCaps, ItemKey, ItemKind, PageCursor, PostingIdentity, TimelineOpts,
    TimelinePage,
};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::{
    best_pull_request, ChecksState, HostedUser, IssueFilter, IssueState, IssueSummary, PrFilter,
    PullRequestState, PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

/// What one page asks for: 50 is Gitea's default `MAX_RESPONSE_ITEMS`, a
/// larger ask is clamped silently.
const PAGE: u32 = 50;

/// How many requests of one list get their checks asked.
const STATUS_FANOUT: usize = 15;

pub struct GiteaClient {
    pub(crate) integration: Integration,
    token: String,
    pub(crate) transport: Arc<dyn HttpTransport>,
    pub(crate) scopes: crate::scope_handle::ScopeHandle,
}

impl GiteaClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
            scopes: Default::default(),
        }
    }

    pub(crate) fn authed(&self, request: HttpRequest) -> HttpRequest {
        request
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/json")
    }

    pub(crate) fn get(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::get(url))
    }

    pub(crate) fn post(&self, url: String, body: &serde_json::Value) -> HttpRequest {
        self.authed(HttpRequest::post_json(url, body))
    }

    pub(crate) fn api(&self, path: &str) -> String {
        format!("{}{path}", self.integration.api_base())
    }

    /// `/repos/{project}/{rest}` under the API root.
    pub(crate) fn repo_url(&self, project: &str, rest: &str) -> String {
        self.api(&format!("/repos/{project}/{rest}"))
    }

    /// Every page of a list endpoint. `X-HasMore` decides when the host
    /// sends it; else a full page means "ask again".
    pub(crate) async fn pages<T: DeserializeOwned>(&self, url: &str, what: &str) -> Result<Vec<T>> {
        let sep = if url.contains('?') { '&' } else { '?' };
        let mut out = Vec::new();
        for page in 1..=MAX_PAGES {
            let response = self
                .transport
                .send(self.get(format!("{url}{sep}limit={PAGE}&page={page}")))
                .await?;
            response.error_for_status(what)?;
            let rows: Vec<T> = response.json()?;
            let count = rows.len();
            out.extend(rows);
            let more = match header(&response, "x-hasmore") {
                Some(flag) => flag.eq_ignore_ascii_case("true"),
                None => count >= PAGE as usize,
            };
            if count == 0 || !more {
                break;
            }
        }
        Ok(out)
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: String, what: &str) -> Result<T> {
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status(what)?;
        response.json()
    }

    /// The cached login when the profile has one, else a `/user` call.
    pub(crate) async fn login(&self) -> Result<String> {
        if let Some(user) = &self.integration.user {
            if !user.login.is_empty() {
                return Ok(user.login.clone());
            }
        }
        Ok(self.fetch_user().await?.login)
    }

    async fn fetch_user(&self) -> Result<HostedUser> {
        let row: UserRow = self
            .get_json(self.api("/user"), "checking the token")
            .await?;
        Ok(row.into_user())
    }

    /// Numbers of the issues / pull requests of `project` that the search
    /// filters `extra` (`mentioned_by=...`) select.
    async fn numbers(
        &self,
        project: &str,
        pull_requests: bool,
        state: StateFilter,
        extra: &str,
    ) -> Result<std::collections::HashSet<u64>> {
        #[derive(serde::Deserialize)]
        struct Row {
            number: u64,
        }
        let kind = if pull_requests { "pulls" } else { "issues" };
        let url = self.repo_url(
            project,
            &format!("issues?type={kind}&state={}&{extra}", state_text(state)),
        );
        let rows: Vec<Row> = self.pages(&url, "searching issues").await?;
        Ok(rows.into_iter().map(|r| r.number).collect())
    }

    /// Fills `checks` and `has_conflicts` of live requests (see
    /// `GitHubClient::fill_status`): the list carries `mergeable`, the
    /// commit status costs one call per request, the user's own first.
    async fn fill_status(&self, project: &str, rows: &mut [(PullRequestSummary, String, bool)]) {
        let me = self.login().await.unwrap_or_default();
        let mut order: Vec<usize> = (0..rows.len())
            .filter(|&i| {
                matches!(
                    rows[i].0.state,
                    PullRequestState::Open | PullRequestState::Draft
                )
            })
            .collect();
        // A draft reports `mergeable: false` whatever its branches do.
        for &i in &order {
            if rows[i].0.state == PullRequestState::Open {
                rows[i].0.has_conflicts = Some(!rows[i].2);
            }
        }
        order.retain(|&i| !rows[i].1.is_empty());
        order.sort_by_key(|&i| !rows[i].0.author.eq_ignore_ascii_case(&me));
        order.truncate(STATUS_FANOUT);
        for i in order {
            let Ok(checks) = self.commit_checks(project, &rows[i].1).await else {
                return;
            };
            rows[i].0.checks = checks;
        }
    }

    /// The rolled-up commit status of `sha`: `None` when nothing reported.
    pub(crate) async fn commit_checks(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Option<ChecksState>> {
        #[derive(serde::Deserialize)]
        struct Combined {
            #[serde(default)]
            state: String,
            #[serde(default)]
            total_count: u64,
        }
        let status: Combined = self
            .get_json(
                self.repo_url(project, &format!("commits/{sha}/status")),
                "reading commit statuses",
            )
            .await?;
        Ok(if status.total_count == 0 {
            None
        } else {
            match status.state.as_str() {
                "success" | "warning" => Some(ChecksState::Passing),
                "failure" | "error" => Some(ChecksState::Failing),
                "pending" => Some(ChecksState::Pending),
                _ => None,
            }
        })
    }

    /// Label ids for `names` (create endpoints take ids, not names).
    async fn label_ids(&self, project: &str, names: &[String]) -> Result<Vec<i64>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let known = self.meta_label_rows(project).await?;
        names
            .iter()
            .map(|name| {
                known
                    .iter()
                    .find(|l| l.name.eq_ignore_ascii_case(name))
                    .map(|l| l.id)
                    .ok_or_else(|| Error::Api {
                        what: "resolving a label".to_string(),
                        status: 422,
                        detail: format!("`{name}` is not a label of {project}"),
                    })
            })
            .collect()
    }

    /// `Gitea` or `Forgejo`, for messages.
    fn label(&self) -> &'static str {
        self.integration.kind.label()
    }
}

pub(crate) fn header(response: &HttpResponse, name: &str) -> Option<String> {
    response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn state_text(state: StateFilter) -> &'static str {
    match state {
        StateFilter::Open => "open",
        StateFilter::Closed => "closed",
        StateFilter::All => "all",
    }
}

fn is_wip(title: &str) -> bool {
    let t = title.trim_start().to_ascii_lowercase();
    t.starts_with("wip:") || t.starts_with("[wip]")
}

#[derive(serde::Deserialize)]
pub(crate) struct UserRow {
    pub(crate) login: String,
    #[serde(default)]
    id: i64,
    #[serde(default)]
    full_name: String,
    html_url: Option<String>,
}

impl UserRow {
    pub(crate) fn into_user(self) -> HostedUser {
        HostedUser {
            login: self.login,
            name: self.full_name,
            id: self.id.to_string(),
            web_url: self.html_url,
        }
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct Login {
    pub(crate) login: String,
}

#[derive(serde::Deserialize)]
struct RepoRow {
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
struct KeyRow {
    id: i64,
    #[serde(default)]
    title: String,
    key: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct RefRow {
    #[serde(rename = "ref")]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) sha: String,
    pub(crate) repo: Option<RefRepo>,
}

#[derive(serde::Deserialize)]
pub(crate) struct RefRepo {
    pub(crate) full_name: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct PullRow {
    pub(crate) number: u64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    merged: bool,
    merged_at: Option<String>,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    user: Option<Login>,
    pub(crate) head: RefRow,
    pub(crate) base: RefRow,
    assignees: Option<Vec<Login>>,
    requested_reviewers: Option<Vec<Login>>,
    pub(crate) mergeable: Option<bool>,
}

impl PullRow {
    pub(crate) fn into_summary(self, me: Option<&str>) -> PullRequestSummary {
        let state = if self.merged || self.merged_at.is_some() {
            PullRequestState::Merged
        } else if self.state == "closed" {
            PullRequestState::Closed
        } else if self.draft || is_wip(&self.title) {
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
        let reviewers: Vec<String> = self
            .requested_reviewers
            .unwrap_or_default()
            .into_iter()
            .map(|u| u.login)
            .collect();
        let review_requested =
            me.is_some_and(|me| reviewers.iter().any(|r| r.eq_ignore_ascii_case(me)));
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
            assignees: self
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(|u| u.login)
                .collect(),
            reviewers,
            checks: None,
            has_conflicts: None,
        }
    }
}

#[derive(serde::Deserialize)]
struct IssueRow {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    user: Option<Login>,
    assignees: Option<Vec<Login>>,
    labels: Option<Vec<LabelName>>,
    pull_request: Option<serde_json::Value>,
    milestone: Option<crate::model::MilestoneTitle>,
}

#[derive(serde::Deserialize)]
struct LabelName {
    name: String,
}

#[derive(serde::Deserialize)]
struct Created {
    number: u64,
    html_url: String,
}

#[async_trait::async_trait]
impl HostingClient for GiteaClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let url = self.api("/user/repos");
        let repos: Vec<RepoRow> = self
            .pages(&url, &format!("listing {} repositories", self.label()))
            .await?;
        Ok(repos
            .into_iter()
            .map(|r| HostedProject {
                integration_id: self.integration.id.clone(),
                id: r.full_name.clone(),
                name: r.name,
                full_name: r.full_name,
                description: r.description.filter(|d| !d.is_empty()),
                default_branch: r.default_branch,
                ssh_url: r.ssh_url.filter(|u| !u.is_empty()),
                https_url: r.clone_url,
                web_url: r.html_url,
                private: r.private,
            })
            .collect())
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        let keys: Vec<KeyRow> = self
            .pages(&self.api("/user/keys"), "listing SSH keys")
            .await?;
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
        let body = serde_json::json!({ "title": title, "key": public_key.trim() });
        let response = self
            .transport
            .send(self.post(self.api("/user/keys"), &body))
            .await?;
        response.error_for_status("uploading an SSH key")?;
        let key: KeyRow = response.json()?;
        Ok(HostedSshKey {
            id: key.id.to_string(),
            title: key.title,
            key: key.key,
        })
    }

    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        let request = self.authed(HttpRequest::delete(self.api(&format!("/user/keys/{id}"))));
        let response = self.transport.send(request).await?;
        response.error_for_status("deleting an SSH key")
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        let head = match req.fork_source() {
            Some(fork) => format!(
                "{}:{}",
                fork.split('/').next().unwrap_or(fork),
                req.source_branch
            ),
            None => req.source_branch.clone(),
        };
        // No draft flag on create: a `WIP:` title is what marks one.
        let title = if req.draft && !is_wip(&req.title) {
            format!("WIP: {}", req.title)
        } else {
            req.title.clone()
        };
        let mut body = serde_json::json!({
            "title": title,
            "head": head,
            "base": req.target_branch,
            "body": req.body,
        });
        if !req.assignees.is_empty() {
            body["assignees"] = serde_json::json!(req.assignees);
        }
        if !req.labels.is_empty() {
            body["labels"] = serde_json::json!(self.label_ids(&req.project, &req.labels).await?);
        }
        if let Some(milestone) = req.milestone {
            body["milestone"] = serde_json::json!(milestone);
        }
        let url = self.repo_url(&req.project, "pulls");
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening a pull request")?;
        let pr: Created = response.json()?;
        // Reviewers are not part of the create call.
        let warning = if req.reviewers.is_empty() {
            None
        } else {
            self.change_reviewers(&req.project, pr.number, &req.reviewers, &[])
                .await
                .err()
                .map(|e| {
                    format!("the pull request was opened, but its reviewers could not be set: {e}")
                })
        };
        Ok(CreatedPullRequest {
            number: pr.number,
            web_url: pr.html_url,
            warning,
        })
    }

    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        crate::account::gitea::fetch(self, query).await
    }

    async fn current_user(&self) -> Result<HostedUser> {
        self.fetch_user().await
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        // `/user` sends no scopes header: unknown, never "missing".
        Ok(TokenCheck {
            user: self.fetch_user().await?,
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
            ..Default::default()
        })
    }

    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        let url = self.repo_url(
            project,
            &format!("pulls?state={}&sort=recentupdate", state_text(filter.state)),
        );
        let pulls: Vec<PullRow> = self
            .pages(&url, &format!("listing {} pull requests", self.label()))
            .await?;
        let me = if filter.mine || filter.review_requested {
            Some(self.login().await?)
        } else {
            self.integration
                .user
                .as_ref()
                .map(|u| u.login.clone())
                .filter(|l| !l.is_empty())
        };
        let mentioning = if filter.mentioned {
            let login = self.login().await?;
            Some(
                self.numbers(
                    project,
                    true,
                    filter.state,
                    &format!("mentioned_by={}", crate::client::encode_component(&login)),
                )
                .await?,
            )
        } else {
            None
        };
        let mut rows: Vec<(PullRequestSummary, String, bool)> = pulls
            .into_iter()
            .map(|p| {
                let sha = p.head.sha.clone();
                let mergeable = p.mergeable.unwrap_or(true);
                (p.into_summary(me.as_deref()), sha, mergeable)
            })
            .filter(|(p, _, _)| !filter.review_requested || p.review_requested)
            .filter(|(p, _, _)| {
                !filter.mine
                    || me
                        .as_deref()
                        .is_some_and(|me| p.author.eq_ignore_ascii_case(me))
            })
            .filter(|(p, _, _)| mentioning.as_ref().is_none_or(|s| s.contains(&p.number)))
            .collect();
        if filter.checks {
            self.fill_status(project, &mut rows).await;
        }
        Ok(rows.into_iter().map(|(p, _, _)| p).collect())
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        // The pulls listing has no head filter: read it newest first and
        // keep the requests whose source branch is `branch`.
        let url = self.repo_url(project, "pulls?state=all&sort=recentupdate");
        let pulls: Vec<PullRow> = self
            .pages(&url, "looking up the pull request for a branch")
            .await?;
        Ok(best_pull_request(
            pulls
                .into_iter()
                .filter(|p| p.head.name == branch)
                .map(|p| p.into_summary(None))
                .collect(),
        ))
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let mut url = self.repo_url(
            project,
            &format!("issues?type=issues&state={}", state_text(filter.state)),
        );
        if filter.assigned_to_me {
            url.push_str(&format!(
                "&assigned_by={}",
                crate::client::encode_component(&self.login().await?)
            ));
        }
        if filter.mentioned {
            url.push_str(&format!(
                "&mentioned_by={}",
                crate::client::encode_component(&self.login().await?)
            ));
        }
        let rows: Vec<IssueRow> = self
            .pages(&url, &format!("listing {} issues", self.label()))
            .await?;
        Ok(rows
            .into_iter()
            .filter(|r| r.pull_request.is_none())
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
                assignees: r
                    .assignees
                    .unwrap_or_default()
                    .into_iter()
                    .map(|u| u.login)
                    .collect(),
                labels: r
                    .labels
                    .unwrap_or_default()
                    .into_iter()
                    .map(|l| l.name)
                    .collect(),
                web_url: r.html_url,
                updated_at: r.updated_at,
                milestone: r.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
                ..IssueSummary::default()
            })
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let mut body = serde_json::json!({ "title": req.title, "body": req.body });
        if !req.labels.is_empty() {
            body["labels"] = serde_json::json!(self.label_ids(&req.project, &req.labels).await?);
        }
        if !req.assignees.is_empty() {
            body["assignees"] = serde_json::json!(req.assignees);
        }
        if let Some(milestone) = req.milestone {
            body["milestone"] = serde_json::json!(milestone);
        }
        let url = self.repo_url(&req.project, "issues");
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening an issue")?;
        let issue: Created = response.json()?;
        Ok(CreatedIssue {
            number: issue.number,
            web_url: issue.html_url,
            reference: format!("#{}", issue.number),
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
        _project: &str,
        _number: u64,
    ) -> Result<crate::issue_flow::IssueLinks> {
        // Gitea links nothing to an issue by itself: the caller matches
        // branches and requests by the naming pattern.
        Ok(crate::issue_flow::IssueLinks::default())
    }

    async fn branch_names(&self, project: &str, search: &str) -> Result<Vec<String>> {
        self.flow_branch_names(project, search).await
    }

    fn review_capabilities(&self) -> crate::review::ReviewCapabilities {
        self.review_caps()
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
        project: &str,
        number: u64,
        thread_id: &str,
        body: &str,
    ) -> Result<crate::review::ReviewComment> {
        self.review_reply(project, number, thread_id, body).await
    }

    async fn resolve_thread(
        &self,
        project: &str,
        _number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        self.review_resolve(project, thread_id, resolved).await
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

    fn conversation_caps(&self, kind: ItemKind) -> ConversationCaps {
        ConversationCaps::gitea(kind, self.review_caps().resolve)
    }

    async fn posting_identity(&self, project: &str) -> Result<PostingIdentity> {
        self.conv_identity(project).await
    }

    async fn item_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        self.conv_timeline(project, key, older, opts).await
    }

    async fn post_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        self.conv_post(project, key, body, internal).await
    }

    async fn reply_in_discussion(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.conv_reply(project, key, discussion_id, body).await
    }

    async fn edit_item_comment(
        &self,
        project: &str,
        _key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.conv_edit(project, comment_id, body).await
    }

    async fn delete_item_comment(
        &self,
        project: &str,
        _key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        self.conv_delete(project, comment_id).await
    }
}
