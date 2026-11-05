//! The hidden/solo ref filter is shared state: `gitpoulpi refs hide|show|solo`
//! (CLI/MCP) and the TUI both go through the daemon, so a running TUI follows
//! the CLI live (`Event::RefVisibilityChanged`) and a hide done in the TUI is
//! visible to the CLI and to `repo-state.toml`.
//!
//! The real TUI in a pty, the real `gitpoulpi` CLI against the same session
//! and config directory, real `git`.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use common::git_trimmed as git;
use gitpoulpi_core::domain::RefVisibility;
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const SIDEBAR_RIGHT: u16 = 40;
/// `HIDDEN_IN_GRAPH_MARK` in `gitpoulpi_tui`: the gutter mark of a hidden ref.
const HIDDEN: &str = "\u{2298}";

/// `main` with two commits, and `side` with a commit of its own.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root();
    git(root, &["checkout", "-b", "side"]);
    std::fs::write(root.join("c.txt"), "three\n").unwrap();
    git(root, &["add", "c.txt"]);
    git(root, &["commit", "-m", "side work"]);
    git(root, &["checkout", "-"]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .ready_when("the log and the sidebar", |s| {
            s.contains("side work") && s.contains("second")
        })
        .spawn()
}

/// The `gitpoulpi` binary, built next to the TUI's by the workspace build
/// (built on demand when only this package's tests were run).
fn cli() -> PathBuf {
    let tui = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui"));
    let path = tui.with_file_name(format!("gitpoulpi{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("run cargo build");
        assert!(status.success(), "building the gitpoulpi CLI failed");
    }
    path
}

fn refs(repo: &Path, session: &str, config: &Path, args: &[&str]) -> serde_json::Value {
    let out = Command::new(cli())
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.to_string_lossy()])
        .arg("refs")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run gitpoulpi");
    assert!(
        out.status.success(),
        "refs {args:?} failed: {} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

/// Whether `text` is drawn in the Log table (between the sidebar and the
/// commit-details pane, which still shows the selected commit).
fn in_log(screen: &Screen, text: &str) -> bool {
    screen
        .find_all(text)
        .into_iter()
        .any(|(col, _)| (SIDEBAR_RIGHT..130).contains(&col))
}

/// Where the sidebar draws `label` (left of `SIDEBAR_RIGHT`), first match.
fn sidebar_label(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    screen
        .find_all(label)
        .into_iter()
        .filter(|(col, _)| *col < SIDEBAR_RIGHT)
        .min_by_key(|(_, row)| *row)
}

/// Whether the sidebar's `label` row wears the hidden mark in its gutter.
fn marked_hidden(screen: &Screen, label: &str) -> bool {
    sidebar_label(screen, label)
        .is_some_and(|(col, row)| screen.cell(col - 2, row).is_some_and(|c| c.text == HIDDEN))
}

fn wait_hidden(term: &Terminal, label: &str, hidden: bool) {
    term.wait_for(
        &format!("{label:?} {}", if hidden { "hidden" } else { "shown" }),
        STEP,
        |s| sidebar_label(s, label).is_some() && marked_hidden(s, label) == hidden,
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_cli_hiding_showing_soloing_and_clearing_refs_reaches_a_running_tui() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let session = "ref-vis-cli";
    let term = launch(repo.root(), session, config.path());
    let run = |args: &[&str]| refs(repo.root(), session, config.path(), args);

    run(&["hide", "side"]);
    wait_hidden(&term, "side", true);
    run(&["show", "side"]);
    wait_hidden(&term, "side", false);

    // `--clear` empties what its mode owns.
    run(&["hide", "side"]);
    wait_hidden(&term, "side", true);
    run(&["hide", "--clear"]);
    wait_hidden(&term, "side", false);

    // Solo reshapes the Log itself: only `main`'s ancestry stays.
    run(&["solo", "main"]);
    term.wait_for("only main's history in the Log", STEP, |s| {
        s.contains("second") && !in_log(s, "side work")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    run(&["solo", "--clear"]);
    term.wait_for("the side commit back", STEP, |s| in_log(s, "side work"))
        .unwrap_or_else(|e| panic!("{e}"));

    let _ = term.stop();
}

#[test]
fn hiding_a_ref_in_the_tui_is_visible_to_the_cli_and_the_state_file() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let session = "ref-vis-tui";
    let mut term = launch(repo.root(), session, config.path());

    let screen = term
        .wait_for("side in the sidebar", STEP, |s| {
            sidebar_label(s, "side").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = sidebar_label(&screen, "side").unwrap();
    // The gutter's eye is two cells left of the name.
    term.click(col - 2, row).unwrap();
    wait_hidden(&term, "side", true);

    // The daemon wrote it: the state file and a CLI query both say so.
    let state_path = repo.state_dir().join("repo-state.toml");
    let deadline = Instant::now() + STEP;
    loop {
        let state = RepoState::load(&state_path).unwrap_or_default();
        if state.ref_visibility.get("side") == Some(&RefVisibility::Hidden) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "repo-state.toml never got `side`"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // `show` of a name nothing hides changes nothing and reports the state.
    let state = refs(repo.root(), session, config.path(), &["show", "nothing"]);
    assert_eq!(state["hidden"], serde_json::json!(["side"]));

    let _ = term.stop();
}

#[test]
fn the_cli_hiding_a_whole_remote_reaches_a_running_tui() {
    let repo = fixture();
    let root = repo.root();
    // A second history behind a remote of its own.
    let archive = FixtureRepoBuilder::new()
        .unwrap()
        .commit("archive only commit")
        .unwrap()
        .build();
    git(
        root,
        &[
            "remote",
            "add",
            "archive",
            &archive.root().to_string_lossy(),
        ],
    );
    git(root, &["fetch", "-q", "archive"]);
    let config = tempfile::tempdir().unwrap();
    let session = "ref-vis-remote";
    let term = common::launch(root)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .ready_when("the log with the archive commit", |s| {
            s.contains("side work") && s.contains("archive only commit")
        })
        .spawn();

    let hid = refs(
        root,
        session,
        config.path(),
        &["hide", "--remote", "archive"],
    );
    assert_eq!(hid["hidden_remotes"], serde_json::json!(["archive"]));
    term.wait_for("the archive commit gone, the rest stays", STEP, |s| {
        !s.contains("archive only commit") && s.contains("side work") && s.contains("second")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let shown = refs(
        root,
        session,
        config.path(),
        &["show", "--remote", "archive"],
    );
    assert_eq!(shown["hidden_remotes"], serde_json::json!([]));
    term.wait_for("the archive commit back", STEP, |s| {
        s.contains("archive only commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}
