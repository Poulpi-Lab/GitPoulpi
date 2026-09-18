//! Selectable, copyable text in the Diff view, end to end through a pty: the
//! real binary, the real daemon, real `git`.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded
//! coordinate, and every wait describes the whole state it waits for.
//! The highlight is asserted on the cells' background color, which the
//! pty's own screen model keeps.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";
/// `text_field::SELECTED_TEXT_BG`, the text fields' own selection fill.
const SELECTED: Color = Color::Rgb(64, 156, 255);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(160, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("notes.txt") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// The text of the lit (selected) cells inside the diff body, row by
/// row. Only the body: the bar's active buttons share the same accent
/// fill. The body is found from its own drawing - below the `@@` header
/// row, left of the title's `✕`.
fn lit(s: &Screen) -> String {
    let (Some((_, top)), Some((right, _))) = (s.find("@@ -1,1"), s.find("\u{2715}")) else {
        return String::from("<no diff on screen>");
    };
    let mut out = String::new();
    for row in top..s.rows() {
        for col in 0..right {
            if let Some(cell) = s.cell(col, row) {
                if cell.bg == SELECTED {
                    out.push_str(cell.display());
                }
            }
        }
    }
    out
}

#[test]
fn dragging_over_diff_text_highlights_it_and_y_copies_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "alpha beta gamma\n", "first")
        .unwrap()
        .commit_file(
            "notes.txt",
            "alpha beta gamma\nsecond LINE here\n",
            "add a line",
        )
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-text-select");

    // Open the changed file's diff from the commit-detail list.
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (c, r) = term
        .wait_for_text("second LINE here", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line_col = c + "second ".len() as u16;

    // A drag over "LINE" lights exactly those four cells.
    term.drag((line_col, r), (line_col + 3, r)).unwrap();
    term.wait_for("LINE highlighted, nothing else", STEP, |s| lit(s) == "LINE")
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `y` copies it -- the status line echoes what went to the
    // clipboard (or says why there is none on this machine).
    term.send_keys("y").unwrap();
    term.wait_for("the copy reported", STEP, |s| {
        s.contains("copied the selection (LINE)") || s.contains("no clipboard available")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A double-click selects the word under the pointer.
    let (g, gr) = term
        .wait_for_text("gamma", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.double_click(g + 1, gr).unwrap();
    term.wait_for("gamma highlighted", STEP, |s| lit(s) == "gamma")
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc clears the text selection first -- the diff stays open.
    term.send_keys("Esc").unwrap();
    term.wait_for("no highlight, the diff still open", STEP, |s| {
        lit(s).is_empty() && s.contains("second LINE here") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A plain click selects no text.
    term.click(line_col, r).unwrap();
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit(&term.screen()).is_empty(), "a click selected text");
}
