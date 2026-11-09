//! Saving a token command on the SSH & credentials page asks first, through
//! a pty: the command is shown verbatim with its SHA-256, no answer is
//! chosen until an arrow key or a digit, Enter alone answers nothing, Esc
//! and Cancel save nothing, and `1` saves - or shows the daemon's own
//! refusal when this build's daemon does not run commands yet. The private
//! profile file is read back, not trusted from the screen alone.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const COMMAND: &str = "pass show work-token";

fn launch(repo: &Path, home: &Path) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session("pty-repo-auth-confirm")
        .size(200, 60)
        .env("HOME", home)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .no_exit_pause()
        .ready_when("the fixture", |s| s.contains("initial commit"))
        .mouse_capture()
        .start()
}

fn wait(term: &Terminal, what: &str, ok: impl FnMut(&xtask::pty::Screen) -> bool) {
    term.wait_for(what, STEP, ok)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn profile_file(config: &Path) -> String {
    let paths = gitpoulpi_config::ConfigPaths::for_dir(config);
    std::fs::read_to_string(paths.private_dir().join("repo-auth.toml")).unwrap_or_default()
}

fn click_button(term: &mut Terminal, label: &str) {
    let needle = format!(" {label} ");
    let (col, row) = common::scroll_settings_to(term, &needle, STEP);
    term.click(col + 1, row).unwrap();
}

/// Types `text` into the field showing `label` (Enter on its row opens the
/// editor; a second Enter commits).
fn type_field(term: &mut Terminal, label: &str, text: &str) {
    let (col, row) = common::scroll_settings_to(term, label, STEP);
    term.click(col + 1, row).unwrap();
    wait(term, "the field in edit", |s| s.contains("\u{2588}"));
    term.send_text(text).unwrap();
    term.send_keys("Enter").unwrap();
}

#[test]
fn a_command_is_confirmed_with_no_default_answer_and_escape_saves_nothing() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial commit")
        .unwrap()
        .build();
    let home = tempfile::tempdir().unwrap();
    let (mut term, config) = launch(repo.root(), home.path());
    common::retry_until(
        &mut term,
        "the SSH & credentials page",
        4,
        Duration::from_secs(10),
        |t| {
            t.send_keys("C-p").unwrap();
            t.send_text("ssh & credentials").unwrap();
            t.send_keys("Enter").unwrap();
        },
        |s| s.contains("SSH & credentials - repo-specific"),
    );

    // The page has read the repository's sign-in settings.
    wait(&term, "the page loaded", |s| {
        s.contains("This repository has no remote.") && !s.contains("Reading the sign-in")
    });

    // HTTPS: System -> Token command.
    let (col, row) = common::scroll_settings_to_choice(&mut term, "System (git's helpers)", STEP);
    term.click(col + 1, row).unwrap();
    wait(&term, "the HTTPS choices", |s| s.contains("Token command"));
    common::click_text_until(&mut term, "Token command", "the command row", |s| {
        s.contains("Runs as")
    });
    type_field(&mut term, "Command", COMMAND);
    wait(&term, "the argv preview", |s| s.contains("[pass] [show] [work-token]"));
    type_field(&mut term, "Hosts", "github.com");

    // Save opens the question, with nothing chosen.
    click_button(&mut term, "Save");
    let screen = term
        .wait_for("the confirmation", STEP, |s| s.contains("Allow this command?"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains(&format!("$ {COMMAND}")), "{}", term.text());
    let sha = gitpoulpi_core::domain::command_sha256(&["pass", "show", "work-token"]);
    assert!(screen.contains(&sha), "the hash is shown: {}", term.text());
    assert!(
        screen.contains("1. Save and allow") && screen.contains("2. Cancel"),
        "{}",
        term.text()
    );
    assert!(!screen.contains("> 1.") && !screen.contains("> 2."), "no default answer");

    // Enter answers nothing.
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        term.screen().contains("Allow this command?"),
        "Enter with nothing chosen must not answer: {}",
        term.text()
    );
    // An arrow only starts the highlight.
    term.send_keys("Down").unwrap();
    wait(&term, "the first answer highlighted", |s| s.contains("> 1. Save and allow"));

    // Esc cancels: nothing saved, the edit stays.
    term.send_keys("Escape").unwrap();
    wait(&term, "the question closed", |s| !s.contains("Allow this command?"));
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !profile_file(config.path()).contains("pass"),
        "Esc saved nothing: {}",
        profile_file(config.path())
    );
    assert!(term.screen().contains("Unsaved changes."), "{}", term.text());

    // Again, then `1`: the daemon saves it, or refuses it with its own words.
    click_button(&mut term, "Save");
    wait(&term, "the confirmation again", |s| s.contains("Allow this command?"));
    term.send_keys("1").unwrap();
    let screen = term
        .wait_for("the daemon's answer", STEP, |s| {
            s.contains("Saved. This repository now signs in as chosen.") || s.contains("Not saved: ")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    if screen.contains("Not saved: ") {
        // This build's daemon does not run commands yet: its refusal is shown cleanly.
        assert!(screen.contains("not available"), "{}", term.text());
        assert!(!profile_file(config.path()).contains("pass"));
    } else {
        let deadline = Instant::now() + STEP;
        while !profile_file(config.path()).contains("work-token") {
            assert!(
                Instant::now() < deadline,
                "never stored: {}",
                profile_file(config.path())
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(profile_file(config.path()).contains(&sha));
    }
}
