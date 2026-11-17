//! The "Skip hooks" checkbox of the commit composer has its own row under the
//! Amend row, with its full label at every usual width; on a very narrow pane
//! it shrinks (a short label, then the box alone) instead of vanishing and
//! stays clickable. Driven through a pty, resizing the terminal.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// The row of the Amend checkbox, and the row right under it.
fn rows(screen: &Screen) -> Option<(u16, String, String)> {
    let (_, row) = screen.find("Amend")?;
    Some((row, screen.line(row), screen.line(row + 1)))
}

#[test]
fn skip_hooks_has_its_own_row_and_shrinks_only_when_very_narrow() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term: Terminal = common::launch(repo.root())
        .session("pty-skip-hooks-narrow")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded, WIP row and all", STARTUP, |s| {
        s.contains("initial commit") && s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the full label on its own row", STEP, |s| {
        rows(s).is_some_and(|(_, amend, skip)| {
            skip.contains("[ ] Skip hooks") && !amend.contains("Skip hooks")
        })
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The usual widths keep the full label, on its own row.
    for cols in [110u16, 90, 75, 60] {
        term.resize(cols, 40).expect("resize");
        let screen = term
            .wait_for_stable(Duration::from_millis(400), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let Some((_, amend, skip)) = rows(&screen) else {
            panic!(
                "no commit options row at {cols} columns:\n{}",
                screen.text()
            );
        };
        assert!(
            skip.contains("[ ] Skip hooks") && !amend.contains("hooks"),
            "the full label left its own row at {cols} columns: {skip:?}\n{}",
            screen.text()
        );
    }

    // Very narrow: it may shorten but the box stays, and clicking it ticks it.
    term.resize(40, 40).expect("resize");
    let screen = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (row, _, skip) = rows(&screen).expect("options row");
    assert!(
        skip.contains("[ ]"),
        "the Skip hooks checkbox vanished at 40 columns:\n{}",
        screen.text()
    );
    let col = skip
        .find("[ ]")
        .map(|i| skip[..i].chars().count() as u16 + 1)
        .unwrap();
    term.click(col, row + 1).expect("click the box");
    term.wait_for("the box ticked", STEP, |s| {
        rows(s).is_some_and(|(_, _, skip)| skip.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
