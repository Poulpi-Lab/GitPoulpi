//! "Fetch when a tab opens" (`general.fetch_on_tab_open`) and "Refresh
//! (fetch, pull requests, issues)". Real binary, real daemon, a bare
//! upstream inside the fixture's temp dir (no network). The pty
//! launcher turns background fetching off by default; these tests turn
//! on exactly what they exercise.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const BRANCH: &str = "zz-upstream-branch";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build()
}

/// Another user pushes a commit to `main` and creates `BRANCH` upstream.
fn push_news(repo: &FixtureRepo) {
    repo.push_from_elsewhere("main", "upstream-news");
    repo.git_in_remote(&["branch", BRANCH, "main"]);
}

fn launch(
    repo: &Path,
    session: &str,
    config: &Path,
    fetch_on_open: Option<&str>,
) -> (Terminal, tempfile::TempDir) {
    let mut l = common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50);
    if let Some(v) = fetch_on_open {
        l = l.env("GITPOULPI_GENERAL_FETCH_ON_TAB_OPEN", v);
    }
    let term = l.spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("base commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, tempfile::tempdir().unwrap())
}

#[test]
fn opening_the_tui_fetches_when_the_setting_is_on() {
    let repo = fixture();
    push_news(&repo);
    let config = tempfile::tempdir().unwrap();
    let (term, _keep) = launch(
        repo.root(),
        "pty-fetch-open-on",
        config.path(),
        Some("true"),
    );
    // No key pressed: the open itself fetched.
    term.wait_for("the upstream branch and commit", STEP, |s| {
        s.contains(BRANCH) && s.contains("upstream-news")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn opening_the_tui_does_not_fetch_when_the_setting_is_off() {
    let repo = fixture();
    push_news(&repo);
    let config = tempfile::tempdir().unwrap();
    let (term, _keep) = launch(repo.root(), "pty-fetch-open-off", config.path(), None);
    std::thread::sleep(Duration::from_secs(4));
    assert!(
        !term.screen().contains(BRANCH),
        "branch present unexpectedly"
    );
}

#[test]
fn f5_fetches_and_refreshes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let (mut term, _keep) = launch(repo.root(), "pty-f5", config.path(), None);
    push_news(&repo);
    assert!(!term.screen().contains(BRANCH));
    term.send_keys("F5").unwrap();
    term.wait_for("the new branch and commit", STEP, |s| {
        s.contains(BRANCH) && s.contains("upstream-news")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn f5_can_be_rebound() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[keybindings.global]\nrefresh_all = \"F6\"\n",
    )
    .unwrap();
    let (mut term, _keep) = launch(repo.root(), "pty-f5-rebound", config.path(), None);
    push_news(&repo);
    // The old key does nothing.
    term.send_keys("F5").unwrap();
    std::thread::sleep(Duration::from_secs(3));
    assert!(
        !term.screen().contains(BRANCH),
        "branch present unexpectedly"
    );
    term.send_keys("F6").unwrap();
    term.wait_for("the new branch", STEP, |s| s.contains(BRANCH))
        .unwrap_or_else(|e| panic!("{e}"));
}
