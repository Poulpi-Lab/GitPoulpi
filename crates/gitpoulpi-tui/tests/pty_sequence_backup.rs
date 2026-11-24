#![cfg(feature = "ai")]
//! The Backup sequence (stage everything, write the message with the AI,
//! commit, push) run end to end through a pty: the real binary, daemon and
//! git, a fake local AI agent (a script that prints a canned message, never a
//! real agent) and a local bare repository as the remote.
//!
//! * By default the AI message step pauses: the message lands in the
//!   composer, the strip says "Review the message, then Continue", and the
//!   Continue button commits and pushes.
//! * With `pause = "never"` on that step the run goes straight through.
//!
//! A temp config dir and fixture repo per test; nothing is located by a
//! hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);

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

fn config(agent: &Path, ai_pause: &str) -> String {
    format!(
        "{}\n[[sequences]]\nid = \"backup\"\nname = \"Backup\"\nscope = \"machine\"\n\n\
         [[sequences.steps]]\nrun = \"stage_all\"\n\n\
         [[sequences.steps]]\nrun = \"ai_generate_commit_message\"\n{ai_pause}\n\n\
         [[sequences.steps]]\nrun = \"commit\"\n\n\
         [[sequences.steps]]\nrun = \"push\"\n",
        ai_config_toml(agent)
    )
}

fn launch(repo: &FixtureRepo, session: &str, toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .start()
}

fn run_from_palette(term: &mut Terminal, row: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(row).unwrap();
    term.wait_for("the row", STEP, |s| s.contains(row))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

/// Waits (bounded) until `ok` holds; panics with the screen otherwise.
fn eventually(term: &Terminal, what: &str, ok: impl Fn() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "{what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The subject of `branch`'s tip in the remote, empty while it is not there.
fn remote_subject(repo: &FixtureRepo, branch: &str) -> String {
    let Some(remote) = repo.remote_path() else {
        return String::new();
    };
    std::process::Command::new("git")
        .arg("-C")
        .arg(remote)
        .args(["log", "-1", "--format=%s", branch])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn branch(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string()
}

#[test]
fn the_default_pause_lands_the_message_then_continue_commits_and_pushes() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["feat: sequence backup"]);
    let (mut term, _config) = launch(&repo, "pty-seq-backup", &config(&agent, ""));
    run_from_palette(&mut term, "Sequence: Backup");

    let screen = term
        .wait_for("the strip asking to review the message", STEP, |s| {
            s.contains("Review the message, then Continue") && s.contains("feat: sequence backup")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "initial commit",
        "nothing is committed before Continue"
    );
    // The real Continue button of the strip: the last "Continue" on its row
    // (the strip's words end in one too).
    let row = (0..screen.rows())
        .find(|&r| screen.line(r).contains("Backup 2/4"))
        .expect("the strip's row");
    let line = screen.line(row);
    let at = line.rfind("Continue").expect("the Continue button");
    let col = line[..at].chars().count() as u16;
    term.click(col + 1, row).unwrap();

    let b = branch(&repo);
    eventually(&term, "the commit and the push", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "feat: sequence backup"
            && remote_subject(&repo, &b)
                == "feat: sequence backup"
    });
    term.wait_for("the strip gone", STEP, |s| !s.contains("\u{25b6} Backup"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn pause_never_runs_straight_through() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["feat: straight through"]);
    let (mut term, _config) = launch(
        &repo,
        "pty-seq-backup-never",
        &config(&agent, "pause = \"never\""),
    );
    run_from_palette(&mut term, "Sequence: Backup");
    let b = branch(&repo);
    eventually(&term, "the commit and the push without a Continue", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "feat: straight through"
            && remote_subject(&repo, &b)
                == "feat: straight through"
    });
    assert!(
        !term.text().contains("Review the message"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The keyboard twin of the Continue button: the palette's `Sequence:
/// Continue` row (and the strip's ✕ twin, `Sequence: Cancel`, is covered by
/// `pty_sequence_cancel.rs`).
#[test]
fn the_palette_continue_row_goes_on_too() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["feat: palette continue"]);
    let (mut term, _config) = launch(&repo, "pty-seq-backup-palette", &config(&agent, ""));
    run_from_palette(&mut term, "Sequence: Backup");
    term.wait_for("the strip asking to review the message", STEP, |s| {
        s.contains("Review the message, then Continue")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    run_from_palette(&mut term, "Sequence: Continue");
    let b = branch(&repo);
    eventually(&term, "the commit and the push", || {
        remote_subject(&repo, &b) == "feat: palette continue"
    });
    term.stop().unwrap();
}
