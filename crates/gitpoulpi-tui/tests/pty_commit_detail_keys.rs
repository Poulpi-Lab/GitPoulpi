//! The commit-detail pane through the real binary: `m` / `a` from the Log,
//! and the `Ctrl+N` badges on its sub-controls (both regressed once under
//! unit tests alone).

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("Filter files:")
        })
        .mouse_capture()
        .start()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("zdir/inner.txt", b"inner\n")
        .unwrap()
        .write_worktree("top.txt", b"top\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add pane files")
        .unwrap()
        .build()
}

/// The badge glyph painted just before `tail`, e.g. ` 4 ter files:` --
/// three cells, space + glyph + space, ending where `tail` starts.
fn badge_before(screen: &Screen, tail: &str) -> Option<char> {
    let (col, row) = screen.find(tail)?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let i = col as usize;
    if i >= 3 && line[i - 3] == ' ' && line[i - 1] == ' ' && line[i - 2] != ' ' {
        Some(line[i - 2])
    } else {
        None
    }
}

#[test]
fn m_and_a_work_from_the_log_with_a_commit_selected() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cd-keys-ma");
    term.wait_for("Path mode lists zdir/inner.txt", STEP, |s| {
        s.contains("zdir/inner.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("m").unwrap();
    term.wait_for("Tree mode folds the path into a folder row", STEP, |s| {
        !s.contains("zdir/inner.txt") && s.contains("zdir")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("m").unwrap();
    term.wait_for("Path mode again", STEP, |s| s.contains("zdir/inner.txt"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("a").unwrap();
    term.wait_for("View all files ticked", STEP, |s| {
        s.contains("[x] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("a").unwrap();
    term.wait_for("View all files unticked", STEP, |s| {
        s.contains("[ ] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn palette_commands_toggle_the_commit_files_list() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cd-keys-palette");
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle View All").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("View all files ticked via the palette", STEP, |s| {
        s.contains("[x] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn ctrl_n_badges_the_commit_detail_sub_controls_and_they_act() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cd-keys-badges");
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("badges on the commit-detail sub-controls", STEP, |s| {
            badge_before(s, "ter files:").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let filter = badge_before(&screen, "ter files:").unwrap();
    let path_tree = badge_before(&screen, "Path ").expect("Path/Tree badge");
    let view_all = badge_before(&screen, "[ ] View all").expect("View all files badge");

    term.send_keys(&view_all.to_string()).unwrap();
    term.wait_for("View all files ticked by its badge", STEP, |s| {
        s.contains("[x] View all files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").unwrap();
    term.wait_for("badges again", STEP, |s| badge_before(s, "Path ").is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys(&path_tree.to_string()).unwrap();
    term.wait_for("Tree mode by its badge", STEP, |s| {
        s.contains("[x] View all files") && !s.contains("zdir/inner.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").unwrap();
    term.wait_for("badges a third time", STEP, |s| {
        badge_before(s, "ter files:").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys(&filter.to_string()).unwrap();
    term.send_text("top").unwrap();
    term.wait_for("the filter field took the typing", STEP, |s| {
        s.contains("Filter files: top")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
