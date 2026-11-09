//! The Activity Log's entry of a failure caused by the repository's sign-in
//! carries a button that opens the SSH & credentials page, with a keyboard
//! twin: a fetch whose repository-local credential helper does not exist
//! reads as "The configured credential helper failed", hints at the page,
//! and the "SSH & credentials..." button (or `r` on the entry) lands on it.
//!
//! Real binary, daemon and system git through a pty (`xtask::pty`); the remote
//! is a loopback server that asks for credentials, so there is no network. The
//! home folder and the global git config are temporary: never the developer's.
#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(40);
const PAGE: &str = "SSH & credentials - repo-specific";

struct Rig {
    repo: FixtureRepo,
    _server: AuthHttpRemote,
    home: tempfile::TempDir,
}

fn rig() -> Rig {
    let server = AuthHttpRemote::start();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("remedy base")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", &server.url()]);
    // The repository's own helper names a script that is not there.
    repo.git(&["config", "credential.helper", "!/no/such/credential-script"]);
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("gitconfig"),
        "[user]\n\tname = T\n\temail = t@example.com\n",
    )
    .unwrap();
    Rig {
        repo,
        _server: server,
        home,
    }
}

fn launch(rig: &Rig, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(rig.repo.root())
        .session(session)
        .size(180, 48)
        .env("HOME", rig.home.path())
        .env("GIT_CONFIG_GLOBAL", rig.home.path().join("gitconfig"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_CLIPBOARD", "memory")
        .no_exit_pause()
        .mouse_capture()
        .ready_when("the fixture loaded", |s| s.contains("remedy base"))
        .start()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// A failed fetch, opened in the Activity Log on its entry.
fn failing_entry(term: &mut Terminal) -> Screen {
    palette(term, "Fetch All Remotes", "Git: Fetch All Remotes");
    common::open_status_error(term, "Fetch failed");
    term.wait_for("the entry with its remedy", STEP, |s| {
        s.contains("Activity Log")
            && s.contains("The configured credential helper failed")
            && s.contains(" SSH & credentials... ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn a_failing_credential_helper_offers_the_page_and_the_button_opens_it() {
    let rig = rig();
    let (mut term, _config) = launch(&rig, "pty-repo-auth-remedy-click");
    let entry = failing_entry(&mut term);
    assert!(
        entry.contains("Pick another source for this repository in SSH & credentials"),
        "the reading points at the page:\n{}",
        entry.text()
    );
    let (col, row) = entry.find(" SSH & credentials... ").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("Settings on the SSH & credentials page", STEP, |s| {
        s.contains(PAGE) && !s.contains("Activity Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_keyboard_twin_of_the_button_opens_the_page_too() {
    let rig = rig();
    let (mut term, _config) = launch(&rig, "pty-repo-auth-remedy-key");
    failing_entry(&mut term);
    term.send_keys("r").unwrap();
    term.wait_for("Settings on the SSH & credentials page", STEP, |s| {
        s.contains(PAGE) && !s.contains("Activity Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
