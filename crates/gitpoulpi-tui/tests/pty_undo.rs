//! Undo / redo in the TUI, end to end through a pty: the real binary,
//! daemon and `git`, with a temporary config dir. Commits are made through
//! the UI (`c`, a summary, `Ctrl+Enter`), so the daemon's journal records
//! them like a user's.
//!
//! The rules are `pty_ui.rs`'s: nothing is found by a hard-coded
//! coordinate, nothing sleeps, every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// One commit and one staged edit: the composer can commit.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

struct Rig {
    term: Terminal,
    _config: tempfile::TempDir,
}

fn launch(repo: &FixtureRepo, session: &str, config_toml: &str, delay_ms: Option<&str>) -> Rig {
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(config_toml)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the toolbar and the WIP row", |s| {
            s.contains("Undo") && s.contains("Redo") && s.contains("initial commit")
        })
        .mouse_capture()
        // A blocked entry is what some of these tests are about: an outside
        // commit is not recorded as an entry here (pty_outside_undo.rs
        // covers it recorded).
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    if let Some(ms) = delay_ms {
        launch = launch.env("GITPOULPI_TEST_ACTION_DELAY_MS", ms);
    }
    let (term, config) = launch.start();
    Rig {
        term,
        _config: config,
    }
}

/// The toolbar button's `(col, row)`: the topmost occurrence of its label.
fn button(term: &Terminal, label: &str) -> (u16, u16) {
    term.screen()
        .find_all(label)
        .into_iter()
        .min_by_key(|(_, row)| *row)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", term.text()))
}

