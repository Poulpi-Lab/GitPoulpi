//! Slice S5b end to end: the "Review options" dialog of the ✦ draft. The
//! model is a **capturing fake local agent** that keeps the prompt it was
//! given, so the tests read what the model was told. The host is
//! `xtask mock-host`; nothing leaves the machine.
//!
//! What it pins down: `s` opens the dialog instead of asking; unticking
//! `AGENTS.md` keeps its text out of the prompt while `CLAUDE.md` stays in;
//! "Save as this repository's defaults" lands in `repo-state.toml` and the
//! dialog starts from it next time; a missing extra file is said on the card.

use crate::review::{launch_review_with, open_from_menu};
use crate::support::*;
use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, Options};

const ANSWER: &str = r#"{"summary": "Adds moons to the table.", "suggestions": []}"#;

/// `review_fixture` with a `CLAUDE.md` and an `AGENTS.md` on the target
/// branch.
fn fixture() -> FixtureRepo {
    let base: String = (1..=30).map(|n| format!("// line {n}\n")).collect();
    let moons = base.replace("// line 12\n", "let moons = Vec::new();\nmoons.push(1);\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/planets.rs", &base, "Start the planet table")
        .unwrap()
        .commit_file("CLAUDE.md", "CLAUDE RULE: tests first\n", "Add CLAUDE.md")
        .unwrap()
        .commit_file("AGENTS.md", "AGENTS RULE: no unwrap\n", "Add AGENTS.md")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("src/planets.rs", &moons, "Add moons to the table")
        .unwrap()
        .commit_file("docs/moons.md", "# Moons\ntext\n", "Document the moons")
        .unwrap()
        .push("feature:refs/merge-requests/58/head")
        .unwrap()
        .build();
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"o/orbit\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

fn repo_state(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

fn open_options(term: &mut xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for("the Draft review button", STEP, |s| {
        s.contains("\u{2726} Draft review...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("s").expect("Draft review");
    term.wait_for("the review options", STEP, |s| {
        s.contains("Review options")
            && s.contains("AGENTS.md")
            && s.contains("No extra file")
            && s.contains("Browse...")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn unticking_agents_md_keeps_it_out_of_the_prompt_and_claude_md_in() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &[ANSWER]);
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-options-prompt",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);
    let screen = open_options(&mut term);
    assert!(
        screen.contains("[x] CLAUDE.md") && screen.contains("[x] AGENTS.md"),
        "both are on by default:\n{}",
        screen.text()
    );
    assert_eq!(agent_calls(&agent), 0, "the dialog asks nothing");
    // Run combobox, CLAUDE.md, AGENTS.md: untick it.
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Space").unwrap();
    term.wait_for("AGENTS.md unticked", STEP, |s| s.contains("[ ] AGENTS.md"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the model was asked", STEP, |_| agent_calls(&agent) == 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let prompt = agent_prompt(&agent);
    assert!(
        prompt.contains("CLAUDE RULE: tests first"),
        "CLAUDE.md is given:\n{prompt}"
    );
    assert!(
        !prompt.contains("AGENTS RULE: no unwrap"),
        "AGENTS.md is left out:\n{prompt}"
    );
    assert!(
        !repo_state(&repo).contains("ai_review"),
        "nothing saved unless asked:\n{}",
        repo_state(&repo)
    );
}

#[test]
fn saved_defaults_prefill_the_next_dialog_and_a_missing_extra_file_is_said() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &[ANSWER]);
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-options-save",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);
    open_options(&mut term);
    // CLAUDE.md off, a missing extra file, "Save as defaults".
    term.send_keys("Tab").unwrap();
    term.send_keys("Space").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("docs/NOPE.md").unwrap();
    // Past Browse and No extra file to "Save as defaults".
    for _ in 0..3 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Space").unwrap();
    term.wait_for("the choices shown", STEP, |s| {
        s.contains("[ ] CLAUDE.md") && s.contains("docs/NOPE.md") && s.contains("[x] Save as")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the answer and the problem on the card", STEP, |s| {
            s.contains("Context not given to the model") && s.contains("docs/NOPE.md")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 1, "{}", screen.text());
    let prompt = agent_prompt(&agent);
    assert!(!prompt.contains("CLAUDE RULE"), "{prompt}");
    assert!(prompt.contains("AGENTS RULE: no unwrap"), "{prompt}");
    let state = repo_state(&repo);
    assert!(
        state.contains("include_claude_md = false")
            && state.contains("extra_file = \"docs/NOPE.md\"")
            && !state.contains("include_agents_md"),
        "only what differs from the global values is kept:\n{state}"
    );
    // Opened again, the dialog starts from the saved values.
    term.send_keys("s").unwrap();
    let screen = term
        .wait_for("the options again, from the saved values", STEP, |s| {
            s.contains("Review options")
                && s.contains("[ ] CLAUDE.md")
                && s.contains("[x] AGENTS.md")
                && s.contains("docs/NOPE.md")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = screen;
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Review options") && s.contains("Review !58")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1, "Esc runs nothing");
}

#[test]
fn browse_picks_the_extra_file_and_its_text_reaches_the_prompt() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &[ANSWER]);
    // A file outside the clone: stored absolute and read from the disk.
    let extras = tempfile::tempdir().unwrap();
    std::fs::write(
        extras.path().join("EXTRA_RULES.md"),
        "EXTRA RULE: marker 42\n",
    )
    .unwrap();
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-options-browse",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);
    let screen = open_options(&mut term);
    assert!(screen.contains("[ ] No extra file"), "{}", screen.text());
    // Run, CLAUDE.md, AGENTS.md, the field, then the Browse button.
    for _ in 0..4 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the file picker", STEP, |s| {
        s.contains("Choose the extra file") && s.contains("QUICK ACCESS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Shift+Tab x3 reaches the path field; type the folder, go there.
    for _ in 0..3 {
        term.send_keys("BTab").unwrap();
    }
    term.send_text(&extras.path().display().to_string())
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the folder's file listed", STEP, |s| {
        s.contains("EXTRA_RULES.md")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Path -> filter -> places -> files; the file is the last row.
    for _ in 0..3 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("End").unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the picker closed with the file in the field", STEP, |s| {
            !s.contains("Choose the extra file")
                && s.contains("EXTRA_RULES.md")
                && s.contains("Review options")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = screen;
    term.send_keys("Enter").unwrap();
    term.wait_for("the model was asked", STEP, |_| agent_calls(&agent) == 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let prompt = agent_prompt(&agent);
    assert!(
        prompt.contains("EXTRA RULE: marker 42"),
        "the picked file is given:\n{prompt}"
    );
}

#[test]
fn no_extra_file_keeps_the_global_extra_file_out_of_the_prompt() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &[ANSWER]);
    let extras = tempfile::tempdir().unwrap();
    let rules = extras.path().join("GLOBAL_RULES.md");
    std::fs::write(&rules, "GLOBAL RULE: marker 7\n").unwrap();
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-options-no-extra",
        &format!(
            "\n{}\n[ai.review]\nextra_file = {:?}\n",
            ai_config_toml(&agent),
            rules.display().to_string()
        ),
    );
    open_from_menu(&mut term);
    let screen = open_options(&mut term);
    assert!(
        screen.contains("GLOBAL_RULES.md"),
        "the dialog starts from the global file:\n{}",
        screen.text()
    );
    // Run, CLAUDE.md, AGENTS.md, the field, Browse, "No extra file".
    for _ in 0..5 {
        term.send_keys("Tab").unwrap();
    }
    term.send_keys("Space").unwrap();
    term.wait_for("the switch ticked", STEP, |s| {
        s.contains("[x] No extra file")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Tab").unwrap();
    term.send_keys("Space").unwrap(); // Save as defaults
    term.send_keys("Enter").unwrap();
    term.wait_for("the model was asked", STEP, |_| agent_calls(&agent) == 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let prompt = agent_prompt(&agent);
    assert!(
        !prompt.contains("GLOBAL RULE: marker 7"),
        "the global file is off for this run:\n{prompt}"
    );
    let state = repo_state(&repo);
    assert!(
        state.contains("no_extra_file = true") && !state.contains("\nextra_file"),
        "the switch is the repository's default now:\n{state}"
    );
}
