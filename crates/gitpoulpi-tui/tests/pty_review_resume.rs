#![cfg(all(feature = "radar", feature = "review"))]
//! A review in progress comes back with its tab: the review of `!58` is
//! opened in the repository a clone made, the window leaves for the other
//! tab (the window ends and restarts) and coming back reopens the review on
//! the same page, file and line. A review the reviewer closed, or submitted,
//! is not reopened.
//!
//! The real binary and daemon, driven through a pty by `xtask::pty`, against
//! `xtask mock-host` (a `TcpListener` on 127.0.0.1; no other network).

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const ROW: &str = "Round invoice totals";
const START: &str = "review-resume-start-commit";

struct Setup {
    term: Terminal,
    mock: MockHost,
    config: tempfile::TempDir,
    _clone_root: tempfile::TempDir,
    _bare: tempfile::TempDir,
    _repo: FixtureRepo,
}

/// The review of `!58` open in the tab a clone made (the second tab; the
/// first is a fixture repository), on the Overview.
fn cloned_review(session: &str) -> Setup {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    let billing = bare.path().join("northwind/billing.git");
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&billing)
        .args(["update-ref", "refs/merge-requests/58/head", "main"])
        .output()
        .expect("git");
    assert!(out.status.success(), "{out:?}");
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", START)
        .unwrap()
        .build();
    let clone_root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n\n[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nreview_requests = true\n",
            root = clone_root.path().display(),
            port = mock.port,
            login = mock_host::GITLAB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(160, 50)
        .kitty_answers()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| s.contains(START))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("M-r").expect("Alt+R");
    let screen = term
        .wait_for("the far request", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    term.click(col + 2, row).expect("select the row");
    term.send_keys("e").expect("e");
    term.wait_for("the question", STEP, |s| s.contains("Review request"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("c");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Full path:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("clone");
    term.wait_for("the review in the new tab", STARTUP, |s| {
        s.contains("Review !58") && s.contains("Add moons to the planet table")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        config,
        _clone_root: clone_root,
        _bare: bare,
        _repo: repo,
    }
}

/// Alt+2: the Files page, with the cursor on a thread's comment (a click on
/// the row puts the keyboard on the diff).
fn on_thread_row(s: &mut Setup) {
    s.term.send_keys("M-2").expect("Alt+2");
    let screen = s
        .term
        .wait_for("the diff", STEP, |s| {
            s.contains("src/planets.rs") && s.contains("Why a Vec here")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("Why a Vec here").unwrap();
    s.term.click(col + 2, row).expect("the thread row");
}

/// The palette's "Repository: Next Tab": a review has the keyboard, so the
/// chord does not reach the tab bar.
fn next_tab(s: &mut Setup) {
    s.term.send_keys("C-p").expect("the palette");
    s.term
        .wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("Repository: Next Tab").unwrap();
    s.term
        .wait_for("the command", STEP, |s| s.contains("Repository: Next Tab"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").expect("run it");
}

/// Leaves for the other tab and comes back (the window ends, then restarts
/// twice).
fn away_and_back(s: &mut Setup) {
    next_tab(s);
    s.term
        .wait_for("the first tab", STARTUP, |s| {
            s.contains(START) && !s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    next_tab(s);
    s.term
        .wait_for("the clone's tab", STARTUP, |s| !s.contains(START))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

#[test]
fn coming_back_to_the_tab_reopens_the_review_on_the_same_file_and_line() {
    let mut s = cloned_review("pty-review-resume-back");
    on_thread_row(&mut s);
    away_and_back(&mut s);
    s.term
        .wait_for("the review reopened on the Files page", STARTUP, |s| {
            s.contains("Review !58") && s.contains("src/planets.rs") && s.contains("Why a Vec here")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // The cursor is on the thread again: `r` replies to it (on the first
    // row of the pane it would not).
    s.term.send_keys("r").expect("reply");
    s.term
        .wait_for("the reply box of the thread", STEP, |s| {
            s.contains("replying to tomas")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

#[test]
fn a_review_the_reviewer_closed_is_not_reopened() {
    let mut s = cloned_review("pty-review-resume-closed");
    on_thread_row(&mut s);
    // Leave and come back first: the marker is on disk and the review is open
    // again, so closing it has something to clear.
    away_and_back(&mut s);
    s.term
        .wait_for("the review reopened", STARTUP, |s| {
            s.contains("Review !58") && s.contains("Why a Vec here")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").expect("close the review");
    s.term
        .wait_for("the review closed", STEP, |s| !s.contains("Review !58"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    away_and_back(&mut s);
    std::thread::sleep(Duration::from_secs(3));
    let screen = s.term.screen();
    assert!(
        !screen.contains("Review !58"),
        "not reopened:\n{}",
        screen.text()
    );
}

#[test]
fn a_submitted_review_is_not_reopened() {
    let mut s = cloned_review("pty-review-resume-submitted");
    on_thread_row(&mut s);
    s.term.send_keys("C-p").expect("the palette");
    s.term
        .wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("Review: Finish Review").unwrap();
    s.term
        .wait_for("the command", STEP, |s| {
            s.contains("Review: Finish Review...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").expect("run it");
    s.term
        .wait_for("the finish dialog", STEP, |s| s.contains("Finish review"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("looks good to me").unwrap();
    s.term.send_keys("C-s").expect("submit");
    s.term
        .wait_for("the review reported", STEP, |s| {
            s.contains("review submitted")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    away_and_back(&mut s);
    std::thread::sleep(Duration::from_secs(3));
    let screen = s.term.screen();
    assert!(
        !screen.contains("Review !58"),
        "a submitted review stays closed:\n{}",
        screen.text()
    );
    let _ = &s.config;
}

/// The review over the Radar, through the API (the repository is cloned
/// nowhere): leaving the Home window for a repository tab and coming back to
/// the Radar reopens it on the same page.
#[test]
fn the_api_review_over_the_radar_comes_back_with_the_radar() {
    let session = "pty-review-resume-api";
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", START)
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nreview_requests = true\n",
            port = mock.port,
            login = mock_host::GITLAB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(160, 50)
        .kitty_answers()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    let mut s = Setup {
        term,
        mock,
        config,
        _clone_root: tempfile::tempdir().unwrap(),
        _bare: tempfile::tempdir().unwrap(),
        _repo: repo,
    };
    s.term
        .wait_for("the repository loaded", STARTUP, |s| s.contains(START))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("M-r").expect("Alt+R");
    let screen = s
        .term
        .wait_for("the far request", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| {
            s.contains("Review through the API")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("a").expect("a");
    s.term
        .wait_for("the review over the Radar", STEP, |s| {
            s.contains("Review !58") && s.contains("Add moons to the planet table")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("M-2").expect("Alt+2");
    s.term
        .wait_for("the Files page", STEP, |s| s.contains("src/planets.rs"))
        .unwrap_or_else(|e| panic!("{e}"));
    // To the repository's tab (the Home window ends), and back to the Radar.
    let (col, row) = s
        .term
        .screen()
        .find("repo ")
        .unwrap_or_else(|| panic!("the repository's tab:\n{}", s.term.text()));
    s.term.click(col + 1, row).expect("the repository's tab");
    s.term
        .wait_for("the repository", STARTUP, |s| {
            s.contains(START) && !s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("M-r").expect("Alt+R");
    s.term
        .wait_for("the review back over the Radar, on Files", STARTUP, |s| {
            s.contains("Review !58") && s.contains("src/planets.rs") && s.contains("Files (2)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let _ = &s.config;
}
