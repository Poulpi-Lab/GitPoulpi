//! The terminal UI's side of the daemon client: [`LinkHost`] forwarding each
//! hook to the TUI module that used to be called directly.

use std::path::Path;
use std::sync::Arc;

use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_core::NotCancellableReason;

use crate::daemon_link::{
    Action, JobEvent, LinkHost, PushGroupEvent, Report, Requester, Ticket, TimingKind, UndoKey,
};

/// The host every TUI link is started with.
pub(crate) fn host() -> Arc<dyn LinkHost> {
    Arc::new(TuiLinkHost)
}

struct TuiLinkHost;

impl LinkHost for TuiLinkHost {
    fn wake(&self) {
        crate::redraw::poke();
    }

    fn before_exit(&self) {
        crate::exit_log::restore_terminal();
    }

    fn raise_window(&self) -> Result<String, String> {
        crate::window_ctl::raise()
    }

    fn ask_sensitive_stage(
        &self,
        root: std::path::PathBuf,
        action: Action,
        files: Vec<Sensitive>,
        ticket: Option<Ticket>,
    ) {
        crate::stage_guard_dialog::ask(root, action, files, ticket);
    }

    fn on_connected(&self, requester: Requester) {
        crate::jobs_ui::set_requester(requester.clone());
        crate::push_group_ui::set_requester(requester.clone());
        crate::hosting_notify::set_requester(requester);
    }

    fn jobs(&self, event: JobEvent) {
        match event {
            JobEvent::Seed(list) => crate::jobs_ui::seed(list),
            JobEvent::Changed(info) => crate::jobs_ui::changed(info),
            JobEvent::Removed(id) => crate::jobs_ui::removed(id),
            JobEvent::Say(text) => crate::jobs_ui::say(text),
        }
    }

    fn push_group(&self, event: PushGroupEvent<'_>) {
        match event {
            PushGroupEvent::Started { root, request } => {
                crate::push_group_ui::on_started(root, request);
            }
            PushGroupEvent::Outcome { root, outcome } => {
                crate::push_group_ui::on_outcome(root, outcome);
            }
            PushGroupEvent::Failed { root, why } => crate::push_group_ui::on_failed(root, why),
            PushGroupEvent::Progress {
                run_id,
                remote,
                status,
            } => crate::push_group_ui::on_progress(run_id, remote, status),
        }
    }

    fn report(&self, report: Report<'_>) {
        match report {
            Report::Success {
                action,
                repo,
                elapsed,
            } => crate::activity_log::report_success(action, repo, elapsed),
            Report::Failure {
                action,
                error,
                repo,
                elapsed,
                context,
            } => crate::activity_log::report_failure(action, error, repo, elapsed, context),
            Report::Rejection {
                action,
                rejection,
                repo,
                elapsed,
            } => crate::activity_log::report_rejection(action, rejection, repo, elapsed),
            Report::Message { text, repo } => crate::activity_log::note_message(text, repo),
            Report::BackgroundFetchSkipped { repo } => {
                crate::activity_log::background_fetch_skipped(repo);
            }
        }
    }

    fn timing(&self, kind: TimingKind, repo: &Path, label: String, ms: u64) {
        match kind {
            TimingKind::Foreground => crate::activity_log::timing_record(repo, label, ms),
            TimingKind::Background => crate::activity_log::timing_background(repo, label, ms),
        }
    }

    fn expect_background_fetch(&self) {
        crate::activity_log::expect_background_fetch();
    }

    fn background_fetch_pending(&self) -> bool {
        crate::activity_log::background_fetch_pending()
    }

    fn job_label(&self, id: u64) -> String {
        crate::jobs_ui::label_of_id(id)
    }

    fn not_cancellable(&self, reason: NotCancellableReason) -> &'static str {
        crate::jobs_ui::reason(reason)
    }

    fn initial_log_filter(&self, root: &Path) -> gitpoulpi_core::domain::RefFilter {
        crate::graph::load_walk_filter(root)
    }

    fn undo_key(&self, which: UndoKey) -> String {
        let action = match which {
            UndoKey::Undo => crate::keymap::Action::GlobalUndo,
            UndoKey::UndoOffer => crate::keymap::Action::GlobalUndoOffer,
        };
        crate::keymap::current().shown(action)
    }

    fn daemon_job_live(&self) -> bool {
        crate::jobs_ui::daemon_job_live()
    }

    fn is_own_hosting_change(&self, change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        crate::hosting_notify::is_own(change)
    }
}
