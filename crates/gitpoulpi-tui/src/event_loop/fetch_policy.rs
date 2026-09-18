//! When the window fetches on its own: the "Fetch when a tab opens" rule
//! and the auto-fetch timer's next deadline.
//!
//! A tab switch rebuilds the window state, so the "has this repository
//! been fetched lately" memory lives in a process-wide map keyed by the
//! repository root, not in `WindowState`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A repository fetched within this long is not fetched again on tab open.
pub(super) const FETCH_ON_OPEN_DEBOUNCE: Duration = Duration::from_secs(60);

static LAST_FETCH: Mutex<Option<HashMap<PathBuf, Instant>>> = Mutex::new(None);

/// The rule: fetch unless the last fetch was less than `debounce` ago.
pub(super) fn should_fetch_on_open(
    last: Option<Instant>,
    now: Instant,
    debounce: Duration,
) -> bool {
    match last {
        Some(at) => now.saturating_duration_since(at) >= debounce,
        None => true,
    }
}

/// Whether the auto-fetch timer fires now. Never in a window with no
/// repository: there is no folder to run git in.
pub(super) fn auto_fetch_due(no_repository: bool, last: Instant, interval: Duration) -> bool {
    !no_repository && last.elapsed() >= interval
}

/// Remembers that this window started a fetch of `repo` just now.
pub(super) fn note_fetch(repo: &Path) {
    let mut guard = LAST_FETCH.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(HashMap::new)
        .insert(repo.to_path_buf(), Instant::now());
}

/// Whether tab open should fetch `repo` now; when it should, records it.
pub(super) fn claim_fetch_on_open(repo: &Path) -> bool {
    let mut guard = LAST_FETCH.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    if should_fetch_on_open(map.get(repo).copied(), now, FETCH_ON_OPEN_DEBOUNCE) {
        map.insert(repo.to_path_buf(), now);
        true
    } else {
        false
    }
}

impl super::init::WindowState {
    /// "Fetch when a tab opens": runs once, on the window's first tick,
    /// through the same `Action::FetchAll` seam as the toolbar, so the
    /// in-flight feedback is the usual one and input never blocks.
    pub(super) fn fetch_on_open_once(&mut self) {
        if !std::mem::take(&mut self.fetch_on_open_pending) || self.no_repository {
            return;
        }
        if claim_fetch_on_open(&self.repo_root) {
            self.link.submit_auto_fetch();
        }
    }

    /// When the auto-fetch timer next fires, `None` when it is off. For
    /// the loop's poll timeout (a pending fetch-on-open is due at once).
    pub(super) fn next_fetch_deadline(&self) -> Option<Instant> {
        if self.fetch_on_open_pending {
            return Some(Instant::now());
        }
        if self.no_repository {
            return None;
        }
        self.auto_fetch_interval
            .map(|interval| self.last_auto_fetch + interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetches_when_never_fetched() {
        assert!(should_fetch_on_open(
            None,
            Instant::now(),
            FETCH_ON_OPEN_DEBOUNCE
        ));
    }

    #[test]
    fn skips_a_fetch_within_the_debounce_and_allows_it_after() {
        let t0 = Instant::now();
        let d = FETCH_ON_OPEN_DEBOUNCE;
        assert!(!should_fetch_on_open(Some(t0), t0, d));
        assert!(!should_fetch_on_open(
            Some(t0),
            t0 + Duration::from_secs(59),
            d
        ));
        assert!(should_fetch_on_open(
            Some(t0),
            t0 + Duration::from_secs(60),
            d
        ));
    }

    #[test]
    fn auto_fetch_never_fires_without_a_repository() {
        let long_ago = Instant::now() - Duration::from_secs(120);
        let minute = Duration::from_secs(60);
        assert!(auto_fetch_due(false, long_ago, minute));
        assert!(!auto_fetch_due(true, long_ago, minute));
        assert!(!auto_fetch_due(false, Instant::now(), minute));
    }

    #[test]
    fn a_second_claim_for_the_same_repo_is_debounced() {
        let repo = Path::new("/tmp/fetch-policy-test-repo");
        assert!(claim_fetch_on_open(repo));
        assert!(!claim_fetch_on_open(repo));
        assert!(claim_fetch_on_open(Path::new("/tmp/fetch-policy-other")));
    }
}
