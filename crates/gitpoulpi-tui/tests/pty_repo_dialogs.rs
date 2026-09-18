//! The Repository Management tab's Init and Clone dialogs, end to end
//! through a pty by `xtask::pty`: the real binary, daemon and git. No
//! network: the clone is from a fixture repository's path.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state
//! it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 45;

/// Forward slashes: typing a literal `\` through ConPTY's synthetic input
/// does not reach the app (see `pty_ui.rs`), and Windows accepts `/`.
fn typed(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Launches the TUI in `repo` with a fresh config dir holding `config`
/// as its `config.toml`, and waits for `wait_for`.
fn launch(
    repo: &Path,
    session: &str,
    config_toml: &str,
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), config_toml).expect("seed config.toml");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    let needle = wait_for.to_string();
    term.wait_for("the first window", STARTUP, move |s| s.contains(&needle))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// `Ctrl+O`, `i`: the Init dialog. Its folder is replaced (`Ctrl+U`,
/// then typed), a name is typed, `Enter` creates the repository through
/// the daemon, and the new repository opens as a tab - on the branch
/// `general.default_branch_name` names.
#[test]
fn init_creates_a_repository_and_opens_it_as_a_tab() {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "init-test-start-commit")
        .unwrap()
        .build();
    // Digits in the folder, always: a digit typed into a dialog must not be
    // swallowed as a vim count prefix (`4j`).
    let tmp = tempfile::tempdir().unwrap();
    let parent = tmp.path().join("folder-2026");
    std::fs::create_dir(&parent).unwrap();
    let (mut term, _config) = launch(
        start.root(),
        "pty-init-dialog",
        "[general]\ndefault_branch_name = 'trunk'\n",
        "init-test-start-commit",
    );

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Init")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("i").expect("Init");
    term.wait_for("the Init dialog", STEP, |s| {
        s.contains("Initialize a repository") && s.contains("Initial branch") && s.contains("trunk")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The dialog opens on Name; Shift+Tab twice reaches the folder.
    term.send_keys("BTab").unwrap();
    term.send_keys("BTab").unwrap();
    term.send_keys("C-u").expect("clear the folder");
    let folder = typed(&parent);
    term.send_text(&folder).unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("made-by-init").unwrap();
    term.wait_for("the plan names the new folder", STEP, |s| {
        s.contains("Will create") && s.contains("made-by-init")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("create it");

    let screen = term
        .wait_for("the new repository open as a tab", STARTUP, |s| {
            s.contains("made-by-init")
                && !s.contains("Initialize a repository")
                && !s.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let made = parent.join("made-by-init");
    assert!(
        made.join(".git").is_dir(),
        "no repository at {}
{}",
        made.display(),
        screen.text()
    );
    let head = std::fs::read_to_string(made.join(".git").join("HEAD")).unwrap();
    assert!(head.contains("refs/heads/trunk"), "{head}");
}

/// `Ctrl+O`, `c`: the Clone dialog, its "Where to clone to" pre-filled
/// from `repos.clone_root`. A fixture repository's path as the URL,
/// `Enter` clones it through the daemon, and the clone opens as a tab
/// showing the source's commit.
#[test]
fn clone_from_a_local_path_lands_under_the_clone_root_and_opens_as_a_tab() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("s.txt", "s\n", "clone-source-only-commit")
        .unwrap()
        .build();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-test-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let (mut term, _config) = launch(
        start.root(),
        "pty-clone-dialog",
        &format!("[repos]\nclone_root = '{}'\n", root.path().display()),
        "clone-test-start-commit",
    );

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    let root_name = root
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let expect_root = root_name.clone();
    term.wait_for("the Clone dialog, pre-filled", STEP, move |s| {
        s.contains("Clone a repository")
            && s.contains("Clone with URL")
            && s.contains("connect a hosting")
            && s.contains("Sparse checkout")
            && s.contains(&expect_root)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let url = typed(source.root());
    term.send_text(&url).expect("type the source path");
    let name = source
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let expect_name = name.clone();
    term.wait_for("the full path, and the button live", STEP, move |s| {
        s.contains("Full path:") && s.contains(&expect_name) && !s.contains("enter the URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("clone it");

    term.wait_for("the clone open as a tab", STARTUP, |s| {
        s.contains("clone-source-only-commit") && !s.contains("Clone a repository")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let dest = root.path().join(&name);
    assert!(dest.join(".git").is_dir(), "no clone at {}", dest.display());
}
