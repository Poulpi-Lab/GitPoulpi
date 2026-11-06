//! The YouTrack client (Cloud and Server).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::auth::auth_headers;
use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
    MAX_PAGES,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{list_delta, HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind};
use crate::model::{Choice, HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, TokenCheck};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::map::{self, BundleValue, ProjectField};
use super::query::{epoch_ms_to_iso, issues_query};

/// Rows of one page.
const PAGE: usize = 100;
/// A list shows at most this many issues.
const MAX_ISSUES: usize = 500;

const BUNDLE_FIELDS: &str =
    "field(name),bundle(values(name,isResolved,archived,released,releaseDate))";

pub(crate) struct YouTrackClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
    /// Short name to database id, asked once.
    project_ids: Mutex<HashMap<String, String>>,
}

impl YouTrackClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
            project_ids: Mutex::new(HashMap::new()),
        }
    }

    fn site(&self) -> String {
        self.integration
            .url
            .trim()
            .trim_end_matches('/')
            .to_string()
    }

    fn api(&self) -> String {
        self.integration.api_base()
    }

    fn authed(&self, request: HttpRequest) -> HttpRequest {
        let mut request = request.header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T> {
        let url = format!("{}{path}", self.api());
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
        response.error_for_status(what)?;
        response.json()
    }

    /// A POST (YouTrack edits with POST) or DELETE whose answer is not read.
    async fn send(&self, request: HttpRequest, what: &str) -> Result<crate::http::HttpResponse> {
        let response = self.transport.send(self.authed(request)).await?;
        response.error_for_status(what)?;
        Ok(response)
    }

    /// Every row of a list endpoint, `$top` / `$skip` paged. `path` already
    /// carries a `?`.
    async fn paged<T: DeserializeOwned>(
        &self,
        path: &str,
        what: &str,
        cap: usize,
    ) -> Result<Vec<T>> {
        let mut out: Vec<T> = Vec::new();
        for _ in 0..MAX_PAGES {
            let rows: Vec<T> = self
                .get_json(&format!("{path}&$top={PAGE}&$skip={}", out.len()), what)
                .await?;
            let got = rows.len();
            out.extend(rows);
            if got < PAGE || out.len() >= cap {
                break;
            }
        }
        Ok(out)
    }

    async fn project_rows(&self) -> Result<Vec<map::ProjectRow>> {
        let rows: Vec<map::ProjectRow> = self
            .paged(
                &format!("/admin/projects?fields={}", map::PROJECT_FIELDS),
                "listing YouTrack projects",
                2000,
            )
            .await?;
        let mut ids = self.project_ids.lock().unwrap();
        for r in &rows {
            ids.insert(r.short_name.clone(), r.id.clone());
        }
        Ok(rows)
    }

    async fn project_id(&self, short: &str) -> Result<String> {
        let short = short.trim();
        if let Some(id) = self.project_ids.lock().unwrap().get(short) {
            return Ok(id.clone());
        }
        self.project_rows().await?;
        self.project_ids
            .lock()
            .unwrap()
            .get(short)
            .cloned()
            .ok_or_else(|| Error::Api {
                what: "finding the YouTrack project".into(),
                status: 404,
                detail: format!("no project {short}"),
            })
    }

    async fn project_fields(&self, short: &str) -> Result<Vec<ProjectField>> {
        let id = self.project_id(short).await?;
        self.paged(
            &format!(
                "/admin/projects/{}/customFields?fields={}",
                encode_component(&id),
                BUNDLE_FIELDS
            ),
            "reading the YouTrack project fields",
            200,
        )
        .await
    }

    async fn bundle(&self, short: &str, field: &str) -> Result<Vec<BundleValue>> {
        Ok(self
            .project_fields(short)
            .await?
            .into_iter()
            .find(|f| f.field.name.eq_ignore_ascii_case(field))
            .and_then(|f| f.bundle)
            .map(|b| b.values)
            .unwrap_or_default())
    }

    /// The open `Fix versions`, in the bundle's order. A milestone id is
    /// the 1-based position here.
    async fn versions(&self, short: &str) -> Result<Vec<BundleValue>> {
        Ok(self
            .bundle(short, "Fix versions")
            .await?
            .into_iter()
            .filter(|v| !v.archived && !v.released)
            .collect())
    }

    async fn tag_rows(&self) -> Result<Vec<map::TagRow>> {
        self.paged(
            "/tags?fields=id,name,color(background)",
            "listing YouTrack tags",
            1000,
        )
        .await
    }

    async fn tag_ids(&self, names: &[String]) -> Result<Vec<(String, String)>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let rows = self.tag_rows().await?;
        names
            .iter()
            .map(|n| {
                rows.iter()
                    .find(|r| r.name.eq_ignore_ascii_case(n.trim()))
                    .map(|r| (r.name.clone(), r.id.clone()))
                    .ok_or_else(|| Error::Api {
                        what: "tagging a YouTrack issue".into(),
                        status: 422,
                        detail: format!("there is no tag {n}"),
                    })
            })
            .collect()
    }

    fn reference(project: &str, number: u64) -> String {
        format!("{}-{number}", project.trim())
    }

    async fn edit(&self, what: &str, reference: &str, body: Value) -> Result<()> {
        let url = format!(
            "{}/issues/{}?fields=id",
            self.api(),
            encode_component(reference)
        );
        self.send(HttpRequest::post_json(url, &body), what).await?;
        Ok(())
    }

    async fn set_state(&self, project: &str, number: u64, state: &str) -> Result<()> {
        self.edit(
            "moving a YouTrack issue",
            &Self::reference(project, number),
            json!({ "customFields": [state_field(state)] }),
        )
        .await
    }
}

