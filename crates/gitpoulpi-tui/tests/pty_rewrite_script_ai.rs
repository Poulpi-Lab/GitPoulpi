#![cfg(feature = "ai")]
//! ✦ Write a script with AI in the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git, and a **scripted local
//! agent** (a program printing a canned answer) standing in for the model:
//! no socket is opened. The palette row opens a Script rule and the dialog;
//! the request is typed and sent; the script lands in the editor marked "✦
//! written by AI", checked by the daemon; Test on selection runs it. Nothing
//! is written to the repository. Same rules as `pty_rewrite_script.rs`: no
//! coordinates for keys, no sleeps, every wait names the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// What the agent answers: a fenced Lua script, plain words only (the
/// Windows stand-in echoes it through `cmd`).
const ANSWER: &[&str] = &[
    "Here is the script:",
    "```lua",
    "--! name: Shout the names",
    "--! kinds: commit",
    "function rewrite_commit(c, ctx)",
    "  return { author = { name = string.upper(c.author.name) } }",
    "end",
    "```",
];

/// `base` by the fixture's identity, then two commits by "Old Name".
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    for message in ["first by old", "second by old"] {
        repo.git(&[
            "-c",
            "user.name=Old Name",
            "-c",
            "user.email=old@example.com",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            message,
        ]);
    }
    repo
}

fn names(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%an"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn launch(repo: &FixtureRepo, session: &str, config: String) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("second by old") && s.contains("first by old")
        })
        .start()
}

