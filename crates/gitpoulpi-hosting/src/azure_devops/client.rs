//! The Azure DevOps client. Endpoints and doc URLs: see the module header
//! of [`super`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gitpoulpi_config::IntegrationKind;
use serde_json::{json, Value};

use super::map::{self, List, Pull, Repo};
use super::spec::split_project;
use crate::auth::auth_headers;
use crate::client::{
    encode_component as enc, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue,
    NewPullRequest, MAX_PAGES, PAGE_SIZE,
};
use crate::conversation::{
    Comment, ConversationCaps, ItemKey, ItemKind, PageCursor, PostingIdentity, TimelineOpts,
    TimelinePage,
};
use crate::http::{HttpRequest, HttpResponse, HttpTransport};
use crate::integration::Integration;
use crate::model::{
    best_pull_request, Choice, HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, PrFilter,
    PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::spec::spec_for;
use crate::{Error, Result};

/// Work items one batch read accepts.
const WORK_ITEM_BATCH: usize = 200;

/// Work item types tried, in order, as the New-item default.
const DEFAULT_TYPES: [&str; 5] = ["Issue", "Task", "Bug", "User Story", "Product Backlog Item"];

mod account;
mod conversation;
mod protection;

pub struct AzureDevopsClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
    /// `Project/Repo` to repository id (the PR endpoints want an id).
    repo_ids: Mutex<HashMap<String, String>>,
    pub(crate) scopes: crate::scope_handle::ScopeHandle,
}

