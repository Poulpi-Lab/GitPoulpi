use super::link_guard::LinkGuard;
use super::*;
use gitpoulpi_core::domain::{CommitOptions, StageTarget};

/// The reader task must never refresh from inside itself on a
/// `RepoChanged` event: the refresh awaits a response only that task can
/// read, so the link would silently stop refreshing the moment anything
/// mutated the repo. This drives a real mutation through a second client
/// and asserts the link keeps producing outcomes afterwards, with a
/// timeout so a regression fails the test instead of hanging CI.
#[tokio::test]
async fn survives_a_repo_changed_event_without_deadlocking() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-test");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    // Bound, never `_`: a bare `_` drops the sender immediately and the
    // loop's `log_page_task_rx.recv()` arm sees the channel closed and returns
    // before the first refresh.
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-test".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    // Initial refresh from link startup.
    recv_with_timeout(&outcome_rx).await;

    // A second client mutates the repo through the daemon, which
    // broadcasts RepoChanged -- the path that would deadlock a reader
    // that refreshed inline.
    let mut mutator = gitpoulpi_ipc::Client::connect(&daemon_handle.endpoint())
        .await
        .unwrap();
    std::fs::write(repo.root.join("new.txt"), "hello").unwrap();
    let stage = mutator
        .call(&Request {
            id: 1,
            session: "daemon-link-test".into(),
            repo: Some(repo.id.clone()),
            command: OperationKind::StageFile,
            args: serde_json::json!({ "target": StageTarget::File("new.txt".into()) }),
        })
        .await
        .unwrap();
    assert!(stage.result.is_ok(), "stage failed: {:?}", stage.result);
    let commit = mutator
        .call(&Request {
            id: 2,
            session: "daemon-link-test".into(),
            repo: Some(repo.id.clone()),
            command: OperationKind::Commit,
            args: serde_json::json!({ "message": "add new.txt", "opts": CommitOptions::default() }),
        })
        .await
        .unwrap();
    assert!(commit.result.is_ok(), "commit failed: {:?}", commit.result);

    // The event-triggered refresh, and then a manual one, must both
    // still arrive -- proving the reader task is still reading.
    recv_with_timeout(&outcome_rx).await;
    let _ = manual_refresh_tx.send(());
    recv_with_timeout(&outcome_rx).await;

    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// The whole of hunk-level staging, end to end over the real IPC
/// path: ask the link for the working-tree diff, pick a hunk out of
/// it, submit `Action::StageHunk`, and check that the index now
/// contains *that hunk and only that hunk*.
///
/// This is what "partial staging is really partial" means, and it is
/// asserted against the daemon's own diff rather than against a
/// message the TUI printed to itself. It also drives the new
/// working-diff request arm of the link loop, which is exactly the
/// kind of addition that can reintroduce the reader-task deadlock the
/// test above exists for - so it waits the same way (`recv_polled`).
#[tokio::test]
async fn staging_one_hunk_through_the_link_leaves_the_other_unstaged() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    // 20 lines, so two edits far apart land in two separate hunks.
    let original: String = (1..=20).map(|n| format!("line{n}\n")).collect();
    std::fs::write(repo.root.join("multi.txt"), &original).unwrap();
    for args in [vec!["add", "multi.txt"], vec!["commit", "-m", "multi"]] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(&repo.root)
            .output()
            .unwrap();
    }
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    lines[1] = "line2-changed".to_string();
    lines[18] = "line19-changed".to_string();
    std::fs::write(repo.root.join("multi.txt"), lines.join("\n") + "\n").unwrap();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-hunk-test");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    // Bound, never `_`: a bare `_` drops the sender immediately and the
    // loop's `log_page_task_rx.recv()` arm sees the channel closed and returns
    // before the first refresh.
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-hunk-test".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));
    recv_with_timeout(&outcome_rx).await;

    // What the Diff view shows for the Unstaged half.
    working_diff_tx
        .send(FileDiffRequest::whole(DiffTarget::WorkingTreeVsIndex))
        .unwrap();
    let unstaged = recv_diff(&working_diff_rx).await;
    let multi = unstaged
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .expect("multi.txt is modified");
    assert_eq!(
        multi.hunks.len(),
        2,
        "two edits 17 lines apart must be two hunks"
    );
    let first = multi.hunks[0].id;

    // The Diff view asks for the open file alone.
    working_diff_tx
        .send(FileDiffRequest::file(
            DiffTarget::WorkingTreeVsIndex,
            "multi.txt".into(),
        ))
        .unwrap();
    let only = recv_diff(&working_diff_rx).await;
    assert_eq!(only.len(), 1, "one file asked, one file answered");
    assert_eq!(only[0].path.to_str(), Some("multi.txt"));
    assert_eq!(only[0].hunks.len(), 2);

    // Exactly what the view's `Stage hunk` button submits.
    action_tx
        .send((
            None,
            Action::StageHunk {
                path: "multi.txt".into(),
                hunk: first,
            },
        ))
        .unwrap();
    // The daemon broadcasts RepoChanged on success, which is what
    // makes the refresh below arrive at all.
    recv_with_timeout(&outcome_rx).await;
    assert!(
        action_error_rx.try_recv().is_err(),
        "the stage itself failed"
    );

    // The outcome received above can be the first load's own follow-up
    // (worktrees, LFS), which arrives on its own after the first refresh:
    // it does not prove the stage has landed. So ask for the staged diff
    // until the stage shows in it.
    let mut staged_multi = None;
    for _ in 0..poll_budget() {
        working_diff_tx
            .send(FileDiffRequest::whole(DiffTarget::IndexVsHead))
            .unwrap();
        let staged = recv_diff(&working_diff_rx).await;
        staged_multi = staged
            .into_iter()
            .find(|f| f.path.to_str() == Some("multi.txt"));
        if staged_multi.is_some() {
            break;
        }
        tokio::time::sleep(POLL).await;
    }
    let staged_multi = staged_multi.expect("multi.txt is now partially staged");
    assert_eq!(staged_multi.hunks.len(), 1, "one hunk staged, not both");
    let staged_text: String = staged_multi.hunks[0]
        .lines
        .iter()
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        staged_text.contains("line2-changed"),
        "the staged hunk is the wrong one: {staged_text}"
    );
    assert!(
        !staged_text.contains("line19-changed"),
        "the second hunk leaked into the index: {staged_text}"
    );

    // ...and the other edit is still sitting in the working tree.
    working_diff_tx
        .send(FileDiffRequest::whole(DiffTarget::WorkingTreeVsIndex))
        .unwrap();
    let left = recv_diff(&working_diff_rx).await;
    let left_multi = left
        .iter()
        .find(|f| f.path.to_str() == Some("multi.txt"))
        .expect("multi.txt still has unstaged changes");
    let left_text: String = left_multi
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(left_text.contains("line19-changed"), "got {left_text}");
    assert!(!left_text.contains("line2-changed"), "got {left_text}");

    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// All three File Insight reads, end to end over the real IPC path.
