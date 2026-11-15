//! The hosting worker: pull/merge requests and issues for the repository
//! of this window, fetched off the input thread.
//!
//! Same shape as `clone_task`: a plain `std::thread` owning a small
//! `current_thread` tokio runtime, fed over an `std::sync::mpsc` channel
//! and answering on another that the render loop drains with `try_recv`.
//! Hosting stays client-side (no daemon, no IPC): what a host says about a
//! repository is not state the other frontends must agree on, and the
//! token lookup (a credential-helper subprocess) must never run on the
//! input thread.
//!
//! Three pieces, each testable without a terminal or a network:
//!
//! - [`fetch_snapshot`]: resolve the profile for the repository, ask the
//!   host, and fold every failure into a typed [`Problem`] the sidebar can
//!   render ("Reconnect", "offline, 5m old").
//! - [`RefreshPolicy`]: *when* to ask (tab open, HEAD or remote-tracking
//!   change, `general.hosting_refresh_minutes`, on demand) -- pure, driven
//!   by an explicit clock.
//! - [`HostingWorker`]: the thread and its channels. Requests that pile up
//!   while a fetch runs are coalesced to the newest.

use crate::redraw::NotifySend as _;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_config::Integration;
use gitpoulpi_config::IntegrationKind;
use gitpoulpi_core::domain::{Ref, RefKind};
use gitpoulpi_hosting::{
    suggest_for_repo, Error as HostingError, HostingClient, IssueFilter, IssueSummary, PrFilter,
    PullRequestSummary, ResolveRemote, StateFilter,
};
use gitpoulpi_i18n::msg;
use serde::{Deserialize, Serialize};

/// Which pull/merge requests the sidebar lists: the state filter of the
/// PULL REQUESTS combobox. Whose they are ("My", "Assigned to me",
/// "Awaiting my review") is the sidebar's four groups, sorted client-side.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PrView {
    // Older caches said `mine` / `review_requested`: those are groups now.
    #[default]
    #[serde(alias = "mine", alias = "review_requested")]
    Open,
    /// Merged and closed ones.
    Closed,
    All,
}

impl PrView {
    pub(crate) const ALL: [PrView; 3] = [PrView::Open, PrView::Closed, PrView::All];

    pub(crate) fn label(self) -> &'static str {
        match self {
            PrView::Open => msg::hosting_task::VIEW_OPEN.get(),
            PrView::Closed => msg::hosting_task::VIEW_MERGED_CLOSED.get(),
            PrView::All => msg::hosting_task::VIEW_ALL.get(),
        }
    }

    pub(crate) fn filter(self) -> PrFilter {
        PrFilter {
            state: match self {
                PrView::Open => StateFilter::Open,
                PrView::Closed => StateFilter::Closed,
                PrView::All => StateFilter::All,
            },
            ..PrFilter::default()
        }
    }
}

/// Which issues the sidebar list shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IssueView {
    #[default]
    Open,
    Assigned,
    Closed,
    All,
}

impl IssueView {
    pub(crate) const ALL: [IssueView; 4] = [
        IssueView::Open,
        IssueView::Assigned,
        IssueView::Closed,
        IssueView::All,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            IssueView::Open => msg::hosting_task::VIEW_ISSUE_OPEN.get(),
            IssueView::Assigned => msg::hosting_task::VIEW_ASSIGNED.get(),
            IssueView::Closed => msg::hosting_task::VIEW_CLOSED.get(),
            IssueView::All => msg::hosting_task::VIEW_ISSUE_ALL.get(),
        }
    }

    pub(crate) fn filter(self) -> IssueFilter {
        match self {
            IssueView::Open => IssueFilter::default(),
            IssueView::Assigned => IssueFilter {
                assigned_to_me: true,
                ..IssueFilter::default()
            },
            IssueView::Closed => IssueFilter {
                state: StateFilter::Closed,
                ..IssueFilter::default()
            },
            IssueView::All => IssueFilter {
                state: StateFilter::All,
                ..IssueFilter::default()
            },
        }
    }
}

/// Why the last fetch did not (fully) work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Problem {
    /// The profile has no usable token.
    NoToken,
    /// The host rejected the token (401) or the token lacks a scope (403):
    /// both are fixed the same way, by reconnecting the profile.
    Reconnect(String),
    /// The host could not be reached.
    Offline(String),
    /// The host said to slow down (or we are already backing off): the
    /// lists stay as they were and the next refresh waits for the reset.
    RateLimited(String),
    /// Anything else the host said.
    Other(String),
}

impl Problem {
    pub(crate) fn from_error(err: &HostingError) -> Self {
        match err {
            HostingError::NoToken { .. } => Problem::NoToken,
            HostingError::Unauthorized { .. } | HostingError::Forbidden { .. } => {
                Problem::Reconnect(error_text(err))
            }
            HostingError::Http(_) => Problem::Offline(error_text(err)),
            HostingError::RateLimited { .. } => Problem::RateLimited(error_text(err)),
            other => Problem::Other(error_text(other)),
        }
    }

    /// The problems a reconnect fixes.
    pub(crate) fn wants_reconnect(&self) -> bool {
        matches!(self, Problem::NoToken | Problem::Reconnect(_))
    }
}