impl AzureDevopsClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
            repo_ids: Mutex::new(HashMap::new()),
            scopes: Default::default(),
        }
    }

    /// The organization (or Server collection) root, without a slash.
    fn root(&self) -> Result<String> {
        let root = self.integration.api_base();
        let root = root.trim_end_matches('/').to_string();
        let services_without_org = self.integration.kind == IntegrationKind::AzureDevops
            && self.integration.azure_org().is_none();
        if root.is_empty() || services_without_org {
            return Err(Error::BadUrl(format!(
                "{} (an Azure DevOps profile needs https://dev.azure.com/<org>)",
                self.integration.url
            )));
        }
        Ok(root)
    }

    /// Server 2022 tops out at 7.0 (versioning table); Services takes 7.1.
    fn api_version(&self) -> &'static str {
        if self.integration.kind == IntegrationKind::AzureDevopsServer {
            "7.0"
        } else {
            "7.1"
        }
    }

    /// `{root}[/{project}]/_apis/{path}?{query&}api-version=...`
    fn url(&self, project: Option<&str>, path: &str, query: &str) -> Result<String> {
        let mut url = self.root()?;
        if let Some(project) = project.filter(|p| !p.is_empty()) {
            url.push('/');
            url.push_str(&enc(project));
        }
        url.push_str("/_apis/");
        url.push_str(path);
        url.push('?');
        if !query.is_empty() {
            url.push_str(query);
            url.push('&');
        }
        url.push_str("api-version=");
        url.push_str(self.api_version());
        Ok(url)
    }

    fn authed(&self, mut request: HttpRequest) -> HttpRequest {
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request.header("Accept", "application/json")
    }

    fn get(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::get(url))
    }

    fn post(&self, url: String, body: &Value, content_type: Option<&str>) -> HttpRequest {
        let mut request = HttpRequest::post_json(url, body);
        if let Some(content_type) = content_type {
            for (name, value) in &mut request.headers {
                if name == "Content-Type" {
                    *value = content_type.to_string();
                }
            }
        }
        self.authed(request)
    }

    /// Sends and checks the status. An invalid PAT is answered **203 with
    /// an HTML sign-in page**, never JSON: that is an `Unauthorized`.
    async fn send(&self, request: HttpRequest, what: &str) -> Result<HttpResponse> {
        let response = self.transport.send(request).await?;
        let html = response.body.trim_start().starts_with('<');
        if response.status == 203 || (response.is_success() && html) {
            return Err(Error::Unauthorized {
                what: what.to_string(),
                detail: "Azure DevOps answered with its sign-in page: the token is invalid, \
                         expired, or not scoped to this organization"
                    .to_string(),
            });
        }
        if response.status == 403 {
            // Azure's `X-RateLimit-Remaining` counts TSTUs, not calls: a 403
            // is a permission problem whatever it says (only 429 is a limit).
            let mut plain = response.clone();
            plain.headers.clear();
            plain.error_for_status(what)?;
        }
        response.error_for_status(what)?;
        Ok(response)
    }

    /// The `Project/Repo` repository's id, via the project-scoped
    /// *Get Repository* (which takes a name or an id).
    async fn repo_id(&self, project: &str) -> Result<String> {
        if let Some(id) = self.repo_ids.lock().unwrap().get(project) {
            return Ok(id.clone());
        }
        let (p, r) = split_project(project);
        let url = self.url(Some(p), &format!("git/repositories/{}", enc(r)), "")?;
        let response = self
            .send(self.get(url), "looking up the repository")
            .await?;
        let repo: Repo = response.json()?;
        if repo.id.is_empty() {
            return Err(Error::Decode(format!("no id for repository {project}")));
        }
        self.repo_ids
            .lock()
            .unwrap()
            .insert(project.to_string(), repo.id.clone());
        Ok(repo.id)
    }

    /// The states of one work item type as `(category rank, name)`.
    async fn type_states(&self, project: &str, kind: &str) -> Result<Vec<(usize, String)>> {
        let url = self.url(
            Some(project),
            &format!("wit/workitemtypes/{}/states", enc(kind)),
            "",
        )?;
        let response = self
            .send(self.get(url), "reading the work item type states")
            .await?;
        let list: List<Value> = response.json()?;
        Ok(list
            .value
            .iter()
            .filter_map(|s| {
                let name = s["name"].as_str().filter(|n| !n.is_empty())?;
                let rank = match s["category"].as_str() {
                    Some("Proposed") => 0,
                    Some("InProgress") => 1,
                    Some("Resolved") => 2,
                    Some("Completed") => 3,
                    _ => 4,
                };
                Some((rank, name.to_string()))
            })
            .collect())
    }

    /// The current user's id, when the server tells.
    async fn me(&self) -> Result<String> {
        match self.connection_user().await? {
            Some(user) if !user.id.is_empty() => Ok(user.id),
            _ => Err(self.unsupported(
                "filtering by the current user (the server did not report the token's user)",
            )),
        }
    }

    /// `connectionData`'s authenticated user; `None` when the endpoint is
    /// missing or odd (the caller falls back to a plain access check).
    async fn connection_user(&self) -> Result<Option<HostedUser>> {
        let url = format!("{}/_apis/connectionData", self.root()?);
        let response = match self
            .send(self.get(url), "reading the Azure DevOps user")
            .await
        {
            Ok(r) => r,
            Err(e @ (Error::Unauthorized { .. } | Error::RateLimited { .. })) => return Err(e),
            Err(_) => return Ok(None),
        };
        let Ok(value) = response.json::<Value>() else {
            return Ok(None);
        };
        let user = &value["authenticatedUser"];
        let id = user["id"].as_str().unwrap_or_default().to_string();
        if id.is_empty() {
            return Ok(None);
        }
        let name = user["providerDisplayName"]
            .as_str()
            .or_else(|| user["customDisplayName"].as_str())
            .unwrap_or_default()
            .to_string();
        let account = user["properties"]["Account"]["$value"]
            .as_str()
            .unwrap_or_default();
        Ok(Some(HostedUser {
            login: if account.is_empty() {
                name.clone()
            } else {
                account.to_string()
            },
            name,
            id,
            web_url: None,
        }))
    }

    /// Every page of a pull-request query (`$top`/`$skip`), until a short
    /// page or [`MAX_PAGES`].
    async fn pull_requests(&self, project: &str, criteria: &str, what: &str) -> Result<Vec<Pull>> {
        let repo_id = self.repo_id(project).await?;
        let (p, _) = split_project(project);
        let mut out = Vec::new();
        for page in 0..MAX_PAGES {
            let query = format!(
                "{criteria}&$top={PAGE_SIZE}&$skip={}",
                page as usize * PAGE_SIZE as usize
            );
            let url = self.url(
                Some(p),
                &format!("git/repositories/{}/pullrequests", enc(&repo_id)),
                &query,
            )?;
            let response = self.send(self.get(url), what).await?;
            let list: List<Pull> = response.json()?;
            let count = list.value.len();
            out.extend(list.value);
            if count < PAGE_SIZE as usize {
                break;
            }
        }
        Ok(out)
    }

    fn map_pulls(&self, pulls: Vec<Pull>, me: Option<&str>) -> Result<Vec<PullRequestSummary>> {
        let root = self.root()?;
        Ok(pulls
            .into_iter()
            .map(|p| map::map_pull(&root, me, p))
            .collect())
    }
}

