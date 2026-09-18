//! Client-side jobs (see the background-ops design, B11): an AI request and a
//! workspace walk run in the TUI process, not the daemon, yet they are in the
//! jobs list, survive a tab switch and stay cancellable with the mouse.
//!
//! Real binary and daemon through a pty. The AI is a slow fake agent; the
//! workspace walk is held by the daemon's `GITPOULPI_TEST_TRANSFER_DELAY_MS`
//! hook (its fetch sleeps, nothing is transferred).

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_slow_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn staged_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

fn other_repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("other base")
        .unwrap()
        .build()
}

/// The first repository with a second one open as a tab, AI on with a slow
/// fake agent that answers `answer` after `seconds`.
fn launch_ai(
    repo: &FixtureRepo,
    other: &FixtureRepo,
    session: &str,
    tools: &Path,
    seconds: u32,
    answer: &str,
) -> (Terminal, tempfile::TempDir) {
    let agent = write_slow_fake_agent(tools, seconds, &[answer]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let dir = config.path().join("sessions");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{session}.toml"));
    let mut seeded = GlobalSession::load(&file).unwrap();
    seeded.open(other.root());
    seeded.save(&file).unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the fixture loaded", Duration::from_secs(60), |s| {
        s.contains("initial commit") && s.contains("// WIP")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Clicks the `nth` (0-based) repository tab on the tab bar.
fn show_tab(term: &mut Terminal, nth: usize, expect: &str) {
    let deadline = std::time::Instant::now() + STEP;
    let col = loop {
        let bar = term.screen().line(1);
        let found: Vec<usize> = bar.match_indices("repo").map(|(i, _)| i).collect();
        if found.len() >= 2 {
            break bar[..found[nth]].chars().count() as u16;
        }
        assert!(std::time::Instant::now() < deadline, "{}", term.text());
        std::thread::sleep(Duration::from_millis(300));
    };
    term.click(col + 1, 1).unwrap();
    term.wait_for("the tab", STEP, |s| s.contains(expect))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn indicator_has(s: &Screen, what: &str) -> bool {
    s.text()
        .lines()
        .any(|l| l.contains(what) && l.contains("[x]"))
}

/// An AI request keeps running when its tab is left: the indicator on the
/// other tab lists it, and its `[x]` cancels it with a click.
#[test]
fn an_ai_request_survives_a_tab_switch_and_its_cancel_is_clickable() {
    let (repo, other) = (staged_repo(), other_repo());
    let tools = tempfile::tempdir().unwrap();
    let (mut term, _config) = launch_ai(&repo, &other, "pty-jc-ai", tools.path(), 40, "feat: late");
    term.send_keys("C-g").unwrap();
    term.send_keys("i").unwrap();
    term.wait_for("the AI job", STEP, |s| indicator_has(s, "\u{2726}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    show_tab(&mut term, 0, "other base");
    let screen = term
        .wait_for("the AI job on the other tab", STEP, |s| {
            indicator_has(s, "\u{2726}")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains("initial commit"),
        "really on the other tab"
    );
    // Still running a while later: leaving the tab did not cancel it.
    for _ in 0..12 {
        assert!(
            !term.text().contains("(Canceled)"),
            "leaving the tab canceled the request:
{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(250));
    }

    let (col, row) = term.screen().find("[x]").expect("the [x] button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the canceled AI job", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("\u{2726}") && l.contains("(Canceled)"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The answer of a request that finished while its tab was hidden lands when
/// the tab is shown again.
#[test]
fn an_ai_answer_lands_when_its_tab_is_shown_again() {
    let (repo, other) = (staged_repo(), other_repo());
    let tools = tempfile::tempdir().unwrap();
    let (mut term, _config) = launch_ai(
        &repo,
        &other,
        "pty-jc-ai-back",
        tools.path(),
        6,
        "feat: landed",
    );
    term.send_keys("C-g").unwrap();
    term.send_keys("i").unwrap();
    term.wait_for("the AI job", STEP, |s| indicator_has(s, "\u{2726}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    show_tab(&mut term, 0, "other base");
    term.wait_for("the job ends elsewhere", STEP, |s| {
        !indicator_has(s, "\u{2726}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    show_tab(&mut term, 1, "initial commit");
    term.wait_for("the message landed", STEP, |s| s.contains("feat: landed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

fn toml_path(p: &Path) -> String {
    format!("'{}'", p.display())
}

/// Fetch all over a workspace is one entry in the jobs list, and the panel's
/// Cancel on it is a click away (the walk stops after the repository it is
/// on).
#[test]
fn a_workspace_walk_is_listed_and_cancellable() {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("orbit.txt", "x\n", "orbit-commit")
        .unwrap()
        .build();
    let root: PathBuf = fx.root().parent().unwrap().join("orbit");
    std::fs::rename(fx.root(), &root).unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("workspaces.toml"),
        format!(
            "[[workspace]]\nid = \"acme\"\nname = \"Acme\"\nrepos = [{}]\n",
            toml_path(&root)
        ),
    )
    .unwrap();
    let mut term = common::launch(&root)
        .session("pty-jc-ws")
        .config_dir(config.path())
        .size(200, 50)
        .no_exit_pause()
        .env("GITPOULPI_TEST_TRANSFER_DELAY_MS", "20000")
        .ready("orbit-commit")
        .mouse_capture()
        .spawn();
    term.send_keys("C-p").unwrap();
    term.send_text("Workspace: Manage").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Workspace: Manage...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the page", STEP, |s| s.contains("Acme"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("F").unwrap();
    term.wait_for("the workspace job in the indicator", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Jobs: 2") || l.contains("Workspace: Acme"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-n").unwrap();
    term.send_keys("J").unwrap();
    let on_row = |l: &str| l.contains("Workspace: Acme") && l.contains("Cancel");
    term.wait_for("the panel row", STEP, |s| s.text().lines().any(on_row))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let row = term
        .screen()
        .text()
        .lines()
        .position(on_row)
        .expect("the row") as u16;
    // The row's own Cancel button sits on the same line.
    let line = term.screen().line(row);
    let cancel = line.rfind("Cancel").expect("the row's Cancel button");
    let col = line[..cancel].chars().count() as u16;
    term.click(col + 1, row).unwrap();
    term.wait_for("the walk is stopping", STEP, |s| {
        s.text()
            .lines()
            .find(|l| l.contains("Workspace: Acme"))
            .is_some_and(|l| l.contains("Canceling") || l.contains("Canceled"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
