//! View-control routing: an agent's `UiShow` reaches exactly one attached
//! viewer window (a fake TUI connection here), is answered by that viewer's
//! `UiReport`, and is never broadcast, journaled or treated as a mutation.
//! The registry's own selection rules are unit-tested in `viewers.rs`; this
//! drives the same code over real sockets.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{OperationKind, RepoId, UiControl};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::ui::{
    RaiseAnswer, UiIntent, UiOrigin, UiOutcome, UiQueryArgs, UiReportArgs, UiReportBody,
    UiShowArgs, UiState, ViewerId, ViewerInfo,
};
use gitpoulpi_ipc::{Client, Event, Request, Response, ServerMessage};
use tokio::net::TcpStream;

fn repo(p: &str) -> RepoId {
    RepoId(p.into())
}

/// A raw connection: sends requests, and keeps every pushed event it reads.
struct Conn {
    stream: TcpStream,
    next: u64,
    events: Vec<Event>,
}

impl Conn {
    async fn open(endpoint: &gitpoulpi_ipc::Endpoint) -> Self {
        Self {
            stream: Client::connect(endpoint).await.unwrap().into_stream(),
            next: 1,
            events: Vec::new(),
        }
    }

    async fn call(&mut self, command: OperationKind, args: impl serde::Serialize) -> Response {
        let id = self.next;
        self.next += 1;
        write_message(
            &mut self.stream,
            &Request {
                id,
                session: "ui-test".into(),
                repo: None,
                command,
                args: serde_json::to_value(args).unwrap(),
            },
        )
        .await
        .unwrap();
        loop {
            match read_message::<_, ServerMessage>(&mut self.stream)
                .await
                .unwrap()
            {
                Some(ServerMessage::Response(r)) if r.request_id == id => return r,
                Some(ServerMessage::Event(e)) => self.events.push(e),
                Some(_) => {}
                None => panic!("daemon closed the connection"),
            }
        }
    }

    /// The next pushed event, within `wait`.
    async fn event(&mut self, wait: Duration) -> Option<Event> {
        if !self.events.is_empty() {
            return Some(self.events.remove(0));
        }
        match tokio::time::timeout(wait, read_message::<_, ServerMessage>(&mut self.stream)).await {
            Ok(Ok(Some(ServerMessage::Event(e)))) => Some(e),
            _ => None,
        }
    }

    async fn attach(&mut self, info: ViewerInfo) {
        let r = self.call(OperationKind::UiAttach, info).await;
        assert!(r.result.is_ok(), "{:?}", r.result);
    }

    async fn report(&mut self, id: u64, outcome: UiOutcome) -> Response {
        self.call(
            OperationKind::UiReport,
            UiReportArgs {
                id,
                body: UiReportBody::Outcome { outcome },
            },
        )
        .await
    }
}

fn info(id: &str, active: &str, tabs: &[&str], focused: Option<bool>) -> ViewerInfo {
    ViewerInfo {
        id: ViewerId(id.into()),
        pid: 1,
        active_repo: Some(repo(active)),
        tabs: tabs.iter().map(|t| repo(t)).collect(),
        focused,
        agent_control: true,
        last_focus_at: None,
    }
}

fn show(intent: UiIntent) -> UiShowArgs {
    UiShowArgs {
        viewer: None,
        intent,
        origin: UiOrigin {
            client: "claude-code".into(),
            label: None,
            control: UiControl::Auto,
        },
        timeout_ms: Some(3_000),
    }
}

fn tab(r: &str) -> UiIntent {
    UiIntent::ShowTab { repo: repo(r) }
}

fn outcome(resp: Response) -> UiOutcome {
    serde_json::from_value(resp.result.expect("UiShow answers Ok")).unwrap()
}

fn applied() -> UiOutcome {
    UiOutcome::Applied {
        state: UiState::default(),
    }
}

async fn start() -> (gitpoulpi_ipc::Endpoint, gitpoulpi_daemon::DaemonHandle) {
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("ui-test").spawn(backend).await.unwrap();
    (handle.endpoint(), handle)
}

