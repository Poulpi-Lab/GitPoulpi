//! The merge conflict editor, end to end through a pty (plan
//! the merge-conflict-editor design, slice S6): opening it
//! from the Status pane, picking by key and by mouse, typing in the Output,
//! saving and marking resolved (back to the conflicted-files list, as
//! as usual), the global Undo taking that back, moving between files
//! and conflicts, the narrow layout, the operation banner's Abort and the
//! stale guard.
//!
//! Real conflicted repositories from the testkit's `ConflictFixture`, a
//! throwaway config per test, and every wait describes the whole state it
//! waits for. Nothing is located by a hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str, cols: u16, rows: u16) -> (Terminal, tempfile::TempDir) {
    launch_with(repo, session, cols, rows, "")
}

/// [`launch`] with `config_toml` seeded into the throwaway config.
fn launch_with(
    repo: &Path,
    session: &str,
    cols: u16,
    rows: u16,
    config_toml: &str,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(cols, rows)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start()
}

/// Selects the row of `name` in the Status pane and opens the merge editor
/// on it with Enter.
fn open_with_enter(term: &mut Terminal, name: &str, header: &str) {
    let (c, r) = term
        .wait_for_text(name, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the file selected", STEP, |s| s.contains(name))
        .unwrap_or_else(|e| panic!("{e}"));
    // A click on a conflicted file already opens its merge editor, and how
    // soon depends on the machine (a ConPTY is slower than a unix pty, a
    // torn frame shows the old screen a while): an Enter sent into an editor
    // that has just opened would pick a chunk. So give the click a good while
    // to show the editor and press Enter only when it left it shut.
    if term
        .wait_for("the editor from the click", Duration::from_secs(8), |s| {
            s.contains(header)
        })
        .is_err()
    {
        term.send_keys("Enter").unwrap();
    }
    term.wait_for("the merge editor", STEP, |s| s.contains(header))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Where B's chunk header checkbox is (the right-hand `[ ] Conflict 1`):
/// waited for with A's on the same row, since a torn frame can show one.
fn b_chunk_header(term: &Terminal) -> (u16, u16) {
    let both = |s: &Screen| {
        s.find("[ ] Conflict 1")
            .and_then(|(c, r)| find_from(s, "[ ] Conflict 1", r, c + 1).map(|b| (b, r)))
    };
    let screen = term
        .wait_for("both chunk headers", STEP, |s| both(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    both(&screen).expect("the predicate just matched")
}

/// The text of the Output pane: every row under its title rule. What the
/// A and B panes show never counts, so a wait on a pick is not satisfied by
/// the panes that always show both sides.
fn output_text(s: &Screen) -> String {
    match (0..s.rows()).find(|r| s.line(*r).contains("Output \u{b7}")) {
        Some(title) => (title + 1..s.rows())
            .map(|r| s.line(r))
            .collect::<Vec<_>>()
            .join("\n"),
        None => String::new(),
    }
}

/// The column of the first `needle` on `row` at or after `from`.
fn find_from(s: &Screen, needle: &str, row: u16, from: u16) -> Option<u16> {
    (from..s.cols()).find(|&c| {
        let mut text = String::new();
        for k in c..s.cols() {
            text.push_str(s.cell(k, row).map_or("", |cell| cell.display()));
            if text.len() >= needle.len() {
                break;
            }
        }
        text.starts_with(needle)
    })
}

/// `ResolveConflict` (take theirs, whole file) sent to the session's daemon
/// on a connection of its own.
fn daemon_resolve(
    config: &tempfile::TempDir,
    session: &str,
    fixture: &ConflictFixture,
    path: &str,
) -> serde_json::Value {
    use gitpoulpi_core::domain::{ConflictResolution, ConflictSide, OperationKind};
    let port_file = config
        .path()
        .join("sessions")
        .join(format!("{session}.port"));
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut client = gitpoulpi_ipc::Client::connect(
            &gitpoulpi_ipc::discovery::endpoint(&port_file)
                .expect("the session published its discovery files"),
        )
        .await
        .expect("connect to the session's daemon");
        let response = client
            .call(&gitpoulpi_ipc::Request {
                id: 1,
                session: session.into(),
                repo: Some(fixture.repo_ref().id),
                command: OperationKind::ResolveConflict,
                args: serde_json::json!({
                    "path": path,
                    "resolution": ConflictResolution::TakeSide(ConflictSide::Theirs),
                }),
            })
            .await
            .expect("call");
        match response.result {
            Ok(v) => v,
            Err(e) => serde_json::json!({ "error": e.message }),
        }
    })
}

fn on_disk(fixture: &ConflictFixture, path: &str) -> String {
    String::from_utf8(std::fs::read(fixture.root().join(path)).unwrap()).unwrap()
}

fn unmerged(fixture: &ConflictFixture) -> Vec<String> {
    common::git(fixture.root(), &["diff", "--name-only", "--diff-filter=U"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// Opens a.txt (three conflicts) from the Status pane, picks A for conflict
/// 1, B then A for conflict 2, a line for conflict 3, types in the Output,
/// saves and marks resolved: the file is staged and the conflicted-files
/// list is back; the global Undo puts the conflict back.
#[test]
fn pick_type_save_and_resolve_then_undo_puts_the_conflict_back() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-flow", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");
    assert!(term.text().contains("file 1 of 3"), "{}", term.text());

    // F7 / Shift+F7 move between the conflicts.
    term.send_keys("F7").unwrap();
    term.wait_for("conflict 2", STEP, |s| s.contains("Conflict 2 / 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Shift+F7: the pty encoder has no shifted F-keys, so the xterm bytes.
    term.send_text("\x1b[18;2~").unwrap();
    term.wait_for("conflict 1 again", STEP, |s| s.contains("Conflict 1 / 3"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Conflict 1: A (current) only.
    term.send_keys("a").unwrap();
    term.wait_for("A's lines in the Output", STEP, |s| {
        output_text(s).contains("A3 ours") && s.contains("2 left")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Conflict 2: B, then A appended: B's line first.
    term.send_keys("F7 b").unwrap();
    term.wait_for("B's line for conflict 2", STEP, |s| {
        output_text(s).contains("A15 theirs") && s.contains("1 left")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("A's chunk appended after B's", STEP, |s| {
        let out = output_text(s);
        matches!((out.find("A15 theirs"), out.find("A15 ours")), (Some(b), Some(a)) if b < a)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Conflict 3: one line from B by the Space key on the line cursor.
    term.send_keys("F7 F6 Space").unwrap();
    // F6 moved to B; Space picked its line.
    term.wait_for("conflict 3 resolved", STEP, |s| s.contains("0 left"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Save and mark resolved: the editor closes, the list is back.
    term.send_keys("C-s").unwrap();
    term.wait_for("the file resolved and the list back", STEP, |s| {
        s.contains("resolved a.txt") && !s.contains("Conflict 3 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = on_disk(&fixture, "a.txt");
    assert!(!text.contains("<<<<<<<"), "{text}");
    assert!(
        text.contains("A3 ours") && text.contains("A28 theirs"),
        "{text}"
    );
    assert_eq!(unmerged(&fixture), ["b.txt", "docs/c.txt"]);

    // The global Undo takes "Save & mark resolved" back.
    term.send_keys("C-z").unwrap();
    // (The index and the file are put back one after the other: wait for both.)
    term.wait_for("the conflict back, markers included", STEP, |_| {
        unmerged(&fixture).contains(&"a.txt".to_string())
            && on_disk(&fixture, "a.txt").contains("<<<<<<<")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The position of `needle` lowest on the screen (the Output sits under
/// the A and B panes, so its copy of a line is the last one).
fn lowest(term: &Terminal, needle: &str) -> (u16, u16) {
    // Waited for: a torn frame can lack the line.
    term.wait_for(&format!("{needle:?} on screen"), STEP, |s| {
        !s.find_all(needle).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
    .find_all(needle)
    .into_iter()
    .max_by_key(|(_, r)| *r)
    .expect("the predicate just matched")
}

/// Typing in the Output marks the conflict edited; a pick replaces the
/// edit with a hint; the editor's undo brings the edit back.
#[test]
fn typing_in_the_output_marks_it_edited_a_pick_replaces_it_and_undo_restores_it() {
    let fixture = ConflictFixture::text();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-type", 200, 50);
    open_with_enter(&mut term, "notes.txt", "Conflict 1 / 1");
    term.send_keys("a").unwrap();
    term.wait_for("A's line in the Output", STEP, |s| {
        output_text(s).contains("three (ours)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Click the Output's copy of the line, go to its end and type.
    // Retried (the line located afresh): a click placed on a torn frame
    // leaves the focus elsewhere, and the typed text would not land.
    common::retry_until(
        &mut term,
        "the typed text and the edited marker",
        4,
        STEP,
        |t| {
            let s = t.screen();
            if !output_text(&s).contains("three (ours)!") {
                if let Some((c, r)) = s
                    .find_all("three (ours)")
                    .into_iter()
                    .max_by_key(|(_, r)| *r)
                {
                    t.click(c + 2, r).unwrap();
                    t.send_keys("End").unwrap();
                    t.send_text("!").unwrap();
                }
            }
        },
        |s| output_text(s).contains("three (ours)!") && s.contains("edited"),
    );

    // Esc goes back to the A pane; taking B replaces the edit.
    // (Esc and the letter in separate writes: together they read as Alt+b.)
    term.send_keys("Escape").unwrap();
    term.wait_for("the A / B key hints back", STEP, |s| {
        s.contains("Space line")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").unwrap();
    term.wait_for("B's line replaced the edit, with the hint", STEP, |s| {
        s.contains("replaced your edit")
            && output_text(s).contains("three (theirs)")
            && !output_text(s).contains("three (ours)!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Ctrl+Z restores the edit (picks and typing share one stack).
    term.send_keys("C-z").unwrap();
    term.wait_for("the edit back", STEP, |s| {
        output_text(s).contains("three (ours)!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Saving writes exactly the Output.
    term.send_keys("C-s").unwrap();
    term.wait_for("the file resolved", STEP, |s| {
        s.contains("resolved notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = on_disk(&fixture, "notes.txt");
    assert!(text.contains("three (ours)!\n"), "{text}");
    assert!(unmerged(&fixture).is_empty());
    term.stop().unwrap();
}

/// The Output is a real editor: select with Shift+arrows, cut and paste,
/// find, indent and outdent with Tab / Shift+Tab, auto-indent on Enter,
/// then one undo stack walks back through a pick and the typing.
#[test]
fn the_output_is_an_editor_selection_clipboard_find_indent_and_one_undo_stack() {
    let fixture = ConflictFixture::text();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-output-edit", 200, 50);
    open_with_enter(&mut term, "notes.txt", "Conflict 1 / 1");
    term.send_keys("a").unwrap();
    term.wait_for("A's line in the Output", STEP, |s| {
        output_text(s).contains("three (ours)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Select the whole line with Shift+Home from its end, cut it.
    let (c, r) = lowest(&term, "three (ours)");
    term.click(c + 2, r).unwrap();
    term.send_keys("End").unwrap();
    term.send_keys("S-Home").unwrap();
    term.send_keys("C-x").unwrap();
    term.wait_for("the line cut out of the Output", STEP, |s| {
        !output_text(s).contains("three (ours)") && s.contains("edited")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-v").unwrap();
    term.wait_for("the line pasted back", STEP, |s| {
        output_text(s).contains("three (ours)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Find: the bar shows in the hint row, the match is selected; Esc
    // closes the bar and keeps the focus in the Output.
    term.send_keys("C-f").unwrap();
    term.send_text("l").unwrap();
    term.send_text("4").unwrap();
    term.wait_for("the find bar", STEP, |s| s.contains("Find: l4"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the find bar closed, the Output still focused", STEP, |s| {
        !s.contains("Find: l4") && s.contains("save & resolve")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Tab indents the match's line, Shift+Tab takes it back.
    term.send_keys("Home").unwrap();
    let (c0, r0) = lowest(&term, "l4");
    term.send_keys("Tab").unwrap();
    term.wait_for("l4 indented", STEP, |s| {
        let (c, r) = s
            .find_all("l4")
            .into_iter()
            .max_by_key(|(_, r)| *r)
            .unwrap();
        r == r0 && c == c0 + 4
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("BTab").unwrap();
    term.wait_for("l4 back in column", STEP, |s| {
        s.find_all("l4")
            .into_iter()
            .any(|(c, r)| r == r0 && c == c0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Enter keeps the indentation: indent l4, then a new line under it.
    term.send_keys("Tab").unwrap();
    term.send_keys("End").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("zq9").unwrap();
    term.wait_for("the typed line aligned under l4", STEP, |s| {
        s.find_all("zq9")
            .into_iter()
            .any(|(c, r)| r == r0 + 1 && c == c0 + 4)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A pick, then undo walks back: the pick, the typing, the Enter.
    term.send_keys("Escape").unwrap();
    term.wait_for("the A / B key hints back", STEP, |s| {
        s.contains("Space line")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").unwrap();
    term.wait_for("B's line in the Output", STEP, |s| {
        output_text(s).contains("three (theirs)") && output_text(s).contains("zq9")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    term.wait_for("the pick undone, the typing kept", STEP, |s| {
        output_text(s).contains("three (ours)") && output_text(s).contains("zq9")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    term.wait_for("the typing undone", STEP, |s| {
        !output_text(s).contains("zq9")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    term.wait_for("the Enter undone: l4 is back to back with l5", STEP, |s| {
        let rows: Vec<_> = s
            .find_all("l4")
            .into_iter()
            .filter(|(_, r)| *r == r0)
            .collect();
        let below = s.find_all("l5").into_iter().any(|(_, r)| r == r0 + 1);
        !rows.is_empty() && below
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // What is saved is exactly the Output (l4 is still indented).
    term.send_keys("C-s").unwrap();
    term.wait_for("the file resolved", STEP, |s| {
        s.contains("resolved notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = on_disk(&fixture, "notes.txt");
    assert!(text.contains("three (ours)\n    l4\nl5\n"), "{text}");
    assert!(!text.contains("zq9"), "{text}");
    term.stop().unwrap();
}

/// Everything by mouse: a chunk checkbox, a single line, the ▼ arrow, then
/// Save & mark resolved with two conflicts left unpicked, which asks first.
#[test]
fn the_mouse_picks_chunks_and_lines_and_the_save_confirms_unpicked_conflicts() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-mouse", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");

    // The checkbox on B's chunk header (the right-hand one).
    let (c, r) = b_chunk_header(&term);
    term.click(c + 1, r).unwrap();
    term.wait_for("B's lines in the Output", STEP, |s| {
        let out = output_text(s);
        out.contains("A3 theirs") && out.contains("A4 theirs") && s.contains("2 left")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // One line of A, by clicking it.
    let (c, r) = term
        .wait_for_text("A3 ours", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("A's line appended after B's", STEP, |s| {
        let out = output_text(s);
        matches!((out.find("A4 theirs"), out.find("A3 ours")), (Some(b), Some(a)) if b < a)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // ▼ moves to conflict 2.
    let (c, r) = term
        .wait_for_text("\u{25bc}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("conflict 2", STEP, |s| s.contains("Conflict 2 / 3"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Save & mark resolved: two conflicts have nothing picked, so it asks.
    let (c, r) = term
        .wait_for_text("Save & mark resolved", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("the confirm box", STEP, |s| {
        s.contains("Save anyway?") && s.contains("2 conflicts have nothing picked")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Save anyway (", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("the file resolved", STEP, |s| s.contains("resolved a.txt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let text = on_disk(&fixture, "a.txt");
    assert!(text.contains("A3 theirs\nA4 theirs\nA3 ours\n"), "{text}");
    assert!(!text.contains("A15") && !text.contains("A28"), "{text}");
    assert!(!unmerged(&fixture).contains(&"a.txt".to_string()));
    term.stop().unwrap();
}

/// Alt+PageDown / Alt+PageUp walk the conflicted files; unsaved work asks
/// first (Discard goes on).
#[test]
fn next_and_previous_file_ask_before_dropping_unsaved_work() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-files", 200, 50);
    open_with_enter(&mut term, "a.txt", "file 1 of 3");
    // Alt+PageDown: the xterm modifier form (an ESC prefix would read as Esc).
    term.send_text("\x1b[6;3~").unwrap();
    term.wait_for("the second file", STEP, |s| {
        s.contains("file 2 of 3") && s.contains("Merge \u{b7} b.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1b[5;3~").unwrap();
    term.wait_for("back on the first", STEP, |s| s.contains("file 1 of 3"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Unsaved picks: going on asks Save / Discard / Cancel.
    term.send_keys("a").unwrap();
    term.send_text("\x1b[6;3~").unwrap();
    term.wait_for("the unsaved-changes box", STEP, |s| {
        s.contains("Unsaved changes") && s.contains("Discard (")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Cancel stays.
    term.send_keys("Escape").unwrap();
    term.wait_for("still on the first file", STEP, |s| {
        s.contains("file 1 of 3") && !s.contains("Unsaved changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("\x1b[6;3~").unwrap();
    // The box opens a frame after the key; a `d` sent with the key, ahead of
    // the box, is read by the editor underneath and lost.
    term.wait_for("the unsaved-changes box again", STEP, |s| {
        s.contains("Unsaved changes") && s.contains("Discard (")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("the second file after discarding", STEP, |s| {
        s.contains("file 2 of 3") && s.contains("Merge \u{b7} b.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The discarded picks never reached the disk.
    assert!(on_disk(&fixture, "a.txt").contains("<<<<<<<"));
    term.stop().unwrap();
}

/// The narrow layout is the last resort, for a really tiny terminal (under
/// 60 columns, the editor having the whole width): a tab strip, Tab
/// switching the side, picks still work.
#[test]
fn the_narrow_layout_has_tabs_and_picks_work() {
    let fixture = ConflictFixture::text();
    // The Status pane is cramped at 56 columns, but its row is there.
    let (mut term, _config) = common::launch(fixture.root())
        .session("pty-merge-narrow")
        .size(56, 30)
        .no_exit_pause()
        .ready_when("the conflicted row", |s| s.contains("! notes.txt"))
        .mouse_capture()
        .start();
    let (c, r) = term
        .wait_for_text("! notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.send_keys("Enter").unwrap();
    // (The narrow toolbar counts `1/1`.)
    // Side A is shown (and only side A) once the editor is up: the tab strip
    // is drawn a frame before its content is, so the content is in the wait.
    term.wait_for("the merge editor with its tab strip on side A", STEP, |s| {
        s.contains("A Current")
            && s.contains("B Incoming")
            && s.contains("three (ours)")
            && !s.contains("three (theirs)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Tab moves to B's tab and its block.
    term.send_keys("Tab").unwrap();
    term.wait_for("B's block", STEP, |s| s.contains("three (theirs)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("B's chunk in the Output", STEP, |s| {
        s.text().matches("three (theirs)").count() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The operation banner: Resolve conflicts opens the editor, Abort asks and
/// then really aborts the merge.
#[test]
fn the_banner_resolves_and_aborts_the_merge() {
    let fixture = ConflictFixture::text();
    let head = common::git_trimmed(fixture.root(), &["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(fixture.root(), "pty-merge-banner", 200, 50);
    let (c, r) = term
        .wait_for_text("Resolve conflicts", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.wait_for("the editor opened by the banner", STEP, |s| {
        s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}"));

    let (c, r) = term
        .wait_for_text("Abort merge", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Abort the merge in progress?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the merge gone", STEP, |s| !s.contains("Merge in progress"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(common::git_try(
        fixture.root(),
        &["rev-parse", "-q", "--verify", "MERGE_HEAD"]
    )
    .is_none());
    assert_eq!(
        common::git_trimmed(fixture.root(), &["rev-parse", "HEAD"]),
        head
    );
    assert!(unmerged(&fixture).is_empty());
    term.stop().unwrap();
}

/// The file is resolved behind the editor's back: a strip says so, and
/// Unresolve and reopen puts the conflict back.
#[test]
fn a_file_resolved_elsewhere_gets_a_strip_and_unresolve_reopens_it() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, config) = launch(fixture.root(), "pty-merge-stale", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");
    // Resolved through the daemon, as `gitpoulpi conflicts resolve` does
    // (so the daemon remembers the stages and Unresolve can put them back).
    let resolved = daemon_resolve(&config, "pty-merge-stale", &fixture, "a.txt");
    assert!(resolved.get("error").is_none(), "{resolved}");
    term.wait_for("the resolved-outside strip", STEP, |s| {
        s.contains("resolved outside the editor") && s.contains("Unresolve and reopen")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A save of the stale picks is refused and writes nothing.
    let before = on_disk(&fixture, "a.txt");
    term.send_keys("a C-s").unwrap();
    term.wait_for("the confirm box", STEP, |s| s.contains("Save anyway?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("s").unwrap();
    term.wait_for("the refusal", STEP, |s| {
        s.contains("not conflicted") || s.contains("nothing was written")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(on_disk(&fixture, "a.txt"), before, "nothing was written");

    let (c, r) = term
        .wait_for_text("Unresolve and reopen", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("the conflict reopened", STEP, |s| {
        s.contains("Conflict 1 / 3") && !s.contains("resolved outside the editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(unmerged(&fixture).contains(&"a.txt".to_string()));
    term.stop().unwrap();
}

/// Every way in: a click on the `!` row, the row's menu, the
/// palette, the Diff view's key and its bar button.
#[test]
fn every_entry_point_opens_the_editor() {
    let fixture = ConflictFixture::text();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-entry", 200, 50);
    let close = |term: &mut Terminal| {
        term.send_keys("Escape").unwrap();
        term.wait_for("the editor closed", STEP, |s| !s.contains("Conflict 1 / 1"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    };

    // A single click.
    common::click_text_until(&mut term, "notes.txt", "the editor (single click)", |s| {
        s.contains("Conflict 1 / 1")
    });
    close(&mut term);

    // The row's context menu leads with it.
    let screen = common::right_click_text_until(&mut term, "notes.txt", "the file menu", |s| {
        s.contains("Open in merge editor")
    });
    let (mc, mr) = screen.find("Open in merge editor").unwrap();
    term.click(mc + 2, mr).unwrap();
    term.wait_for("the editor (menu)", STEP, |s| s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    close(&mut term);

    // The palette.
    term.send_keys("C-p").unwrap();
    term.send_text("Open Merge Editor").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Conflicts: Open Merge Editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor (palette)", STEP, |s| {
        s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    close(&mut term);

    // The Diff view of the file: its bar button and its key.
    // (A click on the row would reopen the editor: the Status pane's `d`
    // opens the diff view, which shows the conflict card.)
    // (Retried: a key sent while the editor is still closing is lost, and
    // a late refresh can drop the row's selection, which `d` needs. A retry
    // selects the row again by opening and closing its editor.)
    let mut tries = 0;
    common::retry_until(
        &mut term,
        "the diff with the Merge Editor button",
        4,
        Duration::from_secs(15),
        |t| {
            if t.screen().contains(" Merge editor ") {
                return;
            }
            if tries > 0 {
                common::click_text_until(t, "notes.txt", "the editor (reselect)", |s| {
                    s.contains("Conflict 1 / 1")
                });
                close(t);
            }
            tries += 1;
            t.send_keys("d").unwrap();
        },
        |s| s.contains(" Merge editor "),
    );
    let (bc, br) = term
        .wait_for_text(" Merge editor ", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(bc + 2, br).unwrap();
    term.wait_for("the editor (diff button)", STEP, |s| {
        s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    close(&mut term);
    term.send_text("E").unwrap();
    term.wait_for("the editor (diff key)", STEP, |s| {
        s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A file edited by hand (no markers left) offers to continue from the
/// disk text or to start over from the merge.
#[test]
fn a_file_edited_outside_offers_the_disk_text() {
    let fixture = ConflictFixture::text();
    std::fs::write(
        fixture.root().join("notes.txt"),
        "hand-resolved line one\nhand-resolved line two\n",
    )
    .unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-disk", 200, 50);
    open_with_enter(&mut term, "notes.txt", "Conflict 1 / 1");
    term.wait_for("the edited-outside strip", STEP, |s| {
        s.contains("edited outside the editor") && s.contains("Continue from the file on disk")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Continue from the file on disk", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("the disk text in the Output, picks off", STEP, |s| {
        output_text(s).contains("hand-resolved line two") && s.contains("edited outside")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Picks are off until Start over: a is refused with a message.
    term.send_keys("a").unwrap();
    term.wait_for("the refusal", STEP, |s| s.contains("picks are off"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The A and B panes are syntax-highlighted like the Output (a Rust file
/// built by `xtask merge-fixture`): the keywords and the name of one line
/// are not all painted in one color, in any of the three panes.
#[test]
fn the_side_panes_are_syntax_highlighted() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    assert_eq!(
        xtask::merge_fixture::run(&["--dir".into(), repo.to_string_lossy().into_owned()]),
        Ok(0)
    );
    let (mut term, _config) = launch(&repo, "pty-merge-highlight", 200, 50);
    open_with_enter(&mut term, "user.rs", "Conflict 1 / 2");
    // Take B's chunk so the Output holds Rust too, then wait for every pane.
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    let needle = "pub fn greeting";
    let colors = |s: &Screen, row: u16, from: u16| -> usize {
        let mut seen = std::collections::HashSet::new();
        for c in from..from + needle.len() as u16 {
            if let Some(cell) = s.cell(c, row) {
                if cell.text.trim() != "" {
                    seen.insert(format!("{:?}", cell.fg));
                }
            }
        }
        seen.len()
    };
    let screen = term
        .wait_for("every pane colored", STEP, |s| {
            let Some((c, r)) = s.find(needle) else {
                return false;
            };
            let b = find_from(s, needle, r, c + 1);
            let out_row = (r + 1..s.rows()).find(|row| s.line(*row).contains(needle));
            let o = out_row.and_then(|row| find_from(s, needle, row, 0).map(|c| (row, c)));
            match (b, o) {
                (Some(b), Some((orow, oc))) => {
                    colors(s, r, c) > 1 && colors(s, r, b) > 1 && colors(s, orow, oc) > 1
                }
                _ => false,
            }
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(screen);
    term.stop().unwrap();
}

/// Full file (S7b): the toolbar toggle (by key and by click) colors A's
/// lines against the base like the diff view does, and picks still work.
#[test]
fn full_file_toggles_by_key_and_click_and_picks_still_work() {
    // The diff view's added-line background.
    let add_bg = xtask::pty::Color::Rgb(24, 52, 38);
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-fullfile", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");
    // A15 is a conflict line of the second conflict, not under the cursor.
    let bg_of_a15 = |s: &Screen| -> Option<xtask::pty::Color> {
        let (c, r) = s.find("A15 ours")?;
        s.cell(c, r).map(|cell| cell.bg)
    };
    let screen = term.screen();
    assert_ne!(bg_of_a15(&screen), Some(add_bg), "{}", term.text());

    // The key (Alt+F) turns it on, and again off.
    term.send_text("\x1bf").unwrap();
    term.wait_for("A's changed lines colored", STEP, |s| {
        bg_of_a15(s) == Some(add_bg)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1bf").unwrap();
    term.wait_for("the colors gone", STEP, |s| {
        bg_of_a15(s).is_some() && bg_of_a15(s) != Some(add_bg)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The toolbar button (a click) turns it on.
    let (c, r) = term
        .wait_for_text("Full file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 2, r).unwrap();
    term.wait_for("A's changed lines colored", STEP, |s| {
        bg_of_a15(s) == Some(add_bg)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Picking by the mouse works on top of it: B's chunk header checkbox.
    let (c, r) = b_chunk_header(&term);
    term.click(c + 1, r).unwrap();
    term.wait_for("B's lines in the Output", STEP, |s| {
        let out = output_text(s);
        out.contains("A3 theirs") && out.contains("A4 theirs") && s.contains("2 left")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // And by key: take A for the next conflict.
    term.send_keys("F7").unwrap();
    term.wait_for("conflict 2", STEP, |s| s.contains("Conflict 2 / 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("a").unwrap();
    term.wait_for("A's lines in the Output", STEP, |s| {
        output_text(s).contains("A15 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The row of the Output's title rule.
fn output_title_row(s: &Screen) -> u16 {
    output_title_at(s).expect("the Output's title")
}

/// [`output_title_row`] for `wait_for` predicates: a frame caught mid-redraw
/// (under load) may not show the title yet, which is "not yet", not a failure.
fn output_title_at(s: &Screen) -> Option<u16> {
    (0..s.rows()).find(|r| s.line(*r).contains("Output \u{b7}"))
}

/// The right-click menu of a conflict, the wheel scrolling A, B and the
/// Output together, `z` cycling the Output's height and the splitter drag.
#[test]
fn the_context_menu_the_wheel_the_height_key_and_the_splitter() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-misc", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");

    // Right-click B's chunk header: the menu's "Take B only" row.
    let (bc, r) = b_chunk_header(&term);
    term.right_click(bc + 1, r).unwrap();
    let menu = term
        .wait_for("the chunk menu", STEP, |s| s.contains("Take B only"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (mc, mr) = menu.find("Take B only").unwrap();
    term.click(mc + 2, mr).unwrap();
    term.wait_for("B's lines in the Output", STEP, |s| {
        let out = output_text(s);
        out.contains("A3 theirs") && out.contains("A4 theirs") && !out.contains("A3 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `z` cycles the Output's height: its title rule moves.
    let before = output_title_row(&term.screen());
    term.send_keys("z").unwrap();
    term.wait_for("the Output taller", STEP, |s| {
        output_title_at(s).is_some_and(|r| r != before)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("z z").unwrap();
    term.wait_for("back to the first height", STEP, |s| {
        output_title_at(s) == Some(before)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Dragging the Output's title rule up makes it taller.
    let row = output_title_row(&term.screen());
    term.drag((60, row), (60, row - 6)).unwrap();
    term.wait_for("the splitter moved", STEP, |s| {
        output_title_at(s).is_some_and(|r| r < row)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The wheel over the A / B panes scrolls them (and the Output follows).
    term.scroll(true, 40, 8).unwrap();
    term.wait_for("the panes scrolled", STEP, |s| {
        !s.text().contains("   1 l1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Owner (first real run): the editor has
/// the whole width. Opening it hides the Refs sidebar and the right panel,
/// A and B are side by side whatever the width, F9 brings the panels back
/// and hides them again, and closing the editor restores them.
#[test]
fn the_editor_takes_the_full_width_side_by_side_and_gives_the_panels_back() {
    let fixture = ConflictFixture::text();
    // The center alone was ~60 columns with the panels around it.
    let (mut term, _config) = launch(fixture.root(), "pty-merge-fullwidth", 150, 40);
    assert!(term.text().contains("LOCAL"), "the Refs sidebar is up");
    open_with_enter(&mut term, "notes.txt", "Conflict 1 / 1");
    term.wait_for("the side panels hidden", STEP, |s| {
        !s.contains("LOCAL") && !s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A | B side by side: both pane titles on one row, no tab strip.
    let screen = term.screen();
    let row = (0..screen.rows())
        .find(|r| screen.line(*r).contains("A \u{b7} ") && screen.line(*r).contains("B \u{b7} "))
        .unwrap_or_else(|| panic!("A and B titles on one row\n{}", term.text()));
    let line = screen.line(row);
    assert!(
        line.find("A \u{b7} ").unwrap() < line.find("B \u{b7} ").unwrap(),
        "{line}"
    );

    // F9: the panels come back around the editor; F9 again hides them.
    term.send_keys("F9").unwrap();
    term.wait_for("the panels back", STEP, |s| {
        s.contains("LOCAL") && s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("F9").unwrap();
    term.wait_for("the panels hidden again", STEP, |s| {
        !s.contains("LOCAL") && s.contains("Conflict 1 / 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Closing restores them.
    term.send_keys("Escape").unwrap();
    term.wait_for("the editor closed, the panels back", STEP, |s| {
        !s.contains("Conflict 1 / 1") && s.contains("LOCAL") && s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A and B pane titles on one row, A to the left of B: the editor is side
/// by side (the narrow layout has a tab strip instead).
fn side_by_side(s: &Screen) -> bool {
    (0..s.rows()).any(|r| {
        let line = s.line(r);
        matches!((line.find("A \u{b7} "), line.find("B \u{b7} ")), (Some(a), Some(b)) if a < b)
    })
}

/// The width of the editor's frame: from its `╭` to the matching `╮` on the
/// title row.
fn editor_width(term: &Terminal) -> u16 {
    let screen = term.screen();
    let row = (0..screen.rows())
        .find(|r| screen.line(*r).contains("\u{256d} Merge"))
        .unwrap_or_else(|| panic!("the editor's title row\n{}", term.text()));
    let cell = |c: u16| screen.cell(c, row).map_or("", |x| x.display()).to_string();
    let left = (0..screen.cols()).find(|c| cell(*c) == "\u{256d}").unwrap();
    let right = (left..screen.cols())
        .rev()
        .find(|c| cell(*c) == "\u{256e}")
        .unwrap();
    right - left + 1
}

/// Opens `notes.txt` in a terminal of `cols` x `rows` with this config.
fn open_with_config(
    session: &str,
    cols: u16,
    rows: u16,
    config: &str,
) -> (Terminal, ConflictFixture, tempfile::TempDir) {
    let fixture = ConflictFixture::text();
    let (mut term, dir) = common::launch(fixture.root())
        .session(session)
        .size(cols, rows)
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
    term.wait_for("the editor", STEP, |s| {
        s.contains("Output \u{b7}") || s.contains("Output\u{b7}") || s.contains("1/1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, fixture, dir)
}

/// Owner: a normal terminal (160x45), the default panels and the default
/// threshold, and A and B are side by side in the top area, on the same row.
#[test]
fn a_normal_terminal_shows_a_and_b_side_by_side_with_the_defaults() {
    let (term, _fixture, _dir) = open_with_config("pty-merge-normal", 160, 45, "");
    term.wait_for("A and B on one row", STEP, side_by_side)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The editor has the window: the sidebar and the right panel are away.
    assert!(!term.text().contains("LOCAL"), "{}", term.text());
    term.stop().unwrap();
}

/// `merge.side_by_side_min_width` at the boundary: an editor exactly that
/// wide is side by side, one column narrower than the setting falls back to
/// the tabs.
#[test]
fn the_side_by_side_threshold_switches_exactly_at_the_setting() {
    // How wide is the editor in a 120-column terminal?
    let (term, _f, _d) = open_with_config(
        "pty-merge-bound-0",
        120,
        40,
        "[merge]\nside_by_side_min_width = 100\n",
    );
    term.wait_for("side by side at the default", STEP, side_by_side)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let width = editor_width(&term);
    assert!(width >= 100, "{width}");
    term.stop().unwrap();

    // The setting equal to the width: still side by side.
    let (term, _f, _d) = open_with_config(
        "pty-merge-bound-1",
        120,
        40,
        &format!("[merge]\nside_by_side_min_width = {width}\n"),
    );
    term.wait_for("side by side at exactly the width", STEP, |s| {
        side_by_side(s)
    })
    .unwrap_or_else(|e| panic!("{width}\n{e}\n{}", term.text()));
    term.stop().unwrap();

    // One more than the width: the tabs.
    let (term, _f, _d) = open_with_config(
        "pty-merge-bound-2",
        120,
        40,
        &format!("[merge]\nside_by_side_min_width = {}\n", width + 1),
    );
    term.wait_for("the tab strip one column below", STEP, |s| {
        s.text().contains("Base") && s.text().contains("A Current") && !side_by_side(s)
    })
    .unwrap_or_else(|e| panic!("{width}\n{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `0` is "always side by side", even in a window that would otherwise get
/// the tabs.
#[test]
fn a_threshold_of_zero_is_always_side_by_side() {
    let (term, _f, _d) = open_with_config(
        "pty-merge-always",
        70,
        30,
        "[merge]\nside_by_side_min_width = 0\n",
    );
    term.wait_for("side by side at 70 columns", STEP, side_by_side)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The column and row of the A | B divider's `┬` on the pane-title rule.
fn divider_at(s: &Screen) -> (u16, u16) {
    s.find("\u{252c}").unwrap_or((0, 0))
}

/// [`divider_at`] once the divider is drawn (a torn frame can lack it).
fn wait_divider(term: &Terminal) -> (u16, u16) {
    let s = term
        .wait_for("the A | B divider", STEP, |s| s.find("\u{252c}").is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    divider_at(&s)
}

/// Slice SR: the A | B divider is draggable (B gets wider), a double click
/// resets it, `Alt+arrows` move it by keys, the Output splitter still
/// drags, and `Reset Pane Sizes` is in the palette.
#[test]
fn the_ab_divider_drags_double_click_resets_and_the_keys_move_it() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-resize", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");
    let (d0, row) = wait_divider(&term);

    // Hovering the divider lights its rule (the cells become bold-weight).
    term.move_to(d0, row + 3).unwrap();
    term.wait_for("the divider lit", STEP, |s| {
        s.cell(d0, row + 3)
            .is_some_and(|c| c.display() == "\u{2503}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Drag it left: A narrower, B wider.
    term.drag((d0, row + 3), (d0 - 30, row + 3)).unwrap();
    term.wait_for("the divider moved left", STEP, |s| {
        divider_at(s).0 == d0 - 30
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A double click on it resets.
    std::thread::sleep(Duration::from_millis(500));
    term.double_click(d0 - 30, row + 3).unwrap();
    term.wait_for("the divider back", STEP, |s| divider_at(s).0 == d0)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The keys: Alt+Right widens A, Alt+Left widens B.
    // xterm's modified-arrow encoding: Alt+Right / Alt+Left.
    term.send_text("\x1b[1;3C").unwrap();
    term.wait_for("A wider", STEP, |s| divider_at(s).0 > d0)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1b[1;3D\x1b[1;3D").unwrap();
    term.wait_for("B wider", STEP, |s| divider_at(s).0 < d0)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The Output splitter still drags.
    let orow = output_title_row(&term.screen());
    term.drag((60, orow), (60, orow - 6)).unwrap();
    term.wait_for("the splitter moved", STEP, |s| {
        output_title_at(s).is_some_and(|r| r < orow)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The palette entry resets both.
    term.send_keys("C-p").unwrap();
    term.send_text("Reset Pane Sizes").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Merge: Reset Pane Sizes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("everything reset", STEP, |s| {
        divider_at(s).0 == d0 && output_title_at(s) == Some(orow)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A click on a `!` row opens the editor at once (no diff view first);
/// moving onto the row with the arrows opens nothing, `d` shows the card
/// instead of a diff, and Enter opens the editor.
#[test]
fn a_click_on_a_conflicted_row_opens_the_editor_and_the_diff_view_shows_a_card() {
    let fixture = ConflictFixture::text();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-click", 200, 50);

    // Arrow onto the row: nothing opens.
    term.send_keys("Tab").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Up").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let s = term.screen();
    assert!(
        !s.contains("Conflict 1 / 1") && !s.contains("A Current"),
        "{}",
        term.text()
    );

    // `d` on the selected `!` row: the card, no diff.
    term.send_keys("d").unwrap();
    term.wait_for("the conflict card", STEP, |s| {
        s.contains("Conflicted file") && s.contains(" Merge editor ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.screen().contains("A Current"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the card closed", STEP, |s| !s.contains("Conflicted file"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Enter opens the editor.
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor (Enter)", STEP, |s| {
        s.contains("A Current") && s.contains("B Incoming")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // One click on the row: the editor, never the diff view.
    let screen = common::click_text_until(&mut term, "notes.txt", "the editor (click)", |s| {
        s.contains("A Current") && s.contains("B Incoming")
    });
    assert!(!screen.contains("Conflicted file"));
    term.stop().unwrap();
}

/// Owner: "When I've selected Take all A or B, how do I un-check everything
/// with a shortcut? We almost need an extra button." `u` un-picks the current
/// conflict, the toolbar's Reset button and Alt+R reset the whole file, and
/// the box asks first when text was typed in the Output.
#[test]
fn unpick_and_reset_by_key_button_and_palette() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-reset", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");

    // `a` picks conflict 1, `u` takes the pick back.
    term.send_keys("a").unwrap();
    term.wait_for("conflict 1 picked", STEP, |s| s.contains("2 left"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("u").unwrap();
    term.wait_for("conflict 1 unresolved again", STEP, |s| {
        s.contains("3 left") && !output_text(s).contains("A3 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Take all A (Shift+A), then the toolbar's Reset button.
    term.send_text("A").unwrap();
    term.wait_for("everything taken", STEP, |s| s.contains("0 left"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Reset", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    term.wait_for("the file reset", STEP, |s| {
        s.contains("3 left") && !output_text(s).contains("A15 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Take all B, type in the Output: Alt+R now asks first.
    term.send_text("B").unwrap();
    term.wait_for("B taken", STEP, |s| s.contains("0 left"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-3").unwrap();
    term.send_text("Z").unwrap();
    term.wait_for("the typed text", STEP, |s| {
        output_text(s).contains("ZA3 theirs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-r").unwrap();
    term.wait_for("the confirm box", STEP, |s| s.contains("Reset the Output?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc keeps the text.
    term.send_keys("Esc").unwrap();
    term.wait_for("the box closed, the text kept", STEP, |s| {
        !s.contains("Reset the Output?") && output_text(s).contains("ZA3 theirs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The palette entry asks too; Enter resets.
    term.send_keys("C-p").unwrap();
    term.send_text("Reset Output").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Merge: Reset Output to Merge Result")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the confirm box", STEP, |s| s.contains("Reset the Output?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("reset, typed text gone", STEP, |s| {
        s.contains("3 left") && !output_text(s).contains("ZA3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Owner: fluid navigation. Alt+1 / 2 / 3 and `Ctrl+N` then 1 / 2 / 3 pick a
/// pane, `h` / `l` and the arrows walk A, B, Output on the SAME conflict,
/// counts repeat motions (`2n`, `2j`).
#[test]
fn panes_by_digit_h_l_on_the_same_conflict_and_vim_counts() {
    let fixture = ConflictFixture::multi_conflict();
    let (mut term, _config) = launch(fixture.root(), "pty-merge-nav", 200, 50);
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");

    // Conflict 2, picked A so its Output region has a line to type in front of.
    term.send_keys("F7 a").unwrap();
    term.wait_for("conflict 2 picked", STEP, |s| {
        s.contains("Conflict 2 / 3") && output_text(s).contains("A15 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `l` `l`: A -> B -> Output, landing on conflict 2 (not the top).
    term.send_keys("l l").unwrap();
    term.send_text("Z").unwrap();
    term.wait_for("typed at conflict 2", STEP, |s| {
        output_text(s).contains("ZA15 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!output_text(&term.screen()).contains("ZA3"));

    // Alt+1 goes back to A, still on conflict 2; `Right` is the same as `l`.
    term.send_keys("M-1").unwrap();
    term.send_keys("Right Right").unwrap();
    term.send_text("Y").unwrap();
    term.wait_for("typed again at conflict 2", STEP, |s| {
        output_text(s).contains("YZA15 ours")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The navigation prefix with the badges: Ctrl+N shows 1 2 3, `1` is A.
    term.send_keys("C-n").unwrap();
    // (`1` and `3` are on any screen; the footer's `<badge>=` is the prefix
    // frame, and the badge key resolves against the last painted frame.)
    term.wait_for("the badges", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("1").unwrap();
    // In A, `j` moves a line cursor and the vim count repeats it: `2n` is two
    // conflicts on (conflict 2 -> 1, wrapping), `2N` back.
    term.send_text("2n").unwrap();
    term.wait_for("two conflicts on", STEP, |s| s.contains("Conflict 1 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("2N").unwrap();
    term.wait_for("two conflicts back", STEP, |s| s.contains("Conflict 2 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("F7").unwrap();
    term.wait_for("conflict 3", STEP, |s| s.contains("Conflict 3 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // `Ctrl+N 3` is the Output, on conflict 3.
    term.send_keys("C-n").unwrap();
    term.wait_for("the badges", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("3").unwrap();
    term.send_text("Q").unwrap();
    term.wait_for("typed at conflict 3", STEP, |s| {
        output_text(s).contains("Q") && !output_text(s).contains("QA15")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc back to a side pane, then `2j` crosses from conflict 1's two-line
    // block into conflict 2.
    term.send_keys("M-1").unwrap();
    term.send_text("2N").unwrap();
    term.wait_for("conflict 1", STEP, |s| s.contains("Conflict 1 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("2j").unwrap();
    term.wait_for("2j crossed into conflict 2", STEP, |s| {
        s.contains("Conflict 2 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("2k").unwrap();
    term.wait_for("2k went back to conflict 1", STEP, |s| {
        s.contains("Conflict 1 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Shift+F7 / Alt+Up: the previous conflict (Shift+F7 is the xterm
    // sequence, the encoder has no shifted F-keys).
    term.send_keys("F7 F7").unwrap();
    term.wait_for("conflict 3", STEP, |s| s.contains("Conflict 3 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1b[18;2~").unwrap();
    term.wait_for("Shift+F7: conflict 2", STEP, |s| {
        s.contains("Conflict 2 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("\x1b[1;3A").unwrap();
    term.wait_for("Alt+Up: conflict 1", STEP, |s| s.contains("Conflict 1 / 3"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Owner: "If the file is very large, does the scrollbar show the position of
/// conflicts? It would be nice." Marks on the track of the A / B panes and of
/// the Output, one per conflict; a click on one jumps there.
#[test]
fn the_scrollbars_mark_the_conflicts_and_a_click_on_a_mark_jumps() {
    let fixture = ConflictFixture::multi_conflict();
    // Short enough that a.txt (30 lines) scrolls. At 24 rows the Unstaged
    // list shows one row, so keep a.txt above the docs/ folder.
    let (mut term, _config) = launch_with(
        fixture.root(),
        "pty-merge-marks",
        200,
        24,
        "[ui]\ntree_folders_first = false\n",
    );
    open_with_enter(&mut term, "a.txt", "Conflict 1 / 3");
    // The scrollbar's column (the up arrow of the A / B track) and the marks
    // on both tracks are one whole state: the frame that opens the editor
    // does not yet carry them all, so they are what the wait is for.
    let layout = |s: &Screen| -> Option<(u16, u16, Vec<u16>, Vec<u16>)> {
        let title = output_title_at(s)?;
        let col = s
            .find_all("\u{2191}")
            .into_iter()
            .filter(|(_, r)| *r < title)
            .map(|(c, _)| c)
            .max()?;
        let marks = |rows: std::ops::Range<u16>| -> Vec<u16> {
            rows.filter(|r| {
                s.cell(col, *r)
                    .is_some_and(|c| c.display() == "\u{2590}" || c.display() == "\u{25c6}")
            })
            .collect()
        };
        Some((col, title, marks(0..title), marks(title..s.rows())))
    };
    let screen = term
        .wait_for("the marks on both tracks", STEP, |s| {
            matches!(layout(s), Some((_, _, top, output)) if top.len() >= 2 && !output.is_empty())
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, _title, top, _output) = layout(&screen).expect("the wait held");
    let marks_in = |s: &Screen, rows: std::ops::Range<u16>| -> Vec<u16> {
        rows.filter(|r| {
            s.cell(col, *r)
                .is_some_and(|c| c.display() == "\u{2590}" || c.display() == "\u{25c6}")
        })
        .collect()
    };

    // Click the lowest mark of the top track: conflict 3.
    let last = *top.last().unwrap();
    term.click(col, last).unwrap();
    // (The marks to click next are part of the wait: `Conflict 3 / 3` is in
    // the header a frame before the scrolled tracks are redrawn.)
    let screen = term
        .wait_for("the click jumped to conflict 3", STEP, |s| {
            s.contains("Conflict 3 / 3")
                && output_title_at(s).is_some_and(|t| !marks_in(s, 0..t).is_empty())
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // And the first mark goes back to conflict 1 (outside the thumb now that
    // the panes scrolled down).
    let first = marks_in(&screen, 0..output_title_row(&screen))
        .into_iter()
        .next()
        .expect("a mark");
    term.click(col, first).unwrap();
    term.wait_for("the click jumped back", STEP, |s| {
        s.contains("Conflict 1 / 3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
