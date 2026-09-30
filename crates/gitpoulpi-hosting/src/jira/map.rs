//! Jira JSON to the host-agnostic model.

use serde::Deserialize;

use crate::model::{Choice, HostedUser, IssueCreateOptions, IssueState, IssueSummary};
use crate::project::HostedProject;

/// The `fields` the list query asks for (the default is `id` only).
pub(super) const LIST_FIELDS: &str = "summary,status,issuetype,assignee,reporter,labels,updated";

#[derive(Deserialize, Default)]
pub(super) struct JiraUser {
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    // Cloud
    #[serde(rename = "accountId", default)]
    pub account_id: Option<String>,
    #[serde(rename = "emailAddress", default)]
    pub email_address: Option<String>,
    // Data Center
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Deserialize)]
struct Named {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct Status {
    #[serde(default)]
    name: String,
    #[serde(rename = "statusCategory", default)]
    category: Option<Category>,
}

#[derive(Deserialize)]
struct Category {
    #[serde(default)]
    key: String,
}

#[derive(Deserialize, Default)]
struct Fields {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    status: Option<Status>,
    #[serde(default)]
    issuetype: Option<Named>,
    #[serde(default)]
    assignee: Option<JiraUser>,
    #[serde(default)]
    reporter: Option<JiraUser>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    updated: String,
}

#[derive(Deserialize)]
pub(super) struct JiraIssue {
    #[serde(default)]
    id: String,
    key: String,
    #[serde(default)]
    fields: Fields,
}

/// One page of `search/jql` (Cloud) or `search` (Data Center).
#[derive(Deserialize)]
pub(super) struct SearchPage {
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
    #[serde(rename = "nextPageToken", default)]
    pub next_page_token: Option<String>,
    #[serde(rename = "isLast", default)]
    pub is_last: Option<bool>,
    #[serde(default)]
    pub total: Option<u64>,
}

#[derive(Deserialize)]
pub(super) struct JiraProject {
    key: String,
    #[serde(default)]
    name: String,
}

/// Cloud's `project/search` page.
#[derive(Deserialize)]
pub(super) struct ProjectPage {
    #[serde(default)]
    pub values: Vec<JiraProject>,
    #[serde(rename = "isLast", default)]
    pub is_last: Option<bool>,
}

#[derive(Deserialize)]
pub(super) struct IssueTypeRow {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    subtask: bool,
}

/// Cloud answers `issueTypes`, Data Center `values`; take whichever is there.
#[derive(Deserialize)]
pub(super) struct IssueTypePage {
    #[serde(rename = "issueTypes", default)]
    issue_types: Vec<IssueTypeRow>,
    #[serde(default)]
    values: Vec<IssueTypeRow>,
    #[serde(default)]
    pub total: Option<u64>,
}

impl IssueTypePage {
    pub fn rows(self) -> Vec<IssueTypeRow> {
        if self.issue_types.is_empty() {
            self.values
        } else {
            self.issue_types
        }
    }
}

#[derive(Deserialize)]
pub(super) struct CreatedRow {
    pub key: String,
}

/// `ABC-123` to `123`; 0 when the suffix is not a number.
pub(super) fn number_of(key: &str) -> u64 {
    key.rsplit('-')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

pub(super) fn browse_url(site: &str, key: &str) -> String {
    format!("{}/browse/{key}", site.trim_end_matches('/'))
}

pub(super) fn map_issue(site: &str, row: JiraIssue) -> IssueSummary {
    let f = row.fields;
    let done = f
        .status
        .as_ref()
        .and_then(|s| s.category.as_ref())
        .is_some_and(|c| c.key == "done");
    let person = |u: Option<JiraUser>| {
        u.and_then(|u| u.display_name.or(u.name))
            .filter(|n| !n.is_empty())
    };
    IssueSummary {
        branch_name: None,
        number: number_of(&row.key),
        id: row.id,
        web_url: browse_url(site, &row.key),
        iid_label: row.key.clone(),
        reference: row.key,
        kind_label: f.issuetype.map(|t| t.name).filter(|n| !n.is_empty()),
        status_label: f.status.map(|s| s.name).filter(|n| !n.is_empty()),
        title: f.summary,
        state: if done {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: person(f.reporter).unwrap_or_default(),
        assignees: person(f.assignee).into_iter().collect(),
        labels: f.labels,
        updated_at: f.updated,
        milestone: None,
    }
}

pub(super) fn map_project(integration_id: &str, site: &str, row: JiraProject) -> HostedProject {
    HostedProject {
        integration_id: integration_id.to_string(),
        id: row.key.clone(),
        name: if row.name.is_empty() {
            row.key.clone()
        } else {
            row.name
        },
        web_url: Some(browse_url(site, &row.key)),
        full_name: row.key,
        description: None,
        default_branch: None,
        ssh_url: None,
        https_url: None,
        private: false,
    }
}

pub(super) fn map_user(site: &str, cloud: bool, u: JiraUser) -> HostedUser {
    let name = u.display_name.clone().unwrap_or_default();
    if cloud {
        let id = u.account_id.unwrap_or_default();
        HostedUser {
            // The email may be hidden by the account's privacy settings.
            login: u
                .email_address
                .filter(|e| !e.is_empty())
                .unwrap_or_else(|| name.clone()),
            web_url: (!id.is_empty()).then(|| format!("{site}/jira/people/{id}")),
            id,
            name,
        }
    } else {
        let login = u.name.unwrap_or_default();
        HostedUser {
            web_url: (!login.is_empty())
                .then(|| format!("{site}/secure/ViewProfile.jspa?name={login}")),
            id: u.key.unwrap_or_else(|| login.clone()),
            login,
            name,
        }
    }
}

/// The create-form choices: no subtasks, "Task" preferred as the default.
pub(super) fn map_options(rows: Vec<IssueTypeRow>) -> IssueCreateOptions {
    let types: Vec<Choice> = rows
        .into_iter()
        .filter(|t| !t.subtask)
        .map(|t| Choice {
            id: t.id,
            label: t.name,
        })
        .collect();
    let default_type = types
        .iter()
        .find(|t| t.label.eq_ignore_ascii_case("task"))
        .or_else(|| types.first())
        .map(|t| t.id.clone());
    IssueCreateOptions {
        types,
        lists: Vec::new(),
        default_type,
        default_list: None,
    }
}
