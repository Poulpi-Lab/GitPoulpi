//! The list panes scroll sideways when a row is wider than the pane: a Refs
//! section (a long branch name), the Unstaged list (a long path), the commit's
//! changed files and the commit-details info lines (a long author name). The
//! gestures are the Diff view's: a horizontal wheel or Shift+wheel over the
//! pane, a bar on its bottom border that exists only on overflow, and
//! Alt+Left / Alt+Right for the pane that has the keyboard. Temp dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const TAIL: &str = "TAILEND";

fn snap(term: &Terminal) -> Screen {
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

fn quit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

/// The sideways bars on screen as `(left arrow col, row)`: a `←` right after
/// a rounded bottom-left corner.
fn bars(s: &Screen) -> Vec<(u16, u16)> {
    s.find_all("\u{2190}")
        .into_iter()
        .filter(|(c, r)| *c > 0 && s.cell(*c - 1, *r).is_some_and(|x| x.text == "\u{2570}"))
        .collect()
}

fn long_branch() -> String {
    format!("feature/{}{TAIL}", "long-".repeat(12))
}

fn long_file(prefix: &str) -> String {
    format!("{prefix}{}{TAIL}.txt", "_file_with_a_long_name".repeat(4))
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .config("user.name", &format!("{}{TAIL}", "Longname ".repeat(8)))
        .unwrap()
        .commit_file(&long_file("committed"), "hello\n", "base")
        .unwrap()
        .branch("zz-short")
        .unwrap()
        .branch(&long_branch())
        .unwrap()
        .write_worktree(&long_file("worktree"), b"x\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .size(120, 40)
        .ready("base")
        .mouse_capture()
        .start()
        .0
}

/// The text of `cols` x `rows` of the screen, rows joined by newlines.
fn region(s: &Screen, cols: std::ops::Range<u16>, rows: std::ops::Range<u16>) -> String {
    rows.map(|r| {
        cols.clone()
            .filter_map(|c| s.cell(c, r).map(|cell| cell.text.clone()))
            .collect::<String>()
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// The left column (Refs sections).
const SIDEBAR: std::ops::Range<u16> = 0..24;
/// The right column (Status, or the commit details).
const RIGHT: std::ops::Range<u16> = 79..119;

fn wheel_right(term: &mut Terminal, col: u16, row: u16, notches: usize) {
    for _ in 0..notches {
        term.scroll_sideways(false, col, row).unwrap();
    }
}

/// A long branch name overflows its section: the bar shows on the section's
/// bottom border (and nowhere else in the sidebar), a horizontal wheel
/// reveals the tail, Shift+wheel brings the head back, a click after
/// scrolling selects the row it is on (and the gutter is where the text is,
/// not where the pane starts), and Alt+Left / Alt+Right do the same from the
/// keyboard.
///
/// Falsified: with `hscroll::render_list` always returning offset 0 the tail
/// never shows (red at "the tail"); with the gutter test of `mouse_left_panes`
/// using the unscrolled column, the click on the scrolled name hides the ref
/// (red at "no hide").
#[test]
fn a_long_branch_name_scrolls_in_its_refs_section() {
    let repo = fixture();
    let mut term = launch(&repo, "hscroll-refs");
    let s = snap(&term);
    assert!(!region(&s, SIDEBAR, 3..22).contains(TAIL), "cut at first");
    let in_sidebar: Vec<_> = bars(&s).into_iter().filter(|(c, _)| *c < 24).collect();
    assert_eq!(
        in_sidebar.len(),
        1,
        "one bar, on the LOCAL section: {in_sidebar:?}"
    );
    let (_, name_row) = s.find("long-long").unwrap();
    let (_, short_row) = s.find("zz-short").unwrap();

    wheel_right(&mut term, 8, name_row, 16);
    term.wait_for("the tail", STEP, |s| {
        region(s, SIDEBAR, 3..22).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Shift+wheel up is the other way (the terminals that report it so).
    for _ in 0..16 {
        term.shift_scroll(false, 8, name_row).unwrap();
    }
    term.wait_for("the head again", STEP, |s| {
        let r = region(s, SIDEBAR, 3..22);
        r.contains("long-long") && !r.contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Shift+wheel down scrolls right too.
    for _ in 0..16 {
        term.shift_scroll(true, 8, name_row).unwrap();
    }
    term.wait_for("the tail by Shift+wheel", STEP, |s| {
        region(s, SIDEBAR, 3..22).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click on the short row (blank where it is scrolled) selects that row
    // and only that row; it lands in the unscrolled gutter's columns but on
    // scrolled text, so nothing is hidden from the graph.
    term.click(4, short_row).unwrap();
    let s = term
        .wait_for("the row selected", STEP, |s| {
            s.cell(8, short_row).map(|c| c.bg) != s.cell(8, name_row).map(|c| c.bg)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        region(&s, SIDEBAR, 3..22).contains(TAIL),
        "a click keeps the scroll"
    );

    // From the keyboard (the section has it now): Alt+Left back, Alt+Right on.
    for _ in 0..16 {
        term.send_keys("M-Left").unwrap();
    }
    let s = term
        .wait_for("Alt+Left: the head", STEP, |s| {
            let r = region(s, SIDEBAR, 3..22);
            r.contains("long-long") && !r.contains(TAIL)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The click landed on scrolled text, not on the (unscrolled) eye gutter:
    // nothing was hidden from the graph.
    assert!(
        !region(&s, SIDEBAR, 3..22).contains('\u{2298}'),
        "no hide: {}",
        term.text()
    );
    for _ in 0..16 {
        term.send_keys("M-Right").unwrap();
    }
    term.wait_for("Alt+Right: the tail", STEP, |s| {
        region(s, SIDEBAR, 3..22).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// Nothing wider than its pane, no bar and no scrolling: the sideways wheel
/// leaves the rows where they are.
#[test]
fn short_rows_get_no_bar() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "hello\n", "base")
        .unwrap()
        .write_worktree("b.txt", b"x\n")
        .unwrap()
        .build();
    let mut term = launch(&repo, "hscroll-none");
    let (c, r) = snap(&term).find("base").unwrap();
    term.click(c, r).unwrap();
    let s = snap(&term);
    assert!(bars(&s).is_empty(), "no bar: {:?}\n{}", bars(&s), s.text());
    let before = region(&s, 0..119, 3..30);
    wheel_right(&mut term, 8, 8, 6);
    assert_eq!(region(&snap(&term), 0..119, 3..30), before);
    quit(term);
}

/// The Unstaged list scrolls its long path: the bar on the section's bottom
/// border, the wheel, and the row's Stage button (hover) stays at the pane's
/// right edge instead of scrolling away.
///
/// Falsified: with the `h_off` argument of `status_row_item` ignored the tail
/// never shows (red at "the tail"; the unit test
/// `a_scrolled_row_keeps_its_hover_button_at_the_right_end` pins the button).
#[test]
fn a_long_path_scrolls_in_the_unstaged_list() {
    let repo = fixture();
    let mut term = launch(&repo, "hscroll-status");
    let s = snap(&term);
    let (_, row) = s.find("worktree_file").unwrap();
    assert!(!region(&s, RIGHT, 3..14).contains(TAIL));
    let right_bars: Vec<_> = bars(&s).into_iter().filter(|(c, _)| *c >= 70).collect();
    assert_eq!(right_bars.len(), 1, "{right_bars:?}");
    wheel_right(&mut term, 90, row, 30);
    term.wait_for("the tail", STEP, |s| region(s, RIGHT, 3..14).contains(TAIL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Hovering the row shows its button at the right edge, text still scrolled.
    term.move_to(90, row).unwrap();
    term.move_to(91, row).unwrap();
    term.wait_for("Stage button pinned", STEP, |s| {
        let r = region(s, RIGHT, row..row + 1);
        r.contains("Stage") && r.contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// The commit's changed files and the info lines under "Commit details"
/// scroll sideways; the Alt keys act on the pane that has the keyboard (the
/// Log: the info lines; inside the files: the files).
///
/// Falsified: with the info lines never cut (`cut_line` returning the line)
/// the author's tail never shows (red at "author tail"); with the commit
/// files list rendered with a fixed offset 0 the file's tail never shows (red
/// at "file tail").
#[test]
fn the_commit_info_and_files_scroll_sideways() {
    let repo = fixture();
    let mut term = launch(&repo, "hscroll-commit");
    let (c, r) = snap(&term).find("base").unwrap();
    term.click(c, r).unwrap();
    let s = term
        .wait_for("the details", STEP, |s| s.contains("Changed files"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = if bars(&s).len() < 3 { snap(&term) } else { s };
    let (_, file_row) = s.find("committed_file").unwrap();
    assert!(!region(&s, RIGHT, 3..40).contains(TAIL));
    assert_eq!(
        bars(&s).into_iter().filter(|(c, _)| *c >= 70).count(),
        2,
        "the info and the files: {}",
        term.text()
    );
    // The wheel over the info lines.
    wheel_right(&mut term, 90, 4, 30);
    term.wait_for("author tail", STEP, |s| {
        region(s, RIGHT, 3..8).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The wheel over the files.
    wheel_right(&mut term, 90, file_row, 30);
    term.wait_for("file tail", STEP, |s| {
        region(s, RIGHT, 11..40).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The keyboard: the Log has it, Alt+Left brings the info lines back.
    for _ in 0..30 {
        term.send_keys("M-Left").unwrap();
    }
    term.wait_for("Alt+Left: the info head", STEP, |s| {
        !region(s, RIGHT, 3..8).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..30 {
        term.send_keys("M-Right").unwrap();
    }
    term.wait_for("Alt+Right: the info tail", STEP, |s| {
        region(s, RIGHT, 3..8).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Inside the files (Enter browses them) the same keys scroll the files.
    term.send_keys("Enter").unwrap();
    for _ in 0..30 {
        term.send_keys("M-Left").unwrap();
    }
    term.wait_for("Alt+Left: the file head", STEP, |s| {
        !region(s, RIGHT, 11..40).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..30 {
        term.send_keys("M-Right").unwrap();
    }
    term.wait_for("Alt+Right: the file tail", STEP, |s| {
        region(s, RIGHT, 11..40).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// A press on the bare track jumps the thumb under the pointer and the pane
/// follows; the thumb dragged back to the start brings the head back.
#[test]
fn the_bar_drags_and_a_track_press_jumps() {
    let repo = fixture();
    let mut term = launch(&repo, "hscroll-bar");
    let s = snap(&term);
    let (left, bar_row) = *bars(&s).iter().find(|(c, _)| *c < 24).unwrap();
    // The track runs from the left arrow to the right one: a press near the
    // far end jumps the thumb there and the tail is in view.
    term.click(left + 17, bar_row).unwrap();
    term.wait_for("track press: the tail", STEP, |s| {
        region(s, SIDEBAR, 3..22).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Grab the thumb at the end of the track and drag it back to the start.
    term.press(left + 17, bar_row).unwrap();
    term.drag_to(left + 8, bar_row).unwrap();
    term.drag_to(left, bar_row).unwrap();
    term.release(left, bar_row).unwrap();
    term.wait_for("dragged back", STEP, |s| {
        !region(s, SIDEBAR, 3..22).contains(TAIL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}
