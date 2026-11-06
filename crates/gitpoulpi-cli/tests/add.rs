//! `gitpoulpi --add [PATH]` end to end: the real `gitpoulpi` binary against
//! real daemons and fake windows (TUI stand-ins speaking the IPC protocol).
//! Asserts on the deserialized JSON and on what each window received. The
//! "nothing is running, start normally" branch opens the TUI, so it is
//! covered by the unit tests of `gitpoulpi_daemon::join` instead.

use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::{OperationKind, RepoId, UiControl};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::ui::{
    DeferReason, UiIntent, UiOrigin, UiOutcome, UiReportArgs, UiReportBody, UiState, ViewerId,
    ViewerInfo,
};
use gitpoulpi_ipc::{Client, Event, Request, ServerMessage};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

type Seen = Arc<Mutex<Vec<(UiIntent, UiOrigin)>>>;

struct World {
    config: tempfile::TempDir,
    rt: tokio::runtime::Runtime,
    /// Where the commands run: a repository that is a tab of nobody.
    here: FixtureRepo,
    sessions: Vec<(String, Seen)>,
}

fn repo(file: &str) -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(file, "x\n", "initial")
        .unwrap()
        .build()
}

impl World {
    fn new() -> Self {
        Self {
            config: tempfile::tempdir().unwrap(),
            rt: tokio::runtime::Runtime::new().unwrap(),
            here: repo("here.txt"),
            sessions: Vec::new(),
        }
    }

    /// A daemon for `name` with one fake window whose tabs are `tabs` and
    /// which answers every intent with `answer`.
    fn session(&mut self, name: &str, tabs: &[&FixtureRepo], answer: UiOutcome) {
        let port_file = ConfigPaths::for_dir(self.config.path()).ipc_port_file(name);
        std::fs::create_dir_all(port_file.parent().unwrap()).unwrap();
        let backend = gitpoulpi_daemon::backend_for(gitpoulpi_config::DefaultBackend::System);
        let handle = self
            .rt
            .block_on(gitpoulpi_daemon::Daemon::new(name).spawn(backend))
            .unwrap();
        let endpoint = handle.endpoint();
        gitpoulpi_ipc::discovery::publish(&port_file, &endpoint).unwrap();
        std::mem::forget(handle);
        let seen: Seen = Arc::default();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let tabs: Vec<RepoId> = tabs.iter().map(|r| r.repo_ref().id).collect();
        self.rt.spawn(fake_window(
            endpoint,
            name.to_string(),
            tabs,
            answer,
            Arc::clone(&seen),
            ready_tx,
        ));
        ready_rx.recv().unwrap();
        self.sessions.push((name.to_string(), seen));
    }

    fn seen(&self, name: &str) -> Vec<(UiIntent, UiOrigin)> {
        let (_, seen) = self.sessions.iter().find(|(n, _)| n == name).unwrap();
        seen.lock().unwrap().clone()
    }

    fn run(&self, args: &[&str]) -> (i32, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .current_dir(self.here.root())
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{args:?}: stdout was not JSON ({e}): {} / {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().unwrap_or(-1), value)
    }
}

async fn fake_window(
    endpoint: gitpoulpi_ipc::Endpoint,
    session: String,
    tabs: Vec<RepoId>,
    answer: UiOutcome,
    seen: Seen,
    ready: std::sync::mpsc::Sender<()>,
) {
    let mut stream = Client::connect(&endpoint).await.unwrap().into_stream();
    let info = ViewerInfo {
        id: ViewerId(format!("window-of-{session}")),
        pid: 4242,
        active_repo: tabs.first().cloned(),
        tabs,
        focused: Some(true),
        agent_control: true,
        last_focus_at: None,
    };
    write_message(
        &mut stream,
        &Request {
            id: 1,
            session: session.clone(),
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
        let report = match message {
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
                UiReportArgs {
                    id: intent_id,
                    body: UiReportBody::Outcome {
                        outcome: answer.clone(),
                    },
                }
            }
            _ => continue,
        };
        write_message(
            &mut stream,
            &Request {
                id: next_id,
                session: session.clone(),
                repo: None,
                command: OperationKind::UiReport,
                args: serde_json::to_value(report).unwrap(),
            },
        )
        .await
        .unwrap();
        next_id += 1;
    }
}

fn applied() -> UiOutcome {
    UiOutcome::Applied {
        state: UiState::default(),
    }
}

