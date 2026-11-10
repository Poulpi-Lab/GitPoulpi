//! `ui.column_resize` on the Log, in the real binary: `line` (the default)
//! rests with a tick in the header's gap, draws a thin line down the table
//! while the pointer is on the boundary and drags from anywhere along it;
//! `header` has no tick and only the first letter of the next header grabs.
//! The setting itself is switched in Settings > Log & graph. (The Radar's and
//! the commit column's twins are in `pty_radar_columns` and
//! `pty_commit_column`.) Temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COMMIT: &str = "column resize probe commit";
const TICK: &str = "\u{2502}";
const LINE: &str = "\u{2502}";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an earlier commit")
        .unwrap()
        .commit(COMMIT)
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(200, 40)
        .ready_when("the Log header", |s| {
            s.contains(COMMIT) && header(s, "Date").is_some()
        })
        .mouse_capture()
        .start()
}

/// Cell of `title` on the Log's header row (the row of `Message`).
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
}

fn text_at(s: &Screen, col: u16, row: u16) -> String {
    s.cell(col, row).map(|c| c.text.clone()).unwrap_or_default()
}

/// The ticks between the first and the last column title; the panels'
/// own border cells (the same glyph) are outside that span.
fn ticks(s: &Screen, row: u16) -> usize {
    let from = s.find_all("Ref").into_iter().find(|(_, r)| *r == row);
    let to = s.find_all("SHA").into_iter().find(|(_, r)| *r == row);
    let (Some((from, _)), Some((to, _))) = (from, to) else {
        return 0;
    };
    (from..to).filter(|c| text_at(s, *c, row) == TICK).count()
}

