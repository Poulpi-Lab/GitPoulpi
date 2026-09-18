//! The editor's syntax highlighting catches up while nothing is touched:
//! the real binary opens a long Rust file, jumps to its end (a parse of
//! every line above, far more than one frame's budget), and then receives
//! no input at all. The "highlighting..." marker must clear and the last
//! lines must be colored without a key or mouse move to wake the loop.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Color;

const STEP: Duration = Duration::from_secs(60);
/// The parse of 40 000 lines in a debug build is real CPU work: on an
/// oversubscribed machine (parallel builds) it outlasts a minute. The deadline
/// only bounds a failing wait; a passing one returns as soon as it is drawn.
const HIGHLIGHT: Duration = Duration::from_secs(240);
const LOG_HEADER: &str = "Message";
const LINES: usize = 40_000;

fn rust_text(edited: bool) -> String {
    let mut text: String = (1..=LINES)
        .map(|n| format!("fn item_{n:05}() {{ let x = {n}; }}\n"))
        .collect();
    if edited {
        text.push_str("// edited\n");
    }
    text
}

#[test]
fn an_idle_editor_finishes_highlighting_without_any_input() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("big.rs", rust_text(false).as_bytes())
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("big.rs", rust_text(true).as_bytes())
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-editor-highlight-idle")
        .size(240, 40)
        .no_exit_pause()
        .ready_when("the repository window", |s| {
            s.contains("big.rs") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start();

    let (c, r) = term
        .wait_for_text("big.rs", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("big.rs - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - big.rs @ working tree") && s.contains("item_00001")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The jump: one key, then silence.
    term.send_text("\x1b[1;5F").unwrap();
    let last = format!("item_{LINES:05}");
    // The colors are part of the wait, not asserted after it: the marker can
    // be gone and the text drawn a frame before the colored spans are (the
    // parse finishes, the frame that paints it is still to come), and a
    // snapshot in between read keyword and name both in the default color.
    let colors = |s: &xtask::pty::Screen| {
        let (c, r) = s.find(&format!("fn {last}"))?;
        Some((s.cell(c, r)?.fg, s.cell(c + 3, r)?.fg))
    };
    let s = term
        .wait_for("the end of the file, fully highlighted", HIGHLIGHT, |s| {
            s.contains(&last)
                && s.contains("edited")
                && s.contains("Rust")
                && !s.contains("highlighting")
                && matches!(colors(s), Some((keyword, ident))
                    if keyword != Color::Default && ident != Color::Default && keyword != ident)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (keyword, ident) = colors(&s).expect("a line of code");
    assert_ne!(keyword, Color::Default, "the keyword is colored");
    assert_ne!(ident, Color::Default, "the name is colored");
    assert_ne!(keyword, ident, "keyword and name differ");
}
