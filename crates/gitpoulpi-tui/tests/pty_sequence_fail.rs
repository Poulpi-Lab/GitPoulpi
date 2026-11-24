//! A sequence whose step fails stops there, end to end through a pty (real
//! binary, daemon and git): the push goes to a remote whose repository is gone.
//! The status line names the sequence and the step, the strip turns into a
//! failure with `Details`, the earlier commit is kept, and the Activity Log
//! has an entry naming the sequence, the step and its options.
//!
//! Temp config dir and fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

const CONFIG: &str = r#"
[[sequences]]
id = "backup"
name = "Backup"
scope = "machine"

[[sequences.steps]]
run = "commit"
[sequences.steps.options]
message = "wip: sequence commit"

[[sequences.steps]]
run = "push"
"#;

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    // The remote is configured but gone: the push has nowhere to go.
    std::fs::remove_dir_all(repo.remote_path().unwrap()).unwrap();
    repo
}

#[test]
fn a_failing_push_stops_the_run_at_its_step_with_an_activity_log_entry() {
    let repo = fixture();
    let (mut term, _config): (Terminal, _) = common::launch(repo.root())
        .session("pty-seq-fail")
        .config(CONFIG)
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Sequence: Backup").unwrap();
    term.wait_for("the row", STEP, |s| s.contains("Sequence: Backup"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();

    let screen = term
        .wait_for("the failure, naming the sequence and the step", STEP, |s| {
            s.contains("Backup stopped at step 2/2")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "wip: sequence commit",
        "the earlier step is kept"
    );
    // The strip is the failure, with a Details button and the close cross.
    let row = (0..screen.rows())
        .find(|&r| screen.line(r).contains("\u{25b6} Backup stopped"))
        .expect("the strip");
    let line = screen.line(row);
    assert!(
        line.contains("Details") && line.contains('\u{2715}'),
        "{line}"
    );
    let at = line.rfind("Details").unwrap();
    let col = line[..at].chars().count() as u16;
    term.click(col + 1, row).unwrap();
    term.wait_for("the Activity Log entry for the sequence", STEP, |s| {
        s.contains("Activity Log") && s.contains("Sequence stopped")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Opened, the entry's details name the sequence, the step and its options.
    term.send_keys("Enter").unwrap();
    term.wait_for("the entry's details", STEP, |s| s.contains("options:"))
        .unwrap_or_else(|e| panic!("{e}"));
    let text = term.text();
    assert!(
        text.contains("Backup") && text.contains("Git: Push") && text.contains("options"),
        "the entry names the sequence, the step and its options:\n{text}"
    );
    term.stop().unwrap();
}
