//! Redmine JSON to the host-agnostic model.

use serde::Deserialize;

use crate::model::{HostedUser, IssueState, IssueSummary};
use crate::project::HostedProject;

/// `{ "id": 1, "name": "Bug" }`, the shape of every reference in an issue.
#[derive(Deserialize, Default, Clone)]
pub(super) struct Named {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize, Default, Clone)]
pub(super) struct StatusRef {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_closed: bool,
}

#[derive(Deserialize)]
pub(super) struct IssueRow {
    pub id: u64,
    #[serde(default)]
    pub tracker: Option<Named>,
    #[serde(default)]
    pub status: Option<StatusRef>,
    #[serde(default)]
    pub author: Option<Named>,
    #[serde(default)]
    pub assigned_to: Option<Named>,
    #[serde(default)]
    pub fixed_version: Option<Named>,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub updated_on: Option<String>,
    #[serde(default)]
    pub closed_on: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct IssuesPage {
    #[serde(default)]
    pub issues: Vec<IssueRow>,
    #[serde(default)]
    pub total_count: usize,
}

#[derive(Deserialize)]
pub(super) struct OneIssue {
    pub issue: IssueRow,
}

pub(super) fn issue_url(site: &str, number: u64) -> String {
    format!("{}/issues/{number}", site.trim_end_matches('/'))
}

pub(super) fn map_issue(site: &str, row: IssueRow) -> IssueSummary {
    // The status decides: `closed_on` stays set after a reopen (seen live).
    let closed = match &row.status {
        Some(status) => status.is_closed,
        None => row.closed_on.is_some(),
    };
    IssueSummary {
        number: row.id,
        id: row.id.to_string(),
        reference: format!("#{}", row.id),
        iid_label: format!("#{}", row.id),
        web_url: issue_url(site, row.id),
        kind_label: row.tracker.map(|t| t.name).filter(|n| !n.is_empty()),
        status_label: row.status.map(|s| s.name).filter(|n| !n.is_empty()),
        title: row.subject,
        state: if closed {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: row.author.map(|a| a.name).unwrap_or_default(),
        assignees: row
            .assigned_to
            .map(|a| a.name)
            .filter(|n| !n.is_empty())
            .into_iter()
            .collect(),
        // Core Redmine has no labels.
        labels: Vec::new(),
        updated_at: row.updated_on.unwrap_or_default(),
        branch_name: None,
        milestone: row.fixed_version.map(|v| v.name).filter(|n| !n.is_empty()),
    }
}

#[derive(Deserialize)]
pub(super) struct UserRow {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub firstname: String,
    #[serde(default)]
    pub lastname: String,
}

#[derive(Deserialize)]
pub(super) struct CurrentUser {
    pub user: UserRow,
}

impl UserRow {
    pub fn into_user(self, site: &str) -> HostedUser {
        let name = format!("{} {}", self.firstname, self.lastname)
            .trim()
            .to_string();
        HostedUser {
            web_url: (self.id != 0)
                .then(|| format!("{}/users/{}", site.trim_end_matches('/'), self.id)),
            login: self.login,
            name,
            id: self.id.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ProjectRow {
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Deserialize)]
pub(super) struct ProjectsPage {
    #[serde(default)]
    pub projects: Vec<ProjectRow>,
    #[serde(default)]
    pub total_count: usize,
}

pub(super) fn map_project(integration_id: &str, site: &str, row: ProjectRow) -> HostedProject {
    HostedProject {
        integration_id: integration_id.to_string(),
        id: row.identifier.clone(),
        name: if row.name.is_empty() {
            row.identifier.clone()
        } else {
            row.name
        },
        web_url: Some(format!(
            "{}/projects/{}",
            site.trim_end_matches('/'),
            row.identifier
        )),
        full_name: row.identifier,
        description: row.description.filter(|d| !d.is_empty()),
        default_branch: None,
        ssh_url: None,
        https_url: None,
        private: !row.is_public,
    }
}

#[derive(Deserialize)]
pub(super) struct StatusRow {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_closed: bool,
}

#[derive(Deserialize)]
pub(super) struct StatusesPage {
    #[serde(default)]
    pub issue_statuses: Vec<StatusRow>,
}

#[derive(Deserialize)]
pub(super) struct TrackersPage {
    #[serde(default)]
    pub trackers: Vec<Named>,
}

#[derive(Deserialize)]
pub(super) struct VersionRow {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct VersionsPage {
    #[serde(default)]
    pub versions: Vec<VersionRow>,
}

/// One member of a project: a user or a group (both can be assigned).
#[derive(Deserialize)]
pub(super) struct Membership {
    #[serde(default)]
    pub user: Option<Named>,
    #[serde(default)]
    pub group: Option<Named>,
}

#[derive(Deserialize)]
pub(super) struct MembershipsPage {
    #[serde(default)]
    pub memberships: Vec<Membership>,
    #[serde(default)]
    pub total_count: usize,
}

#[derive(Deserialize)]
pub(super) struct ProjectDetail {
    pub project: ProjectWithTrackers,
}

#[derive(Deserialize)]
pub(super) struct ProjectWithTrackers {
    #[serde(default)]
    pub trackers: Vec<Named>,
}

#[derive(Deserialize)]
pub(super) struct Created {
    pub issue: CreatedIssueRow,
}

#[derive(Deserialize)]
pub(super) struct CreatedIssueRow {
    pub id: u64,
}