/// "could not start the request task", for the worker threads' runtimes.
pub(crate) fn request_task_failed(err: impl std::fmt::Display) -> String {
    msg::hosting_task::request_task_failed(err)
}

/// A hosting error as the user reads it, in the display language. The host's
/// own `detail` text stays as the host wrote it. Anything without a message of
/// its own keeps the (English) text of the shared hosting crate.
pub(crate) fn error_text(err: &HostingError) -> String {
    let text = error_words(err);
    // The text goes through string channels: keep how it reads for the
    // failure card (a rejected token offers Open Integrations).
    let kind = match err {
        HostingError::NoToken { .. }
        | HostingError::Unauthorized { .. }
        | HostingError::Forbidden { .. } => Some(gitpoulpi_core::FailureKind::AuthFailed),
        HostingError::Http(_) => Some(gitpoulpi_core::FailureKind::NetworkUnreachable),
        _ => None,
    };
    if let Some(kind) = kind {
        crate::activity_log::remember_kind(&text, kind);
    }
    text
}

fn error_words(err: &HostingError) -> String {
    match err {
        HostingError::Unauthorized { detail, .. } => msg::hosting_task::err_unauthorized(detail),
        HostingError::Forbidden { detail, .. } => msg::hosting_task::err_forbidden(detail),
        HostingError::Api {
            status: 404,
            detail,
            ..
        } => msg::hosting_task::err_not_found(detail),
        HostingError::Api {
            status: 422,
            detail,
            ..
        } => msg::hosting_task::err_rejected(detail),
        HostingError::Api { status, detail, .. } => msg::hosting_task::err_http(status, detail),
        HostingError::RateLimited {
            host,
            retry_in_secs,
        } => msg::hosting_task::err_rate_limited(host, wait_text(*retry_in_secs)),
        HostingError::Http(detail) => msg::hosting_task::err_offline(detail),
        HostingError::Decode(detail) => msg::hosting_task::err_decode(detail),
        other => other.to_string(),
    }
}

/// `30s`, `5m`, `2h`: how long a rate limit lasts.
fn wait_text(secs: u64) -> String {
    match secs {
        0..=59 => msg::hosting_task::dur_seconds(secs),
        60..=3599 => msg::hosting_task::dur_minutes(secs.div_ceil(60)),
        _ => msg::hosting_task::dur_hours(secs.div_ceil(3600)),
    }
}

/// What the worker needs to answer for one repository.
#[derive(Debug, Clone)]
pub(crate) struct HostingRequest {
    pub(crate) remotes: Vec<ResolveRemote>,
    pub(crate) integrations: Vec<Integration>,
    /// `RepoState::hosting_integration`.
    pub(crate) override_id: Option<String>,
    /// The checked-out local branch, for the linked-PR chip.
    pub(crate) head_branch: Option<String>,
    pub(crate) pr_view: PrView,
    pub(crate) issue_view: IssueView,
    /// `RepoState::issues_source`: the provider and project the ISSUES
    /// section was pointed at, if the user chose one.
    pub(crate) issues_source: Option<gitpoulpi_session::IssuesSource>,
    /// `RepoState::pulls_source`: the same choice for PULL REQUESTS. With
    /// neither this nor `override_id` the section is unlinked.
    pub(crate) pulls_source: Option<gitpoulpi_session::IssuesSource>,
    /// The repository's folder: its stored hosting settings (a token command
    /// for the host calls, blocked assignment) apply to its project. Empty
    /// when the request is not about one repository.
    pub(crate) repo_root: PathBuf,
}

/// A remote's proposal to link the repository (never applied on its own).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SuggestionRef {
    pub(crate) profile: ProfileRef,
    pub(crate) remote: String,
    pub(crate) project: String,
}

/// A profile as the sidebar's dropdowns name it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProfileRef {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) kind: IntegrationKind,
}

/// Where the ISSUES section reads from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IssuesTarget {
    pub(crate) profile: ProfileRef,
    /// The provider's repository/project; empty until one is chosen.
    pub(crate) project: String,
    /// Chosen by the user (kept in the repo state), not derived from the
    /// remote.
    pub(crate) chosen: bool,
}

