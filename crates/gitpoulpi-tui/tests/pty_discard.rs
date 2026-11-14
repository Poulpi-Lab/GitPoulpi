//! End to end through a pty: the real binary, the real
//! daemon, real `git` -- discarding a hunk and a single line from the
//! Unstaged diff, each through its confirm modal, and checking the file on
//! disk rather than the screen (only the file can say *what* was thrown
//! away, and it is immune to torn frames).
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);

/// `many.txt`: 30 lines, three of them changed far enough apart to make
/// three hunks. Nothing staged.
fn fixture() -> (gitpoulpi_testkit::FixtureRepo, Vec<String>) {
    let original: Vec<String> = (1..=30).map(|n| format!("l{n}")).collect();
    let mut edited = original.clone();
    edited[1] = "CHANGED-TOP".to_string();
    edited[14] = "CHANGED-MIDDLE".to_string();
    edited[27] = "CHANGED-BOTTOM".to_string();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("many.txt", &(original.join("\n") + "\n"), "add many")
        .unwrap()
        .write_worktree("many.txt", (edited.join("\n") + "\n").as_bytes())
        .unwrap()
        .build();
    (repo, edited)
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add many"))
        .start()
}

/// Selects the WIP row (its graph marker), clicks `many.txt` in the
/// Unstaged half and waits for its diff with all three changes.
fn open_the_unstaged_diff(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let (col, row) = term
        .wait_for_text("\u{270E} many.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the unstaged diff with its three changes", STEP, |s| {
        s.contains("many.txt - unstaged")
            && s.contains("CHANGED-TOP")
            && s.contains("CHANGED-MIDDLE")
            && s.contains("CHANGED-BOTTOM")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn on_disk(repo: &Path) -> Vec<String> {
    std::fs::read_to_string(repo.join("many.txt"))
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn nothing_staged(repo: &Path) -> bool {
    let out = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo)
        .output()
        .expect("run git");
    out.stdout.is_empty()
}

fn confirm_prompt(s: &Screen, text: &str) -> bool {
    // The prompt wraps inside the modal, so its tail is not matched as
    // one string; the modal's own Cancel button says it is open.
    s.contains(text) && s.contains("Cancel")
}

/// `x` asks first; `n` keeps everything; `x` then `y` reverts the hunk
/// under the cursor on disk - the bottom one, reached with End then `p`
/// (the previous hunk header), which lands there in either Hunks mode.
/// The index is never touched.
#[test]
fn x_discards_the_hunk_under_the_cursor_after_confirming() {
    let (repo, edited) = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-discard-hunk");
    open_the_unstaged_diff(&mut term);

    term.send_keys("End p").unwrap();
    term.send_keys("x").unwrap();
    term.wait_for("the confirm modal for the bottom hunk", STEP, |s| {
        confirm_prompt(s, "Discard hunk 2 of many.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("n").unwrap();
    term.wait_for("the modal gone, nothing discarded", STEP, |s| {
        !s.contains("Discard hunk 2") && s.contains("CHANGED-BOTTOM")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(on_disk(repo.root()), edited, "n must discard nothing");

    term.send_keys("x").unwrap();
    term.wait_for("the confirm modal again", STEP, |s| {
        confirm_prompt(s, "Discard hunk 2 of many.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    term.wait_for("the bottom change gone from the diff", STEP, |s| {
        !s.contains("CHANGED-BOTTOM") && s.contains("CHANGED-TOP") && s.contains("CHANGED-MIDDLE")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let mut expected = edited.clone();
    expected[27] = "l28".to_string();
    assert_eq!(on_disk(repo.root()), expected);
    assert!(nothing_staged(repo.root()));
}

/// Hovering a changed line reveals its `✗ Discard` button; clicking it
/// asks first, and `y` removes exactly that added line - its removed
/// twin (`l15`) is a different line and stays removed.
#[test]
fn a_changed_lines_hover_discard_button_discards_that_line_only() {
    let (repo, edited) = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-discard-line");
    open_the_unstaged_diff(&mut term);

    let (col, row) = term
        .wait_for_text("CHANGED-MIDDLE", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.move_to(col, row).unwrap();
    let hovered = term
        .wait_for("the line's own Discard button", STEP, |s| {
            s.line(row).contains("CHANGED-MIDDLE") && s.line(row).contains("\u{2717} Discard")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (bcol, brow) = hovered
        .find_all("\u{2717} Discard")
        .into_iter()
        .find(|&(_, r)| r == row)
        .expect("predicate just matched");
    term.click(bcol + 1, brow).unwrap();
    term.wait_for("the confirm modal for one line", STEP, |s| {
        confirm_prompt(s, "Discard 1 changed line of hunk 1 in many.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the line gone from the diff", STEP, |s| {
        !s.contains("CHANGED-MIDDLE") && s.contains("CHANGED-TOP") && s.contains("CHANGED-BOTTOM")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let expected: Vec<String> = edited
        .iter()
        .filter(|l| *l != "CHANGED-MIDDLE")
        .cloned()
        .collect();
    assert_eq!(on_disk(repo.root()), expected);
    assert!(nothing_staged(repo.root()));
}
