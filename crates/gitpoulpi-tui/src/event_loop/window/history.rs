//! The File History view's per-tick work: promoting the History intent,
//! paging, pointing the diff at the selection, and leaving for the Log.

use crate::event_loop::*;
use crate::history_view::{HistoryView, ViewMode as HistoryViewMode};
use init::WindowState;

impl WindowState {
    /// Runs before the frame is settled and drawn.
    pub(in crate::event_loop) fn tick_history(&mut self) {
        self.promote_history_intent();
        // The view lives as long as its diff does: closing the diff, or
        // opening another file's, ends it.
        if self
            .history
            .as_ref()
            .is_some_and(|_| !self.diff_view_state.as_ref().is_some_and(HistoryView::owns))
        {
            self.history = None;
        }
        let now = Instant::now();
        let Some(history) = self.history.as_mut() else {
            return;
        };
        let viewport = history.viewport_rows();
        if let Some(request) = history.next_request(viewport) {
            self.link.request_file_insight(request);
        }
        // The right side's mode is whatever the window shows: a File View or
        // Blame of this file's commits (opened from the control, a key, a tab
        // or a menu) is the File / Blame mode; no such view is the diff.
        let shown = self
            .file_insight
            .as_ref()
            .filter(|f| history.owns_insight(&f.request))
            .and_then(|f| HistoryViewMode::of_kind(f.request.kind));
        history.mode = shown.unwrap_or(HistoryViewMode::Diff);
        if history.mode == HistoryViewMode::Diff {
            history.sync_diff(&mut self.diff_view_state, now);
        } else if now >= history.changed_at() + crate::history_view::DEBOUNCE {
            // Follow the cursor row, once it has stood still a moment.
            if let Some(request) = history.insight_request(history.mode) {
                if self.file_insight.as_ref().map(|f| &f.request) != Some(&request) {
                    self.link.request_file_insight(request.clone());
                    self.file_insight = Some(FileInsight::loading(request));
                }
            }
        }
    }

    /// Every place that opens a History creates a `FileInsightKind::History`
    /// request over the File View; it becomes the split view here, with the
    /// diff of the first row on the right.
    fn promote_history_intent(&mut self) {
        let Some(request) = self
            .file_insight
            .as_ref()
            .filter(|f| f.request.kind == FileInsightKind::History)
            .map(|f| f.request.clone())
        else {
            return;
        };
        self.file_insight = None;
        let same = self
            .history
            .as_ref()
            .is_some_and(|h| h.path == request.path && h.start == request.at);
        if !same {
            let pct = crate::session_store::layout(&self.session).history_split_pct;
            let view = HistoryView::promote(&request, pct);
            self.diff_view_state = Some(view.initial_state());
            self.history = Some(view);
        }
        self.central_view = CentralView::Diff;
    }

    /// "Show in Log" found its row: the Log takes the central column back
    /// with that commit selected.
    pub(in crate::event_loop) fn leave_history_for_log(&mut self, commit: Oid) {
        select_log_commit(
            &self.log_result,
            &self.log_order,
            &mut self.log_state,
            commit,
        );
        self.history = None;
        self.diff_view_state = None;
        self.file_insight = None;
        self.central_view = CentralView::Log;
        self.focus = Focus::Log;
        self.right_panel = RightPanel::CommitDetail;
    }
}
