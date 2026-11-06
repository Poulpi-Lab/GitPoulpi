//! `DaemonHandle::shutdown` returns only once the requests it was already
//! running have finished: a caller that deletes the repository right after
//! it (every test fixture does) must not race a process still reading it.
//! The hold is process-global: this file holds one test.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::{Daemon, Hold};
use gitpoulpi_ipc::transport::write_message;
use gitpoulpi_ipc::Request;
use gitpoulpi_testkit::FixtureRepoBuilder;

const SESSION: &str = "shutdown-drain-test";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_waits_for_a_read_that_is_in_flight() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();

    let hold = Hold::arm(OperationKind::Refs);
    let stream = gitpoulpi_ipc::Client::connect(&handle.endpoint())
        .await
        .unwrap()
        .into_stream();
    let (_read_half, mut write_half) = stream.into_split();
    write_message(
        &mut write_half,
        &Request {
            id: 1,
            session: SESSION.into(),
            repo: Some(fixture.repo_ref().id.clone()),
            command: OperationKind::Refs,
            args: serde_json::json!({}),
        },
    )
    .await
    .unwrap();
    tokio::task::block_in_place(|| hold.wait_reached());

    let mut shutdown = Box::pin(handle.shutdown());
    let early = tokio::time::timeout(Duration::from_millis(500), &mut shutdown).await;
    assert!(
        early.is_err(),
        "shutdown returned while a read was still running"
    );
    hold.release();
    tokio::time::timeout(Duration::from_secs(10), shutdown)
        .await
        .expect("shutdown finishes once the read is done")
        .unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
