//! A mouse press while a chord prefix waits for its second key (`Ctrl+N`,
//! `Ctrl+G`) cancels the chord like `Esc`, and the press is swallowed. The key
//! that would have completed the chord then does not run the chord's action
//! and the hint is gone.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SETTLE: Duration = Duration::from_millis(1200);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("c.txt", b"sea\n")
        .unwrap()
        .build()
}

fn start(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("initial commit")
        .mouse_capture()
        .start()
}

#[test]
fn a_click_cancels_the_navigation_chord() {
    let repo = fixture();
    let (mut term, _c) = start(&repo, "pty-chord-click-nav");
    term.send_keys("C-n").unwrap();
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("initial commit", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for(
        "navigation canceled, the hint gone, the ordinary footer back",
        STEP,
        |s| s.contains("navigation canceled") && !s.contains("collapse"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // `Shift+H` would have collapsed the Refs pane.
    term.send_keys("H").unwrap();
    let s = term.wait_for_stable(SETTLE, STEP).unwrap();
    assert!(
        s.contains("LOCAL") && !s.contains("Refs pane collapsed"),
        "{}",
        s.text()
    );
}

#[test]
fn a_right_click_cancels_the_git_chord() {
    let repo = fixture();
    let (mut term, _c) = start(&repo, "pty-chord-click-git");
    term.send_keys("C-g").unwrap();
    term.wait_for("the git prefix's footer hint", STEP, |s| {
        !s.contains("Ctrl+P: commands")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("initial commit", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    term.wait_for(
        "the git prefix's hint gone, the ordinary footer back",
        STEP,
        |s| s.contains("Ctrl+P: commands"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // `a` would have staged everything.
    term.send_keys("a").unwrap();
    term.wait_for_stable(SETTLE, STEP).unwrap();
    let status = repo.git(&["status", "--porcelain"]);
    assert!(status.contains("?? c.txt"), "staged by the chord: {status}");
}
