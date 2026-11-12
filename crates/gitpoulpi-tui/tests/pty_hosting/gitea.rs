//! Gitea / Forgejo in the sidebar (integrations wave 2, slice G), against the
//! shared `xtask mock-host`'s Gitea flavour: the PULL REQUESTS and ISSUES
//! sections list the linked project, the checked-out branch gets its `GT #12`
//! chip, and the row menu offers the metadata entries (Gitea is a
//! `meta_picker::supports` provider).

use crate::support::*;
use xtask::mock_host::{self, Options};

fn config(port: u16) -> String {
    format!(
        "[general]\nhosting_refresh_minutes = 0\n\n[ui]\npr_terminology = \"pull_request\"\n\n[[integrations]]\nid = \"forge\"\nkind = \"gitea\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:forge\"\nname = \"forge\"\nuser = {{ login = \"mara-quinn\" }}\n"
    )
}

fn linked_fixture(port: u16) -> gitpoulpi_testkit::FixtureRepo {
    let repo = fixture(port);
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://127.0.0.1:{port}/o/orbit.git"),
    ]);
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"forge\"\nproject = \"o/orbit\"\n\n[issues_source]\nintegration = \"forge\"\nproject = \"o/orbit\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

fn launch_forge(
    repo: &gitpoulpi_testkit::FixtureRepo,
    session: &str,
    port: u16,
) -> (xtask::pty::Terminal, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(dir.path().join("config.toml"), config(port)).expect("seed config.toml");
    let term = crate::common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_FORGE", "test-token")
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("Add the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, dir)
}

#[test]
fn a_gitea_project_lists_requests_and_issues_with_its_chip_and_metadata_menu() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = linked_fixture(mock.port);
    let (mut term, _dir) = launch_forge(&repo, "pty-hosting-gitea", mock.port);

    // Both sections read the project through `/api/v1`, with the token scheme.
    let screen = term
        .wait_for("the requests and the issues listed", STARTUP, |s| {
            s.contains("Greet by name")
                && s.contains("WIP: rework the")
                && s.contains("Crash on empty name")
                && s.contains("GT #12")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert!(screen.contains("GT #12"), "the chip:\n{}", screen.text());
    assert!(
        mock.requests()
            .iter()
            .any(|r| r.target.contains("/api/v1/repos/o/orbit/pulls")
                && r.headers
                    .to_lowercase()
                    .contains("authorization: token test-token")),
        "{:?}",
        mock.requests()
            .iter()
            .map(|r| &r.target)
            .collect::<Vec<_>>()
    );

    // The row menu of a request offers the metadata entries.
    let _ = screen;
    crate::common::right_click_text_until(
        &mut term,
        "Greet by name",
        "the metadata entries",
        |s| {
            s.contains("Edit assignees...")
                && s.contains("Edit reviewers...")
                && s.contains("Edit labels...")
                && s.contains("Set milestone...")
        },
    );
    term.stop().ok();
}

/// The Radar (Alt+R) reads a Gitea project like a GitHub one: the request
/// the user was asked to review is under NEEDS MY REVIEW with a failed-checks
/// cross and the conflict mark, the one that mentions them (and the issue that
/// does) under MENTIONING ME.
#[test]
fn the_radar_reads_a_gitea_project_with_checks_conflicts_and_mentions() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = linked_fixture(mock.port);
    let (mut term, _dir) = launch_forge(&repo, "pty-radar-gitea", mock.port);
    term.send_keys("M-r").expect("Alt+R");
    let screen = term
        .wait_for("the Gitea rows in the Radar", STARTUP, |s| {
            s.contains("Radar")
                && s.contains("NEEDS MY REVIEW")
                && s.contains("MENTIONING ME")
                && s.contains("Greet by name")
                // The last row drawn: a snapshot can catch the frame half
                // written ("● re"), so every row asserted below is awaited.
                && s.contains("WIP: rework the log")
                && s.contains("Greeting should be configurable")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let (_, row) = screen.find("Greet by name").expect("request 12");
    let line = screen.line(row);
    assert!(
        line.contains("\u{2717}!"),
        "failed checks + conflict: {line}"
    );
    // The passing request carries a tick and no conflict mark.
    let (_, ok_row) = screen.find("WIP: rework the log").expect("request 13");
    assert!(
        screen.line(ok_row).contains("\u{2713} "),
        "{}",
        screen.text()
    );
    assert!(
        screen.contains("Greeting should be configurable"),
        "{}",
        screen.text()
    );
    term.stop().ok();
}