/// Sends `UiShow` from a fresh agent connection in a task, so the test can
/// play the viewer meanwhile.
fn agent_shows(
    endpoint: &gitpoulpi_ipc::Endpoint,
    args: UiShowArgs,
) -> tokio::task::JoinHandle<UiOutcome> {
    let port = endpoint.clone();
    tokio::spawn(async move {
        let mut agent = Conn::open(&port).await;
        outcome(agent.call(OperationKind::UiShow, args).await)
    })
}

#[tokio::test]
async fn an_intent_reaches_the_attached_viewer_and_its_report_answers_the_agent() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], Some(true))).await;

    let agent = agent_shows(&port, show(tab("/a")));
    let Some(Event::UiIntent {
        viewer: to,
        intent_id,
        intent,
        origin,
    }) = viewer.event(Duration::from_secs(3)).await
    else {
        panic!("the viewer must receive the intent")
    };
    assert_eq!(to, ViewerId("v1".into()));
    assert_eq!(intent, tab("/a"));
    assert_eq!(origin.client, "claude-code");
    assert_eq!(origin.control, UiControl::Auto, "the mode travels with it");
    assert!(viewer.report(intent_id, applied()).await.result.is_ok());
    assert_eq!(agent.await.unwrap(), applied());
}

#[tokio::test]
async fn the_ui_control_mode_is_carried_to_the_viewer() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], None)).await;
    let mut args = show(tab("/a"));
    args.origin.control = UiControl::Ask;
    let agent = agent_shows(&port, args);
    let Some(Event::UiIntent {
        intent_id, origin, ..
    }) = viewer.event(Duration::from_secs(3)).await
    else {
        panic!()
    };
    assert_eq!(origin.control, UiControl::Ask);
    viewer
        .report(
            intent_id,
            UiOutcome::Deferred {
                reason: gitpoulpi_ipc::ui::DeferReason::Ask,
            },
        )
        .await;
    assert_eq!(
        agent.await.unwrap(),
        UiOutcome::Deferred {
            reason: gitpoulpi_ipc::ui::DeferReason::Ask
        }
    );
}

#[tokio::test]
async fn no_viewer_is_a_clear_error_and_nothing_hangs() {
    let (port, _daemon) = start().await;
    let mut agent = Conn::open(&port).await;
    let started = std::time::Instant::now();
    let out = outcome(agent.call(OperationKind::UiShow, show(tab("/a"))).await);
    let UiOutcome::NoViewer { message } = out else {
        panic!("expected NoViewer, got {out:?}")
    };
    assert!(message.contains("No GitPoulpi window"), "{message}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "answered at once"
    );
    // `ui state` has no window to ask either.
    let resp = agent
        .call(
            OperationKind::UiQuery,
            UiQueryArgs::State {
                viewer: None,
                include_text: false,
            },
        )
        .await;
    assert!(resp
        .result
        .unwrap_err()
        .message
        .contains("No GitPoulpi window"));
    // ...but listing viewers is just an empty list.
    let list = agent
        .call(OperationKind::UiQuery, UiQueryArgs::Viewers)
        .await;
    assert_eq!(list.result.unwrap(), serde_json::json!([]));
}

#[tokio::test]
async fn the_window_showing_the_repo_wins_then_the_focused_one() {
    let (port, _daemon) = start().await;
    let mut a = Conn::open(&port).await;
    let mut b = Conn::open(&port).await;
    a.attach(info("a", "/x", &["/x", "/y"], Some(true))).await;
    b.attach(info("b", "/y", &["/y"], None)).await;

    // /y is b's active tab, a only has it open.
    let first = agent_shows(&port, show(tab("/y")));
    let Some(Event::UiIntent { intent_id, .. }) = b.event(Duration::from_secs(3)).await else {
        panic!("b shows /y")
    };
    b.report(intent_id, applied()).await;
    first.await.unwrap();
    assert!(a.event(Duration::from_millis(200)).await.is_none());

    // Neither shows /q: the focused window (a) takes it.
    let second = agent_shows(&port, show(tab("/q")));
    let Some(Event::UiIntent { intent_id, .. }) = a.event(Duration::from_secs(3)).await else {
        panic!("a is focused")
    };
    a.report(intent_id, applied()).await;
    second.await.unwrap();

    // b gains focus (presence), so the next unaddressed intent goes to b.
    let r = b
        .call(
            OperationKind::UiPresence,
            info("b", "/y", &["/y"], Some(true)),
        )
        .await;
    assert!(r.result.is_ok());
    let third = agent_shows(&port, show(UiIntent::ShowHelp { topic: None }));
    let Some(Event::UiIntent { intent_id, .. }) = b.event(Duration::from_secs(3)).await else {
        panic!("b was focused last")
    };
    b.report(intent_id, applied()).await;
    third.await.unwrap();
}

