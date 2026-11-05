#![cfg(feature = "ai")]
//! ✦ Create Rules with AI in the history rewrite workbench through a pty
//! (`xtask::pty`): the real binary, daemon and git, and a **scripted local
//! agent** (a program printing a canned answer) standing in for the model:
//! no socket is opened. The description is typed, the answer fills the
//! workbench, entries that cannot be used are listed with why, and NOTHING is
//! rewritten: the refs of the fixture repository are asserted unchanged.
//! Same rules as `pty_rewrite.rs`: no coordinates for keys, no sleeps, every
//! wait names the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// What the agent answers: one good rule, one the workbench must refuse (a
/// shift of zero) and a remark. Plain words only (the Windows stand-in echoes
/// it through `cmd`).
const ANSWER: &[&str] = &[
    "Here are the rules:",
    "```json",
    "{\"scope\": {\"scope\": \"all_local\"}, \"rules\": [{\"label\": \"Old email\", \"kind\": {\"type\": \"identity\", \"when\": {\"email\": \"old@example.com\"}, \"set\": {\"email\": \"new@example.com\"}}}, {\"kind\": {\"type\": \"date_shift\", \"by_seconds\": 0}}], \"notes\": [\"No date limit exists.\"]}",
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

/// Every ref and what it points at: a rewrite would change this.
fn refs(repo: &FixtureRepo) -> String {
    repo.git(&["for-each-ref", "--format=%(refname) %(objectname)"])
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

/// `Ctrl+G Shift+H`.
fn open_by_chord(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The dialog, described by the window: what is sent is on screen.
fn wait_dialog(term: &Terminal) {
    term.wait_for("the dialog showing what is sent", STEP, |s| {
        s.contains("Create rewrite rules with AI")
            && s.contains("What is sent")
            && s.contains("Sent to: Fake agent")
            && s.contains("[ ] Send the scope")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_palette_row_opens_the_dialog_and_the_answer_fills_the_workbench_without_rewriting() {
    let repo = fixture();
    let before = refs(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-rules-ai", ai_config_toml(&agent));
    // Opening anything asks no model.
    assert_eq!(agent_calls(&agent), 0);

    term.send_keys("C-p").unwrap();
    term.send_text("create rules").unwrap();
    term.wait_for("the palette row, with its sparkle", STEP, |s| {
        s.contains("\u{2726} Rewrite: Create Rules with AI...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_dialog(&term);
    assert_eq!(agent_calls(&agent), 0, "the dialog alone sends nothing");

    // Two lines (Enter is a new line), sent with Ctrl+S.
    term.send_text("replace old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("by new@example.com").unwrap();
    term.wait_for("both lines in the field", STEP, |s| {
        s.contains("replace old@example.com") && s.contains("by new@example.com")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0, "Enter did not send");
    term.send_keys("C-s").unwrap();
    let screen = term
        .wait_for("the result list: what was left out, and why", STEP, |s| {
            s.contains("Left out (the valid rules were kept):")
                && s.contains("2. date_shift: a shift of zero changes nothing")
                && s.contains("No date limit exists.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Nothing is previewed or applied"),
        "{}",
        screen.text()
    );
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(refs(&repo), before, "nothing was rewritten");

    // Closing the list leaves the workbench holding the valid rule, with no
    // preview and Apply unavailable.
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the rule in the workbench", STEP, |s| {
            s.contains("Old email")
                && s.contains("Rules prepared by the AI: 1")
                && !s.contains("Left out")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // No preview ran: the Preview pane still shows its idle text.
    assert!(
        screen.contains("The preview shows what would change before anything is touched."),
        "{}",
        screen.text()
    );
    assert_eq!(refs(&repo), before, "nothing was rewritten");
    assert_eq!(agent_calls(&agent), 1, "no second call: nothing validates");
}

#[test]
fn the_button_and_the_key_reach_the_dialog_and_cancel_leaves_everything_alone() {
    let repo = fixture();
    let before = refs(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-rules-ai-keys", ai_config_toml(&agent));
    open_by_chord(&mut term);

    // The keyboard twin.
    term.send_keys("M-i").unwrap();
    wait_dialog(&term);
    // Esc cancels: no call, back to the workbench.
    term.send_keys("Escape").unwrap();
    term.wait_for("the workbench again", STEP, |s| {
        s.contains("Add a rule to start") && !s.contains("What is sent")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The button, by mouse.
    let (col, row) = term
        .wait_for_text("Create Rules with AI...", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 3, row).expect("click the button");
    wait_dialog(&term);
    // The cross closes it too.
    let (col, row) = term
        .wait_for_text("Create rewrite rules with AI", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r == row)
        .unwrap_or_else(|| panic!("no cross on the dialog's title row\n{}", screen.text()));
    let _ = col;
    term.click(cross.0, cross.1).expect("click the cross");
    term.wait_for("the dialog closed", STEP, |s| !s.contains("What is sent"))
        .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(agent_calls(&agent), 0, "cancelling asked no model");
    assert_eq!(refs(&repo), before);
}

/// With AI off (the default) nothing in the workbench offers it, the key
/// says why, and the palette hides the row.
#[test]
fn with_ai_off_nothing_offers_it() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-rewrite-rules-ai-none", String::new());
    open_by_chord(&mut term);
    let screen = term.screen();
    assert!(
        !screen.contains("Create Rules with AI"),
        "{}",
        screen.text()
    );
    term.send_keys("M-i").unwrap();
    term.wait_for("the notice saying AI is off", STEP, |s| {
        s.contains("AI is turned off")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.screen().contains("What is sent"));
    // The workbench owns the keyboard: close it (empty, so Esc closes) to
    // reach the palette.
    term.send_keys("Escape").unwrap();
    term.wait_for("the workbench closed", STEP, |s| {
        !s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("create rules").unwrap();
    term.wait_for("the query echoed", STEP, |s| s.contains("create rules"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !term.screen().contains("Create Rules with AI"),
        "{}",
        term.screen().text()
    );
}

/// "What is sent" is a pane of its own: collapsed when the dialog opens,
/// toggled by `Alt+W`, by `Enter` on its focused header and by a click, and
/// scrolled by keys, by the wheel and by dragging its own scrollbar.
#[test]
fn what_is_sent_starts_collapsed_and_its_pane_scrolls_by_key_wheel_and_scrollbar() {
    let repo = fixture();
    let before = refs(&repo);
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ANSWER);
    let (mut term, _config) = launch(&repo, "pty-rewrite-rules-ai-pane", ai_config_toml(&agent));
    term.send_keys("C-p").unwrap();
    term.send_text("create rules").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("\u{2726} Rewrite: Create Rules with AI...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_dialog(&term);

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
    assert_eq!(refs(&repo), before);
}
