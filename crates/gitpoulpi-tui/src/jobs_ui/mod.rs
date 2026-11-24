//! Background jobs in the window: the status bar indicator, the jobs panel
//! and their keyboard and palette twins (see the background-ops design, slices
//! B6 and B7).
//!
//! **One process-wide cell.** The window restarts on every tab switch, so
//! the jobs live outside it, like the Activity Log: every link's reader task
//! feeds the cell from the daemon's `JobChanged` / `JobRemoved` events and
//! seeds it with `JobList` when it connects. Every window (a repository tab,
//! the Home tab) therefore sees every job of the session. Nothing here reads
//! a file or spawns a process on the render thread: the indicator and the
//! panel are painted from the cell, and a cancel is a `JobCancel` request
//! handed to the link thread (`Requester`) whose answer is polled.
//!
//! **Every Cancel is clickable.** The indicator's `[x]`, each panel row's
//! Cancel button and the panel's Cancel all are hit rects recorded by the
//! walk that paints them; each has a key (`c` / `Del` in the panel, the
//! panel itself on `Ctrl+N Shift+J`) and a palette command (`Jobs: ...`).
//! Cancelling is not an undoable mutation: a canceled push, fetch or clone
//! leaves the repository as it was, there is nothing to undo.

pub(crate) mod indicator;
mod panel;
mod wording;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_ipc::{JobInfo, JobState};
use ratatui::layout::Rect;

use crate::daemon_link::{Reply, Requester};

pub(crate) use indicator::{clear as clear_indicator, paint as paint_indicator, width_for};
pub(crate) use panel::{is_open as panel_open, on_key as panel_key, render as render_panel};
pub(crate) use wording::{label_of_id, reason};

/// How long a finished job stays on the indicator (✓ / ✗).
const FLASH: Duration = Duration::from_secs(5);
/// A job that ended well after less than this never flashes (an auto-fetch
/// that took 200 ms is not news).
const FLASH_MIN_RUN: Duration = Duration::from_millis(1000);
/// The spinner and elapsed redraw cadence while a job runs.
const SPIN: Duration = Duration::from_millis(100);

/// One job as the window knows it.
#[derive(Clone)]
pub(super) struct Tracked {
    pub(super) info: JobInfo,
    pub(super) first_seen: Instant,
    pub(super) finished_at: Option<Instant>,
}

/// Ids from here up are client-side jobs (the daemon counts from 1 and
/// never gets near).
pub(super) const LOCAL_BASE: u64 = 1 << 62;

/// Whether `id` is a job of this process, not the daemon's.
pub(crate) fn is_local(id: u64) -> bool {
    id >= LOCAL_BASE
}

/// How a client-side job is stopped.
pub(super) type LocalCancel = std::sync::Arc<dyn Fn() + Send + Sync>;

/// A `JobCancel` waiting for its answer.
struct Pending {
    label: String,
    reply: Reply<JobInfo>,
}

