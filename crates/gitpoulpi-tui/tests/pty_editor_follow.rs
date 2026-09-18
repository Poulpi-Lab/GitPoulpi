//! `editor.follow_selection`, end to end through a pty: with the editor
//! open, selecting another file in the right pane opens it in the editor
//! (asking Save / Discard / Cancel first when there are unsaved changes);
//! with the box unchecked the old behavior stays; a file that cannot be
//! edited falls back to its normal view and says why.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate
//! (rows are found from their own text), and every wait describes the whole
//! state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

/// Three changed files: two text ones and a binary one.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"one\n")
        .unwrap()
        .write_worktree("b.txt", b"uno\n")
        .unwrap()
        .write_worktree("c.bin", b"\0\x01\x02\x03")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .write_worktree("b.txt", b"uno\ndos\n")
        .unwrap()
        .write_worktree("c.bin", b"\0\x01\x02\x03\x04")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// Clicks `file` in the right pane's list.
fn click_file(term: &mut Terminal, file: &str) {
    click_file_at(term, file, 0);
}

/// Same, `dx` cells into the name: two clicks on one cell within the double-click
/// window would be a double click.
fn click_file_at(term: &mut Terminal, file: &str, dx: u16) {
    let (c, r) = term
        .wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + dx, r).unwrap();
}

/// Opens `file` in the editor from its unstaged diff.
fn open_editor(term: &mut Terminal, file: &str) {
    click_file(term, file);
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains(&format!("{file} - unstaged")) && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains(&format!("Edit - {file} @ working tree"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn editing(file: &'static str) -> impl Fn(&xtask::pty::Screen) -> bool {
    move |s| s.contains(&format!("Edit - {file} @ working tree"))
}

#[test]
fn a_click_on_another_file_opens_it_in_the_editor_and_esc_lands_on_its_view() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-follow-click", "");
    open_editor(&mut term, "a.txt");
    assert!(
        term.screen().contains("[x] Follow selection"),
        "{}",
        term.text()
    );

    // b.txt is only in the right pane while a.txt is edited.
    click_file(&mut term, "b.txt");
    term.wait_for("b.txt in the editor, a.txt gone", STEP, |s| {
        editing("b.txt")(s) && !s.contains("Edit - a.txt") && s.contains("dos")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc returns to the new file's own diff, not the old one.
    term.send_keys("Escape").unwrap();
    term.wait_for("b.txt's diff", STEP, |s| {
        s.contains("b.txt - unstaged")
            && !s.contains("Edit - b.txt")
            && !s.contains("a.txt - unstaged")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn unchecked_the_selection_leaves_the_editor_as_before() {
    let repo = fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-follow-off",
        "[editor]\nfollow_selection = false\n",
    );
    open_editor(&mut term, "a.txt");
    assert!(
        term.screen().contains("[ ] Follow selection"),
        "{}",
        term.text()
    );
    click_file(&mut term, "b.txt");
    term.wait_for("the diff of b.txt, no editor", STEP, |s| {
        s.contains("b.txt - unstaged") && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(config);
    term.stop().unwrap();
}

#[test]
fn the_checkbox_toggles_live_and_is_saved_to_the_config() {
    let repo = fixture();
    let (mut term, config) = launch(repo.root(), "pty-follow-toggle", "");
    open_editor(&mut term, "a.txt");
    let stored = || std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();

    let (c, r) = term.screen().find("[x] Follow selection").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the box unchecked and saved", STEP, |s| {
        s.contains("[ ] Follow selection") && stored().contains("follow_selection = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", term.text(), stored()));

    // The palette command flips it back.
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Follow Selection").unwrap();
    term.wait_for("the palette hit", STEP, |s| {
        s.contains("Editor: Toggle Follow Selection")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the box checked and saved", STEP, |s| {
        s.contains("[x] Follow selection") && stored().contains("follow_selection = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", term.text(), stored()));
    term.stop().unwrap();
}

#[test]
fn unsaved_changes_ask_first_cancel_stays_discard_and_save_switch() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-follow-dirty", "");
    open_editor(&mut term, "a.txt");
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Cancel: stay on a.txt, edits intact, and Esc later lands on a.txt.
    click_file(&mut term, "b.txt");
    term.wait_for("the prompt", STEP, |s| {
        s.contains("Save changes to a.txt?") && s.contains("Discard")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("Cancel keeps a.txt with its edits", STEP, |s| {
        editing("a.txt")(s) && s.contains("zzone") && !s.contains("Save changes to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Discard: b.txt opens, a.txt is untouched on disk.
    click_file_at(&mut term, "b.txt", 3);
    term.wait_for("the prompt again", STEP, |s| {
        s.contains("Save changes to a.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("b.txt in the editor", STEP, |s| {
        editing("b.txt")(s) && !s.contains("Save changes to") && !s.contains("zzone")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"one\ntwo\n"
    );

    // Save: the edit of b.txt is written, then a.txt opens.
    term.send_text("yy").unwrap();
    term.wait_for("b.txt dirty", STEP, |s| s.contains("yyuno"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file(&mut term, "a.txt");
    term.wait_for("the prompt for b.txt", STEP, |s| {
        s.contains("Save changes to b.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("s").unwrap();
    term.wait_for("a.txt in the editor", STEP, |s| {
        editing("a.txt")(s) && !s.contains("Save changes to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        b"yyuno\ndos\n"
    );
    term.stop().unwrap();
}

#[test]
fn a_file_that_cannot_be_edited_opens_its_view_and_says_why() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-follow-binary", "");
    open_editor(&mut term, "a.txt");
    click_file(&mut term, "c.bin");
    term.wait_for("c.bin's view and the reason", STEP, |s| {
        s.contains("c.bin is a binary file - the editor only opens text") && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.screen().contains("Edit - a.txt"));
    term.stop().unwrap();
}

/// The editor keeps every plain key while open, so the keyboard way to
/// follow is the palette: with the box off the editor stays on its file
/// (dirty) while the pane moved on; turning the box on from the palette makes
/// the two agree - through the same Save / Discard / Cancel prompt.
#[test]
fn turning_follow_on_from_the_palette_catches_up_with_the_selection() {
    let repo = fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-follow-palette",
        "[editor]\nfollow_selection = false\n",
    );
    open_editor(&mut term, "a.txt");
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}"));
    click_file(&mut term, "b.txt");
    term.wait_for("the editor still on a.txt, the pane on b.txt", STEP, |s| {
        editing("a.txt")(s) && s.contains("[ ] Follow selection") && s.contains("zzone")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Follow Selection").unwrap();
    term.wait_for("the palette hit", STEP, |s| {
        s.contains("Editor: Toggle Follow Selection")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the prompt for a.txt", STEP, |s| {
        s.contains("Save changes to a.txt?") && s.contains("[x] Follow selection")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("b.txt in the editor", STEP, |s| {
        editing("b.txt")(s) && !s.contains("Save changes to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Cancel puts the pane back on the edited file: closing the editor lands
/// on that file's diff, not the one that was clicked.
#[test]
fn cancel_leaves_the_pane_and_the_view_on_the_edited_file() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-follow-cancel", "");
    open_editor(&mut term, "a.txt");
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}"));
    click_file(&mut term, "b.txt");
    term.wait_for("the prompt", STEP, |s| s.contains("Save changes to a.txt?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back on a.txt", STEP, |s| {
        editing("a.txt")(s) && !s.contains("Save changes to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the prompt to close", STEP, |s| {
        s.contains("Save changes to a.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("a.txt's diff, not b.txt's", STEP, |s| {
        s.contains("a.txt - unstaged") && !s.contains("b.txt - unstaged") && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
