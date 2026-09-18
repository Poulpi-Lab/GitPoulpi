//! Clicking the file row whose diff is showing goes back out to the Log
//! (the diff bar's X), unless the editor on it holds unsaved edits, in
//! which case the editor's prompt appears and the edits stay. A double
//! click never flip-flops: the second click of the pair is ignored.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "SHA";
/// Longer than the double-click window.
const PAUSE: Duration = Duration::from_millis(700);

fn launch(repo: &Path, session: &str, ready: &'static str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the right pane", move |s| {
            s.contains(ready) && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

fn wip_repo() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"one\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

fn diff_open(s: &Screen) -> bool {
    s.contains("\u{2715}") && !s.contains(LOG_HEADER)
}

#[test]
fn a_second_click_on_the_open_wip_file_returns_to_the_log() {
    let repo = wip_repo();
    let (mut term, _c) = launch(repo.root(), "pty-reclick-wip", "Unstaged");
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, |s| {
        diff_open(s) && s.contains("a.txt - unstaged")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(PAUSE);
    term.click(c, r).unwrap();
    term.wait_for("back to the Log", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains("\u{2715}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_second_click_on_the_open_commit_file_returns_to_the_log() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .commit_file("a.txt", "changed\n", "change")
        .unwrap()
        .build();
    let (mut term, _c) = launch(repo.root(), "pty-reclick-commit", "View all files");
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, |s| diff_open(s) && s.contains("changed"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(PAUSE);
    term.click(c, r).unwrap();
    term.wait_for("back to the Log", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains("\u{2715}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_double_click_on_a_closed_file_leaves_it_open() {
    let repo = wip_repo();
    let (mut term, _c) = launch(repo.root(), "pty-reclick-double", "Unstaged");
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, diff_open)
        .unwrap_or_else(|e| panic!("{e}"));
    // Give a wrongly-acting second click time to close it again.
    std::thread::sleep(PAUSE);
    assert!(diff_open(&term.screen()), "{}", term.text());
    term.stop().unwrap();
}

#[test]
fn clicking_the_row_with_unsaved_edits_asks_and_keeps_them() {
    let repo = wip_repo();
    let (mut term, _c) = launch(repo.root(), "pty-reclick-dirty", "Unstaged");
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, |s| {
        s.contains("a.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("e").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - a.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(PAUSE);
    term.click(c, r).unwrap();
    term.wait_for("the unsaved prompt, edits kept", STEP, |s| {
        s.contains("Save changes to a.txt?") && s.contains("zzone")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Nothing was written to disk.
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"one\ntwo\n"
    );
    term.stop().unwrap();
}
