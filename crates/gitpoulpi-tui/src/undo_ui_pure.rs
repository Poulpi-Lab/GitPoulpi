//! The pure half of the undo vocabulary: the journal cell, the pending
//! confirmation and the reasons a command cannot run, all plain functions
//! over a `JournalState`. No terminal types; re-exported by `undo_ui`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use gitpoulpi_core::domain::{Direction, EntrySummary, JournalState, UndoRefusal};
use gitpoulpi_i18n::msg;

/// Why the Undo button and the palette entry are idle: there is nothing in
/// the journal, and a push or any other operation on a server never is.
pub(crate) fn nothing_to_undo() -> &'static str {
    if outward_comment() {
        msg::undo_ui::NOTHING_TO_UNDO_COMMENT.get()
    } else {
        msg::undo_ui::NOTHING_TO_UNDO.get()
    }
}

thread_local! {
    static OUTWARD_COMMENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A comment was posted, edited or deleted on a host from this window: that
/// is outward and has no journal entry, so when Undo has nothing to offer it
/// says why (the comment itself is changed or deleted from its own view).
#[cfg_attr(
    not(all(feature = "hosting", feature = "conversation")),
    allow(dead_code)
)]
pub(crate) fn note_outward_comment() {
    OUTWARD_COMMENT.with(|c| c.set(true));
}

pub(crate) fn outward_comment() -> bool {
    OUTWARD_COMMENT.with(std::cell::Cell::get)
}

/// An undo or redo the daemon would only do after the user agreed
/// (`TypedError::NeedsConfirmation`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingConfirm {
    pub(crate) direction: Direction,
    /// The entry the user was shown, so a confirmation never undoes
    /// something else that became the latest in the meantime.
    pub(crate) expect: Option<u64>,
    pub(crate) prompt: String,
    /// "Undo whole sequence": the entries still to undo, this one first.
    pub(crate) sequence: Option<gitpoulpi_core::domain::SequenceGroup>,
}

#[derive(Default)]
struct Inner {
    state: JournalState,
    confirm: Option<PendingConfirm>,
    /// The newest outside entry seen so far (`None` before the first state
    /// arrived: what was recorded before this window opened is not news).
    outside_seen: Option<u64>,
    /// A new outside entry arrived since the window last asked.
    outside_news: bool,
}

/// The id of the top undo entry when it was made outside GitPoulpi.
fn outside_top(state: &JournalState) -> Option<u64> {
    state
        .undo
        .as_ref()
        .filter(|e| e.kind == gitpoulpi_core::domain::OperationKind::Outside)
        .map(|e| e.id)
}

/// The shared cell between the link thread (which writes it) and the
/// render loop (which reads it every frame).
#[derive(Default)]
pub struct JournalCell {
    inner: Mutex<Inner>,
    /// Undo/redo requests running now: the buttons are disabled meanwhile.
    busy: AtomicUsize,
}

impl JournalCell {
    /// A cell whose state before the daemon's first report already says
    /// whether undo is off (`general.undo_enabled`, read by the TUI at
    /// startup), so the first frames never draw buttons that vanish.
    pub(crate) fn seeded(undo_enabled: bool) -> Self {
        let cell = Self::default();
        cell.lock().state.disabled = !undo_enabled;
        cell
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn set(&self, state: JournalState) {
        let mut held = self.lock();
        let changed = held.state != state;
        // One notice per new outside entry, so a burst is one line.
        let top = outside_top(&state);
        match (held.outside_seen, top) {
            (None, _) => held.outside_seen = Some(top.unwrap_or(0)),
            (Some(seen), Some(id)) if id > seen => {
                held.outside_seen = Some(id);
                held.outside_news = true;
            }
            _ => {}
        }
        held.state = state;
        drop(held);
        if changed {
            crate::redraw::poke();
        }
    }

    pub(crate) fn state(&self) -> JournalState {
        self.lock().state.clone()
    }

    /// True once per new entry that something outside GitPoulpi made.
    pub(crate) fn take_outside_news(&self) -> bool {
        std::mem::take(&mut self.lock().outside_news)
    }

    pub(crate) fn ask(&self, confirm: PendingConfirm) {
        self.lock().confirm = Some(confirm);
        crate::redraw::poke();
    }

    pub(crate) fn take_confirm(&self) -> Option<PendingConfirm> {
        self.lock().confirm.take()
    }

    pub(crate) fn begin(&self) {
        self.busy.fetch_add(1, Ordering::SeqCst);
        crate::redraw::poke();
    }

    pub(crate) fn end(&self) {
        // A compare-exchange loop rather than `fetch_update`, which newer
        // toolchains deprecate and older ones lack its `try_update` rename.
        let mut n = self.busy.load(Ordering::SeqCst);
        while n > 0 {
            match self
                .busy
                .compare_exchange(n, n - 1, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => break,
                Err(now) => n = now,
            }
        }
        crate::redraw::poke();
    }

    pub(crate) fn busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst) > 0
    }
}

