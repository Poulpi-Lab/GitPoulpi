//! A repository that was a fork and is not one any more keeps its
//! `refs/remotes/<name>/*` while the remote's config is gone: the REMOTE
//! section still shows the name. Editing or removing it used to fail with
//! "no such remote". Now the row says so, its menu offers "Remove leftover
//! branches" and "Add <name> as remote", and the removal is one undo step.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);
const NOTE: &str = "(not configured)";
/// A name nothing else on screen contains (the title bar shows a commit
/// hash, so nothing hex-looking).
const LEFTOVER: &str = "zeta-fork";

/// A commit and two remote-tracking branches of a remote with no config.
fn leftover_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("leftover fixture")
        .unwrap()
        .build();
    let head = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&[
        "update-ref",
        &format!("refs/remotes/{LEFTOVER}/main"),
        &head,
    ]);
    repo.git(&[
        "update-ref",
        &format!("refs/remotes/{LEFTOVER}/topic"),
        &head,
    ]);
    repo
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .spawn();
    term.wait_for("the leftover row", STARTUP, |s| {
        s.contains(LEFTOVER) && s.contains(NOTE)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn refs_of_leftover(repo: &FixtureRepo) -> String {
    repo.git(&[
        "for-each-ref",
        "--format=%(refname)",
        &format!("refs/remotes/{LEFTOVER}"),
    ])
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn open_menu(term: &mut Terminal) -> xtask::pty::Screen {
    let (col, row) = term.wait_for_text(LEFTOVER, STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the leftover");
    term.wait_for("the leftover's menu", STEP, |s| {
        s.contains("Remove leftover branches...")
            && s.contains(&format!("Add {LEFTOVER} as remote"))
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// The leftover's menu has no Edit, Fetch or Set as default (they all end
/// in "no such remote"); Remove leftover branches asks, deletes the
/// branches, and Ctrl+Z brings them back.
#[test]
fn a_leftover_remote_is_removed_with_its_branches_and_undone() {
    let repo = leftover_repo();
    let (mut term, _config) = launch(repo.root(), "pty-remote-leftover");
    assert_eq!(refs_of_leftover(&repo).lines().count(), 2);

    let menu = open_menu(&mut term);
    assert!(
        !menu.contains(&format!("Edit {LEFTOVER}...")),
        "{}",
        menu.text()
    );
    assert!(!menu.contains("Set as default remote"), "{}", menu.text());
    assert!(
        !menu.contains(&format!("Fetch {LEFTOVER}")),
        "{}",
        menu.text()
    );
    let (c, r) = menu.find("Remove leftover branches...").unwrap();
    term.click(c + 1, r)
        .expect("click Remove leftover branches");
    term.wait_for("the confirmation", STEP, |s| {
        s.contains(&format!("Remove the leftover branches of '{LEFTOVER}'?"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();

    wait_until("the leftover branches to go", || {
        refs_of_leftover(&repo).is_empty()
    });
    term.wait_for("the row to disappear", STEP, |s| {
        !s.contains(LEFTOVER) && !s.contains(NOTE) && !s.contains("operation failed")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-z").unwrap();
    wait_until("undo to bring the branches back", || {
        refs_of_leftover(&repo).lines().count() == 2
    });
    term.wait_for("the row to come back", STEP, |s| {
        s.contains(LEFTOVER) && s.contains(NOTE)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// "Add <name> as remote..." opens the Add popup with the name filled in;
/// typing an address adds a real remote and the note goes away.
#[test]
fn a_leftover_can_be_added_back_as_a_remote() {
    let repo = leftover_repo();
    let (mut term, _config) = launch(repo.root(), "pty-remote-leftover-add");

    let menu = open_menu(&mut term);
    let (c, r) = menu.find(&format!("Add {LEFTOVER} as remote")).unwrap();
    term.click(c + 1, r).expect("click Add as remote");
    term.wait_for("the Add remote popup, name filled in", STEP, |s| {
        s.contains("Add remote") && s.contains("Push URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("https://example.invalid/me/zeta.git")
        .unwrap();
    // Enter moves on from the address to the push address, then submits.
    term.send_keys("Enter").unwrap();
    term.send_keys("Enter").unwrap();

    wait_until("the remote to be configured", || {
        repo.git(&["remote"]).contains(LEFTOVER)
    });
    assert_eq!(
        repo.git(&["config", "--get", &format!("remote.{LEFTOVER}.url")]),
        "https://example.invalid/me/zeta.git"
    );
    term.wait_for("the note to go", STEP, |s| {
        s.contains(LEFTOVER) && !s.contains(NOTE)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
