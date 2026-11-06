use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Environment override for the whole config directory. Lets a test use an
/// isolated directory instead of the machine's real one (which would race
/// in parallel or stomp on the user's live sessions); also the escape
/// hatch for a portable install.
pub const CONFIG_DIR_ENV: &str = "GITPOULPI_CONFIG_DIR";

/// OS-correct locations: %APPDATA%\GitPoulpi on Windows, XDG ~/.config/gitpoulpi
/// on Linux, ~/Library/Application Support/GitPoulpi on macOS.
#[derive(Clone)]
pub struct ConfigPaths {
    config_dir: PathBuf,
    /// `directories::ProjectDirs::cache_dir()` - `%LOCALAPPDATA%\GitPoulpi`
    /// on Windows (not roamed, correct for something re-fetchable),
    /// `~/.cache/gitpoulpi` on Linux, `~/Library/Caches/GitPoulpi` on
    /// macOS. `None` under [`CONFIG_DIR_ENV`], where it falls back to a
    /// `cache` subdirectory of the overridden config dir so a test gets an
    /// isolated cache too.
    cache_dir: Option<PathBuf>,
    /// `directories::ProjectDirs::state_dir()` (XDG state home on Linux),
    /// else `None`. Under [`CONFIG_DIR_ENV`] and [`ConfigPaths::for_dir`] it
    /// is `None` too, so a test's log files land in its own directory.
    state_dir: Option<PathBuf>,
    /// `directories::ProjectDirs::data_dir()`: the roaming
    /// `%APPDATA%\GitPoulpi\data` on Windows, the XDG data home on Linux,
    /// `~/Library/Application Support/GitPoulpi` on macOS.
    /// `None` under [`CONFIG_DIR_ENV`] / [`ConfigPaths::for_dir`] (a
    /// `data` subdirectory of the config dir is used instead).
    data_dir: Option<PathBuf>,
}

