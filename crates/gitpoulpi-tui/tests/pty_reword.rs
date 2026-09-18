//! End-to-end check of "Edit commit message..." driven through a pty by
//! `xtask::pty`: the real binary, the real daemon, real `git`, and a bare
//! repository inside the fixture's temp dir as `origin` (no network).
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Right-click an older, already pushed commit -> Edit commit message...
/// -> the popup opens on its message with the force-push warning; typing
/// and Ctrl+S asks once more, a second Ctrl+S rewrites it, and the commit
/// on top is replayed with its own message.
#[test]
fn an_older_pushed_commit_is_reworded_after_confirming() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first reword target")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("second on top")
        .unwrap()
        .build();

    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-reword")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second on top") && s.contains("first reword target")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = term.wait_for_text("first reword target", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Edit commit message...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Edit commit message...").unwrap();
    term.click(c + 1, r).expect("click Edit commit message");
    term.wait_for("the popup, warning about the force push", STEP, |s| {
        s.contains("Reword commit message")
            && s.contains("Already pushed to origin/main")
            && s.contains("Update message")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text(" (edited)").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the second confirmation", STEP, |s| {
        s.contains("Rewrite anyway")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the rewrite reported", STEP, |s| {
        s.contains("1 commit on top re-created") && !s.contains("Rewrite anyway")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "HEAD~1"]),
        "first reword target (edited)"
    );
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s", "HEAD"]),
        "second on top"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}
