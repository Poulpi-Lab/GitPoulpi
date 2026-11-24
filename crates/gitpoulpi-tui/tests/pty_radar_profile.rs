#![cfg(feature = "radar")]
//! The Radar's Profile filter (`Alt+R`, then `P`): the real binary and
//! daemon, driven through a pty by `xtask::pty`, against `xtask mock-host`
//! (a `TcpListener` on 127.0.0.1; no other network).
//!
//! Two profiles on the mock: `work-hub` has its account-wide lists on and so
//! contributes rows; `side-hub` has them off and contributes none. Choosing a
//! profile by key and by click shows only its rows.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    _mock: MockHost,
    _repo: FixtureRepo,
}

fn setup(session: &str) -> Setup {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "profile-start-commit")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{}/lumen-labs/lumen-core.git", mock.port),
    ]);
    let config = tempfile::tempdir().expect("temp config dir");
    let profile = |id: &str, radar: &str| {
        format!(
            "[[integrations]]\nid = \"{id}\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:{id}\"\nname = \"{id}\"\nuser = {{ login = \"{login}\" }}\n{radar}\n",
            port = mock.port,
            login = mock_host::GITHUB_LOGIN,
        )
    };
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "{}{}",
            profile(
                "work-hub",
                "[integrations.radar]\nreview_requests = true\nopen_requests = true\n"
            ),
            profile("side-hub", "")
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
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_WORK_HUB", "test-token")
        .env("GITPOULPI_TOKEN_SIDE_HUB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("profile-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        config,
        _mock: mock,
        _repo: repo,
    }
}

fn rows_ready(s: &Screen) -> bool {
    s.contains("Harden the token build") && s.contains("Lower the render budget")
}

/// The first screen line holding `needle` below the combobox row (the popup
/// row of a choice, never the hint row further down the list).
fn popup_choice(screen: &Screen, needle: &str) -> (u16, u16) {
    let (_, field_row) = screen.find("Profile").expect("the Profile combobox");
    screen
        .find_all(needle)
        .into_iter()
        .filter(|(_, row)| *row > field_row && *row <= field_row + 5)
        .min_by_key(|(_, row)| *row)
        .unwrap_or_else(|| panic!("no popup row {needle}: {}", screen.text()))
}

#[test]
fn p_and_a_click_choose_a_profile_and_only_its_rows_show() {
    let mut s = setup("pty-radar-profile");
    s.term.send_keys("M-r").expect("Alt+R");
    s.term
        .wait_for("the work-hub rows", STARTUP, |s| {
            s.contains("Radar") && s.contains("Profile") && rows_ready(s)
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // By key: P opens the combobox, two Downs reach `side-hub`, Enter picks.
    s.term.send_text("P").expect("P");
    s.term
        .wait_for("the popup", STEP, |s| {
            s.contains("All profiles") && s.contains("work-hub")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Down").expect("Down");
    s.term.send_keys("Down").expect("Down");
    s.term.send_keys("Enter").expect("Enter");
    let screen = s
        .term
        .wait_for("no row of side-hub", STEP, |s| {
            !s.contains("Harden the token build") && !s.contains("Lower the render budget")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("All profiles"), "{}", screen.text());
    let saved = std::fs::read_to_string(s.config.path().join("radar.toml")).unwrap_or_default();
    assert!(saved.contains("profile = \"side-hub\""), "{saved}");

    // By click: open the field, click `work-hub`: its rows are back.
    let (col, row) = screen.find("side-hub").expect("the combobox field");
    s.term.click(col + 1, row).expect("open the combobox");
    let popup = s
        .term
        .wait_for("the popup", STEP, |s| s.contains("All profiles"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = popup_choice(&popup, "work-hub");
    s.term.click(col + 1, row).expect("choose work-hub");
    s.term
        .wait_for("the work-hub rows are back", STEP, rows_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let saved = std::fs::read_to_string(s.config.path().join("radar.toml")).unwrap_or_default();
    assert!(saved.contains("profile = \"work-hub\""), "{saved}");
}
