//! Reverting a merge commit from the Log asks which parent is the
//! mainline (a dropdown, maintainer rule 1), then confirms, then runs
//! `git revert -m <n>`. Driven through the palette, the keyboard twin of
//! the context menu's Revert row. Same rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(60);

/// `feature` adds feature.txt, main adds main.txt, merged with `--no-ff`.
fn merged_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let root = repo.root();
    common::git_ok(root, &["checkout", "-q", "-b", "feature"]);
    std::fs::write(root.join("feature.txt"), "feature").unwrap();
    common::git_ok(root, &["add", "feature.txt"]);
    common::git_ok(root, &["commit", "-q", "-m", "feature work"]);
    common::git_ok(root, &["checkout", "-q", "-"]);
    std::fs::write(root.join("main.txt"), "main").unwrap();
    common::git_ok(root, &["add", "main.txt"]);
    common::git_ok(root, &["commit", "-q", "-m", "main work"]);
    common::git_ok(
        root,
        &[
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "merge the feature",
            "feature",
        ],
    );
    repo
}

#[test]
fn reverting_a_merge_commit_asks_for_the_mainline_parent() {
    let repo = merged_repo();
    let root = repo.root();
    let merge = common::git_trimmed(root, &["rev-parse", "HEAD"]);
    let (mut term, _config) = common::launch(root)
        .session("pty-revert-merge")
        .size(200, 50)
        .start();
    let screen = term
        .wait_for("the merge commit row", STEP, |s| {
            s.contains("merge the feature")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("merge the feature").unwrap();
    term.click(col + 2, row).unwrap();

    term.send_keys("C-p").unwrap();
    term.send_text("Revert Selected").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();

    // The dropdown lists both parents; nothing is reverted yet.
    term.wait_for("the parent dropdown", STEP, |s| {
        s.contains("Which parent is the mainline")
            && s.contains("Parent 1:")
            && s.contains("Parent 2:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(common::git_trimmed(root, &["rev-parse", "HEAD"]), merge);

    // Down to parent 2 (feature), Enter, then the confirm names it.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the confirm", STEP, |s| s.contains("keeping parent 2"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();

    let deadline = std::time::Instant::now() + STEP;
    while common::git_trimmed(root, &["rev-parse", "HEAD"]) == merge {
        assert!(
            std::time::Instant::now() < deadline,
            "no revert commit\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    // Parent 2 is the mainline: main's change goes away, feature's stays.
    assert!(!root.join("main.txt").exists());
    assert!(root.join("feature.txt").exists());
    term.stop().unwrap();
}
