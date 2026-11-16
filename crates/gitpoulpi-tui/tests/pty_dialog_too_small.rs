//! A dialog that does not fit the window draws a minimal frame (title, close
//! cross, "Window too small: enlarge it") instead of an invisible modal that
//! still eats keys. The cross and Esc close it. Driven through a pty by
//! `xtask::pty`: the real binary and daemon, resizing the terminal.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const CROSS: &str = "\u{2715}";
const TITLE: &str = "Initialize a repository";
const WHY: &str = "Window too small";

/// The management tab with the Init dialog open, then the window shrunk until
/// the dialog cannot carry its form.
fn open_init_then_shrink(
    session: &str,
) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "too-small-start-commit")
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
        s.contains("too-small-start-commit")
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
    term.resize(60, 9).expect("shrink the window");
    term.wait_for("the too-small frame", STEP, |s| s.contains(WHY))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config, start)
}

#[test]
fn a_dialog_too_big_for_the_window_says_so_and_the_cross_closes_it() {
    let (mut term, _config, _repo) = open_init_then_shrink("pty-too-small-cross");
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let text = term.text();
    let (row, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(TITLE) || l.contains("Initialize"))
        .unwrap_or_else(|| panic!("the title is still shown:\n{text}"));
    assert!(line.contains(CROSS), "a cross beside the title: {line:?}");
    let (c, r) = term.screen().find(CROSS).expect("the cross");
    assert_eq!(r as usize, row, "the cross sits on the title row");
    term.click(c + 1, r).expect("click the cross");
    term.wait_for_text_to_go(WHY, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn esc_closes_the_too_small_frame() {
    let (mut term, _config, _repo) = open_init_then_shrink("pty-too-small-esc");
    term.send_keys("Escape").expect("Esc");
    term.wait_for_text_to_go(WHY, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
