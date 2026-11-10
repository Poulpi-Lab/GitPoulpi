//! The File History split view through a pty: the file's commits on the
//! left (paged, following renames), the selected commit's diff of that file
//! on the right, a range to compare two commits, the splitter.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

fn launch(repo: &Path, session: &str, config: Option<&str>) -> (Terminal, tempfile::TempDir) {
    let mut launch = common::launch(repo)
        .no_commit_column()
        .session(session)
        .size(180, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture();
    if let Some(config) = config {
        launch = launch.config(config);
    }
    launch.start()
}

/// `notes.txt` is added, edited, renamed to `memo.txt` (with an edit) and
/// edited again: four commits, two names.
fn renamed_file() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", "one\ntwo\nthree\n", "add notes")
        .unwrap()
        .commit_file("notes.txt", "one\nTWO\nthree\n", "shout two")
        .unwrap()
        .build();
    repo.git(&["mv", "notes.txt", "memo.txt"]);
    std::fs::write(repo.root().join("memo.txt"), "one\nTWO\nthree\nTHREE\n").unwrap();
    repo.git(&["add", "memo.txt"]);
    repo.git(&["commit", "-q", "-m", "rename notes to memo"]);
    std::fs::write(
        repo.root().join("memo.txt"),
        "one\nTWO\nthree\nTHREE\nfour\n",
    )
    .unwrap();
    repo.git(&["commit", "-q", "-am", "add four"]);
    repo
}

/// The bar's `File | All` selector: its `File` segment opens the History.
const SELECTOR: &str = " File \u{2502} All ";

