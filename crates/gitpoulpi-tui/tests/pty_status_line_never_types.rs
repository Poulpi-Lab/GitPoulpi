//! Nothing the user types is ever echoed in the status line. Every prompt
//! the app has (new branch and tag names, stash message, annotate, set
//! upstream, a number setting, create file, LFS track and untrack, rename,
//! the Refs filter) is opened in turn, a sentinel is typed into it, and the
//! sentinel must be on screen but not in the status line.
//!
//! The status line is found by `common::status_line_text`, which does not
//! rely on a fixed row: the last test checks it finds the footer in every
//! toolbar, status bar and tab bar layout, on real screens.
//!
//! Falsified: with a footer echo of the typed text put back (the `InlineName`
//! arm of `draw_footer` drawing `value`), every inline-name test and the layout
//! test go red; see `footer_guard_tests` for the structural twin.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{require_git_lfs, FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SENTINEL: &str = "zq9x";

/// Two commits (the older one tagged `mine`), a few branches, an origin that
/// has `main` and `feature`, and optionally a changed file in the work tree.
fn fixture(wip: bool) -> FixtureRepo {
    let before: String = (1..=40).map(|n| format!("row {n:02}\n")).collect();
    let mut builder = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "older target commit")
        .unwrap()
        .tag("mine")
        .unwrap()
        .commit_file("big.txt", &before, "newer head commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .push("feature")
        .unwrap();
    if wip {
        builder = builder.write_worktree("a.txt", b"one\ntwo\n").unwrap();
    }
    let repo = builder.build();
    repo.git(&["branch", "hello"]);
    repo.git(&["branch", "to-delete", "HEAD~1"]);
    repo
}

fn launch_with(repo: &FixtureRepo, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the fixture loaded", |s| {
            s.contains("older target") && s.contains("newer head")
        })
        .start()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    launch_with(repo, session, "")
}

/// Types the sentinel into the open prompt and checks it is shown, but not in
/// the status line.
fn typed_is_not_in_the_status_line(term: &mut Terminal, what: &str) -> Screen {
    term.send_text(SENTINEL).unwrap();
    let screen = term
        .wait_for(&format!("the sentinel in the {what}"), STEP, |s| {
            s.contains(SENTINEL)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let status = common::status_line_text(&screen);
    assert!(
        !status.contains(SENTINEL),
        "{what}: the typed text reached the status line:\n{status}\n{}",
        screen.text()
    );
    screen
}

fn palette(term: &mut Terminal, query: &str, row: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains(row))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn leftmost(screen: &Screen, needle: &str) -> (u16, u16) {
    screen
        .find_all(needle)
        .into_iter()
        .min_by_key(|(c, _)| *c)
        .unwrap_or_else(|| panic!("no `{needle}` on screen:\n{}", screen.text()))
}

#[test]
fn a_new_branch_with_the_log_visible() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-branch-log");
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the branch name field", STEP, |s| {
        s.contains("enter branch name")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "branch name");
    let _ = term.stop();
}

#[test]
fn a_new_branch_in_a_diff_view() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-branch-diff");
    let (c, r) = term.wait_for_text("big.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("row 20") && !s.contains("SHA")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the name box", STEP, |s| {
        s.contains("New branch") && s.contains("enter branch name")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "name box");
    let _ = term.stop();
}

#[test]
fn a_stash_message() {
    let repo = fixture(true);
    let (mut term, _c) = launch(&repo, "pty-slnt-stash");
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("m").unwrap();
    term.wait_for("the stash dialog", STEP, |s| {
        s.contains("Stash changes") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "stash dialog");
    let _ = term.stop();
}

#[test]
fn an_annotated_tag_message() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-annotate");
    let sidebar = term
        .wait_for("the tag in the Refs sidebar", STEP, |s| {
            !s.find_all("mine").is_empty()
        })
        .unwrap();
    let (col, row) = leftmost(&sidebar, "mine");
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the tag menu", STEP, |s| s.contains("Annotate mine..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Annotate mine...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the message dialog", STEP, |s| {
        s.contains("Annotate tag mine") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "tag message dialog");
    let _ = term.stop();
}

#[test]
fn an_upstream() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-upstream");
    palette(&mut term, "Git: Set Upstream", "Git: Set Upstream...");
    term.send_keys("Enter").unwrap();
    term.wait_for("the Set upstream dialog", STEP, |s| {
        s.contains("Set upstream of") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "upstream dialog");
    let _ = term.stop();
}

