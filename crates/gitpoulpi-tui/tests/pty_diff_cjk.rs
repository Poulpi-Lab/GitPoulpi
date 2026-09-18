//! The Split diff with double-width text (Japanese), end to end through a
//! pty: the separator between the halves is on the same screen column on
//! every row, however many wide chars the row holds.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Screen;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

/// Japanese lines of different widths, mixed with ASCII, then a commit
/// editing most of them.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let before = "日本語のテキスト\nplain ascii line\nこんにちは世界\nabc 日本語 def\nlast line\n";
    let after =
        "日本語のテキスト\nplain ascii changed\nこんにちは世界、元気ですか\nabc 日本語 xyz 😀\nlast line\n";
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("nihongo.txt", before, "nihongo")
        .unwrap()
        .commit_file("nihongo.txt", after, "edit nihongo")
        .unwrap()
        .build()
}

/// The screen columns of every `│` on each row that shows diff text.
/// `None` while the diff header is not (fully) painted.
fn separator_columns(screen: &Screen) -> Option<Vec<Vec<u16>>> {
    // The diff pane ends at the corner after its "Esc" button; what lies
    // right of it (the commit details) is not the diff's.
    let (esc, header) = screen.find("\u{2715}")?;
    let right = (esc..screen.cols()).find(|&c| {
        screen
            .cell(c, header)
            .is_some_and(|x| x.display() == "\u{256e}")
    })?;
    // Its left edge too: the panes beside it have borders of their own.
    let left = (0..esc).rev().find(|&c| {
        screen
            .cell(c, header)
            .is_some_and(|x| x.display() == "\u{256d}")
    })?;
    let markers = ["日", "plain ascii", "こ", "abc "];
    let mut rows = Vec::new();
    for row in 0..screen.rows() {
        let line = screen.line(row);
        if !markers.iter().any(|m| line.contains(m)) {
            continue;
        }
        let cols: Vec<u16> = (left + 1..right)
            .filter(|&c| {
                screen
                    .cell(c, row)
                    .is_some_and(|x| x.display() == "\u{2502}")
            })
            .collect();
        rows.push(cols);
    }
    Some(rows)
}

fn check(cols: u16) {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-diff-cjk")
        .config_dir(config.path())
        .size(cols, 40)
        .no_exit_pause()
        .spawn();
    term.wait_for("the commit detail of HEAD", STARTUP, |s| {
        s.contains("Changed files") && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("nihongo.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("\u{2715}") && s.contains("plain ascii changed") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("m").unwrap();
    // Every row the assertion reads comes from the screen the wait returned:
    // the CJK rows and their separators are painted after the first rows.
    let screen = term
        .wait_for("split: every diff row with its separator", STEP, |s| {
            s.text()
                .lines()
                .any(|l| l.contains("plain ascii line") && l.contains("plain ascii changed"))
                && separator_columns(s)
                    .is_some_and(|rows| rows.len() >= 4 && rows.iter().all(|r| !r.is_empty()))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let rows = separator_columns(&screen).expect("the wait guaranteed it");
    assert!(
        rows.iter().all(|r| r == &rows[0]),
        "the separator moved between rows: {rows:?}\n{}",
        term.text()
    );
}

#[test]
fn the_split_separator_is_on_one_column_with_japanese_text() {
    check(160);
}

#[test]
fn the_split_separator_is_on_one_column_in_a_narrow_terminal() {
    check(100);
}
