//! A transfer that runs as a job is named once: by the jobs indicator (with
//! its Cancel), not again by the breadcrumb's "pushing..." spinner.
//!
//! The daemon's `GITPOULPI_TEST_TRANSFER_DELAY_MS` keeps the push running
//! (nothing is transferred); the test waits past the breadcrumb's 150 ms
//! threshold so the old label would have shown.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

#[test]
fn a_running_push_shows_in_the_indicator_and_not_in_the_breadcrumb() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("ahead commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-footer-no-dup")
        .config_dir(config.path())
        .size(160, 40)
        .no_exit_pause()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", "25000")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("ahead commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-g").unwrap();
    term.send_keys("P").unwrap();
    term.wait_for("the indicator with the running push", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Push: ") && l.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Well past the 150 ms the breadcrumb waits before it talks.
    std::thread::sleep(Duration::from_millis(1500));
    let text = term.text();
    assert!(
        text.contains("Push: "),
        "the indicator names the job:\n{text}"
    );
    assert!(
        !text.contains("pushing..."),
        "the breadcrumb must not repeat the job:\n{text}"
    );
    // The time spent moved from the breadcrumb to the indicator, and ticks.
    let seconds = |s: &xtask::pty::Screen| -> Option<u32> {
        let text = s.text();
        let line = text
            .lines()
            .find(|l| l.contains("Push: ") && l.contains("[x]"))?;
        let before = line.split("[x]").next()?.trim_end();
        let digits = before.strip_suffix('s')?;
        digits.rsplit(' ').next()?.parse().ok()
    };
    let first = seconds(&term.screen()).unwrap_or_else(|| panic!("no seconds:\n{}", term.text()));
    term.wait_for("the counter advances", STEP, |s| {
        seconds(s).is_some_and(|n| n >= first + 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Leave nothing running: click the indicator's Cancel.
    let (col, row) = term.screen().find("[x]").expect("the [x] button");
    term.click(col + 1, row).unwrap();
}
