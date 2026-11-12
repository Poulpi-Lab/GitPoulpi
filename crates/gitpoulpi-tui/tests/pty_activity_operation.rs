//! Every failure says what produced it: the operation (a person's name for it,
//! "Remove remote"), its main arguments (`remote=cm6`), the repository, who
//! asked (TUI, CLI, MCP) and the operation id. The Application tab's line
//! names them, Enter on it opens the Git operations entry of that operation,
//! and the copied error report has the `operation:` and `repository:` lines.
//!
//! Real TUI in a pty, the real `gitpoulpi` CLI against the same session and
//! config directory, real `git`; the failing fetch is a closed local port.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;
const SECRET: &str = "topsecret";

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

/// Runs `gitpoulpi remote remove <name>` against the session's daemon and
/// returns whether it succeeded.
fn cli_remove_remote(repo: &Path, session: &str, config: &Path, name: &str) -> bool {
    Command::new(cli())
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.to_string_lossy()])
        .args(["remote", "remove", name])
        .current_dir(repo)
        .output()
        .expect("run gitpoulpi")
        .status
        .success()
}

fn launch(repo: &Path, session: &str, config: &Path, dump: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .no_exit_pause()
        .env("GITPOULPI_CLIPBOARD_DUMP", dump)
        .ready("operation base")
        .spawn()
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

/// The Application tab open, with the failure line of `label` in it (one
/// whole state: the tab, the line and its tag).
fn wait_for_line(term: &Terminal, line: &str) -> Screen {
    term.wait_for("the Application tab with the failure line", STEP, |s| {
        s.contains("Level:") && s.contains(line)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn dump_text(dump: &Path) -> String {
    std::fs::read_to_string(dump).unwrap_or_default()
}

#[test]
fn a_failure_of_the_cli_names_its_operation_and_enter_opens_its_entry_and_the_report_says_it() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("operation base")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let dump_dir = tempfile::tempdir().unwrap();
    let dump = dump_dir.path().join("clipboard.txt");
    let session = "pty-activity-operation-cli";
    let mut term = launch(repo.root(), session, config.path(), &dump);

    // The CLI asks the TUI's daemon to remove a remote that does not exist.
    assert!(
        !cli_remove_remote(repo.root(), session, config.path(), "cm6"),
        "removing a remote that is not there must fail"
    );

    palette(
        &mut term,
        "Application Log",
        "Activity Log: Application Log",
    );
    let screen = wait_for_line(&term, "Remove remote failed (remote=cm6) [op #");
    let text = screen.text();
    let row = text
        .lines()
        .find(|l| l.contains("Remove remote failed (remote=cm6)"))
        .expect("the failure line");
    assert!(row.contains(", CLI, repo "), "who asked and where: {row}");

    // `]` selects the failure line, Enter opens the entry of its operation.
    term.send_keys("]").unwrap();
    term.send_keys("Enter").unwrap();
    let entry = term
        .wait_for("the operation's entry, open", STEP, |s| {
            s.contains(" Git operations ")
                && s.contains("Operation: Remove remote (remote=cm6)")
                && s.contains("Requested by: CLI")
                && s.contains("Operation id:")
                && s.contains("no such remote: cm6")
                && s.contains("Repository:")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        entry.contains("Remove remote failed"),
        "titled by the operation:\n{}",
        entry.text()
    );

    // Copy: the report is self-explanatory.
    term.send_keys("c").unwrap();
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let report = dump_text(&dump);
    for needle in [
        "GitPoulpi error report",
        "operation: Remove remote (RemoteRemove)",
        "arguments: remote=cm6",
        "requested by: CLI",
        "operation id: ",
    ] {
        assert!(report.contains(needle), "{needle}\n{report}");
    }
    let repo_line = format!("repository: {}", repo.root().display());
    assert!(report.contains(&repo_line), "{repo_line}\n{report}");
}

#[test]
fn a_failure_of_the_window_names_its_operation_and_its_line_opens_the_window_own_entry() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("operation base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://me:{SECRET}@127.0.0.1:1/r.git"),
    ]);
    let config = tempfile::tempdir().unwrap();
    let dump_dir = tempfile::tempdir().unwrap();
    let dump = dump_dir.path().join("clipboard.txt");
    let mut term = launch(
        repo.root(),
        "pty-activity-operation-tui",
        config.path(),
        &dump,
    );

    palette(&mut term, "Fetch All Remotes", "Git: Fetch All Remotes");
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("error: Fetch failed: The remote could not be reached.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    palette(
        &mut term,
        "Application Log",
        "Activity Log: Application Log",
    );
    let screen = wait_for_line(&term, "Fetch all remotes failed [op #");
    assert!(!screen.contains(SECRET), "{}", screen.text());
    let text = screen.text();
    assert!(
        text.lines()
            .any(|l| l.contains("Fetch all remotes failed [op #") && l.contains(", TUI, repo ")),
        "the window's own line names who asked:\n{text}"
    );

    // The window ran it: its own entry opens (the one with the git details),
    // not a second one made from the line.
    term.send_keys("]").unwrap();
    term.send_keys("Enter").unwrap();
    let entry = term
        .wait_for("the window's entry, open", STEP, |s| {
            s.contains(" Git operations ")
                && s.contains("Operation: Fetch all remotes")
                && s.contains("Requested by: TUI")
                && s.contains("command: git fetch")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!entry.contains(SECRET), "{}", entry.text());

    term.send_keys("c").unwrap();
    term.wait_for("the copy confirmation", STEP, |s| {
        s.contains("copied the error details")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let report = dump_text(&dump);
    assert!(
        report.contains("operation: Fetch all remotes (FetchAll)"),
        "{report}"
    );
    assert!(
        report.contains(&format!("repository: {}", repo.root().display())),
        "{report}"
    );
    assert!(!report.contains(SECRET), "{report}");
}
