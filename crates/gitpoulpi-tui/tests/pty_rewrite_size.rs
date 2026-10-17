//! The Size tab of the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git. A large file is added
//! and deleted again in a fixture's history; the analysis lists it as
//! deleted, "Remove from history" turns the ticked row into a rule, Preview
//! and Apply rewrite the history, the file is gone from HEAD's history, and
//! Undo brings it back.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file(
            "dump/big.sql",
            &"INSERT INTO t VALUES (1);\n".repeat(8_000),
            "add dump",
        )
        .unwrap()
        .commit_file("notes.txt", "n\n", "add notes")
        .unwrap()
        .build();
    repo.git(&["rm", "-q", "dump/big.sql"]);
    repo.git(&["commit", "-q", "-m", "drop the dump"]);
    repo
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Whether the big file's content is reachable from HEAD.
fn history_has_the_dump(repo: &FixtureRepo) -> bool {
    repo.git(&["log", "HEAD", "--format=", "--name-only"])
        .contains("dump/big.sql")
}

fn open_workbench(repo: &FixtureRepo, session: &str, config: &std::path::Path) -> Terminal {
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("add notes") && s.contains("drop the dump")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

#[test]
fn a_deleted_large_file_is_found_removed_from_history_and_undone() {
    let repo = fixture();
    assert!(history_has_the_dump(&repo));
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = open_workbench(&repo, "pty-rewrite-size", config.path());
    let screen = term.screen();

    // The Size tab by a click on its name (on the tabs' row).
    let (_, tabs_row) = screen.find("Summary").expect("the tabs");
    let (tab_col, _) = screen
        .find_all("Size")
        .into_iter()
        .find(|&(_, row)| row == tabs_row)
        .expect("the Size tab");
    term.click(tab_col + 1, tabs_row).expect("click Size");
    let screen = term
        .wait_for("the Size tab", STEP, |s| {
            s.contains("Analyse sizes") && s.contains("Nothing analysed yet")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // Analyse with the button.
    let (c, r) = screen.find("Analyse sizes").unwrap();
    term.click(c + 1, r).expect("click Analyse sizes");
    let screen = term
        .wait_for("the analysis", STEP, |s| {
            s.contains("kept only by the history") && s.contains("dump/big.sql")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("deleted"),
        "the file left HEAD\n{}",
        screen.text()
    );
    assert!(
        screen.contains("kept only by the history"),
        "{}",
        screen.text()
    );

    // Tick the row, then "Remove from history".
    // The row's state cell ("deleted", below the tabs): the path also shows
    // in the Log's file list beside the workbench. A click sent as soon as
    // the row is drawn can race its hit target, so it is retried until the
    // row is ticked (and never once it is, which would untick it).
    let mut screen = screen;
    for _ in 0..6 {
        if screen.contains("Remove from history (1)") {
            break;
        }
        let (_, tabs_row) = screen.find("Summary").expect("the tabs");
        let (c, r) = screen
            .find_all("deleted")
            .into_iter()
            .filter(|&(_, row)| row > tabs_row)
            .min_by_key(|&(col, _)| col)
            .expect("the deleted row");
        term.click(c + 1, r).expect("click the row");
        if let Ok(ticked) = term.wait_for("the row ticked", Duration::from_secs(5), |s| {
            s.contains("Remove from history (1)")
        }) {
            screen = ticked;
        } else {
            screen = term.screen();
        }
    }
    assert!(
        screen.contains("Remove from history (1)"),
        "the row was never ticked\n{}",
        screen.text()
    );
    assert!(screen.contains("free up to about") || screen.contains("frees up to about"));
    let (c, r) = screen.find("Remove from history (1)").unwrap();
    term.click(c + 1, r).expect("click Remove from history");
    term.wait_for("the rule", STEP, |s| {
        s.contains("Rule 1 will remove 1 path")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Preview: nothing is changed by it. Apply: the file leaves the history.
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(history_has_the_dump(&repo), "the preview changes nothing");
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!history_has_the_dump(&repo), "the file left the history");

    // Undo brings the old history back.
    term.send_keys("C-z").unwrap();
    wait_until("the file is back in the history", || {
        history_has_the_dump(&repo)
    });
}

/// The same path with the keyboard alone: F7 (the palette's "Analyse the
/// History Sizes") opens the tab and runs, Down/Space tick, `r` removes.
#[test]
fn the_size_tab_works_by_keys_alone() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = open_workbench(&repo, "pty-rewrite-size-keys", config.path());
    term.send_keys("F7").unwrap();
    term.wait_for("the analysis", STEP, |s| {
        s.contains("kept only by the history") && s.contains("dump/big.sql")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Space ticks the row under the cursor, `r` writes the rule.
    term.send_text(" ").unwrap();
    term.wait_for("the row ticked", STEP, |s| {
        s.contains("Remove from history (1)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("r").unwrap();
    term.wait_for("the rule added", STEP, |s| {
        s.contains("Rule 1 will remove 1 path")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!history_has_the_dump(&repo));
}
