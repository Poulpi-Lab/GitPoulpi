//! The worker over the hosting fake: no socket, no token, no repository.

use super::*;
use crate::hosting_task::fake::{issue, pr, FakeFactory};
use gitpoulpi_config::IntegrationKind;
use gitpoulpi_hosting::{Error as HostingError, PullRequestState};
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

fn gitlab_profile() -> Integration {
    Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    )
}

fn linked(url: &str) -> HostingRequest {
    let source = gitpoulpi_session::IssuesSource {
        integration: "corp".into(),
        project: "g/p".into(),
        filter: None,
    };
    HostingRequest {
        remotes: vec![ResolveRemote::new("origin", url)],
        integrations: vec![gitlab_profile()],
        override_id: None,
        head_branch: None,
        pr_view: PrView::Open,
        issue_view: IssueView::Open,
        issues_source: Some(source.clone()),
        pulls_source: Some(source),
        repo_root: std::path::PathBuf::new(),
    }
}

fn unlinked(url: &str) -> HostingRequest {
    HostingRequest {
        issues_source: None,
        pulls_source: None,
        repo_root: std::path::PathBuf::new(),
        ..linked(url)
    }
}

struct Fixed(HashMap<PathBuf, Prepared>);

impl Prepare for Fixed {
    fn prepare(&self, repo: &Path) -> Prepared {
        self.0.get(repo).cloned().expect("a prepared repository")
    }
}

fn job(name: &str) -> Job {
    Job {
        path: PathBuf::from(format!("/repos/{name}")),
        name: name.into(),
    }
}

fn fixed(entries: Vec<(&str, HostingRequest, Option<PathBuf>)>) -> Arc<Fixed> {
    Arc::new(Fixed(
        entries
            .into_iter()
            .map(|(name, request, cache)| (job(name).path, Prepared { request, cache }))
            .collect(),
    ))
}

/// Every message of one round, in arrival order.
fn collect(rx: &RadarWorker) -> Vec<Msg> {
    let mut out = Vec::new();
    for _ in 0..400 {
        while let Some(msg) = rx.try_recv() {
            let done = matches!(msg, Msg::Done(_));
            out.push(msg);
            if done {
                return out;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the round never finished: {out:?}");
}

fn answers(msgs: Vec<Msg>) -> Vec<RepoAnswer> {
    msgs.into_iter()
        .filter_map(|m| match m {
            Msg::Answer(a) => Some(*a),
            _ => None,
        })
        .collect()
}

#[test]
fn a_cached_repository_answers_before_the_host_does() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("hosting-cache.json");
    // What an earlier fetch left: one request.
    let mut old = HostingSnapshot::empty(100, &linked("https://gitlab.example.com/g/p.git"));
    old.profile_id = Some("corp".into());
    old.kind = Some(IntegrationKind::GitlabSelfManaged);
    old.can_pulls = true;
    old.project = "g/p".into();
    old.prs = vec![pr(1, "old-branch", PrState::Open)];
    save_cache(&cache, &old);

    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(2, "new-branch", PrState::Open)];
    let prepare = fixed(vec![(
        "app",
        linked("https://gitlab.example.com/g/p.git"),
        Some(cache),
    )]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    let round = worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].cached && got[0].round == round);
    assert_eq!(got[0].snapshot.prs[0].number, 1);
    assert!(!got[1].cached);
    assert_eq!(got[1].snapshot.prs[0].number, 2);
    assert_eq!(got[1].host, "gitlab.example.com");
}

type PrState = PullRequestState;

#[test]
fn an_unlinked_repository_is_never_asked() {
    let factory = FakeFactory::new();
    let seen = Arc::clone(&factory.seen);
    let prepare = fixed(vec![(
        "app",
        unlinked("https://gitlab.example.com/g/p.git"),
        None,
    )]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    assert_eq!(got.len(), 1);
    assert!(!got[0].linked());
    assert!(seen.lock().unwrap().is_empty(), "no host call");
    // The remote still proposes its provider, for the "Link..." row.
    assert_eq!(got[0].snapshot.suggestions.len(), 1);
    assert_eq!(got[0].snapshot.suggestions[0].project, "g/p");
}

#[test]
fn the_round_writes_the_cache_the_sidebar_reads() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("sub").join("hosting-cache.json");
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(7, "b", PrState::Open)];
    factory.issues = vec![issue(9)];
    let prepare = fixed(vec![(
        "app",
        linked("https://gitlab.example.com/g/p.git"),
        Some(cache.clone()),
    )]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    collect(&worker);
    let saved = load_cache(&cache).expect("a cache file");
    assert_eq!(saved.prs[0].number, 7);
    assert_eq!(saved.issues[0].number, 9);
}

