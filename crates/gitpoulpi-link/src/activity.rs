//! What the link is doing right now, for the status line and for the
//! refresh/mutation ordering rules.
//!
//! One registry keyed by the in-flight operation, not per-action ad hoc
//! text. The link registers every action when it starts it and removes it
//! when the daemon has answered; the render loop reads
//! [`Activity::in_flight_text`] each frame to show a spinner and a label
//! ("Staging all changes...") for anything that takes longer than a
//! moment, and clears it on completion (an error replaces it via
//! `action_error_rx`).
//!
//! It also carries the **generation counter**:
//! bumped when a mutation is submitted and again when it finishes. A refresh
//! remembers the generation it started at, and its result is dropped when
//! the number has moved, so a status read that began before a mutation can
//! never overwrite the state the mutation (or the optimistic overlay)
//! produced. The refresh it displaced is simply run again.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::client::Action;

/// How long an operation runs before the status line starts to talk about
/// it: shorter ones finish before the user could read the text.
pub const SHOW_AFTER: Duration = Duration::from_millis(150);

struct Op {
    id: u64,
    label: String,
    /// A local mutation (index, worktree, refs): refresh results are held
    /// back until it ends. Network operations do not block refreshes.
    blocking: bool,
    /// Runs as a daemon job: the jobs indicator already names it (with a
    /// Cancel), so the status line does not say it a second time.
    job: bool,
    started: Instant,
}

#[derive(Default)]
struct Inner {
    next: u64,
    ops: Vec<Op>,
    /// Actions the render loop has not yet applied to its own copy of the
    /// status (see `status_optimistic`).
    optimistic: Vec<Action>,
    /// Transfers submitted and not yet answered, so a second identical press
    /// joins the first instead of being sent (see the background-ops design, B10).
    transfers: Vec<Action>,
}

#[derive(Default)]
pub struct Activity {
    /// The undo journal as the window shows it (`undo_ui`): kept here
    /// because this is the one object the link's reader, refresher and
    /// action tasks already share with the render loop.
    pub journal: crate::undo_ui::JournalCell,
    /// Requests from an agent and the answers to them (`ui_viewer`), here
    /// for the same reason.
    pub ui: crate::ui_viewer::UiBridge,
    /// Requests views make with their own typed answers (`daemon_link::raw`).
    pub raw: crate::client::RawQueue,
    generation: AtomicU64,
    /// The latest report of the history rewrite preview running for this
    /// repository: the daemon job and how far it got. Set by the reader on
    /// `Event::RewriteProgress`, read by the workbench each frame.
    rewrite_progress: Mutex<Option<(u64, gitpoulpi_core::domain::rewrite::RewriteProgress)>>,
    /// Set by the link's reader on `Event::RefVisibilityChanged`; the
    /// render loop takes it and reloads the hidden/solo sets.
    ref_visibility_changed: std::sync::atomic::AtomicBool,
    /// Set on `Event::PendingAutoStashChanged`; the render loop re-reads
    /// the record and the banner follows.
    pending_auto_stash_changed: std::sync::atomic::AtomicBool,
    /// Set on `Event::PendingAutoStashGone`: the index the record last had,
    /// stored as `index + 1` so that 0 means none.
    pending_auto_stash_gone: std::sync::atomic::AtomicU64,
    /// Set on `Event::HostingItemChanged`: what other clients wrote to an
    /// issue or a request since the render loop last looked.
    hosting_changes: Mutex<Vec<gitpoulpi_ipc::HostingChangeArgs>>,
    /// The rename pairs of the latest status (`rename_pairs`): which paths a
    /// staging or discard of one row must bring along.
    rename_pairs: Mutex<crate::rename_pairs::RenamePairs>,
    inner: Mutex<Inner>,
}

