//! The Bitbucket Data Center client (slice S12).
//!
//! Verified 2026-10-08 against Atlassian's published OpenAPI document for
//! Data Center 10.0 (`https://dac-static.atlassian.com/server/bitbucket/10.0.swagger.v3.json`,
//! the machine-readable form of the REST reference at
//! `https://developer.atlassian.com/server/bitbucket/rest/v1000/`):
//!
//! - `GET /rest/api/latest/projects/{key}/repos/{slug}/pull-requests` with
//!   `state` (`OPEN|DECLINED|MERGED|ALL`), `at` (a fully-qualified ref),
//!   `direction` (`INCOMING|OUTGOING`, relative to the repository), `order`,
//!   `draft`, `start`, `limit` (default 25).
//! - Paging: `values`, `isLastPage`, `nextPageStart`, `start`, `limit`, `size`.
//! - Pull request: `id`, `title`, `state` (`DECLINED|MERGED|OPEN`), `draft`,
//!   `updatedDate` (epoch milliseconds), `fromRef`/`toRef` (`id` =
//!   `refs/heads/x`, `displayId`, `repository.slug`, `repository.project.key`),
//!   `author` and `reviewers` (participants: `user.{name,slug,displayName}`,
//!   `role` `AUTHOR|REVIEWER|PARTICIPANT`, `status`
//!   `UNAPPROVED|NEEDS_WORK|APPROVED`).
//! - Create: `POST` the same url with `{title, description, draft, fromRef,
//!   toRef}` (refs carry `id` and `repository.{slug, project.key}`); 201 with
//!   the new pull request.
//! - `GET /rest/api/latest/repos` (all repositories the caller can read;
//!   paged), repository `slug`, `name`, `description`, `public`,
//!   `project.key`.
//! - `GET /rest/api/latest/users/{userSlug}`: `name`, `slug`, `displayName`.
//! - SSH keys of the current user live in another REST root:
//!   `/rest/ssh/latest/keys` (GET paged, POST `{text, label}`, DELETE
//!   `/keys/{id}`); the key has `id` (integer), `label`, `text`.
//!   Without a `user` parameter they act on the authenticated user.
//! - The pull request head ref `refs/pull-requests/{id}/from` is documented
//!   by Atlassian's article "Pull request proficiency: fetching abilities
//!   unlocked" (it calls these refs private and undocumented in the API).
//!
//! NOT verifiable in the OpenAPI document, taken from community answers and
//! the plan (so covered only by canned tests):
//!
//! - HTTP access tokens as `Authorization: Bearer` (`crate::auth`).
//! - The current user's name: the `X-AUSERNAME` response header (here read
//!   from `GET /rest/api/latest/application-properties`).
//! - The `links.self[0].href` web url of a pull request and the
//!   `links.clone[]` (`{href, name: "http"|"ssh"}`) of a repository: the
//!   schema declares `links` as an opaque object.
//!
//! Left out because nothing verifiable backs them: web url builders for new
//! pull requests, branches and commits (`spec`), and parsing the
//! `/scm/KEY/slug.git` clone path beyond stripping it here.
//!
//! There is no issue tracker: Data Center installs use Jira.

use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
    MAX_PAGES,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::model::{
    best_pull_request, HostedUser, IssueFilter, IssueSummary, PrFilter, PullRequestState,
    PullRequestSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

const LIMIT: u32 = 100;

/// Why `list_issues` / `create_issue` always fail.
pub const NO_ISSUES: &str = "Bitbucket Data Center has no issue tracker; link a Jira project";

mod account;
mod protection;

pub struct BitbucketDcClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
}

#[derive(serde::Deserialize)]
struct Page<T> {
    #[serde(default = "Vec::new")]
    values: Vec<T>,
    #[serde(default, rename = "isLastPage")]
    is_last_page: bool,
    #[serde(default, rename = "nextPageStart")]
    next_page_start: Option<u64>,
}

#[derive(serde::Deserialize, Default)]
struct Link {
    #[serde(default)]
    href: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct Links {
    #[serde(default, rename = "self")]
    self_: Vec<Link>,
    #[serde(default)]
    clone: Vec<Link>,
}

#[derive(serde::Deserialize, Default)]
struct User {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    links: Links,
}

impl User {
    fn shown(&self) -> String {
        self.display_name
            .clone()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| self.name.clone())
    }
}

