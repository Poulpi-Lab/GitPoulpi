//! The `.mailmap` slice (S16) end to end through a pty (`xtask::pty`): the
//! real binary, daemon and git.
//!
//! - One person under two names and emails: the Identities tab lists both
//!   pairs, "Merge into..." writes the `.mailmap` (nothing committed), the
//!   Log shows the canonical name, and Undo puts the file and the old names
//!   back.
//! - An author rule on commits that are already pushed offers the `.mailmap`
//!   instead, and the button opens the Identities tab with the matching
//!   identities ticked.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn commit_as(repo: &FixtureRepo, who: &str, file: &str, message: &str) {
    std::fs::write(repo.root().join(file), format!("{message}\n")).unwrap();
    repo.git(&["add", file]);
    repo.git(&["commit", "-q", "-m", message, &format!("--author={who}")]);
}

/// `base` by the fixture's own identity, then the same person twice under
/// two names and emails.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    commit_as(
        &repo,
        "Alec Old <alec@old.example.com>",
        "b.txt",
        "work by the old name",
    );
    commit_as(
        &repo,
        "Alex Doe <alex@example.org>",
        "c.txt",
        "work by the new name",
    );
    commit_as(
        &repo,
        "Alex Doe <alex@example.org>",
        "d.txt",
        "more by the new name",
    );
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(204, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("work by the old name") && s.contains("work by the new name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn merging_two_identities_shows_one_name_in_the_log_and_undo_brings_the_other_back() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-mailmap-merge");
    term.wait_for("the Log shows the stored author names", STEP, |s| {
        s.contains("Alec Old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let tip = repo.git(&["rev-parse", "HEAD"]);

    // Palette: the workbench opens on the Identities tab and lists the pairs.
    term.send_keys("C-p").unwrap();
    term.send_text("merge identities").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Merge Identities (.mailmap)...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let list = term
        .wait_for("the identities", STEP, |s| {
            s.contains("Alec Old <alec@old.example.com>")
                && s.contains("Alex Doe <alex@example.org>")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(list.contains("0 of 3 selected"), "{}", list.text());
    assert!(list.contains("There is no .mailmap yet"), "{}", list.text());

    // Most commits first: the fixture's own identity (it committed all
    // three), then "Alex Doe", then "Alec Old". Tick the last two.
    term.send_keys("Down").unwrap();
    term.send_keys("Space").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Space").unwrap();
    term.wait_for("two ticked", STEP, |s| s.contains("2 of 3 selected"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Merge into...: the dialog asks which one stays; nothing is written
    // until it is answered.
    term.send_keys("m").unwrap();
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Which of the 2 selected identities")
            && s.contains("1  Alex Doe <alex@example.org>")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo.root().join(".mailmap").exists());
    term.send_keys("Enter").unwrap();
    term.wait_for("the merge done", STEP, |s| {
        s.contains("Added 1 line to .mailmap")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let written = std::fs::read_to_string(repo.root().join(".mailmap")).unwrap();
    assert_eq!(
        written,
        "Alex Doe <alex@example.org> Alec Old <alec@old.example.com>\n"
    );
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        tip,
        "nothing was committed or rewritten"
    );
    assert!(repo.git(&["status", "--porcelain"]).contains(".mailmap"));
    // The list now says which pair is mapped.
    term.wait_for("the mapped pair", STEP, |s| {
        s.contains("mapped to Alex Doe <alex@example.org>")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Back on the Log: the old name is gone from it.
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log shows the canonical name", STEP, |s| {
        !s.contains("Alec Old") && s.contains("work by the old name")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Undo takes the file away and the stored name comes back.
    term.send_keys("C-z").unwrap();
    wait_until("the .mailmap is gone", || {
        !repo.root().join(".mailmap").exists()
    });
    term.wait_for("the Log shows the stored name again", STEP, |s| {
        s.contains("Alec Old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}

/// `base` on `main`, pushed; a pushed `feature` with two commits by
/// old@example.com.
fn pushed_fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    commit_as(&repo, "Old Name <old@example.com>", "b.txt", "first by old");
    commit_as(
        &repo,
        "Old Name <old@example.com>",
        "c.txt",
        "second by old",
    );
    repo.git(&["push", "-q", "origin", "feature"]);
    repo
}

#[test]
fn an_author_rule_on_pushed_commits_offers_the_mailmap_and_opens_the_tab_with_the_matches() {
    let repo = pushed_fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-mailmap-offer")
        .config_dir(config.path())
        .size(204, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("first by old") && s.contains("second by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // An author rule by keys: old@example.com becomes Newbie <new@example.org>.
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| s.contains("Add a rule to start"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| s.contains("Replace an author"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Email is") && s.contains("New email")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..2 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.send_text("Newbie").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("new@example.org").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the new identity in the fields", STEP, |s| {
        s.contains("Newbie") && s.contains("new@example.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("F5").unwrap();
    term.wait_for("the summary with the offer", STEP, |s| {
        s.contains("keep their id")
            && s.contains("Use .mailmap instead?")
            && s.contains("A .mailmap fixes the display without rewriting.")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The button, by mouse: the Identities tab with old@example.com ticked.
    // The sentence above the button says "Use .mailmap instead?" too: the
    // button is the lower of the two.
    // Retried: a button located on a torn frame can take the click nowhere.
    let tab = common::retry_until(
        &mut term,
        "the Identities tab with the match ticked",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t
                .screen()
                .find_all("Use .mailmap")
                .into_iter()
                .max_by_key(|(_, r)| *r)
            {
                t.click(c + 1, r).expect("click Use .mailmap");
            }
        },
        |s| s.contains("Old Name <old@example.com>") && s.contains("1 of 2 selected"),
    );
    assert!(!repo.root().join(".mailmap").exists(), "{}", tab.text());
    // Merge into... offers the identity the rule would set, though no
    // commit has it yet.
    term.send_keys("m").unwrap();
    term.wait_for("the dialog with the rule's identity", STEP, |s| {
        s.contains("Which of the 2 selected identities") && s.contains("Newbie <new@example.org>")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    // Esc closed the dialog only; the history is as it was.
    term.wait_for("the dialog is closed", STEP, |s| {
        !s.contains("Which of the 2 selected identities")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo.root().join(".mailmap").exists());
}
