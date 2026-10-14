//! Linear JSON to the host-agnostic model.

use serde::Deserialize;

use crate::metadata::{HostedLabel, HostedMilestone};
use crate::model::{HostedUser, IssueState, IssueSummary};
use crate::project::HostedProject;

#[derive(Deserialize)]
pub(super) struct PageInfo {
    #[serde(rename = "hasNextPage", default)]
    pub has_next_page: bool,
    #[serde(rename = "endCursor", default)]
    pub end_cursor: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Connection<T> {
    #[serde(default = "Vec::new")]
    pub nodes: Vec<T>,
    #[serde(rename = "pageInfo", default)]
    pub page_info: Option<PageInfo>,
}

#[derive(Deserialize, Default)]
pub(super) struct UserRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "yes")]
    pub active: bool,
}

fn yes() -> bool {
    true
}

impl UserRow {
    /// The handle used as a login: Linear's `displayName` is unique in a
    /// workspace; the full name stands in when it is empty.
    pub fn handle(&self) -> &str {
        if self.display_name.is_empty() {
            &self.name
        } else {
            &self.display_name
        }
    }

    pub fn into_user(self) -> HostedUser {
        HostedUser {
            login: self.handle().to_string(),
            name: self.name,
            id: self.id,
            web_url: self.url,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct TeamRow {
    pub id: String,
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Organization {
    #[serde(rename = "urlKey", default)]
    pub url_key: String,
}

#[derive(Deserialize)]
pub(super) struct StateRow {
    pub id: String,
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub position: f64,
}

impl StateRow {
    /// Board order: triage, backlog, unstarted, started, completed, canceled,
    /// then Linear's own `position` inside a type.
    fn rank(&self) -> u8 {
        match self.kind.as_str() {
            "triage" => 0,
            "backlog" => 1,
            "unstarted" => 2,
            "started" => 3,
            "completed" => 4,
            "canceled" => 5,
            _ => 6,
        }
    }
}

/// The states in board order.
pub(super) fn sort_states(states: &mut [StateRow]) {
    states.sort_by(|a, b| {
        a.rank()
            .cmp(&b.rank())
            .then(a.position.total_cmp(&b.position))
    });
}

#[derive(Deserialize)]
pub(super) struct StateRef {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: String,
}

#[derive(Deserialize)]
pub(super) struct IssueRow {
    #[serde(default)]
    pub id: String,
    pub identifier: String,
    #[serde(default)]
    pub number: f64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "branchName", default)]
    pub branch_name: Option<String>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
    #[serde(rename = "priorityLabel", default)]
    pub priority_label: Option<String>,
    #[serde(default)]
    pub state: Option<StateRef>,
    #[serde(default)]
    pub creator: Option<UserRow>,
    #[serde(default)]
    pub assignee: Option<UserRow>,
    #[serde(default)]
    pub labels: Option<Connection<NameRow>>,
}

#[derive(Deserialize)]
pub(super) struct NameRow {
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize)]
pub(super) struct LabelRow {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl LabelRow {
    pub fn into_label(self) -> HostedLabel {
        HostedLabel {
            name: self.name,
            color: self.color.trim_start_matches('#').to_string(),
            description: self.description.unwrap_or_default(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct CycleRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub number: f64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "endsAt", default)]
    pub ends_at: Option<String>,
    #[serde(rename = "isActive", default)]
    pub is_active: bool,
}

impl CycleRow {
    pub fn into_milestone(self) -> HostedMilestone {
        let number = self.number as u64;
        HostedMilestone {
            id: number,
            title: self
                .name
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| format!("Cycle {number}")),
            due_on: self
                .ends_at
                .map(|d| d.split('T').next().unwrap_or(&d).to_string()),
            state: if self.is_active { "active" } else { "upcoming" }.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ItemRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub assignee: Option<UserRow>,
    #[serde(default)]
    pub labels: Option<Connection<NameRow>>,
    #[serde(default)]
    pub cycle: Option<CycleRow>,
}

#[derive(Deserialize)]
pub(super) struct CreatedIssue {
    pub identifier: String,
    #[serde(default)]
    pub number: f64,
    #[serde(default)]
    pub url: String,
}

/// Linear workflow state types that mean "not open any more".
pub(super) const CLOSED_TYPES: [&str; 2] = ["completed", "canceled"];

pub(super) fn map_issue(row: IssueRow) -> IssueSummary {
    let state = row.state.unwrap_or(StateRef {
        name: String::new(),
        kind: String::new(),
    });
    let closed = CLOSED_TYPES.contains(&state.kind.as_str());
    let person = |u: Option<UserRow>| u.map(|u| u.handle().to_string()).filter(|n| !n.is_empty());
    IssueSummary {
        number: row.number as u64,
        id: row.id,
        iid_label: row.identifier.clone(),
        reference: row.identifier,
        // "No priority" says nothing.
        kind_label: row
            .priority_label
            .filter(|p| !p.is_empty() && !p.eq_ignore_ascii_case("no priority")),
        status_label: Some(state.name).filter(|n| !n.is_empty()),
        title: row.title,
        state: if closed {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: person(row.creator).unwrap_or_default(),
        assignees: person(row.assignee).into_iter().collect(),
        labels: row
            .labels
            .map(|c| c.nodes.into_iter().map(|l| l.name).collect())
            .unwrap_or_default(),
        web_url: row.url,
        updated_at: row.updated_at,
        branch_name: row.branch_name.filter(|b| !b.is_empty()),
        milestone: None,
    }
}

/// `https://linear.app/acme` for the profile URL `https://linear.app` (or
/// already `https://linear.app/acme`) and the organization's `urlKey`.
pub(super) fn workspace_root(profile_url: &str, url_key: &str) -> String {
    let url = profile_url.trim().trim_end_matches('/');
    let (scheme, rest) = url.split_once("://").unwrap_or(("https", url));
    let host = rest.split('/').next().unwrap_or(rest);
    if url_key.is_empty() {
        format!("{scheme}://{host}")
    } else {
        format!("{scheme}://{host}/{url_key}")
    }
}

pub(super) fn map_team(integration_id: &str, root: &str, team: TeamRow) -> HostedProject {
    HostedProject {
        integration_id: integration_id.to_string(),
        id: team.key.clone(),
        name: if team.name.is_empty() {
            team.key.clone()
        } else {
            team.name
        },
        web_url: Some(format!("{root}/team/{}/all", team.key)),
        full_name: team.key,
        description: team.description.filter(|d| !d.is_empty()),
        default_branch: None,
        ssh_url: None,
        https_url: None,
        private: false,
    }
}
