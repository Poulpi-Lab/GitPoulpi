//! The TUI's embedded daemon publishes `sessions/<session>.port` and dies with
//! the process, so the file must go with it: a leftover one makes the next
//! start connect to a dead port (about 2 s to be refused on Windows, which is
//! how the repository showed up late on every start).
//!
//! Real binary through a pty; temp config dirs and fixture repositories only.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const COMMIT: &str = "port-file-commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", COMMIT)
        .unwrap()
        .build()
}

fn on_repo(s: &Screen) -> bool {
    s.contains(COMMIT)
}

fn start(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(180, 45)
        .no_exit_pause()
        .ready_when("the repository", on_repo)
        .spawn()
}

fn port_file(config: &Path, session: &str) -> PathBuf {
    config.join("sessions").join(format!("{session}.port"))
}

fn quit(term: &mut Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..900 {
        if term.exited() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the TUI did not exit:\n{}", term.text());
}

fn wait_for(what: &str, mut done: impl FnMut() -> bool) {
    for _ in 0..100 {
        if done() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("timed out waiting for {what}");
}

/// A quit takes the discovery files down.
#[test]
fn quitting_removes_the_port_file() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let file = port_file(config.path(), "port-file-quit");

    let mut term = start(repo.root(), config.path(), "port-file-quit");
    assert!(file.exists(), "the running TUI publishes its port");
    assert!(file.with_extension("host").exists(), "and who owns it");
    quit(&mut term);
    wait_for("the port file to go", || !file.exists());
    assert!(!file.with_extension("host").exists());
}

/// A file left by a crash names a dead process: the next start replaces it
/// (no connect to the dead port) and still reaches the repository.
#[test]
fn a_leftover_port_file_of_a_dead_process_is_replaced() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let session = "port-file-stale";
    let file = port_file(config.path(), session);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();

    let mut child = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "exit 0"])
            .spawn()
            .unwrap()
    } else {
        std::process::Command::new("true").spawn().unwrap()
    };
    let dead = child.id();
    child.wait().unwrap();
    std::fs::write(&file, "1").unwrap();
    std::fs::write(file.with_extension("host"), dead.to_string()).unwrap();

    let mut term = start(repo.root(), config.path(), session);
    let now = std::fs::read_to_string(&file).unwrap();
    assert_ne!(now.trim(), "1", "the stale port was replaced");
    quit(&mut term);
    wait_for("the port file to go", || !file.exists());
}

/// A second window of the same session joins the running daemon instead of
/// starting its own, and leaving does not take the first window's file down.
#[test]
fn a_second_window_joins_and_leaves_the_file_alone() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let session = "port-file-join";
    let file = port_file(config.path(), session);

    let mut first = start(repo.root(), config.path(), session);
    let port = std::fs::read_to_string(&file).unwrap();

    let mut second = start(repo.root(), config.path(), session);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        port,
        "the second window found the first one's daemon"
    );
    quit(&mut second);
    std::thread::sleep(Duration::from_millis(300));
    assert!(file.exists(), "a window that only joined removes nothing");

    quit(&mut first);
    wait_for("the port file to go", || !file.exists());
}
