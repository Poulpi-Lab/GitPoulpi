//! Undo and redo in the TUI (see the undo-redo design).
//!
//! The daemon owns the journal; this module holds what the window needs
//! to draw and drive it: the latest [`JournalState`] (set from
//! `Event::JournalChanged`, and re-read with every refresh so a change made
//! outside GitPoulpi shows within a poll), the wording of the toolbar
//! tooltip, the palette's reasons, and the confirmation the daemon asks for
//! when an undo is risky. All of it is pure over a `JournalState`, so it is
//! unit-tested without a terminal or a daemon.

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
pub(crate) fn note_outward_comment() {
    OUTWARD_COMMENT.with(|c| c.set(true));
}

fn outward_comment() -> bool {
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

/// What the toolbar and the palette need per direction, as plain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Avail {
    Ready,
    /// Nothing recorded to undo/redo (or one is running).
    #[default]
    Nothing,
    /// Something is recorded but the repository moved under it.
    Blocked,
    /// Blocked for good: the entry can be forgotten (dropped from the
    /// journal) to reach the ones under it.
    Stuck,
    /// `general.undo_enabled` is off: the buttons and palette rows are
    /// not shown at all.
    Off,
}

fn side(state: &JournalState, d: Direction) -> (Option<&EntrySummary>, Option<&UndoRefusal>) {
    match d {
        Direction::Undo => (state.undo.as_ref(), state.blocked.as_ref()),
        Direction::Redo => (state.redo.as_ref(), state.redo_blocked.as_ref()),
    }
}

pub(crate) fn avail(state: &JournalState, d: Direction, busy: bool) -> Avail {
    if state.disabled {
        return Avail::Off;
    }
    match side(state, d) {
        _ if busy => Avail::Nothing,
        (None, _) => Avail::Nothing,
        (Some(_), Some(r)) if d == Direction::Undo && r.forgettable() => Avail::Stuck,
        (Some(_), Some(_)) => Avail::Blocked,
        (Some(_), None) => Avail::Ready,
    }
}

/// A refusal that leaves the entry stuck for good also names the way out.
fn with_forget_hint(text: String, refusal: &UndoRefusal) -> String {
    if refusal.forgettable() {
        format!("{text} {}", msg::undo_journal::FORGET_HINT.get())
    } else {
        text
    }
}

/// The way out a reflog position gives a blocked undo: "Restore main to
/// where Commit left it: Alt+Shift+Z" (the daemon filled the position in).
fn offer_text(state: &JournalState) -> Option<String> {
    let (name, _, op) = crate::safety_panel::offer_of(state)?;
    let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndoOffer);
    Some(if key.is_empty() {
        msg::undo_safety::offer_tip_nokey(name, op)
    } else {
        msg::undo_safety::offer_tip(name, op, key)
    })
}

fn with_offer(text: String, state: &JournalState) -> String {
    match offer_text(state) {
        Some(offer) => format!("{text} {offer}"),
        None => text,
    }
}

/// What *Forget* asks the user to confirm for the latest undo entry.
pub(crate) fn forget_prompt(state: &JournalState) -> Option<String> {
    let entry = state.undo.as_ref()?;
    Some(msg::undo_journal::forget_confirm(
        crate::safety_panel::summary_text(entry),
    ))
}

fn action(d: Direction) -> crate::keymap::Action {
    match d {
        Direction::Undo => crate::keymap::Action::GlobalUndo,
        Direction::Redo => crate::keymap::Action::GlobalRedo,
    }
}

