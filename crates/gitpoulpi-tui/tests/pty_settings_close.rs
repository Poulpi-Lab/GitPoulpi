//! Closing Settings through a pty (X1): an unsaved Workflow draft asks
//! before it is lost (Cancel keeps it, Discard closes), a profile without an
//! email offers "Go to it", and the search popup closes on its cross.
//! Temp config dir only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const WORKFLOW: &str = r#"template = "custom"

[[base]]
name = "main"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "main"
"#;

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(repo.state_dir().join("workflow.toml"), WORKFLOW).unwrap();
    repo
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn()
}

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn an_unsaved_workflow_draft_asks_before_settings_closes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-close-workflow");
    open_settings(&mut term);
    common::settings_page(&mut term, "Workflow");
    common::settings_tab(&mut term, "Rules");
    term.wait_for("the save bar", STEP, |s| s.contains("No changes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // + Add > Topic type makes the draft dirty.
    // Clicks retried, each target located afresh: a click placed on a torn
    // frame can land nowhere.
    common::click_text_until(&mut term, "+ Add", "the add menu", |s| {
        s.contains("Finish rule")
    });
    common::retry_until(
        &mut term,
        "the dirty mark",
        4,
        STEP,
        |t| {
            let s = t.screen();
            if !s.contains("Unsaved") {
                if let Some((c, r)) = s
                    .find_all("Topic type")
                    .into_iter()
                    .find(|&(_, row)| row > 10)
                {
                    t.click(c + 1, r).unwrap();
                }
            }
        },
        |s| s.contains("Unsaved"),
    );

    // Esc asks; Esc again (Cancel) keeps the draft.
    term.send_keys("Escape").unwrap();
    term.wait_for("the question", STEP, |s| {
        s.contains("Close Settings?") && s.contains("Discard and close")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back in Settings, draft kept", STEP, |s| {
        !s.contains("Close Settings?") && s.contains("Unsaved")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // `q` asks again; Discard and close leaves Settings.
    term.send_keys("q").unwrap();
    term.wait_for("the question again", STEP, |s| {
        s.contains("Close Settings?") && s.contains("Discard and close")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::click_text_until(&mut term, "Discard and close", "Settings closed", |s| {
        !s.contains("Repo-specific settings") && !s.contains("Close Settings?")
    });
    assert_eq!(
        std::fs::read_to_string(repo.state_dir().join("workflow.toml")).unwrap(),
        WORKFLOW,
        "nothing was saved"
    );
}

#[test]
fn a_profile_without_an_email_offers_go_to_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-close-profile");
    open_settings(&mut term);
    common::settings_page(&mut term, "Profiles");
    common::settings_tab(&mut term, "Profiles");
    term.wait_for("the list", STEP, |s| s.contains("+ Add"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let added = || {
        std::fs::read_to_string(config.path().join("config.toml"))
            .unwrap_or_default()
            .contains("id = \"profile\"")
    };
    // The click is retried (located afresh) until the profile is stored.
    common::retry_until(
        &mut term,
        "a profile added",
        4,
        STEP,
        |t| {
            if !added() {
                if let Some((c, r)) = t.screen().find("+ Add") {
                    t.click(c + 1, r).unwrap();
                }
            }
        },
        |_| added(),
    );

    // Move to another page so Go to it has somewhere to come back from.
    common::settings_page(&mut term, "General");
    term.send_keys("Escape").unwrap();
    term.wait_for("the question", STEP, |s| {
        s.contains("Close Settings?") && s.contains("Close anyway") && s.contains("Go to it")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Go to it is the focused button.
    term.send_keys("Enter").unwrap();
    term.wait_for("the profile page", STEP, |s| {
        !s.contains("Close Settings?") && s.contains("Profiles - global") && s.contains("No email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_search_popup_closes_on_its_cross() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-close-search");
    open_settings(&mut term);
    // The popup with its title row's cross, as one state.
    let with_cross = |s: &xtask::pty::Screen| {
        s.find("Enter: go")
            .is_some_and(|(_, row)| s.find_all("\u{2715}").into_iter().any(|(_, r)| r == row))
    };
    let screen =
        common::click_text_until(&mut term, "Search settings...", "the search popup", |s| {
            with_cross(s)
        });
    let (_, row) = screen.find("Enter: go").unwrap();
    let line = screen.line(row);
    assert!(!line.contains("Esc"), "no Esc text on the title: {line}");
    let (c, r) = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("a cross on the popup's title row:\n{}", term.text()));
    common::retry_until(
        &mut term,
        "the popup closed",
        4,
        STEP,
        |t| {
            if t.screen().contains("Enter: go") {
                t.click(c + 1, r).unwrap();
            }
        },
        |s| !s.contains("Enter: go"),
    );
    assert!(
        term.screen().contains("Repo-specific settings"),
        "still in Settings"
    );
}