#[test]
fn a_cache_of_another_view_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("hosting-cache.json");
    let mut old = HostingSnapshot::empty(100, &linked("https://gitlab.example.com/g/p.git"));
    old.profile_id = Some("corp".into());
    old.pr_view = PrView::Closed;
    save_cache(&cache, &old);
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(7, "b", PrState::Open)];
    let prepare = fixed(vec![(
        "app",
        linked("https://gitlab.example.com/g/p.git"),
        Some(cache.clone()),
    )]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    collect(&worker);
    assert_eq!(load_cache(&cache).unwrap().pr_view, PrView::Closed);
}

/// Counts how many `client()` calls overlap.
struct Counting {
    inner: FakeFactory,
    now: AtomicUsize,
    peak: AtomicUsize,
}

impl ClientFactory for Counting {
    fn client(
        &self,
        integration: &Integration,
        all: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn gitpoulpi_hosting::HostingClient>> {
        let now = self.now.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(60));
        self.now.fetch_sub(1, Ordering::SeqCst);
        self.inner.client(integration, all)
    }
}

#[test]
fn at_most_four_repositories_are_in_flight() {
    let names: Vec<String> = (0..10).map(|i| format!("r{i}")).collect();
    let prepare = fixed(
        names
            .iter()
            .map(|n| {
                (
                    n.as_str(),
                    linked("https://gitlab.example.com/g/p.git"),
                    None,
                )
            })
            .collect(),
    );
    let factory = Arc::new(Counting {
        inner: FakeFactory::new(),
        now: AtomicUsize::new(0),
        peak: AtomicUsize::new(0),
    });
    let mut worker = RadarWorker::spawn(factory.clone(), prepare);
    worker.start(names.iter().map(|n| job(n)).collect(), false);
    let got = answers(collect(&worker));
    assert_eq!(got.len(), 10);
    let peak = factory.peak.load(Ordering::SeqCst);
    assert!(peak > 1, "it fans out: {peak}");
    assert!(peak <= IN_FLIGHT, "at most {IN_FLIGHT}: {peak}");
}

#[test]
fn a_rate_limited_host_is_paused_for_the_rest_of_the_round() {
    let mut factory = FakeFactory::new();
    factory.fail = Some(|| HostingError::RateLimited {
        host: "gitlab.example.com".into(),
        retry_in_secs: 120,
    });
    let seen = Arc::clone(&factory.seen);
    let url = "https://gitlab.example.com/g/p.git";
    let prepare = fixed(vec![
        ("a", linked(url), None),
        ("b", linked(url), None),
        ("c", linked(url), None),
    ]);
    let (tx, rx) = mpsc::channel();
    let current = AtomicU64::new(1);
    let round = Round {
        id: 1,
        jobs: vec![job("a"), job("b"), job("c")],
        force: false,
    };
    // One at a time, so "the first one tells the others" is deterministic.
    run_round(&round, &factory, prepare.as_ref(), &current, &tx, 1);
    let got: Vec<RepoAnswer> = rx
        .try_iter()
        .filter_map(|m| match m {
            Msg::Answer(a) => Some(*a),
            _ => None,
        })
        .collect();
    assert_eq!(got.len(), 3);
    assert!(matches!(
        got[0].snapshot.problem,
        Some(Problem::RateLimited(_))
    ));
    assert!(got[0].paused.is_none());
    assert!(got[1].paused.is_some() && got[2].paused.is_some());
    let asked = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.starts_with("prs"))
        .count();
    assert_eq!(asked, 1, "the host was asked once");
}

