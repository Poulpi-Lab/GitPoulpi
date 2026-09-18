//! A clone runs in the background (see the background-ops design, B9): the Clone
//! dialog closes on Submit, the clone is the daemon's job and carries on
//! across window switches, the jobs indicator shows it with a clickable
//! `[x]`, and when it ends the new repository opens as the active tab (a
//! failure leaves the status line's error, a cancel removes the half folder).
//!
//! The "in flight" clone is either a `SlowRemote` (accepts, never answers:
//! git waits) or the daemon's `GITPOULPI_TEST_TRANSFER_DELAY_MS` hook in
//! front of a real local clone. Real binary, daemon and git through a pty;
//! temp config dirs and fixture repositories only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder, SlowRemote};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    root: tempfile::TempDir,
    _start: FixtureRepo,
    _config: tempfile::TempDir,
}

fn typed(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// The window on the fixture, then the management tab with the Clone dialog
/// open and `url` typed.
fn open_clone(session: &str, delay_ms: Option<u32>, url: &str, name: &str) -> Setup {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "clone-bg-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.path().display()),
    )
    .unwrap();
    let mut launch = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause();
    if let Some(ms) = delay_ms {
        launch = launch.env("GITPOULPI_TEST_TRANSFER_DELAY_MS", ms.to_string());
    }
    let term = launch.spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("clone-bg-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let mut s = Setup {
        term,
        root,
        _start: start,
        _config: config,
    };
    open_dialog(&mut s.term, url, name);
    s
}

fn open_dialog(term: &mut Terminal, url: &str, name: &str) {
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(url).expect("the URL");
    let n = name.to_string();
    term.wait_for("the full path", STEP, move |s| {
        s.contains("Full path:") && s.contains(&n)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The indicator on the footer rows: `Clone: <name>` with its `[x]`.
fn indicator_shows(s: &Screen) -> bool {
    s.text()
        .lines()
        .any(|l| l.contains("Clone: ") && l.contains("[x]"))
}

fn dialog_gone_with_indicator(term: &mut Terminal) {
    term.wait_for("the dialog closed, the clone in the indicator", STEP, |s| {
        !s.contains("Clone a repository") && indicator_shows(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Leaves Home for the first repository's tab with its Switch button.
fn back_to_the_repository(term: &mut Terminal) {
    let (col, row) = term.screen().find("Switch").expect("a Switch button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the first repository again", STEP, |s| {
        s.contains("clone-bg-start-commit")
    })
    .unwrap_or_else(|e| {
        panic!(
            "{e}
{}",
            term.text()
        )
    });
}

/// Submit closes the dialog, the clone shows in the indicator on every
/// window, a click on its `[x]` cancels it and the half folder is gone. A
/// second clone into the same folder meanwhile is refused.
#[test]
fn the_dialog_closes_the_clone_continues_and_a_click_cancels_it() {
    let slow = SlowRemote::start();
    let mut s = open_clone("pty-clone-bg-cancel", None, &slow.url(), "slow");
    s.term.send_keys("Enter").unwrap();
    dialog_gone_with_indicator(&mut s.term);
    let dest = s.root.path().join("slow");

    // The same destination again: refused, nothing overwritten.
    s.term.send_keys("c").unwrap();
    s.term
        .wait_for("the dialog again", STEP, |x| {
            x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text(&slow.url()).unwrap();
    s.term
        .wait_for("the full path", STEP, |x| x.contains("Full path:"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the refusal", STEP, |x| x.contains("already cloning into"))
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                s.term.text()
            )
        });
    s.term.send_keys("Esc").unwrap();
    s.term
        .wait_for("the dialog closed", STEP, |x| {
            !x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // Other window: the indicator follows.
    back_to_the_repository(&mut s.term);
    s.term
        .wait_for("the indicator on the repository window", STEP, |x| {
            indicator_shows(x)
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = s.term.screen().find("[x]").expect("the [x] button");
    s.term.click(col + 1, row).unwrap();
    s.term
        .wait_for("the canceled clone", STEP, |x| {
            x.text()
                .lines()
                .any(|l| l.contains("Clone: ") && l.contains("(Canceled)"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let deadline = std::time::Instant::now() + STEP;
    while dest.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "the half folder is still there: {}",
            dest.display()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    s.term.stop().unwrap();
}

/// A clone that ends while another window is shown opens its repository as
/// the active tab.
#[test]
fn a_finished_clone_switches_to_the_new_tab() {
    let source = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("s.txt", "s\n", "clone-bg-source-commit")
        .unwrap()
        .build();
    let name = source
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut s = open_clone(
        "pty-clone-bg-finish",
        Some(10_000),
        &typed(source.root()),
        &name,
    );
    s.term.send_keys("Enter").unwrap();
    dialog_gone_with_indicator(&mut s.term);
    back_to_the_repository(&mut s.term);
    s.term
        .wait_for("the new repository as the active tab", STARTUP, |x| {
            x.contains("clone-bg-source-commit") && !x.contains("clone-bg-start-commit")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(s.root.path().join(&name).join(".git").is_dir());
    s.term.stop().unwrap();
}

/// A clone that fails leaves its reason on screen (the red status line, with
/// its Activity entry), not a message that expires.
#[test]
fn a_failed_clone_leaves_its_error_in_the_status_line() {
    let mut s = open_clone(
        "pty-clone-bg-fail",
        None,
        "git://127.0.0.1:1/nope.git",
        "nope",
    );
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the failure", STEP, |x| x.contains("error: Clone failed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(!s.root.path().join("nope").exists());
    s.term.stop().unwrap();
}
