//! The I-beam over read-only text that can be selected: the code of the Diff
//! view (a diff row is not a control) and of the File View. The bar buttons
//! and the line-number gutter next to it keep their own shapes. Each shape is
//! the `OSC 22` the program wrote, read back from the raw output.
//!
//! Real binary through a pty; temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

/// Moves onto `(col, row)` and waits for the shape `want` to be the last one
/// requested.
fn hover(term: &mut Terminal, what: &str, (col, row): (u16, u16), want: &str) {
    term.move_to(col, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(term).last().map(String::as_str) != Some(want)
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(term).last().map(String::as_str),
        Some(want),
        "over {what}: {:?}\n{}",
        pointer_shapes(term),
        term.text()
    );
}

#[test]
fn the_diff_and_file_view_text_is_an_i_beam_and_the_bar_buttons_keep_the_hand() {
    let before: String = (1..=30).map(|n| format!("row {n:02}\n")).collect();
    let after = before.replace("row 20\n", "row 20 changed\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("big.txt", &before, "thirty rows")
        .unwrap()
        .commit_file("big.txt", &after, "change row 20")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-pointer-text")
        .config_dir(config.path())
        .size(180, 50)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    let (c, r) = term
        .wait_for_text("big.txt", STARTUP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff", STEP, |s| {
        s.contains("row 20 changed") && s.contains(" File View ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let screen = term.screen();
    let code = screen.find("row 20 changed").expect("the diff code");
    let file_view = screen.find(" File View ").expect("the File View button");
    // The gutter: the line-number column left of the code.
    let gutter = (code.0 - 8, code.1);
    // The hand first, so the I-beam is a change that is written.
    hover(
        &mut term,
        "File View button",
        (file_view.0 + 2, file_view.1),
        "pointer",
    );
    hover(&mut term, "the diff code", (code.0 + 3, code.1), "text");
    hover(&mut term, "the diff gutter", gutter, "");
    hover(
        &mut term,
        "File View button again",
        (file_view.0 + 2, file_view.1),
        "pointer",
    );

    // The File View: the same bar, the file's text below.
    term.click(file_view.0 + 2, file_view.1).unwrap();
    term.wait_for("the file's content", STEP, |s| {
        s.contains("File View - big.txt") && s.contains("row 29") && s.contains(" Blame ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let text = screen.find("row 29").expect("the file text");
    let blame = screen.find(" Blame ").expect("the Blame button");
    hover(&mut term, "Blame button", (blame.0 + 2, blame.1), "pointer");
    hover(&mut term, "the file text", (text.0 + 3, text.1), "text");
    hover(&mut term, "the line number", (text.0 - 3, text.1), "");
    hover(
        &mut term,
        "Blame button again",
        (blame.0 + 2, blame.1),
        "pointer",
    );
    term.stop().unwrap();
}
