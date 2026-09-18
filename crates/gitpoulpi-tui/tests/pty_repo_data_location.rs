//! Where GitPoulpi keeps its per-repository data (`session.per_repo_state_dir`):
//! inside the git directory by default, migrated from a legacy
//! `<repo>/.gitpoulpi`, chosen in Settings > General; and the General page's
//! groups (Undo history holds only undo items, the launcher's per-place rows
//! sit in the Launcher group). Real binary, daemon and system git in a pty.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::RefVisibility;
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("data base")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 300)
        .ready("data base")
        .spawn()
}

fn open_general(term: &mut Terminal) {
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "General");
}

#[test]
fn a_legacy_folder_is_moved_into_the_git_dir_and_removed() {
    let repo = fixture();
    // What an older build left in the working tree (the explicit legacy
    // string reads as `in_worktree`, which is where it was written).
    let legacy = RepoState::path_for(repo.root(), ".gitpoulpi");
    let mut state = RepoState::default();
    state
        .ref_visibility
        .insert("old-branch".into(), RefVisibility::Hidden);
    state.save(&legacy).unwrap();
    assert!(legacy.is_file());

    let config = tempfile::tempdir().unwrap();
    let term = launch(repo.root(), "pty-data-migrate", config.path());
    let moved = repo.state_dir().join("repo-state.toml");
    let deadline = Instant::now() + STEP;
    while !moved.is_file() || repo.root().join(".gitpoulpi").exists() {
        assert!(
            Instant::now() < deadline,
            "the legacy state was not moved: {}\n{}",
            moved.display(),
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let loaded = RepoState::load(&moved).unwrap();
    assert_eq!(
        loaded.ref_visibility.get("old-branch"),
        Some(&RefVisibility::Hidden)
    );
    let _ = term.stop();
}

#[test]
fn the_location_is_a_dropdown_in_general_settings() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-data-setting", config.path());
    open_general(&mut term);
    let screen = term
        .wait_for("the data location row", STEP, |s| {
            s.contains("Repository data location") && s.contains("Inside .git")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let stored = || std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    let (c, r) = screen.find("Repository data location").unwrap();
    term.click(c + 1, r).expect("click the row");
    let list = term
        .wait_for("the locations", STEP, |s| {
            s.contains("In the working tree") && s.contains("In my user data folder")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = list.find("In my user data folder").unwrap();
    term.click(c + 1, r).expect("pick the user data folder");
    let deadline = Instant::now() + STEP;
    while !stored().contains("per_repo_state_dir = \"user_data\"") {
        assert!(
            Instant::now() < deadline,
            "never stored: {}\n{}",
            stored(),
            term.text()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = term.stop();
}

#[test]
fn undo_history_holds_only_undo_items_and_the_launcher_rows_follow_the_launcher_heading() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "pty-general-groups", config.path());
    open_general(&mut term);
    let screen = term
        .wait_for("the Launcher group", STEP, |s| {
            s.contains("Undo history") && s.contains("Launcher")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let row = |needle: &str| {
        screen
            .find_all(needle)
            .into_iter()
            .filter(|(c, _)| *c > 28)
            .map(|(_, r)| r)
            .next()
            .unwrap_or_else(|| panic!("no {needle:?} row\n{}", screen.text()))
    };
    let undo = row("Undo history");
    let depth = row("Undo history depth");
    let outside = row("Record changes made outside");
    let log = row("Application log");
    let launcher = row("Launcher");
    let terminal = row("Terminal for shortcuts");
    assert!(
        undo < depth && depth < outside && outside < log && log < launcher && launcher < terminal,
        "order: undo {undo}, depth {depth}, outside {outside}, log {log}, launcher {launcher}, terminal {terminal}\n{}",
        screen.text()
    );
    let _ = term.stop();
}
