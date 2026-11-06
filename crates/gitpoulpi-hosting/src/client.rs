//! The provider-agnostic hosting port and its plumbing.
//!
//! The implementations live in [`crate::github`] and [`crate::gitlab`];
//! kinds with no client yet (Bitbucket, Azure DevOps, Jira, Trello) get
//! [`UnsupportedClient`], whose every call says so.

// With `hosting` forced on but no provider built, the paging and encoding
// helpers below have no caller and `client_for` ignores its inputs.
#![cfg_attr(
    not(any(
        feature = "github",
        feature = "gitlab",
        feature = "jira",
        feature = "bitbucket",
        feature = "azure-devops",
        feature = "trello",
        feature = "gitea",
        feature = "linear",
        feature = "youtrack",
        feature = "redmine",
        feature = "gerrit"
    )),
    allow(dead_code, unused_variables)
)]

use std::sync::Arc;

#[cfg(any(feature = "github", feature = "gitlab"))]
use serde::de::DeserializeOwned;

#[cfg(feature = "bitbucket")]
use crate::bitbucket::BitbucketClient;
#[cfg(feature = "github")]
use crate::github::GitHubClient;
#[cfg(feature = "gitlab")]
use crate::gitlab::GitLabClient;
use crate::http::HttpTransport;
#[cfg(any(feature = "github", feature = "gitlab"))]
use crate::http::{HttpRequest, HttpResponse};
use crate::integration::Integration;
use crate::issue_flow::{CreatedBranch, IssueLinks, NewIssueBranch};
use crate::metadata::{HostedLabel, HostedMilestone, IssueEdit, ItemMeta, MetaKind, PrEdit};
use crate::model::{
    HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, PrFilter, PullRequestSummary,
    TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::review::{
    PullRequestDetail, ReviewCapabilities, ReviewComment, ReviewFile, ReviewOutcome,
    ReviewSubmission, ReviewThread,
};
use crate::spec::{spec_for, CheckoutSource};
use crate::{Error, Result};
#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "jira",
    feature = "bitbucket",
    feature = "azure-devops",
    feature = "trello",
    feature = "gitea",
    feature = "linear",
    feature = "youtrack",
    feature = "redmine",
    feature = "gerrit",
    feature = "local-issues"
))]
use gitpoulpi_config::ApiFamily;

/// How many rows one page asks for: 100 is the maximum both hosts accept,
/// and a larger ask is silently clamped, which would make the pagination
/// loop's termination condition wrong on one host.
#[cfg(any(feature = "github", feature = "gitlab", feature = "azure-devops"))]
pub(crate) const PAGE_SIZE: u32 = 100;

/// A safety rail on the pagination loop: a host that ignores the page
/// parameter (a misconfigured proxy in front of a self-managed instance)
/// returns page 1 forever. 50 pages is 5000 rows, past any real account.
#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "jira",
    feature = "bitbucket",
    feature = "azure-devops",
    feature = "gitea",
    feature = "linear",
    feature = "youtrack",
    feature = "redmine",
    feature = "gerrit"
))]
pub(crate) const MAX_PAGES: u32 = 50;

/// What opening a pull/merge request needs, host-agnostic.
///
/// `project` is the `remote_web_base`-derived path (`group/sub/proj` for
/// GitLab, `owner/name` for GitHub); each client encodes it as its API
/// needs (GitLab URL-encodes it as one segment).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewPullRequest {
    pub project: String,
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub body: String,
    pub draft: bool,
    /// The project the source branch lives in, when it is a fork of
    /// `project` (`None`: same project). Path form, like `project`.
    pub source_project: Option<String>,
    /// Logins to assign (GitHub, GitLab).
    pub assignees: Vec<String>,
    /// Logins asked to review (GitHub, GitLab).
    pub reviewers: Vec<String>,
    /// Label names (GitHub, GitLab).
    pub labels: Vec<String>,
    /// `HostedMilestone::id` (GitHub, GitLab).
    pub milestone: Option<u64>,
}

impl NewPullRequest {
    /// `source_project`, only when it really differs from `project`.
    pub fn fork_source(&self) -> Option<&str> {
        self.source_project
            .as_deref()
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case(&self.project))
    }
}

/// What the host answers with once the request exists.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct CreatedPullRequest {
    /// The PR number (GitHub) or MR `iid` (GitLab) - the number the
    /// host's own UI shows next to it, not its opaque database id.
    pub number: u64,
    pub web_url: String,
    /// The request exists but part of its metadata could not be set
    /// (GitHub applies it in a second call).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// What opening an issue needs, host-agnostic (v1: title, body, labels).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewIssue {
    /// `owner/name` (GitHub) or `group/sub/proj` (GitLab).
    pub project: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    /// Jira issue-type id, Azure work-item type name.
    pub issue_type: Option<String>,
    /// Trello list id.
    pub list_id: Option<String>,
    /// Logins to assign (GitHub, GitLab).
    pub assignees: Vec<String>,
    /// `HostedMilestone::id` (GitHub, GitLab).
    pub milestone: Option<u64>,
}

/// What the host answers with once the issue exists.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CreatedIssue {
    /// The issue number (GitHub) or `iid` (GitLab).
    pub number: u64,
    pub web_url: String,
    /// What a human types to reference it (`#12`, `ABC-123`).
    pub reference: String,
}

#[async_trait::async_trait]
pub trait HostingClient: Send + Sync {
    fn integration(&self) -> &Integration;

    /// The error for an operation this client does not offer. The optional
    /// operations below return it by default, so one override changes the
    /// answer for all of them.
    fn unsupported(&self, what: &str) -> Error {
        Error::Unsupported(what.to_string())
    }

