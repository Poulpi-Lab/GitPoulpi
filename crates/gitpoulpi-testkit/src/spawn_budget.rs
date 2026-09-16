//! Spawn-count assertions for the perf regression guard
//! (the perf design).
//!
//! A `git` process costs 10-40 ms on Windows, so "how many spawns does this
//! operation make" is the deterministic stand-in for "how slow is it". The
//! counter lives in `gitpoulpi-cli-backend` (`total_spawn_count`); the
//! testkit does not depend on it, so the caller hands it in as a function.

use std::sync::{Mutex, MutexGuard};

/// Process-wide counters see every thread, so tests that diff one must not
/// overlap. Hold the guard from [`SpawnBudget::serial`] for the whole test.
static SERIAL: Mutex<()> = Mutex::new(());

pub struct SpawnBudget {
    counter: fn() -> u64,
    _serial: MutexGuard<'static, ()>,
}

impl SpawnBudget {
    /// Takes the per-binary serial lock (a poisoned lock is fine: a failed
    /// sibling test must not hide this one).
    pub fn serial(counter: fn() -> u64) -> Self {
        let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        Self {
            counter,
            _serial: guard,
        }
    }

    /// Runs `f` and returns its result with the number of spawns it made.
    pub fn measure<T>(&self, f: impl FnOnce() -> T) -> (T, u64) {
        let before = (self.counter)();
        let out = f();
        (out, (self.counter)() - before)
    }

    /// Runs `f` and panics naming `what` if it spawned more than `max`.
    pub fn assert_at_most<T>(&self, what: &str, max: u64, f: impl FnOnce() -> T) -> T {
        let (out, n) = self.measure(f);
        assert!(
            n <= max,
            "{what}: {n} git spawns, budget {max} (perf regression; see the perf design)"
        );
        out
    }
}
