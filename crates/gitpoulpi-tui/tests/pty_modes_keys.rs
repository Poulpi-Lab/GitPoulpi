//! In each input mode the keys that mean something in Normal mode (`q`
//! quits, `?` opens help, `j` moves, `/` filters) are plain text or inert,
//! and `Esc` leaves the mode and hands the keyboard back to Normal. One
//! launch per mode; the Normal-mode proof afterwards is that `?` opens the
//! help overlay again.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .write_worktree("a.txt", b"one\nchanged\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("second") && s.contains("// WIP")
        })
        .start()
}

/// The proof that the keyboard is Normal's again: `?` opens help, `Esc`
/// closes it.
fn assert_normal(term: &mut Terminal, mode: &str) {
    term.send_keys("?").unwrap();
    term.wait_for("the help overlay (Normal mode again)", STEP, |s| {
        s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("after {mode}: {e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Keybindings", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Command palette: `q` and `?` are filter text (nothing quits, no help),
/// `Esc` closes it.
#[test]
fn the_palette_takes_normal_keys_as_text_and_escape_leaves_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-palette");
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("q?j").unwrap();
    term.wait_for("the typed text in the filter", STEP, |s| {
        s.contains("q?j") && !s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.exited());
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Command Palette", STEP).unwrap();
    assert_normal(&mut term, "the palette");
    term.stop().unwrap();
}

/// The Log's find bar: letters are the query, `Esc` closes the bar.
#[test]
fn the_find_bar_takes_normal_keys_as_text_and_escape_leaves_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-find");
    term.send_keys("f").unwrap();
    term.send_text("q?j").unwrap();
    term.wait_for("the query typed in the bar", STEP, |s| {
        s.contains("q?j") && !s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.exited());
    term.send_keys("Esc").unwrap();
    term.wait_for("the bar closed", STEP, |s| !s.contains("q?j"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_normal(&mut term, "the find bar");
    term.stop().unwrap();
}

/// The ref-name prompt (`Ctrl+G b`): letters are the name, `Esc` cancels
/// without creating anything.
#[test]
fn the_branch_prompt_takes_normal_keys_as_text_and_escape_cancels_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-refname");
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the prompt", STEP, |s| {
        s.contains("enter branch name") || s.contains("New branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("q?j").unwrap();
    term.wait_for("the name typed", STEP, |s| {
        s.contains("q?j") && !s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    // The typed name replaced the placeholder, so "no placeholder" is true
    // before Esc is even read; the typed text going away is the proof the
    // prompt closed. Without it the `?` of `assert_normal` could reach the
    // TUI glued to the Esc (read as Alt+?) on a loaded machine.
    term.wait_for("the prompt closed", STEP, |s| {
        !s.contains("q?j") && !s.contains("enter branch name") && !s.contains("New branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(common::git(repo.root(), &["branch", "--list", "q?j"])
        .trim()
        .is_empty());
    assert_normal(&mut term, "the branch prompt");
    term.stop().unwrap();
}

/// The commit Summary (`c`): `q`, `?` and `j` are typed, not commands;
/// `Esc` hands the keyboard back.
#[test]
fn the_commit_summary_takes_normal_keys_as_text_and_escape_leaves_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-summary");
    term.send_keys("c").unwrap();
    term.wait_for("the Summary focused", STEP, |s| {
        s.contains("Staged Files") && !s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("q?j").unwrap();
    term.wait_for("the text in the Summary", STEP, |s| {
        s.contains("q?j") && !s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.exited());
    term.send_keys("Esc").unwrap();
    // The draft stays (Esc leaves the field), and Normal answers again.
    assert_normal(&mut term, "the commit summary");
    term.stop().unwrap();
}

/// The `Ctrl+G` leader and `Ctrl+N` nav prefixes: `Esc` cancels each,
/// nothing is run, Normal answers again.
#[test]
fn the_leader_and_nav_prefixes_cancel_on_escape_without_acting() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-prefix");
    for (prefix, hint) in [("C-g", "s=stage"), ("C-n", "select tab")] {
        term.send_keys(prefix).unwrap();
        term.wait_for("the prefix hint", STEP, |s| s.contains(hint))
            .unwrap_or_else(|e| panic!("{prefix}: {e}"));
        term.send_keys("Esc").unwrap();
        term.wait_for("the hint gone", STEP, |s| !s.contains(hint))
            .unwrap_or_else(|e| panic!("{prefix}: {e}\n{}", term.text()));
        assert!(!term.exited());
    }
    // Nothing was staged by the canceled leader.
    assert!(
        common::git(repo.root(), &["diff", "--cached", "--name-only"])
            .trim()
            .is_empty()
    );
    assert_normal(&mut term, "the prefixes");
    term.stop().unwrap();
}

/// The help overlay owns the keyboard: `c` filters it (no Summary focus
/// behind it), the first `Esc` clears the filter, the second closes it.
#[test]
fn the_help_overlay_swallows_normal_keys_and_escape_closes_it() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-mode-help");
    term.send_keys("?").unwrap();
    term.wait_for_text("Keybindings", STEP).unwrap();
    term.send_text("c").unwrap();
    // `c` goes into the overlay's filter, and the WIP row is untouched.
    term.wait_for("the overlay filtering", STEP, |s| {
        s.contains("filter \"c\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("the filter cleared, overlay still open", STEP, |s| {
        s.contains("Keybindings") && !s.contains("filter \"c\"")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Keybindings", STEP).unwrap();
    assert!(term.screen().contains("// WIP"), "{}", term.text());
    term.stop().unwrap();
}
