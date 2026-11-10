//! Closing, batch B (the ux-settings-and-close design, C3),
//! through a pty by `xtask::pty`: the real binary and daemon.
//!
//! - the issue form with typed text asks "Discard changes?" on Esc, and
//!   Discard closes it;
//! - the AI result popup survives a click inside, and closes on a click
//!   outside;
//! - the help overlay closes through its cross;
//! - the Undo history panel closes on a click outside.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::flow::{self, FlowHost, FlowHostOptions};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const CROSS: &str = "\u{2715}";

fn plain_repo(subject: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", subject)
        .unwrap()
        .build()
}

/// The cross painted on the row that carries `title`, with no "Esc" beside it.
fn cross_on_row_of(term: &Terminal, title: &str) -> (u16, u16) {
    let screen = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen
        .find(title)
        .unwrap_or_else(|| panic!("no {title:?} on screen:\n{}", term.text()));
    let line = screen.line(row);
    assert!(!line.contains("Esc"), "no Esc beside the cross: {line:?}");
    let col = screen
        .find_all(CROSS)
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("no cross on the {title:?} row: {line:?}"))
        .0;
    (col, row)
}

// ---- help overlay ---------------------------------------------------------

#[test]
fn the_help_overlay_closes_through_its_cross() {
    let repo = plain_repo("help close probe");
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-close-help")
        .size(200, 50)
        .ready("help close probe")
        .mouse_capture()
        .start();
    term.send_text("?").unwrap();
    term.wait_for("the help overlay", STEP, |s| {
        s.contains("Keybindings - type to filter")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = cross_on_row_of(&term, "Keybindings - type to filter");
    term.click(col + 1, row).unwrap();
    term.wait_for_text_to_go("Keybindings - type to filter", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

// ---- Undo history ---------------------------------------------------------

#[test]
fn the_undo_history_panel_closes_on_a_click_outside() {
    let repo = plain_repo("safety close probe");
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-close-safety")
        .size(200, 50)
        .ready("safety close probe")
        .mouse_capture()
        .start();
    // Alt+Z, the default key of "Edit: Undo History...".
    term.send_keys("M-z").unwrap();
    term.wait_for("the Undo history panel", STEP, |s| {
        s.contains("Undo history")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term
        .wait_for_stable(Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("Undo history").unwrap();
    assert!(
        !screen.line(row).contains("Esc"),
        "no Esc text beside the cross"
    );
    assert!(screen.find_all(CROSS).iter().any(|&(_, r)| r == row));
    // A corner of the screen is outside the panel.
    term.click(0, 0).unwrap();
    term.wait_for_text_to_go("Undo history", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

// ---- AI result popup ------------------------------------------------------

#[cfg(feature = "ai")]
#[test]
fn the_ai_popup_survives_a_click_inside_and_closes_on_a_click_outside() {
    use gitpoulpi_testkit::fake_agent::{ai_config_toml, write_fake_agent};
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["This change adds a second line to a.txt."]);
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-close-ai")
        .config(ai_config_toml(&agent))
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("initial commit") && s.contains("// WIP")
        })
        .mouse_capture()
        .start();
    term.send_keys("C-p").unwrap();
    term.send_text("explain working").unwrap();
    term.wait_for("the entry offered", STEP, |s| {
        s.contains("AI: Explain Working Changes")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the explanation popup", STEP, |s| {
        s.contains("Explain working changes") && s.contains("adds a second line to a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = cross_on_row_of(&term, "Explain working changes");
    // A click on the text inside the popup does not close it.
    let (tc, tr) = snap(&term).find("adds a second line").unwrap();
    term.click(tc + 2, tr).unwrap();
    term.send_text("x").unwrap();
    let screen = term
        .wait_for("the popup still open", STEP, |s| {
            s.contains("Explain working changes") && s.contains("adds a second line to a.txt")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.find(CROSS).is_some());
    // A click outside (a corner) closes it.
    term.click(0, 0).unwrap();
    term.wait_for_text_to_go("Explain working changes", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = (col, row);
    term.stop().unwrap();
}

// ---- issue form -----------------------------------------------------------

struct Rig {
    _repo: FixtureRepo,
    _host: FlowHost,
    term: Terminal,
}

/// A repository whose ISSUES source is the mock host (no network).
fn issue_rig(session: &str) -> Rig {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let host = flow::start(FlowHostOptions::new(repo.remote_path().unwrap())).expect("host");
    let state = repo.state_dir();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[issues_source]\nintegration = \"h\"\nproject = \"o/p\"\n\n\
         [pulls_source]\nintegration = \"h\"\nproject = \"o/p\"\n",
    )
    .unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[ui]\ngraph_renderer = \"text\"\n\n[[integrations]]\nid = \"h\"\n\
             kind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{}\"\n\
             token_ref = \"integration:h\"\nname = \"h\"\nuser = {{ login = \"mara\" }}\n",
            host.port
        ),
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_TOKEN_H", "test-token")
        .mouse_capture()
        .spawn();
    std::mem::forget(config);
    Rig {
        _repo: repo,
        _host: host,
        term,
    }
}

#[test]
fn a_typed_issue_form_asks_on_esc_and_discard_closes_it() {
    let mut rig = issue_rig("pty-close-issue");
    let term = &mut rig.term;
    term.wait_for("the log", STEP, |s| s.contains("initial commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Driven as a small state machine, one step per poll: the palette is
    // opened, filled, then its entry run; and if the form is still not there
    // (the entry's action can be swallowed while the hosting state loads on a
    // busy machine) the palette is opened again.
    let mut typed = false;
    common::retry_until(
        term,
        "the New issue form",
        30,
        Duration::from_secs(3),
        |t| {
            let s = t.screen();
            if s.contains("New issue") {
                return;
            }
            if s.contains("Command Palette") {
                if !typed {
                    t.send_text("Hosting: New Issue").unwrap();
                    typed = true;
                } else if s.contains("Hosting: New Issue...") {
                    t.send_keys("Enter").unwrap();
                    typed = false;
                }
            } else {
                typed = false;
                t.send_keys("C-p").unwrap();
            }
        },
        |s| s.contains("New issue"),
    );
    // A cross, no Esc hint.
    cross_on_row_of(term, "New issue");
    term.send_text("A typed title").unwrap();
    term.wait_for("the typed title", STEP, |s| s.contains("A typed title"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the discard question", STEP, |s| {
        s.contains("Discard changes?") && s.contains("Keep editing")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Keep editing first: the form is still there, with its text.
    term.send_keys("Escape").unwrap();
    term.wait_for("the form kept its text", STEP, |s| {
        !s.contains("Discard changes?") && s.contains("A typed title")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Again, then Discard (the second button).
    term.send_keys("Escape").unwrap();
    term.wait_for_text("Discard changes?", STEP).unwrap();
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the form closed", STEP, |s| {
        !s.contains("A typed title") && !s.contains("Discard changes?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
