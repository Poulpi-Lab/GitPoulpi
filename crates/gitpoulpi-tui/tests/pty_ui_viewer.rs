//! The TUI as a viewer for an agent's `ui` intents, end to end: the real
//! binary and its embedded daemon in a pty, and a plain IPC client standing
//! in for `gitpoulpi ui ...` / the MCP tools (the commands layer is covered
//! by `gitpoulpi-cli/tests/ui.rs`).
//!
//! Rules are `pty_ui.rs`'s: every wait names the state it waits for.

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{OperationKind, UiControl};
use gitpoulpi_ipc::ui::{
    UiIntent, UiOrigin, UiOutcome, UiQueryArgs, UiShowArgs, UiState, ViewerInfo,
};
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

struct Rig {
    term: Terminal,
    session: String,
    port_file: PathBuf,
    rt: tokio::runtime::Runtime,
    _config: tempfile::TempDir,
    _repo: FixtureRepo,
}

fn launch(session: &str, config: &str) -> Rig {
    let repo = fixture();
    let (term, dir) = common::launch(repo.root())
        .session(session)
        .config(config)
        .no_exit_pause()
        .ready("initial commit")
        .start();
    Rig {
        term,
        session: session.into(),
        port_file: dir.path().join("sessions").join(format!("{session}.port")),
        rt: tokio::runtime::Runtime::new().unwrap(),
        _config: dir,
        _repo: repo,
    }
}

impl Rig {
    /// One request on a fresh connection, as `gitpoulpi ui ...` would send it.
    fn call(&self, command: OperationKind, args: impl serde::Serialize) -> serde_json::Value {
        self.rt.block_on(async {
            let mut client = Client::connect(
                &gitpoulpi_ipc::discovery::endpoint(&self.port_file)
                    .expect("the session published its discovery files"),
            )
            .await
            .expect("connect");
            let response = client
                .call(&Request {
                    id: 1,
                    session: self.session.clone(),
                    repo: None,
                    command,
                    args: serde_json::to_value(args).unwrap(),
                })
                .await
                .expect("call");
            match response.result {
                Ok(v) => v,
                Err(e) => serde_json::json!({ "error": e.message }),
            }
        })
    }

    fn viewers(&self) -> Vec<ViewerInfo> {
        let v = self.call(OperationKind::UiQuery, UiQueryArgs::Viewers);
        serde_json::from_value(v).expect("a viewer list")
    }

    fn state(&self, include_text: bool) -> UiState {
        let v = self.call(
            OperationKind::UiQuery,
            UiQueryArgs::State {
                viewer: None,
                include_text,
            },
        );
        serde_json::from_value(v.clone()).unwrap_or_else(|_| panic!("a state, got {v}"))
    }

