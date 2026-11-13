//! Stash with a message is a dialog, never text typed into the status line:
//! `Ctrl+G m` (and the palette) open it, the message is typed in its field, a
//! button or Enter stashes, the cross, Cancel and Esc close it.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate and every wait describes the whole state it is waiting for.
//! Falsified: with the footer prompt back in place of the dialog the typed
//! sentinel shows in the last row and the title is never drawn.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// What is typed into the dialog: it must be on screen, but not in the
/// status line (the last row).
const SENTINEL: &str = "zq9x stash note";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(160, 50)
        .no_exit_pause()
        .ready_when("the fixture loaded with its WIP row", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start();
    std::mem::forget(config);
    term
}

fn status_row(s: &Screen) -> String {
    s.line(s.rows() - 1)
}

fn open_dialog(term: &mut Terminal) -> Screen {
    term.send_keys("C-g").unwrap();
    term.send_keys("m").unwrap();
    term.wait_for("the stash dialog", STEP, |s| {
        s.contains("Stash changes") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

fn stash_list(repo: &FixtureRepo) -> String {
    repo.git(&["stash", "list"])
}

fn wait_for_stash(repo: &FixtureRepo, message: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stash_list(repo).contains(message) {
        assert!(
            std::time::Instant::now() < deadline,
            "no stash named {message:?}: {}",
            stash_list(repo)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn a_typed_message_lives_in_the_dialog_and_enter_stashes_with_it() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-stash-dialog-enter");
    open_dialog(&mut term);
    term.send_text(SENTINEL).unwrap();
    let typed = term
        .wait_for("the message in the dialog", STEP, |s| s.contains(SENTINEL))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !status_row(&typed).contains("zq9x"),
        "nothing is typed in the status line: {}",
        status_row(&typed)
    );
    term.send_keys("Enter").unwrap();
    wait_for_stash(&repo, SENTINEL);
    term.wait_for("the dialog closed", STEP, |s| !s.contains("Stash changes"))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_stash_button_stashes_and_an_empty_message_is_allowed() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-stash-dialog-button");
    let open = open_dialog(&mut term);
    // The primary button is the last one on the dialog's button row.
    let (_, row) = open.find("Cancel").unwrap();
    let (col, _) = open
        .find_all("Stash")
        .into_iter()
        .filter(|(_, r)| *r == row)
        .max_by_key(|(c, _)| *c)
        .expect("the primary button");
    term.click(col + 1, row).unwrap();
    wait_for_stash(&repo, "WIP on");
}

#[test]
fn the_cross_cancel_and_esc_close_without_stashing() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-stash-dialog-close");

    // The cross, on the title's border row.
    let open = open_dialog(&mut term);
    let (_, title_row) = open.find("Stash changes").unwrap();
    let (cross, _) = open
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r == title_row)
        .expect("the cross on the dialog's top border");
    term.click(cross + 1, title_row).unwrap();
    term.wait_for("the dialog closed by its cross", STEP, |s| {
        !s.contains("Stash changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Cancel.
    let open = open_dialog(&mut term);
    let (col, row) = open.find("Cancel").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the dialog closed by Cancel", STEP, |s| {
        !s.contains("Stash changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Esc.
    open_dialog(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed by Esc", STEP, |s| {
        !s.contains("Stash changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(stash_list(&repo).trim().is_empty(), "nothing was stashed");
}
