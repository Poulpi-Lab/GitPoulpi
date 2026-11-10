//! Settings > General > "Keep log files for": a number and a unit on one
//! line. Set 2 weeks by mouse and by keys, see it persist across a reopen
//! and in `config.toml`, an older config value still loads, and 0 shows the
//! reason under the field. Real binary, daemon and system git through a pty.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LABEL: &str = "Keep log files for";

fn repo() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("retention base")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 100)
        .ready("retention base")
        .spawn()
}

fn open_row(term: &mut Terminal) -> (u16, u16) {
    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "General");
    common::scroll_settings_to(term, LABEL, STEP)
}

/// The cells of the number and of the unit's arrow on the retention row.
fn parts(screen: &Screen, row: u16) -> ((u16, u16), (u16, u16)) {
    let number = screen
        .find_all("1")
        .into_iter()
        .chain(screen.find_all("2"))
        .chain(screen.find_all("3"))
        .find(|&(c, r)| r == row && c > 40)
        .unwrap_or_else(|| panic!("no number on the row:\n{}", screen.text()));
    let arrow = screen
        .find_all("\u{25BE}")
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("no unit arrow on the row:\n{}", screen.text()));
    (number, arrow)
}

/// Waits until the retention row shows every one of `needles` and returns
/// that screen: the file is written before the screen repaints (and the row
/// can move as the page scrolls), so a read of the row right after is stale.
fn wait_row(term: &Terminal, needles: &[&str]) -> Screen {
    term.wait_for(
        &format!("the retention row showing {needles:?}"),
        STEP,
        |s| {
            s.text()
                .lines()
                .find(|l| l.contains(LABEL))
                .is_some_and(|l| needles.iter().all(|n| l.contains(n)))
        },
    )
    .unwrap_or_else(|e| panic!("{e}"))
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
fn two_weeks_by_mouse_then_by_keys_persists_and_zero_is_refused() {
    let repo = repo();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-retention", config.path());
    open_row(&mut term);

    // A fresh config: one week, as a number and a unit on the one line.
    wait_row(&term, &["week", "\u{25BE}"]);

    // The mouse: click the number, type 2, Enter. The row is looked up again
    // on every try (the page scrolls to keep its help in view, and a click
    // placed on a torn frame lands nowhere).
    common::retry_until(
        &mut term,
        "the number stored",
        4,
        STEP,
        |t| {
            if !stored(config.path()).contains("retention_days = 14") {
                let screen = wait_row(t, &["week", "\u{25BE}"]);
                let row = screen.find(LABEL).expect("the row is on screen").1;
                let (number, _) = parts(&screen, row);
                t.click(number.0, number.1).expect("click the number");
                t.send_keys("Backspace").unwrap();
                t.send_text("2").unwrap();
                t.send_keys("Enter").unwrap();
            }
        },
        |_| stored(config.path()).contains("retention_days = 14"),
    );
    wait_stored(config.path(), "retention_unit = \"weeks\"");
    wait_row(&term, &["weeks"]);

    // The mouse: click the unit, a list opens, click "months".
    // Selecting the row scrolled the page to keep its help text in view when
    // the label first showed on the pane's last line, so the row moved: look
    // it up again, never click where it was. The unit list then hangs from
    // the row upward when there is no room below.
    let settled = wait_row(&term, &["weeks", "\u{25BE}"]);
    let row = settled.find(LABEL).expect("the row is on screen").1;
    let (_, arrow) = parts(&settled, row);
    term.click(arrow.0, arrow.1).expect("click the unit");
    let list = term
        .wait_for("the unit list", STEP, |s| {
            s.contains("months") && s.contains("years")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = list
        .find_all("months")
        .into_iter()
        .find(|&(_, r)| r != row)
        .expect("months in the list");
    term.click(c + 1, r).expect("pick months");
    wait_stored(config.path(), "retention_days = 60");
    wait_stored(config.path(), "retention_unit = \"months\"");

    // The keyboard twin: BackTab to the number, type 3, Tab to the unit,
    // Enter opens the list, Down to years, Enter.
    term.send_keys("BTab").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("Backspace").unwrap();
    term.send_text("3").unwrap();
    term.send_keys("Tab").unwrap();
    wait_stored(config.path(), "retention_days = 90");
    term.send_keys("Enter").unwrap();
    term.wait_for("the unit list", STEP, |s| s.contains("years"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "retention_days = 1095");
    wait_stored(config.path(), "retention_unit = \"years\"");

    // Zero is refused, with the reason under the field; nothing is written.
    term.send_keys("BTab").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("Backspace").unwrap();
    term.send_text("0").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason under the field", STEP, |s| {
        s.contains("is outside 1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(stored(config.path()).contains("retention_days = 1095"));
    term.send_keys("Esc").unwrap();
    term.wait_for("the edit cancelled", STEP, |s| !s.contains("is outside 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Close Settings, reopen it: the value is still there.
    term.send_keys("Esc").unwrap();
    term.wait_for("Settings closed", STEP, |s| {
        !s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_row(&mut term);
    wait_row(&term, &["3", "years"]);
}

#[test]
fn an_older_config_value_still_loads_in_the_largest_even_unit() {
    let repo = repo();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[log]\nretention_days = 90\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), "pty-retention-old", config.path());
    open_row(&mut term);
    wait_row(&term, &["3", "months"]);
    // Nothing was rewritten just by looking.
    assert!(!stored(config.path()).contains("retention_unit"));
}
