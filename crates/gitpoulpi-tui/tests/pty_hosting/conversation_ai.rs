//! "Write with AI" inside the Markdown composer, end to end (PTY test 9 of
//! the conversations plan, without the repository part): the new-issue
//! form's description, a **fake local agent** standing in for the model (a
//! script printing a canned answer, so no socket is opened) and the mock
//! host, which must never see a write while the AI works.

use std::time::Duration;

use crate::mock::*;
use crate::support::*;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};

const DRAFT_LINE: &str = "Please attach the stack trace of the crash.";

/// The New issue form (ISSUES section, `n`) with a title and a typed
/// description, the keyboard in the description.
fn open_form(term: &mut xtask::pty::Terminal) {
    term.wait_for("the ISSUES section", STARTUP, |s| {
        s.contains("ISSUES (1)") && s.contains("Crash on startup")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = term.wait_for_text("Crash on startup", STEP).unwrap();
    term.click(col + 2, row).expect("focus the section");
    term.send_keys("n").unwrap();
    term.wait_for("the New issue form", STEP, |s| {
        s.contains("New issue") && s.contains("Labels: none")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("Parser crashes").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("my own words").unwrap();
    term.wait_for("everything typed", STEP, |s| {
        s.contains("Parser crashes") && s.contains("my own words")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The "What is sent" dialog opens before the request leaves: it shows the
/// instruction and the draft, and Enter on its focused Send goes on.
fn send_request(term: &mut xtask::pty::Terminal, expect: &[&str]) {
    let screen = term
        .wait_for("the What is sent dialog", STEP, |s| {
            s.contains("Review what is sent") && s.contains("Send")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    for want in expect {
        assert!(screen.contains(want), "{want:?}:\n{}", screen.text());
    }
    term.send_keys("Enter").expect("Send");
}

/// No request of the mock changed anything on the host.
fn no_write(mock: &Mock) {
    let writes: Vec<_> = mock
        .requests()
        .into_iter()
        .filter(|r| matches!(r.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE"))
        .map(|r| format!("{} {}", r.method, r.target))
        .collect();
    assert!(
        writes.is_empty(),
        "the AI only drafts, nothing reached the host: {writes:?}"
    );
}

#[cfg(unix)]
#[test]
fn the_ai_field_drafts_into_the_description_previews_it_and_ctrl_z_restores_the_text() {
    use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
    let mock = start_mock();
    let repo = fixture(mock.port);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &[DRAFT_LINE]);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-composer-ai",
        &config_toml(mock.port, &format!("\n{}", ai_config_toml(&agent))),
        COLS,
        "Add the feature",
    );
    open_form(&mut term);
    assert_eq!(agent_calls(&agent), 0, "nothing is asked on its own");

    // The button is there; Alt+I opens the field inside the composer.
    let screen = term.screen();
    assert!(
        screen.contains("\u{2726} Write with AI"),
        "{}",
        screen.text()
    );
    term.send_keys("M-i").expect("Alt+I");
    let screen = term
        .wait_for("the AI field", STEP, |s| {
            s.contains("What I want in this answer") && s.contains("Improve my draft")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("[ ] Let the AI read the repository (not built yet)"),
        "unticked and disabled with its reason:\n{}",
        screen.text()
    );
    assert!(
        screen.contains("[x] Improve my draft"),
        "there is a text to improve:\n{}",
        screen.text()
    );

    // The instruction is typed in the field, not in the description.
    term.send_text("ask for a stack trace").unwrap();
    term.wait_for("the instruction typed", STEP, |s| {
        s.contains("ask for a stack trace")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("draft");
    send_request(&mut term, &["Parser crashes", "my own words"]);
    let screen = term
        .wait_for("the draft in the Preview, the dialog gone", STEP, |s| {
            s.contains(DRAFT_LINE)
                && s.contains("Draft ready.")
                && !s.contains("Review what is sent")
                && !s.contains("my own words")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert_eq!(agent_calls(&agent), 1);
    assert!(!screen.contains("my own words"), "{}", screen.text());

    // What the model was given: the title, the instruction and the draft.
    let prompt = agent_prompt(&agent);
    for want in ["Parser crashes", "ask for a stack trace", "my own words"] {
        assert!(prompt.contains(want), "{want:?} missing from:\n{prompt}");
    }

    // Ctrl+Z gives the previous text back, in the Write mode.
    term.send_keys("C-z").expect("undo");
    term.wait_for("the old text back", STEP, |s| {
        s.contains("my own words") && !s.contains(DRAFT_LINE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));

    // The AI never published: no write reached the host, and the form is
    // still open (posting is the user's Create).
    no_write(&mock);
    assert!(term.screen().contains("Labels: none"));
    term.stop().unwrap();
}

#[test]
fn stop_kills_the_agent_and_the_text_stays() {
    use gitpoulpi_testkit::fake_agent::write_slow_fake_agent;
    let mock = start_mock();
    let repo = fixture(mock.port);
    let tools = tempfile::tempdir().unwrap();
    // The agent notes its call only once it finishes: a killed one never does.
    let agent = write_slow_fake_agent(tools.path(), 4, &[DRAFT_LINE]);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-composer-ai-stop",
        &config_toml(mock.port, &format!("\n{}", ai_config_toml(&agent))),
        COLS,
        "Add the feature",
    );
    open_form(&mut term);
    term.send_keys("M-i").expect("Alt+I");
    term.wait_for("the AI field", STEP, |s| {
        s.contains("What I want in this answer")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("draft");
    send_request(&mut term, &[]);
    let screen = term
        .wait_for("the run in progress", STEP, |s| {
            s.contains("Writing...") && s.contains("Stop")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // The Stop button, by mouse.
    let (c, r) = screen
        .find_all("Stop")
        .into_iter()
        .next()
        .expect("the Stop button");
    term.click(c + 1, r).expect("Stop");
    term.wait_for("stopped", STEP, |s| {
        s.contains("Stopped.") && !s.contains("Writing...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    // Past the agent's sleep, it never answered: its process was killed.
    std::thread::sleep(Duration::from_secs(7));
    assert_eq!(
        agent_calls(&agent),
        0,
        "the agent was killed, not waited for"
    );
    let screen = term.screen();
    assert!(!screen.contains(DRAFT_LINE), "{}", screen.text());
    assert!(screen.contains("my own words"), "{}", screen.text());
    no_write(&mock);
    term.stop().unwrap();
}

#[test]
fn with_ai_off_the_composer_shows_no_button_and_no_panel() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-composer-ai-off",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term);
    let screen = term.screen();
    assert!(
        !screen.contains("Write with AI") && !screen.contains("\u{2726}"),
        "{}",
        screen.text()
    );
    assert!(!screen.contains("What I want in this answer"));
    term.stop().unwrap();
}

#[test]
fn a_fake_agent_that_is_not_asked_is_never_called() {
    // Opening the panel asks nothing either: only Enter or the Draft button.
    let mock = start_mock();
    let repo = fixture(mock.port);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &[DRAFT_LINE]);
    let (mut term, _config) = launch_issues(
        &repo,
        "pty-composer-ai-idle",
        &config_toml(mock.port, &format!("\n{}", ai_config_toml(&agent))),
        COLS,
        "Add the feature",
    );
    open_form(&mut term);
    term.send_keys("M-i").expect("Alt+I");
    term.wait_for("the AI field", STEP, |s| {
        s.contains("What I want in this answer")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Close it with its Close button.
    let screen = term.screen();
    let (c, r) = screen
        .find_all("Close")
        .into_iter()
        .last()
        .expect("the Close button");
    term.click(c + 1, r).expect("Close");
    term.wait_for("the panel gone", STEP, |s| {
        !s.contains("What I want in this answer")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0);
    no_write(&mock);
    term.stop().unwrap();
}
