//! A fetch/push holds no repository lock (only the transfer lock, which
//! orders transfers among themselves and with Pull), so a read *or a
//! mutation* issued during the transfer runs at once. Pull stays exclusive.

use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{OperationKind, RepoRef, StageTarget};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Request, ServerMessage};
use gitpoulpi_testkit::FixtureRepoBuilder;

const SESSION: &str = "shared-transfers-test";

fn request(id: u64, repo: &RepoRef, command: OperationKind, args: serde_json::Value) -> Request {
    Request {
        id,
        session: SESSION.into(),
        repo: Some(repo.id.clone()),
        command,
        args,
    }
}

/// Reads until the response for `id` arrives, returning when it did.
async fn answered(read: &mut tokio::net::tcp::OwnedReadHalf, id: u64) -> Instant {
    loop {
        let message: ServerMessage =
            tokio::time::timeout(Duration::from_secs(60), read_message(read))
                .await
                .expect("an answer within a minute")
                .unwrap()
                .expect("the daemon closed the connection");
        if let ServerMessage::Response(response) = message {
            if response.request_id == id {
                assert!(response.result.is_ok(), "{:?}", response.result);
                return Instant::now();
            }
        }
    }
}

/// Reads until every id in `ids` is answered, returning when each was.
async fn answered_all(
    read: &mut tokio::net::tcp::OwnedReadHalf,
    ids: &[u64],
) -> std::collections::HashMap<u64, Instant> {
    let mut at = std::collections::HashMap::new();
    while at.len() < ids.len() {
        let message: ServerMessage =
            tokio::time::timeout(Duration::from_secs(60), read_message(read))
                .await
                .expect("an answer within a minute")
                .unwrap()
                .expect("the daemon closed the connection");
        if let ServerMessage::Response(response) = message {
            if ids.contains(&response.request_id) {
                assert!(response.result.is_ok(), "{:?}", response.result);
                at.insert(response.request_id, Instant::now());
            }
        }
    }
    at
}

#[tokio::test]
async fn a_status_during_a_slow_fetch_is_answered_at_once() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    // Makes the transfer take about a second: git runs this through the
    // shell in place of `git-upload-pack`.
    fixture.git(&[
        "config",
        "remote.origin.uploadpack",
        "sleep 1; git-upload-pack",
    ]);
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let stream = gitpoulpi_ipc::Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let (mut read, mut write) = stream.into_split();

    // Warm the daemon (first-use costs are not what is measured).
    write_message(
        &mut write,
        &request(1, &repo, OperationKind::Status, serde_json::json!({})),
    )
    .await
    .unwrap();
    answered(&mut read, 1).await;

    let fetch_sent = Instant::now();
    write_message(
        &mut write,
        &request(
            2,
            &repo,
            OperationKind::Fetch,
            serde_json::json!({ "remote": "origin" }),
        ),
    )
    .await
    .unwrap();
    // Let the daemon admit the fetch before the read goes in.
    tokio::time::sleep(Duration::from_millis(150)).await;

    let status_sent = Instant::now();
    write_message(
        &mut write,
        &request(3, &repo, OperationKind::Status, serde_json::json!({})),
    )
    .await
    .unwrap();
    // Relative to the fetch, not to a fixed budget: the tests of this file
    // run in parallel, each with its own fetches and daemon.
    let at = answered_all(&mut read, &[2, 3]).await;
    let (fetch_done, status_done) = (at[&2], at[&3]);
    assert!(
        status_done < fetch_done,
        "a Status issued during a Fetch took {:?}, the fetch {:?}",
        status_done - status_sent,
        fetch_done - fetch_sent
    );

    assert!(
        fetch_done - fetch_sent >= Duration::from_millis(900),
        "the fetch was meant to be slow; it took {:?}",
        fetch_done - fetch_sent
    );

    handle.shutdown().await.unwrap();
}

