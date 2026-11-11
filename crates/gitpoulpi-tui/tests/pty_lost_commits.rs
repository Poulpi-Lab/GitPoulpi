//! The Lost work view, and the "can be undone" reassurance after a
//! destructive action, end to end through a pty.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

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

/// A branch deleted with plain git leaves its commit in the reflog only.
/// "Recover lost commits" lists it and "Create branch here" brings it back
/// under a name of the user's choice (the default is offered).
#[test]
fn a_commit_of_a_deleted_branch_is_listed_and_a_branch_recovers_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("doomed")
        .unwrap()
        .checkout("doomed")
        .unwrap()
        .commit_file("d.txt", "d\n", "doomed work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let lost = git(repo.root(), &["rev-parse", "doomed"]);
    git(repo.root(), &["branch", "-D", "doomed"]);

    let mut rig = launch(&repo, "pty-lost-commits");
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("recover lost").unwrap();
    rig.term
        .wait_for("the palette row", STEP, |s| s.contains("Recover Lost"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    rig.term
        .wait_for("the lost commit listed", STEP, |s| {
            s.contains("Lost work") && s.contains(&lost[..7]) && s.contains("doomed work")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    rig.term.send_keys("b").unwrap();
    rig.term
        .wait_for("the name prompt with a default", STEP, |s| {
            s.contains("Name of the new branch") && s.contains(&format!("restored-{}", &lost[..7]))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    let name = format!("restored-{}", &lost[..7]);
    wait_until("the recovered branch", || {
        git(repo.root(), &["branch", "--list", &name]).contains(&name)
    });
    assert_eq!(git(repo.root(), &["rev-parse", &name]), lost);
    rig.term.stop().unwrap();
}

/// After a discard the status line says it can be undone, with the bound
/// key, and that key brings the file back.
#[test]
fn a_discard_says_the_key_that_undoes_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\nchanged\n")
        .unwrap()
        .build();
    let mut rig = launch(&repo, "pty-discard-hint");
    rig.term.send_keys("C-g x").unwrap();
    rig.term
        .wait_for("the discard confirmation", STEP, |s| s.contains("Cancel"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("y").unwrap();
    rig.term
        .wait_for("the reassurance", STEP, |s| {
            s.contains("discard done - Ctrl+Z to undo")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "one\n"
    );
    rig.term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z to bring the change back", || {
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap() == "one\nchanged\n"
    });
    rig.term.stop().unwrap();
}

/// A lost commit cannot be shown in the Log, so the view has no such
/// button. It can be tagged (`t`), its changes applied to the working tree
/// (`w`) and kept as a stash (`s`), each leaving the repository as the
/// action says.
#[test]
fn a_lost_commit_is_tagged_applied_and_stashed() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("doomed")
        .unwrap()
        .checkout("doomed")
        .unwrap()
        .commit_file("d.txt", "d\n", "doomed work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    let lost = git(repo.root(), &["rev-parse", "doomed"]);
    git(repo.root(), &["branch", "-D", "doomed"]);

    let mut rig = launch(&repo, "pty-lost-restore");
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("recover lost").unwrap();
    rig.term
        .wait_for("the palette row", STEP, |s| s.contains("Recover Lost"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    rig.term
        .wait_for("the lost commit and its buttons", STEP, |s| {
            s.contains("doomed work")
                && s.contains("Create tag here")
                && s.contains("Apply to working tree")
                && s.contains("Save as")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert!(
        !rig.term.text().contains("Show in Log"),
        "a lost commit is not in the Log:\n{}",
        rig.term.text()
    );

    // Tag: the default name is offered.
    let name = format!("restored-{}", &lost[..7]);
    rig.term.send_keys("t").unwrap();
    rig.term
        .wait_for("the tag name prompt", STEP, |s| {
            s.contains("Name of the new tag") && s.contains(&name)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    wait_until("the tag", || {
        git(repo.root(), &["tag", "--list", &name]).contains(&name)
    });
    assert_eq!(
        git(repo.root(), &["rev-parse", &format!("{name}^{{commit}}")]),
        lost
    );

    rig.term
        .wait_for("the tag reported", STEP, |s| s.contains("created tag"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // Apply: the file appears in the working tree, nothing is staged.
    rig.term.send_keys("w").unwrap();
    wait_until("the applied file", || repo.root().join("d.txt").exists());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("d.txt")).unwrap(),
        "d\n"
    );
    assert_eq!(git(repo.root(), &["diff", "--cached", "--name-only"]), "");

    // Apply again: the file is in the way, nothing changes and the stash is offered.
    rig.term
        .wait_for("the apply reported", STEP, |s| {
            s.contains("applied the changes")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("w").unwrap();
    rig.term
        .wait_for("the stash offer", STEP, |s| {
            s.contains("could not be applied") && s.contains("Nothing was changed")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    wait_until("the stash", || {
        !git(repo.root(), &["stash", "list"]).is_empty()
    });
    assert_eq!(
        git(repo.root(), &["stash", "show", "--name-only", "stash@{0}"]),
        "d.txt"
    );
    rig.term.stop().unwrap();
}