/// The worker's answer, and what is cached on disk between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct HostingSnapshot {
    /// The profile that answered; `None` when no profile matches the
    /// repository's remotes (then nothing is shown at all).
    pub(crate) profile_id: Option<String>,
    pub(crate) profile_label: String,
    pub(crate) kind: Option<IntegrationKind>,
    /// The remote the project path was read from.
    pub(crate) remote: String,
    pub(crate) project: String,
    pub(crate) can_pulls: bool,
    pub(crate) can_issues: bool,
    pub(crate) prs: Vec<PullRequestSummary>,
    pub(crate) issues: Vec<IssueSummary>,
    /// The open requests that mention the signed-in user, and the issues
    /// (the Radar's "Mentioning me" group); empty unless it asked.
    #[serde(default)]
    pub(crate) prs_mentioned: Vec<PullRequestSummary>,
    #[serde(default)]
    pub(crate) issues_mentioned: Vec<IssueSummary>,
    /// The request that represents `head_branch`.
    pub(crate) head_pr: Option<PullRequestSummary>,
    pub(crate) head_branch: Option<String>,
    pub(crate) pr_view: PrView,
    pub(crate) issue_view: IssueView,
    /// The signed-in user's login on the linked profile (for the "My" and
    /// "Assigned to me" groups); empty when unknown.
    #[serde(default)]
    pub(crate) me: String,
    /// Every profile an issue tracker can be picked from.
    #[serde(default)]
    pub(crate) issue_profiles: Vec<ProfileRef>,
    #[serde(default)]
    pub(crate) issues_target: Option<IssuesTarget>,
    /// Every profile a pull-request source can be picked from.
    #[serde(default)]
    pub(crate) pull_profiles: Vec<ProfileRef>,
    /// Where the PULL REQUESTS section reads from, once the user picked it;
    /// the project is empty until the repository is chosen.
    #[serde(default)]
    pub(crate) pulls_target: Option<IssuesTarget>,
    /// The providers matching the repository's remotes, `origin` first:
    /// offered in the provider dropdowns, never linked automatically.
    #[serde(default)]
    pub(crate) suggestions: Vec<SuggestionRef>,
    /// Why the issues could not be read (separate from the pull requests':
    /// they may come from another profile).
    #[serde(default)]
    pub(crate) issues_problem: Option<Problem>,
    /// Several profiles matched and none is pinned for this repository.
    pub(crate) ambiguous: bool,
    /// Unix seconds.
    pub(crate) fetched_at: i64,
    pub(crate) problem: Option<Problem>,
}

impl HostingSnapshot {
    pub(crate) fn empty(now: i64, req: &HostingRequest) -> Self {
        Self {
            profile_id: None,
            profile_label: String::new(),
            kind: None,
            remote: String::new(),
            project: String::new(),
            can_pulls: false,
            can_issues: false,
            prs: Vec::new(),
            issues: Vec::new(),
            prs_mentioned: Vec::new(),
            issues_mentioned: Vec::new(),
            head_pr: None,
            head_branch: req.head_branch.clone(),
            pr_view: req.pr_view,
            issue_view: req.issue_view,
            me: String::new(),
            issue_profiles: Vec::new(),
            issues_target: None,
            pull_profiles: Vec::new(),
            pulls_target: None,
            suggestions: Vec::new(),
            issues_problem: None,
            ambiguous: false,
            fetched_at: now,
            problem: None,
        }
    }

    /// True when a profile answers for this repository.
    pub(crate) fn linked(&self) -> bool {
        self.profile_id.is_some()
    }
}

/// Builds a client for a profile. Production reads the token through the
/// credential helper and talks over `reqwest`; tests hand back a fake.
pub(crate) trait ClientFactory: Send + Sync {
    fn client(
        &self,
        integration: &Integration,
        all: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>>;

    /// A new round of fetches begins: whatever the factory kept between
    /// two clients (a token read once) is read again.
    #[cfg_attr(not(feature = "radar"), allow(dead_code))]
    fn new_round(&self) {}
}

/// The real thing.
pub(crate) struct DefaultFactory;

impl ClientFactory for DefaultFactory {
    fn client(
        &self,
        integration: &Integration,
        all: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        build_client(integration, all, read_token(integration, all)?)
    }
}

fn read_token(integration: &Integration, all: &[Integration]) -> gitpoulpi_hosting::Result<String> {
    use gitpoulpi_hosting::TokenSource;
    // The security check comes before the secret is even read: a profile whose
    // address changed never has its token resolved (and the refusal is queued
    // for the dialog).
    gitpoulpi_hosting::TrustGate::ambient().preflight(integration)?;
    gitpoulpi_hosting::DefaultTokenSource::for_integrations(all).token(integration)
}

fn build_client(
    integration: &Integration,
    _all: &[Integration],
    token: String,
) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
    // One shared caching transport: ETags and the rate-limit state
    // must outlive the per-refresh client.
    let transport = gitpoulpi_hosting::shared_network_transport()?;
    gitpoulpi_hosting::client_for(
        integration.clone(),
        token,
        transport,
        &gitpoulpi_hosting::TrustGate::ambient(),
    )
}

/// Tokens read once per round. Reading one is a credential-helper
/// subprocess (tens of milliseconds, far more with a heavy helper), and a
/// Radar round builds up to four clients per repository for the same few
/// profiles: the lookups are made once per profile and round, one at a time.
/// A failed lookup is not remembered.
#[cfg_attr(not(feature = "radar"), allow(dead_code))]
#[derive(Default)]
pub(crate) struct TokenMemo {
    tokens: Mutex<HashMap<String, String>>,
}

#[cfg_attr(not(feature = "radar"), allow(dead_code))]
impl TokenMemo {
    pub(crate) fn get(
        &self,
        id: &str,
        load: impl FnOnce() -> gitpoulpi_hosting::Result<String>,
    ) -> gitpoulpi_hosting::Result<String> {
        let mut tokens = self.tokens.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(token) = tokens.get(id) {
            return Ok(token.clone());
        }
        let token = load()?;
        tokens.insert(id.to_string(), token.clone());
        Ok(token)
    }