#[test]
fn a_new_round_stops_the_old_one() {
    let url = "https://gitlab.example.com/g/p.git";
    let prepare = fixed(vec![("a", linked(url), None), ("b", linked(url), None)]);
    let (tx, rx) = mpsc::channel();
    // The round is stale before it starts: nothing is asked or sent.
    let current = AtomicU64::new(2);
    let round = Round {
        id: 1,
        jobs: vec![job("a"), job("b")],
        force: false,
    };
    let factory = FakeFactory::new();
    let seen = Arc::clone(&factory.seen);
    run_round(&round, &factory, prepare.as_ref(), &current, &tx, 4);
    assert!(rx.try_iter().next().is_none());
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn a_cache_of_another_provider_is_not_shown() {
    let mut snap = HostingSnapshot::empty(0, &linked("https://x/g/p.git"));
    snap.profile_id = Some("bb".into());
    snap.kind = Some(IntegrationKind::JiraCloud);
    snap.can_pulls = true;
    snap.prs = vec![pr(1, "b", PrState::Open)];
    let trimmed = trim_cached(snap, &linked("https://x/g/p.git"));
    assert!(trimmed.prs.is_empty() && !trimmed.can_pulls && trimmed.kind.is_none());
}

#[test]
fn the_per_repository_lists_cover_the_code_hosts_that_are_built() {
    use gitpoulpi_config::IntegrationKind as K;
    for kind in [K::Github, K::Gitlab, K::Gitea, K::Forgejo] {
        assert!(lists_requests(kind), "{kind:?}");
    }
    for kind in [K::AzureDevops, K::Bitbucket, K::Gerrit] {
        assert_eq!(
            lists_requests(kind),
            crate::build_features::kind_built(kind),
            "{kind:?}"
        );
    }
    assert!(!lists_requests(K::JiraCloud));
}

#[test]
fn duplicate_paths_are_asked_once() {
    let jobs = unique_jobs(vec![
        (PathBuf::from("/repos/a"), "a".to_string()),
        (PathBuf::from("/repos/a"), "a".to_string()),
        (PathBuf::from("/repos/b"), "b".to_string()),
    ]);
    assert_eq!(jobs.len(), 2);
}

fn github_linked() -> HostingRequest {
    let source = gitpoulpi_session::IssuesSource {
        integration: "gh".into(),
        project: "acme/app".into(),
        filter: None,
    };
    HostingRequest {
        remotes: vec![ResolveRemote::new(
            "origin",
            "https://github.com/acme/app.git",
        )],
        integrations: vec![Integration::new(
            "gh",
            IntegrationKind::Github,
            "https://github.com",
        )],
        override_id: None,
        head_branch: None,
        pr_view: PrView::Open,
        issue_view: IssueView::Open,
        issues_source: Some(source.clone()),
        pulls_source: Some(source),
        repo_root: std::path::PathBuf::new(),
    }
}

#[test]
fn the_radar_also_asks_for_what_mentions_the_user_and_github_for_its_checks() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "feat", PrState::Open), pr(2, "other", PrState::Open)];
    factory.issues = vec![issue(5), issue(6)];
    factory.mentioned_prs = vec![pr(2, "other", PrState::Open)];
    factory.mentioned_issues = vec![issue(6)];
    let seen = Arc::clone(&factory.seen);
    let prepare = fixed(vec![("app", github_linked(), None)]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    let snap = &got[0].snapshot;
    assert_eq!(
        snap.prs_mentioned
            .iter()
            .map(|p| p.number)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(
        snap.issues_mentioned
            .iter()
            .map(|i| i.number)
            .collect::<Vec<_>>(),
        vec![6]
    );
    // GitHub's list has no checks: they were asked for, and merged by number.
    assert!(snap.prs.iter().all(|p| p.checks.is_some()));
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.contains("checks: true")));
}

