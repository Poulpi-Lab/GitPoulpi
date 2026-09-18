//! How long operations take: the Repository tab's "Push - 123 ms" lines.
//!
//! **Where it is measured:** by the window, around each request to the
//! daemon (`daemon_link::run_action`, the first load and the refreshes in
//! `daemon_link`), so the figure is the daemon's work plus the IPC round
//! trip, and an operation started by the CLI or MCP is not timed here.
//! Measuring in the daemon would also time those, but the daemon would then
//! have to push the lines to every window; the round trip is a few
//! milliseconds next to a git call.
//!
//! `log.timings` (default on) shows the durations (and lists the operations
//! that went through); `log.extended` (default off) also lists the
//! background ones. Every operation is written to the application log
//! whatever the two say: `info` for what the user asked for, `debug` for the
//! background.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use gitpoulpi_i18n::msg::activity_log as msg;

use super::feed::{self, Entry};
use crate::daemon_link::Action;

static TIMINGS: AtomicBool = AtomicBool::new(true);
static EXTENDED: AtomicBool = AtomicBool::new(false);
/// Fetches the window started on its own (auto-fetch, fetch on tab open) and
/// that have not finished yet: they travel as plain `Action::FetchAll`.
static BACKGROUND_FETCHES: AtomicU32 = AtomicU32::new(0);

/// Publishes `log.timings` and `log.extended`. Called at start-up and when
/// Preferences changes one of them.
pub(crate) fn install(log: &gitpoulpi_config::LogConfig) {
    TIMINGS.store(log.timings, Ordering::Relaxed);
    EXTENDED.store(log.extended, Ordering::Relaxed);
}

/// Reads the settings file and publishes the two switches (start-up).
pub(crate) fn install_from_settings() {
    if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
        install(&settings.config().log);
    }
}

pub(crate) fn timings_on() -> bool {
    TIMINGS.load(Ordering::Relaxed)
}

pub(crate) fn extended_on() -> bool {
    EXTENDED.load(Ordering::Relaxed)
}

/// The window is about to send an automatic `FetchAll`.
pub(crate) fn expect_background_fetch() {
    BACKGROUND_FETCHES.fetch_add(1, Ordering::SeqCst);
}

/// Whether a fetch the window started on its own is still going.
pub(crate) fn background_fetch_pending() -> bool {
    BACKGROUND_FETCHES.load(Ordering::SeqCst) > 0
}

/// A `FetchAll` finished: whether it was one the window started on its own.
fn take_background_fetch(action: &Action) -> bool {
    if !matches!(action, Action::FetchAll) {
        return false;
    }
    // Not `fetch_update`: deprecated on newer toolchains, see `undo_ui`.
    let mut n = BACKGROUND_FETCHES.load(Ordering::SeqCst);
    while n > 0 {
        match BACKGROUND_FETCHES.compare_exchange(n, n - 1, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return true,
            Err(now) => n = now,
        }
    }
    false
}

/// What an action is called in the list; `None` for the many small
/// mutations (stage a file, create a tag) that are not worth a line.
pub(crate) fn label_for(action: &Action) -> Option<String> {
    let action = match action {
        Action::AutoStashed(inner) => inner,
        other => other,
    };
    Some(
        match action {
            Action::Push { .. }
            | Action::PullThenPush { .. }
            | Action::FetchThenForcePush { .. } => msg::OP_PUSH,
            Action::Pull(_) => msg::OP_PULL,
            Action::Fetch(_) | Action::FetchAll => msg::OP_FETCH,
            Action::CheckoutBranch(_)
            | Action::CheckoutCommit(_)
            | Action::CheckoutThen { .. }
            | Action::CheckoutPullRequestHead { .. } => msg::OP_CHECKOUT,
            Action::Merge(_) => msg::OP_MERGE,
            Action::Rebase(_) | Action::RebaseInteractive(_) => msg::OP_REBASE,
            Action::Commit { .. } | Action::ComposeCommits { .. } => msg::OP_COMMIT,
            Action::StashPush
            | Action::StashPushWith { .. }
            | Action::StashFile(_)
            | Action::StashPop(_)
            | Action::StashApply(_)
            | Action::StashDrop(_) => msg::OP_STASH,
            _ => return None,
        }
        .get()
        .to_string(),
    )
}

