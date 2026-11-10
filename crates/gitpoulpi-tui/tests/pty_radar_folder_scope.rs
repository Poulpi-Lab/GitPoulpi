#![cfg(feature = "radar")]
//! The Radar's folder scopes, in the real binary: "Default clone folder"
//! and "Default browse folder" cover every repository found in the folder set
//! in Settings. Repositories not linked to a host only contribute their work
//! in progress and say so; a folder that is not set is a greyed choice whose
//! pick opens its Settings row.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

/// A repository in `root/name` with one commit and one untracked file (a
/// work-in-progress row), no remote.
fn dirty_repo(root: &Path, name: &str) {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    common::git_ok(&dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join("a.txt"), "a\n").unwrap();
    common::git_ok(&dir, &["add", "a.txt"]);
    common::git_ok(
        &dir,
        &[
            "-c",
            "user.name=T",
            "-c",
            "user.email=t@example.com",
            "commit",
            "-q",
            "-m",
            "first",
        ],
    );
    std::fs::write(dir.join(format!("scratch-{name}.txt")), "x\n").unwrap();
}

struct Setup {
    term: Terminal,
    _config: tempfile::TempDir,
    _window: gitpoulpi_testkit::FixtureRepo,
}

/// The window's own repository is clean; `config` is the body of
/// `config.toml`.
fn setup(session: &str, config: &str) -> Setup {
    let window = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("w.txt", "w\n", "window-repo-start")
        .unwrap()
        .build();
    let config_dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config_dir.path().join("config.toml"), config).unwrap();
    let term = common::launch(window.root())
        .session(session)
        .config_dir(config_dir.path())
        .size(160, 50)
        .mouse_capture()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .startup(STARTUP)
        .ready("window-repo-start")
        .spawn();
    Setup {
        term,
        _config: config_dir,
        _window: window,
    }
}

/// Opens the Radar and the Scope combobox, and moves down `downs` rows.
fn open_scope(term: &mut Terminal, downs: usize) {
    term.send_keys("M-r").expect("Alt+R");
    term.wait_for("the Radar", STEP, |s| {
        s.contains("Scope") && s.contains("Open tabs")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("w").expect("w opens the scope box");
    term.wait_for("the scope choices", STEP, |s| {
        s.contains("Default clone folder") || s.contains("Default browse folder")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..downs {
        term.send_keys("Down").unwrap();
    }
}

/// Two repositories in the clone folder, each with uncommitted work: picking
/// the scope lists both work-in-progress rows, and says they are not linked.
#[test]
fn the_clone_folder_scope_lists_every_repository_in_it() {
    let root = tempfile::tempdir().unwrap();
    dirty_repo(root.path(), "alpha");
    dirty_repo(root.path(), "beta");
    let browse = tempfile::tempdir().unwrap();
    let mut s = setup(
        "pty-lp-folder",
        &format!(
            "[repos]\nclone_root = '{}'\nbrowse_root = '{}'\n",
            root.path().display(),
            browse.path().display()
        ),
    );
    open_scope(&mut s.term, 1);
    s.term.send_keys("Enter").unwrap();
    let screen = s
        .term
        .wait_for("both repositories' work in progress", STEP, |s| {
            s.contains("WORK IN PROGRESS  2")
                && s.contains("alpha")
                && s.contains("beta")
                && s.contains("2 not linked")
                && s.contains("not linked: only its work in progress is shown")
                && s.contains("Link...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("not linked: only its work in progress is shown"),
        "{}",
        screen.text()
    );
    assert!(screen.contains("Link..."), "{}", screen.text());
    // The browse folder is empty: nothing there.
    s.term.send_keys("w").unwrap();
    s.term
        .wait_for("the scope choices", STEP, |s| {
            s.contains("Default browse folder")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Down").unwrap();
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("no repository in the browse folder", STEP, |s| {
            !s.contains("alpha") && !s.contains("beta")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// A repository two levels down (`work/gamma`) is found too, and a repository
/// created after the first scan shows after Refresh, not before.
#[test]
fn a_nested_repository_is_found_and_refresh_scans_again() {
    let root = tempfile::tempdir().unwrap();
    dirty_repo(&root.path().join("work"), "gamma");
    let clone = tempfile::tempdir().unwrap();
    let mut s = setup(
        "pty-lp-folder-refresh",
        &format!(
            "[repos]\nclone_root = '{}'\nbrowse_root = '{}'\n",
            clone.path().display(),
            root.path().display()
        ),
    );
    open_scope(&mut s.term, 2);
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the nested repository", STEP, |s| {
            s.contains("WORK IN PROGRESS  1") && s.contains("gamma")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    dirty_repo(root.path(), "delta");
    s.term.send_keys("r").unwrap();
    s.term
        .wait_for("the new repository after Refresh", STEP, |s| {
            s.contains("WORK IN PROGRESS  2") && s.contains("delta")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// A folder that is not set is greyed with how to set it; picking it opens
/// the Settings row, and the scope stays where it was.
#[test]
fn an_unset_folder_is_greyed_and_opens_its_settings_row() {
    let clone = tempfile::tempdir().unwrap();
    let mut s = setup(
        "pty-lp-folder-unset",
        &format!("[repos]\nclone_root = '{}'\n", clone.path().display()),
    );
    open_scope(&mut s.term, 2);
    let screen = s
        .term
        .wait_for("both folder choices", STEP, |s| {
            s.contains("Default browse folder (set it in Settings)")
                && s.contains("Default clone folder")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Default browse folder (set it in Settings)"),
        "{}",
        screen.text()
    );
    assert!(
        !screen.contains("Default clone folder (set it in Settings)"),
        "{}",
        screen.text()
    );
    s.term.send_keys("Enter").unwrap();
    let screen = s
        .term
        .wait_for("the Settings row", STEP, |s| {
            s.contains("Default browse folder")
                && !s.contains("(set it in Settings)")
                && s.contains("Default clone folder")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("Default clone folder"), "{}", screen.text());
}
