//! How an update ends: the exit code, the phase the status file records and
//! the English reason the TUI shows next start.

use gitpoulpi_update::fsops::FileOps;
use gitpoulpi_update::job::{exit, Phase, UpdateStatus};

use crate::ops::Clock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Done(String),
    /// Failed and rolled back (or nothing was changed).
    Failed(String),
    /// The elevation prompt was declined.
    Declined,
    NothingToDo(String),
    Refused(String),
    WouldNotStop(String),
    BadJob(String),
}

impl Outcome {
    pub fn code(&self) -> i32 {
        match self {
            Outcome::Done(_) => exit::DONE,
            Outcome::Failed(_) => exit::FAILED,
            Outcome::Declined => exit::ELEVATION_DECLINED,
            Outcome::NothingToDo(_) => exit::NOTHING_TO_DO,
            Outcome::Refused(_) => exit::REFUSED_BY_POLICY,
            Outcome::WouldNotStop(_) => exit::WOULD_NOT_STOP,
            Outcome::BadJob(_) => exit::BAD_JOB,
        }
    }

    pub fn phase(&self) -> Phase {
        match self {
            Outcome::Done(_) => Phase::Done,
            Outcome::Declined => Phase::Declined,
            _ => Phase::Failed,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Outcome::Done(m)
            | Outcome::Failed(m)
            | Outcome::NothingToDo(m)
            | Outcome::Refused(m)
            | Outcome::WouldNotStop(m)
            | Outcome::BadJob(m) => m.clone(),
            Outcome::Declined => "the administrator prompt was declined; nothing changed".into(),
        }
    }
}

/// Writes `status.json` at each step, and echoes the message to the console
/// the updater runs in.
pub struct Status<'a> {
    pub fs: &'a dyn FileOps,
    pub clock: &'a dyn Clock,
    pub path: &'a std::path::Path,
    pub echo: bool,
}

impl Status<'_> {
    pub fn set(&self, phase: Phase, message: &str) {
        if self.echo {
            println!("{message}");
        }
        let status = UpdateStatus::new(phase, message, self.clock.now());
        // A status that cannot be written must not stop the update: the
        // exit code still says how it went.
        let _ = self.fs.write(self.path, status.render().as_bytes(), false);
    }

    pub fn finish(&self, outcome: &Outcome) {
        self.set(outcome.phase(), &outcome.message());
    }

    /// What the file says now (the elevated worker writes its own reason).
    pub fn read(&self) -> Option<UpdateStatus> {
        UpdateStatus::parse(&self.fs.read_to_string(self.path).ok()?)
    }
}
