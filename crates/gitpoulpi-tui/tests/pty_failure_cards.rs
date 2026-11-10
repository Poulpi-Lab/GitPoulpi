//! A failure is not only for git actions: a failed clone and a failed
//! external program say it the same way (a red line in the status line, no
//! framed card, the entry in the Activity Log with Copy and its facts) with
//! the same keyboard twins. Real binary, daemon and git through a pty
//! (`xtask::pty`); the clone sources are a path that does not exist and a
//! closed local port, the "tool" is a program that is not installed. No
//! network, no GUI program. (The hosting side is
//! `pty_hosting/failure_card.rs`, the fetch side `pty_status_error.rs`.)

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);
const SECRET: &str = "topsecret";

fn start_repo() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "failure-card-start-commit")
        .unwrap()
        .build()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// Opens the Clone dialog (`Ctrl+O`, `c`), types `url` and presses Enter.
fn clone_from(term: &mut Terminal, url: &str) {
    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(url).expect("type the source");
    term.wait_for("the full path, and the button live", STEP, |s| {
        s.contains("Full path:") && !s.contains("enter the URL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").expect("clone it");
}

fn launch_clone(config: &Path, root: &Path, repo: &Path, session: &str) -> Terminal {
    std::fs::write(
        config.join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.display()),
    )
    .unwrap();
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .config_dir(config)
        .size(180, 48)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .ready("failure-card-start-commit")
        .spawn()
}

/// No framed notification: neither the card's title bar nor its buttons.
fn no_card(screen: &Screen, title: &str) {
    for needle in [format!("{title} \u{2500}"), " Details ".to_string()] {
        assert!(
            !screen.contains(&needle),
            "no card, found {needle:?}:\n{}",
            screen.text()
        );
    }
}

/// The error line of the status line, with the reason it words.
fn wait_for_error(term: &Terminal, title: &str, reason: &str) -> Screen {
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains(&format!("error: {title}: {reason}"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn click_error(term: &mut Terminal, screen: &Screen, title: &str) {
    let (row, line) = screen
        .text()
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(&format!("error: {title}")))
        .map(|(i, l)| (i, l.to_string()))
        .expect("the error line");
    let col = line[..line.find("error:").unwrap()].chars().count();
    term.click(col as u16 + 2, row as u16).expect("click it");
}

#[test]
fn a_clone_from_a_missing_path_is_worded_in_the_status_line_and_its_entry_holds_the_message() {
    let start = start_repo();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_clone(config.path(), root.path(), start.root(), "pty-card-clone");
    let missing = root.path().join("no-such-source").join("repo-1");
    clone_from(&mut term, &missing.display().to_string().replace('\\', "/"));

    // The clone is a daemon job: the dialog closed on Enter, so the line is
    // alone on screen (the Home tab's own status bar).
    let screen = wait_for_error(&term, "Clone failed", "The repository could not be cloned.");
    no_card(&screen, "Clone failed");

    // The keyboard twin opens the entry: the step, the message git left.
    palette(
        &mut term,
        "Open the Last Error",
        "Activity Log: Open the Last Error",
    );
    term.wait_for("the entry", STEP, |s| {
        s.contains("Activity Log")
            && s.contains("Check the address and your access")
            && s.contains("clone into")
            && s.contains(" Copy ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_clone_from_an_unreachable_host_is_worded_and_never_shows_the_password() {
    let start = start_repo();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_clone(
        config.path(),
        root.path(),
        start.root(),
        "pty-card-clone-net",
    );
    clone_from(
        &mut term,
        &format!("http://me:{SECRET}@127.0.0.1:1/some/repo-2.git"),
    );
    let screen = wait_for_error(&term, "Clone failed", "The remote could not be reached.");
    assert!(!screen.contains(SECRET), "{}", screen.text());
    // The mouse path: a click on the line opens the entry with the message.
    click_error(&mut term, &screen, "Clone failed");
    let shown = term
        .wait_for("the entry", STEP, |s| {
            s.contains("Activity Log") && s.contains("clone into")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!shown.contains(SECRET), "{}", shown.text());
    term.stop().unwrap();
}

#[test]
fn a_tool_that_is_not_installed_is_in_the_status_line_and_the_log_names_the_program() {
    let repo = start_repo();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\nopen_in_default = \"Ghost\"\n\
         [[ui.open_in]]\nlabel = \"Ghost\"\ncommand = \"no-such-program-xyz ${cwd}\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .no_commit_column()
        .session("pty-card-tool")
        .config_dir(config.path())
        .size(180, 48)
        .env("GITPOULPI_CLIPBOARD", "memory")
        .no_exit_pause()
        .ready("failure-card-start-commit")
        .spawn();
    palette(&mut term, "Open in Default", "Tab: Open in Default");

    let screen = term
        .wait_for("the tool failure in the status line", STARTUP, |s| {
            s.contains("error: External tool failed: The program could not be run.")
                && s.contains("Activity (1)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    no_card(&screen, "External tool failed");
    assert!(
        screen.contains("Activity (1)"),
        "the status bar counts the unread error:\n{}",
        screen.text()
    );
    // The entry: what the system said, naming the program, and the step.
    click_error(&mut term, &screen, "External tool failed");
    term.wait_for("the entry", STEP, |s| {
        s.contains("Activity Log")
            && s.contains("no-such-program-xyz")
            && s.contains("Check its command in Settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Esc closes the window; Copy by its keyboard twin: the status line says
    // what went to the clipboard.
    term.send_keys("Esc").unwrap();
    term.wait_for("the window closed", STEP, |s| !s.contains("Application"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(
        &mut term,
        "Copy the Last Error",
        "Activity Log: Copy the Last Error",
    );
    term.wait_for("copy by palette", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
