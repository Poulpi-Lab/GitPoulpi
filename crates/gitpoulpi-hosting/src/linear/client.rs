//! The Linear client: one GraphQL endpoint, cursor paging.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::auth::auth_headers;
use crate::client::{
    CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest, MAX_PAGES,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind};
use crate::model::{
    Choice, HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::map::{self, Connection, CLOSED_TYPES};
use super::queries;

/// A list shows at most this many issues.
const MAX_ISSUES: usize = 500;

pub(crate) struct LinearClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
    /// Team key to team id, so a form that needs the id three times asks once.
    teams: Mutex<HashMap<String, String>>,
}

#[derive(serde::Deserialize)]
struct Envelope {
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    errors: Vec<GqlError>,
}

#[derive(serde::Deserialize)]
struct GqlError {
    #[serde(default)]
    message: String,
    #[serde(default)]
    extensions: Option<Value>,
}

impl GqlError {
    fn extension(&self, key: &str) -> String {
        self.extensions
            .as_ref()
            .and_then(|e| e.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase()
    }
}

/// Turns what Linear said into the crate's typed errors.
fn graphql_error(what: &str, status: u16, errors: &[GqlError]) -> Error {
    let detail = errors
        .iter()
        .map(|e| e.message.as_str())
        .filter(|m| !m.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    let any = |f: &dyn Fn(&GqlError) -> bool| errors.iter().any(f);
    if any(&|e| e.extension("code") == "ratelimited") {
        return Error::RateLimited {
            host: what.to_string(),
            retry_in_secs: 60,
        };
    }
    if status == 401 || any(&|e| e.extension("type").contains("authentication")) {
        return Error::Unauthorized {
            what: what.to_string(),
            detail,
        };
    }
    if status == 403 || any(&|e| e.extension("type").contains("forbidden")) {
        return Error::Forbidden {
            what: what.to_string(),
            detail,
        };
    }
    Error::Api {
        what: what.to_string(),
        status,
        detail,
    }
}

/// The `IssueFilter` of a team's list. Pure.
pub(super) fn issues_filter(team_key: &str, filter: &IssueFilter) -> Value {
    let mut f = json!({ "team": { "key": { "eq": team_key.trim() } } });
    match filter.state {
        StateFilter::Open => f["state"] = json!({ "type": { "nin": CLOSED_TYPES } }),
        StateFilter::Closed => f["state"] = json!({ "type": { "in": CLOSED_TYPES } }),
        StateFilter::All => {}
    }
    if filter.assigned_to_me {
        f["assignee"] = json!({ "isMe": { "eq": true } });
    }
    f
}

impl LinearClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
            teams: Mutex::new(HashMap::new()),
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/graphql", self.integration.api_base())
    }

    /// Runs one document and returns its `data`.
    async fn graphql(&self, what: &str, query: &str, variables: Value) -> Result<Value> {
        let body = json!({ "query": query, "variables": variables });
        let mut request =
            HttpRequest::post_json(self.endpoint(), &body).header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        let response = self.transport.send(request).await?;
        let envelope: Option<Envelope> = serde_json::from_str(&response.body).ok();
        if !response.is_success() {
            // 429 is the rate limit proper; 400 can carry `RATELIMITED` or a
            // validation message in `errors`.
            if let Some(e) = envelope.as_ref().filter(|e| !e.errors.is_empty()) {
                return Err(graphql_error(what, response.status, &e.errors));
            }
            response.error_for_status(what)?;
        }
        let envelope = envelope.ok_or_else(|| Error::Decode(format!("{what}: not JSON")))?;
        match envelope.data {
            Some(data) if !data.is_null() => Ok(data),
            _ => Err(graphql_error(what, response.status, &envelope.errors)),
        }
    }

    async fn query<T: DeserializeOwned>(
        &self,
        what: &str,
        query: &str,
        variables: Value,
        pointer: &str,
    ) -> Result<T> {
        let data = self.graphql(what, query, variables).await?;
        let part = data
            .pointer(pointer)
            .cloned()
            .ok_or_else(|| Error::Decode(format!("{what}: no {pointer} in the answer")))?;
        serde_json::from_value(part).map_err(|e| Error::Decode(format!("{what}: {e}")))
    }

