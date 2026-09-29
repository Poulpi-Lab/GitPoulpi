//! Slice M for GitLab (gitlab.com and self-managed): members, labels,
//! milestones and the edit calls. GitLab addresses people by numeric id
//! (`assignee_ids`, `reviewer_ids`), so a login is looked up in the
//! project's member list first; labels are one comma-separated string and
//! `milestone_id: 0` clears the milestone.

use crate::client::fetch_pages;
use crate::gitlab::{encode_gitlab_project_path, GitLabClient};
use crate::http::HttpRequest;
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::HostedUser;
use crate::{Error, Result};

#[derive(serde::Deserialize)]
struct Person {
    id: i64,
    username: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    web_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct Label {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct Milestone {
    id: u64,
    title: String,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    state: String,
}

impl Milestone {
    fn into_hosted(self) -> HostedMilestone {
        HostedMilestone {
            id: self.id,
            title: self.title,
            due_on: self.due_date.filter(|d| !d.is_empty()),
            // GitLab says `active` where GitHub says `open`.
            state: if self.state == "active" {
                "open".to_string()
            } else {
                self.state
            },
        }
    }
}

#[derive(serde::Deserialize)]
struct Username {
    username: String,
}

#[derive(serde::Deserialize)]
struct Item {
    #[serde(default)]
    assignees: Vec<Username>,
    #[serde(default)]
    reviewers: Vec<Username>,
    #[serde(default)]
    labels: Vec<String>,
    milestone: Option<Milestone>,
}

/// The fields an issue or merge request edit may set, as the JSON body.
struct Fields<'a> {
    assignees: Option<&'a [String]>,
    reviewers: Option<&'a [String]>,
    labels: Option<&'a [String]>,
    milestone: Option<Option<u64>>,
}

impl GitLabClient {
    pub(crate) async fn meta_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let mut url = format!(
            "{}/projects/{}/members/all?",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let query = query.trim();
        if !query.is_empty() {
            url.push_str(&format!(
                "query={}&",
                crate::client::encode_component(query)
            ));
        }
        let people: Vec<Person> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab project members",
        )
        .await?;
        Ok(people
            .into_iter()
            .map(|p| HostedUser {
                login: p.username,
                name: p.name.unwrap_or_default(),
                id: p.id.to_string(),
                web_url: p.web_url,
            })
            .collect())
    }

