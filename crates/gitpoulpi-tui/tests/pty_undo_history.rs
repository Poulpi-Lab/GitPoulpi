//! The Undo history panel, end to end through a pty: the real binary, the
//! real daemon, real `git`, a temporary config dir. The point of the panel
//! is that an old mistake can be taken back without taking back what came
//! after it, so every test checks git, not the screen, for the result.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! no sleeps, every wait names the state it waits for.

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

fn head_subject(repo: &Path) -> String {
    git(repo, &["log", "-1", "--format=%s"])
}

/// Commits a new staged file through the UI, so the daemon's journal
/// records it like a user's: `c`, the subject, Enter, `Ctrl+Enter`.
fn commit_via_ui(rig: &mut Rig, repo: &FixtureRepo, subject: &str) {
    std::fs::write(repo.root().join(format!("{subject}.txt")), subject).unwrap();
    git(repo.root(), &["add", "-A"]);
    rig.term
        .wait_for("the staged file listed", STEP, |s| {
            s.contains(&format!("{subject}.txt"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("c").unwrap();
    rig.term
        .wait_for("the Summary field focused", STEP, |s| {
            s.contains("Staged Files") && s.text().contains("\u{2502}_")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_text(subject).unwrap();
    rig.term.send_keys("Enter").unwrap();
    rig.term.send_keys("C-j").unwrap();
    wait_until("the commit to reach git", || {
        head_subject(repo.root()) == subject
    });
}

/// Selects the `feature` branch in the Refs pane and deletes it through
/// the confirm modal (`Ctrl+G d`).
fn delete_branch_via_ui(rig: &mut Rig, repo: &FixtureRepo, name: &str) {
    let (col, row) = rig
        .term
        .screen()
        .find_all(name)
        .into_iter()
        .min_by_key(|(col, _)| *col)
        .unwrap_or_else(|| panic!("no {name} row:\n{}", rig.term.text()));
    rig.term.click(col, row).unwrap();
    rig.term.send_keys("C-g d").unwrap();
    rig.term
        .wait_for("the delete confirmation", STEP, |s| {
            s.contains(&format!("Delete branch '{name}'?"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("y").unwrap();
    wait_until("the branch to be deleted", || {
        git(repo.root(), &["branch", "--list", name]).is_empty()
    });
    // Back to the Log, where `c` starts a commit.
    let (col, row) = rig
        .term
        .wait_for_text("initial commit", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.click(col, row).unwrap();
}

fn open_history(rig: &mut Rig, wanted_row: &str) {
    // Alt+Z, the default key of "Edit: Undo History...".
    rig.term.send_keys("M-z").unwrap();
    rig.term
        .wait_for("the Undo history panel with its rows", STEP, |s| {
            s.contains("can be undone") && s.contains(wanted_row)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build()
}

/// The maintainer's scenario: a branch was deleted several actions ago. "Restore
/// only this" brings the branch back at the commit it had, the commits made
/// since stay, and one Ctrl+Z takes the restore itself back.
#[test]
fn restore_only_this_brings_a_deleted_branch_back_and_keeps_the_later_commits() {
    let repo = fixture();
    let feature = git(repo.root(), &["rev-parse", "feature"]);
    let mut rig = launch(&repo, "pty-history-restore");
    delete_branch_via_ui(&mut rig, &repo, "feature");
    for subject in ["later-one", "later-two", "later-three"] {
        commit_via_ui(&mut rig, &repo, subject);
    }
    let tip = git(repo.root(), &["rev-parse", "HEAD"]);

    open_history(&mut rig, "Delete branch");
    // Newest first: three commits, then the branch deletion.
    rig.term.send_keys("Down Down Down").unwrap();
    rig.term
        .wait_for("Restore only this offered for the deletion", STEP, |s| {
            s.contains("Restore only this brings back")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("r").unwrap();
    rig.term
        .wait_for("the confirmation with what will come back", STEP, |s| {
            s.contains("Restore") && s.contains("Cancel") && s.contains("feature")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert!(
        git(repo.root(), &["branch", "--list", "feature"]).is_empty(),
        "the confirmation must change nothing yet"
    );
    rig.term.send_keys("Enter").unwrap();
    wait_until("the branch to come back", || {
        !git(repo.root(), &["branch", "--list", "feature"]).is_empty()
    });
    assert_eq!(git(repo.root(), &["rev-parse", "feature"]), feature);
    assert_eq!(
        git(repo.root(), &["rev-parse", "HEAD"]),
        tip,
        "the later commits are kept"
    );
    assert_eq!(head_subject(repo.root()), "later-three");
    rig.term
        .wait_for("the result naming the undo key", STEP, |s| {
            s.contains("Ctrl+Z undoes this")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // The restore is itself an action: Ctrl+Z removes the restored branch.
    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for("the panel closed", STEP, |s| {
            !s.contains("actions can be undone")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("C-z").unwrap();
    wait_until("the restored branch to go again", || {
        git(repo.root(), &["branch", "--list", "feature"]).is_empty()
    });
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), tip);
    rig.term.stop().unwrap();
}

/// "Undo up to here" on the oldest of three commits undoes all three, after
/// one confirmation that lists them.
#[test]
fn undo_up_to_here_undoes_every_action_above_the_row() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-history-upto");
    for subject in ["first-one", "second-one", "third-one"] {
        commit_via_ui(&mut rig, &repo, subject);
    }
    open_history(&mut rig, "Commit \"third-one\"");
    rig.term.send_keys("Down Down u").unwrap();
    rig.term
        .wait_for("the confirmation naming all three", STEP, |s| {
            s.contains("Undo these 3 actions")
                && s.contains("first-one")
                && s.contains("second-one")
                && s.contains("third-one")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(head_subject(repo.root()), "third-one");
    rig.term.send_keys("Enter").unwrap();
    wait_until("all three commits to be undone", || {
        head_subject(repo.root()) == "initial commit"
    });
    rig.term
        .wait_for("the count in the status line", STEP, |s| {
            s.contains("undid 3 actions")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.stop().unwrap();
}

/// Types `query` in the command palette (opened from the open panel with
/// Ctrl+P) and runs the first row.
fn palette_run(rig: &mut Rig, query: &str) {
    rig.term.send_keys("C-p").unwrap();
    rig.term
        .wait_for("the command palette over the panel", STEP, |s| {
            s.contains("Command Palette") || s.contains("commands)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_text(query).unwrap();
    rig.term
        .wait_for("the row in the list", STEP, |s| s.contains(query))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
}

/// The palette twin of the `r` key: "Undo History: Restore Only This..."
/// acts on the selected row of the open panel, confirms, and brings the
/// deleted branch back.
#[test]
fn the_palette_restores_only_the_selected_history_row() {
    let repo = fixture();
    let feature = git(repo.root(), &["rev-parse", "feature"]);
    let mut rig = launch(&repo, "pty-history-palette");
    delete_branch_via_ui(&mut rig, &repo, "feature");
    open_history(&mut rig, "Delete branch");

    palette_run(&mut rig, "Undo History: Restore Only This");
    rig.term
        .wait_for("the restore confirmation", STEP, |s| {
            s.contains("Restore") && s.contains("Cancel") && s.contains("feature")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert!(
        git(repo.root(), &["branch", "--list", "feature"]).is_empty(),
        "the confirmation must change nothing yet"
    );
    rig.term.send_keys("Enter").unwrap();
    wait_until("the branch to come back", || {
        !git(repo.root(), &["branch", "--list", "feature"]).is_empty()
    });
    assert_eq!(git(repo.root(), &["rev-parse", "feature"]), feature);
    rig.term.stop().unwrap();
}

/// A row that cannot do what the palette asked says why instead of doing
/// nothing: nothing is undone, so "Redo Up to Here" explains it.
#[test]
fn a_palette_row_the_selection_cannot_do_says_why() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-history-palette-why");
    delete_branch_via_ui(&mut rig, &repo, "feature");
    open_history(&mut rig, "Delete branch");

    palette_run(&mut rig, "Undo History: Redo Up to Here");
    rig.term
        .wait_for("the reason", STEP, |s| s.contains("nothing to redo"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert!(git(repo.root(), &["branch", "--list", "feature"]).is_empty());
    rig.term.stop().unwrap();
}

/// The undone actions are listed above the newest undo entry, under a
/// divider, so Redo is where you look first; the selection starts on the
/// newest undo entry, and one Up reaches the entry Redo brings back.
#[test]
fn the_undone_entries_sit_above_the_stack_and_redo_from_there() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-history-redo-on-top");
    for subject in ["first-one", "second-one"] {
        commit_via_ui(&mut rig, &repo, subject);
    }
    rig.term.send_keys("C-z").unwrap();
    wait_until("the second commit to be undone", || {
        head_subject(repo.root()) == "first-one"
    });
    open_history(&mut rig, "Commit \"first-one\"");
    let row_of = |rig: &Rig, text: &str| -> u16 {
        rig.term
            .screen()
            .find_all(text)
            .into_iter()
            .map(|(_, row)| row)
            .next()
            .unwrap_or_else(|| panic!("no {text:?} row:\n{}", rig.term.text()))
    };
    let undone = row_of(&rig, "Commit \"second-one\"");
    let divider = row_of(&rig, "Redo brings them back");
    let newest = row_of(&rig, "Commit \"first-one\"");
    assert!(
        undone < divider && divider < newest,
        "undone {undone}, divider {divider}, newest undo {newest}:\n{}",
        rig.term.text()
    );
    // The selection starts on the newest undo entry: Undo is the action.
    rig.term
        .wait_for("Undo up to here offered", STEP, |s| {
            s.contains("Undo up to here")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Up y").unwrap();
    rig.term
        .wait_for("the redo confirmation", STEP, |s| {
            s.contains("Redo this action?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    wait_until("the commit to be redone", || {
        head_subject(repo.root()) == "second-one"
    });
    rig.term.stop().unwrap();
}