    /// Every project the token can see, newest-activity first where the
    /// host offers that ordering.
    async fn list_projects(&self) -> Result<Vec<HostedProject>>;

    /// The SSH public keys already on the account, so an *upload a key*
    /// flow can say "you already have this one" instead of creating a
    /// duplicate the user then has to find and delete.
    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>>;

    /// Adds `public_key` (an `ssh-ed25519 AAAA... comment` line) to the
    /// account under `title`.
    async fn upload_ssh_key(&self, title: &str, public_key: &str) -> Result<HostedSshKey>;

    /// Opens a pull request (GitHub) or merge request (GitLab) from
    /// `req.source_branch` into `req.target_branch`.
    async fn create_pull_request(&self, req: &NewPullRequest) -> Result<CreatedPullRequest>;

    /// The account the token belongs to.
    async fn current_user(&self) -> Result<HostedUser> {
        Err(self.unsupported("current_user"))
    }

    /// The branches of `project` the host protects (names and `*`
    /// patterns), for a history rewrite that must not force-push them
    /// blindly. What the token cannot read is an error here; the caller
    /// ([`crate::protection::lookup`]) turns it into "unknown".
    async fn protected_branches(
        &self,
        _project: &str,
    ) -> Result<crate::protection::ProtectedBranches> {
        Err(self.unsupported("protected_branches"))
    }

    /// Validates the token and reports its user, scopes and expiry.
    async fn check_token(&self) -> Result<TokenCheck> {
        Err(self.unsupported("check_token"))
    }

    /// Pull/merge requests of `project` (`owner/name`, `group/sub/proj`).
    async fn list_pull_requests(
        &self,
        _project: &str,
        _filter: &PrFilter,
    ) -> Result<Vec<PullRequestSummary>> {
        Err(self.unsupported("list_pull_requests"))
    }

    /// The request that best represents `branch`: open, then draft, then
    /// merged, then closed; newest first among equals.
    async fn pull_request_for_branch(
        &self,
        _project: &str,
        _branch: &str,
    ) -> Result<Option<PullRequestSummary>> {
        Err(self.unsupported("pull_request_for_branch"))
    }

    async fn list_issues(
        &self,
        _project: &str,
        _filter: &IssueFilter,
    ) -> Result<Vec<IssueSummary>> {
        Err(self.unsupported("list_issues"))
    }

    /// Opens an issue on `req.project`.
    async fn create_issue(&self, _req: &NewIssue) -> Result<CreatedIssue> {
        Err(self.unsupported("create_issue"))
    }

    /// The types / lists the New-issue form offers for `project`. Empty
    /// by default (GitHub and GitLab have neither).
    async fn issue_create_options(&self, _project: &str) -> Result<IssueCreateOptions> {
        Ok(IssueCreateOptions::default())
    }

    /// Removes the account SSH key with the host's own `id`.
    async fn delete_ssh_key(&self, _id: &str) -> Result<()> {
        Err(self.unsupported("delete_ssh_key"))
    }

    // -- in-app code review (see `crate::review`) --

    /// What a review on this host may do, so a UI hides the rest instead
    /// of offering a choice that fails (GitLab has no "request changes").
    /// All-false by default.
    fn review_capabilities(&self) -> ReviewCapabilities {
        ReviewCapabilities::default()
    }

    /// One request, read for review: description, head and base commits,
    /// merge state, checks.
    async fn pull_request_detail(&self, _project: &str, _number: u64) -> Result<PullRequestDetail> {
        Err(self.unsupported("pull_request_detail"))
    }

    /// [`Self::pull_request_detail`] at patch set `patch_set` (`None`: the
    /// newest). Only Gerrit has patch sets; the others ignore it.
    async fn pull_request_detail_at(
        &self,
        project: &str,
        number: u64,
        _patch_set: Option<u32>,
    ) -> Result<PullRequestDetail> {
        self.pull_request_detail(project, number).await
    }

    /// [`Self::pull_request_files`] at a patch set.
    async fn pull_request_files_at(
        &self,
        project: &str,
        number: u64,
        _patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        self.pull_request_files(project, number).await
    }

    /// [`Self::pull_request_files_at`] as the difference from patch set
    /// `base` instead of from the target branch (Gerrit's "compare with"; the
    /// others have no patch sets and ignore it).
    async fn pull_request_files_against(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        _base_patch_set: Option<u32>,
    ) -> Result<Vec<ReviewFile>> {
        self.pull_request_files_at(project, number, patch_set).await
    }

    /// Submits (merges) a change on the host (Gerrit; `PullRequestDetail::
    /// can_submit` says when the server allows it). Outward, not undoable.
    async fn submit_change(&self, _project: &str, _number: u64) -> Result<()> {
        Err(self.unsupported("submit_change"))
    }

    /// Abandons a change, with an optional message to the owner and
    /// reviewers (Gerrit). Outward, not undoable from GitPoulpi.
    async fn abandon_change(
        &self,
        _project: &str,
        _number: u64,
        _message: Option<&str>,
    ) -> Result<()> {
        Err(self.unsupported("abandon_change"))
    }

    /// [`Self::pull_request_threads`] judged against a patch set: threads
    /// written on another one are outdated.
    async fn pull_request_threads_at(
        &self,
        project: &str,
        number: u64,
        _patch_set: Option<u32>,
    ) -> Result<Vec<ReviewThread>> {
        self.pull_request_threads(project, number).await
    }

    /// The files the request changes, for the file list.
    async fn pull_request_files(&self, _project: &str, _number: u64) -> Result<Vec<ReviewFile>> {
        Err(self.unsupported("pull_request_files"))
    }

