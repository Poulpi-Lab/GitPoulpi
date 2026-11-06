//! Pull, LFS pull and workflow jobs through the daemon's real socket: they
//! can be stopped while they download (or between steps) and not past their
//! point of no return, and a stopped pull leaves the repository, its stash and
//! the undo journal exactly as it found them.
//!
//! Test hooks (set once for this binary): a pull sleeps before each download
//! and after it passes its point of no return, a workflow sleeps before each
//! step, so "downloading", "integrating" and "between steps" are states a test
//! can find.

use std::collections::HashMap;
use std::sync::{Arc, Once, OnceLock};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{OperationKind, RepoRef};
use gitpoulpi_core::{NotCancellableReason, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Event, JobInfo, JobState, Request, Response, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

const SESSION: &str = "jobs-phases-test";
const WAIT: Duration = Duration::from_secs(60);

fn setup() {
    static ONCE: Once = Once::new();
    static CONFIG: OnceLock<tempfile::TempDir> = OnceLock::new();
    ONCE.call_once(|| {
        std::env::set_var("GITPOULPI_TEST_PULL_FETCH_DELAY_MS", "4000");
        std::env::set_var("GITPOULPI_TEST_INTEGRATE_DELAY_MS", "3000");
        std::env::set_var("GITPOULPI_TEST_STEP_DELAY_MS", "800");
        let dir = CONFIG.get_or_init(|| tempfile::tempdir().unwrap());
        std::env::set_var("GITPOULPI_CONFIG_DIR", dir.path());
        std::env::set_var("GITPOULPI_DESKTOP_NO_HOOKS", "1");
        std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    });
}

struct Conn {
    read: OwnedReadHalf,
    write: OwnedWriteHalf,
    next: u64,
    responses: HashMap<u64, Response>,
}

impl Conn {
    async fn connect(endpoint: &gitpoulpi_ipc::Endpoint) -> Self {
        let stream = gitpoulpi_ipc::Client::connect(endpoint)
            .await
            .unwrap()
            .into_stream();
        let (read, write) = stream.into_split();
        Self {
            read,
            write,
            next: 1,
            responses: HashMap::new(),
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

    async fn wait(&mut self, id: u64) -> Response {
        while !self.responses.contains_key(&id) {
            let message: ServerMessage = tokio::time::timeout(WAIT, read_message(&mut self.read))
                .await
                .expect("the daemon answers within a minute")
                .unwrap()
                .expect("the daemon closed the connection");
            if let ServerMessage::Response(r) = message {
                self.responses.insert(r.request_id, r);
            } else if let ServerMessage::Event(Event::JobChanged { .. }) = message {
            }
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

async fn daemon() -> (gitpoulpi_daemon::DaemonHandle, Conn) {
    setup();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let conn = Conn::connect(&handle.endpoint()).await;
    (handle, conn)
}

/// `main` tracks `origin/main`, and the origin has a commit we lack.
fn behind() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "theirs");
    repo
}

/// Commits a change to `initial.txt` on the origin, from another clone.
fn change_initial_on_origin(repo: &FixtureRepo, contents: &str) {
    let scratch = tempfile::tempdir().unwrap();
    let remote = repo.remote_path().unwrap().to_string_lossy().to_string();
    let dir = scratch.path().join("c");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let out = std::process::Command::new("git")
            .current_dir(cwd)
            .args(["-c", "user.name=Else", "-c", "user.email=else@x.local"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{args:?}: {out:?}");
    };
    git(
        scratch.path(),
        &["clone", "--branch", "main", &remote, dir.to_str().unwrap()],
    );
    std::fs::write(dir.join("initial.txt"), contents).unwrap();
    git(&dir, &["commit", "-am", "their edit"]);
    git(&dir, &["push", "origin", "main"]);
}

fn kind_of(response: &Response) -> Option<TypedError> {
    response.result.as_ref().err().and_then(|e| e.kind.clone())
}

async fn journal(conn: &mut Conn, repo: &RepoRef) -> Value {
    conn.call(Some(repo), OperationKind::JournalList, json!({}))
        .await
        .result
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_pull_canceled_while_it_downloads_changes_nothing_and_journals_nothing() {
    let (handle, mut conn) = daemon().await;
    let repo = behind();
    let r = repo.repo_ref();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let before = journal(&mut conn, &r).await;

    let pull = conn.send(Some(&r), OperationKind::Pull, json!({})).await;
    let job = conn
        .job_where("the pull is downloading", |j| {
            j.kind == OperationKind::Pull && j.phase.as_deref() == Some("fetch")
        })
        .await;
    assert_eq!(job.state, JobState::Running);
    assert!(
        job.not_cancellable.is_none(),
        "a pull that has not started merging can be stopped: {job:?}"
    );

    let canceled = conn.cancel(job.id).await;
    assert!(canceled.result.is_ok(), "{canceled:?}");
    let answer = conn.wait(pull).await;
    assert_eq!(
        kind_of(&answer),
        Some(TypedError::Canceled { job: job.id }),
        "{answer:?}"
    );

    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head, "HEAD did not move");
    assert_eq!(repo.git(&["status", "--porcelain"]).trim(), "");
    assert!(!repo.root().join(".git").join("MERGE_HEAD").exists());
    assert!(!repo.root().join(".git").join("index.lock").exists());
    assert_eq!(
        journal(&mut conn, &r).await,
        before,
        "a canceled pull records nothing in the undo journal"
    );
    let job = conn
        .jobs()
        .await
        .into_iter()
        .find(|j| j.id == job.id)
        .unwrap();
    assert_eq!(job.state, JobState::Canceled);
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_pull_past_its_point_of_no_return_refuses_cancel_and_completes() {
    let (handle, mut conn) = daemon().await;
    let repo = behind();
    let r = repo.repo_ref();
    let theirs = repo.git_in_remote(&["rev-parse", "main"]);

    let pull = conn.send(Some(&r), OperationKind::Pull, json!({})).await;
    let job = conn
        .job_where("the pull is integrating", |j| {
            j.kind == OperationKind::Pull && j.phase.as_deref() == Some("integrate")
        })
        .await;
    assert_eq!(
        job.not_cancellable,
        Some(NotCancellableReason::Integrating),
        "{job:?}"
    );

    let refused = conn.cancel(job.id).await;
    assert_eq!(
        kind_of(&refused),
        Some(TypedError::NotCancellable {
            reason: NotCancellableReason::Integrating
        }),
        "{refused:?}"
    );

    let answer = conn.wait(pull).await;
    assert!(answer.result.is_ok(), "{answer:?}");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]).trim(), theirs.trim());
    let job = conn
        .jobs()
        .await
        .into_iter()
        .find(|j| j.id == job.id)
        .unwrap();
    assert_eq!(job.state, JobState::Done, "the refusal changed nothing");
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_pull_canceled_after_its_auto_stash_puts_the_changes_back() {
    let (handle, mut conn) = daemon().await;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    change_initial_on_origin(&repo, "their text\n");
    // A local edit of the same file: the merge is blocked, so the pull
    // stashes it, downloads again, and would pop it afterwards.
    std::fs::write(repo.root().join("initial.txt"), "my edit\n").unwrap();
    let r = repo.repo_ref();
    let head = repo.git(&["rev-parse", "HEAD"]);

    let pull = conn
        .send(
            Some(&r),
            OperationKind::Pull,
            json!({ "autostash": "always" }),
        )
        .await;
    // The first attempt is refused, the changes are stashed, and the second
    // download is under way: stoppable again, with a stash waiting.
    let deadline = Instant::now() + Duration::from_secs(30);
    while repo.git(&["stash", "list"]).trim().is_empty() {
        assert!(Instant::now() < deadline, "the pull never stashed");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    let job = conn
        .job_where("the second download is stoppable", |j| {
            j.kind == OperationKind::Pull && j.not_cancellable.is_none()
        })
        .await;
    let canceled = conn.cancel(job.id).await;
    assert!(canceled.result.is_ok(), "{canceled:?}");
    let answer = conn.wait(pull).await;
    assert_eq!(
        kind_of(&answer),
        Some(TypedError::Canceled { job: job.id }),
        "{answer:?}"
    );

    assert_eq!(
        repo.git(&["stash", "list"]).trim(),
        "",
        "the stash was popped"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("initial.txt")).unwrap(),
        "my edit\n",
        "the user's changes are back"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(repo.git(&["status", "--porcelain"]).trim(), "M initial.txt");
    assert!(!repo.root().join(".git").join("MERGE_HEAD").exists());
    let state = repo.state_dir().join("repo-state.toml");
    let recorded = std::fs::read_to_string(state).unwrap_or_default();
    assert!(
        !recorded.contains("pending_auto_stash"),
        "no pending auto-stash is left recorded: {recorded}"
    );
    // Every entry the cancel left in the journal is settled, none pending.
    let entries = journal(&mut conn, &r).await;
    assert!(
        !entries.to_string().contains("Pending"),
        "no orphaned journal entry: {entries}"
    );
    handle.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_workflow_stops_between_steps_when_canceled() {
    use gitpoulpi_core::workflow::{FlowOutcome, FlowStatus, StepStatus, WorkflowTemplate};
    let (handle, mut conn) = daemon().await;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(&exclude, "/.gitpoulpi/\n").unwrap();
    let path = gitpoulpi_session::workflow_path_for(
        repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
    );
    gitpoulpi_session::save_shared(&path, &WorkflowTemplate::GitFlow.workflow(), true).unwrap();
    let r = repo.repo_ref();

    let run = conn
        .send(
            Some(&r),
            OperationKind::FlowRun,
            json!({ "action": "start", "topic": "feature", "fields": { "slug": "Login" } }),
        )
        .await;
    // The first step (create the branch) has run; the flow waits to start the
    // second.
    let deadline = Instant::now() + Duration::from_secs(30);
    while repo
        .try_git(&["rev-parse", "--verify", "feature/login"])
        .is_none()
    {
        assert!(Instant::now() < deadline, "the first step never ran");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let job = conn
        .job_where("the flow runs", |j| {
            j.kind == OperationKind::FlowRun && j.state == JobState::Running
        })
        .await;
    assert!(job.not_cancellable.is_none(), "{job:?}");
    let canceled = conn.cancel(job.id).await;
    assert!(canceled.result.is_ok(), "{canceled:?}");

    let answer = conn.wait(run).await;
    let outcome: FlowOutcome = serde_json::from_value(answer.result.expect("a partial report"))
        .expect("the flow's own report");
    assert_eq!(outcome.status, FlowStatus::Failed, "{outcome:?}");
    assert!(outcome.resumable, "{outcome:?}");
    assert!(outcome.message.contains("canceled"), "{}", outcome.message);
    let ran = |s: StepStatus| outcome.steps.iter().filter(|r| r.status == s).count();
    assert!(ran(StepStatus::Done) >= 1, "the step that ran stays done");
    assert!(ran(StepStatus::NotRun) >= 1, "the rest did not start");
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "main",
        "the checkout step never ran"
    );
    let job = conn
        .jobs()
        .await
        .into_iter()
        .find(|j| j.id == job.id)
        .unwrap();
    assert_eq!(job.state, JobState::Canceled);
    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