#[test]
fn line_mode_ticks_the_header_draws_a_thin_line_on_hover_and_drags_from_mid_height() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-colresize-line", "");
    let screen = term.screen();
    let (date_x, row) = header(&screen, "Date").unwrap();
    let gap = date_x - 1;
    assert_eq!(text_at(&screen, gap, row), TICK, "{}", term.text());
    assert!(ticks(&screen, row) >= 3, "one tick per boundary");
    assert_ne!(text_at(&screen, gap, row + 4), LINE, "nothing at rest");

    // Hovering the gap well below the header draws the line from the header
    // to the table's last row, and nowhere else.
    term.move_to(gap, row + 4).unwrap();
    term.wait_for("the thin line", STEP, |s| {
        text_at(s, gap, row + 4) == LINE
            && text_at(s, gap, row + 1) == LINE
            && text_at(s, gap, row) == LINE
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert_ne!(text_at(&screen, gap - 1, row + 4), LINE, "thin: one cell");
    assert_ne!(text_at(&screen, gap + 1, row + 4), LINE, "thin: one cell");

    // Off it the line goes and the tick is back.
    term.move_to(gap + 6, row + 4).unwrap();
    term.wait_for("the line gone", STEP, |s| {
        text_at(s, gap, row + 4) != LINE && text_at(s, gap, row) == TICK
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A drag that starts mid-height resizes: Date follows the pointer.
    term.press(gap, row + 4).unwrap();
    term.drag_to(gap - 2, row + 4).unwrap();
    term.drag_to(gap - 3, row + 4).unwrap();
    term.wait_for("Date moved left", STEP, |s| {
        header(s, "Date").is_some_and(|(x, _)| x == date_x - 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(gap - 3, row + 4).unwrap();
    term.stop().unwrap();
}

/// After a drag the thin line is gone once the button is up, unless the
/// pointer still sits on the boundary (then it is a hover). Red when the
/// highlight a drag left on its boundary is kept after the release.
#[test]
fn the_line_does_not_stay_after_a_drag_is_released_away_from_the_boundary() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-colresize-release", "");
    let screen = term.screen();
    let (date_x, row) = header(&screen, "Date").unwrap();
    let gap = date_x - 1;
    term.press(gap, row + 4).unwrap();
    term.drag_to(gap - 2, row + 4).unwrap();
    term.drag_to(gap - 3, row + 4).unwrap();
    term.wait_for("Date moved left", STEP, |s| {
        header(s, "Date").is_some_and(|(x, _)| x == date_x - 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Released far from the boundary: no line anywhere down it.
    term.release(gap + 25, row + 4).unwrap();
    let new_gap = gap - 3;
    term.wait_for("the line is gone and the tick is back", STEP, |s| {
        text_at(s, new_gap, row + 4) != LINE && text_at(s, new_gap, row) == TICK
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Released on the boundary: the hover line, gone with the next move away.
    term.press(new_gap, row + 4).unwrap();
    term.release(new_gap, row + 4).unwrap();
    term.wait_for("the hover line", STEP, |s| {
        text_at(s, new_gap, row + 4) == LINE && text_at(s, new_gap, row) == LINE
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.move_to(new_gap + 8, row + 4).unwrap();
    term.wait_for("the line gone", STEP, |s| {
        text_at(s, new_gap, row + 4) != LINE && text_at(s, new_gap, row) == TICK
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn header_mode_has_no_tick_and_grabs_the_first_letter_of_the_next_header_only() {
    let repo = fixture();
    let (mut term, _config) = launch(
        &repo,
        "pty-colresize-header",
        "[ui]\ncolumn_resize = \"header\"\n",
    );
    let screen = term.screen();
    let (date_x, row) = header(&screen, "Date").unwrap();
    assert_eq!(ticks(&screen, row), 0, "no tick in the header");
    let plain = screen.cell(date_x, row).unwrap().bg;

    // The gap below the header is a plain cell: no line.
    term.move_to(date_x - 1, row + 4).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert_ne!(text_at(&term.screen(), date_x - 1, row + 4), LINE);
    // The first letter of the header lights.
    term.move_to(date_x, row).unwrap();
    term.wait_for("the first letter lit", STEP, |s| {
        s.cell(date_x, row).is_some_and(|c| c.bg != plain)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A press mid-height does not drag; one on the header cell does.
    term.press(date_x, row + 4).unwrap();
    term.drag_to(date_x - 3, row + 4).unwrap();
    term.release(date_x - 3, row + 4).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(header(&term.screen(), "Date").unwrap().0, date_x);
    term.drag((date_x, row), (date_x - 3, row)).unwrap();
    term.wait_for("Date moved left", STEP, |s| {
        header(s, "Date").is_some_and(|(x, _)| x == date_x - 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Opens Settings > Log & graph and picks `to` in the Column resizing combo,
/// which shows `from`; `below` is where `to` sits in the popup relative to the
/// ticked `from` (+1 under it, -1 above it).
fn switch(term: &mut Terminal, config: &std::path::Path, from: &str, to: &str, below: i32) {
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Log & graph");
    let (col, row) = common::scroll_settings_to_choice(term, from, STEP);
    term.click(col + 1, row).unwrap();
    let ticked = format!("\u{2713} {from}");
    let screen = term
        .wait_for("the popup", STEP, |s| s.contains(&ticked))
        .unwrap_or_else(|e| panic!("{e}"));
    let (tc, tr) = screen.find(&ticked).unwrap();
    let target = u16::try_from(i32::from(tr) + below).unwrap();
    assert!(
        screen.line(target).contains(to),
        "{to} under the tick:\n{}",
        term.text()
    );
    term.click(tc + 2, target).unwrap();
    let want = to.to_lowercase();
    let deadline = std::time::Instant::now() + STEP;
    let stored_ok = |cfg: &str| cfg.contains(&format!("column_resize = \"{want}\""));
    loop {
        let stored = std::fs::read_to_string(config.join("config.toml")).unwrap_or_default();
        if stored_ok(&stored) {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "not stored: {stored}");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").unwrap();
}

#[test]
fn switching_to_header_in_settings_removes_the_ticks_live() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-colresize-to-header", "");
    let (_, row) = header(&term.screen(), "Date").unwrap();
    assert!(ticks(&term.screen(), row) >= 3);
    switch(&mut term, config.path(), "Line", "Header", 1);
    term.wait_for("the ticks gone", STEP, |s| {
        header(s, "Date").is_some_and(|(_, r)| ticks(s, r) == 0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn switching_to_line_in_settings_brings_the_ticks_back_live() {
    let repo = fixture();
    let (mut term, config) = launch(
        &repo,
        "pty-colresize-to-line",
        "[ui]\ncolumn_resize = \"header\"\n",
    );
    let (_, row) = header(&term.screen(), "Date").unwrap();
    assert_eq!(ticks(&term.screen(), row), 0);
    switch(&mut term, config.path(), "Header", "Line", -1);
    term.wait_for("the ticks back", STEP, |s| {
        header(s, "Date").is_some_and(|(_, r)| ticks(s, r) >= 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// An overlay always wins over the hover line: with the pointer on a Log
/// column boundary, opening the command palette must not leave the line
/// painted through it (it ate a letter of the title and of every row). Red
/// when the strip is painted after the overlays again.
#[test]
fn the_hover_line_never_draws_over_the_command_palette() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-colresize-palette", "");
    let screen = term.screen();
    let (date_x, row) = header(&screen, "Date").unwrap();
    let gap = date_x - 1;
    term.move_to(gap, row + 4).unwrap();
    term.wait_for("the thin line", STEP, |s| text_at(s, gap, row + 4) == LINE)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (_, title_row) = screen.find_all("Command Palette")[0];
    // The palette's column at the boundary: none of its body rows shows the
    // line glyph, which would only be the leftover hover.
    let lined = (title_row + 2..title_row + 6)
        .filter(|r| text_at(&screen, gap, *r) == LINE)
        .count();
    assert_eq!(lined, 0, "{}", term.text());
    term.stop().unwrap();
}
