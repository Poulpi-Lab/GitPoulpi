//! The folder picker's filter field narrows the folder shown as you type
//! (case-insensitive substring, `..` kept), shows "x of n", empties on a
//! folder change, and is reached by `Tab`, `/` / `Ctrl+F` or a click.
//!
//! Real binary in a pty; a temp config dir and a temp folder tree only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn open_picker_in(browse_dir: &std::path::Path) -> (Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "the only commit")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-folder-filter")
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
    let path = browse_dir.display().to_string().replace('\\', "/");
    term.send_text(&path).unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder listed", STEP, |s| {
        s.contains("alpha-dir") && s.contains("notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config)
}

#[test]
fn the_filter_field_narrows_the_listing_and_counts_the_matches() {
    let tree = tempfile::tempdir().unwrap();
    for name in ["alpha-dir", "bravo-dir", "charlie-dir"] {
        std::fs::create_dir(tree.path().join(name)).unwrap();
    }
    std::fs::write(tree.path().join("notes.txt"), "x").unwrap();
    let (mut term, _config) = open_picker_in(tree.path());

    // After going to the typed folder the path field has the keyboard:
    // Tab lands on the filter field.
    term.send_keys("Tab").unwrap();
    term.send_text("BRAV").unwrap();
    term.wait_for("only bravo left, counted", STEP, |s| {
        s.contains(&dir("bravo-dir"))
            && !s.contains(&dir("alpha-dir"))
            && !s.contains(&dir("charlie-dir"))
            && s.contains("1 match of 4")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc empties the filter (and keeps the dialog open).
    term.send_keys("Escape").unwrap();
    term.wait_for("everything back", STEP, |s| {
        s.contains(&dir("alpha-dir"))
            && s.contains(&dir("charlie-dir"))
            && s.contains("Open a repository")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A second Esc only leaves the (empty) field.
    term.send_keys("Escape").unwrap();
    term.wait_for("the placeholder back", STEP, |s| {
        s.contains("Open a repository") && s.text().contains("filter  ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click on the field focuses it: typing filters again.
    let screen = term.screen();
    let (_, new_folder_row) = screen.find("New folder").expect("the toolbar row");
    let (col, _) = screen
        .find_all("filter")
        .into_iter()
        .find(|(_, r)| *r == new_folder_row)
        .unwrap_or_else(|| panic!("no filter field\n{}", term.text()));
    term.click(col + 1, new_folder_row).unwrap();
    term.send_text("char").unwrap();
    term.wait_for("only charlie left", STEP, |s| {
        s.contains(&dir("charlie-dir"))
            && !s.contains(&dir("alpha-dir"))
            && !s.contains(&dir("bravo-dir"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Enter jumps to the first match; `Ctrl+F` comes back to the field.
    term.send_keys("Enter").unwrap();
    term.send_keys("C-f").unwrap();
    term.send_keys("C-u").unwrap();
    term.send_text("note").unwrap();
    term.wait_for("only the file left", STEP, |s| {
        s.contains("notes.txt") && !s.contains(&dir("charlie-dir"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A folder row as the picker lists it: the name and the platform separator.
fn dir(name: &str) -> String {
    format!("{name}{}", std::path::MAIN_SEPARATOR)
}