///
/// Asserted here rather than only on screen because the overlay is
/// driven by one request type with three shapes, and a mistake in any
/// one of them (a `LogOptions` the backend rejects, a `BlobSource`
/// pointing at the wrong thing) looks identical from the outside: an
/// overlay that never fills in.
#[tokio::test]
async fn file_insight_answers_content_blame_and_history_for_a_path() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("f.txt", "first\nsecond\n", "original")
        .unwrap()
        .commit_file("f.txt", "first\nCHANGED\n", "change the second line")
        .unwrap()
        .commit_file("other.txt", "x\n", "unrelated")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-insight-test");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, file_insight_rx) = std_mpsc::channel();
    // Bound, never `_`: a bare `_` drops the sender immediately and the
    // loop's `log_page_task_rx.recv()` arm sees the channel closed and returns
    // before the first refresh.
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-insight-test".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));
    let head = recv_head(&outcome_rx).await;

    let ask = |kind| FileInsightRequest {
        path: PathBuf::from("f.txt"),
        at: head,
        kind,
        worktree: false,
        staged: false,
    };

    file_insight_tx.send(ask(FileInsightKind::Content)).unwrap();
    match recv_insight(&file_insight_rx).await {
        FileInsightPayload::Content(blob) => {
            assert_eq!(blob.content.lines(), ["first", "CHANGED"])
        }
        _ => panic!("Content request answered with the wrong payload"),
    }

    file_insight_tx.send(ask(FileInsightKind::Blame)).unwrap();
    match recv_insight(&file_insight_rx).await {
        FileInsightPayload::Blame(blame) => {
            let lines = blame.content.lines();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[1].content, "CHANGED");
            assert_ne!(
                lines[0].commit, lines[1].commit,
                "the two lines came from different commits"
            );
        }
        _ => panic!("Blame request answered with the wrong payload"),
    }

    file_insight_tx
        .send(ask(FileInsightKind::HistoryPage(0)))
        .unwrap();
    match recv_insight(&file_insight_rx).await {
        FileInsightPayload::HistoryPage(page) => {
            let summaries: Vec<&str> = page.entries.iter().map(|e| e.summary.as_str()).collect();
            assert_eq!(
                summaries,
                ["change the second line", "original"],
                "File History must be this path's commits, newest first, \
                 and must not be empty"
            );
            assert!(!page.more);
        }
        _ => panic!("HistoryPage request answered with the wrong payload"),
    }

    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// The log page limit is process-wide: a test that reads or sets it holds
