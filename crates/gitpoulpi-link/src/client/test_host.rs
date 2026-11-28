//! A [`LinkHost`] for tests that have no frontend: every hook is a no-op,
//! the log filter is empty, the undo key is `Ctrl+Z` and a hosting change is "its own" when it carries
//! [`OWN_ORIGIN`].

use std::path::Path;
use std::sync::Arc;

use gitpoulpi_core::domain::RefFilter;
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_core::NotCancellableReason;

use super::{
    Action, JobEvent, LinkContext, LinkHost, PushGroupEvent, Report, Requester, Ticket, TimingKind,
    UndoKey,
};

/// The origin `TestHost` treats as this process's own.
pub const OWN_ORIGIN: &str = "test-own-origin";

/// The host a test starts a link with.
pub fn host() -> Arc<dyn LinkHost> {
    Arc::new(TestHost)
}

struct TestHost;

impl LinkHost for TestHost {
    fn wake(&self, _link: &LinkContext) {}
    fn before_exit(&self, _link: &LinkContext) {}
    fn raise_window(&self, _link: &LinkContext) -> Result<String, String> {
        Err("no window".to_string())
    }
    fn ask_sensitive_stage(
        &self,
        _link: &LinkContext,
        _root: std::path::PathBuf,
        _action: Action,
        _files: Vec<Sensitive>,
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
    fn not_cancellable(&self, _reason: NotCancellableReason) -> &'static str {
        ""
    }
    fn initial_log_filter(&self, _root: &Path) -> RefFilter {
        RefFilter::default()
    }
    fn undo_key(&self, _which: UndoKey) -> String {
        // The terminal UI's default binding, which the recorded transcripts name.
        "Ctrl+Z".to_string()
    }
    fn daemon_job_live(&self) -> bool {
        false
    }
    fn is_own_hosting_change(&self, change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        change.origin == OWN_ORIGIN
    }
}
