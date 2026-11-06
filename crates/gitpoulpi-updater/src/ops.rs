//! Everything the updater does to the machine, behind traits, so the whole
//! procedure (stop, swap, verify, roll back, relaunch) runs against fakes in
//! the tests and the real implementations (`real`) are thin.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_update::channel::{Hive, RegistryOps};

/// A running process, as far as stopping goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub exe: PathBuf,
}

/// Why an external command did not run to an exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The elevation prompt (UAC) was declined: nothing ran.
    Declined,
    Failed(String),
}

/// What to start once the update is over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// Unix: replace this process, so GitPoulpi takes the terminal over.
    /// Windows: start it detached (a new window), this process ends.
    pub replace_process: bool,
}

pub trait ProcessOps {
    fn is_alive(&self, pid: u32) -> bool;
    /// Processes whose executable lives in `dir`, except `exclude`.
    fn in_dir(&self, dir: &Path, exclude: &[u32]) -> Vec<Proc>;
    /// Processes holding any of `files` open (Windows: the Restart Manager,
    /// which sees other users' processes only when elevated); empty
    /// elsewhere.
    fn locking(&self, files: &[PathBuf], exclude: &[u32]) -> Vec<Proc>;
    /// `force` is the second attempt (Unix: SIGKILL after a SIGTERM).
    fn terminate(&self, pid: u32, force: bool) -> Result<(), String>;
    /// Runs a command with its output on this console, unelevated.
    fn run(&self, argv: &[String]) -> Result<i32, RunError>;
    /// Runs a command through the elevation prompt and waits for it.
    fn run_elevated(&self, argv: &[String]) -> Result<i32, RunError>;
    /// `exe --version`'s stdout.
    fn version_of(&self, exe: &Path) -> Option<String>;
    /// Starts GitPoulpi again. With `replace_process` it does not return on
    /// success.
    fn relaunch(&self, plan: &LaunchPlan) -> Result<(), String>;
}

/// One session daemon found through its `sessions/<name>.port` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDaemon {
    pub name: String,
    pub port_file: PathBuf,
}

pub trait DaemonOps {
    fn sessions(&self, config_dir: &Path) -> Vec<SessionDaemon>;
    /// Asks the daemon to stop for an update (`Shutdown {reason: update}`).
    /// `false` when nothing answered.
    fn request_shutdown(&self, daemon: &SessionDaemon) -> bool;
    /// Whether the daemon is still there: its port file exists and the
    /// process that wrote it is alive.
    fn is_up(&self, daemon: &SessionDaemon) -> bool;
}

/// The Inno uninstall key: read for the channel, written for the version
/// shown in "Apps & features".
pub trait Registry {
    fn reader(&self) -> &dyn RegistryOps;
    fn set_display_version(&self, hive: Hive, version: &str) -> Result<(), String>;
}

pub trait Clock {
    /// Seconds since the Unix epoch.
    fn now(&self) -> u64;
    fn sleep(&self, d: Duration);
}

/// The two reads the elevated worker does itself.
pub trait Transfer {
    /// The body of `url`, over the network.
    fn get(&self, url: &str) -> Result<Vec<u8>, String>;
    /// A file's bytes (the archive the user-level process downloaded).
    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>>;
}
