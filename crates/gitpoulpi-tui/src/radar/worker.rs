//! The Radar's worker: one hosting fetch per repository of the scope, off
//! the input thread, at most [`IN_FLIGHT`] at a time.
//!
//! Same shape as `hosting_task::HostingWorker` (a plain thread, channels the
//! render loop drains with `try_recv`) and the same fetch: every repository
//! goes through `fetch_snapshot`, so the provider resolution, the typed
//! problems and the on-disk cache are the sidebar's, not a second copy.
//!
//! A round has two phases. First every repository's cache is read and
//! answered at once (`cached: true`), so rows show before any network call.
//! Then the repositories are fetched, four at a time; each answer replaces
//! the cached one. Only repositories already linked to a provider are asked
//! (hosting is opt-in per repository): an unlinked one answers without a
//! request, carrying the providers its remotes suggest.
//!
//! A host that says to slow down (429) or rejects the token (401/403) is
//! paused for the rest of the round: its other repositories are answered
//! from their caches with that reason instead of being asked again.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use gitpoulpi_config::{ApiFamily, Integration};
use gitpoulpi_hosting::account::{AccountCache, AccountItems, CachedAccount};
use gitpoulpi_hosting::ResolveRemote;

use super::account::{AccountAnswer, AccountJob, AccountPlan, CloneIndex, Offer};

use crate::hosting_task::{
    cache_path, fetch_snapshot, load_cache, save_cache, unix_now, ClientFactory, HostingRequest,
    HostingSnapshot, IssueView, PrView, Problem,
};
use crate::redraw::NotifySend as _;

/// Repositories asked at the same time.
pub(crate) const IN_FLIGHT: usize = 4;

/// One repository of the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Job {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
}

/// What a job needs to be asked.
#[derive(Debug, Clone)]
pub(crate) struct Prepared {
    pub(crate) request: HostingRequest,
    pub(crate) cache: Option<PathBuf>,
}

/// How a repository becomes a [`Prepared`]: production reads its state file,
/// its git config and the integrations; tests hand over fixed ones.
pub(crate) trait Prepare: Send + Sync {
    fn prepare(&self, repo: &Path) -> Prepared;

    /// A round begins: what was read once for all its repositories (the
    /// configuration) is read again.
    fn begin_round(&self) {}

    /// The account-wide lists to ask and the hosts to offer them for (none
    /// by default).
    fn accounts(&self) -> AccountPlan {
        AccountPlan::default()
    }

    /// The clones the machine knows, given the scope's repositories.
    fn known_clones(&self, _scope: &[PathBuf]) -> CloneIndex {
        CloneIndex::default()
    }
}

/// Something read once per round and shared by every repository of it.
pub(crate) struct PerRound<T>(Mutex<Option<Arc<T>>>);

impl<T> Default for PerRound<T> {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

impl<T> PerRound<T> {
    pub(crate) fn get(&self, load: impl FnOnce() -> T) -> Arc<T> {
        let mut slot = self.0.lock().unwrap_or_else(|e| e.into_inner());
        Arc::clone(slot.get_or_insert_with(|| Arc::new(load())))
    }

