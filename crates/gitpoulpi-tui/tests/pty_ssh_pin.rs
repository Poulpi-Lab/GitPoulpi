//! End-to-end check of the per-repository SSH key pin (ADR-0008): the
//! repository tab menu's "SSH key..." row and its palette twin show the
//! repository-local `core.sshCommand`, pick an integration profile's key
//! for it, and clear it -- through a pty by `xtask::pty`: the real
//! binary, the real daemon, real `git`.
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 45;

const KEY: &str = "/keys/work_ed25519";
const PIN: &str = "ssh -i \"/keys/work_ed25519\" -o IdentitiesOnly=yes";

/// Launches the TUI with a config holding one profile whose SSH key is
/// `KEY`, and waits for the fixture's commit.
fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(format!(
            "[[integrations]]\nid = \"work\"\nkind = \"github\"\nurl = \"https://github.com\"\n\
             token_ref = \"integration:work\"\nname = \"WorkAccount\"\n\n\
             [integrations.ssh]\nmode = \"custom\"\nprivate_key_path = \"{KEY}\"\n"
        ))
        .size(COLS, ROWS)
        .ready_when("the fixture loaded", |s| s.contains("ssh pin fixture"))
        .start()
}

fn pinned(repo: &gitpoulpi_testkit::FixtureRepo) -> String {
    repo.git(&[
        "config",
        "--local",
        "--default",
        "",
        "--get",
        "core.sshCommand",
    ])
}

/// The palette entry opens the pin menu, picking the profile's key writes
/// `core.sshCommand`, and reopening it shows the pin and clears it.
#[test]
fn the_palette_pins_a_profiles_key_and_clears_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("ssh pin fixture")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-ssh-pin-palette");

    term.send_keys("C-p").unwrap();
    term.send_text("SSH Key for This").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let menu = term
        .wait_for("the pin menu, nothing pinned", STEP, |s| {
            s.contains("SSH key for this repository")
                && s.contains("Not pinned")
                && s.contains("WorkAccount")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("WorkAccount").unwrap();
    term.click(c + 1, r).expect("click the profile's key");
    term.wait_for("the pin status line", STEP, |s| {
        s.contains("this repository now uses ssh -i") && !s.contains("Not pinned")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pinned(&repo), PIN);

    // Reopen from the keyboard: it shows the pin, and Clear removes it.
    term.send_keys("C-p").unwrap();
    term.send_text("SSH Key for This").unwrap();
    term.wait_for("the palette narrowed again", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let menu = term
        .wait_for("the pin menu, pinned", STEP, |s| {
            s.contains("Pinned: ssh -i") && s.contains("Clear the pin")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Clear the pin").unwrap();
    term.click(c + 1, r).expect("click Clear");
    term.wait_for("the cleared status line", STEP, |s| {
        s.contains("SSH key pin cleared")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pinned(&repo), "");
}

/// The keyboard reaches the same menu row: arrow to the key, Enter.
#[test]
fn the_menu_row_is_reachable_from_the_keyboard() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("ssh pin fixture")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-ssh-pin-keys");

    // Ctrl+T is the tab menu; its last row is "SSH key...".
    term.send_keys("C-t").unwrap();
    let menu = term
        .wait_for("the tab menu with the SSH row", STEP, |s| {
            s.contains("SSH key...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("SSH key...").unwrap();
    term.click(c + 1, r).expect("activate the row");
    term.wait_for("the pin menu", STEP, |s| {
        s.contains("SSH key for this repository") && s.contains("Not pinned")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the pin status line", STEP, |s| {
        s.contains("this repository now uses ssh -i")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pinned(&repo), PIN);
}
