//! The GitLab client (gitlab.com and self-managed).

use std::sync::Arc;

use crate::client::{
    fetch_pages, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::{
    best_pull_request, missing_scopes, ChecksState, HostedUser, IssueFilter, IssueState,
    IssueSummary, PrFilter, PullRequestState, PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::Result;

const PAGE_SIZE: u32 = crate::client::PAGE_SIZE;

mod protection;

pub struct GitLabClient {
    pub(crate) integration: Integration,
    token: String,
    pub(crate) transport: std::sync::Arc<dyn HttpTransport>,
}

impl GitLabClient {
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

    /// GitLab accepts a personal/project/group access token in
    /// `PRIVATE-TOKEN`, and an OAuth token in `Authorization: Bearer`.
    /// Both are sent: a PAT in the Bearer header is rejected by older
    /// self-managed versions, and `PRIVATE-TOKEN` is ignored rather than
    /// rejected when an OAuth token is in play.
    pub(crate) fn authed(&self, request: HttpRequest) -> HttpRequest {
        request
            .header("PRIVATE-TOKEN", self.token.clone())
            .header("Authorization", format!("Bearer {}", self.token))
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

    pub(crate) fn put(&self, url: String, body: &serde_json::Value) -> HttpRequest {
        self.authed(HttpRequest::put_json(url, body))
    }
}

#[derive(serde::Deserialize)]
struct GitLabProject {
    id: i64,
    name: String,
    path_with_namespace: String,
    description: Option<String>,
    default_branch: Option<String>,
    ssh_url_to_repo: Option<String>,
    http_url_to_repo: Option<String>,
    web_url: Option<String>,
    visibility: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitLabKey {
    id: i64,
    title: String,
    key: String,
}

#[derive(serde::Deserialize)]
struct GitLabProjectId {
    id: i64,
}

#[derive(serde::Deserialize)]
struct GitLabMergeRequest {
    iid: u64,
    web_url: String,
}

/// GitLab's project-id path segment must be its own percent-encoding, not
/// the API's own `/`-joined route: `group/sub/proj` becomes
/// `group%2Fsub%2Fproj`, the documented way to address a project by path
/// rather than by its numeric id.
pub(crate) fn encode_gitlab_project_path(project: &str) -> String {
    project.replace('/', "%2F")
}

#[async_trait::async_trait]
impl HostingClient for GitLabClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        // `membership=true` is the GitLab equivalent of GitHub's
        // affiliation filter: without it, `/projects` is *every public
        // project on the instance*.
        let url = format!(
            "{}/projects?membership=true&simple=true&order_by=last_activity_at",
            self.integration.api_base()
        );
        let projects: Vec<GitLabProject> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab projects",
        )
        .await?;
        Ok(projects
            .into_iter()
            .map(|p| HostedProject {
                integration_id: self.integration.id.clone(),
                id: p.id.to_string(),
                name: p.name,
                full_name: p.path_with_namespace,
                description: p.description,
                default_branch: p.default_branch,
                ssh_url: p.ssh_url_to_repo,
                https_url: p.http_url_to_repo,
                web_url: p.web_url,
                private: p.visibility.as_deref() != Some("public"),
            })
            .collect())
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        let url = format!(
            "{}/user/keys?per_page={PAGE_SIZE}",
            self.integration.api_base()
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("listing GitLab SSH keys")?;
        let keys: Vec<GitLabKey> = response.json()?;
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
        response.error_for_status("uploading an SSH key to GitLab")?;
        let key: GitLabKey = response.json()?;
        Ok(HostedSshKey {
            id: key.id.to_string(),
            title: key.title,
            key: key.key,
        })
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        // A merge request is created *from the source project*, naming the
        // target project's numeric id when they differ (a fork).
        let source_path = req.fork_source().unwrap_or(&req.project);
        let target_project_id = if req.fork_source().is_some() {
            let lookup = format!(
                "{}/projects/{}",
                self.integration.api_base(),
                encode_gitlab_project_path(&req.project)
            );
            let response = self.transport.send(self.get(lookup)).await?;
            response.error_for_status("looking up the target GitLab project")?;
            let target: GitLabProjectId = response.json()?;
            Some(target.id)
        } else {
            None
        };
        let url = format!(
            "{}/projects/{}/merge_requests",
            self.integration.api_base(),
            encode_gitlab_project_path(source_path)
        );
        // GitLab has no `draft` field of its own -- a `Draft: ` title
        // prefix is the documented way to mark one, and its own UI
        // strips it back off again when it renders the row.
        let title = if req.draft {
            format!("Draft: {}", req.title)
        } else {
            req.title.clone()
        };
        let mut body = serde_json::json!({
            "source_branch": req.source_branch,
            "target_branch": req.target_branch,
            "title": title,
            "description": req.body,
        });
        if let Some(id) = target_project_id {
            body["target_project_id"] = serde_json::json!(id);
        }
        // People are chosen from the target project's members.
        let extra = self
            .new_item_fields(
                &req.project,
                &req.assignees,
                &req.reviewers,
                &req.labels,
                req.milestone,
            )
            .await?;
        for (key, value) in extra {
            body[key] = value;
        }
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening a GitLab merge request")?;
        let mr: GitLabMergeRequest = response.json()?;
        Ok(CreatedPullRequest {
            number: mr.iid,
            web_url: mr.web_url,
            warning: None,
        })
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let url = format!(
            "{}/projects/{}/issues",
            self.integration.api_base(),
            encode_gitlab_project_path(&req.project)
        );
        let mut body = serde_json::json!({ "title": req.title, "description": req.body });
        if !req.labels.is_empty() {
            // GitLab takes labels as one comma-separated string.
            body["labels"] = serde_json::json!(req.labels.join(","));
        }
        let extra = self
            .new_item_fields(&req.project, &req.assignees, &[], &[], req.milestone)
            .await?;
        for (key, value) in extra {
            body[key] = value;
        }
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("opening a GitLab issue")?;
        let issue: GitLabCreatedIssue = response.json()?;
        Ok(CreatedIssue {
            number: issue.iid,
            web_url: issue.web_url,
            reference: format!("#{}", issue.iid),
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
        crate::account::gitlab::fetch(self, query).await
    }

    async fn current_user(&self) -> Result<HostedUser> {
        self.fetch_user().await
    }

    async fn protected_branches(
        &self,
        project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        self.protection(project).await
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        let user = self.fetch_user().await?;
        let url = format!(
            "{}/personal_access_tokens/self",
            self.integration.api_base()
        );
        let response = self.transport.send(self.get(url)).await?;
        // 404 is an older GitLab (or a non-PAT token): scopes unknown,
        // which is not the same as missing.
        let (scopes, expires_at) = if response.status == 404 {
            (None, None)
        } else {
            response.error_for_status("reading the GitLab token's scopes")?;
            let row: GitLabTokenSelf = response.json()?;
            (Some(row.scopes), row.expires_at)
        };
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
            expires_at,
        })
    }

    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        if filter.mentioned {
            return self.mentioned_pull_requests(project, filter).await;
        }
        let state = match filter.state {
            StateFilter::Open => "opened",
            // "closed" alone would hide merged requests, which the other
            // host's closed list includes; drop the open ones locally.
            StateFilter::Closed | StateFilter::All => "all",
        };
        let mut url = format!(
            "{}/projects/{}/merge_requests?state={state}&order_by=updated_at&sort=desc",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        if filter.mine {
            url.push_str(&format!(
                "&author_username={}",
                crate::client::encode_component(&self.login().await?)
            ));
        }
        if filter.review_requested {
            url.push_str(&format!(
                "&reviewer_username={}",
                crate::client::encode_component(&self.login().await?)
            ));
        }
        let rows: Vec<GitLabMrRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab merge requests",
        )
        .await?;
        let closed_only = filter.state == StateFilter::Closed;
        Ok(rows
            .into_iter()
            .map(|r| r.into_summary(filter.review_requested))
            .filter(|s| {
                !closed_only
                    || matches!(s.state, PullRequestState::Merged | PullRequestState::Closed)
            })
            .collect())
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let url = format!(
            "{}/projects/{}/merge_requests?state=all&source_branch={}",
            self.integration.api_base(),
            encode_gitlab_project_path(project),
            crate::client::encode_component(branch)
        );
        let rows: Vec<GitLabMrRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "looking up the merge request for a branch",
        )
        .await?;
        Ok(best_pull_request(
            rows.into_iter().map(|r| r.into_summary(false)).collect(),
        ))
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        if filter.mentioned {
            return self.mentioned_issues(project, filter).await;
        }
        let state = match filter.state {
            StateFilter::Open => "opened",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        };
        let mut url = format!(
            "{}/projects/{}/issues?state={state}&order_by=updated_at&sort=desc",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        if filter.assigned_to_me {
            url.push_str(&format!(
                "&assignee_username={}",
                crate::client::encode_component(&self.login().await?)
            ));
        }
        let rows: Vec<GitLabIssueRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab issues",
        )
        .await?;
        Ok(rows.into_iter().map(issue_summary).collect())
    }

    fn review_capabilities(&self) -> crate::review::ReviewCapabilities {
        crate::review::gitlab::CAPABILITIES
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
        number: u64,
        thread_id: &str,
        resolved: bool,
    ) -> Result<()> {
        self.review_resolve(project, number, thread_id, resolved)
            .await
    }

    async fn submit_review(
        &self,
        project: &str,
        number: u64,
        review: &crate::review::ReviewSubmission,
    ) -> Result<crate::review::ReviewOutcome> {
        self.review_submit(project, number, review).await
    }

    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        let url = format!("{}/user/keys/{id}", self.integration.api_base());
        let response = self.transport.send(self.delete(url)).await?;
        response.error_for_status("deleting a GitLab SSH key")
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
}