impl Activity {
    /// The registry with the journal seeded from `general.undo_enabled`
    /// (see `JournalCell::seeded`).
    pub fn with_undo_enabled(undo_enabled: bool) -> Self {
        Self {
            journal: crate::undo_ui::JournalCell::seeded(undo_enabled),
            ..Self::default()
        }
    }

    /// Remembers the rename pairs of a fresh status.
    pub fn set_rename_pairs(&self, tree: &gitpoulpi_core::domain::WorkingTree) {
        let pairs = crate::rename_pairs::RenamePairs::of(tree);
        *self.rename_pairs.lock().unwrap_or_else(|e| e.into_inner()) = pairs;
    }

    /// `action` with what its rename partners need (`rename_pairs`).
    pub fn expand_renames(&self, action: Action) -> Vec<Action> {
        self.rename_pairs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .expand(action)
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    /// Records a preview's progress (called from the link's reader task).
    pub fn set_rewrite_progress(
        &self,
        job: u64,
        progress: gitpoulpi_core::domain::rewrite::RewriteProgress,
    ) {
        *self
            .rewrite_progress
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some((job, progress));
        crate::redraw::poke();
    }

    /// The latest preview progress, if a preview reported one.
    pub fn rewrite_progress(
        &self,
    ) -> Option<(u64, gitpoulpi_core::domain::rewrite::RewriteProgress)> {
        *self
            .rewrite_progress
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// An issue or request was written to by another client.
    pub fn mark_hosting_item_changed(&self, change: gitpoulpi_ipc::HostingChangeArgs) {
        // This window's own write coming back: it already shows it.
        if crate::client::host::is_own_hosting_change(&change) {
            return;
        }
        self.hosting_changes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(change);
        crate::redraw::poke();
    }

    pub fn take_hosting_item_changes(&self) -> Vec<gitpoulpi_ipc::HostingChangeArgs> {
        std::mem::take(
            &mut *self
                .hosting_changes
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
        )
    }

    pub fn mark_ref_visibility_changed(&self) {
        self.ref_visibility_changed.store(true, Ordering::SeqCst);
        crate::redraw::poke();
    }

    pub fn take_ref_visibility_changed(&self) -> bool {
        self.ref_visibility_changed.swap(false, Ordering::SeqCst)
    }

    pub fn mark_pending_auto_stash_changed(&self) {
        self.pending_auto_stash_changed
            .store(true, Ordering::SeqCst);
        crate::redraw::poke();
    }

    pub fn mark_pending_auto_stash_gone(&self, index: u32) {
        self.pending_auto_stash_gone
            .store(u64::from(index) + 1, Ordering::SeqCst);
        crate::redraw::poke();
    }

    /// The index of an auto-stash whose record the daemon cleared because
    /// the stash vanished, once.
    pub fn take_pending_auto_stash_gone(&self) -> Option<u32> {
        match self.pending_auto_stash_gone.swap(0, Ordering::SeqCst) {
            0 => None,
            n => Some((n - 1) as u32),
        }
    }

    pub fn take_pending_auto_stash_changed(&self) -> bool {
        self.pending_auto_stash_changed
            .swap(false, Ordering::SeqCst)
    }

    pub fn bump(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        crate::redraw::poke();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Registers an operation as running. Returns its id for [`end`](Self::end).
    pub fn begin(&self, label: String, blocking: bool) -> u64 {
        self.begin_as(label, blocking, false)
    }

    /// [`begin`](Self::begin) for an operation that may run as a daemon job.
    /// Its label stays off the status line while a job is on the indicator.
    pub fn begin_as(&self, label: String, blocking: bool, job: bool) -> u64 {
        self.bump();
        let mut inner = self.lock();
        inner.next += 1;
        let id = inner.next;
        inner.ops.push(Op {
            id,
            label,
            blocking,
            job,
            started: Instant::now(),
        });
        drop(inner);
        // After the push: a loop woken by this must find the operation.
        crate::redraw::poke();
        id
    }

    pub fn end(&self, id: u64) {
        self.bump();
        self.lock().ops.retain(|op| op.id != id);
        crate::redraw::poke();
    }

    /// Whether a local mutation is running (refresh results wait for it).
    /// What the render loop owes the status line's in-flight text: `Some(at)`
    /// when the oldest operation has not yet run for [`SHOW_AFTER`] (the text
    /// appears at `at`), `None` otherwise. See [`Self::spinning`].
    pub fn text_due_at(&self) -> Option<Instant> {
        let hide = crate::client::host::daemon_job_live();
        let inner = self.lock();
        let oldest = inner.ops.iter().find(|op| !(hide && op.job))?;
        (oldest.started.elapsed() < SHOW_AFTER).then(|| oldest.started + SHOW_AFTER)
    }

    /// The in-flight text is on screen with its spinner, which moves.
    pub fn spinning(&self) -> bool {
        let hide = crate::client::host::daemon_job_live();
        let inner = self.lock();
        inner
            .ops
            .iter()
            .find(|op| !(hide && op.job))
            .is_some_and(|oldest| oldest.started.elapsed() >= SHOW_AFTER)
    }

    pub fn blocking(&self) -> bool {
        self.lock().ops.iter().any(|op| op.blocking)
    }

    /// Records a submitted action for the optimistic overlay and bumps the
    /// generation synchronously, on the caller's thread, so a refresh
    /// outcome already on its way to the render loop is recognized as stale.
    pub fn submitted(&self, action: &Action) {
        self.bump();
        if crate::client::is_optimistic(action) {
            self.lock().optimistic.push(action.clone());
        }
    }

    /// Registers a transfer (push, pull, fetch, LFS transfer) about to be
    /// sent. `Some(what)` when an identical one is already running: the
    /// caller must not send it and says "already <what>" instead.
    pub fn claim_transfer(&self, action: &Action) -> Option<TransferKind> {
        let kind = TransferKind::of(action)?;
        let mut inner = self.lock();
        if inner.transfers.contains(action) {
            return Some(kind);
        }
        inner.transfers.push(action.clone());
        None
    }

    /// The transfer was answered: an identical press is a new one again.
    pub fn release_transfer(&self, action: &Action) {
        let mut inner = self.lock();
        if let Some(i) = inner.transfers.iter().position(|a| a == action) {
            inner.transfers.remove(i);
        }
    }

    pub fn take_optimistic(&self) -> Vec<Action> {
        std::mem::take(&mut self.lock().optimistic)
    }

    /// The status-line text for the longest-running operation, once it has
    /// run for at least `after`, e.g. `Committing... (+1 more)`.
    pub fn in_flight_text(&self, after: Duration) -> Option<String> {
        self.in_flight_text_hiding(after, crate::client::host::daemon_job_live())
    }

    /// [`in_flight_text`](Self::in_flight_text) with `hide_jobs` saying
    /// whether the operations that run as jobs are left out (the jobs
    /// indicator shows them).
    pub fn in_flight_text_hiding(&self, after: Duration, hide_jobs: bool) -> Option<String> {
        let inner = self.lock();
        let mut shown = inner.ops.iter().filter(|op| !(hide_jobs && op.job));
        let oldest = shown.next()?;
        if oldest.started.elapsed() < after {
            return None;
        }
        let more = shown.count();
        Some(if more == 0 {
            oldest.label.clone()
        } else {
            format!("{} (+{more} more)", oldest.label)
        })
    }
}

/// What a duplicate transfer press is told it joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferKind {
    Push,
    Pull,
    Fetch,
    Lfs,
}

impl TransferKind {
    /// The kind of a transfer the daemon joins when it is sent twice; `None`
    /// for every other action (they queue or run as before).
    pub fn of(action: &Action) -> Option<Self> {
        match action {
            Action::Push { .. } | Action::PushForReview { .. } => Some(Self::Push),
            Action::Pull(_) => Some(Self::Pull),
            Action::Fetch(_) | Action::FetchAll => Some(Self::Fetch),
            Action::LfsFetch(_) | Action::LfsPull(_) | Action::LfsPush(_) => Some(Self::Lfs),
            _ => None,
        }
    }

    pub fn already(self) -> String {
        use gitpoulpi_i18n::msg::jobs as m;
        match self {
            Self::Push => m::ALREADY_PUSHING,
            Self::Pull => m::ALREADY_PULLING,
            Self::Fetch => m::ALREADY_FETCHING,
            Self::Lfs => m::ALREADY_LFS,
        }
        .get()
        .to_string()
    }
}

/// The braille spinner frame for `elapsed`.
pub fn spinner_frame(elapsed: Duration) -> char {
    const FRAMES: [char; 10] = [
        '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}',
        '\u{2827}', '\u{2807}', '\u{280F}',
    ];
    FRAMES[(elapsed.as_millis() / 80) as usize % FRAMES.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_echo_of_this_windows_own_hosting_write_is_dropped() {
        use gitpoulpi_ipc::{HostingChange, HostingChangeArgs, HostingItemKind};
        crate::client::host::install(crate::client::test_host::host());
        let activity = Activity::default();
        let change = |origin: &str| HostingChangeArgs {
            integration_id: "gh".into(),
            project: "o/r".into(),
            kind: HostingItemKind::Issue,
            number: 3,
            id: String::new(),
            change: HostingChange::Posted,
            origin: origin.into(),
        };
        activity.mark_hosting_item_changed(change(crate::client::test_host::OWN_ORIGIN));
        assert!(activity.take_hosting_item_changes().is_empty());
        // Another window's, and the CLI's (no origin), still refresh.
        activity.mark_hosting_item_changed(change("tui-other"));
        activity.mark_hosting_item_changed(change(""));
        assert_eq!(activity.take_hosting_item_changes().len(), 2);
    }

    #[test]
    fn nothing_is_shown_before_the_threshold_and_it_clears_on_end() {
        let activity = Activity::default();
        let id = activity.begin("Staging all changes\u{2026}".into(), true);
        assert!(activity.in_flight_text(Duration::from_secs(60)).is_none());
        assert_eq!(
            activity.in_flight_text(Duration::ZERO).as_deref(),
            Some("Staging all changes\u{2026}")
        );
        assert!(activity.blocking());
        activity.end(id);
        assert!(activity.in_flight_text(Duration::ZERO).is_none());
        assert!(!activity.blocking());
    }

    #[test]
    fn the_generation_moves_on_submit_begin_and_end() {
        let activity = Activity::default();
        let g0 = activity.generation();
        activity.submitted(&Action::StageAll);
        let g1 = activity.generation();
        let id = activity.begin("x".into(), true);
        activity.end(id);
        assert!(g1 > g0 && activity.generation() > g1);
    }

    #[test]
    fn extra_operations_are_counted_not_listed() {
        let activity = Activity::default();
        activity.begin("Committing\u{2026}".into(), true);
        activity.begin("Fetching\u{2026}".into(), false);
        assert_eq!(
            activity.in_flight_text(Duration::ZERO).as_deref(),
            Some("Committing\u{2026} (+1 more)")
        );
    }

    #[test]
    fn a_job_operation_is_left_out_while_the_indicator_shows_jobs() {
        let activity = Activity::default();
        activity.begin_as("Fetching\u{2026}".into(), false, true);
        assert_eq!(
            activity
                .in_flight_text_hiding(Duration::ZERO, false)
                .as_deref(),
            Some("Fetching\u{2026}")
        );
        assert!(activity
            .in_flight_text_hiding(Duration::ZERO, true)
            .is_none());
        activity.begin("Staging\u{2026}".into(), true);
        assert_eq!(
            activity
                .in_flight_text_hiding(Duration::ZERO, true)
                .as_deref(),
            Some("Staging\u{2026}")
        );
    }
}