#[tokio::test]
async fn an_explicit_viewer_overrides_the_selection() {
    let (port, _daemon) = start().await;
    let mut a = Conn::open(&port).await;
    let mut b = Conn::open(&port).await;
    a.attach(info("a", "/x", &["/x"], Some(true))).await;
    b.attach(info("b", "/y", &["/y"], None)).await;

    let mut args = show(tab("/x"));
    args.viewer = Some(ViewerId("b".into()));
    let agent = agent_shows(&port, args);
    let Some(Event::UiIntent { intent_id, .. }) = b.event(Duration::from_secs(3)).await else {
        panic!("b was named")
    };
    b.report(intent_id, applied()).await;
    agent.await.unwrap();
    assert!(a.event(Duration::from_millis(200)).await.is_none());

    let mut args = show(tab("/x"));
    args.viewer = Some(ViewerId("ghost".into()));
    let mut agent = Conn::open(&port).await;
    let out = outcome(agent.call(OperationKind::UiShow, args).await);
    assert!(
        matches!(out, UiOutcome::Refused { ref reason } if reason.contains("ghost")),
        "{out:?}"
    );

    // `ui viewers` lists both, with their facts.
    let list = agent
        .call(OperationKind::UiQuery, UiQueryArgs::Viewers)
        .await;
    let viewers: Vec<ViewerInfo> = serde_json::from_value(list.result.unwrap()).unwrap();
    assert_eq!(viewers.len(), 2);
    assert_eq!(
        viewers[0].id,
        ViewerId("a".into()),
        "most recently focused first"
    );
}

#[tokio::test]
async fn a_viewer_that_drops_mid_intent_and_reattaches_still_answers_it() {
    let (port, _daemon) = start().await;
    let mut old = Conn::open(&port).await;
    old.attach(info("v1", "/a", &["/a"], Some(true))).await;

    let agent = agent_shows(&port, show(tab("/b")));
    let Some(Event::UiIntent { intent_id, .. }) = old.event(Duration::from_secs(3)).await else {
        panic!()
    };
    // The window restarts into the new tab: connection gone, intent unanswered.
    drop(old);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut fresh = Conn::open(&port).await;
    fresh
        .attach(info("v1", "/b", &["/a", "/b"], Some(true)))
        .await;
    let Some(Event::UiIntent {
        intent_id: again,
        intent,
        ..
    }) = fresh.event(Duration::from_secs(3)).await
    else {
        panic!("the pending intent is pushed again to the re-attached window")
    };
    assert_eq!(again, intent_id);
    assert_eq!(intent, tab("/b"));
    fresh.report(again, applied()).await;
    assert_eq!(agent.await.unwrap(), applied());
}

#[tokio::test]
async fn an_unanswered_intent_times_out_with_a_refusal() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], None)).await;
    let mut args = show(tab("/a"));
    args.timeout_ms = Some(150);
    let mut agent = Conn::open(&port).await;
    let out = outcome(agent.call(OperationKind::UiShow, args).await);
    assert_eq!(out, UiOutcome::refused("the window did not answer"));
    // A late report for the dropped intent is rejected.
    let Some(Event::UiIntent { intent_id, .. }) = viewer.event(Duration::from_secs(1)).await else {
        panic!()
    };
    assert!(viewer.report(intent_id, applied()).await.result.is_err());
}