    fn show(&self, intent: UiIntent, control: UiControl) -> UiOutcome {
        let v = self.call(
            OperationKind::UiShow,
            UiShowArgs {
                viewer: None,
                intent,
                origin: UiOrigin {
                    client: "claude-code".into(),
                    label: None,
                    control,
                },
                timeout_ms: Some(10_000),
            },
        );
        serde_json::from_value(v.clone()).unwrap_or_else(|_| panic!("an outcome, got {v}"))
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn the_running_tui_is_a_listed_viewer_and_answers_state_and_text() {
    let rig = launch("pty-ui-viewer-state", "");
    wait_until("the window to attach", || !rig.viewers().is_empty());
    let viewers = rig.viewers();
    assert_eq!(viewers.len(), 1, "{viewers:?}");
    assert!(viewers[0].id.0.starts_with("tui-"));
    assert!(viewers[0].agent_control);

    let state = rig.state(false);
    assert_eq!(state.central.as_deref(), Some("log"));
    assert_eq!(state.focus.as_deref(), Some("log"));
    assert_eq!(state.right_panel.as_deref(), Some("commit_detail"));
    assert!(!state.input_busy);
    assert_eq!(state.tabs.len(), 1);
    assert!(state.tabs[0].active);
    assert!(state.text.is_none());

    let text = rig.state(true).text.expect("text was asked for");
    assert!(
        text.iter().any(|l| l.contains("initial commit")),
        "the Log row is in the visible text: {text:?}"
    );
}

#[test]
fn notify_shows_a_sanitized_toast_in_any_control_mode() {
    let rig = launch("pty-ui-viewer-notify", "");
    wait_until("the window to attach", || !rig.viewers().is_empty());
    let out = rig.show(
        UiIntent::Notify {
            message: "build is \u{1b}[31mgreen".into(),
        },
        UiControl::Ask,
    );
    assert!(matches!(out, UiOutcome::Applied { .. }), "{out:?}");
    rig.wait_for("the toast", |t| {
        t.contains("\u{2726} claude-code: build is [31mgreen")
    });
}

#[test]
fn ask_mode_offers_a_chip_the_user_accepts_or_dismisses() {
    let mut rig = launch("pty-ui-viewer-ask", "");
    wait_until("the window to attach", || !rig.viewers().is_empty());

    let out = rig.show(UiIntent::ShowHelp { topic: None }, UiControl::Ask);
    assert!(matches!(out, UiOutcome::Deferred { .. }), "{out:?}");
    rig.wait_for("the chip", |t| {
        t.contains("claude-code wants to show help") && t.contains("Esc: dismiss")
    });
    rig.term.send_keys("Esc").unwrap();
    rig.wait_for("the chip to go", |t| !t.contains("wants to show help"));

    rig.show(UiIntent::ShowHelp { topic: None }, UiControl::Ask);
    rig.wait_for("the chip again", |t| t.contains("wants to show help"));
    rig.term.send_keys("C-n").unwrap();
    rig.term.send_keys("Enter").unwrap();
    rig.wait_for("the help the user accepted", |t| {
        t.contains("shown by claude-code: help") && !t.contains("wants to show help")
    });
}

#[test]
fn a_highlight_with_no_diff_or_file_open_is_refused() {
    let rig = launch("pty-ui-viewer-auto", "");
    wait_until("the window to attach", || !rig.viewers().is_empty());
    let out = rig.show(
        UiIntent::Highlight {
            range: gitpoulpi_ipc::ui::LineRange {
                start_line: 1,
                end_line: 2,
                start_col: None,
                end_col: None,
            },
            scroll: true,
        },
        UiControl::Auto,
    );
    match out {
        UiOutcome::Refused { reason } => assert!(reason.starts_with("no_text_view"), "{reason}"),
        other => panic!("expected Refused, got {other:?}"),
    }
}

#[test]
fn the_master_switch_off_means_no_viewer() {
    let rig = launch(
        "pty-ui-viewer-off",
        "[ui]\nallow_agent_view_control = false\n",
    );
    // Give a (wrongly) attaching window time to show up in the registry.
    std::thread::sleep(Duration::from_secs(2));
    assert!(rig.viewers().is_empty());
    let out = rig.show(
        UiIntent::Notify {
            message: "hi".into(),
        },
        UiControl::Auto,
    );
    assert!(matches!(out, UiOutcome::NoViewer { .. }), "{out:?}");
}

#[test]
fn with_two_windows_an_intent_lands_in_the_one_with_focus() {
    let repo = fixture();
    let session = "pty-ui-two-windows";
    let (mut first, config) = common::launch(repo.root())
        .session(session)
        .no_exit_pause()
        .ready("initial commit")
        .start();
    let mut second = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .no_exit_pause()
        .ready("initial commit")
        .spawn();
    let wait_viewers = |n: usize| {
        wait_until("both windows attached", || {
            let v = common::daemon_call(
                config.path(),
                session,
                OperationKind::UiQuery,
                UiQueryArgs::Viewers,
            );
            v.as_array().is_some_and(|a| a.len() == n)
        })
    };
    wait_viewers(2);
    let palette = |query: &str| UiIntent::ShowPalette {
        query: query.into(),
    };
    let wait = |term: &Terminal, what: &str, needle: &'static str| {
        term.wait_for(what, STEP, |s| s.text().contains(needle))
            .unwrap_or_else(|e| panic!("{e}"));
    };
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));

    // Terminal focus events: the first window has it.
    second.send_text("\x1b[O").unwrap();
    first.send_text("\x1b[I").unwrap();
    wait_until("the daemon knows the first window is focused", || {
        let v = common::daemon_call(
            config.path(),
            session,
            OperationKind::UiQuery,
            UiQueryArgs::Viewers,
        );
        v.as_array()
            .is_some_and(|a| a.iter().filter(|w| w["focused"] == true).count() == 1)
    });
    let out = common::ui_show(config.path(), session, palette("zzfirst"));
    assert!(matches!(out, UiOutcome::Applied { .. }), "{out:?}");
    wait(&first, "the palette query in the focused window", "zzfirst");
    assert!(!second.text().contains("zzfirst"), "the other is untouched");

    // Focus moves to the second window (a modal in the first would defer an
    // intent anyway, so close its palette first).
    first.send_keys("Esc").unwrap();
    first.send_text("\x1b[O").unwrap();
    second.send_text("\x1b[I").unwrap();
    std::thread::sleep(Duration::from_millis(1700));
    wait_until("the first window's palette is closed", || {
        !first.text().contains("zzfirst")
    });
    let out = common::ui_show(config.path(), session, palette("zzsecond"));
    assert!(matches!(out, UiOutcome::Applied { .. }), "{out:?}");
    wait(
        &second,
        "the palette query in the newly focused window",
        "zzsecond",
    );
    assert!(!first.text().contains("zzsecond"), "the other is untouched");
}
