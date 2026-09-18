//! Which tab a start opens on: `general.startup_tab_in_repo`,
//! `general.startup_tab_outside_repo` and `general.startup_tab_launcher`,
//! the last one chosen by the `--launcher` flag the desktop shortcuts pass.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, no network.

#![cfg(unix)]

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const COMMIT: &str = "startup-tab-commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", COMMIT)
        .unwrap()
        .build()
}

fn on_home(s: &Screen) -> bool {
    s.contains("Repository Management") && s.contains("Search repositories")
}

fn radar_shows(s: &Screen) -> bool {
    s.contains("Scope") && s.contains("Open tabs") && s.contains("Refresh")
}

fn on_repo(s: &Screen) -> bool {
    s.contains(COMMIT) && !on_home(s)
}

/// Starts the TUI in `cwd` and waits for `ready` (a description and a test).
fn start(
    cwd: &Path,
    config: &Path,
    session: &str,
    extra: &[&str],
    what: &str,
    ready: fn(&Screen) -> bool,
) -> Terminal {
    let mut launch = common::launch(cwd)
        .session(session)
        .config_dir(config)
        .size(180, 45)
        .no_exit_pause()
        .ready_when(what, ready);
    for arg in extra {
        launch = launch.arg(arg);
    }
    launch.spawn()
}

fn write_config(dir: &Path, general: &str) {
    std::fs::write(dir.join("config.toml"), format!("[general]\n{general}")).unwrap();
}

fn quit(term: &mut Terminal) {
    term.send_keys("q").unwrap();
    for _ in 0..900 {
        if term.exited() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the TUI did not exit:\n{}", term.text());
}

/// Inside a repository the default is that repository's tab; set to `home`
/// the same start opens on Home instead.
#[test]
fn inside_a_repository_the_setting_picks_the_tab() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();

    let term = start(
        repo.root(),
        config.path(),
        "startup-in-default",
        &[],
        "the repository tab",
        on_repo,
    );
    drop(term);

    write_config(config.path(), "startup_tab_in_repo = \"home\"\n");
    let term = start(
        repo.root(),
        config.path(),
        "startup-in-home",
        &[],
        "Home",
        on_home,
    );
    assert!(!term.screen().contains(COMMIT), "{}", term.text());
}

/// Outside a repository the default is Home; `radar` opens the Radar.
#[test]
fn outside_a_repository_the_setting_picks_the_tab() {
    let outside = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();

    let term = start(
        outside.path(),
        config.path(),
        "startup-out-default",
        &[],
        "Home",
        on_home,
    );
    assert!(!radar_shows(&term.screen()), "{}", term.text());
    drop(term);

    write_config(config.path(), "startup_tab_outside_repo = \"radar\"\n");
    let _term = start(
        outside.path(),
        config.path(),
        "startup-out-radar",
        &[],
        "the Radar",
        radar_shows,
    );
}

/// A start with `--launcher` uses `startup_tab_launcher`, here `last`: the
/// tab left active by the previous run opens, though the folder is no
/// repository. Without the flag the same start is Home.
#[test]
fn the_launcher_flag_uses_the_launcher_setting() {
    let repo = fixture();
    let outside = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    write_config(config.path(), "startup_tab_launcher = \"last\"\n");
    let session = "startup-launcher";

    // Run 1: opens the repository's tab, then quits.
    let mut term = start(
        repo.root(),
        config.path(),
        session,
        &[],
        "the repository tab",
        on_repo,
    );
    quit(&mut term);

    // Not from a launcher: outside a repository that is Home.
    let mut term = start(outside.path(), config.path(), session, &[], "Home", on_home);
    quit(&mut term);

    // From a launcher: the last tab.
    let _term = start(
        outside.path(),
        config.path(),
        session,
        &["--launcher"],
        "the last tab",
        on_repo,
    );
}

/// A repository given with `--repo` always opens its own tab, whatever the
/// settings say.
#[test]
fn an_explicit_repo_argument_beats_the_settings() {
    let repo = fixture();
    let outside = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    write_config(
        config.path(),
        "startup_tab_in_repo = \"home\"\nstartup_tab_outside_repo = \"home\"\n\
         startup_tab_launcher = \"radar\"\n",
    );
    let root = repo.root().to_string_lossy().into_owned();
    let _term = start(
        outside.path(),
        config.path(),
        "startup-explicit",
        &["--repo", &root, "--launcher"],
        "the repository tab",
        on_repo,
    );
}

/// The real taskbar case: no `-s`, so every start folder has a session of its
/// own and "last" travels through `last-session`. Run 1 leaves a repository
/// tab active; a `--launcher` start from a plain folder reopens it; the same
/// start without the flag is Home; and a launcher start from inside a
/// repository still follows the launcher setting (`home` here).
#[test]
fn the_launcher_flag_works_with_implicit_sessions_from_any_folder() {
    let repo = fixture();
    let outside = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    write_config(
        config.path(),
        "startup_tab_launcher = \"last\"\nremember_tabs = true\n",
    );
    let go = |cwd: &Path, extra: &[&str], what: &str, ready: fn(&Screen) -> bool| {
        let mut launch = common::launch(cwd)
            .implicit_session()
            .config_dir(config.path())
            .size(180, 45)
            .no_exit_pause()
            .ready_when(what, ready);
        for arg in extra {
            launch = launch.arg(arg);
        }
        launch.spawn()
    };

    let mut term = go(repo.root(), &[], "the repository tab", on_repo);
    quit(&mut term);

    let mut term = go(outside.path(), &[], "Home", on_home);
    quit(&mut term);

    let mut term = go(outside.path(), &["--launcher"], "the last tab", on_repo);
    quit(&mut term);

    write_config(config.path(), "startup_tab_launcher = \"home\"\n");
    let _term = go(repo.root(), &["--launcher"], "Home", on_home);
}
