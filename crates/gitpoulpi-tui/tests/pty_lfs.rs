//! Git LFS in the TUI, driven through a pty by `xtask::pty`: an LFS file
//! wears an `LFS` tag and its diff is a card, and a repository that needs
//! `git-lfs` where it is missing gets a banner -- never a crash.
//!
//! No hard-coded coordinates: every position is found on the screen, and
//! every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{require_git_lfs, FixtureRepoBuilder};
use xtask::pty::Terminal;

// LFS operations spawn several git / git-lfs processes each; on a loaded
// Windows machine that is slow, so the waits are generous.
const STEP: Duration = Duration::from_secs(60);

fn launch(repo: &Path, session: &str, config: &Path, extra_env: &[(&str, &str)]) -> Terminal {
    let mut launch = common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(160, 50)
        .no_exit_pause()
        .env("GITPOULPI_NO_BROWSER", "1")
        .ready("add big");
    for (key, value) in extra_env {
        launch = launch.env(key, value);
    }
    launch.spawn()
}

/// Ends the session. The kill can land while the TUI's daemon is in the
/// middle of an LFS read - a short chain of `git` / `git-lfs` processes,
/// slow on a loaded Windows box - and those children still hold the pty
/// when `stop` looks for the reader to come home. That is the OS taking a
/// moment to tidy up, not what these tests are about, so the stop error is
/// not an assertion here (the `Drop` of `Terminal` cleans up regardless).
fn finish(term: Terminal) {
    let _ = term.stop();
}

/// Selects the WIP row (its graph marker) so the Status pane lists the
/// working changes.
fn select_the_working_changes(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
}

