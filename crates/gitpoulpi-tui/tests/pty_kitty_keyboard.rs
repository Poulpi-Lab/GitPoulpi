//! The kitty keyboard protocol (`ui.kitty_keyboard`) and the key hints on
//! context-menu rows, end to end through a pty. Temp config dirs and
//! fixture repositories only.
//!
//! The pty is a plain xterm-like terminal unless a test asks for
//! `answer_kitty_keyboard()`, in which case it answers the `CSI ? u`
//! query like kitty, WezTerm or foot do.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";
const PUSH: &[u8] = b"\x1b[>1u";
const POP: &[u8] = b"\x1b[<1u";
const QUERY: &[u8] = b"\x1b[?u";
const LEAVE_ALT: &[u8] = b"\x1b[?1049l";

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .write_worktree("a.txt", b"one changed\n")
        .unwrap()
        .build()
}

fn launch(
    repo: &Path,
    session: &str,
    config_toml: &str,
    kitty_answers: bool,
) -> (Terminal, tempfile::TempDir) {
    let mut launch = common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture();
    if kitty_answers {
        launch = launch.kitty_answers();
    }
    launch.start()
}

fn index_of(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

fn wait_for_exit(term: &Terminal) {
    let deadline = Instant::now() + STEP;
    while !term.exited() {
        assert!(Instant::now() < deadline, "the TUI never exited");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// `ui.kitty_keyboard = "off"`: no enhancement flag is pushed or popped,
/// whatever the terminal answered. The merged start-up probe still asks its
/// one question (device attributes and the name decide pictures), once.
#[test]
fn off_never_pushes_the_enhancement_flags() {
    let repo = fixture();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kitty-off",
        "[ui]\nkitty_keyboard = \"off\"\n",
        true,
    );
    // A real keystroke goes through, so the run is past startup.
    term.send_text("?").unwrap();
    term.wait_for("the help overlay", STEP, |s| s.contains("Keybindings -"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Keybindings -", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("q").unwrap();
    wait_for_exit(&term);
    let raw = term.raw_output();
    assert_eq!(count(&raw, PUSH), 0, "an enhancement push was written");
    assert_eq!(count(&raw, POP), 0, "an enhancement pop was written");
    assert_eq!(count(&raw, QUERY), 1, "one merged start-up query, no more");
}

/// A terminal that answers only the device-attributes query (most of
/// them, and every multiplexer that swallows the kitty one): start-up is
/// unchanged and nothing is pushed.
#[test]
fn auto_on_a_terminal_without_the_protocol_keeps_the_legacy_keys() {
    let repo = fixture();
    let (mut term, _c) = launch(repo.root(), "pty-kitty-silent", "", false);
    assert_eq!(count(&term.raw_output(), PUSH), 0);
    // The legacy Ctrl+, byte does not exist, so the key-by-key route
    // still works: Ctrl+P opens the palette as always.
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Navigation: Focus"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Navigation: Focus", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("q").unwrap();
    wait_for_exit(&term);
    let raw = term.raw_output();
    assert_eq!(count(&raw, PUSH), 0);
    assert_eq!(count(&raw, POP), 0);
}

/// `auto` against a terminal that answers: the flags are pushed, a key
/// the legacy protocol cannot send (`Ctrl+,` as `CSI 44;5u`) works, and
/// the flags are popped on exit, before the alternate screen is left.
#[cfg_attr(
    windows,
    ignore = "the probe reads /dev/tty: it never queries on Windows, where the console reports keys itself"
)]
#[test]
fn auto_pushes_when_the_terminal_answers_and_pops_on_exit() {
    let repo = fixture();
    let (mut term, _c) = launch(repo.root(), "pty-kitty-auto", "", true);
    let raw = term.raw_output();
    assert_eq!(count(&raw, QUERY), 1, "queried exactly once");
    assert_eq!(count(&raw, PUSH), 1, "pushed exactly once");
    term.send_text("\x1b[44;5u").unwrap();
    term.wait_for("Settings opened by Ctrl+,", STEP, |s| {
        s.contains("Settings - Tab") || s.contains("Keyboard & mouse")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("back on the graph", STEP, |s| {
        s.contains("Unstaged") && !s.contains("Keyboard & mouse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("q").unwrap();
    wait_for_exit(&term);
    // The stream is read in small pieces under torn frames: wait (bounded)
    // for the whole exit sequence to have arrived.
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    let mut raw = term.raw_output();
    while (index_of(&raw, POP).is_none() || index_of(&raw, LEAVE_ALT).is_none())
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(250));
        raw = term.raw_output();
    }
    let pop = index_of(&raw, POP).expect("the flags were popped on exit");
    let leave = index_of(&raw, LEAVE_ALT).expect("the alternate screen was left");
    assert!(
        pop < leave,
        "pop ({pop}) must precede leaving the screen ({leave})"
    );
    assert_eq!(count(&raw, POP), 1);
}

/// A rebound action's new key is what its context-menu row shows.
#[test]
fn a_rebound_action_shows_its_new_key_in_the_context_menu() {
    let repo = fixture();
    let (mut term, _c) = launch(
        repo.root(),
        "pty-kitty-menu",
        "[keybindings.git_prefix]\nstage_file = \"z\"\n",
        false,
    );
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for_text("\u{270E} a.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = common::right_click_text_until(
        &mut term,
        "\u{270E} a.txt",
        "the file menu with the rebound key",
        |s| s.contains("Stash file"),
    );
    let text = screen.text();
    let line = text
        .lines()
        .find(|l| l.contains("Stage") && l.contains("Ctrl+G"))
        .unwrap_or_else(|| panic!("no Stage row with a key hint:\n{text}"));
    assert!(line.contains("Ctrl+G z"), "{line}");
    assert!(!line.contains("Ctrl+G s"), "{line}");
    term.send_keys("Esc").unwrap();
    term.stop().unwrap();
}
