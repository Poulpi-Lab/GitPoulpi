//! The Jira client (Cloud and Data Center).

use std::sync::Arc;

use crate::auth::auth_headers;
use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
    MAX_PAGES,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::model::{HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, TokenCheck};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::jql::issues_jql;
use super::map::{self, LIST_FIELDS};

/// Rows of one search page.
const SEARCH_PAGE: usize = 100;
/// A list shows at most this many issues.
const MAX_ISSUES: usize = 500;

pub(crate) struct JiraClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
}

impl JiraClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
        }
    }

    fn cloud(&self) -> bool {
        self.integration.kind == IntegrationKind::JiraCloud
    }

    fn site(&self) -> String {
        self.integration
            .url
            .trim()
            .trim_end_matches('/')
            .to_string()
    }

    /// Cloud `/rest/api/3`, Data Center `/rest/api/2`.
    fn api(&self) -> String {
        self.integration.api_base()
    }

    /// `/rest/api/2` on both: its `description` is plain text, where v3
    /// wants an Atlassian Document Format tree.
    fn api_v2(&self) -> String {
        self.integration
            .api_base_v2()
            .unwrap_or_else(|| self.integration.api_base())
    }

    fn authed(&self, request: HttpRequest) -> HttpRequest {
        let mut request = request.header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    fn get(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::get(url))
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: String, what: &str) -> Result<T> {
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status(what)?;
        response.json()
    }

    /// Cloud: `search/jql`, continued by `nextPageToken`. (The old
    /// `search` answers 410 there and is never called.)
    async fn search_cloud(&self, jql: &str) -> Result<Vec<map::JiraIssue>> {
        let mut out = Vec::new();
        let mut token: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut url = format!(
                "{}/search/jql?jql={}&fields={LIST_FIELDS}&maxResults={SEARCH_PAGE}",
                self.api(),
                encode_component(jql)
            );
            if let Some(t) = &token {
                url.push_str(&format!("&nextPageToken={}", encode_component(t)));
            }
            let page: map::SearchPage = self.get_json(url, "listing Jira issues").await?;
            let empty = page.issues.is_empty();
            out.extend(page.issues);
            token = page.next_page_token.filter(|t| !t.is_empty());
            if token.is_none() || page.is_last == Some(true) || empty || out.len() >= MAX_ISSUES {
                break;
            }
        }
        Ok(out)
    }

    /// Data Center: `search`, continued by `startAt`.
    async fn search_dc(&self, jql: &str) -> Result<Vec<map::JiraIssue>> {
        let mut out = Vec::new();
        for _ in 0..MAX_PAGES {
            let url = format!(
                "{}/search?jql={}&fields={LIST_FIELDS}&maxResults={SEARCH_PAGE}&startAt={}",
                self.api(),
                encode_component(jql),
                out.len()
            );
            let page: map::SearchPage = self.get_json(url, "listing Jira issues").await?;
            let got = page.issues.len();
            out.extend(page.issues);
            let reached_total = page.total.is_some_and(|t| out.len() as u64 >= t);
            if got == 0 || reached_total || out.len() >= MAX_ISSUES {
                break;
            }
        }
        Ok(out)
    }
}

