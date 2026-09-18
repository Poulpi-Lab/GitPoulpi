//! End-to-end check of "Amend...": the Log's commit menu, the commit-detail
//! pane's per-property menu (every entry point opens the complete dialog), the double-click and the keyboard chords - all
//! driven through a pty by `xtask::pty` (real binary, real daemon, real
//! `git`).
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the whole state it is
//! waiting for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// base <- target (with a body) <- top. Returns the repo and the author name
/// the fixture commits with.
fn fixture() -> (FixtureRepo, String) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .commit("target summary")
        .unwrap()
        .build();
    repo.git(&[
        "commit",
        "--amend",
        "--allow-empty",
        "-m",
        "target summary",
        "-m",
        "target body text",
    ]);
    std::fs::write(repo.root().join("top.txt"), "top").unwrap();
    repo.git(&["add", "top.txt"]);
    repo.git(&["commit", "-m", "top commit"]);
    let author = repo.git(&["log", "-1", "--format=%an"]).trim().to_string();
    (repo, author)
}

fn start(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("top commit")
        .mouse_capture()
        .start()
}

/// The rightmost place `needle` is on screen: the detail pane (the Log is
/// to its left).
fn in_detail_pane(term: &Terminal, needle: &str) -> Option<(u16, u16)> {
    let screen = term.screen();
    let mut best: Option<(u16, u16)> = None;
    for row in 0..screen.rows() {
        let line = screen.line(row);
        for (byte, _) in line.match_indices(needle) {
            // Not the changed-files row (`target summary.txt`).
            if line[byte + needle.len()..].starts_with('.') {
                continue;
            }
            let col = line[..byte].chars().count() as u16;
            if best.map_or(true, |(c, _)| col > c) {
                best = Some((col, row));
            }
        }
    }
    best
}

