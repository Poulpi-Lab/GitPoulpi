//! The pty driver's own self-tests, against a child that is not the TUI.
//!
//! If these fail, nothing downstream means anything: a failure in
//! `crates/gitpoulpi-tui/tests/pty_ui.rs` could be a UI bug or a driver
//! bug, and these tell the two apart.
//!
//! **They live here, as an integration test, rather than in
//! `src/pty/mod.rs`, so that they run on Windows**: the child is
//! `xtask emit` (see `xtask/src/emit.rs`), reachable through
//! `CARGO_BIN_EXE_xtask`, an env var cargo sets for integration tests but
//! *not* for unit tests inside the library.

use std::time::Duration;

use xtask::emit::encode_hex;
use xtask::pty::{Color, Terminal};

const PATIENCE: Duration = Duration::from_secs(20);

/// Wide enough that none of the payloads below wrap - a wrapped line
/// would move the cells these tests address by hand.
const WIDE_ENOUGH: u16 = 40;

/// Spawns `xtask emit` with `payload` as its exact output.
fn emitting(payload: &[u8], cols: u16, rows: u16) -> Terminal {
    Terminal::builder(env!("CARGO_BIN_EXE_xtask"))
        .args(["emit", &encode_hex(payload)])
        .size(cols, rows)
        .spawn()
        .expect("spawn the emit child in a pty")
}

/// The driver's most basic claim: bytes the child writes end up in the
/// grid, at the cell they were written to.
#[test]
fn a_child_writing_to_the_pty_lands_in_the_grid() {
    let term = emitting(b"hello\r\nworld", 20, 4);
    term.wait_for_text("world", PATIENCE).unwrap();
    let screen = term.screen();
    assert_eq!(screen.line(0), "hello");
    assert_eq!(screen.find("world"), Some((0, 1)));
    term.stop().unwrap();
}

/// Styling has to survive the round trip: asserting on both characters and
/// colors is the capability `tmux capture-pane -e` had.
///
/// Deliberately does *not* pin the palette index, which a terminal may
/// re-encode (ConPTY re-renders what it is given). The exact-value claim
/// the TUI depends on is truecolor, below.
#[test]
fn styling_survives_the_round_trip() {
    let term = emitting(b"\x1b[1;31mRED\x1b[0m plain", WIDE_ENOUGH, 2);
    term.wait_for_text("plain", PATIENCE).unwrap();
    let screen = term.screen();
    let styled = screen.cell(0, 0).unwrap();
    assert_eq!(styled.display(), "R");
    assert!(styled.bold, "bold should reach the grid");
    assert_ne!(
        styled.fg,
        Color::Default,
        "a colored cell should not read as default"
    );

    // And the reset has to take: without this the test would pass on a
    // driver that simply reported every cell as styled.
    let (plain_col, plain_row) = screen.find("plain").unwrap();
    let plain = screen.cell(plain_col, plain_row).unwrap();
    assert!(!plain.bold, "the reset should have cleared bold");
    assert_eq!(plain.fg, Color::Default, "and the color");
    term.stop().unwrap();
}

/// Truecolor, exactly, because the TUI's own `ACCENT` is an `Rgb` and
/// the divider-highlight assertion in
/// `crates/gitpoulpi-tui/tests/pty_ui.rs` compares against its literal
/// value. If this fails on a platform, that one will too, and this says
/// why in one line instead of dumping a 200x50 screen.
#[test]
fn truecolor_backgrounds_arrive_exactly() {
    let term = emitting(b"\x1b[48;2;64;156;255mX\x1b[0m", WIDE_ENOUGH, 2);
    term.wait_for_text("X", PATIENCE).unwrap();
    let screen = term.screen();
    assert_eq!(
        screen.cell(0, 0).map(|c| c.bg),
        Some(Color::Rgb(64, 156, 255))
    );
    term.stop().unwrap();
}

/// The timeout message has to carry the screen, or every failure in CI is
/// the word "timed out" with nothing to look at. This is the assertion
/// that made the two CI failures diagnosable at all.
#[test]
fn a_timeout_reports_what_was_actually_on_screen() {
    let term = emitting(b"only this", 20, 2);
    term.wait_for_text("only", PATIENCE).unwrap();
    let err = term
        .wait_for_text("nothing like this", Duration::from_millis(50))
        .unwrap_err();
    let message = err.to_string();
    assert!(message.contains("only this"), "{message}");
    term.stop().unwrap();
}

/// A predicate that never holds must not hang forever, and one that holds
/// immediately must not wait.
#[test]
fn wait_until_returns_as_soon_as_the_predicate_holds() {
    let term = emitting(b"ready", 20, 2);
    term.wait_for_text("ready", PATIENCE).unwrap();
    let start = std::time::Instant::now();
    term.wait_until(PATIENCE, |s| s.contains("ready")).unwrap();
    assert!(
        start.elapsed() < Duration::from_millis(500),
        "an already-true predicate should return at once, took {:?}",
        start.elapsed()
    );
    term.stop().unwrap();
}

