//! A failed operation has no framed card: the status line words it in red
//! (`error: Fetch failed: The remote could not be reached.`), whichever view
//! is in front; a click on it, or the palette's `Activity Log: Open the Last
//! Error`, opens the Activity Log on the entry, which keeps the details, Copy
//! and the remedy button (Retry...) with their keyboard twins.
//!
//! Real binary, daemon and system git through a pty (`xtask::pty`); the
//! failing remote is a closed local port, so there is no network. The
//! pushes and pulls are in `pty_transfer_notice`.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SECRET: &str = "topsecret";

/// A repository whose `origin` cannot be reached (a closed port, with a
/// password in the URL that must never be shown).
fn unreachable_origin() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("status error base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("status error unpushed")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://me:{SECRET}@127.0.0.1:1/r.git"),
    ]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path, delay_ms: Option<u32>) -> Terminal {
    let mut launch = common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(180, 48)
        .no_exit_pause()
        .env("GITPOULPI_CLIPBOARD", "memory");
    if let Some(ms) = delay_ms {
        launch = launch.env("GITPOULPI_TEST_ACTION_DELAY_MS", ms.to_string());
    }
    launch.ready("status error unpushed").spawn()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn fetch_all(term: &mut Terminal) {
    palette(term, "Fetch All Remotes", "Git: Fetch All Remotes");
}

/// The row that holds the fetch's error line, if the screen shows one.
fn error_row(screen: &Screen) -> Option<(usize, String)> {
    error_row_of(screen, "error: Fetch failed")
}

fn error_row_of(screen: &Screen, needle: &str) -> Option<(usize, String)> {
    screen
        .text()
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(needle))
        .map(|(i, l)| (i, l.to_string()))
}

/// No framed notification: neither a card's title bar nor its buttons.
fn no_card(screen: &Screen) {
    for needle in ["Fetch failed \u{2500}", " Details ", "Hide details"] {
        assert!(
            !screen.contains(needle),
            "no card, found {needle:?}:\n{}",
            screen.text()
        );
    }
}

fn wait_for_error(term: &Terminal) -> Screen {
    wait_for_error_with(term, &[])
}

