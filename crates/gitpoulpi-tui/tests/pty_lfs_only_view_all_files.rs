//! `LFS only` under `View all files`: the box is offered for the viewed
//! revision's LFS files (not just the index's), at any pane width, and the
//! filter keeps only those files. The fixture needs no `git-lfs` program:
//! the pointers are plain blobs and the attributes say `filter=lfs`, and a
//! later commit deletes the pointers, so the index holds none while the
//! viewed (older) commit does.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

fn fixture() -> FixtureRepo {
    let oid = "c".repeat(64);
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("seed.txt", "x\n", "seed")
        .unwrap()
        .commit_file(
            ".gitattributes",
            "*.png filter=lfs diff=lfs merge=lfs -text\n*.mp4 filter=lfs diff=lfs merge=lfs -text\n",
            "route media through lfs",
        )
        .unwrap()
        .commit_pointer_text("assets/logo.png", &oid, 2048, "add the pointers")
        .unwrap()
        .commit_file("src/a.rs", "a\n", "add source")
        .unwrap()
        .commit_removal("assets/logo.png", "drop the media")
        .unwrap()
        .build()
}

/// Opens the TUI, selects the "add source" commit (the pointer is in its
/// tree, not in the index) and turns View all files on.
fn open(repo: &FixtureRepo, session: &str, cols: u16) -> Terminal {
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(cols, 40)
        .ready("drop the media")
        .mouse_capture()
        .start();
    std::mem::forget(config);
    let (c, r) = term
        .screen()
        .find("add source")
        .unwrap_or_else(|| panic!("no row\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    if cols >= 150 {
        let (c, r) = term
            .wait_for("the files toolbar", STEP, |s| s.contains("View all files"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
            .find("View all files")
            .unwrap();
        term.click(c + 1, r).unwrap();
    } else {
        // The narrow pane clips the toolbar's own label: the key it is.
        term.wait_for("the commit's files", STEP, |s| s.contains("Changed files"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_text("a").unwrap();
    }
    term.wait_for("the whole tree listed", STEP, |s| s.contains("seed.txt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

fn run(cols: u16, session: &str) {
    let repo = fixture();
    let mut term = open(&repo, session, cols);
    let all = term
        .wait_for("the box offered and the pointer tagged", STEP, |s| {
            s.contains("[ ] LFS") && s.contains("logo.png LFS")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!all.contains("seed.txt LFS"), "{}", all.text());

    // The checkbox.
    let (c, r) = all.find("[ ] LFS").unwrap();
    term.click(c + 1, r).unwrap();
    let only = term
        .wait_for("only the LFS file left", STEP, |s| {
            s.contains("[x] LFS") && s.contains("logo.png LFS") && !s.contains("seed.txt")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!only.contains("a.rs"), "{}", only.text());
    term.click(c + 1, r).unwrap();
    term.wait_for("every file back", STEP, |s| {
        s.contains("seed.txt") && s.contains("[ ] LFS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The key: focus is on the log, where `o` is the same toggle.
    term.send_text("o").unwrap();
    term.wait_for("filtered again, by key", STEP, |s| {
        s.contains("[x] LFS") && s.contains("logo.png LFS") && !s.contains("seed.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

#[test]
fn lfs_only_is_offered_for_the_viewed_revision_and_filters_it() {
    run(200, "pty-lfs-vaf-wide");
}

#[test]
fn lfs_only_shrinks_instead_of_vanishing_on_a_narrow_pane() {
    run(84, "pty-lfs-vaf-narrow");
}
