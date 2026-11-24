//! Completion tickets: a way to submit an action and learn when and how it
//! ended.
//!
//! Commands are fire-and-forget: the render loop submits an [`Action`] and the
//! answer is a status line. A caller that has to wait for the result (the
//! sequence runner) submits with [`DaemonLink::submit_tracked`], gets a
//! [`Ticket`], and reads [`StepDone`] events from [`take_done`].
//!
//! Everything here is **process-wide** (statics, not fields of the link or of
//! the window): a tab switch rebuilds the window and its `DaemonLink`, and a
//! ticket must outlive both. A ticket is only ever reported once the action
//! ran on a link thread or a dialog gave up on it, so an untracked action
//! (`Option<Ticket>` = `None` everywhere) behaves exactly as before.
//!
//! A ticket's outcome is final except [`StepOutcome::NeedsUser`], which says
//! "waiting for you": a dialog is open, and its answer resubmits the action
//! under the *same* ticket (auto-stash, sensitive files, a diverged push).
//! Closing such a dialog without answering reports [`StepOutcome::Canceled`];
//! the dialogs hold a [`TicketGuard`] so every way of closing one, a window
//! rebuild included, ends the ticket.

// The sequence runner (a later slice) is the reader of most of this.
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// What a submitted action is known by. Copy: it is carried through dialogs
/// and effects that are cloned and compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ticket(pub u64);

/// Why a ticket is waiting for a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserWait {
    /// `general.auto_stash = ask`: the stash-and-continue dialog.
    AutoStash,
    /// The staging guard found files that look like secrets.
    SensitiveStage,
    /// The push was refused; the diverged dialog (pull then push, force with
    /// lease, cancel) decides how the ticket ends.
    PushRejected,
}

/// What a finished action produced besides "it worked".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepPayload {
    /// The worktree `WorktreeAdd` created.
    Worktree(PathBuf),
}

/// How a ticket's action ended, or that it is waiting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Ok(Option<StepPayload>),
    /// `text` is what the status line got (`error_text`).
    Failed {
        text: String,
    },
    /// Intermediate: a dialog waits for the person. A final outcome follows.
    NeedsUser(UserWait),
    /// The daemon job was canceled, or the dialog was closed without an
    /// answer.
    Canceled,
}

impl StepOutcome {
    /// Whether nothing else will be reported for the ticket.
    pub fn is_final(&self) -> bool {
        !matches!(self, StepOutcome::NeedsUser(_))
    }
}

/// One report on the process-wide queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepDone {
    pub ticket: Ticket,
    pub outcome: StepOutcome,
}

/// How an action's error ends its ticket: a canceled daemon job is
/// `Canceled`, everything else `Failed` with the status line's `text`.
pub(super) fn outcome_of_error(error: &gitpoulpi_core::Error, text: &str) -> StepOutcome {
    match error {
        gitpoulpi_core::Error::Canceled { .. } => StepOutcome::Canceled,
        _ => StepOutcome::Failed {
            text: text.to_string(),
        },
    }
}

/// Oldest reports are dropped past this (nobody is reading).
const QUEUE_CAP: usize = 1024;

static NEXT: AtomicU64 = AtomicU64::new(1);
static NEXT_LINK: AtomicU64 = AtomicU64::new(1);
static EVENTS: Mutex<VecDeque<StepDone>> = Mutex::new(VecDeque::new());
static LIVE: Mutex<Option<HashMap<u64, Live>>> = Mutex::new(None);

/// Where a ticket is in its life.
struct Live {
    /// Parts still to finish: a renamed file is two actions for git, one
    /// ticket.
    remaining: usize,
    /// A final outcome was reported (a failed part ends the ticket at once).
    finished: bool,
    /// A dialog holds it (`NeedsUser`): not running until resubmitted.
    waiting: bool,
    /// The link thread the action went to (see [`abandon_link`]).
    link: u64,
}

fn live<R>(f: impl FnOnce(&mut HashMap<u64, Live>) -> R) -> R {
    let mut slot = LIVE.lock().unwrap_or_else(|e| e.into_inner());
    f(slot.get_or_insert_with(HashMap::new))
}

fn push(done: StepDone) {
    let mut queue = EVENTS.lock().unwrap_or_else(|e| e.into_inner());
    if queue.len() >= QUEUE_CAP {
        queue.pop_front();
    }
    queue.push_back(done);
    drop(queue);
    crate::redraw::poke();
}