    pub(crate) fn clear(&self) {
        self.tokens
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

/// The Radar's factory: [`DefaultFactory`] with the tokens remembered for
/// the length of a round.
#[cfg_attr(not(feature = "radar"), allow(dead_code))]
#[derive(Default)]
pub(crate) struct RadarFactory {
    tokens: TokenMemo,
}

impl ClientFactory for RadarFactory {
    fn client(
        &self,
        integration: &Integration,
        all: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        let token = self
            .tokens
            .get(&integration.id, || read_token(integration, all))?;
        build_client(integration, all, token)
    }

    fn new_round(&self) {
        self.tokens.clear();
    }
}

/// Seconds since the Unix epoch.
pub(crate) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `"just now"`, `"5m"`, `"3h"`, `"2d"` - how stale a cached list is.
pub(crate) fn age_label(now: i64, then: i64) -> String {
    let secs = (now - then).max(0);
    match secs {
        0..=59 => msg::reltime::JUST_NOW.get().to_string(),
        60..=3599 => msg::hosting_task::dur_minutes(secs / 60),
        3600..=86_399 => msg::hosting_task::dur_hours(secs / 3600),
        _ => msg::hosting_task::dur_days(secs / 86_400),
    }
}

/// An issue source that is a folder of notes (relative to the repository)
/// read against `repo_root`, so the worker and every call after it see an
/// absolute path. Other sources pass through.
pub(crate) fn resolve_local_source(
    repo_root: &Path,
    integrations: &[Integration],
    source: Option<gitpoulpi_session::IssuesSource>,
) -> Option<gitpoulpi_session::IssuesSource> {
    let mut source = source?;
    let local = integrations
        .iter()
        .find(|i| i.id == source.integration)
        .is_some_and(|i| i.kind.capabilities().local_folder);
    if local && !source.project.trim().is_empty() {
        source.project = gitpoulpi_hosting::resolve_local_folder(repo_root, &source.project)
            .to_string_lossy()
            .into_owned();
    }
    Some(source)
}

fn profile_label(integration: &Integration) -> String {
    if integration.kind.capabilities().local_folder {
        return integration.display_name().to_string();
    }
    if integration.name.trim().is_empty() {
        integration.host().unwrap_or_else(|| integration.id.clone())
    } else {
        integration.name.clone()
    }
}

/// Resolves the profile for the repository and asks its host. Never
/// panics and never returns an `Err`: every failure is a [`Problem`] on
/// the snapshot, so the caller always has something to show.
///
/// Two independent halves, each pointed by the user: the pull requests
/// come from the provider and repository picked in the PULL REQUESTS
/// section, the issues from the ones picked in ISSUES (both kept per
/// repository). Unpicked means unlinked and nothing is fetched: the
/// remotes only *suggest* providers (`snap.suggestions`), they never link.
pub(crate) fn fetch_snapshot(
    req: &HostingRequest,
    factory: &dyn ClientFactory,
    runtime: &tokio::runtime::Runtime,
    now: i64,
) -> HostingSnapshot {
    let mut snap = HostingSnapshot::empty(now, req);
    snap.issue_profiles = req
        .integrations
        .iter()
        .filter(|i| {
            let caps = i.kind.capabilities();
            i.enabled
                && caps.implemented
                && crate::build_features::kind_built(i.kind)
                && caps.issues
        })
        .map(|i| ProfileRef {
            id: i.id.clone(),
            label: profile_label(i),
            kind: i.kind,
        })
        .collect();
    snap.pull_profiles = req
        .integrations
        .iter()
        .filter(|i| {
            let caps = i.kind.capabilities();
            i.enabled
                && caps.implemented
                && crate::build_features::kind_built(i.kind)
                && caps.pull_requests
        })
        .map(|i| ProfileRef {
            id: i.id.clone(),
            label: profile_label(i),
            kind: i.kind,
        })
        .collect();
    // The remotes only propose; nothing below links on their account.
    let suggestions = suggest_for_repo(&req.remotes, &req.integrations);
    snap.suggestions = suggestions
        .iter()
        .filter_map(|s| {
            let integration = req.integrations.iter().find(|i| i.id == s.integration_id)?;
            Some(SuggestionRef {
                profile: ProfileRef {
                    id: integration.id.clone(),
                    label: profile_label(integration),
                    kind: integration.kind,
                },
                remote: s.remote.clone(),
                project: s.project_path.clone(),
            })
        })
        .collect();
    let first_remote = |integration_id: &str, project: &str| {
        suggestions
            .iter()
            .find(|s| s.integration_id == integration_id && s.project_path == project)
            .or_else(|| {
                suggestions
                    .iter()
                    .find(|s| s.integration_id == integration_id)
            })
            .map(|s| s.remote.clone())
            .or_else(|| req.remotes.first().map(|r| r.name.clone()))
            .unwrap_or_default()
    };

    // Which integration and project the issues come from: the user's pick,
    // and only that.
    let issue_target: Option<(&Integration, IssuesTarget)> =
        req.issues_source.as_ref().and_then(|src| {
            let integration = req
                .integrations
                .iter()
                .find(|i| i.id == src.integration && i.enabled)?;
            let profile = snap
                .issue_profiles
                .iter()
                .find(|p| p.id == integration.id)?;
            Some((
                integration,
                IssuesTarget {
                    profile: profile.clone(),
                    project: src.project.clone(),
                    chosen: true,
                },
            ))
        });

    // The pull requests: the user's pick (or a pinned profile, which is a
    // pick too, with the project its remote names).
    let pulls_choice: Option<(&Integration, String)> = req
        .pulls_source
        .as_ref()
        .map(|src| (src.integration.as_str(), src.project.clone()))
        .or_else(|| {
            let id = req.override_id.as_deref()?;
            let project = suggestions
                .iter()
                .find(|s| s.integration_id == id)
                .map(|s| s.project_path.clone())
                .unwrap_or_default();
            Some((id, project))
        })
        .and_then(|(id, project)| {
            let integration = req.integrations.iter().find(|i| i.id == id && i.enabled)?;
            snap.pull_profiles
                .iter()
                .any(|p| p.id == integration.id)
                .then_some((integration, project))
        });
    let mut pulls_job: Option<PullsJob> = None;
    if let Some((integration, project)) = pulls_choice {
        let caps = integration.kind.capabilities();
        snap.pulls_target = Some(IssuesTarget {
            profile: ProfileRef {
                id: integration.id.clone(),
                label: profile_label(integration),
                kind: integration.kind,
            },
            project: project.clone(),
            chosen: true,
        });
        // Linked once there is a repository to ask about.
        if !project.is_empty() {
            snap.profile_id = Some(integration.id.clone());
            snap.profile_label = profile_label(integration);
            snap.kind = Some(integration.kind);
            snap.remote = first_remote(&integration.id, &project);
            snap.project = project;
            snap.me = integration
                .user
                .as_ref()
                .map(|u| u.login.clone())
                .unwrap_or_default();
            snap.can_pulls = caps.implemented
                && crate::build_features::kind_built(integration.kind)
                && caps.pull_requests;
            snap.can_issues = caps.implemented
                && crate::build_features::kind_built(integration.kind)
                && caps.issues;
            if snap.can_pulls {
                pulls_job = pulls_job_of(&mut snap, req, integration, factory);
            }
        }
    }
    let mut issues_job: Option<IssuesJob> = None;
    if let Some((integration, target)) = issue_target {
        issues_job = issues_job_of(&mut snap, req, integration, &target, factory);
        snap.issues_target = Some(target);
    }
    // The two lists share nothing: ask them together, one round trip of
    // waiting instead of two.
    let (pulls, issues) = runtime.block_on(async {
        tokio::join!(
            async {
                match &pulls_job {
                    Some(job) => Some(pulls_part(job).await),
                    None => None,
                }
            },
            async {
                match &issues_job {
                    Some(job) => Some(issues_part(job).await),
                    None => None,
                }
            }
        )
    });
    if let Some(out) = pulls {
        if let Some(prs) = out.prs {
            snap.prs = prs;
        }
        if let Some(found) = out.head_pr {
            snap.head_pr = found;
        }
        if let Some(me) = out.me {
            snap.me = me;
        }
        snap.problem = out.problem;
    }
    if let Some(out) = issues {
        if let Some(issues) = out.issues {
            snap.issues = issues;
        }
        if out.problem.is_some() {
            snap.issues_problem = out.problem;
        }
    }
    register_scopes(req, &mut snap);
    snap
}

/// Tells the hosting clients which repository the project this snapshot
/// answers for belongs to, so the repository's token command and its
/// assignment block apply to the calls made about it (forms, menus and the
/// CLI-style edits alike).
fn register_scopes(req: &HostingRequest, snap: &mut HostingSnapshot) {
    if req.repo_root.as_os_str().is_empty() {
        return;
    }
    if let Some(id) = &snap.profile_id {
        gitpoulpi_daemon::hosting_scope::register(&req.repo_root, id, &snap.project);
        // The calls are made by an App bot, not by the person: say so.
        if gitpoulpi_hosting::scope::acts_as_bot(id, &snap.project) {
            snap.profile_label = format!(
                "{} ({})",
                snap.profile_label,
                msg::hosting_task::BOT_SUFFIX.get()
            );
        }
    }
    if let Some(target) = &snap.issues_target {
        gitpoulpi_daemon::hosting_scope::register(
            &req.repo_root,
            &target.profile.id,
            &target.project,
        );
    }
}

/// What the pull request list needs once its client exists.
struct PullsJob {
    client: Box<dyn HostingClient>,
    project: String,
    filter: gitpoulpi_hosting::PrFilter,
    head_branch: Option<String>,
    need_me: bool,
}

/// What the pull request calls found; `None` where a call did not answer.
struct PullsOut {
    prs: Option<Vec<PullRequestSummary>>,
    head_pr: Option<Option<PullRequestSummary>>,
    me: Option<String>,
    problem: Option<Problem>,
}

fn pulls_job_of(
    snap: &mut HostingSnapshot,
    req: &HostingRequest,
    integration: &Integration,
    factory: &dyn ClientFactory,
) -> Option<PullsJob> {
    match factory.client(integration, &req.integrations) {
        Ok(client) => Some(PullsJob {
            client,
            project: snap.project.clone(),
            filter: req.pr_view.filter(),
            head_branch: req.head_branch.clone(),
            need_me: snap.me.is_empty(),
        }),
        Err(err) => {
            snap.problem = Some(Problem::from_error(&err));
            None
        }
    }
}

async fn pulls_part(job: &PullsJob) -> PullsOut {
    let mut out = PullsOut {
        prs: None,
        head_pr: None,
        me: None,
        problem: None,
    };
    // The first real failure is the one shown; later calls still run,
    // so one flaky endpoint does not blank the other list.
    let note = |out: &mut PullsOut, err: HostingError| {
        if out.problem.is_none() && !matches!(err, HostingError::Unsupported(_)) {
            out.problem = Some(Problem::from_error(&err));
        }
    };
    match job
        .client
        .list_pull_requests(&job.project, &job.filter)
        .await
    {
        Ok(prs) => out.prs = Some(prs),
        Err(err) => note(&mut out, err),
    }
    if let Some(branch) = &job.head_branch {
        match job
            .client
            .pull_request_for_branch(&job.project, branch)
            .await
        {
            Ok(found) => out.head_pr = Some(found),
            Err(err) => note(&mut out, err),
        }
    }
    // The "My pull requests" group needs the login; a profile
    // connected before it was recorded asks the host once.
    if job.need_me && out.problem.is_none() {
        if let Ok(user) = job.client.current_user().await {
            out.me = Some(user.login);
        }
    }
    out
}

struct IssuesJob {
    client: Box<dyn HostingClient>,
    project: String,
    filter: gitpoulpi_hosting::IssueFilter,
}

struct IssuesOut {
    issues: Option<Vec<IssueSummary>>,
    problem: Option<Problem>,
}

fn issues_job_of(
    snap: &mut HostingSnapshot,
    req: &HostingRequest,
    integration: &Integration,
    target: &IssuesTarget,
    factory: &dyn ClientFactory,
) -> Option<IssuesJob> {
    // Nothing to ask until a project is picked.
    if target.project.is_empty() {
        return None;
    }
    let client = match factory.client(integration, &req.integrations) {
        Ok(client) => client,
        Err(err) => {
            snap.issues_problem = Some(Problem::from_error(&err));
            return None;
        }
    };
    // The provider-native extra clause the user typed (a JQL / WIQL fragment,
    // Trello list names): the repository's saved `issues_source.filter`.
    let mut filter = req.issue_view.filter();
    filter.query = req
        .issues_source
        .as_ref()
        .and_then(|s| s.filter.clone())
        .filter(|f| !f.trim().is_empty());
    Some(IssuesJob {
        client,
        project: target.project.clone(),
        filter,
    })
}

async fn issues_part(job: &IssuesJob) -> IssuesOut {
    match job.client.list_issues(&job.project, &job.filter).await {
        Ok(issues) => IssuesOut {
            issues: Some(issues),
            problem: None,
        },
        Err(HostingError::Unsupported(_)) => IssuesOut {
            issues: None,
            problem: None,
        },
        Err(err) => IssuesOut {
            issues: None,
            problem: Some(Problem::from_error(&err)),
        },
    }
}

/// The thread and its two channels.
pub(crate) struct HostingWorker {
    tx: mpsc::Sender<HostingRequest>,
    rx: mpsc::Receiver<HostingSnapshot>,
}

impl HostingWorker {
    pub(crate) fn spawn(factory: Arc<dyn ClientFactory>) -> Self {
        let (tx, req_rx) = mpsc::channel::<HostingRequest>();
        let (snap_tx, rx) = mpsc::channel::<HostingSnapshot>();
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            while let Ok(mut req) = req_rx.recv() {
                // Coalesce: only the newest request matters.
                while let Ok(newer) = req_rx.try_recv() {
                    req = newer;
                }
                let snap = fetch_snapshot(&req, factory.as_ref(), &runtime, unix_now());
                if snap_tx.send_notify(snap).is_err() {
                    break;
                }
            }
        });
        Self { tx, rx }
    }

