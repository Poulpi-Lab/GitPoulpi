//! The Diff view's encoding dropdown, end to end through a pty: a Latin-1
//! file reads right once Windows-1252 is chosen (by key and list), a UTF-16
//! file - which the backends call binary - shows its lines once UTF-16 LE is
//! chosen (by mouse), and the choice is remembered for the path.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    let term = common::launch(repo)
        .no_commit_column()
        .session(session)
        .config_dir(config)
        .size(160, 50)
        .no_exit_pause()
        .spawn();
    term.wait_for("the commit detail of HEAD", STARTUP, |s| {
        s.contains("View all files") && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn open_the_diff(term: &mut Terminal, file: &str, waiting_for: &str) {
    let (c, r) = term
        .wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("\u{2715}") && s.contains(waiting_for) && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_latin_1_file_reads_right_once_the_encoding_is_chosen_and_the_choice_is_remembered() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("latin.txt", b"caf\xe9\nna\xefve\n", "latin one")
        .unwrap()
        .commit_bytes(
            "latin.txt",
            b"caf\xe9\nna\xefve\nd\xe9j\xe0 vu\n",
            "add a line",
        )
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-diff-encoding-latin");
    open_the_diff(&mut term, "latin.txt", "vu");
    // As the backend reads it: not what the file says.
    let screen = term.screen();
    assert!(
        !screen.contains("d\u{e9}j\u{e0} vu"),
        "not decoded yet:\n{}",
        screen.text()
    );
    assert!(screen.contains("Auto"), "{}", screen.text());

    // The key opens the list: Auto-detect first, then the editor's encodings.
    term.send_text("C").unwrap();
    let screen = term
        .wait_for("the encoding list", STEP, |s| {
            s.contains("Auto-detect") && s.contains("Windows-1252") && s.contains("UTF-16 LE")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, auto_row) = screen.find("Auto-detect").unwrap();
    let (_, utf8_row) = screen.find("UTF-8").unwrap();
    assert!(auto_row < utf8_row, "Auto-detect first:\n{}", screen.text());

    // Typing narrows the list, Enter picks: both sides are read again.
    term.send_text("1252").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the file read as Windows-1252", STEP, |s| {
        s.contains("d\u{e9}j\u{e0} vu") && s.contains("Windows-1252 \u{25be}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    assert!(
        !screen.contains("\u{fffd}"),
        "no replacement character:\n{}",
        screen.text()
    );

    // Staging is off while an encoding is chosen: the hunk ids would name
    // other hunks. (A commit's diff has no staging; the word says so on the
    // working tree - see the unit tests.) Closing and reopening keeps the
    // choice for the path.
    term.send_keys("Escape").unwrap();
    term.wait_for("the graph back, and the Refs pane with it", STEP, |s| {
        s.contains(LOG_HEADER) && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The layout is still settling for a moment (a click lands on its old rows).
    term.wait_for_stable(Duration::from_millis(500), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    open_the_diff(&mut term, "latin.txt", "Windows-1252");
    term.wait_for("remembered, and read again", STEP, |s| {
        s.contains("d\u{e9}j\u{e0} vu")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Back to Auto-detect, by the palette's twin of the key.
    term.send_keys("C-p").unwrap();
    term.send_text("Diff: Choose Encoding").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Diff: Choose Encoding...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the list again", STEP, |s| s.contains("Auto-detect"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Home").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Auto-detect again", STEP, |s| {
        !s.contains("d\u{e9}j\u{e0} vu") && s.contains("Auto \u{25be}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_utf_16_file_shows_its_lines_once_utf_16_is_chosen_by_mouse() {
    let utf16 = |text: &str| -> Vec<u8> {
        let mut out = vec![0xFF, 0xFE];
        for unit in text.encode_utf16() {
            out.extend_from_slice(&unit.to_le_bytes());
        }
        out
    };
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_bytes("wide.txt", &utf16("one\ntwo\n"), "wide one")
        .unwrap()
        .commit_bytes("wide.txt", &utf16("one\ntwo\ntr\u{e8}s\n"), "add a line")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-diff-encoding-wide");
    // Git calls it binary: no lines.
    open_the_diff(&mut term, "wide.txt", "Auto");
    assert!(!term.screen().contains("tr\u{e8}s"));

    // The bar's button, then the row, by the mouse.
    let (c, r) = term.screen().find("Auto \u{25be}").expect("the button");
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the encoding list", STEP, |s| s.contains("UTF-16 LE"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("UTF-16 LE").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the file read as UTF-16 LE", STEP, |s| {
        s.contains("tr\u{e8}s") && s.contains("UTF-16 LE \u{25be}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