/// Killing the child has to be noticed, so a test can tell "the program
/// died" from "the program is slow" - on Unix that is a signal, on
/// Windows `TerminateProcess`, and `portable-pty` is what makes that the
/// same call here.
#[test]
fn the_driver_notices_when_the_child_goes_away() {
    let mut term = emitting(b"alive", 20, 2);
    term.wait_for_text("alive", PATIENCE).unwrap();
    assert!(!term.exited());
    term.kill_child();
    let deadline = std::time::Instant::now() + PATIENCE;
    while !term.exited() {
        assert!(
            std::time::Instant::now() < deadline,
            "the reader should have seen EOF once the child was killed"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A child that dies must be reported **as** a dead child, at once.
///
/// Otherwise a program that failed to start looks exactly like a slow
/// one: the wait sits out its whole timeout and then prints a blank
/// screen.
#[test]
fn a_child_that_exits_fails_the_wait_immediately_and_names_its_status() {
    let started = std::time::Instant::now();
    let term = Terminal::builder(env!("CARGO_BIN_EXE_xtask"))
        .args(["emit", &encode_hex(b"bye"), "--hold", "0", "--exit", "3"])
        .size(WIDE_ENOUGH, 2)
        .spawn()
        .expect("spawn");

    let err = term
        .wait_for("something that never comes", PATIENCE, |s| {
            s.contains("never going to appear")
        })
        .unwrap_err();

    let message = err.to_string();
    assert!(
        message.contains("exited"),
        "should say the child exited: {message}"
    );
    assert!(
        message.contains('3'),
        "should name the exit status: {message}"
    );
    // The point of the whole exercise: it does not wait out the timeout.
    assert!(
        started.elapsed() < PATIENCE / 2,
        "should fail as soon as the child dies, took {:?}",
        started.elapsed()
    );
}

/// A child that is alive but has drawn nothing gets its own message,
/// because that is a different diagnosis from "drew the wrong thing": it
/// points at the program or the way it was launched, not at the
/// assertion. This is precisely the state a Windows runner reported for
/// the TUI, and the message it should have been given.
#[test]
fn a_silent_child_is_reported_as_silent_rather_than_as_a_blank_screen() {
    let term = emitting(b"", WIDE_ENOUGH, 2);
    let err = term
        .wait_for("anything at all", Duration::from_millis(300), |s| {
            s.contains("anything")
        })
        .unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("written nothing at all"),
        "should distinguish silence from a wrong screen: {message}"
    );
    term.stop().unwrap();
}

/// The converse: a child that *did* draw, and simply drew something
/// else, is reported with its byte count. One number separates "this
/// driver never received anything" from "this driver received plenty and
/// the assertion is wrong" - the question a blank 200x50 dump could not
/// answer.
#[test]
fn a_child_that_drew_the_wrong_thing_reports_its_byte_count() {
    let payload = b"something else entirely";
    let term = emitting(payload, WIDE_ENOUGH, 2);
    term.wait_for_first_output(PATIENCE).unwrap();
    let err = term
        .wait_for(
            "the thing it did not draw",
            Duration::from_millis(300),
            |s| s.contains("not this"),
        )
        .unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains(&format!("has written {} bytes", payload.len())),
        "should report how much arrived: {message}"
    );
    assert!(
        message.contains("something else entirely"),
        "and what it was: {message}"
    );
    term.stop().unwrap();
}

/// `wait_for_first_output` is what a test should reach for before
/// anything else, so it has to be cheap and to succeed on any child that
/// writes.
#[test]
fn first_output_is_noticed() {
    let term = emitting(b"here", WIDE_ENOUGH, 2);
    term.wait_for_first_output(PATIENCE).unwrap();
    assert!(term.bytes_received() > 0);
    term.stop().unwrap();
}

/// A pty layer may write bytes of its own before the child runs: a
/// **ConPTY emits 27 bytes** of mode-setting and clearing at startup. Those
/// must not be attributed to the child, or `bytes_drawn` is never 0 on
/// Windows, `Error::NoOutput` cannot fire and `wait_for_first_output`
/// returns before the TUI has drawn.
///
/// A driver test, not a Windows test: on a Unix pty the preamble is empty
/// and this passes by arithmetic.
#[test]
fn a_pty_preamble_is_not_counted_as_the_childs_output() {
    let term = emitting(b"", WIDE_ENOUGH, 2);
    // Long enough that any preamble has certainly landed.
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        term.bytes_drawn(),
        0,
        "a child that wrote nothing has drawn nothing, whatever the pty \
         layer wrote for itself (bytes on the wire: {})",
        term.bytes_received()
    );
    term.stop().unwrap();
}

/// And the converse: once the child *does* write, it is charged for
/// exactly what it wrote, not for the preamble as well. Together with
/// the test above this pins both ends of the accounting.
#[test]
fn a_childs_own_bytes_are_counted_exactly() {
    let payload = b"exactly this much";
    let term = emitting(payload, WIDE_ENOUGH, 2);
    term.wait_for_text("exactly this much", PATIENCE).unwrap();
    assert_eq!(term.bytes_drawn(), payload.len() as u64);
    term.stop().unwrap();
}

/// `stop` must return, always. It kills the child, releases the pty and
/// joins the reader, in that order: on Windows the ConPTY holds the pipe's
/// write end until this process lets go of the master, so joining first
/// deadlocks. Six seconds is past `READER_GRACE`, so this fails rather
/// than hangs if the ordering regresses.
#[test]
fn stop_returns_promptly_even_for_a_child_that_never_wrote() {
    let term = emitting(b"", WIDE_ENOUGH, 2);
    let started = std::time::Instant::now();
    term.stop().unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(6),
        "stop() should not block on the reader, took {:?}",
        started.elapsed()
    );
}
