//! Closing the Init dialog (the pattern every migrated repository dialog
//! shares): the cross closes it, a click outside closes a pristine one and is
//! swallowed when it holds typed text, Esc still closes. Driven through a pty
//! by `xtask::pty`: the real binary and daemon.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const CROSS: &str = "\u{2715}";
const TITLE: &str = "Initialize a repository";

/// The management tab with the Init dialog open.
fn open_init(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "close-test-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), "").expect("seed config.toml");
    let mut term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("close-test-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Init")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("i").expect("Init");
    term.wait_for("the Init dialog", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config, start)
}

fn init_is_gone(term: &Terminal) {
    term.wait_for_text_to_go(TITLE, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The cross on the Init dialog's own title row, with no "Esc" beside it.
fn init_cross(term: &Terminal) -> (u16, u16) {
    let text = term.text();
    let (row, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(TITLE))
        .expect("the title row");
    assert!(line.contains(CROSS), "a cross on the title row: {line:?}");
    assert!(!line.contains("Esc"), "no Esc beside the cross: {line:?}");
    let (c, r) = term.screen().find(CROSS).expect("the cross");
    assert_eq!(r as usize, row, "the first cross is the dialog's");
    (c, r)
}

#[test]
fn the_cross_closes_init() {
    let (mut term, _config, _repo) = open_init("pty-init-cross");
    let (c, r) = init_cross(&term);
    term.click(c + 1, r).unwrap();
    init_is_gone(&term);
}

#[test]
fn a_click_outside_closes_a_pristine_init() {
    let (mut term, _config, _repo) = open_init("pty-init-outside");
    term.click(1, 1).unwrap();
    init_is_gone(&term);
}

#[test]
fn a_click_outside_a_dirty_init_is_swallowed_and_esc_still_closes() {
    let (mut term, _config, _repo) = open_init("pty-init-dirty");
    term.send_text("keepme").unwrap();
    term.wait_for("the typed name", STEP, |s| s.contains("keepme"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(1, 1).unwrap();
    // The dialog took the next keys: it is still open and still has the focus.
    term.send_text("7").unwrap();
    let screen = term
        .wait_for("the dialog kept the text", STEP, |s| s.contains("keepme7"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains(TITLE));
    term.send_keys("Escape").unwrap();
    init_is_gone(&term);
}

#[test]
fn esc_closes_init() {
    let (mut term, _config, _repo) = open_init("pty-init-esc");
    term.send_keys("Escape").unwrap();
    init_is_gone(&term);
}