#[derive(serde::Deserialize, Default)]
struct ProjectRef {
    #[serde(default)]
    key: String,
}

#[derive(serde::Deserialize, Default)]
struct Repo {
    #[serde(default)]
    slug: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    public: bool,
    #[serde(default)]
    project: ProjectRef,
    #[serde(default)]
    links: Links,
}

impl Repo {
    fn full_name(&self) -> String {
        format!("{}/{}", self.project.key, self.slug)
    }

    fn into_project(self, integration_id: &str) -> HostedProject {
        let clone = |kinds: &[&str]| {
            self.links
                .clone
                .iter()
                .find(|l| l.name.as_deref().is_some_and(|n| kinds.contains(&n)))
                .and_then(|l| l.href.clone())
        };
        HostedProject {
            integration_id: integration_id.to_string(),
            id: self.full_name(),
            name: if self.name.is_empty() {
                self.slug.clone()
            } else {
                self.name.clone()
            },
            full_name: self.full_name(),
            description: self.description.clone().filter(|d| !d.is_empty()),
            // Not part of the repository payload (`defaultBranch` is write
            // only in the reference): unknown here.
            default_branch: None,
            ssh_url: clone(&["ssh"]),
            https_url: clone(&["http", "https"]).map(|u| without_user(&u)),
            web_url: self.links.self_.first().and_then(|l| l.href.clone()),
            private: !self.public,
        }
    }
}

/// `https://user@host/scm/K/r.git` -> without the `user@` the API embeds.
fn without_user(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => match rest.split_once('@') {
            Some((user, tail)) if !user.contains('/') => format!("{scheme}://{tail}"),
            _ => url.to_string(),
        },
        None => url.to_string(),
    }
}

#[derive(serde::Deserialize, Default)]
struct RepoRef {
    #[serde(default)]
    slug: String,
    #[serde(default)]
    project: ProjectRef,
    #[serde(default)]
    links: Links,
}

#[derive(serde::Deserialize)]
struct Ref {
    #[serde(default, rename = "displayId")]
    display_id: String,
    #[serde(default)]
    repository: RepoRef,
}

#[derive(serde::Deserialize, Default)]
struct Participant {
    #[serde(default)]
    user: User,
    #[serde(default)]
    status: String,
    #[serde(default)]
    approved: bool,
}

#[derive(serde::Deserialize)]
struct PullRequest {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    draft: bool,
    #[serde(rename = "fromRef")]
    from_ref: Ref,
    #[serde(rename = "toRef")]
    to_ref: Ref,
    #[serde(default)]
    author: Participant,
    #[serde(default)]
    reviewers: Vec<Participant>,
    #[serde(default, rename = "updatedDate")]
    updated_date: u64,
    #[serde(default)]
    links: Links,
}

impl PullRequest {
    fn web_url(&self) -> String {
        self.links
            .self_
            .first()
            .and_then(|l| l.href.clone())
            .unwrap_or_default()
    }

    fn is_open(&self) -> bool {
        self.state == "OPEN"
    }

    fn is_mine(&self, me: &str) -> bool {
        self.author.user.name.eq_ignore_ascii_case(me)
    }

    fn review_requested(&self, me: &str) -> bool {
        self.is_open()
            && self.reviewers.iter().any(|r| {
                r.user.name.eq_ignore_ascii_case(me) && !r.approved && r.status != "APPROVED"
            })
    }

    fn into_summary(self, project: &str, me: Option<&str>) -> PullRequestSummary {
        let state = match self.state.as_str() {
            "MERGED" => PullRequestState::Merged,
            "DECLINED" => PullRequestState::Closed,
            _ if self.draft => PullRequestState::Draft,
            _ => PullRequestState::Open,
        };
        let fork = format!(
            "{}/{}",
            self.from_ref.repository.project.key, self.from_ref.repository.slug
        );
        let source_project = (!self.from_ref.repository.slug.is_empty()
            && !fork.eq_ignore_ascii_case(project))
        .then_some(fork);
        let review_requested = me.is_some_and(|me| self.review_requested(me));
        let number = self.id;
        let web_url = self.web_url();
        PullRequestSummary {
            number,
            iid_label: format!("#{number}"),
            title: self.title,
            state,
            source_branch: self.from_ref.display_id,
            target_branch: self.to_ref.display_id,
            source_project,
            author: self.author.user.shown(),
            web_url,
            updated_at: epoch_ms_to_iso(self.updated_date),
            review_requested,
            assignees: Vec::new(),
            reviewers: self.reviewers.iter().map(|r| r.user.shown()).collect(),
            checks: None,
            has_conflicts: None,
        }
    }
}

