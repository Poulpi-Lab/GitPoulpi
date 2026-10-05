//! Slice M for Gitea / Forgejo: members, labels, milestones and the edit
//! calls. A pull request is an issue to Gitea, so assignees, the
//! milestone (an *id*, `0` clears) and labels (`PUT`, by name) of both go
//! through `/issues/{n}`; reviewers have `requested_reviewers`, the same
//! add / remove endpoints GitHub has.

use super::{GiteaClient, Login, UserRow};
use crate::http::HttpRequest;
use crate::metadata::{
    list_delta, HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit,
};
use crate::model::HostedUser;
use crate::Result;

#[derive(serde::Deserialize)]
pub(super) struct LabelRow {
    pub(super) id: i64,
    pub(super) name: String,
    #[serde(default)]
    color: String,
    #[serde(default)]
    description: String,
}

#[derive(serde::Deserialize)]
struct MilestoneRow {
    id: u64,
    title: String,
    #[serde(default)]
    due_on: Option<String>,
    #[serde(default)]
    state: String,
}

impl MilestoneRow {
    fn into_hosted(self) -> HostedMilestone {
        HostedMilestone {
            id: self.id,
            title: self.title,
            // `2026-12-01T12:00:00Z`: the day is what a person reads.
            due_on: self
                .due_on
                .map(|d| d.split('T').next().unwrap_or(&d).to_string()),
            state: self.state,
        }
    }
}

#[derive(serde::Deserialize)]
struct ItemRow {
    assignees: Option<Vec<Login>>,
    requested_reviewers: Option<Vec<Login>>,
    labels: Option<Vec<LabelRow>>,
    milestone: Option<MilestoneRow>,
}

impl GiteaClient {
    pub(super) async fn meta_label_rows(&self, project: &str) -> Result<Vec<LabelRow>> {
        self.pages(
            &self.repo_url(project, "labels"),
            &format!("listing {} labels", self.label()),
        )
        .await
    }

    pub(crate) async fn meta_members(&self, project: &str, query: &str) -> Result<Vec<HostedUser>> {
        let people: Vec<UserRow> = self
            .pages(
                &self.repo_url(project, "assignees"),
                &format!("listing who can be assigned on {}", self.label()),
            )
            .await?;
        let q = query.trim().to_lowercase();
        Ok(people
            .into_iter()
            .filter(|p| q.is_empty() || p.login.to_lowercase().contains(&q))
            .map(UserRow::into_user)
            .collect())
    }

    pub(crate) async fn meta_labels(&self, project: &str) -> Result<Vec<HostedLabel>> {
        Ok(self
            .meta_label_rows(project)
            .await?
            .into_iter()
            .map(|l| HostedLabel {
                name: l.name,
                color: l.color,
                description: l.description,
            })
            .collect())
    }

    pub(crate) async fn meta_milestones(&self, project: &str) -> Result<Vec<HostedMilestone>> {
        let rows: Vec<MilestoneRow> = self
            .pages(
                &self.repo_url(project, "milestones?state=open"),
                &format!("listing {} milestones", self.label()),
            )
            .await?;
        let mut milestones: Vec<HostedMilestone> =
            rows.into_iter().map(MilestoneRow::into_hosted).collect();
        // Soonest due first, those without a date last (the order GitHub's
        // `sort=due_on` gives).
        milestones.sort_by(|a, b| match (&a.due_on, &b.due_on) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        });
        Ok(milestones)
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
        let item: ItemRow = self
            .get_json(
                self.repo_url(project, &format!("{path}/{number}")),
                &format!("reading the metadata of a {} item", self.label()),
            )
            .await?;
        let logins = |list: Option<Vec<Login>>| -> Vec<String> {
            list.unwrap_or_default()
                .into_iter()
                .map(|p| p.login)
                .collect()
        };
        Ok(ItemMeta {
            assignees: logins(item.assignees),
            reviewers: logins(item.requested_reviewers),
            labels: item
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(|l| l.name)
                .collect(),
            milestone: item.milestone.map(MilestoneRow::into_hosted),
        })
    }

    /// `PATCH /issues/{n}` for assignees and the milestone, `PUT
    /// /issues/{n}/labels` for the labels: whichever are set.
    pub(crate) async fn edit_issue_fields(
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
        if let Some(m) = milestone {
            body.insert("milestone".into(), serde_json::json!(m.unwrap_or(0)));
        }
        let url = self.repo_url(project, &format!("issues/{number}"));
        if !body.is_empty() {
            let request = self.authed(HttpRequest::patch_json(
                url.clone(),
                &serde_json::Value::Object(body),
            ));
            let response = self.transport.send(request).await?;
            response.error_for_status("editing an issue")?;
        }
        if let Some(labels) = labels {
            let request = self.authed(HttpRequest::put_json(
                format!("{url}/labels"),
                &serde_json::json!({ "labels": labels }),
            ));
            let response = self.transport.send(request).await?;
            response.error_for_status("setting the labels of an issue")?;
        }
        Ok(())
    }

    /// Closes or reopens an issue (`PATCH /issues/{n}` with `state`).
    pub(crate) async fn meta_set_issue_state(
        &self,
        project: &str,
        number: u64,
        closed: bool,
    ) -> Result<()> {
        let url = self.repo_url(project, &format!("issues/{number}"));
        let body = serde_json::json!({ "state": if closed { "closed" } else { "open" } });
        let request = self.authed(HttpRequest::patch_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("changing the state of an issue")
    }

    pub(crate) async fn meta_update_issue(
        &self,
        project: &str,
        number: u64,
        edit: &IssueEdit,
    ) -> Result<()> {
        self.edit_issue_fields(
            project,
            number,
            edit.assignees.as_deref(),
            edit.labels.as_deref(),
            edit.milestone,
        )
        .await
    }

    /// Asks `add` to review and withdraws the request from `remove`.
    pub(crate) async fn change_reviewers(
        &self,
        project: &str,
        number: u64,
        add: &[String],
        remove: &[String],
    ) -> Result<()> {
        let url = self.repo_url(project, &format!("pulls/{number}/requested_reviewers"));
        if !add.is_empty() {
            let body = serde_json::json!({ "reviewers": add });
            let response = self.transport.send(self.post(url.clone(), &body)).await?;
            response.error_for_status("requesting a review")?;
        }
        if !remove.is_empty() {
            let body = serde_json::json!({ "reviewers": remove });
            let request = self.authed(HttpRequest::delete_json(url, &body));
            let response = self.transport.send(request).await?;
            response.error_for_status("withdrawing a review request")?;
        }
        Ok(())
    }

    pub(crate) async fn meta_update_pull_request(
        &self,
        project: &str,
        number: u64,
        edit: &PrEdit,
    ) -> Result<()> {
        self.edit_issue_fields(
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