/// A mouse press the panel still has to tell apart from a double click.
#[derive(Clone, Copy)]
pub(super) struct ClickPair {
    pub(super) id: u64,
    pub(super) at: Instant,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PanelHit {
    Row(u64),
    Cancel(u64),
    CancelAll,
}

/// Everything the module keeps: the jobs, the link's requester, the panel's
/// state and the rects the last frame painted.
pub(super) struct Cell {
    pub(super) jobs: BTreeMap<u64, Tracked>,
    requester: Option<Requester>,
    pending: Vec<Pending>,
    notice: Option<String>,
    /// The last refusal, kept for the panel's notice row (the status line
    /// takes `notice` the next frame).
    pub(super) panel_notice: Option<String>,
    /// The repository an Enter in the panel asked to show (taken by the
    /// event loop, which owns the tabs).
    goto: Option<RepoId>,
    // ---- the indicator ----
    pub(super) ind_label: Rect,
    pub(super) ind_cancel: Rect,
    pub(super) ind_target: Option<u64>,
    /// What stops a client-side job (an AI request, a workspace walk), by
    /// its local id. Process-wide like the jobs themselves, so a Cancel works
    /// while the window that started the work is gone.
    pub(super) cancels: BTreeMap<u64, LocalCancel>,
    next_local: u64,
    pub(super) pointer: Option<(u16, u16)>,
    // ---- the panel ----
    pub(super) open: bool,
    pub(super) selected: Option<u64>,
    pub(super) offset: usize,
    pub(super) follow: bool,
    pub(super) grab: Option<i32>,
    pub(super) window: Rect,
    pub(super) close: Rect,
    pub(super) list: Rect,
    pub(super) body: Rect,
    pub(super) lines: usize,
    pub(super) hits: Vec<(Rect, PanelHit)>,
    pub(super) last_click: Option<ClickPair>,
}

impl Cell {
    const fn new() -> Self {
        Self {
            jobs: BTreeMap::new(),
            requester: None,
            pending: Vec::new(),
            notice: None,
            panel_notice: None,
            goto: None,
            ind_label: Rect::new(0, 0, 0, 0),
            ind_cancel: Rect::new(0, 0, 0, 0),
            ind_target: None,
            cancels: BTreeMap::new(),
            next_local: 0,
            pointer: None,
            open: false,
            selected: None,
            offset: 0,
            follow: false,
            grab: None,
            window: Rect::new(0, 0, 0, 0),
            close: Rect::new(0, 0, 0, 0),
            list: Rect::new(0, 0, 0, 0),
            body: Rect::new(0, 0, 0, 0),
            lines: 0,
            hits: Vec::new(),
            last_click: None,
        }
    }
}

#[cfg(not(test))]
static CELL: std::sync::Mutex<Cell> = std::sync::Mutex::new(Cell::new());

// A unit test gets a cell of its own (tests run one per thread), so one
// that seeds jobs can never show them in another test's footer.
#[cfg(test)]
thread_local! {
    static CELL: std::cell::RefCell<Cell> = const { std::cell::RefCell::new(Cell::new()) };
}

pub(super) fn with<R>(f: impl FnOnce(&mut Cell) -> R) -> R {
    #[cfg(not(test))]
    {
        f(&mut CELL.lock().unwrap_or_else(|e| e.into_inner()))
    }
    #[cfg(test)]
    {
        CELL.with(|c| f(&mut c.borrow_mut()))
    }
}

// ---- feeding the cell (the link's reader task) ----

/// The daemon's list at connect: the truth for that moment.
pub(crate) fn seed(list: Vec<JobInfo>) {
    with(|c| {
        let now = Instant::now();
        let mut next: BTreeMap<u64, Tracked> = c
            .jobs
            .iter()
            .filter(|(id, _)| is_local(**id))
            .map(|(id, t)| (*id, t.clone()))
            .collect();
        for info in list {
            let old = c.jobs.remove(&info.id);
            let finished_at = info
                .state
                .is_finished()
                .then(|| old.as_ref().and_then(|o| o.finished_at).unwrap_or(now));
            next.insert(
                info.id,
                Tracked {
                    first_seen: old.map_or(now, |o| o.first_seen),
                    finished_at,
                    info,
                },
            );
        }
        c.jobs = next;
    });
    crate::redraw::poke();
}

/// `Event::JobChanged`.
pub(crate) fn changed(info: JobInfo) {
    with(|c| {
        let now = Instant::now();
        let finished = info.state.is_finished();
        match c.jobs.get_mut(&info.id) {
            Some(t) => {
                if finished && t.finished_at.is_none() {
                    t.finished_at = Some(now);
                }
                t.info = info;
            }
            None => {
                let id = info.id;
                c.jobs.insert(
                    id,
                    Tracked {
                        finished_at: finished.then_some(now),
                        first_seen: now,
                        info,
                    },
                );
            }
        }
    });
    crate::redraw::poke();
}

/// `Event::JobRemoved`.
pub(crate) fn removed(id: u64) {
    with(|c| {
        c.jobs.remove(&id);
    });
    crate::redraw::poke();
}

/// The newest link's way to talk to the daemon.
pub(crate) fn set_requester(requester: Requester) {
    with(|c| c.requester = Some(requester));
}

// ---- reading the cell ----

/// The jobs still going (queued, running, canceling), oldest first.
pub(super) fn active() -> Vec<JobInfo> {
    with(|c| {
        c.jobs
            .values()
            .filter(|t| !t.info.state.is_finished())
            .map(|t| t.info.clone())
            .collect()
    })
}

/// The part of the work and the percentage of the newest unfinished job of
/// `kind`, for a view that waits on its answer (the purge screen).
pub(crate) fn running_progress(
    kind: gitpoulpi_core::domain::OperationKind,
) -> Option<(Option<String>, Option<u8>)> {
    with(|c| {
        c.jobs
            .values()
            .rev()
            .find(|t| t.info.kind == kind && !t.info.state.is_finished())
            .map(|t| (t.info.phase.clone(), t.info.progress))
    })
}

/// A daemon job is going: the indicator names it, so the status line's
/// in-flight label for a fetch, pull or push stays off.
pub(crate) fn daemon_job_live() -> bool {
    with(|c| {
        c.jobs
            .iter()
            .any(|(id, t)| !is_local(*id) && !t.info.state.is_finished())
    })
}

/// A job that ended a moment ago and is still worth a glance: failures and
/// cancels always, successes only when they took a while.
pub(super) fn flash(now: Instant) -> Option<JobInfo> {
    with(|c| {
        c.jobs
            .values()
            .filter(|t| {
                let Some(end) = t.finished_at else {
                    return false;
                };
                now.saturating_duration_since(end) < FLASH
                    && (t.info.state != JobState::Done
                        || end.saturating_duration_since(t.first_seen) >= FLASH_MIN_RUN)
            })
            .max_by_key(|t| t.finished_at)
            .map(|t| t.info.clone())
    })
}

/// Every job the daemon still lists, active ones first.
pub(super) fn all() -> Vec<JobInfo> {
    with(|c| {
        let mut jobs: Vec<JobInfo> = c.jobs.values().map(|t| t.info.clone()).collect();
        jobs.sort_by_key(|j| (j.state.is_finished(), j.id));
        jobs
    })
}

/// When the window first heard of job `id` (its start, near enough).
pub(super) fn first_seen(id: u64) -> Option<Instant> {
    with(|c| c.jobs.get(&id).map(|t| t.first_seen))
}

pub(super) fn find(id: u64) -> Option<JobInfo> {
    with(|c| c.jobs.get(&id).map(|t| t.info.clone()))
}

/// The jobs a Cancel could stop right now.
pub(crate) fn cancellable() -> Vec<JobInfo> {
    active()
        .into_iter()
        .filter(JobInfo::is_cancellable)
        .collect()
}

/// Whether any job is going (the palette's "Cancel all" is useless without).
pub(crate) fn any_cancellable() -> bool {
    !cancellable().is_empty()
}

/// A hash of what the per-job palette rows depend on, so the palette's
/// haystack cache rebuilds when a job starts or ends.
pub(crate) fn palette_signature() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for job in cancellable() {
        job.id.hash(&mut h);
    }
    h.finish()
}

