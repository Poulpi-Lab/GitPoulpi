//! The folder picker's content list with many folders, through a pty
//! (`xtask::pty`): the list has a scrollbar, the thumb drags, the wheel
//! scrolls the list under the pointer and the keys keep the selection in
//! view. A temp config dir and a temp folder tree only; the rules are
//! `pty_ui.rs`'s (nothing sleeps, every wait names its state).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const DIRS: usize = 60;

/// Every column of `row` that draws `glyph` (two lists can have a bar on one row).
fn glyph_cols(s: &Screen, row: u16, glyph: &str) -> Vec<u16> {
    (0..s.cols())
        .filter(|&c| s.cell(c, row).is_some_and(|cell| cell.display() == glyph))
        .collect()
}

/// `(first row, last row, bar column)` of the content list's bar: the up
/// arrow on the ".." row and the down arrow, in the same column, below it.
fn bar_ends(s: &Screen) -> Option<(u16, u16, u16)> {
    let first = (0..s.rows()).find(|&r| s.line(r).contains(".. "))?;
    let col = *glyph_cols(s, first, "\u{2191}").last()?;
    let last = (first + 1..s.rows()).find(|&r| glyph_cols(s, r, "\u{2193}").contains(&col))?;
    Some((first, last, col))
}

fn open_picker_in(dir: &std::path::Path) -> (Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "the only commit")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-folder-scrollbars")
        .size(160, 40)
        .ready("the only commit")
        .mouse_capture()
        .start();
    // The fixture must outlive the terminal.
    std::mem::forget(repo);
    term.send_keys("C-o").unwrap();
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").unwrap();
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("Open a repository") && s.contains("FOLDERS")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    term.send_text(&dir.display().to_string().replace('\\', "/"))
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder listed", STEP, |s| s.contains("dir00/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config)
}

#[test]
fn the_content_list_has_a_bar_that_drags_the_wheel_scrolls_and_the_keys_follow() {
    let tree = tempfile::tempdir().unwrap();
    for i in 0..DIRS {
        std::fs::create_dir(tree.path().join(format!("dir{i:02}"))).unwrap();
    }
    let (mut term, _config) = open_picker_in(tree.path());
    let start = term
        .wait_for("the list and its bar", STEP, |s| bar_ends(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (first, last, col) = bar_ends(&start).unwrap();
    assert!(!start.contains("dir59/"), "the list overflows");

    // Drag the thumb (right under the up arrow) past the bottom.
    term.drag((col, first + 1), (col, last + 3)).unwrap();
    term.wait_for("the list at its end", STEP, |s| {
        s.contains("dir59/") && !s.contains("dir00/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The wheel over the list scrolls it back up.
    for _ in 0..30 {
        term.scroll(false, col - 10, first + 2).unwrap();
    }
    term.wait_for("the list back at its start", STEP, |s| {
        s.contains("dir00/") && !s.contains("dir59/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click focuses the list; End moves the selection to the last folder
    // and the list follows it.
    let (c, r) = term.screen().find("dir00/").unwrap();
    term.click(c + 1, r).unwrap();
    term.send_keys("End").unwrap();
    term.wait_for("the last folder in view", STEP, |s| s.contains("dir59/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
