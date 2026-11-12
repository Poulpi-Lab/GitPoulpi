//! Fitting a commit column part to its content: a double click on the
//! part's boundary, `=` / `+` on the part the keys size, the header menu's
//! rows and the palette's. Hand-paced clicks (`Terminal::double_click` writes
//! both presses in one go, so no frame ever separates them). Waits are whole
//! state predicates, so they hold under `GITPOULPI_PTY_TEAR_MS`. Temp config
//! dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
/// The column's title, with the spaces it is framed by.
const TITLE: &str = " Commits ";
/// The first word of the author's name is wider than the part's automatic
/// share (8 cells), so it starts cut.
const FIRST_NAME: &str = "Bartholomewlongfirst";
const AUTHOR: &str = "Bartholomewlongfirst Surname";
const SUMMARY: &str = "probe summary";

fn ms(n: u64) {
    std::thread::sleep(Duration::from_millis(n));
}

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .config("user.name", AUTHOR)
        .unwrap()
        .commit_file("c.txt", "gamma\n", SUMMARY)
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains("c.txt")
        })
        .mouse_capture()
        .start()
}

fn snap(term: &Terminal) -> Screen {
    term.wait_for_stable(Duration::from_millis(250), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Opens HEAD's file in the diff (the commit column opens beside it).
fn open_head_file(term: &mut Terminal) {
    ms(450);
    let (c, r) = snap(term)
        .find_all("c.txt")
        .into_iter()
        .max_by_key(|(c, _)| *c)
        .unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff and the column", STEP, |s| {
        s.contains("+gamma") && s.contains(TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    widen(term);
}

/// Drags the splitter right until the column has room for every part (the
/// date shows from a wider inner width).
fn widen(term: &mut Terminal) {
    let s = snap(term);
    let (tc, tr) = s.find(TITLE).unwrap();
    let line: Vec<char> = s.line(tr).chars().collect();
    let right = line
        .iter()
        .enumerate()
        .skip(tc as usize)
        .find(|(_, c)| **c == '\u{256e}')
        .map(|(i, _)| i as u16)
        .unwrap();
    term.drag((right + 1, tr + 6), (right + 31, tr + 6))
        .unwrap();
    term.wait_for("the column and its Date part", STEP, |s| {
        header(s, "Date").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The column's cells on `row`, between its borders.
fn column_line(s: &Screen, row: u16) -> String {
    let Some((tc, tr)) = s.find(TITLE) else {
        return String::new();
    };
    let x0 = tc.saturating_sub(1) as usize;
    let line: Vec<char> = s.line(tr).chars().collect();
    let x1 = line
        .iter()
        .enumerate()
        .skip(x0)
        .find(|(_, c)| **c == '\u{256e}')
        .map_or(x0 + 40, |(i, _)| i);
    s.line(row).chars().skip(x0).take(x1 - x0).collect()
}

/// The header row of the column (the row with `Author`) and a part's title
/// cell on it.
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, tr) = s.find(TITLE)?;
    (tr + 1..tr + 4).find_map(|row| {
        let line = column_line(s, row);
        let at = line.find(title)?;
        let x0 = s.find(TITLE)?.0.saturating_sub(1);
        Some((x0 + line[..at].chars().count() as u16, row))
    })
}

/// Everything the column shows below its header.
fn body(s: &Screen) -> String {
    let Some((_, row)) = header(s, "Author") else {
        return String::new();
    };
    (row + 1..s.rows())
        .map(|r| column_line(s, r))
        .collect::<Vec<_>>()
        .join("\n")
}

fn full_author(s: &Screen) -> bool {
    body(s).contains(FIRST_NAME)
}

fn full_date(s: &Screen) -> bool {
    // The whole `YYYY-MM-DD`, not the `MM-DD` the automatic share keeps.
    body(s).split_whitespace().any(|w| {
        w.len() == 10
            && w.as_bytes()[4] == b'-'
            && w.chars().filter(|c| c.is_ascii_digit()).count() == 8
    })
}

fn hand_press(term: &mut Terminal, col: u16, row: u16) {
    term.press(col, row).unwrap();
    ms(60);
}

/// A hand-paced double click on the Author boundary (the cell before the
/// Date header, the Author part's last separator) shows the whole first name
/// of the author; the second press starts no drag.
///
/// Falsified: with the `fit_one` call of the double-click branch (`mouse_left_menus`)
/// removed the name stays cut (red at "the whole first name").
#[test]
fn a_double_click_on_a_boundary_fits_the_part_and_starts_no_drag() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-fit-double");
    open_head_file(&mut term);
    let s = snap(&term);
    assert!(!full_author(&s), "cut before the fit\n{}", term.text());
    let (date_x, row) = header(&s, "Date").unwrap_or_else(|| panic!("{}", term.text()));
    let (author_x, _) = header(&s, "Author").unwrap();
    let gap = date_x - 1;

    hand_press(&mut term, gap, row + 2);
    term.release(gap, row + 2).unwrap();
    ms(120);
    hand_press(&mut term, gap, row + 2);
    term.wait_for("the whole first name", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The button is still down: a far drag after the fit must not narrow it.
    term.drag_to(author_x + 1, row + 2).unwrap();
    ms(300);
    let held = snap(&term);
    assert!(full_author(&held), "no drag after the fit\n{}", term.text());
    term.release(author_x + 1, row + 2).unwrap();
    term.wait_for("still fitted", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The keyboard twin: a click on a row gives the column the keys, `Tab` picks
/// the part to size (Title, then Author), `=` fits it.
#[test]
fn the_fit_key_fits_the_picked_part() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-fit-key");
    open_head_file(&mut term);
    assert!(!full_author(&snap(&term)), "{}", term.text());
    let (c, r) = {
        let s = snap(&term);
        let (_, hr) = header(&s, "Author").unwrap();
        let line = column_line(&s, hr + 1);
        let at = line.find(SUMMARY).unwrap();
        (s.find(TITLE).unwrap().0 + at as u16, hr + 1)
    };
    term.click(c, r).unwrap();
    ms(200);
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("=").unwrap();
    term.wait_for("the whole first name", STEP, full_author)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The header's right-click menu offers "Fit all columns" (and the clicked
/// part's own fit): choosing it fits every part, the date included.
///
/// Falsified: with `fit_every` doing nothing the name and the date stay cut.
#[test]
fn the_header_menu_fits_this_part_and_all_parts() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-fit-menu");
    open_head_file(&mut term);
    let s = snap(&term);
    assert!(!full_author(&s) && !full_date(&s), "{}", term.text());
    let (ax, row) = header(&s, "Author").unwrap();
    term.right_click(ax, row).unwrap();
    term.wait_for("the menu's fit rows", STEP, |s| {
        s.contains("Fit Author to content") && s.contains("Fit all columns to content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = snap(&term).find("Fit Author to content").unwrap();
    term.click(c + 2, r).unwrap();
    term.wait_for("this part fitted", STEP, |s| {
        full_author(s) && !full_date(s) && !s.contains("Fit Author to content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (ax, row) = header(&snap(&term), "Author").unwrap();
    term.right_click(ax, row).unwrap();
    term.wait_for("the menu again", STEP, |s| {
        s.contains("Fit all columns to content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = snap(&term).find("Fit all columns to content").unwrap();
    term.click(c + 2, r).unwrap();
    term.wait_for("every part fitted", STEP, |s| {
        full_author(s) && full_date(s) && body(s).contains(SUMMARY)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The palette rows are the twins of the keys.
#[test]
fn the_palette_fits_the_parts() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-cc-fit-palette");
    open_head_file(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("Fit All Parts to Content").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Fit All Parts to Content")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("every part fitted", STEP, |s| {
        full_author(s) && full_date(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
