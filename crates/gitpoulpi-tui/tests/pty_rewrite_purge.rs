//! The purge screen of the history rewrite (slice S15) through a pty
//! (`xtask::pty`): rewrite a folder out of history, open the purge screen from
//! the result panel, see a stash on the old history block it, drop that stash
//! from the screen (after its confirmation), purge, and find the old blob gone
//! from the object store and the undo of the entry refused with the typed
//! reason. Same rules as `pty_rewrite.rs`: no coordinates, no sleeps.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `secrets/key.pem` added between two edits of `keep.txt`, and a stash made
/// of an edit on top of that history.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("keep.txt", "one\n", "add keep")
        .unwrap()
        .commit_file("secrets/key.pem", "PRIVATE KEY purge-me\n", "add key")
        .unwrap()
        .commit_file("keep.txt", "two\n", "edit keep")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("keep.txt"), "three, in a stash\n").unwrap();
    repo.git(&["stash", "push", "-m", "wip on old history"]);
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("edit keep") && s.contains("add key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Palette, the folder typed by keys, Preview, Apply: the result panel.
fn rewrite_the_folder_out(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("remove files from history").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Remove Files from History...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Paths or patterns")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.send_text("secrets").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder in the field", STEP, |s| s.contains("any depth"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_stash_blocks_the_purge_until_it_is_dropped_then_the_old_blob_is_gone() {
    let repo = fixture();
    let blob = repo.git(&["rev-parse", "HEAD:secrets/key.pem"]);
    let blob = blob.trim().to_string();
    assert!(
        repo.try_git(&["cat-file", "-e", &blob]).is_some(),
        "the blob is there before the rewrite"
    );
    let (mut term, _config) = launch(&repo, "pty-rewrite-purge");
    rewrite_the_folder_out(&mut term);
    assert!(
        repo.try_git(&["cat-file", "-e", &blob]).is_some(),
        "a rewrite alone keeps the old objects (undo needs them)"
    );

    // The result panel offers the purge; the stash on the old history blocks.
    term.send_keys("e").unwrap();
    let blocked = term
        .wait_for("the checklist with the blocker", STEP, |s| {
            s.contains("Still holding the old history")
                && s.contains("stash@{0}")
                && s.contains("Purge old history")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(blocked.contains("Rewrite history") || blocked.contains("#"));
    // Purge stays unavailable: the key does nothing.
    term.send_keys("p").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !term.screen().contains("Purge for good?"),
        "{}",
        term.screen().text()
    );

    // Drop the stash from the screen: a confirmation first, work is lost.
    term.send_keys("Enter").unwrap();
    term.wait_for("the drop confirmation", STEP, |s| {
        s.contains("Drop stash@{0}?") && s.contains("1  Drop it")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        repo.git(&["stash", "list"]).contains("wip on old history"),
        "nothing is dropped before the answer"
    );
    term.send_keys("1").unwrap();
    term.wait_for("the checklist is clear", STEP, |s| {
        s.contains("the purge can run") && !s.contains("Still holding")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["stash", "list"]).trim(), "", "the stash is gone");

    // Purge: a confirmation with no default, then the run.
    term.send_keys("p").unwrap();
    term.wait_for("the purge confirmation", STEP, |s| {
        s.contains("Purge for good?") && s.contains("2  Do not purge")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        repo.try_git(&["cat-file", "-e", &blob]).is_some(),
        "Enter without a choice purges nothing"
    );
    term.send_keys("1").unwrap();
    term.wait_for("the outcome", STEP, |s| {
        s.contains("The old history is gone from this clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        repo.try_git(&["cat-file", "-e", &blob]).is_none(),
        "the old blob is gone from the object store"
    );
    assert!(repo.root().join("keep.txt").exists());

    // Back on the result panel, Undo explains why it is refused (the newest
    // entry is the stash drop, which lost its undo with the rewrite's).
    term.send_keys("Escape").unwrap();
    term.wait_for("back on the result panel", STEP, |s| {
        s.contains("Rewrote") && !s.contains("Purge for good")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let tip = repo.git(&["rev-parse", "HEAD"]);
    term.send_keys("C-z").unwrap();
    let refused = term
        .wait_for("the purged refusal", STEP, |s| {
            // The status line is cut at the window's width: the label and
            // the start of the typed reason are what shows.
            s.contains("(purged): the old history of op") && s.contains("can't undo")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = refused;
    wait_until("history unchanged", || {
        repo.try_git(&["rev-parse", "HEAD"]).as_deref() == Some(tip.as_str())
    });
}

/// The palette reaches the screen without the result panel, and a click on
/// the cross closes it and the workbench with it.
#[test]
fn the_palette_row_opens_the_screen_and_the_cross_closes_it() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-purge-palette");
    rewrite_the_folder_out(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the workbench is closed", STEP, |s| !s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("purge old history").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Purge Old History...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the checklist", STEP, |s| {
        s.contains("Purge old history") && s.contains("stash@{0}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Retried: a cross located on a torn frame can land nowhere.
    common::retry_until(
        &mut term,
        "everything closed",
        4,
        Duration::from_secs(10),
        |t| {
            let screen = t.screen();
            if let Some((_, row)) = screen.find("Purge old history") {
                if let Some(cross) = screen
                    .find_all(" \u{2715} ")
                    .into_iter()
                    .find(|&(_, r)| r == row)
                {
                    t.click(cross.0 + 1, cross.1).expect("click the cross");
                }
            }
        },
        |s| !s.contains("Purge old history") && !s.contains("Rewrite history"),
    );
}