/// A new ticket.
pub(crate) fn issue() -> Ticket {
    Ticket(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// `parts` actions are about to run under `ticket` (a first submission or a
/// resubmission after a dialog).
pub(crate) fn begin(ticket: Ticket, parts: usize, link: u64) {
    live(|map| {
        map.insert(
            ticket.0,
            Live {
                remaining: parts.max(1),
                finished: false,
                waiting: false,
                link,
            },
        );
    });
}

/// A new id for a link thread.
pub(crate) fn new_link_id() -> u64 {
    NEXT_LINK.fetch_add(1, Ordering::Relaxed)
}

/// The link thread `link` ended (its window went away, or it lost the
/// daemon): what it still had in flight will never be answered to it, so
/// those tickets fail now instead of hanging a sequence. The action may well
/// have run on the daemon; only the answer is lost.
pub(crate) fn abandon_link(link: u64) {
    let lost: Vec<u64> = live(|map| {
        let ids: Vec<u64> = map
            .iter()
            .filter(|(_, l)| l.link == link && !l.waiting && !l.finished)
            .map(|(id, _)| *id)
            .collect();
        for id in &ids {
            map.remove(id);
        }
        ids
    });
    for id in lost {
        push(StepDone {
            ticket: Ticket(id),
            outcome: StepOutcome::Failed {
                text: gitpoulpi_i18n::msg::link_error::CONNECTION_WAS_LOST
                    .get()
                    .to_string(),
            },
        });
    }
}

/// Whether the ticket's action is submitted and not yet answered (not
/// waiting on a dialog, not finished).
pub(crate) fn ticket_running(ticket: Ticket) -> bool {
    live(|map| {
        map.get(&ticket.0)
            .is_some_and(|l| !l.waiting && !l.finished)
    })
}

/// Reports `outcome` of one part of `ticket`'s action. A no-op for an
/// untracked action. Ok of one part of several is not reported until the last
/// one; the first failure ends the ticket and the rest is ignored.
pub(crate) fn report_ticket(ticket: Option<Ticket>, outcome: StepOutcome) {
    let Some(ticket) = ticket else {
        return;
    };
    let emit = live(|map| {
        let Some(entry) = map.get_mut(&ticket.0) else {
            // Not begun (a dialog reporting on a ticket that was never
            // submitted): reported as it is.
            return Some(outcome);
        };
        if entry.finished {
            entry.remaining = entry.remaining.saturating_sub(1);
            if entry.remaining == 0 {
                map.remove(&ticket.0);
            }
            return None;
        }
        match outcome {
            StepOutcome::NeedsUser(_) => {
                entry.waiting = true;
                Some(outcome)
            }
            StepOutcome::Ok(payload) => {
                entry.remaining = entry.remaining.saturating_sub(1);
                if entry.remaining == 0 {
                    map.remove(&ticket.0);
                    Some(StepOutcome::Ok(payload))
                } else {
                    None
                }
            }
            other => {
                entry.remaining = entry.remaining.saturating_sub(1);
                if entry.remaining == 0 {
                    map.remove(&ticket.0);
                } else {
                    entry.finished = true;
                }
                Some(other)
            }
        }
    });
    if let Some(outcome) = emit {
        push(StepDone { ticket, outcome });
    }
}

/// A dialog gave up on `ticket` (closed without an answer): `Canceled`, final.
pub(crate) fn cancel_ticket(ticket: Ticket) {
    live(|map| {
        map.remove(&ticket.0);
    });
    push(StepDone {
        ticket,
        outcome: StepOutcome::Canceled,
    });
}

/// Every report so far, oldest first (what a sequence runner reads each
/// pass). Takes the queue: other readers see nothing of it.
pub fn take_done() -> Vec<StepDone> {
    EVENTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .drain(..)
        .collect()
}

/// The reports of one ticket, leaving the others queued.
pub fn take_for(ticket: Ticket) -> Vec<StepOutcome> {
    let mut queue = EVENTS.lock().unwrap_or_else(|e| e.into_inner());
    let mut mine = Vec::new();
    queue.retain(|d| {
        if d.ticket == ticket {
            mine.push(d.outcome.clone());
            false
        } else {
            true
        }
    });
    mine
}

/// Holds a ticket while a dialog waits for the person. Dropping it without
/// [`TicketGuard::take`] (the dialog closed, the window rebuilt) reports
/// `Canceled`.
#[derive(Debug)]
pub struct TicketGuard(Option<Ticket>);

impl TicketGuard {
    /// `None` for an untracked action: the guard then does nothing.
    pub fn new(ticket: Option<Ticket>) -> Option<Self> {
        ticket.map(|t| Self(Some(t)))
    }

    /// The ticket, to resubmit under; the guard no longer cancels.
    pub fn take(mut self) -> Ticket {
        self.0.take().expect("a guard holds its ticket until taken")
    }
}

impl Drop for TicketGuard {
    fn drop(&mut self) {
        if let Some(ticket) = self.0.take() {
            cancel_ticket(ticket);
        }
    }
}

tokio::task_local! {
    /// The payload of the action in progress, set by the request that made it.
    static PAYLOAD: std::cell::RefCell<Option<StepPayload>>;
}

/// The action in progress produced `payload` (reported with its `Ok`).
pub(super) fn note_payload(payload: StepPayload) {
    let _ = PAYLOAD.try_with(|slot| *slot.borrow_mut() = Some(payload));
}

/// Runs `work`; returns what it returned and the payload it noted.
pub(super) async fn capture_payload<F: std::future::Future>(
    work: F,
) -> (F::Output, Option<StepPayload>) {
    PAYLOAD
        .scope(std::cell::RefCell::new(None), async {
            let out = work.await;
            let payload = PAYLOAD.with(|slot| slot.borrow_mut().take());
            (out, payload)
        })
        .await
}