/// The error line together with every text in `with` (one whole state: a
/// torn frame can show the line before the rest).
fn wait_for_error_with(term: &Terminal, with: &[&str]) -> Screen {
    term.wait_for("the error in the status line", STEP, |s| {
        error_row(s).is_some() && with.iter().all(|w| s.contains(w))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// Clicks the error line.
fn click_error(term: &mut Terminal, screen: &Screen) {
    let (row, line) = error_row(screen).expect("the error line");
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
}

/// The entry, open in the Activity Log with its facts and buttons.
fn wait_for_entry(term: &Terminal) -> Screen {
    term.wait_for("the entry open in the Activity Log", STEP, |s| {
        s.contains("Activity Log")
            && s.contains("Fetch failed")
            && s.contains("command: git fetch")
            && s.contains("exit code:")
            && s.contains(" Copy ")
            && s.contains(" Retry ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn a_failed_fetch_has_no_card_and_a_click_on_the_status_line_opens_its_entry() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-status-error-fetch", config.path(), None);
    fetch_all(&mut term);
    let screen = wait_for_error_with(&term, &["The remote could not be reached."]);
    no_card(&screen);
    assert!(
        screen.contains("The remote could not be reached."),
        "{}",
        screen.text()
    );
    assert!(!screen.contains(SECRET), "{}", screen.text());
    term.wait_for("the unread error is counted", STEP, |s| {
        s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    click_error(&mut term, &screen);
    let entry = wait_for_entry(&term);
    assert!(!entry.contains(SECRET), "{}", entry.text());
    // Copy: the status line says what went to the clipboard.
    // The entry's own button, on the row of its Retry (the window's
    // "Copy diagnostics" button also starts with " Copy ").
    let (_, retry_row) = entry.find(" Retry ").unwrap();
    let (c, r) = entry
        .find_all(" Copy ")
        .into_iter()
        .find(|(_, row)| *row == retry_row)
        .expect("the entry's Copy");
    term.click(c + 1, r).expect("click Copy");
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_palette_twins_open_copy_and_retry_the_last_error() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-status-error-twins", config.path(), None);
    fetch_all(&mut term);
    wait_for_error(&term);

    // Copy.
    palette(
        &mut term,
        "Copy the Last Error",
        "Activity Log: Copy the Last Error",
    );
    term.wait_for("copy by palette", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Open.
    palette(
        &mut term,
        "Open the Last Error",
        "Activity Log: Open the Last Error",
    );
    wait_for_entry(&term);
    term.send_keys("Esc").unwrap();
    term.wait_for("the window closed and the error read", STEP, |s| {
        !s.contains("Application") && s.contains("Activity") && !s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Run the suggested action: the fetch runs again and fails again, which
    // the unread count shows (it was read a moment ago).
    palette(
        &mut term,
        "Run the Suggested Action",
        "Activity Log: Run the Suggested Action",
    );
    term.wait_for("the retry failing again", STEP, |s| {
        s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_retry_button_of_the_entry_runs_the_operation_again_by_click_and_by_key() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-status-error-retry", config.path(), None);
    fetch_all(&mut term);
    let screen = wait_for_error(&term);

    // The click on Retry.
    click_error(&mut term, &screen);
    let entry = wait_for_entry(&term);
    let (c, r) = entry.find(" Retry ").unwrap();
    term.click(c + 1, r).expect("click Retry");
    term.wait_for("the window closed, the retry failed again", STEP, |s| {
        !s.contains("Application") && s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The key `r` on the selected entry does the same.
    palette(
        &mut term,
        "Open the Last Error",
        "Activity Log: Open the Last Error",
    );
    wait_for_entry(&term);
    term.send_keys("r").unwrap();
    term.wait_for("the window closed, the retry failed again", STEP, |s| {
        !s.contains("Application") && s.contains("Activity (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The error is in the status line whatever the window shows when the
/// operation ends: the fetch is started, then the view is changed, and the
/// failure lands (the test delay holds it back).
fn error_over(name: &str, session: &str, show: impl Fn(&mut Terminal), shown: fn(&Screen) -> bool) {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), session, config.path(), Some(4000));
    fetch_all(&mut term);
    // The fetch is in flight (held back); leave the repository.
    show(&mut term);
    term.wait_for(name, STEP, shown)
        .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
    let screen = term
        .wait_for("the error in the status line", STEP, |s| {
            error_row(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
    no_card(&screen);
    assert!(shown(&screen), "still on {name}:\n{}", screen.text());
    // And it is a working click.
    click_error(&mut term, &screen);
    wait_for_entry(&term);
    term.stop().unwrap();
}

/// A clone that hangs until `release` is sent, then fails: its source is a
/// local listener that accepts the connection and says nothing.
struct HeldSource {
    url: String,
    release: std::sync::mpsc::Sender<()>,
}

fn held_source() -> HeldSource {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://127.0.0.1:{}/held/repo.git",
        listener.local_addr().unwrap().port()
    );
    let (release, hold) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let _ = hold.recv_timeout(Duration::from_secs(60));
            drop(stream);
        }
    });
    HeldSource { url, release }
}

/// Starts a clone from Home that stays in flight, shows `show`, then lets
/// the clone fail: the error must be in the status line of that view.
fn clone_error_over(
    name: &str,
    session: &str,
    show: impl Fn(&mut Terminal),
    shown: fn(&Screen) -> bool,
) {
    let repo = unreachable_origin();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.path().display()),
    )
    .unwrap();
    let mut term = launch(repo.root(), session, config.path(), None);
    let source = held_source();
    term.send_keys("C-o").unwrap();
    term.wait_for("Home", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("c").unwrap();
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text(&source.url).unwrap();
    term.wait_for("the full path, and the button live", STEP, |s| {
        s.contains("Full path:") && !s.contains("enter the URL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    show(&mut term);
    term.wait_for(name, STEP, shown)
        .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
    // The clone is in flight: nothing failed yet.
    assert!(
        error_row_of(&term.screen(), "error: Clone failed").is_none(),
        "{}",
        term.text()
    );
    source.release.send(()).unwrap();
    let screen = term
        .wait_for("the clone error in the status line", STEP, |s| {
            error_row_of(s, "error: Clone failed").is_some()
        })
        .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
    no_card(&screen);
    assert!(shown(&screen), "still on {name}:\n{}", screen.text());
    // And it is a working click.
    let (row, line) = error_row_of(&screen, "error: Clone failed").unwrap();
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
    term.wait_for("the entry in the Activity Log", STEP, |s| {
        s.contains("Activity Log") && s.contains("Clone failed") && s.contains(" Copy ")
    })
    .unwrap_or_else(|e| panic!("{name}: {e}\n{}", term.text()));
    term.stop().unwrap();
}

fn on_home(s: &Screen) -> bool {
    s.contains("Repository Management") && s.contains("Search repositories")
}

#[test]
fn the_error_shows_over_home() {
    clone_error_over("Home", "pty-status-error-home", |_| {}, on_home);
}

#[test]
fn the_error_shows_over_the_radar() {
    clone_error_over(
        "the Radar",
        "pty-status-error-radar",
        |t| t.send_keys("M-r").unwrap(),
        |s| s.contains("Scope") && s.contains("Open tabs") && s.contains("Refresh"),
    );
}

#[test]
fn the_error_shows_over_the_workspaces() {
    clone_error_over(
        "the Workspaces",
        "pty-status-error-workspaces",
        |t| palette(t, "Workspace: Manage", "Workspace: Manage..."),
        |s| s.contains("New workspace") && s.contains("From open tabs"),
    );
}

#[test]
fn an_unread_error_follows_the_user_to_home_and_back_to_the_tab() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-status-error-follow", config.path(), None);
    fetch_all(&mut term);
    wait_for_error(&term);

    // A tab switch builds a new window: the error is still on the line,
    // on Home (where the footer is not) and back on the tab.
    term.send_keys("C-o").unwrap();
    term.wait_for("the error over Home", STEP, |s| {
        on_home(s) && error_row(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the error on the tab again", STEP, |s| {
        !on_home(s) && error_row(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_error_shows_over_settings() {
    error_over(
        "Settings",
        "pty-status-error-settings",
        |t| palette(t, "Settings", "App: Settings..."),
        |s| s.contains("Repo-specific settings"),
    );
}

#[test]
fn the_error_shows_while_the_activity_log_is_open() {
    let repo = unreachable_origin();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(
        repo.root(),
        "pty-status-error-open-log",
        config.path(),
        Some(4000),
    );
    fetch_all(&mut term);
    term.send_keys("C-n").unwrap();
    term.send_text("A").unwrap();
    term.wait_for("the Activity Log", STEP, |s| s.contains("Application"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = wait_for_error_with(&term, &["Application"]);
    no_card(&screen);
    assert!(screen.contains("Application"), "{}", screen.text());
}
