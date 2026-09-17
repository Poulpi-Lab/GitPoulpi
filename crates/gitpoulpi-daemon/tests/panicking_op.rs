//! A daemon operation that panics must be answered with an error (not
//! leave the client waiting), and the daemon must keep serving. The
//! panic is injected with the test-only `panic_on_for_tests` hook, which
//! is process-global: this file holds one test.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{OperationKind, StageTarget};
use gitpoulpi_daemon::{panic_on_for_tests, Daemon};
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;

const SESSION: &str = "panicking-op-test";

#[tokio::test]
async fn a_panicking_operation_is_answered_with_an_error_and_the_daemon_keeps_serving() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree("new.txt", b"x\n")
        .unwrap()
        .build();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new(SESSION).spawn(backend).await.unwrap();
    let mut client = Client::connect_port(handle.port).await.unwrap();
    let request = |id: u64, command: OperationKind| Request {
        id,
        session: SESSION.into(),
        repo: Some(repo.repo_ref().id.clone()),
        command,
        args: match command {
            OperationKind::StageFile => {
                serde_json::json!({ "target": StageTarget::File("new.txt".into()) })
            }
            _ => serde_json::json!({}),
        },
    };

    panic_on_for_tests(&[OperationKind::Status, OperationKind::StageFile]);
    // A read and a mutation (exclusive path) that panic.
    for (id, command) in [(1, OperationKind::Status), (2, OperationKind::StageFile)] {
        let resp =
            tokio::time::timeout(Duration::from_secs(20), client.call(&request(id, command)))
                .await
                .unwrap_or_else(|_| panic!("{command:?}: the client waited forever"))
                .unwrap();
        let error = resp.result.expect_err("a panicking operation is an error");
        assert!(error.message.contains("panicked"), "{}", error.message);
    }

    // Disarmed, the same connection still works.
    panic_on_for_tests(&[]);
    let resp = tokio::time::timeout(
        Duration::from_secs(20),
        client.call(&request(3, OperationKind::Status)),
    )
    .await
    .expect("the daemon answers after a panic")
    .unwrap();
    assert!(resp.result.is_ok(), "{:?}", resp.result);
    let resp = client
        .call(&request(4, OperationKind::StageFile))
        .await
        .unwrap();
    assert!(resp.result.is_ok(), "{:?}", resp.result);

    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
