//! The folder picker's "Show sibling folders" checkbox: the left tree lists
//! the other folders of every open level (collapsed mark) around the open
//! path (expanded mark). `Alt+S` and a click toggle it; a click on a sibling
//! goes into it.
//!
//! Real binary in a pty; a temp config dir and a temp folder tree only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const COLLAPSED: &str = "\u{25B8} alpha-dir";

fn open_picker_in(dir: &std::path::Path) -> (Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "the only commit")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-folder-siblings")
        .size(160, 40)
        .ready("the only commit")
        .mouse_capture()
        .start();
    // The fixture must outlive the terminal.
    std::mem::forget(repo);
    term.send_keys("C-o").unwrap();
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").unwrap();
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("Open a repository") && s.contains("FOLDERS")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    term.send_text(&dir.display().to_string().replace('\\', "/"))
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder listed", STEP, |s| s.contains("inner"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config)
}

#[test]
fn siblings_show_by_key_and_by_click_and_a_click_on_one_goes_there() {
    let tree = tempfile::tempdir().unwrap();
    std::fs::create_dir(tree.path().join("alpha-dir")).unwrap();
    std::fs::write(tree.path().join("alpha-dir").join("marker.txt"), "x").unwrap();
    std::fs::create_dir_all(tree.path().join("bravo-dir").join("inner")).unwrap();
    let (mut term, _config) = open_picker_in(&tree.path().join("bravo-dir"));

    // Unchecked: the sibling is not in the tree.
    assert!(!term.text().contains("alpha-dir"), "{}", term.text());

    // The key checks it, and shows the checkbox ticked with the fold mark.
    term.send_keys("M-s").unwrap();
    term.wait_for("the sibling, collapsed", STEP, |s| s.contains(COLLAPSED))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("Show sibling folders"));

    // The key again unchecks it.
    term.send_keys("M-s").unwrap();
    term.wait_for("the sibling gone", STEP, |s| !s.contains("alpha-dir"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click on the checkbox checks it.
    let (col, row) = term
        .screen()
        .find("Show sibling folders")
        .unwrap_or_else(|| panic!("no checkbox\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    term.wait_for("the sibling, collapsed", STEP, |s| s.contains(COLLAPSED))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click on the sibling goes into it.
    let (col, row) = term
        .screen()
        .find("alpha-dir")
        .unwrap_or_else(|| panic!("no sibling\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    term.wait_for("the sibling's content", STEP, |s| s.contains("marker.txt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
