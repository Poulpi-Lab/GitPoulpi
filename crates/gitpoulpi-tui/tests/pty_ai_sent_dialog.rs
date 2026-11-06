#![cfg(all(feature = "ai", unix))]
//! "What is sent": the dialog between an AI action and its request, driven
//! through a pty with a **capturing fake agent** standing in for the model.
//! The request is cleaned before the dialog shows it (a token replaced, a
//! `.env` left out), nothing is sent until Send, Cancel and Esc send
//! nothing, and "Don't show again" writes the setting.
//!
//! The harness turns every per-action preview off (`tests/common`); the tests
//! here turn the commit message's back on with its environment key (which
//! also wins over the file, so "Don't show again" is checked in the file; that
//! the next request then skips the dialog is `ai_sent_dialog`'s unit test and
//! `AiSentPreview::shows`).

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

/// A staged source file holding a token and a staged `.env`.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", format!("one\ntoken = ghp_{BODY}\n").as_bytes())
        .unwrap()
        .write_worktree(".env", b"DB_PASSWORD=hunter2hunter2\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .stage(".env")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, agent: &Path) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(ai_config_toml(agent))
        .env("GITPOULPI_AI_SENT_PREVIEW_COMMIT_MESSAGE", "true")
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("Amend previous commit")
        })
        .start()
}

fn generate(term: &mut Terminal) {
    let screen = term
        .wait_for("the Generate sparkle on the Summary row", STEP, |s| {
            s.find_all("\u{2726}")
                .into_iter()
                .any(|(_, r)| s.line(r).contains("Summary"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen
        .find_all("\u{2726}")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("Summary"))
        .unwrap();
    term.click(col + 1, row).expect("click Generate");
}

fn wait_dialog(term: &Terminal) -> xtask::pty::Screen {
    // The whole dialog, buttons included: the title can be on screen a frame
    // before the rows under it.
    term.wait_for("the What is sent dialog with its buttons", STEP, |s| {
        s.contains("Review what is sent to Fake agent")
            && (0..s.rows()).any(|r| {
                let line = s.line(r);
                line.contains("Send") && line.contains("Cancel")
            })
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

fn staged_ok(repo: &FixtureRepo) {
    // The fixture staged both files through git, which the guard does not see.
    let staged = repo.git(&["diff", "--cached", "--name-only"]);
    assert!(
        staged.contains(".env") && staged.contains("a.txt"),
        "{staged}"
    );
}

#[test]
fn the_dialog_shows_the_cleaned_request_and_nothing_is_sent_until_send() {
    let repo = fixture();
    staged_ok(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: add things"]);
    let (mut term, _config) = launch(&repo, "pty-sent-send", &agent);

    generate(&mut term);
    let screen = wait_dialog(&term);
    let text = screen.text();
    assert!(text.contains("1 secret replaced"), "{text}");
    assert!(text.contains("github-token"), "{text}");
    assert!(text.contains("1 sensitive file left out"), "{text}");
    assert!(text.contains("[REDACTED:github-token]"), "{text}");
    assert!(
        !text.contains(BODY),
        "the screen never holds the token: {text}"
    );
    assert!(!text.contains("hunter2"), "{text}");
    assert!(
        text.contains("Don't show again for Commit message"),
        "{text}"
    );
    assert_eq!(
        agent_calls(&agent),
        0,
        "nothing is sent while the dialog is up"
    );

    // Send is focused: Enter sends.
    term.send_keys("Enter").expect("Enter");
    term.wait_for("the composer filled from the agent", STEP, |s| {
        s.contains("feat: add things") && !s.contains("Review what is sent")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("[REDACTED:github-token]"), "{prompt}");
    assert!(!prompt.contains(BODY), "{prompt}");
    assert!(
        !prompt.contains("hunter2") && !prompt.contains("DB_PASSWORD"),
        "{prompt}"
    );
}

#[test]
fn escape_cancel_and_the_cross_send_nothing() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: never"]);
    let (mut term, _config) = launch(&repo, "pty-sent-cancel", &agent);

    generate(&mut term);
    wait_dialog(&term);
    term.send_keys("Escape").expect("Esc");
    term.wait_for("the dialog gone and the cancellation said", STEP, |s| {
        !s.contains("Review what is sent") && s.contains("canceled")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0);

    // The Cancel button, by the mouse.
    generate(&mut term);
    let screen = wait_dialog(&term);
    let (col, row) = screen
        .find_all("Cancel")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("Send"))
        .expect("the Cancel button beside Send");
    term.click(col + 1, row).expect("click Cancel");
    term.wait_for_text_to_go("Review what is sent", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0);

    // The close cross on the top border.
    generate(&mut term);
    let screen = wait_dialog(&term);
    let (title_col, title_row) = screen.find("Review what is sent").unwrap();
    let border = screen.line(title_row);
    let cross = border
        .rfind('\u{2715}')
        .map(|byte| border[..byte].chars().count() as u16)
        .expect("a close cross on the border row");
    let _ = title_col;
    term.click(cross, title_row).expect("click the cross");
    term.wait_for_text_to_go("Review what is sent", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        agent_calls(&agent),
        0,
        "none of the three answers sent anything"
    );
}

#[test]
fn dont_show_again_is_written_to_config_toml_when_the_request_is_sent() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: once"]);
    let (mut term, config) = launch(&repo, "pty-sent-remember", &agent);

    generate(&mut term);
    wait_dialog(&term);
    // Shift+Tab from Send reaches the box, Space ticks it, Tab Tab reaches Send.
    term.send_keys("BTab").expect("Shift+Tab");
    term.send_keys("Space").expect("Space");
    term.wait_for_text("[x] Don't show again", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").expect("Tab");
    term.send_keys("Enter").expect("Enter on Send");
    term.wait_for_text("feat: once", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap();
    assert!(
        stored.contains("commit_message = false"),
        "the choice is in config.toml:\n{stored}"
    );
}
