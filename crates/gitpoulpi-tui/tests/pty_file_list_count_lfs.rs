//! The file lists' two small helpers, driven through a pty by `xtask::pty`:
//! a collapsed folder shows how many files are behind it (subfolders
//! included), and a repository with LFS files gets an `LFS only` pre-filter
//! (checkbox, `o` key and palette command). Whole-state waits, no
//! coordinates.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{require_git_lfs, FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

/// Nested folders in one commit: `src` has 4 files (2 of them in subfolders),
/// `docs` 1, and `assets` 1 (an LFS file when `lfs`).
fn fixture(lfs: bool) -> FixtureRepo {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    if lfs {
        builder = builder.with_lfs().unwrap().lfs_track("*.bin").unwrap();
    }
    let repo = builder
        .commit_file("seed.txt", "x\n", "seed")
        .unwrap()
        .build();
    for (path, body) in [
        ("src/a.rs", "a\n"),
        ("src/d.rs", "d\n"),
        ("src/models/b.rs", "b\n"),
        ("src/models/deep/c.rs", "c\n"),
        ("docs/guide.md", "g\n"),
    ] {
        let full = repo.root().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    if lfs {
        std::fs::create_dir_all(repo.root().join("assets")).unwrap();
        std::fs::write(repo.root().join("assets/hero.bin"), vec![7u8; 1500]).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-m", "add nested files"]);
    repo
}

/// Starts the TUI on the fixture's last commit in Tree mode.
fn open_tree(repo: &FixtureRepo, session: &str) -> Terminal {
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 40)
        .ready("add nested files")
        .mouse_capture()
        .start();
    // Kept alive until the test ends (the config dir is the app's).
    std::mem::forget(config);
    let (c, r) = term
        .wait_for("the files header", STEP, |s| s.contains("Tree [m]"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
        .find("Tree [m]")
        .unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the tree with its folders", STEP, |s| s.contains("src/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

#[test]
fn a_collapsed_folder_shows_how_many_files_it_holds_subfolders_included() {
    let repo = fixture(false);
    let mut term = open_tree(&repo, "pty-folder-count");
    // Expanded: no count on the row.
    let open = term.screen();
    assert!(
        !open.contains("src/ ("),
        "an expanded folder shows no count:\n{}",
        open.text()
    );
    // Fold every folder (a click puts the focus in the list, `-` folds all).
    let (fc, fr) = open.find("docs/").unwrap();
    term.click(fc + 1, fr).unwrap();
    term.send_text("-").unwrap();
    let folded = term
        .wait_for("every folder folded, each with its count", STEP, |s| {
            s.contains("src/ (4)") && s.contains("docs/ (1)") && !s.contains("a.rs")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !folded.contains("models/"),
        "the subfolders are inside the count:\n{}",
        folded.text()
    );
    // Opening `src` drops its count and keeps the others'.
    let (sc, sr) = folded.find("src/").unwrap();
    term.click(sc + 1, sr).unwrap();
    term.wait_for("src open, docs still counted", STEP, |s| {
        s.contains("a.rs") && !s.contains("src/ (4)") && s.contains("docs/ (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

#[test]
fn lfs_files_wear_a_flag_and_the_lfs_only_filter_hides_the_rest() {
    require_git_lfs!("lfs_files_wear_a_flag_and_the_lfs_only_filter_hides_the_rest");
    let repo = fixture(true);
    let mut term = open_tree(&repo, "pty-lfs-only");
    let all = term
        .wait_for("the LFS file flagged and the filter offered", STEP, |s| {
            s.contains("hero.bin LFS") && s.contains("LFS only") && s.contains("a.rs")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!all.contains("a.rs LFS"), "a plain file has no flag");

    // The checkbox.
    let (c, r) = all.find("LFS only").unwrap();
    term.click(c + 1, r).unwrap();
    let only = term
        .wait_for("only the LFS file left", STEP, |s| {
            s.contains("hero.bin LFS") && s.contains("[x] LFS only") && !s.contains("a.rs")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!only.contains("guide.md"), "{}", only.text());
    term.click(c + 1, r).unwrap();
    term.wait_for("every file back", STEP, |s| {
        s.contains("a.rs") && s.contains("guide.md") && s.contains("[ ] LFS only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The palette command is the keyboard way in.
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle LFS Only").unwrap();
    term.wait_for("the palette narrowed to the command", STEP, |s| {
        s.contains("Command Palette (1/") && s.contains("Files: Toggle LFS Only")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("filtered again, through the palette", STEP, |s| {
        s.contains("hero.bin LFS") && s.contains("[x] LFS only") && !s.contains("guide.md")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

#[test]
fn a_repository_without_lfs_files_never_offers_the_filter() {
    let repo = fixture(false);
    let term = open_tree(&repo, "pty-no-lfs-only");
    let s = term
        .wait_for_stable(Duration::from_millis(500), STEP)
        .unwrap();
    assert!(
        !s.contains("LFS only"),
        "no LFS files, no filter:\n{}",
        s.text()
    );
    let _ = term.stop();
}