/// The open-time fetch of a real repository takes seconds to tens of
/// seconds. A stage clicked meanwhile must not wait for it - nor must
/// every read queued behind that stage (the fair lock let no read pass a
/// waiting writer, so one click froze every pane until the fetch ended).
#[tokio::test]
async fn a_stage_during_a_slow_fetch_runs_at_once_and_so_do_the_reads_after_it() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    fixture.git(&[
        "config",
        "remote.origin.uploadpack",
        "sleep 2; git-upload-pack",
    ]);
    std::fs::write(fixture.root().join("new.txt"), "hello").unwrap();
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let stream = gitpoulpi_ipc::Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let (mut read, mut write) = stream.into_split();

    write_message(
        &mut write,
        &request(1, &repo, OperationKind::Status, serde_json::json!({})),
    )
    .await
    .unwrap();
    answered(&mut read, 1).await;

    let fetch_sent = Instant::now();
    write_message(
        &mut write,
        &request(2, &repo, OperationKind::FetchAll, serde_json::json!({})),
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    let stage_sent = Instant::now();
    write_message(
        &mut write,
        &request(
            3,
            &repo,
            OperationKind::StageFile,
            serde_json::json!({ "target": StageTarget::File("new.txt".into()) }),
        ),
    )
    .await
    .unwrap();
    write_message(
        &mut write,
        &request(4, &repo, OperationKind::Status, serde_json::json!({})),
    )
    .await
    .unwrap();
    let at = answered_all(&mut read, &[2, 3, 4]).await;
    let (fetch_done, stage_done, status_done) = (at[&2], at[&3], at[&4]);
    assert!(
        fetch_done - fetch_sent >= Duration::from_millis(1800),
        "the fetch was meant to be slow; it took {:?}",
        fetch_done - fetch_sent
    );
    assert!(
        stage_done < fetch_done && status_done < fetch_done,
        "the stage took {:?} and the status after it {:?}, the fetch {:?}",
        stage_done - stage_sent,
        status_done - stage_sent,
        fetch_done - fetch_sent
    );

    handle.shutdown().await.unwrap();
}

/// Transfers that are not the same request still queue among themselves: a
/// fetch of one remote and a fetch of all contend on the same remote-tracking
/// refs, so the second starts when the first has ended. (Two *identical*
/// requests join one job instead: see the next test.)
#[tokio::test]
async fn two_different_fetches_run_one_after_the_other() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    fixture.git(&[
        "config",
        "remote.origin.uploadpack",
        "sleep 1; git-upload-pack",
    ]);
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let stream = gitpoulpi_ipc::Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let (mut read, mut write) = stream.into_split();

    let sent = Instant::now();
    let sends = [
        (
            1,
            OperationKind::Fetch,
            serde_json::json!({ "remote": "origin" }),
        ),
        (2, OperationKind::FetchAll, serde_json::json!({})),
    ];
    for (id, kind, args) in sends {
        write_message(&mut write, &request(id, &repo, kind, args))
            .await
            .unwrap();
    }
    let at = answered_all(&mut read, &[1, 2]).await;
    let last = at[&1].max(at[&2]) - sent;
    assert!(
        last >= Duration::from_millis(1900),
        "two one-second fetches were done after {last:?}: they overlapped"
    );

    handle.shutdown().await.unwrap();
}

/// Two identical fetches join one job: both callers get the answer, and the
/// remote is contacted once (about one second, not two).
#[tokio::test]
async fn two_identical_fetches_join_one_job_and_both_get_the_answer() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    // Each contact with the remote appends a line to this file. The sleep
    // keeps the first fetch running while the second request arrives.
    let counter = tempfile::tempdir().unwrap();
    let contacts = counter.path().join("contacts");
    let contacts_arg = contacts.to_string_lossy().replace('\\', "/");
    fixture.git(&[
        "config",
        "remote.origin.uploadpack",
        &format!("echo x >> '{contacts_arg}'; sleep 1; git-upload-pack"),
    ]);
    let repo = fixture.repo_ref();

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(Git2Backend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let stream = gitpoulpi_ipc::Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let (mut read, mut write) = stream.into_split();

    let contacts_of = || {
        std::fs::read_to_string(&contacts)
            .unwrap_or_default()
            .lines()
            .count()
    };

    // The baseline: how often one lone fetch contacts the remote (git may
    // talk to it more than once per fetch).
    write_message(
        &mut write,
        &request(1, &repo, OperationKind::FetchAll, serde_json::json!({})),
    )
    .await
    .unwrap();
    answered_all(&mut read, &[1]).await;
    let one_fetch = contacts_of();
    assert!(one_fetch >= 1, "the remote was never contacted");

    // Two at once: one job, so the remote sees no more than for one fetch.
    // No wall-clock involved.
    for id in [2, 3] {
        write_message(
            &mut write,
            &request(id, &repo, OperationKind::FetchAll, serde_json::json!({})),
        )
        .await
        .unwrap();
    }
    answered_all(&mut read, &[2, 3]).await;
    assert_eq!(
        contacts_of() - one_fetch,
        one_fetch,
        "two identical fetches contacted the remote more than one fetch does: they did not join"
    );

    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