fn fg(term: &Terminal, label: &str) -> Option<Color> {
    let (col, row) = button(term, label);
    term.screen().cell(col, row).map(|c| c.fg)
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

fn git(repo: &Path, args: &[&str]) -> String {
    common::git(repo, args).trim().to_string()
}

/// Commits the staged changes through the UI: `c` (the WIP row's Summary),
/// the subject, Enter to the Description, `Ctrl+Enter` (a line feed in a pty).
fn commit_via_ui(rig: &mut Rig, repo: &FixtureRepo, subject: &str) {
    rig.term.send_keys("c").expect("press c in the Log");
    rig.term
        .wait_for("the Summary field focused", STEP, |s| {
            // The caret `_` opens the empty Summary box (the WIP row is
            // still listed after a first commit, so it is no signal).
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

fn stage_new_file(repo: &FixtureRepo, name: &str) {
    std::fs::write(repo.root().join(name), name).unwrap();
    git(repo.root(), &["add", name]);
}

/// Commit through the UI, click Undo: the commit is gone and its files are
/// staged again; Redo brings back the very same commit; the buttons light
/// and gray with what there is to do.
#[test]
fn clicking_undo_takes_a_commit_back_staged_and_redo_restores_the_same_commit() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-click", "", None);
    let grayed = fg(&rig.term, "Undo");
    assert_eq!(fg(&rig.term, "Redo"), grayed, "both start grayed");

    commit_via_ui(&mut rig, &repo, "the subject");
    let commit = git(repo.root(), &["rev-parse", "HEAD"]);
    rig.term
        .wait_for("Undo to light up", STEP, |_| {
            fg(&rig.term, "Undo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(fg(&rig.term, "Redo"), grayed, "nothing to redo yet");

    let (col, row) = button(&rig.term, "Undo");
    rig.term.click(col, row).unwrap();
    wait_until("the commit to be undone", || {
        head_subject(repo.root()) == "initial commit"
    });
    assert_eq!(
        git(repo.root(), &["diff", "--cached", "--name-only"]),
        "a.txt",
        "the commit's files are staged again"
    );
    rig.term
        .wait_for("Redo lit, Undo grayed", STEP, |_| {
            fg(&rig.term, "Redo") != grayed && fg(&rig.term, "Undo") == grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = button(&rig.term, "Redo");
    rig.term.click(col, row).unwrap();
    wait_until("the commit to come back", || {
        git(repo.root(), &["rev-parse", "HEAD"]) == commit
    });
    assert_eq!(git(repo.root(), &["status", "--porcelain"]), "");
    rig.term.stop().unwrap();
}

/// Ctrl+Z and Ctrl+Y do the same, and Ctrl+Z does not suspend the process
/// (raw mode: it arrives as a key).
#[test]
fn ctrl_z_undoes_and_ctrl_y_redoes_without_suspending_the_app() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-keys", "", None);
    commit_via_ui(&mut rig, &repo, "keyed");
    let commit = git(repo.root(), &["rev-parse", "HEAD"]);

    // Let the journal reach the window before the key (Undo lit).
    let grayed = fg(&rig.term, "Redo");
    rig.term
        .wait_for("Undo to light up", STEP, |_| {
            fg(&rig.term, "Undo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z to undo", || {
        head_subject(repo.root()) == "initial commit"
    });
    assert!(!rig.term.exited(), "Ctrl+Z must not stop the app");
    rig.term
        .wait_for("Redo to light up", STEP, |_| {
            fg(&rig.term, "Redo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("C-y").unwrap();
    wait_until("Ctrl+Y to redo", || {
        git(repo.root(), &["rev-parse", "HEAD"]) == commit
    });
    rig.term.stop().unwrap();
}

/// Ctrl+Z in the commit message box types nothing and undoes nothing.
#[test]
fn ctrl_z_in_the_commit_message_field_does_not_undo_a_commit() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-field", "", None);
    commit_via_ui(&mut rig, &repo, "first");
    let commit = git(repo.root(), &["rev-parse", "HEAD"]);
    stage_new_file(&repo, "b.txt");
    rig.term
        .wait_for("the new staged file", STEP, |s| s.contains("b.txt"))
        .unwrap_or_else(|e| panic!("{e}"));

    rig.term.send_keys("c").unwrap();
    rig.term.send_text("draft").unwrap();
    rig.term.send_keys("C-z").unwrap();
    rig.term
        .wait_for("the draft still there", STEP, |s| s.contains("draft"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        git(repo.root(), &["rev-parse", "HEAD"]),
        commit,
        "the commit must not have been undone"
    );
    rig.term.stop().unwrap();
}

/// Hovering Undo names the operation; with nothing recorded it says so.
#[test]
fn the_tooltip_names_the_operation_that_would_be_undone() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-tooltip", "", None);
    let (col, row) = button(&rig.term, "Undo");
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("the nothing-to-undo tooltip", STEP, |s| {
            s.contains("Nothing to undo")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    commit_via_ui(&mut rig, &repo, "named op");
    // Off the button and back on, so the tooltip restarts on the new state.
    rig.term.move_to(col + 40, row + 10).unwrap();
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("the tooltip naming the commit", STEP, |s| {
            s.contains("Undo Commit \"named op\"") && s.contains("Ctrl+Z")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.stop().unwrap();
}

/// A grayed Undo, clicked, says why and changes nothing.
#[test]
fn undo_with_nothing_recorded_is_grayed_and_explains_itself() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-nothing", "", None);
    let (col, row) = button(&rig.term, "Undo");
    rig.term.click(col, row).unwrap();
    rig.term
        .wait_for("the explanation", STEP, |s| s.contains("nothing to undo"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(head_subject(repo.root()), "initial commit");
    rig.term.stop().unwrap();
}

/// After a commit made outside GitPoulpi the button is blocked within one
/// poll, its tooltip says why, and clicking it refuses and changes nothing.
#[test]
fn after_an_outside_commit_undo_is_blocked_and_the_reason_is_shown() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-blocked", "", None);
    commit_via_ui(&mut rig, &repo, "ours");
    let (col, row) = button(&rig.term, "Undo");
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("Undo ready", STEP, |s| s.contains("Undo Commit \"ours\""))
        .unwrap_or_else(|e| panic!("{e}"));

    git(
        repo.root(),
        &["commit", "--allow-empty", "-q", "-m", "from the shell"],
    );
    let theirs = git(repo.root(), &["rev-parse", "HEAD"]);
    rig.term.move_to(col + 40, row + 10).unwrap();
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("the blocked tooltip", STEP, |s| {
            s.contains("Can't undo Commit \"ours\"") && s.contains("outside GitPoulpi")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // The status line is transient and a loaded machine can drop a click
    // on the tooltip's dismissal: click again until the refusal shows.
    let deadline = Instant::now() + STEP;
    loop {
        rig.term.click(col, row).unwrap();
        let shown = rig
            .term
            .wait_for(
                "the refusal in the status line",
                Duration::from_secs(4),
                |s| s.contains("can't undo Commit \"ours\""),
            )
            .is_ok();
        if shown {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "no refusal:\n{}",
            rig.term.text()
        );
    }
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), theirs);
    rig.term.stop().unwrap();
}

/// `general.undo_depth = 1` keeps only the latest operation.
#[test]
fn depth_one_remembers_only_the_latest_commit() {
    let repo = fixture();
    let mut rig = launch(
        &repo,
        "pty-undo-depth-1",
        "[general]\nundo_depth = 1\n",
        None,
    );
    commit_via_ui(&mut rig, &repo, "one");
    stage_new_file(&repo, "b.txt");
    rig.term
        .wait_for("b.txt staged", STEP, |s| s.contains("b.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    commit_via_ui(&mut rig, &repo, "two");
    let grayed = fg(&rig.term, "Redo");

    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);
    rig.term.send_keys("C-z").unwrap();
    wait_until("the first undo", || head_subject(repo.root()) == "one");
    // Redo lighting up means the undo's reply has landed; an Undo grayed
    // only while the undo is still in flight would swallow the next Ctrl+Z.
    rig.term
        .wait_for("Undo grayed again", STEP, |_| {
            fg(&rig.term, "Undo") == grayed && fg(&rig.term, "Redo") != grayed
        })
        .unwrap_or_else(|e| panic!("depth 1 forgets the older commit: {e}"));
    rig.term.send_keys("C-z").unwrap();
    rig.term
        .wait_for("nothing to undo", STEP, |s| s.contains("nothing to undo"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(head_subject(repo.root()), "one");
    rig.term.stop().unwrap();
}

/// `undo_depth = 0` is unlimited: two undos in a row.
#[test]
fn depth_zero_remembers_every_commit() {
    let repo = fixture();
    let mut rig = launch(
        &repo,
        "pty-undo-depth-0",
        "[general]\nundo_depth = 0\n",
        None,
    );
    commit_via_ui(&mut rig, &repo, "one");
    stage_new_file(&repo, "b.txt");
    rig.term
        .wait_for("b.txt staged", STEP, |s| s.contains("b.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    commit_via_ui(&mut rig, &repo, "two");
    let grayed = fg(&rig.term, "Redo");

    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);
    rig.term.send_keys("C-z").unwrap();
    wait_until("the first undo", || head_subject(repo.root()) == "one");
    rig.term
        .wait_for("the second Ctrl+Z to be possible", STEP, |_| {
            fg(&rig.term, "Undo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("C-z").unwrap();
    wait_until("the second undo", || {
        head_subject(repo.root()) == "initial commit"
    });
    rig.term.stop().unwrap();
}

/// A commit already on its upstream is a risky undo: a confirm dialog
/// names why; canceling changes nothing, confirming undoes it.
#[test]
fn undoing_a_pushed_commit_asks_first() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    git(
        repo.root(),
        &["branch", "--set-upstream-to=origin/main", "main"],
    );
    let mut rig = launch(&repo, "pty-undo-pushed", "", None);
    commit_via_ui(&mut rig, &repo, "pushed one");
    let commit = git(repo.root(), &["rev-parse", "HEAD"]);
    git(repo.root(), &["push", "origin", "main"]);
    let grayed = fg(&rig.term, "Redo");
    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);

    rig.term.send_keys("C-z").unwrap();
    rig.term
        .wait_for("the confirm dialog", STEP, |s| {
            s.contains("already on origin/main") && s.contains("Undo anyway")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), commit);

    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for("the dialog gone", STEP, |s| {
            !s.contains("already on origin/main")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), commit);

    rig.term.send_keys("C-z").unwrap();
    rig.term
        .wait_for("the confirm dialog again", STEP, |s| {
            s.contains("already on origin/main")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("y").unwrap();
    wait_until("the confirmed undo", || {
        head_subject(repo.root()) == "initial commit"
    });
    rig.term.stop().unwrap();
}

/// A slow undo shows the in-flight text and grays the buttons meanwhile.
#[test]
fn a_slow_undo_shows_in_flight_feedback() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-inflight", "", Some("2500"));
    commit_via_ui(&mut rig, &repo, "slow");
    let grayed = fg(&rig.term, "Redo");
    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);
    let (col, row) = button(&rig.term, "Undo");
    rig.term.click(col, row).unwrap();
    rig.term
        .wait_for("the Undoing... status", STEP, |s| s.contains("undoing..."))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        fg(&rig.term, "Undo"),
        grayed,
        "Undo grays out while it runs"
    );
    wait_until("the undo to land", || {
        head_subject(repo.root()) == "initial commit"
    });
    rig.term
        .wait_for("the status clear again", STEP, |s| {
            !s.contains("undoing...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.stop().unwrap();
}

/// The palette names the operation ("Edit: Undo Commit ...") and runs it.
#[test]
fn the_palette_lists_undo_with_the_operation_and_runs_it() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-palette", "", None);
    let grayed = fg(&rig.term, "Redo");
    commit_via_ui(&mut rig, &repo, "palette op");
    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);

    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("undo").unwrap();
    rig.term
        .wait_for("the Edit: Undo row naming the commit", STEP, |s| {
            s.contains("Edit: Undo Commit \"palette op\"") && s.contains("Ctrl+Z")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("Enter").unwrap();
    wait_until("the palette's undo", || {
        head_subject(repo.root()) == "initial commit"
    });
    rig.term.stop().unwrap();
}

/// A merge commit made from the Refs pane (select the branch, palette:
/// "Merge Selected Branch into Current"), then the toolbar Undo takes the
/// whole merge back, files included, and Redo brings back the same merge.
#[test]
fn clicking_undo_takes_a_merge_back_and_redo_restores_the_same_merge() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("feature.txt", "f\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("main.txt", "m\n", "main work")
        .unwrap()
        .build();
    let before = git(repo.root(), &["rev-parse", "HEAD"]);
    let mut rig = launch(&repo, "pty-undo-merge", "", None);
    let grayed = fg(&rig.term, "Redo");

    // The branch's row in the Refs pane (the leftmost "feature" on screen).
    let (col, row) = rig
        .term
        .screen()
        .find_all("feature")
        .into_iter()
        .min_by_key(|(col, _)| *col)
        .unwrap_or_else(|| panic!("no feature row:\n{}", rig.term.text()));
    rig.term.click(col, row).unwrap();
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("merge selected").unwrap();
    rig.term
        .wait_for("the merge palette row", STEP, |s| {
            s.contains("Merge Selected Branch into Current")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("Enter").unwrap();
    wait_until("the merge commit", || {
        git(repo.root(), &["rev-parse", "HEAD"]) != before
    });
    let merged = git(repo.root(), &["rev-parse", "HEAD"]);
    assert!(repo.root().join("feature.txt").exists());
    rig.term
        .wait_for("Undo to light up for the merge", STEP, |_| {
            fg(&rig.term, "Undo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = button(&rig.term, "Undo");
    rig.term.click(col, row).unwrap();
    wait_until("the merge to be undone", || {
        git(repo.root(), &["rev-parse", "HEAD"]) == before
    });
    assert!(
        !repo.root().join("feature.txt").exists(),
        "the merge's files went with it"
    );
    rig.term
        .wait_for("Redo to light up", STEP, |_| {
            fg(&rig.term, "Redo") != grayed
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = button(&rig.term, "Redo");
    rig.term.click(col, row).unwrap();
    wait_until("the merge to come back", || {
        git(repo.root(), &["rev-parse", "HEAD"]) == merged
    });
    assert!(repo.root().join("feature.txt").exists());
    rig.term.stop().unwrap();
}

/// `git stash list` as `<commit> <message>`, or `None` while the TUI is
/// mid-write (a poll must retry then, not panic).
fn stash_list(repo: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["stash", "list", "--format=%H %gs"])
        .current_dir(repo)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Drop a stash from the sidebar's menu, click the toolbar's
/// Undo: the very same stash commit is back as `stash@{0}`; Redo drops it
/// again.
#[test]
fn clicking_undo_brings_back_a_dropped_stash_with_the_same_commit() {
    let repo = fixture();
    let mut rig = launch(&repo, "pty-undo-stash-drop", "", None);

    // Stash the staged edit with the toolbar's Stash button.
    let (col, row) = button(&rig.term, "Stash");
    rig.term.click(col, row).unwrap();
    wait_until("the stash to land", || {
        stash_list(repo.root()).is_some_and(|l| !l.is_empty())
    });
    let stashed = stash_list(repo.root()).unwrap();

    // Drop it from the stash row's context menu, and confirm.
    let screen = rig
        .term
        .wait_for("the stash row in the sidebar", STEP, |s| {
            s.contains("WIP on main")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (scol, srow) = screen.find("WIP on main").unwrap();
    rig.term.right_click(scol + 1, srow).unwrap();
    let menu = rig
        .term
        .wait_for("the stash menu with Drop", STEP, |s| s.contains("Drop"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (mcol, mrow) = menu.find("Drop").unwrap();
    rig.term.click(mcol + 1, mrow).unwrap();
    rig.term
        .wait_for("the drop confirmation", STEP, |s| {
            s.contains("Drop stash@{0}?") && s.contains("Ctrl+Z undoes this")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("y").unwrap();
    wait_until("the stash to be dropped", || {
        stash_list(repo.root()).is_some_and(|l| l.is_empty())
    });

    // Undo from the toolbar: the same commit is stash@{0} again.
    let grayed = fg(&rig.term, "Redo");
    wait_until("Undo lit", || fg(&rig.term, "Undo") != grayed);
    let (ucol, urow) = button(&rig.term, "Undo");
    rig.term.click(ucol, urow).unwrap();
    wait_until("the stash to come back", || {
        stash_list(repo.root()).is_some_and(|l| l == stashed)
    });
    wait_until("Redo lit", || fg(&rig.term, "Redo") != grayed);

    let (rcol, rrow) = button(&rig.term, "Redo");
    rig.term.click(rcol, rrow).unwrap();
    wait_until("the stash to be dropped again", || {
        stash_list(repo.root()).is_some_and(|l| l.is_empty())
    });
    rig.term.stop().unwrap();
}

/// A stuck top entry (the repository moved on outside) names its way out
/// in the tooltip; the palette's *Forget blocked undo entry* asks, then
/// drops it from the journal without touching the repository, and the
/// entry under it shows next.
#[test]
fn the_palette_forgets_a_blocked_undo_entry_and_the_repository_is_untouched() {
    let repo = fixture();
    let mut rig = launch(
        &repo,
        "pty-undo-forget",
        "[general]\nundo_depth = 0\n",
        None,
    );
    commit_via_ui(&mut rig, &repo, "first");
    stage_new_file(&repo, "b.txt");
    rig.term
        .wait_for("b.txt staged", STEP, |s| s.contains("b.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    commit_via_ui(&mut rig, &repo, "second");
    git(
        repo.root(),
        &["commit", "--allow-empty", "-q", "-m", "from the shell"],
    );
    let theirs = git(repo.root(), &["rev-parse", "HEAD"]);

    let (col, row) = button(&rig.term, "Undo");
    rig.term.move_to(col + 40, row + 10).unwrap();
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("the tooltip naming the way out", STEP, |s| {
            s.contains("Can't undo Commit \"second\"") && s.contains("Forget it to reach")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("Forget blocked").unwrap();
    rig.term
        .wait_for("the palette entry", STEP, |s| {
            s.contains("Edit: Forget Blocked Undo Entry")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("Enter").unwrap();
    rig.term
        .wait_for("the confirmation", STEP, |s| {
            s.contains("Forget Commit \"second\"")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("y").unwrap();

    rig.term.move_to(col + 40, row + 10).unwrap();
    rig.term.move_to(col, row).unwrap();
    rig.term
        .wait_for("the entry under it", STEP, |s| {
            s.contains("Commit \"first\"") && !s.contains("Commit \"second\"")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), theirs);
    rig.term.stop().unwrap();
}
