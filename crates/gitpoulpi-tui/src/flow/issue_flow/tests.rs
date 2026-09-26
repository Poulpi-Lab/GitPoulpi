use std::sync::Mutex;

use gitpoulpi_config::IntegrationKind;
use gitpoulpi_core::workflow::{IssueFlow, WorkflowTemplate};
use gitpoulpi_hosting::RecordedTransport;

use super::*;
use crate::flow::test_support;

struct FakeFactory(Arc<RecordedTransport>);

impl ClientFactory for FakeFactory {
    fn client(
        &self,
        integration: &Integration,
        _all: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        Ok(gitpoulpi_hosting::client_for(
            integration.clone(),
            "tok".into(),
            self.0.clone(),
        ))
    }
}

#[derive(Default)]
struct FakePick(Mutex<Vec<(String, String, Option<String>)>>);

impl PickUp for FakePick {
    fn pick_up(
        &self,
        remote: &str,
        branch: &str,
        parent: Option<&str>,
    ) -> Receiver<Result<String, String>> {
        self.0.lock().unwrap().push((
            remote.to_string(),
            branch.to_string(),
            parent.map(str::to_string),
        ));
        let (tx, rx) = channel();
        let _ = tx.send(Ok(msg::issue_picked(branch)));
        rx
    }
}

fn view(mode: IssueFlowMode) -> Arc<FlowView> {
    let mut wf = WorkflowTemplate::GithubFlow.workflow();
    wf.issue_flow = Some(IssueFlow {
        mode,
        issue_title: Some("[{type}] {title}".into()),
        issue_labels: vec!["workflow::{type}".into()],
        timeout_secs: Some(5),
        ..IssueFlow::default()
    });
    Arc::new(FlowView::new(test_support::report(
        wf,
        &["main"],
        "main",
        &[],
    )))
}

fn target() -> Target {
    Target {
        integration: Integration::new(
            "gl",
            IntegrationKind::GitlabSelfManaged,
            "https://gl.example.com",
        ),
        project: "g/p".into(),
    }
}

fn mr(iid: u64, source: &str) -> String {
    format!(
        r#"{{"iid":{iid},"title":"t","state":"opened","source_branch":"{source}","target_branch":"main",
        "source_project_id":1,"target_project_id":1,"web_url":"u","updated_at":""}}"#
    )
}

