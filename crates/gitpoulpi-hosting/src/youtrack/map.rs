//! YouTrack JSON to the host-agnostic model.

use serde::Deserialize;
use serde_json::Value;

use crate::metadata::HostedLabel;
use crate::model::{HostedUser, IssueState, IssueSummary};
use crate::project::HostedProject;

use super::query::epoch_ms_to_iso;

/// The `fields=` of the list query (the default answer is the `id` only).
pub(super) const ISSUE_FIELDS: &str = "id,idReadable,summary,resolved,updated,\
reporter(login,fullName),tags(name),\
customFields(name,value(name,login,fullName,isResolved))";

/// The fields of the single-issue reads.
pub(super) const META_FIELDS: &str = "id,idReadable,tags(name),\
customFields(name,value(name,login,fullName,isResolved))";

pub(super) const USER_FIELDS: &str = "id,login,fullName,name,email";

pub(super) const PROJECT_FIELDS: &str = "id,shortName,name,description,archived";

#[derive(Deserialize, Default)]
pub(super) struct UserRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub login: String,
    #[serde(rename = "fullName", default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

impl UserRow {
    pub fn display(&self) -> String {
        self.full_name
            .clone()
            .or_else(|| self.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| self.login.clone())
    }

    pub fn into_user(self, site: &str) -> HostedUser {
        HostedUser {
            web_url: (!self.id.is_empty()).then(|| format!("{site}/users/{}", self.id)),
            name: self.display(),
            login: self.login,
            id: self.id,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct NameRow {
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize)]
pub(super) struct FieldRow {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: Value,
}

impl FieldRow {
    /// The value as readable names: a state or version `name`, a user's
    /// `login`; a multi-valued field gives several.
    pub fn names(&self) -> Vec<String> {
        let one = |v: &Value| -> Option<String> {
            v.get("name")
                .or_else(|| v.get("login"))
                .and_then(|n| n.as_str())
                .or_else(|| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        match &self.value {
            Value::Array(rows) => rows.iter().filter_map(one).collect(),
            Value::Null => Vec::new(),
            other => one(other).into_iter().collect(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct IssueRow {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "idReadable", default)]
    pub id_readable: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub resolved: Option<u64>,
    #[serde(default)]
    pub updated: Option<u64>,
    #[serde(default)]
    pub reporter: Option<UserRow>,
    #[serde(default)]
    pub tags: Vec<NameRow>,
    #[serde(rename = "customFields", default)]
    pub custom_fields: Vec<FieldRow>,
}

impl IssueRow {
    pub fn field(&self, name: &str) -> Vec<String> {
        self.custom_fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
            .map(FieldRow::names)
            .unwrap_or_default()
    }
}

/// `ABC-123` to `123`; 0 when the suffix is not a number.
pub(super) fn number_of(reference: &str) -> u64 {
    reference
        .rsplit('-')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

pub(super) fn issue_url(site: &str, reference: &str) -> String {
    format!("{}/issue/{reference}", site.trim_end_matches('/'))
}

pub(super) fn map_issue(site: &str, row: IssueRow) -> IssueSummary {
    let state = row.field("State").into_iter().next();
    let kind = row.field("Type").into_iter().next();
    let assignees = row.field("Assignee");
    let milestone = row.field("Fix versions").into_iter().next();
    IssueSummary {
        number: number_of(&row.id_readable),
        id: row.id,
        web_url: issue_url(site, &row.id_readable),
        iid_label: row.id_readable.clone(),
        reference: row.id_readable,
        kind_label: kind,
        status_label: state,
        title: row.summary,
        state: if row.resolved.is_some() {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        author: row.reporter.map(|u| u.login).unwrap_or_default(),
        assignees,
        labels: row.tags.into_iter().map(|t| t.name).collect(),
        updated_at: epoch_ms_to_iso(row.updated.unwrap_or(0)),
        branch_name: None,
        milestone,
    }
}

#[derive(Deserialize)]
pub(super) struct ProjectRow {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "shortName", default)]
    pub short_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub archived: bool,
}

pub(super) fn map_project(integration_id: &str, site: &str, row: ProjectRow) -> HostedProject {
    HostedProject {
        integration_id: integration_id.to_string(),
        id: row.short_name.clone(),
        name: if row.name.is_empty() {
            row.short_name.clone()
        } else {
            row.name
        },
        web_url: Some(format!("{site}/projects/{}", row.short_name)),
        full_name: row.short_name,
        description: row.description.filter(|d| !d.is_empty()),
        default_branch: None,
        ssh_url: None,
        https_url: None,
        private: false,
    }
}

/// One value of a bundle (a state, a version).
#[derive(Deserialize)]
pub(super) struct BundleValue {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "isResolved", default)]
    pub is_resolved: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub released: bool,
    #[serde(rename = "releaseDate", default)]
    pub release_date: Option<u64>,
}

#[derive(Deserialize)]
pub(super) struct Bundle {
    #[serde(default)]
    pub values: Vec<BundleValue>,
}

#[derive(Deserialize)]
pub(super) struct FieldName {
    #[serde(default)]
    pub name: String,
}

/// A project custom field and, for an enum-like one, its values.
#[derive(Deserialize)]
pub(super) struct ProjectField {
    pub field: FieldName,
    #[serde(default)]
    pub bundle: Option<Bundle>,
}

#[derive(Deserialize)]
pub(super) struct TagRow {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<TagColor>,
}

#[derive(Deserialize)]
pub(super) struct TagColor {
    #[serde(default)]
    pub background: Option<String>,
}

impl TagRow {
    pub fn into_label(self) -> HostedLabel {
        HostedLabel {
            color: self
                .color
                .and_then(|c| c.background)
                .map(|c| c.trim_start_matches('#').to_string())
                .unwrap_or_default(),
            name: self.name,
            description: String::new(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct CreatedRow {
    #[serde(rename = "idReadable", default)]
    pub id_readable: String,
}
