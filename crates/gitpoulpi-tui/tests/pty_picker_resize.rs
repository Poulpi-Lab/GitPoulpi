//! The folder picker (Browse...): its pane divider is a resize line like the
//! Log's and the Radar's: the sideways resize pointer (`ew-resize`) on hover
//! and while dragging, the shared accent colour when lit, and unlit again
//! once the button is up.
//!
//! Real binary in a pty; a temp config dir and fixture repository only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

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

fn wait_shape(term: &Terminal, what: &str, want: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(term).last().map(String::as_str) != Some(want)
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(term).last().map(String::as_str),
        Some(want),
        "{what}: {:?}\n{}",
        pointer_shapes(term),
        term.text()
    );
}

/// The `│` columns on the picker's `row`: the divider is the third, after
/// the window's frame and the dialog's left border.
fn divider_bars(screen: &Screen, row: u16) -> Vec<u16> {
    (0..200u16)
        .filter(|&c| screen.cell(c, row).is_some_and(|x| x.text == "\u{2502}"))
        .collect()
}

/// The picker's divider row and column, waited for as one state with the
/// folders header (a torn frame can show the header before the divider).
fn wait_divider(term: &Terminal) -> (u16, u16) {
    let found = |s: &Screen| {
        let (_, header) = s.find("FOLDERS")?;
        let row = header + 3;
        let bars = divider_bars(s, row);
        (bars.len() >= 3).then(|| (row, bars[2]))
    };
    let screen = term
        .wait_for("the picker's divider", STEP, |s| found(s).is_some())
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                term.text()
            )
        });
    found(&screen).expect("the predicate just matched")
}

fn fg_at(term: &Terminal, col: u16, row: u16) -> Color {
    term.screen().cell(col, row).expect("a cell").fg
}

#[test]
fn the_browse_divider_is_ew_resize_lit_like_the_other_resize_lines_and_unlit_on_release() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "the only commit")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-picker-resize")
        .size(160, 40)
        .ready("the only commit")
        .mouse_capture()
        .start();
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

    let (row, col) = wait_divider(&term);
    // The unlit colour, read once the picker has stopped redrawing.
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let rest = fg_at(&term, col, row);

    // Hover: the sideways resize pointer, and the accent line.
    // The move is repeated (from beside the line, so each one is a real
    // move): one sent before the divider's hit target is registered is lost
    // and no other follows.
    let mut over = false;
    for _ in 0..6 {
        term.move_to(col - 2, row).unwrap();
        term.move_to(col, row).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(4);
        while std::time::Instant::now() < deadline {
            if pointer_shapes(&term).last().map(String::as_str) == Some("ew-resize") {
                over = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if over {
            break;
        }
    }
    wait_shape(&term, "over the divider", "ew-resize");
    let lit = {
        let deadline = std::time::Instant::now() + STEP;
        loop {
            let fg = fg_at(&term, col, row);
            if fg != rest || std::time::Instant::now() > deadline {
                break fg;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    assert_ne!(lit, rest, "the divider lights on hover\n{}", term.text());

    // Dragging keeps the shape and moves the line.
    term.press(col, row).unwrap();
    term.drag_to(col + 6, row).unwrap();
    term.wait_for("the divider moved", STEP, |s| {
        s.cell(col + 6, row).is_some_and(|c| c.text == "\u{2502}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_shape(&term, "while dragging", "ew-resize");
    assert_eq!(fg_at(&term, col + 6, row), lit, "lit while dragged");

    // Released away from the line: no `Moved` follows, it must not stay lit.
    term.release(col + 14, row + 4).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while fg_at(&term, col + 6, row) == lit && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        fg_at(&term, col + 6, row),
        rest,
        "the line stays lit after release\n{}",
        term.text()
    );
}