/// When the next frame is due on its own: the spinner and the elapsed text
/// move while a job runs, and a flash ends.
pub(crate) fn next_deadline() -> Option<Instant> {
    let now = Instant::now();
    if with(|c| c.jobs.values().any(|t| !t.info.state.is_finished())) {
        return Some(now + SPIN);
    }
    with(|c| {
        c.jobs
            .values()
            .filter_map(|t| t.finished_at)
            .map(|end| end + FLASH)
            .filter(|due| *due > now)
            .min()
    })
}

// ---- cancelling ----

/// Registers a job of this process (an AI request, a workspace walk) so it
/// shows in the indicator, the panel and the palette like the daemon's. `label`
/// is already worded for the user; `cancel` stops the work and may be called
/// from any thread, any number of times.
pub(crate) fn local_start(label: String, cancel: LocalCancel) -> u64 {
    let id = with(|c| {
        c.next_local += 1;
        let id = LOCAL_BASE + c.next_local;
        let info = JobInfo {
            id,
            // The kind is unused for a local job; `label` is the wording.
            kind: OperationKind::Status,
            repo: None,
            dest: None,
            label,
            state: JobState::Running,
            phase: None,
            progress: None,
            started_at: None,
            joiners: 0,
            queued_after: None,
            not_cancellable: None,
            error: None,
        };
        c.jobs.insert(
            id,
            Tracked {
                info,
                first_seen: Instant::now(),
                finished_at: None,
            },
        );
        c.cancels.insert(id, cancel);
        id
    });
    crate::redraw::poke();
    id
}

