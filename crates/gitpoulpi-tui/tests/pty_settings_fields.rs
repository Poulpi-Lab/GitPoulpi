//! Settings text fields, profile editing, checkboxes and the AI presets,
//! end to end through a pty: the real binary, a temp config dir (never the
//! developer's own), no network and no real AI.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

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
        .size(200, 110)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains(LOG_HEADER))
        .mouse_capture()
        .spawn()
}

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

/// Double clicks the first `needle` once its position has stopped moving: a
/// popup is laid out against the height it finds free, so its first frame can
/// differ from the next. A searchable popup row is picked by the second click
/// (one click only selects).
fn double_click_when_settled(term: &mut Terminal, needle: &str) {
    double_click_when_settled_not_on(term, needle, "\u{0}");
}

/// [`double_click_when_settled`], skipping screen rows that contain `avoid`.
fn double_click_when_settled_not_on(term: &mut Terminal, needle: &str, avoid: &str) {
    let mut last = None;
    for _ in 0..50 {
        let screen = term.screen();
        let at = screen
            .find_all(needle)
            .into_iter()
            .find(|&(_, row)| !screen.line(row).contains(avoid));
        if let (Some((c, r)), true) = (at, at == last) {
            term.double_click(c, r).unwrap();
            return;
        }
        last = at;
        std::thread::sleep(Duration::from_millis(150));
    }
    panic!("{needle:?} never settled:\n{}", term.text());
}

/// Sidebar page `title` (clicked by name), then into its rows.
fn go_to_page(term: &mut Terminal, title: &str) {
    common::settings_page(term, title);
    term.send_keys("Enter").unwrap();
}