    pub(crate) fn request(&self, req: HostingRequest) {
        let _ = self.tx.send(req);
    }

    pub(crate) fn try_recv(&self) -> Option<HostingSnapshot> {
        self.rx.try_recv().ok()
    }
}

// -- a provider's project list ------------------------------------------------

/// Which answer of a project-list fetch this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListPhase {
    /// The saved copy, sent first when there is one.
    Cached,
    /// The host's answer (or failure): the last message of a fetch.
    Final,
}

/// What a project-list fetch reports: the repositories the profile's token
/// can see (the Clone dialog's hosted sources, the ISSUES repository
/// dropdown, the pull-request form's repository dropdowns).
pub(crate) struct ProjectsAnswer {
    pub(crate) profile_id: String,
    pub(crate) result: Result<Vec<gitpoulpi_hosting::HostedProject>, String>,
    pub(crate) phase: ListPhase,
    /// The host rejected the token: a saved list must not stand in for it.
    pub(crate) unauthorized: bool,
}

/// A provider's project list, as far as it got.
#[derive(Debug, Clone)]
pub(crate) enum ProjectsState {
    Loading,
    Ready(Vec<gitpoulpi_hosting::HostedProject>),
    Failed(String),
}

/// Where a fetch keeps the saved copy of its list.
pub(crate) struct ListCacheJob {
    pub(crate) dir: PathBuf,
    pub(crate) key: gitpoulpi_hosting::list_cache::ListKey,
}

