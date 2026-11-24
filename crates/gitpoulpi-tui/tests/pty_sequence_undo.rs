#![cfg(feature = "ai")]
//! "Undo whole sequence" end to end through a pty: a Backup-like run (fake
//! local agent, local bare remote) leaves its entries tagged, the Undo
//! history shows the `⟳ Backup` badge, and the button undoes the commit and
//! the stage; the commit is already on the remote, so the daemon's "undo
//! anyway?" question comes up in between and its answer carries on.
//!
//! A temp config dir and fixture repo; nothing is located by coordinates.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

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

fn wait_until(what: &str, f: impl Fn() -> bool) {
    let deadline = Instant::now() + STEP;
    while !f() {
        assert!(Instant::now() < deadline, "{what}");
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The subject of the remote's tip of `branch`, empty while it is not there.
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

#[test]
fn undo_whole_sequence_undoes_the_run_and_asks_for_the_pushed_commit() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["feat: sequence backup"]);
    let toml = format!(
        "{}\n[[sequences]]\nid = \"backup\"\nname = \"Backup\"\nscope = \"machine\"\n\n\
         [[sequences.steps]]\nrun = \"stage_all\"\n\n\
         [[sequences.steps]]\nrun = \"ai_generate_commit_message\"\npause = \"never\"\n\n\
         [[sequences.steps]]\nrun = \"commit\"\n\n\
         [[sequences.steps]]\nrun = \"push\"\n",
        ai_config_toml(&agent)
    );
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-seq-undo")
        .config(&toml)
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
    let branch = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    wait_until("the commit and the push", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "feat: sequence backup"
            && remote_subject(&repo, &branch) == "feat: sequence backup"
    });
    term.wait_for("the strip gone", STEP, |s| !s.contains("\u{25b6} Backup"))
        .unwrap_or_else(|e| panic!("{e}"));

    // The history: the badge and the whole-sequence button.
    term.send_keys("M-z").unwrap();
    term.wait_for("the badge on the run's rows", STEP, |s| {
        s.contains("can be undone") && s.contains("\u{27F3} Backup")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.text().contains("Undo whole sequence"),
        "{}",
        term.text()
    );
    term.send_keys("x").unwrap();
    term.wait_for("the confirmation naming the sequence", STEP, |s| {
        s.contains("Undo the whole sequence \"Backup\"?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    // The commit is on the remote: the daemon asks before it goes.
    term.wait_for("the pushed-commit question", STEP, |s| {
        s.contains("Undo anyway")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "feat: sequence backup",
        "nothing was undone before the answer"
    );
    term.send_keys("Enter").unwrap();
    wait_until("the commit undone", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "initial commit"
    });
    term.wait_for("the sequence named in the status", STEP, |s| {
        s.contains("undid the sequence \"Backup\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The stage is undone too: the change is back in the working tree only.
    assert_eq!(repo.git(&["status", "--porcelain"]).trim(), "M a.txt");
    term.stop().unwrap();
}