    /// Every node of the connection at `pointer`, following `endCursor`.
    /// `variables` must be an object; `after` is added to it.
    async fn paged<T: DeserializeOwned>(
        &self,
        what: &str,
        query: &str,
        variables: Value,
        pointer: &str,
        cap: usize,
    ) -> Result<Vec<T>> {
        let mut out = Vec::new();
        let mut after: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut vars = variables.clone();
            vars["after"] = json!(after);
            let page: Connection<T> = self.query(what, query, vars, pointer).await?;
            let empty = page.nodes.is_empty();
            out.extend(page.nodes);
            let next = page
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
            if empty || next.is_none() || out.len() >= cap {
                break;
            }
            after = next;
        }
        Ok(out)
    }

    /// The id of the team with this key (cached).
    async fn team_id(&self, key: &str) -> Result<String> {
        let key = key.trim().to_string();
        if let Some(id) = self.teams.lock().unwrap().get(&key) {
            return Ok(id.clone());
        }
        let rows: Connection<map::TeamRow> = self
            .query(
                "finding the Linear team",
                queries::TEAM_BY_KEY,
                json!({ "key": key }),
                "/teams",
            )
            .await?;
        let team = rows.nodes.into_iter().next().ok_or_else(|| Error::Api {
            what: "finding the Linear team".into(),
            status: 404,
            detail: format!("no team with the key {key}"),
        })?;
        self.teams.lock().unwrap().insert(key, team.id.clone());
        Ok(team.id)
    }

    /// The team's workflow states in board order, and the default one.
    async fn states(&self, team_key: &str) -> Result<(Vec<map::StateRow>, Option<String>)> {
        let id = self.team_id(team_key).await?;
        let mut states = Vec::new();
        let mut default = None;
        let mut after: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let data = self
                .graphql(
                    "reading the Linear workflow states",
                    queries::STATES,
                    json!({ "id": id, "after": after }),
                )
                .await?;
            if default.is_none() {
                default = data
                    .pointer("/team/defaultIssueState/id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
            }
            let page: Connection<map::StateRow> = serde_json::from_value(
                data.pointer("/team/states").cloned().unwrap_or(Value::Null),
            )
            .map_err(|e| Error::Decode(e.to_string()))?;
            states.extend(page.nodes);
            after = page
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
            if after.is_none() {
                break;
            }
        }
        map::sort_states(&mut states);
        Ok((states, default))
    }

    async fn members(&self, team_key: &str) -> Result<Vec<map::UserRow>> {
        let id = self.team_id(team_key).await?;
        let rows: Vec<map::UserRow> = self
            .paged(
                "listing the Linear team members",
                queries::MEMBERS,
                json!({ "id": id }),
                "/team/members",
                1000,
            )
            .await?;
        Ok(rows.into_iter().filter(|u| u.active).collect())
    }

    async fn label_rows(&self, team_key: &str) -> Result<Vec<map::LabelRow>> {
        let id = self.team_id(team_key).await?;
        self.paged(
            "listing the Linear labels",
            queries::LABELS,
            json!({ "id": id }),
            "/issueLabels",
            1000,
        )
        .await
    }

    /// The id of the issue `<KEY>-<number>`.
    async fn issue_id(&self, team_key: &str, number: u64) -> Result<String> {
        let filter = json!({
            "team": { "key": { "eq": team_key.trim() } },
            "number": { "eq": number },
        });
        let rows: Connection<map::ItemRow> = self
            .query(
                "finding the Linear issue",
                queries::ISSUE_ID,
                json!({ "filter": filter }),
                "/issues",
            )
            .await?;
        rows.nodes
            .into_iter()
            .next()
            .map(|r| r.id)
            .ok_or_else(|| Error::Api {
                what: "finding the Linear issue".into(),
                status: 404,
                detail: format!("no issue {}-{number}", team_key.trim()),
            })
    }

    async fn resolve_user(&self, team_key: &str, login: &str) -> Result<String> {
        let wanted = login.trim();
        self.members(team_key)
            .await?
            .into_iter()
            .find(|u| {
                u.handle().eq_ignore_ascii_case(wanted)
                    || u.name.eq_ignore_ascii_case(wanted)
                    || (!u.email.is_empty() && u.email.eq_ignore_ascii_case(wanted))
                    || u.id == wanted
            })
            .map(|u| u.id)
            .ok_or_else(|| Error::Api {
                what: "assigning a Linear issue".into(),
                status: 422,
                detail: format!("{wanted} is not a member of the team"),
            })
    }

    /// Label names to ids; an unknown one is refused before anything is sent.
    async fn resolve_labels(&self, team_key: &str, names: &[String]) -> Result<Vec<String>> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let rows = self.label_rows(team_key).await?;
        names
            .iter()
            .map(|n| {
                rows.iter()
                    .find(|r| r.name.eq_ignore_ascii_case(n.trim()))
                    .map(|r| r.id.clone())
                    .ok_or_else(|| Error::Api {
                        what: "labelling a Linear issue".into(),
                        status: 422,
                        detail: format!("the team has no label {n}"),
                    })
            })
            .collect()
    }

    async fn cycle_id(&self, team_key: &str, number: u64) -> Result<String> {
        let id = self.team_id(team_key).await?;
        let rows: Connection<map::CycleRow> = self
            .query(
                "finding the Linear cycle",
                queries::CYCLE_BY_NUMBER,
                json!({ "id": id, "number": number }),
                "/cycles",
            )
            .await?;
        rows.nodes
            .into_iter()
            .next()
            .map(|c| c.id)
            .ok_or_else(|| Error::Api {
                what: "finding the Linear cycle".into(),
                status: 404,
                detail: format!("the team has no cycle {number}"),
            })
    }

    async fn update(&self, what: &str, id: &str, input: Value) -> Result<()> {
        let data = self
            .graphql(
                what,
                queries::ISSUE_UPDATE,
                json!({ "id": id, "input": input }),
            )
            .await?;
        if data
            .pointer("/issueUpdate/success")
            .and_then(|v| v.as_bool())
            == Some(false)
        {
            return Err(Error::Api {
                what: what.to_string(),
                status: 422,
                detail: "Linear did not apply the change".into(),
            });
        }
        Ok(())
    }

    async fn state_id_named(&self, team_key: &str, name: &str) -> Result<String> {
        let (states, _) = self.states(team_key).await?;
        states
            .into_iter()
            .find(|s| s.name.eq_ignore_ascii_case(name.trim()))
            .map(|s| s.id)
            .ok_or_else(|| Error::Api {
                what: "moving a Linear issue".into(),
                status: 422,
                detail: format!("the team has no state {name}"),
            })
    }
}

