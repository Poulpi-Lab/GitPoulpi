//! The commit-files worker end to end: a real daemon,
//! the real link loop, and the requests the render loop sends.
//!
//! Where a test needs several requests to arrive *together* (a burst of
//! selections), they are queued before the link starts: the worker's
//! first act is to drain its channel, so what it serves is decided before
//! any timing can creep in.

use super::*;
use gitpoulpi_config::DefaultBackend;

use crate::commit_files_cache::{CommitFilesPurpose, QueuedFetch};

struct Harness {
    commit_files_tx: tokio::sync::mpsc::UnboundedSender<CommitFilesRequest>,
    commit_files_rx: std_mpsc::Receiver<CommitFilesOutcome>,
    cache: SharedCommitFilesCache,
    _daemon: gitpoulpi_daemon::DaemonHandle,
    // Held so the link loop does not see its other channels close.
    _keep: Vec<Box<dyn std::any::Any + Send>>,
}

fn fetch(commit: Oid, with_tree: bool, purpose: CommitFilesPurpose) -> CommitFilesRequest {
    CommitFilesRequest::Fetch(QueuedFetch {
        commit,
        with_tree,
        purpose,
        sent: std::time::Instant::now(),
    })
}

fn selection(commit: Oid) -> CommitFilesRequest {
    fetch(commit, false, CommitFilesPurpose::Selection)
}

/// Starts a daemon over `backend` and a link to it, with `queued`
/// already waiting in the commit-files channel.
async fn start(
    repo: &RepoRef,
    backend: DefaultBackend,
    session: &str,
    queued: Vec<CommitFilesRequest>,
) -> Harness {
    let daemon = gitpoulpi_daemon::Daemon::new(session)
        .spawn(gitpoulpi_daemon::backend_for(backend))
        .await
        .unwrap();
    let (manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, push_rejected_rx) = std_mpsc::channel();
    let (commit_files_out_tx, commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, working_diff_rx) = std_mpsc::channel();
    let (file_insight_out_tx, file_insight_rx) = std_mpsc::channel();
    let (log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, log_page_rx) = std_mpsc::channel();
    for request in queued {
        commit_files_tx.send(request).unwrap();
    }
    let cache = SharedCommitFilesCache::default();
    tokio::spawn(link_loop_with_port(
        daemon.port,
        session.to_string(),
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
        Arc::clone(&cache),
    ));
    Harness {
        commit_files_tx,
        commit_files_rx,
        cache,
        _daemon: daemon,
        _keep: vec![
            Box::new(manual_refresh_tx),
            Box::new(action_tx),
            Box::new(working_diff_tx),
            Box::new(file_insight_tx),
            Box::new(log_page_tx),
            Box::new(std::sync::Mutex::new(outcome_rx)),
            Box::new(std::sync::Mutex::new(action_error_rx)),
            Box::new(std::sync::Mutex::new(action_message_rx)),
            Box::new(std::sync::Mutex::new(push_rejected_rx)),
            Box::new(std::sync::Mutex::new(working_diff_rx)),
            Box::new(std::sync::Mutex::new(file_insight_rx)),
            Box::new(std::sync::Mutex::new(log_page_rx)),
        ],
    }
}

const BACKSTOP: std::time::Duration = std::time::Duration::from_secs(60);

/// Every outcome up to and including the first one for
/// `commit` (the list is the whole answer; hunks are fetched by
/// the Diff view, one file at a time), then whatever else arrives in the next 300ms.
async fn outcomes_until_answered(
    rx: &std_mpsc::Receiver<CommitFilesOutcome>,
    commit: Oid,
) -> Vec<CommitFilesOutcome> {
    let started = std::time::Instant::now();
    let mut seen = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(outcome) => {
                let done = outcome.commit == commit;
                seen.push(outcome);
                if done {
                    break;
                }
            }
            Err(std_mpsc::TryRecvError::Empty) => {
                assert!(
                    started.elapsed() < BACKSTOP,
                    "no full answer for {commit:?}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(std_mpsc::TryRecvError::Disconnected) => panic!("commit-files channel closed"),
        }
    }
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    seen.extend(rx.try_iter());
    seen
}

fn names(outcome: &CommitFilesOutcome) -> Vec<String> {
    outcome
        .files
        .as_ref()
        .unwrap()
        .iter()
        .map(|f| f.path.to_string_lossy().into_owned())
        .collect()
}

/// Four commits, each adding one file - oldest first.
fn history() -> (gitpoulpi_testkit::FixtureRepo, Vec<Oid>) {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "one")
        .unwrap()
        .commit_file("b.txt", "b\n", "two")
        .unwrap()
        .commit_file("c.txt", "c\n", "three")
        .unwrap()
        .commit_file("d.txt", "d\n", "four")
        .unwrap()
        .build();
    let repo = fixture.repo_ref();
    let backend = gitpoulpi_cli_backend::SystemGitBackend::new();
    let commits = ["HEAD~3", "HEAD~2", "HEAD~1", "HEAD"]
        .iter()
        .map(|rev| backend.resolve(&repo, rev).unwrap())
        .collect();
    (fixture, commits)
}

