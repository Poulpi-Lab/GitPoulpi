//! Keeps the window's workflow report fresh: one request in flight at most,
//! asked again whenever the refs changed (a branch moved, was made or
//! deleted) or a workflow file was saved. The daemon caches the answer on
//! git's own fingerprint plus the two workflow files, so asking is cheap;
//! the wait is on the link thread, never here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::workflow::WorkflowReport;

use crate::daemon_link::{Reply, Requester};

use super::view::{self, FlowView};

static RELOAD: AtomicBool = AtomicBool::new(false);

/// A workflow file was written (the Settings page, an import): read the
/// report again at the next tick.
pub(crate) fn request_reload() {
    RELOAD.store(true, Ordering::SeqCst);
    crate::redraw::poke();
}

#[derive(Default)]
pub(crate) struct FlowSource {
    reply: Option<Reply<WorkflowReport>>,
    /// The `refs_gen` the last request was made at.
    asked_at: Option<u64>,
}

impl FlowSource {
    /// Clears the process-wide report: a window starting.
    pub(crate) fn reset() {
        view::install(None);
    }

    /// Called every frame. Installs an answer that arrived, and asks again
    /// when the refs moved on or a reload was requested.
    pub(crate) fn tick(&mut self, requester: &Requester, refs_gen: u64) {
        if let Some(reply) = &self.reply {
            if let Some(answer) = reply.take() {
                self.reply = None;
                // A failed read keeps the report already held: "could not
                // ask" is not "no workflow".
                if let Ok(report) = answer {
                    view::install(Some(Arc::new(FlowView::new(report))));
                    crate::redraw::poke();
                }
            }
        }
        let reload = RELOAD.load(Ordering::SeqCst);
        if self.reply.is_none() && (reload || self.asked_at != Some(refs_gen)) {
            RELOAD.store(false, Ordering::SeqCst);
            self.asked_at = Some(refs_gen);
            self.reply = Some(
                requester.call::<WorkflowReport>(OperationKind::Workflow, serde_json::json!({})),
            );
        }
    }

    /// A request is on its way (the Settings page says "reading...").
    #[allow(dead_code)]
    pub(crate) fn busy(&self) -> bool {
        self.reply.is_some()
    }
}