/// Opens `memo.txt`'s diff from the commit-detail list, then its History.
fn open_history(term: &mut Terminal, file: &str, title: &str) {
    let (c, r) = term
        .wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(SELECTOR, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    let title = title.to_string();
    term.wait_for("the History split", STEP, move |s| s.contains(&title))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Selecting a row shows that commit's diff of the file, the selection
/// follows the cursor, and a row before a rename is diffed at its old path.
/// Red when the right side keeps showing the first commit's diff, or when a
/// rename's older rows are asked for under the new name (an added file).
#[test]
fn the_selected_commits_diff_follows_the_cursor_across_a_rename() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-follow", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");

    // Newest first, all four commits across the rename, with the stats.
    term.wait_for("the four rows", STEP, |s| {
        s.contains("add four")
            && s.contains("rename notes to memo")
            && s.contains("shout two")
            && s.contains("add notes")
            && s.contains("+1 -0")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The rename row says where the file came from (cut at this width).
    assert!(snap(&term).text().contains("notes.txt"), "{}", term.text());
    // The first row's diff: only "four".
    term.wait_for("HEAD's diff of the file", STEP, |s| {
        s.contains("\u{2715}") && s.contains("+four") && !s.contains("+THREE")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Down: the rename commit, shown as a rename (one file, its one edit).
    term.send_keys("Down").unwrap();
    term.wait_for("the rename commit's diff", STEP, |s| {
        s.contains("+THREE") && !s.contains("+four") && !s.contains("+one")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Down: before the rename, the file was called notes.txt: its diff is
    // the edit of line two, found under the old path.
    term.send_keys("Down").unwrap();
    term.wait_for("the diff under the old name", STEP, |s| {
        s.contains("+TWO") && s.contains("-two") && !s.contains("+THREE") && !s.contains("+four")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !snap(&term).text().contains("New file"),
        "an old row's diff must not be asked under the new name:\n{}",
        term.text()
    );

    // The first commit adds the whole file.
    term.send_keys("Down").unwrap();
    term.wait_for("the first commit's diff", STEP, |s| s.contains("+three"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Shift+Down extends the selection to a range: the right side then shows
/// the file's diff between the two ends, using the path each end had.
#[test]
fn a_shift_range_compares_the_two_ends_of_the_file() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-range", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the list", STEP, |s| s.contains("shout two"))
        .unwrap();

    // Rows 0..=2 are HEAD, the rename and "shout two": the diff from the
    // file after "shout two" (named notes.txt) to HEAD (memo.txt): the two
    // added lines, found across the rename; "TWO" is unchanged context.
    term.send_keys("S-Down S-Down").unwrap();
    term.wait_for("three rows compared", STEP, |s| {
        s.contains("3 selected") && s.contains("+THREE") && s.contains("+four")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!snap(&term).text().contains("+TWO"), "{}", term.text());
    // Esc drops the range: one commit again, the one the cursor is on
    // ("shout two", whose file was still called notes.txt).
    term.send_keys("Escape").unwrap();
    term.wait_for("the range dropped", STEP, |s| {
        !s.contains("3 selected") && s.contains("+TWO") && s.contains("-two")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The column where the diff's own frame starts: the splitter's cell.
fn diff_left(term: &Terminal) -> Option<usize> {
    snap(&term).text().lines().find_map(|l| {
        let at = l.find("\u{256d} memo.txt")?;
        Some(l[..at].chars().count())
    })
}

/// Tab gives the keyboard to the diff (its own keys), and back.
#[test]
fn tab_gives_the_keyboard_to_the_diff_and_back() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-tab", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("HEAD's diff", STEP, |s| s.contains("+four"))
        .unwrap();
    // In the diff, Down moves the diff's cursor: the history row stays.
    term.send_keys("Tab Down").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(snap(&term).text().contains("+four") && !snap(&term).text().contains("+THREE"));
    // Back in the list, Down selects the next commit.
    term.send_keys("Tab Down").unwrap();
    term.wait_for("the rename commit's diff", STEP, |s| {
        s.contains("+THREE") && !s.contains("+four")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The splitter moves by key (Alt+Right / Alt+Left) and by dragging, and
/// the width is remembered in the session.
#[test]
fn the_splitter_resizes_by_key_and_by_drag_and_is_remembered() {
    let repo = renamed_file();
    let (mut term, config) = launch(repo.root(), "pty-history-drag", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the list", STEP, |s| s.contains("shout two"))
        .unwrap();
    term.wait_for("the diff frame", STEP, |_| diff_left(&term).is_some())
        .unwrap();
    let start = diff_left(&term).expect("the diff frame");

    term.send_keys("M-Right").unwrap();
    term.wait_for("the list got wider", STEP, |_| {
        diff_left(&term).is_some_and(|c| c > start)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-Left M-Left").unwrap();
    term.wait_for("the list got narrower", STEP, |_| {
        diff_left(&term).is_some_and(|c| c < start)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Drag the splitter (the diff's left border, below its bar) to the right.
    let narrow = diff_left(&term).unwrap();
    let (_, row) = snap(&term).find("\u{256d} memo.txt").unwrap();
    term.drag((narrow as u16, row + 4), (narrow as u16 + 30, row + 4))
        .unwrap();
    term.wait_for("the drag widened the list", STEP, |_| {
        diff_left(&term).is_some_and(|c| c >= narrow + 25)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Persisted with the session layout.
    std::thread::sleep(Duration::from_millis(300));
    let saved = std::fs::read_dir(config.path().join("sessions"))
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .any(|t| t.contains("history_split_pct"));
    assert!(saved, "the width is saved in the session file");
    term.stop().unwrap();
}

/// Right-click opens the row menu; Restore (confirmed) checks the file out
/// of that commit, under the name it had there; `b` blames at that commit.
#[test]
fn the_row_menu_restores_and_blames_at_that_commit() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-menu", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    let (c, r) = term
        .wait_for_text("shout two", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click_with(xtask::pty::Button::Right, c, r).unwrap();
    term.wait_for("the row menu", STEP, |s| {
        s.contains("Show in the Log") && s.contains("Restore this version")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| {
        !s.contains("Restore this version")
    })
    .unwrap();

    // The cursor is on "shout two" (the right-click moved it).
    term.send_keys("S-R").unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Overwrite notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    let restored = repo.root().join("notes.txt");
    let deadline = std::time::Instant::now() + STEP;
    while !restored.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(
        std::fs::read_to_string(&restored).unwrap(),
        "one\nTWO\nthree\n",
        "the file as it was at that commit"
    );

    term.send_keys("b").unwrap();
    term.wait_for("Blame at that commit", STEP, |s| {
        s.contains("Blame - notes.txt @")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The row of the view tabs (File View / Diff View / Blame / History).
fn bar_row(term: &Terminal) -> u16 {
    let screen = snap(&term);
    (0..screen.rows())
        .find(|r| {
            let l = screen.line(*r);
            l.contains(" File View ") && l.contains(" Blame ") && l.contains(SELECTOR)
        })
        .unwrap_or_else(|| panic!("the tab bar\n{}", term.text()))
}

/// The background of the cell just after where `needle` starts on screen row `row`.
fn bg_of(term: &Terminal, row: u16, needle: &str) -> xtask::pty::Color {
    let screen = snap(&term);
    let line = screen.line(row);
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} in {line:?}"));
    let col = line[..at].chars().count() as u16;
    screen.cell(col + 1, row).unwrap().bg
}

/// Every commit is two terminal rows (summary over author and date) and
/// stays so in a narrow list: the author and the date never disappear, and
/// a click on the second line selects that commit.
/// Red when the list goes back to one row per commit, or when a click on the
/// second line lands on nothing / the neighbor.
#[test]
fn rows_are_two_lines_with_author_and_date_even_narrow_and_the_second_line_clicks() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-two-lines", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the rows", STEP, |s| s.contains("shout two"))
        .unwrap();
    // Narrow the list to its minimum.
    term.send_keys("M-Left M-Left M-Left M-Left M-Left M-Left M-Left")
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let (c, r) = snap(&term)
        .find("shout two")
        .unwrap_or_else(|| panic!("{}", term.text()));
    let below = snap(&term).line(r + 1);
    assert!(
        below.contains("just now") && below.contains("G\u{2026}"),
        "author and date on the line under the summary, even narrow: {below:?}\n{}",
        term.text()
    );
    // The next commit starts two rows down, not one.
    let (_, r_next) = snap(&term).find("add notes").unwrap();
    assert_eq!(r_next, r + 2, "two rows per commit");

    // A click on the SECOND line of "shout two" selects it: its diff shows.
    assert!(!snap(&term).text().contains("+TWO"), "still on HEAD's diff");
    term.click(c + 3, r + 1).unwrap();
    term.wait_for("shout two's diff", STEP, |s| {
        s.contains("+TWO") && s.contains("-two")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The right side shows Diff, File or Blame through the bar's own File View /
/// Diff View / Blame buttons (the History has no control of its own), stays
/// in the split, keeps its mode while the cursor moves, and the History tab
/// stays lit.
/// Red when a bar button leaves the History (the list disappears), when
/// moving rows drops back to the diff, or when the bar lights the wrong tab.
#[test]
fn the_bar_buttons_pick_the_right_side_and_stay_in_the_history() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-modes", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("HEAD's diff", STEP, |s| s.contains("+four"))
        .unwrap();

    // The four tabs are on the bar, History lit; the old Diff / File / Blame
    // control is gone (no second row of mode buttons).
    let bar = bar_row(&term);
    assert_ne!(
        bg_of(&term, bar, SELECTOR),
        bg_of(&term, bar, " Blame "),
        "History is the lit tab"
    );
    let tab_rows = (0..snap(&term).rows())
        .filter(|r| {
            let l = snap(&term).line(*r);
            *r != bar && l.contains(" Blame ") && l.contains('\u{2500}')
        })
        .count();
    assert_eq!(tab_rows, 0, "no mode control in a border\n{}", term.text());

    // File View: the file at HEAD, in the split (the list stays).
    click_bar(&mut term, " File View ");
    term.wait_for("the file at HEAD", STEP, |s| {
        s.contains("5 four") && s.contains("File View - memo.txt") && s.contains("add notes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bar = bar_row(&term);
    assert_ne!(
        bg_of(&term, bar, SELECTOR),
        bg_of(&term, bar, " Blame "),
        "History stays the lit tab in File mode"
    );

    // The click gave the keyboard to the right half: Tab gives it back to
    // the list. Down: the mode is kept, the file is the one at the rename
    // commit.
    term.send_keys("Tab").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("the file at the rename commit", STEP, |s| {
        s.contains("4 THREE") && !s.contains("5 four") && s.contains("add notes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Blame at that commit, by the bar's Blame button.
    click_bar(&mut term, " Blame ");
    term.wait_for("blame at the commit", STEP, |s| {
        s.contains("Blame - memo.txt") || s.contains("Blame - notes.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(snap(&term).text().contains("add notes"), "the list stays");

    // Back to the diff with the bar's Diff View button: the rename commit's
    // diff, still in the History.
    click_bar(&mut term, " Diff View ");
    term.wait_for("the diff again", STEP, |s| {
        s.contains("+THREE") && !s.contains("Blame - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(snap(&term).text().contains("add notes"), "the list stays");

    // The keys of the old control still work (the keyboard twins).
    term.send_keys("M-2").unwrap();
    term.wait_for("the file by the key", STEP, |s| {
        s.contains("File View - memo.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-1").unwrap();
    term.wait_for("the diff by the key", STEP, |s| {
        s.contains("+THREE") && !s.contains("File View - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A click on a label of the view bar.
fn click_bar(term: &mut Terminal, label: &str) {
    let screen = snap(&term);
    // The first hit that is not the list's own title (" History - file").
    let (bar, col) = (0..screen.rows())
        .find_map(|r| {
            let line = screen.line(r);
            line.match_indices(label)
                .find(|(i, _)| !line[*i..].starts_with(" History - "))
                .map(|(i, _)| (r, line[..i].chars().count() as u16))
        })
        .unwrap_or_else(|| panic!("{label:?} on the bar\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(450));
    term.click(col + 2, bar).unwrap();
}

/// The first cell of the History list's frame on its title row.
fn list_origin(term: &Terminal) -> Option<(u16, u16)> {
    let screen = snap(&term);
    let (c, r) = screen.find(" File History - ")?;
    Some((c.saturating_sub(1), r))
}

/// The reopen strip's chevron, in the cell right of the central column's
/// left edge `x0`.
fn chevron(term: &Terminal, x0: u16) -> Option<(u16, u16)> {
    let screen = snap(&term);
    (0..screen.rows()).find_map(|row| {
        let at = screen.line(row).chars().nth(usize::from(x0) + 1)?;
        (at == '\u{203A}').then_some((x0 + 1, row))
    })
}

/// The list hides by its cross, comes back by the strip, and the key hides
/// and shows it.
/// Red when the list has no cross or strip (no way to hide it), when the
/// diff on the right goes with it, or when the key stops cycling.
#[test]
fn the_list_hides_by_its_cross_and_comes_back_by_the_strip_and_the_key() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-hide", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the list", STEP, |s| {
        s.contains("shout two") && s.contains("+four")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x0, row) = list_origin(&term).unwrap();
    // The bar lights the selector's `File` segment: the list is shown.
    let bar = bar_row(&term);
    assert_ne!(
        bg_of(&term, bar, SELECTOR),
        bg_of(&term, bar, " Blame "),
        "the bar names the state\n{}",
        term.text()
    );
    // The cross sits on the list's top border.
    let cross = snap(&term)
        .find_all("\u{2715}")
        .into_iter()
        .find(|(c, r)| *r == row && *c > x0 && *c < x0 + 60)
        .unwrap_or_else(|| panic!("no cross on the list\n{}", term.text()));
    term.click(cross.0, cross.1).unwrap();
    term.wait_for("the list hidden", STEP, |s| {
        !s.contains("shout two") && !s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The diff stays, and the strip is where the list was.
    assert!(snap(&term).text().contains("+four"), "{}", term.text());
    term.wait_for("the strip", STEP, |_| chevron(&term, x0).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The selector stays where it was; its `File` segment is no longer lit.
    let bar = bar_row(&term);
    assert_eq!(
        bg_of(&term, bar, SELECTOR),
        bg_of(&term, bar, " Blame "),
        "the bar says off\n{}",
        term.text()
    );
    let (c, r) = chevron(&term, x0).unwrap();
    std::thread::sleep(Duration::from_millis(450));
    term.click(c, r).unwrap();
    term.wait_for("the list back", STEP, |s| {
        s.contains("shout two") && s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(chevron(&term, x0).is_none(), "{}", term.text());

    // The key hides it (the diff stays), and shows it again.
    term.send_keys("M-c").unwrap();
    term.wait_for("hidden by the key", STEP, |s| {
        !s.contains("shout two") && !s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(snap(&term).text().contains("+four"), "{}", term.text());
    term.wait_for("the strip", STEP, |_| chevron(&term, x0).is_some())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("M-c").unwrap();
    term.wait_for("the file history again", STEP, |s| {
        s.contains(" History - ") && s.contains("shout two")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The bar's selector picks the list by the mouse (a segment) and hides or
/// shows the column (a click on the selected segment), and the palette rows
/// pick a state: every state has a keyboard twin.
/// Red when a click does not switch the list or a palette row is missing.
#[test]
fn the_selector_switches_lists_by_the_mouse_and_the_palette_rows_pick_a_state() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-states", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the list", STEP, |s| s.contains("shout two"))
        .unwrap();
    let on_segment = |term: &mut Terminal, dx: u16| {
        let (col, row) = snap(&term)
            .find(SELECTOR)
            .unwrap_or_else(|| panic!("the selector\n{}", term.text()));
        std::thread::sleep(Duration::from_millis(450));
        term.click(col + dx, row).unwrap();
    };
    // file -> all (click on `All`).
    on_segment(&mut term, 8);
    term.wait_for("graph", STEP, |s| {
        s.contains(" Commits ") && !s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A click on the selected segment hides it, another shows the same list.
    on_segment(&mut term, 8);
    term.wait_for("off", STEP, |s| !s.contains(" Commits "))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    on_segment(&mut term, 8);
    term.wait_for("graph again", STEP, |s| {
        s.contains(" Commits ") && !s.contains(" History - ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // all -> file (click on `File`).
    on_segment(&mut term, 3);
    term.wait_for("file", STEP, |s| s.contains(" History - "))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let palette = |term: &mut Terminal, text: &str, done: &str| {
        term.send_keys("C-p").unwrap();
        term.send_text(text).unwrap();
        term.wait_for("the palette row", STEP, |s| s.contains(text))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term.send_keys("Enter").unwrap();
        let done = done.to_string();
        term.wait_for("the state", STEP, move |s| {
            !s.contains("Command Palette") && s.contains(&done)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    };
    palette(&mut term, "Commit Column: Hide", "Commit column: off");
    palette(&mut term, "Commit Column: All Commits", " Commits ");
    palette(&mut term, "Commit Column: File History", " History - ");
    term.stop().unwrap();
}

/// While the splitter is dragged, its highlight is on the border where it
/// is now, never where the press started (a stale bar inside the diff).
/// Red when the strip painted at the press is not re-lit with the drag.
#[test]
fn the_drag_highlight_follows_the_splitter() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-hot", None);
    open_history(&mut term, "memo.txt", "History - memo.txt");
    term.wait_for("the list", STEP, |s| s.contains("shout two"))
        .unwrap();
    term.wait_for("the diff frame", STEP, |_| diff_left(&term).is_some())
        .unwrap();
    let start = diff_left(&term).unwrap() as u16;
    let (_, row) = snap(&term).find("\u{256d} memo.txt").unwrap();
    let y = row + 4;
    term.press(start, y).unwrap();
    term.drag_to(start + 12, y).unwrap();
    term.drag_to(start + 24, y).unwrap();
    term.wait_for("the splitter moved", STEP, |_| {
        diff_left(&term).is_some_and(|c| c as u16 >= start + 20)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let now = diff_left(&term).unwrap() as u16;
    let screen = snap(&term);
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
    term.release(start + 24, y).unwrap();
    term.stop().unwrap();
}

/// The File History has one name wherever it is reached: the palette's
/// `Diff: File History` opens it from a file's diff (the commit column's
/// File list, titled `File History - <path>`), and its rows are
/// `File History: ...`, no bare `History:` left.
/// Red when the title or the palette rows go back to the bare `History`.
#[test]
fn the_file_history_has_one_name_in_the_palette_and_the_title() {
    let repo = renamed_file();
    let (mut term, _config) = launch(repo.root(), "pty-history-name", None);
    let (c, r) = term
        .wait_for_text("memo.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for_text(SELECTOR, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.send_text("Diff: File History").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Diff: File History")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the titled list", STEP, |s| {
        s.contains("File History - memo.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.send_text("File History: Copy the Summary").unwrap();
    term.wait_for("the list's row", STEP, |s| {
        s.contains("File History: Copy the Summary")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
