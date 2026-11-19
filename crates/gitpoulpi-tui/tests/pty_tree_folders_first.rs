//! `ui.tree_folders_first`: in the tree view of a file pane a folder lists its
//! sub-folders before its files (default on); off keeps the plain path order.
//! Keyboard navigation walks the rows in the order they are drawn.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);

/// `alpha.txt` beside the folders `zeta/` and `beta/`, all modified.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("alpha.txt", "1\n", "seed alpha")
        .unwrap()
        .commit_file("zeta/f.txt", "2\n", "seed zeta")
        .unwrap()
        .commit_file("beta/g.txt", "3\n", "seed beta")
        .unwrap()
        .write_worktree("alpha.txt", b"1 changed\n")
        .unwrap()
        .write_worktree("zeta/f.txt", b"2 changed\n")
        .unwrap()
        .write_worktree("beta/g.txt", b"3 changed\n")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config_toml: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    if !config_toml.is_empty() {
        std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    }
    let mut term = common::launch(repo)
        .session("tree-folders-first")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("seed beta"))
        .spawn();
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).unwrap();
    term.wait_for("the changed files", STEP, rows_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config)
}

/// The rows of `alpha.txt`, `zeta/` and `beta/` in the right pane.
fn rows(s: &Screen) -> Option<(u16, u16, u16)> {
    let pick = |label: &str| {
        s.find_all(label)
            .into_iter()
            .filter(|(c, _)| *c > 60)
            .map(|(_, r)| r)
            .min()
    };
    Some((pick("alpha.txt")?, pick("zeta/")?, pick("beta/")?))
}

fn rows_shown(s: &Screen) -> bool {
    rows(s).is_some()
}

#[test]
fn folders_come_before_files_by_default() {
    let repo = fixture();
    let (term, _config) = launch(repo.root(), "");
    let (alpha, zeta, beta) = rows(&term.screen()).unwrap();
    assert!(
        beta < zeta && zeta < alpha,
        "beta/ {beta}, zeta/ {zeta}, alpha.txt {alpha}\n{}",
        term.text()
    );
}

#[test]
fn turning_the_setting_off_restores_the_path_order() {
    let repo = fixture();
    let (term, _config) = launch(repo.root(), "[ui]\ntree_folders_first = false\n");
    let (alpha, zeta, beta) = rows(&term.screen()).unwrap();
    assert!(
        alpha < beta && beta < zeta,
        "alpha.txt {alpha}, beta/ {beta}, zeta/ {zeta}\n{}",
        term.text()
    );
}

/// The Reflog panel's file list in tree view, with Settings opened over it
/// (from the palette) and the toggle flipped while the panel stays open: its
/// cached rows must follow the new order.
#[test]
fn the_safety_panel_tree_follows_the_setting_while_open() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("seed.txt", "0\n", "initial commit")
        .unwrap()
        .build();
    let root = repo.root();
    for (path, text) in [
        ("alpha.txt", "1\n"),
        ("zeta/f.txt", "2\n"),
        ("beta/g.txt", "3\n"),
    ] {
        std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        std::fs::write(root.join(path), text).unwrap();
        common::git(root, &["add", path]);
    }
    common::git(root, &["commit", "-m", "second commit"]);

    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(root)
        .session("tree-folders-first-safety")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| s.contains("second commit"))
        .spawn();
    // Reflog, newest position (the second commit), file list as a tree.
    term.send_keys("M-z").unwrap();
    term.wait_for("the Undo history", STEP, |s| {
        s.contains("can be undone") || s.contains("Undo history")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("2").unwrap();
    term.wait_for("the reflog's files", STEP, |s| {
        s.contains("commit: second commit") && s.contains("alpha.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("v").unwrap();
    let order = |s: &Screen| -> Option<(u16, u16, u16)> {
        let pick = |label: &str| s.find_all(label).into_iter().map(|(_, r)| r).min();
        Some((pick("alpha.txt")?, pick("zeta/")?, pick("beta/")?))
    };
    let s = term
        .wait_for("the tree, folders first", STEP, |s| {
            order(s).is_some_and(|(a, z, b)| b < z && z < a)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(s);

    // Settings over the open panel, the toggle off.
    term.send_keys("C-p").unwrap();
    term.send_text("settings").unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains("Settings"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::settings_page(&mut term, "Log & graph");
    let (c, r) = common::scroll_settings_to(&mut term, "Folders first in the tree view", STEP);
    term.click(c, r).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
        if stored.contains("tree_folders_first = false") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "not stored: {stored}");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").unwrap();
    term.wait_for("the tree in path order", STEP, |s| {
        order(s).is_some_and(|(a, z, b)| a < b && b < z)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
