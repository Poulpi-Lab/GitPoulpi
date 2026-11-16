//! "Fetch all" must show what the remote gained -- a new commit on
//! a branch the repository already tracks, and a branch it had never
//! seen -- without a restart. Real binary, real daemon, real `git`, a
//! bare upstream inside the fixture's temp dir (no network).
//!
//! Once through `Ctrl+G f` and once through the Pull button's menu.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("base commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

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

/// Another user pushes a commit to `main` and creates `branch` upstream.
fn push_news(repo: &FixtureRepo, commit: &str, branch: &str) {
    repo.push_from_elsewhere("main", commit);
    repo.git_in_remote(&["branch", branch, "main"]);
}

#[test]
fn fetch_all_by_key_shows_the_new_remote_commit_and_branch() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-fetch-all-key");
    push_news(&repo, "upstream-news", "zz-upstream-branch");
    // Nothing fetched yet: the TUI must not know them.
    assert!(!term.screen().contains("zz-upstream-branch"));

    term.send_keys("C-g").unwrap();
    term.send_keys("f").unwrap();
    term.wait_for("the fetch's progress or result", STEP, |s| {
        s.contains("fetching every remote") || s.contains("fetched every remote")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for(
        "the new commit and branch, the button still saying Pull",
        STEP,
        |s| {
            s.contains("upstream-news")
                && s.contains("zz-upstream-branch")
                && s.contains("Pull  \u{25BE}")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn fetch_all_from_the_pull_menu_shows_the_new_remote_commit_and_branch() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-fetch-all-menu");
    push_news(&repo, "upstream-news", "zz-upstream-branch");

    let screen = term
        .wait_for("the toolbar's Pull split button", STEP, |s| {
            s.contains("Pull  \u{25BE}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Pull  \u{25BE}").unwrap();
    term.click(c + 5, r).expect("click the Pull dropdown arrow");
    let menu = term
        .wait_for("the Pull menu", STEP, |s| s.contains("Fetch All"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Fetch All").unwrap();
    term.click(c + 1, r).expect("click Fetch All");
    term.wait_for(
        "the new commit and branch, the button still saying Pull",
        STEP,
        |s| {
            s.contains("upstream-news")
                && s.contains("zz-upstream-branch")
                && s.contains("Pull  \u{25BE}")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The Pull button itself pulls: the upstream commit lands on `main`
/// (its remote-tracking ref moves and the working branch follows).
#[test]
fn the_pull_button_pulls() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-pull-button");
    repo.push_from_elsewhere("main", "upstream-news");

    let screen = term
        .wait_for("the toolbar's Pull split button", STEP, |s| {
            s.contains("Pull  \u{25BE}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Pull  \u{25BE}").unwrap();
    term.click(c + 2, r).expect("click Pull");
    term.wait_for("the pulled commit", STEP, |s| s.contains("upstream-news"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "upstream-news"
    );
}
