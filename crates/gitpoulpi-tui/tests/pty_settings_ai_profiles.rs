#![cfg(feature = "ai")]
//! Settings > AI on the master-detail layout, end to end through a pty:
//! duplicate a profile from the list, then remove its environment variables
//! by their `[x]` buttons (one with the keyboard, one with the mouse).
//! `config.toml` follows every step. Temp config dir only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

const CONFIG: &str = r#"
[ai]
enabled = true
default_profile = "alpha"

[[ai.profiles]]
id = "alpha"
name = "Alpha"
kind = "local_agent"
command = "claude"
args = ["-p"]

[ai.profiles.env]
MY_FLAG = "on"
OTHER_FLAG = "1"
"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
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

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn wait_stored(term: &Terminal, config: &Path, what: &str, ok: impl Fn(&str) -> bool) {
    term.wait_for(what, STEP, |_| ok(&stored(config)))
        .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config), term.text()));
}

#[test]
fn a_profile_is_duplicated_and_its_env_variables_removed_by_their_buttons() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), CONFIG).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ai-profiles-list");
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    common::settings_tab(&mut term, "Profiles (connections)");
    term.wait_for("the list and the env table", STEP, |s| {
        s.contains("+ Add") && s.contains("Env MY_FLAG") && s.contains("Env OTHER_FLAG")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Duplicate the profile with the list's toolbar button.
    let (c, r) = term
        .wait_for_text("Duplicate", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    wait_stored(&term, config.path(), "a copy stored", |t| {
        t.matches("[[ai.profiles]]").count() == 2
            && t.contains("name = \"Alpha (copy)\"")
            && t.matches("MY_FLAG").count() == 2
    });
    term.wait_for("the copy selected", STEP, |s| {
        s.contains("Alpha (copy)") && s.contains("Env MY_FLAG")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Keyboard: select the MY_FLAG row, Right to its remove button, Enter.
    let (c, r) = term
        .wait_for_text("Env MY_FLAG", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(&term, config.path(), "MY_FLAG removed from the copy", |t| {
        t.matches("MY_FLAG").count() == 1 && t.matches("OTHER_FLAG").count() == 2
    });
    term.wait_for("the row gone", STEP, |s| !s.contains("Env MY_FLAG"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Mouse: the [x] on the OTHER_FLAG row (not the close cross, which sits
    // on another row).
    let (_, row) = term
        .wait_for_text("Env OTHER_FLAG", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (cross, _) = term
        .screen()
        .find_all("\u{2715}")
        .into_iter()
        .find(|&(_, r)| r == row)
        .expect("a remove button on the OTHER_FLAG row");
    term.click(cross + 1, row).unwrap();
    wait_stored(&term, config.path(), "OTHER_FLAG removed too", |t| {
        t.matches("OTHER_FLAG").count() == 1 && t.matches("MY_FLAG").count() == 1
    });
}
