use super::host::NotifySend as _;
use super::*;

/// Runs the link's refreshes off the select loop: one at a time, with
/// whatever was asked for while it ran folded into a single follow-up.
///
/// A refresh's result is delivered only if no mutation was submitted,
/// started or finished since it began (the `Activity` generation) and no
/// local mutation is still running; otherwise it is dropped and its scope
/// re-queued, so stale status can never overwrite the state a mutation
/// produced, optimistic or real.
pub(super) struct Refresher {
    pub(super) conn: LinkConn,
    pub(super) log_depth: Arc<AtomicUsize>,
    pub(super) log_filter: LogFilterCell,
    pub(super) lfs_gate: Arc<crate::lfs_ui::LfsGate>,
    pub(super) activity: Arc<crate::activity::Activity>,
    pub(super) done_tx: tokio::sync::mpsc::UnboundedSender<(RefreshOutcome, u64)>,
    /// Refreshes running now. More than one only when a mutation made the
    /// older one stale (it is left to finish and be dropped, so the fresh
    /// read does not wait for it).
    pub(super) inflight: usize,
    /// The generation the newest running refresh started at.
    pub(super) latest_started: Option<u64>,
    pub(super) queued: Option<ChangeScope>,
    /// What the last delivered outcome carried and the generation it was read
    /// at: an identical one is delivered without waking the render loop.
    pub(super) last_delivered: Option<(u64, u64)>,
    /// The hash of the last LFS snapshot an outcome carried.
    pub(super) last_lfs: Option<u64>,
}

impl Refresher {
    /// Asks for a refresh of `scope`, now or as soon as one may run.
    pub(super) fn want(&mut self, scope: ChangeScope) {
        self.queued = Some(self.queued.map_or(scope, |queued| queued | scope));
        self.kick();
    }

    /// Starts the queued refresh unless one is running or a local mutation
    /// is (its result would be dropped; its end calls this again).
    pub(super) fn kick(&mut self) {
        if self.activity.blocking() {
            return;
        }
        // Single flight per generation: one refresh already reading the
        // current state covers the request.
        if self.inflight > 0 && self.latest_started == Some(self.activity.generation()) {
            return;
        }
        let Some(scope) = self.queued.take() else {
            return;
        };
        let started_at = self.activity.generation();
        self.inflight += 1;
        self.latest_started = Some(started_at);
        let conn = self.conn.clone();
        let log_depth = Arc::clone(&self.log_depth);
        let log_filter = self.log_filter.clone();
        let lfs_gate = Arc::clone(&self.lfs_gate);
        let done_tx = self.done_tx.clone();
        tokio::spawn(refresh_journal(
            self.conn.clone(),
            Arc::clone(&self.activity),
        ));
        tokio::spawn(async move {
            let began = std::time::Instant::now();
            let outcome =
                collect_refresh(&conn, &log_depth, &log_filter, &lfs_gate, scope, false).await;
            // The poll and the refresh after a change: background work.
            super::host::timing(
                super::host::TimingKind::Background,
                &conn.repo.root,
                gitpoulpi_i18n::msg::activity_log::OP_REFRESH
                    .get()
                    .to_string(),
                u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX),
            );
            let _ = done_tx.send((outcome, started_at));
        });
    }

    pub(super) fn finished(
        &mut self,
        mut outcome: RefreshOutcome,
        started_at: u64,
        outcome_tx: &std_mpsc::Sender<RefreshOutcome>,
    ) {
        self.inflight = self.inflight.saturating_sub(1);
        if started_at == self.activity.generation() && !self.activity.blocking() {
            outcome.generation = started_at;
            // Nothing new since the last one (and no mutation in between): the
            // loop takes it whenever it next runs, no frame is due for it.
            let delivered = Some((outcome.fingerprint, started_at));
            // A snapshot equal to the last one seen is no news either; a
            // refresh without one says nothing about LFS.
            let lfs_same = outcome
                .read_hashes
                .lfs
                .is_none_or(|h| self.last_lfs == Some(h));
            if let Some(h) = outcome.read_hashes.lfs {
                self.last_lfs = Some(h);
            }
            let unchanged = self.last_delivered == delivered && lfs_same;
            self.last_delivered = delivered;
            if unchanged {
                let _ = outcome_tx.send(outcome);
            } else {
                let _ = outcome_tx.send_notify(outcome);
            }
        } else {
            // Read before a mutation landed: drop it, ask again. An LFS
            // snapshot it carried was consumed from the gate, so mark it
            // due again.
            if outcome.lfs.is_some() {
                self.lfs_gate.mark_dirty();
            }
            self.queued = Some(
                self.queued
                    .map_or(outcome.scope, |queued| queued | outcome.scope),
            );
        }
        self.kick();
    }

    /// A mutation's answer is in. A failure sends no `RepoChanged`, so ask
    /// for the status that rolls the optimistic rows back.
    pub(super) fn action_finished(&mut self, op: u64, ok: bool) {
        self.activity.end(op);
        if !ok {
            let rollback = ChangeScope::INDEX | ChangeScope::WORKTREE;
            self.queued = Some(self.queued.map_or(rollback, |queued| queued | rollback));
        }
        self.kick();
    }
}
