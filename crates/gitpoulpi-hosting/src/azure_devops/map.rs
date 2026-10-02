//! Wire shapes of the Azure DevOps responses and their mapping onto the
//! host-agnostic model. Pure: no I/O.

use serde::Deserialize;
use serde_json::Value;

use crate::model::{IssueState, IssueSummary, PullRequestState, PullRequestSummary};
use crate::project::HostedProject;

/// Work item states read as closed when the type's state categories are
/// not fetched (plan D4).
pub(super) const CLOSED_STATES: [&str; 4] = ["Done", "Closed", "Removed", "Resolved"];

#[derive(Deserialize)]
pub(super) struct List<T> {
    #[serde(default = "Vec::new")]
    pub value: Vec<T>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct Project {
    pub name: String,
    pub visibility: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct Repo {
    pub id: String,
    pub name: String,
    pub project: Project,
    pub remote_url: Option<String>,
    pub ssh_url: Option<String>,
    pub web_url: Option<String>,
    pub default_branch: Option<String>,
    pub is_disabled: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct Identity {
    pub id: String,
    pub display_name: String,
    pub unique_name: String,
    pub is_container: bool,
    pub vote: i32,
}

impl Identity {
    fn name(&self) -> String {
        if self.display_name.is_empty() {
            self.unique_name.clone()
        } else {
            self.display_name.clone()
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct ForkSource {
    pub repository: Option<Repo>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct Pull {
    pub pull_request_id: u64,
    pub title: String,
    pub status: String,
    pub is_draft: bool,
    pub source_ref_name: String,
    pub target_ref_name: String,
    pub created_by: Identity,
    pub creation_date: String,
    pub closed_date: Option<String>,
    pub reviewers: Vec<Identity>,
    pub repository: Repo,
    pub fork_source: Option<ForkSource>,
}

pub(super) fn branch_name(reference: &str) -> String {
    reference
        .strip_prefix("refs/heads/")
        .unwrap_or(reference)
        .to_string()
}

pub(super) fn pull_state(status: &str, is_draft: bool) -> PullRequestState {
    match status {
        "completed" => PullRequestState::Merged,
        "abandoned" => PullRequestState::Closed,
        _ if is_draft => PullRequestState::Draft,
        _ => PullRequestState::Open,
    }
}

pub(super) fn map_pull(root: &str, me: Option<&str>, p: Pull) -> PullRequestSummary {
    let project = p.repository.project.name.clone();
    let repo = p.repository.name.clone();
    let spec = crate::spec::spec_for(gitpoulpi_config::ApiFamily::AzureDevops);
    let web_url = (spec.pr_web_url)(root, &format!("{project}/{repo}"), p.pull_request_id)
        .unwrap_or_default();
    let source_project = p
        .fork_source
        .as_ref()
        .and_then(|f| f.repository.as_ref())
        .filter(|r| !r.id.is_empty() && r.id != p.repository.id)
        .map(|r| format!("{}/{}", r.project.name, r.name));
    let review_requested = me.is_some_and(|me| {
        p.reviewers
            .iter()
            .any(|r| r.id.eq_ignore_ascii_case(me) && r.vote == 0)
    });
    PullRequestSummary {
        number: p.pull_request_id,
        iid_label: format!("!{}", p.pull_request_id),
        title: p.title,
        state: pull_state(&p.status, p.is_draft),
        source_branch: branch_name(&p.source_ref_name),
        target_branch: branch_name(&p.target_ref_name),
        source_project,
        author: p.created_by.name(),
        web_url,
        // The API has no "updated": closed, else created.
        updated_at: p.closed_date.unwrap_or(p.creation_date),
        review_requested,
        assignees: Vec::new(),
        reviewers: p
            .reviewers
            .iter()
            .filter(|r| !r.is_container)
            .map(Identity::name)
            .collect(),
        checks: None,
        has_conflicts: None,
    }
}

/// `remoteUrl` as Services spells it carries `org@`: drop the userinfo.
pub(super) fn strip_userinfo(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
            match authority.rsplit_once('@') {
                Some((_, host)) => format!("{scheme}://{host}/{path}"),
                None => url.to_string(),
            }
        }
        None => url.to_string(),
    }
}

pub(super) fn map_repo(integration_id: &str, root: &str, r: Repo) -> HostedProject {
    let full_name = format!("{}/{}", r.project.name, r.name);
    let web_url = r.web_url.clone().or_else(|| {
        Some(format!(
            "{}/{}/_git/{}",
            root.trim_end_matches('/'),
            crate::client::encode_component(&r.project.name),
            crate::client::encode_component(&r.name)
        ))
    });
    HostedProject {
        integration_id: integration_id.to_string(),
        id: r.id,
        name: r.name,
        full_name,
        description: None,
        default_branch: r.default_branch.as_deref().map(branch_name),
        ssh_url: r.ssh_url,
        https_url: r.remote_url.as_deref().map(strip_userinfo),
        web_url,
        private: r.project.visibility.as_deref() != Some("public"),
    }
}

/// An identity field: an object (`displayName`) or `"Name <email>"`.
fn identity_name(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => {
            let name = s.split('<').next().unwrap_or(s).trim();
            (!name.is_empty()).then(|| name.to_string())
        }
        Value::Object(o) => o
            .get("displayName")
            .or_else(|| o.get("uniqueName"))
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn text(fields: &serde_json::Map<String, Value>, key: &str) -> String {
    fields
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(super) fn is_closed_state(state: &str) -> bool {
    CLOSED_STATES.iter().any(|s| s.eq_ignore_ascii_case(state))
}

/// One work item of the batch read. `project` is the queried project (the
/// item's own `System.TeamProject` wins when present).
pub(super) fn map_work_item(root: &str, project: &str, item: &Value) -> Option<IssueSummary> {
    let id = item.get("id")?.as_u64()?;
    let empty = serde_json::Map::new();
    let fields = item
        .get("fields")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let team_project = text(fields, "System.TeamProject");
    let project = if team_project.is_empty() {
        project
    } else {
        &team_project
    };
    let state = text(fields, "System.State");
    let spec = crate::spec::spec_for(gitpoulpi_config::ApiFamily::AzureDevops);
    let kind = text(fields, "System.WorkItemType");
    Some(IssueSummary {
        branch_name: None,
        number: id,
        id: id.to_string(),
        reference: format!("#{id}"),
        kind_label: (!kind.is_empty()).then_some(kind),
        status_label: (!state.is_empty()).then(|| state.clone()),
        iid_label: format!("#{id}"),
        title: text(fields, "System.Title"),
        state: if is_closed_state(&state) {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: identity_name(fields.get("System.CreatedBy")).unwrap_or_default(),
        assignees: identity_name(fields.get("System.AssignedTo"))
            .into_iter()
            .collect(),
        labels: text(fields, "System.Tags")
            .split(';')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .collect(),
        web_url: (spec.issue_web_url)(root, project, &id.to_string()).unwrap_or_default(),
        updated_at: text(fields, "System.ChangedDate"),
        milestone: None,
    })
}

/// Work item descriptions are HTML fields.
pub(super) fn html_text(body: &str) -> String {
    body.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace("\r\n", "\n")
        .replace('\n', "<br>")
}