#[test]
fn gitlab_already_carries_the_checks_so_only_the_mentions_are_asked() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "feat", PrState::Open)];
    factory.mentioned_prs = vec![pr(1, "feat", PrState::Open)];
    let seen = Arc::clone(&factory.seen);
    let prepare = fixed(vec![(
        "app",
        linked("https://gitlab.example.com/g/p.git"),
        None,
    )]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    assert_eq!(got[0].snapshot.prs_mentioned.len(), 1);
    assert!(got[0].snapshot.prs[0].checks.is_none());
    let calls = seen.lock().unwrap().clone();
    assert!(
        calls.iter().any(|s| s.contains("mentioned: true")),
        "{calls:?}"
    );
    assert!(
        !calls.iter().any(|s| s.contains("checks: true")),
        "{calls:?}"
    );
}

#[test]
fn a_repository_with_a_problem_asks_nothing_more() {
    let mut factory = FakeFactory::new();
    factory.fail = Some(|| HostingError::RateLimited {
        host: "github.com".into(),
        retry_in_secs: 60,
    });
    let seen = Arc::clone(&factory.seen);
    let prepare = fixed(vec![("app", github_linked(), None)]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    assert!(got[0].snapshot.problem.is_some());
    assert!(got[0].snapshot.prs_mentioned.is_empty());
    let calls = seen.lock().unwrap().clone();
    assert!(
        !calls
            .iter()
            .any(|s| s.starts_with("prs") && s.contains("mentioned: true")),
        "{calls:?}"
    );
}

/// One round over `repos` GitHub repositories, with a network that costs
/// `latency` per call and a client (a credential-helper lookup in
/// production) that costs `client_cost`. Returns the time to the first
/// fetched row, the time to `Done`, the clients built and the most list calls
/// in flight at once.
fn timed_round(
    repos: usize,
    latency: Duration,
    client_cost: Duration,
    memoize: bool,
) -> (Duration, Duration, usize, usize) {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "feat", PrState::Open)];
    factory.wire.latency = latency;
    factory.client_cost = client_cost;
    factory.memoize = memoize;
    let built = Arc::clone(&factory.built);
    let peak = Arc::clone(&factory.wire.peak);
    let names: Vec<String> = (0..repos).map(|i| format!("app{i}")).collect();
    let prepare = fixed(
        names
            .iter()
            .map(|n| (n.as_str(), github_linked(), None))
            .collect(),
    );
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    let start = std::time::Instant::now();
    worker.start(names.iter().map(|n| job(n)).collect(), false);
    let mut first = None;
    loop {
        match worker.try_recv() {
            Some(Msg::Answer(a)) if first.is_none() && !a.cached => first = Some(start.elapsed()),
            Some(Msg::Done(_)) => break,
            Some(_) => {}
            None => std::thread::sleep(Duration::from_millis(1)),
        }
    }
    (
        first.unwrap_or_default(),
        start.elapsed(),
        built.load(Ordering::SeqCst),
        peak.load(Ordering::SeqCst),
    )
}

/// Prints the scan timings (`cargo test -p gitpoulpi-tui --lib
/// radar_scan_timings -- --ignored --nocapture`); perf-budgets.md has the
/// numbers.
#[test]
#[ignore = "a measurement, not a check"]
fn radar_scan_timings() {
    for (repos, latency, cost) in [(16, 100, 50), (16, 100, 0), (4, 100, 50)] {
        let (first, done, built, peak) = timed_round(
            repos,
            Duration::from_millis(latency),
            Duration::from_millis(cost),
            true,
        );
        println!(
            "{repos} repos, {latency} ms per call, {cost} ms per client: first row {} ms, done {} ms, {built} clients, {peak} calls at once",
            first.as_millis(),
            done.as_millis()
        );
    }
}

// -- the scan's cost ------------------------------------------------------
//
// perf-budgets.md "The Radar scan": what a round costs is the calls it makes
// one after the other, so these count calls and overlap, not seconds, and one
// budget is a wall clock with a wide margin.

