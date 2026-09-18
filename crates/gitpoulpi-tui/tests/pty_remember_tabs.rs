//! With `general.remember_tabs` on (the default), launching from a
//! repository's folder restores the tabs the last run left open and makes
//! that repository's tab the active one. Sessions are derived from the
//! launch folder, so a launch from a different folder has its own session
//! file and must still see the saved tabs.
//!
//! No `-s`: the session is the folder-derived default, as for a plain
//! `gitpoulpi`. Isolation is the temp config dir (sessions, recent list
//! and daemon port files all live under it).

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn exit(mut term: xtask::pty::Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..100 {
        if term.exited() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

#[test]
fn launching_from_a_repo_restores_the_remembered_tabs_and_activates_that_repo() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let bravo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("recent.toml"),
        format!(
            "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
            bravo.root().display()
        ),
    )
    .unwrap();

    // Run 1, from alpha: open bravo as a second tab, then quit.
    let mut term = common::launch(alpha.root())
        .implicit_session()
        .config_dir(config.path())
        .no_exit_pause()
        .ready("alpha-only-commit")
        .spawn();
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("bravo-repo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("bravo").unwrap();
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("bravo open as a tab", STARTUP, |s| {
        s.contains("bravo-only-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    exit(term);

    // Run 2, from *bravo's* folder: a different folder is a different
    // session file, which is where the bug lived. Both tabs come back,
    // bravo's is the active one. (Both fixtures are named `repo`, so the
    // session files are what tell the tabs apart.)
    let term = common::launch(bravo.root())
        .implicit_session()
        .config_dir(config.path())
        .no_exit_pause()
        .ready("bravo-only-commit")
        .spawn();
    exit(term);
    let (paths, active) = session_tabs(config.path(), bravo.root());
    assert_eq!(paths.len(), 2, "both tabs restored: {paths:?}");
    assert!(paths[active].ends_with(bravo.root().file_name().unwrap()));
    assert_eq!(paths[active], bravo.root().canonicalize().unwrap());

    // And back to alpha's folder: alpha active, both tabs still there.
    let term = common::launch(alpha.root())
        .implicit_session()
        .config_dir(config.path())
        .no_exit_pause()
        .ready("alpha-only-commit")
        .spawn();
    term.wait_for("alpha active", STEP, |s| {
        s.contains("alpha-only-commit") && !s.contains("bravo-only-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    exit(term);
    let (paths, active) = session_tabs(config.path(), alpha.root());
    assert_eq!(paths.len(), 2, "{paths:?}");
    assert_eq!(paths[active], alpha.root().canonicalize().unwrap());
}

/// With the setting off, a launch from a folder that is not a repository
/// starts on Home alone: the tabs the session had open do not come back.
#[test]
fn remember_off_outside_a_repository_starts_with_no_repository_tab() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let outside = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let session_file = config.path().join("sessions").join("forgetful.toml");

    // Run 1, from alpha, in a named session: alpha is saved as a tab.
    let term = common::launch(alpha.root())
        .session("forgetful")
        .config_dir(config.path())
        .no_exit_pause()
        .ready("alpha-only-commit")
        .spawn();
    exit(term);
    let saved = gitpoulpi_session::GlobalSession::load(&session_file).expect("session file");
    assert_eq!(saved.tabs.len(), 1, "the first run left a tab");

    // Run 2, same session, from a plain folder, remembering off.
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nremember_tabs = false\n",
    )
    .unwrap();
    let term = common::launch(outside.path())
        .session("forgetful")
        .config_dir(config.path())
        .no_exit_pause()
        .ready("Repository Management")
        .spawn();
    exit(term);
    let after = gitpoulpi_session::GlobalSession::load(&session_file).expect("session file");
    assert!(after.tabs.is_empty(), "no tab came back: {:?}", after.tabs);
}

/// The tab paths and the active index in `launch_dir`'s session file.
fn session_tabs(config: &Path, launch_dir: &Path) -> (Vec<PathBuf>, usize) {
    let name = gitpoulpi_config::default_session_name(launch_dir);
    let file = config.join("sessions").join(format!("{name}.toml"));
    let session = gitpoulpi_session::GlobalSession::load(&file).expect("session file");
    let paths = session
        .tabs
        .iter()
        .map(|t| t.repo_path.canonicalize().unwrap_or(t.repo_path.clone()))
        .collect();
    (paths, session.active_index().unwrap())
}

/// A config that never mentions `remember_tabs` shows the row checked in
/// Settings > General (absent means on).
#[test]
fn remember_tabs_is_checked_in_settings_when_the_key_is_absent() {
    let alpha = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let (mut term, _config) = common::launch(alpha.root())
        .config("[general]\nauto_prune = false\nstartup_tab_outside_repo = \"last\"\n")
        .size(200, 60)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .start();
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "General");
    let (_, row) = common::scroll_settings_to(&mut term, "Remember tabs", STEP);
    let line = term.screen().line(row);
    assert!(line.contains("[x]"), "Remember tabs not checked: {line}");
}
