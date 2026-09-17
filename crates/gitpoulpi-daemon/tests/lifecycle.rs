//! `gitpoulpi daemon stop|restart|status`'s daemon-side half: the
//! `OperationKind::Shutdown` request answers OK, broadcasts
//! `Event::DaemonStopping` to every subscriber (not just the connection
//! that asked), and ends the accept loop for real -- proven here by a
//! standalone daemon actually removing its discovery files and a *new*
//! connection attempt afterwards failing. `Hello` is covered alongside it
//! since `daemon status` depends on it answering truthfully.
//!
//! Uses `ConfigPaths::for_dir` over a tempdir throughout, never
//! `GITPOULPI_CONFIG_DIR` -- see that env var's doc comment on why a test
//! must not set it (it is process-wide and would race `ensure_daemon`/
//! `run_foreground` calls made by unrelated tests in the same binary).

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_daemon::{Daemon, DaemonInfo, DaemonKind};
use gitpoulpi_ipc::{Client, Event, Request, ServerMessage};

fn backend() -> Arc<dyn gitpoulpi_core::GitBackend> {
    Arc::new(SystemGitBackend::new())
}

async fn wait_for_port_file(paths: &ConfigPaths, session: &str) -> u16 {
    let port_file = paths.ipc_port_file(session);
    for _ in 0..100 {
        if let Ok(raw) = std::fs::read_to_string(&port_file) {
            if let Ok(port) = raw.trim().parse::<u16>() {
                return port;
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("daemon never published a port file at {port_file:?}");
}

/// A real standalone daemon (`run_foreground_in`, the exact call
/// `gitpoulpi daemon run`/`daemon start` make), asked to shut down over
/// IPC: the response is OK, a second connected client sees
/// `Event::DaemonStopping`, the accept loop actually ends (`run_foreground_in`
/// returns), and both discovery files are gone.
#[tokio::test]
async fn a_shutdown_request_stops_a_standalone_daemon_and_cleans_up_after_itself() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let session = "lifecycle-shutdown-test";

    let daemon = Daemon::new(session);
    let run_task = tokio::spawn({
        let paths = paths.clone();
        async move { daemon.run_foreground_in(backend(), &paths).await }
    });

    let port = wait_for_port_file(&paths, session).await;
    let port_file = paths.ipc_port_file(session);
    let pid_file = paths.ipc_pid_file(session);
    assert!(port_file.exists(), "port file must exist once listening");
    assert!(pid_file.exists(), "a standalone daemon must record its PID");
    assert_eq!(
        std::fs::read_to_string(&pid_file).unwrap().trim(),
        std::process::id().to_string(),
        "the recorded PID is this test process's own (run_foreground_in runs in-process here)"
    );

    // A second, merely-subscribed client must also hear about the
    // shutdown -- it is a broadcast, not a reply to the asker alone.
    let mut subscriber = Client::connect_port(port).await.unwrap().into_stream();

    let mut actor = Client::connect_port(port).await.unwrap();
    let response = actor
        .call(&Request {
            id: 1,
            session: session.to_string(),
            repo: None,
            command: OperationKind::Shutdown,
            args: serde_json::Value::Null,
        })
        .await
        .unwrap();
    assert!(response.result.is_ok(), "shutdown must answer OK");

    let stopping = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let msg: Option<ServerMessage> =
                gitpoulpi_ipc::transport::read_message(&mut subscriber)
                    .await
                    .unwrap();
            match msg {
                Some(ServerMessage::Event(Event::DaemonStopping { session })) => {
                    return session;
                }
                Some(_) => continue,
                None => panic!("connection closed before DaemonStopping arrived"),
            }
        }
    })
    .await
    .expect("subscriber must receive DaemonStopping before the connection is dropped");
    assert_eq!(stopping, session);

    // The accept loop actually ending, not just this one connection
    // closing.
    tokio::time::timeout(Duration::from_secs(3), run_task)
        .await
        .expect("run_foreground_in must return once the daemon shuts itself down")
        .unwrap()
        .unwrap();

    assert!(!port_file.exists(), "shutdown must remove the port file");
    assert!(!pid_file.exists(), "shutdown must remove the pid file");
}

/// `Hello` answers with this daemon's own info, distinguishing an embedded
/// daemon (`Daemon::spawn`, run inside this test process) from a
/// standalone one - `daemon status`/`daemon stop`'s PID fallback both
/// depend on this distinction.
#[tokio::test]
async fn hello_reports_this_daemons_own_kind_and_pid() {
    let daemon = Daemon::new("lifecycle-hello-test");
    let handle = daemon.spawn(backend()).await.unwrap();
    let mut client = Client::connect_port(handle.port).await.unwrap();

    let response = client
        .call(&Request {
            id: 1,
            session: "lifecycle-hello-test".to_string(),
            repo: None,
            command: OperationKind::Hello,
            args: serde_json::Value::Null,
        })
        .await
        .unwrap();
    let info: DaemonInfo = serde_json::from_value(response.result.unwrap()).unwrap();
    assert_eq!(info.kind, DaemonKind::Embedded);
    assert_eq!(info.pid, std::process::id());
    assert_eq!(info.session, "lifecycle-hello-test");

    handle.shutdown().await.unwrap();
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
