//! Settings > Repository > Sparse checkout, through a pty: the real binary,
//! daemon and git. Enable with a path typed in the page's editor, the work
//! tree only holds that path, Reapply restores a deleted file, Disable brings
//! everything back.
//!
//! No hard-coded coordinates: every position is found on the screen.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn layered() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("README.md", "root\n", "sparse-page-root")
        .unwrap()
        .commit_file("src/main.rs", "fn main() {}\n", "sparse-page-src")
        .unwrap()
        .commit_file("docs/guide.md", "guide\n", "sparse-page-docs")
        .unwrap()
        .commit_file("lib/util.rs", "// util\n", "sparse-page-lib")
        .unwrap()
        .build()
}

fn open_page(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let (mut term, config) = common::launch(repo)
        .session(session)
        .size(200, 60)
        .ready("sparse-page-lib")
        .start();
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Sparse checkout");
    (term, config)
}

fn click_text(term: &mut Terminal, text: &str) {
    let screen = term
        .wait_for(&format!("`{text}` on screen"), STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 1, row).unwrap();
}

/// A click on an inline button presses it.
fn press_button(term: &mut Terminal, text: &str) {
    click_text(term, text);
}

/// Answers a destructive button's question with its confirming button
/// (Cancel is focused, so Right then Enter).
fn confirm_yes(term: &mut Terminal) {
    term.wait_for("the confirmation", STEP, |s| s.contains("undone"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
}

fn wait_fs(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// The page lists its field and its three buttons, Disable and Reapply grey
/// while sparse checkout is off. Typing `src` in the editor and pressing
/// Enable leaves only `src` (and the root files) on disk; the button then
/// reads "Update". Reapply takes a hand-restored file out again; Disable restores the
/// whole tree.
#[test]
fn the_sparse_page_enables_reapplies_and_disables() {
    let repo = layered();
    let (mut term, _config) = open_page(repo.root(), "pty-repo-settings-sparse");
    term.wait_for("the page rows", STEP, |s| {
        s.contains("Directories to keep")
            && s.contains("Start sparse checkout")
            && !s.contains("Stop sparse checkout")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(repo.root().join("docs/guide.md").is_file());

    // The field opens the multi-line editor.
    click_text(&mut term, "Directories to keep");
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("src").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the editor closed with the path in the field", STEP, |s| {
        !s.contains("Cancel") && s.contains("src")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    press_button(&mut term, "Start sparse checkout");
    wait_fs(&term, "docs and lib are gone", || {
        !repo.root().join("docs").exists() && !repo.root().join("lib").exists()
    });
    assert!(repo.root().join("src/main.rs").is_file(), "src stays");
    assert!(repo.root().join("README.md").is_file(), "root files stay");
    term.wait_for("the button now updates", STEP, |s| {
        s.contains("Apply directory list") && !s.contains("Start sparse checkout")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Reapply: a file outside the list that got checked out by hand (a merge
    // or a script can do the same) is taken out again.
    repo.git(&["update-index", "--no-skip-worktree", "docs/guide.md"]);
    repo.git(&["checkout", "--", "docs/guide.md"]);
    assert!(repo.root().join("docs/guide.md").is_file());
    press_button(&mut term, "Refresh sparse checkout");
    wait_fs(&term, "docs/guide.md is taken out again", || {
        !repo.root().join("docs/guide.md").exists()
    });

    press_button(&mut term, "Stop sparse checkout");
    confirm_yes(&mut term);
    wait_fs(&term, "everything is back", || {
        repo.root().join("docs/guide.md").is_file() && repo.root().join("lib/util.rs").is_file()
    });
    term.wait_for("the page offers Enable again", STEP, |s| {
        s.contains("Start sparse checkout") && !s.contains("Apply directory list")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
