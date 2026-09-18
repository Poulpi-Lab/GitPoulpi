//! Drives the proactive conflict badge (`crate::conflict_badge`): asks the
//! daemon when the refs moved or Settings closed, and hands the frame the
//! prediction to badge. Nothing here waits: the request is a channel send and
//! the answer is read on a later frame.

use gitpoulpi_core::domain::conflict_prevention::ConflictPrediction;

use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    pub(in crate::event_loop) fn tick_conflict_badge(&mut self) -> Option<ConflictPrediction> {
        // Settings closed: a switch or the Base branches may have changed.
        if self
            .conflict_badge
            .settings_closed(self.preferences.is_some())
        {
            self.conflict_settings_gen += 1;
        }
        let head = match &self.refs_result {
            Ok(refs) => crate::refs_sidebar::head_oid(refs),
            Err(_) => None,
        };
        // Nothing to ask about while the first load runs, on a failed refs
        // read or a detached / unborn HEAD.
        let key = match (&self.refs_result, head) {
            (Ok(refs), Some(_)) if !self.still_loading => Some(crate::conflict_badge::key_of(
                refs,
                self.conflict_settings_gen,
            )),
            _ => None,
        };
        let requester = self.link.activity_handle().raw.requester();
        self.conflict_badge
            .tick(key, std::time::Instant::now(), || {
                requester.call(
                    gitpoulpi_core::domain::OperationKind::PredictConflicts,
                    serde_json::json!({ "debounce_ms": crate::conflict_badge::DEBOUNCE_MS }),
                )
            });
        self.conflict_badge.shown(head).cloned()
    }
}
