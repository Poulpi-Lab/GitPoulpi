//! The Activity Log's Application tab: one display checkbox per log level
//! (Error, Warning, Info, Debug, Trace) plus All and None, by click and by
//! key. Real binary, daemon and system git through a pty (`xtask::pty`);
//! the remote is unreachable on purpose so a push leaves a warning line next
//! to the info lines.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn has(screen: &Screen, level: &str) -> bool {
    screen
        .text()
        .lines()
        .any(|l| l.contains(&format!(" {level} ")))
}

/// The cell of `needle` on the row that holds the "Show:" label.
fn on_show_row(screen: &Screen, needle: &str) -> (u16, u16) {
    let (_, row) = screen.find("Show:").expect("the Show row");
    screen
        .find_all(needle)
        .into_iter()
        .find(|(_, r)| *r == row)
        .unwrap_or_else(|| panic!("{needle:?} on the Show row\n{}", screen.text()))
}

fn wait(term: &mut Terminal, what: &str, f: impl Fn(&Screen) -> bool) {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn level_checkboxes_hide_and_show_lines_by_click_and_key() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("levels base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("levels unpushed")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        "http://127.0.0.1:1/nowhere.git",
    ]);
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-activity-levels")
        .config_dir(config.path())
        .size(160, 44)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("levels unpushed")
        .spawn();
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
    wait(&mut term, "the failure card", |s| s.contains("Push failed"));
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    wait(&mut term, "the Activity Log", |s| {
        s.contains("Activity Log")
    });
    term.send_keys("Tab").unwrap();
    wait(&mut term, "info and warning lines", |s| {
        s.contains("Show:") && has(s, "INFO") && has(s, "WARN")
    });

    // A click on the Info box hides the info lines, the warning stays.
    let (c, r) = on_show_row(&term.screen(), "[x] Info");
    term.click(c + 1, r).expect("click the Info box");
    wait(&mut term, "no info line", |s| {
        !has(s, "INFO") && has(s, "WARN") && s.contains("[ ] Info")
    });

    // The key brings it back (3 is Info), and takes it away again.
    term.send_text("3").unwrap();
    wait(&mut term, "info back by key", |s| {
        has(s, "INFO") && has(s, "WARN")
    });
    term.send_text("3").unwrap();
    wait(&mut term, "info hidden by key", |s| {
        !has(s, "INFO") && has(s, "WARN")
    });

    // None empties the list; All restores it.
    let (c, r) = on_show_row(&term.screen(), "None");
    term.click(c, r).expect("click None");
    wait(&mut term, "nothing shown", |s| {
        !has(s, "INFO") && !has(s, "WARN") && s.contains("[ ] Warning")
    });
    let (c, r) = on_show_row(&term.screen(), "All");
    term.click(c, r).expect("click All");
    wait(&mut term, "everything back", |s| {
        has(s, "INFO") && has(s, "WARN") && s.contains("[x] Warning")
    });
    term.send_text("n").unwrap();
    wait(&mut term, "nothing shown by key", |s| {
        !has(s, "INFO") && !has(s, "WARN")
    });
    term.send_text("a").unwrap();
    wait(&mut term, "everything back by key", |s| {
        has(s, "INFO") && has(s, "WARN")
    });
}
