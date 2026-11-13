//! An annotated tag's message is typed in a dialog (several lines), never in
//! the status line: `Create annotated tag here...` asks the name inline in the
//! Log and then opens the dialog; `Annotate <tag>...` on an existing tag opens
//! it directly. Enter breaks the line, `Ctrl+S` applies, an empty message is
//! refused inside the dialog.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate and every wait describes the whole state it is waiting for.
//! Falsified: with the old footer prompt the dialog title never appears and
//! the typed sentinel shows in the last row.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .tag("mine")
        .unwrap()
        .commit("target commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("target commit")
        .start();
    std::mem::forget(config);
    term
}

fn status_row(s: &Screen) -> String {
    s.line(s.rows() - 1)
}

/// Waits for `git tag -l -n2 <tag>` to show `needle`.
fn wait_tag_message(repo: &FixtureRepo, tag: &str, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let shown = repo.git(&["tag", "-l", "-n2", tag]);
        if shown.contains(needle) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "tag {tag}: {shown:?}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn a_new_annotated_tag_asks_its_name_inline_then_its_message_in_a_dialog() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-annotate-dialog-new");
    let (col, row) = term.wait_for_text("target commit", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Create annotated tag here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Create annotated tag here...").unwrap();
    term.click(c + 1, r).unwrap();
    // Step one: the name, inline.
    term.wait_for("the name box", STEP, |s| s.contains("enter tag name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("v7").unwrap();
    term.send_keys("Enter").unwrap();
    // Step two: the dialog.
    term.wait_for("the message dialog", STEP, |s| {
        s.contains("New annotated tag v7 at") && s.contains("Create tag")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // An empty message is refused inside the dialog.
    term.send_keys("C-s").unwrap();
    term.wait_for("the reason inside the dialog", STEP, |s| {
        s.contains("needs a message") && s.contains("New annotated tag v7 at")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Two lines: Enter breaks the line, Ctrl+S applies.
    term.send_text("zq9x first line").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("second line").unwrap();
    let typed = term
        .wait_for("both lines in the dialog", STEP, |s| {
            s.contains("zq9x first line") && s.contains("second line")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !status_row(&typed).contains("zq9x"),
        "nothing is typed in the status line: {}",
        status_row(&typed)
    );
    term.send_keys("C-s").unwrap();
    wait_tag_message(&repo, "v7", "second line");
    assert!(repo
        .git(&["tag", "-l", "-n2", "v7"])
        .contains("zq9x first line"));
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("New annotated tag v7 at")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn annotating_an_existing_tag_opens_the_dialog_and_esc_asks_before_discarding() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-annotate-dialog-existing");
    // The sidebar row of the tag is the leftmost `mine`.
    let sidebar = term
        .wait_for("the tag in the Refs sidebar", STEP, |s| {
            !s.find_all("mine").is_empty()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = sidebar
        .find_all("mine")
        .into_iter()
        .min_by_key(|(c, _)| *c)
        .unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the tag menu", STEP, |s| s.contains("Annotate mine..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Annotate mine...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the message dialog", STEP, |s| {
        s.contains("Annotate tag mine") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_text("typed and not kept").unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for("the discard question", STEP, |s| {
        s.contains("Discard changes?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Keep editing: the dialog and its text are still there.
    term.send_keys("Escape").unwrap();
    term.wait_for("back in the dialog", STEP, |s| {
        s.contains("typed and not kept") && !s.contains("Discard changes?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_tag_message(&repo, "mine", "typed and not kept");
}
