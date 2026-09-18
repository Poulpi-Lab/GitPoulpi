//! Duplicates and the Cancel of local work (see the background-ops design, B10):
//! a second Push press joins the running push and says so, and a running AI
//! request has a Cancel button next to the jobs indicator that a click
//! presses. Real binary and daemon through a pty; the push is held by the
//! daemon's `GITPOULPI_TEST_TRANSFER_DELAY_MS` hook (nothing is transferred),
//! the AI by a slow fake agent.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_slow_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn pushable() -> FixtureRepo {
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
        .commit("ahead commit")
        .unwrap()
        .build()
}

/// Push twice: one job, the second press says it joined, and the indicator's
/// click still cancels the one push.
#[test]
fn a_second_push_press_joins_the_running_push() {
    let repo = pushable();
    let before = repo.git_in_remote(&["rev-parse", "main"]);
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-dup-push")
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", "25000")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("ahead commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-g").unwrap();
    term.send_keys("P").unwrap();
    term.wait_for("the running push", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-g").unwrap();
    term.send_keys("P").unwrap();
    let screen = term
        .wait_for("the join message", STEP, |s| {
            s.contains("push is already running")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains("more)") && !screen.contains("Jobs: "),
        "one push, not two:\n{}",
        screen.text()
    );

    let (col, row) = term.screen().find("[x]").expect("the [x] button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the canceled push", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("(Canceled)"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(repo.git_in_remote(&["rev-parse", "main"]), before);
    term.stop().unwrap();
}

/// A running AI request is a job with a ✦ and an `[x]`; clicking it cancels (Esc
/// stays the keyboard twin).
#[test]
fn the_ai_cancel_button_is_clickable() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_slow_fake_agent(tools.path(), 30, &["feat: too late"]);
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-dup-ai")
        .config(ai_config_toml(&agent))
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start();
    term.send_keys("C-g").unwrap();
    term.send_keys("i").unwrap();
    term.wait_for("the Cancel AI button", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("\u{2726}") && l.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = term.screen().find("[x]").expect("the button");
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for("the cancellation reported", STEP, |s| {
            s.contains("canceled") && !s.contains("[x]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!screen.contains("feat: too late"));
}
