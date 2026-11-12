//! Slice R3 end to end: the review of merge request `!58` with the ✦
//! suggestions. The model is a **fake local agent** (a script that prints a
//! canned answer), so no socket is opened; the host is `xtask mock-host`.
//!
//! What it pins down: nothing is asked until the reviewer asks; suggestions
//! show under their lines and in the Overview list; accepting one makes an
//! ordinary pending comment (the counter says 1), dismissing another makes
//! none; and nothing reaches the host before the review is submitted.

use crate::review::{launch_review_with, open_files, open_from_menu, review_fixture, wait_until};
use crate::support::*;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
use xtask::mock_host::{self, Options};

/// What the fake model answers: one comment on the second added line of
/// `src/planets.rs`, one on the first line of `docs/moons.md`.
const ANSWER: &str = r#"{"summary": "Adds moons to the table and documents them.", "suggestions": [{"path": "src/planets.rs", "line": 13, "side": "new", "severity": "warning", "body": "Say why there are two moons."}, {"path": "docs/moons.md", "line": 1, "side": "new", "severity": "info", "body": "Add a one line intro."}]}"#;

#[test]
fn suggest_accept_one_dismiss_one_and_nothing_reaches_the_host() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[ANSWER]);
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-ai",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);

    // Opening the review asks nothing: the model runs on the explicit action.
    let screen = term
        .wait_for("the Suggest review button", STEP, |s| {
            s.contains("\u{2726} Draft review...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Suggestions ("), "{}", screen.text());
    assert_eq!(agent_calls(&agent), 0, "nothing is asked on its own");

    // The key asks; the answer lands in the Overview with the summary.
    term.send_keys("s").expect("Draft review");
    term.wait_for("the review options", STEP, |s| s.contains("Review options"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("Run");
    let screen = term
        .wait_for("the suggestions listed", STEP, |s| {
            s.contains("Suggestions (2)") && s.contains("Say why there are two moons.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 1);
    for want in [
        "Adds moons to the table and documents them.",
        "src/planets.rs:13",
        "docs/moons.md:1",
        "Accept all",
    ] {
        assert!(screen.contains(want), "{want}:\n{}", screen.text());
    }
    assert!(
        !screen.contains("pending comment"),
        "a suggestion is not a comment yet:\n{}",
        screen.text()
    );

    // Files: the ghost comment sits under its line.
    open_files(&mut term);
    let screen = term
        .wait_for("the ghost comment under its line", STEP, |s| {
            s.contains("warning - line 13") && s.contains("Say why there are two moons.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, code) = screen.find("moons.push(1);").unwrap();
    let (_, ghost) = screen.find("warning - line 13").unwrap();
    assert_eq!(ghost, code + 1, "right under the line:\n{}", screen.text());

    // Accept it with its button: one pending comment, editable like any.
    let (c, r) = screen
        .find_all("Accept")
        .into_iter()
        .find(|(_, row)| *row == ghost)
        .expect("the Accept button on the ghost");
    term.click(c + 1, r).expect("Accept");
    term.wait_for("one pending comment", STEP, |s| {
        s.contains("1 pending comment")
            && s.contains("Pending - line 13")
            && s.contains("added to your pending review")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        repo_state(&repo).contains("Say why there are two moons."),
        "kept in the repository's state like any pending comment:\n{}",
        repo_state(&repo)
    );

    // Dismiss the other one from the Overview list: no comment from it.
    term.send_keys("M-1").expect("Overview");
    let screen = term
        .wait_for("the remaining suggestion", STEP, |s| {
            s.contains("Suggestions (1)") && s.contains("Add a one line intro.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("docs/moons.md:1").unwrap();
    let (c, r) = screen
        .find_all("Dismiss")
        .into_iter()
        .find(|(_, y)| *y == row)
        .expect("the Dismiss button on the suggestion");
    term.click(c + 1, r).expect("Dismiss");
    let screen = term
        .wait_for("the list emptied", STEP, |s| {
            s.contains("suggestion dismissed") && !s.contains("Add a one line intro.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("1 pending comment"),
        "still exactly one:\n{}",
        screen.text()
    );

    // Nothing has gone to the host: only Finish review sends, and it was not used.
    assert!(!mock.saw("POST", "/draft_notes"), "no draft note");
    assert!(!mock.saw("POST", "/discussions"), "no comment");
    assert!(!mock.saw("POST", "/approve"), "no verdict");
    wait_until("the agent was called exactly once", || {
        agent_calls(&agent) == 1
    });
}

#[test]
fn escape_cancels_a_slow_suggestion_and_with_ai_off_there_is_no_button() {
    use gitpoulpi_testkit::fake_agent::write_slow_fake_agent;
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_slow_fake_agent(tools.path(), 30, &[ANSWER]);
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-ai-cancel",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);
    term.wait_for("the Suggest review button", STEP, |s| {
        s.contains("\u{2726} Draft review...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("s").expect("Draft review");
    term.wait_for("the review options", STEP, |s| s.contains("Review options"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("Run");
    // In-flight feedback appears, the button turns into Cancel.
    term.wait_for("the in-flight line", STEP, |s| {
        s.contains("suggesting a review...") && s.contains("Esc cancels")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").expect("cancel");
    let screen = term
        .wait_for("canceled, view still open", STEP, |s| {
            s.contains("suggestion canceled") && s.contains("Review !58")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("Suggestions ("), "{}", screen.text());
    assert!(
        screen.contains("\u{2726} Draft review..."),
        "ready to ask again:\n{}",
        screen.text()
    );
}

#[test]
fn with_ai_off_the_review_shows_nothing_of_it() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let (mut term, _config) = launch_review_with(&mock, &repo, "pty-review-ai-off", "");
    open_from_menu(&mut term);
    let screen = term.screen();
    assert!(
        !screen.contains("Draft review") && !screen.contains("\u{2726}"),
        "{}",
        screen.text()
    );
    term.send_keys("C-p").expect("palette");
    term.wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Draft Review").unwrap();
    let screen = term
        .wait_for("the query echoed", STEP, |s| s.contains("Draft Review"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Review: \u{2726} Draft Review"),
        "the palette row is hidden while AI is off:\n{}",
        screen.text()
    );
}

/// The full draft: verdict and text come with the suggestions.
const DRAFT_ANSWER: &str = r#"{"verdict": "approve", "body": "Looks good once the two remarks are addressed.", "summary": "Adds moons to the table and documents them.", "suggestions": [{"path": "src/planets.rs", "line": 13, "side": "new", "severity": "warning", "body": "Say why there are two moons."}, {"path": "docs/moons.md", "line": 1, "side": "new", "severity": "info", "body": "Add a one line intro."}]}"#;

#[test]
fn use_the_draft_fills_finish_review_and_the_host_gets_the_approval_and_two_comments() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = review_fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[DRAFT_ANSWER]);
    let (mut term, _config) = launch_review_with(
        &mock,
        &repo,
        "pty-review-ai-draft",
        &format!("\n{}", ai_config_toml(&agent)),
    );
    open_from_menu(&mut term);
    term.wait_for("the Suggest review button", STEP, |s| {
        s.contains("\u{2726} Draft review...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("s").expect("Draft review");
    term.wait_for("the review options", STEP, |s| s.contains("Review options"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("Run");
    let screen = term
        .wait_for("the draft card", STEP, |s| {
            s.contains("AI review draft")
                && s.contains("Looks good once the two remarks")
                && s.contains("Use draft...")
                && s.contains("Approve")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = screen;
    assert!(
        repo_state(&repo).contains("Looks good once the two remarks"),
        "the draft is kept like a pending comment:\n{}",
        repo_state(&repo)
    );

    // `u`: both suggestions become pending, Finish review opens filled.
    term.send_keys("u").expect("Use draft");
    let screen = term
        .wait_for("Finish review filled", STEP, |s| {
            s.contains("2 pending comments will be sent.")
                && s.contains("Looks good once the two remarks")
                && s.text()
                    .lines()
                    .any(|l| l.contains("Verdict") && l.contains("Approve"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = screen;
    assert!(!mock.saw("POST", "/approve"), "nothing sent yet");

    term.send_keys("C-s").expect("submit");
    wait_until("the host got the comments and the approval", || {
        mock.saw("POST", "/draft_notes/bulk_publish") && mock.saw("POST", "/approve")
    });
    let notes: Vec<String> = mock
        .requests()
        .into_iter()
        .filter(|r| r.method == "POST" && r.target.ends_with("/draft_notes"))
        .map(|r| r.body)
        .collect();
    for want in ["Say why there are two moons.", "Add a one line intro."] {
        assert!(
            notes.iter().any(|b| b.contains(want)),
            "a draft note for {want}: {notes:?}"
        );
    }
    term.wait_for("the review reported", STEP, |s| {
        s.contains("review submitted with 2 comments") && !s.contains("AI review draft")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !repo_state(&repo).contains("Looks good once the two remarks"),
        "the spent draft is gone from disk"
    );
}
