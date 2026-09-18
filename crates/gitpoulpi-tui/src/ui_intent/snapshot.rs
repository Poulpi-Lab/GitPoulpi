//! The view an agent's intent replaced, so Back can bring it back (plan
//! section 4.5). A snapshot holds what an intent can move and the window
//! can put back by itself: the repository, Home, the focused pane, the
//! pane collapse and the Log's selected commit. Overlays an intent opens
//! (Settings, Help, the palette) are simply closed by Back.

use std::path::PathBuf;
use std::sync::Mutex;

use gitpoulpi_core::domain::Oid;

/// The deepest Back history.
const DEPTH: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ViewSnapshot {
    pub(crate) repo_root: PathBuf,
    /// The window was the Repository Management tab.
    pub(crate) home: bool,
    /// `refs`, `log` or `status`.
    pub(crate) focus: &'static str,
    pub(crate) left_collapsed: bool,
    pub(crate) right_collapsed: bool,
    pub(crate) selected_commit: Option<Oid>,
    /// The central column showed the Log (no diff or file open): Back
    /// closes what an intent opened over it.
    pub(crate) central_log: bool,
}

static STACK: Mutex<Vec<ViewSnapshot>> = Mutex::new(Vec::new());
static RESTORE: Mutex<Option<ViewSnapshot>> = Mutex::new(None);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Remembers the view about to be replaced; the oldest of ten is forgotten.
pub(crate) fn push(snapshot: ViewSnapshot) {
    let mut stack = lock(&STACK);
    stack.push(snapshot);
    if stack.len() > DEPTH {
        stack.remove(0);
    }
}

pub(crate) fn pop() -> Option<ViewSnapshot> {
    lock(&STACK).pop()
}

pub(crate) fn depth() -> usize {
    lock(&STACK).len()
}

/// A Back across tabs: the window it switches to restores this on arrival.
pub(crate) fn hand_over_restore(snapshot: ViewSnapshot) {
    *lock(&RESTORE) = Some(snapshot);
}

pub(crate) fn take_restore() -> Option<ViewSnapshot> {
    lock(&RESTORE).take()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(n: u8) -> ViewSnapshot {
        ViewSnapshot {
            repo_root: PathBuf::from(format!("/r{n}")),
            home: false,
            focus: "log",
            left_collapsed: false,
            right_collapsed: false,
            selected_commit: None,
            central_log: true,
        }
    }

    #[test]
    fn the_stack_keeps_ten_and_pops_the_newest_first() {
        while pop().is_some() {}
        for n in 0..12 {
            push(snap(n));
        }
        assert_eq!(depth(), 10);
        assert_eq!(pop().unwrap().repo_root, PathBuf::from("/r11"));
        while pop().is_some() {}
        assert_eq!(depth(), 0);
    }
}
