#![cfg(all(feature = "ai", unix))]
//! Expected states are guided, never reported as failures: a conflict over
//! the AI input limit warns (Send anyway / Open setting / Cancel), and the
//! AI commit composer and every way of committing explain that files still
//! have conflicts, with a button into the merge editor. End to end through a
//! pty: the real binary and daemon, a real conflicted repository, and a
//! **capturing fake agent** standing in for the model (no socket is opened).

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

/// A conflict of about 50 KB in all (25 KB a side), a token leading ours.
fn big_conflict() -> ConflictFixture {
    ConflictFixture::large_text(25_000, &format!("token = ghp_{BODY}"))
}

fn open_editor(
    fixture: &ConflictFixture,
    session: &str,
    config: &str,
    preview: bool,
) -> (Terminal, tempfile::TempDir) {
    let mut launch = common::launch(fixture.root())
        .session(session)
        .size(200, 50)
        .config(config)
        .no_exit_pause()
        .ready_when("the conflicted row", |s| s.contains("! notes.txt"))
        .mouse_capture();
    if preview {
        launch = launch.env("GITPOULPI_AI_SENT_PREVIEW_CONFLICT_RESOLVE", "true");
    }
    let (mut term, dir) = launch.start();
    let (c, r) = term
        .wait_for_text("! notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the merge editor", STEP, |s| s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, dir)
}

fn click_suggest(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2726} Suggest", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
}