/// this for its whole run.
static LOG_LIMIT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The wait budget of the helpers below is counted in **polls** (`POLLS`,
/// about 60 s), not derived from `Instant::now()`: a wall-clock bound turns
/// a correctness test into an assertion about machine speed. Under load
/// (several builds at once) the first refresh can take ~8 s, because
/// spawning git is what slows, while a tokio sleep barely stretches.
///
/// These tests check that the link **answers at all**; a deadlocked link
/// never answers however long it waits, so the bound only decides how long
/// a genuine deadlock takes to be reported, and a false failure under load
/// teaches people to re-run instead of look. The deadlock test stays
/// falsifiable: `GITPOULPI_TEST_POLLS=1` makes it fail, and that variable
/// also lets anyone on a pathologically slow box raise the budget.
///
/// `WALL_CLOCK_BACKSTOP` keeps a wedged scheduler from turning a failing
/// test into a hanging one; it is far too generous to fire on a busy
/// machine.
const POLL: std::time::Duration = std::time::Duration::from_millis(20);
const POLLS: u32 = 3_000; // ~60 s of sleeping on an idle machine
const WALL_CLOCK_BACKSTOP: std::time::Duration = std::time::Duration::from_secs(120);

fn poll_budget() -> u32 {
    std::env::var("GITPOULPI_TEST_POLLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(POLLS)
}

/// Polls `rx` until it yields, or until the budget above runs out.
/// `what` names the thing being waited for, so a failure says which
/// channel went quiet rather than just "timed out".
async fn recv_polled<T>(rx: &std_mpsc::Receiver<T>, what: &str) -> T {
    let started = std::time::Instant::now();
    let budget = poll_budget();
    for _ in 0..budget {
        match rx.try_recv() {
            Ok(value) => return value,
            Err(std_mpsc::TryRecvError::Empty) => {
                assert!(
                    started.elapsed() < WALL_CLOCK_BACKSTOP,
                    "waiting for {what} exceeded the {WALL_CLOCK_BACKSTOP:?} \
                     backstop - the scheduler, not just the link, is wedged"
                );
                tokio::time::sleep(POLL).await;
            }
            Err(std_mpsc::TryRecvError::Disconnected) => {
                panic!("daemon_link's {what} channel closed unexpectedly")
            }
        }
    }
    panic!(
        "timed out waiting for {what} after {budget} polls ({:?} of wall \
         clock) - the link is deadlocked",
        started.elapsed()
    );
}

async fn recv_head(rx: &std_mpsc::Receiver<RefreshOutcome>) -> Oid {
    recv_polled(rx, "the first refresh")
        .await
        .log_result
        .expect("log failed")
        .nodes[0]
        .id
}

async fn recv_insight(rx: &std_mpsc::Receiver<FileInsightOutcome>) -> FileInsightPayload {
    recv_polled(rx, "a file-insight answer")
        .await
        .result
        .expect("the file-insight request failed")
}

async fn recv_diff(rx: &std_mpsc::Receiver<WorkingDiffOutcome>) -> Vec<FileDiff> {
    recv_polled(rx, "a working-tree diff")
        .await
        .files
        .expect("the diff request failed")
}

async fn recv_with_timeout(rx: &std_mpsc::Receiver<RefreshOutcome>) {
    recv_polled(rx, "a refresh outcome").await;
}

/// A stage's `RepoChanged` carries an index-only scope, and the link
/// answers it with a status read alone: no `Log`, `Refs`, `Remotes`,
/// `LfsStatus` and so on (1.5-10 s of reads, for a change that touched
/// the index). Driven against a fake daemon that records every request
/// and pushes the scoped event after the `StageFile` it is sent.
#[tokio::test]
async fn a_stage_file_event_refreshes_status_without_log_or_lfs() {
    let commands: Arc<std::sync::Mutex<Vec<OperationKind>>> = Default::default();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::clone(&commands);
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let repo_id = repo.id.clone();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        while let Ok(Some(request)) = read_message::<_, Request>(&mut stream).await {
            seen.lock().unwrap().push(request.command);
            let result = match request.command {
                OperationKind::Hello | OperationKind::LfsStatus => {
                    Err(gitpoulpi_ipc::ResponseError {
                        message: "not in this fake".into(),
                        unsupported: true,
                        kind: None,
                        detail: None,
                        context: None,
                        auth_required: false,
                    })
                }
                OperationKind::Status => Ok(serde_json::to_value(WorkingTree::default()).unwrap()),
                OperationKind::Log => Ok(serde_json::to_value(CommitGraph::default()).unwrap()),
                OperationKind::SshCommand
                | OperationKind::RebaseState
                | OperationKind::OperationState
                | OperationKind::StageFile => Ok(serde_json::Value::Null),
                _ => Ok(serde_json::json!([])),
            };
            let staged = request.command == OperationKind::StageFile;
            let reply = ServerMessage::Response(Response {
                request_id: request.id,
                result,
            });
            write_message(&mut stream, &reply).await.unwrap();
            if staged {
                let event = ServerMessage::Event(Event::RepoChanged {
                    repo: repo_id.clone(),
                    scope: ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFLICTS,
                });
                write_message(&mut stream, &event).await.unwrap();
            }
        }
    });

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let _link = LinkGuard::spawn(link_loop_with_port(
        gitpoulpi_ipc::Endpoint::new(port, None),
        "scoped-refresh-test".to_string(),
        repo,
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    let first = recv_polled(&outcome_rx, "the first refresh").await;
    assert_eq!(first.scope, FIRST_LOAD_NOW);
    assert!(first.log_result.is_ok(), "the first load reads the log");
    let later = recv_polled(&outcome_rx, "the rest of the first load").await;
    assert_eq!(later.scope, FIRST_LOAD_LATER);
    let mark = commands.lock().unwrap().len();

    action_tx
        .send((None, Action::StageFile(PathBuf::from("a.txt"))))
        .unwrap();
    let scoped = recv_polled(&outcome_rx, "the scoped refresh").await;
    assert_ne!(scoped.scope, ChangeScope::ALL, "not the 2 s poll");
    assert!(scoped.status_result.is_ok());

    let sent: Vec<OperationKind> = commands.lock().unwrap()[mark..].to_vec();
    assert!(sent.contains(&OperationKind::StageFile), "{sent:?}");
    assert!(sent.contains(&OperationKind::Status), "{sent:?}");
    for wide in [
        OperationKind::Log,
        OperationKind::LfsStatus,
        OperationKind::LfsFiles,
        OperationKind::Refs,
        OperationKind::Stashes,
        OperationKind::Worktrees,
        OperationKind::Remotes,
        OperationKind::SshCommand,
    ] {
        assert!(!sent.contains(&wide), "{wide:?} was sent: {sent:?}");
    }
}

/// An action is never queued behind a refresh. A fake daemon
/// answers `Log` only after 2 s (a slow refresh in flight); a
/// `StageAll` submitted meanwhile reaches the daemon within 50 ms, the
/// stale refresh's result is dropped (the optimistic rows would
/// otherwise be undone), and the refresh that follows the mutation is
/// delivered. Falsified by awaiting the refresh inline in `link_loop`:
/// the `StageAll` then arrives about 2 s late.
#[tokio::test]
async fn a_stage_all_is_sent_at_once_while_a_slow_refresh_is_in_flight() {
    let slow_log = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stage_all_seen_at: Arc<std::sync::Mutex<Option<std::time::Instant>>> = Default::default();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let repo_id = repo.id.clone();
    {
        let slow_log = Arc::clone(&slow_log);
        let seen_at = Arc::clone(&stage_all_seen_at);
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (mut read_half, write_half) = stream.into_split();
            let write_half = Arc::new(AsyncMutex::new(write_half));
            while let Ok(Some(request)) = read_message::<_, Request>(&mut read_half).await {
                let write_half = Arc::clone(&write_half);
                let slow_log = Arc::clone(&slow_log);
                let seen_at = Arc::clone(&seen_at);
                let repo_id = repo_id.clone();
                // One task per request: a delayed `Log` must not hold
                // up the answers behind it.
                tokio::spawn(async move {
                    if request.command == OperationKind::StageAll {
                        *seen_at.lock().unwrap() = Some(std::time::Instant::now());
                    }
                    if request.command == OperationKind::Log && slow_log.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    let result = match request.command {
                        OperationKind::Hello | OperationKind::LfsStatus => {
                            Err(gitpoulpi_ipc::ResponseError {
                                message: "not in this fake".into(),
                                unsupported: true,
                                kind: None,
                                detail: None,
                                context: None,
                                auth_required: false,
                            })
                        }
                        OperationKind::Status => {
                            Ok(serde_json::to_value(WorkingTree::default()).unwrap())
                        }
                        OperationKind::Log => {
                            Ok(serde_json::to_value(CommitGraph::default()).unwrap())
                        }
                        OperationKind::SshCommand
                        | OperationKind::RebaseState
                        | OperationKind::OperationState
                        | OperationKind::StageAll => Ok(serde_json::Value::Null),
                        _ => Ok(serde_json::json!([])),
                    };
                    let staged = request.command == OperationKind::StageAll;
                    let mut write = write_half.lock().await;
                    let reply = ServerMessage::Response(Response {
                        request_id: request.id,
                        result,
                    });
                    write_message(&mut *write, &reply).await.unwrap();
                    if staged {
                        let event = ServerMessage::Event(Event::RepoChanged {
                            repo: repo_id,
                            scope: ChangeScope::INDEX | ChangeScope::WORKTREE,
                        });
                        write_message(&mut *write, &event).await.unwrap();
                    }
                });
            }
        });
    }

    let (manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let activity = Arc::new(crate::activity::Activity::default());
    let _link = LinkGuard::spawn(link_loop_with_port(
        gitpoulpi_ipc::Endpoint::new(port, None),
        "non-queueing-test".to_string(),
        repo,
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Arc::clone(&activity),
        },
        Default::default(),
    ));

    let first = recv_polled(&outcome_rx, "the first refresh").await;
    assert_eq!(first.scope, FIRST_LOAD_NOW);
    let later = recv_polled(&outcome_rx, "the rest of the first load").await;
    assert_eq!(later.scope, FIRST_LOAD_LATER);

    // A full refresh whose `Log` takes 2 s...
    slow_log.store(true, Ordering::SeqCst);
    manual_refresh_tx.send(()).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    // ...and a StageAll submitted while it runs.
    activity.submitted(&Action::StageAll);
    let submitted_at = std::time::Instant::now();
    action_tx.send((None, Action::StageAll)).unwrap();
    let mut waited = Duration::ZERO;
    let seen = loop {
        if let Some(at) = *stage_all_seen_at.lock().unwrap() {
            break at;
        }
        assert!(
            waited < Duration::from_secs(5),
            "StageAll never reached the daemon"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
        waited += Duration::from_millis(5);
    };
    let queued_for = seen.saturating_duration_since(submitted_at);
    assert!(
        queued_for < Duration::from_millis(50),
        "StageAll waited {queued_for:?} behind the refresh"
    );

    // The 2 s refresh started before the mutation: its result is
    // dropped; the next outcome is the one that followed the StageAll.
    slow_log.store(false, Ordering::SeqCst);
    let outcome = recv_polled(&outcome_rx, "the refresh after the stage").await;
    assert_eq!(
        outcome.generation,
        activity.generation(),
        "the delivered refresh must be current, not the stale one"
    );
    assert!(
        submitted_at.elapsed() < Duration::from_millis(1900),
        "the stale refresh was delivered (it takes 2 s)"
    );
}

/// Lazy loading of the graph, end to end over the real IPC path: a
/// first `Log` bounded to three commits, then two pages asked for
/// through `request_log_page`, then the assertion that matters --
/// **the pages tile onto the initial load exactly**, giving the same
/// list the unpaged walk gives.
///
/// Driven through the link rather than against `LazyLog` alone
/// because everything `LazyLog`'s own tests cannot see lives here:
/// that `skip`/`limit` survive the JSON round trip (the field is
/// `#[serde(default)]` and additive, so a spelling mistake would
/// silently page from zero every time), that the page walk carries
/// the *same* `all_refs` the refresh does, and that the answer comes
/// back on `log_page_rx` tagged with the request it belongs to.
///
/// Red if `skip` is dropped from the page request (`skip: 0`): every page
/// re-delivers the head of history.
#[tokio::test]
async fn a_paged_log_tiles_onto_the_initial_load_over_the_real_link() {
    let _limit = LOG_LIMIT_LOCK.lock().await;
    let mut builder = gitpoulpi_testkit::FixtureRepoBuilder::new().unwrap();
    for i in 0..9 {
        builder = builder.commit(&format!("c{i}")).unwrap();
    }
    let fixture = builder.build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-paging");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    // The initial load is three commits, so the nine-commit fixture
    // needs real paging to be seen whole.
    set_log_page_limit(3);

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-paging".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    let first = recv_polled(&outcome_rx, "the initial refresh").await;
    let mut held = first
        .log_result
        .expect("the initial log must succeed")
        .nodes;
    assert_eq!(held.len(), 3, "the initial load must be bounded");

    let settings = crate::graph::LogLoadSettings {
        initial_commits: 3,
        lazy_load: true,
        page_commits: 4,
        max_commits: None,
    };
    let mut lazy = crate::graph::LazyLog::new(settings);
    lazy.on_refresh(held.len());

    // Two pages of four: the nine-commit fixture holds three (the
    // initial load) plus four (page one) plus two (page two, short of
    // the four asked for) -- and that shortfall is how the loader
    // learns history has ended.
    for expected in [4usize, 2] {
        let request = lazy
            .next_page(0, 20)
            .expect("the loader must want another page");
        lazy.mark_requested(request);
        let _ = log_page_tx.send(request);
        let page = recv_polled(&log_page_rx, "a lazily-loaded log page").await;
        assert_eq!(page.request, request, "a page must answer its request");
        let nodes = page.result.expect("a page request must succeed").nodes;
        assert_eq!(nodes.len(), expected, "page sizes: {request:?}");
        assert!(lazy.on_page(page.request, nodes, &mut held));
    }
    assert!(lazy.is_exhausted(), "a short page ends the walk");
    assert_eq!(lazy.next_page(0, 20), None);

    // The whole point: what paging produced is what one unpaged walk
    // would have produced, in the same order, with nothing repeated
    // and nothing missing.
    let mut whole = gitpoulpi_ipc::Client::connect(&daemon_handle.endpoint())
        .await
        .unwrap();
    let unpaged = whole
        .call(&Request {
            id: 1,
            session: "daemon-link-paging".into(),
            repo: Some(repo.id.clone()),
            command: OperationKind::Log,
            args: serde_json::json!({
                "opts": LogOptions { all_refs: true, ..LogOptions::default() }
            }),
        })
        .await
        .unwrap();
    let unpaged: CommitGraph = serde_json::from_value(unpaged.result.unwrap()).unwrap();
    assert_eq!(
        held.iter().map(|n| n.id).collect::<Vec<_>>(),
        unpaged.nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
        "the paged list and the unpaged walk must be the same history"
    );

    set_log_page_limit(gitpoulpi_config::DEFAULT_INITIAL_COMMITS_IN_GRAPH);
    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// **Stashing clears the working tree, so the WIP row goes away.** After
/// stashing, the stash must appear in STASHES, the Log must drop its WIP
/// row and the right pane must stop counting working-directory changes.
///
/// Driven exactly the way the toolbar's Stash button drives it --
/// `Action::StashPush` into the link - with a working tree holding
/// both kinds of change a user has: edits to tracked files *and* a
/// new, untracked file.
#[tokio::test]
async fn stashing_through_the_link_leaves_a_clean_working_tree() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "a")
        .unwrap()
        .commit_file("b.txt", "b\n", "b")
        .unwrap()
        .write_worktree("a.txt", b"a changed\n")
        .unwrap()
        .write_worktree("b.txt", b"b changed\n")
        .unwrap()
        .write_worktree("new.txt", b"brand new\n")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-stash-test");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-stash-test".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    let before = recv_polled(&outcome_rx, "the first refresh").await;
    assert_eq!(
        before.status_result.as_ref().map(|t| t.files.len()).ok(),
        Some(3),
        "precondition: two edits and one new file"
    );

    action_tx.send((None, Action::StashPush)).unwrap();
    // Refreshes until the one that sees the stash -- the event
    // refresh can race a periodic one, so do not assume it is next.
    let after = loop {
        let outcome = recv_polled(&outcome_rx, "the refresh after the stash").await;
        if outcome.stashes_result.as_ref().is_ok_and(|s| !s.is_empty()) {
            break outcome;
        }
    };
    assert!(
        action_error_rx.try_recv().is_err(),
        "the stash itself failed"
    );
    // Both read off this one value: the Log's WIP row (`has_wip_row`) and the pane's "N file changes in
    // working directory" (the same `files.len()`).
    assert!(
        !crate::has_wip_row(&after.status_result),
        "the WIP row must go once the stash has taken every change"
    );
    // And the Log draws the new stash from this same refresh -- no
    // second request, no git call per frame: the stash names its base,
    // the base is among the refresh's commits, so it gets a row (in
    // place of the WIP row it replaces, right above HEAD).
    let order = crate::log_rows_now(
        &after.log_result,
        &after.status_result,
        &after.stashes_result,
        true,
        &Default::default(),
        &after.refs_result,
    );
    let head = after.log_result.as_ref().expect("log failed").nodes[0].id;
    let stash = &after.stashes_result.as_ref().unwrap()[0];
    assert_eq!(stash.base, Some(head), "a stash made on HEAD names HEAD");
    assert_eq!(
        order.slots().len(),
        1,
        "the refresh after a stash must carry everything its graph row needs"
    );
    assert_eq!(order.entry(0), crate::graph::LogEntry::Stash(0));
    assert_eq!(order.entry(1), crate::graph::LogEntry::Commit(0));
    let left: Vec<_> = after
        .status_result
        .expect("status failed")
        .files
        .into_iter()
        .map(|f| f.path)
        .collect();
    assert!(
        left.is_empty(),
        "after a stash the working tree must be clean -- the Log's WIP \
         row and the pane's count read this - but it still has {left:?}"
    );

    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// `general.delete_orig_files_after_merge`'s sweep: every `.orig`
/// under the root is gone, a nested one is found too, `.git` is
/// never descended into (a real `.git` can itself contain files that
/// happen to end in `.orig` in some setups, and walking it is also
/// needless work), and a file that is merely named like an `.orig`
/// elsewhere in its name (`orig.txt`) survives.
#[test]
fn cleanup_orig_files_removes_every_orig_file_but_leaves_git_and_lookalikes_alone() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.txt.orig"), "backup").unwrap();
    std::fs::write(root.join("orig.txt"), "not an orig file").unwrap();
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("sub").join("b.orig"), "backup").unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join(".git").join("c.orig"), "must survive").unwrap();

    let removed = cleanup_orig_files(root);

    assert_eq!(removed, 2, "one at the root, one nested");
    assert!(!root.join("a.txt.orig").exists());
    assert!(!root.join("sub").join("b.orig").exists());
    assert!(root.join("orig.txt").exists(), "not an .orig extension");
    assert!(
        root.join(".git").join("c.orig").exists(),
        ".git is never descended into"
    );
}

