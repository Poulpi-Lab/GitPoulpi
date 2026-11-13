//! Edit in Working Directory, end to end through a pty.
//!
//! From a working-tree file's diff, the bar's `Edit` opens the editor in
//! the central column; typed text lands at the caret, `Ctrl+S` writes the
//! file to disk (CRLF line endings and the missing final newline kept), the
//! status refreshes so the diff under the editor shows the edit, and Esc
//! goes back to that diff. A commit's diff and a File View of a commit's
//! version also offer Edit; it always opens the working-directory copy of
//! the path, never the commit's blob, and is refused when that copy does
//! not exist.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded
//! coordinate, and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// Same launch, but waits for the commit-detail pane (the right panel
/// defaults to it regardless of a dirty tree).
fn launch_commit_detail(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

#[test]
fn edit_in_working_directory_types_saves_and_the_wip_diff_shows_the_change() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\r\nbeta\r\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        // A CRLF file with no final newline, changed in the working tree.
        .write_worktree("notes.txt", b"alpha\r\nbeta\r\ngamma")
        .unwrap()
        .build();
    let path = repo.root().join("notes.txt");
    let (mut term, _config) = launch(repo.root(), "pty-editor-save");

    // The unstaged diff of notes.txt, from the Changed files list.
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains("gamma") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The bar's Edit button opens the editor over the diff.
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor in the central column", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree")
            && s.contains("CRLF")
            && s.contains("alpha")
            && !s.contains("modified")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The caret starts on the first char: typing goes before "alpha".
    term.send_text("EDITED ").unwrap();
    term.wait_for("the typed text and the dirty marker", STEP, |s| {
        s.contains("EDITED alpha") && s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-s").unwrap();
    term.wait_for("the save", STEP, |s| {
        s.contains("saved notes.txt") && !s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"EDITED alpha\r\nbeta\r\ngamma",
        "CRLF kept, no final newline added"
    );

    // Esc goes back to the diff, which the refresh has brought up to
    // date with what was just written.
    term.send_keys("Escape").unwrap();
    term.wait_for("the WIP diff showing the saved edit", STEP, |s| {
        s.contains("notes.txt - unstaged")
            && s.contains("EDITED alpha")
            && !s.contains("Edit - notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "the TUI is still running");
    term.stop().unwrap();
}

/// The Editor has a `Ctrl+N` badge. Collapsing both side panes first leaves
/// it the only focusable target, so its badge is `1`.
#[test]
fn the_editor_gets_a_ctrl_n_badge() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("notes.txt", b"alpha\nbeta\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-editor-badge");

    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor in the central column", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The Refs pane auto-collapses at this width, so only the Status pane needs
    // collapsing by hand, with the mouse chevron: the Editor owns every plain
    // key while open.
    term.wait_for("the Refs pane already auto-collapsed", STEP, |s| {
        !s.contains("to filter)") && !s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (lc, lr) = term
        .screen()
        .find("\u{203A}")
        .unwrap_or_else(|| panic!("the Status pane's collapse chevron\n{}", term.text()));
    term.click(lc, lr).unwrap();
    // A chevron click sets no footer message; collapse is confirmed by the
    // Status pane's content leaving the screen.
    term.wait_for("the Status pane collapsed", STEP, |s| {
        !s.contains("Unstaged Files") && !s.contains("Staged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-n").expect("enter navigation mode");
    // The badge key resolves against the last painted frame, so the frame
    // with the badges (the footer's `<badge>=`) has to be on screen first;
    // `Ctrl+N` and `focus` are on the ordinary screen too.
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("1").expect("press the Editor's badge");
    term.wait_for(
        "the Editor reports it already owns the keyboard, proving `1` named it",
        STEP,
        |s| s.contains("the editor already has the keyboard"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.exited(), "the TUI is still running");
    term.stop().unwrap();
}

/// `Ctrl+N h` / `Ctrl+N l` (the pane collapse toggles) must work while the
/// editor owns the central column, end to end through the real event loop.
/// Uses `l`: opening the editor already auto-collapses the Refs pane.
#[test]
fn ctrl_n_l_collapses_the_status_pane_while_the_editor_has_focus() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("notes.txt", b"alpha\nbeta\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-editor-ctrl-n");

    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor in the central column", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.wait_for(
        "the Status pane's Unstaged Files section, before anything is collapsed",
        STEP,
        |s| s.contains("Unstaged Files"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n")
        .expect("enter navigation mode from the editor");
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("l")
        .expect("collapse the Status pane from the editor");
    term.wait_for(
        "the Status pane collapsed to its restore strip",
        STEP,
        |s| s.contains("Status pane collapsed") && !s.contains("Unstaged Files"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // The editor is unharmed: still on screen, and a key it owns still reaches
    // it.
    term.wait_for("the editor still showing the file", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree") && s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n l")
        .expect("restore the Status pane from the editor");
    term.wait_for(
        "the Status pane restored, Unstaged Files back on screen",
        STEP,
        |s| s.contains("Status pane restored") && s.contains("Unstaged Files"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // `h` too, in the harder case: the Refs pane is already auto-collapsed, so
    // `Ctrl+N h` must expand it (the user taking over), not fail silently.
    assert!(
        !term.screen().contains("LOCAL"),
        "the Refs pane should already be auto-collapsed while the editor is the center"
    );
    term.send_keys("C-n")
        .expect("enter navigation mode from the editor, again");
    term.wait_for("the navigation prefix's footer hint, again", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("h")
        .expect("expand the auto-collapsed Refs pane from the editor");
    term.wait_for("the Refs pane expanded, LOCAL back on screen", STEP, |s| {
        s.contains("Refs pane restored") && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    assert!(!term.exited(), "the TUI is still running");
    term.stop().unwrap();
}

/// With unsaved changes, Esc asks; Cancel keeps the editor, Discard
/// closes it and leaves the file on disk untouched.
#[test]
fn esc_with_unsaved_changes_asks_before_closing_and_discard_keeps_the_file() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"one\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-editor-discard");
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("a.txt - unstaged") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The `e` key is the keyboard way in.
    term.send_keys("e").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - a.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Escape").unwrap();
    term.wait_for("the prompt", STEP, |s| {
        s.contains("Save changes to a.txt?") && s.contains("Discard")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("Cancel keeps the editor", STEP, |s| {
        !s.contains("Save changes to") && s.contains("zzone")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Escape").unwrap();
    term.wait_for("the prompt again", STEP, |s| {
        s.contains("Save changes to a.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("d").unwrap();
    term.wait_for("Discard back to the diff", STEP, |s| {
        s.contains("a.txt - unstaged") && !s.contains("Edit - a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"one\ntwo\n"
    );
    term.stop().unwrap();
}

/// A commit's diff offers Edit, which opens the working-directory copy, not
/// the commit's blob. An uncommitted "gamma" is appended straight on disk
/// (outside the TUI) after the diff opens, proving Edit re-reads the working
/// copy at open time. The tree stays clean until then so the commit-detail
/// default is undisturbed.
#[test]
fn edit_from_a_commits_diff_opens_the_working_copy_not_the_commits_blob() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .commit_file("notes.txt", "alpha\nbeta\n", "add beta")
        .unwrap()
        .build();
    let (mut term, _config) = launch_commit_detail(repo.root(), "pty-editor-commit-diff");

    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("HEAD's own diff (the committed +beta only)", STEP, |s| {
        s.contains("notes.txt")
            && s.contains("beta")
            && s.contains(" Edit ")
            && !s.contains("gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Never shown by HEAD's diff and never committed, so the daemon's status
    // read is irrelevant to what Edit opens.
    std::fs::write(repo.root().join("notes.txt"), b"alpha\nbeta\ngamma\n").unwrap();

    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor on the working copy, gamma and all", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree") && s.contains("gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_text("EDITED ").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the save", STEP, |s| s.contains("saved notes.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(repo.root().join("notes.txt")).unwrap(),
        b"EDITED alpha\nbeta\ngamma\n"
    );

    term.send_keys("Escape").unwrap();
    term.wait_for("Esc back to the commit's own diff", STEP, |s| {
        !s.contains("Edit - notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// View all files on a commit, for a file it did not change, opens a File
/// View that also offers Edit: same working-copy target, same button.
#[test]
fn edit_from_view_all_files_on_an_unchanged_commit_file() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("a.txt", b"a\n")
        .unwrap()
        .write_worktree("b.txt", b"b\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .commit_file("a.txt", "a changed\n", "change a only")
        .unwrap()
        .build();
    let (mut term, _config) = launch_commit_detail(repo.root(), "pty-editor-vaf-unchanged");

    let (c, r) = term.wait_for_text("View all files", STEP).unwrap();
    term.click(c + 6, r).unwrap();
    let (fc, fr) = term.wait_for_text(" b.txt", STEP).unwrap();
    term.click(fc + 1, fr).unwrap();
    term.wait_for(
        "b.txt's content at the commit, with Edit offered",
        STEP,
        |s| s.contains("File View - b.txt @") && s.contains(" Edit "),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor on b.txt's working copy", STEP, |s| {
        s.contains("Edit - b.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("EDITED ").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the save", STEP, |s| s.contains("saved b.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read(repo.root().join("b.txt")).unwrap(),
        b"EDITED b\n"
    );
    term.stop().unwrap();
}

/// The bar's Edit stays live on a commit's diff when the path is gone from
/// the working tree, and the click is refused with the same message a
/// deleted WIP file produces (the button is not grayed).
#[test]
fn edit_is_refused_when_the_commits_path_is_missing_from_the_working_tree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"alpha\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .commit_file("notes.txt", "alpha\nbeta\n", "add beta")
        .unwrap()
        .build();
    std::fs::remove_file(repo.root().join("notes.txt")).unwrap();
    let (mut term, _config) = launch_commit_detail(repo.root(), "pty-editor-missing-refused");

    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the commit's diff, Edit still offered", STEP, |s| {
        s.contains("notes.txt") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the refusal, no editor opened", STEP, |s| {
        s.contains("not in the working tree") && !s.contains("Edit - notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
