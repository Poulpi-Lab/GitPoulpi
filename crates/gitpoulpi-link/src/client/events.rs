//! One call that gathers everything the link has for the frontend.
//!
//! [`DaemonLink::drain`] reads every receiver of the link and every "took
//! since the last call" flag of its shared state, in a fixed order, and
//! returns them as typed [`LinkEvent`]s. It is built on the existing
//! receivers, which stay public and untouched, so the terminal UI keeps
//! reading them one by one; a frontend uses one or the other for a link, never
//! both (each message is delivered once).

use super::*;

/// One thing the link has for the frontend.
pub enum LinkEvent {
    /// A refresh result (`outcome_rx`); `summary` says what it changed.
    Refreshed(Box<RefreshOutcome>),
    /// The daemon refused or failed an action (`action_error_rx`).
    ActionError(String),
    /// An informational line an action wants shown (`action_message_rx`).
    ActionMessage(String),
    /// A push the remote refused because it moved on (`push_rejected_rx`).
    PushRejected(PushRejectedNotice),
    /// Local changes are in the way and auto-stash is `ask` (`autostash_rx`).
    AutoStashNeeded(AutoStashNeeded),
    /// A commit's changed files (`commit_files_rx`).
    CommitFiles(CommitFilesOutcome),
    /// One half of the working tree's diff (`working_diff_rx`).
    WorkingDiff(WorkingDiffOutcome),
    /// A file-centric read (`file_insight_rx`).
    FileInsight(FileInsightOutcome),
    /// An answer for the merge conflict editor (`merge_rx`).
    Merge(MergeReply),
    /// One page of older history (`log_page_rx`).
    LogPage(LogPageOutcome),
    /// A batch of commit stats (`commit_stats_rx`).
    CommitStats(CommitStatsOutcome),
    /// A commit's numbered changes for the split editor (`split_source_rx`).
    SplitSource(SplitSourceOutcome),
    /// The undo journal as the daemon last reported it differs from what was
    /// last drained: the undo and redo availability moved.
    JournalChanged(gitpoulpi_core::domain::JournalState),
    /// Something outside GitPoulpi was recorded in the journal.
    OutsideChange,
    /// The daemon asks the user to confirm a risky undo or redo.
    UndoConfirm(crate::undo_ui::PendingConfirm),
    /// The pending auto-stash record changed.
    PendingAutoStashChanged,
    /// The record's stash vanished outside GitPoulpi (its index).
    PendingAutoStashGone(u32),
    /// The hidden or solo refs changed (the log's filter is to be reloaded).
    RefVisibilityChanged,
    /// Another client wrote to an issue or request.
    HostingItemChanged(gitpoulpi_ipc::HostingChangeArgs),
    /// An agent's request for this window's view.
    Ui(crate::ui_viewer::Inbound),
    /// The link stopped (`failure_rx`): nothing else arrives after it.
    Failed(String),
}

impl DaemonLink {
    /// Everything that arrived since the last call, oldest first within each
    /// kind: refresh results, then answers and notices, then the state the
    /// shared flags report, then the link's failure (always last).
    ///
    /// Never blocks. Call it on every wake; an empty result means a spurious
    /// wake. The ticket queue (`take_done`, `take_for`) is process-wide and
    /// stays separate.
    pub fn drain(&self) -> Vec<LinkEvent> {
        let mut events = Vec::new();
        events.extend(
            self.outcome_rx
                .try_iter()
                .map(|outcome| LinkEvent::Refreshed(Box::new(outcome))),
        );
        events.extend(self.action_error_rx.try_iter().map(LinkEvent::ActionError));
        events.extend(
            self.action_message_rx
                .try_iter()
                .map(LinkEvent::ActionMessage),
        );
        events.extend(
            self.push_rejected_rx
                .try_iter()
                .map(LinkEvent::PushRejected),
        );
        events.extend(self.autostash_rx.try_iter().map(LinkEvent::AutoStashNeeded));
        events.extend(self.commit_files_rx.try_iter().map(LinkEvent::CommitFiles));
        events.extend(self.working_diff_rx.try_iter().map(LinkEvent::WorkingDiff));
        events.extend(self.file_insight_rx.try_iter().map(LinkEvent::FileInsight));
        events.extend(self.merge_rx.try_iter().map(LinkEvent::Merge));
        events.extend(self.log_page_rx.try_iter().map(LinkEvent::LogPage));
        events.extend(self.commit_stats_rx.try_iter().map(LinkEvent::CommitStats));
        events.extend(self.split_source_rx.try_iter().map(LinkEvent::SplitSource));
        if self.activity.journal.take_changed() {
            events.push(LinkEvent::JournalChanged(self.journal_state()));
        }
        if self.take_outside_news() {
            events.push(LinkEvent::OutsideChange);
        }
        if let Some(confirm) = self.take_undo_confirm() {
            events.push(LinkEvent::UndoConfirm(confirm));
        }
        if self.take_pending_auto_stash_changed() {
            events.push(LinkEvent::PendingAutoStashChanged);
        }
        if let Some(index) = self.take_pending_auto_stash_gone() {
            events.push(LinkEvent::PendingAutoStashGone(index));
        }
        if self.take_ref_visibility_changed() {
            events.push(LinkEvent::RefVisibilityChanged);
        }
        events.extend(
            self.take_hosting_item_changes()
                .into_iter()
                .map(LinkEvent::HostingItemChanged),
        );
        events.extend(self.ui().take_inbound().into_iter().map(LinkEvent::Ui));
        events.extend(self.failure_rx.try_iter().map(LinkEvent::Failed));
        events
    }
}