/// A drag-and-drop merge (`Action::CheckoutThen`) whose checkout carries the
/// local changes along but whose merge is then blocked by them, with
/// `general.auto_stash = ask`: the merge's refusal must reach the auto-stash
/// dialog (as the whole `CheckoutThen`) instead of a channel nobody read, and
/// answering yes must finish the checkout, the merge and the pop.
#[tokio::test]
async fn a_merge_blocked_after_a_dropped_checkout_asks_instead_of_vanishing() {
    let _env = super::ticket_tests::AUTOSTASH_ENV.lock().await;
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "ask");
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base")
        .unwrap()
        .branch("target")
        .unwrap()
        .branch("side")
        .unwrap()
        .checkout("side")
        .unwrap()
        .commit_file("a.txt", "1 side\n2\n3\n4\n5\n", "side edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .write_worktree("a.txt", b"1\n2\n3\n4\n5 local\n")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let side = backend
        .refs(&repo)
        .unwrap()
        .into_iter()
        .find(|r| {
            matches!(&r.kind, gitpoulpi_core::domain::RefKind::LocalBranch { name } if name == "side")
        })
        .expect("the side branch");
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-nested-autostash");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, _outcome_rx) = std_mpsc::channel();
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (autostash_tx, autostash_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-nested-autostash".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    let drop_merge = Action::CheckoutThen {
        branch: "target".to_string(),
        then: Box::new(Action::Merge(side)),
    };
    action_tx.send((None, drop_merge.clone())).unwrap();
    let needed = recv_polled(&autostash_rx, "the auto-stash dialog's request").await;
    assert_eq!(needed.operation, "merge");
    assert!(matches!(needed.action, Action::CheckoutThen { .. }));
    assert!(action_error_rx.try_recv().is_err());

    // Yes: stash, (re)check out, merge, pop.
    action_tx
        .send((None, Action::AutoStashed(Box::new(needed.action))))
        .unwrap();
    let file = fixture.root().join("a.txt");
    for _ in 0..poll_budget() {
        if std::fs::read_to_string(&file).is_ok_and(|t| t == "1 side\n2\n3\n4\n5 local\n")
            && fixture
                .try_git(&["stash", "list"])
                .is_some_and(|l| l.is_empty())
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "1 side\n2\n3\n4\n5 local\n"
    );
    // `try_git`: a stash list raced with the daemon's pop fails with `bad revision
    // 'refs/stash'`; the loop above already waited for the empty list.
    assert_eq!(fixture.try_git(&["stash", "list"]).as_deref(), Some(""));
    assert_eq!(
        fixture.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "target"
    );
    assert!(action_error_rx.try_recv().is_err());
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// **An idle poll rebuilds nothing.** Two polls that read the same state
/// must leave the window's held Log/Refs/Stashes alone and so its
/// generation counters where they were - the Log layout cache is keyed on
/// them, and a rebuild is milliseconds on a few thousand commits. Driven
/// over the real link against a real daemon, applying every outcome the
/// way `window/apply.rs` does (`ReadHashes::apply`), with a real
/// `LogLayoutCache` behind it.
#[tokio::test]
async fn two_polls_with_no_change_cause_no_layout_rebuild() {
    let _limit = LOG_LIMIT_LOCK.lock().await;
    let mut builder = gitpoulpi_testkit::FixtureRepoBuilder::new().unwrap();
    for i in 0..4 {
        builder = builder.commit(&format!("c{i}")).unwrap();
    }
    let fixture = builder.build();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn GitBackend> = Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new());
    let daemon = gitpoulpi_daemon::Daemon::new("daemon-link-idle-polls");
    let daemon_handle = daemon.spawn(backend).await.unwrap();

    let (manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, _action_error_rx) = std_mpsc::channel();
    let (action_message_tx, _action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, _push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, _commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, _working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, _file_insight_rx) = std_mpsc::channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, _log_page_rx) = std_mpsc::channel();
    let link = LinkGuard::spawn(link_loop_with_port(
        daemon_handle.endpoint(),
        "daemon-link-idle-polls".to_string(),
        repo.clone(),
        LinkChannels {
            manual_refresh_rx,
            action_rx,
            commit_files_task_rx,
            working_diff_task_rx,
            file_insight_task_rx,
            merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
            outcome_tx,
            action_error_tx,
            action_message_tx,
            push_rejected_tx,
            autostash_tx: std_mpsc::channel().0,
            commit_files_out_tx,
            working_diff_out_tx,
            file_insight_out_tx,
            merge_out_tx: std_mpsc::channel().0,
            log_page_task_rx,
            log_page_out_tx,
            stats_worker: None,
            split_worker: None,
            log_depth: Arc::new(AtomicUsize::new(0)),
            log_filter: Default::default(),
            lfs_gate: Default::default(),
            activity: Default::default(),
        },
        Default::default(),
    ));

    let mut hashes = ReadHashes::default();
    let (mut log_gen, mut refs_gen, mut stashes_gen) = (0u64, 0u64, 0u64);
    let mut layout = crate::graph::LogLayoutCache::default();
    let (mut log, mut refs, mut stashes) = (skipped(), skipped(), skipped());
    let solo = std::collections::HashSet::new();
    let hidden = std::collections::HashSet::new();
    let mut fingerprints = Vec::new();

    // The first load (its follow-up outcome included), then three full polls (the first takes the LFS snapshot, which the
    // fingerprint sees).
    // Every outcome is applied; the polls are the ones with the whole scope.
    let mut polls = 0;
    let mut asked = false;
    while polls < 3 {
        if asked {
            manual_refresh_tx.send(()).unwrap();
        }
        let outcome = recv_polled(&outcome_rx, "a refresh outcome").await;
        if outcome.scope == ChangeScope::ALL && asked {
            polls += 1;
            fingerprints.push(outcome.fingerprint);
        }
        asked = true;
        let changed = hashes.apply(outcome.read_hashes);
        if changed.log {
            log = outcome.log_result;
            log_gen += 1;
        }
        if changed.refs {
            refs = outcome.refs_result;
            refs_gen += 1;
        }
        if changed.stashes {
            stashes = outcome.stashes_result;
            stashes_gen += 1;
        }
        layout.update(
            &crate::graph::LayoutInputs {
                log: &log,
                log_gen,
                refs: &refs,
                refs_gen,
                stashes: &stashes,
                stashes_gen,
                solo: &solo,
                hidden: &hidden,
                has_wip: false,
                show_stashes: true,
                settings: Default::default(),
            },
            None,
            &Default::default(),
        );
    }
    assert_eq!(log.as_ref().expect("the log read").nodes.len(), 4);
    assert_eq!(
        fingerprints[1], fingerprints[2],
        "nothing changed between the polls"
    );
    assert_eq!(layout.layout_builds, 1, "idle polls must not rebuild");
    assert_eq!((log_gen, refs_gen, stashes_gen), (1, 1, 1));

    link.stop().await;
    daemon_handle.shutdown().await.unwrap();
}