    /// The code threads of the request, resolved ones included.
    async fn pull_request_threads(
        &self,
        _project: &str,
        _number: u64,
    ) -> Result<Vec<ReviewThread>> {
        Err(self.unsupported("pull_request_threads"))
    }

    /// The comments that are not on code, oldest first.
    async fn pull_request_conversation(
        &self,
        _project: &str,
        _number: u64,
    ) -> Result<Vec<ReviewComment>> {
        Err(self.unsupported("pull_request_conversation"))
    }

    /// The reviews already submitted (verdict and summary), oldest first.
    /// Empty drafts are not listed.
    async fn pull_request_reviews(
        &self,
        _project: &str,
        _number: u64,
    ) -> Result<Vec<crate::review::SubmittedReview>> {
        Err(self.unsupported("pull_request_reviews"))
    }

    /// Posts a conversation comment (not on code), immediately.
    async fn add_comment(
        &self,
        _project: &str,
        _number: u64,
        _body: &str,
    ) -> Result<ReviewComment> {
        Err(self.unsupported("add_comment"))
    }

    /// Answers a thread (`ReviewThread::id`), immediately.
    async fn reply_to_thread(
        &self,
        _project: &str,
        _number: u64,
        _thread_id: &str,
        _body: &str,
    ) -> Result<ReviewComment> {
        Err(self.unsupported("reply_to_thread"))
    }

    /// Resolves (`true`) or reopens (`false`) a thread.
    async fn resolve_thread(
        &self,
        _project: &str,
        _number: u64,
        _thread_id: &str,
        _resolved: bool,
    ) -> Result<()> {
        Err(self.unsupported("resolve_thread"))
    }

    /// Submits a review: the collected line comments and the verdict in
    /// one go. `Unsupported` for an event [`Self::review_capabilities`]
    /// does not offer.
    async fn submit_review(
        &self,
        _project: &str,
        _number: u64,
        _review: &ReviewSubmission,
    ) -> Result<ReviewOutcome> {
        Err(self.unsupported("submit_review"))
    }

    // -- host-driven flows (slice W4) ----------------------------------------

    /// Creates a branch on the host for an issue (GitLab: the branches API,
    /// GitHub: `createLinkedBranch`, Gitea / Forgejo: the branches API).
    /// Outward, not undoable from GitPoulpi.
    async fn create_issue_branch(&self, _req: &NewIssueBranch) -> Result<CreatedBranch> {
        Err(self.unsupported("create_issue_branch"))
    }

    /// The branches and requests the host links to issue `number` itself.
    async fn issue_links(&self, _project: &str, _number: u64) -> Result<IssueLinks> {
        Err(self.unsupported("issue_links"))
    }

    /// Branch names of `project` that start with (GitHub) or contain
    /// (GitLab, Gitea) `search`; the caller filters them exactly.
    async fn branch_names(&self, _project: &str, _search: &str) -> Result<Vec<String>> {
        Err(self.unsupported("branch_names"))
    }

    // -- metadata (slice M): GitHub and GitLab only ---------------------------

    /// The people who can be assigned to or asked to review in `project`,
    /// narrowed by `query` (empty: everyone, paged).
    async fn project_members(&self, _project: &str, _query: &str) -> Result<Vec<HostedUser>> {
        Err(self.unsupported("project_members"))
    }

    /// The labels of `project`.
    async fn project_labels(&self, _project: &str) -> Result<Vec<HostedLabel>> {
        Err(self.unsupported("project_labels"))
    }

    /// The open milestones of `project`.
    async fn project_milestones(&self, _project: &str) -> Result<Vec<HostedMilestone>> {
        Err(self.unsupported("project_milestones"))
    }

    /// What issue / request `number` carries now (the picker's checked rows).
    async fn item_meta(&self, _project: &str, _number: u64, _kind: MetaKind) -> Result<ItemMeta> {
        Err(self.unsupported("item_meta"))
    }

    /// Replaces the assignees / labels / milestone of an issue.
    async fn update_issue(&self, _project: &str, _number: u64, _edit: &IssueEdit) -> Result<()> {
        Err(self.unsupported("update_issue"))
    }

    // -- local issues (slice L1): status-based sources ----------------------

    /// The statuses an issue can be in, board order (the local Markdown
    /// source; the Kanban slice reads it too). Empty by default.
    async fn issue_statuses(&self, _project: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Moves an issue to `status` (local Markdown).
    async fn set_issue_status(&self, _project: &str, _number: u64, _status: &str) -> Result<()> {
        Err(self.unsupported("set_issue_status"))
    }

    /// Closes (`true`) or reopens (`false`) an issue (local Markdown: the
    /// configured done status, or the first other one; GitHub, GitLab,
    /// Gitea: the issue's state).
    async fn set_issue_state(&self, _project: &str, _number: u64, _closed: bool) -> Result<()> {
        Err(self.unsupported("set_issue_state"))
    }

    // -- the Kanban board (slice L2) -----------------------------------------

    /// How this source forms the board's columns (`crate::board`). Open /
    /// Closed by default; a provider with a workflow overrides it and, for
    /// the status-like sources, `issue_statuses`.
    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::OpenClosed
    }

    /// Moves `issue` to column `to` of the board `spec`. The default runs
    /// the [`crate::board::plan_move`] plan through `set_issue_status`,
    /// `update_issue` and `set_issue_state`, so a client gets a move by
    /// implementing those (Unsupported otherwise: a read-only board).
    async fn move_issue(
        &self,
        project: &str,
        issue: &IssueSummary,
        spec: &crate::board::BoardSpec,
        to: &crate::board::BoardColumn,
    ) -> Result<()> {
        let plan = crate::board::plan_move(spec, issue, to)?;
        if let Some(_list) = &plan.list {
            return Err(self.unsupported("move_issue"));
        }
        if let Some(status) = &plan.status {
            self.set_issue_status(project, issue.number, status).await?;
        }
        if let Some(labels) = &plan.labels {
            let edit = IssueEdit {
                labels: Some(labels.clone()),
                ..IssueEdit::default()
            };
            self.update_issue(project, issue.number, &edit).await?;
        }
        if let Some(closed) = plan.closed {
            self.set_issue_state(project, issue.number, closed).await?;
        }
        Ok(())
    }

