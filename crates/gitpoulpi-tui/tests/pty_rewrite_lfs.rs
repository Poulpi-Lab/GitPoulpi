//! The LFS import rule (S21) end to end through a pty (`xtask::pty`): the real
//! binary, daemon, git and git-lfs. The palette opens the rule, `*.bin` is
//! typed, Preview says what moves, Apply rewrites the history, the Log diff of
//! the rewritten commit shows an LFS pointer while the file on disk keeps
//! its real content, and Undo puts the old blob back.
//!
//! Same rules as `pty_rewrite.rs`: no coordinates, no sleeps, every wait
//! describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{require_git_lfs, FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(60);

fn content() -> Vec<u8> {
    (0..4000u32).map(|i| (i * 11) as u8).collect()
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "notes\n", "add notes")
        .unwrap()
        .commit_bytes("art.bin", &content(), "add art")
        .unwrap()
        .build()
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn a_binary_moves_to_lfs_with_a_pointer_in_the_log_and_the_real_file_on_disk() {
    require_git_lfs!("a_binary_moves_to_lfs_with_a_pointer_in_the_log_and_the_real_file_on_disk");
    let repo = fixture();
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let blob = repo.git(&["rev-parse", "HEAD:art.bin"]);
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-lfs")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("add art") && s.contains("add notes")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The palette row opens the workbench with the rule in place.
    term.send_keys("C-p").unwrap();
    term.send_text("migrate files to lfs").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Migrate Files to LFS...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Rewrite history")
            && s.contains("Files to move to LFS")
            && s.contains("Only files of at least")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.send_text("*.bin").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the pattern in its field", STEP, |s| s.contains("*.bin"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Preview: the counts, the file on the Commits tab, nothing changed.
    term.send_keys("F5").unwrap();
    term.wait_for("the summary counting the file", STEP, |s| {
        s.contains("keep their id") && s.contains("moves to LFS")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Retried: a click placed from a torn frame can land nowhere.
    common::retry_until(
        &mut term,
        "the file that becomes a pointer",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Commits") {
                t.click(c + 1, r).expect("click the Commits tab");
            }
        },
        |s| s.contains("moves art.bin to LFS"),
    );
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        tip,
        "a preview moves nothing"
    );

    // Apply.
    term.send_keys("C-s").unwrap();
    // The whole panel: a torn frame shows the first line before its buttons.
    term.wait_for("the result panel with its buttons", STEP, |s| {
        s.contains("Rewrote 1") && s.contains("Purge old history")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let pointer = repo.git(&["show", "HEAD:art.bin"]);
    assert!(
        pointer.starts_with("version https://git-lfs.github.com/spec/v1\noid sha256:"),
        "{pointer}"
    );
    assert!(pointer.ends_with("size 4000"), "{pointer}");
    assert_eq!(
        std::fs::read(repo.root().join("art.bin")).unwrap(),
        content(),
        "the file on disk is the real content"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");

    // The Log: the rewritten commit's diff of the file is an LFS pointer.
    // Retried: the Close button of a torn frame can sit where it is gone.
    common::retry_until(
        &mut term,
        "the workbench closed",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Close") {
                t.click(c + 1, r).expect("click Close");
            }
        },
        |s| !s.contains("Rewrite history"),
    );
    let (col, row) = term
        .wait_for_text("add art", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("select the commit");
    term.wait_for_text("art.bin", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // Retried: under load the click can land on a torn frame of the list.
    common::retry_until(
        &mut term,
        "the LFS card",
        6,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("art.bin") {
                t.click(c + 1, r).expect("open the file's diff");
            }
        },
        |s| s.contains("Git LFS object"),
    );
    term.send_keys("r").unwrap();
    term.wait_for("the raw pointer in the diff", STEP, |s| {
        s.contains("oid sha256") && s.contains("size 4000")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("q").unwrap();

    // Undo: the old commit and the real blob come back; the file stays real.
    term.send_keys("C-z").unwrap();
    wait_until("the old tip is back", || {
        repo.try_git(&["rev-parse", "HEAD"]).as_deref() == Some(tip.as_str())
    });
    assert_eq!(repo.git(&["rev-parse", "HEAD:art.bin"]), blob);
    assert_eq!(
        std::fs::read(repo.root().join("art.bin")).unwrap(),
        content()
    );
    let _ = term.stop();
}
