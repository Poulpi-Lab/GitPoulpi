//! Keeps a failing poll from flooding the application log.
//!
//! A client that polls (the TUI every 2 s) and keeps failing the same way
//! used to add one WARN line per request, thousands an hour. The same
//! operation failing with the same message for the same repository is logged
//! once, then again only after [`QUIET`], carrying how many identical
//! failures were held back in between. A different message logs at once.
//! The response to the client is never touched: only the log line is.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long an identical failure stays quiet after its line was logged.
const QUIET: Duration = Duration::from_secs(300);

struct Entry {
    message: String,
    logged_at: Instant,
    held_back: u64,
}

#[derive(Default)]
pub(crate) struct FailureLog {
    seen: Mutex<HashMap<(String, Option<String>), Entry>>,
}

impl FailureLog {
    /// `Some(held_back)` when the failure is to be logged now (`held_back`
    /// identical ones were skipped since the previous line), `None` when it
    /// repeats one logged less than [`QUIET`] ago.
    pub(crate) fn observe(
        &self,
        operation: &str,
        repo: Option<&str>,
        message: &str,
        now: Instant,
    ) -> Option<u64> {
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        let key = (operation.to_string(), repo.map(str::to_string));
        if let Some(entry) = seen.get_mut(&key) {
            if entry.message == message && now.duration_since(entry.logged_at) < QUIET {
                entry.held_back += 1;
                return None;
            }
            let held_back = if entry.message == message {
                entry.held_back
            } else {
                0
            };
            *entry = Entry {
                message: message.to_string(),
                logged_at: now,
                held_back: 0,
            };
            return Some(held_back);
        }
        seen.insert(
            key,
            Entry {
                message: message.to_string(),
                logged_at: now,
                held_back: 0,
            },
        );
        Some(0)
    }
}

/// The process-wide log of failures (one daemon per process).
pub(crate) fn global() -> &'static FailureLog {
    static LOG: std::sync::OnceLock<FailureLog> = std::sync::OnceLock::new();
    LOG.get_or_init(FailureLog::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identical_failure_is_logged_once_then_summarised_after_the_quiet_period() {
        let log = FailureLog::default();
        let t0 = Instant::now();
        assert_eq!(log.observe("Remotes", Some("/r"), "boom", t0), Some(0));
        for i in 1..=5 {
            let at = t0 + Duration::from_secs(2 * i);
            assert_eq!(log.observe("Remotes", Some("/r"), "boom", at), None);
        }
        assert_eq!(
            log.observe(
                "Remotes",
                Some("/r"),
                "boom",
                t0 + QUIET + Duration::from_secs(1)
            ),
            Some(5),
            "after the quiet period the line comes back with the held-back count"
        );
    }

    #[test]
    fn another_message_operation_or_repository_logs_at_once() {
        let log = FailureLog::default();
        let t0 = Instant::now();
        assert_eq!(log.observe("Remotes", Some("/r"), "boom", t0), Some(0));
        assert_eq!(log.observe("Remotes", Some("/r"), "bang", t0), Some(0));
        assert_eq!(log.observe("Refs", Some("/r"), "bang", t0), Some(0));
        assert_eq!(log.observe("Refs", Some("/other"), "bang", t0), Some(0));
        assert_eq!(log.observe("Refs", Some("/other"), "bang", t0), None);
    }
}
