//! Canceling a running sequence, end to end through a pty (real binary,
//! daemon and git; a local bare repository as the remote). Every action takes
//! a few seconds (`GITPOULPI_TEST_ACTION_DELAY_MS`), so there is time to
//! cancel the first step while it is in flight:
//!
//! * Esc cancels (nothing else owns it), and the status line names the step.
//! * The Jobs panel's cancel (the palette's `Jobs: Cancel ...` row) cancels.
//! * The strip's close cross cancels.
//!
//! In every case the steps after the canceled one never run: no commit, no
//! push. Temp config dir and fixture repo per test.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
/// Longer than one delayed action: a step that wrongly went on would have
/// run by then.
const SETTLE: Duration = Duration::from_secs(9);

const CONFIG: &str = r#"
[[sequences]]
id = "backup"
name = "Backup"
scope = "machine"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "commit"
[sequences.steps.options]
message = "must not happen"

[[sequences.steps]]
run = "push"
"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(CONFIG)
        .env("GITPOULPI_TEST_ACTION_DELAY_MS", "4000")
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start()
}

fn run_from_palette(term: &mut Terminal, typed: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(typed).unwrap();
    term.wait_for("the row", STEP, |s| s.contains(typed))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

fn start_and_wait_for_strip(term: &mut Terminal) {
    run_from_palette(term, "Sequence: Backup");
    term.wait_for("the strip on the first step", STEP, |s| {
        s.contains("\u{25b6} Backup 1/3")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Nothing after the first step ran: the branch tip is the fixture's, the
/// remote never got the branch, and the strip is gone.
fn assert_stopped_after_the_first_step(term: &Terminal, repo: &FixtureRepo) {
    std::thread::sleep(SETTLE);
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "initial commit",
        "a later step committed\n{}",
        term.text()
    );
    let remote = repo.remote_path().unwrap();
    let heads = std::process::Command::new("git")
        .arg("-C")
        .arg(remote)
        .args(["branch", "--list"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&heads.stdout).trim().is_empty(),
        "a later step pushed"
    );
    assert!(!term.text().contains("\u{25b6} Backup"), "{}", term.text());
}

#[test]
fn esc_cancels_the_run_and_the_status_names_the_step() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-seq-cancel-esc");
    start_and_wait_for_strip(&mut term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the cancel message", STEP, |s| {
        s.contains("Backup canceled after step 1/3") && s.contains("earlier steps are kept")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_stopped_after_the_first_step(&term, &repo);
    term.stop().unwrap();
}

#[test]
fn the_jobs_panel_cancel_cancels_the_run() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-seq-cancel-jobs");
    start_and_wait_for_strip(&mut term);
    // The run is a job of its own: its Cancel row is in the palette.
    run_from_palette(&mut term, "Jobs: Cancel Sequence Backup");
    term.wait_for("the cancel message", STEP, |s| {
        s.contains("Backup canceled after step 1/3")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_stopped_after_the_first_step(&term, &repo);
    term.stop().unwrap();
}

#[test]
fn the_strips_close_cross_cancels_the_run() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-seq-cancel-cross");
    start_and_wait_for_strip(&mut term);
    let screen = term.screen();
    let row = (0..screen.rows())
        .find(|&r| screen.line(r).contains("\u{25b6} Backup 1/3"))
        .expect("the strip's row");
    let line = screen.line(row);
    let at = line.rfind('\u{2715}').expect("the close cross");
    let col = line[..at].chars().count() as u16;
    term.click(col + 1, row).unwrap();
    let deadline = Instant::now() + STEP;
    while !term.text().contains("Backup canceled after step 1/3") {
        assert!(Instant::now() < deadline, "{}", term.text());
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_stopped_after_the_first_step(&term, &repo);
    term.stop().unwrap();
}