    pub(crate) fn reset(&self) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// What the worker says about one repository.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RepoAnswer {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    /// The provider's host (`github.com`), the first part of an item's key;
    /// empty when the repository is not linked.
    pub(crate) host: String,
    /// The signed-in login on the provider; empty when unknown.
    pub(crate) me: String,
    pub(crate) snapshot: HostingSnapshot,
    /// Read from the disk cache, not (yet) from the host.
    pub(crate) cached: bool,
    pub(crate) round: u64,
    /// The host was paused for this round: the reason, as the user reads it.
    pub(crate) paused: Option<String>,
}

impl RepoAnswer {
    /// Linked to a provider for pull requests or for issues.
    pub(crate) fn linked(&self) -> bool {
        self.snapshot.linked() || self.snapshot.issues_target.is_some()
    }
}

/// What a round starts with, before any row: the integrations whose lists
/// are being asked, the hosts to offer a `[Turn on]` for, and the clones.
#[derive(Debug)]
pub(crate) struct Setup {
    pub(crate) round: u64,
    pub(crate) active: Vec<String>,
    pub(crate) offers: Vec<Offer>,
    pub(crate) clones: CloneIndex,
}

#[derive(Debug)]
pub(crate) enum Msg {
    Answer(Box<RepoAnswer>),
    /// The account-wide rows of one integration.
    Account(Box<AccountAnswer>),
    Setup(Box<Setup>),
    /// Every repository of the round answered.
    Done(u64),
}

struct Round {
    id: u64,
    jobs: Vec<Job>,
    /// Ask the account-wide lists again even when the cache is fresh.
    force: bool,
}

pub(crate) struct RadarWorker {
    tx: mpsc::Sender<Round>,
    rx: mpsc::Receiver<Msg>,
    current: Arc<AtomicU64>,
    next: u64,
}

impl RadarWorker {
    pub(crate) fn spawn(factory: Arc<dyn ClientFactory>, prepare: Arc<dyn Prepare>) -> Self {
        let (tx, round_rx) = mpsc::channel::<Round>();
        let (msg_tx, rx) = mpsc::channel::<Msg>();
        let current = Arc::new(AtomicU64::new(0));
        let shared = Arc::clone(&current);
        std::thread::spawn(move || {
            while let Ok(mut round) = round_rx.recv() {
                // Coalesce: only the newest round matters.
                while let Ok(newer) = round_rx.try_recv() {
                    round = newer;
                }
                shared.store(round.id, Ordering::SeqCst);
                run_round(
                    &round,
                    factory.as_ref(),
                    prepare.as_ref(),
                    &shared,
                    &msg_tx,
                    IN_FLIGHT,
                );
            }
        });
        Self {
            tx,
            rx,
            current,
            next: 0,
        }
    }

    /// Starts a round over `jobs` and returns its id; the answers of an
    /// earlier round that arrive later carry an older id.
    ///
    /// `force`: the account-wide lists are asked again whatever their age
    /// (the Refresh button); otherwise a cache younger than five minutes is
    /// reused.
    pub(crate) fn start(&mut self, jobs: Vec<Job>, force: bool) -> u64 {
        self.next += 1;
        // Tells a running round to stop asking at once.
        self.current.store(self.next, Ordering::SeqCst);
        let _ = self.tx.send(Round {
            id: self.next,
            jobs,
            force,
        });
        self.next
    }