/// Selects the "target summary" row and waits for its body to show in the
/// detail pane.
fn select_target(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("target summary", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the target's body in the detail pane", STEP, |s| {
        s.contains("target body text")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Right-click the commit in the Log -> Amend... -> the dialog opens with
/// every field filled; the author name is changed (two Tabs from the
/// summary) and applied; summary and body stay, the commit on top is
/// replayed with its own author.
#[test]
fn the_log_menu_amend_changes_the_author_and_keeps_the_message() {
    let (repo, author) = fixture();
    let top_author_date = repo.git(&["log", "-1", "--date=raw", "--format=%an|%ad", "HEAD"]);
    let (mut term, _cfg) = start(&repo, "pty-amend-log");

    let (col, row) = term.wait_for_text("target summary", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains("Amend..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Amend...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the dialog with every field filled", STEP, |s| {
        s.contains("Amend commit")
            && s.contains("Author name")
            && s.contains("Author email")
            && s.contains("Author date")
            && s.contains("target summary")
            && s.contains("target body text")
            && s.contains(&author)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Summary -> Description -> Author name.
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text(" Jr").unwrap();
    term.wait_for("the typed suffix", STEP, |s| {
        s.contains(&format!("{author} Jr"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-s").unwrap();
    term.wait_for("the amend reported", STEP, |s| {
        s.contains("amended") && !s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    assert_eq!(
        repo.git(&["log", "-1", "--format=%an", "HEAD~1"]),
        format!("{author} Jr")
    );
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", "HEAD~1"]),
        "target summary\n\ntarget body text"
    );
    assert_eq!(
        repo.git(&["log", "-1", "--date=raw", "--format=%an|%ad", "HEAD"]),
        top_author_date,
        "the descendant keeps its own author"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    let _ = term.stop();
}

/// Waits for the complete Amend dialog: every field is on screen whatever
/// the entry point.
fn wait_full_dialog(term: &mut Terminal, what: &str) {
    term.wait_for(what, STEP, |s| {
        s.contains("Amend commit")
            && s.contains("Author name")
            && s.contains("Author email")
            && s.contains("Author date")
            && s.contains("Apply")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Applies the open dialog and waits for it to close.
fn apply(term: &mut Terminal) {
    term.send_keys("C-s").unwrap();
    term.wait_for("the amend reported", STEP, |s| {
        s.contains("amended") && !s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The commit under the target: its message and author name.
fn target_state(repo: &FixtureRepo) -> (String, String) {
    (
        repo.git(&["log", "-1", "--format=%B", "HEAD~1"]),
        repo.git(&["log", "-1", "--format=%an", "HEAD~1"]),
    )
}

/// Right-click the description in the detail pane -> Amend description...:
/// the complete dialog opens with the caret in the description; only that
/// edit is applied.
#[test]
fn the_detail_pane_menu_opens_the_full_dialog_on_the_description() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-detail");
    select_target(&mut term);

    let (col, row) = in_detail_pane(&term, "target body text").unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the description's menu", STEP, |s| {
            s.contains("Amend description...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = menu.find("Amend description...").unwrap();
    term.click(c + 1, r).unwrap();
    wait_full_dialog(&mut term, "the complete dialog");
    term.send_text(" and more").unwrap();
    term.wait_for("the text in the description", STEP, |s| {
        s.contains("target body text and more")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    apply(&mut term);

    assert_eq!(
        target_state(&repo),
        (
            "target summary\n\ntarget body text and more".to_string(),
            author
        )
    );
    let _ = term.stop();
}

/// Double click on the summary: the complete dialog, caret in the summary;
/// only the summary changes.
#[test]
fn double_click_on_the_summary_opens_the_full_dialog_on_the_summary() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-dbl-summary");
    select_target(&mut term);

    // The detail pane's copy is the rightmost one.
    let (col, row) = in_detail_pane(&term, "target summary").unwrap();
    term.double_click(col + 1, row).unwrap();
    wait_full_dialog(&mut term, "the complete dialog");
    term.send_text("!").unwrap();
    term.wait_for("the character in the summary", STEP, |s| {
        s.contains("target summary!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    apply(&mut term);

    assert_eq!(
        target_state(&repo),
        ("target summary!\n\ntarget body text".to_string(), author)
    );
    let _ = term.stop();
}

/// Double click on the description: the complete dialog, caret in the
/// description.
#[test]
fn double_click_on_the_description_opens_the_full_dialog_on_the_description() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-dbl-description");
    select_target(&mut term);

    let (col, row) = in_detail_pane(&term, "target body text").unwrap();
    term.double_click(col + 1, row).unwrap();
    wait_full_dialog(&mut term, "the complete dialog");
    term.send_text("!").unwrap();
    term.wait_for("the character in the description", STEP, |s| {
        s.contains("target body text!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    apply(&mut term);

    assert_eq!(
        target_state(&repo),
        ("target summary\n\ntarget body text!".to_string(), author)
    );
    let _ = term.stop();
}

/// Double click on the author: the complete dialog, caret in the author
/// name; the message is untouched.
#[test]
fn double_click_on_the_author_opens_the_full_dialog_on_the_author_name() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-dbl-author");
    select_target(&mut term);

    // The author block: the line that also carries `parent:`.
    let screen = term.screen();
    let (col, row) = screen
        .find_all(&author)
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("parent:"))
        .expect("the author line of the detail pane");
    term.double_click(col + 1, row).unwrap();
    wait_full_dialog(&mut term, "the complete dialog");
    term.send_text(" Jr").unwrap();
    term.wait_for("the suffix in the author name", STEP, |s| {
        s.contains(&format!("{author} Jr"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    apply(&mut term);

    assert_eq!(
        target_state(&repo),
        (
            "target summary\n\ntarget body text".to_string(),
            format!("{author} Jr")
        )
    );
    let _ = term.stop();
}

/// Double click on the authored-on line: the complete dialog, caret in the
/// author date (a stray character is shown there and refused on Apply).
#[test]
fn double_click_on_the_date_opens_the_full_dialog_on_the_date() {
    let (repo, _author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-dbl-date");
    select_target(&mut term);

    let (col, row) = in_detail_pane(&term, "authored on").unwrap();
    term.double_click(col + 1, row).unwrap();
    wait_full_dialog(&mut term, "the complete dialog");
    term.send_text("x").unwrap();
    term.wait_for("the character in the date", STEP, |s| s.contains("+00:00x"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    let _ = term.stop();
}

/// The keyboard twins: `Ctrl+G Shift+E` opens the dialog on the summary,
/// `Ctrl+G Shift+A` on the author name.
#[test]
fn the_chords_open_the_full_dialog_on_the_right_field() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-keys");
    select_target(&mut term);

    term.send_keys("C-g").unwrap();
    term.send_text("E").unwrap();
    wait_full_dialog(&mut term, "the dialog from Ctrl+G E");
    term.send_text("?").unwrap();
    term.wait_for("the character in the summary", STEP, |s| {
        s.contains("target summary?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    // The edited dialog asks before closing: a second Escape discards.
    term.wait_for_text("Discard your changes", STEP).unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Author email", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-g").unwrap();
    term.send_text("A").unwrap();
    wait_full_dialog(&mut term, "the dialog from Ctrl+G A");
    term.send_text("?").unwrap();
    term.wait_for("the character in the author name", STEP, |s| {
        s.contains(&format!("{author}?"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for_text("Discard your changes", STEP).unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Author email", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Nothing was applied.
    assert_eq!(
        target_state(&repo),
        ("target summary\n\ntarget body text".to_string(), author)
    );
    let _ = term.stop();
}

/// The dialog's "Modify History Deeper..." button closes it without applying
/// anything and opens the rewrite workbench (from the amended commit).
#[cfg(feature = "history-rewrite")]
#[test]
fn the_deeper_button_opens_the_rewrite_workbench() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-deeper");
    select_target(&mut term);

    term.send_keys("C-g").unwrap();
    term.send_text("E").unwrap();
    wait_full_dialog(&mut term, "the dialog");
    let screen = term
        .wait_for("the button", STEP, |s| {
            s.contains("Modify History Deeper...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen.find("Modify History Deeper...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the workbench instead of the dialog", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        target_state(&repo),
        ("target summary\n\ntarget body text".to_string(), author)
    );
    let _ = term.stop();
}

/// Its key (`Alt+H`) does the same; with an edited field it asks first
/// and `Esc` confirms.
#[cfg(feature = "history-rewrite")]
#[test]
fn the_deeper_key_asks_before_discarding_edits() {
    let (repo, _author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-amend-deeper-key");
    select_target(&mut term);

    term.send_keys("C-g").unwrap();
    term.send_text("E").unwrap();
    wait_full_dialog(&mut term, "the dialog");
    term.send_text("!").unwrap();
    term.wait_for("the edit", STEP, |s| s.contains("target summary!"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-h").unwrap();
    term.wait_for_text("Discard your changes", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%B", "HEAD~1"]),
        "target summary\n\ntarget body text"
    );
    let _ = term.stop();
}
