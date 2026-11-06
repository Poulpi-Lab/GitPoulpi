//! The contract between the TUI and the `gitpoulpi-updater` executable (the
//! install-channels design, 1.3): the TUI writes an [`UpdateJob`] and starts
//! the updater; the updater reports progress in an [`UpdateStatus`] file the
//! next TUI start reads.
//!
//! Both are versioned JSON (`schema`). A job whose schema this build does not
//! know is refused ([`UpdateJob::parse`] fails, the updater exits with
//! [`exit::BAD_JOB`]) and the panel falls back to "Download": an old
//! updater sitting in Program Files must never guess at a newer job.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The version of both file formats.
pub const SCHEMA: u32 = 1;

/// The updater's process exit codes.
pub mod exit {
    /// Done and relaunched.
    pub const DONE: i32 = 0;
    /// Failed, rolled back.
    pub const FAILED: i32 = 1;
    /// The elevation prompt was declined; nothing changed.
    pub const ELEVATION_DECLINED: i32 = 2;
    pub const NOTHING_TO_DO: i32 = 3;
    pub const REFUSED_BY_POLICY: i32 = 4;
    /// Processes that hold the install would not stop.
    pub const WOULD_NOT_STOP: i32 = 5;
    /// The job file is unreadable, of another schema, or its paths are wrong.
    pub const BAD_JOB: i32 = 6;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Apply,
    Revert,
    /// Run the package manager (`manager_argv`).
    Manager,
}

/// Where the new files are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staged {
    /// Already staged as `.new` files in the install folder.
    InPlace,
    /// Downloaded in the user's cache: the elevated worker verifies it again
    /// against the `.minisig` and the `.sha256` it fetches itself, then
    /// stages it. A job without `minisig_url` is refused (it does not parse).
    Archive {
        archive: PathBuf,
        sha256_url: String,
        minisig_url: String,
    },
}

/// `"in_place"` or `{"archive": ..., "sha256_url": ..., "minisig_url": ...}`
/// on the wire.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StagedRepr {
    Name(String),
    Archive {
        archive: PathBuf,
        sha256_url: String,
        minisig_url: String,
    },
}

impl TryFrom<StagedRepr> for Staged {
    type Error = String;

    fn try_from(r: StagedRepr) -> Result<Self, String> {
        match r {
            StagedRepr::Name(n) if n == "in_place" => Ok(Staged::InPlace),
            StagedRepr::Name(n) => Err(format!("unknown staged value {n:?}")),
            StagedRepr::Archive {
                archive,
                sha256_url,
                minisig_url,
            } => Ok(Staged::Archive {
                archive,
                sha256_url,
                minisig_url,
            }),
        }
    }
}

impl From<Staged> for StagedRepr {
    fn from(s: Staged) -> Self {
        match s {
            Staged::InPlace => StagedRepr::Name("in_place".into()),
            Staged::Archive {
                archive,
                sha256_url,
                minisig_url,
            } => StagedRepr::Archive {
                archive,
                sha256_url,
                minisig_url,
            },
        }
    }
}

impl Serialize for Staged {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        StagedRepr::from(self.clone()).serialize(s)
    }
}

impl<'de> Deserialize<'de> for Staged {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let repr = StagedRepr::deserialize(d)?;
        Staged::try_from(repr).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobElevation {
    None,
    Uac,
}

/// How to start GitPoulpi again once the update is done (unelevated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relaunch {
    pub exe: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateJob {
    pub schema: u32,
    pub action: Action,
    pub install_dir: PathBuf,
    pub from_version: String,
    pub to_version: String,
    pub staged: Staged,
    /// The package manager's command for [`Action::Manager`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager_argv: Option<Vec<String>>,
    pub elevation: JobElevation,
    pub relaunch: Relaunch,
    /// The TUI process the updater waits for before touching files.
    pub requester_pid: u32,
    pub config_dir: PathBuf,
    pub status_path: PathBuf,
}

/// Why a job file was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JobError {
    #[error("the update job is not valid: {0}")]
    Invalid(String),
    #[error("the update job has schema {found}, this updater understands {SCHEMA}")]
    Schema { found: u32 },
    #[error("the update job's {0} is not an absolute path")]
    RelativePath(&'static str),
    #[error("a manager job needs a command")]
    NoManagerCommand,
}

impl UpdateJob {
    pub fn render(&self) -> String {
        serde_json::to_string_pretty(self).expect("a job always serializes")
    }

    /// Reads and validates a job. The schema is checked before the rest so
    /// that a future job, whatever it adds, fails with [`JobError::Schema`].
    pub fn parse(text: &str) -> Result<Self, JobError> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|e| JobError::Invalid(e.to_string()))?;
        let found = value
            .get("schema")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| JobError::Invalid("no schema".into()))?;
        if found != u64::from(SCHEMA) {
            return Err(JobError::Schema {
                found: u32::try_from(found).unwrap_or(u32::MAX),
            });
        }
        let job: UpdateJob =
            serde_json::from_value(value).map_err(|e| JobError::Invalid(e.to_string()))?;
        job.validate()?;
        Ok(job)
    }

    fn validate(&self) -> Result<(), JobError> {
        for (name, path) in [
            ("install_dir", &self.install_dir),
            ("relaunch exe", &self.relaunch.exe),
            ("config_dir", &self.config_dir),
            ("status_path", &self.status_path),
        ] {
            if !path.is_absolute() {
                return Err(JobError::RelativePath(name));
            }
        }
        if self.action == Action::Manager
            && self.manager_argv.as_deref().unwrap_or_default().is_empty()
        {
            return Err(JobError::NoManagerCommand);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Started,
    WaitingForApp,
    StoppingProcesses,
    Working,
    Verifying,
    RollingBack,
    Done,
    Failed,
    Declined,
}

/// What the updater last did, for the TUI's next start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub schema: u32,
    pub phase: Phase,
    /// English technical text: the UI words the outcome and shows this as
    /// the reason.
    pub message: String,
    /// Seconds since the Unix epoch.
    pub at: u64,
}

impl UpdateStatus {
    pub fn new(phase: Phase, message: impl Into<String>, at: u64) -> Self {
        Self {
            schema: SCHEMA,
            phase,
            message: message.into(),
            at,
        }
    }

    pub fn render(&self) -> String {
        serde_json::to_string(self).expect("a status always serializes")
    }

    /// `None` for text this build cannot read (another schema, garbage): no
    /// status line is better than a wrong one.
    pub fn parse(text: &str) -> Option<Self> {
        let s: Self = serde_json::from_str(text).ok()?;
        (s.schema == SCHEMA).then_some(s)
    }

    /// Whether the update is over (successfully or not).
    pub fn is_final(&self) -> bool {
        matches!(self.phase, Phase::Done | Phase::Failed | Phase::Declined)
    }
}

#[cfg(test)]
mod tests;
