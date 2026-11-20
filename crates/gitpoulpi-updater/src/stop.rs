//! Stopping GitPoulpi: the session daemons first (their frontends follow),
//! then whatever of this install is still running.
//!
//! Order matters. A daemon asked to stop for an update tells its TUIs and MCP
//! servers to exit, so asking first lets them save and leave on their own;
//! terminating comes only after the grace period. Windows cannot replace a
//! running exe, so a straggler fails the update (exit 5) rather than being
//! worked around.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use gitpoulpi_update::job::UpdateJob;

use crate::env::Env;
use crate::ops::{Proc, SessionDaemon, StopScope};

pub const DAEMON_GRACE: Duration = Duration::from_secs(5);
pub const EXIT_GRACE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(100);

/// Polls `done` until it is true or `limit` has passed; whether it ended true.
pub fn wait_until(env: &Env, limit: Duration, mut done: impl FnMut() -> bool) -> bool {
    let started = env.clock.now();
    let mut waited = Duration::ZERO;
    loop {
        if done() {
            return true;
        }
        if waited >= limit {
            return false;
        }
        env.clock.sleep(POLL);
        waited += POLL;
        // A clock that jumped (suspend) must not stretch the wait.
        if env.clock.now().saturating_sub(started) > limit.as_secs() + 1 {
            return done();
        }
    }
}

/// Asks the session daemons of this installation to stop for the update, then waits for them to
/// go. A daemon that does not go is dealt with by [`stop_processes`].
pub fn stop_daemons(env: &Env, job: &UpdateJob) -> Vec<SessionDaemon> {
    let scope = StopScope {
        config_dir: job.config_dir.clone(),
        env: job.environment.clone(),
        install_pids: env
            .proc
            .in_dir(&job.install_dir, &[])
            .into_iter()
            .map(|p| p.pid)
            .collect(),
    };
    let daemons = env.daemons.sessions(&scope);
    for d in &daemons {
        env.daemons.request_shutdown(d);
    }
    wait_until(env, DAEMON_GRACE, || {
        daemons.iter().all(|d| !env.daemons.is_up(d))
    });
    daemons
}

/// What still runs from the install folder (and, elevated, whatever holds its
/// files open, other users' processes included).
fn leftovers(
    env: &Env,
    job: &UpdateJob,
    files: &[PathBuf],
    exclude: &[u32],
    elevated: bool,
) -> Vec<Proc> {
    let mut found: BTreeMap<u32, Proc> = BTreeMap::new();
    for p in env.proc.in_dir(&job.install_dir, exclude) {
        found.insert(p.pid, p);
    }
    if elevated {
        for p in env.proc.locking(files, exclude) {
            found.entry(p.pid).or_insert(p);
        }
    }
    found.into_values().collect()
}

/// Gives the leftovers a grace period to exit, terminates what remains and
/// waits again. `Err` names what would not stop.
pub fn stop_processes(
    env: &Env,
    job: &UpdateJob,
    files: &[PathBuf],
    exclude: &[u32],
    elevated: bool,
) -> Result<(), String> {
    let left = || leftovers(env, job, files, exclude, elevated);
    if left().is_empty() {
        return Ok(());
    }
    if wait_until(env, EXIT_GRACE, || left().is_empty()) {
        return Ok(());
    }
    for p in left() {
        let _ = env.proc.terminate(p.pid, false);
    }
    if wait_until(env, EXIT_GRACE, || left().is_empty()) {
        return Ok(());
    }
    for p in left() {
        let _ = env.proc.terminate(p.pid, true);
    }
    if wait_until(env, EXIT_GRACE, || left().is_empty()) {
        return Ok(());
    }
    let names: Vec<String> = left()
        .iter()
        .map(|p| format!("{} (pid {})", p.exe.display(), p.pid))
        .collect();
    Err(format!("these would not stop: {}", names.join(", ")))
}
