//! The `rewrite` part of the config schema: history rewrite
//! (the history-rewrite design).

use serde::{Deserialize, Serialize};

use super::quantity::{read_quantity, DisplayUnit};
use gitpoulpi_core::domain::rewrite::script::{
    DEFAULT_SCRIPT_TIMEOUT_MS, DEFAULT_SCRIPT_TOTAL_TIMEOUT_S,
};

/// The most an external script gets to answer one commit (Settings): ten
/// minutes, in milliseconds.
pub const MAX_SCRIPT_TIMEOUT_MS: u64 = 600_000;

/// The most one pass of an external script may take in all (Settings): a
/// day, in seconds.
pub const MAX_SCRIPT_TOTAL_TIMEOUT_S: u64 = 86_400;

/// The units `rewrite.script_timeout_ms` is offered in.
pub const SCRIPT_TIMEOUT_UNITS: &[DisplayUnit] = &[DisplayUnit::Milliseconds, DisplayUnit::Seconds];

/// The units `rewrite.script_total_timeout_s` is offered in.
pub const SCRIPT_TOTAL_TIMEOUT_UNITS: &[DisplayUnit] = &[
    DisplayUnit::Seconds,
    DisplayUnit::Minutes,
    DisplayUnit::Hours,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RewriteConfig {
    /// Live: branches (names, or patterns with `*`) that a history rewrite
    /// leaves alone when they are already pushed, unless the run lists them
    /// (`--include-protected`, or the checkbox of the workbench). Added to
    /// the bases a repository's `workflow.toml` marks `protected`. Not a
    /// host-side protection: the host's own rules still apply to the push.
    pub protected_branches: Vec<String>,
    /// Live: which external script languages a history rewrite may run, a
    /// subset of `"shell"`, `"python"` and `"lua"` (an installed Lua, not the
    /// embedded one, which is always on). Shell scripts are on by default
    /// (when a shell is found); Python and external Lua are off, because an
    /// external script is not sandboxed: it runs with your rights. Even when
    /// on, a script from the repository runs only after you trusted it.
    pub external_scripts: Vec<String>,
    /// Live: how long an external script gets to answer one commit, in
    /// milliseconds. `0` reads as the default (5000).
    pub script_timeout_ms: u64,
    /// Live: the unit Settings shows `script_timeout_ms` in (`auto`,
    /// `milliseconds`, `seconds`). Display only. See [`DisplayUnit`].
    pub script_timeout_unit: DisplayUnit,
    /// Live: how long one pass of an external script may take in all, in
    /// seconds. `0` reads as the default (3600).
    pub script_total_timeout_s: u64,
    /// Live: the unit Settings shows `script_total_timeout_s` in (`auto`,
    /// `seconds`, `minutes`, `hours`). Display only. See [`DisplayUnit`].
    pub script_total_timeout_unit: DisplayUnit,
}

impl Default for RewriteConfig {
    fn default() -> Self {
        Self {
            protected_branches: gitpoulpi_core::domain::rewrite::DEFAULT_PROTECTED_BRANCHES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            external_scripts: vec!["shell".to_string()],
            script_timeout_ms: DEFAULT_SCRIPT_TIMEOUT_MS,
            script_timeout_unit: DisplayUnit::default(),
            script_total_timeout_s: DEFAULT_SCRIPT_TOTAL_TIMEOUT_S,
            script_total_timeout_unit: DisplayUnit::default(),
        }
    }
}

impl RewriteConfig {
    /// The per-answer timeout as Settings reads it back: `0` reads as the
    /// default, a value above [`MAX_SCRIPT_TIMEOUT_MS`] as that maximum.
    pub fn script_timeout_quantity(&self) -> (u64, DisplayUnit) {
        let ms = match self.script_timeout_ms {
            0 => DEFAULT_SCRIPT_TIMEOUT_MS,
            n => n.min(MAX_SCRIPT_TIMEOUT_MS),
        };
        read_quantity(
            ms,
            DisplayUnit::Milliseconds,
            self.script_timeout_unit,
            SCRIPT_TIMEOUT_UNITS,
        )
    }

    /// The whole-pass timeout as Settings reads it back, like
    /// [`Self::script_timeout_quantity`] (seconds, up to a day).
    pub fn script_total_timeout_quantity(&self) -> (u64, DisplayUnit) {
        let s = match self.script_total_timeout_s {
            0 => DEFAULT_SCRIPT_TOTAL_TIMEOUT_S,
            n => n.min(MAX_SCRIPT_TOTAL_TIMEOUT_S),
        };
        read_quantity(
            s,
            DisplayUnit::Seconds,
            self.script_total_timeout_unit,
            SCRIPT_TOTAL_TIMEOUT_UNITS,
        )
    }
}