/// A live update lock keeps the TUI from auto-starting its own daemon: the
/// refusal is the localized status text, no daemon is started and no
/// discovery file written. A stale lock, or none, does not refuse.
#[tokio::test]
async fn a_live_update_lock_refuses_the_embedded_daemon_start() {
    use gitpoulpi_config::{update_lock, ConfigPaths};
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
    let port_file = dir.path().join("session.port");
    std::fs::create_dir_all(paths.update_lock_file().parent().unwrap()).unwrap();
    std::fs::write(
        paths.update_lock_file(),
        update_lock::render(1, update_lock::now_secs()),
    )
    .unwrap();
    let refused = conn::ensure_daemon_in(Some(&port_file), "update-lock-test", Some(&paths)).await;
    assert_eq!(
        refused.unwrap_err(),
        gitpoulpi_i18n::msg::link_error::UPDATING.get().to_string()
    );
    assert!(!port_file.exists(), "no daemon was started");

    let now = update_lock::now_secs();
    assert!(conn::refuse_while_updating(Some(&paths), None, now).is_err());
    let stale = now + update_lock::STALE_AFTER_SECS + 5;
    assert_eq!(
        conn::refuse_while_updating(Some(&paths), None, stale),
        Ok(())
    );
    std::fs::remove_file(paths.update_lock_file()).unwrap();
    assert_eq!(conn::refuse_while_updating(Some(&paths), None, now), Ok(()));
    assert_eq!(conn::refuse_while_updating(None, None, now), Ok(()));
}