    pub(crate) async fn meta_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        let url = format!(
            "{}/projects/{}/labels?",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let labels: Vec<Label> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab labels",
        )
        .await?;
        Ok(labels
            .into_iter()
            .map(|l| HostedLabel {
                name: l.name,
                color: l
                    .color
                    .unwrap_or_default()
                    .trim_start_matches('#')
                    .to_string(),
                description: l.description.unwrap_or_default(),
            })
            .collect())
    }

    pub(crate) async fn meta_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        let url = format!(
            "{}/projects/{}/milestones?state=active",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let rows: Vec<Milestone> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitLab milestones",
        )
        .await?;
        Ok(rows.into_iter().map(Milestone::into_hosted).collect())
    }

    pub(crate) async fn meta_item(
        &self,
        project: &str,
        number: u64,
        kind: MetaKind,
    ) -> Result<ItemMeta> {
        let path = match kind {
            MetaKind::Issue => "issues",
            MetaKind::PullRequest => "merge_requests",
        };
        let url = format!(
            "{}/projects/{}/{path}/{number}",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the metadata of a GitLab item")?;
        let item: Item = response.json()?;
        Ok(ItemMeta {
            assignees: item.assignees.into_iter().map(|p| p.username).collect(),
            reviewers: item.reviewers.into_iter().map(|p| p.username).collect(),
            labels: item.labels,
            milestone: item.milestone.map(Milestone::into_hosted),
        })
    }

    /// The ids of `logins`, from the project's member list (one paged
    /// call, whatever the number of logins).
    async fn ids_of(&self, project: &str, logins: &[String]) -> Result<Vec<i64>> {
        if logins.is_empty() {
            return Ok(Vec::new());
        }
        let members = self.meta_members(project, "").await?;
        logins
            .iter()
            .map(|login| {
                members
                    .iter()
                    .find(|m| m.login.eq_ignore_ascii_case(login))
                    .and_then(|m| m.id.parse::<i64>().ok())
                    .ok_or_else(|| {
                        Error::Decode(format!("`{login}` is not a member of this project"))
                    })
            })
            .collect()
    }

    /// The JSON fields for `fields` (people turned into ids).
    async fn meta_body(
        &self,
        project: &str,
        fields: Fields<'_>,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        let mut body = serde_json::Map::new();
        if let Some(logins) = fields.assignees {
            body.insert(
                "assignee_ids".into(),
                serde_json::json!(self.ids_of(project, logins).await?),
            );
        }
        if let Some(logins) = fields.reviewers {
            body.insert(
                "reviewer_ids".into(),
                serde_json::json!(self.ids_of(project, logins).await?),
            );
        }
        if let Some(labels) = fields.labels {
            body.insert("labels".into(), serde_json::json!(labels.join(",")));
        }
        if let Some(milestone) = fields.milestone {
            body.insert(
                "milestone_id".into(),
                serde_json::json!(milestone.unwrap_or(0)),
            );
        }
        Ok(body)
    }

    /// The extra fields a create call carries (see `create_pull_request`).
    pub(crate) async fn new_item_fields(
        &self,
        project: &str,
        assignees: &[String],
        reviewers: &[String],
        labels: &[String],
        milestone: Option<u64>,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        self.meta_body(
            project,
            Fields {
                assignees: (!assignees.is_empty()).then_some(assignees),
                reviewers: (!reviewers.is_empty()).then_some(reviewers),
                labels: (!labels.is_empty()).then_some(labels),
                milestone: milestone.map(Some),
            },
        )
        .await
    }

    async fn put_item(
        &self,
        project: &str,
        path: &str,
        number: u64,
        body: serde_json::Map<String, serde_json::Value>,
        what: &str,
    ) -> Result<()> {
        if body.is_empty() {
            return Ok(());
        }
        let url = format!(
            "{}/projects/{}/{path}/{number}",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let request = self.authed(HttpRequest::put_json(url, &serde_json::Value::Object(body)));
        let response = self.transport.send(request).await?;
        response.error_for_status(what)
    }

    /// Closes or reopens an issue (`PUT /issues/{iid}` with `state_event`).
    pub(crate) async fn meta_set_issue_state(
        &self,
        project: &str,
        number: u64,
        closed: bool,
    ) -> Result<()> {
        let mut body = serde_json::Map::new();
        body.insert(
            "state_event".into(),
            serde_json::json!(if closed { "close" } else { "reopen" }),
        );
        self.put_item(
            project,
            "issues",
            number,
            body,
            "changing the state of a GitLab issue",
        )
        .await
    }

    pub(crate) async fn meta_update_issue(
        &self,
        project: &str,
        number: u64,
        edit: &IssueEdit,
    ) -> Result<()> {
        let body = self
            .meta_body(
                project,
                Fields {
                    assignees: edit.assignees.as_deref(),
                    reviewers: None,
                    labels: edit.labels.as_deref(),
                    milestone: edit.milestone,
                },
            )
            .await?;
        self.put_item(project, "issues", number, body, "editing a GitLab issue")
            .await
    }

    pub(crate) async fn meta_update_pull_request(
        &self,
        project: &str,
        number: u64,
        edit: &PrEdit,
    ) -> Result<()> {
        let body = self
            .meta_body(
                project,
                Fields {
                    assignees: edit.assignees.as_deref(),
                    reviewers: edit.reviewers.as_deref(),
                    labels: edit.labels.as_deref(),
                    milestone: edit.milestone,
                },
            )
            .await?;
        self.put_item(
            project,
            "merge_requests",
            number,
            body,
            "editing a GitLab merge request",
        )
        .await
    }
}

#[cfg(test)]
#[path = "gitlab_meta_tests.rs"]
mod tests;
