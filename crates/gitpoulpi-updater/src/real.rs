//! The real machine: processes, daemons over IPC, the clock, the network.
//! The Windows pieces (Toolhelp, Restart Manager, `ShellExecuteExW runas`,
//! the registry) live in `real_windows.rs`; this file is the part that is the
//! same everywhere.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::{discovery, Client, Request, ShutdownArgs};

use crate::ops::{Clock, DaemonOps, LaunchPlan, RunError, SessionDaemon, StopScope, Transfer};

pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> u64 {
        gitpoulpi_config::update_lock::now_secs()
    }

    fn sleep(&self, d: Duration) {
        std::thread::sleep(d);
    }
}

/// Session daemons through their `sessions/*.port` files. Every call builds a
/// tiny runtime: the updater is synchronous everywhere else.
pub struct RealDaemons;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a current-thread runtime always builds")
}

fn port_files(dir: &Path) -> Vec<SessionDaemon> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "port"))
        .filter_map(|p| {
            let name = p.file_stem()?.to_str()?.to_string();
            Some(SessionDaemon { name, port_file: p })
        })
        .collect()
}

/// The daemons an update stops: those of `sessions/env/<env>/`, and the
/// flat `sessions/*.port` ones (older builds) whose recorded process (a
/// standalone daemon's pid, an embedded one's host) runs from the install
/// folder. Another installation's daemons, in its own environment or hosted
/// from its own folder, are left alone.
pub(crate) fn scan_sessions(scope: &StopScope) -> Vec<SessionDaemon> {
    let paths = gitpoulpi_config::ConfigPaths::for_dir(&scope.config_dir);
    let flat = paths.sessions_dir();
    let mut found = match &scope.env {
        Some(env) => port_files(
            &flat
                .join("env")
                .join(gitpoulpi_config::installation::sanitize(env)),
        ),
        None => Vec::new(),
    };
    found.extend(port_files(&flat).into_iter().filter(|d| {
        discovery::recorded_pid(&d.port_file).is_some_and(|pid| scope.install_pids.contains(&pid))
    }));
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

impl DaemonOps for RealDaemons {
    fn sessions(&self, scope: &StopScope) -> Vec<SessionDaemon> {
        scan_sessions(scope)
    }

    fn request_shutdown(&self, daemon: &SessionDaemon) -> bool {
        runtime().block_on(async {
            let Some(endpoint) = discovery::live_endpoint(&daemon.port_file).await else {
                return false;
            };
            // Without the token file (another account, an old daemon's
            // leftovers) this fails and the caller falls back to stopping the
            // processes.
            let Ok(mut client) = Client::connect(&endpoint).await else {
                return false;
            };
            let request = Request {
                id: 1,
                session: daemon.name.clone(),
                repo: None,
                command: OperationKind::Shutdown,
                args: serde_json::to_value(ShutdownArgs::update()).unwrap_or_default(),
            };
            client.call(&request).await.is_ok()
        })
    }

    fn is_up(&self, daemon: &SessionDaemon) -> bool {
        daemon.port_file.exists()
            && !discovery::remove_if_stale(&daemon.port_file, &discovery::pid_alive)
    }
}

/// The network read of the elevated worker and the archive file.
pub struct RealTransfer;

impl Transfer for RealTransfer {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        use gitpoulpi_hosting::http::{HttpRequest, HttpTransport, ReqwestTransport};
        let transport = ReqwestTransport::new().map_err(|e| e.to_string())?;
        runtime()
            .block_on(async {
                transport
                    .download(HttpRequest::get(url.to_string()), &|_, _| true)
                    .await
            })
            .map_err(|e| e.to_string())
    }

    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }
}

/// `argv` as a command: the program and its arguments.
pub(crate) fn command(argv: &[String]) -> Result<Command, RunError> {
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| RunError::Failed("an empty command".into()))?;
    let mut cmd = Command::new(program);
    cmd.args(rest);
    Ok(cmd)
}

pub(crate) fn run_inherit(argv: &[String]) -> Result<i32, RunError> {
    command(argv)?
        .status()
        .map(|s| s.code().unwrap_or(1))
        .map_err(|e| RunError::Failed(e.to_string()))
}

