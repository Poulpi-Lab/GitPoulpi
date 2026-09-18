//! End-to-end checks of the remote UI -- a remote's own menu, the Edit
//! remote popup, the Pull button's dropdown and the diverged-push dialog
//! -- driven through a pty by `xtask::pty`: the real binary, the real
//! daemon, real `git`, and bare repositories inside the fixture's temp
//! dir as the remotes (no network).
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;

/// A name no other part of the screen can contain: the Log's pills show
/// a remote branch without its remote, so this only ever appears in the
/// REMOTE section (and in menus about it).
const REMOTE: &str = "zeta-fork";

fn launch(repo: &Path, session: &str, wait_for: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    let needle = wait_for.to_string();
    term.wait_for("the fixture loaded", STARTUP, move |s| s.contains(&needle))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// A repository with two remotes, both holding `main` and both fetched,
/// so the REMOTE tree shows a row for each.
fn two_remotes() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("remotes fixture")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote(REMOTE)
        .unwrap()
        .push("main")
        .unwrap()
        .push_to(REMOTE, "main")
        .unwrap()
        .build();
    repo.git(&["fetch", "--all"]);
    repo
}

/// Right-click the remote's own row -> its menu; "Set as default
/// remote" is remembered in `repo-state.toml`; "Edit..." opens the popup,
/// and a push URL typed there reaches `.git/config`.
#[test]
fn a_remotes_menu_sets_the_default_and_edits_its_push_url() {
    let repo = two_remotes();
    let (mut term, _config) = launch(repo.root(), "pty-remote-menu", REMOTE);

    let (col, row) = term.wait_for_text(REMOTE, STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the remote");
    let menu = term
        .wait_for("the remote's menu", STEP, |s| {
            s.contains("Set as default remote")
                && s.contains("Copy link to remote")
                && s.contains(&format!("Edit {REMOTE}..."))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Set as default remote").unwrap();
    term.click(c + 1, r).expect("click Set as default");
    term.wait_for("the default-remote status line", STEP, |s| {
        s.contains(&format!("{REMOTE} is now this repository's default remote"))
            && !s.contains("Copy link to remote")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let state = std::fs::read_to_string(repo.state_dir().join("repo-state.toml"))
        .expect("repo-state.toml written");
    assert!(
        state.contains(&format!("default_remote = \"{REMOTE}\"")),
        "{state}"
    );

    // Edit: the popup opens on the fetch URL; Tab to the push URL, type
    // one, Enter submits.
    let (col, row) = term.wait_for_text(REMOTE, STEP).unwrap();
    term.right_click(col + 1, row).expect("right-click again");
    let menu = term
        .wait_for("the remote's menu, without Set as default now", STEP, |s| {
            s.contains(&format!("Edit {REMOTE}...")) && !s.contains("Set as default remote")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(&format!("Edit {REMOTE}...")).unwrap();
    term.click(c + 1, r).expect("click Edit");
    term.wait_for("the Edit remote popup", STEP, |s| {
        s.contains(&format!("Edit remote {REMOTE}")) && s.contains("Push URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    term.send_text("https://example.invalid/me/push.git")
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the push URL saved", STEP, |s| {
        s.contains(&format!(
            "{REMOTE} now pushes to https://example.invalid/me/push.git"
        )) && !s.contains("Edit Remote")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["config", "--get", &format!("remote.{REMOTE}.pushurl")]),
        "https://example.invalid/me/push.git"
    );
}

/// The Pull button's `▾`: Fetch All and the three pull variants, each
/// running when chosen. The button stays "Pull" and `git.pull_mode` is
/// left alone.
#[test]
fn the_pull_dropdown_runs_its_item_and_leaves_the_button_alone() {
    let repo = two_remotes();
    let (mut term, config) = launch(repo.root(), "pty-pull-dropdown", REMOTE);

    let screen = term
        .wait_for("the toolbar's Pull split button", STEP, |s| {
            s.contains("Pull \u{25BE}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Pull \u{25BE}").unwrap();
    // "Pull " is five columns; the arrow chip is the sixth.
    term.click(c + 5, r).expect("click the Pull dropdown arrow");
    let menu = term
        .wait_for("the pull dropdown", STEP, |s| {
            s.contains("Fetch All")
                && s.contains("Pull (fast-forward if possible)")
                && s.contains("Pull (fast-forward only)")
                && s.contains("Pull (rebase)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Pull (rebase)").unwrap();
    term.click(c + 1, r).expect("choose rebase");
    // It ran (the status line says what it is doing or did), the menu
    // is gone, and the button still reads Pull.
    term.wait_for("the pull ran and the menu closed", STEP, |s| {
        !s.contains("Pull (fast-forward only)") && s.contains("Pull \u{25BE}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let toml = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(!toml.contains("pull_mode"), "{toml}");
}

/// A push the remote rejects because it moved on opens a
/// dialog, and "Pull (merge), then push" lands both histories on the
/// remote - the typed rejection travelling backend -> daemon -> TUI.
#[test]
fn a_diverged_push_offers_pull_then_push_and_it_lands() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("diverged base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("mine locally")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "theirs remotely");
    let (mut term, _config) = launch(repo.root(), "pty-diverged", "mine locally");

    let screen = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(screen.0 + 2, screen.1).expect("click Push");
    let dialog = term
        .wait_for("the diverged-push dialog", STEP, |s| {
            s.contains("origin/main has diverged")
                && s.contains("Pull (merge), then push")
                && s.contains("Force push (with lease)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = dialog.find("Pull (merge), then push").unwrap();
    term.click(c + 1, r).expect("choose pull then push");
    term.wait_for("the push after the pull", STEP, |s| {
        s.contains("pushed main to origin") && !s.contains("has diverged")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let remote_main = repo.git_in_remote(&["rev-parse", "main"]);
    assert_eq!(remote_main, repo.git(&["rev-parse", "main"]));
    let log = repo.git_in_remote(&["log", "--format=%s", "main"]);
    assert!(
        log.contains("mine locally") && log.contains("theirs remotely"),
        "{log}"
    );
}

/// The dialog's other answer: Force push (with lease), confirmed, lands
/// the local history on the remote. A rejected push leaves the local
/// remote-tracking ref behind the remote, so a lease against it alone
/// would always be refused - the dialog fetches first.
#[test]
fn a_diverged_push_can_be_force_pushed_with_lease_after_confirming() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("force base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .commit("mine to force")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "theirs to lose");
    let (mut term, _config) = launch(repo.root(), "pty-force-lease", "mine to force");

    let push = term.wait_for_text("\u{21E7} Push", STEP).unwrap();
    term.click(push.0 + 2, push.1).expect("click Push");
    let dialog = term
        .wait_for("the diverged-push dialog", STEP, |s| {
            s.contains("origin/main has diverged") && s.contains("Force push (with lease)")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = dialog.find("Force push (with lease)").unwrap();
    term.click(c + 1, r).expect("choose force push");
    term.wait_for("the force-push confirmation", STEP, |s| {
        s.contains("Force push main to origin?") && !s.contains("has diverged")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    term.wait_for("the force push done", STEP, |s| {
        s.contains("force-pushed (with lease) main to origin")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(
        repo.git_in_remote(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "main"])
    );
    let log = repo.git_in_remote(&["log", "--format=%s", "main"]);
    assert!(!log.contains("theirs to lose"), "{log}");
}
