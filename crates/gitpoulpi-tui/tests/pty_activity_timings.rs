//! The Activity Log: an Application tab then a Repository
//! tab, the operations of the repository listed with how long they took
//! (`log.timings`, `log.extended`), the options in Settings, and a
//! horizontal scrollbar in the Application tab so a long log line is never
//! cut. Real binary, daemon and system git through a pty (`xtask::pty`);
//! the remote is a local bare repository, there is no network.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 44;

/// A repository with a local bare `origin` and one commit not pushed yet.
fn with_local_origin() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("timing base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("timing unpushed")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &Path, cols: u16) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(cols, if cols >= 200 { 100 } else { ROWS })
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("timing unpushed")
        .spawn()
}

fn click_push(term: &mut Terminal) {
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
}

/// The push reached the remote (a push has no card: the status line and
/// the Activity Log say it).
fn wait_pushed(repo: &gitpoulpi_testkit::FixtureRepo) {
    let start = std::time::Instant::now();
    while !repo
        .git_in_remote(&["log", "--format=%s", "main"])
        .contains("timing unpushed")
    {
        assert!(start.elapsed() < STEP, "the push never arrived");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn open_log_by_key(term: &mut Terminal) -> Screen {
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    // The title and the tabs: a torn frame can show the first without the rest.
    term.wait_for("the Activity Log by its key", STEP, |s| {
        s.contains("Activity Log") && s.contains(" Git operations ") && s.contains(" Application ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The screen once two reads a moment apart agree (a torn frame is never the
/// one a later screen is compared with).
fn settled(term: &Terminal) -> Screen {
    let mut last = term.screen();
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        let now = term.screen();
        if now.text() == last.text() {
            return now;
        }
        last = now;
    }
    panic!("the screen never settled:\n{}", last.text());
}

/// Whether a line reads `<label> - <digits> ms`.
fn has_timing(screen: &Screen, label: &str) -> bool {
    screen.text().lines().any(|line| {
        line.find(&format!("{label} - ")).is_some_and(|at| {
            let rest = &line[at + label.len() + 3..];
            let digits = rest.chars().take_while(char::is_ascii_digit).count();
            digits > 0 && rest[digits..].starts_with(" ms")
        })
    })
}

#[test]
fn the_tabs_are_operations_then_application() {
    let repo = with_local_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-timings-tabs", config.path(), COLS);
    let screen = open_log_by_key(&mut term);
    let (ops_col, ops_row) = screen.find(" Git operations ").expect("operations tab");
    let (app_col, app_row) = screen.find(" Application ").expect("Application tab");
    assert_eq!(app_row, ops_row, "one row of tabs");
    assert!(ops_col < app_col, "operations first\n{}", screen.text());
    assert!(
        !screen.contains("Level:"),
        "it opens on the operations, the Application toolbar is not shown"
    );

    // Tab (the keyboard twin of the click) goes to Application, then back.
    term.send_keys("Tab").unwrap();
    term.wait_for("the Application tab", STEP, |s| s.contains("Level:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.wait_for("the operations tab", STEP, |s| !s.contains("Level:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A click on the tab name does the same.
    common::retry_until(
        &mut term,
        "the Application tab by click",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find(" Application ") {
                t.click(c + 2, r).expect("click Application");
            }
        },
        |s| s.contains("Level:"),
    );
}

#[test]
fn a_push_and_the_opening_are_listed_with_their_duration() {
    let repo = with_local_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-timings-push", config.path(), COLS);
    click_push(&mut term);
    wait_pushed(&repo);
    open_log_by_key(&mut term);
    let screen = term
        .wait_for("the timing lines", STEP, |s| {
            has_timing(s, "Push")
                && has_timing(s, "Open repository")
                && has_timing(s, "Load commit graph")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        has_timing(&screen, "Load commit graph"),
        "{}",
        screen.text()
    );

    // The same lines are in the application log, at info level.
    let logs = config.path().join("cache").join("logs");
    let text = std::fs::read_dir(&logs)
        .expect("the log folder exists")
        .flatten()
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect::<String>();
    assert!(
        text.lines()
            .any(|l| l.contains("INFO") && l.contains("Push - ") && l.contains(" ms")),
        "{text}"
    );
}

#[test]
fn timings_off_hides_the_durations_and_extended_adds_background_operations() {
    let repo = with_local_origin();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[log]\ntimings = false\nextended = true\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), "pty-timings-off", config.path(), COLS);
    click_push(&mut term);
    wait_pushed(&repo);
    open_log_by_key(&mut term);
    // The log is loaded once the push is listed (without its duration).
    term.wait_for("the push listed", STEP, |s| s.contains("Push done"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = settled(&term);
    assert!(
        !has_timing(&screen, "Push") && !screen.contains(" ms"),
        "no durations when log.timings is off\n{}",
        screen.text()
    );
    term.send_keys("Esc").unwrap();

    // Extended on, timings on: the background refresh is listed too.
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[log]\nextended = true\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), "pty-timings-extended", config.path(), COLS);
    click_push(&mut term);
    open_log_by_key(&mut term);
    term.wait_for("the background refresh", STEP, |s| has_timing(s, "Refresh"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Without extended the refresh is not listed (but is in the log file).
    let plain = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-timings-plain", plain.path(), COLS);
    click_push(&mut term);
    open_log_by_key(&mut term);
    term.wait_for("the push line", STEP, |s| has_timing(s, "Push"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!has_timing(&settled(&term), "Refresh"), "{}", term.text());
}

#[test]
fn the_log_options_are_in_settings_and_write_the_config() {
    let repo = with_local_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-timings-settings", config.path(), 200);
    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "General");
    // The log rows sit far down the page, below the fold: scroll to them.
    common::scroll_settings_to(&mut term, "Extended logging", STEP);
    let stored = || std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();

    // A toggle: one click on the row flips it.
    // Retried until saved: a click aimed at a scrolling or torn frame misses.
    // (Never toggled back: it stops once the config says true.)
    common::retry_until(
        &mut term,
        "the toggle saved",
        4,
        Duration::from_secs(10),
        |t| {
            if !stored().contains("extended = true") {
                common::scroll_settings_to(t, "Extended logging", STEP);
                let (c, r) = settled(t).find("Extended logging").expect("the row");
                t.click(c + 1, r).expect("click the toggle row");
            }
        },
        |_| stored().contains("extended = true"),
    );

    // The retention is a number and a unit on one line (the full flow is
    // `pty_settings_retention.rs`); the default is one week.
    let (_, _) = common::scroll_settings_to(&mut term, "Keep log files for", STEP);
    term.wait_for("the retention row with its unit", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Keep log files for") && l.contains("week"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The level is a dropdown too.
    let list = common::retry_until(
        &mut term,
        "the levels",
        4,
        Duration::from_secs(10),
        |t| {
            let (c, r) = common::scroll_settings_to(t, "Log level", STEP);
            t.click(c + 1, r).expect("click the level row");
        },
        |s| s.contains("Warning") && s.contains("Trace") && s.contains("Debug"),
    );
    let (c, r) = list.find("Debug").unwrap();
    term.click(c + 1, r).expect("pick Debug");
    wait_stored(&stored, "level = \"debug\"");
}

fn wait_stored(stored: &dyn Fn() -> String, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored().contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} never reached config.toml: {}",
            stored()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// A repository whose `origin` cannot be reached: the failed push leaves a
/// log line longer than the window.
fn unreachable_origin() -> gitpoulpi_testkit::FixtureRepo {
    let repo = with_local_origin();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        "http://127.0.0.1:1/a-long-path/to/a/repository-that-is-not-there.git",
    ]);
    repo
}

#[test]
fn a_long_application_line_scrolls_by_key_track_press_and_drag() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    // Narrow enough that the failure line overflows the window.
    let mut term = launch(repo.root(), "pty-timings-hscroll", config.path(), 100);
    click_push(&mut term);
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("error: Push failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_log_by_key(&mut term);
    term.send_keys("Tab").unwrap();
    let screen = term
        .wait_for("the long line, cut at the edge", STEP, |s| {
            s.contains("Level:") && s.contains("Push failed:") && s.contains("\u{2192}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The end of that line, from the file the window reads.
    let logs = config.path().join("cache").join("logs");
    let text = std::fs::read_dir(&logs)
        .expect("the log folder exists")
        .flatten()
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect::<String>();
    let line = text
        .lines()
        .find(|l| l.contains("Push failed:"))
        .expect("the failure is in the log");
    let end: String = line
        .trim_end()
        .chars()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert!(
        !screen.text().contains(end.trim()),
        "the end of the line is cut off before scrolling: {end:?}\n{}",
        screen.text()
    );

    // The keyboard: Right scrolls a step, Home comes back.
    let before = settled(&term).text();
    term.send_keys("Right").unwrap();
    term.wait_for("the view moved sideways", STEP, |s| s.text() != before)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Home").unwrap();
    term.wait_for("back at the start", STEP, |s| s.text() == before)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The bar: the left and right arrows sit at the ends of its row.
    let screen = term.screen();
    let (left, bar_row) = screen.find("\u{2190}").expect("the horizontal bar");
    let right = screen
        .find_all("\u{2192}")
        .into_iter()
        .find(|(_, r)| *r == bar_row)
        .expect("its right arrow")
        .0;
    // A press on the bare track near the right end jumps the thumb there
    // and shows the end of the line.
    term.press(right - 2, bar_row).unwrap();
    term.release(right - 2, bar_row).unwrap();
    term.wait_for("the end of the line", STEP, |s| s.contains(end.trim()))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Dragging the thumb back to the left end returns to the start.
    term.press(right - 2, bar_row).unwrap();
    term.drag_to(left + 1, bar_row).unwrap();
    term.release(left + 1, bar_row).unwrap();
    term.wait_for("the start of the line again", STEP, |s| {
        !s.contains(end.trim())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A failed push whose error line (the `unable to access '<url>'` message)
/// is longer than the window: the remote's URL carries a marker at its end.
fn long_url_origin() -> gitpoulpi_testkit::FixtureRepo {
    let repo = with_local_origin();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://127.0.0.1:1/{}/tail-marker.git", "x".repeat(150)),
    ]);
    repo
}

#[test]
fn a_long_operations_line_scrolls_by_key_track_press_and_drag() {
    let repo = long_url_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-timings-ops-hscroll", config.path(), 100);
    click_push(&mut term);
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("error: Push failed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The Activity Log opens on the first tab: open the entry.
    open_log_by_key(&mut term);
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the long error line, cut at the edge", STEP, |s| {
            s.contains("unable to access") && s.contains("\u{2192}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains("tail-marker"),
        "the end of the line is cut off before scrolling\n{}",
        screen.text()
    );

    // The keyboard: Right scrolls a step (the tab stays), Home comes back.
    let before = screen.text();
    term.send_keys("Right").unwrap();
    term.wait_for("the view moved sideways", STEP, |s| s.text() != before)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen().contains(" Git operations "),
        "{}",
        term.text()
    );
    for _ in 0..30 {
        term.send_keys("Right").unwrap();
    }
    term.wait_for("the end of the line", STEP, |s| s.contains("tail-marker"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Home").unwrap();
    term.wait_for("back at the start", STEP, |s| !s.contains("tail-marker"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The bar: arrows at the ends of its row. A press on the bare track near
    // the right end jumps the thumb there and shows the end of the line.
    let screen = term.screen();
    let (left, bar_row) = screen.find("\u{2190}").expect("the horizontal bar");
    let right = screen
        .find_all("\u{2192}")
        .into_iter()
        .find(|(_, r)| *r == bar_row)
        .expect("its right arrow")
        .0;
    term.press(right - 2, bar_row).unwrap();
    term.release(right - 2, bar_row).unwrap();
    term.wait_for("the end of the line by the bar", STEP, |s| {
        s.contains("tail-marker")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Dragging the thumb back to the left end returns to the start.
    term.press(right - 2, bar_row).unwrap();
    term.drag_to(left + 1, bar_row).unwrap();
    term.release(left + 1, bar_row).unwrap();
    term.wait_for("the start of the line again", STEP, |s| {
        !s.contains("tail-marker")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
