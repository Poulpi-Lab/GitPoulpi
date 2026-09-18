//! A bracketed paste (`ESC[200~ ... ESC[201~`, one `Event::Paste`) lands in
//! whichever input the active mode owns (the commit Summary, the ref-name
//! prompt, the command palette, the Log's find bar) and in none of the
//! others. Control characters are dropped from single-line fields.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! temp config dir and fixture repository.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start()
}

fn paste(term: &mut Terminal, text: &str) {
    term.send_text(&format!("\x1b[200~{text}\x1b[201~"))
        .unwrap();
}

/// Into the commit Summary: the text lands, the line break is dropped.
#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY has no bracketed paste: the pasted line break arrives as Ctrl+Enter, which commits"
)]
fn a_paste_into_the_commit_summary_lands_and_loses_its_line_break() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-paste-summary");
    term.send_keys("c").unwrap();
    term.wait_for("the Summary field focused", STEP, |s| {
        s.contains("Staged Files") && !s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    paste(&mut term, "pasted subject\nrest");
    term.wait_for("the pasted text in the summary", STEP, |s| {
        s.contains("pasted subjectrest")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Into the new-branch name prompt, then Enter creates exactly that name.
#[test]
fn a_paste_into_the_branch_name_prompt_is_the_name() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-paste-branch");
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the branch-name prompt", STEP, |s| {
        s.contains("enter branch name") || s.contains("New branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    paste(&mut term, "pasted-branch");
    term.wait_for("the pasted name in the prompt", STEP, |s| {
        s.contains("pasted-branch")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    for _ in 0..300 {
        if common::git(repo.root(), &["branch", "--list", "pasted-branch"])
            .trim()
            .ends_with("pasted-branch")
        {
            term.stop().unwrap();
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the pasted branch was never created");
}

/// Into the command palette's filter: it narrows like typing does.
#[test]
fn a_paste_into_the_palette_filters_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-paste-palette");
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    paste(&mut term, "Toggle Help");
    term.wait_for("the palette narrowed to one row", STEP, |s| {
        s.contains("Command Palette (1/") && s.contains("View: Toggle Help Overlay")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Into the Log's find bar (`f`): the one commit matching is counted.
#[test]
fn a_paste_into_the_log_find_bar_searches() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-paste-find");
    term.send_keys("f").unwrap();
    paste(&mut term, "initial commit");
    term.wait_for("the find bar counting its match", STEP, |s| {
        s.contains("1 of 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
