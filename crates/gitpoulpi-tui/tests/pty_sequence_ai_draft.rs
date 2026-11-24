#![cfg(feature = "ai")]
//! Settings > Sequences "Create with AI" / "Edit with AI" through a pty
//! (`xtask::pty`): the real binary and daemon and a **scripted local agent**
//! (a program printing a canned answer) standing in for the model, so no
//! socket is opened. The description is typed, the proposal is read (what
//! was left out is listed with why, the diff of an edit is marked), and
//! NOTHING reaches `config.toml` before Accept: Reject leaves it alone.
//! A temp config dir per test; nothing is located by a hard-coded coordinate.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// A new sequence: three good steps and one a sequence may never hold.
const CREATE_ANSWER: &[&str] = &[
    "Here is the sequence:",
    "{\"name\": \"Nightly\", \"id\": \"nightly\", \"description\": \"Save my work\", \"steps\": [{\"run\": \"stage_all\"}, {\"run\": \"commit\"}, {\"run\": \"push\"}, {\"run\": \"discard_all_changes\"}], \"notes\": [\"No tag step was asked for.\"]}",
];

/// The change of an existing sequence: a fetch up front, the commit pauses.
const EDIT_ANSWER: &[&str] = &[
    "{\"name\": \"Daily\", \"id\": \"ignored\", \"steps\": [{\"run\": \"fetch\"}, {\"run\": \"stage_all\"}, {\"run\": \"commit\", \"pause\": \"always\"}]}",
];

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

fn parsed(config: &Path) -> toml::Table {
    stored(config).parse().unwrap_or_default()
}

fn sequences(config: &Path) -> Vec<toml::Value> {
    parsed(config)
        .get("sequences")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default()
}

