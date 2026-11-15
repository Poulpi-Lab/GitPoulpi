//! Migration of the per-repository SSH key pin (ADR-0008) to the SSH &
//! credentials page, end to end through a pty by `xtask::pty`: the real
//! binary, the real daemon, real `git`.
//!
//! The tab menu's row and the palette entry that used to open the pin menu
//! open the page now. A repository that still has a `core.sshCommand` of its
//! own sees it there read-only, and "Move into GitPoulpi" saves the
//! equivalent key-file profile and unsets the local `core.sshCommand`.
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 52;

const KEY: &str = "/keys/work_ed25519";
const PIN: &str = "ssh -i \"/keys/work_ed25519\" -o IdentitiesOnly=yes";
const PAGE: &str = "SSH & credentials - repo-specific";

/// A repository that carries the old pin, launched with a fresh config.
fn pinned_repo(session: &str) -> (gitpoulpi_testkit::FixtureRepo, Terminal, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("ssh pin fixture")
        .unwrap()
        .build();
    repo.git(&["config", "--local", "core.sshCommand", PIN]);
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(COLS, ROWS)
        .no_exit_pause()
        .mouse_capture()
        .ready_when("the fixture loaded", |s| s.contains("ssh pin fixture"))
        .start();
    (repo, term, config)
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

fn profile_file(config: &Path) -> String {
    let paths = gitpoulpi_config::ConfigPaths::for_dir(config);
    std::fs::read_to_string(paths.private_dir().join("repo-auth.toml")).unwrap_or_default()
}

/// The palette's entry for the page.
fn open_from_the_palette(term: &mut Terminal) {
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
        |s| s.contains(PAGE),
    );
}

/// The palette no longer offers the pin menu.
#[test]
fn the_palette_has_no_pin_entry_any_more() {
    let (_repo, mut term, _config) = pinned_repo("pty-ssh-pin-palette");
    term.send_keys("C-p").unwrap();
    term.send_text("SSH Key for This").unwrap();
    term.wait_for("the palette found nothing", STEP, |s| {
        s.contains("Command Palette (0/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Palette, page, "Move into GitPoulpi": the profile holds the key, and the
/// repository's own `core.sshCommand` is gone.
#[test]
fn moving_the_pin_creates_the_key_profile_and_unsets_core_ssh_command() {
    let (repo, mut term, config) = pinned_repo("pty-ssh-pin-move");
    open_from_the_palette(&mut term);
    // The SSH rows are below the fold of the page: ask for the pin by name.
    let (_, row) = common::scroll_settings_to(&mut term, "Pinned in git's config", STEP);
    assert!(
        term.screen().line(row).contains("ssh -i"),
        "the pin's command is shown read-only on its row:\n{}",
        term.text()
    );
    assert_eq!(pinned(&repo), PIN, "showing the pin changes nothing");

    common::scroll_settings_to(&mut term, " Move into GitPoulpi ", STEP);
    // The page may still be scrolling: a click at a position seen mid-scroll
    // lands on the row that moved there (the Hosting rows sit right below).
    let (col, row) = term
        .wait_for_settled_text(" Move into GitPoulpi ", Duration::from_millis(500), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(col + 1, row).unwrap();
    // The status line is at the foot of the page.
    common::scroll_settings_to(
        &mut term,
        "Moved: this repository now signs in over SSH",
        STEP,
    );

    // The pin is gone from git's config, and the profile holds the key.
    let deadline = Instant::now() + STEP;
    while !pinned(&repo).is_empty() || !profile_file(config.path()).contains(KEY) {
        assert!(
            Instant::now() < deadline,
            "core.sshCommand = {:?}, profile = {:?}",
            pinned(&repo),
            profile_file(config.path())
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    // The page reads the view again: no pin row, the key chosen.
    term.wait_for("the page without the pin", STEP, |s| {
        !s.contains("Pinned in git's config") && s.contains("Private key file")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The tab menu's row opens the page (with the pin on it).
#[test]
fn the_tab_menu_row_opens_the_page() {
    let (repo, mut term, _config) = pinned_repo("pty-ssh-pin-keys");
    // Ctrl+T is the tab menu; its last-but-one row is the page.
    term.send_keys("C-t").unwrap();
    let menu = term
        .wait_for("the tab menu with the page's row", STEP, |s| {
            s.contains("SSH & credentials...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !menu.contains("SSH key..."),
        "the pin row is replaced:\n{}",
        menu.text()
    );
    let (c, r) = menu.find("SSH & credentials...").unwrap();
    term.click(c + 1, r).expect("activate the row");
    term.wait_for_text(PAGE, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::scroll_settings_to(&mut term, "Pinned in git's config", STEP);
    assert_eq!(pinned(&repo), PIN);
}
