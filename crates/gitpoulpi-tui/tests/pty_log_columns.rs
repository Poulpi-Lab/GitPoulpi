//! The Log column layout is global and persisted (`[ui] log_columns_*`):
//! hide a column through the header menu, quit, relaunch with the same
//! config directory -- in the same repo and in another one -- and it is
//! still hidden. Temp config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture(message: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit(message)
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &Path, message: &str) -> Terminal {
    let message = message.to_string();
    common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .ready_when("the fixture loaded", move |s| s.contains(&message))
        .spawn()
}

/// Whether the Log header row (the row of the `Message` title) shows `SHA`.
fn header_shows_sha(screen: &xtask::pty::Screen) -> bool {
    let Some((_, row)) = screen.find_all("Message").into_iter().next() else {
        return false;
    };
    screen.find_all("SHA").into_iter().any(|(_, r)| r == row)
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

#[test]
fn a_hidden_log_column_survives_a_restart_and_applies_to_another_repo() {
    let config = tempfile::tempdir().unwrap();
    let repo_a = fixture("column persistence commit");
    let repo_b = fixture("second repository commit");

    let mut term = launch(
        &repo_a,
        "cols-a",
        config.path(),
        "column persistence commit",
    );
    term.wait_for("SHA header", STEP, header_shows_sha)
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = {
        let screen = term.screen();
        let (_, header_row) = screen.find_all("Message").into_iter().next().unwrap();
        let (c, _) = screen
            .find_all("SHA")
            .into_iter()
            .find(|(_, r)| *r == header_row)
            .unwrap();
        (c, header_row)
    };
    term.right_click(col + 1, row).unwrap();
    term.wait_for("the column menu", STEP, |s| s.contains("[x]"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (mc, mr) = {
        let screen = term.screen();
        screen
            .find_all("SHA")
            .into_iter()
            .find(|(_, r)| *r != row)
            .expect("a SHA row in the column menu")
    };
    term.click(mc + 1, mr).unwrap();
    term.wait_for("the SHA box unchecked", STEP, |s| s.contains("[ ]"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("SHA header gone", STEP, |s| !header_shows_sha(s))
        .unwrap_or_else(|e| panic!("{e}"));
    // Give the background save a moment, then quit.
    std::thread::sleep(Duration::from_millis(1000));
    quit(term);

    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap();
    assert!(saved.contains("log_columns_hidden"), "{saved}");
    assert!(saved.contains("sha"), "{saved}");

    // Same repo, fresh process.
    let term = launch(
        &repo_a,
        "cols-a",
        config.path(),
        "column persistence commit",
    );
    term.wait_for("the Log header", STEP, |s| s.contains("Message"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!header_shows_sha(&term.screen()), "{}", term.text());
    quit(term);

    // Another repository, another session, same config directory.
    let term = launch(&repo_b, "cols-b", config.path(), "second repository commit");
    term.wait_for("the Log header", STEP, |s| s.contains("Message"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!header_shows_sha(&term.screen()), "{}", term.text());
    quit(term);
}