#[test]
fn one_repositorys_lists_are_asked_together() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "feat", PrState::Open)];
    factory.wire.latency = Duration::from_millis(30);
    let peak = Arc::clone(&factory.wire.peak);
    let seen = Arc::clone(&factory.seen);
    let prepare = fixed(vec![("app", github_linked(), None)]);
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(vec![job("app")], false);
    let got = answers(collect(&worker));
    // Nothing is lost by asking together: the same five calls, the same rows.
    assert_eq!(seen.lock().unwrap().len(), 5, "{:?}", seen.lock().unwrap());
    assert_eq!(got[0].snapshot.prs.len(), 1);
    assert!(got[0].snapshot.prs[0].checks.is_some(), "the checks landed");
    // The requests and the issues (2), then the mentions, the checks and the
    // mentioned issues (3): never one at a time (it was 1).
    assert_eq!(peak.load(Ordering::SeqCst), 3);
}

#[test]
fn a_round_reads_each_token_once_and_the_next_round_reads_it_again() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(1, "feat", PrState::Open)];
    factory.memoize = true;
    let built = Arc::clone(&factory.built);
    let rounds = Arc::clone(&factory.rounds);
    let names: Vec<String> = (0..8).map(|i| format!("app{i}")).collect();
    let prepare = fixed(
        names
            .iter()
            .map(|n| (n.as_str(), github_linked(), None))
            .collect(),
    );
    let mut worker = RadarWorker::spawn(Arc::new(factory), prepare);
    worker.start(names.iter().map(|n| job(n)).collect(), false);
    collect(&worker);
    // Eight repositories, four clients each, one profile: one lookup.
    assert_eq!(built.load(Ordering::SeqCst), 1);
    worker.start(names.iter().map(|n| job(n)).collect(), false);
    collect(&worker);
    assert_eq!(built.load(Ordering::SeqCst), 2, "a refresh reads it again");
    assert_eq!(rounds.load(Ordering::SeqCst), 2);
}

#[test]
fn a_token_that_failed_to_load_is_asked_for_again() {
    use crate::hosting_task::TokenMemo;
    let memo = TokenMemo::default();
    let calls = std::cell::Cell::new(0);
    let failing = || {
        calls.set(calls.get() + 1);
        Err(HostingError::BadUrl("x".into()))
    };
    assert!(memo.get("gh", failing).is_err());
    assert!(memo.get("gh", failing).is_err());
    assert_eq!(calls.get(), 2);
    assert_eq!(memo.get("gh", || Ok("t1".into())).unwrap(), "t1");
    assert_eq!(memo.get("gh", || Ok("t2".into())).unwrap(), "t1");
    assert_eq!(memo.get("gl", || Ok("t3".into())).unwrap(), "t3");
    memo.clear();
    assert_eq!(memo.get("gh", || Ok("t4".into())).unwrap(), "t4");
}

#[test]
fn what_is_read_once_per_round_is_read_once_until_the_next_round() {
    let shared: PerRound<u32> = PerRound::default();
    let loads = std::cell::Cell::new(0);
    let load = || {
        loads.set(loads.get() + 1);
        7
    };
    assert_eq!(*shared.get(load), 7);
    assert_eq!(*shared.get(load), 7);
    assert_eq!(loads.get(), 1);
    shared.reset();
    shared.get(load);
    assert_eq!(loads.get(), 2);
}

#[test]
fn a_sixteen_repository_scan_waits_two_round_trips_per_wave() {
    // 16 repositories, four at a time: four waves of two round trips (the
    // lists, then the extras) = 8 x 40 ms = 320 ms. Asked one call at a time
    // with a token read per client it took 4 x (5 x 40 + 4 x 20) = 1120 ms.
    let (first, done, built, peak) = timed_round(
        16,
        Duration::from_millis(40),
        Duration::from_millis(20),
        true,
    );
    assert_eq!(built, 1);
    assert!(peak <= IN_FLIGHT * 3, "{peak}");
    assert!(first < Duration::from_millis(250), "first row {first:?}");
    assert!(done < Duration::from_millis(600), "done {done:?}");
}

// -- account-wide rows ---------------------------------------------------

use crate::radar::account::{AccountJob, AccountPlan, CloneIndex, Offer};
use gitpoulpi_hosting::account::{AccountPr, AccountQuery};

/// `Fixed` plus an account plan and a clone index.
struct WithAccounts {
    repos: Arc<Fixed>,
    plan: AccountPlan,
    clones: CloneIndex,
}

impl Prepare for WithAccounts {
    fn prepare(&self, repo: &Path) -> Prepared {
        self.repos.prepare(repo)
    }

