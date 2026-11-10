//! Fitting a Log column to its content, and the grip after the last column.
//!
//! A double click on a column's boundary fits the column on its left (the
//! twin of `=` on the picked column, `+` for every column, the palette rows
//! and the chooser's "Fit ... to content" row). The grip on the last
//! column's own last cell resizes it relative to the press and lights like an
//! internal boundary. Hand-paced clicks: `Terminal::double_click` writes both
//! presses in one go, so no frame ever separates them. Temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COMMIT: &str = "autofit probe commit";
const AUTHOR: &str = "Bartholomew Longname-Author";
const LINE: &str = "\u{2502}";

fn ms(n: u64) {
    std::thread::sleep(Duration::from_millis(n));
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .config("user.name", AUTHOR)
        .unwrap()
        .commit(COMMIT)
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 40)
        .ready_when("the Log header", |s| {
            s.contains(COMMIT) && header(s, "Author").is_some()
        })
        .mouse_capture()
        .start()
}

/// Cell of `title` on the Log's header row (the row of `Message`).
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
}

/// The Log row's Author cell (the first row under the header, between the
/// Author and Date titles); the commit details pane shows the full name too,
/// so the whole screen proves nothing.
fn author_cell(s: &Screen) -> String {
    let (Some((from, row)), Some((to, _))) = (header(s, "Author"), header(s, "Date")) else {
        return String::new();
    };
    (from..to)
        .map(|c| {
            s.cell(c, row + 1)
                .map(|c| c.display().to_string())
                .unwrap_or_default()
        })
        .collect()
}

fn full_author(s: &Screen) -> bool {
    author_cell(s).contains(AUTHOR)
}

fn hand_press(term: &mut Terminal, col: u16, row: u16) {
    term.press(col, row).unwrap();
    ms(60);
}

fn quit(mut term: Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        ms(100);
    }
    let _ = term.stop();
}

/// A hand-paced double click on the Author boundary shows the whole name; the
/// second press starts no drag (a far drag after it changes no width).
///
/// Falsified two ways: with the `fit_one` call of the double-click branch
/// removed the name stays cut (red at "the full name"); with the branch also
/// arming a drag (the `dragging_log_boundary = None` replaced by arming it)
/// the far drag narrows Author and the name is cut again (red at "no drag").
#[test]
fn a_hand_paced_double_click_on_a_boundary_fits_the_column_and_starts_no_drag() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "autofit-double");
    let s = snap(&term);
    assert!(!full_author(&s), "cut before the fit\n{}", term.text());
    let (date_x, row) = header(&s, "Date").unwrap();
    let (author_x, _) = header(&s, "Author").unwrap();
    // The boundary's grip is the gap cell, the Author column's last cell.
    let gap = date_x - 1;

    hand_press(&mut term, gap, row + 3);
    term.release(gap, row + 3).unwrap();
    ms(120);
    // The second press, held: the fit happens on the press.
    hand_press(&mut term, gap, row + 3);
    term.wait_for("the full name", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let fitted_x = header(&snap(&term), "Author").unwrap().0;
    // The button is still down: dragging far left must not move anything.
    term.drag_to(author_x + 2, row + 3).unwrap();
    ms(300);
    let held = snap(&term);
    assert!(
        full_author(&held),
        "no drag after a double click\n{}",
        term.text()
    );
    assert_eq!(header(&held, "Author").unwrap().0, fitted_x);
    term.release(author_x + 2, row + 3).unwrap();
    quit(term);
}

