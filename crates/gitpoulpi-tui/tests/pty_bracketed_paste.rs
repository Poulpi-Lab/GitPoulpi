//! The TUI asks the terminal to bracket pastes (`ESC[?2004h`) and gives that
//! up on the way out (`ESC[?2004l`), so a real multi-line paste arrives as
//! one event instead of keystrokes (an Enter that submits, letters that fire
//! shortcuts). `pty_paste.rs` injects the brackets itself, which proves the
//! handlers but not that a real terminal would send them; this file proves
//! the request, and that a pasted line break in a single-line field neither
//! submits nor is typed.
//!
//! The mode is not requested on Windows (the console input path has no paste
//! brackets), so every test here is Unix only.
#![cfg(unix)]

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const ON: &[u8] = b"\x1b[?2004h";
const OFF: &[u8] = b"\x1b[?2004l";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start()
}

fn index_of(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn wait_for_exit(term: &Terminal) {
    let deadline = Instant::now() + STEP;
    while !term.exited() {
        assert!(Instant::now() < deadline, "the TUI never exited");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// On at start, off again on a normal exit, and the terminal's own mode
/// follows.
#[test]
fn bracketed_paste_is_requested_at_start_and_released_on_exit() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-bpaste-lifecycle");
    assert!(
        index_of(&term.raw_output(), ON).is_some(),
        "ESC[?2004h was never written"
    );
    assert!(
        term.bracketed_paste_enabled(),
        "the terminal is not in paste mode"
    );
    term.send_text("q").unwrap();
    wait_for_exit(&term);
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut raw = term.raw_output();
    while index_of(&raw, OFF).is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(250));
        raw = term.raw_output();
    }
    let on = index_of(&raw, ON).expect("enabled");
    let off = index_of(&raw, OFF).expect("ESC[?2004l was never written on exit");
    assert!(on < off, "disabled before it was enabled");
    assert!(
        !term.bracketed_paste_enabled(),
        "the terminal is still in paste mode after exit"
    );
}

/// A pasted line break in a single-line field (the new-branch name) is
/// neither Enter nor part of the name: the prompt stays open, no branch is
/// made, and the text before the break is what is typed. Sent as a real
/// bracketed paste, which only reaches the field as one event because the
/// TUI asked for it.
#[test]
fn a_paste_with_a_newline_into_a_single_line_field_does_not_submit() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-bpaste-newline");
    assert!(term.bracketed_paste_enabled());
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the branch-name prompt", STEP, |s| {
        s.contains("enter branch name") || s.contains("New branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("\x1b[200~first-line\nsecond-line\x1b[201~")
        .unwrap();
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
    // The inline prompt is still open: its draft (the line break dropped) is
    // still on screen.
    let screen = term.text();
    assert!(
        screen.contains("linesecond-line"),
        "the prompt closed:\n{screen}"
    );
    term.stop().unwrap();
}
