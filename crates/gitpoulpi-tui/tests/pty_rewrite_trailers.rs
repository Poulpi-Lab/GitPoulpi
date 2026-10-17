//! The trailer rule (S17) end to end through a pty (`xtask::pty`): the real
//! binary, daemon and git. "Rewrite history > From this commit to HEAD..."
//! scopes the last three commits, the Add menu adds a trailer rule, the
//! form is filled by keys, Preview shows the trailer on the Commits tab,
//! Apply rewrites the three commits (the base keeps its id) and Undo puts
//! them back.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `base`, then three commits.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .commit_file("b.txt", "two\n", "second change")
        .unwrap()
        .commit_file("c.txt", "three\n", "third change")
        .unwrap()
        .commit_file("d.txt", "four\n", "fourth change")
        .unwrap()
        .build()
}

fn bodies(repo: &FixtureRepo) -> String {
    repo.git(&["log", "--format=%B%x00"])
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

const TRAILER: &str = "Signed-off-by: Sam Roe <sam@example.com>";

#[test]
fn a_signed_off_by_is_added_to_the_last_three_commits_and_undo_takes_it_away() {
    let repo = fixture();
    let before = repo.git(&["rev-parse", "HEAD"]);
    let base = repo.git(&["rev-parse", "HEAD~3"]);
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-trailers")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second change") && s.contains("fourth change")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Right-click the oldest of the three: "From this commit to HEAD...".
    let (col, row) = term.wait_for_text("second change", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains("Rewrite history"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Rewrite history").unwrap();
    term.click(c + 1, r).expect("click Rewrite history");
    let menu = term
        .wait_for("the submenu rows", STEP, |s| {
            s.contains("From this commit to HEAD...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("From this commit to HEAD...").unwrap();
    term.click(c + 1, r).expect("click From this commit");
    term.wait_for("the workbench scoped after the parent", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Only commits after")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Add menu: the trailer rule.
    term.send_text("a").unwrap();
    let menu = term
        .wait_for("the Add menu", STEP, |s| {
            s.contains("Add or remove a trailer")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Add or remove a trailer").unwrap();
    term.click(c + 1, r).expect("click the trailer rule");
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("If it already exists") && s.contains("Signed-off-by")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Fields: Action, Trailer (Signed-off-by), Value.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("Sam Roe <sam@example.com>").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the value in its field", STEP, |s| {
        s.contains("Sam Roe <sam@example.com>")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        before,
        "typing changes nothing"
    );

    // Preview: the Commits tab shows the trailer each commit gets.
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    let summary = term.screen();
    assert!(summary.contains("3 of 3"), "{}", summary.text());
    let (c, r) = summary.find("Commits").expect("the Commits tab");
    term.click(c + 1, r).expect("click the Commits tab");
    let rows = term
        .wait_for("the message changes", STEP, |s| {
            s.contains("message: +Signed-off-by: Sam Roe")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!rows.contains("message: first"), "{}", rows.text());
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        before,
        "the preview changes nothing"
    );

    // Apply: three commits carry the trailer, the base is untouched.
    term.send_keys("C-s").unwrap();
    wait_until("three commits carry the trailer", || {
        bodies(&repo).matches(TRAILER).count() == 3
    });
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["rev-parse", "HEAD~3"]), base);
    assert!(repo
        .git(&["log", "-1", "--format=%B", "HEAD"])
        .contains(TRAILER));
    assert!(!repo
        .git(&["log", "-1", "--format=%B", "HEAD~3"])
        .contains("Signed-off-by"));

    // Undo puts the old commits back.
    term.send_keys("C-z").unwrap();
    wait_until("the old tip is back", || {
        repo.git(&["rev-parse", "HEAD"]) == before
    });
    assert!(!bodies(&repo).contains("Signed-off-by"));
}
