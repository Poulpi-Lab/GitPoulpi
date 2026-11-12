//! mintty reads `OSC 22` only in its own Windows cursor names (`hand`, not
//! the CSS `pointer`; mintty 3.8.3 measured with `GetCursorInfo`, and its
//! `set_cursor_style`), so under `TERM_PROGRAM=mintty` the pointing hand over a
//! tab is `hand`, and everywhere else it stays `pointer`.
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

fn shape_over_the_repository_tab(program: &str, session: &str) -> Option<String> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pointer-mintty-commit")
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
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .env("TERM_PROGRAM", program)
        .env("TMUX", "")
        .env("ZELLIJ", "")
        .env("STY", "")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .spawn();
    term.wait_for("the window", STARTUP, |s| {
        s.contains("pointer-mintty-commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the Radar tab", STEP, |s| s.line(1).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bar = term.screen().line(1);
    let at = bar
        .find(&name)
        .unwrap_or_else(|| panic!("no {name:?} on the tab bar: {bar:?}"));
    let col = bar[..at].chars().count() as u16 + 1;
    term.move_to(col, 1).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(&term).is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let shape = pointer_shapes(&term).last().cloned();
    term.stop().unwrap();
    shape
}

#[test]
fn mintty_gets_its_own_cursor_name_and_other_terminals_the_css_one() {
    assert_eq!(
        shape_over_the_repository_tab("mintty", "pty-pointer-mintty").as_deref(),
        Some("hand")
    );
    assert_eq!(
        shape_over_the_repository_tab("kitty", "pty-pointer-notmintty").as_deref(),
        Some("pointer")
    );
}
