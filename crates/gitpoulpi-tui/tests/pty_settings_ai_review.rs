#![cfg(feature = "ai")]
//! Settings > AI, the Code review rows (`[ai.review]`): the global ones write
//! `config.toml`, the repository page's write the repository's own state and
//! show the global value while they inherit. Reached through the Settings
//! search, which finds both. Real binary through a pty; temp dirs only.

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
"#;

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    let mut term = common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn();
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn repo_stored(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

/// Opens the Settings search, types `query` and takes the `nth` result.
fn jump(term: &mut Terminal, query: &str, nth: usize) {
    term.send_keys("/").unwrap();
    term.wait_for("the search", STEP, |s| s.contains("Search"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(query).unwrap();
    term.wait_for("the query echoed", STEP, |s| s.contains(query))
        .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..nth {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_global_rows_write_config_toml() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), CONFIG).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ai-review-global");
    common::settings_page(&mut term, "AI");

    jump(&mut term, "Include AGENTS.md", 0);
    term.wait_for("the row, ticked", STEP, |s| {
        s.contains("Include AGENTS.md") && s.contains("global")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Space").unwrap();
    term.wait_for("config.toml follows", STEP, |_| {
        stored(config.path()).contains("include_agents_md = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));

    jump(&mut term, "Extra file", 0);
    term.send_keys("Enter").unwrap();
    term.send_text("docs/REVIEW.md").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("config.toml has the extra file", STEP, |_| {
        stored(config.path()).contains("extra_file = \"docs/REVIEW.md\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
    assert!(
        repo_stored(&repo).is_empty() || !repo_stored(&repo).contains("ai_review"),
        "the repository kept nothing:\n{}",
        repo_stored(&repo)
    );
}

#[test]
fn the_repository_rows_inherit_and_write_the_repository_state() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("{CONFIG}\n[ai.review]\ninclude_agents_md = false\n"),
    )
    .unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ai-review-repo");
    common::settings_page_nth(&mut term, "AI", 1);
    term.wait_for("the repository page", STEP, |s| {
        s.contains("AI - repo-specific")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The second hit of the search is the repository page's row.
    jump(&mut term, "Include AGENTS.md", 1);
    let screen = term
        .wait_for("the inheriting row shows the global value", STEP, |s| {
            s.contains("Use global (Off)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("repo-specific"), "{}", screen.text());
    // Enter opens the combobox; Down + Enter picks "On".
    term.send_keys("Enter").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the repository state follows", STEP, |_| {
        repo_stored(&repo).contains("include_agents_md = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", repo_stored(&repo), term.text()));
    assert!(
        !stored(config.path()).contains("include_agents_md = true"),
        "config.toml is untouched:\n{}",
        stored(config.path())
    );
}

#[test]
fn the_no_extra_file_switch_is_kept_in_the_repository_state() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("{CONFIG}\n[ai.review]\nextra_file = \"docs/GLOBAL.md\"\n"),
    )
    .unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ai-review-no-extra");
    common::settings_page_nth(&mut term, "AI", 1);
    term.wait_for("the repository page", STEP, |s| {
        s.contains("AI - repo-specific")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Only the repository page has the switch.
    jump(&mut term, "No extra file", 0);
    term.wait_for("the row, unticked", STEP, |s| {
        s.contains("No extra file") && s.contains("Give the model no extra file")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Space").unwrap();
    term.wait_for("the repository state follows", STEP, |_| {
        repo_stored(&repo).contains("no_extra_file = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", repo_stored(&repo), term.text()));
    assert!(
        !repo_stored(&repo).contains("\nextra_file"),
        "an empty field still inherits, the switch is separate:\n{}",
        repo_stored(&repo)
    );
    assert!(
        stored(config.path()).contains("docs/GLOBAL.md")
            && !stored(config.path()).contains("no_extra"),
        "config.toml is untouched:\n{}",
        stored(config.path())
    );
}

#[test]
fn browse_next_to_the_extra_file_fills_it_with_a_path_in_the_repository() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), CONFIG).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-ai-review-browse");
    common::settings_page_nth(&mut term, "AI", 1);
    term.wait_for("the repository page", STEP, |s| {
        s.contains("AI - repo-specific")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    jump(&mut term, "Extra file", 1);
    // The Browse button is the next row; Enter on it opens the picker.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the file picker", STEP, |s| {
        s.contains("Choose the extra file") && s.contains("a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The picker starts at the repository root; a.txt is the only file.
    term.send_keys("End").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the repository state has the path", STEP, |_| {
        repo_stored(&repo).contains("extra_file = \"a.txt\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", repo_stored(&repo), term.text()));
}