#[test]
fn an_lfs_file_wears_a_tag_and_its_diff_is_a_card_that_r_turns_into_the_pointer() {
    require_git_lfs!(
        "an_lfs_file_wears_a_tag_and_its_diff_is_a_card_that_r_turns_into_the_pointer"
    );
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", &vec![1u8; 1500], "add big")
        .unwrap()
        .commit_file("plain.txt", "one\n", "add plain")
        .unwrap()
        .write_worktree("big.bin", &vec![2u8; 4000])
        .unwrap()
        .write_worktree("plain.txt", b"two\n")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-lfs-card", config.path(), &[]);
    select_the_working_changes(&mut term);

    let screen = term
        .wait_for("both changed files, the LFS one tagged", STEP, |s| {
            s.contains("big.bin LFS") && s.contains("plain.txt")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("plain.txt LFS"),
        "a plain file has no tag:\n{}",
        screen.text()
    );

    let (col, row) = screen.find("big.bin LFS").unwrap();
    term.click(col + 1, row).unwrap();
    let card = term
        .wait_for("the LFS card", STEP, |s| {
            s.contains("Git LFS object") && s.contains("1.5 KB -> 4.0 KB")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !card.contains("oid sha256"),
        "the card replaces the pointer text:\n{}",
        card.text()
    );

    term.send_keys("r").unwrap();
    term.wait_for("the raw pointer diff", STEP, |s| s.contains("oid sha256"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("q").unwrap();
    term.wait_for("the diff closed, the window still open", STEP, |s| {
        !s.contains("oid sha256") && s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "q closes the diff and only the diff");
    finish(term);
}

/// `GITPOULPI_GIT_LFS=missing` is the test seam that makes the probe answer
/// "not installed" (a `PATH` edit cannot hide git-lfs on Windows). Needs
/// no `git-lfs` itself: the repository only has to *use* LFS.
#[test]
fn a_repo_needing_lfs_without_the_program_shows_a_banner_and_nothing_crashes() {
    let oid = "c".repeat(64);
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            ".gitattributes",
            "*.bin filter=lfs diff=lfs merge=lfs -text\n",
            "attributes",
        )
        .unwrap()
        .commit_pointer_text("big.bin", &oid, 12_000_000, "add big")
        .unwrap()
        .write_worktree("big.bin", b"replaced by plain text\n")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    // Hermetic: whether git-lfs is installed, and whether its clean/smudge
    // filters are in the machine's global or system git config, must not
    // change what "no git-lfs" looks like. Pointing git at an empty global
    // and system config leaves `filter.lfs.*` undefined for the app and its
    // git children, so nothing cleans the replaced file.
    let empty = config.path().join("empty.gitconfig");
    std::fs::write(&empty, "").unwrap();
    let empty = empty.to_str().unwrap().to_owned();
    let mut term = launch(
        repo.root(),
        "pty-lfs-missing",
        config.path(),
        &[
            ("GITPOULPI_GIT_LFS", "missing"),
            ("GIT_CONFIG_GLOBAL", &empty),
            ("GIT_CONFIG_SYSTEM", &empty),
            ("GIT_CONFIG_NOSYSTEM", "1"),
        ],
    );
    select_the_working_changes(&mut term);

    let screen = term
        .wait_for("the banner naming the install page", STEP, |s| {
            s.contains("large files show as pointers") && s.contains("git-lfs.com")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The banner is inside the Status pane, under its title row.
    let (_, banner_row) = screen.find("large files show as pointers").unwrap();
    let (_, title_row) = screen.find("Unstaged Files").unwrap();
    assert!(
        banner_row < title_row,
        "banner above the file lists:\n{}",
        screen.text()
    );

    // It is a link: clicking opens the install page (the test suppresses
    // the browser, the status line still says so).
    let (col, row) = screen.find("git-lfs.com").unwrap();
    term.click(col, row).unwrap();
    term.wait_for("the install page reported opened", STEP, |s| {
        s.contains("opened https://git-lfs.com")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Opening a diff of the LFS file works and never takes the window
    // down.
    let (col, row) = term
        .wait_for_text("big.bin", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).unwrap();
    term.wait_for("the LFS card for the replaced pointer", STEP, |s| {
        s.contains("Git LFS object") && s.contains("removed: 12.0 MB")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "the window survives a missing git-lfs");

    term.send_keys("q").unwrap();
    term.wait_for("the diff closed", STEP, |s| !s.contains("Git LFS object"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "q closes the diff only");
    finish(term);
}

/// The palette's `LFS: Track Pattern...` asks for a pattern in the footer
/// prompt (no mouse needed), and `git lfs track` then leaves
/// `.gitattributes` as a visible, unstaged change.
#[test]
fn the_palette_tracks_a_pattern_and_gitattributes_shows_up_unstaged() {
    require_git_lfs!("the_palette_tracks_a_pattern_and_gitattributes_shows_up_unstaged");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", &vec![1u8; 500], "add big")
        .unwrap()
        .write_worktree("scratch.txt", b"so the working changes are listed\n")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-lfs-track", config.path(), &[]);
    select_the_working_changes(&mut term);
    term.wait_for("the working changes", STEP, |s| {
        s.contains("Unstaged Files") && s.contains("scratch.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").expect("open the palette");
    term.send_text("LFS: Track Pattern").unwrap();
    // "Track Pattern" also matches "Untrack Pattern"; the exact prefix
    // match ranks first and Enter takes the top row.
    term.wait_for(
        "the palette narrowed to the two pattern commands",
        STEP,
        |s| s.contains("Command Palette (2/") && s.contains("LFS: Track Pattern..."),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the pattern prompt", STEP, |s| {
        s.contains("Track with LFS (pattern") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("*.psd").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for(".gitattributes listed as an unstaged change", STEP, |s| {
        s.contains(".gitattributes") && s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let attributes = std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap();
    assert!(attributes.contains("*.psd filter=lfs"), "{attributes}");
    assert!(
        repo.git(&["diff", "--cached", "--name-only"]).is_empty(),
        ".gitattributes is left unstaged"
    );
    finish(term);
}

/// The file's right-click menu offers `Lock file (LFS)` for an LFS file;
/// against the testkit's loopback lock server the lock is taken and the
/// row says who holds it. Nothing is hard-coded: the menu row and the file
/// row are found on the screen.
#[test]
fn the_file_menu_locks_an_lfs_file_and_the_row_says_it_is_locked() {
    require_git_lfs!("the_file_menu_locks_an_lfs_file_and_the_row_says_it_is_locked");
    let server = gitpoulpi_testkit::lfs::LfsLockServer::start();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", &vec![1u8; 500], "add big")
        .unwrap()
        .config("lfs.url", &server.url())
        .unwrap()
        .config("credential.helper", "")
        .unwrap()
        .write_worktree("big.bin", &vec![2u8; 900])
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-lfs-lock", config.path(), &[]);
    select_the_working_changes(&mut term);

    let screen = term
        .wait_for("the LFS file row", STEP, |s| s.contains("big.bin LFS"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("big.bin LFS").unwrap();
    term.right_click(col + 1, row).unwrap();
    // The LFS rows live in the menu's `LFS >` submenu (drill-in).
    let submenu = term
        .wait_for("the file menu with its LFS submenu", STEP, |s| {
            s.contains("LFS \u{203A}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (scol, srow) = submenu.find("LFS \u{203A}").unwrap();
    term.click(scol + 1, srow).unwrap();
    let menu = term
        .wait_for("the LFS submenu with the lock row", STEP, |s| {
            s.contains("Lock file (LFS)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (mcol, mrow) = menu.find("Lock file (LFS)").unwrap();
    term.click(mcol + 1, mrow).unwrap();

    term.wait_for("the row saying who holds the lock", STEP, |s| {
        s.contains("big.bin LFS locked by you")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(server.locked_paths(), ["big.bin"]);
    finish(term);
}