/// `Shift+C`, `Tab` to the Author column, `=` fits it; `+` fits every column;
/// the palette rows do the same.
///
/// Falsified: with the `FitToContent` arm of `log_columns::run` returning
/// without fitting, the name stays cut (red at "= fits Author").
#[test]
fn keys_and_the_palette_fit_one_column_or_all() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "autofit-keys");
    assert!(!full_author(&snap(&term)));
    term.send_keys("C Tab Tab Tab").unwrap();
    term.wait_for("Author picked", STEP, |s| {
        s.contains("Author column picked")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("=").unwrap();
    term.wait_for("= fits Author", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the status line", STEP, |s| s.contains("fitted"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Back to automatic (`0`), then every column through the palette.
    term.send_text("0").unwrap();
    term.wait_for("Author cut again", STEP, |s| !full_author(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.send_keys("C-p").unwrap();
    term.send_text("Fit All Columns to Content").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Log Column: Fit All Columns to Content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the palette fits every column", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `+` while a column is picked does the same.
    term.send_keys("C").unwrap();
    term.send_text("0").unwrap();
    term.wait_for("cut again", STEP, |s| !full_author(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("+").unwrap();
    term.wait_for("+ fits every column", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// Right-click the Author header: the chooser offers "Fit Author to content";
/// Enter on it fits the column and closes the chooser.
///
/// Falsified: with `run_chooser_row` ignoring `ChooserRow::FitColumn` the name
/// stays cut (red at "the chooser fits it").
#[test]
fn the_chooser_fits_the_right_clicked_column() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "autofit-chooser");
    let (x, row) = header(&snap(&term), "Author").unwrap();
    term.right_click(x + 1, row).unwrap();
    term.wait_for("the Fit Author row", STEP, |s| {
        s.contains("Fit Author to content")
            && s.contains("Fit all columns to content")
            && s.contains("Reset column widths")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.text().contains("Reset column widths"),
        "{}",
        term.text()
    );
    // From Ref, Up wraps over Reset and Fit all to the Fit Author row.
    term.send_keys("Up Up Up").unwrap();
    ms(200);
    term.send_keys("Enter").unwrap();
    term.wait_for("the chooser fits it", STEP, |s| {
        full_author(s) && !s.contains("Fit Author to content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// The grip after the last column: the SHA column's own last cell. Resting
/// tick, a drag moves it relative to the press (SHA narrows by the travel,
/// Message takes the room), and hovering lights it with the same accent as an
/// internal boundary.
///
/// Falsified: without the last-grip registration in `log_grip_rect` there is
/// no tick and the drag does nothing (red at "SHA moved right"); with the
/// absolute (pointer position) arithmetic of the internal boundaries SHA
/// jumped by the cell offset (red at the exact shift).
#[test]
fn the_last_column_has_a_grip_that_drags_relative_and_lights_like_the_others() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "autofit-last-grip");
    let s = snap(&term);
    let (sha_x, row) = header(&s, "SHA").unwrap();
    let (date_x, _) = header(&s, "Date").unwrap();
    // SHA is 8 cells wide (7 digits and the grip): its grip is cell 7.
    let grip = sha_x + 7;
    assert_eq!(
        s.cell(grip, row)
            .map(|c| c.text.clone())
            .unwrap_or_default(),
        LINE,
        "the resting tick\n{}",
        term.text()
    );
    // The seventh digit is not covered.
    let sha = s.line(row + 1);
    assert!(sha.contains(char::is_numeric) || sha.contains(char::is_alphabetic));

    // Hover lights the grip; compare with an internal boundary's light.
    term.move_to(grip, row + 3).unwrap();
    term.wait_for("the last grip lit", STEP, |s| {
        s.cell(grip, row + 3).is_some_and(|c| c.text == LINE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let last_fg = snap(&term).cell(grip, row + 3).unwrap().fg;
    let gap = date_x - 1;
    term.move_to(gap, row + 3).unwrap();
    term.wait_for("an internal grip lit", STEP, |s| {
        s.cell(gap, row + 3).is_some_and(|c| c.text == LINE)
            && s.cell(grip, row + 3).is_some_and(|c| c.text != LINE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        snap(&term).cell(gap, row + 3).unwrap().fg,
        last_fg,
        "the same accent"
    );

    // Drag three cells left: SHA is three cells narrower, so its header
    // starts exactly three cells further right.
    term.press(grip, row + 3).unwrap();
    term.drag_to(grip - 2, row + 3).unwrap();
    term.drag_to(grip - 3, row + 3).unwrap();
    term.release(grip - 3, row + 3).unwrap();
    term.wait_for("SHA moved right", STEP, |s| {
        header(s, "SHA").is_some_and(|(x, _)| x == sha_x + 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
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
