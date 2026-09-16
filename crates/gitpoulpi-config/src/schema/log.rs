//! The `log` part of the config schema: the application log the Activity
//! Log shows and "Copy diagnostics" quotes.

use serde::{Deserialize, Serialize};

use super::quantity::{read_quantity, DisplayUnit};

/// The most days of log files kept: ten years. The size cap (`log.max_file_mb`, one file per day, rotated) still bounds
/// the disk use of each day.
pub const MAX_LOG_RETENTION_DAYS: u32 = 3650;

/// Days of log files kept unless the config says otherwise: a week.
pub const DEFAULT_LOG_RETENTION_DAYS: u32 = 7;

/// The unit Settings shows `log.retention_days` in. The stored value is
/// always a number of days; the unit is how that number is read back.
/// A week is 7 days, a month 30 days and a year 365 days (fixed lengths, so
/// "2 months" is the same 60 days whatever today's date).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionUnit {
    /// No unit chosen (an older config): the largest unit that divides the
    /// number of days evenly, so 14 days read "2 weeks" and 45 days "45 days".
    #[default]
    Auto,
    Days,
    Weeks,
    Months,
    Years,
}

impl RetentionUnit {
    pub const ALL: &'static [RetentionUnit] = &[
        RetentionUnit::Auto,
        RetentionUnit::Days,
        RetentionUnit::Weeks,
        RetentionUnit::Months,
        RetentionUnit::Years,
    ];

    /// The four units Settings offers, smallest first.
    pub const CHOICES: &'static [RetentionUnit] = &[
        RetentionUnit::Days,
        RetentionUnit::Weeks,
        RetentionUnit::Months,
        RetentionUnit::Years,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RetentionUnit::Auto => "auto",
            RetentionUnit::Days => "days",
            RetentionUnit::Weeks => "weeks",
            RetentionUnit::Months => "months",
            RetentionUnit::Years => "years",
        }
    }

    /// Days in one of this unit (`Auto` counts as one day).
    pub fn days(self) -> u32 {
        match self {
            RetentionUnit::Auto | RetentionUnit::Days => 1,
            RetentionUnit::Weeks => 7,
            RetentionUnit::Months => 30,
            RetentionUnit::Years => 365,
        }
    }

    /// The largest unit that divides `days` evenly.
    pub fn auto_for(days: u32) -> RetentionUnit {
        [
            RetentionUnit::Years,
            RetentionUnit::Months,
            RetentionUnit::Weeks,
        ]
        .into_iter()
        .find(|u| days >= u.days() && days % u.days() == 0)
        .unwrap_or(RetentionUnit::Days)
    }
}

/// A log file stops growing at this size (MiB) unless the config says
/// otherwise.
pub const DEFAULT_LOG_MAX_FILE_MB: u32 = 5;

/// The most MiB one log file may be capped at (Settings), one GiB.
pub const MAX_LOG_MAX_FILE_MB: u32 = 1024;

/// The units the two size settings are offered in.
pub const MIB_OR_GIB: &[DisplayUnit] = &[DisplayUnit::Mib, DisplayUnit::Gib];

/// The most verbose a recorded line may be. Anything below the chosen level
/// is not recorded at all.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub const ALL: &'static [LogLevel] = &[
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

/// How the status bar's `loaded / total` learns the total: `log.commit_count`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitCountMethod {
    /// `git rev-list --count`: one process, fastest where spawning is cheap
    /// (it reads the commit-graph file libgit2 ignores).
    #[default]
    #[serde(rename = "system-git")]
    SystemGit,
    /// An in-process libgit2 revwalk: spawns nothing, for machines where
    /// every process start is scanned by antivirus software.
    #[serde(rename = "libgit2")]
    Libgit2,
    /// No count at all: the status bar shows only the loaded commits.
    #[serde(rename = "off")]
    Off,
}

impl CommitCountMethod {
    pub const ALL: &'static [CommitCountMethod] = &[
        CommitCountMethod::SystemGit,
        CommitCountMethod::Libgit2,
        CommitCountMethod::Off,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CommitCountMethod::SystemGit => "system-git",
            CommitCountMethod::Libgit2 => "libgit2",
            CommitCountMethod::Off => "off",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Live: the level recorded to the log files (`error` ... `trace`).
    /// The Activity Log's level box changes it.
    pub level: LogLevel,
    /// Live: how many days of log files are kept, today included: 1 to
    /// 3650 (default 7). Older files are deleted when the app starts.
    pub retention_days: u32,
    /// Live: the unit Settings shows `retention_days` in (`auto`, `days`,
    /// `weeks`, `months`, `years`). Display only: the retention itself is
    /// `retention_days`. `auto` (also an older config without the key)
    /// picks the largest unit that divides the days evenly.
    pub retention_unit: RetentionUnit,
    /// Live: the size at which a day's file is rotated, in MiB.
    pub max_file_mb: u32,
    /// Live: the unit Settings shows `max_file_mb` in (`auto`, `mib`, `gib`).
    /// Display only. See [`DisplayUnit`].
    pub max_file_unit: DisplayUnit,
    /// Live: show how long each operation took (the Activity Log's
    /// Repository tab, in milliseconds). Default on.
    pub timings: bool,
    /// Live: "use extended logging" - also list the background operations
    /// (auto-fetch, background refresh, status polling) with their
    /// durations. Default off.
    pub extended: bool,
    /// Live: how the status bar's total commit count is computed
    /// (`system-git`, `libgit2` or `off`). Default `system-git`.
    pub commit_count: CommitCountMethod,
    /// Live: show authors as the repository's `.mailmap` names them in the
    /// Log, blame and file history (the stored commits are untouched).
    /// Default on.
    pub use_mailmap: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            retention_days: DEFAULT_LOG_RETENTION_DAYS,
            retention_unit: RetentionUnit::default(),
            max_file_mb: DEFAULT_LOG_MAX_FILE_MB,
            max_file_unit: DisplayUnit::default(),
            timings: true,
            extended: false,
            commit_count: CommitCountMethod::default(),
            use_mailmap: true,
        }
    }
}