#[tokio::test]
async fn a_burst_of_six_intents_hits_the_rate_limit() {
    let (port, _daemon) = start().await;
    // No viewer: each call is answered at once, but still takes a token.
    let mut agent = Conn::open(&port).await;
    for _ in 0..5 {
        let out = outcome(agent.call(OperationKind::UiShow, show(tab("/a"))).await);
        assert!(matches!(out, UiOutcome::NoViewer { .. }), "{out:?}");
    }
    let out = outcome(agent.call(OperationKind::UiShow, show(tab("/a"))).await);
    assert_eq!(out, UiOutcome::refused("rate limited"));
    // The limit is per connection: another client is not affected.
    let mut other = Conn::open(&port).await;
    let out = outcome(other.call(OperationKind::UiShow, show(tab("/a"))).await);
    assert!(matches!(out, UiOutcome::NoViewer { .. }));
}

#[tokio::test]
async fn control_off_and_a_user_switch_refuse_without_bothering_the_window() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    let mut closed = info("v1", "/a", &["/a"], Some(true));
    closed.agent_control = false;
    viewer.attach(closed).await;

    let mut agent = Conn::open(&port).await;
    let out = outcome(agent.call(OperationKind::UiShow, show(tab("/a"))).await);
    assert!(
        matches!(out, UiOutcome::Refused { ref reason } if reason.contains("turned agent view control off")),
        "{out:?}"
    );

    let mut off = show(tab("/a"));
    off.origin.control = UiControl::Off;
    let out = outcome(agent.call(OperationKind::UiShow, off).await);
    assert!(
        matches!(out, UiOutcome::Refused { ref reason } if reason.contains("off")),
        "{out:?}"
    );
    assert!(viewer.event(Duration::from_millis(200)).await.is_none());
}

#[tokio::test]
async fn intents_are_never_broadcast_journaled_or_a_mutation() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], Some(true))).await;
    // A plain subscriber: it must hear nothing at all.
    let mut bystander = Conn::open(&port).await;
    let mut other_viewer = Conn::open(&port).await;
    other_viewer.attach(info("v2", "/z", &["/z"], None)).await;

    for intent in [
        tab("/a"),
        UiIntent::Notify {
            message: "hello".into(),
        },
    ] {
        let agent = agent_shows(&port, show(intent));
        let Some(Event::UiIntent { intent_id, .. }) = viewer.event(Duration::from_secs(3)).await
        else {
            panic!()
        };
        viewer.report(intent_id, applied()).await;
        agent.await.unwrap();
    }
    assert!(
        bystander.event(Duration::from_millis(300)).await.is_none(),
        "a plain client gets no event (no RepoChanged, JournalChanged, nor the intent)"
    );
    assert!(
        other_viewer
            .event(Duration::from_millis(100))
            .await
            .is_none(),
        "another viewer does not get an intent addressed to a window"
    );

    // A connection that is not the viewer cannot answer for it.
    let agent = agent_shows(&port, show(tab("/a")));
    let Some(Event::UiIntent { intent_id, .. }) = viewer.event(Duration::from_secs(3)).await else {
        panic!()
    };
    assert!(bystander.report(intent_id, applied()).await.result.is_err());
    viewer.report(intent_id, applied()).await;
    agent.await.unwrap();

    // The operations are classified as view control, with no journal or scope.
    for kind in [
        OperationKind::UiAttach,
        OperationKind::UiPresence,
        OperationKind::UiReport,
        OperationKind::UiShow,
        OperationKind::UiQuery,
    ] {
        let t = kind.traits();
        assert!(kind.is_ui_control() && !t.journaled && !t.read);
    }
}

#[tokio::test]
async fn escape_sequences_in_a_toast_are_stripped_before_the_window_sees_them() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], None)).await;
    let agent = agent_shows(
        &port,
        show(UiIntent::Notify {
            message: "pwn\u{1b}]0;title\u{7}ed".into(),
        }),
    );
    let Some(Event::UiIntent {
        intent_id, intent, ..
    }) = viewer.event(Duration::from_secs(3)).await
    else {
        panic!()
    };
    assert_eq!(
        intent,
        UiIntent::Notify {
            message: "pwn]0;titleed".into()
        }
    );
    viewer.report(intent_id, applied()).await;
    agent.await.unwrap();
}