impl ConfigPaths {
    /// Builds a `ConfigPaths` rooted at an arbitrary directory, without
    /// touching [`CONFIG_DIR_ENV`]: a test must never set that
    /// process-wide variable (it would race daemon-lifecycle tests in the
    /// same binary), so those use this with a tempdir.
    pub fn for_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: dir.into(),
            cache_dir: None,
            state_dir: None,
            data_dir: None,
        }
    }

    pub fn discover() -> Option<Self> {
        if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
            let dir = PathBuf::from(dir);
            if !dir.as_os_str().is_empty() {
                return Some(Self {
                    config_dir: dir,
                    cache_dir: None,
                    state_dir: None,
                    data_dir: None,
                });
            }
        }
        ProjectDirs::from("", "", "GitPoulpi").map(|dirs| Self {
            config_dir: dirs.config_dir().to_path_buf(),
            cache_dir: Some(dirs.cache_dir().to_path_buf()),
            state_dir: dirs.state_dir().map(|d| d.to_path_buf()),
            data_dir: Some(dirs.data_dir().to_path_buf()),
        })
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// The OS-correct cache directory, or `config_dir/cache` when
    /// [`CONFIG_DIR_ENV`] overrode discovery (see the field doc). Nothing
    /// here is precious data - deleting it just means a re-fetch.
    pub fn cache_dir(&self) -> PathBuf {
        match &self.cache_dir {
            Some(dir) => dir.clone(),
            None => self.config_dir.join("cache"),
        }
    }

    /// Where the application log files live: `<state>/logs` where the OS has
    /// a state directory (Linux), else `<cache>/logs`; always `<config>/logs`
    /// under [`CONFIG_DIR_ENV`].
    pub fn log_dir(&self) -> PathBuf {
        match (&self.state_dir, &self.cache_dir) {
            (Some(state), _) => state.join("logs"),
            (None, _) => self.cache_dir().join("logs"),
        }
    }

    /// The OS-correct user data directory (see the field doc); things the
    /// user would miss if they were deleted, unlike [`Self::cache_dir`].
    pub fn data_dir(&self) -> PathBuf {
        match &self.data_dir {
            Some(dir) => dir.clone(),
            None => self.config_dir.join("data"),
        }
    }

    /// Where `per_repo_state_dir = "user_data"` keeps each repository's
    /// folder: `<data>/repos`.
    pub fn repos_data_dir(&self) -> PathBuf {
        self.data_dir().join("repos")
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.config_dir.join("sessions")
    }

    /// Where user theme files live: `<config>/themes/*.toml`.
    pub fn themes_dir(&self) -> PathBuf {
        self.config_dir.join("themes")
    }

    pub fn session_file(&self, session_name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_name}.toml"))
    }

    pub fn ipc_socket_name(&self, session_name: &str) -> String {
        format!("gitpoulpi-{session_name}")
    }

    /// Where the daemon for `session_name` publishes the loopback TCP port
    /// it bound (see `gitpoulpi-ipc`'s transport doc for why TCP plus a
    /// discovery file rather than a named pipe/Unix socket). The daemon
    /// writes it on start and best-effort removes it on clean shutdown.
    ///
    /// `session_name` is expected to have been through
    /// `crate::sanitize_session_name` already, since it becomes a filename.
    pub fn ipc_port_file(&self, session_name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_name}.port"))
    }

    /// Where a *standalone* daemon (`gitpoulpi daemon run`, including the
    /// detached process `gitpoulpi daemon start` spawns) records its own
    /// process id, alongside its `.port` file. `daemon stop`'s fallback,
    /// used only when the graceful `Shutdown` request does not answer in
    /// time, reads it to find a PID it is safe to kill.
    ///
    /// An *embedded* daemon (inside a TUI's or CLI command's own process)
    /// never writes it, so the fallback can never target a host process
    /// (see `gitpoulpi_ipc::DaemonKind`).
    pub fn ipc_pid_file(&self, session_name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_name}.pid"))
    }

    /// Where secrets that must stay user-only live (the integration trust
    /// store, MCP secrets, update keys): `<data>/private`, created `0700` by
    /// [`crate::secure_fs::harden_app_dirs`].
    pub fn private_dir(&self) -> PathBuf {
        self.data_dir().join("private")
    }

    /// Where the daemon for `session_name` publishes the token a client must
    /// present first (`Hello`). Written user-only before the `.port` file.
    pub fn ipc_token_file(&self, session_name: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_name}.token"))
    }

    /// Where the updater and the TUI hand an update over (the install-channels
    /// design): `<data>/updates`.
    pub fn updates_dir(&self) -> PathBuf {
        self.data_dir().join("updates")
    }

    /// The job the TUI writes and `gitpoulpi-updater run --job` reads.
    pub fn update_job_file(&self) -> PathBuf {
        self.updates_dir().join("job.json")
    }

    /// What the updater last did; the TUI reads it on its next start.
    pub fn update_status_file(&self) -> PathBuf {
        self.updates_dir().join("status.json")
    }

    /// Present while an update runs; the CLI and every daemon auto-start
    /// refuse to start a daemon then (see [`crate::update_lock`]).
    pub fn update_lock_file(&self) -> PathBuf {
        self.updates_dir().join("updating.lock")
    }

    /// The copy of the updater the TUI runs from, so the install folder's
    /// own copy is never locked and a package manager can overwrite it.
    pub fn update_bin_dir(&self) -> PathBuf {
        self.updates_dir().join("bin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Puts `CONFIG_DIR_ENV` back as it was, even when the test fails.
    struct Restore(Option<std::ffi::OsString>);

    impl Drop for Restore {
        fn drop(&mut self) {
            match self.0.take() {
                Some(v) => std::env::set_var(CONFIG_DIR_ENV, v),
                None => std::env::remove_var(CONFIG_DIR_ENV),
            }
        }
    }

    /// The one test in this crate that mutates process env (under
    /// `ENV_LOCK`, restoring what it found).
    #[test]
    fn the_env_override_relocates_every_derived_path() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let _restore = Restore(std::env::var_os(CONFIG_DIR_ENV));
        std::env::set_var(CONFIG_DIR_ENV, dir.path());
        let paths = ConfigPaths::discover().unwrap();
        assert_eq!(paths.config_dir(), dir.path());
        assert_eq!(paths.config_file(), dir.path().join("config.toml"));
        assert_eq!(
            paths.ipc_port_file("work"),
            dir.path().join("sessions").join("work.port")
        );
        assert_eq!(
            paths.ipc_pid_file("work"),
            dir.path().join("sessions").join("work.pid")
        );
        assert_eq!(
            paths.update_lock_file(),
            dir.path()
                .join("data")
                .join("updates")
                .join("updating.lock")
        );
        assert_eq!(paths.update_job_file().file_name().unwrap(), "job.json");
        assert_eq!(
            paths.update_status_file().file_name().unwrap(),
            "status.json"
        );
        assert_eq!(paths.update_bin_dir(), dir.path().join("data/updates/bin"));
        assert_eq!(paths.cache_dir(), dir.path().join("cache"));
        assert_eq!(paths.log_dir(), dir.path().join("cache").join("logs"));
        assert!(ConfigPaths::for_dir(dir.path())
            .log_dir()
            .starts_with(dir.path()));
    }
}