/// `1760000000000` -> `2025-10-09T08:53:20Z`.
fn epoch_ms_to_iso(ms: u64) -> String {
    if ms == 0 {
        return String::new();
    }
    let secs = ms / 1000;
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Howard Hinnant's civil_from_days.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[derive(serde::Deserialize)]
struct SshKey {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    text: String,
}

impl SshKey {
    fn into_key(self) -> HostedSshKey {
        HostedSshKey {
            id: self.id.to_string(),
            title: self.label.unwrap_or_default(),
            key: self.text,
        }
    }
}

/// `KEY/slug`, however the remote spelled it: the clone path is
/// `/scm/KEY/slug`, the browse path `projects/KEY/repos/slug`.
fn split_project(project: &str) -> Result<(String, String)> {
    let p = project.trim_matches('/').trim_end_matches(".git");
    let p = p.strip_prefix("scm/").unwrap_or(p);
    let parts: Vec<&str> = p.split('/').collect();
    match parts.as_slice() {
        [key, slug] if !key.is_empty() && !slug.is_empty() => {
            Ok((key.to_string(), slug.to_string()))
        }
        ["projects", key, "repos", slug, ..] => Ok((key.to_string(), slug.to_string())),
        _ => Err(Error::Decode(format!(
            "not a Bitbucket Data Center repository (want KEY/slug): {project}"
        ))),
    }
}

impl BitbucketDcClient {
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

    fn api(&self) -> String {
        self.integration.api_base()
    }

    /// The SSH keys REST root sits beside `/rest/api/...`.
    fn ssh_api(&self) -> String {
        let api = self.api();
        let root = api.strip_suffix("/api/1.0").unwrap_or(&api);
        format!("{root}/ssh/latest")
    }

    /// Every page, following `nextPageStart` (never a host-supplied url),
    /// capped at [`MAX_PAGES`]. `first` already holds its query string.
    async fn fetch_all<T: DeserializeOwned>(&self, first: String, what: &str) -> Result<Vec<T>> {
        let mut out = Vec::new();
        let mut start: Option<u64> = None;
        for _ in 0..MAX_PAGES {
            let url = match start {
                None => first.clone(),
                Some(s) => format!("{first}&start={s}"),
            };
            let response = self
                .transport
                .send(self.authed(HttpRequest::get(url)))
                .await?;
            response.error_for_status(what)?;
            let page: Page<T> = response.json()?;
            out.extend(page.values);
            match (page.is_last_page, page.next_page_start) {
                (false, Some(next)) => start = Some(next),
                _ => break,
            }
        }
        Ok(out)
    }

    /// The authenticated user's name, from the `X-AUSERNAME` header.
    async fn my_name(&self) -> Result<String> {
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(format!(
                "{}/application-properties",
                self.api()
            ))))
            .await?;
        response.error_for_status("reading the Bitbucket Data Center account")?;
        response
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-ausername"))
            .map(|(_, v)| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                Error::Decode(
                    "Bitbucket Data Center did not name the authenticated user (no X-AUSERNAME header)"
                        .to_string(),
                )
            })
    }

    fn pr_url(&self, project: &str) -> Result<String> {
        let (key, slug) = split_project(project)?;
        Ok(format!(
            "{}/projects/{}/repos/{}/pull-requests",
            self.api(),
            encode_component(&key),
            encode_component(&slug)
        ))
    }

    async fn pull_requests(
        &self,
        project: &str,
        state: &str,
        extra: &str,
        what: &str,
    ) -> Result<Vec<PullRequest>> {
        let url = format!(
            "{}?state={state}&order=NEWEST&limit={LIMIT}{extra}",
            self.pr_url(project)?
        );
        self.fetch_all(url, what).await
    }

    fn ref_json(project: &str, branch: &str) -> Result<serde_json::Value> {
        let (key, slug) = split_project(project)?;
        Ok(serde_json::json!({
            "id": format!("refs/heads/{branch}"),
            "repository": { "slug": slug, "project": { "key": key } },
        }))
    }
}

