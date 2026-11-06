//! The MCP server leaves when the updater stops its session: the watcher
//! resolves `true` on `ShuttingDown {reason: "update"}` and not on a plain
//! stop. Against a real in-process daemon, no process is started.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ShutdownArgs};
use gitpoulpi_mcp::update_watch::wait_for_update_shutdown;

async fn stop(port: u16, session: &str, args: serde_json::Value) {
    let mut client = Client::connect_port(port).await.unwrap();
    client
        .call(&Request {
            id: 1,
            session: session.to_string(),
            repo: None,
            command: OperationKind::Shutdown,
            args,
        })
        .await
        .unwrap();
}

async fn watched(args: serde_json::Value) -> bool {
    let session = "mcp-update-exit-test";
    let handle = Daemon::new(session)
        .spawn(Arc::new(SystemGitBackend::new()))
        .await
        .unwrap();
    let port = handle.port;
    let watcher = tokio::spawn(wait_for_update_shutdown(port));
    // The watcher must be subscribed before the daemon stops.
    tokio::time::sleep(Duration::from_millis(200)).await;
    stop(port, session, args).await;
    let seen = tokio::time::timeout(Duration::from_secs(3), watcher)
        .await
        .expect("the watcher must end once the daemon is gone")
        .unwrap();
    let _ = handle.shutdown().await;
    seen
}

#[tokio::test]
async fn an_update_shutdown_ends_the_server_and_a_plain_stop_does_not() {
    assert!(watched(serde_json::to_value(ShutdownArgs::update()).unwrap()).await);
    assert!(!watched(serde_json::Value::Null).await);
}

gitpoulpi_testkit::isolate_binary!();
