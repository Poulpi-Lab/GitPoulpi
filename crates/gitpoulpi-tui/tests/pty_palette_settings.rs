//! The Settings fields as command-palette rows ("Settings: Toggle Minimap",
//! a choice's sub-list, a number's prompt), end to end. A temp
//! `GITPOULPI_CONFIG_DIR` per test; the file is read back, not trusted from
//! the screen alone.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str, config_toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains("Message")
        })
        .mouse_capture()
        .start()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "palette-settings-probe")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build()
}

fn config_text(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(dir.path().join("config.toml")).unwrap_or_default()
}

fn wait_config(dir: &tempfile::TempDir, what: &str, ok: impl Fn(&str) -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while std::time::Instant::now() < deadline {
        if ok(&config_text(dir)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("config.toml never showed {what}: {}", config_text(dir));
}

fn palette(term: &mut Terminal, query: &str, row: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette row", STEP, |s| s.contains(row))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// `Settings: Toggle Minimap` flips `ui.minimap`, persists it, and the row
/// then reads the new state.
#[test]
fn a_toggle_setting_flips_and_persists() {
    let repo = fixture();
    let (mut term, config) = launch(repo.root(), "pty-ps-toggle", "[ui]\nminimap = false\n");
    palette(
        &mut term,
        "settings: toggle minimap",
        "Settings: Toggle Minimap",
    );
    // The row shows the current state where a shortcut would be.
    term.wait_for("the off state on the row", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Toggle Minimap") && l.contains("[ ]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_config(&config, "minimap on", |t| t.contains("minimap = true"));
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("ui.minimap = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Again: the row now reads on, and Enter turns it off.
    palette(
        &mut term,
        "settings: toggle minimap",
        "Settings: Toggle Minimap",
    );
    term.wait_for("the on state on the row", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Toggle Minimap") && l.contains("[x]"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_config(&config, "minimap off", |t| t.contains("minimap = false"));
    term.stop().unwrap();
}

/// A choice opens a sub-list of its values in the palette itself, the
/// current one marked; picking one writes it.
#[test]
fn a_choice_setting_opens_its_sub_list() {
    let repo = fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-ps-choice",
        "[ui]\ndiff_layout = \"inline\"\n",
    );
    palette(
        &mut term,
        "settings: default layout",
        "Settings: Default layout...",
    );
    // The values are not in the list until the setting is opened.
    assert!(
        !term.screen().contains("Settings: Default layout: "),
        "{}",
        term.text()
    );
    term.send_keys("Enter").unwrap();
    term.wait_for("the sub-list, the current choice marked", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Default layout: ") && l.contains("current"))
            && s.text()
                .lines()
                .any(|l| l.contains("Settings: Default layout: "))
            && !s.contains("Settings: Default layout...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("split").unwrap();
    term.wait_for("only the split choice", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Default layout: ") && l.to_lowercase().contains("split"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_config(&config, "the split layout", |t| {
        t.contains("diff_layout = \"split\"")
    });
    term.stop().unwrap();
}

/// A number opens a prompt prefilled with its value; a refused value keeps
/// the prompt open with the reason, an accepted one is written.
#[test]
fn a_number_setting_goes_through_the_prompt_and_refuses_bad_values() {
    let repo = fixture();
    let (mut term, config) = launch(
        repo.root(),
        "pty-ps-number",
        "[ui]\nmouse_scroll_lines = 3\n",
    );
    palette(
        &mut term,
        "settings: mouse wheel",
        "Settings: Mouse wheel lines...",
    );
    term.send_keys("Enter").unwrap();
    term.wait_for("the prompt prefilled with 3", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Mouse wheel lines: 3"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Not a number.
    term.send_keys("BSpace").unwrap();
    term.send_text("abc").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the refusal in the prompt", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Mouse wheel lines [") && l.contains("abc"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Outside the range.
    for _ in 0..3 {
        term.send_keys("BSpace").unwrap();
    }
    term.send_text("9999").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the refusal for the range", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("Settings: Mouse wheel lines [") && l.contains("9999"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        config_text(&config).contains("mouse_scroll_lines = 3"),
        "a refused value changed the file: {}",
        config_text(&config)
    );
    // A good one.
    for _ in 0..4 {
        term.send_keys("BSpace").unwrap();
    }
    term.send_text("5").unwrap();
    term.send_keys("Enter").unwrap();
    wait_config(&config, "5 lines", |t| t.contains("mouse_scroll_lines = 5"));
    term.stop().unwrap();
}

/// A setting that needs its page opens Settings on it.
#[test]
fn a_page_setting_opens_settings_on_its_row() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ps-page", "");
    palette(
        &mut term,
        "settings: terminal for shortcuts",
        "Settings: Terminal for shortcuts...",
    );
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings open on that row", STEP, |s| {
        s.contains("Settings - Tab")
            && s.contains("Terminal for shortcuts")
            && !s.contains("Unstaged")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
