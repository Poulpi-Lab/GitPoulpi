//! The commit column left of the Diff / File View / Blame / Edit views of a
//! commit's file: shown from 140 terminal columns, picking a commit moves
//! the view and the right panel along, and the choice and the width are
//! remembered.

mod common;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// The column's title, with the spaces it is framed by.
const TITLE: &str = " Commits ";

fn launch(repo: &Path, session: &str, cols: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(cols, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("c.txt")
        })
        .mouse_capture()
        .start()
}

/// `ui.column_resize = "header"`: the first letter of the next header word is
/// the grab, no tick, no line.
const HEADER_MODE: &str = "[ui]\ncolumn_resize = \"header\"\n";

fn launch_header(repo: &Path, session: &str, cols: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(HEADER_MODE)
        .size(cols, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("c.txt")
        })
        .mouse_capture()
        .start()
}

/// The same, in a config directory that outlives the run (a restart).
fn launch_in(repo: &Path, config: &Path, session: &str, cols: u16) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(cols, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("c.txt")
        })
        .mouse_capture()
        .spawn()
}

/// Five commits: `a.txt` is added, then `b.txt`, then both are edited, and
/// `c.txt` is added last (so HEAD touches neither a nor b).
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "alpha\n", "add a")
        .unwrap()
        .commit_file("b.txt", "beta\n", "add b")
        .unwrap()
        .commit_file("a.txt", "alpha\nALPHA-TWO\n", "edit a")
        .unwrap()
        .commit_file("b.txt", "beta\nBETA-THREE\n", "edit b")
        .unwrap()
        .commit_file("c.txt", "gamma\n", "add c")
        .unwrap()
        .build()
}

/// The text rightmost on the screen: the right panel's file list, never the
/// view's title or the column.
fn rightmost(term: &Terminal, text: &str) -> (u16, u16) {
    term.screen()
        .find_all(text)
        .into_iter()
        .max_by_key(|(c, _)| *c)
        .unwrap_or_else(|| panic!("{text:?} is not on the screen\n{}", term.text()))
}

/// Opens `file`'s diff from the right panel (the selected commit's list).
fn open_from_the_list(term: &mut Terminal, file: &str) {
    // Two clicks on one cell within the double click window are a double
    // click (the second is ignored): a person is slower than this test.
    std::thread::sleep(Duration::from_millis(450));
    let (c, r) = rightmost(term, file);
    term.click(c, r).unwrap();
}

