//! The SUBMODULES section of the Refs sidebar in the real TUI: it lists a
//! repository's submodules with their state, its context menu and the
//! palette update them through the daemon, and Enter opens one as a tab.
//! Driven through a pty by `xtask::pty` (the rules are `pty_ui.rs`'s:
//! nothing located by a hard-coded coordinate, nothing sleeps).

mod common;

use common::git_trimmed as git;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SIDEBAR_RIGHT: u16 = 40;

/// A gitconfig letting git clone local submodules (it refuses the `file`
/// transport otherwise since 2.38.1); handed to the TUI's daemon through
/// `GIT_CONFIG_GLOBAL`.
fn gitconfig(dir: &Path) -> PathBuf {
    let path = dir.join("gitconfig");
    std::fs::write(&path, "[protocol \"file\"]\n\tallow = always\n").unwrap();
    path
}

/// A clone of a repository with a submodule: the submodule is declared and
/// recorded but not cloned. Returns the clone's root, with its holder.
fn clone_with_uncloned_submodule(gitconfig: &Path) -> (tempfile::TempDir, PathBuf) {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_submodule("libs/lib")
        .unwrap()
        .build();
    let holder = tempfile::tempdir().unwrap();
    let clone = holder.path().join("clone");
    let status = std::process::Command::new("git")
        .args(["clone", "-q"])
        .arg(source.root())
        .arg(&clone)
        .env("GIT_CONFIG_GLOBAL", gitconfig)
        .status()
        .unwrap();
    assert!(status.success());
    // The source must outlive the clone's submodule update.
    std::mem::forget(source);
    (holder, clone)
}

fn launch(repo: &Path, session: &str, gitconfig: &Path) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .env("GIT_CONFIG_GLOBAL", gitconfig)
        .size(200, 50)
        .ready_when("the log and the SUBMODULES section", |s| {
            s.contains("SUBMODULES") && s.contains("initial")
        })
        .start()
}

fn sidebar_row(screen: &Screen, text: &str) -> Option<(u16, u16)> {
    screen
        .find_all(text)
        .into_iter()
        .find(|(col, _)| *col < SIDEBAR_RIGHT)
}

fn tick_for(screen: &Screen, row: u16) -> bool {
    screen.line(row).contains('\u{2713}')
}

#[test]
fn the_section_lists_the_submodule_and_its_menu_updates_it() {
    let config = tempfile::tempdir().unwrap();
    let gc = gitconfig(config.path());
    let (_holder, clone) = clone_with_uncloned_submodule(&gc);
    let (mut term, _cfg) = launch(&clone, "pty-submodules-menu", &gc);

    let screen = term
        .wait_for("the submodule row, not cloned yet", STEP, |s| {
            sidebar_row(s, "libs/lib").is_some_and(|(_, row)| s.line(row).contains('\u{25CB}'))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!clone.join("libs/lib/lib.txt").exists());

    let (col, row) = sidebar_row(&screen, "libs/lib").unwrap();
    term.right_click(col + 2, row).expect("right-click the row");
    let menu = term
        .wait_for("the submodule menu", STEP, |s| {
            s.contains("Update (clone if needed)")
                && s.contains("Initialize")
                && s.contains("Open in new tab")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Update (clone if needed)").unwrap();
    term.click(c + 1, r).expect("click Update");

    term.wait_for("the submodule cloned and up to date", STEP, |s| {
        !s.contains("Update (clone if needed)")
            && sidebar_row(s, "libs/lib").is_some_and(|(_, row)| tick_for(s, row))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(clone.join("libs/lib/lib.txt").is_file());
    // Trimmed, so a leading `-` (not cloned) or `+` (moved) would show.
    let status = git(&clone, &["submodule", "status"]);
    assert!(!status.starts_with(['-', '+', 'U']), "{status}");
}

#[test]
fn the_keyboard_twins_update_all_and_open_the_submodule() {
    let config = tempfile::tempdir().unwrap();
    let gc = gitconfig(config.path());
    let (_holder, clone) = clone_with_uncloned_submodule(&gc);
    let (mut term, _cfg) = launch(&clone, "pty-submodules-keys", &gc);

    // The palette: Update All Submodules.
    term.wait_for("the SUBMODULES row", STEP, |s| {
        sidebar_row(s, "libs/lib").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Update All Submodules").unwrap();
    term.wait_for("the palette narrowed to that command", STEP, |s| {
        s.contains("Command Palette (1/") && s.contains("Git: Update All Submodules")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the submodule cloned and up to date", STEP, |s| {
        sidebar_row(s, "libs/lib").is_some_and(|(_, row)| tick_for(s, row))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(clone.join("libs/lib/lib.txt").is_file());

    // Select the row (a click) and Enter: it opens as a tab of its own.
    let screen = term.screen();
    let (col, row) = sidebar_row(&screen, "libs/lib").unwrap();
    term.click(col + 2, row).unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("a tab named lib next to the clone's", STEP, |s| {
        // The tab bar is the second line of the window.
        let bar = s.line(1);
        bar.contains("clone") && bar.contains("lib")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
