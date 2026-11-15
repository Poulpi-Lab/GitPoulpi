//! The hosting rows of the SSH & credentials page, through a pty: the token
//! command's "Use for the hosting API" toggle is saved with the profile (the
//! private file is read back), and the Assignment choice is saved at once to
//! the repository's state file; with assignment blocked the palette's assign
//! entry says why it is grayed.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

fn launch(repo: &Path, home: &Path) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session("pty-repo-auth-hosting")
        .size(200, 60)
        .scratch_home(home)
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

fn wait_for_file(what: &str, mut read: impl FnMut() -> String, want: &str) {
    let deadline = Instant::now() + STEP;
    while !read().contains(want) {
        assert!(
            Instant::now() < deadline,
            "{what}: never saw `{want}` in:\n{}",
            read()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn click_button(term: &mut Terminal, label: &str) {
    let needle = format!(" {label} ");
    let (col, row) = common::scroll_settings_to(term, &needle, STEP);
    term.click(col + 1, row).unwrap();
}

fn type_field(term: &mut Terminal, label: &str, text: &str) {
    let (col, row) = common::scroll_settings_to(term, label, STEP);
    term.click(col + 1, row).unwrap();
    wait(term, "the field in edit", |s| s.contains("\u{2588}"));
    term.send_text(text).unwrap();
    term.send_keys("Enter").unwrap();
}

fn open_page(term: &mut Terminal) {
    common::retry_until(
        term,
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
    wait(term, "the page loaded", |s| {
        s.contains("This repository has no remote.") && !s.contains("Reading the sign-in")
    });
}

#[test]
fn the_hosting_toggle_is_saved_with_the_profile_and_assignment_blocks_at_once() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial commit")
        .unwrap()
        .build();
    let home = tempfile::tempdir().unwrap();
    let (mut term, config) = launch(repo.root(), home.path());
    open_page(&mut term);

    // A token command, marked for the hosting API.
    let (col, row) = common::scroll_settings_to_choice(&mut term, "System (git's helpers)", STEP);
    term.click(col + 1, row).unwrap();
    wait(&term, "the HTTPS choices", |s| s.contains("Token command"));
    common::click_text_until(&mut term, "Token command", "the command row", |s| {
        s.contains("Runs as")
    });
    type_field(&mut term, "Command", "pass show app-token");
    type_field(&mut term, "Hosts", "github.com");
    let (col, row) = common::scroll_settings_to(&mut term, "Use for the hosting API", STEP);
    term.click(col + 1, row).unwrap();
    click_button(&mut term, "Save");
    wait(&term, "the confirmation", |s| {
        s.contains("Allow this command?")
    });
    term.send_keys("1").unwrap();
    wait_for_file(
        "the profile",
        || profile_file(config.path()),
        "use_for_hosting = true",
    );

    // Assignment: Allowed -> Blocked, saved without a Save.
    let (col, row) = common::scroll_settings_to_choice(&mut term, "Allowed", STEP);
    term.click(col + 1, row).unwrap();
    let state_file = repo.state_dir().join("repo-state.toml");
    let read_state = || std::fs::read_to_string(&state_file).unwrap_or_default();
    common::click_text_until(&mut term, "Blocked", "the Blocked option", |_| {
        read_state().contains("assignment_blocked = true")
    });
    wait_for_file(
        "the repository state",
        read_state,
        "assignment_blocked = true",
    );

    // Back on the repository, the palette says why the assign entry is grayed.
    term.send_keys("Escape").unwrap();
    wait(&term, "the repository again", |s| {
        s.contains("initial commit") && !s.contains("SSH & credentials - repo-specific")
    });
    term.send_keys("C-p").unwrap();
    wait(&term, "the palette", |s| s.contains("Command Palette"));
    term.send_text("assignees").unwrap();
    wait(&term, "the Hosting entry", |s| {
        s.contains("Hosting: Edit Assignees...")
    });
    // The entry is grayed: running it says why instead.
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    wait(&term, "the reason", |s| {
        s.contains("assignment is blocked in this repository")
    });
}
