//! The Redmine client (issues only).

use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::auth::auth_headers;
use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
    MAX_PAGES,
};
use crate::conversation::{
    Comment, ConversationCaps, ItemKey, ItemKind, PageCursor, PostingIdentity, TimelineOpts,
    TimelinePage,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind};
use crate::model::{
    Choice, HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::map;

/// Rows of one page (Redmine caps `limit` at 100).
const PAGE: usize = 100;
/// A list shows at most this many issues.
const MAX_ISSUES: usize = 500;

pub(crate) struct RedmineClient {
    pub(super) integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
}

impl RedmineClient {
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

    pub(super) fn site(&self) -> String {
        self.integration
            .url
            .trim()
            .trim_end_matches('/')
            .to_string()
    }

    pub(super) fn api(&self) -> String {
        self.integration.api_base()
    }

    fn authed(&self, request: HttpRequest) -> HttpRequest {
        let mut request = request.header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    pub(super) async fn get_json<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T> {
        let url = format!("{}{path}", self.api());
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
        response.error_for_status(what)?;
        response.json()
    }

    pub(super) async fn send(
        &self,
        request: HttpRequest,
        what: &str,
    ) -> Result<crate::http::HttpResponse> {
        let response = self.transport.send(self.authed(request)).await?;
        response.error_for_status(what)?;
        Ok(response)
    }

    /// `PUT /issues/{n}.json`: an edit; Redmine answers 204 with no body.
    pub(super) async fn edit(&self, what: &str, number: u64, issue: Value) -> Result<()> {
        let url = format!("{}/issues/{number}.json", self.api());
        self.send(HttpRequest::put_json(url, &json!({ "issue": issue })), what)
            .await?;
        Ok(())
    }

    async fn statuses(&self) -> Result<Vec<map::StatusRow>> {
        let page: map::StatusesPage = self
            .get_json("/issue_statuses.json", "listing the Redmine statuses")
            .await?;
        Ok(page.issue_statuses)
    }

    /// The project's trackers, falling back on all of them.
    async fn trackers(&self, project: &str) -> Result<Vec<map::Named>> {
        let path = format!(
            "/projects/{}.json?include=trackers",
            encode_component(project.trim())
        );
        if let Ok(detail) = self
            .get_json::<map::ProjectDetail>(&path, "reading the Redmine project")
            .await
        {
            if !detail.project.trackers.is_empty() {
                return Ok(detail.project.trackers);
            }
        }
        let page: map::TrackersPage = self
            .get_json("/trackers.json", "listing the Redmine trackers")
            .await?;
        Ok(page.trackers)
    }

    /// Every user and group that can be assigned in the project.
    async fn members(&self, project: &str) -> Result<Vec<map::Named>> {
        let mut out: Vec<map::Named> = Vec::new();
        for _ in 0..MAX_PAGES {
            let page: map::MembershipsPage = self
                .get_json(
                    &format!(
                        "/projects/{}/memberships.json?limit={PAGE}&offset={}",
                        encode_component(project.trim()),
                        out.len()
                    ),
                    "listing the Redmine project members",
                )
                .await?;
            let got = page.memberships.len();
            out.extend(
                page.memberships
                    .into_iter()
                    .filter_map(|m| m.user.or(m.group)),
            );
            if got < PAGE || out.len() >= page.total_count {
                break;
            }
        }
        Ok(out)
    }

    /// The id of the member named `name` (what a picker calls the login).
    async fn member_id(&self, project: &str, name: &str) -> Result<u64> {
        let wanted = name.trim();
        self.members(project)
            .await?
            .into_iter()
            .find(|m| m.name.eq_ignore_ascii_case(wanted))
            .map(|m| m.id)
            .ok_or_else(|| Error::Api {
                what: "assigning a Redmine issue".into(),
                status: 422,
                detail: format!("{wanted} is not a member of the project"),
            })
    }

    async fn versions(&self, project: &str) -> Result<Vec<map::VersionRow>> {
        let page: map::VersionsPage = self
            .get_json(
                &format!(
                    "/projects/{}/versions.json",
                    encode_component(project.trim())
                ),
                "listing the Redmine versions",
            )
            .await?;
        Ok(page.versions)
    }

    async fn set_status_named(&self, number: u64, status: &str, what: &str) -> Result<()> {
        let id = self
            .statuses()
            .await?
            .into_iter()
            .find(|s| s.name.eq_ignore_ascii_case(status.trim()))
            .map(|s| s.id)
            .ok_or_else(|| Error::Api {
                what: what.into(),
                status: 422,
                detail: format!("Redmine has no status {status}"),
            })?;
        self.edit(what, number, json!({ "status_id": id })).await
    }
}

fn milestone_of(v: &map::VersionRow) -> HostedMilestone {
    HostedMilestone {
        id: v.id,
        title: v.name.clone(),
        due_on: v.due_date.clone().filter(|d| !d.is_empty()),
        state: v.status.clone(),
    }
}

fn no_labels() -> Error {
    Error::Api {
        what: "labelling a Redmine issue".into(),
        status: 422,
        detail: "Redmine has no labels".into(),
    }
}

#[async_trait::async_trait]
impl HostingClient for RedmineClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let site = self.site();
        let mut out = Vec::new();
        for _ in 0..MAX_PAGES {
            let page: map::ProjectsPage = self
                .get_json(
                    &format!("/projects.json?limit={PAGE}&offset={}", out.len()),
                    "listing Redmine projects",
                )
                .await?;
            let got = page.projects.len();
            out.extend(
                page.projects
                    .into_iter()
                    .map(|p| map::map_project(&self.integration.id, &site, p)),
            );
            if got < PAGE || out.len() >= page.total_count {
                break;
            }
        }
        Ok(out)
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        Err(self.unsupported("list_ssh_keys"))
    }

    async fn upload_ssh_key(&self, _title: &str, _public_key: &str) -> Result<HostedSshKey> {
        Err(self.unsupported("upload_ssh_key"))
    }

    async fn create_pull_request(&self, _req: &NewPullRequest) -> Result<CreatedPullRequest> {
        Err(self.unsupported("create_pull_request"))
    }

    fn conversation_caps(&self, kind: ItemKind) -> ConversationCaps {
        ConversationCaps::redmine(
            kind,
            self.integration.markup == gitpoulpi_config::RedmineMarkup::Textile,
        )
    }

    async fn posting_identity(&self, _project: &str) -> Result<PostingIdentity> {
        self.conv_identity().await
    }

    async fn item_timeline(
        &self,
        _project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        self.conv_timeline(key, older, opts).await
    }

    async fn post_item_comment(
        &self,
        _project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        self.conv_post(key, body, internal).await
    }

    async fn edit_item_comment(
        &self,
        _project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        self.conv_edit(key, comment_id, body).await
    }

    async fn current_user(&self) -> Result<HostedUser> {
        let me: map::CurrentUser = self
            .get_json("/users/current.json", "checking the Redmine API key")
            .await?;
        Ok(me.user.into_user(&self.site()))
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        Ok(TokenCheck {
            user: self.current_user().await?,
            // An API key reports no scope list.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
            ..Default::default()
        })
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let status = match filter.state {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "*",
        };
        let mut query = format!(
            "project_id={}&status_id={}&sort=updated_on:desc",
            encode_component(project.trim()),
            encode_component(status)
        );
        if filter.assigned_to_me {
            query.push_str("&assigned_to_id=me");
        }
        let site = self.site();
        let mut out: Vec<IssueSummary> = Vec::new();
        for _ in 0..MAX_PAGES {
            let page: map::IssuesPage = self
                .get_json(
                    &format!("/issues.json?{query}&limit={PAGE}&offset={}", out.len()),
                    "listing Redmine issues",
                )
                .await?;
            let got = page.issues.len();
            out.extend(page.issues.into_iter().map(|r| map::map_issue(&site, r)));
            if got < PAGE || out.len() >= page.total_count || out.len() >= MAX_ISSUES {
                break;
            }
        }
        out.truncate(MAX_ISSUES);
        Ok(out)
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        if !req.labels.is_empty() {
            return Err(no_labels());
        }
        let project = req.project.trim();
        let mut issue = json!({ "project_id": project, "subject": req.title });
        if !req.body.trim().is_empty() {
            issue["description"] = json!(req.body);
        }
        if let Some(tracker) = req.issue_type.as_deref().filter(|t| !t.is_empty()) {
            issue["tracker_id"] = json!(tracker.parse::<u64>().map_err(|_| Error::Api {
                what: "opening a Redmine issue".into(),
                status: 422,
                detail: format!("`{tracker}` is not a tracker id"),
            })?);
        }
        if let Some(status) = req.list_id.as_deref().filter(|t| !t.is_empty()) {
            issue["status_id"] = json!(status.parse::<u64>().unwrap_or(0));
        }
        if let Some(name) = req.assignees.first() {
            issue["assigned_to_id"] = json!(self.member_id(project, name).await?);
        }
        if let Some(version) = req.milestone {
            issue["fixed_version_id"] = json!(version);
        }
        let url = format!("{}/issues.json", self.api());
        let response = self
            .send(
                HttpRequest::post_json(url, &json!({ "issue": issue })),
                "opening a Redmine issue",
            )
            .await?;
        let created: map::Created = response.json()?;
        let id = created.issue.id;
        Ok(CreatedIssue {
            number: id,
            web_url: map::issue_url(&self.site(), id),
            reference: format!("#{id}"),
        })
    }

    /// The project's trackers are the New-issue form's dropdown.
    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let types: Vec<Choice> = self
            .trackers(project)
            .await?
            .into_iter()
            .map(|t| Choice {
                id: t.id.to_string(),
                label: t.name,
            })
            .collect();
        Ok(IssueCreateOptions {
            default_type: types.first().map(|t| t.id.clone()),
            types,
            lists: Vec::new(),
            default_list: None,
        })
    }

    // -- metadata (slice M) -------------------------------------------------

    /// Members of the project; the "login" a picker works with is the
    /// display name, the only identity an issue carries.
    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let q = query.trim().to_lowercase();
        Ok(self
            .members(project)
            .await?
            .into_iter()
            .filter(|m| q.is_empty() || m.name.to_lowercase().contains(&q))
            .map(|m| HostedUser {
                login: m.name.clone(),
                name: m.name,
                id: m.id.to_string(),
                web_url: Some(format!("{}/users/{}", self.site(), m.id)),
            })
            .collect())
    }

    /// Core Redmine has no labels.
    async fn project_labels(&self, _project: &str) -> Result<Vec<HostedLabel>> {
        Ok(Vec::new())
    }

    /// The project's open versions.
    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        Ok(self
            .versions(project)
            .await?
            .iter()
            .filter(|v| v.status == "open")
            .map(milestone_of)
            .collect())
    }

    async fn item_meta(&self, project: &str, number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        let one: map::OneIssue = self
            .get_json(
                &format!("/issues/{number}.json"),
                "reading the metadata of a Redmine issue",
            )
            .await?;
        let issue = one.issue;
        let milestone = match issue.fixed_version {
            Some(v) => {
                let known = self.versions(project).await.unwrap_or_default();
                Some(
                    known
                        .iter()
                        .find(|k| k.id == v.id)
                        .map(milestone_of)
                        .unwrap_or(HostedMilestone {
                            id: v.id,
                            title: v.name,
                            due_on: None,
                            state: String::new(),
                        }),
                )
            }
            None => None,
        };
        Ok(ItemMeta {
            assignees: issue
                .assigned_to
                .map(|a| a.name)
                .filter(|n| !n.is_empty())
                .into_iter()
                .collect(),
            reviewers: Vec::new(),
            labels: Vec::new(),
            milestone,
        })
    }

    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        if edit.labels.as_ref().is_some_and(|l| !l.is_empty()) {
            return Err(no_labels());
        }
        let mut issue = serde_json::Map::new();
        if let Some(assignees) = &edit.assignees {
            let id = match assignees.first() {
                Some(name) => json!(self.member_id(project, name).await?),
                None => Value::Null,
            };
            issue.insert("assigned_to_id".into(), id);
        }
        if let Some(milestone) = edit.milestone {
            issue.insert(
                "fixed_version_id".into(),
                milestone.map(|id| json!(id)).unwrap_or(Value::Null),
            );
        }
        if issue.is_empty() {
            return Ok(());
        }
        self.edit("editing a Redmine issue", number, Value::Object(issue))
            .await
    }

    // -- statuses (the board's columns) -------------------------------------

    async fn issue_statuses(&self, _project: &str) -> Result<Vec<String>> {
        Ok(self.statuses().await?.into_iter().map(|s| s.name).collect())
    }

    async fn set_issue_status(&self, _project: &str, number: u64, status: &str) -> Result<()> {
        self.set_status_named(number, status, "moving a Redmine issue")
            .await
    }

    /// Close: the first closed status; reopen: the first open one.
    async fn set_issue_state(&self, _project: &str, number: u64, closed: bool) -> Result<()> {
        let id = self
            .statuses()
            .await?
            .into_iter()
            .find(|s| s.is_closed == closed)
            .map(|s| s.id)
            .ok_or_else(|| Error::Api {
                what: "changing a Redmine issue".into(),
                status: 422,
                detail: "Redmine has no status for that".into(),
            })?;
        self.edit(
            "changing a Redmine issue",
            number,
            json!({ "status_id": id }),
        )
        .await
    }

    /// A status the tool can change: a card move is a status change.
    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::LocalStatus
    }
}
