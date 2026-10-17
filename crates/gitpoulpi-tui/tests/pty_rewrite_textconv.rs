//! The line-ending rule (S22) end to end through a pty (`xtask::pty`): the
//! palette opens the rule, `*.txt` is typed, Preview says what changes,
//! Apply converts the LF file to CRLF in the history and on disk, and Undo
//! puts the old blob back.
//!
//! Same rules as `pty_rewrite.rs`: no coordinates, no sleeps, every wait
//! describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(60);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.md", "keep\r\nme\r\n", "add notes")
        .unwrap()
        .commit_file("win.txt", "a\nb\n", "add win")
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
fn a_linux_file_becomes_windows_in_history_and_on_disk_then_undo_restores_it() {
    let repo = fixture();
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let blob = repo.git(&["rev-parse", "HEAD:win.txt"]);
    let notes = repo.git(&["rev-parse", "HEAD:notes.md"]);
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-eol")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("add win") && s.contains("add notes")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.send_text("convert line endings").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Convert Line Endings...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Rewrite history")
            && s.contains("Except files")
            && s.contains("Convert to")
            && s.contains("CRLF (Windows)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.send_text("*.txt").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the pattern in its field", STEP, |s| s.contains("*.txt"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("F5").unwrap();
    let summary = term
        .wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        summary.contains("1 file version is rewritten"),
        "the summary counts the file:\n{}",
        summary.text()
    );
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        tip,
        "a preview moves nothing"
    );

    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote 1"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["show", "HEAD:win.txt"]), "a\r\nb");
    assert_eq!(
        std::fs::read(repo.root().join("win.txt")).unwrap(),
        b"a\r\nb\r\n",
        "the file on disk follows"
    );
    assert_eq!(
        repo.git(&["rev-parse", "HEAD:notes.md"]),
        notes,
        "a file the filter does not name keeps its blob"
    );

    let (c, r) = term.screen().find("Close").expect("the Close button");
    term.click(c + 1, r).expect("click Close");
    term.wait_for("the workbench closed", STEP, |s| {
        !s.contains("Rewrite history")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-z").unwrap();
    wait_until("the old tip is back", || {
        repo.git(&["rev-parse", "HEAD"]) == tip
    });
    assert_eq!(repo.git(&["rev-parse", "HEAD:win.txt"]), blob);
    assert_eq!(
        std::fs::read(repo.root().join("win.txt")).unwrap(),
        b"a\nb\n"
    );
    let _ = term.stop();
}
