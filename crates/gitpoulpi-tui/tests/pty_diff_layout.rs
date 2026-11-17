//! Two axes of the diff, end to end through a pty:
//! - the diff's Inline / Split and the Hunks toggle: with Hunks off the
//!   whole file is drawn (unchanged lines read from the commit's blob), and
//!   the choice is written to `config.toml`;
//! - opening a diff collapses the Refs pane, closing it restores it, and the
//!   automatic collapse never reaches the saved layout.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "SHA";

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    let term = spawn(repo, config, session);
    term.wait_for(
        "the commit detail of HEAD and the Refs pane",
        STARTUP,
        |s| s.contains("View all files") && s.contains(LOG_HEADER) && s.contains("LOCAL"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn spawn(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(160, 50)
        .no_exit_pause()
        .spawn()
}

/// A 40-line file, then a commit changing only its line 20.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let before: String = (1..=40).map(|n| format!("row {n:02}\n")).collect();
    let after = before.replace("row 20\n", "row 20 changed\n");
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("big.txt", &before, "forty rows")
        .unwrap()
        .commit_file("big.txt", &after, "change row 20")
        .unwrap()
        .build()
}

fn open_the_diff(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("big.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("\u{2715}") && s.contains("row 20 changed") && !s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The diff takes the Refs pane's room and gives it back.
/// Collapsing the Status pane while the diff is open saves the layout --
/// and the saved Refs pane is the user's (expanded), not the automatic
/// collapse: a relaunch shows it.
#[test]
fn a_diff_collapses_the_refs_pane_restores_it_on_close_and_never_saves_the_collapse() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-diff-collapse");

    open_the_diff(&mut term);
    term.wait_for("the Refs pane collapsed by the diff", STEP, |s| {
        !s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A layout save while auto-collapsed (collapse the Status pane).
    term.send_keys("C-n").unwrap();
    term.wait_for("the navigation prefix", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("l").unwrap();
    term.wait_for("the Status pane collapsed", STEP, |s| {
        s.contains("Status pane collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Escape").unwrap();
    term.wait_for("the graph back, and the Refs pane with it", STEP, |s| {
        s.contains(LOG_HEADER) && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("q").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !term.exited() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(term.exited(), "q over the graph should quit");
    drop(term);

    // Relaunch: the saved layout is the Log's own - the Refs pane expanded
    // (the diff's collapse was not saved) and the Status pane as the Log had
    // it (the collapse made while the diff was open belongs to the diff).
    // Wait for the loaded window, then for a quiet one: the Status pane's
    // "View all files" only shows once the commit has loaded, so a wait for
    // its absence passes on a window that has not drawn it yet.
    let term = spawn(repo.root(), config.path(), "pty-diff-collapse");
    term.wait_for("the saved layout: Refs and Status expanded", STARTUP, |s| {
        s.contains(LOG_HEADER) && s.contains("LOCAL") && s.contains("View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(500), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.text();
    assert!(
        screen.contains("LOCAL") && screen.contains("View all files"),
        "{screen}"
    );
}

/// The pane belongs to the Log: restoring it by hand while the diff is open
/// and collapsing it again applies to the diff only, and closing the diff
/// brings the Log's own state (shown) back.
#[test]
fn a_refs_pane_toggled_by_hand_during_the_diff_is_the_logs_again_after_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-diff-collapse-user");
    open_the_diff(&mut term);
    term.wait_for("auto-collapsed", STEP, |s| !s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n H").unwrap();
    term.wait_for("restored by hand", STEP, |s| {
        s.contains("Refs pane restored") && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n H").unwrap();
    term.wait_for("collapsed by hand", STEP, |s| {
        s.contains("Refs pane collapsed") && !s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for(
        "the graph back, the Refs pane shown as the Log had it",
        STEP,
        |s| s.contains(LOG_HEADER) && s.contains("LOCAL"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Hunks on shows only the change and its context; `H` (the
/// Hunks button's key) draws the whole file with the change in place,
/// in Inline and in Split, and the choice is saved.
#[test]
fn the_hunks_toggle_draws_the_whole_file_in_both_layouts_and_is_saved() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-diff-hunks");
    open_the_diff(&mut term);
    term.wait_for("only the hunk: row 17..23, not row 05", STEP, |s| {
        s.contains("row 17") && s.contains("row 23") && !s.contains("row 05")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("H").unwrap();
    term.wait_for("the whole file, the change in place", STEP, |s| {
        s.contains("row 01") && s.contains("row 05") && s.contains("row 20 changed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("diff_hunks_only = false"), "{saved}");

    // Split, still the whole file: an unchanged row on both sides.
    term.send_keys("m").unwrap();
    term.wait_for("split, whole file", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.matches("row 05").count() == 2 && l.contains('\u{2502}'))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Back to only the hunks, in Split.
    term.send_keys("H").unwrap();
    term.wait_for("split, only the hunk", STEP, |s| {
        !s.contains("row 05") && s.contains("row 20 changed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
