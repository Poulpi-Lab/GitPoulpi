//! The session's messages: what the Activity tab lists.
//!
//! Process-wide, because the link's worker threads report failures while the
//! render loop reads them; a poke wakes the loop. Under `cfg(test)` the feed
//! is per thread, so unit tests never see each other's entries.

use gitpoulpi_core::{ErrorDetail, FailureKind};
use std::collections::VecDeque;

use crate::daemon_link::Action;

/// How many entries are kept; the oldest go first.
pub(crate) const CAPACITY: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Error,
    Warning,
    Info,
}

/// One message.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) id: u64,
    pub(crate) ts_ms: i64,
    pub(crate) severity: Severity,
    /// "Push failed".
    pub(crate) title: String,
    /// The friendly one-line reason, when the failure was recognized.
    pub(crate) summary: Option<String>,
    /// The suggested next step, when there is one.
    pub(crate) hint: Option<String>,
    /// The error as the daemon worded it (or the status message).
    pub(crate) message: String,
    pub(crate) detail: Option<ErrorDetail>,
    pub(crate) kind: FailureKind,
    /// The action that failed, to offer a retry.
    pub(crate) retry: Option<Action>,
    /// Looked at: the Activity Log was opened.
    pub(crate) read: bool,
    /// The repository the operation ran in; the Repository tab lists the
    /// entries of the active tab's repository (an entry with none is shown
    /// everywhere).
    pub(crate) repo: Option<std::path::PathBuf>,
    /// How long the operation took, in milliseconds.
    pub(crate) duration_ms: Option<u64>,
    /// What the Repository tab calls the entry when it differs from the
    /// card's `title` ("Push" for the card that says "Push done").
    pub(crate) list_title: Option<String>,
    /// A timing line of an operation that went through (as opposed to a
    /// message or a failure).
    pub(crate) operation: bool,
    /// The operation was not asked for (the periodic refresh, auto-fetch).
    pub(crate) background: bool,
    /// The workspace (its name) a workspace-wide task belonged to: the
    /// Global tab's workspace filter lists the entries of its repositories
    /// and these.
    pub(crate) workspace: Option<String>,
}

impl Entry {
    pub(crate) fn new(severity: Severity, title: String, message: String) -> Self {
        Entry {
            id: 0,
            ts_ms: gitpoulpi_log::now_ms(),
            severity,
            title,
            summary: None,
            hint: None,
            message,
            detail: None,
            kind: FailureKind::Other,
            retry: None,
            read: severity == Severity::Info,
            repo: None,
            list_title: None,
            duration_ms: None,
            operation: false,
            background: false,
            workspace: None,
        }
    }

    /// A timing line: `label` took `ms` in `repo`.
    pub(crate) fn operation(
        repo: &std::path::Path,
        label: String,
        ms: u64,
        background: bool,
    ) -> Self {
        let mut entry = Entry::new(Severity::Info, label, String::new());
        entry.repo = Some(repo.to_path_buf());
        entry.duration_ms = Some(ms);
        entry.operation = true;
        entry.background = background;
        entry
    }
}

#[derive(Default)]
struct Feed {
    next: u64,
    /// Oldest first.
    entries: VecDeque<Entry>,
}

#[cfg(not(test))]
static FEED: std::sync::Mutex<Feed> = std::sync::Mutex::new(Feed {
    next: 0,
    entries: VecDeque::new(),
});

#[cfg(test)]
thread_local! {
    static FEED: std::cell::RefCell<Feed> = std::cell::RefCell::new(Feed::default());
}

#[cfg(not(test))]
fn with<R>(f: impl FnOnce(&mut Feed) -> R) -> R {
    f(&mut FEED.lock().unwrap_or_else(|e| e.into_inner()))
}

#[cfg(test)]
fn with<R>(f: impl FnOnce(&mut Feed) -> R) -> R {
    FEED.with(|feed| f(&mut feed.borrow_mut()))
}

/// Adds `entry` and wakes the loop. Returns its id.
pub(crate) fn push(mut entry: Entry) -> u64 {
    let id = with(|feed| {
        feed.next += 1;
        entry.id = feed.next;
        if feed.entries.len() >= CAPACITY {
            feed.entries.pop_front();
        }
        feed.entries.push_back(entry);
        feed.next
    });
    crate::redraw::poke();
    id
}

/// A background timing line: it replaces the previous one of the same kind
/// in the same repository, so a refresh every few seconds never pushes the
/// operations the user ran out of the list.
pub(crate) fn push_background(entry: Entry) -> u64 {
    with(|feed| {
        feed.entries.retain(|e| {
            !(e.background && e.operation && e.title == entry.title && e.repo == entry.repo)
        })
    });
    push(entry)
}

/// Every entry, newest first.
pub(crate) fn snapshot() -> Vec<Entry> {
    with(|feed| feed.entries.iter().rev().cloned().collect())
}

pub(crate) fn get(id: u64) -> Option<Entry> {
    with(|feed| feed.entries.iter().find(|e| e.id == id).cloned())
}

/// Errors not looked at yet: the status bar button's badge.
pub(crate) fn unread_errors() -> usize {
    with(|feed| {
        feed.entries
            .iter()
            .filter(|e| e.severity == Severity::Error && !e.read)
            .count()
    })
}

pub(crate) fn mark_all_read() {
    with(|feed| feed.entries.iter_mut().for_each(|e| e.read = true));
}

/// Whether entry `id` is still unread (the Activity button counts it).
pub(crate) fn is_unread(id: u64) -> bool {
    with(|feed| feed.entries.iter().any(|e| e.id == id && !e.read))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(title: &str) -> Entry {
        Entry::new(Severity::Error, title.into(), "boom".into())
    }

    #[test]
    fn errors_count_as_unread_until_looked_at() {
        let first = push(error("Push failed"));
        push(error("Pull failed"));
        push(Entry::new(Severity::Info, "done".into(), "ok".into()));
        assert_eq!(unread_errors(), 2);
        assert!(is_unread(first));
        mark_all_read();
        assert_eq!(unread_errors(), 0);
        assert!(!is_unread(first));
        assert_eq!(snapshot().len(), 3);
        assert_eq!(snapshot()[0].title, "done", "newest first");
    }

    #[test]
    fn the_feed_keeps_only_the_newest_entries() {
        for i in 0..CAPACITY + 5 {
            push(error(&format!("e{i}")));
        }
        let all = snapshot();
        assert_eq!(all.len(), CAPACITY);
        assert_eq!(all[0].title, format!("e{}", CAPACITY + 4));
    }
}