fn state_field(name: &str) -> Value {
    json!({ "name": "State", "$type": "StateIssueCustomField", "value": { "name": name } })
}

fn assignee_field(login: Option<&str>) -> Value {
    json!({
        "name": "Assignee",
        "$type": "SingleUserIssueCustomField",
        "value": login.map(|l| json!({ "login": l })),
    })
}

fn versions_field(name: Option<&str>) -> Value {
    json!({
        "name": "Fix versions",
        "$type": "MultiVersionIssueCustomField",
        "value": name.map(|n| json!([{ "name": n }])).unwrap_or_else(|| json!([])),
    })
}

fn milestone_of(position: usize, v: &BundleValue) -> HostedMilestone {
    HostedMilestone {
        id: position as u64 + 1,
        title: v.name.clone(),
        due_on: v
            .release_date
            .map(epoch_ms_to_iso)
            .filter(|d| !d.is_empty())
            .map(|d| d.split('T').next().unwrap_or(&d).to_string()),
        state: "open".to_string(),
    }
}

#[async_trait::async_trait]
impl HostingClient for YouTrackClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let site = self.site();
        Ok(self
            .project_rows()
            .await?
            .into_iter()
            .filter(|p| !p.archived)
            .map(|p| map::map_project(&self.integration.id, &site, p))
            .collect())
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

    async fn current_user(&self) -> Result<HostedUser> {
        let user: map::UserRow = self
            .get_json(
                &format!("/users/me?fields={}", map::USER_FIELDS),
                "checking the YouTrack token",
            )
            .await?;
        Ok(user.into_user(&self.site()))
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        Ok(TokenCheck {
            user: self.current_user().await?,
            // A permanent token reports no scope list.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
            ..Default::default()
        })
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let query = encode_component(&issues_query(project, filter));
        let rows: Vec<map::IssueRow> = self
            .paged(
                &format!("/issues?fields={}&query={query}", map::ISSUE_FIELDS),
                "listing YouTrack issues",
                MAX_ISSUES,
            )
            .await?;
        let site = self.site();
        Ok(rows
            .into_iter()
            .take(MAX_ISSUES)
            .map(|r| map::map_issue(&site, r))
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let short = req.project.trim();
        let mut body = json!({
            "project": { "id": self.project_id(short).await? },
            "summary": req.title,
        });
        if !req.body.trim().is_empty() {
            body["description"] = json!(req.body);
        }
        let mut fields = Vec::new();
        if let Some(state) = req.list_id.as_deref().filter(|s| !s.is_empty()) {
            fields.push(state_field(state));
        }
        if let Some(login) = req.assignees.first() {
            fields.push(assignee_field(Some(login)));
        }
        if let Some(n) = req.milestone {
            let versions = self.versions(short).await?;
            let name = n
                .checked_sub(1)
                .and_then(|i| versions.get(i as usize))
                .map(|v| v.name.clone())
                .ok_or_else(|| Error::Api {
                    what: "opening a YouTrack issue".into(),
                    status: 422,
                    detail: format!("the project has no fix version number {n}"),
                })?;
            fields.push(versions_field(Some(&name)));
        }
        if !fields.is_empty() {
            body["customFields"] = json!(fields);
        }
        if !req.labels.is_empty() {
            let tags = self.tag_ids(&req.labels).await?;
            body["tags"] = json!(tags
                .iter()
                .map(|(_, id)| json!({ "id": id }))
                .collect::<Vec<_>>());
        }
        let url = format!("{}/issues?fields=id,idReadable", self.api());
        let response = self
            .send(
                HttpRequest::post_json(url, &body),
                "opening a YouTrack issue",
            )
            .await?;
        let created: map::CreatedRow = response.json()?;
        Ok(CreatedIssue {
            number: map::number_of(&created.id_readable),
            web_url: map::issue_url(&self.site(), &created.id_readable),
            reference: created.id_readable,
        })
    }

    /// The unresolved `State` values are the "lists" of the New-issue form.
    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let lists: Vec<Choice> = self
            .bundle(project, "State")
            .await?
            .into_iter()
            .filter(|s| !s.is_resolved)
            .map(|s| Choice {
                id: s.name.clone(),
                label: s.name,
            })
            .collect();
        Ok(IssueCreateOptions {
            types: Vec::new(),
            default_list: lists.first().map(|l| l.id.clone()),
            lists,
            default_type: None,
        })
    }

    // -- metadata (slice M) -------------------------------------------------

    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let id = self.project_id(project).await?;
        let rows: Vec<map::UserRow> = self
            .paged(
                &format!(
                    "/admin/projects/{}/team/users?fields={}",
                    encode_component(&id),
                    map::USER_FIELDS
                ),
                "listing the YouTrack project team",
                1000,
            )
            .await?;
        let q = query.trim().to_lowercase();
        let site = self.site();
        Ok(rows
            .into_iter()
            .filter(|u| {
                q.is_empty()
                    || u.login.to_lowercase().contains(&q)
                    || u.display().to_lowercase().contains(&q)
            })
            .map(|u| u.into_user(&site))
            .collect())
    }

    async fn project_labels(&self, _project: &str) -> Result<Vec<HostedLabel>> {
        // Tags belong to a user or are shared; the token sees its own.
        Ok(self
            .tag_rows()
            .await?
            .into_iter()
            .map(map::TagRow::into_label)
            .collect())
    }

    /// The project's unreleased fix versions.
    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        Ok(self
            .versions(project)
            .await?
            .iter()
            .enumerate()
            .map(|(i, v)| milestone_of(i, v))
            .collect())
    }

    async fn item_meta(&self, project: &str, number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        let reference = Self::reference(project, number);
        let row: map::IssueRow = self
            .get_json(
                &format!(
                    "/issues/{}?fields={}",
                    encode_component(&reference),
                    map::META_FIELDS
                ),
                "reading the metadata of a YouTrack issue",
            )
            .await?;
        let version = row.field("Fix versions").into_iter().next();
        let milestone = match version {
            Some(name) => {
                let versions = self.versions(project).await?;
                Some(
                    versions
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.name == name)
                        .map(|(i, v)| milestone_of(i, v))
                        .unwrap_or(HostedMilestone {
                            id: 0,
                            title: name,
                            due_on: None,
                            state: String::new(),
                        }),
                )
            }
            None => None,
        };
        Ok(ItemMeta {
            assignees: row.field("Assignee"),
            reviewers: Vec::new(),
            labels: row.tags.into_iter().map(|t| t.name).collect(),
            milestone,
        })
    }

    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        let reference = Self::reference(project, number);
        let mut fields = Vec::new();
        if let Some(assignees) = &edit.assignees {
            fields.push(assignee_field(assignees.first().map(String::as_str)));
        }
        if let Some(milestone) = edit.milestone {
            let name = match milestone {
                Some(n) => {
                    let versions = self.versions(project).await?;
                    Some(
                        n.checked_sub(1)
                            .and_then(|i| versions.get(i as usize))
                            .map(|v| v.name.clone())
                            .ok_or_else(|| Error::Api {
                                what: "editing a YouTrack issue".into(),
                                status: 422,
                                detail: format!("the project has no fix version number {n}"),
                            })?,
                    )
                }
                None => None,
            };
            fields.push(versions_field(name.as_deref()));
        }
        if !fields.is_empty() {
            self.edit(
                "editing a YouTrack issue",
                &reference,
                json!({ "customFields": fields }),
            )
            .await?;
        }
        if let Some(wanted) = &edit.labels {
            let have = self
                .item_meta(project, number, MetaKind::Issue)
                .await?
                .labels;
            let (add, remove) = list_delta(&have, wanted);
            let add_ids = self.tag_ids(&add).await?;
            let remove_ids = self.tag_ids(&remove).await?;
            let base = format!(
                "{}/issues/{}/tags",
                self.api(),
                encode_component(&reference)
            );
            for (_, id) in add_ids {
                self.send(
                    HttpRequest::post_json(format!("{base}?fields=id"), &json!({ "id": id })),
                    "tagging a YouTrack issue",
                )
                .await?;
            }
            for (_, id) in remove_ids {
                self.send(
                    HttpRequest::delete(format!("{base}/{}", encode_component(&id))),
                    "untagging a YouTrack issue",
                )
                .await?;
            }
        }
        Ok(())
    }

    // -- states (the board's columns) ---------------------------------------

    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        Ok(self
            .bundle(project, "State")
            .await?
            .into_iter()
            .map(|s| s.name)
            .collect())
    }

    async fn set_issue_status(&self, project: &str, number: u64, status: &str) -> Result<()> {
        let states = self.bundle(project, "State").await?;
        let name = states
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(status.trim()))
            .map(|s| s.name.clone())
            .ok_or_else(|| Error::Api {
                what: "moving a YouTrack issue".into(),
                status: 422,
                detail: format!("the project has no state {status}"),
            })?;
        self.set_state(project, number, &name).await
    }

    /// Close: the first resolved `State`; reopen: the first unresolved one.
    async fn set_issue_state(&self, project: &str, number: u64, closed: bool) -> Result<()> {
        let states = self.bundle(project, "State").await?;
        let name = states
            .iter()
            .find(|s| s.is_resolved == closed)
            .map(|s| s.name.clone())
            .ok_or_else(|| Error::Api {
                what: "changing a YouTrack issue".into(),
                status: 422,
                detail: "the project has no State value for that".into(),
            })?;
        self.set_state(project, number, &name).await
    }
}
