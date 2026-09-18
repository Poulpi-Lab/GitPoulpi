//! A burst of input or of daemon events arriving between two frames must
//! all be applied, in order, with no input lost and no redraw missed: also
//! the suite's one place where external `git` activity is seen live and
//! "many keys in one write" is checked.
//!
//! Same rules as `pty_ui.rs`: temp config dir and fixture repositories,
//! no sleeps beyond polling.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture(commits: usize) -> FixtureRepo {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for i in 0..commits {
        builder = builder.commit(&format!("burst commit {i:02}")).unwrap();
    }
    builder.build()
}

fn launch(repo: &FixtureRepo, session: &str, last: &str) -> (Terminal, tempfile::TempDir) {
    let last = last.to_string();
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded", move |s| s.contains(&last))
        .start()
}

/// Nine keys in one write: the Log selection moves exactly nine commits,
/// which the commit-detail pane's `commit:` line names.
#[test]
fn a_burst_of_keys_in_one_write_is_all_applied_in_order() {
    let repo = fixture(12);
    let (mut term, _c) = launch(&repo, "pty-burst-keys", "burst commit 11");
    let shas = common::git_trimmed(repo.root(), &["log", "--format=%h"]);
    let shas: Vec<&str> = shas.lines().collect();
    term.wait_for("the newest commit selected", STEP, |s| {
        s.contains(&format!("commit: {}", shas[0]))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("jjjjjjjjj").unwrap();
    term.wait_for("the tenth commit selected", STEP, |s| {
        s.contains(&format!("commit: {}", shas[9]))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // And back: k x4 in one write.
    term.send_text("kkkk").unwrap();
    term.wait_for("the sixth commit selected", STEP, |s| {
        s.contains(&format!("commit: {}", shas[5]))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Six commits and a branch made by `git` outside the TUI, in a tight
/// loop, with no key pressed: every one shows up live (the daemon's
/// events are applied in one go or several, never dropped), the tree
/// stays consistent, and the TUI is still responsive afterwards.
#[test]
fn a_burst_of_external_git_changes_is_shown_live_without_input() {
    let repo = fixture(2);
    let (mut term, _c) = launch(&repo, "pty-burst-external", "burst commit 01");
    for i in 0..6 {
        std::fs::write(repo.root().join(format!("ext{i}.txt")), format!("{i}\n")).unwrap();
        common::git_ok(repo.root(), &["add", "-A"]);
        common::git_ok(repo.root(), &["commit", "-qm", &format!("external {i}")]);
    }
    common::git_ok(repo.root(), &["branch", "burst-branch"]);
    term.wait_for("every external commit and the branch", STEP, |s| {
        (0..6).all(|i| s.contains(&format!("external {i}"))) && s.contains("burst-branch")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Still alive: a key still works.
    term.send_keys("?").unwrap();
    term.wait_for_text("Keybindings", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// A dirty tree made externally appears as the WIP row with its count,
/// then a stage by `git` moves it to the staged count: two outcomes in a
/// row, each applied to the visible pane without a keypress.
#[test]
fn external_edits_and_stages_update_the_wip_row_and_status_live() {
    let repo = fixture(2);
    let (mut term, _c) = launch(&repo, "pty-burst-wip", "burst commit 01");
    std::fs::write(repo.root().join("live.txt"), "x\n").unwrap();
    term.wait_for("the WIP row", STEP, |s| s.contains("// WIP"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("one unstaged file", STEP, |s| {
        s.contains("Unstaged Files (1)") && s.contains("Staged Files (0)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::git_ok(repo.root(), &["add", "live.txt"]);
    term.wait_for("the file staged", STEP, |s| {
        s.contains("Unstaged Files (0)") && s.contains("Staged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
