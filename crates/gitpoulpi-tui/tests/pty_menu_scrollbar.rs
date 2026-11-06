//! A context menu taller than the window: the Log commit menu in a short
//! terminal. Its right border carries the scrollbar; the thumb drags (the
//! menu stays open), the wheel over the menu scrolls it, and the keys keep
//! moving the selection with the window following it.
//!
//! Real binary in a pty; a temp config dir only. The rules are `pty_ui.rs`'s.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);

fn is(s: &Screen, col: u16, row: u16, glyph: &str) -> bool {
    s.cell(col, row).is_some_and(|c| c.display() == glyph)
}

/// `(first row, last row, bar column)` of the menu's bar: the end arrows in
/// the column of a rounded frame's right corners.
fn menu_bar(s: &Screen) -> Option<(u16, u16, u16)> {
    for (col, up) in s.find_all("\u{2191}") {
        if up == 0 || !is(s, col, up - 1, "\u{256E}") {
            continue;
        }
        let down = (up + 1..s.rows()).find(|&r| is(s, col, r, "\u{2193}"))?;
        if is(s, col, down + 1, "\u{256F}") {
            return Some((up, down, col));
        }
    }
    None
}

/// What the menu shows: its rows, up to the bar.
fn shown(s: &Screen) -> Vec<String> {
    let (first, last, col) = menu_bar(s).expect("the menu's bar");
    (first..=last)
        .map(|r| {
            (0..col)
                .filter_map(|c| s.cell(c, r).map(|cell| cell.display().to_string()))
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// The screen once the menu's window has stopped moving: the same rows on
/// several reads in a row, 100 ms apart (a state check, not a blind wait).
fn settled(term: &mut xtask::pty::Terminal) -> Screen {
    let mut last = term.screen();
    let mut same = 0;
    for _ in 0..300 {
        std::thread::sleep(Duration::from_millis(100));
        let now = term.screen();
        if menu_bar(&now).is_some() && menu_bar(&last).is_some() && shown(&now) == shown(&last) {
            same += 1;
            if same >= 5 {
                return now;
            }
        } else {
            same = 0;
        }
        last = now;
    }
    panic!("the menu never settled\n{}", term.text());
}

#[test]
fn a_tall_menu_has_a_bar_that_drags_the_wheel_scrolls_and_the_keys_follow() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("b.txt", "two\n", "add b")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-menu-scrollbar")
        .config("[ui]\ntoolbar_position = \"top\"\n")
        .size(120, 14)
        .ready_when("the fixture's log", |s| {
            s.contains("add b") && s.contains("add a")
        })
        .mouse_capture()
        .start();
    common::retry_until(
        &mut term,
        "the Log menu with its bar",
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = t.screen().find("add b") {
                t.right_click(c + 1, r).unwrap();
            }
        },
        |s| menu_bar(s).is_some(),
    );
    let start = term.screen();
    let (first, last, col) = menu_bar(&start).unwrap();
    let before = shown(&start);

    // Drag the thumb (right under the up arrow) past the bottom: the menu
    // scrolls and stays open.
    term.drag((col, first + 1), (col, last + 2)).unwrap();
    term.wait_for("the menu scrolled by its thumb", STEP, |s| {
        menu_bar(s).is_some() && shown(s) != before
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The first frame that differs can be mid-drag under load: the drag
    // ends at the bottom, so take the window once it stops moving.
    let dragged = settled(&mut term);

    // The wheel over the menu scrolls it back to the top.
    for _ in 0..10 {
        term.scroll(false, col - 3, first + 2).unwrap();
    }
    term.wait_for("the menu back at its top", STEP, |s| {
        menu_bar(s).is_some() && shown(s) == before
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // End moves the selection to the last row and the window follows it.
    term.send_keys("End").unwrap();
    let end = term
        .wait_for("the window at the last row", STEP, |s| {
            menu_bar(s).is_some() && shown(s) == shown(&dragged)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(shown(&end), before);

    // Esc closes it.
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| menu_bar(s).is_none())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