    /// Replaces the assignees / reviewers / labels / milestone of a pull /
    /// merge request.
    async fn update_pull_request(
        &self,
        _project: &str,
        _number: u64,
        _edit: &PrEdit,
    ) -> Result<()> {
        Err(self.unsupported("update_pull_request"))
    }

    // -- the code of a request through the API, no clone (slice S3a) --

    /// The diff of a request, file by file, with hunks, read from the host.
    /// `patch_set` and `compare` are Gerrit's; the others ignore them. A
    /// file whose patch the host left out carries `omitted`.
    async fn pull_request_diff(
        &self,
        _project: &str,
        _number: u64,
        _patch_set: Option<u32>,
        _compare: Option<u32>,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        Err(self.unsupported("pull_request_diff"))
    }

    /// The commits of a request, oldest first.
    async fn pull_request_commits(
        &self,
        _project: &str,
        _number: u64,
    ) -> Result<Vec<crate::review::ApiCommit>> {
        Err(self.unsupported("pull_request_commits"))
    }

    /// What one commit changed (against its first parent), read from the host.
    async fn commit_diff(
        &self,
        _project: &str,
        _sha: &str,
    ) -> Result<Vec<crate::review::ApiFileDiff>> {
        Err(self.unsupported("commit_diff"))
    }

    /// The bytes of `path` at `rev` (a branch, tag or commit; `None`: the
    /// project's default branch). `Ok(None)` when the file does not exist
    /// there.
    async fn file_content(
        &self,
        _project: &str,
        _path: &str,
        _rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        Err(self.unsupported("file_content"))
    }

    /// The account-wide lists `query` asks for (requests waiting for my
    /// review, every open request of my namespaces, issues assigned to me),
    /// across every repository rather than one project. Every code host but
    /// Bitbucket Cloud answers; the others say `Unsupported`.
    async fn account_items(
        &self,
        _query: &crate::account::AccountQuery,
    ) -> Result<crate::account::AccountItems> {
        Err(self.unsupported("account_items"))
    }
}

/// Builds the client for `integration`, over `transport`, once `gate` has
/// admitted it.
///
/// The gate checks the address against the trust record **before** anything
/// exists (see [`crate::trust`]), and the transport the client gets is
/// wrapped so that no request leaves the integration's origins. A folder of
/// local notes has no network and skips the gate.
///
/// Families without a client return an [`UnsupportedClient`] rather than
/// failing here; callers that want to hide the feature altogether gate on
/// `kind.capabilities().implemented`.
pub fn client_for(
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
    gate: &crate::trust::TrustGate,
) -> Result<Box<dyn HostingClient>> {
    let transport = if integration.kind.capabilities().local_folder {
        transport
    } else {
        let admitted = gate.admit(&integration)?;
        Arc::new(crate::trust::GuardedTransport::new(
            transport,
            admitted.subject,
            admitted.record.origin.clone(),
            admitted.origins,
            admitted.record.tls.clone(),
        )) as Arc<dyn HttpTransport>
    };
    Ok(build_client(integration, token, transport))
}

fn build_client(
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
) -> Box<dyn HostingClient> {
    match integration.kind.family() {
        #[cfg(feature = "github")]
        ApiFamily::Github => Box::new(GitHubClient::new(integration, token, transport)),
        #[cfg(feature = "gitlab")]
        ApiFamily::Gitlab => Box::new(GitLabClient::new(integration, token, transport)),
        #[cfg(feature = "jira")]
        ApiFamily::Jira => Box::new(crate::jira::JiraClient::new(integration, token, transport)),
        #[cfg(feature = "bitbucket")]
        ApiFamily::Bitbucket if integration.kind == crate::IntegrationKind::Bitbucket => {
            Box::new(BitbucketClient::new(integration, token, transport))
        }
        #[cfg(feature = "bitbucket")]
        ApiFamily::Bitbucket => Box::new(crate::bitbucket::dc::BitbucketDcClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "azure-devops")]
        ApiFamily::AzureDevops => Box::new(crate::azure_devops::AzureDevopsClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "trello")]
        ApiFamily::Trello => Box::new(crate::trello::TrelloClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "gitea")]
        ApiFamily::Gitea => Box::new(crate::gitea::GiteaClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "linear")]
        ApiFamily::Linear => Box::new(crate::linear::LinearClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "youtrack")]
        ApiFamily::YouTrack => Box::new(crate::youtrack::YouTrackClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "redmine")]
        ApiFamily::Redmine => Box::new(crate::redmine::RedmineClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "gerrit")]
        ApiFamily::Gerrit => Box::new(crate::gerrit::GerritClient::new(
            integration,
            token,
            transport,
        )),
        #[cfg(feature = "local-issues")]
        ApiFamily::LocalMarkdown => {
            Box::new(crate::local_markdown::LocalMarkdownClient::new(integration))
        }
        // A kind this binary was built without.
        #[allow(unreachable_patterns)]
        _ => Box::new(UnsupportedClient { integration }),
    }
}

/// Stands in for a family with no client yet: every call fails with
/// [`Error::NotImplemented`] naming the product.
pub struct UnsupportedClient {
    integration: Integration,
}

