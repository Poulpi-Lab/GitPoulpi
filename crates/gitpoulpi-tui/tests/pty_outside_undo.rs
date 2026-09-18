//! A change made outside GitPoulpi (a shell, an AI agent) appears in the Undo
//! history and Ctrl+Z takes it back, end to end through a pty: the real
//! binary, daemon and `git`, a temporary config dir
//! (the outside-operations design).
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! no sleeps, every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

struct Rig {
    term: Terminal,
    _config: tempfile::TempDir,
}

fn launch(repo: &FixtureRepo, session: &str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the toolbar and the log", |s| {
            s.contains("Undo") && s.contains("initial commit")
        })
        .mouse_capture()
        .start();
    Rig {
        term,
        _config: config,
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    common::git(repo, args).trim().to_string()
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// A branch an agent deletes with plain `git` shows up as an outside row in
/// the Undo history, the status line says so once, and Ctrl+Z brings the
/// branch back at the commit it had.
#[test]
fn a_branch_deleted_outside_is_an_outside_row_and_ctrl_z_restores_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("agent-work")
        .unwrap()
        .commit_file("w.txt", "w\n", "agent work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let tip = git(repo.root(), &["rev-parse", "agent-work"]);
    let mut rig = launch(&repo, "pty-outside-undo");
    // The daemon's first look at the repository is its baseline: wait for it
    // before the outside change, as a user's shell would come later.
    let snapshot = repo.root().join(".git/gitpoulpi/refs-snapshot.json");
    wait_until("the baseline snapshot", || snapshot.exists());

    git(repo.root(), &["branch", "-D", "agent-work"]);

    // The next poll notices; the status line says so, with the key.
    rig.term
        .wait_for("the outside notice", STEP, |s| {
            s.contains("outside change recorded - Ctrl+Z to undo")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // Alt+Z: the Undo history lists it, marked as outside, with its words.
    rig.term.send_keys("M-z").unwrap();
    rig.term
        .wait_for("an outside row in the Undo history", STEP, |s| {
            // The tag column of that very row says it was made outside.
            s.contains("can be undone")
                && s.text()
                    .lines()
                    .any(|l| l.contains("deleted branch agent-work") && l.contains("outside"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for("the panel closed", STEP, |s| !s.contains("can be undone"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // Ctrl+Z takes it back.
    rig.term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z to restore the branch", || {
        !git(repo.root(), &["branch", "--list", "agent-work"]).is_empty()
    });
    assert_eq!(git(repo.root(), &["rev-parse", "agent-work"]), tip);
    rig.term.stop().unwrap();
}
