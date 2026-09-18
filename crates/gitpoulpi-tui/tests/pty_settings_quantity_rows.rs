//! Settings > Synchronization: the auto-fetch interval and the history
//! rewrite script time limit are a number and a unit on one line, like the
//! log retention. One is changed by mouse (the unit list), the other by keys;
//! `config.toml` keeps each key in its own base unit (minutes, milliseconds)
//! plus the display unit, and an older config still loads. Real binary,
//! daemon and system git through a pty.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const FETCH: &str = "Auto-fetch interval";
const SCRIPT: &str = "Script answer time limit";

fn repo() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("quantity base")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 100)
        .ready("quantity base")
        .spawn()
}

fn open_sync(term: &mut Terminal) {
    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Synchronization");
}

fn row_text(term: &Terminal, label: &str) -> String {
    term.screen()
        .text()
        .lines()
        .find(|l| l.contains(label))
        .unwrap_or_default()
        .to_string()
}

/// The first cell of the number on the row of `label`, and of the unit's
/// arrow.
fn parts(screen: &Screen, label: &str, row: u16) -> ((u16, u16), (u16, u16)) {
    let line = screen.line(row);
    let after = line.find(label).unwrap() + label.len();
    let at = line[after..]
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or_else(|| panic!("no number on the row:\n{}", screen.text()));
    let number = (line[..after + at].chars().count() as u16, row);
    let arrow = screen
        .find_all("\u{25BE}")
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("no unit arrow on the row:\n{}", screen.text()));
    (number, arrow)
}

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn wait_stored(dir: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored(dir).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} never reached config.toml: {}",
            stored(dir)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn two_rows_by_mouse_and_by_keys_keep_their_base_unit() {
    let repo = repo();
    let config = tempfile::tempdir().unwrap();
    // An older file: only the minutes, 60 of them.
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nauto_fetch_interval_minutes = 60\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), "pty-quantity-rows", config.path());
    open_sync(&mut term);

    // The older value reads back in the largest even unit, number and unit
    // on one line; looking changed nothing.
    let (_, fetch_row) = common::scroll_settings_to(&mut term, FETCH, STEP);
    let line = row_text(&term, FETCH);
    assert!(line.contains('1') && line.contains("hour"), "{line}");
    assert!(!stored(config.path()).contains("auto_fetch_interval_unit"));

    // The mouse: click the unit (the list names the units for 1: "minute"
    // and "hour"), pick the minute.
    let screen = term.screen();
    let (_, arrow) = parts(&screen, FETCH, fetch_row);
    term.click(arrow.0, arrow.1).expect("click the unit");
    let list = term
        .wait_for("the unit list", STEP, |s| {
            s.find_all("minute")
                .iter()
                .any(|&(c, r)| r != fetch_row && c >= 70)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = list
        .find_all("minute")
        .into_iter()
        .find(|&(c, r)| r != fetch_row && c >= 70)
        .expect("minute in the list");
    term.click(c + 1, r).expect("pick minutes");
    // The number stays (1): one minute, still the minutes key.
    wait_stored(config.path(), "auto_fetch_interval_minutes = 1\n");
    wait_stored(config.path(), "auto_fetch_interval_unit = \"minutes\"");

    // The mouse: click the number, type 15.
    let screen = term.screen();
    let (number, _) = parts(&screen, FETCH, fetch_row);
    term.click(number.0, number.1).expect("click the number");
    term.send_keys("Backspace").unwrap();
    term.send_text("15").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "auto_fetch_interval_minutes = 15");

    // The keys, on the script time limit (milliseconds under the hood).
    let (_, script_row) = common::scroll_settings_to(&mut term, SCRIPT, STEP);
    let line = row_text(&term, SCRIPT);
    assert!(line.contains('5') && line.contains("seconds"), "{line}");
    let screen = term.screen();
    let (number, _) = parts(&screen, SCRIPT, script_row);
    term.click(number.0, number.1).expect("click the number");
    term.send_keys("Backspace").unwrap();
    term.send_text("2").unwrap();
    term.send_keys("Tab").unwrap();
    wait_stored(config.path(), "script_timeout_ms = 2000");
    wait_stored(config.path(), "script_timeout_unit = \"seconds\"");
    // Tab landed on the unit: Enter opens the list, Up is milliseconds.
    term.send_keys("Enter").unwrap();
    term.wait_for("the unit list", STEP, |s| s.contains("milliseconds"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "script_timeout_ms = 2\n");
    wait_stored(config.path(), "script_timeout_unit = \"milliseconds\"");
    // Back to the number, 1500 milliseconds.
    term.send_keys("BTab").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..4 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("1500").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "script_timeout_ms = 1500");
    let line = row_text(&term, SCRIPT);
    assert!(
        line.contains("1500") && line.contains("milliseconds"),
        "{line}"
    );
    // Over ten minutes is refused with the reason under the row.
    term.send_keys("Enter").unwrap();
    for _ in 0..6 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("600001").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason under the row", STEP, |s| {
        s.contains("is outside 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(stored(config.path()).contains("script_timeout_ms = 1500"));
}