pub(crate) struct ProjectsReceiver {
    rx: mpsc::Receiver<ProjectsAnswer>,
    /// Set when the receiver is dropped: the worker stops and stores nothing.
    cancelled: Arc<std::sync::atomic::AtomicBool>,
}

impl ProjectsReceiver {
    pub(crate) fn try_recv(&self) -> Option<ProjectsAnswer> {
        self.rx.try_recv().ok()
    }
}

impl Drop for ProjectsReceiver {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// How often a running fetch looks at its cancel flag.
const CANCEL_POLL: Duration = Duration::from_millis(100);

/// Lists `integration`'s projects on their own thread (a token lookup is a
/// subprocess and the listing is many HTTP pages: never on the input
/// thread). With `cache`, the saved copy is sent first ([`ListPhase::Cached`]),
/// the fresh list is saved when it lands, and the stale files are pruned; the
/// host's answer is always one [`ListPhase::Final`] message. A dropped
/// receiver cancels the fetch.
pub(crate) fn start_projects(
    factory: Arc<dyn ClientFactory>,
    integration: Integration,
    all: Vec<Integration>,
    cache: Option<ListCacheJob>,
) -> ProjectsReceiver {
    use gitpoulpi_hosting::list_cache;
    use std::sync::atomic::Ordering;
    let (tx, rx) = mpsc::channel();
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    std::thread::spawn(move || {
        let profile_id = integration.id.clone();
        if let Some(job) = &cache {
            if let Some(saved) = list_cache::load(&job.dir, &job.key) {
                let _ = tx.send_notify(ProjectsAnswer {
                    profile_id: profile_id.clone(),
                    result: Ok(saved.projects),
                    phase: ListPhase::Cached,
                    unauthorized: false,
                });
            }
        }
        let mut unauthorized = false;
        let fetched = (|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(request_task_failed)?;
            let client = factory
                .client(&integration, &all)
                .map_err(|e| error_text(&e))?;
            if flag.load(Ordering::SeqCst) {
                return Err(String::new());
            }
            let outcome = runtime.block_on(async {
                let list = client.list_projects();
                tokio::pin!(list);
                loop {
                    tokio::select! {
                        done = &mut list => break Some(done),
                        () = tokio::time::sleep(CANCEL_POLL) => {
                            if flag.load(Ordering::SeqCst) {
                                break None;
                            }
                        }
                    }
                }
            });
            match outcome {
                None => Err(String::new()),
                Some(Ok(list)) => Ok(list),
                Some(Err(e)) => {
                    unauthorized = matches!(e, HostingError::Unauthorized { .. });
                    Err(error_text(&e))
                }
            }
        })();
        if flag.load(Ordering::SeqCst) {
            return;
        }
        if let Some(job) = &cache {
            match &fetched {
                Ok(list) => {
                    let saved = list_cache::CachedList::new(
                        job.key.clone(),
                        unix_now().max(0) as u64,
                        list.clone(),
                    );
                    if list_cache::store(&job.dir, &saved).is_ok() {
                        let live: Vec<_> = all
                            .iter()
                            .filter_map(|i| {
                                let login = i.user.as_ref().map(|u| u.login.as_str());
                                list_cache::ListKey::of(i, login)
                            })
                            .collect();
                        list_cache::prune(&job.dir, &live);
                    }
                }
                Err(_) if unauthorized => list_cache::remove(&job.dir, &job.key),
                Err(_) => {}
            }
        }
        let _ = tx.send_notify(ProjectsAnswer {
            profile_id,
            result: fetched,
            phase: ListPhase::Final,
            unauthorized,
        });
    });
    ProjectsReceiver { rx, cancelled }
}