/// Selects the Log row `commit` (the Log is the central view), then opens
/// `file` from the commit's list.
fn open_file_at(term: &mut Terminal, commit: &str, file: &str) {
    let (c, r) = term
        .wait_for_text(commit, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the commit's files", STEP, |s| {
        s.contains("Changed files") && s.find_all(file).into_iter().any(|(col, _)| col > 100)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_from_the_list(term, file);
}

/// Where the column starts: the left cell of its frame, on its title row.
fn column_origin(screen: &Screen) -> Option<(u16, u16)> {
    let (c, r) = screen.find(TITLE)?;
    Some((c.saturating_sub(1), r))
}

/// Where the column's frame ends (its right border), on its title row.
fn column_right(screen: &Screen) -> Option<u16> {
    let (x0, row) = column_origin(screen)?;
    let line: Vec<char> = screen.line(row).chars().collect();
    line.iter()
        .enumerate()
        .skip(x0 as usize)
        .find(|(_, c)| **c == '\u{256e}')
        .map(|(i, _)| i as u16)
}

/// What the column shows, one line per row (not the view or the panel).
fn column_text(screen: &Screen) -> String {
    let Some((x0, title_row)) = column_origin(screen) else {
        return String::new();
    };
    let x1 = column_right(screen).unwrap_or(x0 + 30);
    (title_row + 1..screen.rows())
        .map(|row| {
            screen
                .line(row)
                .chars()
                .skip(x0 as usize)
                .take((x1 - x0) as usize)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The cell of `text` inside the column (not in the view or the panel).
fn in_column(term: &Terminal, text: &str) -> (u16, u16) {
    let screen = term.screen();
    let (x0, title_row) =
        column_origin(&screen).unwrap_or_else(|| panic!("no commit column\n{}", term.text()));
    let x1 = column_right(&screen).unwrap_or(x0 + 30);
    for row in title_row + 1..screen.rows() {
        let line: Vec<char> = screen.line(row).chars().collect();
        let seg: String = line
            .iter()
            .skip(x0 as usize)
            .take((x1 - x0) as usize)
            .collect();
        if let Some(i) = seg.find(text) {
            let col = x0 + seg[..i].chars().count() as u16;
            return (col, row);
        }
    }
    panic!(
        "{text:?} is not a row of the commit column\n{}",
        term.text()
    );
}

fn pick(term: &mut Terminal, commit: &str) {
    let (c, r) = in_column(term, commit);
    term.click(c, r).unwrap();
}

/// The saved session layout, as text.
fn session_text(config: &Path) -> String {
    std::fs::read_dir(config.join("sessions"))
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

fn wait_for_saved(config: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if session_text(config).contains(needle) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{needle:?} was not saved:\n{}", session_text(config));
}

fn open_head_file(term: &mut Terminal) {
    open_from_the_list(term, "c.txt");
    term.wait_for("c.txt's diff", STEP, |s| s.contains("+gamma"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Shown on its own from 140 columns, with the Log's commits in the Log's
/// order, one line each; hidden (and the view whole) below.
/// Red when the width rule is dropped (the column never shows, or always).
#[test]
fn the_column_shows_from_140_columns_and_stays_hidden_below() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-wide", 140);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The Log's own commits, newest first, one per row.
    let rows: Vec<u16> = ["add c", "edit b", "edit a", "add b", "add a"]
        .iter()
        .map(|t| in_column(&term, t).1)
        .collect();
    assert!(
        rows.windows(2).all(|w| w[0] + 1 == w[1]),
        "one line each, in the Log's order: {rows:?}\n{}",
        term.text()
    );
    // The view keeps a usable width beside it.
    assert!(term.text().contains("c.txt"), "{}", term.text());
    term.stop().unwrap();

    let (mut term, _config) = launch(repo.root(), "pty-cc-narrow", 139);
    open_head_file(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !term.text().contains(TITLE),
        "below 140 columns the column is hidden:\n{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The key and the palette command flip it, on any width, and the choice
/// is remembered across a restart.
#[test]
fn the_toggle_overrides_the_width_and_is_remembered() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-toggle", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-c").unwrap();
    term.wait_for("the column hidden", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column = 2");
    term.stop().unwrap();

    // A restart on the same wide terminal: still hidden, as chosen.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-toggle", 180);
    open_head_file(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    assert!(!term.text().contains(TITLE), "{}", term.text());
    // The palette command brings it back.
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Commit Column").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("View: Toggle Commit Column")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the column back", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column = 1");
    term.stop().unwrap();

    // On a narrow terminal the same toggle turns it on (a window wide
    // enough for the view and the column still), and it stays on.
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-toggle-narrow", 130);
    open_head_file(&mut term);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!term.text().contains(TITLE), "{}", term.text());
    term.send_keys("M-c").unwrap();
    term.wait_for("the column on a narrow terminal", STEP, |s| {
        s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The cross on the column closes it, and it stays closed.
#[test]
fn the_cross_closes_the_column_and_it_stays_closed() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-cross", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, row) = column_origin(&term.screen()).unwrap();
    let right = column_right(&term.screen()).unwrap();
    // The cross sits on the column's top border, left of its corner.
    let cross = term
        .screen()
        .find_all("\u{2715}")
        .into_iter()
        .find(|(c, r)| *r == row && *c < right)
        .unwrap_or_else(|| panic!("no cross on the column\n{}", term.text()));
    term.click(cross.0, cross.1).unwrap();
    term.wait_for("the column closed", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The view is still open (the cross closed the column, not the view).
    assert!(term.text().contains("+gamma"), "{}", term.text());
    wait_for_saved(config.path(), "commit_column = 2");
    term.stop().unwrap();
}

/// Picking another commit in the column moves the diff to that commit's
/// diff of the file, and the right panel to that commit's details and file
/// list. Red when the view or the panel stays on the first commit.
#[test]
fn picking_a_commit_changes_the_diff_and_the_detail_panels_file_list() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-pick", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();

    // The first commit added the file: its diff is the whole file.
    pick(&mut term, "add a");
    term.wait_for("a.txt at add a", STEP, |s| {
        s.contains("+alpha") && !s.contains("+ALPHA-TWO")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The panel follows: that commit's message and its one file.
    let screen = term.screen();
    assert!(
        screen.find_all("add a").into_iter().any(|(c, _)| c > 100),
        "the detail panel shows the picked commit\n{}",
        term.text()
    );
    assert!(
        screen.find_all("a.txt").into_iter().any(|(c, _)| c > 100),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// A file the picked commit did not touch stays open, as it was there, with
/// a notice; the panel lists the commit's own files, and one of them opens
/// from there. Red when the notice or the content is missing.
#[test]
fn an_unchanged_file_shows_as_it_was_with_the_notice_and_another_file_opens() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-unchanged", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // "edit b" did not touch a.txt: the file as it was there.
    pick(&mut term, "edit b");
    term.wait_for("the notice and the file as it was", STEP, |s| {
        s.contains("unchanged in this commit") && s.contains("ALPHA-TWO")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The panel lists that commit's file, b.txt; one click opens its diff.
    assert!(rightmost(&term, "b.txt").0 > 100, "{}", term.text());
    open_from_the_list(&mut term, "b.txt");
    term.wait_for("b.txt's diff at edit b", STEP, |s| {
        s.contains("+BETA-THREE") && !s.contains("unchanged in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A file the picked commit does not have says so and the view stays usable:
/// another file of that commit opens from the panel.
/// Red when the view is left blank, or keeps the old commit's content.
#[test]
fn a_file_missing_at_the_picked_commit_says_so_and_the_view_stays_usable() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-missing", 180);
    open_file_at(&mut term, "edit b", "b.txt");
    term.wait_for("b.txt at edit b", STEP, |s| s.contains("+BETA-THREE"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // b.txt does not exist before "add b": the first commit has only a.txt.
    pick(&mut term, "add a");
    term.wait_for("the missing notice", STEP, |s| {
        s.contains("does not exist in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("BETA-THREE"), "{}", term.text());
    // Still usable: open the commit's own file.
    open_from_the_list(&mut term, "a.txt");
    term.wait_for("a.txt's diff at add a", STEP, |s| {
        s.contains("+alpha") && !s.contains("does not exist in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The column's border moves by dragging, by the palette's Widen / Narrow,
/// and the width is remembered across a restart.
#[test]
fn the_width_is_resized_by_drag_and_by_key_and_remembered() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-resize", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();
    let start = column_right(&term.screen()).unwrap();

    // Palette: Widen, then Narrow twice.
    let palette = |term: &mut Terminal, text: &str| {
        term.send_keys("C-p").unwrap();
        term.send_text(text).unwrap();
        term.wait_for("the palette row", STEP, |s| s.contains(text))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
        let text = text.to_string();
        term.wait_for("the palette closed", STEP, move |s| !s.contains(&text))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    };
    palette(&mut term, "Widen the Commit Column");
    term.wait_for("wider", STEP, |s| {
        column_right(s).is_some_and(|r| r > start)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let wide = column_right(&term.screen()).unwrap();
    palette(&mut term, "Narrow the Commit Column");
    palette(&mut term, "Narrow the Commit Column");
    term.wait_for("narrower", STEP, |s| {
        column_right(s).is_some_and(|r| r < start)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Drag the border (the view's left edge, under its bar row) to the right.
    let now = column_right(&term.screen()).unwrap();
    let (_, row) = column_origin(&term.screen()).unwrap();
    term.drag((now + 1, row + 6), (now + 11, row + 6)).unwrap();
    term.wait_for("the drag widened it", STEP, |s| {
        column_right(s).is_some_and(|r| r >= now + 10)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let dragged = column_right(&term.screen()).unwrap();
    assert!(dragged > wide || dragged >= now + 10);
    wait_for_saved(config.path(), "commit_column_cols");
    term.stop().unwrap();

    // The same width after a restart.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-resize", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();
    assert_eq!(
        column_right(&term.screen()),
        Some(dragged),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// `Ctrl+N` badges the column; its badge gives it the arrows, which pick a
/// commit (the view and the panel follow), and Enter goes on to the panel's
/// list. Red when the column has no badge or the arrows move the diff.
#[test]
fn the_nav_badge_gives_the_column_the_arrows() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-badge", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();
    term.send_keys("C-n").unwrap();
    // The badge is painted over the start of the column's title.
    let screen = term
        .wait_for("a badge on the column", STEP, |s| column_badge(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let badge = column_badge(&screen).unwrap();
    term.send_keys(&badge.to_string()).unwrap();

    // Down: the next older commit, "add b", which left a.txt as "add a" had it.
    term.send_keys("Down").unwrap();
    term.wait_for("the view followed the arrow", STEP, |s| {
        s.contains("unchanged in this commit") && !s.contains("ALPHA-TWO")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Up: back on "edit a", the diff again.
    term.send_keys("Up").unwrap();
    term.wait_for("back on edit a", STEP, |s| {
        s.contains("+ALPHA-TWO") && !s.contains("unchanged in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Up again: "edit b" left a.txt as it was.
    term.send_keys("Up").unwrap();
    term.wait_for("edit b", STEP, |s| {
        s.contains("unchanged in this commit") && s.contains("ALPHA-TWO")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The badge character painted over the column's title (`1 mmits`).
fn column_badge(screen: &Screen) -> Option<char> {
    let (col, row) = screen.find("mmits")?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let i = col as usize;
    (i >= 3 && line[i - 3] == ' ' && line[i - 1] == ' ' && line[i - 2] != ' ').then(|| line[i - 2])
}

/// A click on a row gives the column the arrows too (the keyboard twin of
/// the click is the badge, then the arrows).
#[test]
fn a_click_on_a_row_then_the_arrows_walk_the_commits() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-click-arrows", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap();
    pick(&mut term, "edit b");
    term.wait_for("edit b", STEP, |s| s.contains("unchanged in this commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("j").unwrap();
    term.wait_for("edit a again", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The editor opened from a commit's file has no column (it edits the
/// working copy, which no commit's column serves), stays on the working
/// copy, and closing it brings the diff and the column back.
#[test]
fn the_editor_drops_the_column_and_stays_on_the_working_copy() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-editor", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("e").unwrap();
    term.wait_for("the editor without the column", STEP, |s| {
        s.contains("Edit - a.txt @ working tree") && !s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(term.text().contains("ALPHA-TWO"), "{}", term.text());
    term.send_keys("Escape").unwrap();
    term.wait_for("the diff and the column back", STEP, |s| {
        s.contains(TITLE) && s.contains("+ALPHA-TWO") && !s.contains("Edit - a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A Blame stays a Blame at the picked commit (as of that commit), with the
/// notice when the commit left the file alone.
#[test]
fn blame_stays_blame_at_the_picked_commit() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-blame", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The bar's Blame button sits in the overflow beside the column: the
    // palette's command opens it.
    term.send_keys("C-p").unwrap();
    term.send_text("Diff: Blame").unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains("Diff: Blame"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the blame", STEP, |s| s.contains("Blame - a.txt @"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    pick(&mut term, "edit b");
    term.wait_for("blame at edit b", STEP, |s| {
        s.contains("Blame - a.txt @") && s.contains("unchanged in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A commit that changed the file: the notice goes.
    pick(&mut term, "edit a");
    term.wait_for("blame at edit a", STEP, |s| {
        s.contains("Blame - a.txt @") && !s.contains("unchanged in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A repository of `n` commits that each edit `f.txt`, built in one
/// `fast-import`.
fn big_repo(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp repo");
    common::git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for i in 1..=n {
        let msg = format!("commit-{i:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{i}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + i * 60,
            msg.len()
        ));
        if i > 1 {
            stream.push_str(&format!("from :{}\n", i - 1));
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\nline {i}\n\n",
            format!("line {i}").len() + 1
        ));
    }
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn fast-import");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    common::git_ok(dir.path(), &["reset", "-q", "--hard", "main"]);
    dir
}

/// The wheel scrolls the hovered column, and scrolling it to its end loads
/// the Log's next pages (the Log is not on screen to ask for them).
/// Red when the column's end never asks for more commits.
#[test]
fn the_wheel_scrolls_the_column_and_its_end_loads_more_commits() {
    let repo = big_repo(620);
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ninitial_commits_in_graph = 500\nlazy_load_commits_in_graph = true\n\
         commits_per_lazy_page = 500\nmax_commits_in_graph = 2000\n",
    )
    .unwrap();
    let mut term = common::launch(repo.path())
        .session("pty-cc-scroll")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("f.txt")
        })
        .mouse_capture()
        .spawn();
    open_from_the_list(&mut term, "f.txt");
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x, row) = in_column(&term, "commit-0620");
    // The wheel moves the list: the first row leaves the screen.
    for _ in 0..10 {
        term.scroll(true, x + 4, row + 3).unwrap();
    }
    term.wait_for("the list scrolled", STEP, |s| {
        let list = column_text(s);
        !list.contains("commit-0620") && list.contains("commit-0590")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Focus it and go to the end: the first 500 commits end at 0121, the
    // pages after it reach the root.
    term.send_keys("C-n").unwrap();
    let screen = term
        .wait_for("a badge on the column", STEP, |s| column_badge(s).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys(&column_badge(&screen).unwrap().to_string())
        .unwrap();
    term.send_keys("End").unwrap();
    term.wait_for("the first page's last commit", STEP, |s| {
        column_text(s).contains("commit-0121")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The next page arrives on its own, and End reaches the root commit.
    let deadline = std::time::Instant::now() + STEP;
    while !column_text(&term.screen()).contains("commit-0001")
        && std::time::Instant::now() < deadline
    {
        term.send_keys("End").unwrap();
        std::thread::sleep(Duration::from_millis(300));
    }
    assert!(
        column_text(&term.screen()).contains("commit-0001"),
        "the column loaded the rest of the history\n{}",
        term.text()
    );
    term.stop().unwrap();
}

/// Where the reopen strip's chevron is: the cell right of the central
/// column's left edge (`x0`), the one `\u{203A}` of that column.
fn strip_chevron(screen: &Screen, x0: u16) -> Option<(u16, u16)> {
    // `find_all` reports one hit per row, and the collapsed Refs pane's own
    // chevron (column 1) would shadow ours: look at the one cell.
    (0..screen.rows()).find_map(|row| {
        let at = screen.line(row).chars().nth(usize::from(x0) + 1)?;
        (at == '\u{203A}').then_some((x0 + 1, row))
    })
}

/// Closed by its cross, the column comes back by the strip left of the view
/// (mouse) and by Alt+C (keyboard), and each choice is remembered. Red when
/// the strip is not painted or its click does nothing (the mouse has no way
/// back), or the key stops reopening it.
#[test]
fn a_closed_column_comes_back_by_the_strip_and_by_the_key() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reopen", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x0, row) = column_origin(&term.screen()).unwrap();
    assert!(
        strip_chevron(&term.screen(), x0).is_none(),
        "no strip while the column is shown\n{}",
        term.text()
    );
    let right = column_right(&term.screen()).unwrap();
    let cross = term
        .screen()
        .find_all("\u{2715}")
        .into_iter()
        .find(|(c, r)| *r == row && *c < right)
        .unwrap_or_else(|| panic!("no cross on the column\n{}", term.text()));
    term.click(cross.0, cross.1).unwrap();
    term.wait_for("the column closed", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column = 2");

    // The strip is where the column was, and a click on it brings it back.
    let screen = term
        .wait_for("the strip", STEP, |s| strip_chevron(s, x0).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = strip_chevron(&screen, x0).unwrap();
    std::thread::sleep(Duration::from_millis(450));
    term.click(c, r).unwrap();
    term.wait_for("the column back", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column = 1");
    assert!(
        strip_chevron(&term.screen(), x0).is_none(),
        "the strip goes with the hidden column\n{}",
        term.text()
    );
    // The view is still the diff.
    assert!(term.text().contains("+gamma"), "{}", term.text());

    // Hide it by the key, bring it back by the key.
    term.send_keys("M-c").unwrap();
    term.wait_for("hidden by the key", STEP, |s| {
        !s.contains(TITLE) && strip_chevron(s, x0).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-c").unwrap();
    term.wait_for("back by the key", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The strip names its key: the tooltip after a rest on it carries Alt+C
/// from the keymap.
#[test]
fn the_strips_tooltip_names_the_key() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reopen-tip", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x0, _) = column_origin(&term.screen()).unwrap();
    term.send_keys("M-c").unwrap();
    let screen = term
        .wait_for("the strip", STEP, |s| {
            !s.contains(TITLE) && strip_chevron(s, x0).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = strip_chevron(&screen, x0).unwrap();
    term.move_to(c, r).unwrap();
    term.wait_for("the tooltip", STEP, |s| {
        s.contains("Show the commit column (Alt+C)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The bar's `File | All` selector, as the cell its first character (the
/// space before `File`) is in.
const SELECTOR: &str = " File \u{2502} All ";

fn selector_at(screen: &Screen) -> Option<(u16, u16)> {
    screen.find(SELECTOR)
}

/// A click on the selector's `File` segment.
fn click_file_segment(term: &mut Terminal) {
    let (col, row) =
        selector_at(&term.screen()).unwrap_or_else(|| panic!("the selector\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(450));
    term.click(col + 3, row).unwrap();
}

/// A click on the selector's `All` segment.
fn click_all_segment(term: &mut Terminal) {
    let (col, row) =
        selector_at(&term.screen()).unwrap_or_else(|| panic!("the selector\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(450));
    term.click(col + 8, row).unwrap();
}

/// A Ctrl+click on the selector's `All` segment: no longer special, it is a
/// plain click.
fn ctrl_click_all_segment(term: &mut Terminal) {
    let (col, row) =
        selector_at(&term.screen()).unwrap_or_else(|| panic!("the selector\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(450));
    term.ctrl_click(col + 8, row).unwrap();
}

/// The file's history list shows (its title), the graph column does not.
fn history_shown(s: &Screen) -> bool {
    s.contains(" History - c.txt ") && !s.contains(TITLE)
}

/// The bar never moves: the selector is on the same cell with the graph
/// shown, hidden, in the file history and hidden again, and so is the bar's
/// next button. Red when the column is cut out of the bar's row again (the
/// bar then starts where the view does, and shifts by the column's width).
#[test]
fn the_bar_buttons_do_not_move_when_the_column_opens_closes_or_changes_list() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-bar-still", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let home =
        selector_at(&term.screen()).unwrap_or_else(|| panic!("the selector\n{}", term.text()));
    let edit_at = |s: &Screen| s.find(" Edit ");
    let edit = edit_at(&term.screen());
    assert!(edit.is_some(), "{}", term.text());
    // The column starts under the bar row, not beside it.
    let (_, title_row) = column_origin(&term.screen()).unwrap();
    assert!(title_row > home.1, "{}", term.text());

    term.send_keys("M-c").unwrap();
    term.wait_for("hidden", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(selector_at(&term.screen()), Some(home), "{}", term.text());
    assert_eq!(edit_at(&term.screen()), edit, "{}", term.text());

    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(selector_at(&term.screen()), Some(home), "{}", term.text());
    assert_eq!(edit_at(&term.screen()), edit, "{}", term.text());

    term.send_keys("M-c").unwrap();
    term.wait_for("the list hidden", STEP, |s| {
        !s.contains(" History - c.txt ") && !s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(selector_at(&term.screen()), Some(home), "{}", term.text());
    assert_eq!(edit_at(&term.screen()), edit, "{}", term.text());
    term.stop().unwrap();
}

/// A click on a segment picks that list and shows the column (opening it
/// when it was closed); the pick is remembered. Red when a segment only
/// highlights, or the pick is not saved.
#[test]
fn a_segment_click_picks_the_list_and_opens_the_column() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-segments", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Closed, then `All` opens the graph.
    term.send_keys("M-c").unwrap();
    term.wait_for("hidden", STEP, |s| !s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_all_segment(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // `File` swaps it for the file's history.
    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column_mode = 1");
    // And `All` back.
    click_all_segment(&mut term);
    term.wait_for("the graph column again", STEP, |s| {
        s.contains(TITLE) && !s.contains(" History - c.txt ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column_mode = 0");
    // The view stays the file's diff throughout.
    assert!(term.text().contains("+gamma"), "{}", term.text());
    term.stop().unwrap();
}

/// A click on the selected segment while the column is shown hides it and
/// keeps the list; a click on a segment while hidden shows the column with
/// that list; Ctrl+click is a plain click. The pick survives a restart (Alt+C
/// then opens that list). Red when a re-click only re-selects, the hidden
/// click does not show, Ctrl+click still toggles, or the pick is forgotten.
#[test]
fn reclicking_the_selected_segment_hides_the_column_and_keeps_the_list() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reclick", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let hidden = |s: &Screen| !s.contains(" History - c.txt ") && !s.contains(TITLE);
    // The other segment switches; the selected one hides; hidden, a click
    // shows that list.
    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("hidden", STEP, hidden)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("the file history again", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The same with the graph, and a click on the other segment while hidden.
    click_all_segment(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_all_segment(&mut term);
    term.wait_for("hidden", STEP, hidden)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("the file history, picked while hidden", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Ctrl+click on the other segment switches the list, it does not hide.
    ctrl_click_all_segment(&mut term);
    term.wait_for("the graph by Ctrl+click", STEP, |s| {
        s.contains(TITLE) && !s.contains(" History - c.txt ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Hidden in the file's mode, remembered across a restart.
    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("hidden", STEP, hidden)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column_mode = 1");
    wait_for_saved(config.path(), "commit_column = 2");
    term.stop().unwrap();

    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reclick", 180);
    open_head_file(&mut term);
    std::thread::sleep(Duration::from_millis(500));
    assert!(!term.text().contains(TITLE), "{}", term.text());
    term.send_keys("M-c").unwrap();
    term.wait_for("the file history by the key", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The list picked on the selector is remembered: the next file opened, after
/// the view was closed, shows that list at once, and so does a restart. Red
/// when the saved mode is only applied by Alt+C (a fresh file opens on the
/// graph again), or `All` is not what a later pick of `All` leaves behind.
#[test]
fn the_picked_list_applies_to_the_next_file_and_survives_a_restart() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-remember", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column_mode = 1");

    // Close the view, open another file: its history, not the graph.
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log back", STEP, |s| {
        !s.contains(" History - c.txt ") && !s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt's history", STEP, |s| {
        s.contains(" History - a.txt ") && !s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();

    // A restart: the same list on the first file opened.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-remember", 180);
    open_from_the_list(&mut term, "c.txt");
    term.wait_for("the file history after a restart", STEP, history_shown)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Picking `All` is remembered the same way.
    click_all_segment(&mut term);
    term.wait_for("the graph column", STEP, |s| {
        s.contains(TITLE) && !s.contains(" History - c.txt ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_saved(config.path(), "commit_column_mode = 0");
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log back", STEP, |s| {
        !s.contains(TITLE) && s.contains("Refs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("the graph for the next file", STEP, |s| {
        s.contains(TITLE) && !s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The selector's tooltip names the click, the re-click and the key.
#[test]
fn the_selectors_tooltip_names_the_click_the_reclick_and_the_key() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-selector-tip", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = selector_at(&term.screen()).unwrap();
    term.move_to(col + 3, row).unwrap();
    term.wait_for("the tooltip", STEP, |s| {
        s.contains("Click: show")
            && s.contains("again: hide the column")
            && !s.contains("Ctrl+click")
            && s.contains("Alt+C")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The palette keeps its three entries for the column: hide, the file's
/// history, all the commits.
#[test]
fn the_palette_reaches_hide_file_history_and_all_commits() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-palette", 180);
    open_head_file(&mut term);
    term.wait_for("the graph column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for (row, done) in [
        ("Commit Column: Hide", "Commit column: off"),
        ("Commit Column: File History", "Commit column: file"),
        ("Commit Column: All Commits", "Commit column: graph"),
    ] {
        term.send_keys("C-p").unwrap();
        term.send_text(row).unwrap();
        term.wait_for("the palette row", STEP, |s| s.contains(row))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
        term.wait_for(done, STEP, |s| {
            !s.contains("Command Palette") && (0..s.rows()).any(|r| s.line(r).contains(done))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.stop().unwrap();
}

/// While the splitter is dragged, its highlight is on the border where it is
/// now, never where the press started: the stale bar inside the view's text.
/// Red when the strip lit at the press is not re-lit with the drag.
#[test]
fn the_drag_highlight_follows_the_column_splitter() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-hot", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();
    let (_, row) = column_origin(&term.screen()).unwrap();
    let start = column_right(&term.screen()).unwrap() + 1;
    let y = row + 6;
    term.press(start, y).unwrap();
    term.drag_to(start + 8, y).unwrap();
    term.drag_to(start + 16, y).unwrap();
    term.wait_for("the column widened", STEP, |s| {
        column_right(s).is_some_and(|r| r + 1 >= start + 12)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let now = column_right(&term.screen()).unwrap() + 1;
    let screen = term.screen();
    let lit = screen.cell(now, y).unwrap().bg;
    assert_ne!(
        lit,
        screen.cell(now + 6, y).unwrap().bg,
        "the new border is lit"
    );
    for r in row + 1..row + 12 {
        assert_ne!(
            screen.cell(start, r).unwrap().bg,
            lit,
            "a stale highlight at the old x on row {r}\n{}",
            term.text()
        );
    }
    term.release(start + 16, y).unwrap();
    term.stop().unwrap();
}

/// A release produces no motion, so the highlight a drag left on the splitter
/// is recomputed at the release point: released away from the border, nothing
/// stays lit. Red when the release keeps the dragged border lit.
#[test]
fn the_highlight_is_gone_after_a_drag_released_away_from_the_splitter() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-unlit", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap();
    let (_, row) = column_origin(&term.screen()).unwrap();
    let start = column_right(&term.screen()).unwrap() + 1;
    let y = row + 6;
    term.press(start, y).unwrap();
    term.drag_to(start + 8, y).unwrap();
    term.drag_to(start + 16, y).unwrap();
    term.wait_for("the column widened", STEP, |s| {
        column_right(s).is_some_and(|r| r + 1 >= start + 12)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let now = column_right(&term.screen()).unwrap() + 1;
    let lit = term.screen().cell(now, y).unwrap().bg;
    term.release(now + 12, y + 3).unwrap();
    term.wait_for("the border is no longer lit", STEP, |s| {
        s.cell(now, y).is_some_and(|c| c.bg != lit)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

// ---- the commit rows: tooltip and menu -------------------------------------

/// The same five commits as [`fixture`], two of them with a body that only
/// the Log's tooltip (the whole message) shows: the column lists the summary.
fn tipped_fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "alpha\n", "add a\n\nBODY-ADD-A the first file")
        .unwrap()
        .commit_file("b.txt", "beta\n", "add b")
        .unwrap()
        .commit_file(
            "a.txt",
            "alpha\nALPHA-TWO\n",
            "edit a\n\nBODY-EDIT-A the second edit",
        )
        .unwrap()
        .commit_file("b.txt", "beta\nBETA-THREE\n", "edit b")
        .unwrap()
        .commit_file("c.txt", "gamma\n", "add c")
        .unwrap()
        .build()
}

/// Hovering a commit row of the column shows the Log's tooltip for it: the
/// whole message (the column lists the summary alone). Red when the column's
/// rows carry no tooltip.
#[test]
fn hovering_a_commit_row_of_the_column_shows_the_logs_tooltip() {
    let repo = tipped_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-row-tip", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("BODY-EDIT-A"), "{}", term.text());
    let (c, r) = in_column(&term, "edit a");
    term.move_to(c + 1, r).unwrap();
    term.wait_for("the message tooltip", STEP, |s| {
        s.contains("BODY-EDIT-A the second edit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The date part has the Log's date tooltip too (the full time).
    let (_, row) = in_column(&term, "edit a");
    let screen = term.screen();
    let (x0, _) = column_origin(&screen).unwrap();
    let right = column_right(&screen).unwrap();
    term.move_to(right - 2, row).unwrap();
    term.wait_for("the date tooltip", STEP, |s| !s.contains("BODY-EDIT-A"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(right > x0);
    term.stop().unwrap();
}

/// The same for a row of the File History's list (the column's File list):
/// the first line of a row is the message, the Log's tooltip.
#[test]
fn hovering_a_row_of_the_file_history_list_shows_the_logs_tooltip() {
    let repo = tipped_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-hist-tip", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_file_segment(&mut term);
    term.wait_for("the file history", STEP, |s| {
        s.contains(" History - a.txt ") && s.contains("add a")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("BODY-ADD-A"), "{}", term.text());
    // The row of "add a" in the list (left of the diff).
    let (c, r) = term
        .screen()
        .find_all("add a")
        .into_iter()
        .find(|(c, _)| *c < 60)
        .unwrap_or_else(|| panic!("the row\n{}", term.text()));
    term.move_to(c + 1, r).unwrap();
    term.wait_for("the message tooltip", STEP, |s| {
        s.contains("BODY-ADD-A the first file")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A right-click on a commit row of the column opens the menu: the views of
/// the open file at that commit, then the Log's own rows (the destructive
/// ones after a rule); one row works: Blame at that commit.
/// Red when there is no menu, or the Blame row does not blame at the row's
/// commit.
#[test]
fn a_right_click_on_a_commit_row_opens_its_menu_and_a_row_works() {
    let repo = tipped_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-menu", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = in_column(&term, "edit b");
    term.right_click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the menu", STEP, |s| s.contains("Blame at this commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for row in [
        "Show in the Log",
        "Show the diff of this commit",
        "Open the file at this commit",
        "Cherry-pick",
        "Revert",
    ] {
        assert!(screen.contains(row), "{row:?}\n{}", term.text());
    }
    // The views come first, the Log's rows (the destructive ones included)
    // after them.
    let line_of = |text: &str| {
        (0..screen.rows())
            .find(|r| screen.line(*r).contains(text))
            .unwrap()
    };
    assert!(line_of("Blame at this commit") < line_of("Cherry-pick"));
    assert!(line_of("Cherry-pick") < line_of("Revert"));
    let (bc, br) = screen.find("Blame at this commit").unwrap();
    term.click(bc + 2, br).unwrap();
    term.wait_for("the blame at edit b", STEP, |s| {
        s.contains("Blame - a.txt @") && s.contains("unchanged in this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The panel followed: the right-clicked commit is the selected one.
    assert!(
        term.screen()
            .find_all("edit b")
            .into_iter()
            .any(|(c, _)| c > 100),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The menu key opens the same menu from the keyboard once the column has the
/// keys (a click on a row, or the nav badge): anchored on the selected row,
/// and a row picked with the arrows works. Red when the key does nothing in
/// the column.
#[test]
fn the_menu_key_opens_the_rows_menu_from_the_keyboard() {
    let repo = tipped_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-menu-key", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A click on a row gives the column the keys and selects it: "add a"
    // added the file, so its view there is a diff (all added lines).
    pick(&mut term, "add a");
    term.wait_for("the diff at add a", STEP, |s| {
        s.contains("+alpha") && !s.contains("+ALPHA-TWO")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("S-F10").unwrap();
    term.wait_for("the menu", STEP, |s| {
        s.contains("Open the file at this commit") && s.contains("Cherry-pick")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Down twice (Show in the Log, the diff, then the file), Enter: the
    // file as it was there, not the diff the commit made of it.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the file as it was at add a", STEP, |s| {
        s.contains("File View - a.txt @") && s.contains("1 alpha") && !s.contains("+alpha")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The palette reaches the menu of the selected commit and its rows.
#[test]
fn the_palette_reaches_the_rows_menu_and_its_views() {
    let repo = tipped_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-menu-palette", 180);
    open_file_at(&mut term, "edit a", "a.txt");
    term.wait_for("a.txt at edit a", STEP, |s| s.contains("+ALPHA-TWO"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.send_text("Menu of the Selected Row").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Menu of the Selected Row")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the menu", STEP, |s| s.contains("Blame at this commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| {
        !s.contains("Blame at this commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A view of the selected commit by its own palette row.
    term.send_keys("C-p").unwrap();
    term.send_text("Blame the File at the Selected Row")
        .unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Blame the File at the Selected Row")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the blame", STEP, |s| s.contains("Blame - a.txt @"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

// ---- the rows' sideways scroll and the parts' widths -----------------------

/// The column's header row: the row under its title.
fn header_row(screen: &Screen) -> Option<u16> {
    column_origin(screen).map(|(_, r)| r + 1)
}

/// Where the header word `word` starts, inside the column.
fn header_word(screen: &Screen, word: &str) -> Option<u16> {
    let (x0, _) = column_origin(screen)?;
    let right = column_right(screen)?;
    let line: Vec<char> = screen.line(header_row(screen)?).chars().collect();
    let text: String = line
        .iter()
        .skip(usize::from(x0))
        .take(usize::from(right - x0))
        .collect();
    text.find(word)
        .map(|i| x0 + text[..i].chars().count() as u16)
}

/// The header's rules (the boundaries of the parts), left to right, inside
/// the column's frame.
fn header_rules(screen: &Screen) -> Vec<u16> {
    let Some((x0, _)) = column_origin(screen) else {
        return Vec::new();
    };
    let Some(right) = column_right(screen) else {
        return Vec::new();
    };
    let Some(row) = header_row(screen) else {
        return Vec::new();
    };
    screen
        .line(row)
        .chars()
        .enumerate()
        .filter(|(i, c)| *c == '\u{2502}' && *i as u16 > x0 && (*i as u16) < right)
        .map(|(i, _)| i as u16)
        .collect()
}

/// Gives the column the keys (a click on a row) and widens the author until
/// the row is wider than the room: `Tab` twice picks the author (the lanes,
/// then the message come first), `]` widens it.
fn widen_the_author(term: &mut Terminal) {
    pick(term, "add b");
    term.wait_for("the column has the keys", STEP, |s| {
        s.contains("unchanged in this commit") || s.contains("+gamma") || s.contains("add b")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    for _ in 0..12 {
        term.send_keys("]").unwrap();
    }
    term.wait_for("the author is wide", STEP, |s| {
        s.contains("Author width: 32 cells")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

// ---- the header's boundaries, its parts menu, the Refs part ----------------

/// Runs a palette command and waits for the palette to close.
fn run_palette(term: &mut Terminal, text: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(text).unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The `[ui]` table of the config file (the column's parts are global
/// settings there, beside the Log's columns).
fn config_ui(config: &Path) -> toml::Table {
    let text = std::fs::read_to_string(config.join("config.toml")).unwrap_or_default();
    let doc: toml::Table = toml::from_str(&text).unwrap_or_default();
    doc.get("ui")
        .and_then(|u| u.as_table())
        .cloned()
        .unwrap_or_default()
}

fn config_list(config: &Path, key: &str) -> Vec<String> {
    config_ui(config)
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// The saved part widths, name -> cells.
fn saved_widths(config: &Path) -> std::collections::BTreeMap<String, i64> {
    config_ui(config)
        .get("commit_column_columns_widths")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_integer().map(|n| (k.clone(), n)))
                .collect()
        })
        .unwrap_or_default()
}

/// Waits until the config's list `key` has an entry.
fn wait_for_config_list(config: &Path, key: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if !config_list(config, key).is_empty() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{key:?} was not saved:\n{:?}", config_ui(config));
}

/// Waits until the config holds a width for some part.
fn wait_for_saved_widths(config: &Path) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if !saved_widths(config).is_empty() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("no width was saved:\n{:?}", config_ui(config));
}

/// The fixture with a branch on the commit before HEAD, short enough for its
/// pill to fit the Refs part whole.
fn refs_fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = fixture();
    repo.git(&["branch", "zed", "HEAD~1"]);
    repo
}

/// The cell of the header word's first letter.
fn first_letter(screen: &Screen, word: &str) -> (u16, u16) {
    (
        header_word(screen, word).unwrap_or_else(|| panic!("no {word:?} in the header")),
        header_row(screen).unwrap(),
    )
}

/// Hovering the first letter of a header word lights that letter (the Log's
/// way: the boundary is the next column's first letter), draws no rule
/// anywhere in the header, and the light goes with the pointer.
/// Red when the boundary is a separator cell, or a rule is drawn.
#[test]
fn hovering_a_boundary_lights_the_first_letter_and_draws_no_rule() {
    let repo = fixture();
    let (mut term, _config) = launch_header(repo.root(), "pty-cc-letter", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert!(header_rules(&screen).is_empty(), "{}", term.text());
    let (c, r) = first_letter(&screen, "Author");
    let plain = screen.cell(c, r).unwrap().bg;
    assert_eq!(screen.cell(c, r).unwrap().text, "A");
    term.move_to(c, r).unwrap();
    term.wait_for("the first letter is lit", STEP, |s| {
        s.cell(c, r).is_some_and(|x| x.bg != plain && x.text == "A")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Only that letter: the next one is not.
    let screen = term.screen();
    assert_eq!(screen.cell(c + 1, r).unwrap().bg, plain, "{}", term.text());
    assert!(header_rules(&screen).is_empty(), "{}", term.text());
    term.move_to(c, r + 6).unwrap();
    term.wait_for("the light is gone", STEP, |s| {
        s.cell(c, r).is_some_and(|x| x.bg == plain)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Dragging a boundary sizes the part on its left, the message included
/// (it has a width of its own now): the next part's first letter follows the
/// pointer and stays lit, nothing stale is left where the press was, the
/// other parts keep their widths (they shift), and the five widths are saved.
/// Red when the message cannot be sized or the highlight is left behind.
#[test]
fn a_boundary_drag_sizes_the_message_with_its_highlight_and_is_remembered() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), HEADER_MODE).unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-parts", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let author0 = header_word(&screen, "Author").unwrap();
    let date0 = header_word(&screen, "Date").unwrap();
    let row = header_row(&screen).unwrap();

    // Press on the author's first letter, walk it six cells to the left.
    term.press(author0, row).unwrap();
    term.drag_to(author0 - 3, row).unwrap();
    term.drag_to(author0 - 6, row).unwrap();
    term.wait_for("the message is narrower", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(6)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert_eq!(
        header_word(&screen, "Date"),
        date0.checked_sub(6),
        "the author kept its width, the date moved with it\n{}",
        term.text()
    );
    let now = author0 - 6;
    let lit = screen.cell(now, row).unwrap().bg;
    assert_eq!(screen.cell(now, row).unwrap().text, "A");
    assert_ne!(
        lit,
        screen.cell(now + 2, row).unwrap().bg,
        "the boundary being dragged is lit\n{}",
        term.text()
    );
    assert_ne!(
        screen.cell(author0, row).unwrap().bg,
        lit,
        "a stale highlight where the press was\n{}",
        term.text()
    );
    term.release(now, row).unwrap();
    wait_for_saved_widths(config.path());
    let saved = saved_widths(config.path());
    assert!(
        saved.get("message").is_some_and(|w| *w > 0),
        "the message has a width of its own, and every part is pinned: {saved:?}"
    );
    assert_eq!(saved.len(), 4, "{saved:?}");
    let session = session_text(config.path());
    assert!(!session.contains("commit_column_widths"), "{session}");
    term.stop().unwrap();

    // The widths come back after a restart.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-parts", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        header_word(&term.screen(), "Author"),
        author0.checked_sub(6),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The keys size the message like any part: `Tab` goes lanes, message,
/// author; `]` widens the message and the author's name moves right with it.
/// Red when the message is skipped by `Tab` or has no width of its own.
#[test]
fn the_keys_size_the_message_part() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-msg-keys", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let author0 = header_word(&term.screen(), "Author").unwrap();
    pick(&mut term, "add b");
    term.send_keys("Tab").unwrap();
    term.wait_for("the message is the part to size", STEP, |s| {
        s.contains("Message width:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("]").unwrap();
    term.send_keys("]").unwrap();
    term.wait_for("the author moved four cells right", STEP, |s| {
        header_word(s, "Author") == Some(author0 + 4)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("0").unwrap();
    term.wait_for("back to automatic", STEP, |s| {
        header_word(s, "Author") == Some(author0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A right-click on the header opens the menu of the parts; a click on a row
/// hides or shows that part, and the choice is remembered across a restart.
/// Red when the right-click opens nothing, a row does nothing, or the choice
/// is not saved.
#[test]
fn a_right_click_on_the_header_chooses_the_parts_and_it_is_remembered() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-menu", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = first_letter(&term.screen(), "Date");
    term.right_click(c, r).unwrap();
    term.wait_for("the parts menu", STEP, |s| {
        s.contains("Columns") && s.contains("[ ] Ref") && s.contains("[x] Date")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The message is listed, not selectable.
    assert!(term.text().contains("[x] Message"), "{}", term.text());
    let (dc, dr) = term.screen().find("[x] Date").unwrap();
    term.click(dc + 2, dr).unwrap();
    term.wait_for("Date is gone from the header", STEP, |s| {
        header_word(s, "Date").is_none() && header_word(s, "Author").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_config_list(config.path(), "commit_column_columns_visible");
    term.stop().unwrap();

    let mut term = launch_in(repo.root(), config.path(), "pty-cc-menu", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        header_word(&term.screen(), "Date").is_none(),
        "{}",
        term.text()
    );
    assert!(header_word(&term.screen(), "Author").is_some());
    term.stop().unwrap();
}

/// The keyboard reaches the same: the palette's `Choose the Columns` opens
/// the menu (arrows and Enter pick a row), and its per-part toggles flip a
/// part at once. Red when the palette has no such command.
#[test]
fn the_palette_chooses_the_parts_like_the_header_menu() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-menu-keys", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A toggle.
    run_palette(&mut term, "Column: Toggle Author");
    term.wait_for("the author is hidden", STEP, |s| {
        header_word(s, "Author").is_none() && s.contains("Author hidden")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    run_palette(&mut term, "Column: Toggle Author");
    term.wait_for("the author is back", STEP, |s| {
        header_word(s, "Author").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The menu, by the keys: Date is the third selectable row (the message is skipped).
    run_palette(&mut term, "Column: Choose the Columns");
    term.wait_for("the parts menu", STEP, |s| s.contains("[x] Date"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the date is hidden", STEP, |s| {
        header_word(s, "Date").is_none() && !s.contains("[x] Date")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The Refs part is an option of the menu: shown, it leads the rows with the
/// commit's branch label (the Log's pill), and hiding it takes it away.
/// Red when the part is missing, empty, or on by default.
#[test]
fn the_refs_part_shows_the_branch_labels() {
    let repo = refs_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-refs", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        header_word(&term.screen(), "Ref").is_none(),
        "off by default"
    );
    assert!(!column_text(&term.screen()).contains("zed"));
    run_palette(&mut term, "Column: Toggle Ref");
    term.wait_for("the Refs part with the branch", STEP, |s| {
        header_word(s, "Ref").is_some() && column_text(s).contains("zed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // It leads the row, before the message.
    let screen = term.screen();
    assert!(header_word(&screen, "Ref") < header_word(&screen, "Message"));
    run_palette(&mut term, "Column: Toggle Ref");
    term.wait_for("the Refs part is gone", STEP, |s| {
        header_word(s, "Ref").is_none() && !column_text(s).contains("zed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The Log's header line (the one with its Graph and Message columns), when
/// the Log is on screen.
fn log_header(screen: &Screen) -> Option<String> {
    (0..screen.rows())
        .map(|r| screen.line(r))
        .find(|l| l.contains("Message") && l.contains("Graph"))
}

/// The column's parts are its own, apart from the Log's columns: hiding a
/// part in one never touches the other, in both directions.
/// Red when the two share their state.
#[test]
fn the_commit_columns_parts_are_apart_from_the_logs_columns() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-apart", 180);
    // The Log first: hide its Author column (the palette's toggle).
    run_palette(&mut term, "View: Toggle Author Column");
    term.wait_for("the Log has no Author", STEP, |s| {
        log_header(s).is_some_and(|h| !h.contains("Author") && h.contains("Date"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        header_word(&term.screen(), "Author").is_some(),
        "the Log's choice does not hide the column's part\n{}",
        term.text()
    );
    // Now the column: hide its Date, leave the Log.
    run_palette(&mut term, "Column: Toggle Date");
    term.wait_for("the column has no Date", STEP, |s| {
        header_word(s, "Date").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_config_list(config.path(), "commit_column_columns_visible");
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log again", STEP, |s| {
        !s.contains(TITLE) && log_header(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let header = log_header(&term.screen()).unwrap();
    assert!(
        header.contains("Date"),
        "the Log keeps its Date\n{}",
        term.text()
    );
    assert!(
        !header.contains("Author"),
        "the Log keeps its own choice\n{}",
        term.text()
    );
    term.stop().unwrap();
    // On disk: both lists are the config's, each under its own key, and the
    // session holds neither.
    let ui = config_ui(config.path());
    assert!(
        !config_list(config.path(), "commit_column_columns_visible").contains(&"date".to_string()),
        "{ui:?}"
    );
    let deadline = std::time::Instant::now() + STEP;
    while !config_list(config.path(), "log_columns_hidden").contains(&"author".to_string())
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(100));
    }
    let log_hidden = config_list(config.path(), "log_columns_hidden");
    assert!(
        log_hidden.contains(&"author".to_string()) && !log_hidden.contains(&"date".to_string()),
        "the Log's list holds only the Log's choice: {ui:?}"
    );
    let session = session_text(config.path());
    assert!(!session.contains("commit_column_visible"), "{session}");
    assert!(!session.contains("log_columns_hidden"), "{session}");
}

/// The column may take up to 70% of the central column: dragged far right on
/// a very wide window it goes past the 100 cells it used to stop at.
/// Red when the cap is the old flat 100.
#[test]
fn the_column_may_be_wider_than_a_hundred_cells_on_a_wide_window() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-wide", 320);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x0, row) = column_origin(&term.screen()).unwrap();
    let start = column_right(&term.screen()).unwrap() + 1;
    let y = row + 6;
    term.press(start, y).unwrap();
    term.drag_to(start + 60, y).unwrap();
    term.drag_to(start + 140, y).unwrap();
    term.release(start + 140, y).unwrap();
    term.wait_for("wider than the old cap", STEP, |s| {
        column_right(s).is_some_and(|r| r + 1 - x0 > 100)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let width = column_right(&term.screen()).unwrap() + 1 - x0;
    // And the diff keeps its room: the column stops at 70% of the central
    // column, never the whole of it.
    assert!(width < 220, "{width}\n{}", term.text());
    assert!(term.text().contains("+gamma"), "{}", term.text());
    term.stop().unwrap();
}

/// The keys and the palette size the author and reset the widths.
#[test]
fn the_keys_and_the_palette_size_the_parts_and_reset_them() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-part-keys", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let author0 = header_word(&term.screen(), "Author").unwrap();
    pick(&mut term, "add b");
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.wait_for("the author is the part to size", STEP, |s| {
        s.contains("Author width: 8 cells")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("]").unwrap();
    term.send_keys("]").unwrap();
    term.wait_for("the author is four cells wider", STEP, |s| {
        s.contains("Author width: 12 cells")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("[").unwrap();
    term.wait_for("two cells narrower again", STEP, |s| {
        s.contains("Author width: 10 cells")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("0").unwrap();
    term.wait_for("back to automatic", STEP, |s| {
        header_word(s, "Author") == Some(author0) && s.contains("automatic widths")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The palette: widen the chosen part, then reset.
    for (text, done) in [
        ("Widen the Part", "Author width"),
        ("Reset the Part Widths", "automatic widths"),
    ] {
        term.send_keys("C-p").unwrap();
        term.send_text(text).unwrap();
        term.wait_for("the palette row", STEP, |s| s.contains(text))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
        term.wait_for(done, STEP, |s| {
            !s.contains("Command Palette") && s.contains(done)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    term.stop().unwrap();
}

/// Shift+wheel, the horizontal wheel and Shift+Left / Shift+Right move the
/// rows sideways (the header with them) when a part was sized wider than the
/// room, four cells a step, and stop at the ends. Red when any of the three
/// does nothing.
#[test]
fn the_rows_scroll_sideways_by_shift_wheel_the_horizontal_wheel_and_the_keys() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-hscroll", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    widen_the_author(&mut term);
    let screen = term.screen();
    let author0 = header_word(&screen, "Author").unwrap_or_else(|| panic!("{}", term.text()));
    let (_, row) = column_origin(&screen).unwrap();
    let on_rows = (column_origin(&screen).unwrap().0 + 6, row + 4);

    // Shift+wheel: four cells to the right.
    term.shift_scroll(true, on_rows.0, on_rows.1).unwrap();
    term.wait_for("scrolled by Shift+wheel", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(4)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The horizontal wheel: four more.
    if cfg!(windows) {
        // A ConPTY does not turn the SGR horizontal wheel (buttons 66 and
        // 67) into a console wheel event: nothing reaches the app, so there
        // is nothing to drive. The same four cells go in by Shift+wheel,
        // which the rest of the test (and the Linux run) still checks.
        term.shift_scroll(true, on_rows.0, on_rows.1).unwrap();
    } else {
        term.scroll_sideways(false, on_rows.0, on_rows.1).unwrap();
    }
    term.wait_for("scrolled by the horizontal wheel", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(8)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The keys: back four, forward four.
    term.send_keys("S-Left").unwrap();
    term.wait_for("scrolled back by the key", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(4)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("S-Right").unwrap();
    term.wait_for("scrolled on by the key", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(8)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Shift+wheel the other way brings the start back, and no further.
    for _ in 0..6 {
        term.shift_scroll(false, on_rows.0, on_rows.1).unwrap();
    }
    term.wait_for("back at the start", STEP, |s| {
        header_word(s, "Author") == Some(author0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A session written before the parts moved to `config.toml` still has its
/// shown parts and widths: they are used once, copied to the config, and the
/// session stops holding them.
/// Red when the old values are ignored (Date comes back) or never copied.
#[test]
fn an_older_sessions_parts_are_migrated_to_the_config() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config.path().join("sessions").join("pty-cc-migrate.toml"),
        "[layout]\ncommit_column_visible = [\"graph\", \"message\", \"author\"]\n\
         commit_column_widths = [0, 14, 0, 0, 0]\n",
    )
    .unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-migrate", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        header_word(&term.screen(), "Date").is_none(),
        "the old session's parts are used\n{}",
        term.text()
    );
    assert!(header_word(&term.screen(), "Author").is_some());
    wait_for_config_list(config.path(), "commit_column_columns_visible");
    assert_eq!(saved_widths(config.path()).get("author"), Some(&14));
    term.stop().unwrap();
    let session = session_text(config.path());
    assert!(!session.contains("commit_column_visible"), "{session}");
    assert!(!session.contains("commit_column_widths"), "{session}");

    // Once migrated, the config rules on the next start.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-migrate", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(header_word(&term.screen(), "Date").is_none());
    term.stop().unwrap();
}

// ---- the order of the parts, every Log column, the end handle --------------

/// The saved order, waiting for the config to hold one.
fn saved_order(config: &Path) -> Vec<String> {
    wait_for_config_list(config, "commit_column_columns_order");
    config_list(config, "commit_column_columns_order")
}

/// Whether the header shows `first` left of `second`.
fn left_of(screen: &Screen, first: &str, second: &str) -> bool {
    matches!(
        (header_word(screen, first), header_word(screen, second)),
        (Some(a), Some(b)) if a < b
    )
}

/// Dragging a name of the header over another moves the part, as in the Log;
/// the order is the column's own (`commit_column_columns_order`), is kept
/// across a restart and leaves the Log's order alone.
/// Red when the drag does nothing, the order is not saved, or the Log's
/// header follows.
#[test]
fn dragging_a_header_moves_the_part_and_is_remembered_apart_from_the_log() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reorder", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (author0, message0) = (
        header_word(&screen, "Author").unwrap(),
        header_word(&screen, "Message").unwrap(),
    );
    let row = header_row(&screen).unwrap();
    assert!(message0 < author0, "{}", term.text());

    // Press on the author's name (not its first letter: that is the
    // boundary), drag it over the message.
    term.press(author0 + 2, row).unwrap();
    term.drag_to(author0 - 2, row).unwrap();
    term.drag_to(message0 + 2, row).unwrap();
    term.release(message0 + 2, row).unwrap();
    term.wait_for("the author is left of the message", STEP, |s| {
        left_of(s, "Author", "Message")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        saved_order(config.path()),
        ["ref", "graph", "author", "message", "date", "changes", "sha"],
        "{:?}",
        config_ui(config.path())
    );
    // The Log keeps its own order, on screen and on disk.
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log again", STEP, |s| {
        !s.contains(TITLE) && log_header(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let header = log_header(&term.screen()).unwrap();
    assert!(
        header.find("Message") < header.find("Author"),
        "the Log's order did not move: {header}"
    );
    assert!(
        config_list(config.path(), "log_columns_order").is_empty(),
        "{:?}",
        config_ui(config.path())
    );
    term.stop().unwrap();

    // After a restart the column is still in its order.
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-reorder", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        left_of(&term.screen(), "Author", "Message"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// Mid-drag of a commit column header: the ghost of its name follows the
/// pointer, an accent bar marks the drop, nothing has moved; Esc lets go with
/// the order intact. The parts menu has a close cross that closes it.
/// Red when the ghost or the bar is missing, the order moves before the
/// release or on Esc, or the cross does nothing.
#[test]
fn a_carried_header_shows_a_ghost_and_a_bar_and_esc_cancels() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-ghost", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (author0, message0) = (
        header_word(&screen, "Author").unwrap(),
        header_word(&screen, "Message").unwrap(),
    );
    let row = header_row(&screen).unwrap();
    assert!(message0 < author0);
    term.press(author0 + 2, row).unwrap();
    term.drag_to(author0 - 2, row).unwrap();
    term.drag_to(message0 + 2, row).unwrap();
    term.wait_for("the ghost and the bar", STEP, |s| {
        s.find_all(" Author ").iter().any(|(_, r)| *r == row + 1)
            && s.find_all("\u{2503}").iter().any(|(_, r)| *r == row)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        left_of(&term.screen(), "Message", "Author"),
        "nothing moves before the release\n{}",
        term.text()
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the drag canceled", STEP, |s| {
        !s.contains("\u{2503}") && s.contains("canceled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(message0 + 2, row).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        left_of(&term.screen(), "Message", "Author"),
        "{}",
        term.text()
    );

    // The parts menu: a cross in its frame closes it.
    let (c, r) = first_letter(&term.screen(), "Message");
    term.right_click(c + 2, r).unwrap();
    term.wait_for("the parts menu", STEP, |s| s.contains("[x] Date"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains('\u{2611}'), "{}", term.text());
    let (cx, cy) = term
        .screen()
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, y)| term.screen().line(*y).contains("Columns"))
        .expect("a cross in the menu's frame");
    term.click(cx, cy).unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("[x] Date"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The keys twin of the drag: `Tab` picks the part, `>` and `<` move it one
/// place among the shown ones, and the palette has both as commands.
/// Red when a key or a command does nothing, or the status does not say so.
#[test]
fn the_keys_and_the_palette_move_the_chosen_part() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-move-keys", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(left_of(&term.screen(), "Message", "Author"));
    pick(&mut term, "add b");
    term.send_keys("Tab").unwrap();
    term.wait_for("the message is the part", STEP, |s| {
        s.contains("Message width:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text(">").unwrap();
    term.wait_for("the message moved right", STEP, |s| {
        s.contains("Message moved right") && left_of(s, "Author", "Message")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        saved_order(config.path())[..4],
        ["ref", "graph", "author", "message"],
        "{:?}",
        config_ui(config.path())
    );
    // And back by the palette.
    run_palette(&mut term, "Move the Part Left");
    term.wait_for("the message moved left", STEP, |s| {
        s.contains("Message moved left") && left_of(s, "Message", "Author")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The lanes are before it: one more step left, then an end.
    term.send_text("<").unwrap();
    term.send_text("<").unwrap();
    term.wait_for("the first place", STEP, |s| {
        s.contains("Message cannot move further")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Every column the Log offers is a part of the column: Changes and SHA are
/// off by default, shown by the palette's toggles with the Log's own cells
/// (the changes of HEAD, its short hash), and the choice is saved.
/// Red when either is missing from the toggles or shows no value.
#[test]
fn changes_and_sha_are_parts_of_the_column_with_the_logs_values() {
    let repo = fixture();
    let sha = repo.git(&["rev-parse", "--short=7", "HEAD"]);
    let sha = sha.trim().to_string();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-more-parts", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(header_word(&term.screen(), "Changes").is_none());
    assert!(header_word(&term.screen(), "SHA").is_none());
    run_palette(&mut term, "Column: Toggle Changes");
    run_palette(&mut term, "Column: Toggle SHA");
    term.wait_for("both parts and their values", STEP, |s| {
        header_word(s, "Changes").is_some()
            && header_word(s, "SHA").is_some()
            && column_text(s).contains(&sha)
            && column_text(s).contains("+1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_for_config_list(config.path(), "commit_column_columns_visible");
    let visible = config_list(config.path(), "commit_column_columns_visible");
    assert!(
        visible.contains(&"changes".to_string()) && visible.contains(&"sha".to_string()),
        "{visible:?}"
    );
    // The header menu lists all seven.
    let (c, r) = first_letter(&term.screen(), "Message");
    term.right_click(c + 2, r).unwrap();
    term.wait_for("the parts menu", STEP, |s| {
        [
            "Ref", "Graph", "Message", "Author", "Date", "Changes", "SHA",
        ]
        .iter()
        .all(|w| s.contains(&format!("[ ] {w}")) || s.contains(&format!("[x] {w}")))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The last part has a resize boundary too: the cell right after it (the
/// column's own border when the row fills the room). Dragging it sizes the
/// last part, lit like the other boundaries.
/// Red when there is no grab after the last part, or it is not lit.
#[test]
fn the_cell_after_the_last_part_resizes_it_and_lights_while_dragged() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-end-handle", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let row = header_row(&screen).unwrap();
    let border = column_right(&screen).unwrap();
    // Date is last; its automatic width is five cells. Press on the cell
    // after it, pull three cells to the right.
    term.press(border, row).unwrap();
    term.drag_to(border + 2, row).unwrap();
    term.drag_to(border + 3, row).unwrap();
    term.wait_for("the end handle is lit", STEP, |s| {
        s.cell(border, row)
            .zip(s.cell(border - 3, row))
            .is_some_and(|(a, b)| a.bg != b.bg)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(border + 3, row).unwrap();
    wait_for_saved_widths(config.path());
    let saved = saved_widths(config.path());
    assert_eq!(saved.get("date"), Some(&8), "{saved:?}");
    // The other parts kept their widths (the first grab pins them all).
    assert!(saved.get("author").is_some_and(|w| *w >= 8), "{saved:?}");
    term.stop().unwrap();
}

/// `line` mode (the default): a tick rests in the header's separator, a thin
/// line runs from the header down the rows while the pointer is on it, and
/// the light goes with the pointer. Red when the grab is the next letter, or
/// the line is only on the header.
#[test]
fn line_mode_hovering_a_separator_draws_a_thin_line_down_the_column() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-line-hover", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (author, row) = first_letter(&screen, "Author");
    let gap = author - 1;
    let text = |s: &Screen, y: u16| s.cell(gap, y).map(|c| c.text.clone()).unwrap_or_default();
    assert_eq!(
        text(&screen, row),
        "\u{2502}",
        "the resting tick\n{}",
        term.text()
    );
    assert_ne!(text(&screen, row + 3), "\u{2502}");

    term.move_to(gap, row + 3).unwrap();
    term.wait_for("the thin line", STEP, |s| {
        (row..=row + 6).all(|y| text(s, y) == "\u{2502}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert_ne!(
        screen.cell(gap + 1, row + 3).unwrap().text,
        "\u{2502}",
        "one cell wide"
    );
    // The commit rows keep their text: the line is only in the separator.
    term.move_to(gap + 5, row + 3).unwrap();
    term.wait_for("the line is gone", STEP, |s| {
        text(s, row + 3) != "\u{2502}" && text(s, row) == "\u{2502}"
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `line` mode: a drag that starts on the separator in the middle of the rows
/// sizes the message, like the header's, and the widths are saved.
#[test]
fn line_mode_a_drag_from_the_middle_of_the_rows_sizes_the_part() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_in(repo.root(), config.path(), "pty-cc-line-drag", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (author0, row) = first_letter(&screen, "Author");
    let date0 = header_word(&screen, "Date").unwrap();
    let gap = author0 - 1;

    term.press(gap, row + 4).unwrap();
    term.drag_to(gap - 3, row + 4).unwrap();
    term.drag_to(gap - 6, row + 4).unwrap();
    term.wait_for("the message is narrower", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(6)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        header_word(&term.screen(), "Date"),
        date0.checked_sub(6),
        "the author kept its width\n{}",
        term.text()
    );
    term.release(gap - 6, row + 4).unwrap();
    wait_for_saved_widths(config.path());
    assert!(saved_widths(config.path())
        .get("message")
        .is_some_and(|w| *w > 0));
    term.stop().unwrap();
}

/// `line` mode: after a drag of the separator ends, the thin line goes with
/// the button unless the pointer is still on the separator (a hover that the
/// next move away takes down). Red when the highlight the drag left stays.
#[test]
fn line_mode_the_line_does_not_stay_after_a_drag_is_released() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-line-release", 180);
    open_head_file(&mut term);
    term.wait_for("the column", STEP, |s| s.contains(TITLE))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (author0, row) = first_letter(&screen, "Author");
    let gap = author0 - 1;
    let new_gap = gap - 4;
    let text =
        |s: &Screen, x: u16, y: u16| s.cell(x, y).map(|c| c.text.clone()).unwrap_or_default();

    term.press(gap, row + 4).unwrap();
    term.drag_to(gap - 2, row + 4).unwrap();
    term.drag_to(new_gap, row + 4).unwrap();
    term.wait_for("the author moved", STEP, |s| {
        header_word(s, "Author") == author0.checked_sub(4)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Released away from the separator: no line, the tick is back.
    term.release(new_gap - 6, row + 4).unwrap();
    term.wait_for("no line, the tick back", STEP, |s| {
        text(s, new_gap, row + 4) != "\u{2502}" && text(s, new_gap, row) == "\u{2502}"
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Released on it: a hover, gone with the next move away.
    term.press(new_gap, row + 4).unwrap();
    term.release(new_gap, row + 4).unwrap();
    term.wait_for("the hover line", STEP, |s| {
        (row..=row + 6).all(|y| text(s, new_gap, y) == "\u{2502}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.move_to(new_gap + 8, row + 4).unwrap();
    term.wait_for("the line gone", STEP, |s| {
        text(s, new_gap, row + 4) != "\u{2502}"
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
