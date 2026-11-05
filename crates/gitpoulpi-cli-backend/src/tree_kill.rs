//! Killing a canceled network command *and everything it started*.
//!
//! A network `git` spawns helpers (`git-remote-https`, `ssh`, the credential
//! manager, an `uploadpack` shell script and its `sleep`). Dropping the
//! future that awaits it must not leave any of them running, so every async
//! git command is spawned through [`spawn_guarded`]:
//!
//! - **Unix**: its own process group; the guard sends SIGTERM to the group
//!   (git removes its lock files on SIGTERM), then SIGKILL two seconds later.
//! - **Windows**: a Job Object with `KILL_ON_JOB_CLOSE`; the guard terminates
//!   the job. Trap: a grandchild started *before* the assignment (a few
//!   microseconds after the spawn) escapes the job; git starts its helpers
//!   later than that.
//!
//! The guard is **disarmed** when the command ran to its end: whatever git
//! left running on purpose (the fsmonitor daemon) is not ours to kill.

use std::process::Stdio;

/// Kills the process tree of a spawned command when dropped while armed.
pub(crate) struct TreeGuard {
    armed: bool,
    #[cfg(unix)]
    pgid: Option<i32>,
    #[cfg(windows)]
    job: Option<win::Job>,
}

/// Spawns `cmd` (stdin closed, stdout and stderr piped when `piped`) under a
/// [`TreeGuard`]. `kill_on_drop` is set too: it covers the window before the
/// guard exists and the child itself if the guard could not be built.
pub(crate) fn spawn_guarded(
    cmd: &mut tokio::process::Command,
) -> std::io::Result<(tokio::process::Child, TreeGuard)> {
    cmd.kill_on_drop(true);
    #[cfg(unix)]
    detach_from_terminal(cmd);
    #[cfg(windows)]
    {
        // CREATE_NEW_PROCESS_GROUP
        cmd.creation_flags(0x0000_0200);
    }
    let child = cmd.spawn()?;
    let guard = TreeGuard::attach(&child);
    Ok((child, guard))
}

/// Starts the child in a new session: it leads its own process group (the
/// tree kill signals that group) *and* has no controlling terminal. Without
/// that, an ssh host-key or passphrase prompt opens `/dev/tty`, the child is
/// in a background group of the daemon's terminal, gets SIGTTIN and stops
/// forever. With no terminal ssh fails at once ("Host key verification
/// failed", "Permission denied (publickey)").
#[cfg(unix)]
fn detach_from_terminal(cmd: &mut tokio::process::Command) {
    // SAFETY: `setsid` is async-signal-safe and touches no Rust state.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
}

/// The usual `.output()` stdio: nothing on stdin, stdout and stderr piped.
pub(crate) fn captured(cmd: &mut tokio::process::Command) {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
}

impl TreeGuard {
    fn attach(child: &tokio::process::Child) -> Self {
        let pid = child.id();
        #[cfg(unix)]
        {
            Self {
                armed: true,
                pgid: pid.map(|p| p as i32),
            }
        }
        #[cfg(windows)]
        {
            Self {
                armed: true,
                job: pid.and_then(win::Job::containing),
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = pid;
            Self { armed: true }
        }
    }

    /// The command ran to its end: leave whatever it left behind alone.
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }

    /// A guard for a process this code spawned itself (a `std` child, a
    /// script), which must already be its own process group on Unix
    /// (`CommandExt::process_group(0)`); on Windows the process joins a new
    /// Job Object here, so create it right after the spawn.
    #[cfg(feature = "rewrite-scripts")]
    pub(crate) fn for_pid(pid: u32) -> Self {
        #[cfg(unix)]
        {
            Self {
                armed: true,
                pgid: Some(pid as i32),
            }
        }
        #[cfg(windows)]
        {
            Self {
                armed: true,
                job: win::Job::containing(pid),
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = pid;
            Self { armed: true }
        }
    }

    /// Kills the whole tree now and for good: SIGKILL to the group (no grace
    /// period, unlike dropping the guard) or the Job Object terminated.
    #[cfg(feature = "rewrite-scripts")]
    pub(crate) fn kill_now(&mut self) {
        #[cfg(unix)]
        if let Some(pgid) = self.pgid.take() {
            // SAFETY: a plain signal to a process group we created.
            unsafe {
                libc::kill(-pgid, libc::SIGKILL);
            }
        }
        #[cfg(windows)]
        if let Some(job) = self.job.take() {
            job.close(true);
        }
        self.armed = false;
    }
}

impl Drop for TreeGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if self.armed {
            if let Some(pgid) = self.pgid {
                unix_kill_group(pgid);
            }
        }
        #[cfg(windows)]
        if let Some(job) = self.job.take() {
            job.close(self.armed);
        }
    }
}

#[cfg(unix)]
fn unix_kill_group(pgid: i32) {
    // SAFETY: plain signals to a process group we created.
    unsafe {
        libc::kill(-pgid, libc::SIGTERM);
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));
        // SAFETY: as above; ESRCH when the group is already gone.
        unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        }
    });
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };

    pub(super) struct Job(HANDLE);

    // SAFETY: a kernel handle, usable from any thread.
    unsafe impl Send for Job {}

    fn set_flags(job: HANDLE, flags: u32) -> bool {
        // SAFETY: a zeroed struct is the documented starting point; the size
        // passed is that of the struct passed.
        unsafe {
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = flags;
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) != 0
        }
    }

    impl Job {
        /// A job holding the process `pid` (and so its future children).
        pub(super) fn containing(pid: u32) -> Option<Job> {
            // SAFETY: handles are checked before use and closed on every path.
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                if !set_flags(job, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE) {
                    CloseHandle(job);
                    return None;
                }
                let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if process.is_null() {
                    CloseHandle(job);
                    return None;
                }
                let assigned = AssignProcessToJobObject(job, process) != 0;
                CloseHandle(process);
                if !assigned {
                    CloseHandle(job);
                    return None;
                }
                Some(Job(job))
            }
        }

        /// Ends the guard: `kill` terminates every process of the job, else
        /// the kill-on-close flag is cleared so the survivors live on.
        pub(super) fn close(self, kill: bool) {
            // SAFETY: our own handle, closed once.
            unsafe {
                if kill {
                    TerminateJobObject(self.0, 1);
                } else {
                    set_flags(self.0, 0);
                }
                CloseHandle(self.0);
            }
        }
    }
}
