//! The shared text-field editing core, end to end through a pty (`xtask::pty`):
//! the real binary, daemon and git. The rules are `pty_ui.rs`'s.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// One commit and one staged edit, so the composer can commit.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start()
}

/// Enter in the commit Description is a new line; the commit is
/// `Ctrl+Enter`, whose Unix spelling (a line feed, `Ctrl+J`) is what a pty
/// sends.
#[test]
fn enter_in_the_description_breaks_the_line_and_ctrl_enter_commits() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-description-newline");

    // `c` in the Log: the WIP row's Summary field (the composer's own).
    term.send_keys("c").expect("press c in the Log");
    term.wait_for("the Summary field focused", STEP, |s| {
        s.contains("Staged Files") && !s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("the subject").expect("type the summary");
    term.send_keys("Enter").expect("on to the Description");
    term.send_text("line one").expect("type the first line");
    term.send_keys("Enter").expect("a new line, not a commit");
    term.send_text("line two").expect("type the second line");

    let screen = term
        .wait_for(
            "two Description lines, the caret on the second",
            STEP,
            |s| {
                let one = s.find("line one");
                let two = s.find("line two_");
                matches!((one, two), (Some((c1, r1)), Some((c2, r2))) if c1 == c2 && r2 == r1 + 1)
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Committing"),
        "Enter in the Description must not commit"
    );
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "initial commit");

    term.send_keys("C-j")
        .expect("Ctrl+Enter's line-feed spelling");
    term.wait_for("the commit made, the Log showing it", STEP, |s| {
        s.contains("the subject") && !s.contains("line two_")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if repo.git(&["log", "-1", "--format=%s"]) == "the subject" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the commit never reached git"
        );
        std::thread::yield_now();
    }
    assert_eq!(
        repo.git(&["log", "-1", "--format=%b"]).trim_end(),
        "line one\nline two"
    );
}

/// The mouse half: a double-click selects the word under it, a drag
/// selects a range, and the next key typed replaces either - in the WIP
/// row's Summary field. After the replace the caret sits mid-text (a
/// reversed char, no `_`), so the last wait only looks for the text.
#[test]
fn double_click_and_drag_select_in_a_field_and_typing_replaces() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-field-mouse-selection");

    term.send_keys("c").expect("press c in the Log");
    term.send_text("alpha beta gamma")
        .expect("type the summary");
    let screen = term
        .wait_for("the typed summary with its caret", STEP, |s| {
            s.contains("alpha beta gamma_")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // Double-click inside "beta".
    let (col, row) = screen.find("alpha beta gamma_").unwrap();
    term.double_click(col + 7, row)
        .expect("double-click the middle word");
    term.send_text("B").expect("type over the word");
    term.wait_for("the word replaced", STEP, |s| s.contains("alpha B gamma"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Drag from the start of "alpha" to the start of "gamma": "alpha B "
    // is selected (the char under the pointer is where the caret stops).
    let screen = term
        .wait_for("the edited summary", STEP, |s| s.contains("alpha B gamma"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("alpha B gamma").unwrap();
    term.drag((col, row), (col + 8, row))
        .expect("drag across the first two words");
    term.send_text("x ").expect("type over the selection");
    term.wait_for("the dragged range replaced", STEP, |s| {
        s.contains("x gamma") && !s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+K` deletes to the end of the line, `Ctrl+U` to its start, and
/// `Ctrl+A` selects all so the next key typed replaces it - here in the
/// WIP row's Summary field, the same core every other field uses.
#[test]
fn ctrl_k_ctrl_u_and_ctrl_a_edit_a_single_line_field() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-field-line-editing");

    term.send_keys("c").expect("press c in the Log");
    term.send_text("hello world").expect("type the summary");
    term.wait_for("the typed summary with its caret", STEP, |s| {
        s.contains("hello world_")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Caret back onto the `w`, then Ctrl+K: "world" goes.
    term.send_keys("Left Left Left Left Left C-k")
        .expect("move left and kill to the end");
    term.wait_for("everything after the caret deleted", STEP, |s| {
        s.contains("hello _") && !s.contains("world")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Ctrl+U from the end: the whole line goes; typing starts afresh.
    term.send_keys("C-u").expect("kill to the start");
    term.send_text("abc").expect("type after the kill");
    term.wait_for("only the new text left", STEP, |s| {
        s.contains("abc_") && !s.contains("hello")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Ctrl+A then a key: the selection is replaced.
    term.send_keys("C-a").expect("select all");
    term.send_text("Z").expect("type over the selection");
    term.wait_for("the selection replaced by the key", STEP, |s| {
        s.contains("Z_") && !s.contains("abc")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
