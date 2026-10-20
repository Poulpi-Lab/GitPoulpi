//! The Script rule of the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git. A tiny Lua script that
//! upper-cases the author name is typed into the editor, checked live,
//! tested on commits, previewed, applied and undone; a script with a syntax
//! error shows its problem line and Preview is refused. Same rules as
//! `pty_rewrite.rs`: no coordinates for keys, no sleeps, every wait names the
//! state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// A one-line script (the pty types it key by key; no newline needed).
const UPPERCASE: &str =
    "function rewrite_commit(c, ctx) return { author = { name = string.upper(c.author.name) } } end";

/// `base` by the fixture's identity, then two commits by "Old Name".
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    for message in ["first by old", "second by old"] {
        repo.git(&[
            "-c",
            "user.name=Old Name",
            "-c",
            "user.email=old@example.com",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            message,
        ]);
    }
    repo
}

fn names(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%an"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// The palette row adds a Script rule and opens its editor on the starter
/// template; the daemon's live check of that template is shown.
fn open_script_editor(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("run a script").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Run a Script...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor on the template, checked", STEP, |s| {
        s.contains("Lua script")
            && s.contains("function rewrite_commit(c, ctx)")
            && s.contains("OK: rewrite_commit found")
            && s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Replaces the script with `text`.
fn type_script(term: &mut Terminal, text: &str) {
    term.send_keys("C-a").unwrap();
    term.send_text(text).unwrap();
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Palette, template, the script typed, the live check, Test on selection,
/// Preview, Apply, `git log`, Undo.
#[test]
fn a_script_is_typed_checked_tested_applied_and_undone() {
    let repo = fixture();
    let original = names(&repo);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(&repo, "pty-rewrite-script");
    open_script_editor(&mut term);
    type_script(&mut term, UPPERCASE);
    // The new text has no header: no parameters, and the check answered for
    // this text (not the template's).
    term.wait_for("the new script checked", STEP, |s| {
        s.contains("upper(c.author.name)")
            && s.contains("declares no parameters")
            && s.contains("OK: rewrite_commit found")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Test on the newest commits (nothing is selected): the new name shows.
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the test result", STEP, |s| {
            s.contains("would change") && s.contains("author name: Old Name -> OLD NAME")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("3 commits tested, 3 would change."),
        "{}",
        screen.text()
    );
    assert_eq!(names(&repo), original, "a test writes nothing");
    // Close the editor (the rule stays), Preview, Apply.
    term.send_keys("Esc").unwrap();
    term.wait_for("the workbench again", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("3 of 3"), "{}", screen.text());
    assert_eq!(names(&repo), original, "the preview changes nothing");
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    let shouted: Vec<String> = original.iter().map(|n| n.to_uppercase()).collect();
    wait_until("authors upper-cased", || names(&repo) == shouted);
    assert_eq!(
        repo.git(&["log", "--format=%an", "-1"]).trim(),
        "OLD NAME",
        "git log shows the script's result"
    );
    assert_ne!(repo.git(&["rev-parse", "HEAD"]), tip);
    // Ctrl+Z in the result panel is the journal's Undo.
    term.send_keys("C-z").unwrap();
    wait_until("authors restored", || names(&repo) == original);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}

/// A syntax error shows the problem and its line under the script, the test
/// is refused, and so is Preview: nothing runs, nothing changes.
#[test]
fn a_syntax_error_shows_the_problem_line_and_preview_is_refused() {
    let repo = fixture();
    let original = names(&repo);
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-error");
    open_script_editor(&mut term);
    type_script(&mut term, "function rewrite_commit(c, ctx) return { end");
    let screen = term
        .wait_for("the problem with its line", STEP, |s| {
            s.contains("script:1:") && !s.contains("OK: rewrite_commit found")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Checking...") || screen.contains("script:1:"),
        "{}",
        screen.text()
    );
    // Test on selection is refused with the reason.
    term.send_keys("F5").unwrap();
    term.wait_for("the test refused", STEP, |s| s.contains("Fix the problems"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Close the editor and Preview: the daemon refuses and the panel names
    // the problem line.
    term.send_keys("Esc").unwrap();
    term.wait_for("the workbench again", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the refusal", STEP, |s| {
            s.contains("a script failed") && s.contains("Fix the script")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("script:1:"), "{}", screen.text());
    assert!(
        !screen.contains("keep their id"),
        "no preview was made\n{}",
        screen.text()
    );
    assert_eq!(names(&repo), original);
}

/// By mouse: the Test button runs the script, the cross closes the editor.
#[test]
fn the_test_button_and_the_cross_work_by_mouse() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-mouse");
    open_script_editor(&mut term);
    let (c, r) = term
        .screen()
        .find("Test on selection")
        .expect("the Test button");
    term.click(c + 2, r).expect("click Test on selection");
    term.wait_for("the template's result", STEP, |s| {
        s.contains("would change") && s.contains("author name: ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The editor's cross (the inner panel's, right under the workbench's).
    let crosses = term.screen().find_all(" \u{2715} ");
    assert!(crosses.len() >= 2, "{crosses:?}");
    let (xc, xr) = crosses.into_iter().max_by_key(|&(_, row)| row).unwrap();
    term.click(xc + 1, xr).expect("click the editor's cross");
    term.wait_for("only the workbench is left", STEP, |s| {
        s.contains("Rewrite history") && !s.contains("Test on selection")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