/// Rewords a running local job (a download's percentage).
pub(crate) fn local_relabel(id: u64, label: String) {
    with(|c| {
        if let Some(t) = c.jobs.get_mut(&id) {
            t.info.label = label;
        }
    });
}

/// A call to a host that has been sent and cannot be taken back: listed, with
/// a disabled Cancel that says why.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn local_start_sent(label: String) -> u64 {
    let id = local_start(label, std::sync::Arc::new(|| {}));
    with(|c| {
        if let Some(t) = c.jobs.get_mut(&id) {
            t.info.not_cancellable = Some(gitpoulpi_core::NotCancellableReason::Sent);
        }
    });
    id
}

/// Ends a local job (idempotent). A canceled or failed one flashes on the
/// indicator, a success that was quick does not.
pub(crate) fn local_finish(id: u64, state: JobState) {
    with(|c| {
        c.cancels.remove(&id);
        if let Some(t) = c.jobs.get_mut(&id) {
            if !t.info.state.is_finished() {
                t.info.state = state;
                t.finished_at = Some(Instant::now());
            }
        }
    });
    crate::redraw::poke();
}

/// A line for the status bar (a duplicate press that joined a running job).
pub(crate) fn say(text: String) {
    with(|c| c.notice = Some(text));
    crate::redraw::poke();
}

/// Asks the daemon to cancel `id`. A job that cannot be stopped says why
/// on the status line instead; nothing is sent for it.
pub(crate) fn cancel(id: u64) {
    let Some(job) = find(id) else {
        return;
    };
    if let Some(why) = wording::refusal(&job) {
        with(|c| {
            c.panel_notice = Some(why.clone());
            c.notice = Some(why);
        });
        crate::redraw::poke();
        return;
    }
    if is_local(id) {
        let stop = with(|c| c.cancels.get(&id).cloned());
        if let Some(stop) = stop {
            with(|c| {
                if let Some(t) = c.jobs.get_mut(&id) {
                    t.info.state = JobState::Cancelling;
                }
            });
            stop();
            crate::redraw::poke();
        }
        return;
    }
    let label = wording::label(&job);
    with(|c| {
        let Some(requester) = c.requester.clone() else {
            return;
        };
        let reply =
            requester.call::<JobInfo>(OperationKind::JobCancel, serde_json::json!({ "id": id }));
        c.pending.push(Pending { label, reply });
    });
}

/// What [`cancel_daemon_since`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DaemonCancel {
    /// No daemon job of the run's step is going.
    None,
    /// `JobCancel` was sent.
    Sent,
    /// A job is going that cannot be stopped (it already reached the
    /// remote): the step ends by itself.
    NotCancellable,
}

