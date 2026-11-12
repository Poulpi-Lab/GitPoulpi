//! Settings > Repository > Merge options, through a pty: with "Squash" on, a
//! merge from a branch's menu stages its changes and makes no merge commit
//! (HEAD stays, nothing in progress), the commit panel opens with git's
//! squash message, and committing it makes one single-parent commit. Without
//! the setting the same menu row makes the merge commit.
//!
//! No hard-coded coordinates: every position is found on the screen.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// `main` and `feature` have each a commit the other lacks, so a merge makes
/// a merge commit.
fn diverged() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pty-squash-base")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("f.txt", "f\n", "pty-squash-feature")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("m.txt", "m\n", "pty-squash-main")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the fixture", |s| {
            s.contains("pty-squash-feature") && s.contains("pty-squash-main")
        })
        .mouse_capture()
        .start()
}

fn open_page(term: &mut Terminal, title: &str) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, title);
}

fn close_settings(term: &mut Terminal) {
    common::retry_until(
        term,
        "Settings closed",
        6,
        Duration::from_secs(3),
        |t| t.send_keys("Escape").unwrap(),
        |s| !s.contains("Repo-specific settings"),
    );
}

fn wait_for(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Merge `feature` into the checked-out branch from the pill's menu.
fn merge_feature_from_its_pill(term: &mut Terminal) {
    let (_, row) = term
        .wait_for_text("pty-squash-feature", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let col = screen
        .find_all("feature")
        .into_iter()
        .find(|(_, r)| *r == row)
        .expect("the feature pill")
        .0;
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the pill menu", STEP, |s| {
            s.contains("Merge feature into main")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = menu.find("Merge feature into main").unwrap();
    term.click(c + 1, r).unwrap();
}

#[test]
fn squash_stages_the_changes_without_a_merge_commit_and_the_panel_opens_with_the_squash_message() {
    let repo = diverged();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _config) = launch(repo.root(), "pty-repo-merge-squash");
    open_page(&mut term, "Merge options");
    let screen = term
        .wait_for("the Merge options page", Duration::from_secs(20), |s| {
            s.contains("Squash") && s.contains("finish it yourself")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen.find("Squash").unwrap();
    assert!(screen.line(r).contains("[ ]"), "off by default");
    term.click(c, r).unwrap();
    term.wait_for("ticked and saved", STEP, |s| {
        s.line(r).contains("[x]")
            && std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml"))
                .unwrap_or_default()
                .contains("squash_merge = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    close_settings(&mut term);

    merge_feature_from_its_pill(&mut term);
    wait_for(&term, "f.txt staged", || {
        repo.git(&["diff", "--cached", "--name-only"]).trim() == "f.txt"
    });
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head, "no commit was made");
    assert!(
        !repo.root().join(".git/MERGE_HEAD").exists(),
        "nothing is in progress"
    );
    term.wait_for("the squash message in the composer", STEP, |s| {
        s.contains("Squashed commit of the following")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = term
        .wait_for_text("Commit changes to 1 file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Commit");
    wait_for(&term, "the squash commit", || {
        repo.git(&["log", "-1", "--format=%s"])
            .starts_with("Squashed commit of the following")
    });
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "HEAD"])
            .split_whitespace()
            .count(),
        2,
        "one parent: a squash, not a merge"
    );
    let _ = term.stop();
}

/// The same menu row with the setting off makes the merge commit.
#[test]
fn without_squash_the_menu_row_makes_the_merge_commit() {
    let repo = diverged();
    let (mut term, _config) = launch(repo.root(), "pty-repo-merge-plain");
    merge_feature_from_its_pill(&mut term);
    wait_for(&term, "the merge commit", || {
        repo.git(&["rev-list", "--parents", "-n1", "HEAD"])
            .split_whitespace()
            .count()
            == 3
    });
    assert_eq!(repo.git(&["diff", "--cached", "--name-only"]).trim(), "");
    let _ = term.stop();
}
