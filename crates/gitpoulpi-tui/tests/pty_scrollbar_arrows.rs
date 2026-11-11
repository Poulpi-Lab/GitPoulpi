//! The scrollbars' end arrows, end to end through a pty:
//! the Log's bar draws an up arrow at its top and a down arrow at its
//! bottom, and clicking them scrolls the list by one row. (The commit
//! Description bar's arrows are covered in `pty_input_fields.rs`.)

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Screen;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Number of the first `commit-NN` visible in the Log pane.
fn top_visible(screen: &Screen) -> Option<usize> {
    let text = screen.text();
    let idx = text.find("commit-")?;
    text.get(idx + 7..idx + 9)?.parse().ok()
}

/// `(column, up row, down row)` of the Log's scrollbar arrows.
fn log_arrows(screen: &Screen) -> Option<(u16, u16, u16)> {
    let (_, log_top) = screen.find("\u{256e}\u{256d}")?;
    let (col, up_row) = screen
        .find_all("\u{2191}")
        .into_iter()
        .find(|(_, r)| *r == log_top + 1)?;
    let down_row = screen
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, r)| *c == col && *r > up_row)
        .map(|(_, r)| r)
        .max()?;
    Some((col, up_row, down_row))
}

#[test]
fn the_log_scrollbar_has_end_arrows_that_scroll_one_row() {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for n in 10..=99 {
        builder = builder
            .commit_file("f.txt", &format!("{n}\n"), &format!("commit-{n}"))
            .unwrap();
    }
    let repo = builder.build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-scrollbar-arrows")
        .config_dir(config.path())
        .size(160, 30)
        .spawn();
    // The up arrow is the first cell under the Log's top border; the down
    // arrow is in its column further down (other panes draw bars too, so
    // wait until the Log's own pair is on screen, not just any arrow).
    let screen = term
        .wait_for("HEAD in the Log with its scrollbar", STARTUP, |s| {
            s.contains("commit-99") && log_arrows(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, up_row, down_row) = log_arrows(&screen).expect("the Log's arrows");
    assert!(up_row < down_row);
    assert_eq!(top_visible(&screen), Some(99));

    term.click(col, down_row).expect("click the down arrow");
    term.wait_for("the Log one row down", STEP, |s| top_visible(s) == Some(98))
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, down_row).expect("click again");
    term.wait_for("two rows down", STEP, |s| top_visible(s) == Some(97))
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, up_row).expect("click the up arrow");
    term.wait_for("one row back", STEP, |s| top_visible(s) == Some(98))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn holding_the_down_arrow_keeps_scrolling_until_released() {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for n in 10..=99 {
        builder = builder
            .commit_file("f.txt", &format!("{n}\n"), &format!("commit-{n}"))
            .unwrap();
    }
    let repo = builder.build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-scrollbar-arrow-hold")
        .config_dir(config.path())
        .size(160, 30)
        .spawn();
    let screen = term
        .wait_for("HEAD in the Log with its scrollbar", STARTUP, |s| {
            s.contains("commit-99") && log_arrows(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, _, down_row) = log_arrows(&screen).expect("the Log's arrows");
    assert_eq!(top_visible(&screen), Some(99));

    // Button down on the arrow, no release: one row at once, then it repeats.
    term.press(col, down_row).expect("press the down arrow");
    // Waited for (not a fixed sleep: a torn frame delays what is drawn).
    let held_screen = term
        .wait_for("the held arrow kept scrolling", STEP, |s| {
            top_visible(s).is_some_and(|t| t <= 99 - 5)
        })
        .unwrap_or_else(|e| {
            panic!(
                "holding the arrow must keep scrolling: {e}\n{}",
                term.text()
            )
        });
    let held = top_visible(&held_screen).expect("a Log row");
    assert!(
        held <= 99 - 5,
        "holding the arrow must keep scrolling, top is commit-{held}"
    );

    term.release(col, down_row).expect("release");
    let stopped_screen = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let stopped = top_visible(&stopped_screen).expect("a Log row");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        top_visible(&term.screen()),
        Some(stopped),
        "releasing the button must stop the repeat"
    );
    term.stop().unwrap();
}
