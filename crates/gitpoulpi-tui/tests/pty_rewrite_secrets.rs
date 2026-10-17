//! The Secrets tab of the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git. A token is planted in a
//! fixture's history; the scan finds it and the screen shows only its
//! redacted preview, never the value; "Remove from history" adds the rule,
//! Preview and Apply rewrite the history, the token is gone from HEAD's
//! history and the working tree, and Undo brings it back.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// 36 characters after the prefix: the shape of a classic GitHub token.
const TOKEN: &str = "ghp_9aB3xK7mQ2vL5nR8tY1cE4wZ6dF0hJsPqUoV";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "add a")
        .unwrap()
        .commit_file(
            "config.env",
            &format!("GITHUB_TOKEN={TOKEN}\nMODE=prod\n"),
            "add config",
        )
        .unwrap()
        .commit_file("notes.txt", "n\n", "add notes")
        .unwrap()
        .build()
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn history_with_token(repo: &FixtureRepo) -> bool {
    repo.git(&["log", "HEAD", "-p", "--format=%s"])
        .contains(TOKEN)
}

/// No screen of this test may hold any part of the value past the redacted
/// preview's own last four characters.
fn assert_no_value(term: &Terminal) {
    let screen = term.screen();
    let text = screen.text();
    assert!(!text.contains(TOKEN), "{text}");
    assert!(!text.contains(&TOKEN[4..30]), "{text}");
}

#[test]
fn a_planted_token_is_found_shown_redacted_removed_and_undone() {
    let repo = fixture();
    assert!(history_with_token(&repo));
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-secrets")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("add notes") && s.contains("add config")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    // Both tab names are part of the wait: the title and the body can be on
    // screen before the tab strip is painted (under load the strip came
    // later), and the click needs the strip's own row.
    let on_tabs_row = |s: &xtask::pty::Screen| {
        let (_, tabs_row) = s.find("Summary")?;
        s.find_all("Secrets")
            .into_iter()
            .find(|&(_, row)| row == tabs_row)
    };
    let screen = term
        .wait_for("the workbench with its tab strip", STEP, |s| {
            s.contains("Rewrite history")
                && s.contains("Add a rule to start")
                && on_tabs_row(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The Secrets tab by a click on its name (on the tabs' row).
    let (tab_col, tabs_row) = on_tabs_row(&screen).expect("the Secrets tab");
    term.click(tab_col + 1, tabs_row).expect("click Secrets");
    let screen = term
        .wait_for("the Secrets tab", STEP, |s| {
            s.contains("rotate it first") && s.contains("Scan for secrets")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("not exhaustive"), "{}", screen.text());

    // Scan with the button.
    let (c, r) = screen.find("Scan for secrets").unwrap();
    term.click(c + 1, r).expect("click Scan for secrets");
    let screen = term
        .wait_for("the finding and its remove button", STEP, |s| {
            s.contains("GitHub token") && s.contains("Remove from history (1)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("config.env"), "{}", screen.text());
    let preview = format!("ghp_****{}", &TOKEN[TOKEN.len() - 4..]);
    assert!(
        screen.contains(&preview),
        "the redacted preview {preview}\n{}",
        screen.text()
    );
    assert_no_value(&term);
    assert!(
        screen.contains("Remove from history (1)"),
        "{}",
        screen.text()
    );

    // Remove: the rule is added; the tab still shows no value.
    let (c, r) = screen.find("Remove from history (1)").unwrap();
    term.click(c + 1, r).expect("click Remove from history");
    term.wait_for("the rule in the list", STEP, |s| {
        s.contains("1 Search and replace") || s.contains("Rule 1 will remove 1 finding")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_no_value(&term);

    // Preview: the usual counts; nothing was changed by it.
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("2 of 3") || screen.contains("2 commits"),
        "the commits holding the token change\n{}",
        screen.text()
    );
    assert_no_value(&term);
    assert!(history_with_token(&repo), "the preview changes nothing");

    // Apply: the token leaves HEAD's history and the working tree.
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_no_value(&term);
    assert!(!history_with_token(&repo), "the token left the history");
    let config_env = std::fs::read_to_string(repo.root().join("config.env")).unwrap();
    assert!(!config_env.contains(TOKEN), "{config_env}");
    assert!(config_env.contains("MODE=prod"), "{config_env}");

    // Undo brings the old history back.
    term.send_keys("C-z").unwrap();
    wait_until("the token is back", || history_with_token(&repo));
}

/// The same path with the keyboard alone: Tab to the preview pane, Right to
/// the tab, `s` scans, `r` removes the ticked findings.
#[test]
fn the_secrets_tab_works_by_keys_alone() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-secrets-stale")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("add notes"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| s.contains("Add a rule to start"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Tab moves the focus to the preview pane; Right walks to the Secrets tab.
    term.send_keys("Tab").unwrap();
    for _ in 0..4 {
        term.send_keys("Right").unwrap();
    }
    term.wait_for("the Secrets tab", STEP, |s| s.contains("Scan for secrets"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("s").unwrap();
    term.wait_for("the finding", STEP, |s| s.contains("GitHub token"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Remove by key, preview, apply.
    term.send_text("r").unwrap();
    term.wait_for("the rule added", STEP, |s| {
        s.contains("Rule 1 will remove 1 finding")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!history_with_token(&repo));
}
