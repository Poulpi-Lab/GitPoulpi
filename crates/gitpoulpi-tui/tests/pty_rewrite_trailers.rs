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

/// The messages of the history; a read that fails while a ref swap is in
/// flight (the rewrite's apply or undo) is retried.
fn bodies(repo: &FixtureRepo) -> String {
    for _ in 0..100 {
        if let Some(out) = repo.try_git(&["log", "--format=%B%x00"]) {
            return out;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
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
    common::right_click_text_until(&mut term, "second change", "the commit menu", |s| {
        s.contains("Rewrite history")
    });
    common::click_text_until(&mut term, "Rewrite history", "the submenu rows", |s| {
        s.contains("From this commit to HEAD...")
    });
    common::click_text_until(
        &mut term,
        "From this commit to HEAD...",
        "the workbench scoped after the parent",
        |s| s.contains("Rewrite history") && s.contains("Only commits after"),
    );

    // Add menu: the trailer rule.
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| {
        s.contains("Add or remove a trailer")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::click_text_until(
        &mut term,
        "Add or remove a trailer",
        "the rule's fields",
        |s| s.contains("If it already exists") && s.contains("Signed-off-by"),
    );
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
    term.wait_for("the summary counting three commits", STEP, |s| {
        s.contains("keep their id") && s.contains("3 of 3")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::click_text_until(&mut term, "Commits", "the message changes", |s| {
        s.contains("message: +Signed-off-by: Sam Roe") && !s.contains("message: first")
    });
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
        repo.try_git(&["rev-parse", "HEAD"]).as_deref() == Some(before.as_str())
    });
    assert!(!bodies(&repo).contains("Signed-off-by"));
}
