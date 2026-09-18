//! The Base branches editor of Conflict prevention takes the mouse like the
//! app's text editor: a click places the caret, a drag selects, a double-click
//! selects a word, Shift+click extends - and the keyboard then edits the
//! selection. No hard-coded coordinates: positions are found on the screen.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::prediction_fixture::conflicting_feature;
use gitpoulpi_testkit::FixtureRepo;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const BADGE: &str = "Conflicts with origin/main in 1 file";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("feature work"))
        .mouse_capture()
        .start()
}

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_badge(term: &mut Terminal) {
    term.wait_for("the conflict badge", STEP, |s| s.contains(BADGE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn wait_for(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn state_file(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml")).unwrap_or_default()
}

fn open_page(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Conflict prevention");
}

fn text_pos(term: &Terminal, needle: &str) -> (u16, u16) {
    term.screen().find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` not on screen
{}",
            term.text()
        )
    })
}

#[test]
fn the_base_branches_editor_takes_click_drag_double_click_and_shift_click() {
    let repo = conflicting_feature();
    let (mut term, _config) = launch(repo.root(), "pty-conflict-field-mouse");
    wait_badge(&mut term);
    open_page(&mut term);
    click_text(&mut term, "Base branches");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-a").unwrap();
    term.send_text("alpha beta gamma").unwrap();
    term.wait_for("the typed text", STEP, |s| s.contains("alpha beta gamma"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click before `b` of `beta` places the caret there.
    let (c, r) = text_pos(&term, "beta");
    term.click(c, r).unwrap();
    term.send_text("X").unwrap();
    term.wait_for("X typed at the click", STEP, |s| {
        s.contains("alpha Xbeta gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A drag from `g` past the end selects `gamma`; typing replaces it.
    let (c, r) = text_pos(&term, "gamma");
    term.drag((c, r), (c + 12, r)).unwrap();
    term.send_text("Q").unwrap();
    term.wait_for("the drag selection replaced", STEP, |s| {
        s.contains("alpha Xbeta Q")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Shift+click before `X` extends the selection from the caret (the end).
    let (c, r) = text_pos(&term, "Xbeta");
    term.shift_click(c, r).unwrap();
    term.send_text("Z").unwrap();
    term.wait_for("the shift selection replaced", STEP, |s| {
        s.contains("alpha Z") && !s.contains("Xbeta")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A double-click selects the word.
    let (c, r) = text_pos(&term, "alpha");
    term.double_click(c + 1, r).unwrap();
    term.send_text("W").unwrap();
    term.wait_for("the word replaced", STEP, |s| s.contains("W Z"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("alpha"), "{}", term.text());

    term.send_keys("C-s").unwrap();
    wait_for(&term, "the patterns saved", || {
        state_file(&repo).contains("W Z")
    });
    let _ = term.stop();
}
