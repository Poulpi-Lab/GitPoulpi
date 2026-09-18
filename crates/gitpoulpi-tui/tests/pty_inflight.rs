//! In-flight feedback and optimistic staging, through a pty with a
//! deliberately slow operation: the link's test hook
//! `GITPOULPI_TEST_ACTION_DELAY_MS` makes every action take 3 s, standing
//! in for `git add -A` over thousands of files.
//!
//! While it runs, the status line must say what is happening with a
//! spinner ("staging all changes..."), the rows must already sit under
//! Staged (the optimistic move) although the index has not changed yet,
//! and afterwards the status line is clear and the index is the real one.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(20);

fn cached(repo: &Path) -> String {
    let out = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo)
        .output()
        .expect("run git");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn a_slow_stage_all_shows_a_spinner_moves_the_rows_at_once_and_settles() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha.txt", "one\n", "initial")
        .unwrap()
        .commit_file("beta.txt", "one\n", "second")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("alpha.txt"), "changed\n").unwrap();
    std::fs::write(repo.root().join("beta.txt"), "changed\n").unwrap();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("inflight-stage-all")
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .env("GITPOULPI_TEST_ACTION_DELAY_MS", "3000")
        .spawn();
    term.wait_for("the unstaged files", STARTUP, |s| {
        s.contains("Unstaged") && s.contains("alpha.txt") && s.contains("beta.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = term
        .wait_for_text("Stage all", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();

    // The status line names what is running...
    term.wait_for("the Staging... status text", STEP, |s| {
        s.contains("staging all changes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // ...while nothing has reached the index yet (the delay is still
    // running) and the rows are already shown as staged.
    assert_eq!(
        cached(repo.root()),
        "",
        "the delayed action must not have run"
    );
    term.wait_for("both rows under Staged, none under Unstaged", STEP, |s| {
        s.contains("Unstaged Files (0)") && s.contains("Staged Files (2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Then the real state, and the status line is clear again.
    let deadline = Instant::now() + STEP;
    while cached(repo.root()) != "alpha.txt\nbeta.txt" {
        assert!(
            Instant::now() < deadline,
            "the stage never landed:\n{}",
            term.screen().text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.wait_for("the spinner gone", STEP, |s| {
        !s.contains("staging all changes...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
