use std::time::Duration;

use crate::common;
use crate::mock::*;
use crate::support::*;

/// The ISSUES section: only supported providers are offered, the repo
/// combobox lists the provider's projects, and the choice is remembered
/// for this repository.
#[test]
fn the_issues_section_picks_a_provider_and_a_repository_and_remembers_them() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    // A Bitbucket profile must never be offered.
    let extra = "\n[[integrations]]\nid = \"bb\"\nkind = \"bitbucket\"\nurl = \"https://bitbucket.org\"\ntoken_ref = \"integration:bb\"\nname = \"bb\"\n";
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-issues",
        &config_toml(mock.port, extra),
        COLS,
        "Add the feature",
    );
    // Linked to g/p to start with: GitLab issues of g/p.
    term.wait_for("the ISSUES section", STARTUP, |s| {
        s.contains("GITLAB ISSUES") && s.contains("Crash on startup") && s.contains("g/p")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The provider dropdown: GitLab only.
    let screen = term.screen();
    let (col, row) = screen.find("GitLab \u{00B7} corp").expect("provider row");
    term.click(col + 2, row)
        .expect("open the provider dropdown");
    let opened = term
        .wait_for("the provider dropdown", STEP, |s| {
            s.find_all("GitLab \u{00B7} corp").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !opened.contains("bb") || !opened.contains("Bitbucket"),
        "unsupported providers are not offered: {}",
        opened.text()
    );
    assert!(!opened.contains("Bitbucket"), "{}", opened.text());
    term.send_keys("Escape").unwrap();

    // The repository dropdown lists the account's projects; pick one.
    let screen = term
        .wait_for("the dropdown closed", STEP, |s| {
            s.find_all("GitLab \u{00B7} corp").len() == 1
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, provider_row) = screen.find("GitLab \u{00B7} corp").unwrap();
    let repo_row = provider_row + 1;
    let (col, _) = screen.find("GITLAB ISSUES").unwrap();
    term.click(col + 4, repo_row)
        .expect("open the repository dropdown");
    term.wait_for("the projects listed", STEP, |s| {
        s.contains("team/tool") && s.contains("g/p")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let (pc, pr) = screen
        .find_all("team/tool")
        .into_iter()
        .find(|(_, r)| *r > repo_row)
        .expect("team/tool in the popup");
    term.double_click(pc + 2, pr).expect("pick team/tool");
    term.wait_for("the section follows the choice", STEP, |s| {
        s.line(repo_row).contains("team/tool")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The host was asked for that project's issues.
    let deadline = std::time::Instant::now() + STEP;
    while !mock.saw("GET", "team%2Ftool/issues") {
        assert!(
            std::time::Instant::now() < deadline,
            "{:?}",
            mock.requests()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // Remembered for this repository (per-repo state, not the config).
    let state = std::fs::read_to_string(repo.state_dir().join("repo-state.toml"))
        .expect("repo-state.toml written");
    assert!(state.contains("[issues_source]"), "{state}");
    assert!(state.contains("integration = \"corp\""), "{state}");
    assert!(state.contains("project = \"team/tool\""), "{state}");
    term.stop().ok();

    // A new window on the same repository starts from the saved choice.
    let (term2, _config2) = launch_unlinked(
        &repo,
        "pty-hosting-issues-2",
        &config_toml(mock.port, extra),
        COLS,
        "Add the feature",
    );
    term2
        .wait_for("the remembered repository", STARTUP, |s| {
            s.contains("GITLAB ISSUES") && s.contains("team/tool")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// "None" in the provider dropdown clears `[issues_source]` for the
/// repository, and the cleared state survives a restart.
#[test]
fn unlinking_the_issue_source_clears_it_and_survives_a_restart() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let config = config_toml(mock.port, "");
    let state_path = repo.state_dir().join("repo-state.toml");
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-hosting-unlink",
        &config,
        COLS,
        "Add the feature",
    );
    let repo_row = open_repo_dropdown(&mut term);
    term.wait_for("the projects listed", STEP, |s| s.contains("team/tool"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (pc, pr) = term
        .screen()
        .find_all("team/tool")
        .into_iter()
        .find(|(_, r)| *r > repo_row)
        .expect("team/tool in the popup");
    term.double_click(pc + 2, pr).expect("pick team/tool");
    term.wait_for("team/tool chosen", STEP, |s| {
        s.line(repo_row).contains("team/tool")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let saved = std::fs::read_to_string(&state_path).expect("state written");
    assert!(saved.contains("[issues_source]"), "{saved}");

    // The provider dropdown now offers None; pick it.
    let (col, provider_row) = term.screen().find("GitLab \u{00B7} corp").unwrap();
    term.click(col + 2, provider_row)
        .expect("open the provider");
    let opened = term
        .wait_for("the None row", STEP, |s| s.contains("None (unlink"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (nc, nr) = opened.find("None (unlink").unwrap();
    term.click(nc + 2, nr).expect("pick None");
    term.wait_for("the source cleared", STEP, |s| {
        !s.contains("team/tool") && !s.contains("None (unlink")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let saved = std::fs::read_to_string(&state_path).unwrap_or_default();
    assert!(!saved.contains("[issues_source]"), "{saved}");
    term.stop().ok();

    // A new window: still cleared -- the remote only ever proposes.
    let (term2, _config2) = launch_unlinked(
        &repo,
        "pty-hosting-unlink-2",
        &config,
        COLS,
        "Add the feature",
    );
    let screen = term2
        .wait_for("the section still unlinked", STARTUP, |s| {
            s.find_all("Select provider...").len() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("team/tool") && !screen.contains("GITLAB ISSUES"),
        "{}",
        screen.text()
    );
    let saved = std::fs::read_to_string(&state_path).unwrap_or_default();
    assert!(!saved.contains("[issues_source]"), "{saved}");
}

/// A section's provider: a fresh repository is
/// unlinked and fetches nothing, the combo only *proposes* the provider
/// matching the remote, a pick is persisted, and an unlink is persisted
/// too - nothing ever links it again on its own.
fn provider_semantics(title: &str, key: &str, session: &str) {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let config = config_toml(mock.port, "");
    let (mut term, _config) = launch_unlinked(
        &repo,
        &format!("{session}-1"),
        &config,
        COLS,
        "Add the feature",
    );
    // Fresh: both sections are just their dropdowns, saying Select....
    let screen = term
        .wait_for("both sections unlinked", STARTUP, |s| {
            s.find_all("Select provider...").len() >= 2 && s.contains("Select repository...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("GITLAB ISSUES") && !screen.contains("!53"),
        "{}",
        screen.text()
    );
    // Give a (wrongly) automatic link time to show itself.
    term.wait_for_stable(Duration::from_millis(2500), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(host_list_requests(&mock), 0, "{:?}", mock.requests());
    assert_eq!(repo_state(&repo), "", "nothing was written");

    // The dropdown proposes the remote's provider and repository.
    let (_, header) = section_rows(&term.screen(), title).expect("the section header");
    let provider_row = header + 1;
    let proposals = |s: &xtask::pty::Screen| {
        s.find_all("origin")
            .iter()
            .any(|&(_, r)| r > provider_row && s.line(r).contains("g/p"))
    };
    // A click on a torn frame can be lost: retry until the dropdown shows.
    let opened = common::retry_until(
        &mut term,
        "the proposals",
        6,
        Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if proposals(&screen) {
                return;
            }
            if let Some((col, _)) = section_rows(&screen, title) {
                t.click(col + 4, provider_row).expect("open the provider");
            }
        },
        proposals,
    );
    assert_eq!(
        host_list_requests(&mock),
        0,
        "listing the proposals fetches nothing"
    );
    assert_eq!(repo_state(&repo), "", "listing the proposals links nothing");
    let _ = opened;
    common::retry_until(
        &mut term,
        "the pick persisted",
        6,
        Duration::from_secs(5),
        |t| {
            if repo_state(&repo).contains(&format!("[{key}]")) {
                return;
            }
            let screen = t.screen();
            if let Some((pc, pr)) = screen
                .find_all("origin")
                .into_iter()
                .find(|&(_, r)| r > provider_row && screen.line(r).contains("g/p"))
            {
                t.click(pc, pr).expect("pick the proposal");
            }
        },
        |_| repo_state(&repo).contains(&format!("[{key}]")),
    );
    let state = repo_state(&repo);
    assert!(
        state.contains("integration = \"corp\"") && state.contains("project = \"g/p\""),
        "{state}"
    );
    term.wait_for("the section linked", STEP, |s| {
        s.contains("GitLab \u{00B7} corp") && s.line(provider_row + 1).contains("g/p")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    while host_list_requests(&mock) == 0 {
        assert!(
            std::time::Instant::now() < deadline,
            "{:?}",
            mock.requests()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    term.stop().ok();

    // Restart: the pick is remembered.
    let (mut term, _config) = launch_unlinked(
        &repo,
        &format!("{session}-2"),
        &config,
        COLS,
        "Add the feature",
    );
    term.wait_for("the remembered link", STARTUP, |s| {
        s.contains("GitLab \u{00B7} corp")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(repo_state(&repo).contains(&format!("[{key}]")));

    // Unlink: the first row of the dropdown.
    common::retry_until(
        &mut term,
        "the unlink persisted",
        8,
        Duration::from_secs(5),
        |t| {
            if !repo_state(&repo).contains(&format!("[{key}]")) {
                return;
            }
            let screen = t.screen();
            if let Some((nc, nr)) = screen.find("None (unlink") {
                t.click(nc + 2, nr).expect("pick None");
            } else if let Some((col, header)) = section_rows(&screen, title) {
                t.click(col + 4, header + 1).expect("open the provider");
            }
        },
        |_| !repo_state(&repo).contains(&format!("[{key}]")),
    );
    let deadline = std::time::Instant::now() + STEP;
    while repo_state(&repo).contains(&format!("[{key}]")) {
        assert!(
            std::time::Instant::now() < deadline,
            "the unlink never persisted"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    term.stop().ok();
    let before = host_list_requests(&mock);

    // Restart again: still unlinked, however well the remote matches.
    let (term, _config) = launch_unlinked(
        &repo,
        &format!("{session}-3"),
        &config,
        COLS,
        "Add the feature",
    );
    term.wait_for("unlinked after the restart", STARTUP, |s| {
        s.find_all("Select provider...").len() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_stable(Duration::from_millis(2500), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!repo_state(&repo).contains(&format!("[{key}]")));
    assert_eq!(
        host_list_requests(&mock),
        before,
        "nothing was fetched again"
    );
}

#[test]
fn the_pull_requests_section_starts_unlinked_proposes_and_keeps_pick_and_unlink() {
    provider_semantics("PULL REQUESTS", "pulls_source", "pty-hosting-pr-link");
}

#[test]
fn the_issues_section_starts_unlinked_proposes_and_keeps_pick_and_unlink() {
    provider_semantics("ISSUES", "issues_source", "pty-hosting-issue-link");
}
