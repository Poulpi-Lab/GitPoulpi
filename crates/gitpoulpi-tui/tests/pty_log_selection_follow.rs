//! The Log selection follows its commit when rows appear above it.
//!
//! The selection is a row index. A stash created elsewhere (another client,
//! a terminal) adds a row above the selected commit; the refresh that brings
//! it used to leave the index alone, so the selection slid onto the commit
//! below. Found through `pty_workspace_search`, whose "reveal this commit"
//! selected a commit and then lost it to a late stash row.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "log-selection-follow";

#[test]
fn a_stash_appearing_above_the_selected_commit_does_not_move_the_selection() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base commit")
        .unwrap()
        .commit_file("b.txt", "b\n", "middle commit")
        .unwrap()
        .commit_file("c.txt", "c\n", "newest commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session(SESSION)
        .config_dir(config.path())
        .size(180, 50)
        .mouse_capture()
        .ready("newest commit")
        .spawn();
    common::wait_for_viewer(config.path(), SESSION);

    let middle = repo.git(&["rev-parse", "HEAD~1"]);
    let middle = middle.trim();
    // Select the middle commit by clicking its row, until the window says so.
    term.act_until(
        "the middle commit selected",
        Duration::from_secs(5),
        STEP,
        |t| {
            if let Some((col, row)) = t.screen().find("middle commit") {
                t.click(col + 1, row)?;
            }
            Ok(())
        },
        |_| {
            common::ui_state(config.path(), SESSION)
                .log
                .and_then(|l| l["selected_commit"].as_str().map(str::to_string))
                .as_deref()
                == Some(middle)
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // A stash made from outside: its row is drawn above the selection.
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();
    repo.git(&["stash", "push", "-m", "parked giraffe work"]);
    term.wait_for("the stash row in the Log", STEP, |s| {
        // (the row's text is cut at the column; the stash message's
        // start, drawn in the Message column, is enough)
        s.contains("STASHES (1)")
            && s.text()
                .lines()
                .any(|l| l.contains("On main") && l.contains("stash@"))
            && s.text().lines().any(|l| l.contains("newest commit"))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // It must stay on the middle commit once the new row is in (the state
    // is read until it has been the middle commit for a moment, so a slide
    // that happens a frame late is still caught).
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        let now = common::ui_state(config.path(), SESSION)
            .log
            .and_then(|l| l["selected_commit"].as_str().map(str::to_string));
        assert_eq!(
            now.as_deref(),
            Some(middle),
            "the selection slid off its commit when the stash row appeared"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = term.stop();
}