// -- the choices of the New issue form --------------------------------------

/// What a create-options fetch reports: the issue types (Jira, Azure) or
/// lists (Trello) a project offers.
pub(crate) type OptionsResult = Result<gitpoulpi_hosting::IssueCreateOptions, String>;

pub(crate) struct OptionsReceiver {
    rx: mpsc::Receiver<OptionsResult>,
}

impl OptionsReceiver {
    pub(crate) fn try_recv(&self) -> Option<OptionsResult> {
        self.rx.try_recv().ok()
    }
}

/// Asks `integration` what `project` lets a new issue choose (a Jira issue
/// type, an Azure work item type, a Trello list) on its own thread: a token
/// lookup is a subprocess and the call is HTTP, never on the input thread.
pub(crate) fn start_issue_options(
    factory: Arc<dyn ClientFactory>,
    integration: Integration,
    all: Vec<Integration>,
    project: String,
) -> OptionsReceiver {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(request_task_failed)?;
            let client = factory
                .client(&integration, &all)
                .map_err(|e| error_text(&e))?;
            runtime
                .block_on(client.issue_create_options(&project))
                .map_err(|e| error_text(&e))
        })();
        let _ = tx.send_notify(result);
    });
    OptionsReceiver { rx }
}

// -- when to refresh --------------------------------------------------------

