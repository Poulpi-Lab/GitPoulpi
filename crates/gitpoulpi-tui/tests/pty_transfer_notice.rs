//! A push or a pull says how it went in the status line and nowhere else:
//! no framed notification card, on success or on failure. A failure is in
//! the error style, a click on it (or the palette's `Activity Log: Open the
//! Last Error`) opens the Activity Log on its entry, and the details stay
//! there. Real binary, daemon and system git through a pty (`xtask::pty`);
//! the remote is a local bare repository, there is no network.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn with_origin() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("transfer base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("transfer unpushed")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(180, 48)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("transfer unpushed")
        .spawn()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// No framed notification: neither the card's buttons nor a "done"/"failed"
/// title in a box.
fn no_card(screen: &Screen) {
    for needle in [
        "Push done",
        "Pull done",
        "Push failed \u{2500}",
        "Pull failed \u{2500}",
    ] {
        assert!(
            !screen.contains(needle),
            "no card, found {needle:?}:\n{}",
            screen.text()
        );
    }
    for button in [" Copy ", " Details "] {
        assert!(
            !screen.contains(button),
            "no card buttons:\n{}",
            screen.text()
        );
    }
}

/// The status line (the row that holds the Activity button), and its row.
fn status_line(screen: &Screen) -> (usize, String) {
    screen
        .text()
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("Activity ("))
        .last()
        .map(|(i, l)| (i, l.to_string()))
        .unwrap_or((0, String::new()))
}

fn status_row(screen: &Screen) -> String {
    status_line(screen).1
}

fn click_pull(term: &mut Terminal) {
    let (col, row) = term
        .screen()
        .find("Pull")
        .expect("the toolbar's Pull button");
    term.click(col + 1, row).expect("click Pull");
}

fn wait_until(mut done: impl FnMut() -> bool) {
    let start = std::time::Instant::now();
    while !done() {
        assert!(start.elapsed() < STEP, "timed out");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn break_origin(repo: &gitpoulpi_testkit::FixtureRepo) {
    repo.git(&["remote", "set-url", "origin", "/no/such/remote.git"]);
}

#[test]
fn a_push_that_went_through_has_no_card() {
    let repo = with_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-transfer-push", config.path());
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
    // The push really ran (the remote has the commit), and nothing popped.
    wait_until(|| {
        repo.git_in_remote(&["log", "--format=%s", "main"])
            .contains("transfer unpushed")
    });
    std::thread::sleep(Duration::from_millis(800));
    no_card(&term.screen());
}

#[test]
fn a_pull_that_went_through_has_no_card() {
    let repo = with_origin();
    repo.git(&["push", "origin", "main"]);
    repo.push_from_elsewhere("main", "transfer from elsewhere");
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-transfer-pull", config.path());
    click_pull(&mut term);
    wait_until(|| {
        repo.git(&["log", "--format=%s"])
            .contains("transfer from elsewhere")
    });
    std::thread::sleep(Duration::from_millis(800));
    no_card(&term.screen());
}

#[test]
fn a_pull_that_fails_says_so_in_the_status_line_and_opens_its_entry() {
    let repo = with_origin();
    break_origin(&repo);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-transfer-pull-fail", config.path());
    click_pull(&mut term);
    let screen = term
        .wait_for("the error in the status line", STEP, |s| {
            status_row(s).contains("error: P")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    no_card(&screen);
    assert!(
        screen.contains("Activity (1)"),
        "the unread error is counted:\n{}",
        screen.text()
    );

    // A click on the message opens the Activity Log on the entry.
    let (row, line) = status_line(&screen);
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
    term.wait_for("the entry open in the Activity Log", STEP, |s| {
        s.contains("Activity Log") && s.contains("Pull failed") && s.contains("exit code:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_palette_opens_the_last_error_too() {
    let repo = with_origin();
    break_origin(&repo);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-transfer-push-fail", config.path());
    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
    let screen = term
        .wait_for("the error in the status line", STEP, |s| {
            status_row(s).contains("error: P")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    no_card(&screen);
    palette(
        &mut term,
        "Open the Last Error",
        "Activity Log: Open the Last Error",
    );
    term.wait_for("the entry open in the Activity Log", STEP, |s| {
        s.contains("Activity Log") && s.contains("Push failed") && s.contains("exit code:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
