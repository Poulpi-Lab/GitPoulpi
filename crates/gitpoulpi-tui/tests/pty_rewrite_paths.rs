//! Extracting a folder into a new clone through a pty (`xtask::pty`), S20:
//! the palette opens the path rule, the kind is Extract a folder, Preview
//! asks how (nothing is previewed before the answer), the clone answer opens
//! the folder picker, the Summary names the new repository, Apply makes it
//! and a new tab opens on it with only that history. The original is not
//! touched. Same rules as `pty_rewrite.rs`: no coordinates, no sleeps.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// `add lib`, `other work`, `edit lib`.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("lib/a.txt", "a1\n", "add lib")
        .unwrap()
        .commit_file("other/x.txt", "x\n", "other work")
        .unwrap()
        .commit_file("lib/a.txt", "a2\n", "edit lib")
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

fn refs(repo: &FixtureRepo) -> String {
    repo.git(&["for-each-ref", "--format=%(refname) %(objectname)"])
}

#[test]
fn extracting_a_folder_into_a_clone_opens_a_tab_with_only_that_history() {
    let repo = fixture();
    let before = refs(&repo);
    let out = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-extract-clone")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("edit lib") && s.contains("other work")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The palette row opens the workbench with a path rule in place.
    term.send_keys("C-p").unwrap();
    term.send_text("move or rename paths").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Rewrite: Move or Rename Paths...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Rename a path")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The Move dropdown: Extract a folder as the root.
    term.send_keys("Enter").unwrap();
    let menu = term
        .wait_for("the kinds", STEP, |s| {
            s.contains("Extract a folder as the root")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Extract a folder as the root").unwrap();
    term.click(c + 1, r).expect("click the kind");
    term.wait_for("the Folder field", STEP, |s| s.contains("Folder"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("lib").unwrap();
    term.send_keys("Enter").unwrap();

    // Preview asks first; nothing is previewed before the answer.
    term.send_keys("F5").unwrap();
    // The whole post-condition is in the wait: the dialog paints top down,
    // so its title and first line can be on screen before the highlighted
    // way's note and the answers (testing.md, waiting pattern 1).
    let ask = term
        .wait_for(
            "the extract question with its note and answers",
            STEP,
            |s| {
                s.contains("How should the folder be extracted?")
                    && s.contains("The clone opens in a new tab.")
                    && s.contains("Into a new clone")
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        ask.contains("The clone opens in a new tab."),
        "the highlighted way says what it does\n{}",
        ask.text()
    );
    assert_eq!(refs(&repo), before);
    let (c, r) = ask.find("Into a new clone").unwrap();
    term.click(c + 1, r).expect("click the clone answer");

    // The folder picker: type the folder, choose it.
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("Where should the new repository go?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    let path = out.path().display().to_string().replace('\\', "/");
    term.send_text(&path).unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("C-s").unwrap();

    // The preview is the new repository's history, and says where it goes.
    let summary = term
        .wait_for("the summary", STEP, |s| s.contains("New repository:"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        summary.contains("This repository is not changed."),
        "{}",
        summary.text()
    );
    assert_eq!(refs(&repo), before, "the preview changes nothing");

    // Apply: the new repository is made and opens in a new tab.
    term.send_keys("C-s").unwrap();
    let target = out.path().join("lib");
    wait_until("the new repository exists", || {
        target.join(".git").exists() && target.join("a.txt").exists()
    });
    term.wait_for("the new tab shows only that history", STEP, |s| {
        s.contains(" lib x")
            && s.contains("edit lib")
            && s.contains("add lib")
            && !s.contains("other work")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git_at(&target, &["log", "--reverse", "--format=%s"]),
        "add lib\nedit lib"
    );
    assert_eq!(refs(&repo), before, "the original is untouched");
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}
