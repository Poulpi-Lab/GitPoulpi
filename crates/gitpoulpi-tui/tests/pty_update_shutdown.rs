//! The updater stops a session with `Shutdown {reason: "update"}` (the
//! install-channels design): the TUI attached to it leaves for good, with one
//! line on the restored terminal, instead of sitting on a dead daemon link
//! (or restarting a daemon from files that are being replaced). A plain
//! `Shutdown` does not close the window.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::ShutdownArgs;
use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(40);

fn launch(session: &str) -> (xtask::pty::Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("update shutdown probe")
        .unwrap()
        .build();
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(160, 40)
        .no_exit_pause()
        .ready("update shutdown probe")
        .start();
    std::mem::forget(repo);
    common::wait_for_viewer(config.path(), session);
    (term, config)
}

#[test]
fn an_update_shutdown_closes_the_window_with_one_line() {
    let session = "pty-update-shutdown";
    let (term, config) = launch(session);
    common::daemon_call(
        config.path(),
        session,
        OperationKind::Shutdown,
        ShutdownArgs::update(),
    );
    let deadline = Instant::now() + STEP;
    while !term.exited() {
        assert!(
            Instant::now() < deadline,
            "the window stayed open:\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let raw = String::from_utf8_lossy(&term.raw_output()).into_owned();
    assert!(raw.contains("closing for an update"), "{raw}");
    term.stop().unwrap();
}

#[test]
fn a_plain_shutdown_does_not_close_the_window() {
    let session = "pty-plain-shutdown";
    let (term, config) = launch(session);
    common::daemon_call(
        config.path(),
        session,
        OperationKind::Shutdown,
        serde_json::Value::Null,
    );
    std::thread::sleep(Duration::from_secs(3));
    assert!(!term.exited(), "a plain stop is not an update");
    let raw = String::from_utf8_lossy(&term.raw_output()).into_owned();
    assert!(!raw.contains("closing for an update"), "{raw}");
    term.stop().unwrap();
}
