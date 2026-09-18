//! End-to-end checks of the Refs sidebar's resizable sub-sections and of
//! Settings landing back on its last section, driven through a pty by
//! `xtask::pty` -- the real binary, the real daemon, real `git`.
//!
//! The fixture has twelve local branches and six tags, so at 34 rows the
//! sections cannot all show their content and each one scrolls on its own.

mod common;

use common::git_ok;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 120;
const ROWS: u16 = 34;
const SIDEBAR_RIGHT: u16 = 24;

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    for i in 1..=12 {
        git_ok(&root, &["branch", &format!("br{i:02}")]);
    }
    for i in 1..=6 {
        git_ok(&root, &["tag", &format!("v{i}.0")]);
    }
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the log and the sidebar sections", |s| {
            s.contains("second")
                && header_row(s, "LOCAL (").is_some()
                && header_row(s, "TAGS (").is_some()
        })
        .spawn()
}

fn header_row(screen: &Screen, header: &str) -> Option<u16> {
    screen
        .find_all(header)
        .into_iter()
        .find(|(col, _)| *col < SIDEBAR_RIGHT)
        .map(|(_, row)| row)
}

/// Waits for the sidebar to hold `header` at `row`.
fn wait_header_at(term: &Terminal, header: &str, row: u16) -> Screen {
    term.wait_for(&format!("{header} header on row {row}"), STEP, |s| {
        header_row(s, header) == Some(row)
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// The row of `header` once the layout has settled.
fn settled(term: &Terminal, header: &str) -> u16 {
    let screen = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    header_row(&screen, header).expect("header on screen")
}

#[test]
fn dragging_a_sections_bottom_border_resizes_it_and_the_size_survives_a_restart() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "sb-drag", config.path());

    let remote = settled(&term, "REMOTE (");
    let local = settled(&term, "LOCAL (");
    // LOCAL's bottom border is the row above REMOTE's header.
    let border = remote - 1;
    assert!(border - local >= 5, "LOCAL must be tall enough to shrink");

    term.drag((6, border), (6, border - 3)).unwrap();
    wait_header_at(&term, "REMOTE (", remote - 3);
    term.stop().unwrap();

    // Same session, same config dir: the size was persisted.
    let term = launch(repo.root(), "sb-drag", config.path());
    wait_header_at(&term, "REMOTE (", remote - 3);
    term.stop().unwrap();
}

#[test]
fn the_keyboard_and_the_palette_resize_the_active_section() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "sb-keys", config.path());

    let remote = settled(&term, "REMOTE (");
    let local = settled(&term, "LOCAL (");
    // Focus LOCAL: a click on its first branch row.
    term.click(8, local + 1).unwrap();
    term.wait_for("the click landed", STEP, |s| {
        header_row(s, "LOCAL (").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").unwrap();
    term.send_keys("-").unwrap();
    wait_header_at(&term, "REMOTE (", remote - 1);

    term.send_keys("C-n").unwrap();
    term.send_keys("+").unwrap();
    wait_header_at(&term, "REMOTE (", remote);

    term.send_keys("C-p").unwrap();
    term.send_text("Shrink Refs Section").unwrap();
    term.send_keys("Enter").unwrap();
    wait_header_at(&term, "REMOTE (", remote - 1);

    term.send_keys("C-p").unwrap();
    term.send_text("Reset Refs Section Sizes").unwrap();
    term.send_keys("Enter").unwrap();
    wait_header_at(&term, "REMOTE (", remote);

    term.send_keys("C-p").unwrap();
    term.send_text("Grow Refs Section").unwrap();
    term.send_keys("Enter").unwrap();
    // LOCAL grew: the neighbors below it move down (or, when the column
    // is full, the automatic sections give the row back -- REMOTE moves
    // either way once LOCAL is user-sized).
    let screen = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap();
    assert_ne!(header_row(&screen, "REMOTE ("), Some(remote - 1));
    term.stop().unwrap();
}

#[test]
fn settings_reopens_on_the_section_it_was_left_on() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "sb-settings", config.path());

    term.send_keys("C-n").unwrap();
    term.send_keys(",").unwrap();
    term.wait_for("Settings open", STEP, |s| s.contains("Git & encoding"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The sections are focused when it opens: move down to Profiles
    // (past Synchronization).
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("Profiles' body", STEP, |s| s.contains("Profiles - global"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Git & encoding", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").unwrap();
    term.send_keys(",").unwrap();
    // Reopened on Profiles, not back on General.
    term.wait_for("Settings reopened on Profiles", STEP, |s| {
        s.contains("Git & encoding") && s.contains("Profiles - global")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