fn wait_warning(term: &Terminal) -> Screen {
    // The whole dialog, buttons included: the title can be on screen a frame
    // before the rows under it.
    term.wait_for("the too large warning with its buttons", STEP, |s| {
        s.contains("Conflict too large to send")
            && s.contains("Send anyway (S)")
            && s.contains("Open setting (O)")
            && s.contains("Cancel (C)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// Every row under the Output's title rule.
fn output_text(s: &Screen) -> String {
    match (0..s.rows()).find(|r| s.line(*r).contains("Output \u{b7}")) {
        Some(title) => (title + 1..s.rows())
            .map(|r| s.line(r))
            .collect::<Vec<_>>()
            .join("\n"),
        None => String::new(),
    }
}

fn agent(dir: &Path) -> std::path::PathBuf {
    write_capturing_fake_agent(dir, &["```", "resolved by the agent", "```"])
}

/// Over the limit: a warning with the numbers and the setting, no error
/// report, nothing sent. "Send anyway" then goes through the same privacy
/// pipeline as any request (the "What is sent" pane shows the cleaned text,
/// and the agent never receives the token).
#[test]
fn an_oversized_conflict_warns_and_send_anyway_still_cleans_the_request() {
    let fixture = big_conflict();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) =
        open_editor(&fixture, "pty-ai-limit-send", &ai_config_toml(&agent), true);

    click_suggest(&mut term);
    let screen = wait_warning(&term);
    let text = screen.text();
    assert!(text.contains("notes.txt"), "{text}");
    assert!(text.contains("40.0 KB (40000"), "the limit: {text}");
    assert!(text.contains("ai.limits.conflict_bytes"), "{text}");
    assert!(
        !text.contains("could not suggest a resolution")
            && !text.contains("too large to send to the AI"),
        "a warning, not a failure: {text}"
    );
    assert_eq!(agent_calls(&agent), 0, "nothing is sent while it warns");

    term.send_text("s").unwrap();
    // Past the warning the request is an ordinary one: the review pane first.
    term.wait_for("the What is sent pane", STEP, |s| {
        s.contains("Review what is sent to Fake agent") && s.contains("1 secret replaced")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 0, "the pane gates the send");
    term.send_keys("Enter").unwrap();
    term.wait_for("the suggestion in the Output", STEP, |s| {
        output_text(s).contains("resolved by the agent")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 1);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("[REDACTED:github-token]"), "{prompt}");
    assert!(!prompt.contains(BODY), "{prompt}");
    term.stop().unwrap();
}

/// Cancel (and Esc) send nothing and leave the editor as it was.
#[test]
fn cancel_and_escape_send_nothing() {
    let fixture = big_conflict();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = open_editor(
        &fixture,
        "pty-ai-limit-cancel",
        &ai_config_toml(&agent),
        false,
    );
    for key in ["c", "Escape"] {
        click_suggest(&mut term);
        wait_warning(&term);
        if key == "c" {
            term.send_text("c").unwrap();
        } else {
            term.send_keys("Escape").unwrap();
        }
        term.wait_for("the dialog gone", STEP, |s| {
            !s.contains("Conflict too large to send") && s.contains("Conflict 1 / 1")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
    assert_eq!(agent_calls(&agent), 0);
    term.stop().unwrap();
}

/// "Open setting" lands on the limit's row in Settings > AI.
#[test]
fn open_setting_lands_on_the_limit_row() {
    let fixture = big_conflict();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config) = open_editor(
        &fixture,
        "pty-ai-limit-setting",
        &ai_config_toml(&agent),
        false,
    );
    click_suggest(&mut term);
    wait_warning(&term);
    term.send_text("o").unwrap();
    let screen = term
        .wait_for("Settings on the limit row", STEP, |s| {
            s.contains("Largest conflict to send (bytes)") && s.contains("40000")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !screen.contains("Conflict too large to send"),
        "the dialog closed: {}",
        screen.text()
    );
    assert_eq!(agent_calls(&agent), 0);
    term.stop().unwrap();
}

/// The limit is a setting: raised past the conflict's size, the request goes
/// out at once, with no warning.
#[test]
fn a_raised_limit_sends_without_asking() {
    let fixture = big_conflict();
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let config = format!(
        "{}\n[ai.limits]\nconflict_bytes = 60000\n",
        ai_config_toml(&agent)
    );
    let (mut term, _config) = open_editor(&fixture, "pty-ai-limit-raised", &config, false);
    click_suggest(&mut term);
    term.wait_for("the suggestion, no warning", STEP, |s| {
        output_text(s).contains("resolved by the agent")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("Conflict too large to send"));
    assert_eq!(agent_calls(&agent), 1);
    term.stop().unwrap();
}

fn start_conflicted(session: &str, config: &str) -> (Terminal, tempfile::TempDir, ConflictFixture) {
    let fixture = ConflictFixture::text();
    let (term, dir) = common::launch(fixture.root())
        .session(session)
        .size(200, 50)
        .config(config)
        .no_exit_pause()
        .ready_when("the conflicted row", |s| s.contains("! notes.txt"))
        .mouse_capture()
        .start();
    (term, dir, fixture)
}

fn wait_conflicts_dialog(term: &Terminal) -> Screen {
    term.wait_for("the conflicts first dialog", STEP, |s| {
        s.contains("Resolve the conflicts first")
            && s.contains("1 file still has conflicts")
            && s.contains("Resolve conflicts (R)")
            && s.contains("Cancel (C)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// Committing with unresolved conflicts: a guiding dialog listing the files,
/// no "Commit failed" report, and [Resolve conflicts] opens the merge editor.
#[test]
fn committing_with_conflicts_guides_to_the_merge_editor() {
    let (mut term, _config, fixture) = start_conflicted("pty-commit-conflicts", "");
    let head = fixture.git(&["rev-parse", "HEAD"]);
    term.send_keys("C-g").unwrap();
    term.send_keys("Enter").unwrap();
    let screen = wait_conflicts_dialog(&term);
    let text = screen.text();
    assert!(text.contains("notes.txt"), "{text}");
    assert!(
        !text.contains("Commit failed") && !text.contains("unmerged files"),
        "an expected state, not a failure: {text}"
    );
    term.send_text("r").unwrap();
    term.wait_for("the merge editor", STEP, |s| s.contains("Conflict 1 / 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        fixture.git(&["rev-parse", "HEAD"]),
        head,
        "nothing committed"
    );
    term.stop().unwrap();
}

/// The palette grays the commit commands with the reason, and Cancel / Esc
/// close the dialog without opening anything.
#[test]
fn the_palette_grays_commit_and_cancel_closes_the_dialog() {
    let (mut term, _config, _fixture) = start_conflicted("pty-commit-conflicts-palette", "");
    term.wait_for_text("! notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.send_text("commit staged changes now").unwrap();
    term.wait_for("the reason on the grayed entry", STEP, |s| {
        s.contains("files still have conflicts: resolve them first")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    // `Esc` then `Ctrl+G` at once reads as `Alt+Ctrl+G`: wait for the palette.
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("commit staged changes now")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-g").unwrap();
    term.send_keys("Enter").unwrap();
    wait_conflicts_dialog(&term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog gone", STEP, |s| {
        !s.contains("Resolve the conflicts first")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("Conflict 1 / 1"), "no editor opened");
    term.stop().unwrap();
}

/// The AI commit composer with unresolved conflicts: the same dialog, no
/// "AI request failed", and nothing is sent.
#[test]
fn the_composer_with_conflicts_guides_instead_of_failing() {
    let tools = tempfile::tempdir().unwrap();
    let agent = agent(tools.path());
    let (mut term, _config, _fixture) =
        start_conflicted("pty-compose-conflicts", &ai_config_toml(&agent));
    term.wait_for_text("! notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // `Ctrl+G C`: AI splits the working changes into commits.
    term.send_keys("C-g").unwrap();
    term.send_text("C").unwrap();
    let screen = wait_conflicts_dialog(&term);
    let text = screen.text();
    assert!(
        text.contains("cannot split the changes"),
        "says what is blocked: {text}"
    );
    assert!(
        !text.contains("AI request failed") && !text.contains("resolve the conflicts first"),
        "no error report: {text}"
    );
    assert_eq!(agent_calls(&agent), 0);
    term.send_text("c").unwrap();
    term.wait_for("the dialog gone", STEP, |s| {
        !s.contains("Resolve the conflicts first")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