fn runs(seq: &toml::Value) -> Vec<String> {
    seq.get("steps")
        .and_then(|s| s.as_array())
        .map(|steps| {
            steps
                .iter()
                .filter_map(|s| s.get("run").and_then(|r| r.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn wait_stored(term: &Terminal, config: &Path, what: &str, ok: impl Fn(&[toml::Value]) -> bool) {
    term.wait_for(what, STEP, |_| ok(&sequences(config)))
        .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config), term.text()));
}

/// The ✕ on the title row of the dialog titled `title`.
fn click_cross_of(term: &mut Terminal, title: &str) {
    let screen = term.screen();
    let (c, r) = screen.find_all(title)[0];
    let cross = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(cc, rr)| *rr == r && *cc > c)
        .expect("the dialog's cross on its title row");
    term.click(cross.0, cross.1).expect("click the cross");
}

/// Ctrl+S sends; the generic "What is sent" question of every AI action
/// comes first (Send is focused: Enter).
fn send_and_confirm(term: &mut Terminal) {
    term.send_keys("C-s").unwrap();
    term.wait_for("the What is sent question", STEP, |s| {
        s.contains("Don't show again for Sequence draft")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

fn open_settings_sequences_by_mouse(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Sequences");
}

#[test]
fn create_shows_the_proposal_with_what_was_left_out_reject_writes_nothing_and_accept_writes_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), CREATE_ANSWER);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sequence-ai-create");
    assert_eq!(agent_calls(&agent), 0, "starting asks no model");

    // The palette row, with its sparkle, opens Settings with the dialog.
    term.send_keys("C-p").unwrap();
    term.send_text("sequence: create").unwrap();
    term.wait_for("the palette row, with its sparkle", STEP, |s| {
        s.contains("\u{2726} Sequence: Create with AI...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Create a sequence with AI") && s.contains("[ ] Send context")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 0, "the dialog alone sends nothing");

    // Its cross closes an untouched dialog.
    click_cross_of(&mut term, "Create a sequence with AI");
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Create a sequence with AI") && s.contains("Sequences - global")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The Add menu's last row opens it again (the mouse path).
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    let (c, r) = term
        .wait_for_text("Create with AI...", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 3, r).expect("click Create with AI");
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Create a sequence with AI")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Two lines (Enter is a new line), sent with Ctrl+S.
    term.send_text("save my work").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("every night").unwrap();
    term.wait_for("both lines in the field", STEP, |s| {
        s.contains("save my work") && s.contains("every night")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 0, "Enter did not send");
    send_and_confirm(&mut term);
    term.wait_for("the proposal, with what was left out", STEP, |s| {
        s.contains("Proposed sequence")
            && s.contains("Name: Nightly")
            && s.contains("Steps (3)")
            && s.contains("Left out")
            && s.contains("step 4 `discard_all_changes`")
            && s.contains("No tag step was asked for.")
            && s.contains("Accept")
            && s.contains("Reject")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 1);
    assert!(sequences(config.path()).is_empty(), "nothing before Accept");

    // Reject: Accept, Edit before saving, then Reject (Right Right Enter).
    term.send_keys("Right").unwrap();
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the dialog gone", STEP, |s| {
        !s.contains("Proposed sequence")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(sequences(config.path()).is_empty(), "Reject wrote nothing");

    // Again, and this time Accept by the button.
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    let (c, r) = term
        .wait_for_text("Create with AI...", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 3, r).expect("click Create with AI");
    term.wait_for("the dialog", STEP, |s| {
        s.contains("Create a sequence with AI")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("save my work").unwrap();
    send_and_confirm(&mut term);
    term.wait_for("the proposal", STEP, |s| s.contains("Proposed sequence"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent_calls(&agent), 2);
    let (c, r) = term
        .wait_for_text("Accept", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Accept");
    wait_stored(&term, config.path(), "the accepted sequence", |seqs| {
        seqs.len() == 1
            && seqs[0].get("id").and_then(|v| v.as_str()) == Some("nightly")
            && runs(&seqs[0]) == ["stage_all", "commit", "push"]
    });
    // The list selects it.
    term.wait_for("the new sequence selected", STEP, |s| {
        s.contains("Editing sequence 1/1")
            && s.contains("Nightly")
            && !s.contains("Proposed sequence")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn edit_shows_the_diff_and_accept_replaces_the_sequence() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), EDIT_ANSWER);
    let seeded = format!(
        "{}\n[[sequences]]\nid = \"daily\"\nname = \"Daily\"\nscope = \"machine\"\nsteps = [{{ run = \"stage_all\" }}, {{ run = \"commit\" }}]\n",
        ai_config_toml(&agent)
    );
    std::fs::write(config.path().join("config.toml"), seeded).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sequence-ai-edit");
    open_settings_sequences_by_mouse(&mut term);

    let (c, r) = term
        .wait_for_text("Edit with AI", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 3, r).expect("click Edit with AI");
    term.wait_for("the edit dialog", STEP, |s| {
        s.contains("Edit a sequence with AI") && s.contains("Describe how to change \"Daily\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("fetch first and ask before the commit")
        .unwrap();
    send_and_confirm(&mut term);
    term.wait_for("the diff", STEP, |s| {
        s.contains("Proposed change")
            && s.contains("+ 1.")
            && s.contains("~ 3.")
            && s.contains("Pause: Always")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(agent_calls(&agent), 1);
    let before = sequences(config.path());
    assert_eq!(runs(&before[0]), ["stage_all", "commit"], "not written yet");

    term.send_keys("Enter").unwrap();
    wait_stored(&term, config.path(), "the replaced sequence", |seqs| {
        seqs.len() == 1
            && seqs[0].get("id").and_then(|v| v.as_str()) == Some("daily")
            && runs(&seqs[0]) == ["fetch", "stage_all", "commit"]
            && seqs[0]["steps"][2].get("pause").and_then(|p| p.as_str()) == Some("always")
    });
}

#[test]
fn with_ai_off_nothing_offers_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[[sequences]]\nid = \"daily\"\nname = \"Daily\"\nscope = \"machine\"\nsteps = [{ run = \"commit\" }]\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-sequence-ai-off");
    // The palette hides the row.
    term.send_keys("C-p").unwrap();
    term.send_text("create with").unwrap();
    std::thread::sleep(Duration::from_millis(600));
    let text = term.text();
    assert!(!text.contains("Create with AI"), "{text}");
    term.send_keys("Esc").unwrap();

    open_settings_sequences_by_mouse(&mut term);
    term.wait_for("the page", STEP, |s| s.contains("Edit steps"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.text().contains("Edit with AI"), "{}", term.text());
    let (c, r) = term
        .wait_for_text("+ Add", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click + Add");
    term.wait_for("the Add menu", STEP, |s| s.contains("Empty sequence"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.text().contains("Create with AI"), "{}", term.text());
}
