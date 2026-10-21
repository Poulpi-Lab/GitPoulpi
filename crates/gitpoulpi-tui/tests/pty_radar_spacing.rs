#![cfg(feature = "radar")]
//! The Radar leaves one empty line between the WORK IN PROGRESS section
//! and REPOSITORIES (the maintainer's ask), in the real binary. The first
//! repository is dirty (a WIP row), the second one is not linked to any
//! host (a REPOSITORIES note).

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "pty_hosting/mock.rs"]
mod mock;
#[path = "pty_hosting/support.rs"]
mod support;

use gitpoulpi_testkit::FixtureRepoBuilder;
use mock::start_mock;
use support::{config_toml, COLS, ROWS, STARTUP};

#[test]
fn an_empty_line_separates_work_in_progress_from_repositories() {
    let mock = start_mock();
    let a = support::fixture(mock.port);
    // Dirty: a tracked file edited.
    std::fs::write(a.root().join("scratch-wip.txt"), "x\n").unwrap();
    let b = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("t.txt", "t\n", "second-repo-start")
        .unwrap()
        .build();
    b.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{}/team/tool.git", mock.port),
    ]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        config_toml(mock.port, ""),
    )
    .unwrap();
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config.path().join("sessions").join("pty-lp-spacing.toml"),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n\n[[tabs]]\nrepo_path = '{}'\nactive = false\n",
            a.root().display(),
            b.root().display()
        ),
    )
    .unwrap();
    let mut term = common::launch(a.root())
        .session("pty-lp-spacing")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the first repository loaded", STARTUP, |s| {
        s.contains("Add the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("M-r").expect("Alt+R");
    let screen = term
        .wait_for("WIP and REPOSITORIES sections", STARTUP, |s| {
            s.contains("WORK IN PROGRESS") && s.contains("REPOSITORIES")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let lines: Vec<String> = screen.text().lines().map(str::to_string).collect();
    let heading = lines
        .iter()
        .position(|l| l.contains("REPOSITORIES"))
        .unwrap();
    assert!(
        lines[heading - 1].trim().is_empty()
            || lines[heading - 1].trim_matches('│').trim().is_empty(),
        "an empty line above REPOSITORIES:\n{}",
        screen.text()
    );
    assert!(
        !lines[heading - 2].trim_matches('│').trim().is_empty(),
        "the WIP row right above the gap:\n{}",
        screen.text()
    );
}