/// An action finished in `ms` milliseconds. Returns the duration to attach
/// to the success or failure entry the caller is about to make (`None` when
/// timings are off or the action is not one that is timed).
pub(crate) fn action_finished(action: &Action, repo: &Path, ms: u64, ok: bool) -> Option<u64> {
    let label = label_for(action)?;
    let background = take_background_fetch(action);
    log_line(repo, &label, ms, ok, background);
    let shown = timings_on().then_some(ms);
    if !ok {
        return shown;
    }
    if background {
        record_background(repo, label, ms);
        return None;
    }
    shown
}

/// A timed operation that has no feed entry of its own (the caller passes
/// `ms` on to an entry it already makes, or this makes one).
pub(crate) fn record(repo: &Path, label: String, ms: u64) {
    log_line(repo, &label, ms, true, false);
    if timings_on() {
        feed::push(Entry::operation(repo, label, ms, false));
    }
}

/// A background operation: always in the application log (`debug`), in the
/// list only under `log.extended`.
pub(crate) fn record_background(repo: &Path, label: String, ms: u64) {
    if extended_on() && timings_on() {
        feed::push_background(Entry::operation(repo, label, ms, true));
    }
}

/// The background refresh and its kind of work: debug line and (extended)
/// list entry.
pub(crate) fn background(repo: &Path, label: String, ms: u64) {
    log_line(repo, &label, ms, true, true);
    record_background(repo, label, ms);
}

fn log_line(repo: &Path, label: &str, ms: u64, ok: bool, background: bool) {
    let outcome = if ok { "" } else { " (failed)" };
    let repo = super::repo_text(repo);
    let repo = repo.as_str();
    if background {
        tracing::debug!(target: "tui", repo, "{label} - {ms} ms{outcome}");
    } else {
        tracing::info!(target: "tui", repo, "{label} - {ms} ms{outcome}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The switches are process-wide: tests that flip them take the lock.
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        crate::activity_log::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn repo() -> PathBuf {
        PathBuf::from("/tmp/timing-repo")
    }

    #[test]
    fn a_pull_is_timed_with_its_duration() {
        let _guard = lock();
        install(&gitpoulpi_config::LogConfig::default());
        let action = Action::Pull(Default::default());
        let ms = action_finished(&action, &repo(), 123, true);
        assert_eq!(ms, Some(123));
    }

    #[test]
    fn an_action_without_a_name_is_not_timed() {
        let _guard = lock();
        install(&gitpoulpi_config::LogConfig::default());
        let action = Action::StageAll;
        assert_eq!(action_finished(&action, &repo(), 5, true), None);
    }

    #[test]
    fn timings_off_means_no_duration() {
        let _guard = lock();
        let off = gitpoulpi_config::LogConfig {
            timings: false,
            ..Default::default()
        };
        install(&off);
        let action = Action::Pull(Default::default());
        assert_eq!(action_finished(&action, &repo(), 9, true), None);
        install(&gitpoulpi_config::LogConfig::default());
    }

    #[test]
    fn background_lines_need_extended_logging() {
        let _guard = lock();
        install(&gitpoulpi_config::LogConfig::default());
        let before = feed::snapshot().len();
        background(&repo(), "Refresh".into(), 4);
        assert_eq!(feed::snapshot().len(), before, "extended is off");
        install(&gitpoulpi_config::LogConfig {
            extended: true,
            ..Default::default()
        });
        background(&repo(), "Refresh".into(), 4);
        background(&repo(), "Refresh".into(), 6);
        let lines: Vec<_> = feed::snapshot()
            .into_iter()
            .filter(|e| e.background)
            .collect();
        assert_eq!(lines.len(), 1, "the newest refresh replaces the older one");
        assert_eq!(lines[0].duration_ms, Some(6));
        install(&gitpoulpi_config::LogConfig::default());
    }

    #[test]
    fn an_automatic_fetch_is_background_and_a_manual_one_is_not() {
        let _guard = lock();
        install(&gitpoulpi_config::LogConfig::default());
        expect_background_fetch();
        assert!(take_background_fetch(&Action::FetchAll));
        assert!(!take_background_fetch(&Action::FetchAll), "used up");
        assert!(!take_background_fetch(&Action::StageAll));
    }
}
