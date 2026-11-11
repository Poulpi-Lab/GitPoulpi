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

/// Clicks `text` (found on a settled screen) until `done` holds; a click
/// aimed at a torn or still-scrolling frame lands nowhere.
fn click_until(
    term: &mut Terminal,
    text: &str,
    what: &str,
    mut done: impl FnMut(&xtask::pty::Screen) -> bool,
) {
    let mut ready = |s: &xtask::pty::Screen| done(s);
    let mut last = None;
    for _ in 0..4 {
        let screen = settled(term);
        if ready(&screen) {
            return;
        }
        if let Some((col, row)) = screen.find(text) {
            term.click(col + 1, row).unwrap();
        }
        match term.wait_for(what, Duration::from_secs(10), &mut ready) {
            Ok(_) => return,
            Err(e @ xtask::pty::Error::Timeout { .. }) => last = Some(e),
            Err(e) => panic!("{e}"),
        }
    }
    panic!("{}\n{}", last.unwrap(), term.text());
}

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

/// The screen once two reads a moment apart agree (a scrolling page or a
/// torn frame is not a place to aim a click at).
fn settled(term: &Terminal) -> xtask::pty::Screen {
    let mut last = term.screen();
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(150));
        let now = term.screen();
        if now.text() == last.text() {
            return now;
        }
        last = now;
    }
    panic!("the screen never settled:\n{}", last.text());
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
    common::settings_page_nth(term, "Conflict prevention", 1);
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
        .wait_for("the whole popup", STEP, |s| {
            s.contains("Predicted conflicts")
                && s.contains("A merge with origin/main would conflict in:")
                && s.contains("a.txt")
                && s.contains("Open merge preview")
                && s.contains("Rebase onto base")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = popup;
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
    click_until(&mut term, "Show files", "the popup", |s| {
        s.contains("Open merge preview")
    });
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
            s.contains("Base branches")
                && s.find("Check for conflicts with the")
                    .is_some_and(|(_, r)| s.line(r).contains("[x]"))
                && s.contains("origin/feature/**")
                && s.contains("!origin/abc")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen.find("Check for conflicts with the").unwrap();
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

    // Base branches: switch to the repository's own list, then exclude the
    // only candidate.
    let (c, r) = common::wait_for_choice(&term, "Default (every branch)", STEP);
    term.click(c, r).unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for(&term, "the own list chosen", || {
        state_file(&repo).contains("conflict_base_source = \"own\"")
    });
    let screen = term.screen();
    let (c, r) = screen
        .find_all("Base branches")
        .into_iter()
        .find(|(_, r)| !screen.line(*r).contains("from"))
        .unwrap();
    term.click(c + 1, r).unwrap();
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

/// The global lists: create one, give it patterns, make it the default, and
/// a repository with no list of its own follows it (the badge goes, because
/// the default excludes the only candidate) without a restart. Then the
/// repository follows the list by name and resets back to the default.
#[test]
fn a_default_list_applies_to_a_repository_without_its_own_and_the_source_can_be_reset() {
    let repo = conflicting_feature();
    let (mut term, config) = launch(repo.root(), "pty-conflict-lists");
    wait_badge(&mut term);
    let config_file =
        || std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();

    // The palette opens Settings on the global group.
    common::retry_until(
        &mut term,
        "the palette opens the lists",
        4,
        Duration::from_secs(8),
        |t| {
            t.send_keys("C-p").unwrap();
            t.send_text("edit base branch lists").unwrap();
            t.send_keys("Enter").unwrap();
        },
        |s| s.contains("Conflict prevention - global"),
    );
    common::scroll_settings_to(&mut term, "New list", STEP);
    // New list: a click on its row opens the name field.
    // (Retried until the name field is open: the value no longer reads
    // "(not set)".)
    common::retry_until(
        &mut term,
        "the name field open",
        4,
        Duration::from_secs(10),
        |t| {
            // On a settled screen: the page may still be scrolling.
            let s = settled(t);
            if let Some((c, r)) = s.find("New list") {
                if s.line(r).contains("(not set)") {
                    t.click(c + 1, r).unwrap();
                }
            }
        },
        |s| {
            s.find("New list")
                .is_some_and(|(_, r)| s.line(r).contains("\u{2588}"))
        },
    );
    term.send_text("nomain").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for(&term, "the list saved", || {
        config_file().contains("[conflict_prevention.lists.nomain]")
    });

    // Its patterns: exclude the only candidate.
    common::scroll_settings_to(&mut term, "Patterns", STEP);
    click_until(&mut term, "Patterns", "the editor", |s| {
        s.contains("Save") && s.contains("Cancel")
    });
    term.send_keys("Backspace").unwrap();
    term.send_keys("Backspace").unwrap();
    term.send_text("!origin/main").unwrap();
    term.send_keys("C-s").unwrap();
    wait_for(&term, "the patterns saved", || {
        config_file().contains("!origin/main")
    });

    // Make it the default: the badge is still there until Settings closes.
    common::scroll_settings_to(&mut term, "Make default", STEP);
    click_text(&mut term, "Make default");
    wait_for(&term, "the default saved", || {
        config_file().contains("default_list = \"nomain\"")
    });
    close_settings(&mut term);
    term.wait_for("no base under the default list, no badge", STEP, |s| {
        !s.contains(BADGE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The repository's page: it follows the default; pick the list by name.
    open_page(&mut term);
    let (c, r) = common::wait_for_choice(&term, "Default (list nomain)", STEP);
    term.click(c, r).unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for(&term, "the list followed", || {
        state_file(&repo).contains("conflict_base_list = \"nomain\"")
    });
    // Reset to default (it asks first: Tab to the confirm, Enter).
    click_text(&mut term, "Reset to default");
    term.wait_for("the question", STEP, |s| s.contains("Cancel"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    wait_for(&term, "the choice dropped", || {
        !state_file(&repo).contains("conflict_base_list")
    });
    common::wait_for_choice(&term, "Default (list nomain)", STEP);

    let _ = term.stop();
}
