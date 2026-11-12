//! The merged start-up probe decides the picture protocol for a terminal the
//! environment cannot name. The pty driver plays the terminal: it answers the
//! device attributes query with sixel (`4`), names itself through XTVERSION,
//! or says nothing at all. Same rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "startup probe fixture";

/// A launch that no environment variable can name: `TERM=xterm-256color`
/// and every terminal or multiplexer marker blanked.
fn run(session: &str, attributes: Option<&str>, version: Option<&str>, mute: bool) -> Vec<u8> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first of the probe graph")
        .unwrap()
        .commit("second of the probe graph")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"auto\"\navatars = \"off\"\n",
    )
    .unwrap();
    let mut launch = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .env("TMUX", "")
        .env("ZELLIJ", "")
        .env("STY", "")
        .env("TERM_PROGRAM", "")
        .env("WEZTERM_EXECUTABLE", "")
        .env("KITTY_WINDOW_ID", "")
        .env("GHOSTTY_RESOURCES_DIR", "")
        .env("WT_SESSION", "")
        .env("KONSOLE_VERSION", "");
    if let Some(params) = attributes {
        launch = launch.answer_attributes(params);
    }
    if let Some(name) = version {
        launch = launch.answer_version(name);
    }
    if mute {
        launch = launch.mute_attributes();
    }
    let term = launch.spawn();
    term.wait_for("the fixture loaded", STEP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(1500), STEP)
        .unwrap();
    let raw = term.raw_output();
    let _ = term.stop();
    raw
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

#[test]
#[cfg_attr(
    windows,
    ignore = "the start-up probe is Unix only; ConPTY drops the sixel DCS"
)]
fn a_terminal_that_reports_sixel_gets_the_sixel_graph() {
    let raw = run("pty-probe-sixel", Some("62;4;22"), None, false);
    assert!(
        count(&raw, b"\x1bP0;1;0q") > 0,
        "no sixel picture was written for a terminal that reported sixel"
    );
    assert_eq!(count(&raw, b"]1337;File="), 0);
    assert_eq!(count(&raw, b"_Ga="), 0);
}

#[test]
#[cfg_attr(windows, ignore = "the start-up probe is Unix only")]
fn a_terminal_that_names_itself_kitty_gets_the_kitty_graph() {
    // The XTVERSION name is what recognizes a terminal reached over SSH.
    let raw = run(
        "pty-probe-kitty",
        Some("62;c"),
        Some("kitty(0.35.2)"),
        false,
    );
    assert!(
        count(&raw, b"_Ga=T,U=1") > 0,
        "no kitty picture for a terminal that named itself kitty"
    );
    assert_eq!(count(&raw, b"\x1bP0;1;0q"), 0);
}

#[test]
fn a_terminal_without_sixel_in_its_attributes_stays_text() {
    let raw = run("pty-probe-plain", Some("1;2"), None, false);
    assert_eq!(count(&raw, b"\x1bP0;1;0q"), 0, "a sixel picture was sent");
    assert_eq!(count(&raw, b"]1337;File="), 0);
    assert_eq!(count(&raw, b"_Ga="), 0);
}

#[test]
#[cfg_attr(
    windows,
    ignore = "a console host does not start the child until the attributes are answered"
)]
fn a_terminal_that_answers_nothing_stays_text_and_the_start_is_bounded() {
    // The probe waits its budget (~100 ms) and gives up: the window still
    // opens and draws within a generous bound, with the text graph.
    let started = std::time::Instant::now();
    let raw = run("pty-probe-silent", None, None, true);
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "start-up took {:?}",
        started.elapsed()
    );
    assert_eq!(count(&raw, b"\x1bP0;1;0q"), 0, "a sixel picture was sent");
    assert_eq!(count(&raw, b"_Ga="), 0);
}