/// GitLab has no "mentions me" filter on the lists; its to-do list does
/// (`action=mentioned`), one entry per mention with the whole item inside.
/// Only the pending to-dos: one marked done is a mention already dealt with.
impl GitLabClient {
    async fn mentioned_todos<T: serde::de::DeserializeOwned>(
        &self,
        target_type: &str,
        project: &str,
        what: &str,
    ) -> Result<Vec<T>> {
        let url = format!(
            "{}/todos?action=mentioned&state=pending&type={target_type}",
            self.integration.api_base()
        );
        let todos: Vec<GitLabTodo<T>> =
            fetch_pages(&*self.transport, |u| self.get(u), &url, what).await?;
        Ok(todos
            .into_iter()
            .filter(|t| {
                t.project
                    .as_ref()
                    .is_some_and(|p| p.path_with_namespace.eq_ignore_ascii_case(project))
            })
            .map(|t| t.target)
            .collect())
    }

    async fn mentioned_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        let rows: Vec<GitLabMrRow> = self
            .mentioned_todos(
                "MergeRequest",
                project,
                "listing GitLab to-dos that mention you",
            )
            .await?;
        let mut out: Vec<PullRequestSummary> = rows
            .into_iter()
            .map(|r| r.into_summary(false))
            .filter(|s| match filter.state {
                StateFilter::Open => {
                    matches!(s.state, PullRequestState::Open | PullRequestState::Draft)
                }
                StateFilter::Closed => {
                    matches!(s.state, PullRequestState::Merged | PullRequestState::Closed)
                }
                StateFilter::All => true,
            })
            .collect();
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out)
    }

    async fn mentioned_issues(
        &self,
        project: &str,
        filter: &IssueFilter,
    ) -> Result<Vec<IssueSummary>> {
        let rows: Vec<GitLabIssueRow> = self
            .mentioned_todos("Issue", project, "listing GitLab to-dos that mention you")
            .await?;
        let mut out: Vec<IssueSummary> = rows
            .into_iter()
            .map(issue_summary)
            .filter(|i| match filter.state {
                StateFilter::Open => i.state == IssueState::Open,
                StateFilter::Closed => i.state == IssueState::Closed,
                StateFilter::All => true,
            })
            .collect();
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out)
    }

    async fn fetch_user(&self) -> Result<HostedUser> {
        let url = format!("{}/user", self.integration.api_base());
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("checking the GitLab token")?;
        let row: GitLabUserRow = response.json()?;
        Ok(HostedUser {
            login: row.username,
            name: row.name.unwrap_or_default(),
            id: row.id.to_string(),
            web_url: row.web_url,
        })
    }

    pub(crate) async fn login(&self) -> Result<String> {
        if let Some(user) = &self.integration.user {
            if !user.login.is_empty() {
                return Ok(user.login.clone());
            }
        }
        Ok(self.fetch_user().await?.login)
    }
}

