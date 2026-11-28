//! The bridge half of the agent-view viewer: the queues the daemon link and
//! the render loop hand each other (inbound intents and state requests,
//! outgoing reports and presence). Plain data; the painting stays in
//! `ui_viewer`, which re-exports this.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use gitpoulpi_ipc::ui::{
    UiIntent, UiOrigin, UiOutcome, UiReportArgs, UiReportBody, UiState, ViewerId, ViewerInfo,
};

/// This process' viewer id, created once: every window the process restarts
/// into (a tab switch) re-attaches with it, so the daemon sees one viewer.
pub fn viewer_id() -> &'static ViewerId {
    static ID: OnceLock<ViewerId> = OnceLock::new();
    ID.get_or_init(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        ViewerId(format!("tui-{}-{nanos:x}", std::process::id()))
    })
}

/// A request pushed by the daemon to this window.
pub enum Inbound {
    Intent {
        id: u64,
        /// Boxed: a conflict proposal carries a whole file's picks.
        intent: Box<UiIntent>,
        origin: UiOrigin,
    },
    State {
        id: u64,
        include_text: bool,
    },
}

/// What the link thread and the render loop hand each other.
pub struct UiBridge {
    /// Tells one bridge from the next (a window's link is replaced on Retry).
    pub serial: u64,
    inbox: Mutex<VecDeque<Inbound>>,
    outbox: Mutex<Vec<UiReportArgs>>,
    info: Mutex<Option<ViewerInfo>>,
    info_dirty: AtomicBool,
    wake: tokio::sync::Notify,
}

impl Default for UiBridge {
    fn default() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            serial: NEXT.fetch_add(1, Ordering::Relaxed),
            inbox: Default::default(),
            outbox: Default::default(),
            info: Default::default(),
            info_dirty: Default::default(),
            wake: Default::default(),
        }
    }
}

impl UiBridge {
    /// Reader side: a request for this window.
    pub fn push_inbound(&self, request: Inbound) {
        self.inbox
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(request);
        crate::redraw::poke();
    }

    pub fn take_inbound(&self) -> Vec<Inbound> {
        self.inbox
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
            .collect()
    }

    pub fn report(&self, report: UiReportArgs) {
        self.outbox
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(report);
        self.wake.notify_one();
    }

    /// Answers a launcher's `RaiseWindow` (`Ok`: it did what it could; `Err`:
    /// nothing worked, and the launcher starts a new window).
    pub fn report_raise(&self, id: u64, result: Result<String, String>) {
        let outcome = match result {
            Ok(_) => UiOutcome::Applied {
                state: UiState::default(),
            },
            Err(reason) => UiOutcome::refused(reason),
        };
        self.report(UiReportArgs {
            id,
            body: UiReportBody::Outcome { outcome },
        });
    }

    pub fn publish(&self, info: ViewerInfo) {
        *self.info.lock().unwrap_or_else(|e| e.into_inner()) = Some(info);
        self.info_dirty.store(true, Ordering::SeqCst);
        self.wake.notify_one();
    }

    /// Link side: resolves when there is something to send.
    pub async fn woken(&self) {
        self.wake.notified().await;
    }

    /// Link side: the presence to send if it changed, and the answers.
    pub fn take_outgoing(&self) -> (Option<ViewerInfo>, Vec<UiReportArgs>) {
        let info = if self.info_dirty.swap(false, Ordering::SeqCst) {
            self.info.lock().unwrap_or_else(|e| e.into_inner()).clone()
        } else {
            None
        };
        let reports = std::mem::take(&mut *self.outbox.lock().unwrap_or_else(|e| e.into_inner()));
        (info, reports)
    }
}
