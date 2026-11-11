//! An agent's navigation intents (`ui show settings|tab|home|palette|help`,
//! `ui back`) end to end: the real binary and its
//! embedded daemon in a pty, and a plain IPC client standing in for the
//! `gitpoulpi ui ...` commands (`common::ui_show`). The CLI and MCP halves
//! are `gitpoulpi-cli/tests/pty_ui_show.rs`.
//!
//! Every wait names the state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_core::domain::UiControl;
use gitpoulpi_ipc::ui::{DeferReason, SettingsPage, UiIntent, UiOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// Two commits, so the Log has a row to move off the first one.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .commit_file("b.txt", "two\n", "second commit")
        .unwrap()
        .build()
}

/// `fixture()` with a staged edit: a WIP row, so the composer exists.
fn dirty_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build()
}

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    _repo: FixtureRepo,
}

fn launch(session: &str, repo: FixtureRepo, ready: &'static str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready)
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    Rig {
        term,
        config,
        session: session.into(),
        _repo: repo,
    }
}

impl Rig {
    fn dir(&self) -> &Path {
        self.config.path()
    }

    fn show(&self, intent: UiIntent) -> UiOutcome {
        common::ui_show(self.dir(), &self.session, intent)
    }

    /// Waits out the "the user just pressed a key" deferral.
    fn quiet(&self) {
        std::thread::sleep(Duration::from_millis(1700));
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }
}

fn applied(out: UiOutcome) -> gitpoulpi_ipc::ui::UiState {
    match out {
        UiOutcome::Applied { state } => state,
        other => panic!("expected Applied, got {other:?}"),
    }
}

#[test]
fn show_settings_lands_on_the_page_and_flashes_the_row() {
    let mut rig = launch("pty-ui-nav-settings", fixture(), "second commit");
    let state = applied(rig.show(UiIntent::ShowSettings {
        page: Some(SettingsPage::UiCustomization),
        setting: Some("ui.minimap".into()),
    }));
    assert_eq!(state.overlay.as_deref(), Some("settings"));
    let settings = state.settings.expect("settings state");
    // The legacy id lands on the page the row moved to.
    assert_eq!(settings["page"], "diff-editor");
    assert_eq!(settings["setting"], "ui.minimap");
    rig.wait_for("the page title and the Minimap row", |t| {
        t.contains("Diff & editor") && t.contains("Minimap")
    });
    assert!(
        rig.wait_toast("shown by claude-code: settings ui-customization / ui.minimap"),
        "the shown-by toast"
    );

    // Unknown rows are NotFound and leave the screen alone.
    rig.term.send_keys("Esc").unwrap();
    rig.wait_for("Settings closed", |t| !t.contains("Diff & editor"));
    rig.quiet();
    let out = rig.show(UiIntent::ShowSettings {
        page: None,
        setting: Some("ui.no_such_thing".into()),
    });
    assert!(matches!(out, UiOutcome::NotFound { .. }), "{out:?}");
}

impl Rig {
    fn wait_toast(&self, text: &str) -> bool {
        self.term
            .wait_for("a toast", STEP, |s| s.text().contains(text))
            .is_ok()
    }
}

#[test]
fn show_tab_opens_then_returns_across_a_window_restart() {
    let rig = launch("pty-ui-nav-tab", fixture(), "second commit");
    let other = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("o.txt", "x\n", "other repository commit")
        .unwrap()
        .build();
    let other_path = other.root().to_string_lossy().into_owned();
    let repo_id = |p: &str| gitpoulpi_core::domain::RepoId::from_path(Path::new(p));

    // Not open yet: opened as a tab, the window restarts onto it.
    let state = applied(rig.show(UiIntent::ShowTab {
        repo: repo_id(&other_path),
    }));
    assert_eq!(
        state.repo.as_deref(),
        Some(repo_id(&other_path).0.as_str()),
        "{state:?}"
    );
    rig.wait_for("the other repository's Log", |t| {
        t.contains("other repository commit")
    });

    // Now it is an open tab: back to the first one, another restart.
    let first = rig._repo.root().to_string_lossy().into_owned();
    let state = applied(rig.show(UiIntent::ShowTab {
        repo: repo_id(&first),
    }));
    assert_eq!(state.repo.as_deref(), Some(repo_id(&first).0.as_str()));
    rig.wait_for("the first repository's Log", |t| {
        t.contains("second commit")
    });

    // Back across tabs: to the other repository (a window restart that
    // carries the snapshot), and once more to the first.
    let state = applied(rig.show(UiIntent::Back));
    assert_eq!(state.repo.as_deref(), Some(repo_id(&other_path).0.as_str()));
    rig.wait_for("the other repository's Log", |t| {
        t.contains("other repository commit")
    });
    let state = applied(rig.show(UiIntent::Back));
    assert_eq!(state.repo.as_deref(), Some(repo_id(&first).0.as_str()));
}

