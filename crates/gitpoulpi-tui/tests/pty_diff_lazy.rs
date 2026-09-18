//! The Diff view loads the file list first and only the opened file's
//! hunks, by path, on selection, for a commit and for the working tree.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "SHA";

fn spawn(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(160, 50)
        .no_exit_pause()
        .spawn()
}

/// Three files, then one commit that changes all three differently.
fn commit_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha.txt", "a1\na2\n", "alpha")
        .unwrap()
        .commit_file("bravo.txt", "b1\nb2\n", "bravo")
        .unwrap()
        .commit_file("charlie.txt", "c1\nc2\n", "charlie")
        .unwrap()
        .commit_file("alpha.txt", "a1\nALPHA-EDIT\n", "edit alpha")
        .unwrap()
        .commit_file("bravo.txt", "b1\nBRAVO-EDIT\n", "edit bravo")
        .unwrap()
        .commit_file("charlie.txt", "c1\nCHARLIE-EDIT\n", "edit charlie")
        .unwrap()
        .build()
}

#[test]
fn a_commit_lists_its_files_and_loads_one_files_hunks_when_it_is_opened() {
    // One commit touching three files (the helper commits one at a time,
    // so squash the last three edits into a single tip).
    let repo = commit_fixture();
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(repo.root())
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}");
    };
    git(&["reset", "--soft", "HEAD~3"]);
    git(&["commit", "-q", "-m", "edit all three"]);

    let config = tempfile::tempdir().unwrap();
    let mut term = spawn(repo.root(), config.path(), "pty-diff-lazy-commit");
    // The list: every changed file named, before any diff is opened.
    term.wait_for("the commit's three files listed", STARTUP, |s| {
        s.contains("alpha.txt")
            && s.contains("bravo.txt")
            && s.contains("charlie.txt")
            && s.contains("View all files")
            && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    // Opening the middle file loads its hunks, and only its own.
    let (c, r) = term
        .wait_for_text("bravo.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("bravo's hunk in the diff, not the others'", STEP, |s| {
        s.contains("\u{2715}")
            && s.contains("BRAVO-EDIT")
            && !s.contains("ALPHA-EDIT")
            && !s.contains("CHARLIE-EDIT")
            && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_working_tree_diff_loads_the_opened_files_hunks_by_path() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha.txt", "a1\na2\n", "alpha")
        .unwrap()
        .commit_file("bravo.txt", "b1\nb2\n", "bravo")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("alpha.txt"), "a1\nALPHA-WIP\n").unwrap();
    std::fs::write(repo.root().join("bravo.txt"), "b1\nBRAVO-WIP\n").unwrap();

    let config = tempfile::tempdir().unwrap();
    let mut term = spawn(repo.root(), config.path(), "pty-diff-lazy-wip");
    term.wait_for("both dirty files listed", STARTUP, |s| {
        s.contains("alpha.txt") && s.contains("bravo.txt") && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    let (c, r) = term
        .wait_for_text("bravo.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("bravo's working diff, not alpha's", STEP, |s| {
        s.contains("\u{2715}") && s.contains("BRAVO-WIP") && !s.contains("ALPHA-WIP")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