pub(crate) fn version_of(exe: &Path) -> Option<String> {
    let out = Command::new(exe)
        .arg("--version")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub(crate) fn relaunch_now(plan: &LaunchPlan) -> Result<(), String> {
    let mut cmd = Command::new(&plan.program);
    cmd.args(&plan.args).current_dir(&plan.cwd);
    #[cfg(unix)]
    if plan.replace_process {
        use std::os::unix::process::CommandExt;
        // Only returns when it failed.
        return Err(cmd.exec().to_string());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // The launcher has no console of its own; GitPoulpi itself needs one.
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        let launcher = plan
            .program
            .file_stem()
            .is_some_and(|n| n == "gitpoulpi-launch");
        cmd.creation_flags(if launcher {
            DETACHED_PROCESS
        } else {
            CREATE_NEW_CONSOLE
        });
    }
    cmd.stdin(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(unix)]
mod unix {
    use super::*;
    use crate::ops::{Proc, ProcessOps};
    use std::path::PathBuf;

    pub struct RealProc;

    /// `(pid, exe)` of every process this user can see.
    #[cfg(target_os = "linux")]
    fn all_processes() -> Vec<Proc> {
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|e| {
                let pid: u32 = e.file_name().to_str()?.parse().ok()?;
                let exe = std::fs::read_link(e.path().join("exe")).ok()?;
                // A replaced binary reads back with this suffix.
                let text = exe.to_string_lossy();
                let exe = PathBuf::from(text.trim_end_matches(" (deleted)"));
                Some(Proc { pid, exe })
            })
            .collect()
    }

    /// `ps` reports the command as started; good enough to see what runs
    /// from the install folder.
    #[cfg(not(target_os = "linux"))]
    fn all_processes() -> Vec<Proc> {
        let Ok(out) = Command::new("ps")
            .args(["-axww", "-o", "pid=,command="])
            .output()
        else {
            return Vec::new();
        };
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| {
                let l = l.trim_start();
                let (pid, command) = l.split_once(' ')?;
                Some(Proc {
                    pid: pid.parse().ok()?,
                    exe: PathBuf::from(command.trim_start()),
                })
            })
            .collect()
    }

    impl ProcessOps for RealProc {
        fn is_alive(&self, pid: u32) -> bool {
            discovery::pid_alive(pid)
        }

        fn in_dir(&self, dir: &Path, exclude: &[u32]) -> Vec<Proc> {
            all_processes()
                .into_iter()
                .filter(|p| !exclude.contains(&p.pid) && p.exe.starts_with(dir))
                .collect()
        }

        fn locking(&self, _files: &[PathBuf], _exclude: &[u32]) -> Vec<Proc> {
            Vec::new()
        }

        fn terminate(&self, pid: u32, force: bool) -> Result<(), String> {
            let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
            let pid = libc::pid_t::try_from(pid).map_err(|e| e.to_string())?;
            // SAFETY: sending a signal to a pid has no memory-safety effect.
            if unsafe { libc::kill(pid, signal) } == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error().to_string())
            }
        }

        fn run(&self, argv: &[String]) -> Result<i32, RunError> {
            run_inherit(argv)
        }

        fn run_elevated(&self, _argv: &[String]) -> Result<i32, RunError> {
            Err(RunError::Failed(
                "elevation prompts only exist on Windows".into(),
            ))
        }

        fn version_of(&self, exe: &Path) -> Option<String> {
            version_of(exe)
        }

        fn relaunch(&self, plan: &LaunchPlan) -> Result<(), String> {
            relaunch_now(plan)
        }
    }
}

#[cfg(unix)]
pub use unix::RealProc;

#[cfg(windows)]
pub use crate::real_windows::{RealProc, RealRegistry};

#[cfg(not(windows))]
pub struct RealRegistry;

#[cfg(not(windows))]
impl crate::ops::Registry for RealRegistry {
    fn reader(&self) -> &dyn gitpoulpi_update::channel::RegistryOps {
        &gitpoulpi_update::channel::NoRegistry
    }

    fn set_display_version(
        &self,
        _hive: gitpoulpi_update::channel::Hive,
        _version: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}