/// Why a refresh was requested (shown nowhere; kept for tests and logs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    Opened,
    HeadChanged,
    RemotesChanged,
    Interval,
    Demand,
}

/// A repository's coarse "did anything the host cares about change"
/// signature: the checked-out branch and every remote-tracking ref's
/// target. A push or a fetch moves remote-tracking refs, so this notices
/// them without hooking every submit site; local commits (which move the
/// local branch tip constantly) deliberately do not count.
pub(crate) fn refs_signature(refs: &[Ref]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut entries: Vec<(String, [u8; 20])> = Vec::new();
    for r in refs {
        match &r.kind {
            RefKind::LocalBranch { name } if r.is_head => {
                entries.push((format!("HEAD:{name}"), [0; 20]));
            }
            RefKind::RemoteBranch { remote, name } => {
                entries.push((format!("{remote}/{name}"), r.target.0));
            }
            _ => {}
        }
    }
    entries.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entries.hash(&mut hasher);
    hasher.finish()
}

/// Decides when the next request goes out. Pure: `now` is passed in.
#[derive(Debug)]
pub(crate) struct RefreshPolicy {
    interval: Option<Duration>,
    last_sent: Option<Instant>,
    last_head: Option<Option<String>>,
    last_signature: Option<u64>,
    demanded: bool,
}

/// Requests closer together than this are not sent (except a demand).
const MIN_GAP: Duration = Duration::from_secs(2);

impl RefreshPolicy {
    pub(crate) fn new(interval: Option<Duration>) -> Self {
        Self {
            interval,
            last_sent: None,
            last_head: None,
            last_signature: None,
            demanded: false,
        }
    }

    /// The user asked (palette, `Ctrl+G g`-adjacent keys, a changed
    /// filter, Preferences closing).
    pub(crate) fn demand(&mut self) {
        self.demanded = true;
    }

    pub(crate) fn set_interval(&mut self, interval: Option<Duration>) {
        self.interval = interval;
    }

    /// When the periodic refresh falls due, if it is on.
    pub(crate) fn next_due(&self) -> Option<Instant> {
        self.interval.zip(self.last_sent).map(|(i, at)| at + i)
    }

    /// Called whenever a repository refresh landed, with what it says.
    pub(crate) fn observe(&mut self, head: Option<&str>, signature: u64) -> Option<Reason> {
        let head_owned = head.map(str::to_string);
        let reason = if self.last_head.is_none() {
            Some(Reason::Opened)
        } else if self.last_head.as_ref() != Some(&head_owned) {
            Some(Reason::HeadChanged)
        } else if self.last_signature != Some(signature) {
            Some(Reason::RemotesChanged)
        } else {
            None
        };
        self.last_head = Some(head_owned);
        self.last_signature = Some(signature);
        reason
    }

    /// Whether to send now, given the reason `observe` found (if any).
    /// Records the send when it says yes.
    pub(crate) fn should_send(&mut self, now: Instant, observed: Option<Reason>) -> Option<Reason> {
        let demanded = std::mem::take(&mut self.demanded);
        let due = self
            .interval
            .zip(self.last_sent)
            .is_some_and(|(interval, at)| now.duration_since(at) >= interval);
        let reason = if demanded {
            Some(Reason::Demand)
        } else if let Some(r) = observed {
            let quiet = self
                .last_sent
                .is_some_and(|at| now.duration_since(at) < MIN_GAP);
            // Debounce a burst of ref changes, but never the very first
            // request.
            if quiet && r != Reason::Opened {
                // Remember to retry once the gap has passed.
                self.last_signature = None;
                None
            } else {
                Some(r)
            }
        } else if due {
            Some(Reason::Interval)
        } else {
            None
        };
        if reason.is_some() {
            self.last_sent = Some(now);
        }
        reason
    }
}

// -- the on-disk snapshot cache ----------------------------------------------

/// Where a repository's last snapshot lives: beside its
/// `repo-state.toml`.
pub(crate) fn cache_path(repo_root: &Path, state_dir: &str) -> PathBuf {
    gitpoulpi_session::RepoState::hosting_cache_path_for(repo_root, state_dir)
}

pub(crate) fn load_cache(path: &Path) -> Option<HostingSnapshot> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Written only for a repository a profile answers for (a repository with
/// no integration never grows a `.gitpoulpi/` directory from this), and
/// the directory ignores itself exactly like `RepoState::save` makes it:
/// the cache sits in the working tree, and a file git can see turns the
/// repository "dirty".
pub(crate) fn save_cache(path: &Path, snapshot: &HostingSnapshot) {
    if !snapshot.linked() && snapshot.issues_target.is_none() {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
        let ignore = parent.join(".gitignore");
        if !ignore.exists() {
            let _ = std::fs::write(ignore, include_str!("state.gitignore"));
        }
    }
    if let Ok(json) = serde_json::to_string(snapshot) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(all(test, feature = "github", feature = "gitlab"))]
pub(crate) mod fake;

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests;
