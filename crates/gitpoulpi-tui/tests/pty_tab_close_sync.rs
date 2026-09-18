//! The tab bar and the repository on screen agree after a tab is closed:
//! the tab the window shows is the one the bar marks active, whether the
//! tab was closed from the TUI (Ctrl+W) or by `gitpoulpi repo close`.
//!
//! The external half drives the session file exactly as the CLI's
//! `repo open` / `repo close` do (`gitpoulpi-commands::repo_cmd`).

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-tab-sync";

/// A fixture repository whose folder is called `name` (the tab's label).
fn repo(name: &str, file: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(file, "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

fn session_file(config: &Path) -> std::path::PathBuf {
    config.join("sessions").join(format!("{SESSION}.toml"))
}

/// `gitpoulpi repo open <path>` on the session file.
fn cli_open(config: &Path, path: &Path) {
    let file = session_file(config);
    let mut s = GlobalSession::load(&file).unwrap();
    s.open(path);
    s.save(&file).unwrap();
}

/// `gitpoulpi repo close <path>` on the session file.
fn cli_close(config: &Path, path: &Path) {
    let file = session_file(config);
    let mut s = GlobalSession::load(&file).unwrap();
    s.close_path(path);
    s.save(&file).unwrap();
}

fn start(alpha: &Path, config: &Path) -> xtask::pty::Terminal {
    common::launch(alpha)
        .session(SESSION)
        .config_dir(config)
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .mouse_capture()
        .spawn()
}

/// Opens `name` as the shown tab through the management page.
fn switch_to(term: &mut xtask::pty::Terminal, name: &str, commit: &str) {
    term.send_keys("C-o").unwrap();
    term.wait_for("management", STEP, |s| s.contains("OPEN REPOSITORIES"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("/").unwrap();
    term.send_text(name).unwrap();
    term.wait_for("filter", STEP, |s| s.contains("Enter open best match"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the repo", STEP, |s| {
        s.contains(commit) && !s.contains("OPEN REPOSITORIES")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Whether the screen shows `repo` as the repository on screen and the
/// tab bar lists exactly `tabs`.
fn agrees(s: &str, shown: &str, gone: &str, tabs: &[&str]) -> bool {
    let bar = s.lines().nth(1).unwrap_or("");
    s.contains(shown) && !s.contains(gone) && tabs.iter().all(|t| bar.contains(t))
}

#[test]
fn closing_the_shown_tab_from_the_tui_lands_on_the_remaining_one() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, config.path());
    cli_open(config.path(), &bravo);
    switch_to(&mut term, "moonbase", "bravo-only-commit");
    term.send_keys("C-w").unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("Close"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("orbit shown, one tab", STEP, |s| {
        let s = s.text();
        agrees(&s, "alpha-only-commit", "bravo-only-commit", &["orbit"])
            && !s.lines().nth(1).unwrap_or("").contains("moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `gitpoulpi repo close` of the tab on screen, from outside: the window
/// follows the session file at its next frame instead of keeping the
/// closed repository under a bar that marks another tab.
#[test]
fn closing_the_shown_tab_from_the_cli_lands_on_the_remaining_one() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, config.path());
    cli_open(config.path(), &bravo);
    switch_to(&mut term, "moonbase", "bravo-only-commit");
    cli_close(config.path(), &bravo);
    // Any input is a frame: the sync is not a timer.
    let mut tries = 0;
    let ok = loop {
        term.send_keys("Down").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        let s = term.text();
        if agrees(&s, "alpha-only-commit", "bravo-only-commit", &["orbit"])
            && !s.lines().nth(1).unwrap_or("").contains("moonbase")
        {
            break true;
        }
        tries += 1;
        if tries > 20 {
            break false;
        }
    };
    assert!(ok, "{}", term.text());
    term.stop().unwrap();
}

/// A tab opened from outside shows up in the bar at the next frame.
#[test]
fn a_tab_opened_from_the_cli_shows_in_the_bar() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, config.path());
    cli_open(config.path(), &bravo);
    let mut tries = 0;
    loop {
        term.send_keys("Down").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        if term
            .text()
            .lines()
            .nth(1)
            .unwrap_or("")
            .contains("moonbase")
        {
            break;
        }
        tries += 1;
        assert!(tries < 20, "{}", term.text());
    }
    // The window keeps the repository it was showing.
    assert!(term.text().contains("alpha-only-commit"), "{}", term.text());
    term.stop().unwrap();
}