    fn accounts(&self) -> AccountPlan {
        self.plan.clone()
    }

    fn known_clones(&self, _: &[PathBuf]) -> CloneIndex {
        self.clones.clone()
    }
}

fn account_rows(numbers: &[u64]) -> AccountItems {
    AccountItems {
        prs: numbers
            .iter()
            .map(|n| AccountPr {
                project: "acme/far".into(),
                clone_url: "https://gitlab.example.com/acme/far.git".into(),
                pr: pr(*n, "feat", PrState::Open),
            })
            .collect(),
        ..AccountItems::default()
    }
}

fn account_plan(cache: Option<PathBuf>) -> AccountPlan {
    let mut integration = gitlab_profile();
    integration.radar = Some(gitpoulpi_config::IntegrationRadar {
        review_requests: true,
        ..Default::default()
    });
    AccountPlan {
        all: vec![integration.clone()],
        jobs: vec![AccountJob {
            query: AccountQuery {
                review_requests: true,
                include_drafts: true,
                ..AccountQuery::default()
            },
            integration,
            label: "GitLab".into(),
        }],
        offers: vec![Offer {
            id: "other".into(),
            label: "GitHub".into(),
        }],
        cache,
    }
}

fn accounts_of(msgs: &[Msg]) -> Vec<&AccountAnswer> {
    msgs.iter()
        .filter_map(|m| match m {
            Msg::Account(a) => Some(a.as_ref()),
            _ => None,
        })
        .collect()
}

fn with_accounts(plan: AccountPlan) -> Arc<WithAccounts> {
    Arc::new(WithAccounts {
        repos: fixed(vec![(
            "app",
            linked("https://gitlab.example.com/g/p.git"),
            None,
        )]),
        plan,
        clones: CloneIndex::of(&[("gitlab.example.com", "acme/near", Path::new("/work/near"))]),
    })
}

fn stored(path: &Path, numbers: &[u64], fetched_at: u64) {
    let plan = account_plan(None);
    AccountCache::store(
        path,
        "corp",
        CachedAccount {
            fetched_at,
            query_hash: plan.jobs[0].query.fingerprint(),
            items: account_rows(numbers),
        },
    )
    .unwrap();
}

#[test]
fn the_round_opens_with_the_setup_then_the_cached_account_rows_then_the_host() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("radar-account.json");
    stored(&cache, &[1], 100);
    let mut factory = FakeFactory::new();
    factory.account = account_rows(&[2, 3]);
    let seen = Arc::clone(&factory.seen);
    let mut worker = RadarWorker::spawn(
        Arc::new(factory),
        with_accounts(account_plan(Some(cache.clone()))),
    );
    let round = worker.start(vec![job("app")], false);
    let msgs = collect(&worker);
    let Some(Msg::Setup(setup)) = msgs.first() else {
        panic!("the setup comes first: {msgs:?}");
    };
    assert_eq!(setup.round, round);
    assert_eq!(setup.active, vec!["corp".to_string()]);
    assert_eq!(setup.offers[0].id, "other");
    assert!(setup
        .clones
        .find("gitlab.example.com", "ACME/near")
        .is_some());
    let got = accounts_of(&msgs);
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].cached && got[0].items.prs.len() == 1);
    assert!(!got[1].cached && got[1].items.prs.len() == 2);
    assert_eq!(got[1].host, "gitlab.example.com");
    assert_eq!(got[1].label, "GitLab");
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .any(|s| s == "account corp r1 o0 i0 d1"));
    // What the host said is what the next round (and the CLI) read.
    let saved = AccountCache::load(&cache);
    assert_eq!(saved.entries["corp"].items.prs.len(), 2);
}

