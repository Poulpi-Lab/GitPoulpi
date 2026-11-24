//! Radar pins and snoozes: which pull requests, issues and WIP rows the
//! user pinned to the top or hid for a while.
//!
//! **A file of the user, not of a repository or a daemon**, like
//! `favorites.toml`: `<config dir>/radar.toml`, shared by every session
//! on the machine. Items are keyed by host + repository + kind + number (a
//! pull request `#5` and an issue `#5` are two items); a WIP row has no host
//! and no number, its key is the repository path.
//!
//! A snooze lasts until a moment (1 h, 4 h, a day) or **until the item
//! changes**: the key then remembers the item's `updated_at` at snooze time
//! (its "stamp") and the snooze ends when the host says something else.
//! Expiry is decided against an explicit clock, so tests never sleep.
//!
//! Best-effort I/O: a missing or corrupt file is an empty state, and a save
//! that fails loses a pin, never the window.

use std::path::{Path, PathBuf};

use chrono::TimeZone;
use serde::{Deserialize, Serialize};

use crate::Result;

/// The kind of a Radar item, as stored.
pub const KIND_PR: &str = "pr";
pub const KIND_ISSUE: &str = "issue";
pub const KIND_WIP: &str = "wip";

/// What identifies an item across sessions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemKey {
    /// The provider's host (`github.com`); empty for a WIP row.
    #[serde(default)]
    pub host: String,
    /// `owner/name` on the host; the repository path for a WIP row.
    pub repo: String,
    /// [`KIND_PR`], [`KIND_ISSUE`] or [`KIND_WIP`].
    pub kind: String,
    /// The pull request / issue number; 0 for a WIP row.
    #[serde(default)]
    pub number: u64,
}

impl ItemKey {
    pub fn new(host: &str, repo: &str, kind: &str, number: u64) -> Self {
        Self {
            host: host.to_string(),
            repo: repo.to_string(),
            kind: kind.to_string(),
            number,
        }
    }
}

/// How long a snooze lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnoozeFor {
    Hour,
    FourHours,
    /// Until tomorrow 08:00, local time (the next calendar day, even before 08:00).
    Day,
    /// Until the host reports the item changed (its stamp differs).
    UntilChanged,
}

impl SnoozeFor {
    pub const ALL: [SnoozeFor; 4] = [
        SnoozeFor::Hour,
        SnoozeFor::FourHours,
        SnoozeFor::Day,
        SnoozeFor::UntilChanged,
    ];

    /// The unix time the snooze ends, or `None` for "until it changes"
    /// (local time zone of the machine).
    pub fn ends_at(self, now: i64) -> Option<i64> {
        self.ends_at_in(now, &chrono::Local)
    }

    /// [`SnoozeFor::ends_at`] in an explicit time zone (tests inject one).
    pub fn ends_at_in<Tz: TimeZone>(self, now: i64, tz: &Tz) -> Option<i64> {
        match self {
            SnoozeFor::Hour => Some(now + 3600),
            SnoozeFor::FourHours => Some(now + 4 * 3600),
            SnoozeFor::Day => Some(tomorrow_morning(now, tz)),
            SnoozeFor::UntilChanged => None,
        }
    }
}

/// Tomorrow 08:00 in `tz`; a local time that does not exist (a DST gap)
/// falls back to 24 hours.
fn tomorrow_morning<Tz: TimeZone>(now: i64, tz: &Tz) -> i64 {
    let fallback = now + 24 * 3600;
    let Some(here) = tz.timestamp_opt(now, 0).single() else {
        return fallback;
    };
    let Some(day) = here.date_naive().succ_opt() else {
        return fallback;
    };
    let Some(at) = day.and_hms_opt(8, 0, 0) else {
        return fallback;
    };
    tz.from_local_datetime(&at)
        .earliest()
        .map_or(fallback, |t| t.timestamp())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snooze {
    #[serde(flatten)]
    pub key: ItemKey,
    /// Unix seconds the snooze ends; absent = until the stamp changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    /// The item's `updated_at` when it was snoozed.
    #[serde(default)]
    pub stamp: String,
}

impl Snooze {
    /// Whether the snooze still hides an item whose current stamp is `stamp`.
    pub fn active(&self, now: i64, stamp: &str) -> bool {
        match self.until {
            Some(until) => now < until,
            None => self.stamp == stamp,
        }
    }
}

/// The widths (in cells) the user gave the Radar's sizable columns; `None` =
/// automatic (fitted to the content).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadarColumns {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<u16>,
    /// The branches column (`source -> target`) right after the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branches: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u16>,
}

