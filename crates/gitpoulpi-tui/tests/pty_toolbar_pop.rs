//! The toolbar's Stash / Pop pair, end to end: Pop is grayed and inert with
//! no stash, Stash then Pop round-trips a working-tree change, and there is
//! no Fetch button any more. Driven through a pty by `xtask::pty` (the real
//! binary, daemon and `git`), with a temporary config dir. Nothing is found
//! by a hard-coded coordinate and nothing sleeps.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `git stash list`, or `None` while the TUI is mid-write (`bad revision
/// 'refs/stash'`): a poll must retry then, not panic like `repo.git`.
fn stash_list(root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["stash", "list"])
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn pop_is_grayed_without_a_stash_and_restores_the_stashed_change_when_clicked() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "original\n", "add notes")
        .unwrap()
        .build();
    let file = repo.root().join("notes.txt");
    std::fs::write(&file, "changed\n").unwrap();

    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-toolbar-pop")
        .config_dir(config.path())
        .size(170, 40)
        .spawn();
    term.wait_for("the toolbar", STARTUP, |s| {
        let t = s.text();
        t.contains("Stash") && t.contains("Pop") && t.contains("Pull")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !term.text().contains("Fetch"),
        "the toolbar has no Fetch button:\n{}",
        term.text()
    );

    // No stash: Pop is grayed, and clicking it says so and changes nothing.
    let (pop_col, pop_row) = term.wait_for_text("Pop", STEP).unwrap();
    let grayed = term.screen().cell(pop_col, pop_row).unwrap().fg;
    term.click(pop_col, pop_row).unwrap();
    term.wait_for_text("no stash to pop", STEP)
        .unwrap_or_else(|e| panic!("a disabled Pop should explain itself: {e}"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "changed\n");

    // Stash the change: the file reverts, a stash exists, Pop lights up.
    let (col, row) = term.wait_for_text("Stash", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for("the stash to land", STEP, |_| {
        stash_list(repo.root()).is_some_and(|l| !l.trim().is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // `git stash` writes the stash ref before it resets the working tree,
    // so the file reverts a moment after the list stops being empty.
    term.wait_for("the file to revert", STEP, |_| {
        std::fs::read_to_string(&file).is_ok_and(|t| t == "original\n")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("Pop to light up", STEP, |s| {
        s.cell(pop_col, pop_row).is_some_and(|c| c.fg != grayed)
    })
    .unwrap_or_else(|e| panic!("Pop should be enabled once a stash exists: {e}"));

    // Pop it: the change is back and the stash list is empty.
    term.click(pop_col, pop_row).unwrap();
    term.wait_for("the stash to be popped", STEP, |_| {
        stash_list(repo.root()).is_some_and(|l| l.trim().is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the change to come back", STEP, |_| {
        std::fs::read_to_string(&file).is_ok_and(|t| t == "changed\n")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("Pop to gray out again", STEP, |s| {
        s.cell(pop_col, pop_row).is_some_and(|c| c.fg == grayed)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
