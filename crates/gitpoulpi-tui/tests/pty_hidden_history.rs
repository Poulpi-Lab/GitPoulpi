//! Hiding refs changes the Log's walk: the commits only a hidden ref reaches
//! disappear (the daemon walks from the visible tips), and showing the refs
//! again brings them back, the paging restarted.
//!
//! The scenario is an `archive` remote whose history is unrelated to the
//! repository's own, hidden branch by branch through the CLI (the same
//! daemon state the TUI's own eye icons write).
//!
//! The real TUI in a pty, the real `gitpoulpi` CLI against the same session
//! and config directory, real `git`.

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use common::{git_ok, git_trimmed as git};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);
const SIDEBAR_RIGHT: u16 = 40;
const COLS: u16 = 200;
const ROWS: u16 = 50;

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

fn refs(repo: &Path, session: &str, config: &Path, args: &[&str]) {
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
}

/// The loaded commit count from the status bar ("main - 700 commits - clean",
/// or "500 / 2500 commits" while more history exists).
fn loaded_count(screen: &Screen) -> Option<usize> {
    let text = screen.text();
    let end = text.find(" commits \u{2022}")?;
    let segment = text[..end].rsplit('\u{2022}').next()?;
    segment.split(" / ").next()?.trim().parse().ok()
}

/// Whether `text` is drawn in the Log table (between the sidebar and the
/// commit-details pane, which still shows the selected commit).
fn in_log(screen: &Screen, text: &str) -> bool {
    log_hits(screen, text) > 0
}

fn log_hits(screen: &Screen, text: &str) -> usize {
    screen
        .find_all(text)
        .into_iter()
        .filter(|(col, _)| (SIDEBAR_RIGHT..130).contains(col))
        .count()
}

/// A repository of its own: `origin work 1..=3` on main.
fn origin() -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for n in 1..=3 {
        b = b.commit(&format!("origin work {n}")).unwrap();
    }
    b.build()
}

/// An unrelated history with two branches, as `archive commit N`.
fn archive() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("archive commit 1")
        .unwrap()
        .commit("archive commit 2")
        .unwrap()
        .branch("old-a")
        .unwrap()
        .commit("archive commit 3")
        .unwrap()
        .checkout("main")
        .unwrap()
        .branch("old-b")
        .unwrap()
        .commit("archive commit 4")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build()
}

fn add_remote(repo: &Path, name: &str, source: &Path) {
    git(repo, &["remote", "add", name, &source.to_string_lossy()]);
    git(repo, &["fetch", "-q", name]);
}

#[test]
fn hiding_the_archive_branches_removes_their_commits_and_showing_brings_them_back() {
    let repo = origin();
    let archive = archive();
    add_remote(repo.root(), "archive", archive.root());
    let config = tempfile::tempdir().unwrap();
    let session = "hidden-history-archive";
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .ready_when("both histories in the Log", |s| {
            s.contains("origin work 3") && s.contains("archive commit 4")
        })
        .spawn();
    let run = |args: &[&str]| refs(repo.root(), session, config.path(), args);

    run(&["hide", "archive/main", "archive/old-a", "archive/old-b"]);
    let screen = term
        .wait_for("no archive commit left in the Log", STEP, |s| {
            s.contains("origin work 3") && !s.contains("archive commit")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for("the status line counts main's commits only", STEP, |s| {
        loaded_count(s) == Some(3)
    })
    .unwrap_or_else(|e| panic!("{e}: {:?}", loaded_count(&screen)));

    run(&["hide", "--clear"]);
    term.wait_for("the archive commits back", STEP, |s| {
        s.contains("archive commit 4")
            && s.contains("archive commit 1")
            && loaded_count(s) == Some(7)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn hiding_the_checked_out_branch_keeps_its_commits_and_the_wip_row() {
    let repo = origin();
    let root = repo.root();
    // A branch of its own commit, not checked out.
    git(root, &["checkout", "-q", "-b", "other"]);
    git(
        root,
        &["commit", "-q", "--allow-empty", "-m", "other only commit"],
    );
    git(root, &["checkout", "-q", "main"]);
    git(root, &["checkout", "-q", "-b", "topic"]);
    git(
        root,
        &["commit", "-q", "--allow-empty", "-m", "topic only commit"],
    );
    // A dirty tree: the WIP row sits on HEAD.
    std::fs::write(root.join("dirty.txt"), "d\n").unwrap();
    let config = tempfile::tempdir().unwrap();
    let session = "hidden-history-head";
    let term = common::launch(root)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .ready_when("both branch commits and the WIP row", |s| {
            s.contains("topic only commit")
                && s.contains("other only commit")
                && s.contains("// WIP")
        })
        .spawn();

    // Hide every branch: no visible ref reaches the topic commit, HEAD
    // alone keeps it (and the working-tree row on it).
    refs(
        root,
        session,
        config.path(),
        &["hide", "topic", "main", "other"],
    );
    term.wait_for("only HEAD's history and the WIP row", STEP, |s| {
        s.contains("topic only commit")
            && s.contains("// WIP")
            && s.contains("origin work 3")
            && !s.contains("other only commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// 1200 commits `archive-0001..` on main, timestamps in 2020 (older than any
/// commit of the other repository).
fn big_archive() -> tempfile::TempDir {
    const COMMITS: usize = 1200;
    let dir = tempfile::tempdir().expect("temp repo");
    git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for n in 1..=COMMITS {
        let msg = format!("archive-{n:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{n}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + n * 60,
            msg.len()
        ));
        if n > 1 {
            stream.push_str(&format!("from :{}\n", n - 1));
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{n}\n\n",
            n.to_string().len()
        ));
    }
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn fast-import");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    dir
}

#[test]
fn hiding_after_paging_deep_restarts_the_paging_and_showing_reaches_the_archive_root() {
    let repo = origin();
    let archive = big_archive();
    add_remote(repo.root(), "archive", archive.path());
    let config = tempfile::tempdir().unwrap();
    // The smallest initial load and page the preferences allow.
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ninitial_commits_in_graph = 500\ncommits_per_lazy_page = 50\n",
    )
    .unwrap();
    let session = "hidden-history-paging";
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .startup(STARTUP)
        .ready_when("origin and archive commits", |s| {
            s.contains("origin work 3") && s.contains("archive-1200")
        })
        .spawn();
    let (col, row) = term
        .wait_for_text("\u{256e}\u{256d}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 3, row + 4).unwrap();

    // Page deep into the archive.
    let deadline = std::time::Instant::now() + STEP;
    loop {
        term.send_keys("End").unwrap();
        if loaded_count(&term.screen()).is_some_and(|n| n >= 700) {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "never paged to 700");
    }

    refs(
        repo.root(),
        session,
        config.path(),
        &["hide", "archive/main"],
    );
    term.wait_for("only main's commits, nothing loading", STEP, |s| {
        loaded_count(s) == Some(3) && !s.contains("archive-")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        in_log(&term.screen(), "origin work 1"),
        "main's root is shown"
    );

    refs(repo.root(), session, config.path(), &["hide", "--clear"]);
    // End pages through the whole archive again, to its root, once.
    let deadline = std::time::Instant::now() + STEP * 2;
    loop {
        term.send_keys("End").unwrap();
        let screen = term.screen();
        if loaded_count(&screen) == Some(1203) && in_log(&screen, "archive-0001") {
            assert_eq!(log_hits(&screen, "archive-0001"), 1, "no duplicate rows");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the archive root was never reached: {:?}",
            loaded_count(&screen)
        );
    }
    term.stop().unwrap();
}
