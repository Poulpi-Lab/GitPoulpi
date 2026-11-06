//! The Bitbucket Cloud client.

use std::sync::Arc;

use serde::de::DeserializeOwned;

use super::map::{Account, Page, PullRequest, Repo, SshKey, WorkspaceAccess};
use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
    MAX_PAGES,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::model::{
    best_pull_request, excess_scopes, missing_scopes, HostedUser, IssueFilter, IssueSummary,
    PrFilter, PullRequestSummary, StateFilter, TokenCheck, TokenFlavor,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

/// The pull request endpoints cap `pagelen` at 50; repositories and the
/// rest accept 100.
const PR_PAGELEN: u32 = 50;
const PAGELEN: u32 = 100;

/// Why `list_issues` / `create_issue` always fail.
pub const ISSUES_RETIRED: &str =
    "Bitbucket issues were retired on 2026-08-20; link a Jira project instead";

mod protection;

pub struct BitbucketClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
}

impl BitbucketClient {
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

    fn authed(&self, mut request: HttpRequest) -> HttpRequest {
        for (name, value) in crate::auth::auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    fn get(&self, url: String) -> HttpRequest {
        self.authed(HttpRequest::get(url))
    }

    fn api(&self) -> String {
        self.integration.api_base()
    }

    /// `scheme://host[:port]` of the API root: a `next` link may only point
    /// there, so the token is never sent to another host.
    fn origin(&self) -> String {
        let api = self.api();
        match api.split_once("://") {
            Some((scheme, rest)) => format!("{scheme}://{}", rest.split('/').next().unwrap_or("")),
            None => api,
        }
    }

    /// Every page of a list endpoint, following the `next` URLs the host
    /// returns (never constructing its own), capped at [`MAX_PAGES`].
    async fn fetch_all<T: DeserializeOwned>(&self, first: String, what: &str) -> Result<Vec<T>> {
        let origin = self.origin();
        let mut out = Vec::new();
        let mut url = first;
        for _ in 0..MAX_PAGES {
            let response = self.transport.send(self.get(url)).await?;
            response.error_for_status(what)?;
            let page: Page<T> = response.json()?;
            out.extend(page.values);
            match page.next.filter(|n| !n.is_empty()) {
                None => break,
                Some(next) if next.starts_with(&origin) => url = next,
                Some(next) => {
                    return Err(Error::Decode(format!(
                        "{what}: refusing to follow a next link to another host ({next})"
                    )))
                }
            }
        }
        Ok(out)
    }

    async fn fetch_account(&self, what: &str) -> Result<Account> {
        let response = self
            .transport
            .send(self.get(format!("{}/user", self.api())))
            .await?;
        response.error_for_status(what)?;
        response.json()
    }

    async fn workspaces(&self) -> Result<Vec<String>> {
        let url = format!("{}/user/workspaces?pagelen={PAGELEN}", self.api());
        let rows: Vec<WorkspaceAccess> =
            self.fetch_all(url, "listing Bitbucket workspaces").await?;
        Ok(rows.into_iter().map(|w| w.workspace.slug).collect())
    }

    async fn my_uuid(&self) -> Result<String> {
        Ok(self
            .fetch_account("reading the Bitbucket account")
            .await?
            .uuid)
    }

    fn pr_url(&self, project: &str) -> String {
        format!("{}/repositories/{project}/pullrequests", self.api())
    }

    async fn pull_requests(
        &self,
        project: &str,
        states: &[&str],
        q: &[String],
        what: &str,
    ) -> Result<Vec<PullRequest>> {
        let mut url = format!(
            "{}?pagelen={PR_PAGELEN}&sort=-updated_on",
            self.pr_url(project)
        );
        for state in states {
            url.push_str(&format!("&state={state}"));
        }
        if !q.is_empty() {
            url.push_str(&format!("&q={}", encode_component(&q.join(" AND "))));
        }
        self.fetch_all(url, what).await
    }
}

/// A string literal in Bitbucket's filter language.
fn q_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

#[async_trait::async_trait]
impl HostingClient for BitbucketClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let mut out = Vec::new();
        for workspace in self.workspaces().await? {
            let url = format!(
                "{}/repositories/{}?role=member&sort=-updated_on&pagelen={PAGELEN}",
                self.api(),
                encode_component(&workspace)
            );
            match self
                .fetch_all::<Repo>(url, "listing Bitbucket repositories")
                .await
            {
                Ok(rows) => out.extend(
                    rows.into_iter()
                        .map(|r| r.into_project(&self.integration.id)),
                ),
                // A workspace the token cannot read (scope or SSO) must not
                // hide the others.
                Err(Error::Forbidden { .. }) => {}
                Err(Error::Api { status: 404, .. }) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    async fn account_items(
        &self,
        _query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        Err(Error::Unsupported(
            crate::account::NO_BITBUCKET_CLOUD.to_string(),
        ))
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        let uuid = encode_component(&self.my_uuid().await?);
        let url = format!("{}/users/{uuid}/ssh-keys?pagelen={PAGELEN}", self.api());
        let rows: Vec<SshKey> = self.fetch_all(url, "listing Bitbucket SSH keys").await?;
        Ok(rows.into_iter().map(SshKey::into_key).collect())
    }

    async fn upload_ssh_key(&self, title: &str, public_key: &str) -> Result<HostedSshKey> {
        let uuid = encode_component(&self.my_uuid().await?);
        let url = format!("{}/users/{uuid}/ssh-keys", self.api());
        let body = serde_json::json!({ "label": title, "key": public_key.trim() });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &body)))
            .await?;
        response.error_for_status("uploading an SSH key to Bitbucket")?;
        Ok(response.json::<SshKey>()?.into_key())
    }

    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        let uuid = encode_component(&self.my_uuid().await?);
        let url = format!(
            "{}/users/{uuid}/ssh-keys/{}",
            self.api(),
            encode_component(id)
        );
        let response = self
            .transport
            .send(self.authed(HttpRequest::delete(url)))
            .await?;
        response.error_for_status("deleting a Bitbucket SSH key")
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        let mut source = serde_json::json!({ "branch": { "name": req.source_branch } });
        if let Some(fork) = req.fork_source() {
            source["repository"] = serde_json::json!({ "full_name": fork });
        }
        let body = serde_json::json!({
            "title": req.title,
            "description": req.body,
            "source": source,
            "destination": { "branch": { "name": req.target_branch } },
            "draft": req.draft,
        });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(self.pr_url(&req.project), &body)))
            .await?;
        response.error_for_status("opening a Bitbucket pull request")?;
        let created: PullRequest = response.json()?;
        let number = created.id;
        Ok(CreatedPullRequest {
            number,
            web_url: created.links.html.href.unwrap_or_default(),
            ..Default::default()
        })
    }

    async fn protected_branches(
        &self,
        project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        self.protection(project).await
    }

    async fn current_user(&self) -> Result<HostedUser> {
        Ok(self
            .fetch_account("checking the Bitbucket token")
            .await?
            .into_user())
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        // Bitbucket sends the token's scopes in `x-oauth-scopes` when it
        // sends them at all (an API token does not): absent is unknown,
        // never "missing".
        let response = self
            .transport
            .send(self.get(format!("{}/user", self.api())))
            .await?;
        response.error_for_status("checking the Bitbucket token")?;
        let scopes = response
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-oauth-scopes"))
            .map(|(_, v)| {
                v.split([',', ' '])
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .filter(|s| !s.is_empty());
        let required = self
            .integration
            .kind
            .token_help()
            .map(|h| h.scopes)
            .unwrap_or(&[]);
        let (missing, excess) = scopes.as_deref().map_or_else(Default::default, |granted| {
            (
                missing_scopes(required, granted),
                excess_scopes(required, &[], granted),
            )
        });
        let account: Account = response.json()?;
        Ok(TokenCheck {
            user: account.into_user(),
            token_flavor: Some(if scopes.is_some() {
                TokenFlavor::Classic
            } else {
                TokenFlavor::Unknown
            }),
            scopes,
            missing_scopes: missing,
            excess_scopes: excess,
            ..Default::default()
        })
    }

    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        let states: &[&str] = match filter.state {
            StateFilter::Open => &["OPEN"],
            StateFilter::Closed => &["MERGED", "DECLINED", "SUPERSEDED"],
            StateFilter::All => &["OPEN", "MERGED", "DECLINED", "SUPERSEDED"],
        };
        let me = if filter.mine || filter.review_requested {
            Some(self.my_uuid().await?)
        } else {
            None
        };
        let mut q = Vec::new();
        if let (true, Some(me)) = (filter.mine, &me) {
            q.push(format!("author.uuid={}", q_string(me)));
        }
        let rows = self
            .pull_requests(project, states, &q, "listing Bitbucket pull requests")
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| r.into_summary(project, me.as_deref()))
            .filter(|s| !filter.review_requested || s.review_requested)
            .collect())
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let q = [format!("source.branch.name={}", q_string(branch))];
        let rows = self
            .pull_requests(
                project,
                &["OPEN", "MERGED", "DECLINED", "SUPERSEDED"],
                &q,
                "looking up the pull request for a branch",
            )
            .await?;
        Ok(best_pull_request(
            rows.into_iter()
                .map(|r| r.into_summary(project, None))
                .collect(),
        ))
    }

    async fn list_issues(
        &self,
        _project: &str,
        _filter: &IssueFilter,
    ) -> Result<Vec<IssueSummary>> {
        Err(Error::Unsupported(ISSUES_RETIRED.to_string()))
    }

    async fn create_issue(&self, _req: &NewIssue) -> Result<CreatedIssue> {
        Err(Error::Unsupported(ISSUES_RETIRED.to_string()))
    }
}
