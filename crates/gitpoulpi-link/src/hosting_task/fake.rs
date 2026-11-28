//! A `HostingClient` that answers from memory, for this module's and
//! `hosting_ui`'s tests.

use super::TokenMemo;
use super::*;
use gitpoulpi_hosting::{
    CreatedPullRequest, HostedProject, HostedSshKey, NewPullRequest, PullRequestState,
};
use std::sync::Mutex;

pub fn pr(number: u64, branch: &str, state: PullRequestState) -> PullRequestSummary {
    PullRequestSummary {
        number,
        iid_label: format!("!{number}"),
        title: format!("Change number {number}"),
        state,
        source_branch: branch.to_string(),
        target_branch: "main".into(),
        source_project: None,
        author: "kai".into(),
        web_url: format!("https://gitlab.example.com/g/p/-/merge_requests/{number}"),
        updated_at: "2026-09-29T10:00:00Z".into(),
        review_requested: false,
        assignees: Vec::new(),
        reviewers: Vec::new(),
        checks: None,
        has_conflicts: None,
    }
}

pub fn issue(number: u64) -> IssueSummary {
    IssueSummary {
        number,
        id: number.to_string(),
        reference: format!("#{number}"),
        iid_label: format!("#{number}"),
        title: format!("Fix the thing {number}"),
        state: gitpoulpi_hosting::IssueState::Open,
        author: "kai".into(),
        assignees: Vec::new(),
        labels: Vec::new(),
        web_url: format!("https://gitlab.example.com/g/p/-/issues/{number}"),
        updated_at: "2026-09-29T10:00:00Z".into(),
        ..IssueSummary::default()
    }
}

pub fn project(full_name: &str) -> HostedProject {
    HostedProject {
        integration_id: "corp".into(),
        id: full_name.into(),
        name: full_name.rsplit('/').next().unwrap_or(full_name).into(),
        full_name: full_name.into(),
        description: None,
        default_branch: Some("main".into()),
        ssh_url: Some(format!("git@gitlab.example.com:{full_name}.git")),
        https_url: Some(format!("https://gitlab.example.com/{full_name}.git")),
        web_url: None,
        private: false,
    }
}

pub struct FakeClient {
    pub projects: Vec<HostedProject>,
    pub integration: Integration,
    pub prs: Vec<PullRequestSummary>,
    pub issues: Vec<IssueSummary>,
    pub head: Option<PullRequestSummary>,
    pub fail: Option<fn() -> HostingError>,
    pub seen: Arc<Mutex<Vec<String>>>,
    pub mentioned_prs: Vec<PullRequestSummary>,
    pub mentioned_issues: Vec<IssueSummary>,
    pub wire: Wire,
    /// What `account_items` answers, and how it fails.
    pub account: gitpoulpi_hosting::account::AccountItems,
    pub account_fail: Option<fn() -> HostingError>,
}

/// The cost of the network, for the Radar's timing tests: every list call
/// waits `latency` (a timer, so calls joined on one runtime overlap) and
/// `live` / `peak` count the calls in flight.
#[derive(Clone, Default)]
pub struct Wire {
    pub latency: std::time::Duration,
    pub live: Arc<std::sync::atomic::AtomicUsize>,
    pub peak: Arc<std::sync::atomic::AtomicUsize>,
}

impl Wire {
    async fn call(&self) {
        use std::sync::atomic::Ordering::SeqCst;
        let now = self.live.fetch_add(1, SeqCst) + 1;
        self.peak.fetch_max(now, SeqCst);
        if !self.latency.is_zero() {
            tokio::time::sleep(self.latency).await;
        }
        self.live.fetch_sub(1, SeqCst);
    }
}

