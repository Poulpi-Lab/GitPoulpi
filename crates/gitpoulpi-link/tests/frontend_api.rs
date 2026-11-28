//! The surface a frontend sees, used only through the crate's public paths:
//! if an item a frontend needs becomes private, this file stops compiling.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_link::{
    Action, DaemonLink, JobEvent, LinkContext, LinkEvent, LinkHost, LinkId, PushGroupEvent,
    RefreshSummary, Report, Requester, Ticket, TimingKind, UndoKey,
};

/// The smallest frontend: it records which link each wake came from.
#[derive(Default)]
struct Frontend {
    wakes: Mutex<Vec<LinkId>>,
}

impl LinkHost for Frontend {
    fn wake(&self, link: &LinkContext) {
        self.wakes.lock().unwrap().push(link.id());
    }
    fn before_exit(&self, _link: &LinkContext) {}
    fn raise_window(&self, _link: &LinkContext) -> Result<String, String> {
        Err("no window".into())
    }
    fn ask_sensitive_stage(
        &self,
        _link: &LinkContext,
        _root: std::path::PathBuf,
        _action: Action,
        _files: Vec<gitpoulpi_core::privacy::Sensitive>,
        _ticket: Option<Ticket>,
    ) {
    }
    fn on_connected(&self, _link: &LinkContext, _requester: Requester) {}
    fn jobs(&self, _link: &LinkContext, _event: JobEvent) {}
    fn push_group(&self, _link: &LinkContext, _event: PushGroupEvent<'_>) {}
    fn report(&self, _link: &LinkContext, _report: Report<'_>) {}
    fn timing(
        &self,
        _link: &LinkContext,
        _kind: TimingKind,
        _repo: &Path,
        _label: String,
        _ms: u64,
    ) {
    }
    fn expect_background_fetch(&self, _link: &LinkContext) {}
    fn background_fetch_pending(&self, _link: &LinkContext) -> bool {
        false
    }
    fn job_label(&self, id: u64) -> String {
        format!("#{id}")
    }
    fn not_cancellable(&self, _reason: gitpoulpi_core::NotCancellableReason) -> &'static str {
        ""
    }
    fn initial_log_filter(&self, _root: &Path) -> gitpoulpi_core::domain::RefFilter {
        Default::default()
    }
    fn undo_key(&self, _which: UndoKey) -> String {
        String::new()
    }
    fn daemon_job_live(&self) -> bool {
        false
    }
    fn is_own_hosting_change(&self, _change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        false
    }
}

/// A public, translated label for an action (the in-flight text a frontend
/// shows for what it submitted). Red when `activity_label` is not `pub`.
#[test]
fn an_action_has_a_public_label() {
    assert_eq!(Action::FetchAll.activity_label(), "fetching...");
    assert_eq!(Action::StageAll.activity_label(), "staging all changes...");
}

/// A frontend started with its own host drains typed events and reads the
/// refresh summary and the link identity. Red when `drain`, `LinkEvent`,
/// `RefreshSummary`, `DaemonLink::id` or `LinkContext` stop being public.
#[test]
fn a_frontend_drains_typed_events_from_its_link() {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let host = Arc::new(Frontend::default());
    let link: DaemonLink = gitpoulpi_link::start_at(
        "frontend-api".into(),
        fixture.repo_ref(),
        Some(config.path().join("sessions").join("frontend-api.port")),
        host.clone(),
    );
    let started = Instant::now();
    let summary: RefreshSummary = loop {
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "no first refresh"
        );
        let found = link.drain().into_iter().find_map(|event| match event {
            LinkEvent::Refreshed(outcome) => Some(outcome.summary),
            _ => None,
        });
        if let Some(summary) = found {
            break summary;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(summary.first && summary.any());
    let wakes = host.wakes.lock().unwrap().clone();
    assert!(!wakes.is_empty() && wakes.iter().all(|id| *id == link.id()));
    assert_eq!(link.context().root(), fixture.repo_ref().root);
}
