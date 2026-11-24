//! A user-defined sequence (`[[sequences]]`) is a palette row and a bindable
//! key (`[keybindings.sequence]`), end to end. The run itself is a stub for
//! now: pressing the key reaches it and says so on the status line.
//!
//! A temp `GITPOULPI_CONFIG_DIR` per test; nothing is located by a
//! hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const CONFIG: &str = r#"
[[sequences]]
id = "backup"
name = "Nightly Backup"
scope = "machine"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "commit"

[[sequences]]
id = "empty"
name = "Hollow One"
scope = "machine"

[keybindings.sequence]
backup = "Alt+B"
"#;

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(CONFIG)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains("Message")
        })
        .mouse_capture()
        .start()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "sequence-probe-commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

/// The palette lists the sequence with the key the config gave it, and a
/// sequence `validate` rejects stays listed, grayed with the reason.
#[test]
fn the_palette_lists_a_sequence_with_its_key() {
    let repo = fixture();
    let (mut term, _c) = launch(repo.root(), "pty-sequence-palette");
    term.send_keys("C-p").unwrap();
    term.send_text("Sequence:").unwrap();
    term.wait_for("both sequence rows and the bound key", STEP, |s| {
        s.contains("Sequence: Nightly Backup")
            && s.contains("Alt+B")
            && s.contains("Sequence: Hollow One")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The key reaches the run (a stub for now) from the main window.
#[test]
fn the_bound_key_starts_the_sequence() {
    let repo = fixture();
    let (mut term, _c) = launch(repo.root(), "pty-sequence-key");
    term.send_keys("M-b").unwrap();
    term.wait_for("the stub's status line", STEP, |s| {
        s.contains("Nightly Backup: sequences are not wired yet")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Choosing the row in the palette runs the same thing.
#[test]
fn choosing_the_row_starts_the_sequence() {
    let repo = fixture();
    let (mut term, _c) = launch(repo.root(), "pty-sequence-row");
    term.send_keys("C-p").unwrap();
    term.send_text("Sequence: Nightly").unwrap();
    term.wait_for("the row", STEP, |s| s.contains("Sequence: Nightly Backup"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the stub's status line", STEP, |s| {
        s.contains("Nightly Backup: sequences are not wired yet")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
