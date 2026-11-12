//! Real buttons and the text edit history, end to end through a pty: the
//! prompt editor's snippet buttons are hands that insert their snippet,
//! `Ctrl+Z` / `Ctrl+Y` walk the text of a field (the prompt editor, a
//! single-line field) and its Undo / Redo buttons do the same, while
//! `Ctrl+Z` outside a field is still the repository undo.
//!
//! Real binary through a pty; temp config dirs and fixture repositories.
//! Words typed here avoid hex-looking strings: the title bar shows a commit
//! hash.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

#[cfg(feature = "ai")]
const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
#[cfg(feature = "ai")]
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

#[cfg(feature = "ai")]
fn settled(term: &Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Settings > AI > Global "Custom instructions" editor, open.
#[cfg(feature = "ai")]
fn open_editor(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), "[ai]\nenabled = true\n").unwrap();
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    term.wait_for("the AI page", STEP, |s| s.contains("Custom instructions"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("Custom instructions", STEP).unwrap();
    term.click(c + 1, r).expect("select the row");
    settled(&term);
    term.send_keys("Enter").expect("open the editor");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, repo)
}

/// A footer button of the editor (the row holding Save and Cancel).
#[cfg(feature = "ai")]
fn footer_button(s: &xtask::pty::Screen, label: &str) -> (u16, u16) {
    s.find_all(label)
        .into_iter()
        .find(|&(_, r)| s.line(r).contains("Save") && s.line(r).contains("Cancel"))
        .unwrap_or_else(|| panic!("no {label} button. Screen:\n{}", s.text()))
}

/// The snippet button is a real button: the pointing hand (never the text
/// cursor of the area below it), no bracket, and a click inserts its snippet.
#[cfg(feature = "ai")]
#[test]
fn the_snippet_button_shows_the_hand_and_a_click_inserts_its_snippet() {
    let (mut term, _config, _repo) = open_editor("pty-undo-snippet-hand");
    let screen = settled(&term);
    assert!(!screen.contains("[+ "), "{}", screen.text());
    let (c, r) = screen
        .find_all("+ Keep responses concise")
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no snippet button:\n{}", screen.text()));
    term.wait_for_mouse_capture(STEP).unwrap();
    term.move_to(c + 2, r).unwrap();
    let deadline = Instant::now() + STEP;
    while pointer_shapes(&term).last().map(String::as_str) != Some("pointer")
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(&term).last().map(String::as_str),
        Some("pointer"),
        "over the snippet button: {:?}\n{}",
        pointer_shapes(&term),
        term.text()
    );
    term.click(c + 2, r).unwrap();
    term.wait_for("the snippet inserted", STEP, |s| {
        s.contains("Keep responses concise.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// Ctrl+Z undoes the last word, Ctrl+Y brings it back, and the Undo / Redo
/// buttons do the same with the mouse.
#[cfg(feature = "ai")]
#[test]
fn ctrl_z_and_ctrl_y_and_the_buttons_walk_the_prompt_text() {
    let (mut term, _config, _repo) = open_editor("pty-undo-prompt");
    term.send_text("gold fish").unwrap();
    term.wait_for("the text typed", STEP, |s| s.contains("gold fish"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-z").unwrap();
    term.wait_for("the last word undone", STEP, |s| {
        s.contains("gold") && !s.contains("fish")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-y").unwrap();
    term.wait_for("the word redone", STEP, |s| s.contains("gold fish"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The Undo button, then the Redo button.
    let (c, r) = footer_button(&settled(&term), "Undo");
    term.click(c + 1, r).unwrap();
    term.wait_for("undone by the button", STEP, |s| {
        s.contains("gold") && !s.contains("fish")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = footer_button(&settled(&term), "Redo");
    term.click(c + 1, r).unwrap();
    term.wait_for("redone by the button", STEP, |s| s.contains("gold fish"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

fn dirty_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("b.txt", "two\n", "add b")
        .unwrap()
        .write_worktree("a.txt", b"one changed\n")
        .unwrap()
        .write_worktree("b.txt", b"two changed\n")
        .unwrap()
        .build()
}

fn launch(repo: &std::path::Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("add b"))
        .start()
}

/// In a single-line field (the WIP row's message), Ctrl+Z / Ctrl+Y act on
/// the text and never on the repository.
#[test]
fn ctrl_z_and_ctrl_y_edit_a_single_line_field_not_the_repository() {
    let repo = dirty_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-undo-single-line");
    term.wait_for("the WIP row", STEP, |s| s.contains("// WIP"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").unwrap();
    term.wait_for("the WIP field focused", STEP, |s| {
        s.contains("Unstaged Files") && !s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("cold soup").unwrap();
    term.wait_for("the text in both fields", STEP, |s| {
        s.text().matches("cold soup").count() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-z").unwrap();
    // The field keeps the keyboard (no placeholder while it is focused) and
    // the repository's Unstaged Files are still listed: no git undo ran.
    term.wait_for("the text undone in both fields", STEP, |s| {
        !s.contains("cold soup") && s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-y").unwrap();
    term.wait_for("the text redone", STEP, |s| {
        s.text().matches("cold soup").count() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The repository was never touched by either key.
    assert_eq!(
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap(),
        "two changed\n"
    );
    term.stop().unwrap();
}

/// Selects the WIP row and right-clicks `name`'s row in the Unstaged half.
fn open_menu_on(term: &mut Terminal, name: &str) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    let needle = format!("\u{270E} {name}");
    term.wait_for_text(&needle, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    common::right_click_text_until(term, &needle, "the file menu", |s| s.contains("Stash file"));
}

fn click_row(term: &mut Terminal, label: &str) {
    let screen = term
        .wait_for(&format!("a menu row '{label}'"), STEP, |s| {
            s.contains(label)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find(label).unwrap();
    term.click(col + 1, row).unwrap();
}

/// With no field focused, Ctrl+Z is still the repository undo: it brings a
/// deleted file back.
#[test]
fn ctrl_z_outside_a_field_is_still_the_repository_undo() {
    let repo = dirty_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-undo-outside-field");
    open_menu_on(&mut term, "b.txt");
    click_row(&mut term, "Delete file...");
    term.wait_for("the confirm modal", STEP, |s| {
        s.contains("Delete b.txt from disk?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    let deadline = Instant::now() + STEP;
    while repo.root().join("b.txt").exists() {
        assert!(Instant::now() < deadline, "b.txt was never deleted");
        std::thread::sleep(Duration::from_millis(100));
    }
    // Let the journal reach the window before the key.
    std::thread::sleep(Duration::from_millis(800));
    term.send_keys("C-z").unwrap();
    let deadline = Instant::now() + STEP;
    while !repo.root().join("b.txt").exists() {
        assert!(Instant::now() < deadline, "Ctrl+Z never restored b.txt");
        std::thread::sleep(Duration::from_millis(100));
    }
    term.stop().unwrap();
}