#[test]
fn a_number_setting_from_the_palette() {
    let repo = fixture(false);
    let (mut term, _c) = launch_with(&repo, "pty-slnt-setting", "[ui]\nmouse_scroll_lines = 3\n");
    palette(
        &mut term,
        "settings: mouse wheel",
        "Settings: Mouse wheel lines...",
    );
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings open on the row being edited", STEP, |s| {
        s.contains("Settings - Tab")
            && s.text()
                .lines()
                .any(|l| l.contains("Mouse wheel lines") && l.contains("3\u{2588}"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("BSpace").unwrap();
    typed_is_not_in_the_status_line(&mut term, "settings page");
    let _ = term.stop();
}

#[test]
fn a_new_file_name() {
    let repo = fixture(true);
    let (mut term, _c) = launch(&repo, "pty-slnt-create-file");
    palette(
        &mut term,
        "Working Changes: Create File",
        "Working Changes: Create File",
    );
    term.send_keys("Enter").unwrap();
    term.wait_for("the file name field", STEP, |s| {
        s.contains("file name, e.g. notes.md")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "file name field");
    let _ = term.stop();
}

fn lfs_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.psd")
        .unwrap()
        .commit_lfs_file("big.psd", &vec![1u8; 500], "add big")
        .unwrap()
        .write_worktree("big.psd", &vec![2u8; 4000])
        .unwrap()
        .build()
}

fn launch_lfs(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(160, 50)
        .no_exit_pause()
        .ready("add big")
        .start();
    term.wait_for("the LFS file tagged", STEP, |s| s.contains("big.psd LFS"))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

#[test]
fn an_lfs_track_pattern() {
    require_git_lfs!("an_lfs_track_pattern");
    let repo = lfs_fixture();
    let (mut term, _c) = launch_lfs(&repo, "pty-slnt-lfs-track");
    palette(&mut term, "LFS: Track Pattern", "LFS: Track Pattern...");
    term.send_keys("Enter").unwrap();
    term.wait_for("the track dialog", STEP, |s| {
        s.contains("Track with Git LFS") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "LFS track dialog");
    let _ = term.stop();
}

#[test]
fn an_lfs_untrack_pattern() {
    require_git_lfs!("an_lfs_untrack_pattern");
    let repo = lfs_fixture();
    let (mut term, _c) = launch_lfs(&repo, "pty-slnt-lfs-untrack");
    palette(&mut term, "LFS: Untrack Pattern", "LFS: Untrack Pattern...");
    term.send_keys("Enter").unwrap();
    term.wait_for("the untrack dialog", STEP, |s| {
        s.contains("Stop tracking with Git LFS") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "LFS untrack dialog");
    let _ = term.stop();
}

#[test]
fn a_rename_from_the_refs_row() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-rename-refs");
    let sidebar = term
        .wait_for("the branch in the sidebar", STEP, |s| {
            !s.find_all("to-delete").is_empty()
        })
        .unwrap();
    let (col, row) = leftmost(&sidebar, "to-delete");
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the ref menu", STEP, |s| s.contains("Rename to-delete..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Rename to-delete...").unwrap();
    term.click(c + 1, r).unwrap();
    for _ in 0.."to-delete".len() {
        term.send_keys("BSpace").unwrap();
    }
    typed_is_not_in_the_status_line(&mut term, "Refs row");
    let _ = term.stop();
}

#[test]
fn a_rename_from_the_pill() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-rename-pill");
    let (_, row) = term.wait_for_text("older target commit", STEP).unwrap();
    let col = term
        .screen()
        .find_all("to-delete")
        .into_iter()
        .find(|(c, r)| *r == row && *c > 40)
        .unwrap_or_else(|| panic!("no pill on the row:\n{}", term.text()))
        .0;
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the pill menu", STEP, |s| s.contains("Rename to-delete..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Rename to-delete...").unwrap();
    term.click(c + 1, r).unwrap();
    for _ in 0.."to-delete".len() {
        term.send_keys("BSpace").unwrap();
    }
    typed_is_not_in_the_status_line(&mut term, "Log pill");
    let _ = term.stop();
}

#[test]
fn the_refs_filter() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-slnt-filter");
    term.send_text("/").unwrap();
    term.wait_for("the filter prompt", STEP, |s| {
        common::status_line_text(s).contains("Enter: apply")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    typed_is_not_in_the_status_line(&mut term, "Refs filter box");
    let _ = term.stop();
}

/// The helper finds the status line in every layout, on real screens: the
/// idle line shows `session:`, an open prompt shows its key hint, and neither
/// is found anywhere but the status line.
#[test]
fn the_helper_finds_the_status_line_in_every_layout() {
    for (name, ui) in [
        ("default", ""),
        ("toolbar-top", "toolbar_position = \"top\"\n"),
        ("toolbar-left", "toolbar_position = \"left\"\n"),
        ("status-top", "statusbar_position = \"top\"\n"),
        ("tabs-bottom", "tabs_position = \"bottom\"\n"),
        (
            "status-top-tabs-bottom",
            "statusbar_position = \"top\"\ntabs_position = \"bottom\"\n",
        ),
    ] {
        let repo = fixture(false);
        let (mut term, _c) = launch_with(
            &repo,
            &format!("pty-slnt-layout-{name}"),
            &format!("[ui]\ngraph_renderer = \"text\"\n{ui}"),
        );
        let idle = term.screen();
        assert!(
            common::status_line_text(&idle).contains("session:"),
            "{name}: the idle status line is not found:\n{}",
            idle.text()
        );
        term.send_keys("C-g").unwrap();
        term.wait_for_text("Ctrl+G:", STEP).unwrap();
        term.send_text("b").unwrap();
        term.wait_for("the name field", STEP, |s| s.contains("enter branch name"))
            .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
        let screen = typed_is_not_in_the_status_line(&mut term, name);
        assert!(
            common::status_line_text(&screen).contains("Esc: cancel"),
            "{name}: the prompt's key hint is not in the found status line:\n{}",
            screen.text()
        );
        let _ = term.stop();
    }
}
