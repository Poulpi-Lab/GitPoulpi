//! The Log's column chooser (right-click the header) and the header drag.
//!
//! The chooser draws the app's own checkbox (`[x]` / `[ ]`), has a close
//! cross, and is operated by keys (arrows, Space) as well as by clicks. A
//! dragged header shows a ghost of its title that follows the pointer and an
//! insertion bar where it would land; the columns reorder on release, and Esc
//! cancels. Temp config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const MARKER: &str = "\u{2503}";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("chooser and drag commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &Path) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .ready_when("the Log header", |s| {
            s.contains("chooser and drag commit") && header(s, "Author").is_some()
        })
        .mouse_capture()
        .spawn()
}

/// Cell of `title` on the Log's header row (the row of `Message`).
fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
}

fn sha_shown(s: &Screen) -> bool {
    header(s, "SHA").is_some()
}

fn chooser_open(s: &Screen) -> bool {
    s.contains("Columns (click to toggle")
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

fn open_chooser(term: &mut Terminal) {
    let (x, row) = header(&term.screen(), "Author").unwrap();
    term.right_click(x + 1, row).unwrap();
    term.wait_for("the chooser", STEP, chooser_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Red when a glyph other than the app's checkbox is drawn, when a row loses
/// its box, when the cross is missing, or when a key or a click does not
/// toggle (or the cross does not close).
#[test]
fn the_chooser_has_real_checkboxes_a_cross_and_works_by_keys_and_clicks() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, "chooser-keys", config.path());
    open_chooser(&mut term);
    let text = term.text();
    assert!(
        !text.contains('\u{2611}') && !text.contains('\u{2610}'),
        "{text}"
    );
    assert!(text.contains("[x] Author"), "{text}");
    assert!(text.contains("[x] Message"), "{text}");
    assert!(text.contains("(always shown)"), "{text}");
    assert!(text.contains('\u{2715}'), "a close cross\n{text}");

    // Keyboard: Up from the first row wraps through the actions (Reset, Fit all,
    // Fit Author) to the last toggleable one (SHA);
    // Space unchecks it and the popup stays open.
    assert!(sha_shown(&term.screen()), "{}", term.text());
    term.send_keys("Up Up Up Up Up").unwrap();
    term.send_text(" ").unwrap();
    term.wait_for("SHA hidden by Space", STEP, |s| {
        !sha_shown(s) && s.contains("[ ] SHA") && chooser_open(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("SHA shown again by Enter", STEP, |s| {
        sha_shown(s) && s.contains("[x] SHA")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Mouse: a click on a row toggles it.
    let (mc, mr) = term
        .screen()
        .find("[x] SHA")
        .expect("the SHA row of the chooser");
    term.click(mc + 4, mr).unwrap();
    term.wait_for("SHA hidden by a click", STEP, |s| {
        !sha_shown(s) && s.contains("[ ] SHA")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The cross closes the popup.
    let (cx, cy) = term.screen().find('\u{2715}'.to_string().as_str()).unwrap();
    term.click(cx, cy).unwrap();
    term.wait_for("the chooser closed", STEP, |s| !chooser_open(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc closes it too.
    open_chooser(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("Esc closed it", STEP, |s| !chooser_open(s))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// The palette reaches the chooser.
#[test]
fn the_palette_opens_the_chooser() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, "chooser-palette", config.path());
    term.send_keys("C-p").unwrap();
    term.send_text("Log: Choose the Columns to Show").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Log: Choose the Columns to Show")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the chooser", STEP, chooser_open)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// Mid-drag: the ghost of the title is below-right of the pointer, the bar
/// sits on the header row, and nothing has moved yet; after the release the
/// column is where the bar was. Red when the ghost or the bar is missing, or
/// when the order changes before the release.
#[test]
fn a_dragged_header_shows_a_ghost_and_a_bar_and_reorders_on_release() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, "header-drag", config.path());
    let s = term.screen();
    let (author_x, row) = header(&s, "Author").unwrap();
    let (date_x, _) = header(&s, "Date").unwrap();
    assert!(author_x < date_x);
    assert!(
        !s.contains(MARKER),
        "no bar before the drag\n{}",
        term.text()
    );

    term.press(date_x + 1, row).unwrap();
    term.drag_to(date_x - 1, row).unwrap();
    term.drag_to(author_x + 1, row).unwrap();
    term.wait_for("the ghost and the bar mid-drag", STEP, |s| {
        s.find_all(" Date ").iter().any(|(_, r)| *r == row + 1)
            && s.find_all(MARKER).iter().any(|(_, r)| *r == row)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Nothing moved while the button is down.
    let mid = term.screen();
    assert!(
        header(&mid, "Author").unwrap().0 < header(&mid, "Date").unwrap().0,
        "{}",
        term.text()
    );
    // The bar sits at the left edge of the column it would precede.
    let (bar_x, _) = mid
        .find_all(MARKER)
        .into_iter()
        .find(|(_, r)| *r == row)
        .unwrap();
    assert!(
        bar_x <= author_x + 1,
        "bar at {bar_x}, Author at {author_x}"
    );

    term.release(author_x + 1, row).unwrap();
    term.wait_for("Date before Author", STEP, |s| {
        matches!((header(s, "Date"), header(s, "Author")), (Some(d), Some(a)) if d.0 < a.0)
            && !s.contains(MARKER)
            && !s.find_all(" Date ").iter().any(|(_, r)| *r == row + 1)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    quit(term);
}

/// Esc during the drag lets the header go: no ghost, no bar, and the release
/// moves nothing.
#[test]
fn escape_cancels_a_header_drag() {
    let config = tempfile::tempdir().unwrap();
    let repo = fixture();
    let mut term = launch(&repo, "header-drag-esc", config.path());
    let s = term.screen();
    let (author_x, row) = header(&s, "Author").unwrap();
    let (date_x, _) = header(&s, "Date").unwrap();

    term.press(date_x + 1, row).unwrap();
    term.drag_to(date_x - 1, row).unwrap();
    term.drag_to(author_x + 1, row).unwrap();
    term.wait_for("the bar mid-drag", STEP, |s| s.contains(MARKER))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the drag canceled", STEP, |s| {
        !s.contains(MARKER) && s.contains("canceled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(author_x + 1, row).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let after = term.screen();
    assert!(
        header(&after, "Author").unwrap().0 < header(&after, "Date").unwrap().0,
        "the order must not change\n{}",
        term.text()
    );
    quit(term);
}
