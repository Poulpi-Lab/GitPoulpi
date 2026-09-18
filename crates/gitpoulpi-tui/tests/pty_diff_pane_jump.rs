//! A pane jump away from an open diff hands the navigation keys to the list
//! it lands on: with the commit's files focused again (`Ctrl+N <badge>`),
//! `j`/`k`/arrows move the file selection instead of scrolling the diff that
//! stays open beside it. The diff badge (or opening a file) gives them back.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("Filter files:") && s.contains("0b.txt")
        })
        .mouse_capture()
        .start()
}

/// Two files in one commit, each with a line only its own diff shows. `0a.txt`
/// is long, so a stray `j` would visibly move its diff cursor.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let long: String = (1..=80).map(|n| format!("alpha row {n:02}\n")).collect();
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("0a.txt", long.as_bytes())
        .unwrap()
        .write_worktree("0b.txt", b"beta only line\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add two files")
        .unwrap()
        .build()
}

/// The `Ctrl+N` badge painted just before `tail` (space + glyph + space).
fn badge_before(screen: &Screen, tail: &str) -> Option<char> {
    let (col, row) = screen.find(tail)?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let i = col as usize;
    if i >= 3 && line[i - 3] == ' ' && line[i - 1] == ' ' && line[i - 2] != ' ' {
        Some(line[i - 2])
    } else {
        None
    }
}

/// Focus the files list, open `0a.txt`'s diff (the keyboard focus is now in
/// the diff), then jump back to the files pane by its `Ctrl+N` badge.
fn open_a_then_jump_back_to_the_files(term: &mut Terminal) {
    jump_to_the_files_pane(term);
    term.wait_for("the files list focused", STEP, |s| s.contains("j/k browse"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("0a.txt's diff open", STEP, |s| {
        s.contains("alpha row 01") && !s.contains("beta only line")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    jump_to_the_files_pane(term);
}

/// `Ctrl+N`, then the badge painted on the commit's files pane.
fn jump_to_the_files_pane(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("badges over the panes", STEP, |s| {
            badge_before(s, "anged files").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let files = badge_before(&screen, "anged files")
        .unwrap_or_else(|| panic!("no badge on the files pane\n{}", screen.text()));
    term.send_keys(&files.to_string()).unwrap();
}

#[test]
fn after_a_pane_jump_back_to_the_files_j_k_move_the_file_selection_not_the_diff() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-diff-pane-jump");
    open_a_then_jump_back_to_the_files(&mut term);

    // `j` selects 0b.txt; Enter opens the selected file. Were `j` still
    // scrolling the diff, the selection would stay on 0a.txt.
    term.send_keys("j").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("0b.txt's diff, reached by j in the files list", STEP, |s| {
        s.contains("beta only line") && !s.contains("alpha row 01")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Back in the diff after opening a file: the diff owns j/k again.
    term.send_keys("Down").unwrap();
    term.send_keys("Up").unwrap();
    term.stop().unwrap();
}