fn root(r: &FixtureRepo) -> String {
    r.root().to_string_lossy().into_owned()
}

#[test]
fn one_running_session_gets_the_folder_as_a_tab_without_a_question() {
    let mut w = World::new();
    let open = repo("open.txt");
    let target = repo("target.txt");
    w.session("only", &[&open], applied());

    let (code, out) = w.run(&["--add", &root(&target), "--no-raise"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["action"], "joined");
    assert_eq!(out["session"], "only");
    assert_eq!(out["repo"], target.repo_ref().id.0);
    assert_eq!(out["already_open"], false);
    assert_eq!(out["raised"], false);
    assert_eq!(out["outcome"]["outcome"], "applied");

    let seen = w.seen("only");
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].0,
        UiIntent::ShowTab {
            repo: target.repo_ref().id
        }
    );
    // The user's own command: named so, and never asked about.
    assert_eq!(seen[0].1.client, "gitpoulpi-add");
    assert_eq!(seen[0].1.control, UiControl::Auto);
}

#[test]
fn a_folder_inside_a_repository_resolves_to_the_repository_and_open_tabs_are_flagged() {
    let mut w = World::new();
    let open = repo("open.txt");
    w.session("only", &[&open], applied());
    let sub = open.root().join("deeper");
    std::fs::create_dir_all(&sub).unwrap();

    let (code, out) = w.run(&["--add", &sub.to_string_lossy(), "--no-raise"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["repo"], open.repo_ref().id.0);
    assert_eq!(out["already_open"], true, "it was a tab already");
}

#[test]
fn the_current_directory_is_the_default_folder() {
    let mut w = World::new();
    let open = repo("open.txt");
    w.session("only", &[&open], applied());
    let (code, out) = w.run(&["--add", "--no-raise"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["repo"], w.here.repo_ref().id.0);
}

#[test]
fn several_sessions_without_a_terminal_ask_for_session_and_nothing_is_opened() {
    let mut w = World::new();
    let (a, b) = (repo("a.txt"), repo("b.txt"));
    w.session("alpha", &[&a], applied());
    w.session("beta", &[&b], applied());

    let (code, out) = w.run(&["--add", &root(&w.here), "--no-raise"]);
    assert_eq!(code, 1, "{out}");
    let error = out["error"].as_str().unwrap();
    assert!(
        error.contains("--session NAME") && error.contains("alpha, beta"),
        "{error}"
    );
    assert!(w.seen("alpha").is_empty() && w.seen("beta").is_empty());
}

#[test]
fn session_picks_the_target_among_several() {
    let mut w = World::new();
    let (a, b) = (repo("a.txt"), repo("b.txt"));
    w.session("alpha", &[&a], applied());
    w.session("beta", &[&b], applied());

    let (code, out) = w.run(&["-s", "beta", "--add", &root(&w.here), "--no-raise"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["session"], "beta");
    assert_eq!(w.seen("beta").len(), 1);
    assert!(w.seen("alpha").is_empty());
}

#[test]
fn a_busy_window_that_defers_exits_3_and_a_refusal_exits_1() {
    let mut w = World::new();
    let open = repo("open.txt");
    w.session(
        "busy",
        &[&open],
        UiOutcome::Deferred {
            reason: DeferReason::InputBusy,
        },
    );
    let (code, out) = w.run(&["--add", "--no-raise"]);
    assert_eq!(code, 3, "{out}");
    assert_eq!(out["action"], "joined");
    assert_eq!(out["outcome"]["outcome"], "deferred");

    let mut w = World::new();
    w.session("no", &[&open], UiOutcome::refused("nope"));
    let (code, out) = w.run(&["--add", "--no-raise"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["outcome"]["outcome"], "refused");
}

#[test]
fn a_folder_outside_any_repository_is_refused_before_anything_reaches_the_window() {
    let mut w = World::new();
    let open = repo("open.txt");
    w.session("only", &[&open], applied());
    // A folder that does not exist has no repository around it.
    let missing = w.config.path().join("no/such/folder");
    let (code, out) = w.run(&["--add", &missing.to_string_lossy(), "--no-raise"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out["error"]
            .as_str()
            .unwrap()
            .contains("is not inside a git repository"),
        "{out}"
    );
    assert!(w.seen("only").is_empty());
}

#[test]
fn add_takes_no_subcommand() {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .args(["--add=.", "status"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("takes no subcommand"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
