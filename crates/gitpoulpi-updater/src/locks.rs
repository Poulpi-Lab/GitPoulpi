//! The two lock files that say "an update is running" (the CLI and the
//! daemon auto-start look for them, see `gitpoulpi_config::update_lock`).

use std::path::{Path, PathBuf};

use gitpoulpi_config::update_lock;

use crate::env::Env;

pub struct Held {
    data: PathBuf,
    install: Option<PathBuf>,
}

/// Takes `<updates_dir>/updating.lock`, and `<install_dir>/.gitpoulpi-updating`
/// when that folder can be written. A lock that is still live and belongs to
/// another process is an update already running; a stale one is replaced.
pub fn acquire(env: &Env, updates_dir: &Path, install_dir: &Path) -> Result<Held, String> {
    let now = env.clock.now();
    let data = updates_dir.join("updating.lock");
    if let Ok(text) = env.fs.read_to_string(&data) {
        let ours = update_lock::parse(&text).is_some_and(|(pid, _)| pid == env.own_pid);
        if update_lock::is_live(&text, now) && !ours {
            return Err("another update is already running".into());
        }
    }
    let body = update_lock::render(env.own_pid, now);
    env.fs
        .write(&data, body.as_bytes(), false)
        .map_err(|e| format!("could not take the update lock: {e}"))?;
    let install = install_dir.join(update_lock::INSTALL_LOCK_FILE);
    // Best effort: Program Files is not writable for the coordinator.
    let install = env
        .fs
        .write(&install, body.as_bytes(), false)
        .is_ok()
        .then_some(install);
    Ok(Held { data, install })
}

impl Held {
    pub fn release(&self, env: &Env) {
        let _ = env.fs.remove_file(&self.data);
        if let Some(p) = &self.install {
            let _ = env.fs.remove_file(p);
        }
    }
}

/// The elevated worker's own marker in the install folder: it never takes the
/// data lock (the coordinator holds it).
pub fn install_lock(env: &Env, install_dir: &Path) -> Option<PathBuf> {
    let path = install_dir.join(update_lock::INSTALL_LOCK_FILE);
    let body = update_lock::render(env.own_pid, env.clock.now());
    env.fs
        .write(&path, body.as_bytes(), false)
        .is_ok()
        .then_some(path)
}
