//! The connection test goes through the daemon's network path: a success and
//! a failed connection both arrive as a report (a failure is not an error
//! response), an unknown remote is an error, and it is a network read, not a
//! mutation.

use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{OperationKind, RemoteAccessOptions, RemoteAccessReport};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::FixtureRepoBuilder;

#[tokio::test]
async fn a_remote_access_check_runs_on_the_network_path_and_reports() {
    assert!(OperationKind::RemoteAccessCheck.traits().network);

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let daemon = Daemon::new("remote-access-test");
    let handle = daemon.spawn(backend).await.unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let call = |id: u64, remote: &str| Request {
        id,
        session: "remote-access-test".into(),
        repo: Some(repo.repo_ref().id.clone()),
        command: OperationKind::RemoteAccessCheck,
        args: serde_json::json!({ "remote": remote, "opts": RemoteAccessOptions::default() }),
    };

    let resp = client.call(&call(1, "origin")).await.unwrap();
    let report: RemoteAccessReport =
        serde_json::from_value(resp.result.expect("a reachable remote")).unwrap();
    assert!(report.ok, "{report:?}");
    assert_eq!(report.remote, "origin");
    assert_eq!(report.ref_count, 1);

    // A remote that cannot be reached is a report, not an error response.
    repo.git(&["remote", "set-url", "origin", "/no/such/remote.git"]);
    let resp = client.call(&call(2, "origin")).await.unwrap();
    let report: RemoteAccessReport =
        serde_json::from_value(resp.result.expect("a failure is still a report")).unwrap();
    assert!(!report.ok);
    assert!(report.error.is_some());

    // One that does not exist is an error.
    let resp = client.call(&call(3, "nowhere")).await.unwrap();
    assert!(resp.result.is_err());

    handle.shutdown().await.unwrap();
}
