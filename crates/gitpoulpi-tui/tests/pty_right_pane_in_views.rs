//! The right pane's shown/collapsed state belongs to the Log: what is done to
//! it while a diff is open applies there only, and the Log gets its own state
//! back when the diff closes (`pane_layout::AutoCollapse::enter_view`).

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
            s.contains("View all files") && s.contains("Filter files:") && s.contains("0a.txt")
        })
        .mouse_capture()
        .start()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("0a.txt", b"alpha row 01\nalpha row 02\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add a file")
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

fn toggle_right_pane(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.send_keys("l").unwrap();
}

fn wait_shown(term: &mut Terminal, what: &str) {
    term.wait_for(what, STEP, |s| s.contains("Filter files:"))
        .unwrap_or_else(|e| panic!("{e}"));
}

fn wait_collapsed(term: &mut Terminal, what: &str) {
    term.wait_for(what, STEP, |s| !s.contains("Filter files:"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Focus the files list by badge and open the first file's diff.
fn open_diff_from_the_files(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("badges over the panes", STEP, |s| {
            badge_before(s, "anged files").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let badge = badge_before(&screen, "anged files").unwrap();
    term.send_keys(&badge.to_string()).unwrap();
    term.wait_for("the files list focused", STEP, |s| s.contains("j/k browse"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the diff open", STEP, |s| s.contains("alpha row 01"))
        .unwrap_or_else(|e| panic!("{e}"));
}

fn close_diff(term: &mut Terminal) {
    term.send_keys("Esc").unwrap();
    term.wait_for("the diff closed", STEP, |s| !s.contains("alpha row 01"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn collapsing_the_right_pane_in_a_diff_leaves_it_shown_in_the_log() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-right-pane-views-a");
    open_diff_from_the_files(&mut term);
    wait_shown(&mut term, "the pane shown beside the diff");
    toggle_right_pane(&mut term);
    wait_collapsed(&mut term, "the pane collapsed in the diff");
    close_diff(&mut term);
    wait_shown(&mut term, "the pane shown again in the Log");
    term.stop().unwrap();
}

#[test]
fn expanding_the_right_pane_in_a_diff_leaves_it_collapsed_in_the_log() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-right-pane-views-b");
    // A first diff leaves the file selected for the palette's Open Diff.
    open_diff_from_the_files(&mut term);
    close_diff(&mut term);
    toggle_right_pane(&mut term);
    wait_collapsed(&mut term, "the pane collapsed in the Log");

    term.send_keys("C-p").unwrap();
    term.send_text("Open Diff for Selected File").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the diff open", STEP, |s| s.contains("alpha row 01"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    toggle_right_pane(&mut term);
    wait_shown(&mut term, "the pane expanded in the diff");
    close_diff(&mut term);
    wait_collapsed(&mut term, "the pane collapsed again in the Log");
    term.stop().unwrap();
}
