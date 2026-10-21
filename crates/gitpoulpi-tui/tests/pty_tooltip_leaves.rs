#![cfg(feature = "radar")]
//! A tooltip goes as soon as the pointer leaves its target, also while a
//! view that owns the whole body (the Radar) is shown: that view swallows the
//! pointer motion, and the tooltip rule used to never hear that the pointer
//! had left the tab, so the tooltip stayed until something else reset it.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
/// Well under any periodic tick the old behaviour could have waited for.
const SHORT: Duration = Duration::from_millis(1500);
const TIP: &str = "the requests, issues and work in progress";

#[test]
fn the_radar_tab_tooltip_goes_when_the_pointer_leaves_the_tab_over_the_radar() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "tooltip-leave-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-tooltip-leaves")
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("tooltip-leave-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("M-r").unwrap();
    term.wait_for("the Radar", STEP, |s| {
        s.contains("Scope") && s.contains("Open tabs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line = term.screen().line(1);
    let at = line.find("Radar").expect("the Radar tab in the bar");
    let col = line[..at].chars().count() as u16 + 1;
    term.move_to(col, 1).unwrap();
    term.move_to(col + 1, 1).unwrap();
    term.wait_for("the Radar tab tooltip", STEP, |s| s.contains(TIP))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Into the Radar's body, which owns the pointer.
    term.move_to(col, 20).unwrap();
    term.wait_for_text_to_go(TIP, SHORT)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
