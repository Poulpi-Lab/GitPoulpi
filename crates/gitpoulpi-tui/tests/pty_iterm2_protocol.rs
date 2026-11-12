//! Which picture protocol the binary really speaks to an iTerm2 window,
//! read back from the bytes it writes to its pty. iTerm2 3.7.7 opens a modal
//! "Allow Terminal-Initiated Display?" dialog for every `OSC 1337` image
//! (real window, November 2026) and draws the Kitty protocol, Unicode
//! placeholders included, without one: from that version on `auto` sends the
//! Kitty pictures; older or unreadable versions keep the `OSC 1337` picture.
//! Same rules as `pty_ui.rs`; the terminal is named by its environment only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(60);
const MESSAGE: &str = "iterm2 protocol probe commit";

/// The bytes the binary wrote while the Log came up in a terminal that
/// names itself iTerm2 `version` (nothing set: a plain `TERM_PROGRAM`).
fn raw_in_iterm2(session: &str, version: Option<&str>, ui: &str) -> Vec<u8> {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(format!("[ui]\n{ui}"))
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .env("TERM", "xterm-256color")
        .env("TERM_PROGRAM", "iTerm.app")
        .env("LC_TERMINAL", "iTerm2")
        .env("HERDR_ENV", "")
        .env("HERDR_PANE_ID", "")
        .env("HERDR_SOCKET_PATH", "")
        .env("TMUX", "")
        .env("ZELLIJ", "")
        .env("STY", "")
        .env("WEZTERM_EXECUTABLE", "")
        .env("KITTY_WINDOW_ID", "")
        .env("GHOSTTY_RESOURCES_DIR", "")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE));
    launch = launch.env("TERM_PROGRAM_VERSION", version.unwrap_or(""));
    let (term, _config) = launch.start();
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

const KITTY: &[u8] = b"_Ga=T,";
const ITERM2: &[u8] = b"]1337;File=";

/// Falsified: dropped `iterm2_draws_kitty` from `speaks_kitty_natively`
/// (the Log went back to `OSC 1337`, the dialog's trigger, and this went
/// red); restored.
#[test]
fn iterm2_3_7_7_gets_kitty_pictures_and_no_osc_1337() {
    let raw = raw_in_iterm2("pty-iterm2-kitty", Some("3.7.7"), "avatars = \"graph\"\n");
    assert!(
        count(&raw, KITTY) > 0,
        "no Kitty picture was sent to iTerm2 3.7.7"
    );
    assert_eq!(
        count(&raw, ITERM2),
        0,
        "an OSC 1337 image (the dialog's trigger) was sent to iTerm2 3.7.7"
    );
}

#[test]
fn older_iterm2_keeps_the_osc_1337_picture() {
    for (version, session) in [
        (Some("3.7.6"), "pty-iterm2-old"),
        (None, "pty-iterm2-unversioned"),
    ] {
        let raw = raw_in_iterm2(session, version, "avatars = \"graph\"\n");
        assert!(
            count(&raw, ITERM2) > 0,
            "{version:?}: no OSC 1337 picture was sent"
        );
        assert_eq!(count(&raw, KITTY), 0, "{version:?}: Kitty was sent");
    }
}

#[test]
fn the_iterm2_protocol_can_still_be_asked_for() {
    let raw = raw_in_iterm2(
        "pty-iterm2-forced",
        Some("3.7.7"),
        "image_protocol = \"iterm2\"\navatars = \"graph\"\n",
    );
    assert!(count(&raw, ITERM2) > 0, "no OSC 1337 picture was sent");
    assert_eq!(count(&raw, KITTY), 0, "Kitty was sent");
}
