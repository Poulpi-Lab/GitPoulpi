//! End to end through a pty: "Discard all changes" deletes the untracked
//! files AND the folders that leaves empty, and Undo brings the files back
//! with their folders. The disk is the oracle, not the screen.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(20);

fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let start = Instant::now();
    while !f() {
        assert!(start.elapsed() < STEP, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add keep"))
        .start()
}

/// A tracked `keep/kept.txt` (so `keep/` must stay), new nested files in
/// fresh folders, and a folder that was empty to begin with.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("keep/kept.txt", "kept\n", "add keep")
        .unwrap()
        .write_worktree("fresh/deep/new.txt", b"brand new\n")
        .unwrap()
        .write_worktree("keep/sub/other.txt", b"also new\n")
        .unwrap()
        .build();
    std::fs::create_dir_all(repo.root().join("old-empty")).unwrap();
    repo
}

#[test]
fn discard_all_removes_the_emptied_folders_and_undo_brings_them_back() {
    let repo = fixture();
    let root = repo.root();
    let (mut term, _config) = launch(root, "pty-discard-folders");

    let screen = term
        .wait_for("the header trash", STEP, |s| s.contains(" \u{2717}"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, row) = screen.find(" \u{2717}").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the confirm modal", STEP, |s| {
        s.contains("Discard") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();

    wait_until("the new files and their folders to be gone", || {
        !root.join("fresh").exists() && !root.join("keep/sub").exists()
    });
    assert!(root.join("keep/kept.txt").is_file(), "tracked file stays");
    assert!(root.join("old-empty").is_dir(), "an old empty folder stays");
    assert_eq!(git(root, &["status", "--porcelain"]), "", "tree is clean");

    // Undo: the journal entry reaches the window first; retry the key
    // until the files are back.
    let start = Instant::now();
    loop {
        term.send_keys("C-z").unwrap();
        std::thread::sleep(Duration::from_millis(700));
        if root.join("fresh/deep/new.txt").is_file() {
            break;
        }
        assert!(start.elapsed() < STEP, "Ctrl+Z never brought the file back");
    }
    assert_eq!(
        std::fs::read_to_string(root.join("fresh/deep/new.txt")).unwrap(),
        "brand new\n"
    );
    wait_until("the other file back", || {
        root.join("keep/sub/other.txt").is_file()
    });
    assert!(root.join("old-empty").is_dir());
    term.stop().unwrap();
}