impl RadarColumns {
    pub fn is_auto(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadarState {
    /// The sizable columns' widths (`[columns]`).
    #[serde(default, skip_serializing_if = "RadarColumns::is_auto")]
    pub columns: RadarColumns,
    #[serde(default)]
    pub pins: Vec<ItemKey>,
    #[serde(default)]
    pub snoozes: Vec<Snooze>,
    /// The scope the user last picked in the view (a workspace id, or `""`
    /// for the open tabs); absent = never picked, the default applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// `[x] Not cloned`: account-wide rows of repositories with no known
    /// local clone are listed. On by default.
    #[serde(default = "default_true")]
    pub show_not_cloned: bool,
    /// `[ ] Drafts`: draft requests of the account-wide rows are listed.
    /// Off by default.
    #[serde(default)]
    pub show_drafts: bool,
}

fn default_true() -> bool {
    true
}

impl Default for RadarState {
    fn default() -> Self {
        Self {
            columns: RadarColumns::default(),
            pins: Vec::new(),
            snoozes: Vec::new(),
            scope: None,
            show_not_cloned: true,
            show_drafts: false,
        }
    }
}

impl RadarState {
    /// `<config dir>/radar.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("radar.toml")
    }

    /// A missing or corrupt file is an empty state: a convenience file must
    /// never make the Radar unusable.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Temp file + rename, so a crash never leaves half a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
        }
        gitpoulpi_config::secure_fs::write_private(path, toml::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }

    pub fn is_pinned(&self, key: &ItemKey) -> bool {
        self.pins.contains(key)
    }

    /// Pins `key` (newest pin last), or unpins it. Returns whether it is
    /// pinned afterwards.
    pub fn toggle_pin(&mut self, key: &ItemKey) -> bool {
        let before = self.pins.len();
        self.pins.retain(|p| p != key);
        if self.pins.len() != before {
            return false;
        }
        self.pins.push(key.clone());
        true
    }

    /// Snoozes `key` for `how`, replacing an earlier snooze of it. `stamp`
    /// is the item's `updated_at` now (used by [`SnoozeFor::UntilChanged`]).
    pub fn snooze(&mut self, key: &ItemKey, how: SnoozeFor, now: i64, stamp: &str) {
        self.unsnooze(key);
        self.snoozes.push(Snooze {
            key: key.clone(),
            until: how.ends_at(now),
            stamp: stamp.to_string(),
        });
    }

    pub fn unsnooze(&mut self, key: &ItemKey) -> bool {
        let before = self.snoozes.len();
        self.snoozes.retain(|s| &s.key != key);
        self.snoozes.len() != before
    }

    /// The snooze hiding `key` right now, if any.
    pub fn snoozed(&self, key: &ItemKey, now: i64, stamp: &str) -> Option<&Snooze> {
        self.snoozes
            .iter()
            .find(|s| &s.key == key && s.active(now, stamp))
    }

    /// Drops the snoozes that ended by time (a stamp snooze has no clock; it
    /// is dropped by [`RadarState::forget`] when its item is gone).
    /// Returns how many.
    pub fn prune(&mut self, now: i64) -> usize {
        let before = self.snoozes.len();
        self.snoozes
            .retain(|s| s.until.is_none_or(|until| now < until));
        before - self.snoozes.len()
    }

