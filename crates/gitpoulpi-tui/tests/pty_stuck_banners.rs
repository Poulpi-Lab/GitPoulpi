//! The two "stuck state" strips (plan `stuck-states.md`, slice S8): a
//! `git am` stopped on a patch (Continue / Skip / Abort) and a stale
//! `.git/index.lock` (Remove index.lock). Each button has a palette twin;
//! Abort and Remove ask first, and nothing happens before the answer.

mod common;

use std::time::{Duration, SystemTime};

use gitpoulpi_testkit::contract::am_fixture;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &std::path::Path, session: &str, ready: &'static str) -> Terminal {
    let (term, _config) = common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", move |s| s.contains(ready))
        .start();
    // The temp config dir lives as long as the process: leak it with the
    // terminal's lifetime rather than dropping it under the daemon.
    std::mem::forget(_config);
    term
}

fn wait_am_banner(term: &mut Terminal) {
    term.wait_for("the git am banner", STEP, |s| {
        s.contains("Applying patches (git am)") && s.contains("Skip") && s.contains("Abort")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn skip_on_the_am_banner_drops_the_patch_and_the_banner_vanishes() {
    let (repo, _) = am_fixture(true);
    let mut term = launch(repo.root(), "pty-am-skip", "main change");
    wait_am_banner(&mut term);
    let (col, row) = term
        .wait_for_text("Skip", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Applying patches (git am)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo.root().join(".git/rebase-apply").exists());
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "second patch");
}

#[test]
fn abort_on_the_am_banner_asks_then_restores_the_branch() {
    let (repo, _) = am_fixture(true);
    let head = repo.git(&["rev-parse", "HEAD"]);
    let mut term = launch(repo.root(), "pty-am-abort", "main change");
    wait_am_banner(&mut term);
    let (col, row) = term
        .wait_for_text("Abort", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("Abort git am?"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Nothing aborted before the answer.
    assert!(repo.root().join(".git/rebase-apply").exists());
    term.send_keys("y").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Applying patches (git am)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo.root().join(".git/rebase-apply").exists());
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
}

#[test]
fn the_palette_reaches_skip_on_an_am() {
    let (repo, _) = am_fixture(true);
    let mut term = launch(repo.root(), "pty-am-palette", "main change");
    wait_am_banner(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("Skip this patch").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("git am: Skip This Patch")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Applying patches (git am)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo.root().join(".git/rebase-apply").exists());
}

fn repo_with_lock(age_secs: u64) -> (gitpoulpi_testkit::FixtureRepo, std::path::PathBuf) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "base commit")
        .unwrap()
        .build();
    let lock = repo.root().join(".git/index.lock");
    std::fs::write(&lock, "").unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&lock)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(age_secs))
        .unwrap();
    (repo, lock)
}

#[test]
fn a_stale_lock_shows_a_strip_and_removing_it_asks_first() {
    let (repo, lock) = repo_with_lock(300);
    let mut term = launch(repo.root(), "pty-lock-click", "base commit");
    term.wait_for("the lock strip", STEP, |s| {
        s.contains("leftover index.lock") && s.contains("Remove index.lock")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term
        .wait_for_text("Remove index.lock", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Delete the leftover")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(lock.exists(), "nothing deleted before the answer");
    term.send_keys("y").unwrap();
    term.wait_for("the strip gone", STEP, |s| {
        !s.contains("leftover index.lock")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!lock.exists());
}

#[test]
fn the_palette_removes_a_stale_lock() {
    let (repo, lock) = repo_with_lock(300);
    let mut term = launch(repo.root(), "pty-lock-palette", "base commit");
    term.wait_for("the lock strip", STEP, |s| {
        s.contains("leftover index.lock")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Remove stale").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Repository: Remove Stale index.lock")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Delete the leftover")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    term.wait_for("the strip gone", STEP, |s| {
        !s.contains("leftover index.lock")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!lock.exists());
}

#[test]
fn a_fresh_lock_shows_no_strip() {
    let (repo, lock) = repo_with_lock(0);
    let term = launch(repo.root(), "pty-lock-fresh", "base commit");
    std::thread::sleep(Duration::from_millis(1500));
    assert!(!term.screen().contains("leftover index.lock"));
    assert!(lock.exists());
}
