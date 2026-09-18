//! Proactive conflict detection, through a pty: `feature` and `origin/main`
//! edit the same line, so a badge "Conflicts with origin/main in 1 file"
//! appears on the Log; the badge opens the popup with the file (the palette
//! command is its twin); the repository's switch and the Base branches
//! patterns hide it; fixing the conflict on the branch removes it.
//!
//! No hard-coded coordinates: every position is found on the screen. The
//! Log pane has no title: the fixture's own commit summary is the anchor.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::prediction_fixture::{conflicting_feature, fix_the_conflict};
use gitpoulpi_testkit::FixtureRepo;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const BADGE: &str = "Conflicts with origin/main in 1 file";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("feature work"))
        .mouse_capture()
        .start()
}

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_badge(term: &mut Terminal) {
    term.wait_for("the conflict badge", STEP, |s| s.contains(BADGE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn wait_for(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn state_file(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml")).unwrap_or_default()
}

fn open_page(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Conflict prevention");
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

/// The badge appears (a background merge, 2 s after the repository went
/// quiet), its button opens the popup, the popup acts, and the palette
/// command opens the same popup from the keyboard.
#[test]
fn the_badge_names_the_conflicting_file_and_opens_the_popup() {
    let repo = conflicting_feature();
    let (mut term, _config) = launch(repo.root(), "pty-conflict-badge");
    wait_badge(&mut term);

    // The badge's button.
    click_text(&mut term, "Show files");
    let popup = term
        .wait_for("the popup", STEP, |s| s.contains("Predicted conflicts"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(popup.contains("A merge with origin/main would conflict in:"));
    assert!(popup.contains("a.txt"), "{}", term.text());
    assert!(popup.contains("Open merge preview") && popup.contains("Rebase onto base"));
    // Escape closes it.
    term.send_keys("Escape").unwrap();
    term.wait_for("the popup closed", STEP, |s| {
        !s.contains("Predicted conflicts")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The keyboard twin: the palette.
    common::retry_until(
        &mut term,
        "the palette command opens the popup",
        4,
        Duration::from_secs(8),
        |t| {
            t.send_keys("C-p").unwrap();
            t.send_text("show predicted conflicts").unwrap();
            t.send_keys("Enter").unwrap();
        },
        |s| s.contains("Predicted conflicts"),
    );
    // Left, Enter on the focused button: Close does nothing at all.
    term.send_keys("Escape").unwrap();
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]).trim().len(),
        40,
        "still a plain checkout"
    );
    let _ = term.stop();
}

/// "Open merge preview" runs the branch menu's Merge of the base: the
/// conflicting merge stops in the repository like any other.
#[test]
fn the_popup_merges_the_base_and_the_conflict_is_real() {
    let repo = conflicting_feature();
    let (mut term, _config) = launch(repo.root(), "pty-conflict-merge");
    wait_badge(&mut term);
    click_text(&mut term, "Show files");
    click_text(&mut term, "Open merge preview");
    wait_for(&term, "the merge stopped on the predicted file", || {
        repo.root().join(".git/MERGE_HEAD").exists()
            && repo.git(&["diff", "--name-only", "--diff-filter=U"]).trim() == "a.txt"
    });
    let _ = term.stop();
}

/// The repository's switch hides the badge, and the Base branches patterns
/// can leave no base at all.
#[test]
fn the_repository_switch_and_the_base_branches_hide_the_badge() {
    let repo = conflicting_feature();
    let (mut term, _config) = launch(repo.root(), "pty-conflict-settings");
    wait_badge(&mut term);

    open_page(&mut term);
    let screen = term
        .wait_for("the page", STEP, |s| {
            s.contains("Check for conflicts with the") && s.contains("Base branches")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("origin/feature/**") && screen.contains("!origin/abc"));
    let (c, r) = screen.find("Check for conflicts with the").unwrap();
    assert!(screen.line(r).contains("[x]"), "on by default");
    term.click(c, r).unwrap();
    term.wait_for("ticked off and saved", STEP, |s| {
        s.line(r).contains("[ ]") && state_file(&repo).contains("conflict_detection = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    close_settings(&mut term);
    term.wait_for("the badge hidden", STEP, |s| !s.contains(BADGE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // On again: the badge comes back.
    open_page(&mut term);
    let screen = term
        .wait_for("the page again", STEP, |s| {
            s.contains("Check for conflicts with the")
        })
        .unwrap();
    let (c, r) = screen.find("Check for conflicts with the").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("ticked on", STEP, |s| {
        s.line(r).contains("[x]") && !state_file(&repo).contains("conflict_detection = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Base branches: exclude the only candidate.
    click_text(&mut term, "Base branches");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Backspace").unwrap();
    term.send_keys("Backspace").unwrap();
    term.send_text("!origin/main").unwrap();
    term.send_keys("C-s").unwrap();
    wait_for(&term, "the patterns saved", || {
        state_file(&repo).contains("conflict_base_branches = \"!origin/main\"")
    });
    close_settings(&mut term);
    term.wait_for("no base, no badge", STEP, |s| !s.contains(BADGE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// Fixing the conflict on the branch removes the badge.
#[test]
fn fixing_the_conflict_on_the_branch_removes_the_badge() {
    let repo = conflicting_feature();
    let (mut term, _config) = launch(repo.root(), "pty-conflict-fix");
    wait_badge(&mut term);
    fix_the_conflict(&repo);
    term.wait_for("the badge gone", STEP, |s| {
        !s.contains("Conflicts with origin/main")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // And it stays gone: the prediction for the new HEAD is clean.
    std::thread::sleep(Duration::from_secs(4));
    assert!(!term.text().contains("Conflicts with origin/main"));
    let _ = term.stop();
}
