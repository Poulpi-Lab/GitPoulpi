//! What a window hands to the next one.
//!
//! Opening a repository from the Radar ends the Home window and starts the
//! repository's. Some actions (check out the PR's branch, create its worktree,
//! reveal it in the sidebar) need that repository's own window to exist, so
//! they travel here, in the process, and the new window runs them once its
//! refs and its hosting cache are in. The same trick as `ui_intent::carry`.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::vocab::Act;

/// A carried action older than this is dropped (the window it was meant for
/// never came).
const FRESH_FOR: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pending {
    /// Check out a pull request's source branch (a fork's head is fetched).
    CheckoutPr { number: u64 },
    /// Create a worktree on a branch (the existing Create worktree dialog).
    Worktree { branch: String },
    /// Select the item in the sidebar's PULL REQUESTS / ISSUES list.
    Show { pr: bool, number: u64 },
    /// Put the keyboard on the PULL REQUESTS section, where a provider is
    /// picked: the existing "link this repository" flow.
    Link,
    /// The New branch prompt with a name made from an issue.
    StartWork { branch: String },
    /// The metadata picker (`act` says which field) for a request or issue.
    EditMeta { pr: bool, number: u64, act: Act },
    /// The in-app code review of a request. `repo` names the repository
    /// whose window runs it when the action leaves a window that is not that
    /// repository's (the workspace search); the Radar ends its own
    /// window first, so it leaves `None`.
    Review {
        number: u64,
        repo: Option<std::path::PathBuf>,
    },
}

struct Slot {
    action: Pending,
    at: Instant,
}

static SLOT: Mutex<Option<Slot>> = Mutex::new(None);

fn lock() -> std::sync::MutexGuard<'static, Option<Slot>> {
    SLOT.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn set(action: Pending) {
    *lock() = Some(Slot {
        action,
        at: Instant::now(),
    });
}

/// Whether something waits (a window not ready yet leaves it there).
pub(crate) fn waiting() -> bool {
    lock().as_ref().is_some_and(|s| s.at.elapsed() < FRESH_FOR)
}

/// The carried action, if it is still fresh, without taking it.
pub(crate) fn peek() -> Option<Pending> {
    lock()
        .as_ref()
        .filter(|s| s.at.elapsed() < FRESH_FOR)
        .map(|s| s.action.clone())
}

/// Takes the carried action, if it is still fresh.
pub(crate) fn take() -> Option<Pending> {
    lock()
        .take()
        .filter(|s| s.at.elapsed() < FRESH_FOR)
        .map(|s| s.action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_is_taken_once() {
        set(Pending::Link);
        assert!(waiting());
        assert_eq!(take(), Some(Pending::Link));
        assert_eq!(take(), None);
        assert!(!waiting());
    }
}
