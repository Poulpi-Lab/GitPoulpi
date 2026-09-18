//! The Activity Log's Application tab has a "Clear log" button (key `x`,
//! palette "Activity Log: Clear the Application Log") that asks first, and
//! on confirm empties the application log files of every kept day. Real
//! binary and daemon through a pty (`xtask::pty`), scratch config dir.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn wait(term: &mut Terminal, what: &str, f: impl Fn(&Screen) -> bool) {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn info_rows(screen: &Screen) -> usize {
    screen
        .text()
        .lines()
        .filter(|l| l.contains(" INFO "))
        .count()
}

#[test]
fn clear_log_asks_first_cancel_keeps_the_lines_and_confirm_empties_them() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("clear base")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-app-log-clear")
        .config_dir(config.path())
        .size(160, 44)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("clear base")
        .spawn();
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    wait(&mut term, "the Activity Log", |s| {
        s.contains("Activity Log")
    });
    term.send_keys("Tab").unwrap();
    wait(&mut term, "Application lines and the button", |s| {
        s.contains("Show:") && s.contains("Clear log") && info_rows(s) >= 2
    });
    let before = info_rows(&term.screen());
    let files_before = file_lines(config.path());
    assert!(files_before >= 2, "the daemon and the window log to files");

    // The button asks first; Cancel keeps every line.
    let (c, r) = term.screen().find("Clear log").expect("the button");
    term.click(c + 1, r).expect("click Clear log");
    wait(&mut term, "the confirmation", |s| {
        s.contains("cannot be undone") && s.contains("all kept days")
    });
    let (c, r) = term.screen().find("Cancel").expect("Cancel");
    term.click(c + 1, r).expect("click Cancel");
    wait(&mut term, "dialog closed", |s| {
        !s.contains("cannot be undone")
    });
    assert!(info_rows(&term.screen()) >= before, "{}", term.text());

    // The key twin: x, then y.
    term.send_text("x").unwrap();
    wait(&mut term, "the confirmation by key", |s| {
        s.contains("cannot be undone")
    });
    term.send_text("y").unwrap();
    wait(&mut term, "the log emptied", |s| {
        s.contains("Application log cleared.") && info_rows(s) < before
    });

    // The log files of the scratch config dir were emptied too.
    let after = file_lines(config.path());
    assert!(
        after < files_before,
        "files emptied: {after} lines left, {files_before} before"
    );
}

/// Lines in every application log file under `dir`.
fn file_lines(dir: &std::path::Path) -> usize {
    walk(dir)
        .iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| gitpoulpi_log::file_date(&n.to_string_lossy()))
                .is_some()
        })
        .map(|p| {
            std::fs::read_to_string(p)
                .unwrap_or_default()
                .lines()
                .count()
        })
        .sum()
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}