#[async_trait::async_trait]
impl HostingClient for FakeClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }
    async fn list_projects(&self) -> gitpoulpi_hosting::Result<Vec<HostedProject>> {
        if let Some(fail) = self.fail {
            return Err(fail());
        }
        Ok(self.projects.clone())
    }
    async fn list_ssh_keys(&self) -> gitpoulpi_hosting::Result<Vec<HostedSshKey>> {
        Ok(Vec::new())
    }
    async fn upload_ssh_key(&self, _: &str, _: &str) -> gitpoulpi_hosting::Result<HostedSshKey> {
        Err(HostingError::Unsupported("upload".into()))
    }
    async fn create_pull_request(
        &self,
        _: &NewPullRequest,
    ) -> gitpoulpi_hosting::Result<CreatedPullRequest> {
        Err(HostingError::Unsupported("create".into()))
    }
    async fn list_pull_requests(
        &self,
        project: &str,
        filter: &PrFilter,
    ) -> gitpoulpi_hosting::Result<Vec<PullRequestSummary>> {
        self.seen
            .lock()
            .unwrap()
            .push(format!("prs {project} {filter:?}"));
        self.wire.call().await;
        if let Some(fail) = self.fail {
            return Err(fail());
        }
        if filter.mentioned {
            return Ok(self.mentioned_prs.clone());
        }
        // A host that fills the checks only when asked (GitHub).
        if filter.checks {
            return Ok(self
                .prs
                .iter()
                .cloned()
                .map(|mut p| {
                    p.checks = Some(gitpoulpi_hosting::ChecksState::Passing);
                    p.has_conflicts = Some(false);
                    p
                })
                .collect());
        }
        Ok(self.prs.clone())
    }
    async fn pull_request_for_branch(
        &self,
        project: &str,
        branch: &str,
    ) -> gitpoulpi_hosting::Result<Option<PullRequestSummary>> {
        self.seen
            .lock()
            .unwrap()
            .push(format!("head {project} {branch}"));
        Ok(self.head.clone())
    }
    async fn list_issues(
        &self,
        project: &str,
        filter: &IssueFilter,
    ) -> gitpoulpi_hosting::Result<Vec<IssueSummary>> {
        self.seen
            .lock()
            .unwrap()
            .push(format!("issues {project} {filter:?}"));
        self.wire.call().await;
        if filter.mentioned {
            return Ok(self.mentioned_issues.clone());
        }
        Ok(self.issues.clone())
    }
    async fn account_items(
        &self,
        query: &gitpoulpi_hosting::account::AccountQuery,
    ) -> gitpoulpi_hosting::Result<gitpoulpi_hosting::account::AccountItems> {
        self.seen.lock().unwrap().push(format!(
            "account {} r{} o{} i{} d{}",
            self.integration.id,
            query.review_requests as u8,
            query.open_requests as u8,
            query.assigned_issues as u8,
            query.include_drafts as u8
        ));
        self.wire.call().await;
        if let Some(fail) = self.account_fail {
            return Err(fail());
        }
        Ok(self.account.clone())
    }
}

/// Hands out a [`FakeClient`] per call, built from a template.
pub struct FakeFactory {
    pub projects: Vec<HostedProject>,
    pub prs: Vec<PullRequestSummary>,
    pub issues: Vec<IssueSummary>,
    pub head: Option<PullRequestSummary>,
    pub fail: Option<fn() -> HostingError>,
    pub no_token: bool,
    pub seen: Arc<Mutex<Vec<String>>>,
    pub mentioned_prs: Vec<PullRequestSummary>,
    pub mentioned_issues: Vec<IssueSummary>,
    pub wire: Wire,
    pub account: gitpoulpi_hosting::account::AccountItems,
    pub account_fail: Option<fn() -> HostingError>,
    /// What building a client costs (production: a credential-helper
    /// subprocess to read the token), and how many were built.
    pub client_cost: std::time::Duration,
    pub built: Arc<std::sync::atomic::AtomicUsize>,
    /// Read the token once per round, as the Radar's factory does: the
    /// cost is paid inside the memo.
    pub memoize: bool,
    pub tokens: TokenMemo,
    pub rounds: Arc<std::sync::atomic::AtomicUsize>,
}

#[allow(clippy::new_without_default)]
impl FakeFactory {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            prs: Vec::new(),
            issues: Vec::new(),
            head: None,
            fail: None,
            no_token: false,
            seen: Arc::new(Mutex::new(Vec::new())),
            mentioned_prs: Vec::new(),
            mentioned_issues: Vec::new(),
            wire: Wire::default(),
            account: Default::default(),
            account_fail: None,
            client_cost: std::time::Duration::ZERO,
            built: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            memoize: false,
            tokens: TokenMemo::default(),
            rounds: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

impl ClientFactory for FakeFactory {
    fn client(
        &self,
        integration: &Integration,
        _: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        if self.no_token {
            return Err(HostingError::NoToken {
                id: integration.id.clone(),
                token_ref: integration.token_ref.clone(),
                env_var: "X".into(),
                host: "h".into(),
            });
        }
        let read_token = || {
            self.built.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if !self.client_cost.is_zero() {
                std::thread::sleep(self.client_cost);
            }
            Ok(String::new())
        };
        if self.memoize {
            self.tokens.get(&integration.id, read_token)?;
        } else {
            read_token()?;
        }
        Ok(Box::new(FakeClient {
            projects: self.projects.clone(),
            integration: integration.clone(),
            prs: self.prs.clone(),
            issues: self.issues.clone(),
            head: self.head.clone(),
            fail: self.fail,
            seen: Arc::clone(&self.seen),
            mentioned_prs: self.mentioned_prs.clone(),
            mentioned_issues: self.mentioned_issues.clone(),
            wire: self.wire.clone(),
            account: self.account.clone(),
            account_fail: self.account_fail,
        }))
    }

    fn new_round(&self) {
        self.rounds
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.tokens.clear();
    }
}
