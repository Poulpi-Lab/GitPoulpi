//! A dialog that is not a full page (the Activity Log, the Undo history)
//! leaves the live main window visible around its box, like the command
//! palette: the Log's text outside the box stays.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(60);
const COLS: u16 = 300;
const ROWS: u16 = 50;
const MESSAGE: &str = "overlay keeps the log probe";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

#[test]
fn the_log_text_behind_the_activity_dialog_stays_visible_outside_its_box() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-overlay-keeps-log")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .ready(MESSAGE)
        .spawn();
    let (mx, my) = term.wait_for_text(MESSAGE, STEP).unwrap();
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    let open = term
        .wait_for("the Activity Log", STEP, |s| s.contains("Activity Log"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The dialog is 120 wide and 40 high, centred on 300x50: the Log's first
    // row (and the sidebar) are outside it.
    let (dx, dy) = open.find_all("Activity Log").into_iter().next().unwrap();
    assert!(my < dy && mx < dx + 120, "the probe row is outside the box");
    let screen = term
        .wait_for("the Log row still drawn", STEP, |s| {
            s.find_all(MESSAGE).contains(&(mx, my))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("LOCAL"), "the Refs sidebar is still drawn");
    assert!(screen.contains("Commit details"), "the right pane is too");
    let _ = term.stop();
}

/// On a Kitty terminal the Graph picture is not deleted when the dialog
/// covers most of its column: it stays, and the dialog's own cells cover it.
#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the kitty graphics APC from the stream the driver reads"
)]
fn the_graph_picture_is_not_deleted_while_the_activity_dialog_covers_it() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-overlay-keeps-graph")
        .config("[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"kitty\"\n")
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .size(200, 40)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start();
    let quiet = Duration::from_millis(800);
    term.wait_for_stable(quiet, STEP).unwrap();
    assert!(
        count(&term.raw_output(), b"_Ga=T,") > 0,
        "the Log never sent a picture"
    );
    let before = term.raw_output().len();
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    term.wait_for("the Activity Log", STEP, |s| s.contains("Activity Log"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_stable(quiet, STEP).unwrap();
    let raw = term.raw_output();
    assert_eq!(
        count(&raw[before..], b"a=d"),
        0,
        "the picture was deleted under the dialog"
    );
    let _ = term.stop();
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}