pub(crate) fn side(
    state: &JournalState,
    d: Direction,
) -> (Option<&EntrySummary>, Option<&UndoRefusal>) {
    match d {
        Direction::Undo => (state.undo.as_ref(), state.blocked.as_ref()),
        Direction::Redo => (state.redo.as_ref(), state.redo_blocked.as_ref()),
    }
}

/// A refusal that leaves the entry stuck for good also names the way out.
pub(crate) fn with_forget_hint(text: String, refusal: &UndoRefusal) -> String {
    if refusal.forgettable() {
        format!("{text} {}", msg::undo_journal::FORGET_HINT.get())
    } else {
        text
    }
}

/// The way out a reflog position gives a blocked undo: "Restore main to
/// where Commit left it: Alt+Shift+Z" (the daemon filled the position in).
pub(crate) fn offer_text(state: &JournalState) -> Option<String> {
    let (name, _, op) = crate::safety_panel::offer_of(state)?;
    let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndoOffer);
    Some(if key.is_empty() {
        msg::undo_safety::offer_tip_nokey(name, op)
    } else {
        msg::undo_safety::offer_tip(name, op, key)
    })
}

pub(crate) fn with_offer(text: String, state: &JournalState) -> String {
    match offer_text(state) {
        Some(offer) => format!("{text} {offer}"),
        None => text,
    }
}

/// Why the command cannot run now, in full; `None` when it can.
pub(crate) fn reason(state: &JournalState, d: Direction, busy: bool) -> Option<String> {
    if state.disabled {
        return Some(msg::undo_ui::DISABLED.get().to_string());
    }
    if busy {
        return Some(
            match d {
                Direction::Undo => msg::undo_ui::ALREADY_UNDOING,
                Direction::Redo => msg::undo_ui::ALREADY_REDOING,
            }
            .get()
            .to_string(),
        );
    }
    match side(state, d) {
        (None, _) if d == Direction::Undo => Some(nothing_to_undo().to_string()),
        (None, _) => Some(msg::undo_ui::NOTHING_TO_REDO.get().to_string()),
        (Some(entry), Some(refusal)) => Some(match d {
            Direction::Undo => with_offer(
                with_forget_hint(
                    msg::undo_ui::cant_undo(
                        crate::safety_panel::summary_text(entry),
                        crate::daemon_link::error_text::undo_refusal(refusal),
                    ),
                    refusal,
                ),
                state,
            ),
            Direction::Redo => msg::undo_ui::cant_redo(
                crate::safety_panel::summary_text(entry),
                crate::daemon_link::error_text::undo_refusal(refusal),
            ),
        }),
        (Some(_), None) => None,
    }
}

/// Why "Undo whole sequence" cannot run now; `None` when it can: the plain
/// Undo must be possible and the newest entries must be one run's.
pub(crate) fn sequence_reason(state: &JournalState, busy: bool) -> Option<String> {
    reason(state, Direction::Undo, busy).or_else(|| {
        state
            .sequence
            .is_none()
            .then(|| msg::undo_ui::WHY_NO_SEQUENCE.get().to_string())
    })
}

/// The name of the sequence the newest entries belong to, quoted for the
/// palette row (`"Backup"`).
pub(crate) fn sequence_suffix(state: &JournalState) -> Option<String> {
    state.sequence.as_ref().map(|g| format!("\"{}\"", g.name))
}

/// The operation's name to append to the palette row ("Edit: Undo" +
/// `Commit "x" (abc1234)`), when there is one.
pub(crate) fn palette_suffix(state: &JournalState, d: Direction) -> Option<String> {
    side(state, d).0.map(|e| e.label.clone())
}

/// The id of the entry the user is looking at, for the `expect` guard.
pub(crate) fn top_id(state: &JournalState, d: Direction) -> Option<u64> {
    side(state, d).0.map(|e| e.id)
}