fn wiql_state_clause(state: StateFilter) -> Option<String> {
    let names = map::CLOSED_STATES
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ");
    match state {
        StateFilter::Open => Some(format!("[System.State] NOT IN ({names})")),
        StateFilter::Closed => Some(format!("[System.State] IN ({names})")),
        StateFilter::All => None,
    }
}

#[async_trait::async_trait]
impl HostingClient for AzureDevopsClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let root = self.root()?;
        let url = self.url(None, "git/repositories", "")?;
        let response = self
            .send(self.get(url), "listing Azure DevOps repositories")
            .await?;
        let list: List<Repo> = response.json()?;
        Ok(list
            .value
            .into_iter()
            .filter(|r| !r.is_disabled)
            .map(|r| map::map_repo(&self.integration.id, &root, r))
            .collect())
    }

    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        account::fetch(self, query).await
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        // No documented REST API for SSH keys with a PAT (the UI only).
        Err(self.unsupported("list_ssh_keys"))
    }

    async fn upload_ssh_key(&self, _title: &str, _public_key: &str) -> Result<HostedSshKey> {
        Err(self.unsupported("upload_ssh_key"))
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        if req.fork_source().is_some() {
            return Err(Error::Unsupported(
                "opening an Azure DevOps pull request from a fork (open it in the browser)"
                    .to_string(),
            ));
        }
        let repo_id = self.repo_id(&req.project).await?;
        let (p, _) = split_project(&req.project);
        let url = self.url(
            Some(p),
            &format!("git/repositories/{}/pullrequests", enc(&repo_id)),
            "",
        )?;
        let body = json!({
            "sourceRefName": format!("refs/heads/{}", req.source_branch),
            "targetRefName": format!("refs/heads/{}", req.target_branch),
            "title": req.title,
            "description": req.body,
            "isDraft": req.draft,
        });
        let response = self
            .send(
                self.post(url, &body, None),
                "creating the Azure DevOps pull request",
            )
            .await?;
        let created: Value = response.json()?;
        let number = created["pullRequestId"]
            .as_u64()
            .ok_or_else(|| Error::Decode("the created pull request has no id".to_string()))?;
        let web_url = (spec_for(self.integration.kind.family()).pr_web_url)(
            &self.root()?,
            &req.project,
            number,
        )
        .unwrap_or_default();
        Ok(CreatedPullRequest {
            number,
            web_url,
            ..Default::default()
        })
    }

    async fn current_user(&self) -> Result<HostedUser> {
        self.connection_user().await?.ok_or_else(|| {
            self.unsupported("current_user (the server did not report the token's user)")
        })
    }

    async fn protected_branches(
        &self,
        project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        self.protection(project).await
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        let user = match self.connection_user().await? {
            Some(user) => user,
            None => {
                // Proves access only; the user stays unknown.
                let url = self.url(None, "projects", "$top=1")?;
                self.send(self.get(url), "checking the Azure DevOps token")
                    .await?;
                HostedUser::default()
            }
        };
        Ok(TokenCheck {
            user,
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
        let me = if filter.mine || filter.review_requested {
            Some(self.me().await?)
        } else {
            None
        };
        let status = match filter.state {
            StateFilter::Open => "active",
            StateFilter::Closed | StateFilter::All => "all",
        };
        let mut criteria = format!("searchCriteria.status={status}");
        if let Some(me) = &me {
            if filter.mine {
                criteria.push_str(&format!("&searchCriteria.creatorId={}", enc(me)));
            }
            if filter.review_requested {
                criteria.push_str(&format!("&searchCriteria.reviewerId={}", enc(me)));
            }
        }
        let pulls = self
            .pull_requests(project, &criteria, "listing Azure DevOps pull requests")
            .await?;
        let mut all = self.map_pulls(pulls, me.as_deref())?;
        if filter.state == StateFilter::Closed {
            all.retain(|p| p.state != crate::model::PullRequestState::Open);
            all.retain(|p| p.state != crate::model::PullRequestState::Draft);
        }
        Ok(all)
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let criteria = format!(
            "searchCriteria.status=all&searchCriteria.sourceRefName={}",
            enc(&format!("refs/heads/{branch}"))
        );
        let pulls = self
            .pull_requests(project, &criteria, "finding the pull request of the branch")
            .await?;
        Ok(best_pull_request(self.map_pulls(pulls, None)?))
    }

    /// The work item states form the columns; a move sets `System.State`.
    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::LocalStatus
    }

    /// The states of every enabled work item type, by state category
    /// (proposed, in progress, resolved, completed), each name once.
    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        let (project, _) = split_project(project);
        let types = self.issue_create_options(project).await?.types;
        let mut rows: Vec<(usize, String)> = Vec::new();
        for ty in types {
            for (rank, name) in self.type_states(project, &ty.id).await? {
                if !rows.iter().any(|(_, n)| n.eq_ignore_ascii_case(&name)) {
                    rows.push((rank, name));
                }
            }
        }
        rows.sort_by_key(|(rank, _)| *rank);
        Ok(rows.into_iter().map(|(_, n)| n).collect())
    }

    /// `PATCH /_apis/wit/workitems/{id}` on `System.State`. A work item
    /// type that lacks the target state is refused with the reason, before
    /// anything is sent.
    async fn move_issue(
        &self,
        project: &str,
        issue: &IssueSummary,
        _spec: &crate::board::BoardSpec,
        to: &crate::board::BoardColumn,
    ) -> Result<()> {
        let (project, _) = split_project(project);
        let from = issue.status_label.as_deref().unwrap_or_default();
        if from.trim().eq_ignore_ascii_case(to.name.trim()) {
            return Ok(());
        }
        let kind = issue.kind_label.clone().unwrap_or_default();
        let state = self
            .type_states(project, &kind)
            .await?
            .into_iter()
            .map(|(_, n)| n)
            .find(|n| n.eq_ignore_ascii_case(to.name.trim()))
            .ok_or_else(|| {
                Error::Local(format!(
                    "a {kind} has no state {} (work item {})",
                    to.name.trim(),
                    issue.reference
                ))
            })?;
        let url = self.url(
            Some(project),
            &format!("wit/workitems/{}", issue.number),
            "",
        )?;
        let ops = json!([{ "op": "add", "path": "/fields/System.State", "value": state }]);
        let mut request = HttpRequest::patch_json(url, &ops);
        for (name, value) in &mut request.headers {
            if name == "Content-Type" {
                *value = "application/json-patch+json".to_string();
            }
        }
        self.send(self.authed(request), "moving the Azure DevOps work item")
            .await?;
        Ok(())
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let (project, _) = split_project(project);
        if project.is_empty() {
            return Err(Error::Unsupported(
                "listing work items without a project".to_string(),
            ));
        }
        let mut clauses = vec!["[System.TeamProject] = @project".to_string()];
        clauses.extend(wiql_state_clause(filter.state));
        if filter.assigned_to_me {
            clauses.push("[System.AssignedTo] = @Me".to_string());
        }
        if let Some(extra) = filter
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty())
        {
            clauses.push(format!("({extra})"));
        }
        let wiql = format!(
            "SELECT [System.Id] FROM WorkItems WHERE {} ORDER BY [System.ChangedDate] DESC",
            clauses.join(" AND ")
        );
        let url = self.url(
            Some(project),
            "wit/wiql",
            &format!("$top={WORK_ITEM_BATCH}"),
        )?;
        let response = self
            .send(
                self.post(url, &json!({ "query": wiql }), None),
                "querying Azure DevOps work items",
            )
            .await?;
        let found: Value = response.json()?;
        let ids: Vec<u64> = found["workItems"]
            .as_array()
            .map(|a| a.iter().filter_map(|w| w["id"].as_u64()).collect())
            .unwrap_or_default();

        let root = self.root()?;
        let mut out = Vec::with_capacity(ids.len());
        for chunk in ids.chunks(WORK_ITEM_BATCH) {
            let list = chunk
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "ids={list}&fields=System.Title,System.State,System.WorkItemType,\
                 System.AssignedTo,System.CreatedBy,System.ChangedDate,System.Tags,\
                 System.TeamProject"
            );
            let url = self.url(Some(project), "wit/workitems", &query)?;
            let response = self
                .send(self.get(url), "reading Azure DevOps work items")
                .await?;
            let items: List<Value> = response.json()?;
            // Keep WIQL's order (most recently changed first).
            let mut mapped: Vec<(usize, IssueSummary)> = items
                .value
                .iter()
                .filter_map(|i| map::map_work_item(&root, project, i))
                .filter_map(|s| chunk.iter().position(|id| *id == s.number).map(|i| (i, s)))
                .collect();
            mapped.sort_by_key(|(i, _)| *i);
            out.extend(mapped.into_iter().map(|(_, s)| s));
        }
        Ok(out)
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let (project, _) = split_project(&req.project);
        let work_item_type = match req.issue_type.as_deref().filter(|t| !t.is_empty()) {
            Some(t) => t.to_string(),
            None => self
                .issue_create_options(project)
                .await?
                .default_type
                .ok_or_else(|| {
                    Error::Unsupported(
                        "creating a work item without a type (none is available in this project)"
                            .to_string(),
                    )
                })?,
        };
        let mut ops =
            vec![json!({"op": "add", "path": "/fields/System.Title", "value": req.title})];
        if !req.body.trim().is_empty() {
            ops.push(json!({
                "op": "add",
                "path": "/fields/System.Description",
                "value": map::html_text(&req.body),
            }));
        }
        if !req.labels.is_empty() {
            ops.push(json!({
                "op": "add",
                "path": "/fields/System.Tags",
                "value": req.labels.join("; "),
            }));
        }
        let url = self.url(
            Some(project),
            &format!("wit/workitems/${}", enc(&work_item_type)),
            "",
        )?;
        let response = self
            .send(
                self.post(url, &Value::Array(ops), Some("application/json-patch+json")),
                "creating the Azure DevOps work item",
            )
            .await?;
        let created: Value = response.json()?;
        let number = created["id"]
            .as_u64()
            .ok_or_else(|| Error::Decode("the created work item has no id".to_string()))?;
        let web_url = (spec_for(self.integration.kind.family()).issue_web_url)(
            &self.root()?,
            project,
            &number.to_string(),
        )
        .unwrap_or_default();
        Ok(CreatedIssue {
            number,
            web_url,
            reference: format!("#{number}"),
        })
    }

    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let (project, _) = split_project(project);
        let url = self.url(Some(project), "wit/workitemtypes", "")?;
        let response = self
            .send(self.get(url), "listing Azure DevOps work item types")
            .await?;
        let list: List<Value> = response.json()?;
        let types: Vec<Choice> = list
            .value
            .iter()
            .filter(|t| !t["isDisabled"].as_bool().unwrap_or(false))
            .filter_map(|t| t["name"].as_str())
            .map(|name| Choice {
                id: name.to_string(),
                label: name.to_string(),
            })
            .collect();
        let default_type = DEFAULT_TYPES
            .iter()
            .find_map(|d| types.iter().find(|t| t.id == *d))
            .or_else(|| types.first())
            .map(|t| t.id.clone());
        Ok(IssueCreateOptions {
            types,
            lists: Vec::new(),
            default_type,
            default_list: None,
        })
    }

    fn conversation_caps(&self, kind: ItemKind) -> ConversationCaps {
        ConversationCaps::azure_devops(kind, self.is_server())
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
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.conv_edit(project, key, comment_id, body).await
    }

    async fn delete_item_comment(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        self.conv_delete(project, key, comment_id).await
    }
}
