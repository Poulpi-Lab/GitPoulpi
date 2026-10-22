#![cfg(feature = "ai")]
//! ✦ "Suggest resolution" in the merge conflict editor, end to end through
//! a pty (see the merge-conflict-editor design, slice S9):
//! the real binary and daemon, a real conflicted repository, and a **fake
//! local agent** (a script printing a canned answer) -- no socket is opened.
//!
//! The suggestion fills the current conflict's Output region as an edit
//! (one undo step, marked edited) and is never saved; with AI off nothing
//! on screen mentions it.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{
    agent_calls, ai_config_toml, write_fake_agent, write_slow_fake_agent,
};
use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn open_editor(
    fixture: &ConflictFixture,
    session: &str,
    config: &str,
) -> (Terminal, tempfile::TempDir) {
    let (mut term, dir) = common::launch(fixture.root())
        .session(session)
        .size(200, 50)
        .config(config)
        .no_exit_pause()
        .ready_when("the conflicted row", |s| s.contains("! notes.txt"))
        .mouse_capture()
        .start();
    let (c, r) = term
        .wait_for_text("! notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the merge editor", STEP, |s| s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, dir)
}

/// Every row under the Output's title rule.
fn output_text(s: &Screen) -> String {
    match (0..s.rows()).find(|r| s.line(*r).contains("Output \u{b7}")) {
        Some(title) => (title + 1..s.rows())
            .map(|r| s.line(r))
            .collect::<Vec<_>>()
            .join("\n"),
        None => String::new(),
    }
}

fn config_for(agent: &Path) -> String {
    ai_config_toml(agent)
}

fn still_conflicted(fixture: &ConflictFixture) -> bool {
    std::fs::read_to_string(fixture.root().join("notes.txt"))
        .unwrap()
        .contains("<<<<<<<")
}

/// Click the toolbar's button: the Output is filled, marked edited, and
/// nothing is saved; Ctrl+Z takes the suggestion back in one step.
#[test]
fn the_suggest_button_fills_the_output_as_one_undoable_edit_and_saves_nothing() {
    let fixture = ConflictFixture::text();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["```", "resolved by the agent", "```"]);
    let (mut term, _config) = open_editor(&fixture, "pty-merge-ai", &config_for(&agent));

    let (c, r) = term
        .wait_for_text("\u{2726} Suggest", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.wait_for("the suggestion in the Output, marked edited", STEP, |s| {
        output_text(s).contains("resolved by the agent")
            && output_text(s).contains('\u{270e}')
            && s.contains("edited")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 1);
    assert!(still_conflicted(&fixture), "nothing was saved");

    term.send_keys("C-z").unwrap();
    term.wait_for("the Output back as it was", STEP, |s| {
        !output_text(s).contains("resolved by the agent")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(still_conflicted(&fixture));
    term.stop().unwrap();
}

/// With AI off (the default) there is no button and no palette entry.
#[test]
fn with_ai_off_the_editor_shows_nothing_about_it() {
    let fixture = ConflictFixture::text();
    let (mut term, _config) = open_editor(&fixture, "pty-merge-ai-off", "");
    let screen = term.screen();
    assert!(!screen.contains("Suggest"), "{}", term.text());
    term.send_keys("C-p").unwrap();
    term.send_text("suggest merge").unwrap();
    let screen = term
        .wait_for("the query echoed", STEP, |s| s.contains("suggest merge"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Suggest resolution"), "{}", term.text());
    term.stop().unwrap();
}

/// The palette starts it, Esc cancels it while it runs, and the Output is
/// untouched.
#[test]
fn the_palette_starts_it_and_escape_cancels_it() {
    let fixture = ConflictFixture::text();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_slow_fake_agent(tools.path(), 20, &["```", "late", "```"]);
    let (mut term, _config) = open_editor(&fixture, "pty-merge-ai-cancel", &config_for(&agent));

    term.send_keys("C-p").unwrap();
    term.send_text("suggest merge").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Suggest Merge Conflict Resolution")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the progress line", STEP, |s| s.contains("Esc cancels"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the cancellation", STEP, |s| {
        s.contains("suggestion canceled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!output_text(&term.screen()).contains("late"));
    // The editor is still open on its conflict.
    assert!(term.text().contains("Conflict 1 / 1"));
    term.stop().unwrap();
}

/// An answer that is not a resolution is shown as an error and changes
/// nothing.
#[test]
fn a_junk_answer_is_an_inline_error_and_the_output_is_unchanged() {
    let fixture = ConflictFixture::text();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["Sorry, I cannot help with that."]);
    let (mut term, _config) = open_editor(&fixture, "pty-merge-ai-junk", &config_for(&agent));
    let (c, r) = term
        .wait_for_text("\u{2726} Suggest", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("the inline error", STEP, |s| {
        s.contains("could not suggest a resolution")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!output_text(&term.screen()).contains("Sorry"));
    assert!(still_conflicted(&fixture));
    term.stop().unwrap();
}
