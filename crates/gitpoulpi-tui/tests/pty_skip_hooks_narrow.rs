//! The "Skip hooks" checkbox of the commit composer shrinks with the right
//! pane instead of vanishing: the full label, a short one, then the box alone,
//! which stays clickable. Driven through a pty, resizing the terminal.

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

/// The commit-options row (the one with the Amend checkbox).
fn options_row(screen: &Screen) -> Option<(u16, String)> {
    let (_, row) = screen.find("Amend")?;
    Some((row, screen.line(row)))
}

#[test]
fn skip_hooks_shrinks_to_the_box_and_stays_clickable() {
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
    term.wait_for("the full label beside Amend", STEP, |s| {
        options_row(s).is_some_and(|(_, line)| line.contains("Skip hooks"))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let mut seen_box_only = false;
    for cols in [110u16, 90, 75, 60, 50] {
        term.resize(cols, 40).expect("resize");
        let screen = term
            .wait_for_stable(Duration::from_millis(400), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        let Some((_, line)) = options_row(&screen) else {
            panic!(
                "no commit options row at {cols} columns:\n{}",
                screen.text()
            );
        };
        assert!(
            line.matches("[ ]").count() >= 2,
            "the Skip hooks checkbox vanished at {cols} columns: {line:?}\n{}",
            screen.text()
        );
        if !line.contains("hooks") {
            seen_box_only = true;
        }
    }
    assert!(seen_box_only, "the checkbox never shrank to its box");

    // The box alone is still the checkbox: clicking it ticks it.
    let screen = term.screen();
    let (row, line) = options_row(&screen).expect("options row");
    let first = line.find("[ ]").expect("amend box");
    let second = line[first + 3..].find("[ ]").expect("skip hooks box") + first + 3;
    let col = line[..second].chars().count() as u16 + 1;
    term.click(col, row).expect("click the box");
    term.wait_for("the box ticked", STEP, |s| {
        options_row(s).is_some_and(|(_, line)| line.matches("[x]").count() == 1)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
