//! Fakes for every trait in `ops`, sharing one event log so a test can assert
//! the order things happened in. Files are real (a temp folder): the swap
//! logic is the engine's and needs no simulation here.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gitpoulpi_config::policy::PolicySource;
use gitpoulpi_update::channel::{ChannelEnv, Hive, RegistryOps};

use crate::ops::{
    Clock, DaemonOps, LaunchPlan, Proc, ProcessOps, Registry, RunError, SessionDaemon, StopScope,
    Transfer,
};

pub type Log = Rc<RefCell<Vec<String>>>;

pub fn new_log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

/// Virtual time: `sleep` only moves the clock, so a five second grace costs
/// nothing.
pub struct FakeClock {
    ms: Cell<u64>,
}

impl FakeClock {
    pub fn new(start_secs: u64) -> Self {
        Self {
            ms: Cell::new(start_secs * 1000),
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> u64 {
        self.ms.get() / 1000
    }

    fn sleep(&self, d: Duration) {
        self.ms.set(self.ms.get() + d.as_millis() as u64);
    }
}

pub struct FakeProc {
    pub log: Log,
    /// Running processes of the install folder.
    pub running: RefCell<Vec<Proc>>,
    /// Pids that ignore every termination request.
    pub stubborn: RefCell<Vec<u32>>,
    /// Pids `is_alive` reports.
    pub alive: RefCell<Vec<u32>>,
    /// What `run_elevated` answers; `None` = `Ok(0)`.
    pub elevated: RefCell<Option<Result<i32, RunError>>>,
    pub run_code: Cell<i32>,
    /// Processes only an elevated look sees (other users').
    pub other_users: RefCell<Vec<Proc>>,
    /// The version `version_of` reports instead of reading the exe.
    pub reports: RefCell<Option<String>>,
}

impl FakeProc {
    pub fn new(log: &Log) -> Self {
        Self {
            log: log.clone(),
            running: RefCell::new(Vec::new()),
            stubborn: RefCell::new(Vec::new()),
            alive: RefCell::new(Vec::new()),
            elevated: RefCell::new(None),
            run_code: Cell::new(0),
            other_users: RefCell::new(Vec::new()),
            reports: RefCell::new(None),
        }
    }
}

impl ProcessOps for FakeProc {
    fn is_alive(&self, pid: u32) -> bool {
        self.alive.borrow().contains(&pid)
    }

    fn in_dir(&self, _dir: &Path, exclude: &[u32]) -> Vec<Proc> {
        self.running
            .borrow()
            .iter()
            .filter(|p| !exclude.contains(&p.pid))
            .cloned()
            .collect()
    }

    fn locking(&self, _files: &[PathBuf], exclude: &[u32]) -> Vec<Proc> {
        self.log.borrow_mut().push("locking".into());
        self.other_users
            .borrow()
            .iter()
            .filter(|p| !exclude.contains(&p.pid))
            .cloned()
            .collect()
    }

    fn terminate(&self, pid: u32, force: bool) -> Result<(), String> {
        self.log.borrow_mut().push(format!(
            "terminate:{pid}:{}",
            if force { "force" } else { "soft" }
        ));
        if !self.stubborn.borrow().contains(&pid) {
            self.running.borrow_mut().retain(|p| p.pid != pid);
            self.other_users.borrow_mut().retain(|p| p.pid != pid);
        }
        Ok(())
    }

    fn run(&self, argv: &[String]) -> Result<i32, RunError> {
        self.log
            .borrow_mut()
            .push(format!("run:{}", argv.join(" ")));
        Ok(self.run_code.get())
    }

    fn run_elevated(&self, argv: &[String]) -> Result<i32, RunError> {
        self.log
            .borrow_mut()
            .push(format!("run_elevated:{}", argv.join(" ")));
        self.elevated.borrow().clone().unwrap_or(Ok(0))
    }

    fn version_of(&self, exe: &Path) -> Option<String> {
        if let Some(v) = self.reports.borrow().clone() {
            return Some(format!("gitpoulpi {v} (abc123)"));
        }
        let text = std::fs::read_to_string(exe).ok()?;
        let v = text.trim().strip_prefix("v:")?;
        Some(format!("gitpoulpi {v} (abc123)"))
    }

    fn relaunch(&self, plan: &LaunchPlan) -> Result<(), String> {
        self.log.borrow_mut().push(format!(
            "relaunch:{}:{}:{}",
            plan.program.display(),
            plan.args.join(" "),
            plan.replace_process
        ));
        Ok(())
    }
}

pub struct FakeDaemons {
    pub log: Log,
    pub names: Vec<String>,
    /// Daemons that never go away.
    pub stubborn: bool,
    pub down: RefCell<Vec<String>>,
    /// Recorded at each shutdown request: was the lock file there?
    pub lock_path: PathBuf,
    /// The scope the last `sessions` call got.
    pub scope: RefCell<Option<StopScope>>,
}

impl DaemonOps for FakeDaemons {
    fn sessions(&self, scope: &StopScope) -> Vec<SessionDaemon> {
        *self.scope.borrow_mut() = Some(scope.clone());
        self.names
            .iter()
            .map(|n| SessionDaemon {
                name: n.clone(),
                port_file: PathBuf::from(format!("{n}.port")),
            })
            .collect()
    }

    fn request_shutdown(&self, daemon: &SessionDaemon) -> bool {
        self.log.borrow_mut().push(format!(
            "shutdown:{}:lock={}",
            daemon.name,
            self.lock_path.exists()
        ));
        if !self.stubborn {
            self.down.borrow_mut().push(daemon.name.clone());
        }
        true
    }

    fn is_up(&self, daemon: &SessionDaemon) -> bool {
        !self.down.borrow().contains(&daemon.name)
    }
}

#[derive(Default)]
pub struct FakeRegistry {
    pub log: Log,
    pub inno: BTreeMap<&'static str, String>,
    pub fail_write: bool,
}

impl RegistryOps for FakeRegistry {
    fn inno_install_location(&self, hive: Hive) -> Option<String> {
        let key = match hive {
            Hive::CurrentUser => "user",
            Hive::LocalMachine => "machine",
        };
        self.inno.get(key).cloned()
    }
}

impl Registry for FakeRegistry {
    fn reader(&self) -> &dyn RegistryOps {
        self
    }

    fn set_display_version(&self, hive: Hive, version: &str) -> Result<(), String> {
        self.log
            .borrow_mut()
            .push(format!("display:{hive:?}:{version}"));
        if self.fail_write {
            Err("denied".into())
        } else {
            Ok(())
        }
    }
}

pub struct FakeTransfer {
    pub log: Log,
    pub urls: BTreeMap<String, Vec<u8>>,
}

impl Transfer for FakeTransfer {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.log.borrow_mut().push(format!("get:{url}"));
        self.urls.get(url).cloned().ok_or_else(|| "404".into())
    }

    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }
}

pub struct FakeChannelEnv {
    pub writable: bool,
}

impl ChannelEnv for FakeChannelEnv {
    fn cargo_home(&self) -> Option<PathBuf> {
        None
    }

    fn dir_writable(&self, _dir: &Path) -> bool {
        self.writable
    }
}

pub struct FakePolicy {
    pub file: Option<String>,
    pub registry: Option<BTreeMap<String, String>>,
}

impl PolicySource for FakePolicy {
    fn file_text(&self) -> Option<String> {
        self.file.clone()
    }

    fn registry_values(&self) -> Option<BTreeMap<String, String>> {
        self.registry.clone()
    }
}
