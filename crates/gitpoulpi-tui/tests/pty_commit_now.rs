//! `Ctrl+G Enter` ("Git: Commit Staged Changes Now") end to end through a
//! pty (`xtask::pty`): the real binary, daemon and git. It commits the staged
//! changes with the composer's message from any view, without moving the
//! focus; when it cannot, it says why and puts the caret in the Summary.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// `a.txt` committed, `b.txt` staged, `c.txt` untracked.
fn fixture(stage_b: bool) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .write_worktree("c.txt", b"sea\n")
        .unwrap()
        .build();
    if stage_b {
        repo.git(&["add", "b.txt"]);
    }
    repo
}

fn start(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("initial commit")
        .mouse_capture()
        .start()
}

fn subject(repo: &FixtureRepo) -> String {
    repo.git(&["log", "-1", "--format=%s"]).trim().to_string()
}

fn wait_for_subject(repo: &FixtureRepo, wanted: &str) {
    let deadline = Instant::now() + STEP;
    while subject(repo) != wanted {
        assert!(
            Instant::now() < deadline,
            "HEAD never became {wanted:?}: {}",
            repo.git(&["log", "--format=%s"])
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Selects the "initial commit" row: the right pane then shows that commit,
/// not the composer.
///
/// The row moves after its first paint: the working-tree row (the composer's
/// summary and the file count) is inserted above it once the status reaches
/// the window, so a click at the first position lands on that row, or on
/// nothing, and selects no commit. A selection is idempotent, so the click
/// is repeated with the row located afresh until the commit's details show.
fn focus_the_log(term: &mut Terminal) {
    term.act_until(
        "the commit's details on the right",
        Duration::from_secs(3),
        STEP,
        |t| {
            let (col, row) = t.wait_for_text("initial commit", STEP)?;
            t.click(col + 1, row)
        },
        |s| s.contains("commit: "),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

fn chord(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_keys("Enter").unwrap();
}

/// Types a summary through the chord's own failure path (an empty summary
/// moves the caret to the composer), then leaves the field with `Esc`.
fn type_summary(term: &mut Terminal, summary: &str) {
    chord(term);
    term.wait_for("the composer's Summary asked for", STEP, |s| {
        s.contains("Enter a commit summary")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text(summary).unwrap();
    term.wait_for("the typed summary", STEP, |s| s.contains(summary))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
}

/// Staged file + typed summary, the Log focused: the chord commits, the
/// composer is cleared (a second chord with another file staged asks for a
/// summary again), and `Ctrl+Z` takes the commit back like a composer commit.
#[test]
fn the_chord_commits_the_staged_changes_from_the_log_and_is_undoable() {
    let repo = fixture(true);
    let (mut term, _cfg) = start(&repo, "pty-commit-now-chord");
    type_summary(&mut term, "from the chord");
    focus_the_log(&mut term);
    assert_eq!(subject(&repo), "initial commit", "nothing committed yet");

    chord(&mut term);
    wait_for_subject(&repo, "from the chord");
    assert_eq!(
        repo.git(&["show", "--name-only", "--format=", "HEAD"])
            .trim(),
        "b.txt"
    );

    // Undo first, with the Log still focused: nothing is pending in the
    // keyboard. (After a failing chord the caret sits in the Summary, and
    // leaving it with `Esc` and pressing `Ctrl+Z` at once reads the pair as
    // `Alt+Ctrl+Z`: the caret stays and `Ctrl+Z` is the field's own undo.)
    // The daemon moves HEAD, then journals it, so a Ctrl+Z the moment HEAD
    // moved can find nothing to undo yet: press it until HEAD is back (it
    // is the only journaled operation, a repeat undoes nothing more).
    let deadline = Instant::now() + xtask::pty::scaled(STEP);
    while subject(&repo) != "initial commit" {
        assert!(
            Instant::now() < deadline,
            "Ctrl+Z never took the commit back: {}\n{}",
            repo.git(&["log", "--format=%s"]),
            term.text()
        );
        term.send_keys("C-z").unwrap();
        std::thread::sleep(Duration::from_millis(500));
    }

    // The composer was cleared by the commit: with b.txt staged again, the
    // chord finds no summary, so no second commit. The working-tree row
    // says "+ 2" (b.txt, c.txt) only once the window drew the status taken
    // after the undo, which is the one the key reads.
    term.wait_for("the working-tree row counting b.txt and c.txt", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("WIP") && l.contains("+ 2"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    chord(&mut term);
    // The reason alone is not new: the first chord's message is still in
    // the status line. Only this chord moves the right pane to the Status.
    term.wait_for("the empty summary reported, Status shown", STEP, |s| {
        s.contains("Enter a commit summary") && s.contains("Staged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(subject(&repo), "initial commit", "no second commit");
    assert_eq!(repo.git(&["rev-list", "--count", "HEAD"]).trim(), "1");
    term.stop().unwrap();
}

/// No summary: no commit, the reason is said, and the keyboard is in the
/// composer's Summary (typed text lands there).
#[test]
fn an_empty_summary_commits_nothing_and_focuses_the_summary() {
    let repo = fixture(true);
    let (mut term, _cfg) = start(&repo, "pty-commit-now-empty");
    focus_the_log(&mut term);
    chord(&mut term);
    term.wait_for("the reason, with the Summary focused", STEP, |s| {
        s.contains("Enter a commit summary")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("typed here").unwrap();
    term.wait_for("the text in the Summary", STEP, |s| {
        s.contains("typed here")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(subject(&repo), "initial commit");
    term.stop().unwrap();
}

/// Nothing staged: no commit, the reason is said, the caret goes to the
/// Summary.
#[test]
fn nothing_staged_commits_nothing_and_focuses_the_summary() {
    let repo = fixture(false);
    let (mut term, _cfg) = start(&repo, "pty-commit-now-unstaged");
    focus_the_log(&mut term);
    chord(&mut term);
    term.wait_for("the reason", STEP, |s| {
        s.contains("Stage changes to commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("typed here").unwrap();
    term.wait_for("the text in the Summary", STEP, |s| {
        s.contains("typed here")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(subject(&repo), "initial commit");
    term.stop().unwrap();
}

/// The palette entry runs the same command.
#[test]
fn the_palette_entry_commits_the_staged_changes() {
    let repo = fixture(true);
    let (mut term, _cfg) = start(&repo, "pty-commit-now-palette");
    type_summary(&mut term, "from the palette");
    focus_the_log(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("Commit Staged Changes Now").unwrap();
    term.wait_for("the entry offered with its chord", STEP, |s| {
        s.contains("Git: Commit Staged Changes Now") && s.contains("Ctrl+G Enter")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_for_subject(&repo, "from the palette");
    term.stop().unwrap();
}
