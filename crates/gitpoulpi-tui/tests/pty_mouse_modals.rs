//! The two popup lists the event loop's mouse prelude handles by hand (the
//! command palette and the tab picker) answer the same gestures the same
//! way: a click on a row selects it, a double click runs it, a click outside
//! closes, the wheel inside scrolls the list and never reaches what is behind. Plus the tab
//! bar's own mouse.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! temp config dirs and fixture repositories.

mod common;

use std::path::PathBuf;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// Two repositories whose tabs have names of their own (a fixture's root
/// is always `<tempdir>/repo`), the second already open as a tab.
fn two_tabs(session: &str) -> (Terminal, tempfile::TempDir, [FixtureRepo; 2], [PathBuf; 2]) {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("one.txt", "1\n", "the first repositorys own commit")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("two.txt", "2\n", "the second repositorys own commit")
        .unwrap()
        .build();
    let first_root = first.root().parent().unwrap().join("first-repo");
    std::fs::rename(first.root(), &first_root).unwrap();
    let second_root = second.root().parent().unwrap().join("second-repo");
    std::fs::rename(second.root(), &second_root).unwrap();

    let (mut term, config) = common::launch(&first_root)
        .session(session)
        .size(200, 50)
        .ready("the first repositorys own commit")
        .mouse_capture()
        .start();
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
    let path = second_root.display().to_string().replace('\\', "/");
    term.send_text(&path).unwrap();
    let tail: String = path
        .chars()
        .rev()
        .take(12)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    term.wait_for("the path typed", STEP, move |s| s.contains(&tail))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the picker in the folder", STEP, |s| {
        s.contains("two.txt") && s.contains("git repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the second repository open", STEP, |s| {
        s.contains("the second repositorys own commit") && !s.contains("Open a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, [first, second], [first_root, second_root])
}

fn open_picker(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.wait_for_text("select tab", STEP).unwrap();
    term.send_keys("t").unwrap();
    term.wait_for_text("Select Tab (2/2)", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Double-clicking a row of the tab picker switches to that tab and closes it
/// (one click only selects: `pty_popup_lists.rs`).
#[test]
fn double_clicking_a_tab_picker_row_switches_to_that_tab() {
    let (mut term, _c, _repos, _roots) = two_tabs("pty-mouse-tabpicker-row");
    open_picker(&mut term);
    let (c, r) = term
        .wait_for_text("first-repo", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // The picker's list, not the tab bar: the row below the title.
    let screen = term.screen();
    let (_, title_row) = screen.find("Select Tab (2/2)").unwrap();
    let (c, r) = screen
        .find_all("first-repo")
        .into_iter()
        .find(|(_, row)| *row > title_row)
        .unwrap_or((c, r));
    term.double_click(c + 1, r).unwrap();
    term.wait_for("the first repository, the picker closed", STEP, |s| {
        !s.contains("Select Tab") && s.contains("the first repositorys own commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A click outside the picker closes it and stays on the same tab.
#[test]
fn clicking_outside_the_tab_picker_closes_it_without_switching() {
    let (mut term, _c, _repos, _roots) = two_tabs("pty-mouse-tabpicker-away");
    open_picker(&mut term);
    let screen = term.screen();
    // The bottom-left corner: far from a centered popup.
    term.click(0, screen.rows() - 1).unwrap();
    term.wait_for("the picker closed, same tab", STEP, |s| {
        !s.contains("Select Tab") && s.contains("the second repositorys own commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The wheel over the picker is swallowed: the Log behind it does not
/// move, the picker stays open.
#[test]
fn the_wheel_over_the_tab_picker_stays_in_the_picker() {
    let (mut term, _c, _repos, _roots) = two_tabs("pty-mouse-tabpicker-wheel");
    open_picker(&mut term);
    let screen = term.screen();
    let (c, r) = screen.find("Select Tab (2/2)").unwrap();
    term.scroll(true, c + 2, r + 1).unwrap();
    term.scroll(false, c + 2, r + 1).unwrap();
    term.wait_for("the picker still open", STEP, |s| {
        s.contains("Select Tab (2/2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Select Tab", STEP).unwrap();
    term.stop().unwrap();
}

/// The tab bar's mouse: clicking the first tab's label switches to it.
#[test]
fn clicking_a_tab_label_in_the_tab_bar_switches_tabs() {
    let (mut term, _c, _repos, _roots) = two_tabs("pty-mouse-tabbar");
    let screen = term.screen();
    let (c, r) = screen.find("first-repo").expect("the first tab's label");
    term.click(c + 1, r).unwrap();
    term.wait_for("the first repository", STEP, |s| {
        s.contains("the first repositorys own commit")
            && !s.contains("the second repositorys own commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The command palette is the other hand-rolled popup: a click outside
/// closes it, exactly like the tab picker.
#[test]
fn clicking_outside_the_command_palette_closes_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("palette outside")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-mouse-palette-away")
        .size(200, 50)
        .ready("palette outside")
        .mouse_capture()
        .start();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    let screen = term.screen();
    term.click(0, screen.rows() - 1).unwrap();
    term.wait_for_text_to_go("Command Palette", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Text and background of every cell: the palette's selection is a
/// background color, which `Screen::text` does not show.
fn look(s: &xtask::pty::Screen) -> Vec<(String, xtask::pty::Color)> {
    let mut cells = Vec::new();
    for row in 0..s.rows() {
        for col in 0..s.cols() {
            if let Some(c) = s.cell(col, row) {
                cells.push((c.display().to_string(), c.bg));
            }
        }
    }
    cells
}

/// The wheel over the palette scrolls its list and not the Log behind.
#[test]
fn the_wheel_over_the_command_palette_scrolls_its_list() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("palette wheel")
        .unwrap()
        .build();
    let (mut term, _c) = common::launch(repo.root())
        .session("pty-mouse-palette-wheel")
        .size(200, 50)
        .ready("palette wheel")
        .mouse_capture()
        .start();
    term.send_keys("C-p").unwrap();
    let before = term
        .wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap();
    let (c, r) = before.find("Command Palette").unwrap();
    term.scroll(true, c + 4, r + 3).unwrap();
    term.wait_for("the list scrolled", STEP, |s| {
        s.contains("Command Palette") && look(s) != look(&before)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