#[async_trait::async_trait]
impl HostingClient for UnsupportedClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }
    fn unsupported(&self, _what: &str) -> Error {
        Error::NotImplemented(self.integration.kind.label().to_string())
    }
    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        Err(self.unsupported("list_projects"))
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
}

/// Fetches every page of a list endpoint (`base_url` already carries a
/// `?`), stopping on the first short page or at [`MAX_PAGES`].
///
/// Counting rows rather than following `Link: rel="next"` keeps this
/// working behind proxies that rewrite or drop that header.
#[cfg(any(feature = "github", feature = "gitlab"))]
pub(crate) async fn fetch_pages<T: DeserializeOwned>(
    transport: &dyn HttpTransport,
    request: impl Fn(String) -> HttpRequest,
    base_url: &str,
    what: &str,
) -> Result<Vec<T>> {
    fetch_pages_with(transport, request, base_url, what, |r| r.json()).await
}

/// [`fetch_pages`] for an endpoint whose rows sit inside a wrapper object
/// (GitHub's search API): `rows` pulls one page's rows out of the response.
#[cfg(any(feature = "github", feature = "gitlab"))]
pub(crate) async fn fetch_pages_with<T>(
    transport: &dyn HttpTransport,
    request: impl Fn(String) -> HttpRequest,
    base_url: &str,
    what: &str,
    rows: impl Fn(&HttpResponse) -> Result<Vec<T>>,
) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for page in 1..=MAX_PAGES {
        let url = format!("{base_url}&per_page={PAGE_SIZE}&page={page}");
        let response = transport.send(request(url)).await?;
        response.error_for_status(what)?;
        let page_rows = rows(&response)?;
        let count = page_rows.len();
        out.extend(page_rows);
        if count < PAGE_SIZE as usize {
            break;
        }
    }
    Ok(out)
}

/// Percent-encodes everything but RFC 3986 unreserved characters, for a
/// query-string value (`feature/x` becomes `feature%2Fx`).
// Linear (GraphQL, a JSON body) is the one provider that never encodes a URL part.
#[cfg_attr(
    not(any(
        feature = "github",
        feature = "gitlab",
        feature = "jira",
        feature = "bitbucket",
        feature = "azure-devops",
        feature = "trello",
        feature = "gitea",
        feature = "youtrack",
        feature = "redmine",
        feature = "gerrit",
        feature = "local-issues"
    )),
    allow(dead_code)
)]
pub(crate) fn encode_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub use crate::spec::web_base;

/// The browser page of pull/merge request `number` in `project`.
pub fn pull_request_web_url(
    integration: &Integration,
    project: &str,
    number: u64,
) -> Option<String> {
    let base = web_base(integration)?;
    (spec_for(integration.kind.family()).pr_web_url)(&base, project, number)
}

/// Where a host keeps the head of pull/merge request `number` - the ref
/// a fork's commits are fetched from - and the local branch name to
/// track it with: `("refs/pull/7/head", "pr/7")` on GitHub,
/// `("refs/merge-requests/7/head", "mr/7")` on GitLab. `None` for hosts
/// that fetch a pull request by branch instead ([`CheckoutSource::Branch`]).
pub fn pull_request_head_ref(
    kind: crate::IntegrationKind,
    number: u64,
) -> Option<(String, String)> {
    let probe = PullRequestSummary {
        number,
        iid_label: String::new(),
        title: String::new(),
        state: crate::model::PullRequestState::Open,
        source_branch: String::new(),
        target_branch: String::new(),
        source_project: None,
        author: String::new(),
        web_url: String::new(),
        updated_at: String::new(),
        review_requested: false,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    };
    match (spec_for(kind.family()).checkout)(&probe) {
        CheckoutSource::HeadRef { remote_ref, local } => Some((remote_ref, local)),
        CheckoutSource::Branch { .. } => None,
    }
}

/// The page to edit a pull / merge request whose browser page is
/// `web_url`: GitLab has a dedicated edit view (`.../merge_requests/53/edit`);
/// GitHub edits the title and description in place on the request's page.
pub fn pull_request_edit_url(kind: Option<crate::IntegrationKind>, web_url: &str) -> String {
    match kind {
        Some(kind) => (spec_for(kind.family()).pr_edit_url)(web_url),
        None => web_url.to_string(),
    }
}

/// The browser page of issue `number` in `project`.
pub fn issue_web_url(integration: &Integration, project: &str, number: u64) -> Option<String> {
    let base = web_base(integration)?;
    (spec_for(integration.kind.family()).issue_web_url)(&base, project, &number.to_string())
}

/// The page on the host that mints a token with the scopes the app
/// needs, pre-filled. `None` for kinds without a token flow, or when no
/// host is known yet (a self-managed profile still being typed).
pub fn token_create_url(integration: &Integration) -> Option<String> {
    let base = web_base(integration)?;
    integration.token_link_url(&base)
}

#[cfg(all(test, feature = "github", feature = "gitlab"))]
mod tests {
    use super::*;
    use crate::http::RecordedTransport;
    use crate::integration::IntegrationKind;
    use std::sync::Arc;