/// Holding j: twenty selections queued before any was served. Only the
/// last is fetched - as a list, no hunks - and nothing at all is sent for the rows the key only
/// passed over.
#[tokio::test]
async fn a_burst_of_selections_is_served_as_its_newest_only() {
    let (fixture, commits) = history();
    let mut burst: Vec<CommitFilesRequest> = (0..19).map(|i| selection(commits[i % 3])).collect();
    burst.push(selection(commits[3]));
    let h = start(&fixture.repo_ref(), DefaultBackend::Git2, "cf-burst", burst).await;

    let seen = outcomes_until_answered(&h.commit_files_rx, commits[3]).await;
    assert!(
        seen.iter().all(|o| o.commit == commits[3]),
        "a superseded selection was answered: {:?}",
        seen.iter().map(|o| o.commit).collect::<Vec<_>>()
    );
    assert_eq!(seen.len(), 1, "one answer: the list");
    assert!(seen[0].hunks_pending, "hunks are left to the Diff view");
    assert!(seen[0].files.as_ref().unwrap()[0].hunks.is_empty());
    assert_eq!(names(&seen[0]), vec!["d.txt"]);
}

/// Coming back to a commit is answered from the link's cache: with the
/// daemon gone, the commit fetched earlier still arrives, whole and
/// identical, while one never fetched reports the lost connection.
#[tokio::test]
async fn a_commit_already_fetched_is_answered_from_the_cache() {
    let (fixture, commits) = history();
    let h = start(
        &fixture.repo_ref(),
        DefaultBackend::Git2,
        "cf-cache",
        vec![selection(commits[2])],
    )
    .await;
    let first = outcomes_until_answered(&h.commit_files_rx, commits[2]).await;
    let first_full = first.last().unwrap();
    assert!(first_full.files.is_ok());
    assert!(first_full.hunks_pending);

    // Take the repository away from under the daemon: any read that
    // reaches it now fails. (Stopping the daemon would not do: a
    // connection already accepted keeps being served.) Windows refuses the
    // rename while the daemon still holds a file in there, which a loaded
    // machine can stretch: retry for a while.
    let (from, to) = (
        fixture.root().join(".git"),
        fixture.root().join(".git-away"),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while let Err(e) = std::fs::rename(&from, &to) {
        assert!(std::time::Instant::now() < deadline, "rename .git: {e}");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    h.commit_files_tx.send(selection(commits[2])).unwrap();
    let again = outcomes_until_answered(&h.commit_files_rx, commits[2]).await;
    assert_eq!(again.len(), 1, "a cache hit is one complete answer");
    assert!(
        again[0].hunks_pending,
        "still the list, hunks fetched per file"
    );
    assert_eq!(
        again[0].files.as_ref().unwrap(),
        first_full.files.as_ref().unwrap(),
        "the very same files and hunks"
    );

    h.commit_files_tx.send(selection(commits[0])).unwrap();
    let never = outcomes_until_answered(&h.commit_files_rx, commits[0]).await;
    assert!(
        never.last().unwrap().files.is_err(),
        "an uncached commit really does need the daemon - so the hit above came from the cache"
    );
}

/// The WIP row's tree request shares the channel with the selection but
/// is never dropped for one: queued before two selections, it is still
/// answered, with the tree, and only the newer selection is.
#[tokio::test]
async fn a_selection_never_drops_the_wip_tree_request() {
    let (fixture, commits) = history();
    let head = commits[3];
    let h = start(
        &fixture.repo_ref(),
        DefaultBackend::Git2,
        "cf-wip",
        vec![
            fetch(head, true, CommitFilesPurpose::Wip),
            selection(commits[0]),
            selection(commits[1]),
        ],
    )
    .await;

    let mut seen = outcomes_until_answered(&h.commit_files_rx, commits[1]).await;
    let started = std::time::Instant::now();
    while !seen.iter().any(|o| o.commit == head) {
        assert!(
            started.elapsed() < BACKSTOP,
            "the WIP tree was never answered"
        );
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        seen.extend(h.commit_files_rx.try_iter());
    }
    assert!(
        !seen.iter().any(|o| o.commit == commits[0]),
        "the superseded selection was answered"
    );
    let wip = seen.iter().find(|o| o.commit == head).unwrap();
    let tree: Vec<String> = wip
        .tree
        .as_ref()
        .expect("the tree was asked for")
        .as_ref()
        .unwrap()
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(tree, vec!["a.txt", "b.txt", "c.txt", "d.txt"]);
}

/// A prefetch fills the cache and answers nobody.
#[tokio::test]
async fn prefetched_neighbors_land_in_the_cache_without_an_answer() {
    let (fixture, commits) = history();
    let h = start(
        &fixture.repo_ref(),
        DefaultBackend::Git2,
        "cf-prefetch",
        vec![CommitFilesRequest::Prefetch(vec![commits[1], commits[2]])],
    )
    .await;
    let started = std::time::Instant::now();
    loop {
        {
            let cache = h.cache.lock().unwrap();
            if cache.has_files(commits[1]) && cache.has_files(commits[2]) {
                break;
            }
        }
        assert!(
            started.elapsed() < BACKSTOP,
            "the neighbors were never fetched"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(
        h.commit_files_rx.try_recv().is_err(),
        "a prefetch must not send an outcome"
    );
}

/// System git lists with `--name-status` (no patch), so it is
/// lazy too - one list outcome, hunks left to the per-file request.
#[tokio::test]
async fn with_system_git_the_list_is_hunkless_too() {
    let (fixture, commits) = history();
    let h = start(
        &fixture.repo_ref(),
        DefaultBackend::System,
        "cf-system",
        vec![selection(commits[1])],
    )
    .await;
    let seen = outcomes_until_answered(&h.commit_files_rx, commits[1]).await;
    assert_eq!(seen.len(), 1);
    assert!(seen[0].hunks_pending);
    assert_eq!(names(&seen[0]), vec!["b.txt"]);
    assert!(seen[0].files.as_ref().unwrap()[0].hunks.is_empty());
}