/// The palette row: a Script rule, its editor, and the dialog on top.
fn open_dialog(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("write a script").unwrap();
    term.wait_for("the palette row, with its sparkle", STEP, |s| {
        s.contains("\u{2726} Rewrite: Write a Script with AI...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dialog showing what is sent", STEP, |s| {
        s.contains("Write a script with AI")
            && s.contains("What is sent")
            && s.contains("Sent to: Fake agent")
            && s.contains("[ ] Send a sample of")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Types the request and sends it: Enter leaves the field for Send, Enter
/// sends.
fn send_request(term: &mut Terminal, text: &str) {
    term.send_text(text).unwrap();
    term.wait_for("the request in the dialog", STEP, |s| s.contains(text))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_ai_writes_a_script_that_lands_marked_in_the_editor_and_can_be_tested() {
    let repo = fixture();
    let original = names(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-ai", ai_config_toml(&agent));
    // Opening the workbench asks no model.
    assert_eq!(agent_calls(&agent), 0);
    open_dialog(&mut term);
    assert_eq!(agent_calls(&agent), 0, "the dialog alone sends nothing");

    send_request(&mut term, "shout the author names");
    let screen = term
        .wait_for(
            "the AI's script in the editor, marked and checked",
            STEP,
            |s| {
                s.contains("--! name: Shout the names")
                    && s.contains("\u{2726} written by AI: read it before Preview")
                    && s.contains("OK: rewrite_commit found")
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("What is sent"), "{}", screen.text());
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(names(&repo), original, "nothing was applied");

    // Test on selection: the script runs on the newest commits, writes
    // nothing.
    term.send_keys("F5").unwrap();
    let screen = term
        .wait_for("the test result", STEP, |s| {
            s.contains("would change") && s.contains("author name: Old Name -> OLD NAME")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("\u{2726} written by AI"),
        "the mark stays until the script is edited\n{}",
        screen.text()
    );
    assert_eq!(names(&repo), original, "a test writes nothing");
}

/// Unix only: the Windows stand-in cannot capture what it is sent. With the
/// sample box left alone, no commit data reaches the agent.
#[cfg(unix)]
#[test]
fn the_sample_is_off_by_default_and_no_commit_data_is_sent() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-ai-off", ai_config_toml(&agent));
    open_dialog(&mut term);
    send_request(&mut term, "shout the author names");
    term.wait_for("the script arrived", STEP, |s| {
        s.contains("\u{2726} written by AI")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("shout the author names"), "{prompt}");
    assert!(
        prompt.contains("rewrite_commit(c, ctx)"),
        "the contract\n{prompt}"
    );
    for needle in [
        "Old Name",
        "old@example.com",
        "first by old",
        "second by old",
        "sample",
    ] {
        assert!(!prompt.contains(needle), "{needle:?} was sent\n{prompt}");
    }
}

/// With AI off (the default) the editor offers nothing with a sparkle and
/// the palette hides the row.
#[test]
fn with_ai_off_nothing_offers_it() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-ai-none", String::new());
    term.send_keys("C-p").unwrap();
    term.send_text("write a script").unwrap();
    term.wait_for("the query echoed", STEP, |s| s.contains("write a script"))
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    assert!(
        !screen.contains("Write a Script with AI"),
        "{}",
        screen.text()
    );
}

#[test]
fn what_is_sent_starts_collapsed_and_its_pane_scrolls_by_key_wheel_and_scrollbar() {
    let repo = fixture();
    let original = names(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-script-ai-pane", ai_config_toml(&agent));
    open_dialog(&mut term);

    // Collapsed: the header with its mark and its key, not the text.
    let screen = term.screen();
    assert!(
        screen.contains("\u{25B8} What is sent"),
        "{}",
        screen.text()
    );
    assert!(screen.contains("Alt+W: show or hide"), "{}", screen.text());
    assert!(
        !screen.contains("Instructions to the model:"),
        "{}",
        screen.text()
    );

    // Alt+W opens it.
    term.send_keys("M-w").unwrap();
    term.wait_for("the pane open", STEP, |s| {
        s.contains("\u{25BE} What is sent") && s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !term.screen().contains("Your request:"),
        "the prompt is longer than the pane\n{}",
        term.screen().text()
    );

    // Keys: Tab reaches the pane (description, box, pane), PageDown / End /
    // Home scroll it.
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("PageDown").unwrap();
    term.wait_for("the pane scrolled by a page", STEP, |s| {
        !s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("End").unwrap();
    term.wait_for("the end of the pane", STEP, |s| s.contains("Your request:"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Home").unwrap();
    term.wait_for("the top of the pane", STEP, |s| {
        s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The wheel over the pane.
    let (col, row) = term
        .wait_for_text("Instructions to the model:", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.scroll(true, col + 2, row + 1).unwrap();
    term.wait_for("the wheel scrolled the pane", STEP, |s| {
        !s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.scroll(false, col + 2, row + 1).unwrap();
    term.wait_for("the wheel scrolled it back", STEP, |s| {
        s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Its own scrollbar: on the pane's right edge, under the header's last
    // column. Dragging the thumb to the bottom shows the end.
    let (hint_col, head_row) = term
        .wait_for_text("Alt+W: show or hide", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let bar_col = hint_col + "Alt+W: show or hide".len() as u16;
    let screen = term.screen();
    assert!(
        screen
            .find_all("\u{2191}")
            .contains(&(bar_col, head_row + 1)),
        "no up arrow at the pane's top right\n{}",
        screen.text()
    );
    let down = screen
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, _)| *c == bar_col)
        .map(|(_, r)| r)
        .max()
        .unwrap_or_else(|| panic!("no down arrow on the bar\n{}", screen.text()));
    term.drag((bar_col, head_row + 2), (bar_col, down - 1))
        .unwrap();
    term.wait_for("the drag reached the end", STEP, |s| {
        s.contains("Your request:")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A click on the header hides it; Enter on the focused header shows it.
    let (col, row) = term
        .wait_for_text("\u{25BE} What is sent", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 4, row).unwrap();
    term.wait_for("the pane collapsed", STEP, |s| {
        s.contains("\u{25B8} What is sent") && !s.contains("Your request:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the pane open again", STEP, |s| {
        s.contains("\u{25BE} What is sent") && s.contains("Instructions to the model:")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("M-w").unwrap();
    term.wait_for("closed by the key", STEP, |s| {
        s.contains("\u{25B8} What is sent")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(agent_calls(&agent), 0, "looking sends nothing");
    assert_eq!(names(&repo), original);
}
