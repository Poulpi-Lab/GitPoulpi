//! The "an update is running" lock files (the install-channels design, 1.3).
//!
//! Two files say it: `<data>/updates/<install-id>/updating.lock` and, when the updater
//! can write there, `<install_dir>/.gitpoulpi-updating`. Both hold the
//! updater's pid and the time it took the lock. The CLI and the daemon
//! auto-start refuse while one is live; the updater itself ignores one that
//! is [`STALE_AFTER_SECS`] old (a crashed updater must never lock GitPoulpi
//! out for good).
//!
//! Trap: staleness is by age only. A pid check would be wrong on Windows
//! (pid reuse), and a lock whose owner died costs at most ten minutes of
//! "try again in a moment".

use std::path::Path;

/// The marker the updater drops in the install folder.
pub const INSTALL_LOCK_FILE: &str = ".gitpoulpi-updating";

/// A lock older than this is ignored.
pub const STALE_AFTER_SECS: u64 = 600;

/// What the CLI and the daemon auto-start say while an update runs.
pub const UPDATING_MESSAGE: &str = "GitPoulpi is updating, try again in a moment";

pub fn render(pid: u32, at: u64) -> String {
    format!("pid={pid}\nat={at}\n")
}

/// `(pid, at)` of a lock file's text; `None` for text that is not one.
pub fn parse(text: &str) -> Option<(u32, u64)> {
    let mut pid = None;
    let mut at = None;
    for line in text.lines() {
        match line.split_once('=') {
            Some(("pid", v)) => pid = v.trim().parse().ok(),
            Some(("at", v)) => at = v.trim().parse().ok(),
            _ => {}
        }
    }
    Some((pid?, at?))
}

/// Whether `text` is a lock that still counts at `now`. Text that cannot be
/// read is stale: a half-written lock must not block anything.
pub fn is_live(text: &str, now: u64) -> bool {
    parse(text).is_some_and(|(_, at)| now.saturating_sub(at) < STALE_AFTER_SECS)
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Whether an update is running right now: a live lock in the data folder or
/// in `install_dir`.
pub fn update_in_progress(
    paths: &crate::ConfigPaths,
    install_dir: Option<&Path>,
    now: u64,
) -> bool {
    let live = |p: &Path| {
        std::fs::read_to_string(p)
            .map(|t| is_live(&t, now))
            .unwrap_or(false)
    };
    live(&paths.update_lock_file()) || install_dir.is_some_and(|d| live(&d.join(INSTALL_LOCK_FILE)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lock_round_trips_and_goes_stale_after_ten_minutes() {
        let text = render(42, 1_000);
        assert_eq!(parse(&text), Some((42, 1_000)));
        assert!(is_live(&text, 1_000 + STALE_AFTER_SECS - 1));
        assert!(!is_live(&text, 1_000 + STALE_AFTER_SECS));
        assert!(!is_live("garbage", 1_000));
    }

    #[test]
    fn progress_is_seen_in_either_lock_file_and_not_in_a_stale_one() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::ConfigPaths::for_dir(dir.path().join("cfg"));
        let install = dir.path().join("install");
        std::fs::create_dir_all(&install).unwrap();
        assert!(!update_in_progress(&paths, Some(&install), 100));

        std::fs::create_dir_all(paths.updates_dir()).unwrap();
        std::fs::write(paths.update_lock_file(), render(1, 90)).unwrap();
        assert!(update_in_progress(&paths, None, 100));
        assert!(!update_in_progress(&paths, None, 90 + STALE_AFTER_SECS));
        std::fs::remove_file(paths.update_lock_file()).unwrap();

        std::fs::write(install.join(INSTALL_LOCK_FILE), render(1, 90)).unwrap();
        assert!(update_in_progress(&paths, Some(&install), 100));
        assert!(!update_in_progress(&paths, None, 100));
    }
}
