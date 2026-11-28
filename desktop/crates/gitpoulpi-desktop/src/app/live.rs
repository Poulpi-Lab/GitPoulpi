//! Live updates and the write path of the window.
//!
//! The link's thread wakes the [`DesktopHost`] whenever it queued something. A task on the window's
//! foreground executor waits for that wake ([`Workspace::start_pump`]), drains the link's receivers and
//! applies what arrived: a refresh the daemon pushed because *any* client changed the repository
//! reloads the Log and the Refs sidebar, failures and messages of actions go to the status line.
//! Nothing here polls: no wake, no work.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use gitpoulpi_link::{DaemonLink, ReadHashes, RefreshOutcome};
use gpui_kit::Context;

use super::workspace::Workspace;
use crate::daemon::host::rejection_text;
use crate::daemon::transfer::Transfer;
use crate::strings;

/// One line of the status line about what an action did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub text: String,
    pub error: bool,
}

impl Notice {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: true,
        }
    }
}

/// What the window remembers between two wakes.
#[derive(Default)]
pub struct LiveState {
    /// The hashes of the reads the panels currently show (see `ReadHashes::apply`).
    hashes: ReadHashes,
    /// The working tree the Log's WIP row and the right pane show, hashed.
    status_hash: Option<u64>,
    /// The first outcome only records the baseline: the panels loaded the same state themselves.
    baseline_taken: bool,
    /// Something reached the status line since the last transfer started (so a late "started" line
    /// does not overwrite a result that already arrived).
    settled: bool,
    /// Refreshes applied to the panels, for tests and diagnostics.
    pub refreshes: u64,
}

/// Everything one wake took out of the link.
#[derive(Default)]
struct Drained {
    outcome: Option<RefreshOutcome>,
    filter_generation: u64,
    errors: Vec<String>,
    messages: Vec<String>,
    failure: Option<String>,
}

fn drain(link: &DaemonLink) -> Drained {
    let mut d = Drained {
        filter_generation: link.log_filter_generation(),
        ..Drained::default()
    };
    // Only the newest refresh matters; older ones describe states that are gone.
    while let Ok(outcome) = link.outcome_rx.try_recv() {
        d.outcome = Some(outcome);
    }
    while let Ok(text) = link.action_error_rx.try_recv() {
        d.errors.push(text);
    }
    while let Ok(notice) = link.push_rejected_rx.try_recv() {
        d.errors.push(rejection_text(&notice.rejection));
    }
    while let Ok(text) = link.action_message_rx.try_recv() {
        d.messages.push(text);
    }
    // Other receivers (autostash, file insight, merge, ...) have no desktop surface yet; they are
    // emptied so they do not grow without bound.
    while link.autostash_rx.try_recv().is_ok() {}
    while link.file_insight_rx.try_recv().is_ok() {}
    while link.merge_rx.try_recv().is_ok() {}
    while link.commit_stats_rx.try_recv().is_ok() {}
    while link.split_source_rx.try_recv().is_ok() {}
    while link.working_diff_rx.try_recv().is_ok() {}
    while link.commit_files_rx.try_recv().is_ok() {}
    while link.log_page_rx.try_recv().is_ok() {}
    d.failure = link.failure_rx.try_recv().ok();
    d
}

fn hash_status(outcome: &RefreshOutcome) -> Option<u64> {
    let tree = outcome.status_result.as_ref().ok()?;
    let mut h = DefaultHasher::new();
    format!("{:?}", tree.files).hash(&mut h);
    Some(h.finish())
}

impl Workspace {
    /// Starts the task that applies what the link queues. Call once from `Workspace::new`.
    pub(super) fn start_pump(&mut self, cx: &mut Context<Self>) {
        let host = self.backend.host().clone();
        let mut seen = host.epoch();
        self._pump = Some(cx.spawn(async move |this, cx| loop {
            host.changed(&mut seen).await;
            if this.update(cx, |this, cx| this.on_link_wake(cx)).is_err() {
                break;
            }
        }));
    }

    fn on_link_wake(&mut self, cx: &mut Context<Self>) {
        let drained = self.backend.with_link(drain);
        let news = self.backend.host().take_news(&self.backend.repo_root());
        let mut notice = None;
        if let Some(why) = drained.failure {
            notice = Some(Notice::error(why));
        }
        if let Some(text) = drained.errors.into_iter().last() {
            notice = Some(Notice::error(text));
        } else if let Some(text) = drained.messages.into_iter().last() {
            notice = Some(Notice::info(text));
        } else if let Some(text) = news.said.into_iter().last() {
            notice = Some(Notice::info(text));
        } else if let Some(title) = news.done {
            notice = Some(Notice::info(strings::finished(&title)));
        }
        if let Some(n) = notice {
            self.live.settled = true;
            self.notice = Some(n);
        }
        if let Some(outcome) = drained.outcome {
            self.apply_outcome(outcome, drained.filter_generation, cx);
        }
        cx.notify();
    }

    /// Reloads what a refresh says changed.
    fn apply_outcome(&mut self, outcome: RefreshOutcome, filter_gen: u64, cx: &mut Context<Self>) {
        let changes = self.live.hashes.apply(
            outcome
                .read_hashes
                .for_filter_generation(outcome.log_filter_gen, filter_gen),
        );
        let status_hash = hash_status(&outcome);
        let status_changed = status_hash.is_some() && status_hash != self.live.status_hash;
        if status_hash.is_some() {
            self.live.status_hash = status_hash;
        }
        if !std::mem::replace(&mut self.live.baseline_taken, true) {
            return;
        }
        if changes.log || changes.stashes || status_changed {
            self.log.update(cx, |l, cx| l.reload(cx));
        }
        if changes.refs || changes.stashes {
            self.refs.update(cx, |r, cx| r.reload(cx));
        }
        if changes.log || changes.refs || changes.stashes || status_changed {
            self.live.refreshes += 1;
        }
    }

    /// The window's link to the daemon.
    pub fn backend(&self) -> &crate::daemon::Shared {
        &self.backend
    }

    /// The last line an action left for the status line.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// How many times a daemon-pushed refresh reloaded the panels.
    pub fn live_refreshes(&self) -> u64 {
        self.live.refreshes
    }

    /// Starts Fetch, Pull, Push, Undo or Redo; the status line follows what the daemon does.
    pub fn transfer(&mut self, what: Transfer, cx: &mut Context<Self>) {
        self.notice = None;
        self.live.settled = false;
        let backend = self.backend.clone();
        cx.spawn(async move |this, cx| {
            let started = backend.start_transfer(what).await;
            let _ = this.update(cx, |this, cx| {
                match started {
                    Ok(text) if !this.live.settled => this.notice = Some(Notice::info(text)),
                    Ok(_) => {}
                    Err(why) => this.notice = Some(Notice::error(why)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The repository switch restarts the link: its refreshes start from scratch.
    pub(super) fn reset_live(&mut self) {
        self.live = LiveState::default();
        self.notice = None;
    }
}
