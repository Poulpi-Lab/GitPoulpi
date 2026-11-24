//! `gitpoulpi ui ...` end to end: the real `gitpoulpi` binary against a real
//! daemon and a fake viewer (a TUI stand-in speaking the IPC protocol). Asserts
//! on the deserialized JSON and on what the viewer received, so a refused
//! argument is proven never to have reached the window.

use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::{OperationKind, RepoId, UiControl};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::ui::{
    DiffAt, UiIntent, UiOrigin, UiOutcome, UiReportArgs, UiReportBody, UiState, ViewerId,
    ViewerInfo,
};
use gitpoulpi_ipc::{Client, Event, Request, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

type Seen = Arc<Mutex<Vec<(UiIntent, UiOrigin)>>>;

struct Harness {
    config: tempfile::TempDir,
    session: String,
    repo: FixtureRepo,
    seen: Seen,
    texts: Arc<Mutex<Vec<bool>>>,
    #[cfg_attr(not(unix), allow(dead_code))]
    endpoint: gitpoulpi_ipc::Endpoint,
    /// Keeps the daemon and the fake viewer alive.
    _rt: tokio::runtime::Runtime,
}

/// A fixture repo whose last commit touches `src/a.rs`, a daemon published
/// for `session`, and (when `viewer`) a fake window answering every intent
/// with `answer`.
fn harness(session: &str, viewer: bool, answer: UiOutcome, config_toml: &str) -> Harness {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .commit_file("src/a.rs", "fn a() {}\n", "add a")
        .unwrap()
        .build();
    harness_on(session, repo, viewer, answer, config_toml)
}

/// [`harness`] on a repository of the caller's.
fn harness_on(
    session: &str,
    repo: FixtureRepo,
    viewer: bool,
    answer: UiOutcome,
    config_toml: &str,
) -> Harness {
    let config = tempfile::tempdir().unwrap();
    if !config_toml.is_empty() {
        std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    let port_file = ConfigPaths::for_dir(config.path()).ipc_port_file(session);
    std::fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    let backend = gitpoulpi_daemon::backend_for(gitpoulpi_config::DefaultBackend::System);
    let handle = rt
        .block_on(gitpoulpi_daemon::Daemon::new(session).spawn(backend))
        .unwrap();
    let endpoint = handle.endpoint();
    gitpoulpi_ipc::discovery::publish(&port_file, &endpoint).unwrap();
    std::mem::forget(handle); // lives as long as the runtime
    let seen: Seen = Arc::default();
    let texts: Arc<Mutex<Vec<bool>>> = Arc::default();
    if viewer {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        rt.spawn(fake_viewer(
            endpoint.clone(),
            repo.repo_ref().id,
            answer,
            Arc::clone(&seen),
            Arc::clone(&texts),
            ready_tx,
        ));
        ready_rx.recv().unwrap();
    }
    Harness {
        config,
        session: session.to_string(),
        repo,
        seen,
        texts,
        endpoint,
        _rt: rt,
    }
}

async fn fake_viewer(
    endpoint: gitpoulpi_ipc::Endpoint,
    repo: RepoId,
    answer: UiOutcome,
    seen: Seen,
    texts: Arc<Mutex<Vec<bool>>>,
    ready: std::sync::mpsc::Sender<()>,
) {
    let mut stream = Client::connect(&endpoint).await.unwrap().into_stream();
    let info = ViewerInfo {
        id: ViewerId("fake-viewer".into()),
        pid: 4242,
        active_repo: Some(repo.clone()),
        tabs: vec![repo],
        focused: Some(true),
        agent_control: true,
        last_focus_at: None,
    };
    write_message(
        &mut stream,
        &Request {
            id: 1,
            session: "t".into(),
            repo: None,
            command: OperationKind::UiAttach,
            args: serde_json::to_value(info).unwrap(),
        },
    )
    .await
    .unwrap();
    let mut next_id = 2;
    let mut attached = false;
    while let Ok(Some(message)) = read_message::<_, ServerMessage>(&mut stream).await {
        let body = match message {
            ServerMessage::Response(_) if !attached => {
                attached = true;
                let _ = ready.send(());
                continue;
            }
            ServerMessage::Event(Event::UiIntent {
                intent_id,
                intent,
                origin,
                ..
            }) => {
                seen.lock().unwrap().push((intent, origin));
                (
                    intent_id,
                    UiReportBody::Outcome {
                        outcome: answer.clone(),
                    },
                )
            }
            ServerMessage::Event(Event::UiStateRequest {
                query_id,
                include_text,
                ..
            }) => {
                texts.lock().unwrap().push(include_text);
                let state = UiState {
                    focus: Some("status".into()),
                    text: include_text.then(|| vec!["visible line".to_string()]),
                    ..UiState::default()
                };
                (query_id, UiReportBody::State { state })
            }
            _ => continue,
        };
        write_message(
            &mut stream,
            &Request {
                id: next_id,
                session: "t".into(),
                repo: None,
                command: OperationKind::UiReport,
                args: serde_json::to_value(UiReportArgs {
                    id: body.0,
                    body: body.1,
                })
                .unwrap(),
            },
        )
        .await
        .unwrap();
        next_id += 1;
    }
}

fn applied() -> UiOutcome {
    UiOutcome::Applied {
        state: UiState {
            central: Some("log".into()),
            ..UiState::default()
        },
    }
}

impl Harness {
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .arg("--json")
            .args(["--session", &self.session])
            .args(["--repo", &self.repo.root().to_string_lossy()])
            .arg("ui")
            .args(args)
            .current_dir(self.repo.root())
            .output()
            .unwrap()
    }

    /// Runs one command and returns (exit code, stdout JSON).
    fn json(&self, args: &[&str]) -> (i32, Value) {
        let output = self.run(args);
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{args:?}: stdout was not JSON ({e}): {} / {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().unwrap_or(-1), value)
    }

    fn head(&self) -> String {
        self.repo.git(&["rev-parse", "HEAD"]).trim().to_string()
    }

    fn received(&self) -> Vec<(UiIntent, UiOrigin)> {
        self.seen.lock().unwrap().clone()
    }
}

#[test]
fn show_commit_resolves_the_revision_and_reaches_the_window() {
    let h = harness("cli-ui-commit", true, applied(), "");
    let (code, out) = h.json(&["show", "commit", "HEAD", "--files", "--focus", "status"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["outcome"], "applied");
    assert_eq!(out["state"]["central"], "log");
    let seen = h.received();
    assert_eq!(seen.len(), 1);
    let UiIntent::ShowCommit {
        repo,
        commit,
        files,
        focus,
        ..
    } = &seen[0].0
    else {
        panic!("{:?}", seen[0])
    };
    assert_eq!(repo, &h.repo.repo_ref().id);
    assert_eq!(serde_json::to_value(commit).unwrap(), h.head());
    assert!(*files);
    assert_eq!(*focus, Some(gitpoulpi_ipc::ui::PaneId::Status));
    // Default mode is `ask`, and the caller is named "cli".
    assert_eq!(seen[0].1.control, UiControl::Ask);
    assert_eq!(seen[0].1.client, "cli");
}

#[test]
fn the_configured_ui_control_mode_travels_with_the_intent() {
    let h = harness(
        "cli-ui-auto",
        true,
        applied(),
        "[mcp]\nui_control = \"auto\"\n",
    );
    let (code, _) = h.json(&["show", "keys"]);
    assert_eq!(code, 0);
    assert_eq!(h.received()[0].1.control, UiControl::Auto);
}

#[test]
fn a_bad_revision_is_rejected_before_routing() {
    let h = harness("cli-ui-badrev", true, applied(), "");
    let (code, out) = h.json(&["show", "commit", "no-such-rev"]);
    assert_eq!(code, 1);
    assert_eq!(out["outcome"], "not_found");
    assert!(
        out["what"].as_str().unwrap().contains("no-such-rev"),
        "{out}"
    );
    let (code, out) = h.json(&["show", "commit", "HEAD", "--also", "nope"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    assert!(h.received().is_empty(), "nothing reached the window");
}

#[test]
fn a_bad_path_is_rejected_before_routing_and_a_good_one_reaches_the_window() {
    let h = harness("cli-ui-badpath", true, applied(), "");
    let (code, out) = h.json(&["show", "diff", "src/missing.rs"]);
    assert_eq!(code, 1);
    assert_eq!(out["outcome"], "not_found");
    assert!(out["what"].as_str().unwrap().contains("src/missing.rs"));
    let (code, out) = h.json(&["show", "file", "src/missing.rs", "--rev", "HEAD"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    assert!(h.received().is_empty());

    let (code, _) = h.json(&[
        "show",
        "diff",
        "src/a.rs",
        "--line",
        "1",
        "--split",
        "--whole-file",
        "--highlight",
        "1-2",
    ]);
    assert_eq!(code, 0);
    let (code, _) = h.json(&[
        "show", "file", "src/a.rs", "--rev", "HEAD", "--blame", "--line", "1", "--column", "4",
    ]);
    assert_eq!(code, 0);
    let seen = h.received();
    let UiIntent::ShowDiff {
        target,
        path,
        line,
        layout,
        whole_file,
        highlight,
        ..
    } = &seen[0].0
    else {
        panic!()
    };
    assert!(matches!(target, DiffAt::Commit(_)));
    assert_eq!(path, "src/a.rs");
    assert_eq!(*line, Some(1));
    assert_eq!(*layout, Some(gitpoulpi_ipc::ui::DiffLayout::Split));
    assert_eq!(*whole_file, Some(true));
    assert_eq!(highlight.unwrap().end_line, 2);
    assert!(matches!(
        seen[1].0,
        UiIntent::ShowFile {
            mode: gitpoulpi_ipc::ui::FileMode::Blame,
            column: Some(4),
            ..
        }
    ));
}

#[test]
fn no_window_attached_says_so_with_the_repository_and_exits_nonzero() {
    let h = harness("cli-ui-noviewer", false, applied(), "");
    let (code, out) = h.json(&[
        "show",
        "settings",
        "ui-customization",
        "--setting",
        "ui.minimap",
    ]);
    assert_eq!(code, 1);
    assert_eq!(out["outcome"], "no_viewer");
    let message = out["message"].as_str().unwrap();
    assert!(message.contains("No GitPoulpi window is open"), "{message}");
    assert!(
        message.contains(&*h.repo.root().to_string_lossy()),
        "names the repository: {message}"
    );
    // Human mode: the same sentence on stderr.
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", h.config.path())
        .args(["--session", &h.session, "ui", "show", "keys"])
        .current_dir(h.repo.root())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("No GitPoulpi window"));
    // `ui state` answers the same way; `ui viewers` is an empty list.
    let (code, out) = h.json(&["state"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("no_viewer")));
    let (code, out) = h.json(&["viewers"]);
    assert_eq!((code, out), (0, serde_json::json!([])));
}

#[test]
fn viewers_lists_the_window_and_viewer_selects_one() {
    let h = harness("cli-ui-viewer", true, applied(), "");
    let (code, out) = h.json(&["viewers"]);
    assert_eq!(code, 0);
    assert_eq!(out[0]["id"], "fake-viewer");
    assert_eq!(out[0]["pid"], 4242);
    assert_eq!(out[0]["focused"], true);

    let (code, _) = h.json(&["--viewer", "fake-viewer", "notify", "hello"]);
    assert_eq!(code, 0);
    let (code, out) = h.json(&["--viewer", "ghost", "notify", "hello"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("refused")));
    assert!(out["reason"].as_str().unwrap().contains("ghost"));
    assert_eq!(h.received().len(), 1);
}

#[test]
fn notify_strips_escape_sequences_and_deferral_exits_3() {
    let h = harness(
        "cli-ui-notify",
        true,
        UiOutcome::Deferred {
            reason: gitpoulpi_ipc::ui::DeferReason::Ask,
        },
        "",
    );
    let (code, out) = h.json(&["notify", "pwn\u{1b}[31med"]);
    assert_eq!(code, 3, "deferred: the user decides");
    assert_eq!(out["outcome"], "deferred");
    assert_eq!(out["reason"], "ask");
    assert_eq!(
        h.received()[0].0,
        UiIntent::Notify {
            message: "pwn[31med".into()
        }
    );
    let (code, out) = h.json(&["notify", "\u{1b}"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
}

#[test]
fn state_reads_the_window_and_text_is_opt_in() {
    let h = harness("cli-ui-state", true, applied(), "");
    let (code, out) = h.json(&["state"]);
    assert_eq!(code, 0);
    assert_eq!(out["focus"], "status");
    assert!(out["text"].is_null());
    let (_, out) = h.json(&["state", "--text"]);
    assert_eq!(out["text"][0], "visible line");
    assert_eq!(*h.texts.lock().unwrap(), vec![false, true]);
}

#[test]
fn the_other_show_targets_build_the_right_intent() {
    let h = harness("cli-ui-others", true, applied(), "");
    for args in [
        vec!["show", "tab"],
        vec!["show", "home", "--filter", "poulpi"],
        vec!["show", "search", "fix"],
        vec!["show", "ref", "main"],
        vec!["show", "pr", "12"],
        vec!["show", "issue", "7", "--view", "conversation"],
        vec!["show", "wip", "src/a.rs", "--staged", "--composer"],
        vec!["show", "palette", "stash"],
        vec!["highlight", "3:2-5:9", "--no-scroll"],
        vec!["back"],
    ] {
        let (code, out) = h.json(&args);
        assert_eq!(code, 0, "{args:?}: {out}");
    }
    let kinds: Vec<&str> = h.received().iter().map(|(i, _)| i.kind()).collect();
    // The daemon's rate limit allows a burst of 5 per connection, and each
    // CLI call is its own connection, so all ten arrive.
    assert_eq!(
        kinds,
        [
            "show_tab",
            "show_home",
            "show_search",
            "show_ref",
            "show_hosting",
            "show_hosting",
            "show_wip",
            "show_palette",
            "highlight",
            "back"
        ]
    );
    let seen = h.received();
    assert!(matches!(&seen[1].0, UiIntent::ShowHome { filter: Some(f) } if f == "poulpi"));
    assert!(matches!(
        &seen[5].0,
        UiIntent::ShowHosting {
            item: gitpoulpi_ipc::ui::HostingItem::Issue(7),
            view: gitpoulpi_ipc::ui::HostingView::Conversation,
            ..
        }
    ));
    // The default view is the details.
    assert!(matches!(
        &seen[4].0,
        UiIntent::ShowHosting {
            view: gitpoulpi_ipc::ui::HostingView::Details,
            ..
        }
    ));
    assert!(matches!(
        &seen[8].0,
        UiIntent::Highlight { range, scroll: false } if range.start_col == Some(2)
    ));
}

#[test]
fn clap_rejects_an_unknown_settings_page_and_a_bad_range_is_not_found() {
    let h = harness("cli-ui-args", true, applied(), "");
    let output = h.run(&["show", "settings", "no-such-page"]);
    assert_eq!(output.status.code(), Some(2), "clap usage error");
    let (code, out) = h.json(&["highlight", "9-3"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    let (code, _) = h.json(&[
        "show",
        "settings",
        "ui-customization",
        "--setting",
        "bad key!",
    ]);
    assert_eq!(code, 1);
    assert!(h.received().is_empty());
}

// ---- Cross-session routing and launching -------------------------------

impl Harness {
    /// Another daemon in the same config dir (its own session), optionally
    /// with a fake window that has this harness's repository open.
    #[cfg_attr(not(unix), allow(dead_code))]
    fn other_session(&self, session: &str, with_viewer: bool) -> Seen {
        let port_file = ConfigPaths::for_dir(self.config.path()).ipc_port_file(session);
        let backend = gitpoulpi_daemon::backend_for(gitpoulpi_config::DefaultBackend::System);
        let handle = self
            ._rt
            .block_on(gitpoulpi_daemon::Daemon::new(session).spawn(backend))
            .unwrap();
        let endpoint = handle.endpoint();
        gitpoulpi_ipc::discovery::publish(&port_file, &endpoint).unwrap();
        std::mem::forget(handle);
        let seen: Seen = Arc::default();
        if with_viewer {
            let (tx, rx) = std::sync::mpsc::channel();
            self._rt.spawn(fake_viewer(
                endpoint,
                self.repo.repo_ref().id,
                applied(),
                Arc::clone(&seen),
                Arc::default(),
                tx,
            ));
            rx.recv().unwrap();
        }
        seen
    }

    /// Like `run`, with extra environment.
    #[cfg_attr(not(unix), allow(dead_code))]
    fn run_env(&self, args: &[&str], env: &[(&str, &str)]) -> (i32, Value) {
        let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
        cmd.env("GITPOULPI_CONFIG_DIR", self.config.path())
            .arg("--json")
            .args(["--session", &self.session])
            .args(["--repo", &self.repo.root().to_string_lossy()])
            .arg("ui")
            .args(args)
            .current_dir(self.repo.root());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let output = cmd.output().unwrap();
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{args:?}: not JSON ({e}): {} / {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().unwrap_or(-1), value)
    }

    /// A launcher script that records its arguments in `marker` and starts
    /// nothing else; when `attach` is set it then connects a fake window to
    /// this harness's daemon, the way a real terminal's GitPoulpi would.
    #[cfg_attr(not(unix), allow(dead_code))]
    fn fake_launcher(&self, attach: bool) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = self.config.path().join("launcher");
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("marker.txt");
        let script = dir.join("launch.sh");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nfor a in \"$@\"; do echo \"$a\" >> '{}'; done\npwd >> '{}'\n",
                marker.display(),
                marker.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        if attach {
            let (endpoint, repo, handle) = (
                self.endpoint.clone(),
                self.repo.repo_ref().id,
                self._rt.handle().clone(),
            );
            let marker = marker.clone();
            std::thread::spawn(move || {
                while !marker.exists() {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                let (tx, rx) = std::sync::mpsc::channel();
                handle.spawn(fake_viewer(
                    endpoint,
                    repo,
                    applied(),
                    Arc::default(),
                    Arc::default(),
                    tx,
                ));
                let _ = rx.recv();
            });
        }
        (script, marker)
    }
}

#[cfg(unix)]
#[test]
fn a_viewer_only_in_another_session_gets_the_intent() {
    let h = harness("s8-own", false, applied(), "");
    let elsewhere = h.other_session("s8-elsewhere", true);
    let (code, out) = h.json(&["show", "commit", "HEAD"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["outcome"], "applied");
    assert_eq!(out["routed_session"], "s8-elsewhere");
    assert_eq!(elsewhere.lock().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn two_other_sessions_with_the_repo_are_ambiguous() {
    let h = harness("s8-amb", false, applied(), "");
    let a = h.other_session("s8-amb-a", true);
    let b = h.other_session("s8-amb-b", true);
    let (code, out) = h.json(&["show", "commit", "HEAD"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["outcome"], "no_viewer");
    let message = out["message"].as_str().unwrap();
    assert!(
        message.contains("s8-amb-a") && message.contains("s8-amb-b"),
        "{message}"
    );
    assert!(a.lock().unwrap().is_empty() && b.lock().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn viewers_all_sessions_lists_every_session_with_its_name() {
    let h = harness("s8-list", true, applied(), "");
    h.other_session("s8-list-other", true);
    let (code, out) = h.json(&["viewers", "--all-sessions"]);
    assert_eq!(code, 0);
    let mut sessions: Vec<_> = out
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["session"].as_str().unwrap().to_string())
        .collect();
    sessions.sort();
    assert_eq!(sessions, ["s8-list", "s8-list-other"]);
    // Without the flag only the own session, and no `session` key.
    let (_, own) = h.json(&["viewers"]);
    assert_eq!(own.as_array().unwrap().len(), 1);
    assert!(own[0].get("session").is_none());
}

#[cfg(unix)]
#[test]
fn launch_runs_the_expanded_template_and_delivers_to_the_new_window() {
    let h = harness("s8-launch", false, applied(), "");
    let (script, marker) = h.fake_launcher(true);
    let toml = format!(
        "[ui]\nlaunch = \"custom\"\nlaunch_command = \"{} --dir {{repo}} {{gitpoulpi}} --session {{session}}\"\n",
        script.display()
    );
    std::fs::write(h.config.path().join("config.toml"), toml).unwrap();
    let (code, out) = h.run_env(&["--launch", "show", "keys"], &[]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["outcome"], "applied");
    assert_eq!(out["launched"], true);
    let lines: Vec<String> = std::fs::read_to_string(&marker)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    let repo = h.repo.root().canonicalize().unwrap();
    assert_eq!(lines[0], "--dir");
    assert_eq!(
        std::path::Path::new(&lines[1]).canonicalize().unwrap(),
        repo
    );
    assert!(lines[2].ends_with("gitpoulpi"), "{lines:?}");
    assert_eq!(&lines[3..5], ["--session", "s8-launch"]);
    // The launcher ran in the repository.
    assert_eq!(
        std::path::Path::new(&lines[5]).canonicalize().unwrap(),
        repo
    );
}

/// Whether the fake launcher's marker shows up. The first run of a freshly
/// written script can start late (macOS scans it before it first runs), after
/// the short bounded wait the command under test was given.
#[cfg(unix)]
fn marker_appears(marker: &std::path::Path) -> bool {
    for _ in 0..200 {
        if marker.exists() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}

#[cfg(unix)]
#[test]
fn launch_waits_a_bounded_time_and_reports_when_no_window_attaches() {
    let h = harness("s8-nowin", false, applied(), "");
    let (script, marker) = h.fake_launcher(false);
    std::fs::write(
        h.config.path().join("config.toml"),
        format!(
            "[ui]\nlaunch = \"custom\"\nlaunch_command = \"{}\"\n",
            script.display()
        ),
    )
    .unwrap();
    let (code, out) = h.run_env(
        &["--launch", "show", "keys"],
        &[("GITPOULPI_UI_LAUNCH_WAIT_MS", "600")],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["outcome"], "no_viewer");
    assert!(out["message"].as_str().unwrap().contains("none attached"));
    assert!(marker_appears(&marker), "the launcher must have run");
}

#[cfg(unix)]
#[test]
fn nothing_is_launched_without_the_flag_unless_configured_and_auto() {
    let h = harness("s8-optin", false, applied(), "");
    let (script, marker) = h.fake_launcher(false);
    let cfg = |mcp: &str| {
        std::fs::write(
            h.config.path().join("config.toml"),
            format!(
                "[ui]\nlaunch = \"custom\"\nlaunch_command = \"{}\"\n{mcp}",
                script.display()
            ),
        )
        .unwrap();
    };
    let short = [("GITPOULPI_UI_LAUNCH_WAIT_MS", "300")];
    // Configured, but `ask` (the default) and no flag: no launch.
    cfg("");
    let (code, out) = h.run_env(&["show", "keys"], &short);
    assert_eq!((code, out["outcome"].as_str()), (1, Some("no_viewer")));
    assert!(!marker.exists());
    // `ui_control = off` never launches, even with the flag.
    cfg("[mcp]\nui_control = \"off\"\n");
    let _ = h.run_env(&["--launch", "show", "keys"], &short);
    assert!(!marker.exists());
    // `auto` + configured: launches without the flag.
    cfg("[mcp]\nui_control = \"auto\"\n");
    let _ = h.run_env(&["show", "keys"], &short);
    assert!(marker_appears(&marker));
}

#[test]
fn launch_with_the_setting_off_explains_how_to_turn_it_on() {
    let h = harness("s8-off", false, applied(), "");
    let (code, out) = h.json(&["--launch", "show", "keys"]);
    assert_eq!(code, 1);
    assert_eq!(out["outcome"], "no_viewer");
    assert!(out["message"].as_str().unwrap().contains("ui.launch"));
}

#[test]
fn propose_is_refused_unless_allow_ui_propose_is_on() {
    let h = harness("cli-ui-propose-off", true, applied(), "");
    let (code, out) = h.json(&["propose", "branch", "feature/x"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["outcome"], "refused");
    assert!(
        out["reason"].as_str().unwrap().contains("allow_ui_propose"),
        "{out}"
    );
    assert!(h.received().is_empty(), "nothing reached the window");
}

#[test]
fn propose_resolves_sanitizes_and_names_the_proposal_in_every_answer() {
    let deferred = UiOutcome::Deferred {
        reason: gitpoulpi_ipc::ui::DeferReason::Ask,
    };
    let h = harness(
        "cli-ui-propose-on",
        true,
        deferred,
        "[mcp]\nallow_ui_propose = true\n",
    );
    // Checked before routing: a bad revision, an empty message.
    let (code, out) = h.json(&["propose", "reword", "nope", "-m", "x"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    assert!(out["proposal"].as_str().unwrap().starts_with("p-"), "{out}");
    let (code, out) = h.json(&["propose", "reword", "HEAD", "-m", "\u{1b}\u{7}  \n"]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    assert!(h.received().is_empty());

    let (code, out) = h.json(&[
        "propose",
        "reword",
        "HEAD",
        "-m",
        "fix: better\u{1b}[31m\n\nbody\u{202e} line",
    ]);
    assert_eq!(code, 3, "deferred: {out}");
    assert_eq!(out["outcome"], "deferred");
    let id = out["proposal"]
        .as_str()
        .expect("the proposal id")
        .to_string();
    let seen = h.received();
    let UiIntent::Propose {
        id: sent, proposal, ..
    } = &seen[0].0
    else {
        panic!("{:?}", seen[0])
    };
    assert_eq!(sent, &id);
    assert_eq!(
        proposal,
        &gitpoulpi_ipc::ui::Proposal::Reword {
            commit: gitpoulpi_core::domain::Oid::from_hex(&h.head()).unwrap(),
            message: "fix: better[31m\n\nbody line".into(),
        }
    );

    let (code, out) = h.json(&["propose", "tag", "v1", "--message", "release"]);
    assert_eq!(code, 3, "{out}");
    let seen = h.received();
    assert!(
        matches!(
            &seen[1].0,
            UiIntent::Propose {
                proposal: gitpoulpi_ipc::ui::Proposal::Tag { message: Some(m), .. },
                ..
            } if m == "release"
        ),
        "{:?}",
        seen[1]
    );
    // A split plan: read from a file, at least two parts, sanitized.
    let plan = h.config.path().join("plan.json");
    std::fs::write(&plan, r#"{"parts": [{"message": "one", "pieces": []}]}"#).unwrap();
    let plan_arg = plan.to_str().unwrap();
    let (code, out) = h.json(&["propose", "split", "HEAD", "--plan", plan_arg]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    std::fs::write(
        &plan,
        r#"{"parts": [{"message": "one\u001b[0m", "pieces": [{"path": "a.txt"}]}, {"message": "two", "pieces": []}]}"#,
    )
    .unwrap();
    let (code, out) = h.json(&["propose", "split", "HEAD", "--plan", plan_arg]);
    assert_eq!(code, 3, "{out}");
    let seen = h.received();
    let UiIntent::Propose {
        proposal: gitpoulpi_ipc::ui::Proposal::SplitCommit { commit, parts },
        ..
    } = &seen[2].0
    else {
        panic!("{:?}", seen[2])
    };
    assert_eq!(commit.to_string(), h.head());
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].message, "one[0m");
    let (code, _) = h.json(&["propose", "commit-message", "feat: x", "--body", "b"]);
    assert_eq!(code, 3);
    let (code, _) = h.json(&["propose", "branch", "feature/x", "--start-point", "HEAD~1"]);
    assert_eq!(code, 3);
    assert_eq!(h.received().len(), 5);
}

#[test]
fn propose_rewrite_sends_the_recipe_without_secrets_and_never_a_script() {
    let deferred = UiOutcome::Deferred {
        reason: gitpoulpi_ipc::ui::DeferReason::Ask,
    };
    let h = harness(
        "cli-ui-propose-rewrite",
        true,
        deferred,
        "[mcp]\nallow_ui_propose = true\n",
    );
    let head = h.head();
    let file = h.config.path().join("recipe.json");
    let recipe_arg = file.to_str().unwrap();
    // Not a recipe, then no rule: refused before routing.
    std::fs::write(&file, "{ not json").unwrap();
    let (code, out) = h.json(&["propose", "rewrite", "--recipe", recipe_arg]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    std::fs::write(&file, r#"{"name": "empty", "rules": []}"#).unwrap();
    let (code, out) = h.json(&["propose", "rewrite", "--recipe", recipe_arg]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    // A script is never proposed: it would run when previewed.
    use gitpoulpi_core::domain::rewrite::{
        ContentFind, ContentReplace, Recipe, Rule, RuleKind, Scope, ScriptLanguage, ScriptRef,
        ScriptSource, SecretText,
    };
    let write = |recipe: &Recipe| std::fs::write(&file, serde_json::to_string(recipe).unwrap());
    let script = Recipe::new(
        "script",
        Scope::default(),
        vec![Rule::new(RuleKind::Script(ScriptRef {
            language: ScriptLanguage::Lua,
            source: ScriptSource::Inline { text: "x".into() },
            params: Default::default(),
        }))],
    );
    write(&script).unwrap();
    let (code, out) = h.json(&["propose", "rewrite", "--recipe", recipe_arg]);
    assert_eq!((code, &out["outcome"]), (1, &Value::from("not_found")));
    assert!(
        out["what"].as_str().unwrap().contains("script"),
        "refused for the script, not for a typo: {out}"
    );
    assert!(h.received().is_empty(), "nothing reached the window");

    // A recipe with a message rule and secret values: sent, secrets dropped.
    let mut recipe = Recipe::new(
        "n\u{1b}[0m",
        Scope::default(),
        vec![
            Rule::new(RuleKind::MessageEncoding { names: true }),
            Rule::new(RuleKind::ContentReplace(ContentReplace {
                paths: Vec::new(),
                find: ContentFind::Secrets {
                    literals: vec![SecretText("hunter2".into())],
                    scan: false,
                },
                with: String::new(),
                max_blob_bytes: None,
            })),
        ],
    );
    recipe.rules[0].label = Some("fix".into());
    write(&recipe).unwrap();
    let (code, out) = h.json(&["propose", "rewrite", "--recipe", recipe_arg]);
    assert_eq!(code, 3, "deferred: {out}");
    let seen = h.received();
    let UiIntent::Propose {
        proposal: gitpoulpi_ipc::ui::Proposal::Rewrite { recipe },
        ..
    } = &seen[0].0
    else {
        panic!("{:?}", seen[0])
    };
    assert_eq!(recipe.rules.len(), 2);
    assert_eq!(recipe.name, "n[0m");
    assert!(
        !serde_json::to_string(recipe).unwrap().contains("hunter2"),
        "a secret value never travels in a proposal"
    );
    // The proposal did nothing to the repository.
    assert_eq!(h.head(), head);
}

/// What the tests need to know of the conflicted repository.
struct Conflicted {
    path: String,
    stages: [Option<String>; 3],
}

/// A repository stopped on a text conflict (`notes.txt`, one conflict).
fn conflicted(session: &str, config_toml: &str) -> (Harness, Conflicted) {
    let fixture = gitpoulpi_testkit::ConflictFixture::text();
    let path = fixture.path().to_string();
    let stages = [1u8, 2, 3].map(|n| fixture.stage_oid(n, &path));
    (
        harness_on(session, fixture.into_repo(), true, applied(), config_toml),
        Conflicted { path, stages },
    )
}

#[test]
fn show_conflict_checks_the_file_and_the_number_before_routing() {
    let (h, c) = conflicted("cli-ui-conflict-show", "");
    let path = c.path.clone();
    let (code, out) = h.json(&["show", "conflict", "nope.txt"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    let (code, out) = h.json(&["show", "conflict", &path, "--conflict", "9"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    let (code, out) = h.json(&["show", "conflict", &path, "--conflict", "0"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    assert!(h.received().is_empty(), "nothing reached the window");

    let (code, out) = h.json(&["show", "conflict", &path, "--conflict", "1"]);
    assert_eq!(code, 0, "{out}");
    let seen = h.received();
    let UiIntent::ShowConflict {
        repo,
        path: sent,
        conflict,
    } = &seen[0].0
    else {
        panic!("{:?}", seen[0])
    };
    assert_eq!(repo, &h.repo.repo_ref().id);
    assert_eq!((sent.as_str(), *conflict), (path.as_str(), Some(1)));
}

#[test]
fn propose_resolution_composes_picks_checks_the_stages_and_is_gated() {
    // Gated like every proposal.
    let (h, c) = conflicted("cli-ui-resolution-off", "");
    let path = c.path.clone();
    let (code, out) = h.json(&["propose", "resolution", &path, "--pick", "1=b"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("refused")),
        "{out}"
    );
    assert!(h.received().is_empty());

    let (h, c) = conflicted("cli-ui-resolution-on", "[mcp]\nallow_ui_propose = true\n");
    let path = c.path.clone();
    // A pick for every conflict, or a default; a bad choice.
    let (code, out) = h.json(&["propose", "resolution", &path, "--pick", ""]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    let (code, out) = h.json(&["propose", "resolution", &path, "--pick", "1=x"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    let (code, out) = h.json(&["propose", "resolution", "nope.txt", "--pick", "1=a"]);
    assert_eq!(
        (code, &out["outcome"]),
        (1, &Value::from("not_found")),
        "{out}"
    );
    assert!(h.received().is_empty());

    let (code, out) = h.json(&["propose", "resolution", &path, "--pick", "1=ba"]);
    assert_eq!(code, 0, "{out}");
    assert!(out["proposal"].as_str().unwrap().starts_with("p-"), "{out}");
    let (code, _) = h.json(&[
        "propose",
        "resolution",
        &path,
        "--pick",
        "x=a",
        "--default",
        "b",
    ]);
    assert_eq!(code, 1);
    let content = h.config.path().join("proposed.txt");
    std::fs::write(&content, "one\u{1b}[31m\ttwo\r\n").unwrap();
    let (code, _) = h.json(&[
        "propose",
        "resolution",
        &path,
        "--content-file",
        &content.to_string_lossy(),
    ]);
    assert_eq!(code, 0);

    let seen = h.received();
    assert_eq!(seen.len(), 2);
    let UiIntent::Propose { proposal, .. } = &seen[0].0 else {
        panic!("{:?}", seen[0])
    };
    let gitpoulpi_ipc::ui::Proposal::ConflictResolution {
        path: sent,
        expected,
        picks,
        content,
    } = proposal
    else {
        panic!("{proposal:?}")
    };
    assert_eq!(sent, &path);
    assert!(content.is_none());
    for (slot, stage) in expected.iter().zip([1u8, 2, 3]) {
        assert_eq!(
            slot.map(|o| o.to_hex()),
            c.stages[usize::from(stage) - 1],
            "stage {stage}"
        );
    }
    let picks = picks.as_ref().expect("picks");
    assert_eq!(picks.len(), 1);
    assert_eq!(
        picks[0].order,
        [
            gitpoulpi_core::domain::ConflictSide::Theirs,
            gitpoulpi_core::domain::ConflictSide::Ours
        ]
    );
    let UiIntent::Propose { proposal, .. } = &seen[1].0 else {
        panic!("{:?}", seen[1])
    };
    assert!(matches!(
        proposal,
        gitpoulpi_ipc::ui::Proposal::ConflictResolution { content: Some(c), picks: None, .. }
            if c == "one[31m\ttwo\n"
    ));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

/// `gitpoulpi sequence <args>` on the harness's repository (a person's own
/// command: only a person trusts a repository's sequences).
fn sequence_cmd(h: &Harness, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", h.config.path())
        .arg("--json")
        .args(["--session", &h.session])
        .args(["--repo", &h.repo.root().to_string_lossy()])
        .arg("sequence")
        .args(args)
        .current_dir(h.repo.root())
        .output()
        .unwrap();
    let value = serde_json::from_slice(&output.stdout).unwrap();
    (output.status.code().unwrap_or(-1), value)
}

#[test]
fn ui_sequence_run_proposes_only_what_the_user_could_run_themselves() {
    let deferred = UiOutcome::Deferred {
        reason: gitpoulpi_ipc::ui::DeferReason::Ask,
    };
    let h = harness(
        "cli-ui-sequence-run",
        true,
        deferred,
        "[mcp]\nallow_ui_propose = false\n\n[[sequences]]\nid = \"mine\"\nname = \"Mine\"\nscope = \"machine\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n",
    );
    // The gate: proposals are off.
    let (code, out) = h.json(&["sequence", "run", "mine"]);
    assert_eq!(code, 1, "{out}");
    assert!(out["reason"].as_str().unwrap().contains("allow_ui_propose"));
    assert!(h.received().is_empty());

    // Gate on.
    std::fs::write(
        h.config.path().join("config.toml"),
        "[mcp]\nallow_ui_propose = true\n\n[[sequences]]\nid = \"mine\"\nname = \"Mine\"\nscope = \"machine\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n",
    )
    .unwrap();
    // Unknown: refused before routing.
    let (code, out) = h.json(&["sequence", "run", "ghost"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["outcome"], "refused");
    assert!(h.received().is_empty(), "nothing reached the window");

    // A sequence of the user's: proposed, nothing runs from here.
    let (code, out) = h.json(&["sequence", "run", "mine"]);
    assert_eq!(code, 3, "deferred: {out}");
    assert!(out["proposal"].as_str().unwrap().starts_with("p-"), "{out}");
    let seen = h.received();
    let UiIntent::Propose { proposal, .. } = &seen[0].0 else {
        panic!("{:?}", seen[0])
    };
    assert_eq!(
        proposal,
        &gitpoulpi_ipc::ui::Proposal::SequenceRun { id: "mine".into() }
    );

    // The repository's own file: refused until a person trusts it.
    let dir = h.repo.root().join(".gitpoulpi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("sequences.toml"),
        "[[sequences]]\nid = \"shared\"\nname = \"Shared\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n",
    )
    .unwrap();
    let (code, out) = h.json(&["sequence", "run", "shared"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out["reason"].as_str().unwrap().contains("not trusted"),
        "{out}"
    );
    assert_eq!(h.received().len(), 1);
    let (code, _) = sequence_cmd(&h, &["trust"]);
    assert_eq!(code, 0);
    let (code, out) = h.json(&["sequence", "run", "shared"]);
    assert_eq!(code, 3, "{out}");
    assert_eq!(h.received().len(), 2);
    // A change after the trust asks again.
    std::fs::write(
        dir.join("sequences.toml"),
        "[[sequences]]\nid = \"shared\"\nname = \"Shared\"\n\n[[sequences.steps]]\nrun = \"push\"\n",
    )
    .unwrap();
    let (code, out) = h.json(&["sequence", "run", "shared"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(h.received().len(), 2, "nothing reached the window");
}