fn seed(
    kind: IssueFlowKind,
    mode: IssueFlowMode,
    transport: Arc<RecordedTransport>,
    picker: Arc<FakePick>,
) -> (IssueFlowSeed, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (
        IssueFlowSeed {
            view: view(mode),
            kind,
            issue_target: Some(target()),
            code_target: Some(target()),
            integrations: Vec::new(),
            factory: Arc::new(FakeFactory(transport)),
            picker,
            remote: "origin".into(),
            remote_branches: vec!["main".into()],
            repo_root: dir.path().to_path_buf(),
            state_path: dir.path().join("repo-state.toml"),
            user: Some("sg".into()),
        },
        dir,
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text(d: &mut IssueFlowDialog, text: &str) {
    for c in text.chars() {
        d.on_key(key(KeyCode::Char(c)));
    }
}

fn pump(d: &mut IssueFlowDialog, until: impl Fn(&IssueFlowDialog) -> bool) {
    for _ in 0..400 {
        d.poll();
        if until(d) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the dialog never got there");
}

fn screen(d: &mut IssueFlowDialog) -> String {
    let area = Rect::new(0, 0, 100, 24);
    let mut buf = Buffer::empty(area);
    d.render(issue_flow_rect(area), &mut buf);
    crate::dialog_kit::buffer_text(&buf)
}

fn issue_response() -> &'static str {
    r#"{"iid":7,"web_url":"https://gl.example.com/g/p/-/issues/7"}"#
}

#[test]
fn a_bot_issue_is_created_waited_for_and_picked_up_with_the_requests_target() {
    std::env::set_var("GITPOULPI_FLOW_POLL_MS", "20");
    let transport = Arc::new(
        RecordedTransport::new()
            .on(
                "/issues/7/related_merge_requests",
                200,
                &format!("[{}]", mr(3, "feature/7-login")),
            )
            .on("/closed_by", 200, "[]")
            .on(
                "/repository/branches",
                200,
                r#"[{"name":"feature/7-login"}]"#,
            )
            .on("/issues", 201, issue_response()),
    );
    let picker = Arc::new(FakePick::default());
    let (seed, dir) = seed(
        IssueFlowKind::New(Some(0)),
        IssueFlowMode::Bot,
        transport.clone(),
        picker.clone(),
    );
    let mut d = IssueFlowDialog::new(seed);
    assert!(screen(&mut d).contains("a bot (picked up when the branch appears)"));
    // An empty title is refused in the form.
    d.on_key(key(KeyCode::Enter));
    assert_eq!(d.message.as_deref(), Some(msg::ISSUE_NEED_TITLE.get()));
    type_text(&mut d, "Login page");
    d.on_key(key(KeyCode::Enter));
    pump(&mut d, |d| matches!(d.phase, Phase::Done(_)));
    let calls = picker.0.lock().unwrap().clone();
    assert_eq!(
        calls,
        [(
            "origin".to_string(),
            "feature/7-login".to_string(),
            Some("main".to_string())
        )]
    );
    // The issue went through the template.
    let post = transport
        .requests()
        .into_iter()
        .find(|r| r.method == gitpoulpi_hosting::Method::Post && r.url.ends_with("/issues"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(post.body.as_deref().unwrap()).unwrap();
    assert_eq!(body["title"], "[feature] Login page");
    assert_eq!(body["labels"], "workflow::feature");
    // The pick-up settles the watched issue (off the render thread).
    let path = dir.path().join("repo-state.toml");
    for _ in 0..200 {
        let state = gitpoulpi_session::RepoState::load(&path).unwrap();
        if state.watched_issues.is_empty() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the issue is still watched");
}

#[test]
fn the_waiting_card_shows_a_spinner_the_elapsed_time_and_stop_closes_without_losing_the_issue() {
    std::env::set_var("GITPOULPI_FLOW_POLL_MS", "20");
    let transport = Arc::new(
        RecordedTransport::new()
            .on("/related_merge_requests", 200, "[]")
            .on("/closed_by", 200, "[]")
            .on("/repository/branches", 200, "[]")
            .on("/merge_requests", 200, "[]")
            .on("/issues", 201, issue_response()),
    );
    let picker = Arc::new(FakePick::default());
    let (seed, dir) = seed(
        IssueFlowKind::New(Some(0)),
        IssueFlowMode::Bot,
        transport,
        picker.clone(),
    );
    let mut d = IssueFlowDialog::new(seed);
    type_text(&mut d, "Slow bot");
    d.on_key(key(KeyCode::Enter));
    pump(&mut d, |d| matches!(d.phase, Phase::Waiting { .. }));
    assert!(d.animating());
    let text = screen(&mut d);
    assert!(text.contains("issue #7"), "{text}");
    assert!(text.contains("so far, giving up after 5 s"), "{text}");
    assert!(text.contains("Stop waiting"), "{text}");
    // The keyboard stops it (Esc), as the button does.
    assert_eq!(d.on_key(key(KeyCode::Esc)), IssueFlowOutcome::Close);
    assert!(d.cancel.load(Ordering::SeqCst));
    assert!(picker.0.lock().unwrap().is_empty());
    let state = gitpoulpi_session::RepoState::load(&dir.path().join("repo-state.toml")).unwrap();
    assert_eq!(state.watched_issues.len(), 1, "the next fetch still tells");
    assert_eq!(state.watched_issues[0].number, 7);
}

#[test]
fn host_creates_branch_makes_the_branch_and_the_request_then_picks_it_up() {
    let transport = Arc::new(
        RecordedTransport::new()
            .on(
                "/repository/branches",
                201,
                r#"{"name":"feature/7-dark-mode"}"#,
            )
            .on(
                "/merge_requests",
                201,
                r#"{"iid":9,"web_url":"https://gl.example.com/mr/9"}"#,
            )
            .on("/issues", 201, issue_response()),
    );
    let picker = Arc::new(FakePick::default());
    let (mut seed, _dir) = seed(
        IssueFlowKind::New(Some(0)),
        IssueFlowMode::HostCreatesBranch,
        transport.clone(),
        picker.clone(),
    );
    seed.remote_branches = vec!["main".into()];
    let mut d = IssueFlowDialog::new(seed);
    type_text(&mut d, "Dark mode");
    d.on_key(key(KeyCode::Enter));
    pump(&mut d, |d| matches!(d.phase, Phase::Done(_)));
    let urls: Vec<String> = transport.requests().iter().map(|r| r.url.clone()).collect();
    assert!(
        urls.iter().any(|u| u.ends_with("/repository/branches")),
        "{urls:?}"
    );
    assert!(
        urls.iter().any(|u| u.ends_with("/merge_requests")),
        "{urls:?}"
    );
    let calls = picker.0.lock().unwrap().clone();
    assert_eq!(calls[0].1, "feature/7-dark-mode");
}

#[test]
fn the_appeared_card_offers_pick_up_and_later_keeps_the_issue() {
    let picker = Arc::new(FakePick::default());
    let (seed, _dir) = seed(
        IssueFlowKind::Appeared {
            number: 12,
            branches: vec!["feature/12-login".into()],
        },
        IssueFlowMode::Bot,
        Arc::new(RecordedTransport::new()),
        picker.clone(),
    );
    let mut d = IssueFlowDialog::new(seed);
    let text = screen(&mut d);
    assert!(text.contains("A branch for issue #12 appeared"), "{text}");
    assert!(text.contains("feature/12-login"), "{text}");
    assert!(text.contains("Pick up") && text.contains("Later"), "{text}");
    // Click the Pick up button: found in the paint walk's hit rects.
    let (rect, _) = *d
        .hits
        .iter()
        .find(|(_, f)| *f == Focus::Primary)
        .expect("the primary button was painted");
    let outcome = d.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rect.x + 1,
        row: rect.y,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(outcome, IssueFlowOutcome::Stay);
    pump(&mut d, |d| matches!(d.phase, Phase::Done(_)));
    assert_eq!(picker.0.lock().unwrap()[0].1, "feature/12-login");
}

#[test]
fn later_closes_the_appeared_card() {
    let picker = Arc::new(FakePick::default());
    let (seed, _dir) = seed(
        IssueFlowKind::Appeared {
            number: 12,
            branches: vec!["feature/12-login".into()],
        },
        IssueFlowMode::Bot,
        Arc::new(RecordedTransport::new()),
        picker.clone(),
    );
    let mut d = IssueFlowDialog::new(seed);
    assert_eq!(d.on_key(key(KeyCode::Esc)), IssueFlowOutcome::Close);
    assert!(picker.0.lock().unwrap().is_empty());
}

#[test]
fn references_are_issues_requests_or_branches() {
    assert!(matches!(parse_reference("#12"), Some(Reference::Issue(12))));
    assert!(matches!(parse_reference("12"), Some(Reference::Issue(12))));
    assert!(matches!(parse_reference("!5"), Some(Reference::Request(5))));
    assert!(matches!(
        parse_reference("feature/x"),
        Some(Reference::Branch(b)) if b == "feature/x"
    ));
    assert!(parse_reference("  ").is_none());
}

#[test]
fn a_typed_branch_name_goes_straight_to_pick_up() {
    let picker = Arc::new(FakePick::default());
    let (seed, _dir) = seed(
        IssueFlowKind::PickUp,
        IssueFlowMode::Bot,
        Arc::new(RecordedTransport::new()),
        picker.clone(),
    );
    let mut d = IssueFlowDialog::new(seed);
    assert!(screen(&mut d).contains("Take over"));
    d.on_key(key(KeyCode::Enter));
    assert_eq!(d.message.as_deref(), Some(msg::ISSUE_NEED_REFERENCE.get()));
    type_text(&mut d, "bot/branch-1");
    d.on_key(key(KeyCode::Enter));
    assert!(matches!(d.phase, Phase::Found { .. }));
    d.on_key(key(KeyCode::Enter));
    pump(&mut d, |d| matches!(d.phase, Phase::Done(_)));
    assert_eq!(picker.0.lock().unwrap()[0].1, "bot/branch-1");
}
