#![cfg(all(feature = "ai", unix))]
//! The input caps of `[ai.limits]` besides the conflict's: a diff over
//! `diff_bytes` is never cut silently. End to end through a pty: the real
//! binary and daemon, a real repository with a staged diff of about 90 KB,
//! and a **capturing fake agent** standing in for the model (no socket is
//! opened). "Over the AI limit" offers Send all / Send truncated / Open
//! setting / Cancel; what the agent receives is read back from its capture.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

/// One commit and one staged file of 3000 changed lines (about 90 KB of
/// diff, over the 60000 default), a token on its first line.
fn fixture() -> FixtureRepo {
    let big: String = std::iter::once(format!("token = ghp_{BODY}\n"))
        .chain((0..3_000).map(|i| format!("let value_{i} = compute({i});\n")))
        .collect();
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("big.rs", big.as_bytes())
        .unwrap()
        .stage("big.rs")
        .unwrap()
        .build()
}

fn launch(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    preview: bool,
) -> (Terminal, tempfile::TempDir) {
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 50)
        .ready_when("the fixture loaded, WIP row and all", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        });
    if preview {
        launch = launch.env("GITPOULPI_AI_SENT_PREVIEW_COMMIT_MESSAGE", "true");
    }
    launch.start()
}

/// The Generate sparkle on the commit message's Summary row.
fn click_generate(term: &mut Terminal) {
    let screen = term
        .wait_for("the Generate sparkle on the Summary row", STEP, |s| {
            s.find_all("\u{2726}")
                .into_iter()
                .any(|(_, r)| s.line(r).contains("Summary"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen
        .find_all("\u{2726}")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("Summary"))
        .unwrap();
    term.click(col + 1, row).unwrap();
}

fn wait_dialog(term: &Terminal) -> Screen {
    term.wait_for("the truncation dialog with its buttons", STEP, |s| {
        s.contains("Over the AI limit")
            && s.contains("Send all (A)")
            && s.contains("Send truncated (T)")
            && s.contains("Open setting (O)")
            && s.contains("Cancel (C)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

fn agent(dir: &Path) -> std::path::PathBuf {
    write_capturing_fake_agent(dir, &["feat: add things", "", "Adds a big file."])
}

fn wait_filled(term: &Terminal) {
    term.wait_for("the composer filled from the agent", STEP, |s| {
        s.contains("feat: add things") && s.contains("Adds a big file.")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Over the cap: a warning with the part, the sizes and the setting, nothing
/// sent. "Send all" then sends the whole diff, still through the privacy
/// pipeline (the "What is sent" pane comes next and the token never leaves).
#[test]
fn an_oversized_diff_warns_and_send_all_sends_the_whole_diff_still_cleaned() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = launch(&repo, "pty-trunc-all", &ai_config_toml(&agent), true);

    click_generate(&mut term);
    let screen = wait_dialog(&term);
    let text = screen.text();
    assert!(text.contains("The diff is"), "{text}");
    assert!(text.contains("limit 60.0 KB (60000"), "{text}");
    assert!(text.contains("ai.limits.diff_bytes"), "{text}");
    assert!(
        !text.contains("AI request failed"),
        "a question, not a failure: {text}"
    );
    assert_eq!(agent_calls(&agent), 0, "nothing is sent while it asks");

    term.send_text("a").unwrap();
    term.wait_for("the What is sent pane", STEP, |s| {
        s.contains("Review what is sent to Fake agent") && s.contains("1 secret replaced")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 0, "the pane gates the send");
    term.send_keys("Enter").unwrap();
    wait_filled(&term);
    assert_eq!(agent_calls(&agent), 1);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("value_2999"), "the whole diff");
    assert!(!prompt.contains("diff truncated"), "{prompt}");
    assert!(prompt.contains("[REDACTED:github-token]"), "{prompt}");
    assert!(!prompt.contains(BODY), "the token never leaves");
    term.stop().unwrap();
}

/// "Send truncated" is today's behavior: the end of the diff is left out,
/// and the prompt says so.
#[test]
fn send_truncated_sends_the_cut_diff() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = launch(&repo, "pty-trunc-cut", &ai_config_toml(&agent), false);

    click_generate(&mut term);
    wait_dialog(&term);
    term.send_text("t").unwrap();
    wait_filled(&term);
    assert_eq!(agent_calls(&agent), 1);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("[diff truncated: showing"), "{prompt}");
    assert!(prompt.contains("value_0"), "the start is there");
    assert!(!prompt.contains("value_2999"), "the end is cut");
    term.stop().unwrap();
}

/// "Open setting" cancels the request and lands on the limit's row.
#[test]
fn open_setting_lands_on_the_diff_limit_row() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = launch(&repo, "pty-trunc-setting", &ai_config_toml(&agent), false);

    click_generate(&mut term);
    wait_dialog(&term);
    term.send_text("o").unwrap();
    let screen = term
        .wait_for("Settings on the diff limit row", STEP, |s| {
            s.contains("Largest diff to send (bytes)") && s.contains("60000")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains("Over the AI limit"),
        "the dialog closed: {}",
        screen.text()
    );
    assert_eq!(agent_calls(&agent), 0);
    term.stop().unwrap();
}

/// Focus starts on Open setting: Enter never sends.
#[test]
fn enter_alone_does_not_send() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = launch(&repo, "pty-trunc-enter", &ai_config_toml(&agent), false);

    click_generate(&mut term);
    wait_dialog(&term);
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings, not a send", STEP, |s| {
        s.contains("Largest diff to send (bytes)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 0);
    term.stop().unwrap();
}

/// Cancel and Esc send nothing and leave the composer as it was.
#[test]
fn cancel_and_escape_send_nothing() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = launch(&repo, "pty-trunc-cancel", &ai_config_toml(&agent), false);
    for key in ["c", "Escape"] {
        click_generate(&mut term);
        wait_dialog(&term);
        if key == "c" {
            term.send_text("c").unwrap();
        } else {
            term.send_keys("Escape").unwrap();
        }
        term.wait_for("the dialog gone", STEP, |s| {
            !s.contains("Over the AI limit") && s.contains("Summary")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    assert_eq!(agent_calls(&agent), 0);
    assert!(!term.text().contains("feat: add things"));
    term.stop().unwrap();
}

/// The limit is a setting: raised past the diff's size, the request goes out
/// at once with no question.
#[test]
fn a_raised_limit_sends_without_asking() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let config = format!(
        "{}\n[ai.limits]\ndiff_bytes = 200000\n",
        ai_config_toml(&agent)
    );
    let (mut term, _config) = launch(&repo, "pty-trunc-raised", &config, false);
    click_generate(&mut term);
    wait_filled(&term);
    assert!(!term.text().contains("Over the AI limit"));
    assert_eq!(agent_calls(&agent), 1);
    assert!(agent_prompt(&agent).contains("value_2999"));
    term.stop().unwrap();
}
