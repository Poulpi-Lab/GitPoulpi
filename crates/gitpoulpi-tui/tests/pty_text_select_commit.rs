//! The commit details' message block is selectable: a drag over a line of it
//! lights the words, `Ctrl+C` copies them (the status line echoes what went
//! to the clipboard), the pointer is the I-beam over the text, `Esc` drops the
//! selection and a plain click selects nothing. Real binary and daemon
//! through a pty, a fixture repository, no network.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// `ui.text_selection_bg` of the default theme.
const SELECTED: Color = Color::Rgb(64, 156, 255);

/// The lit cells of one screen row.
fn lit_on(s: &Screen, row: u16) -> String {
    (0..s.cols())
        .filter_map(|col| s.cell(col, row))
        .filter(|c| c.bg == SELECTED)
        .map(|c| c.display())
        .collect()
}

fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

#[test]
fn dragging_over_the_commit_message_selects_it_and_ctrl_c_copies_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            "notes.txt",
            "x\n",
            "subject of the commit\n\nalpha beta gamma delta",
        )
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-text-select-commit")
        .size(160, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("alpha beta gamma delta")
        })
        .mouse_capture()
        .start();
    let (col, row) = term
        .wait_for_text("alpha beta gamma delta", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let from = col + "alpha ".len() as u16;
    let to = from + "beta gamma".len() as u16 - 1;

    // The I-beam over the message text.
    term.move_to(from + 1, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(&term).last().map(String::as_str) != Some("text")
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(&term).last().map(String::as_str),
        Some("text"),
        "{:?}",
        pointer_shapes(&term)
    );

    // The drag lights those words and nothing else on the row.
    term.drag((from, row), (to, row)).unwrap();
    term.wait_for("the words highlighted", STEP, |s| {
        lit_on(s, row) == "beta gamma"
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}\nLIT={:?}\n{}",
            lit_on(&term.screen(), row),
            term.text()
        )
    });

    // Ctrl+C copies them.
    term.send_keys("C-c").unwrap();
    term.wait_for("the copy reported", STEP, |s| {
        s.contains("copied the selection (beta gamma)") || s.contains("no clipboard available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc drops it; the Log stays.
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight", STEP, |s| lit_on(s, row).is_empty())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A plain click selects nothing.
    term.click(from, row).unwrap();
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit_on(&term.screen(), row).is_empty(), "a click selected");
    term.stop().unwrap();
}
