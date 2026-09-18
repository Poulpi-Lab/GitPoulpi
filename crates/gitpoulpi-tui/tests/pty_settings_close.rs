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
    let (c, r) = term.screen().find("+ Add").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the add menu", STEP, |s| s.contains("Finish rule"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .screen()
        .find_all("Topic type")
        .into_iter()
        .find(|&(_, row)| row > 10)
        .expect("the Topic type entry");
    term.click(c + 1, r).unwrap();
    term.wait_for("the dirty mark", STEP, |s| s.contains("Unsaved"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

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
    let screen = term
        .wait_for("the question again", STEP, |s| {
            s.contains("Close Settings?")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen.find("Discard and close").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("Settings closed", STEP, |s| {
        !s.contains("Repo-specific settings") && !s.contains("Close Settings?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
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
    let (c, r) = term.screen().find("+ Add").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("a profile added", STEP, |_| {
        std::fs::read_to_string(config.path().join("config.toml"))
            .unwrap_or_default()
            .contains("id = \"profile\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

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
    let (c, r) = term.screen().find("Search settings...").unwrap();
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the search popup", STEP, |s| s.contains("Enter: go"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, row) = screen.find("Enter: go").unwrap();
    let line = screen.line(row);
    assert!(!line.contains("Esc"), "no Esc text on the title: {line}");
    let (c, r) = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("a cross on the popup's title row:\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    term.wait_for("the popup closed", STEP, |s| !s.contains("Enter: go"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        term.screen().contains("Repo-specific settings"),
        "still in Settings"
    );
}
