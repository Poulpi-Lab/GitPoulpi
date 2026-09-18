//! A window with no repository (Home) sends no repository-scoped request:
//! the application log stays free of "failed" lines about the placeholder
//! path "(no repository)", which the 2 s poll used to add for ever. Opening
//! a repository from Home still loads it.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, no network.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::scaled;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Every line of the application log files under `config`.
fn log_text(config: &std::path::Path) -> String {
    std::fs::read_dir(config.join("cache").join("logs"))
        .map(|dir| {
            dir.flatten()
                .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
                .collect::<String>()
        })
        .unwrap_or_default()
}

#[test]
fn home_without_a_repository_logs_no_failed_requests() {
    let empty = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(empty.path())
        .session("pty-home-quiet-log")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .spawn();
    term.wait_for("Home", STARTUP, |s| {
        s.contains("Repository Management") && s.contains("Search repositories")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The Radar is part of the window too: it must not ask either.
    term.send_keys("M-r").unwrap();
    term.wait_for("the Radar", STEP, |s| {
        s.contains("Scope") && s.contains("Open tabs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Several polls (every 2 s) must have run.
    std::thread::sleep(scaled(Duration::from_secs(7)));
    let log = log_text(config.path());
    assert!(
        log.contains("gitpoulpi-tui starting"),
        "the log was not read: {log}"
    );
    let bad: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("WARN") && l.contains("failed"))
        .collect();
    assert!(bad.is_empty(), "failed requests were logged:\n{bad:#?}");
    term.stop().unwrap();
}

#[test]
fn opening_a_repository_from_home_still_loads_it() {
    let empty = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "quiet-log-open-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'quiet-repo'\nlast_opened = 1\n",
            repo.root().display()
        ),
    )
    .unwrap();
    let mut term = common::launch(empty.path())
        .session("pty-home-quiet-open")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .spawn();
    term.wait_for("Home", STARTUP, |s| {
        s.contains("Repository Management") && s.contains("quiet-repo")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let deadline = Instant::now() + scaled(STEP);
    loop {
        term.send_keys("Enter").unwrap();
        if term
            .wait_for("the repository", Duration::from_secs(5), |s| {
                s.contains("quiet-log-open-commit")
            })
            .is_ok()
        {
            break;
        }
        assert!(Instant::now() < deadline, "{}", term.text());
    }
    term.stop().unwrap();
}
