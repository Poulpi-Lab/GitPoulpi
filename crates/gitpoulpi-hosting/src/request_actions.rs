//! Merging and closing a pull / merge request on the host.
//!
//! The one outward-facing write the Radar's row actions make: GitHub, GitLab
//! and Gitea / Forgejo (the three families the Radar lists account-wide).
//! Every other provider answers `Unsupported`; [`request_actions`] says which
//! calls a kind takes, so a caller can gray the button before asking.
//!
//! Neither call is undoable locally: the host owns the result.

use serde::{Deserialize, Serialize};

use crate::integration::IntegrationKind;

/// How a request is merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    /// A merge commit.
    Merge,
    /// All commits squashed into one.
    Squash,
    /// The commits replayed on the target (a fast-forward history).
    Rebase,
}

impl MergeMethod {
    pub const ALL: [MergeMethod; 3] =
        [MergeMethod::Merge, MergeMethod::Squash, MergeMethod::Rebase];
}

/// What the host of a kind lets a request do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RequestActions {
    pub merge: bool,
    /// Rebase and merge (GitLab has no such call: its rebase is a separate,
    /// asynchronous one).
    pub rebase: bool,
    pub close: bool,
    /// Take a draft out of draft (GitHub, GitLab, Gitea, Forgejo).
    pub ready: bool,
}

impl RequestActions {
    pub fn takes(&self, method: MergeMethod) -> bool {
        match method {
            MergeMethod::Merge | MergeMethod::Squash => self.merge,
            MergeMethod::Rebase => self.rebase,
        }
    }
}

/// `title` without its draft marker (`Draft:`, `[Draft]`, `(Draft)`, `WIP:`,
/// `[WIP]`), or `None` when it carries none.
pub fn strip_draft_prefix(title: &str) -> Option<String> {
    let t = title.trim_start();
    let lower = t.to_ascii_lowercase();
    for prefix in ["draft:", "[draft]", "(draft)", "wip:", "[wip]"] {
        if lower.starts_with(prefix) {
            return Some(t[prefix.len()..].trim_start().to_string());
        }
    }
    None
}

/// The calls the client of `kind` implements.
pub fn request_actions(kind: IntegrationKind) -> RequestActions {
    use gitpoulpi_config::ApiFamily;
    match kind.family() {
        ApiFamily::Github | ApiFamily::Gitea => RequestActions {
            merge: true,
            rebase: true,
            close: true,
            ready: true,
        },
        ApiFamily::Gitlab => RequestActions {
            merge: true,
            rebase: false,
            close: true,
            ready: true,
        },
        _ => RequestActions::default(),
    }
}

#[cfg(any(feature = "github", feature = "gitea"))]
fn method_word(method: MergeMethod) -> &'static str {
    match method {
        MergeMethod::Merge => "merge",
        MergeMethod::Squash => "squash",
        MergeMethod::Rebase => "rebase",
    }
}

#[cfg(feature = "github")]
impl crate::github::GitHubClient {
    /// `PUT /pulls/{n}/merge` with the method.
    pub(crate) async fn merge_request_now(
        &self,
        project: &str,
        number: u64,
        method: MergeMethod,
    ) -> crate::Result<()> {
        let url = format!(
            "{}/repos/{project}/pulls/{number}/merge",
            self.integration.api_base()
        );
        let body = serde_json::json!({ "merge_method": method_word(method) });
        let request = self.authed(crate::http::HttpRequest::put_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("merging a GitHub pull request")
    }

    /// `PATCH /pulls/{n}` with `state: closed`.
    pub(crate) async fn close_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = format!(
            "{}/repos/{project}/pulls/{number}",
            self.integration.api_base()
        );
        let body = serde_json::json!({ "state": "closed" });
        let request = self.authed(crate::http::HttpRequest::patch_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("closing a GitHub pull request")
    }

    /// GraphQL `markPullRequestReadyForReview` (REST has no such call).
    pub(crate) async fn ready_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = format!(
            "{}/repos/{project}/pulls/{number}",
            self.integration.api_base()
        );
        let response = self
            .transport
            .send(self.authed(crate::http::HttpRequest::get(url)))
            .await?;
        response.error_for_status("reading a GitHub pull request")?;
        let pull: serde_json::Value = response.json()?;
        let id = pull["node_id"]
            .as_str()
            .ok_or_else(|| crate::Error::Decode("no node_id in the pull request".to_string()))?;
        self.graphql(
            "marking a GitHub pull request ready for review",
            "mutation($id: ID!) { markPullRequestReadyForReview(input: {pullRequestId: $id}) { pullRequest { id } } }",
            serde_json::json!({ "id": id }),
        )
        .await?;
        Ok(())
    }
}