#[test]
fn a_young_account_cache_is_reused_unless_the_refresh_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("radar-account.json");
    stored(&cache, &[1], unix_now() as u64);
    let mut factory = FakeFactory::new();
    factory.account = account_rows(&[2]);
    let seen = Arc::clone(&factory.seen);
    let asked = move || {
        seen.lock()
            .unwrap()
            .iter()
            .any(|s| s.starts_with("account"))
    };
    let mut worker = RadarWorker::spawn(
        Arc::new(factory),
        with_accounts(account_plan(Some(cache.clone()))),
    );
    worker.start(vec![job("app")], false);
    let msgs = collect(&worker);
    let got = accounts_of(&msgs);
    assert_eq!(got.len(), 1, "only the cache: {got:?}");
    assert!(got[0].cached);
    assert!(!asked(), "the host was not asked");
    worker.start(vec![job("app")], true);
    let msgs = collect(&worker);
    let got = accounts_of(&msgs);
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(!got[1].cached && got[1].items.prs[0].pr.number == 2);
    assert!(asked(), "Refresh asks again");
}

#[test]
fn a_cache_of_another_question_is_not_an_answer() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("radar-account.json");
    AccountCache::store(
        &cache,
        "corp",
        CachedAccount {
            fetched_at: unix_now() as u64,
            query_hash: "another".into(),
            items: account_rows(&[1]),
        },
    )
    .unwrap();
    let mut factory = FakeFactory::new();
    factory.account = account_rows(&[2]);
    let mut worker =
        RadarWorker::spawn(Arc::new(factory), with_accounts(account_plan(Some(cache))));
    worker.start(vec![job("app")], false);
    let msgs = collect(&worker);
    let got = accounts_of(&msgs);
    assert_eq!(got.len(), 1);
    assert!(!got[0].cached && got[0].items.prs[0].pr.number == 2);
}

#[test]
fn a_host_paused_by_the_round_skips_its_account_fetch() {
    let mut factory = FakeFactory::new();
    factory.fail = Some(|| HostingError::RateLimited {
        host: "gitlab.example.com".into(),
        retry_in_secs: 120,
    });
    factory.account = account_rows(&[2]);
    let seen = Arc::clone(&factory.seen);
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("radar-account.json");
    stored(&cache, &[1], 100);
    let prepare = with_accounts(account_plan(Some(cache)));
    let (tx, rx) = mpsc::channel();
    let round = Round {
        id: 1,
        jobs: vec![job("app")],
        force: false,
    };
    let current = AtomicU64::new(1);
    run_round(&round, &factory, prepare.as_ref(), &current, &tx, 1);
    let msgs: Vec<Msg> = rx.try_iter().collect();
    let got = accounts_of(&msgs);
    let last = got.last().unwrap();
    assert!(last.paused.is_some(), "{last:?}");
    assert_eq!(last.items.prs.len(), 1, "the cache's rows are kept");
    assert!(!seen
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.starts_with("account")));
}

#[test]
fn a_rejected_token_keeps_the_cached_rows_and_asks_to_reconnect() {
    let mut factory = FakeFactory::new();
    factory.account_fail = Some(|| HostingError::Unauthorized {
        what: "gitlab.example.com".into(),
        detail: String::new(),
    });
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("radar-account.json");
    stored(&cache, &[1], 100);
    let mut worker = RadarWorker::spawn(
        Arc::new(factory),
        with_accounts(account_plan(Some(cache.clone()))),
    );
    worker.start(vec![job("app")], false);
    let msgs = collect(&worker);
    let got = accounts_of(&msgs);
    let last = got.last().unwrap();
    assert!(
        matches!(last.problem, Some(Problem::Reconnect(_))),
        "{last:?}"
    );
    assert_eq!(last.items.prs.len(), 1);
    // The failure does not overwrite the last good answer.
    assert_eq!(AccountCache::load(&cache).entries["corp"].fetched_at, 100);
}

#[test]
fn no_account_plan_means_no_account_message_and_no_clone_index() {
    let mut worker = RadarWorker::spawn(
        Arc::new(FakeFactory::new()),
        fixed(vec![(
            "app",
            linked("https://gitlab.example.com/g/p.git"),
            None,
        )]),
    );
    worker.start(vec![job("app")], false);
    let msgs = collect(&worker);
    assert!(accounts_of(&msgs).is_empty());
    let Some(Msg::Setup(setup)) = msgs.first() else {
        panic!("{msgs:?}");
    };
    assert!(setup.active.is_empty() && setup.offers.is_empty());
}