    pub(crate) fn try_recv(&self) -> Option<Msg> {
        self.rx.try_recv().ok()
    }
}

/// One unit of a round's second phase.
enum Work {
    Repo(Job, Prepared),
    Account(AccountJob),
}

fn run_round(
    round: &Round,
    factory: &dyn ClientFactory,
    prepare: &dyn Prepare,
    current: &AtomicU64,
    out: &mpsc::Sender<Msg>,
    in_flight: usize,
) {
    let stale = || current.load(Ordering::SeqCst) != round.id;
    factory.new_round();
    prepare.begin_round();
    if stale() {
        return;
    }
    // Phase 0: what the account-wide rows need to be placed (which hosts are
    // asked, which could be, which clones exist), before any of them shows.
    let plan = prepare.accounts();
    let clones = if plan.jobs.is_empty() {
        CloneIndex::default()
    } else {
        let paths: Vec<PathBuf> = round.jobs.iter().map(|j| j.path.clone()).collect();
        prepare.known_clones(&paths)
    };
    let setup = Setup {
        round: round.id,
        active: plan.jobs.iter().map(|j| j.integration.id.clone()).collect(),
        offers: plan.offers.clone(),
        clones,
    };
    if out.send_notify(Msg::Setup(Box::new(setup))).is_err() {
        return;
    }
    // Phase 1: the caches, at once.
    let mut queue: VecDeque<Work> = VecDeque::new();
    let mut cached: HashMap<PathBuf, HostingSnapshot> = HashMap::new();
    for job in &round.jobs {
        if stale() {
            return;
        }
        let prepared = prepare.prepare(&job.path);
        if let Some(snap) = prepared.cache.as_deref().and_then(load_cache) {
            let snap = trim_cached(snap, &prepared.request);
            cached.insert(job.path.clone(), snap.clone());
            let answer = answer_of(job, &prepared.request, snap, true, round.id, None);
            if out.send_notify(Msg::Answer(Box::new(answer))).is_err() {
                return;
            }
        }
        queue.push_back(Work::Repo(job.clone(), prepared));
    }
    // The account rows of the last answer, and which of them are young
    // enough to be the answer.
    let account_cache = plan.cache.as_deref().map(AccountCache::load);
    let mut account_cached: HashMap<String, CachedAccount> = HashMap::new();
    for job in &plan.jobs {
        if stale() {
            return;
        }
        let hash = job.query.fingerprint();
        let hit = account_cache
            .as_ref()
            .and_then(|c| c.get(&job.integration.id, &hash))
            .cloned();
        if let Some(hit) = &hit {
            let answer = account_answer(job, round.id, hit.items.clone(), hit.fetched_at, true);
            if out.send_notify(Msg::Account(Box::new(answer))).is_err() {
                return;
            }
        }
        let fresh = hit.as_ref().is_some_and(|h| h.is_fresh(unix_now() as u64));
        if let Some(hit) = hit {
            account_cached.insert(job.integration.id.clone(), hit);
        }
        // After the repositories: their hosts' pauses apply to these too.
        if round.force || !fresh {
            queue.push_back(Work::Account(job.clone()));
        }
    }
    // Phase 2: the hosts, four at a time.
    let total = queue.len();
    let queue = Mutex::new(queue);
    let paused: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    let cached = &cached;
    let account_cached = &account_cached;
    let plan = &plan;
    std::thread::scope(|scope| {
        for _ in 0..in_flight.min(total).max(1) {
            scope.spawn(|| {
                let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                else {
                    return;
                };
                loop {
                    if stale() {
                        return;
                    }
                    let Some(work) = queue.lock().ok().and_then(|mut q| q.pop_front()) else {
                        return;
                    };
                    let message = match work {
                        Work::Repo(job, prepared) => Msg::Answer(Box::new(repo_work(
                            &job, &prepared, round.id, factory, &runtime, cached, &paused,
                        ))),
                        Work::Account(job) => Msg::Account(Box::new(account_work(
                            &job,
                            plan,
                            round.id,
                            factory,
                            &runtime,
                            account_cached.get(&job.integration.id),
                            &paused,
                        ))),
                    };
                    if out.send_notify(message).is_err() {
                        return;
                    }
                }
            });
        }
    });
    let _ = out.send_notify(Msg::Done(round.id));
}

/// Pauses `host` for the rest of the round when the problem says to slow down
/// or to reconnect.
fn pause_on(
    problem: Option<&Problem>,
    host: Option<String>,
    paused: &Mutex<HashMap<String, String>>,
) {
    if let (Some(Problem::RateLimited(why) | Problem::Reconnect(why)), Some(host), Ok(mut p)) =
        (problem, host, paused.lock())
    {
        p.entry(host).or_insert_with(|| why.clone());
    }
}

fn repo_work(
    job: &Job,
    prepared: &Prepared,
    round: u64,
    factory: &dyn ClientFactory,
    runtime: &tokio::runtime::Runtime,
    cached: &HashMap<PathBuf, HostingSnapshot>,
    paused: &Mutex<HashMap<String, String>>,
) -> RepoAnswer {
    let host = planned_host(&prepared.request);
    let pause = host
        .as_ref()
        .and_then(|h| paused.lock().ok().and_then(|p| p.get(h).cloned()));
    if let Some(reason) = pause {
        // Not asked again this round: what the cache said.
        let snap = cached
            .get(&job.path)
            .cloned()
            .unwrap_or_else(|| HostingSnapshot::empty(unix_now(), &prepared.request));
        return answer_of(job, &prepared.request, snap, true, round, Some(reason));
    }
    let mut snap = fetch_snapshot(&prepared.request, factory, runtime, unix_now());
    fetch_extras(&mut snap, &prepared.request, factory, runtime);
    let stop = [snap.problem.as_ref(), snap.issues_problem.as_ref()]
        .into_iter()
        .flatten()
        .find(|p| matches!(p, Problem::RateLimited(_) | Problem::Reconnect(_)));
    pause_on(stop, host, paused);
    remember(prepared, &snap);
    answer_of(job, &prepared.request, snap, false, round, None)
}

fn account_answer(
    job: &AccountJob,
    round: u64,
    items: AccountItems,
    fetched_at: u64,
    cached: bool,
) -> AccountAnswer {
    AccountAnswer {
        round,
        id: job.integration.id.clone(),
        label: job.label.clone(),
        kind: job.integration.kind,
        host: job.integration.host().unwrap_or_default(),
        me: job
            .integration
            .user
            .as_ref()
            .map(|u| u.login.clone())
            .unwrap_or_default(),
        items,
        fetched_at,
        cached,
        problem: None,
        paused: None,
    }
}

/// One integration's account-wide rows from its host. A host paused by the
/// round answers with what the cache had; a failure keeps the cache's rows
/// and says why.
fn account_work(
    job: &AccountJob,
    plan: &AccountPlan,
    round: u64,
    factory: &dyn ClientFactory,
    runtime: &tokio::runtime::Runtime,
    cached: Option<&CachedAccount>,
    paused: &Mutex<HashMap<String, String>>,
) -> AccountAnswer {
    let host = job.integration.host();
    let old = || match cached {
        Some(c) => account_answer(job, round, c.items.clone(), c.fetched_at, true),
        None => account_answer(job, round, AccountItems::default(), 0, true),
    };
    let pause = host
        .as_ref()
        .and_then(|h| paused.lock().ok().and_then(|p| p.get(h).cloned()));
    if let Some(reason) = pause {
        return AccountAnswer {
            paused: Some(reason),
            ..old()
        };
    }
    let fetched = factory
        .client(&job.integration, &plan.all)
        .and_then(|client| runtime.block_on(client.account_items(&job.query)));
    match fetched {
        Ok(items) => {
            let now = unix_now() as u64;
            if let Some(path) = plan.cache.as_deref() {
                // A cache that cannot be written only costs the next round
                // a fetch.
                let _ = AccountCache::store(
                    path,
                    &job.integration.id,
                    CachedAccount {
                        fetched_at: now,
                        query_hash: job.query.fingerprint(),
                        items: items.clone(),
                    },
                );
            }
            account_answer(job, round, items, now, false)
        }
        Err(err) => {
            let problem = Problem::from_error(&err);
            pause_on(Some(&problem), host, paused);
            AccountAnswer {
                problem: Some(problem),
                ..old()
            }
        }
    }
}

/// What only the Radar asks, on top of the sidebar's fetch: the open
/// requests and the issues that mention the user, and on GitHub the checks and
/// conflict state of the requests (a bounded number of extra calls, see the
/// client). GitHub, GitLab, Gitea and Forgejo. Best effort: a failing call leaves the
/// group empty / the state unknown, it never turns the repository's answer
/// into a problem the main fetch did not have.
fn fetch_extras(
    snap: &mut HostingSnapshot,
    req: &HostingRequest,
    factory: &dyn ClientFactory,
    runtime: &tokio::runtime::Runtime,
) {
    use gitpoulpi_hosting::{IssueFilter, PrFilter};
    let family = |kind: gitpoulpi_config::IntegrationKind| kind.family();
    let shown = |kind| {
        matches!(
            family(kind),
            ApiFamily::Github | ApiFamily::Gitlab | ApiFamily::Gitea
        )
    };
    let client_for = |id: &str| {
        let integration = req.integrations.iter().find(|i| i.id == id)?;
        factory.client(integration, &req.integrations).ok()
    };
    // Which calls are due, decided before any is made, so that the three
    // (mentioned requests, checks, mentioned issues) wait on the network
    // together instead of one after the other.
    let prs = if snap.problem.is_none() && snap.can_pulls && !snap.project.is_empty() {
        snap.kind
            .filter(|k| shown(*k))
            .zip(snap.profile_id.as_deref().and_then(client_for))
    } else {
        None
    };
    let issues = if snap.issues_problem.is_none() {
        snap.issues_target
            .clone()
            .filter(|t| !t.project.is_empty() && shown(t.profile.kind))
            .and_then(|t| client_for(&t.profile.id).map(|c| (t.project, c)))
    } else {
        None
    };
    if prs.is_none() && issues.is_none() {
        return;
    }
    let project = snap.project.clone();
    let mentioned_prs = PrFilter {
        mentioned: true,
        ..PrFilter::default()
    };
    let with_checks = PrFilter {
        checks: true,
        ..PrFilter::default()
    };
    let mentioned_issues = IssueFilter {
        mentioned: true,
        ..IssueFilter::default()
    };
    let (found_prs, checked, found_issues) = runtime.block_on(async {
        tokio::join!(
            async {
                match &prs {
                    Some((_, client)) => client
                        .list_pull_requests(&project, &mentioned_prs)
                        .await
                        .ok(),
                    None => None,
                }
            },
            async {
                match &prs {
                    // GitLab's list already carries both.
                    Some((kind, client))
                        if matches!(family(*kind), ApiFamily::Github | ApiFamily::Gitea) =>
                    {
                        client.list_pull_requests(&project, &with_checks).await.ok()
                    }
                    _ => None,
                }
            },
            async {
                match &issues {
                    Some((project, client)) => {
                        client.list_issues(project, &mentioned_issues).await.ok()
                    }
                    None => None,
                }
            }
        )
    });
    if let Some(found) = found_prs {
        snap.prs_mentioned = found;
    }
    if let Some(rows) = checked {
        for pr in &mut snap.prs {
            if let Some(row) = rows.iter().find(|r| r.number == pr.number) {
                pr.checks = row.checks;
                pr.has_conflicts = row.has_conflicts;
            }
        }
    }
    if let Some(found) = found_issues {
        snap.issues_mentioned = found;
    }
}

/// The host a request is going to ask, before asking: the provider the user
/// pinned, for pull requests or issues. `None` for an unlinked repository
/// (nothing is asked).
fn planned_host(req: &HostingRequest) -> Option<String> {
    let id = req
        .pulls_source
        .as_ref()
        .map(|s| s.integration.as_str())
        .or(req.override_id.as_deref())
        .or_else(|| req.issues_source.as_ref().map(|s| s.integration.as_str()))?;
    req.integrations
        .iter()
        .find(|i| i.id == id)
        .and_then(Integration::host)
}

fn answer_of(
    job: &Job,
    req: &HostingRequest,
    snapshot: HostingSnapshot,
    cached: bool,
    round: u64,
    paused: Option<String>,
) -> RepoAnswer {
    let profile = |id: &Option<String>| {
        id.as_ref()
            .and_then(|id| req.integrations.iter().find(|i| &i.id == id))
    };
    let pulls = profile(&snapshot.profile_id);
    let issues = profile(
        &snapshot
            .issues_target
            .as_ref()
            .map(|t| t.profile.id.clone()),
    );
    let host = pulls
        .or(issues)
        .and_then(Integration::host)
        .unwrap_or_default();
    // The login: the snapshot's, else what the profile recorded when it was
    // connected (an issues-only repository never asks the host who you are).
    let me = if snapshot.me.is_empty() {
        pulls
            .or(issues)
            .and_then(|i| i.user.as_ref())
            .map(|u| u.login.clone())
            .unwrap_or_default()
    } else {
        snapshot.me.clone()
    };
    RepoAnswer {
        path: job.path.clone(),
        name: job.name.clone(),
        host,
        me,
        snapshot,
        cached,
        round,
        paused,
    }
}

/// The providers whose pull / merge requests the Radar lists per repository:
/// GitHub, GitLab, Gitea and Forgejo (with checks, conflicts and mentions),
/// and, lists only, Azure DevOps, Bitbucket and Gerrit when built in.
fn lists_requests(kind: gitpoulpi_config::IntegrationKind) -> bool {
    match kind.family() {
        ApiFamily::Github | ApiFamily::Gitlab | ApiFamily::Gitea => true,
        ApiFamily::AzureDevops | ApiFamily::Bitbucket | ApiFamily::Gerrit => {
            crate::build_features::kind_built(kind)
        }
        _ => false,
    }
}

/// A cache may predate an unlink, and the sidebar may have cached a provider
/// the Radar does not show: drop what is not the Radar's.
fn trim_cached(mut snap: HostingSnapshot, req: &HostingRequest) -> HostingSnapshot {
    if req.pulls_source.is_none() && req.override_id.is_none() {
        snap.profile_id = None;
        snap.pulls_target = None;
        snap.prs.clear();
        snap.head_pr = None;
        snap.can_pulls = false;
    }
    if req.issues_source.is_none() {
        snap.issues_target = None;
        snap.issues.clear();
    }
    let shown = lists_requests;
    if snap.kind.is_some_and(|k| !shown(k)) {
        snap.profile_id = None;
        snap.kind = None;
        snap.prs.clear();
        snap.head_pr = None;
        snap.can_pulls = false;
    }
    if snap
        .issues_target
        .as_ref()
        .is_some_and(|t| !shown(t.profile.kind))
    {
        snap.issues_target = None;
        snap.issues.clear();
    }
    if !snap.can_pulls {
        snap.prs_mentioned.clear();
    }
    if snap.issues_target.is_none() {
        snap.issues_mentioned.clear();
    }
    snap
}

/// Writes the answer into the repository's hosting cache (the one the sidebar
/// reads), unless that would change what the sidebar's filters show: the
/// Radar asks for open requests and open issues, so it only overwrites a
/// cache that holds the same views.
fn remember(prepared: &Prepared, snap: &HostingSnapshot) {
    if snap.problem.is_some() || snap.issues_problem.is_some() {
        return;
    }
    let Some(path) = prepared.cache.as_deref() else {
        return;
    };
    let compatible = load_cache(path)
        .is_none_or(|old| old.pr_view == PrView::Open && old.issue_view == IssueView::Open);
    if compatible {
        save_cache(path, snap);
    }
}

// -- production wiring ---------------------------------------------------

/// Reads each repository's own state, remotes and the configured providers.
#[derive(Default)]
pub(crate) struct FsPrepare {
    /// The providers and the state directory: `config.toml` is read once for
    /// the round, not twice per repository.
    config: PerRound<(Vec<Integration>, String)>,
    /// Where the Radar's own files live (recents, workspaces, the account
    /// cache).
    config_dir: Option<PathBuf>,
}

impl FsPrepare {
    pub(crate) fn new(config_dir: Option<PathBuf>) -> Self {
        Self {
            config: PerRound::default(),
            config_dir,
        }
    }
}

impl Prepare for FsPrepare {
    fn begin_round(&self) {
        self.config.reset();
    }

