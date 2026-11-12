//! The commit column and the editor: the column belongs to the views of a
//! commit's file, so it is gone in the editor (which edits the working
//! copy) and back when the editor closes onto the diff; the right panel
//! stays resizable both ways and the resize pointer is taken down after a
//! drag; picking another commit's file leaves the editor instead of
//! stacking a view under it.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
/// The column's title, with the spaces it is framed by.
const TITLE: &str = " Commits ";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(180, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("first.txt")
        })
        .mouse_capture()
        .start()
}

/// HEAD adds `first.txt` and `second.txt`, over a commit that added `a.txt`.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "alpha\n", "add a")
        .unwrap()
        .write_worktree("first.txt", b"delta\n")
        .unwrap()
        .write_worktree("second.txt", b"epsilon\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("add two")
        .unwrap()
        .build()
}

/// The text rightmost on the screen: the right panel's file list.
fn rightmost(term: &Terminal, text: &str) -> (u16, u16) {
    term.screen()
        .find_all(text)
        .into_iter()
        .max_by_key(|(c, _)| *c)
        .unwrap_or_else(|| panic!("{text:?} is not on the screen\n{}", term.text()))
}

/// Opens `file`'s diff from the right panel's list (HEAD's).
fn open_diff(term: &mut Terminal, file: &str) {
    // Two clicks on one cell within the double click window are a double
    // click: a person is slower than this test.
    std::thread::sleep(Duration::from_millis(450));
    let (c, r) = rightmost(term, file);
    term.click(c, r).unwrap();
}

fn edit_d(term: &mut Terminal) {
    open_diff(term, "first.txt");
    term.wait_for("first.txt's diff", STEP, |s| s.contains("+delta"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("e").unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - first.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Column of the right panel's left border, on its "Commit details" row.
/// Read from a frame that shows it whole (a torn frame can miss the title or
/// the border).
fn panel_left(term: &Terminal) -> u16 {
    fn left_of(s: &xtask::pty::Screen) -> Option<u16> {
        let (c, r) = s.find("Commit details")?;
        let line: Vec<char> = s.line(r).chars().collect();
        (0..c as usize)
            .rev()
            .find(|&i| line.get(i) == Some(&'\u{256d}'))
            .map(|i| i as u16)
    }
    let screen = term
        .wait_for("the right panel's left border", STEP, |s| {
            left_of(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    left_of(&screen).unwrap()
}

/// Every `OSC 22` pointer shape the program asked for, in order ("" is the
/// reset).
fn pointer_shapes(term: &Terminal) -> Vec<String> {
    let raw = String::from_utf8_lossy(&term.raw_output()).to_string();
    raw.split("\x1b]22;")
        .skip(1)
        .map(|s| s.split("\x1b\\").next().unwrap_or("").to_string())
        .collect()
}

/// The editor edits the working copy, so a commit's column has no place
/// beside it; closing the editor brings the diff and the column back.
/// Red when the column is kept in the editor.
#[test]
fn the_editor_has_no_commit_column_and_esc_brings_it_back() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ec-gone");
    edit_d(&mut term);
    term.wait_for("the column gone", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the diff with its column", STEP, |s| {
        s.contains("+delta") && s.contains(TITLE) && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The right panel's divider follows the pointer in the editor in both
/// directions, as it does in the diff. Red when a drag that crosses the
/// editor's body is eaten by the editor (the panel only ever shrank).
#[test]
fn the_right_panel_resizes_both_ways_in_the_editor() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ec-resize");
    edit_d(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    let row = term.screen().find("Commit details").unwrap().1 + 5;
    for delta in [-20i32, 10, -20] {
        let before = panel_left(&term);
        let to = (before as i32 + delta) as u16;
        term.drag((before, row), (to, row)).unwrap();
        // The divider lands on the pointer's percentage, a cell or two off.
        let near = |x: u16| (x as i32 - to as i32).abs() <= 3;
        let deadline = std::time::Instant::now() + STEP;
        while !near(panel_left(&term)) && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(
            near(panel_left(&term)),
            "dragged {delta} from {before}\n{}",
            term.text()
        );
    }
    term.stop().unwrap();
}

/// The resize pointer a drag showed is taken down by its release, even
/// when the drag ended far from the divider (the limit) over the editor.
/// Red when the release never reaches the drag handler.
#[test]
fn the_resize_pointer_is_reset_after_a_drag_released_over_the_editor() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ec-pointer");
    edit_d(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    let row = term.screen().find("Commit details").unwrap().1 + 5;
    let x = panel_left(&term);
    term.move_to(x, row).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !pointer_shapes(&term).iter().any(|s| s == "ew-resize")
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        pointer_shapes(&term).iter().any(|s| s == "ew-resize"),
        "hovering the divider asks for the resize pointer: {:?}",
        pointer_shapes(&term)
    );
    // Far past the limit: the divider stops well short of the pointer.
    term.drag((x, row), (30, row)).unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while pointer_shapes(&term).last().map(String::as_str) != Some("")
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        pointer_shapes(&term).last().map(String::as_str),
        Some(""),
        "the pointer is back to the arrow: {:?}",
        pointer_shapes(&term)
    );
    term.stop().unwrap();
}

/// Picking another commit's file while editing leaves the editor and shows
/// that file's diff, with its column; one Esc then closes it to the Log
/// (no view stays hidden under the editor). Red when the editor stays over
/// the new file's view.
#[test]
fn picking_another_commit_file_leaves_the_editor_for_its_diff() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ec-pick");
    edit_d(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    let (c, r) = rightmost(&term, "second.txt");
    term.click(c, r).unwrap();
    term.wait_for("second.txt's diff, no editor, the column back", STEP, |s| {
        s.contains("+epsilon") && s.contains(TITLE) && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // One close: the Log, not another view underneath.
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log", STEP, |s| {
        !s.contains("+epsilon") && !s.contains("+delta") && !s.contains("Edit - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Unsaved edits are asked about first (Save / Discard / Cancel); Cancel
/// stays in the editor with the edits, the column still gone.
#[test]
fn picking_a_commit_file_with_unsaved_edits_asks_first() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ec-dirty");
    edit_d(&mut term);
    term.send_text("zz").unwrap();
    term.wait_for("dirty", STEP, |s| s.contains("\u{25CF} modified"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = rightmost(&term, "second.txt");
    term.click(c, r).unwrap();
    term.wait_for("the prompt", STEP, |s| {
        s.contains("Save changes to first.txt?") && s.contains("Discard")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("Cancel keeps the edits, no column", STEP, |s| {
        s.contains("Edit - first.txt @ working tree")
            && s.contains("zzdelta")
            && !s.contains("Save changes to")
            && !s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Discard: the editor goes, second.txt's diff shows.
    let (c, r) = rightmost(&term, "second.txt");
    term.click(c + 2, r).unwrap();
    term.wait_for("the prompt again", STEP, |s| {
        s.contains("Save changes to first.txt?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("d").unwrap();
    term.wait_for("second.txt's diff", STEP, |s| {
        s.contains("+epsilon") && !s.contains("Edit - ") && !s.contains("Save changes to")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("first.txt")).unwrap(),
        b"delta\n"
    );
    term.stop().unwrap();
}
