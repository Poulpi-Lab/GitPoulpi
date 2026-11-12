//! `ui.text_cursor`: with `terminal` the terminal's own cursor is shown at
//! the caret of the focused text field (a bar, DECSCUSR) and hidden
//! elsewhere, with no caret glyph drawn; with `drawn` the cursor stays hidden
//! and GitPoulpi paints its own caret. The harness pins `drawn` for every
//! other test, so these two set it explicitly.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! temp config dir and fixture repository.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const BAR: &[u8] = b"\x1b[5 q";
const RESTORE: &[u8] = b"\x1b[0 q";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, mode: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .env("GITPOULPI_UI_TEXT_CURSOR", mode)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Opens the Log's find bar and types `abc`.
fn type_into_the_find_bar(term: &mut Terminal) {
    term.send_keys("f").unwrap();
    term.send_text("abc").unwrap();
    term.wait_for("the typed query", STEP, |s| s.contains("abc"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Polls until the cursor is where `want` says (frames land in pieces).
fn wait_cursor(term: &Terminal, what: &str, want: impl Fn((u16, u16, bool)) -> bool) {
    let deadline = Instant::now() + STEP;
    loop {
        let cursor = term.cursor();
        if want(cursor) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: the cursor is {cursor:?}\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn terminal_mode_shows_the_real_cursor_at_the_caret_as_a_bar_and_hides_it_elsewhere() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-textcursor-terminal", "terminal");
    wait_cursor(&term, "no field is focused: hidden", |(_, _, visible)| {
        !visible
    });
    assert!(
        contains(&term.raw_output(), BAR),
        "the bar style (DECSCUSR 5) was never sent"
    );
    type_into_the_find_bar(&mut term);
    let (col, row) = term.screen().find("abc").expect("the typed query");
    wait_cursor(&term, "a focused find bar: shown just after `abc`", |c| {
        c == (col + 3, row, true)
    });
    // Leaving the field hides it again.
    term.send_keys("Esc").unwrap();
    wait_cursor(&term, "the find bar closed: hidden", |(_, _, visible)| {
        !visible
    });
    // The user's own cursor style comes back on the way out.
    term.send_text("q").unwrap();
    let deadline = Instant::now() + STEP;
    while !term.exited() {
        assert!(Instant::now() < deadline, "the TUI never exited");
        std::thread::sleep(Duration::from_millis(50));
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    while !contains(&term.raw_output(), RESTORE) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(250));
    }
    assert!(
        contains(&term.raw_output(), RESTORE),
        "the cursor style was not restored (DECSCUSR 0)"
    );
}

#[test]
fn terminal_mode_draws_no_caret_glyph_of_its_own() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-textcursor-noglyph", "terminal");
    type_into_the_find_bar(&mut term);
    let (col, row) = term.screen().find("abc").expect("the typed query");
    let after = term
        .screen()
        .cell(col + 3, row)
        .map(|c| c.display().to_string());
    assert_ne!(
        after.as_deref(),
        Some("\u{2588}"),
        "a block caret was drawn"
    );
    term.stop().unwrap();
}

#[test]
fn drawn_mode_keeps_the_cursor_hidden_and_paints_its_own_caret() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-textcursor-drawn", "drawn");
    type_into_the_find_bar(&mut term);
    let (col, row) = term.screen().find("abc").expect("the typed query");
    term.wait_for("the drawn caret after the query", STEP, |s| {
        s.cell(col + 3, row)
            .is_some_and(|c| c.display() == "\u{2588}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, _, visible) = term.cursor();
    assert!(!visible, "the terminal cursor is shown in `drawn` mode");
    assert!(
        !contains(&term.raw_output(), BAR),
        "the bar style was sent in `drawn` mode"
    );
    term.stop().unwrap();
}
