//! Undo and redo in the TUI (see the undo-redo design).
//!
//! The daemon owns the journal; this module holds what the window needs
//! to draw and drive it: the latest [`JournalState`] (set from
//! `Event::JournalChanged`, and re-read with every refresh so a change made
//! outside GitPoulpi shows within a poll), the wording of the toolbar
//! tooltip, the palette's reasons, and the confirmation the daemon asks for
//! when an undo is risky. All of it is pure over a `JournalState`, so it is
//! unit-tested without a terminal or a daemon.

use gitpoulpi_core::domain::{Direction, JournalState};
use gitpoulpi_i18n::msg;

pub(crate) use gitpoulpi_link::undo_ui::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{EntryState, OperationKind};
    use gitpoulpi_core::domain::{EntrySummary, UndoRefusal};

    fn entry(label: &str) -> EntrySummary {
        EntrySummary {
            id: 7,
            kind: OperationKind::Commit,
            label: label.into(),
            at: 0,
            state: EntryState::Done,
            origin: None,
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
            sequence: None,
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
                origin: None,
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
