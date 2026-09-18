//! `takeover_stale_daemon` through the real `start_at` / `link_loop`, with an
//! explicit discovery file (never `GITPOULPI_CONFIG_DIR`). The "daemon of
//! another build" is a fixture `Daemon` whose `Hello` reports another sha.

use super::*;

const WAIT: Duration = Duration::from_secs(60);
const OLD_SHA: &str = "0ldb1ld";

struct OldDaemon {
    rt: tokio::runtime::Runtime,
    port: u16,
}

fn old_daemon(port_file: &std::path::Path) -> OldDaemon {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let port = rt.block_on(async {
        let daemon = gitpoulpi_daemon::Daemon::new("takeover").with_git_sha(OLD_SHA);
        let handle = daemon
            .spawn(gitpoulpi_daemon::backend_for(Default::default()))
            .await
            .unwrap();
        let port = handle.port;
        std::mem::forget(handle);
        port
    });
    std::fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    std::fs::write(port_file, port.to_string()).unwrap();
    OldDaemon { rt, port }
}

fn listening(old: &OldDaemon) -> bool {
    old.rt
        .block_on(gitpoulpi_ipc::discovery::is_listening(old.port))
}

/// Needs this build's sha to differ from the fixture's; without one (a build
/// outside a git checkout) the mismatch rule never fires at all.
fn has_build_sha() -> bool {
    this_build_git_sha().is_some()
}

/// Red without the `takeover_stale_daemon` call in `link_loop`: the link then
/// fails with the mismatch error and never loads the repository.
#[test]
fn an_idle_daemon_of_another_build_is_replaced_by_this_window() {
    if !has_build_sha() {
        return;
    }
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join("takeover.port");
    let old = old_daemon(&port_file);

    let link = start_at(
        "takeover".into(),
        fixture.repo_ref(),
        Some(port_file.clone()),
    );
    let outcome = link.outcome_rx.recv_timeout(WAIT).unwrap_or_else(|e| {
        panic!(
            "no first refresh ({e}); link failure: {:?}",
            link.failure_rx.try_recv()
        )
    });
    assert!(outcome.log_result.is_ok(), "{:?}", outcome.log_result.err());
    assert!(
        link.failure_rx.try_recv().is_err(),
        "the link must not fail"
    );

    let new_port: u16 = std::fs::read_to_string(&port_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_ne!(new_port, old.port, "a new daemon was published");
    assert!(!listening(&old), "the old daemon was shut down");
    let notice = link.action_message_rx.try_recv().expect("an info notice");
    assert!(notice.contains(OLD_SHA), "names the old build: {notice}");
}

/// A running job (a clone from a remote that never answers) on the old
/// daemon: it is left alone and the link says why. Red if the job check is
/// dropped (the daemon would be shut down under the clone).
#[test]
fn a_daemon_of_another_build_with_a_running_job_is_left_alone() {
    if !has_build_sha() {
        return;
    }
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join("takeover.port");
    let old = old_daemon(&port_file);

    let slow = gitpoulpi_testkit::SlowRemote::start();
    let dest = config.path().join("cloned");
    let old_port = old.port;
    // The clone request keeps its connection (and so its job) until the end.
    let _clone_conn = old.rt.block_on(async {
        use gitpoulpi_ipc::transport::write_message;
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", old_port))
            .await
            .unwrap();
        let (read, mut write) = stream.into_split();
        write_message(
            &mut write,
            &Request {
                id: 1,
                session: "takeover".into(),
                repo: None,
                command: OperationKind::Clone,
                args: serde_json::json!({ "url": slow.url(), "dest": dest }),
            },
        )
        .await
        .unwrap();
        (read, write)
    });
    assert!(slow.wait_accepted(1, WAIT), "the clone never started");

    let link = start_at(
        "takeover".into(),
        fixture.repo_ref(),
        Some(port_file.clone()),
    );
    let reason = link.failure_rx.recv_timeout(WAIT).expect("a failure");
    assert!(reason.contains("jobs"), "says jobs are running: {reason}");
    assert!(reason.contains(OLD_SHA), "{reason}");
    assert!(listening(&old), "the old daemon was not touched");
    let still: u16 = std::fs::read_to_string(&port_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(still, old.port);
    old.rt.shutdown_background();
}