#[test]
fn leaving_a_text_field_commits_it_and_esc_cancels() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-blur");
    open_settings(&mut term);
    term.wait_for("the General page", STEP, |s| {
        s.contains("Default branch name") && s.contains("Default clone folder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Type in the branch-name field, then click the next text field's row:
    // the first value is kept and saved, the second opens for editing.
    let (c, r) = term
        .wait_for_text("Default branch name", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the field open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("End").unwrap();
    term.send_text("-x").unwrap();
    term.send_keys("Left").unwrap();
    term.send_text("Y").unwrap();
    term.wait_for("the typed text", STEP, |s| s.contains("main-Yx"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Default clone folder", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("saved on leaving", STEP, |_| {
        stored(config.path()).contains("default_branch_name = \"main-Yx\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
    assert!(term.screen().contains("main-Yx"), "{}", term.text());

    // Esc cancels the edit now open on the clone folder.
    term.send_text("zzz").unwrap();
    term.wait_for("zzz typed", STEP, |s| s.contains("zzz"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("zzz gone, Settings still open", STEP, |s| {
        !s.contains("zzz") && s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!stored(config.path()).contains("zzz"));
}

#[test]
fn a_refused_value_stays_in_the_field_and_the_click_is_spent() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-refused");
    open_settings(&mut term);
    common::settings_page(&mut term, "Log & graph");
    let (c, r) = term
        .wait_for_text("Initial commits in graph", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the field open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("abc").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("still editing, text kept", STEP, |s| {
        s.contains("abc") && s.contains("\u{2588}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!stored(config.path()).contains("abc"));
}

#[test]
fn the_diff_and_editor_page_has_word_wrap_as_a_checkbox_and_the_merge_row_after_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-editor");
    open_settings(&mut term);
    go_to_page(&mut term, "Diff & editor");
    let screen = term
        .wait_for("the Diff & editor page", STEP, |s| {
            s.contains("Diff & editor - global")
                && s.find("Word wrap")
                    .is_some_and(|(_, r)| s.line(r).contains("[ ]"))
                && s.contains("Merge editor")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, wrap) = screen.find("Word wrap").unwrap();
    assert!(
        screen.line(wrap).contains("[ ]"),
        "a checkbox: {}",
        screen.line(wrap)
    );
    let (_, merge) = screen.find("Merge editor").unwrap();
    assert!(
        merge > wrap,
        "the merge editor row comes after the editor rows"
    );
    // Space ticks it.
    let (c, r) = screen.find("Word wrap").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("ticked and saved", STEP, |s| {
        s.line(wrap).contains("[x]") && stored(config.path()).contains("diff_word_wrap = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
}

const AI_CONFIG: &str = r#"
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

#[test]
fn the_first_ai_profile_stays_editable_after_adding_a_second() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), AI_CONFIG).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-ai-profiles");
    open_settings(&mut term);
    go_to_page(&mut term, "AI");
    common::settings_tab(&mut term, "Profiles (connections)");
    term.wait_for("the AI page with its profile list", STEP, |s| {
        s.contains("+ Add") && s.contains("Alpha")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Add a second profile from the list's toolbar.
    // (Not a "[+ Add ...]" chip lower on the page: the list is the left pane.)
    let (c, r) = term
        .screen()
        .find_all("+ Add")
        .into_iter()
        .find(|(c, _)| *c < 40)
        .unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("two profiles stored", STEP, |_| {
        stored(config.path()).matches("[[ai.profiles]]").count() == 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));

    // The list shows both; pick the first row (the list is the left pane).
    let (c, r) = term
        .screen()
        .find_all("Alpha")
        .into_iter()
        .find(|(c, _)| *c < 40)
        .expect("Alpha in the list");
    term.click(c + 1, r).unwrap();
    term.wait_for("Alpha selected", STEP, |s| {
        s.contains("Command")
            && s.contains("claude")
            && s.find("Name")
                .is_some_and(|(_, r)| s.line(r).contains("Alpha"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap();
    // Rename it: the Name row now edits Alpha.
    let (c, r) = term
        .wait_for_text("Name", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the name open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("2").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Alpha renamed, the other kept", STEP, |_| {
        let toml = stored(config.path());
        toml.contains("name = \"Alpha2\"") && toml.matches("[[ai.profiles]]").count() == 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
}

#[test]
fn picking_an_agent_preset_writes_its_arguments_and_the_model_flag() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        AI_CONFIG
            .replace("command = \"claude\"", "command = \"\"")
            .replace("args = [\"-p\"]", "args = []"),
    )
    .unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-ai-agent");
    open_settings(&mut term);
    go_to_page(&mut term, "AI");
    common::settings_tab(&mut term, "Profiles (connections)");
    term.wait_for("the profile editor", STEP, |s| s.contains("Agent type"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term
        .wait_for_text("Agent type", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the agent list", STEP, |s| {
        s.contains("OpenAI Codex CLI  (codex)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    double_click_when_settled(&mut term, "OpenAI Codex CLI  (codex)");
    term.wait_for("codex's headless arguments stored", STEP, |_| {
        let t = stored(config.path());
        t.contains("command = \"codex\"")
            && t.contains("\"exec\"")
            && t.contains("harness = \"codex\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));

    // Back to Claude Code, then its model: haiku.
    let (c, r) = term
        .wait_for_text("Agent type", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the agent list", STEP, |s| {
        s.contains("Claude Code  (claude)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    double_click_when_settled(&mut term, "Claude Code  (claude)");
    term.wait_for("claude's -p stored", STEP, |_| {
        let t = stored(config.path());
        t.contains("command = \"claude\"") && t.contains("\"-p\"") && !t.contains("\"exec\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
    let (c, r) = term
        .wait_for_text("Model", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the model list", STEP, |s| s.contains("claude-opus-5-5"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The popup's `haiku` row (the help line above also says `--model haiku`).
    double_click_when_settled_not_on(&mut term, "haiku", "--model");
    term.wait_for("--model haiku stored", STEP, |_| {
        let t = stored(config.path());
        t.contains("\"--model\"") && t.contains("\"haiku\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
}

/// The maintainer's two asks: the type of agent is separate from the exact
/// command (an alias stays an alias and keeps the type's suggestions), and
/// a profile carries environment variables for its process only.
#[test]
fn an_alias_command_keeps_the_agent_type_and_env_vars_are_stored_per_profile() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), AI_CONFIG).unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-ai-env");
    open_settings(&mut term);
    go_to_page(&mut term, "AI");
    common::settings_tab(&mut term, "Profiles (connections)");
    term.wait_for("the profile editor", STEP, |s| {
        s.contains("Agent type") && s.contains("Command")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Type the exact command: an alias that names no known agent.
    let (c, r) = term
        .wait_for_text("Command", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the command open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..12 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("my-work-claude").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the alias stored with its type", STEP, |_| {
        let t = stored(config.path());
        t.contains("command = \"my-work-claude\"") && t.contains("harness = \"claude\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
    // The suggestions of the type are still offered.
    term.wait_for("Claude's options and model still offered", STEP, |s| {
        s.contains("Argument options") && s.contains("Add an environment variable")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Add a suggested variable.
    let (c, r) = term
        .wait_for_text("Add an environment variable", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the suggestions", STEP, |s| s.contains("ANTHROPIC_MODEL"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    double_click_when_settled(&mut term, "ANTHROPIC_MODEL");
    term.wait_for("the variable stored in the profile", STEP, |_| {
        let t = stored(config.path());
        t.contains("[ai.profiles.env]") && t.contains("ANTHROPIC_MODEL = \"haiku\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));

    // Its value is a real text field: commit on Enter.
    term.wait_for("its row on the page", STEP, |s| {
        s.contains("Env ANTHROPIC_MODEL")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The value is a button of the row's small table; pressing it opens the
    // text field.
    let (_, r) = term
        .wait_for_text("Env ANTHROPIC_MODEL", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let line = term.screen().line(r);
    let value_c = line
        .find("haiku")
        .map(|byte| line[..byte].chars().count())
        .expect("the value on the row") as u16;
    term.click(value_c, r).unwrap();
    term.wait_for("the value open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..10 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("sonnet").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the value stored", STEP, |_| {
        stored(config.path()).contains("ANTHROPIC_MODEL = \"sonnet\"")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", stored(config.path()), term.text()));
}

/// L1: a value is a band at the page's value column. A click on the band
/// (not on the label) opens it for editing, a second click inside it puts
/// the caret under the pointer, and the dot leader is gone.
#[test]
fn a_click_on_a_value_band_edits_it_and_the_caret_lands_where_clicked() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-fields-band");
    open_settings(&mut term);
    term.wait_for("the General page", STEP, |s| {
        s.contains("Default branch name") && s.contains("Default clone folder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.text().contains("\u{00B7}\u{00B7}\u{00B7}"),
        "no dot leader\n{}",
        term.text()
    );
    let (label_c, row) = term
        .wait_for_text("Default branch name", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // The value "main" sits right of the label, in its band.
    let line = term.screen().line(row);
    let value_c = line
        .find("main")
        .map(|byte| line[..byte].chars().count())
        .expect("the value on the label's row") as u16;
    assert!(value_c > label_c + 18, "{line}");
    term.click(value_c + 2, row).unwrap();
    term.wait_for("the band open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Second click on the `i` of "main": the caret goes before it.
    term.click(value_c + 2, row).unwrap();
    term.send_text("Z").unwrap();
    term.wait_for("Z typed where clicked", STEP, |s| s.contains("maZin"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