#[async_trait::async_trait]
impl HostingClient for LinearClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let mut out = Vec::new();
        let mut after: Option<String> = None;
        let mut url_key = String::new();
        for _ in 0..MAX_PAGES {
            let data = self
                .graphql(
                    "listing Linear teams",
                    queries::TEAMS,
                    json!({ "after": after }),
                )
                .await?;
            if url_key.is_empty() {
                if let Some(o) = data.get("organization") {
                    if let Ok(o) = serde_json::from_value::<map::Organization>(o.clone()) {
                        url_key = o.url_key;
                    }
                }
            }
            let page: Connection<map::TeamRow> =
                serde_json::from_value(data.get("teams").cloned().unwrap_or(Value::Null))
                    .map_err(|e| Error::Decode(e.to_string()))?;
            out.extend(page.nodes);
            after = page
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
            if after.is_none() {
                break;
            }
        }
        let root = map::workspace_root(&self.integration.url, &url_key);
        Ok(out
            .into_iter()
            .map(|t| map::map_team(&self.integration.id, &root, t))
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
            .query(
                "checking the Linear token",
                queries::VIEWER,
                json!({}),
                "/viewer",
            )
            .await?;
        Ok(user.into_user())
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        Ok(TokenCheck {
            user: self.current_user().await?,
            // A personal API key reports no scope list.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
        })
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let rows: Vec<map::IssueRow> = self
            .paged(
                "listing Linear issues",
                queries::ISSUES,
                json!({ "filter": issues_filter(project, filter) }),
                "/issues",
                MAX_ISSUES,
            )
            .await?;
        Ok(rows
            .into_iter()
            .take(MAX_ISSUES)
            .map(map::map_issue)
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let team = req.project.trim();
        let mut input = json!({
            "teamId": self.team_id(team).await?,
            "title": req.title,
        });
        if !req.body.trim().is_empty() {
            input["description"] = json!(req.body);
        }
        if !req.labels.is_empty() {
            input["labelIds"] = json!(self.resolve_labels(team, &req.labels).await?);
        }
        // Linear has one assignee; the first wins.
        if let Some(login) = req.assignees.first() {
            input["assigneeId"] = json!(self.resolve_user(team, login).await?);
        }
        if let Some(cycle) = req.milestone {
            input["cycleId"] = json!(self.cycle_id(team, cycle).await?);
        }
        if let Some(state) = req.list_id.as_deref().filter(|s| !s.is_empty()) {
            input["stateId"] = json!(state);
        }
        let data = self
            .graphql(
                "opening a Linear issue",
                queries::ISSUE_CREATE,
                json!({ "input": input }),
            )
            .await?;
        let issue = data
            .pointer("/issueCreate/issue")
            .filter(|v| !v.is_null())
            .cloned()
            .ok_or_else(|| Error::Api {
                what: "opening a Linear issue".into(),
                status: 422,
                detail: "Linear did not create the issue".into(),
            })?;
        let created: map::CreatedIssue =
            serde_json::from_value(issue).map_err(|e| Error::Decode(e.to_string()))?;
        Ok(CreatedIssue {
            number: created.number as u64,
            web_url: created.url,
            reference: created.identifier,
        })
    }

    /// The states are the "lists" of the New-issue form.
    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let (states, default) = self.states(project).await?;
        let lists: Vec<Choice> = states
            .into_iter()
            .filter(|s| !CLOSED_TYPES.contains(&s.kind.as_str()) && s.kind != "triage")
            .map(|s| Choice {
                id: s.id,
                label: s.name,
            })
            .collect();
        Ok(IssueCreateOptions {
            types: Vec::new(),
            default_list: default.filter(|d| lists.iter().any(|l| &l.id == d)),
            lists,
            default_type: None,
        })
    }

    // -- metadata (slice M) -------------------------------------------------

    async fn project_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let q = query.trim().to_lowercase();
        Ok(self
            .members(project)
            .await?
            .into_iter()
            .filter(|u| {
                q.is_empty()
                    || u.handle().to_lowercase().contains(&q)
                    || u.name.to_lowercase().contains(&q)
            })
            .map(map::UserRow::into_user)
            .collect())
    }

    async fn project_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        Ok(self
            .label_rows(project)
            .await?
            .into_iter()
            .map(map::LabelRow::into_label)
            .collect())
    }

    /// The team's open cycles, by number (the milestone of Linear).
    async fn project_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        let id = self.team_id(project).await?;
        let rows: Vec<map::CycleRow> = self
            .paged(
                "listing Linear cycles",
                queries::CYCLES,
                json!({ "id": id }),
                "/team/cycles",
                200,
            )
            .await?;
        let mut cycles: Vec<HostedMilestone> = rows
            .into_iter()
            .map(map::CycleRow::into_milestone)
            .collect();
        cycles.sort_by_key(|c| c.id);
        Ok(cycles)
    }

    async fn item_meta(&self, project: &str, number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        let filter = json!({
            "team": { "key": { "eq": project.trim() } },
            "number": { "eq": number },
        });
        let rows: Connection<map::ItemRow> = self
            .query(
                "reading the metadata of a Linear issue",
                queries::ISSUE_META,
                json!({ "filter": filter }),
                "/issues",
            )
            .await?;
        let row = rows.nodes.into_iter().next().ok_or_else(|| Error::Api {
            what: "reading the metadata of a Linear issue".into(),
            status: 404,
            detail: format!("no issue {}-{number}", project.trim()),
        })?;
        Ok(ItemMeta {
            assignees: row
                .assignee
                .map(|u| u.handle().to_string())
                .into_iter()
                .collect(),
            reviewers: Vec::new(),
            labels: row
                .labels
                .map(|c| c.nodes.into_iter().map(|l| l.name).collect())
                .unwrap_or_default(),
            milestone: row.cycle.map(map::CycleRow::into_milestone),
        })
    }

    async fn update_issue(&self, project: &str, number: u64, edit: &IssueEdit) -> Result<()> {
        let mut input = serde_json::Map::new();
        if let Some(assignees) = &edit.assignees {
            let value = match assignees.first() {
                Some(login) => json!(self.resolve_user(project, login).await?),
                None => Value::Null,
            };
            input.insert("assigneeId".into(), value);
        }
        if let Some(labels) = &edit.labels {
            input.insert(
                "labelIds".into(),
                json!(self.resolve_labels(project, labels).await?),
            );
        }
        if let Some(cycle) = edit.milestone {
            let value = match cycle {
                Some(n) => json!(self.cycle_id(project, n).await?),
                None => Value::Null,
            };
            input.insert("cycleId".into(), value);
        }
        if input.is_empty() {
            return Ok(());
        }
        let id = self.issue_id(project, number).await?;
        self.update("editing a Linear issue", &id, Value::Object(input))
            .await
    }

    // -- states (the board's columns) ---------------------------------------

    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        let (states, _) = self.states(project).await?;
        let mut names: Vec<String> = Vec::new();
        for s in states {
            if !names.iter().any(|n| n.eq_ignore_ascii_case(&s.name)) {
                names.push(s.name);
            }
        }
        Ok(names)
    }

    async fn set_issue_status(&self, project: &str, number: u64, status: &str) -> Result<()> {
        let state = self.state_id_named(project, status).await?;
        let id = self.issue_id(project, number).await?;
        self.update("moving a Linear issue", &id, json!({ "stateId": state }))
            .await
    }

    /// Close: the first `completed` state (Done); reopen: the first
    /// `unstarted` one (Todo), else the backlog's.
    async fn set_issue_state(&self, project: &str, number: u64, closed: bool) -> Result<()> {
        let (states, _) = self.states(project).await?;
        let wanted: &[&str] = if closed {
            &["completed", "canceled"]
        } else {
            &["unstarted", "backlog"]
        };
        let state = wanted
            .iter()
            .find_map(|t| states.iter().find(|s| s.kind == *t))
            .ok_or_else(|| Error::Api {
                what: "changing a Linear issue".into(),
                status: 422,
                detail: "the team has no workflow state for that".into(),
            })?;
        let id = self.issue_id(project, number).await?;
        self.update(
            "changing a Linear issue",
            &id,
            json!({ "stateId": state.id }),
        )
        .await
    }
}
