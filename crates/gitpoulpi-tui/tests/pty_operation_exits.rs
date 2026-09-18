//! Stuck states #11 and #12 (the stuck-states design): a merge
//! whose resolved result equals HEAD ("take ours" everywhere) is concluded
//! from the banner, and an empty cherry-pick or revert is left with the
//! banner's Skip (or its palette twin). No state is left only by editing a
//! file or running plain git.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{ConflictFixture, FixtureOperation};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// Resolves `notes.txt` by taking our side: the result equals HEAD.
fn take_ours(fx: &ConflictFixture) {
    common::git_ok(fx.root(), &["checkout", "--ours", "notes.txt"]);
    common::git_ok(fx.root(), &["add", "notes.txt"]);
}

fn launch(repo: &Path, session: &str, banner: &'static str) -> (Terminal, tempfile::TempDir) {
    let (term, config) = common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the operation banner, all resolved", move |s| {
            s.contains(banner) && s.contains("all conflicts resolved")
        })
        .mouse_capture()
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    (term, config)
}

#[test]
fn a_merge_that_equals_head_is_concluded_from_the_banner() {
    let fx = ConflictFixture::text();
    let head = common::git_trimmed(fx.root(), &["rev-parse", "HEAD"]);
    let theirs = common::git_trimmed(fx.root(), &["rev-parse", "MERGE_HEAD"]);
    take_ours(&fx);
    assert_eq!(common::git(fx.root(), &["status", "--porcelain"]), "");
    let (mut term, _config) = launch(fx.root(), "pty-op-exit-merge", "Merge in progress");

    let (c, r) = term.wait_for_text("Commit merge", STEP).unwrap();
    term.click(c + 2, r).unwrap();
    // The one-line prompt opens with the merge message; Enter commits.
    term.wait_for("the commit prompt", STEP, |s| s.contains("Merge feature"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the merge concluded", STEP, |s| {
        !s.contains("Merge in progress")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let parents = common::git_trimmed(fx.root(), &["rev-list", "--parents", "-n1", "HEAD"]);
    assert!(parents.ends_with(&format!("{head} {theirs}")), "{parents}");
    assert!(common::git_try(fx.root(), &["rev-parse", "-q", "--verify", "MERGE_HEAD"]).is_none());
    term.stop().unwrap();
}

#[test]
fn skip_in_the_banner_leaves_an_empty_cherry_pick() {
    let fx = ConflictFixture::text_with(FixtureOperation::CherryPick);
    let head = common::git_trimmed(fx.root(), &["rev-parse", "HEAD"]);
    take_ours(&fx);
    let (mut term, _config) = launch(fx.root(), "pty-op-exit-pick", "Cherry-pick in progress");

    // Continue on an empty pick says why and points to Skip.
    let (c, r) = term.wait_for_text("Continue", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the empty-pick hint", STEP, |s| s.contains("Use Skip"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = term.wait_for_text("Skip", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Skip the cherry-pick in progress?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the cherry-pick gone", STEP, |s| {
        !s.contains("Cherry-pick in progress")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(common::git_trimmed(fx.root(), &["rev-parse", "HEAD"]), head);
    assert!(common::git_try(
        fx.root(),
        &["rev-parse", "-q", "--verify", "CHERRY_PICK_HEAD"]
    )
    .is_none());
    term.stop().unwrap();
}

#[test]
fn the_palette_skips_an_empty_revert() {
    let fx = ConflictFixture::text_with(FixtureOperation::Revert);
    let head = common::git_trimmed(fx.root(), &["rev-parse", "HEAD"]);
    take_ours(&fx);
    let (mut term, _config) = launch(fx.root(), "pty-op-exit-revert", "Revert in progress");

    term.send_keys("C-p").unwrap();
    term.send_text("Skip Cherry-pick").unwrap();
    term.wait_for("the Skip entry", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Skip the revert in progress?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the revert gone", STEP, |s| {
        !s.contains("Revert in progress")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(common::git_trimmed(fx.root(), &["rev-parse", "HEAD"]), head);
    assert!(common::git_try(fx.root(), &["rev-parse", "-q", "--verify", "REVERT_HEAD"]).is_none());
    term.stop().unwrap();
}
