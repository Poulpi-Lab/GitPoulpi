//! The data behind the Activity Log: which entries a tab lists under the chosen levels and scope, and the
//! text a line copies. Pure functions over the feed and the application log's records.

use std::path::Path;

use gitpoulpi_i18n::msg::activity_log as m;
use gitpoulpi_log::{Level, Record};

use crate::daemon::activity::FeedEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTab {
    /// What was done to the repository: operations with their duration, failures, messages.
    Git,
    /// The application log: the window's and the daemon's lines.
    App,
}

/// How far the tabs reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// This repository (and what belongs to no repository: startup, the daemon itself).
    ThisRepo,
    All,
}

/// The levels a tab shows; Debug and Trace are off until asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Levels(pub [bool; 5]);

impl Default for Levels {
    fn default() -> Self {
        Levels([true, true, true, false, false])
    }
}

impl Levels {
    pub fn shows(&self, level: Level) -> bool {
        self.0[level as usize]
    }

    pub fn toggle(&mut self, level: Level) {
        self.0[level as usize] = !self.0[level as usize];
    }

    pub fn all() -> Self {
        Levels([true; 5])
    }

    pub fn none() -> Self {
        Levels([false; 5])
    }
}

/// The word of a level in the UI language.
pub fn level_name(level: Level) -> &'static str {
    match level {
        Level::Error => m::LEVEL_ERROR.get(),
        Level::Warn => m::LEVEL_WARN.get(),
        Level::Info => m::LEVEL_INFO.get(),
        Level::Debug => m::LEVEL_DEBUG.get(),
        Level::Trace => m::LEVEL_TRACE.get(),
    }
}

fn in_scope(scope: Scope, repo: Option<&str>, root: &Path) -> bool {
    match (scope, repo) {
        (Scope::All, _) | (Scope::ThisRepo, None) => true,
        (Scope::ThisRepo, Some(r)) => Path::new(r) == root,
    }
}

/// The Git operations entries to list, newest first.
pub fn git_entries(
    feed: &[FeedEntry],
    levels: &Levels,
    scope: Scope,
    root: &Path,
) -> Vec<FeedEntry> {
    feed.iter()
        .rev()
        .filter(|e| levels.shows(e.level))
        .filter(|e| in_scope(scope, e.repo.as_deref().and_then(|p| p.to_str()), root))
        .cloned()
        .collect()
}

/// The application log lines to list, newest first.
pub fn app_records(records: &[Record], levels: &Levels, scope: Scope, root: &Path) -> Vec<Record> {
    records
        .iter()
        .rev()
        .filter(|r| levels.shows(r.level))
        .filter(|r| in_scope(scope, r.repo.as_deref(), root))
        .cloned()
        .collect()
}

/// The one line a record is listed as.
pub fn record_line(r: &Record) -> String {
    let first = r.message.lines().next().unwrap_or("");
    format!(
        "{}  {:<5}  {}: {first}",
        r.clock(),
        r.level.as_str(),
        r.target
    )
}

/// The one line a feed entry is listed as.
pub fn entry_line(e: &FeedEntry) -> String {
    let mut line = format!(
        "{}  {:<5}  {}",
        gitpoulpi_log::clock(e.ts_ms),
        e.level.as_str(),
        e.title
    );
    if let Some(ms) = e.duration_ms {
        line.push_str(&format!("  ({})", m::duration(ms)));
    }
    line
}

/// The index of the next problem (an error or a warning) after `from`, in the direction asked; wraps.
pub fn next_problem(levels: &[Level], from: Option<usize>, forward: bool) -> Option<usize> {
    let n = levels.len();
    if n == 0 {
        return None;
    }
    let start = from.unwrap_or(if forward { n - 1 } else { 0 });
    (1..=n)
        .map(|step| {
            if forward {
                (start + step) % n
            } else {
                (start + n - step % n) % n
            }
        })
        .find(|&i| levels[i] <= Level::Warn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(level: Level, title: &str, repo: Option<&str>) -> FeedEntry {
        FeedEntry {
            id: 0,
            ts_ms: 0,
            level,
            title: title.to_string(),
            details: Vec::new(),
            repo: repo.map(Into::into),
            duration_ms: None,
        }
    }

    #[test]
    fn levels_default_to_the_quiet_ones_and_toggle() {
        let mut levels = Levels::default();
        assert!(levels.shows(Level::Error) && levels.shows(Level::Info));
        assert!(!levels.shows(Level::Debug));
        levels.toggle(Level::Debug);
        assert!(levels.shows(Level::Debug));
        assert!(!Levels::none().shows(Level::Error));
        assert!(Levels::all().shows(Level::Trace));
    }

    #[test]
    fn the_scope_keeps_this_repository_and_what_belongs_to_none() {
        let feed = [
            entry(Level::Info, "here", Some("/r/a")),
            entry(Level::Info, "there", Some("/r/b")),
            entry(Level::Info, "app", None),
        ];
        let titles = |scope| -> Vec<String> {
            git_entries(&feed, &Levels::default(), scope, Path::new("/r/a"))
                .into_iter()
                .map(|e| e.title)
                .collect()
        };
        assert_eq!(titles(Scope::ThisRepo), ["app", "here"], "newest first");
        assert_eq!(titles(Scope::All), ["app", "there", "here"]);
    }

    #[test]
    fn hidden_levels_are_left_out() {
        let feed = [
            entry(Level::Error, "failed", None),
            entry(Level::Debug, "poll", None),
        ];
        let shown = git_entries(&feed, &Levels::default(), Scope::All, Path::new("/r"));
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].title, "failed");
    }

    #[test]
    fn the_next_problem_skips_quiet_lines_and_wraps() {
        use Level::*;
        let levels = [Info, Error, Info, Warn, Info];
        assert_eq!(next_problem(&levels, Some(1), true), Some(3));
        assert_eq!(next_problem(&levels, Some(3), true), Some(1), "wraps");
        assert_eq!(next_problem(&levels, Some(3), false), Some(1));
        assert_eq!(next_problem(&levels, Some(1), false), Some(3), "wraps back");
        assert_eq!(next_problem(&[Info, Info], None, true), None);
        assert_eq!(next_problem(&levels, None, true), Some(1));
    }
}