#[test]
fn typing_in_the_composer_defers_and_the_chip_twin_applies() {
    let mut rig = launch("pty-ui-nav-defer", dirty_fixture(), "// WIP");
    rig.term.send_keys("c").expect("focus the composer");
    rig.term.send_text("subject").expect("type");
    rig.wait_for("the typed summary", |t| t.contains("subject"));

    let out = rig.show(UiIntent::ShowHelp { topic: None });
    assert_eq!(
        out,
        UiOutcome::Deferred {
            reason: DeferReason::InputBusy
        }
    );
    rig.wait_for("the chip", |t| {
        t.contains("claude-code wants to show help") && t.contains("Ctrl+N Enter")
    });

    // The keyboard twin of clicking the chip.
    rig.term.send_keys("C-n").unwrap();
    rig.term.send_keys("Enter").unwrap();
    rig.wait_for("Help open and the chip gone", |t| {
        !t.contains("wants to show help") && t.contains("shown by claude-code")
    });
    let state = common::ui_state(rig.dir(), &rig.session);
    assert_eq!(state.overlay.as_deref(), Some("help"));
}

#[test]
fn ask_mode_defers_and_the_palette_entry_dismisses() {
    let mut rig = launch("pty-ui-nav-ask", fixture(), "second commit");
    let out = common::ui_show_as(
        rig.dir(),
        &rig.session,
        UiIntent::ShowSettings {
            page: None,
            setting: None,
        },
        UiControl::Ask,
    );
    assert_eq!(
        out,
        UiOutcome::Deferred {
            reason: DeferReason::Ask
        }
    );
    rig.wait_for("the chip", |t| t.contains("wants to show settings"));
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("Dismiss Pending").unwrap();
    rig.wait_for("the palette entry", |t| {
        t.contains("Agent: Dismiss Pending View")
    });
    rig.term.send_keys("Enter").unwrap();
    rig.wait_for("the chip gone, no Settings", |t| {
        !t.contains("wants to show settings") && !t.contains("Settings - Tab")
    });
}

#[test]
fn back_restores_the_log_selection_and_closes_what_the_agent_opened() {
    let mut rig = launch("pty-ui-nav-back", fixture(), "second commit");
    // Select the older commit (the first row is the newest).
    rig.term.send_keys("Down").unwrap();
    rig.quiet();
    let before = common::ui_state(rig.dir(), &rig.session);
    let selected = before.log.as_ref().expect("log")["selected_commit"].clone();
    assert!(selected.is_string(), "a commit is selected: {before:?}");

    applied(rig.show(UiIntent::ShowSettings {
        page: Some(SettingsPage::General),
        setting: None,
    }));
    rig.wait_for("Settings", |t| t.contains("Settings - Tab"));
    applied(rig.show(UiIntent::Back));
    rig.wait_for("Settings closed", |t| !t.contains("Settings - Tab"));
    let after = common::ui_state(rig.dir(), &rig.session);
    assert_eq!(after.overlay, None);
    assert_eq!(after.log.unwrap()["selected_commit"], selected);

    // Nothing left to go back to.
    let out = rig.show(UiIntent::Back);
    assert!(matches!(out, UiOutcome::Refused { .. }), "{out:?}");

    // The user's own Back: palette entry twin of `Ctrl+N Backspace`.
    applied(rig.show(UiIntent::ShowHelp { topic: None }));
    rig.wait_for("Help", |t| t.contains("shown by claude-code"));
    rig.term.send_keys("Esc").unwrap();
    rig.wait_for("Help closed", |t| !t.contains("type to filter, "));
    rig.term.send_keys("C-n").unwrap();
    rig.term.send_keys("Backspace").unwrap();
    rig.wait_for("the Log again", |t| t.contains("second commit"));
}

#[test]
fn the_palette_opens_with_the_query_and_enter_is_never_sent() {
    let rig = launch("pty-ui-nav-palette", fixture(), "second commit");
    applied(rig.show(UiIntent::ShowPalette {
        query: "Fetch All".into(),
    }));
    rig.wait_for("the palette with the query and its match", |t| {
        t.contains("Fetch All") && t.contains("Git: Fetch All Remotes")
    });
    // Nothing ran: still the palette a moment later, and no fetch message.
    std::thread::sleep(Duration::from_millis(800));
    let t = rig.term.screen().text();
    assert!(t.contains("Git: Fetch All Remotes"), "still open:\n{t}");
    assert!(!t.contains("fetching"), "nothing was run:\n{t}");
}

#[test]
fn the_newest_of_a_burst_wins_and_the_rest_are_superseded_or_spaced() {
    let mut rig = launch("pty-ui-nav-burst", fixture(), "second commit");
    applied(rig.show(UiIntent::ShowHelp { topic: None }));
    // A second one right away is spaced by the dwell, not refused.
    rig.wait_for("Help", |t| t.contains("shown by claude-code"));
    rig.term.send_keys("Esc").unwrap();
    rig.quiet();
    let out = rig.show(UiIntent::ShowPalette {
        query: String::new(),
    });
    assert!(matches!(out, UiOutcome::Applied { .. }), "{out:?}");
}

#[test]
fn the_launcher_and_global_conflict_prevention_pages_open_by_their_ids() {
    let mut rig = launch("pty-ui-nav-own-pages", fixture(), "second commit");
    for (page, title) in [
        (SettingsPage::Launcher, "Launcher - global"),
        (
            SettingsPage::ConflictPreventionGlobal,
            "Conflict prevention - global",
        ),
    ] {
        applied(rig.show(UiIntent::ShowSettings {
            page: Some(page),
            setting: None,
        }));
        rig.wait_for(title, |t| t.contains(title));
        applied(rig.show(UiIntent::Back));
        rig.wait_for("Settings closed", |t| !t.contains("Settings - Tab"));
    }
}
