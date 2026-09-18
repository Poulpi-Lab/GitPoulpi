//! The Graph column's picture must not outlive the overlay that covers
//! it: with Settings open over the Log, no image escape may keep being
//! written and, on Kitty, a delete must reach the terminal; closing the
//! overlay brings the picture back. Both protocols, driven through the
//! real binary with a pinned cell size.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "graph overlay probe commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

#[derive(Clone, Copy)]
enum Wire {
    Iterm2,
    /// iTerm2 escape with the transparent picture (WezTerm).
    Wezterm,
    Kitty,
}

impl Wire {
    /// A byte pattern that only an image upload or placement contains.
    fn image_marker(self) -> &'static [u8] {
        match self {
            Wire::Iterm2 | Wire::Wezterm => b"]1337;File=",
            Wire::Kitty => b"_Ga=T,",
        }
    }
}

fn launch(repo: &FixtureRepo, wire: Wire, session: &str) -> (Terminal, tempfile::TempDir) {
    let protocol = match wire {
        Wire::Iterm2 | Wire::Wezterm => "iterm2",
        Wire::Kitty => "kitty",
    };
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(format!(
            "[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"{protocol}\"\n"
        ))
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE));
    if let Wire::Wezterm = wire {
        launch = launch
            .env("TERM_PROGRAM", "WezTerm")
            .env("TERM", "xterm-256color")
            .env("HERDR_ENV", "")
            .env("HERDR_PANE_ID", "")
            .env("HERDR_SOCKET_PATH", "");
    }
    launch.start()
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

fn open_settings(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn run(wire: Wire, session: &str) {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, wire, session);
    let quiet = Duration::from_millis(800);
    term.wait_for_stable(quiet, STEP).unwrap();
    let marker = wire.image_marker();
    assert!(
        count(&term.raw_output(), marker) > 0,
        "the Log never sent a picture ({session})"
    );

    let before_open = term.raw_output().len();
    open_settings(&mut term);
    term.wait_for_stable(quiet, STEP).unwrap();
    let settled = term.raw_output();
    let on_open = &settled[before_open..];
    // The overlay is up: nothing image-shaped may be (re)sent for it.
    assert_eq!(
        count(on_open, marker),
        0,
        "{session}: an image escape was written while Settings covers the Graph"
    );
    std::thread::sleep(Duration::from_millis(1200));
    let steady = term.raw_output();
    assert_eq!(
        count(&steady[settled.len()..], marker),
        0,
        "{session}: images kept being written under Settings"
    );

    term.send_keys("Escape").unwrap();
    term.wait_for("back on the Log", STEP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(quiet, STEP).unwrap();
    let closed = term.raw_output();
    let again = &closed[steady.len()..];
    // Kitty's picture is placeholder text in the grid and needs no new
    // upload; the iTerm2 escapes must be written again.
    if !matches!(wire, Wire::Kitty) {
        assert!(
            count(again, marker) > 0,
            "{session}: the picture was not redrawn after Settings closed"
        );
    }
    let _ = term.stop();
}

#[test]
fn iterm2_picture_is_gone_while_settings_covers_the_graph() {
    run(Wire::Iterm2, "overlay-iterm2");
}

#[test]
fn wezterm_picture_is_gone_while_settings_covers_the_graph() {
    run(Wire::Wezterm, "overlay-wezterm");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY drops the kitty graphics APC from the stream the driver reads"
)]
fn kitty_picture_is_not_resent_while_settings_covers_the_graph() {
    run(Wire::Kitty, "overlay-kitty");
}
