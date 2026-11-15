//! Unread markers: what the user last saw, and the next entry after it.
//!
//! The mark is the time of the newest entry shown at the last visit
//! (`RepoState::conversation_seen`); an entry strictly after it, written by
//! someone else, is unread. A conversation visited for the first time has no
//! mark, so nothing in it is "new".

use gitpoulpi_hosting::conversation::TimelineEntry;

use super::rows::{entry_key, Item};

/// The next unread item after `from`, wrapping around once.
pub(super) fn next(items: &[Item], from: usize) -> Option<usize> {
    let n = items.len();
    (1..=n)
        .map(|d| (from + d) % n.max(1))
        .find(|&i| items.get(i).is_some_and(|it| it.unread) && i != from)
}

/// `(time, id)` of the newest entry: what a visit leaves as "seen".
pub(super) fn newest(entries: &[TimelineEntry]) -> Option<(String, String)> {
    entries
        .iter()
        .max_by(|a, b| a.at().cmp(b.at()))
        .map(|e| (e.at().to_string(), entry_key(e)))
}
