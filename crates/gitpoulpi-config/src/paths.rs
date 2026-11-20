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
    /// The environment (the installations design, 1.5): `Some` selects the
    /// environment layout, where everything a daemon or a window coordinates
    /// through lives under `sessions/env/<env>/`. `None` is the flat legacy
    /// layout (an explicit [`CONFIG_DIR_ENV`], plain [`ConfigPaths::for_dir`]).
    env: Option<String>,
    /// The installation id, which names this installation's `updates/` folder.
    /// `None` keeps the flat `<data>/updates`.
    install_id: Option<String>,
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
            env: None,
            install_id: None,
        }
    }

    /// The same paths in the environment layout of `env` (sanitised, so it is
    /// always a safe folder name; an empty result keeps the flat layout).
    /// Tests use this instead of setting `GITPOULPI_ENV` process-wide.
    pub fn with_env(mut self, env: &str) -> Self {
        self.env = Some(crate::installation::sanitize(env)).filter(|e| !e.is_empty());
        self
    }

    /// The same paths with `id` as the installation whose `updates/` folder
    /// [`Self::updates_dir`] returns.
    pub fn with_install_id(mut self, id: &str) -> Self {
        self.install_id = Some(crate::installation::sanitize(id)).filter(|e| !e.is_empty());
        self
    }

    /// The environment of the layout, `None` in the flat legacy layout.
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref()
    }

    /// The installation id naming `updates/`, `None` for the flat folder.
    pub fn install_id(&self) -> Option<&str> {
        self.install_id.as_deref()
    }

    /// The paths of the running installation.
    ///
    /// * [`CONFIG_DIR_ENV`] set: the flat legacy layout (an explicit config
    ///   folder is already its own world), unless `GITPOULPI_ENV` forces the
    ///   environment layout.
    /// * Otherwise: the OS folders in the environment layout of this
    ///   installation (`GITPOULPI_ENV`, else the `installation.settings.<id>.env`
    ///   entry, else the installation id; see `installation::environment`).
    pub fn discover() -> Option<Self> {
        Self::discover_with(&|k| std::env::var_os(k), &real_installation)
    }

    /// [`Self::discover`] with the process environment and the installation
    /// lookup (`config file -> (installation id, environment)`) injected.
    fn discover_with(
        var: &dyn Fn(&str) -> Option<std::ffi::OsString>,
        resolve: &dyn Fn(&Path) -> (String, String),
    ) -> Option<Self> {
        let forced = var(crate::installation::ENV_OVERRIDE_ENV)
            .map(|v| crate::installation::sanitize(&v.to_string_lossy()))
            .filter(|e| !e.is_empty());
        if let Some(dir) = var(CONFIG_DIR_ENV) {
            let dir = PathBuf::from(dir);
            if !dir.as_os_str().is_empty() {
                return Some(Self {
                    env: forced,
                    ..Self::for_dir(dir)
                });
            }
        }
        ProjectDirs::from("", "", "GitPoulpi").map(|dirs| {
            let config_dir = dirs.config_dir().to_path_buf();
            let (install_id, env) = resolve(&config_dir.join("config.toml"));
            Self {
                config_dir,
                cache_dir: Some(dirs.cache_dir().to_path_buf()),
                state_dir: dirs.state_dir().map(|d| d.to_path_buf()),
                data_dir: Some(dirs.data_dir().to_path_buf()),
                env: Some(env).filter(|e| !e.is_empty()),
                install_id: Some(install_id).filter(|e| !e.is_empty()),
            }
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

    /// Where a daemon's discovery files and the tab lists live: the one seam
    /// every consumer goes through. `<config>/sessions` in the flat layout,
    /// `<config>/sessions/env/<env>` in the environment layout.
    pub fn sessions_dir(&self) -> PathBuf {
        let flat = self.legacy_sessions_dir();
        match &self.env {
            Some(env) => flat.join("env").join(env),
            None => flat,
        }
    }

    /// The flat `<config>/sessions`, where builds without environments keep
    /// their discovery files and tab lists.
    pub fn legacy_sessions_dir(&self) -> PathBuf {
        self.config_dir.join("sessions")
    }

    /// The one-line file naming the session a launch restores the tabs of:
    /// beside the tab lists in the environment layout, `<config>/last-session`
    /// in the flat one.
    pub fn last_session_file(&self) -> PathBuf {
        match self.env {
            Some(_) => self.sessions_dir().join("last-session"),
            None => self.config_dir.join("last-session"),
        }
    }

    /// The first use of an environment COPIES the flat layout's tab lists
    /// (`sessions/*.toml`) and `last-session` into it, never moves them, so old
    /// builds keep theirs. Done once (a marker file records it); a file the
    /// environment already has is never overwritten. No-op in the flat layout.
    /// Best effort: a failure only means empty tab lists.
    pub fn seed_env_from_legacy(&self) {
        if self.env.is_none() {
            return;
        }
        let dir = self.sessions_dir();
        let marker = dir.join(".seeded");
        if marker.exists() || std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let copy = |from: PathBuf, to: PathBuf| {
            if from.is_file() && !to.exists() {
                let _ = std::fs::copy(from, to);
            }
        };
        if let Ok(entries) = std::fs::read_dir(self.legacy_sessions_dir()) {
            for path in entries.flatten().map(|e| e.path()) {
                if path.extension().is_some_and(|e| e == "toml") {
                    if let Some(name) = path.file_name() {
                        let to = dir.join(name);
                        copy(path, to);
                    }
                }
            }
        }
        copy(
            self.config_dir.join("last-session"),
            self.last_session_file(),
        );
        let _ = std::fs::write(marker, "");
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
    /// design): `<data>/updates/<install-id>`, so one installation's update
    /// never blocks another's. The flat `<data>/updates` without an id.
    pub fn updates_dir(&self) -> PathBuf {
        let dir = self.data_dir().join("updates");
        match &self.install_id {
            Some(id) => dir.join(id),
            None => dir,
        }
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

/// The running installation's `(id, environment)`. The identity is a property
/// of the process, so it is worked out once (a changed `env` setting applies
/// at the next start, as the Settings row says).
fn real_installation(config_file: &Path) -> (String, String) {
    use crate::installation::{environment, identify, IdentityEnv, RealFs};
    static RESOLVED: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    RESOLVED
        .get_or_init(|| {
            let fs = RealFs;
            let env = IdentityEnv::real(&fs);
            let exe = std::env::current_exe().unwrap_or_default();
            let identity = identify(&exe, &env);
            let config = crate::Config::load(config_file).unwrap_or_default();
            let name = environment(&config, &identity, &env);
            (identity.id, name)
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn vars<'a>(
        pairs: &'a [(&'a str, &'a str)],
    ) -> impl Fn(&str) -> Option<std::ffi::OsString> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(name, _)| *name == k)
                .map(|(_, v)| std::ffi::OsString::from(v))
        }
    }

    fn resolved(_: &Path) -> (String, String) {
        ("abc123".to_string(), "abc123".to_string())
    }

    #[test]
    fn the_discovered_layout_puts_discovery_and_tabs_under_the_env() {
        let paths = ConfigPaths::discover_with(&vars(&[]), &resolved).unwrap();
        let sessions = paths
            .config_dir()
            .join("sessions")
            .join("env")
            .join("abc123");
        assert_eq!(paths.env(), Some("abc123"));
        assert_eq!(paths.sessions_dir(), sessions);
        assert_eq!(paths.ipc_port_file("w"), sessions.join("w.port"));
        assert_eq!(paths.ipc_token_file("w"), sessions.join("w.token"));
        assert_eq!(paths.ipc_pid_file("w"), sessions.join("w.pid"));
        assert_eq!(paths.session_file("w"), sessions.join("w.toml"));
        assert_eq!(paths.last_session_file(), sessions.join("last-session"));
        assert_eq!(
            paths.updates_dir(),
            paths.data_dir().join("updates").join("abc123")
        );
    }

    #[test]
    fn an_explicit_config_dir_keeps_the_flat_layout() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().to_str().unwrap();
        let paths = ConfigPaths::discover_with(&vars(&[(CONFIG_DIR_ENV, cfg)]), &resolved).unwrap();
        assert_eq!(paths.env(), None);
        assert_eq!(paths.sessions_dir(), dir.path().join("sessions"));
        assert_eq!(paths.last_session_file(), dir.path().join("last-session"));
        assert_eq!(paths.updates_dir(), dir.path().join("data").join("updates"));
    }

    #[test]
    fn gitpoulpi_env_forces_the_env_layout() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().to_str().unwrap();
        let paths = ConfigPaths::discover_with(
            &vars(&[(CONFIG_DIR_ENV, cfg), ("GITPOULPI_ENV", "Beta 2")]),
            &resolved,
        )
        .unwrap();
        assert_eq!(paths.env(), Some("beta-2"));
        assert_eq!(
            paths.sessions_dir(),
            dir.path().join("sessions").join("env").join("beta-2")
        );
        assert_eq!(
            ConfigPaths::for_dir(dir.path())
                .with_env("x")
                .sessions_dir(),
            dir.path().join("sessions").join("env").join("x")
        );
    }

    #[test]
    fn first_use_of_an_env_copies_legacy_tab_lists_and_leaves_them() {
        let dir = tempfile::tempdir().unwrap();
        let flat = ConfigPaths::for_dir(dir.path());
        std::fs::create_dir_all(flat.sessions_dir()).unwrap();
        std::fs::write(flat.session_file("work"), "legacy").unwrap();
        std::fs::write(flat.sessions_dir().join("work.port"), "1234").unwrap();
        std::fs::write(dir.path().join("last-session"), "work").unwrap();

        let env = ConfigPaths::for_dir(dir.path()).with_env("a");
        env.seed_env_from_legacy();
        assert_eq!(
            std::fs::read_to_string(env.session_file("work")).unwrap(),
            "legacy"
        );
        assert_eq!(
            std::fs::read_to_string(env.last_session_file()).unwrap(),
            "work"
        );
        assert!(
            !env.ipc_port_file("work").exists(),
            "discovery files are not tab lists"
        );
        assert!(flat.session_file("work").exists());
        assert!(dir.path().join("last-session").exists());

        // Once: later edits of either side are not merged again.
        std::fs::write(env.session_file("work"), "mine").unwrap();
        std::fs::write(flat.session_file("other"), "late").unwrap();
        env.seed_env_from_legacy();
        assert_eq!(
            std::fs::read_to_string(env.session_file("work")).unwrap(),
            "mine"
        );
        assert!(!env.session_file("other").exists());
        // The flat layout seeds nothing.
        flat.seed_env_from_legacy();
    }

    #[test]
    fn one_installations_update_lock_does_not_block_another() {
        let dir = tempfile::tempdir().unwrap();
        let a = ConfigPaths::for_dir(dir.path()).with_install_id("aaaa");
        let b = ConfigPaths::for_dir(dir.path()).with_install_id("bbbb");
        assert_ne!(a.updates_dir(), b.updates_dir());
        std::fs::create_dir_all(a.updates_dir()).unwrap();
        std::fs::write(a.update_lock_file(), crate::update_lock::render(1, 90)).unwrap();
        assert!(crate::update_lock::update_in_progress(&a, None, 100));
        assert!(!crate::update_lock::update_in_progress(&b, None, 100));
    }

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