#[cfg(feature = "gitlab")]
impl crate::gitlab::GitLabClient {
    /// `PUT /merge_requests/{iid}/merge` (`squash` for the squash method).
    pub(crate) async fn merge_request_now(
        &self,
        project: &str,
        number: u64,
        method: MergeMethod,
    ) -> crate::Result<()> {
        if method == MergeMethod::Rebase {
            return Err(crate::Error::Unsupported("merge_pull_request".to_string()));
        }
        let url = format!(
            "{}/projects/{}/merge_requests/{number}/merge",
            self.integration.api_base(),
            crate::gitlab::encode_gitlab_project_path(project)
        );
        let body = serde_json::json!({ "squash": method == MergeMethod::Squash });
        let request = self.authed(crate::http::HttpRequest::put_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("merging a GitLab merge request")
    }

    /// `PUT /merge_requests/{iid}` with `state_event: close`.
    pub(crate) async fn close_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = format!(
            "{}/projects/{}/merge_requests/{number}",
            self.integration.api_base(),
            crate::gitlab::encode_gitlab_project_path(project)
        );
        let body = serde_json::json!({ "state_event": "close" });
        let request = self.authed(crate::http::HttpRequest::put_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("closing a GitLab merge request")
    }

    /// Removes the `Draft:` marker from the title (`PUT` with the new title).
    pub(crate) async fn ready_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = format!(
            "{}/projects/{}/merge_requests/{number}",
            self.integration.api_base(),
            crate::gitlab::encode_gitlab_project_path(project)
        );
        let response = self
            .transport
            .send(self.authed(crate::http::HttpRequest::get(url.clone())))
            .await?;
        response.error_for_status("reading a GitLab merge request")?;
        let mr: serde_json::Value = response.json()?;
        let title = mr["title"].as_str().unwrap_or_default();
        let Some(clean) = strip_draft_prefix(title) else {
            return Ok(());
        };
        let body = serde_json::json!({ "title": clean });
        let response = self
            .transport
            .send(self.authed(crate::http::HttpRequest::put_json(url, &body)))
            .await?;
        response.error_for_status("marking a GitLab merge request ready")
    }
}

#[cfg(feature = "gitea")]
impl crate::gitea::GiteaClient {
    /// `POST /pulls/{n}/merge` with `Do`.
    pub(crate) async fn merge_request_now(
        &self,
        project: &str,
        number: u64,
        method: MergeMethod,
    ) -> crate::Result<()> {
        let url = self.repo_url(project, &format!("pulls/{number}/merge"));
        let body = serde_json::json!({ "Do": method_word(method) });
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("merging a pull request")
    }

    /// `PATCH /pulls/{n}` with `state: closed`.
    pub(crate) async fn close_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = self.repo_url(project, &format!("pulls/{number}"));
        let body = serde_json::json!({ "state": "closed" });
        let request = self.authed(crate::http::HttpRequest::patch_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("closing a pull request")
    }

    /// Removes the `WIP:` marker from the title (`PATCH` with the new title).
    pub(crate) async fn ready_request_now(&self, project: &str, number: u64) -> crate::Result<()> {
        let url = self.repo_url(project, &format!("pulls/{number}"));
        let response = self.transport.send(self.get(url.clone())).await?;
        response.error_for_status("reading a pull request")?;
        let pull: serde_json::Value = response.json()?;
        let title = pull["title"].as_str().unwrap_or_default();
        let Some(clean) = strip_draft_prefix(title) else {
            return Ok(());
        };
        let body = serde_json::json!({ "title": clean });
        let request = self.authed(crate::http::HttpRequest::patch_json(url, &body));
        let response = self.transport.send(request).await?;
        response.error_for_status("marking a pull request ready")
    }
}

#[cfg(all(test, any(feature = "github", feature = "gitlab", feature = "gitea")))]
mod tests;