#[derive(serde::Deserialize)]
struct GitLabUserRow {
    id: i64,
    username: String,
    name: Option<String>,
    web_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitLabTokenSelf {
    #[serde(default)]
    scopes: Vec<String>,
    expires_at: Option<String>,
}

#[derive(serde::Deserialize)]
struct GitLabUsername {
    username: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct GitLabMrRow {
    iid: u64,
    title: String,
    state: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    work_in_progress: bool,
    source_branch: String,
    target_branch: String,
    source_project_id: Option<i64>,
    target_project_id: Option<i64>,
    author: Option<GitLabUsername>,
    web_url: String,
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    assignees: Vec<GitLabUsername>,
    #[serde(default)]
    reviewers: Vec<GitLabUsername>,
    #[serde(default)]
    has_conflicts: Option<bool>,
    #[serde(default)]
    head_pipeline: Option<GitLabPipeline>,
}

#[derive(serde::Deserialize)]
struct GitLabPipeline {
    #[serde(default)]
    status: String,
}

/// A pipeline status as a rolled-up state; skipped, manual and canceled runs
/// say nothing about whether the change is good.
fn pipeline_checks(status: &str) -> Option<ChecksState> {
    match status {
        "success" | "warning" => Some(ChecksState::Passing),
        "failed" => Some(ChecksState::Failing),
        "created" | "waiting_for_resource" | "preparing" | "pending" | "running" | "scheduled" => {
            Some(ChecksState::Pending)
        }
        _ => None,
    }
}

impl GitLabMrRow {
    pub(crate) fn into_summary(self, review_requested: bool) -> PullRequestSummary {
        let state = match self.state.as_str() {
            "merged" => PullRequestState::Merged,
            "closed" | "locked" => PullRequestState::Closed,
            _ if self.draft || self.work_in_progress => PullRequestState::Draft,
            _ => PullRequestState::Open,
        };
        // A different source project id means a fork; its path is not in
        // this payload, so the id is what identifies it.
        let source_project = match (self.source_project_id, self.target_project_id) {
            (Some(s), Some(t)) if s != t => Some(s.to_string()),
            _ => None,
        };
        PullRequestSummary {
            number: self.iid,
            iid_label: format!("!{}", self.iid),
            title: self.title,
            state,
            source_branch: self.source_branch,
            target_branch: self.target_branch,
            source_project,
            author: self.author.map(|a| a.username).unwrap_or_default(),
            web_url: self.web_url,
            updated_at: self.updated_at,
            review_requested,
            assignees: self.assignees.into_iter().map(|u| u.username).collect(),
            reviewers: self.reviewers.into_iter().map(|u| u.username).collect(),
            checks: self.head_pipeline.and_then(|p| pipeline_checks(&p.status)),
            has_conflicts: self.has_conflicts,
        }
    }
}

#[derive(serde::Deserialize)]
struct GitLabIssueRow {
    iid: u64,
    title: String,
    state: String,
    web_url: String,
    #[serde(default)]
    updated_at: String,
    author: Option<GitLabUsername>,
    #[serde(default)]
    assignees: Vec<GitLabUsername>,
    #[serde(default)]
    labels: Vec<String>,
    milestone: Option<crate::model::MilestoneTitle>,
}

#[derive(serde::Deserialize)]
struct GitLabTodo<T> {
    target: T,
    project: Option<GitLabTodoProject>,
}

#[derive(serde::Deserialize)]
struct GitLabTodoProject {
    path_with_namespace: String,
}

fn issue_summary(r: GitLabIssueRow) -> IssueSummary {
    IssueSummary {
        number: r.iid,
        id: r.iid.to_string(),
        reference: format!("#{}", r.iid),
        iid_label: format!("#{}", r.iid),
        title: r.title,
        state: if r.state == "closed" {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: r.author.map(|u| u.username).unwrap_or_default(),
        assignees: r.assignees.into_iter().map(|u| u.username).collect(),
        labels: r.labels,
        web_url: r.web_url,
        updated_at: r.updated_at,
        milestone: r.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
        ..IssueSummary::default()
    }
}

#[derive(serde::Deserialize)]
struct GitLabCreatedIssue {
    iid: u64,
    web_url: String,
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

    fn client(transport: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
        client_for(
            Integration::new(
                "gl",
                IntegrationKind::GitlabSelfManaged,
                "https://gitlab.example.com",
            ),
            "tok".into(),
            transport,
        )
    }

    const USER: &str =
        r#"{"id":5,"username":"sg","name":"S G","web_url":"https://gitlab.example.com/sg"}"#;

    fn mr(iid: u64, state: &str, extra: &str) -> String {
        format!(
            r#"{{"iid":{iid},"title":"t{iid}","state":"{state}","source_branch":"feature","target_branch":"main",
            "source_project_id":1,"target_project_id":1,"author":{{"username":"sg"}},
            "web_url":"https://gitlab.example.com/g/p/-/merge_requests/{iid}",
            "updated_at":"2026-01-0{iid}T00:00:00Z"{extra}}}"#
        )
    }