/// The toolbar tooltip: the operation that would be undone or redone with
/// its key, or why nothing can be.
pub(crate) fn tooltip(state: &JournalState, d: Direction, busy: bool) -> String {
    if state.disabled {
        return msg::undo_ui::DISABLED.get().to_string();
    }
    if busy {
        return match d {
            Direction::Undo => msg::undo_ui::UNDOING,
            Direction::Redo => msg::undo_ui::REDOING,
        }
        .get()
        .to_string();
    }
    match side(state, d) {
        (None, _) if d == Direction::Undo && outward_comment() => {
            msg::undo_ui::TIP_NOTHING_TO_UNDO_COMMENT.get().to_string()
        }
        (None, _) if d == Direction::Undo => msg::undo_ui::TIP_NOTHING_TO_UNDO.get().to_string(),
        (None, _) => msg::undo_ui::TIP_NOTHING_TO_REDO.get().to_string(),
        (Some(entry), Some(refusal)) => match d {
            Direction::Undo => with_offer(
                with_forget_hint(
                    msg::undo_ui::tip_cant_undo(
                        crate::safety_panel::summary_text(entry),
                        crate::daemon_link::error_text::undo_refusal(refusal),
                    ),
                    refusal,
                ),
                state,
            ),
            Direction::Redo => msg::undo_ui::tip_cant_redo(
                crate::safety_panel::summary_text(entry),
                crate::daemon_link::error_text::undo_refusal(refusal),
            ),
        },
        (Some(entry), None) => {
            let key = crate::keymap::current().shown(action(d));
            let words = crate::safety_panel::summary_text(entry);
            let label = match (d, key.is_empty()) {
                (Direction::Undo, true) => msg::undo_ui::undo_label(words),
                (Direction::Redo, true) => msg::undo_ui::redo_label(words),
                (Direction::Undo, false) => msg::undo_ui::undo_label_key(words, key),
                (Direction::Redo, false) => msg::undo_ui::redo_label_key(words, key),
            };
            // How far back one can go: the guarantee, said where it is looked for.
            if d == Direction::Undo && state.undo_count > 1 {
                format!(
                    "{label} - {}",
                    msg::undo_safety::tip_depth(state.undo_count as u64)
                )
            } else {
                label
            }
        }
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

/// The operation's name to append to the palette row ("Edit: Undo" +
/// `Commit "x" (abc1234)`), when there is one.
pub(crate) fn palette_suffix(state: &JournalState, d: Direction) -> Option<String> {
    side(state, d).0.map(|e| e.label.clone())
}

/// The id of the entry the user is looking at, for the `expect` guard.
pub(crate) fn top_id(state: &JournalState, d: Direction) -> Option<u64> {
    side(state, d).0.map(|e| e.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{EntryState, OperationKind};

    fn entry(label: &str) -> EntrySummary {
        EntrySummary {
            id: 7,
            kind: OperationKind::Commit,
            label: label.into(),
            at: 0,
            state: EntryState::Done,
        }
    }

    fn state() -> JournalState {
        JournalState {
            undo: Some(entry("Commit \"fix\" (a81b9e4)")),
            ..Default::default()
        }
    }

    #[test]
    fn a_ready_undo_names_the_operation_and_its_key() {
        let s = state();
        assert_eq!(avail(&s, Direction::Undo, false), Avail::Ready);
        let tip = tooltip(&s, Direction::Undo, false);
        assert!(tip.starts_with("Undo Commit \"fix\" (a81b9e4)"), "{tip}");
        assert!(tip.ends_with("Ctrl+Z"), "{tip}");
        assert_eq!(reason(&s, Direction::Undo, false), None);
        assert_eq!(top_id(&s, Direction::Undo), Some(7));
    }

    #[test]
    fn nothing_recorded_is_nothing_to_undo_or_redo() {
        let s = JournalState::default();
        assert_eq!(avail(&s, Direction::Undo, false), Avail::Nothing);
        let tip = tooltip(&s, Direction::Undo, false);
        assert!(tip.starts_with("Nothing to undo"), "{tip}");
        assert!(tip.contains("remote server"), "{tip}");
        assert_eq!(tooltip(&s, Direction::Redo, false), "Nothing to redo");
        assert_eq!(
            reason(&s, Direction::Redo, false).as_deref(),
            Some("nothing to redo")
        );
    }

    #[test]
    fn a_comment_posted_to_a_host_is_not_undoable_and_undo_says_so() {
        let s = JournalState::default();
        note_outward_comment();
        let tip = tooltip(&s, Direction::Undo, false);
        assert!(
            tip.contains("comment posted to a host can't be undone here"),
            "{tip}"
        );
        assert!(tip.contains("edit or delete it"), "{tip}");
        let why = reason(&s, Direction::Undo, false).unwrap();
        assert!(why.contains("comment posted to a host"), "{why}");
        assert_eq!(nothing_to_undo(), why);
        // An undoable entry still shows as the thing Undo would do.
        let ready = tooltip(&state(), Direction::Undo, false);
        assert!(ready.starts_with("Undo Commit"), "{ready}");
    }

    #[test]
    fn a_blocked_undo_says_why_in_the_tooltip_and_the_reason() {
        let mut s = state();
        s.blocked = Some(UndoRefusal::HeadMoved {
            expected: gitpoulpi_core::domain::HeadState::Branch("main".into()),
            actual: gitpoulpi_core::domain::HeadState::Branch("other".into()),
        });
        // The repository moved on for good: the way out is named.
        assert_eq!(avail(&s, Direction::Undo, false), Avail::Stuck);
        let tip = tooltip(&s, Direction::Undo, false);
        assert!(tip.starts_with("Can't undo Commit"), "{tip}");
        assert!(tip.contains("outside GitPoulpi"), "{tip}");
        assert!(tip.contains("Forget"), "{tip}");
        // Blocked only for now (an operation is running): no Forget.
        let mut running = state();
        running.blocked = Some(UndoRefusal::InProgress { op: "merge".into() });
        assert_eq!(avail(&running, Direction::Undo, false), Avail::Blocked);
        assert!(!tooltip(&running, Direction::Undo, false).contains("Forget"));
        assert!(reason(&s, Direction::Undo, false)
            .unwrap()
            .contains("outside"));
    }

    #[test]
    fn a_running_undo_disables_the_button() {
        let s = state();
        assert_eq!(avail(&s, Direction::Undo, true), Avail::Nothing);
        assert_eq!(tooltip(&s, Direction::Undo, true), "undoing...");
        assert_eq!(tooltip(&s, Direction::Redo, true), "redoing...");
    }

    #[test]
    fn turned_off_hides_the_controls_and_says_why() {
        let s = JournalState {
            disabled: true,
            ..state()
        };
        assert_eq!(avail(&s, Direction::Undo, false), Avail::Off);
        assert_eq!(avail(&s, Direction::Redo, false), Avail::Off);
        assert_eq!(
            reason(&s, Direction::Undo, false).as_deref(),
            Some("Undo is turned off (Settings > General)")
        );
        assert_eq!(
            tooltip(&s, Direction::Redo, false),
            "Undo is turned off (Settings > General)"
        );
    }

    #[test]
    fn the_cell_hands_a_confirmation_over_once() {
        let cell = JournalCell::default();
        cell.ask(PendingConfirm {
            direction: Direction::Undo,
            expect: Some(3),
            prompt: "sure?".into(),
        });
        assert_eq!(cell.take_confirm().unwrap().expect, Some(3));
        assert!(cell.take_confirm().is_none());
        cell.begin();
        assert!(cell.busy());
        cell.end();
        cell.end();
        assert!(!cell.busy());
    }

    fn top(id: u64, kind: OperationKind) -> JournalState {
        JournalState {
            undo: Some(EntrySummary {
                id,
                kind,
                label: "x".into(),
                at: 0,
                state: EntryState::Done,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn a_new_outside_entry_is_news_once_and_what_was_there_at_start_is_not() {
        let cell = JournalCell::default();
        // An outside entry from an earlier run, there when the window opens.
        cell.set(top(3, OperationKind::Outside));
        assert!(!cell.take_outside_news(), "old entries are not news");
        // One of our own: not news.
        cell.set(top(4, OperationKind::Commit));
        assert!(!cell.take_outside_news());
        // A new outside one is, once, however often the state is re-read.
        cell.set(top(5, OperationKind::Outside));
        cell.set(top(5, OperationKind::Outside));
        assert!(cell.take_outside_news());
        assert!(!cell.take_outside_news());
        // The next one, again.
        cell.set(top(6, OperationKind::Outside));
        assert!(cell.take_outside_news());
    }
}