#[async_trait::async_trait]
impl HostingClient for JiraClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    fn unsupported(&self, what: &str) -> Error {
        Error::Unsupported(what.to_string())
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let site = self.site();
        let id = &self.integration.id;
        let mut out = Vec::new();
        if self.cloud() {
            const PAGE: usize = 100;
            for _ in 0..MAX_PAGES {
                let url = format!(
                    "{}/project/search?startAt={}&maxResults={PAGE}",
                    self.api(),
                    out.len()
                );
                let page: map::ProjectPage = self.get_json(url, "listing Jira projects").await?;
                let got = page.values.len();
                out.extend(
                    page.values
                        .into_iter()
                        .map(|p| map::map_project(id, &site, p)),
                );
                if got == 0 || page.is_last != Some(false) {
                    break;
                }
            }
        } else {
            let rows: Vec<map::JiraProject> = self
                .get_json(format!("{}/project", self.api()), "listing Jira projects")
                .await?;
            out.extend(rows.into_iter().map(|p| map::map_project(id, &site, p)));
        }
        Ok(out)
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
        let user: map::JiraUser = self
            .get_json(format!("{}/myself", self.api()), "checking the Jira token")
            .await?;
        Ok(map::map_user(&self.site(), self.cloud(), user))
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        Ok(TokenCheck {
            user: self.current_user().await?,
            // Jira API tokens carry no scope list.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
        })
    }

    /// The statuses form the columns; a move is a workflow transition.
    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::LocalStatus
    }

    /// The project's statuses, todo-like first (`/project/{key}/statuses`
    /// lists them per issue type; the union, by status category).
    async fn issue_statuses(&self, project: &str) -> Result<Vec<String>> {
        let url = format!(
            "{}/project/{}/statuses",
            self.api(),
            encode_component(project.trim())
        );
        let types: Vec<serde_json::Value> = self.get_json(url, "reading the Jira statuses").await?;
        let mut rows: Vec<(usize, String)> = Vec::new();
        for status in types
            .iter()
            .filter_map(|t| t["statuses"].as_array())
            .flatten()
        {
            let Some(name) = status["name"].as_str().filter(|n| !n.is_empty()) else {
                continue;
            };
            if rows.iter().any(|(_, n)| n.eq_ignore_ascii_case(name)) {
                continue;
            }
            let rank = match status["statusCategory"]["key"].as_str() {
                Some("new") => 0,
                Some("indeterminate") => 1,
                Some("done") => 3,
                _ => 2,
            };
            rows.push((rank, name.to_string()));
        }
        rows.sort_by_key(|(rank, _)| *rank);
        Ok(rows.into_iter().map(|(_, n)| n).collect())
    }

    /// `GET .../transitions`, then `POST` the one whose destination is the
    /// column (by name or status id). No such transition: a clear error,
    /// nothing is sent.
    async fn move_issue(
        &self,
        _project: &str,
        issue: &IssueSummary,
        _spec: &crate::board::BoardSpec,
        to: &crate::board::BoardColumn,
    ) -> Result<()> {
        let from = issue.status_label.as_deref().unwrap_or_default();
        if from.trim().eq_ignore_ascii_case(to.name.trim()) {
            return Ok(());
        }
        let key = encode_component(&issue.reference);
        let url = format!("{}/issue/{key}/transitions", self.api());
        let found: serde_json::Value = self
            .get_json(url.clone(), "reading the Jira transitions")
            .await?;
        let wanted = to.name.trim();
        let id = found["transitions"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|t| {
                t["to"]["name"]
                    .as_str()
                    .is_some_and(|n| n.trim().eq_ignore_ascii_case(wanted))
                    || t["to"]["id"].as_str().is_some_and(|i| i == wanted)
            })
            .and_then(|t| t["id"].as_str())
            .ok_or_else(|| {
                Error::Local(format!(
                    "no transition from {} to {wanted} for {}",
                    if from.is_empty() { "its status" } else { from },
                    issue.reference
                ))
            })?
            .to_string();
        let body = serde_json::json!({ "transition": { "id": id } });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &body)))
            .await?;
        response.error_for_status("moving the Jira issue")?;
        Ok(())
    }

    async fn list_issues(&self, project: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let jql = issues_jql(project, filter);
        let rows = if self.cloud() {
            self.search_cloud(&jql).await?
        } else {
            self.search_dc(&jql).await?
        };
        let site = self.site();
        Ok(rows
            .into_iter()
            .take(MAX_ISSUES)
            .map(|r| map::map_issue(&site, r))
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let issue_type = match req.issue_type.as_deref().filter(|t| !t.is_empty()) {
            Some(t) => t.to_string(),
            None => self
                .issue_create_options(&req.project)
                .await?
                .default_type
                .ok_or_else(|| Error::Api {
                    what: "opening a Jira issue".into(),
                    status: 400,
                    detail: "the project offers no issue type to create".into(),
                })?,
        };
        let mut fields = serde_json::json!({
            "project": { "key": req.project.trim() },
            "summary": req.title,
            "issuetype": { "id": issue_type },
        });
        if !req.body.trim().is_empty() {
            fields["description"] = serde_json::json!(req.body);
        }
        // A Jira label cannot contain a space.
        let labels: Vec<String> = req
            .labels
            .iter()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join("-"))
            .filter(|l| !l.is_empty())
            .collect();
        if !labels.is_empty() {
            fields["labels"] = serde_json::json!(labels);
        }
        let url = format!("{}/issue", self.api_v2());
        let body = serde_json::json!({ "fields": fields });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &body)))
            .await?;
        response.error_for_status("opening a Jira issue")?;
        let created: map::CreatedRow = response.json()?;
        Ok(CreatedIssue {
            number: map::number_of(&created.key),
            web_url: map::browse_url(&self.site(), &created.key),
            reference: created.key,
        })
    }

    async fn issue_create_options(&self, project: &str) -> Result<IssueCreateOptions> {
        let mut rows = Vec::new();
        for _ in 0..MAX_PAGES {
            let url = format!(
                "{}/issue/createmeta/{}/issuetypes?startAt={}&maxResults=200",
                self.api(),
                encode_component(project.trim()),
                rows.len()
            );
            let page: map::IssueTypePage =
                self.get_json(url, "reading the Jira issue types").await?;
            let total = page.total;
            let page_rows = page.rows();
            let got = page_rows.len();
            rows.extend(page_rows);
            if got == 0 || total.map_or(true, |t| rows.len() as u64 >= t) {
                break;
            }
        }
        Ok(map::map_options(rows))
    }
}