    #[test]
    fn check_token_reports_scopes_and_missing_ones() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on(
                    "/personal_access_tokens/self",
                    200,
                    r#"{"scopes":["read_user"],"expires_at":"2027-01-01"}"#,
                )
                .on("/user", 200, USER),
        );
        let c = client(transport.clone());
        let check = block_on(c.check_token()).unwrap();
        assert_eq!(check.user.login, "sg");
        assert_eq!(check.missing_scopes, vec!["api"]);
        assert_eq!(check.expires_at.as_deref(), Some("2027-01-01"));
        assert!(transport.requests()[0]
            .url
            .starts_with("https://gitlab.example.com/api/v4/user"));
    }

    #[test]
    fn a_404_on_the_token_endpoint_means_unknown_scopes() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on(
                    "/personal_access_tokens/self",
                    404,
                    r#"{"message":"404 Not found"}"#,
                )
                .on("/user", 200, USER),
        );
        let check = block_on(client(transport).check_token()).unwrap();
        assert_eq!(check.scopes, None);
        assert!(check.missing_scopes.is_empty());
    }

    #[test]
    fn an_api_scoped_token_satisfies_read_user() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/personal_access_tokens/self", 200, r#"{"scopes":["api"]}"#)
                .on("/user", 200, USER),
        );
        let check = block_on(client(transport).check_token()).unwrap();
        assert!(check.missing_scopes.is_empty());
    }

    #[test]
    fn a_401_on_user_is_unauthorized() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/user",
            401,
            r#"{"message":"401 Unauthorized"}"#,
        ));
        let err = block_on(client(transport).check_token()).unwrap_err();
        assert!(matches!(err, Error::Unauthorized { .. }));
    }

    #[test]
    fn merge_requests_normalise_state_and_labels() {
        let body = format!(
            "[{},{},{}]",
            mr(1, "opened", ""),
            mr(2, "opened", r#","draft":true"#),
            mr(3, "merged", "")
        );
        let transport = Arc::new(RecordedTransport::new().on("/merge_requests?", 200, &body));
        let c = client(transport.clone());
        let prs = block_on(c.list_pull_requests("g/sub/p", &PrFilter::default())).unwrap();
        assert_eq!(prs[0].iid_label, "!1");
        assert_eq!(prs[0].state, PullRequestState::Open);
        assert_eq!(prs[1].state, PullRequestState::Draft);
        assert_eq!(prs[2].state, PullRequestState::Merged);
        let url = &transport.requests()[0].url;
        assert!(
            url.contains("/projects/g%2Fsub%2Fp/merge_requests"),
            "{url}"
        );
        assert!(url.contains("state=opened"), "{url}");
    }

    #[test]
    fn closed_includes_merged_but_not_open() {
        let body = format!(
            "[{},{},{}]",
            mr(1, "opened", ""),
            mr(2, "closed", ""),
            mr(3, "merged", "")
        );
        let transport = Arc::new(RecordedTransport::new().on("/merge_requests?", 200, &body));
        let filter = PrFilter {
            state: StateFilter::Closed,
            ..Default::default()
        };
        let prs = block_on(client(transport).list_pull_requests("g/p", &filter)).unwrap();
        assert_eq!(prs.len(), 2);
    }

    #[test]
    fn mine_and_review_requested_use_the_hosts_own_filters() {
        let transport = Arc::new(RecordedTransport::new().on("/user", 200, USER).on(
            "/merge_requests?",
            200,
            &format!("[{}]", mr(1, "opened", "")),
        ));
        let c = client(transport.clone());
        let filter = PrFilter {
            mine: true,
            review_requested: true,
            ..Default::default()
        };
        let prs = block_on(c.list_pull_requests("g/p", &filter)).unwrap();
        assert!(prs[0].review_requested);
        let url = &transport
            .requests()
            .into_iter()
            .find(|r| r.url.contains("merge_requests"))
            .unwrap()
            .url;
        assert!(url.contains("author_username=sg"), "{url}");
        assert!(url.contains("reviewer_username=sg"), "{url}");
    }

    #[test]
    fn full_pages_are_followed() {
        let full = format!(
            "[{}]",
            (0..PAGE_SIZE)
                .map(|_| mr(1, "opened", ""))
                .collect::<Vec<_>>()
                .join(",")
        );
        let transport = Arc::new(
            RecordedTransport::new()
                .on("&page=1", 200, &full)
                .on("&page=2", 200, "[]"),
        );
        let prs =
            block_on(client(transport.clone()).list_pull_requests("g/p", &PrFilter::default()))
                .unwrap();
        assert_eq!(prs.len(), PAGE_SIZE as usize);
        assert_eq!(transport.requests().len(), 2);
    }

    #[test]
    fn the_merge_request_for_a_branch_prefers_open_over_merged_and_encodes_the_branch() {
        let body = format!("[{},{}]", mr(5, "merged", ""), mr(2, "opened", ""));
        let transport = Arc::new(RecordedTransport::new().on("/merge_requests?", 200, &body));
        let found = block_on(client(transport.clone()).pull_request_for_branch("g/p", "feature/x"))
            .unwrap()
            .unwrap();
        assert_eq!(found.number, 2);
        let url = &transport.requests()[0].url;
        assert!(url.contains("source_branch=feature%2Fx"), "{url}");
    }

    #[test]
    fn a_fork_merge_request_names_its_source_project() {
        let fork =
            mr(1, "opened", "").replace(r#""source_project_id":1"#, r#""source_project_id":9"#);
        let transport =
            Arc::new(RecordedTransport::new().on("/merge_requests?", 200, &format!("[{fork}]")));
        let prs =
            block_on(client(transport).list_pull_requests("g/p", &PrFilter::default())).unwrap();
        assert_eq!(prs[0].source_project.as_deref(), Some("9"));
    }

    #[test]
    fn issues_map_fields_and_filter_by_assignee() {
        let body = r#"[{"iid":12,"title":"bug","state":"opened","web_url":"https://gitlab.example.com/g/p/-/issues/12",
            "updated_at":"2026-01-01T00:00:00Z","author":{"username":"sg"},
            "assignees":[{"username":"sg"}],"labels":["bug","ui"]}]"#;
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/user", 200, USER)
                .on("/issues?", 200, body),
        );
        let filter = IssueFilter {
            assigned_to_me: true,
            ..Default::default()
        };
        let issues = block_on(client(transport.clone()).list_issues("g/p", &filter)).unwrap();
        assert_eq!(issues[0].iid_label, "#12");
        assert_eq!(issues[0].reference, "#12");
        assert_eq!(issues[0].id, "12");
        assert_eq!(issues[0].labels, vec!["bug", "ui"]);
        assert_eq!(issues[0].state, IssueState::Open);
        assert!(transport
            .requests()
            .iter()
            .any(|r| r.url.contains("assignee_username=sg")));
    }

    #[test]
    fn deleting_a_key_sends_delete() {
        let transport = Arc::new(RecordedTransport::new().on("/user/keys/7", 204, ""));
        block_on(client(transport.clone()).delete_ssh_key("7")).unwrap();
        let request = &transport.requests()[0];
        assert_eq!(request.method, crate::http::Method::Delete);
        assert_eq!(request.url, "https://gitlab.example.com/api/v4/user/keys/7");
    }

    #[test]
    fn creating_an_issue_posts_to_the_encoded_project_on_a_self_managed_host() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/issues",
            201,
            r#"{"iid":7,"web_url":"https://gitlab.example.com/grp/sub/proj/-/issues/7"}"#,
        ));
        let c = client(transport.clone());
        let created = block_on(c.create_issue(&NewIssue {
            project: "grp/sub/proj".into(),
            title: "Broken".into(),
            body: "Details".into(),
            labels: vec!["bug".into(), "ui".into()],
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(created.number, 7);
        assert_eq!(created.reference, "#7");
        let request = &transport.requests()[0];
        assert_eq!(request.method, crate::http::Method::Post);
        assert_eq!(
            request.url,
            "https://gitlab.example.com/api/v4/projects/grp%2Fsub%2Fproj/issues"
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["title"], "Broken");
        assert_eq!(body["description"], "Details");
        assert_eq!(body["labels"], "bug,ui");
    }

    #[test]
    fn creating_an_issue_surfaces_each_failure_clearly() {
        for (status, body) in [
            (401, r#"{"message":"401 Unauthorized"}"#),
            (403, r#"{"message":"403 Forbidden"}"#),
            (404, r#"{"message":"404 Project Not Found"}"#),
            (422, r#"{"message":{"title":["can't be blank"]}}"#),
        ] {
            let transport = Arc::new(RecordedTransport::new().on("/issues", status, body));
            let c = client(transport);
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
            assert!(text.contains("opening a GitLab issue"), "{text}");
            if status == 422 {
                assert!(text.contains("title") && text.contains("blank"), "{text}");
            }
        }
    }

    #[test]
    fn a_merge_request_carries_its_pipeline_and_conflict_state() {
        let rows = format!(
            "[{},{},{},{}]",
            mr(
                1,
                "opened",
                r#","has_conflicts":true,"head_pipeline":{"status":"failed"}"#
            ),
            mr(
                2,
                "opened",
                r#","has_conflicts":false,"head_pipeline":{"status":"success"}"#
            ),
            mr(3, "opened", r#","head_pipeline":{"status":"running"}"#),
            mr(4, "opened", r#","head_pipeline":null"#),
        );
        let transport = Arc::new(RecordedTransport::new().on("/merge_requests?", 200, &rows));
        let c = client(transport);
        let prs = block_on(c.list_pull_requests("g/p", &PrFilter::default())).unwrap();
        assert_eq!(prs[0].checks, Some(ChecksState::Failing));
        assert_eq!(prs[0].has_conflicts, Some(true));
        assert_eq!(prs[1].checks, Some(ChecksState::Passing));
        assert_eq!(prs[1].has_conflicts, Some(false));
        assert_eq!(prs[2].checks, Some(ChecksState::Pending));
        assert_eq!(prs[2].has_conflicts, None);
        assert_eq!(prs[3].checks, None);
    }

    #[test]
    fn mentioned_reads_the_pending_todos_of_the_project() {
        let todos = format!(
            r#"[
            {{"target_type":"MergeRequest","project":{{"path_with_namespace":"g/p"}},"target":{}}},
            {{"target_type":"MergeRequest","project":{{"path_with_namespace":"g/other"}},"target":{}}},
            {{"target_type":"MergeRequest","project":{{"path_with_namespace":"G/P"}},"target":{}}}
            ]"#,
            mr(1, "opened", ""),
            mr(2, "opened", ""),
            mr(3, "merged", "")
        );
        let transport = Arc::new(RecordedTransport::new().on("/todos?", 200, &todos));
        let c = client(transport.clone());
        let filter = PrFilter {
            mentioned: true,
            ..PrFilter::default()
        };
        let prs = block_on(c.list_pull_requests("g/p", &filter)).unwrap();
        // The other project's to-do and the merged request are dropped.
        assert_eq!(prs.iter().map(|p| p.number).collect::<Vec<_>>(), vec![1]);
        let url = transport.requests()[0].url.clone();
        assert!(url.contains("action=mentioned"), "{url}");
        assert!(url.contains("type=MergeRequest"), "{url}");
        assert!(url.contains("state=pending"), "{url}");
    }

    #[test]
    fn mentioned_issues_come_from_the_todos_too() {
        let issue = |iid: u64, state: &str| {
            format!(
                r#"{{"iid":{iid},"title":"i{iid}","state":"{state}","web_url":"u","updated_at":"2026-01-0{iid}T00:00:00Z","author":{{"username":"x"}}}}"#
            )
        };
        let todos = format!(
            r#"[{{"project":{{"path_with_namespace":"g/p"}},"target":{}}},
                {{"project":{{"path_with_namespace":"g/p"}},"target":{}}}]"#,
            issue(1, "opened"),
            issue(2, "closed")
        );
        let transport = Arc::new(RecordedTransport::new().on("/todos?", 200, &todos));
        let c = client(transport.clone());
        let filter = IssueFilter {
            mentioned: true,
            ..IssueFilter::default()
        };
        let found = block_on(c.list_issues("g/p", &filter)).unwrap();
        assert_eq!(found.iter().map(|i| i.number).collect::<Vec<_>>(), vec![1]);
        assert!(transport.requests()[0].url.contains("type=Issue"));
    }
}