    fn accounts(&self) -> AccountPlan {
        super::account_fs::plan_of(
            crate::pull_request::load_integrations(),
            self.config_dir.as_deref(),
        )
    }

    fn known_clones(&self, scope: &[PathBuf]) -> CloneIndex {
        super::account_fs::known_clones(scope, self.config_dir.as_deref())
    }

    fn prepare(&self, repo: &Path) -> Prepared {
        let state =
            gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(repo)).ok();
        let remotes = crate::repo_probe::git_config_path(repo)
            .and_then(|config| std::fs::read_to_string(config).ok())
            .map(|raw| crate::repo_probe::parse_remote_urls(&raw))
            .unwrap_or_default()
            .into_iter()
            .map(|(name, url)| ResolveRemote::new(name, url))
            .collect();
        // GitHub and GitLab (maintainer rule 6), plus the local Markdown issue
        // source, which is the repository's own.
        let config = self.config.get(|| {
            let integrations: Vec<Integration> = crate::pull_request::load_integrations()
                .into_iter()
                .filter(|i| lists_requests(i.kind) || i.kind.family() == ApiFamily::LocalMarkdown)
                .collect();
            let state_dir = gitpoulpi_config::Settings::resolve_ambient()
                .map(|s| s.config().session.per_repo_state_dir.clone())
                .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
            (integrations, state_dir)
        });
        let (integrations, state_dir) = (config.0.clone(), config.1.as_str());
        let issues_source = crate::hosting_task::resolve_local_source(
            repo,
            &integrations,
            state.as_ref().and_then(|s| s.issues_source.clone()),
        );
        Prepared {
            request: HostingRequest {
                remotes,
                integrations,
                override_id: state.as_ref().and_then(|s| s.hosting_integration.clone()),
                head_branch: None,
                pr_view: PrView::Open,
                issue_view: IssueView::Open,
                issues_source,
                pulls_source: state.and_then(|s| s.pulls_source),
                repo_root: repo.to_path_buf(),
            },
            cache: Some(cache_path(repo, state_dir)),
        }
    }
}

/// The repositories whose hosting the worker would ask: everything that is
/// not a duplicate path.
pub(crate) fn unique_jobs(repos: impl IntoIterator<Item = (PathBuf, String)>) -> Vec<Job> {
    let mut seen: HashSet<String> = HashSet::new();
    repos
        .into_iter()
        .filter(|(path, _)| seen.insert(gitpoulpi_session::repo_path_key(path)))
        .map(|(path, name)| Job { path, name })
        .collect()
}

#[cfg(test)]
pub(crate) fn empty_snapshot() -> HostingSnapshot {
    HostingSnapshot::empty(
        0,
        &HostingRequest {
            remotes: Vec::new(),
            integrations: Vec::new(),
            override_id: None,
            head_branch: None,
            pr_view: PrView::Open,
            issue_view: IssueView::Open,
            issues_source: None,
            pulls_source: None,
            repo_root: PathBuf::new(),
        },
    )
}

#[cfg(all(test, feature = "github", feature = "gitlab"))]
mod tests;