    fn integration(kind: IntegrationKind, url: &str) -> Integration {
        Integration::new("h", kind, url)
    }

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn header<'a>(request: &'a HttpRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    const GITHUB_PAGE: &str = r#"[
        {"full_name":"me/one","name":"one","description":"first",
         "default_branch":"main","ssh_url":"git@github.com:me/one.git",
         "clone_url":"https://github.com/me/one.git",
         "html_url":"https://github.com/me/one","private":true}
    ]"#;

    const GITHUB_ORGS_PAGE: &str = r#"[
        {"full_name":"me/secret","name":"secret","private":true,
         "clone_url":"https://github.com/me/secret.git"},
        {"full_name":"acme/site","name":"site","private":false,
         "clone_url":"https://github.com/acme/site.git"},
        {"full_name":"acme/internal","name":"internal","private":true,
         "clone_url":"https://github.com/acme/internal.git"}
    ]"#;

    const GITLAB_GROUPS_PAGE: &str = r#"[
        {"id":1,"name":"mine","path_with_namespace":"kai/mine","visibility":"private"},
        {"id":2,"name":"open","path_with_namespace":"corp/tools/open","visibility":"public"},
        {"id":3,"name":"shared","path_with_namespace":"corp/tools/shared","visibility":"internal"}
    ]"#;

    const GITLAB_PAGE: &str = r#"[
        {"id":42,"name":"one","path_with_namespace":"group/sub/one",
         "description":"first","default_branch":"main",
         "ssh_url_to_repo":"git@gitlab.example.com:group/sub/one.git",
         "http_url_to_repo":"https://gitlab.example.com/group/sub/one.git",
         "web_url":"https://gitlab.example.com/group/sub/one",
         "visibility":"private"}
    ]"#;

    #[test]
    fn the_default_issue_create_options_are_empty_and_new_issue_has_no_type() {
        let transport = std::sync::Arc::new(RecordedTransport::new());
        let gh = Integration::new("gh", IntegrationKind::Github, "https://github.com");
        let client = client_for(
            gh,
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let options = block_on(client.issue_create_options("me/thing")).unwrap();
        assert_eq!(options, IssueCreateOptions::default());
        let new = NewIssue::default();
        assert!(new.issue_type.is_none() && new.list_id.is_none());
    }

    #[test]
    fn github_repositories_normalise_into_hosted_projects() {
        let transport = Arc::new(RecordedTransport::new().on("/user/repos", 200, GITHUB_PAGE));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let projects = block_on(client.list_projects()).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].full_name, "me/one");
        assert_eq!(projects[0].id, "me/one");
        assert_eq!(projects[0].integration_id, "h");
        assert!(projects[0].private);
        assert_eq!(
            projects[0].clone_url(true),
            Some("git@github.com:me/one.git")
        );

        let request = &transport.requests()[0];
        assert!(request.url.starts_with("https://api.github.com/user/repos"));
        assert_eq!(header(request, "Authorization"), Some("Bearer tok"));
        assert!(
            request.url.contains("affiliation="),
            "the token's own repositories, not a public user listing: {}",
            request.url
        );
    }

    #[test]
    fn gitlab_projects_normalise_into_hosted_projects() {
        let transport = Arc::new(RecordedTransport::new().on("/projects?", 200, GITLAB_PAGE));
        let client = client_for(
            integration(IntegrationKind::Gitlab, "https://gitlab.example.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let projects = block_on(client.list_projects()).unwrap();
        assert_eq!(projects[0].full_name, "group/sub/one");
        assert_eq!(projects[0].id, "42");
        assert!(projects[0].private);

        let request = &transport.requests()[0];
        assert!(
            request
                .url
                .starts_with("https://gitlab.example.com/api/v4/projects"),
            "a self-managed instance's api lives under its own url: {}",
            request.url
        );
        assert_eq!(header(request, "PRIVATE-TOKEN"), Some("tok"));
        assert!(
            request.url.contains("membership=true"),
            "without membership=true this lists every public project on the instance: {}",
            request.url
        );
    }

    #[test]
    fn github_listing_carries_owner_and_private_flag_for_user_and_org_repos() {
        let transport = Arc::new(RecordedTransport::new().on("/user/repos", 200, GITHUB_ORGS_PAGE));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let projects = block_on(client.list_projects()).unwrap();
        let seen: Vec<(&str, bool)> = projects.iter().map(|p| (p.owner(), p.private)).collect();
        assert_eq!(seen, vec![("me", true), ("acme", false), ("acme", true)]);
        assert!(
            transport.requests()[0]
                .url
                .contains("affiliation=owner,collaborator,organization_member"),
            "organisation repositories are part of the listing"
        );
    }

    #[test]
    fn gitlab_listing_carries_the_group_path_and_visibility() {
        let transport =
            Arc::new(RecordedTransport::new().on("/projects?", 200, GITLAB_GROUPS_PAGE));
        let client = client_for(
            integration(IntegrationKind::Gitlab, "https://gitlab.example.com"),
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let projects = block_on(client.list_projects()).unwrap();
        let seen: Vec<(&str, bool)> = projects.iter().map(|p| (p.owner(), p.private)).collect();
        assert_eq!(
            seen,
            vec![("kai", true), ("corp/tools", false), ("corp/tools", true)]
        );
    }

    /// The pagination loop has to ask for the next page while a page is
    /// full, and stop on the first short one.
    #[test]
    fn a_full_page_is_followed_by_another_request() {
        let full: String = format!(
            "[{}]",
            (0..PAGE_SIZE)
                .map(|n| format!(
                    r#"{{"id":{n},"name":"p{n}","path_with_namespace":"g/p{n}","visibility":"public"}}"#
                ))
                .collect::<Vec<_>>()
                .join(",")
        );
        let transport = Arc::new(RecordedTransport::new().on("&page=1", 200, &full).on(
            "&page=2",
            200,
            GITLAB_PAGE,
        ));
        let client = client_for(
            integration(IntegrationKind::Gitlab, "https://gitlab.example.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let projects = block_on(client.list_projects()).unwrap();
        assert_eq!(projects.len(), PAGE_SIZE as usize + 1);
        assert_eq!(transport.requests().len(), 2, "must stop on a short page");
    }

    /// A rejected token must say so in words the user can act on, not
    /// as a bare status code - and must not be reported as an empty
    /// project list.
    #[test]
    fn a_rejected_token_is_an_unauthorized_error_not_an_empty_list() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/user/repos",
            401,
            r#"{"message":"Bad credentials"}"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "stale".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let err = block_on(client.list_projects()).unwrap_err();
        assert!(matches!(err, crate::Error::Unauthorized { .. }));
        assert!(err.to_string().contains("Bad credentials"));
    }

    #[test]
    fn uploading_a_key_sends_the_hosts_own_field_names() {
        for (kind, url, fragment) in [
            (
                IntegrationKind::Github,
                "https://github.com",
                "api.github.com",
            ),
            (
                IntegrationKind::Gitlab,
                "https://gitlab.example.com",
                "gitlab.example.com/api/v4",
            ),
        ] {
            let transport = Arc::new(RecordedTransport::new().on(
                "/user/keys",
                201,
                r#"{"id":7,"title":"laptop","key":"ssh-ed25519 AAAA laptop"}"#,
            ));
            let client = client_for(
                integration(kind, url),
                "tok".into(),
                transport.clone(),
                &crate::trust::TrustGate::for_tests(),
            )
            .unwrap();

            let key =
                block_on(client.upload_ssh_key("laptop", "ssh-ed25519 AAAA laptop\n")).unwrap();
            assert_eq!(key.id, "7");
            assert_eq!(key.title, "laptop");

            let request = &transport.requests()[0];
            assert!(request.url.contains(fragment), "{}", request.url);
            let body: serde_json::Value =
                serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
            assert_eq!(body["title"], "laptop");
            assert_eq!(
                body["key"], "ssh-ed25519 AAAA laptop",
                "the trailing newline must be trimmed - both hosts reject it"
            );
        }
    }

    fn new_pr(project: &str) -> NewPullRequest {
        NewPullRequest {
            project: project.into(),
            source_branch: "feature".into(),
            target_branch: "main".into(),
            title: "Add feature".into(),
            body: "does the thing".into(),
            draft: false,
            source_project: None,
            ..Default::default()
        }
    }

    #[test]
    fn github_pull_request_sends_head_base_and_returns_html_url() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/pulls",
            201,
            r#"{"number":42,"html_url":"https://github.com/me/thing/pull/42"}"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let created = block_on(client.create_pull_request(&new_pr("me/thing"))).unwrap();
        assert_eq!(created.number, 42);
        assert_eq!(created.web_url, "https://github.com/me/thing/pull/42");

        let request = &transport.requests()[0];
        assert!(
            request.url.ends_with("/repos/me/thing/pulls"),
            "{}",
            request.url
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["head"], "feature");
        assert_eq!(body["base"], "main");
        assert_eq!(body["title"], "Add feature");
        assert_eq!(body["draft"], false);
    }

    #[test]
    fn gitlab_merge_request_url_encodes_the_project_path() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/merge_requests",
            201,
            r#"{"iid":7,"web_url":"https://gitlab.example.com/group/sub/proj/-/merge_requests/7"}"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Gitlab, "https://gitlab.example.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let mut req = new_pr("group/sub/proj");
        req.draft = true;
        let created = block_on(client.create_pull_request(&req)).unwrap();
        assert_eq!(created.number, 7);
        assert_eq!(
            created.web_url,
            "https://gitlab.example.com/group/sub/proj/-/merge_requests/7"
        );

        let request = &transport.requests()[0];
        assert!(
            request
                .url
                .contains("/projects/group%2Fsub%2Fproj/merge_requests"),
            "{}",
            request.url
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["source_branch"], "feature");
        assert_eq!(body["target_branch"], "main");
        assert_eq!(
            body["title"], "Draft: Add feature",
            "GitLab marks drafts by title prefix"
        );
    }

    #[test]
    fn a_fork_pull_request_names_the_fork_owner_on_github() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/pulls",
            201,
            r#"{"number":3,"html_url":"https://github.com/up/thing/pull/3"}"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let mut req = new_pr("up/thing");
        req.source_project = Some("me/thing".into());
        block_on(client.create_pull_request(&req)).unwrap();
        let request = &transport.requests()[0];
        assert!(
            request.url.ends_with("/repos/up/thing/pulls"),
            "{}",
            request.url
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["head"], "me:feature");
        // Same project spelled as the source is not a fork.
        let mut same = new_pr("up/thing");
        same.source_project = Some("UP/thing".into());
        assert_eq!(same.fork_source(), None);
    }

    #[test]
    fn a_fork_merge_request_is_posted_from_the_source_project_with_the_target_id() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on(
                    "/merge_requests",
                    201,
                    r#"{"iid":9,"web_url":"https://g.example.com/up/thing/-/merge_requests/9"}"#,
                )
                .on("/projects/up%2Fthing", 200, r#"{"id":77}"#),
        );
        let client = client_for(
            integration(IntegrationKind::Gitlab, "https://g.example.com"),
            "tok".into(),
            transport.clone(),
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let mut req = new_pr("up/thing");
        req.source_project = Some("me/thing".into());
        let created = block_on(client.create_pull_request(&req)).unwrap();
        assert_eq!(created.number, 9);
        let requests = transport.requests();
        let post = requests.last().unwrap();
        assert!(
            post.url.contains("/projects/me%2Fthing/merge_requests"),
            "{}",
            post.url
        );
        let body: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["target_project_id"], 77);
    }

    /// A duplicate PR (git already has one open for this branch pair) is
    /// GitHub's classic 422; the host's own message must survive into the
    /// error, not a bare status code.
    #[test]
    fn an_existing_pull_request_surfaces_the_hosts_message() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/pulls",
            422,
            r#"{"message":"Validation Failed","errors":[{"message":"A pull request already exists for me:feature."}]}"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();

        let err = block_on(client.create_pull_request(&new_pr("me/thing"))).unwrap_err();
        assert!(matches!(err, crate::Error::Api { status: 422, .. }));
        assert!(err.to_string().contains("Validation Failed"), "{err}");
    }

    /// 401/403 on create must map to the same `Unauthorized`/`Forbidden`
    /// errors every other call already uses - one vocabulary for a
    /// rejected or under-scoped token, not a special case per endpoint.
    #[test]
    fn create_pull_request_401_and_403_map_to_the_existing_errors() {
        for (status, want_unauthorized) in [(401, true), (403, false)] {
            let transport =
                Arc::new(RecordedTransport::new().on("/pulls", status, r#"{"message":"nope"}"#));
            let client = client_for(
                integration(IntegrationKind::Github, "https://github.com"),
                "tok".into(),
                transport,
                &crate::trust::TrustGate::for_tests(),
            )
            .unwrap();
            let err = block_on(client.create_pull_request(&new_pr("me/thing"))).unwrap_err();
            if want_unauthorized {
                assert!(matches!(err, crate::Error::Unauthorized { .. }));
            } else {
                assert!(matches!(err, crate::Error::Forbidden { .. }));
            }
        }
    }

    #[test]
    fn existing_keys_can_be_listed_so_a_duplicate_upload_is_avoidable() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/user/keys",
            200,
            r#"[{"id":7,"title":"laptop","key":"ssh-ed25519 AAAA laptop"}]"#,
        ));
        let client = client_for(
            integration(IntegrationKind::Github, "https://github.com"),
            "tok".into(),
            transport,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap();
        let keys = block_on(client.list_ssh_keys()).unwrap();
        assert_eq!(keys[0].title, "laptop");
    }

    /// A client that implements only the original four operations gets
    /// `Unsupported` for the new ones by default.
    #[test]
    fn new_operations_default_to_unsupported() {
        struct Old(Integration);
        #[async_trait::async_trait]
        impl HostingClient for Old {
            fn integration(&self) -> &Integration {
                &self.0
            }
            async fn list_projects(&self) -> Result<Vec<HostedProject>> {
                Ok(vec![])
            }
            async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
                Ok(vec![])
            }
            async fn upload_ssh_key(&self, _: &str, _: &str) -> Result<HostedSshKey> {
                unreachable!()
            }
            async fn create_pull_request(&self, _: &NewPullRequest) -> Result<CreatedPullRequest> {
                unreachable!()
            }
        }
        let old = Old(integration(IntegrationKind::Github, "https://github.com"));
        assert!(matches!(
            block_on(old.current_user()).unwrap_err(),
            Error::Unsupported(_)
        ));
        assert!(matches!(
            block_on(old.delete_ssh_key("1")).unwrap_err(),
            Error::Unsupported(_)
        ));
        assert!(matches!(
            block_on(old.pull_request_for_branch("a/b", "x")).unwrap_err(),
            Error::Unsupported(_)
        ));
    }

    #[test]
    fn pull_request_heads_live_in_the_hosts_special_refs() {
        assert_eq!(
            pull_request_head_ref(IntegrationKind::Github, 7),
            Some(("refs/pull/7/head".into(), "pr/7".into()))
        );
        assert_eq!(
            pull_request_head_ref(IntegrationKind::GitlabSelfManaged, 7),
            Some(("refs/merge-requests/7/head".into(), "mr/7".into()))
        );
    }

    #[test]
    fn web_url_helpers_per_family() {
        let gh = integration(IntegrationKind::Github, "https://github.com/");
        assert_eq!(
            pull_request_web_url(&gh, "me/thing", 12).as_deref(),
            Some("https://github.com/me/thing/pull/12")
        );
        assert_eq!(
            issue_web_url(&gh, "me/thing", 3).as_deref(),
            Some("https://github.com/me/thing/issues/3")
        );
        let ghe = integration(IntegrationKind::GithubEnterprise, "https://ghe.example.com");
        assert_eq!(
            pull_request_web_url(&ghe, "o/r", 1).as_deref(),
            Some("https://ghe.example.com/o/r/pull/1")
        );
        let gl = integration(
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        );
        assert_eq!(
            pull_request_web_url(&gl, "g/sub/p", 53).as_deref(),
            Some("https://gitlab.example.com/g/sub/p/-/merge_requests/53")
        );
        assert_eq!(
            issue_web_url(&gl, "g/sub/p", 12).as_deref(),
            Some("https://gitlab.example.com/g/sub/p/-/issues/12")
        );
        let jira = integration(IntegrationKind::JiraCloud, "https://x.atlassian.net");
        assert_eq!(pull_request_web_url(&jira, "p", 1), None);
    }

    #[test]
    fn the_token_create_url_needs_a_host_and_a_token_flow() {
        assert_eq!(
            token_create_url(&integration(IntegrationKind::Github, "")).as_deref(),
            Some("https://github.com/settings/tokens/new?description=GitPoulpi&scopes=repo,read:user,admin:public_key")
        );
        assert_eq!(
            token_create_url(&integration(
                IntegrationKind::GitlabSelfManaged,
                "https://gitlab.example.com/"
            ))
            .as_deref(),
            Some("https://gitlab.example.com/-/user_settings/personal_access_tokens?name=GitPoulpi&scopes=api,read_user")
        );
        assert_eq!(
            token_create_url(&integration(IntegrationKind::GitlabSelfManaged, "")),
            None
        );
        assert_eq!(
            token_create_url(&integration(IntegrationKind::Trello, "")),
            None
        );
    }
}
