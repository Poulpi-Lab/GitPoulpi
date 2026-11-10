//! Settings > Workflow on the master-detail layout, through a pty: add a topic
//! type from the list's `+ Add` menu, reorder it with Alt+Up, press `Only for
//! me` on the pinned save bar, and the repository state follows (the shared
//! file stays untouched). Temp config dir only.

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

[[topic]]
id = "hotfix"
label = "Hotfix"
pattern = "hotfix/{version}"
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

fn launch(repo: &Path, config: &Path) -> Terminal {
    common::launch(repo)
        .session("pty-settings-workflow")
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn()
}

/// The list rows showing `text` (the list pane is the left of the body).
fn list_rows(term: &Terminal, text: &str) -> Vec<(u16, u16)> {
    term.screen()
        .find_all(text)
        .into_iter()
        .filter(|(c, _)| (28..66).contains(c))
        .collect()
}

#[test]
fn a_topic_type_is_added_reordered_and_saved_only_for_me() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path());
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Workflow");
    common::settings_tab(&mut term, "Rules");
    term.wait_for("the grouped list and the save bar", STEP, |s| {
        s.contains("Topic types") && s.contains("Hotfix") && s.contains("No changes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // + Add > Topic type.
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    let menu = term
        .wait_for("the add menu", STEP, |s| {
            s.contains("Finish rule")
                && s.find_all("Topic type")
                    .into_iter()
                    .any(|(_, row)| row > 10)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = menu
        .find_all("Topic type")
        .into_iter()
        .find(|&(_, row)| row > 10)
        .expect("the Topic type entry");
    term.click(c + 1, r).unwrap();
    term.wait_for("the new topic type and the dirty mark", STEP, |s| {
        s.contains("Unsaved") && s.contains("topic/{slug}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Reorder: the new one goes up above Hotfix.
    let y_of = |t: &Terminal, text: &str| list_rows(t, text).first().map(|&(_, r)| r);
    assert!(
        y_of(&term, "topic").unwrap() > y_of(&term, "Hotfix").unwrap(),
        "added last:\n{}",
        term.text()
    );
    term.send_keys("M-Up").unwrap();
    term.wait_for(
        "moved above Hotfix",
        STEP,
        |_| matches!((y_of(&term, "topic"), y_of(&term, "Hotfix")), (Some(a), Some(b)) if a < b),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The save bar is pinned: Only for me.
    let (c, r) = term
        .wait_for_text("Only for me", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    let state = repo.state_dir().join("repo-state.toml");
    term.wait_for("repo-state follows", STEP, |_| {
        std::fs::read_to_string(&state)
            .unwrap_or_default()
            .contains("topic/{slug}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for("the bar is clean again", STEP, |s| s.contains("No changes"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read_to_string(repo.state_dir().join("workflow.toml")).unwrap(),
        WORKFLOW,
        "the shared file is untouched"
    );
}
