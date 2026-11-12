//! Windows has no paste brackets, so a terminal that pastes without asking
//! (WezTerm, Rio, Alacritty, Tabby, Warp, mintty) delivers the text as
//! keystrokes: every line break is an `Enter`. `paste_burst` recognises such a
//! run of already-queued keys and hands it to the handlers as one paste, so a
//! line break in a single-line field neither submits nor is typed.
//!
//! The text goes into the ConPTY in one write, which is what a terminal's
//! paste is (the keys are all queued when the TUI reads them). Windows only:
//! elsewhere the terminal brackets the paste (`pty_bracketed_paste.rs`).
#![cfg(windows)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

/// A line break pasted as `Enter` keys into the new-branch prompt does not
/// create the branch: the prompt is still open with the text typed, the break
/// dropped (the same outcome as the bracketed paste on Unix).
#[test]
fn a_paste_that_arrives_as_keys_does_not_submit_a_single_line_field() {
    let repo = fixture();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-paste-burst")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start();
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the branch-name prompt", STEP, |s| {
        s.contains("enter branch name") || s.contains("New branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("first-line\rsecond-line").unwrap();
    term.wait_for("the pasted text in the prompt", STEP, |s| {
        s.contains("linesecond-line")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Give a wrongly-submitting Enter time to create the branch.
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        common::git(repo.root(), &["branch", "--list", "first-line*"])
            .trim()
            .is_empty(),
        "the pasted line break submitted the prompt"
    );
    let screen = term.text();
    assert!(
        screen.contains("linesecond-line"),
        "the prompt closed:\n{screen}"
    );
}
