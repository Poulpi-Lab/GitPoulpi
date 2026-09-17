//! The job registry end to end, over the daemon's real socket: duplicates
//! join or queue, a job is canceled at each point of the cancel policy, and the
//! job requests are answered while the connection's own pull runs.
//!
//! `GITPOULPI_TEST_TRANSFER_DELAY_MS` (set once for this binary) makes every
//! network job sleep 1.5 s after it has its lock, so "running" and "queued
//! behind it" are states a test can find. The tests that need git to be truly
//! in flight use a `SlowRemote` (a TCP server that never answers).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{OperationKind, RepoRef};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Event, JobInfo, JobState, Request, Response, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, SlowRemote};
use serde_json::{json, Value};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

const SESSION: &str = "jobs-test";
const WAIT: Duration = Duration::from_secs(60);

fn delay_every_job() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| std::env::set_var("GITPOULPI_TEST_TRANSFER_DELAY_MS", "1500"));
}

struct Conn {
    read: OwnedReadHalf,
    write: OwnedWriteHalf,
    next: u64,
    responses: HashMap<u64, Response>,
    /// Response ids in the order they arrived.
    arrival: Vec<u64>,
    events: Vec<Event>,
}

impl Conn {
    async fn connect(port: u16) -> Self {
        let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let (read, write) = stream.into_split();
        Self {
            read,
            write,
            next: 1,
            responses: HashMap::new(),
            arrival: Vec::new(),
            events: Vec::new(),
        }
    }

    async fn send(&mut self, repo: Option<&RepoRef>, command: OperationKind, args: Value) -> u64 {
        let id = self.next;
        self.next += 1;
        let request = Request {
            id,
            session: SESSION.into(),
            repo: repo.map(|r| r.id.clone()),
            command,
            args,
        };
        write_message(&mut self.write, &request).await.unwrap();
        id
    }

    async fn pump(&mut self) {
        let message: ServerMessage = tokio::time::timeout(WAIT, read_message(&mut self.read))
            .await
            .expect("the daemon answers within a minute")
            .unwrap()
            .expect("the daemon closed the connection");
        match message {
            ServerMessage::Response(r) => {
                self.arrival.push(r.request_id);
                self.responses.insert(r.request_id, r);
            }
            ServerMessage::Event(e) => self.events.push(e),
        }
    }

    async fn wait(&mut self, id: u64) -> Response {
        while !self.responses.contains_key(&id) {
            self.pump().await;
        }
        self.responses.remove(&id).unwrap()
    }

    async fn call(
        &mut self,
        repo: Option<&RepoRef>,
        command: OperationKind,
        args: Value,
    ) -> Response {
        let id = self.send(repo, command, args).await;
        self.wait(id).await
    }

    async fn jobs(&mut self) -> Vec<JobInfo> {
        let r = self.call(None, OperationKind::JobList, json!({})).await;
        serde_json::from_value(r.result.expect("JobList")).unwrap()
    }

    /// Polls `JobList` until a job satisfies `wanted`.
    async fn job_where(&mut self, what: &str, wanted: impl Fn(&JobInfo) -> bool) -> JobInfo {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(job) = self.jobs().await.into_iter().find(|j| wanted(j)) {
                return job;
            }
            assert!(Instant::now() < deadline, "no job became: {what}");
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    }

    async fn cancel(&mut self, job: u64) -> Response {
        self.call(None, OperationKind::JobCancel, json!({ "id": job }))
            .await
    }
}

fn kind_of(response: &Response) -> Option<TypedError> {
    response.result.as_ref().err().and_then(|e| e.kind.clone())
}

fn assert_canceled(response: &Response, job: u64) {
    assert_eq!(
        kind_of(response),
        Some(TypedError::Canceled { job }),
        "{:?}",
        response.result
    );
}

async fn daemon() -> (gitpoulpi_daemon::DaemonHandle, Conn) {
    delay_every_job();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let conn = Conn::connect(handle.port).await;
    (handle, conn)
}

/// A repository with a commit on `main` that the origin does not have yet.
fn unpushed() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build()
}

fn push_args(branch: &str, force: bool) -> Value {
    json!({ "remote": "origin", "opts": { "branch": branch, "force_with_lease": force } })
}