impl LogConfig {
    /// `retention_days` within 1 to [`MAX_LOG_RETENTION_DAYS`].
    pub fn retention_days_clamped(&self) -> u32 {
        self.retention_days.clamp(1, MAX_LOG_RETENTION_DAYS)
    }

    /// The retention as Settings reads it back: a count and a concrete unit
    /// (never `Auto`). The stored unit is used when it divides the days
    /// evenly, else the largest unit that does, so the pair always means
    /// exactly `retention_days_clamped()` days.
    pub fn retention_quantity(&self) -> (u32, RetentionUnit) {
        let days = self.retention_days_clamped();
        let unit = match self.retention_unit {
            RetentionUnit::Auto => RetentionUnit::auto_for(days),
            u if days % u.days() == 0 => u,
            _ => RetentionUnit::auto_for(days),
        };
        (days / unit.days(), unit)
    }

    /// `max_file_mb` as Settings reads it back (1 to
    /// [`MAX_LOG_MAX_FILE_MB`]): a count and `mib` or `gib`.
    pub fn max_file_quantity(&self) -> (u64, DisplayUnit) {
        read_quantity(
            u64::from(self.max_file_mb.clamp(1, MAX_LOG_MAX_FILE_MB)),
            DisplayUnit::Mib,
            self.max_file_unit,
            MIB_OR_GIB,
        )
    }

    /// The per-file size cap in bytes (at least 1 MiB).
    pub fn max_file_bytes(&self) -> u64 {
        u64::from(self.max_file_mb.max(1)) * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_defaults_to_a_week_and_is_clamped_to_ten_years() {
        let mut log = LogConfig::default();
        assert_eq!(log.retention_days, 7);
        assert_eq!(log.retention_days_clamped(), 7);
        log.retention_days = 0;
        assert_eq!(log.retention_days_clamped(), 1);
        log.retention_days = 30;
        assert_eq!(log.retention_days_clamped(), 30);
        log.retention_days = 100_000;
        assert_eq!(log.retention_days_clamped(), 3650);
    }

    #[test]
    fn an_old_config_without_a_unit_reads_back_in_the_largest_even_unit() {
        let read = |days: u32| {
            let log: LogConfig = toml::from_str(&format!("retention_days = {days}")).unwrap();
            assert_eq!(log.retention_unit, RetentionUnit::Auto);
            log.retention_quantity()
        };
        assert_eq!(read(1), (1, RetentionUnit::Days));
        assert_eq!(read(3), (3, RetentionUnit::Days));
        assert_eq!(read(7), (1, RetentionUnit::Weeks));
        assert_eq!(read(14), (2, RetentionUnit::Weeks));
        assert_eq!(read(30), (1, RetentionUnit::Months));
        assert_eq!(read(90), (3, RetentionUnit::Months));
        assert_eq!(read(180), (6, RetentionUnit::Months));
        assert_eq!(read(365), (1, RetentionUnit::Years));
        assert_eq!(read(45), (45, RetentionUnit::Days));
    }

    #[test]
    fn a_chosen_unit_is_kept_while_it_divides_and_dropped_when_it_does_not() {
        let log: LogConfig =
            toml::from_str("retention_days = 14\nretention_unit = \"days\"").unwrap();
        assert_eq!(log.retention_quantity(), (14, RetentionUnit::Days));
        let log: LogConfig =
            toml::from_str("retention_days = 60\nretention_unit = \"months\"").unwrap();
        assert_eq!(log.retention_quantity(), (2, RetentionUnit::Months));
        // Hand-edited to a mismatch: still exactly the stored days.
        let log: LogConfig =
            toml::from_str("retention_days = 10\nretention_unit = \"weeks\"").unwrap();
        assert_eq!(log.retention_quantity(), (10, RetentionUnit::Days));
        assert!(toml::from_str::<LogConfig>("retention_unit = \"fortnights\"").is_err());
    }

    #[test]
    fn commit_count_parses_its_three_spellings_and_defaults_to_system_git() {
        assert_eq!(
            LogConfig::default().commit_count,
            CommitCountMethod::SystemGit
        );
        for m in CommitCountMethod::ALL {
            let log: LogConfig =
                toml::from_str(&format!("commit_count = \"{}\"", m.as_str())).unwrap();
            assert_eq!(log.commit_count, *m);
        }
        assert!(toml::from_str::<LogConfig>("commit_count = \"cycle\"").is_err());
    }
}
