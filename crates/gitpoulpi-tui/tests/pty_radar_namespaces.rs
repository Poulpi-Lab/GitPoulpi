#![cfg(feature = "radar")]
//! The Namespaces field of a code-host profile, end to end: two lines and
//! the "Regular expressions" checkbox typed in Settings > Integrations are
//! written to `config.toml`, and the Radar then lists only the repositories
//! they match -- the real binary and daemon through a pty, against
//! `xtask mock-host` (127.0.0.1 only; it answers the review list with
//! requests of `harbor-guild`, `lumen-labs` and `orion-labs`).

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::mock_host::{self, Options};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

#[test]
fn two_regex_lines_saved_in_settings_filter_the_radar() {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "namespaces-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nreview_requests = true\n",
            port = mock.port,
            login = mock_host::GITHUB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join("pty-radar-namespaces.toml"),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let mut term = common::launch(repo.root())
        .session("pty-radar-namespaces")
        .config_dir(config.path())
        .size(160, 60)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_HUB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("namespaces-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Settings > Integrations, on the Radar block of the profile.
    term.send_keys("C-p").unwrap();
    term.send_text("Radar Options of a Profile").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Radar block", STEP, |s| {
        s.contains("[x] Requests waiting for my review") && s.contains("[ ] Regular expressions")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Down to the Namespaces area (open requests, drafts, issues, area), two
    // lines, then Tab to the checkbox.
    for _ in 0..4 {
        term.send_keys("Down").unwrap();
    }
    term.send_text("^lumen-labs/").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("telemetry$").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Space").unwrap();
    let screen = term
        .wait_for("the checkbox ticked", STEP, |s| {
            s.contains("[x] Regular expressions")
                && s.contains("^lumen-labs/")
                && s.contains("telemetry$")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!screen.contains("Invalid expression"), "{}", screen.text());
    let file = config.path().join("config.toml");
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        if text.contains("namespaces_regex = true")
            && text.contains("^lumen-labs/")
            && text.contains("telemetry$")
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "config.toml never held the namespaces:\n{text}"
        );
        std::thread::yield_now();
    }

    // Back out of Settings (one Escape steps one zone back, then closes),
    // into the Radar.
    common::retry_until(
        &mut term,
        "Settings closed",
        10,
        Duration::from_secs(5),
        |t| t.send_keys("Escape").unwrap(),
        |s| !s.contains("Repo-specific settings"),
    );
    term.send_keys("M-r").expect("Alt+R");
    term.wait_for("only the matching rows", STARTUP, |s| {
        s.contains("Lower the render budget")
            && s.contains("Batch the metric flushes")
            && !s.contains("Harden the token build")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