fn slow_origin() -> (FixtureRepo, SlowRemote) {
    let slow = SlowRemote::start();
    let repo = unpushed();
    repo.git(&["remote", "set-url", "origin", &slow.url()]);
    (repo, slow)
}

async fn journal(conn: &mut Conn, repo: &RepoRef) -> Value {
    conn.call(Some(repo), OperationKind::JournalList, json!({}))
        .await
        .result
        .unwrap()
}

fn has_branch_on_origin(repo: &FixtureRepo, branch: &str) -> bool {
    !repo
        .git_in_remote(&["branch", "--list", branch])
        .trim()
        .is_empty()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_pushes_of_one_branch_join_one_job() {
    let (handle, mut conn) = daemon().await;
    let repo = unpushed();
    let r = repo.repo_ref();

    let first = conn
        .send(Some(&r), OperationKind::Push, push_args("main", false))
        .await;
    let running = conn
        .job_where("the push is running", |j| j.state == JobState::Running)
        .await;
    let second = conn
        .send(Some(&r), OperationKind::Push, push_args("main", false))
        .await;

    let a = conn.wait(first).await;
    let b = conn.wait(second).await;
    assert!(a.result.is_ok() && b.result.is_ok(), "{a:?} {b:?}");
    assert_eq!(
        a.result.unwrap(),
        b.result.unwrap(),
        "the joiner gets the same outcome"
    );

    let jobs = conn.jobs().await;
    assert_eq!(jobs.len(), 1, "one push ran, not two: {jobs:?}");
    assert_eq!(jobs[0].id, running.id);
    assert_eq!(jobs[0].state, JobState::Done);
    assert_eq!(jobs[0].joiners, 1);
    assert!(has_branch_on_origin(&repo, "main"));

    // Every state change was announced to the clients.
    let states: Vec<JobState> = conn
        .events
        .iter()
        .filter_map(|e| match e {
            Event::JobChanged { job } if job.id == running.id => Some(job.state),
            _ => None,
        })
        .collect();
    for wanted in [JobState::Queued, JobState::Running, JobState::Done] {
        assert!(
            states.contains(&wanted),
            "{wanted:?} missing from {states:?}"
        );
    }
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_push_of_another_branch_or_a_force_push_queues_behind_it() {
    let (handle, mut conn) = daemon().await;
    let repo = unpushed();
    repo.git(&["branch", "other"]);
    let r = repo.repo_ref();

    let first = conn
        .send(Some(&r), OperationKind::Push, push_args("main", false))
        .await;
    let running = conn
        .job_where("the first push runs", |j| j.state == JobState::Running)
        .await;
    let other = conn
        .send(Some(&r), OperationKind::Push, push_args("other", false))
        .await;
    let forced = conn
        .send(Some(&r), OperationKind::Push, push_args("main", true))
        .await;

    let queued = conn
        .job_where("two queued pushes", |j| j.state == JobState::Queued)
        .await;
    assert_eq!(queued.queued_after, Some(running.id));
    let jobs = conn.jobs().await;
    assert_eq!(jobs.len(), 3, "neither joined: {jobs:?}");
    assert_eq!(
        jobs.iter().filter(|j| j.state == JobState::Queued).count(),
        2,
        "{jobs:?}"
    );
    assert!(jobs.iter().all(|j| j.joiners == 0));

    for id in [first, other, forced] {
        let r = conn.wait(id).await;
        assert!(r.result.is_ok(), "{r:?}");
    }
    assert!(has_branch_on_origin(&repo, "other"));
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_queued_job_is_canceled_before_it_starts_and_nothing_is_spawned_for_it() {
    let (handle, mut conn) = daemon().await;
    let repo = unpushed();
    repo.git(&["branch", "other"]);
    let r = repo.repo_ref();

    let first = conn
        .send(Some(&r), OperationKind::Push, push_args("main", false))
        .await;
    conn.job_where("the first push runs", |j| j.state == JobState::Running)
        .await;
    let queued_push = conn
        .send(Some(&r), OperationKind::Push, push_args("other", false))
        .await;
    let queued_pull = conn.send(Some(&r), OperationKind::Pull, json!({})).await;
    let push_job = conn
        .job_where("the second push is queued", |j| {
            j.kind == OperationKind::Push && j.state == JobState::Queued
        })
        .await;
    let pull_job = conn
        .job_where("the pull is queued", |j| {
            j.kind == OperationKind::Pull && j.state == JobState::Queued
        })
        .await;
    assert!(pull_job.not_cancellable.is_none(), "a queued pull can go");

    let canceled = conn.cancel(push_job.id).await;
    assert!(canceled.result.is_ok(), "{canceled:?}");
    assert!(conn.cancel(pull_job.id).await.result.is_ok());

    // The originators are answered at once, while the first push still runs.
    let a = conn.wait(queued_push).await;
    assert_canceled(&a, push_job.id);
    assert_canceled(&conn.wait(queued_pull).await, pull_job.id);
    let jobs = conn.jobs().await;
    assert_eq!(
        jobs.iter()
            .find(|j| j.id != push_job.id && j.kind == OperationKind::Push)
            .unwrap()
            .state,
        JobState::Running,
        "the first push was still running when the others were canceled: {jobs:?}"
    );
    assert_eq!(
        jobs.iter()
            .filter(|j| j.state == JobState::Canceled)
            .count(),
        2
    );

    assert!(conn.wait(first).await.result.is_ok());
    assert!(
        !has_branch_on_origin(&repo, "other"),
        "the canceled push never ran"
    );
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_running_fetch_is_canceled_and_git_is_gone() {
    let (handle, mut conn) = daemon().await;
    let (repo, slow) = slow_origin();
    let r = repo.repo_ref();

    let fetch = conn
        .send(
            Some(&r),
            OperationKind::Fetch,
            json!({ "remote": "origin" }),
        )
        .await;
    assert!(slow.wait_accepted(1, WAIT), "git never connected");
    let job = conn
        .job_where("the fetch runs", |j| j.state == JobState::Running)
        .await;
    assert!(
        job.not_cancellable.is_none(),
        "a fetch is freely cancellable"
    );

    let canceled = conn.cancel(job.id).await;
    assert!(canceled.result.is_ok(), "{canceled:?}");
    assert_canceled(&conn.wait(fetch).await, job.id);
    assert!(
        slow.wait_closed(1, WAIT),
        "git must be dead after the cancel"
    );
    assert_eq!(conn.jobs().await[0].state, JobState::Canceled);
    handle.shutdown().await.unwrap();
}

fn clone_args(url: &str, dest: &Path) -> Value {
    json!({ "url": url, "dest": dest })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_running_clone_is_canceled_and_its_folder_is_gone() {
    let (handle, mut conn) = daemon().await;
    let slow = SlowRemote::start();
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("cloned");

    let clone = conn
        .send(None, OperationKind::Clone, clone_args(&slow.url(), &dest))
        .await;
    assert!(slow.wait_accepted(1, WAIT), "git never connected");
    assert!(dest.exists(), "git started creating the destination");
    let job = conn
        .job_where("the clone runs", |j| j.state == JobState::Running)
        .await;
    assert_eq!(job.dest.as_deref(), Some(dest.as_path()));

    assert!(conn.cancel(job.id).await.result.is_ok());
    assert_canceled(&conn.wait(clone).await, job.id);
    assert!(slow.wait_closed(1, WAIT), "git must be dead");
    assert!(!dest.exists(), "the half-made folder is removed");
    // Events and responses travel on separate paths: the event may land
    // after the response, so read on until it does.
    let deadline = Instant::now() + Duration::from_secs(30);
    while !conn
        .events
        .iter()
        .any(|e| matches!(e, Event::CloneFinished { result: Err(_), .. }))
    {
        assert!(
            Instant::now() < deadline,
            "listeners of the clone are told it ended"
        );
        conn.pump().await;
    }
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_second_clone_into_the_same_destination_is_refused() {
    let (handle, mut conn) = daemon().await;
    let slow = SlowRemote::start();
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("cloned");

    let first = conn
        .send(None, OperationKind::Clone, clone_args(&slow.url(), &dest))
        .await;
    let job = conn
        .job_where("the clone is registered", |j| {
            j.kind == OperationKind::Clone
        })
        .await;
    // The same folder, spelled with a trailing separator.
    let spelled = format!("{}/", dest.display());
    let second = conn
        .call(
            None,
            OperationKind::Clone,
            json!({ "url": slow.url(), "dest": spelled }),
        )
        .await;
    assert_eq!(
        kind_of(&second),
        Some(TypedError::JobConflict { existing: job.id }),
        "{second:?}"
    );
    assert_eq!(conn.jobs().await.len(), 1, "the refused clone made no job");

    assert!(conn.cancel(job.id).await.result.is_ok());
    assert_canceled(&conn.wait(first).await, job.id);
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn job_requests_are_answered_while_a_pull_runs_and_later_requests_keep_their_order() {
    let (handle, mut conn) = daemon().await;
    let repo = unpushed();
    let r = repo.repo_ref();

    let pull = conn.send(Some(&r), OperationKind::Pull, json!({})).await;
    let started = Instant::now();
    let listed = conn.send(None, OperationKind::JobList, json!({})).await;
    let status = conn.send(Some(&r), OperationKind::Status, json!({})).await;
    let list = conn.wait(listed).await;
    assert!(
        started.elapsed() < Duration::from_millis(1200),
        "JobList waited behind the pull ({:?})",
        started.elapsed()
    );
    let jobs: Vec<JobInfo> = serde_json::from_value(list.result.unwrap()).unwrap();
    assert!(
        jobs.iter().any(|j| j.kind == OperationKind::Pull),
        "the pull is a job"
    );
    // Cancel per pull phase is covered by `jobs_phases.rs` (B8).
    conn.wait(pull).await;
    conn.wait(status).await;
    let order: Vec<u64> = conn
        .arrival
        .iter()
        .copied()
        .filter(|id| *id == pull || *id == status)
        .collect();
    assert_eq!(order, vec![pull, status], "Status stayed behind the Pull");
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_automatic_fetch_is_skipped_while_a_push_runs() {
    let (handle, mut conn) = daemon().await;
    let repo = unpushed();
    let r = repo.repo_ref();

    let push = conn
        .send(Some(&r), OperationKind::Push, push_args("main", false))
        .await;
    conn.job_where("the push runs", |j| j.state == JobState::Running)
        .await;

    let started = Instant::now();
    let auto = conn
        .call(
            Some(&r),
            OperationKind::Fetch,
            json!({ "remote": "origin", "auto": true }),
        )
        .await;
    assert!(auto.result.is_ok(), "{auto:?}");
    assert!(
        started.elapsed() < Duration::from_millis(1000),
        "it must not wait"
    );
    assert!(
        conn.jobs()
            .await
            .iter()
            .all(|j| j.kind != OperationKind::Fetch),
        "a skipped fetch is no job"
    );

    // The same fetch, asked for by the user, queues as before.
    let manual = conn
        .send(
            Some(&r),
            OperationKind::Fetch,
            json!({ "remote": "origin" }),
        )
        .await;
    conn.job_where("the manual fetch queues", |j| {
        j.kind == OperationKind::Fetch && j.state == JobState::Queued
    })
    .await;
    assert!(conn.wait(push).await.result.is_ok());
    assert!(conn.wait(manual).await.result.is_ok());
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn canceling_a_journaled_transfer_leaves_the_journal_as_it_was() {
    let (handle, mut conn) = daemon().await;
    let (repo, slow) = slow_origin();
    let r = repo.repo_ref();
    let before = journal(&mut conn, &r).await;

    let fetch = conn
        .send(
            Some(&r),
            OperationKind::FetchRefAsBranch,
            json!({ "remote": "origin", "remote_ref": "refs/pull/1/head", "branch": "pr-1" }),
        )
        .await;
    assert!(slow.wait_accepted(1, WAIT), "git never connected");
    let job = conn
        .job_where("the fetch runs", |j| j.state == JobState::Running)
        .await;
    assert!(conn.cancel(job.id).await.result.is_ok());
    assert_canceled(&conn.wait(fetch).await, job.id);

    assert_eq!(
        journal(&mut conn, &r).await,
        before,
        "a canceled op records nothing"
    );
    assert!(
        repo.try_git(&["rev-parse", "--verify", "--quiet", "refs/heads/pr-1"])
            .is_none(),
        "the branch was never made"
    );
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
