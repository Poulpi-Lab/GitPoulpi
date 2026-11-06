//! `RealDaemons` against a real standalone daemon (the in-process harness the
//! daemon's own lifecycle tests use): it is found through its port file, stops
//! on the updater's `Shutdown`, and reads as gone afterwards.

use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_config::ConfigPaths;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_updater::ops::DaemonOps;
use gitpoulpi_updater::real::RealDaemons;

#[test]
fn a_session_daemon_is_found_stopped_and_seen_gone() {
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    let session = "updater-real-daemons";

    // The daemon runs on its own runtime thread: `RealDaemons` blocks on
    // its own and must not be called from inside one.
    let daemon_paths = paths.clone();
    let host = std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                Daemon::new(session)
                    .run_foreground_in(Arc::new(SystemGitBackend::new()), &daemon_paths)
                    .await
            })
    });

    let port_file = paths.ipc_port_file(session);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !port_file.exists() {
        assert!(
            Instant::now() < deadline,
            "the daemon never published its port"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    let daemons = RealDaemons;
    let found = daemons.sessions(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].name, session);
    assert!(daemons.is_up(&found[0]));

    assert!(daemons.request_shutdown(&found[0]));
    let deadline = Instant::now() + Duration::from_secs(10);
    while daemons.is_up(&found[0]) {
        assert!(Instant::now() < deadline, "the daemon did not stop");
        std::thread::sleep(Duration::from_millis(20));
    }
    host.join().unwrap().unwrap();
    assert!(daemons.sessions(dir.path()).is_empty());
}

gitpoulpi_testkit::isolate_binary!();
