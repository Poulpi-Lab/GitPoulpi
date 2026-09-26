//! Slice M for GitHub (and Enterprise Server): members, labels,
//! milestones and the edit calls. Pull requests are issues to GitHub, so
//! assignees, labels and the milestone of both go through
//! `PATCH /repos/{p}/issues/{n}`; reviewers have their own
//! `requested_reviewers` add / remove endpoints.

use crate::client::fetch_pages;
use crate::github::GitHubClient;
use crate::http::HttpRequest;
use crate::metadata::{
    list_delta, HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit,
};
use crate::model::HostedUser;
use crate::Result;

#[derive(serde::Deserialize)]
struct Person {
    login: String,
    id: i64,
    html_url: Option<String>,
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
    number: u64,
    title: String,
    #[serde(default)]
    due_on: Option<String>,
    #[serde(default)]
    state: String,
}

impl Milestone {
    fn into_hosted(self) -> HostedMilestone {
        HostedMilestone {
            id: self.number,
            title: self.title,
            // `2026-11-01T00:00:00Z`: the day is what a person reads.
            due_on: self
                .due_on
                .map(|d| d.split('T').next().unwrap_or(&d).to_string()),
            state: self.state,
        }
    }
}

#[derive(serde::Deserialize)]
struct Item {
    #[serde(default)]
    assignees: Vec<Person>,
    #[serde(default)]
    requested_reviewers: Vec<Person>,
    #[serde(default)]
    labels: Vec<Label>,
    milestone: Option<Milestone>,
}

impl GitHubClient {
    pub(crate) async fn meta_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let url = format!("{}/repos/{project}/assignees?", self.integration.api_base());
        let people: Vec<Person> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing who can be assigned on GitHub",
        )
        .await?;
        let q = query.trim().to_lowercase();
        Ok(people
            .into_iter()
            .filter(|p| q.is_empty() || p.login.to_lowercase().contains(&q))
            .map(|p| HostedUser {
                login: p.login,
                name: String::new(),
                id: p.id.to_string(),
                web_url: p.html_url,
            })
            .collect())
    }

    pub(crate) async fn meta_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        let url = format!("{}/repos/{project}/labels?", self.integration.api_base());
        let labels: Vec<Label> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub labels",
        )
        .await?;
        Ok(labels
            .into_iter()
            .map(|l| HostedLabel {
                name: l.name,
                color: l.color.unwrap_or_default(),
                description: l.description.unwrap_or_default(),
            })
            .collect())
    }

    pub(crate) async fn meta_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        let url = format!(
            "{}/repos/{project}/milestones?state=open&sort=due_on&direction=asc",
            self.integration.api_base()
        );
        let rows: Vec<Milestone> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "listing GitHub milestones",
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
            MetaKind::PullRequest => "pulls",
        };
        let url = format!(
            "{}/repos/{project}/{path}/{number}",
            self.integration.api_base()
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the metadata of a GitHub item")?;
        let item: Item = response.json()?;
        Ok(ItemMeta {
            assignees: item.assignees.into_iter().map(|p| p.login).collect(),
            reviewers: item
                .requested_reviewers
                .into_iter()
                .map(|p| p.login)
                .collect(),
            labels: item.labels.into_iter().map(|l| l.name).collect(),
            milestone: item.milestone.map(Milestone::into_hosted),
        })
    }

    /// `PATCH /issues/{n}` with whichever of the three fields are set.
    pub(crate) async fn patch_issue_fields(
        &self,
        project: &str,
        number: u64,
        assignees: Option<&[String]>,
        labels: Option<&[String]>,
        milestone: Option<Option<u64>>,
    ) -> Result<()> {
        let mut body = serde_json::Map::new();
        if let Some(a) = assignees {
            body.insert("assignees".into(), serde_json::json!(a));
        }
        if let Some(l) = labels {
            body.insert("labels".into(), serde_json::json!(l));
        }
        if let Some(m) = milestone {
            body.insert("milestone".into(), serde_json::json!(m));
        }
        if body.is_empty() {
            return Ok(());
        }
        let url = format!(
            "{}/repos/{project}/issues/{number}",
            self.integration.api_base()
        );
        let request = self.authed(HttpRequest::patch_json(
            url,
            &serde_json::Value::Object(body),
        ));
        let response = self.transport.send(request).await?;
        response.error_for_status("editing a GitHub issue")
    }

    /// Closes or reopens an issue (`PATCH /issues/{n}` with `state`).
    pub(crate) async fn meta_set_issue_state(
        &self,
        project: &str,
        number: u64,
        closed: bool,
    ) -> Result<()> {
        let url = format!(
            "{}/repos/{project}/issues/{number}",
            self.integration.api_base()
        );
        let body = serde_json::json!({ "state": if closed { "closed" } else { "open" } });
        let response = self
            .transport
            .send(self.authed(HttpRequest::patch_json(url, &body)))
            .await?;
        response.error_for_status("changing the state of a GitHub issue")
    }

    pub(crate) async fn meta_update_issue(
        &self,
        project: &str,
        number: u64,
        edit: &IssueEdit,
    ) -> Result<()> {
        self.patch_issue_fields(
            project,
            number,
            edit.assignees.as_deref(),
            edit.labels.as_deref(),
            edit.milestone,
        )
        .await
    }

    /// Asks `reviewers` to review and withdraws the request from `remove`.
    pub(crate) async fn change_reviewers(
        &self,
        project: &str,
        number: u64,
        add: &[String],
        remove: &[String],
    ) -> Result<()> {
        let url = format!(
            "{}/repos/{project}/pulls/{number}/requested_reviewers",
            self.integration.api_base()
        );
        if !add.is_empty() {
            let body = serde_json::json!({ "reviewers": add });
            let response = self.transport.send(self.post(url.clone(), &body)).await?;
            response.error_for_status("requesting a review on GitHub")?;
        }
        if !remove.is_empty() {
            let body = serde_json::json!({ "reviewers": remove });
            let request = self.authed(HttpRequest::delete_json(url, &body));
            let response = self.transport.send(request).await?;
            response.error_for_status("withdrawing a review request on GitHub")?;
        }
        Ok(())
    }

    pub(crate) async fn meta_update_pull_request(
        &self,
        project: &str,
        number: u64,
        edit: &PrEdit,
    ) -> Result<()> {
        self.patch_issue_fields(
            project,
            number,
            edit.assignees.as_deref(),
            edit.labels.as_deref(),
            edit.milestone,
        )
        .await?;
        if let Some(wanted) = &edit.reviewers {
            let have = self
                .meta_item(project, number, MetaKind::PullRequest)
                .await?
                .reviewers;
            let (add, remove) = list_delta(&have, wanted);
            self.change_reviewers(project, number, &add, &remove)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "github_meta_tests.rs"]
mod tests;