#[async_trait::async_trait]
impl HostingClient for BitbucketDcClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let url = format!("{}/repos?limit={LIMIT}", self.api());
        let rows: Vec<Repo> = self
            .fetch_all(url, "listing Bitbucket repositories")
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| r.into_project(&self.integration.id))
            .collect())
    }

    async fn account_items(
        &self,
        query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        account::fetch(self, query).await
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        let url = format!("{}/keys?limit={LIMIT}", self.ssh_api());
        let rows: Vec<SshKey> = self.fetch_all(url, "listing Bitbucket SSH keys").await?;
        Ok(rows.into_iter().map(SshKey::into_key).collect())
    }

    async fn upload_ssh_key(&self, title: &str, public_key: &str) -> Result<HostedSshKey> {
        let body = serde_json::json!({ "text": public_key.trim(), "label": title });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(
                format!("{}/keys", self.ssh_api()),
                &body,
            )))
            .await?;
        response.error_for_status("uploading an SSH key to Bitbucket")?;
        Ok(response.json::<SshKey>()?.into_key())
    }

    async fn delete_ssh_key(&self, id: &str) -> Result<()> {
        let url = format!("{}/keys/{}", self.ssh_api(), encode_component(id));
        let response = self
            .transport
            .send(self.authed(HttpRequest::delete(url)))
            .await?;
        response.error_for_status("deleting a Bitbucket SSH key")
    }

    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest> {
        let from_project = req.fork_source().unwrap_or(&req.project);
        let body = serde_json::json!({
            "title": req.title,
            "description": req.body,
            "draft": req.draft,
            "fromRef": Self::ref_json(from_project, &req.source_branch)?,
            "toRef": Self::ref_json(&req.project, &req.target_branch)?,
        });
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(self.pr_url(&req.project)?, &body)))
            .await?;
        response.error_for_status("opening a Bitbucket pull request")?;
        let created: PullRequest = response.json()?;
        Ok(CreatedPullRequest {
            number: created.id,
            web_url: created.web_url(),
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
        let name = self.my_name().await?;
        let url = format!("{}/users/{}", self.api(), encode_component(&name));
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
        response.error_for_status("checking the Bitbucket token")?;
        let user: User = response.json()?;
        Ok(HostedUser {
            login: if user.name.is_empty() {
                name
            } else {
                user.name
            },
            name: user.display_name.unwrap_or_default(),
            id: user.id.map(|i| i.to_string()).unwrap_or(user.slug),
            web_url: user.links.self_.first().and_then(|l| l.href.clone()),
        })
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        // An access token's permissions are not reported: unknown, never
        // "missing".
        Ok(TokenCheck {
            user: self.current_user().await?,
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
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
            // `state` takes one value: closed is two requests.
            StateFilter::Closed => &["MERGED", "DECLINED"],
            StateFilter::All => &["ALL"],
        };
        let me = if filter.mine || filter.review_requested {
            Some(self.my_name().await?)
        } else {
            None
        };
        let mut rows = Vec::new();
        for state in states {
            rows.extend(
                self.pull_requests(project, state, "", "listing Bitbucket pull requests")
                    .await?,
            );
        }
        rows.sort_by_key(|r| std::cmp::Reverse(r.updated_date));
        Ok(rows
            .into_iter()
            .filter(|r| match (&me, filter.mine) {
                (Some(me), true) => r.is_mine(me),
                _ => true,
            })
            .map(|r| r.into_summary(project, me.as_deref()))
            .filter(|s| !filter.review_requested || s.review_requested)
            .collect())
    }

    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        let extra = format!(
            "&direction=OUTGOING&at={}",
            encode_component(&format!("refs/heads/{branch}"))
        );
        let rows = self
            .pull_requests(
                project,
                "ALL",
                &extra,
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
        Err(Error::Unsupported(NO_ISSUES.to_string()))
    }

    async fn create_issue(&self, _req: &NewIssue) -> Result<CreatedIssue> {
        Err(Error::Unsupported(NO_ISSUES.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_for;
    use crate::http::RecordedTransport;
    use crate::spec::{spec_for, CheckoutSource};
    use crate::IntegrationKind;
    use gitpoulpi_config::ApiFamily;

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn profile() -> Integration {
        let mut i = Integration::new(
            "dc",
            IntegrationKind::BitbucketDataCenter,
            "https://git.example.com/",
        );
        i.account = String::new();
        i
    }

    fn pr(id: u64, state: &str, draft: bool, author: &str, reviewer: &str, ms: u64) -> String {
        format!(
            r#"{{"id":{id},"title":"T{id}","state":"{state}","draft":{draft},"updatedDate":{ms},
            "fromRef":{{"id":"refs/heads/feat","displayId":"feat","repository":{{"slug":"fork","project":{{"key":"~ANN"}}}}}},
            "toRef":{{"id":"refs/heads/main","displayId":"main","repository":{{"slug":"repo","project":{{"key":"PRJ"}}}}}},
            "author":{{"user":{{"name":"{author}","displayName":"Author {author}"}},"role":"AUTHOR"}},
            "reviewers":[{{"user":{{"name":"{reviewer}","displayName":"Rev {reviewer}"}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}}],
            "links":{{"self":[{{"href":"https://git.example.com/projects/PRJ/repos/repo/pull-requests/{id}"}}]}}}}"#
        )
    }

    fn page(rows: &[String], last: bool, next: u64) -> String {
        format!(
            r#"{{"size":{},"limit":100,"isLastPage":{last},"nextPageStart":{next},"values":[{}]}}"#,
            rows.len(),
            rows.join(",")
        )
    }

    fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
        client_for(
            profile(),
            "tok".into(),
            t,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap()
    }

    #[test]
    fn the_dc_kind_gets_the_dc_client_with_a_bearer_token() {
        let t = Arc::new(RecordedTransport::new().on("/repos", 200, &page(&[], true, 0)));
        let c = client(t.clone());
        assert!(block_on(c.list_projects()).unwrap().is_empty());
        let seen = t.requests();
        assert_eq!(
            seen[0].url,
            "https://git.example.com/rest/api/1.0/repos?limit=100"
        );
        assert!(seen[0]
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer tok"));
    }

    #[test]
    fn repositories_page_with_next_page_start_and_map_clone_links() {
        let repo = |slug: &str| {
            format!(
                r#"{{"slug":"{slug}","name":"{slug}","public":false,"project":{{"key":"PRJ"}},
                "links":{{"clone":[{{"href":"https://me@git.example.com/scm/prj/{slug}.git","name":"http"}},
                {{"href":"ssh://git@git.example.com:7999/prj/{slug}.git","name":"ssh"}}],
                "self":[{{"href":"https://git.example.com/projects/PRJ/repos/{slug}/browse"}}]}}}}"#
            )
        };
        let t = Arc::new(
            RecordedTransport::new()
                .on("start=1", 200, &page(&[repo("b")], true, 0))
                .on("/repos", 200, &page(&[repo("a")], false, 1)),
        );
        let rows = block_on(client(t.clone()).list_projects()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].full_name, "PRJ/a");
        assert_eq!(
            rows[0].https_url.as_deref(),
            Some("https://git.example.com/scm/prj/a.git")
        );
        assert_eq!(
            rows[0].ssh_url.as_deref(),
            Some("ssh://git@git.example.com:7999/prj/a.git")
        );
        assert!(rows[0].private);
        assert_eq!(t.requests().len(), 2);
        assert!(t.requests()[1].url.ends_with("limit=100&start=1"));
    }

    #[test]
    fn current_user_reads_x_ausername_then_the_user_record() {
        let t = Arc::new(
            RecordedTransport::new()
                .on_with_headers(
                    "/application-properties",
                    200,
                    "{}",
                    &[("X-AUSERNAME", "jdoe")],
                )
                .on(
                    "/users/jdoe",
                    200,
                    r#"{"name":"jdoe","slug":"jdoe","id":7,"displayName":"Jane Doe",
                    "links":{"self":[{"href":"https://git.example.com/users/jdoe"}]}}"#,
                ),
        );
        let user = block_on(client(t.clone()).current_user()).unwrap();
        assert_eq!(
            (user.login.as_str(), user.name.as_str(), user.id.as_str()),
            ("jdoe", "Jane Doe", "7")
        );
        assert_eq!(
            user.web_url.as_deref(),
            Some("https://git.example.com/users/jdoe")
        );
    }

    #[test]
    fn a_missing_x_ausername_is_a_clear_error() {
        let t = Arc::new(RecordedTransport::new().on("/application-properties", 200, "{}"));
        let err = block_on(client(t).current_user()).unwrap_err();
        assert!(err.to_string().contains("X-AUSERNAME"), "{err}");
    }

    #[test]
    fn pull_requests_map_state_draft_fork_and_milliseconds() {
        let rows = [
            pr(1, "OPEN", false, "ann", "jdoe", 1_760_000_000_000),
            pr(2, "OPEN", true, "bob", "zed", 1_760_000_100_000),
            pr(3, "MERGED", false, "ann", "zed", 1_760_000_200_000),
            pr(4, "DECLINED", false, "ann", "zed", 1_760_000_300_000),
        ];
        let t = Arc::new(RecordedTransport::new().on("/pull-requests", 200, &page(&rows, true, 0)));
        let got = block_on(client(t.clone()).list_pull_requests(
            "PRJ/repo",
            &PrFilter {
                state: StateFilter::All,
                ..PrFilter::default()
            },
        ))
        .unwrap();
        assert_eq!(
            t.requests()[0].url,
            "https://git.example.com/rest/api/1.0/projects/PRJ/repos/repo/pull-requests?state=ALL&order=NEWEST&limit=100"
        );
        // Newest first.
        assert_eq!(
            got.iter().map(|p| p.number).collect::<Vec<_>>(),
            [4, 3, 2, 1]
        );
        let by = |n: u64| got.iter().find(|p| p.number == n).unwrap();
        assert_eq!(by(1).state, PullRequestState::Open);
        assert_eq!(by(2).state, PullRequestState::Draft);
        assert_eq!(by(3).state, PullRequestState::Merged);
        assert_eq!(by(4).state, PullRequestState::Closed);
        assert_eq!(by(1).source_project.as_deref(), Some("~ANN/fork"));
        assert_eq!(by(1).source_branch, "feat");
        assert_eq!(by(1).target_branch, "main");
        assert_eq!(by(1).updated_at, "2025-10-09T08:53:20Z");
        assert_eq!(by(1).author, "Author ann");
        assert_eq!(by(1).reviewers, ["Rev jdoe"]);
        assert_eq!(
            by(1).web_url,
            "https://git.example.com/projects/PRJ/repos/repo/pull-requests/1"
        );
    }

    #[test]
    fn closed_is_two_requests_and_mine_and_review_requested_filter_locally() {
        let open = [
            pr(1, "OPEN", false, "ann", "jdoe", 1_000),
            pr(2, "OPEN", false, "jdoe", "zed", 2_000),
        ];
        let t = Arc::new(
            RecordedTransport::new()
                .on_with_headers(
                    "/application-properties",
                    200,
                    "{}",
                    &[("x-ausername", "jdoe")],
                )
                .on("/pull-requests", 200, &page(&open, true, 0)),
        );
        let c = client(t.clone());
        let mine = block_on(c.list_pull_requests(
            "PRJ/repo",
            &PrFilter {
                mine: true,
                ..PrFilter::default()
            },
        ))
        .unwrap();
        assert_eq!(mine.iter().map(|p| p.number).collect::<Vec<_>>(), [2]);
        let review = block_on(c.list_pull_requests(
            "PRJ/repo",
            &PrFilter {
                review_requested: true,
                ..PrFilter::default()
            },
        ))
        .unwrap();
        assert_eq!(review.iter().map(|p| p.number).collect::<Vec<_>>(), [1]);

        let closed =
            Arc::new(RecordedTransport::new().on("/pull-requests", 200, &page(&[], true, 0)));
        block_on(client(closed.clone()).list_pull_requests(
            "PRJ/repo",
            &PrFilter {
                state: StateFilter::Closed,
                ..PrFilter::default()
            },
        ))
        .unwrap();
        let urls: Vec<String> = closed.requests().into_iter().map(|r| r.url).collect();
        assert!(urls[0].contains("state=MERGED") && urls[1].contains("state=DECLINED"));
    }

    #[test]
    fn the_branch_lookup_asks_for_outgoing_requests_at_the_ref() {
        let t = Arc::new(RecordedTransport::new().on(
            "/pull-requests",
            200,
            &page(&[pr(9, "OPEN", false, "ann", "zed", 5)], true, 0),
        ));
        // A remote spelled with the clone path still resolves.
        let found =
            block_on(client(t.clone()).pull_request_for_branch("scm/PRJ/repo.git", "feat/x"))
                .unwrap()
                .unwrap();
        assert_eq!(found.number, 9);
        let url = &t.requests()[0].url;
        assert!(
            url.contains("/projects/PRJ/repos/repo/pull-requests?state=ALL"),
            "{url}"
        );
        assert!(
            url.contains("direction=OUTGOING&at=refs%2Fheads%2Ffeat%2Fx"),
            "{url}"
        );
    }

    #[test]
    fn create_posts_refs_with_slug_and_project_key_and_reads_the_web_url() {
        let t = Arc::new(RecordedTransport::new().on(
            "/pull-requests",
            201,
            &pr(12, "OPEN", true, "ann", "zed", 5),
        ));
        let created = block_on(client(t.clone()).create_pull_request(&NewPullRequest {
            project: "PRJ/repo".into(),
            source_branch: "feat".into(),
            target_branch: "main".into(),
            title: "Title".into(),
            body: "Body".into(),
            draft: true,
            source_project: Some("~ANN/fork".into()),
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(created.number, 12);
        assert_eq!(
            created.web_url,
            "https://git.example.com/projects/PRJ/repos/repo/pull-requests/12"
        );
        let req = &t.requests()[0];
        assert_eq!(
            req.url,
            "https://git.example.com/rest/api/1.0/projects/PRJ/repos/repo/pull-requests"
        );
        let body: serde_json::Value = serde_json::from_str(req.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["title"], "Title");
        assert_eq!(body["description"], "Body");
        assert_eq!(body["draft"], true);
        assert_eq!(body["fromRef"]["id"], "refs/heads/feat");
        assert_eq!(body["fromRef"]["repository"]["slug"], "fork");
        assert_eq!(body["fromRef"]["repository"]["project"]["key"], "~ANN");
        assert_eq!(body["toRef"]["id"], "refs/heads/main");
        assert_eq!(body["toRef"]["repository"]["project"]["key"], "PRJ");
    }

    #[test]
    fn ssh_keys_use_the_ssh_rest_root() {
        let key = r#"{"id":5,"label":"laptop","text":"ssh-ed25519 AAAA me"}"#;
        let t = Arc::new(RecordedTransport::new().on("/keys/5", 204, "").on(
            "/ssh/latest/keys",
            200,
            &format!(r#"{{"isLastPage":true,"values":[{key}]}}"#),
        ));
        let c = client(t.clone());
        let keys = block_on(c.list_ssh_keys()).unwrap();
        assert_eq!(
            (keys[0].id.as_str(), keys[0].title.as_str()),
            ("5", "laptop")
        );
        assert_eq!(
            t.requests()[0].url,
            "https://git.example.com/rest/ssh/latest/keys?limit=100"
        );
        block_on(c.delete_ssh_key("5")).unwrap();
        assert_eq!(
            t.requests()[1].url,
            "https://git.example.com/rest/ssh/latest/keys/5"
        );
    }

    #[test]
    fn issues_are_unsupported() {
        let c = client(Arc::new(RecordedTransport::new()));
        assert!(matches!(
            block_on(c.list_issues("PRJ/repo", &IssueFilter::default())),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn project_spellings_and_epoch_dates() {
        assert_eq!(
            split_project("PRJ/repo").unwrap(),
            ("PRJ".into(), "repo".into())
        );
        assert_eq!(
            split_project("scm/~ann/repo.git").unwrap(),
            ("~ann".into(), "repo".into())
        );
        assert_eq!(
            split_project("projects/PRJ/repos/repo/browse").unwrap(),
            ("PRJ".into(), "repo".into())
        );
        assert!(split_project("just-one").is_err());
        assert_eq!(epoch_ms_to_iso(0), "");
        assert_eq!(epoch_ms_to_iso(951_782_400_000), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn a_data_center_pull_request_is_fetched_from_its_head_ref() {
        let summary = block_on(async {
            let t = Arc::new(RecordedTransport::new().on(
                "/pull-requests",
                200,
                &page(&[pr(7, "OPEN", false, "ann", "zed", 5)], true, 0),
            ));
            client(t)
                .list_pull_requests("PRJ/repo", &PrFilter::default())
                .await
                .unwrap()
                .remove(0)
        });
        assert_eq!(
            (spec_for(ApiFamily::Bitbucket).checkout)(&summary),
            CheckoutSource::HeadRef {
                remote_ref: "refs/pull-requests/7/from".into(),
                local: "pr/7".into(),
            }
        );
    }
}
