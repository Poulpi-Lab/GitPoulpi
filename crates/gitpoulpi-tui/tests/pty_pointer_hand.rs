//! The pointing hand over the tab bar: a repository tab, Home and the Radar
//! tab all show the same `pointer` shape (OSC 22), and it is written on a
//! change only.
//!
//! Real binary through a pty; temp config dir and a fixture repository.

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
fn a_repository_tab_home_and_the_radar_tab_all_show_the_pointing_hand() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pointer-hand-commit")
        .unwrap()
        .build();
    let name = repo
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-pointer-hand")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the window", STARTUP, |s| s.contains("pointer-hand-commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the Radar tab", STEP, |s| s.line(1).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bar = term.screen().line(1);
    let col_of = |needle: &str| -> u16 {
        let at = bar
            .find(needle)
            .unwrap_or_else(|| panic!("no {needle:?} on the tab bar: {bar:?}"));
        bar[..at].chars().count() as u16 + 1
    };
    // Not on a column grip: the last column's grip line runs down the table at
    // x = 90 at this width.
    let empty = (60, 20);
    // The arrow over nothing, then each tab in turn, with the arrow between
    // so every hover is a change that is written.
    for (what, spot) in [
        ("the repository tab", (col_of(&name), 1)),
        ("Home", (col_of("\u{2302}"), 1)),
        ("the Radar tab", (col_of("Radar"), 1)),
    ] {
        hover(&mut term, what, spot, "pointer");
        // Moving along the same element writes nothing more.
        let seen = pointer_shapes(&term).len();
        term.move_to(spot.0 + 1, spot.1).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(pointer_shapes(&term).len(), seen, "over {what}");
        hover(&mut term, "an empty area", empty, "");
    }
    term.stop().unwrap();
}