    /// Drops every pin and snooze whose key `keep` refuses (an item the host
    /// no longer lists).
    pub fn forget(&mut self, keep: impl Fn(&ItemKey) -> bool) -> usize {
        let before = self.pins.len() + self.snoozes.len();
        self.pins.retain(&keep);
        self.snoozes.retain(|s| keep(&s.key));
        before - self.pins.len() - self.snoozes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(n: u64) -> ItemKey {
        ItemKey::new("github.com", "acme/app", KIND_PR, n)
    }

    #[test]
    fn a_pr_and_an_issue_with_one_number_are_two_items() {
        let mut state = RadarState::default();
        assert!(state.toggle_pin(&pr(5)));
        assert!(!state.is_pinned(&ItemKey::new("github.com", "acme/app", KIND_ISSUE, 5)));
        assert!(!state.toggle_pin(&pr(5)));
        assert!(state.pins.is_empty());
    }

    #[test]
    fn a_timed_snooze_ends_with_the_clock() {
        let mut state = RadarState::default();
        state.snooze(&pr(1), SnoozeFor::Hour, 1000, "t0");
        assert!(state.snoozed(&pr(1), 1000, "t0").is_some());
        assert!(state.snoozed(&pr(1), 4599, "t9").is_some(), "stamp is moot");
        assert!(state.snoozed(&pr(1), 4600, "t0").is_none());
        assert_eq!(state.prune(4599), 0);
        assert_eq!(state.prune(4600), 1);
    }

    #[test]
    fn four_hours_and_a_day() {
        assert_eq!(SnoozeFor::FourHours.ends_at(10), Some(10 + 14_400));
    }

    #[test]
    fn a_day_ends_tomorrow_at_eight_local() {
        use chrono::FixedOffset;
        let tz = FixedOffset::east_opt(2 * 3600).unwrap(); // UTC+2
        let at = |y, m, d, h, mi| tz.with_ymd_and_hms(y, m, d, h, mi, 0).unwrap().timestamp();
        // Afternoon: tomorrow 08:00.
        assert_eq!(
            SnoozeFor::Day.ends_at_in(at(2026, 10, 14, 15, 30), &tz),
            Some(at(2026, 10, 15, 8, 0))
        );
        // Before 08:00: still the next calendar day.
        assert_eq!(
            SnoozeFor::Day.ends_at_in(at(2026, 10, 14, 6, 0), &tz),
            Some(at(2026, 10, 15, 8, 0))
        );
        // Just after midnight, and across a month end.
        assert_eq!(
            SnoozeFor::Day.ends_at_in(at(2026, 10, 31, 0, 5), &tz),
            Some(at(2026, 11, 1, 8, 0))
        );
    }

    #[test]
    fn until_changed_ends_when_the_stamp_moves() {
        let mut state = RadarState::default();
        state.snooze(&pr(2), SnoozeFor::UntilChanged, 0, "2026-10-01T10:00:00Z");
        assert!(state
            .snoozed(&pr(2), 10_000_000, "2026-10-01T10:00:00Z")
            .is_some());
        assert!(state
            .snoozed(&pr(2), 10_000_000, "2026-10-02T09:00:00Z")
            .is_none());
        // A stamp snooze is not pruned by the clock.
        assert_eq!(state.prune(i64::MAX), 0);
    }

    #[test]
    fn snoozing_again_replaces_the_earlier_snooze() {
        let mut state = RadarState::default();
        state.snooze(&pr(3), SnoozeFor::Hour, 0, "");
        state.snooze(&pr(3), SnoozeFor::FourHours, 0, "");
        assert_eq!(state.snoozes.len(), 1);
        assert!(state.snoozed(&pr(3), 7200, "").is_some());
        assert!(state.unsnooze(&pr(3)));
        assert!(!state.unsnooze(&pr(3)));
    }

    #[test]
    fn forget_drops_what_the_host_no_longer_lists() {
        let mut state = RadarState::default();
        state.toggle_pin(&pr(1));
        state.toggle_pin(&pr(2));
        state.snooze(&pr(2), SnoozeFor::Day, 0, "");
        assert_eq!(state.forget(|k| k.number == 1), 2);
        assert_eq!(state.pins, vec![pr(1)]);
        assert!(state.snoozes.is_empty());
    }

    #[test]
    fn the_file_round_trips_and_a_bad_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let file = RadarState::file_in(dir.path());
        assert_eq!(RadarState::load(&file), RadarState::default());
        let mut state = RadarState::default();
        state.toggle_pin(&pr(7));
        state.toggle_pin(&ItemKey::new("", "/home/me/app", KIND_WIP, 0));
        state.snooze(&pr(8), SnoozeFor::FourHours, 100, "x");
        state.snooze(&pr(9), SnoozeFor::UntilChanged, 100, "stamp");
        state.scope = Some("ws-1".into());
        state.show_not_cloned = false;
        state.show_drafts = true;
        state.save(&file).unwrap();
        assert_eq!(RadarState::load(&file), state);
        std::fs::write(&file, "pins = [[[").unwrap();
        assert_eq!(RadarState::load(&file), RadarState::default());
    }

    #[test]
    fn the_account_checkboxes_default_to_not_cloned_on_and_drafts_off() {
        let state = RadarState::default();
        assert!(state.show_not_cloned && !state.show_drafts);
        // A file written before the checkboxes existed keeps the defaults.
        let old: RadarState = toml::from_str("pins = []\nsnoozes = []\n").unwrap();
        assert!(old.show_not_cloned && !old.show_drafts);
        let off: RadarState =
            toml::from_str("show_not_cloned = false\nshow_drafts = true\n").unwrap();
        assert!(!off.show_not_cloned && off.show_drafts);
    }

    #[test]
    fn the_branches_width_loads_when_present_and_old_files_load_unchanged() {
        let old: RadarState =
            toml::from_str("[columns]\nrepo = 20\nnumber = 5\nauthor = 9\nage = 4\n").unwrap();
        assert_eq!(old.columns.branches, None);
        assert_eq!(old.columns.repo, Some(20));
        let new: RadarState = toml::from_str("[columns]\nbranches = 31\n").unwrap();
        assert_eq!(new.columns.branches, Some(31));
        assert!(!new.columns.is_auto());
        let text = toml::to_string(&new).unwrap();
        assert!(text.contains("branches = 31"), "{text}");
        assert!(!toml::to_string(&old).unwrap().contains("branches"));
    }
}