/// A sequence was canceled: stops the daemon jobs that started since `since`
/// (the step in flight). Only jobs this window first heard of after `since`
/// count, so a background fetch that was already running is left alone.
pub(crate) fn cancel_daemon_since(since: Instant) -> DaemonCancel {
    let mine: Vec<JobInfo> = with(|c| {
        c.jobs
            .iter()
            .filter(|(id, t)| {
                !is_local(**id) && !t.info.state.is_finished() && t.first_seen >= since
            })
            .map(|(_, t)| t.info.clone())
            .collect()
    });
    let mut out = DaemonCancel::None;
    for job in mine {
        if wording::refusal(&job).is_some() {
            if out == DaemonCancel::None {
                out = DaemonCancel::NotCancellable;
            }
        } else {
            cancel(job.id);
            out = DaemonCancel::Sent;
        }
    }
    out
}

/// Cancels every job that can be.
pub(crate) fn cancel_all() {
    for job in cancellable() {
        cancel(job.id);
    }
    crate::redraw::poke();
}

/// The indicator's `[x]` and its keyboard twin: the oldest job that can be
/// canceled, else the reason the first job gives.
pub(crate) fn cancel_oldest() {
    let jobs = active();
    if let Some(job) = jobs.iter().find(|j| j.is_cancellable()) {
        cancel(job.id);
    } else if let Some(job) = jobs.first() {
        cancel(job.id);
    }
}

/// Polls the answers of the cancels in flight and returns the line the
/// window should show (a refusal, usually; a canceled job is announced by
/// the error of the action that was running).
pub(crate) fn take_notice() -> Option<String> {
    with(|c| {
        let mut still = Vec::new();
        for p in std::mem::take(&mut c.pending) {
            match p.reply.take() {
                None => still.push(p),
                Some(Ok(_)) => {}
                Some(Err(error)) => {
                    let text = crate::daemon_link::error_text::error_text(&error);
                    c.notice = Some(format!("{}: {text}", p.label));
                }
            }
        }
        c.pending = still;
        c.notice.take()
    })
}

/// The repository the panel's Enter asked to go to.
pub(crate) fn take_goto() -> Option<RepoId> {
    with(|c| c.goto.take())
}

// ---- commands (palette and keymap) ----

/// A `Jobs: ...` palette command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobsCmd {
    /// `Jobs: Show background jobs`: opens the panel (`Ctrl+N Shift+J`).
    Show,
    /// `Jobs: Cancel <label>`: one row per job that can be canceled.
    Cancel(u64),
    /// `Jobs: Cancel all that can be canceled`.
    CancelAll,
}

pub(crate) fn run(cmd: JobsCmd) {
    match cmd {
        JobsCmd::Show => panel::open(),
        JobsCmd::Cancel(id) => cancel(id),
        JobsCmd::CancelAll => cancel_all(),
    }
}

/// Whether a click at the point does something: the pointing hand. In the
/// open panel that is its cross and its Cancel buttons (a row keeps the arrow); otherwise the status
/// bar's indicator (its label and its `[x]`).
pub(crate) fn pointer_hand(col: u16, row: u16) -> bool {
    let at = (col, row).into();
    if panel::is_open() {
        // The rows are not controls: they keep the arrow and light as a whole.
        return with(|c| {
            c.close.contains(at)
                || c.hits
                    .iter()
                    .any(|(r, h)| !matches!(h, PanelHit::Row(_)) && r.contains(at))
        });
    }
    indicator::hit(col, row).is_some()
}

/// The mouse, before the panes: the open panel takes every event; otherwise
/// a press on the indicator opens the panel or cancels. `true` when the
/// event was used.
pub(crate) fn route_mouse(mouse: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::{MouseButton, MouseEventKind};
    with(|c| c.pointer = Some((mouse.column, mouse.row)));
    if panel::is_open() {
        panel::on_mouse(mouse);
        return true;
    }
    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
        match indicator::hit(mouse.column, mouse.row) {
            Some(indicator::IndicatorHit::Cancel) => {
                cancel_oldest();
                return true;
            }
            Some(indicator::IndicatorHit::Label) => {
                panel::open();
                return true;
            }
            None => {}
        }
    }
    false
}
