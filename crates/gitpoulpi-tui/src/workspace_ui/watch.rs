//! Notices `workspaces.toml` being edited from outside (a shell, or
//! `gitpoulpi workspace ...` while the TUI runs).
//!
//! A cheap fingerprint (modification time, length) of the file, looked at on
//! the loop's tick at most once per [`EVERY`]: one `stat`, never a read, so it
//! cannot hold up a frame. A moved fingerprint is a change like any made in
//! this process (`apply::changed` + the chip's refresh), so the Workspaces
//! page, the chip and the switcher all follow. The loop idles in `poll`, so
//! the next look is also a registered deadline ([`next_deadline`]).
//!
//! The first look only records the fingerprint: what was on disk at start is
//! what every part already loaded.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

/// How often the file is looked at.
const EVERY: Duration = Duration::from_millis(750);

type Fingerprint = Option<(SystemTime, u64)>;

#[derive(Default)]
struct Watch {
    seen: Option<Fingerprint>,
    checked: Option<Instant>,
}

impl Watch {
    /// One tick: whether the file changed since the last look. `stat` is only
    /// called when a look is due.
    fn step(&mut self, now: Instant, stat: impl FnOnce() -> Fingerprint) -> bool {
        if self
            .checked
            .is_some_and(|at| now.duration_since(at) < EVERY)
        {
            return false;
        }
        self.checked = Some(now);
        let fingerprint = stat();
        let changed = self.seen.is_some_and(|before| before != fingerprint);
        self.seen = Some(fingerprint);
        changed
    }

    fn deadline(&self) -> Option<Instant> {
        self.checked.map(|at| at + EVERY)
    }
}

static STATE: Mutex<Option<Watch>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Watch) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Watch::default))
}

fn stat_of(path: Option<PathBuf>) -> Fingerprint {
    let meta = std::fs::metadata(path?).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// When the loop must look in again.
pub(crate) fn next_deadline() -> Option<Instant> {
    if !crate::build_features::WORKSPACES {
        return None;
    }
    with(|w| w.deadline())
}

/// One loop iteration; carries a change to the page, the chip and the
/// switcher.
pub(crate) fn poll(session: &str) {
    if !crate::build_features::WORKSPACES {
        return;
    }
    let changed = with(|w| {
        w.step(Instant::now(), || {
            stat_of(
                crate::session_store::config_dir()
                    .map(|d| gitpoulpi_session::Workspaces::file_in(&d)),
            )
        })
    });
    if changed {
        super::apply::changed();
        super::chip::refresh(session);
        crate::redraw::poke();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(secs: u64, len: u64) -> Fingerprint {
        Some((SystemTime::UNIX_EPOCH + Duration::from_secs(secs), len))
    }

    #[test]
    fn the_first_look_only_records_and_a_moved_fingerprint_is_a_change() {
        let mut w = Watch::default();
        let t0 = Instant::now();
        assert!(!w.step(t0, || fp(1, 10)));
        // Not due yet: not even asked.
        assert!(!w.step(t0 + Duration::from_millis(100), || panic!("stat too early")));
        assert!(!w.step(t0 + EVERY, || fp(1, 10)), "same file");
        assert!(w.step(t0 + EVERY * 2, || fp(2, 10)), "new mtime");
        assert!(w.step(t0 + EVERY * 3, || fp(2, 99)), "new length");
        assert!(w.step(t0 + EVERY * 4, || None), "deleted");
        assert!(w.step(t0 + EVERY * 5, || fp(3, 1)), "created again");
        assert_eq!(w.deadline(), Some(t0 + EVERY * 5 + EVERY));
    }

    #[test]
    fn a_file_that_appears_after_a_missing_start_is_a_change() {
        let mut w = Watch::default();
        let t0 = Instant::now();
        assert!(!w.step(t0, || None));
        assert!(w.step(t0 + EVERY, || fp(5, 40)));
    }
}
