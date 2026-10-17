//! The content rules of the history rewrite workbench through a pty
//! (`xtask::pty`): remove a folder from all history by keys, preview, apply,
//! undo; a secret value never reaches the screen, not even in the diff of the
//! file it was removed from; a dirty file in the way is listed in the
//! refusal. Same rules as `pty_rewrite.rs`: no coordinates, no sleeps.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `keep.txt` twice around a `secrets/key.pem` that was added in between.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("keep.txt", "one\n", "add keep")
        .unwrap()
        .commit_file("secrets/key.pem", "PRIVATE KEY\n", "add key")
        .unwrap()
        .commit_file("keep.txt", "two\n", "edit keep")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("edit keep") && s.contains("add key")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// The palette row opens the workbench with the rule in place and its first
/// field focused.
fn open_rule(term: &mut Terminal, query: &str, row: &str, first_field: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains(row))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Rewrite history") && s.contains(first_field)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn type_into_first_field(term: &mut Terminal, text: &str) {
    term.send_keys("Enter").unwrap();
    term.send_text(text).unwrap();
    term.send_keys("Enter").unwrap();
}

fn history_of_key(repo: &FixtureRepo) -> String {
    repo.git(&["log", "HEAD", "--format=%s", "--", "secrets/key.pem"])
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Palette, folder typed by keys, Preview, Apply: the folder is gone from
/// history and from the working tree; Ctrl+Z brings both back.
#[test]
fn a_folder_is_removed_from_all_history_by_keys_and_undone() {
    let repo = fixture();
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(&repo, "pty-rewrite-remove-paths");
    open_rule(
        &mut term,
        "remove files from history",
        "Rewrite: Remove Files from History...",
        "Paths or patterns",
    );
    type_into_first_field(&mut term, "secrets");
    term.wait_for("the folder in the field", STEP, |s| {
        s.contains("Paths or patterns") && s.contains("secrets") && s.contains("any depth")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("is removed"),
        "the commit that only added the folder disappears\n{}",
        screen.text()
    );
    assert!(
        history_of_key(&repo).contains("add key"),
        "the preview changes nothing"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(history_of_key(&repo).trim(), "", "the folder left history");
    assert!(
        !repo.root().join("secrets/key.pem").exists(),
        "and the working tree"
    );
    assert!(repo.root().join("keep.txt").exists());
    term.send_keys("C-z").unwrap();
    wait_until("history restored", || {
        repo.git(&["rev-parse", "HEAD"]) == tip
    });
    wait_until("the file is back", || {
        repo.root().join("secrets/key.pem").exists()
    });
    assert!(history_of_key(&repo).contains("add key"));
}

/// A secret value is masked while typed and never shown again: not in the
/// rule, not in the preview, not in the diff of the file it came out of.
#[test]
fn a_secret_value_never_reaches_the_screen_not_even_in_the_diff() {
    let secret = "hunter2secret";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            "config.env",
            &format!("TOKEN={secret}\nMODE=prod\n"),
            "add config",
        )
        .unwrap()
        .commit_file("notes.txt", "n\n", "add notes")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-secret")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("add notes"))
        .unwrap_or_else(|e| panic!("{e}"));
    open_rule(
        &mut term,
        "search and replace in files",
        "Rewrite: Search and Replace in Files...",
        "Look for",
    );
    // "Look for": Right picks the secret values, Down moves to them.
    term.send_keys("Down").unwrap();
    term.send_keys("Right").unwrap();
    term.wait_for("the secret field", STEP, |s| s.contains("Secret values"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !term.screen().contains("Also scan"),
        "scanning is not offered yet"
    );
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text(secret).unwrap();
    let typing = term
        .wait_for("bullets while typing", STEP, |s| s.contains("\u{2022}"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!typing.contains(secret), "{}", typing.text());
    term.send_keys("Tab").unwrap();
    term.send_keys("F5").unwrap();
    let summary = term
        .wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!summary.contains(secret), "{}", summary.text());
    // Commits tab, first row, Enter: the file and its diff.
    term.send_keys("Tab").unwrap();
    term.send_keys("Right").unwrap();
    term.wait_for("the commits tab", STEP, |s| {
        s.contains("changes config.env")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let diff = term
        .wait_for("the diff with hidden lines", STEP, |s| {
            s.contains("removed lines hidden: they held a secret")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!diff.contains(secret), "{}", diff.text());
    assert!(diff.contains("MODE=prod"), "{}", diff.text());
    // The cross closes the diff pane, not the workbench.
    let (xc, xr) = diff.find("Files changed in").expect("the drill title");
    let cross = term
        .screen()
        .find_all(" \u{2715} ")
        .into_iter()
        .find(|&(_, r)| r == xr)
        .unwrap_or((xc, xr));
    term.click(cross.0 + 1, cross.1).expect("click the cross");
    term.wait_for("back on the preview", STEP, |s| {
        !s.contains("Files changed in") && s.contains("Rewrite history")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the result panel", STEP, |s| s.contains("Rewrote"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.screen().contains(secret));
    let rewritten = repo.git(&["show", "HEAD~1:config.env"]);
    assert!(!rewritten.contains(secret), "{rewritten}");
    assert!(rewritten.contains("MODE=prod"));
}

/// A file with local edits that the rewrite would remove is listed in the
/// refusal, with what to do; nothing changed.
#[test]
fn a_dirty_file_in_the_way_is_listed_in_the_refusal() {
    let repo = fixture();
    let tip = repo.git(&["rev-parse", "HEAD"]);
    std::fs::write(
        repo.root().join("secrets/key.pem"),
        "PRIVATE KEY\nlocal edit\n",
    )
    .unwrap();
    let (mut term, _config) = launch(&repo, "pty-rewrite-dirty");
    open_rule(
        &mut term,
        "remove files from history",
        "Rewrite: Remove Files from History...",
        "Paths or patterns",
    );
    type_into_first_field(&mut term, "secrets");
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the refusal", STEP, |s| s.contains("uncommitted edits"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("secrets/key.pem"), "{}", screen.text());
    assert!(screen.contains("discard"), "{}", screen.text());
    assert!(!screen.contains("keep their id"));
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
    assert!(history_of_key(&repo).contains("add key"));
}
