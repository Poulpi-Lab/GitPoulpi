//! Dragging a Log column boundary resizes the column on its left, dragging
//! a header onto another reorders, and both are persisted to
//! the repository's `log_columns` and `[ui] log_columns_order` and honoured by the next
//! process. (`pty_log_columns.rs` covers hide/show.) Temp config dirs only.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("column layout commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &Path) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .ready_when("the Log header", |s| {
            s.contains("column layout commit") && header(s, "Author").is_some()
        })
        .mouse_capture()
        .spawn()
}

/// Column of `title` on the Log's header row (the row of `Message`).
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
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

fn wait_config(config: &Path, what: &str, ok: impl Fn(&str) -> bool) -> String {
    let deadline = Instant::now() + STEP;
    loop {
        let text = std::fs::read_to_string(config.join("config.toml")).unwrap_or_default();
        if ok(&text) {
            return text;
        }
        assert!(
            Instant::now() < deadline,
            "config.toml never showed {what}:\n{text}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Dragging the boundary left of `Date` by 3 cells narrows `Author` (the
/// column on its left): `Date` moves left by as much, the width is saved,
/// and a fresh process draws the same layout. `grab_dx` is where the press
/// lands relative to `Date`'s first letter: the gap before it in `line` mode
/// (the default), the letter itself in `header` mode.
fn drag_and_restart(session: &str, toml: &str, grab_dx: i32, grab_dy: u16) {
    let config = tempfile::tempdir().unwrap();
    if !toml.is_empty() {
        std::fs::write(config.path().join("config.toml"), toml).unwrap();
    }
    let repo = fixture();
    let mut term = launch(&repo, session, config.path());
    let (date_x, row) = header(&term.screen(), "Date").unwrap();
    let grab = (
        u16::try_from(i32::from(date_x) + grab_dx).unwrap(),
        row + grab_dy,
    );
    term.drag(grab, (grab.0 - 3, grab.1)).unwrap();
    let moved = term
        .wait_for("Date moved left", STEP, |s| {
            header(s, "Date").is_some_and(|(x, _)| x < date_x)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let new_x = header(&moved, "Date").unwrap().0;
    assert_eq!(new_x, date_x - 3, "the boundary follows the pointer");
    let state = gitpoulpi_config::repo_data_file(
        repo.root(),
        gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION,
        "repo-state.toml",
    );
    let deadline = Instant::now() + STEP;
    while !std::fs::read_to_string(&state).is_ok_and(|t| t.contains("log_columns")) {
        assert!(Instant::now() < deadline, "no saved width in {state:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
    quit(term);

    let term = launch(&repo, session, config.path());
    let screen = term
        .wait_for("the saved layout", STEP, |s| {
            header(s, "Date").is_some_and(|(x, _)| x == new_x)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    drop(screen);
    quit(term);
}

/// `line` (the default): the press is on the gap before `Date`, on the header
/// row.
#[test]
fn dragging_a_column_boundary_resizes_it_and_the_width_survives_a_restart() {
    drag_and_restart("pty-colresize", "", -1, 0);
}

/// `line`: the same drag starting well below the header, in the table body.
#[test]
fn line_mode_dragging_the_gap_from_mid_height_resizes_it_too() {
    drag_and_restart("pty-colresize-body", "", -1, 5);
}

/// `header`: the press is on `Date`'s first letter.
#[test]
fn header_mode_dragging_the_next_headers_first_letter_resizes_it() {
    drag_and_restart(
        "pty-colresize-header",
        "[ui]\ncolumn_resize = \"header\"\n",
        0,
        0,
    );
}

/// Dragging the `Date` header onto the `Author` header swaps their order;
/// the order is saved and the next process starts with it.
#[test]
fn dragging_a_header_reorders_the_columns_and_the_order_survives_a_restart() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, "pty-colorder", config.path());
    let s = term.screen();
    let (author_x, row) = header(&s, "Author").unwrap();
    let (date_x, _) = header(&s, "Date").unwrap();
    assert!(author_x < date_x);
    // A dragged header lands *before* the column it is dropped on, so
    // moving Date left onto Author is what swaps the pair.
    term.drag((date_x + 1, row), (author_x + 1, row)).unwrap();
    term.wait_for(
        "Date before Author",
        STEP,
        |s| matches!((header(s, "Date"), header(s, "Author")), (Some(d), Some(a)) if d.0 < a.0),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_config(config.path(), "a saved order", |t| {
        t.contains("log_columns_order")
    });
    quit(term);

    let term = launch(&repo, "pty-colorder", config.path());
    term.wait_for(
        "the saved order",
        STEP,
        |s| matches!((header(s, "Date"), header(s, "Author")), (Some(d), Some(a)) if d.0 < a.0),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}
