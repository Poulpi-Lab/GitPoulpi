//! "Edit commit message..." on a commit that is not on the checked-out
//! branch asks what to do instead of refusing: check out a branch that holds
//! it and edit there, edit in the advanced history rewrite, or cancel. Real
//! binary, real daemon, real `git`; whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const DIALOG: &str = "Edit a commit of another branch";

/// `main` is checked out; `topic work` lives on `topic` (and `topic2` when
/// asked) only.
fn repo(two_branches: bool) -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("topic")
        .unwrap()
        .commit("topic work")
        .unwrap();
    if two_branches {
        b = b.branch("topic2").unwrap();
    }
    b.checkout("main")
        .unwrap()
        .commit("main tip")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .mouse_capture()
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("main tip") && s.contains("topic work")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Right-click `topic work` -> Edit commit message... -> the question.
fn ask(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("topic work", STEP).unwrap();
    term.right_click(col + 1, row).expect("right-click");
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Edit commit message...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Edit commit message...").unwrap();
    term.click(c + 1, r).expect("click Edit commit message");
    term.wait_for("the question", STEP, |s| {
        s.contains(DIALOG) && s.contains("Cancel (C)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn current_branch(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--show-current"])
}

/// A click on the branch's button checks it out and opens the edit on it.
///
/// Falsified: without the follow-up (`off_branch_edit::wait_for` not called)
/// the branch is checked out but the popup never opens.
#[test]
fn a_click_checks_the_branch_out_and_opens_the_edit() {
    let repo = repo(false);
    let (mut term, _config) = launch(&repo, "pty-off-branch-click");
    ask(&mut term);
    let s = term.screen();
    assert!(
        s.contains("Edit with advanced history rewrite"),
        "{}",
        s.text()
    );
    let (c, r) = s.find("Check out topic and edit (1)").expect("the button");
    term.click(c + 2, r).unwrap();
    term.wait_for("the edit open on topic", STEP, |s| {
        s.contains("Reword commit message") && !s.contains(DIALOG)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(current_branch(&repo), "topic");
    let _ = term.stop();
}

/// The keyboard twin: the branch's digit, and with several branches the
/// arrows (and `Ctrl+J`) move to the next one before Enter.
#[test]
fn the_keys_pick_among_several_branches() {
    let repo = repo(true);
    let (mut term, _config) = launch(&repo, "pty-off-branch-keys");
    ask(&mut term);
    term.wait_for("both branches listed", STEP, |s| {
        s.contains("Check out topic and edit (1)") && s.contains("Check out topic2 and edit (2)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the edit open", STEP, |s| {
        s.contains("Reword commit message") && !s.contains(DIALOG)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(current_branch(&repo), "topic2");
    let _ = term.stop();
}

/// `R` opens the advanced history rewrite, nothing is checked out.
#[test]
fn r_opens_the_advanced_history_rewrite() {
    let repo = repo(false);
    let (mut term, _config) = launch(&repo, "pty-off-branch-rewrite");
    ask(&mut term);
    term.send_text("r").unwrap();
    term.wait_for("the rewrite workbench", STEP, |s| {
        s.contains("Rewrite history") && !s.contains(DIALOG)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(current_branch(&repo), "main");
    let _ = term.stop();
}

/// Esc (and `C`) cancel: nothing changes.
#[test]
fn cancel_leaves_everything_as_it_was() {
    let repo = repo(false);
    let (mut term, _config) = launch(&repo, "pty-off-branch-cancel");
    ask(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the question closed", STEP, |s| !s.contains(DIALOG))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(current_branch(&repo), "main");
    let _ = term.stop();
}