#[tokio::test]
async fn ui_state_asks_the_window_and_returns_its_answer() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    viewer.attach(info("v1", "/a", &["/a"], Some(true))).await;
    let task = tokio::spawn(async move {
        let mut agent = Conn::open(&port).await;
        agent
            .call(
                OperationKind::UiQuery,
                UiQueryArgs::State {
                    viewer: None,
                    include_text: true,
                },
            )
            .await
    });
    let Some(Event::UiStateRequest {
        viewer: to,
        query_id,
        include_text,
    }) = viewer.event(Duration::from_secs(3)).await
    else {
        panic!("the window gets a state request")
    };
    assert_eq!(to, ViewerId("v1".into()));
    assert!(include_text);
    let state = UiState {
        focus: Some("status".into()),
        text: Some(vec!["line".into()]),
        ..UiState::default()
    };
    let r = viewer
        .call(
            OperationKind::UiReport,
            UiReportArgs {
                id: query_id,
                body: UiReportBody::State {
                    state: state.clone(),
                },
            },
        )
        .await;
    assert!(r.result.is_ok());
    let answer: UiState = serde_json::from_value(task.await.unwrap().result.unwrap()).unwrap();
    assert_eq!(answer, state);
}

fn raise_answer(resp: Response) -> RaiseAnswer {
    serde_json::from_value(resp.result.expect("a raise always answers Ok")).unwrap()
}

#[tokio::test]
async fn a_launcher_raise_reaches_the_most_recent_window_and_reports_what_it_did() {
    let (port, _daemon) = start().await;
    let mut old = Conn::open(&port).await;
    old.attach(info("old", "/a", &["/a"], None)).await;
    let mut recent = Conn::open(&port).await;
    recent
        .attach(info("recent", "/b", &["/b"], Some(true)))
        .await;
    let task = tokio::spawn(async move {
        let mut launcher = Conn::open(&port).await;
        launcher
            .call(OperationKind::UiQuery, UiQueryArgs::Raise { viewer: None })
            .await
    });
    let Some(Event::RaiseWindow { viewer, raise_id }) = recent.event(Duration::from_secs(3)).await
    else {
        panic!("the most recently focused window is asked to raise itself")
    };
    assert_eq!(viewer, ViewerId("recent".into()));
    assert!(old.event(Duration::from_millis(200)).await.is_none());
    assert!(recent.report(raise_id, applied()).await.result.is_ok());
    let answer = raise_answer(task.await.unwrap());
    assert!(answer.raised, "{answer:?}");
    assert_eq!(answer.viewer.unwrap().id, ViewerId("recent".into()));
}

#[tokio::test]
async fn a_raise_the_window_could_not_do_says_why_so_the_launcher_starts_a_new_one() {
    let (port, _daemon) = start().await;
    let mut viewer = Conn::open(&port).await;
    // The user switched agent control off: a launcher click still raises.
    let mut quiet = info("v1", "/a", &["/a"], Some(true));
    quiet.agent_control = false;
    viewer.attach(quiet).await;
    let task = tokio::spawn(async move {
        let mut launcher = Conn::open(&port).await;
        launcher
            .call(OperationKind::UiQuery, UiQueryArgs::Raise { viewer: None })
            .await
    });
    let Some(Event::RaiseWindow { raise_id, .. }) = viewer.event(Duration::from_secs(3)).await
    else {
        panic!("agent control does not gate a raise")
    };
    viewer
        .report(
            raise_id,
            UiOutcome::refused("no way to raise this terminal"),
        )
        .await;
    let answer = raise_answer(task.await.unwrap());
    assert!(!answer.raised);
    assert_eq!(
        answer.reason.as_deref(),
        Some("no way to raise this terminal")
    );
}

#[tokio::test]
async fn a_raise_with_no_window_attached_is_a_plain_no() {
    let (port, _daemon) = start().await;
    let mut launcher = Conn::open(&port).await;
    let answer = raise_answer(
        launcher
            .call(OperationKind::UiQuery, UiQueryArgs::Raise { viewer: None })
            .await,
    );
    assert!(!answer.raised);
    assert!(answer.viewer.is_none());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
