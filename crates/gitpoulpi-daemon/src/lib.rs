mod auto_resolve;
mod autostash;
mod autostash_start_over;
mod backend;
mod cache;
pub mod conflict_prevention;
mod credentials;
mod dispatch;
mod encoding;
mod flow;
mod jobs;
pub mod join;
mod journal;
mod journal_snap;
mod journal_stashes_remotes;
mod journal_store;
#[cfg(feature = "lfs")]
mod lfs_preflight;
mod mailmap;
#[cfg(test)]
mod op_classification;
mod opstate;
mod ref_visibility;
mod repo_auth;
mod resolved_record;
mod rewrite_runs;
mod search;
mod server;
mod stage_guard;
mod status;
mod submodules;
mod test_hook;
mod undo_settings;
mod viewers;
mod workflow;

pub use backend::{backend_for, backend_from_ambient_config};
pub use gitpoulpi_ipc::{DaemonInfo, DaemonKind};
pub use status::{DaemonStatus, SessionDaemonInfo};
pub use test_hook::{panic_on_for_tests, Hold};

/// Forgets every in-memory rewrite plan, as a daemon restart does (the
/// plan store is process-wide, so tests that restart a daemon in one process
/// call this).
#[doc(hidden)]
pub fn forget_rewrite_plans_for_tests() {
    rewrite_runs::forget_all();
}

/// Writes the undo journal bytes still waiting behind a staging
/// operation (see `journal_store`). Shutdown does it; tests that read
/// `journal.json` call it.
pub fn flush_undo_journal() {
    journal::flush_pending();
}

use std::sync::Arc;

use gitpoulpi_core::GitBackend;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("session daemon '{0}' is already running")]
    AlreadyRunning(String),
    #[error("session daemon '{0}' is not running")]
    NotRunning(String),
    #[error("ipc error: {0}")]
    Ipc(#[from] gitpoulpi_ipc::TransportError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not determine the config directory for this OS")]
    NoConfigDir,
    #[error("daemon failed to start: {0}")]
    StartFailed(String),
}

/// A running daemon's IPC endpoint plus the means to shut it down.
/// Returned by `Daemon::spawn` for both production use (`run_foreground`
/// wraps it with port-file publishing and a ctrl-c-triggered shutdown) and
/// tests (which want direct control over the port and shutdown timing
/// without touching the filesystem).
pub struct DaemonHandle {
    pub port: u16,
    token: gitpoulpi_ipc::Token,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    join: tokio::task::JoinHandle<Result<()>>,
}

impl DaemonHandle {
    /// The token a client must present first (see `gitpoulpi_ipc::auth`). An
    /// in-process daemon hands it to its own clients through memory.
    pub fn token(&self) -> &gitpoulpi_ipc::Token {
        &self.token
    }

    /// Port and token, what `Client::connect` takes.
    pub fn endpoint(&self) -> gitpoulpi_ipc::Endpoint {
        gitpoulpi_ipc::Endpoint::new(self.port, Some(self.token.clone()))
    }

    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.join.await;
        Ok(())
    }
}

/// Owns everything for one named session: the `GitBackend` every connected
/// client's requests are dispatched against, and the IPC endpoint
/// (loopback TCP; see `gitpoulpi_ipc::transport` for why not a named
/// pipe/socket) those clients connect to.
///
/// One `GitBackend` instance is shared across all repos a client asks
/// about: the `RepoId` on each `Request` *is* the canonicalized path and is
/// resolved straight to a `RepoRef` (see `dispatch::repo_ref`), so every
/// request opens the repo at that path fresh.
pub struct Daemon {
    session_name: String,
    /// What `Hello` reports as the build sha instead of this binary's own.
    /// Only for tests that need a daemon "from another build"; production
    /// code never sets it.
    git_sha_override: Option<String>,
}

impl Daemon {
    pub fn new(session_name: impl Into<String>) -> Self {
        Self {
            session_name: session_name.into(),
            git_sha_override: None,
        }
    }

    /// Makes `Hello` report `sha` as this daemon's build (a fixture for a
    /// daemon started by another build; see the TUI's takeover tests).
    #[must_use]
    pub fn with_git_sha(mut self, sha: impl Into<String>) -> Self {
        self.git_sha_override = Some(sha.into());
        self
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    /// Binds an ephemeral loopback port and starts serving immediately,
    /// without touching the filesystem. Use this directly in tests; use
    /// `run_foreground` for the real CLI-facing lifecycle.
    ///
    /// Reports itself as `DaemonKind::Embedded`: the PID it reports is the
    /// embedding client's, so `daemon stop`'s PID-kill fallback must never
    /// target it (see `gitpoulpi_ipc::DaemonKind`).
    pub async fn spawn(&self, backend: Arc<dyn GitBackend>) -> Result<DaemonHandle> {
        let info = self.info(backend.as_ref(), DaemonKind::Embedded);
        let token = new_token()?;
        let (port_tx, port_rx) = tokio::sync::oneshot::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let join = tokio::spawn(server::run(
            backend,
            port_tx,
            shutdown_rx,
            info,
            token.clone(),
        ));
        let port = port_rx
            .await
            .map_err(|_| Error::StartFailed("daemon task exited before binding".into()))?;
        Ok(DaemonHandle {
            port,
            token,
            shutdown_tx: Some(shutdown_tx),
            join,
        })
    }

    /// Publishes the bound port (and, being a dedicated process, the PID --
    /// see `ConfigPaths::ipc_pid_file`) to this session's discovery files,
    /// then blocks until Ctrl-C or a client's `OperationKind::Shutdown`
    /// ends the accept loop, and removes both files. This is what
    /// `gitpoulpi daemon run --session <name>` calls, directly for the
    /// foreground case and via a detached child for `daemon start`.
    pub async fn run_foreground(&self, backend: Arc<dyn GitBackend>) -> Result<()> {
        let paths = gitpoulpi_config::ConfigPaths::discover().ok_or(Error::NoConfigDir)?;
        gitpoulpi_config::secure_fs::harden_app_dirs(&paths);
        self.run_foreground_in(backend, &paths).await
    }

    /// `run_foreground`, against an explicit [`gitpoulpi_config::ConfigPaths`]
    /// rather than the ambient one `discover()` resolves. Tests use it
    /// (`ConfigPaths::for_dir` over a tempdir) so they never set
    /// `GITPOULPI_CONFIG_DIR`, which would race unrelated tests in the
    /// same binary.
    pub async fn run_foreground_in(
        &self,
        backend: Arc<dyn GitBackend>,
        paths: &gitpoulpi_config::ConfigPaths,
    ) -> Result<()> {
        let port_file = paths.ipc_port_file(&self.session_name);
        let pid_file = paths.ipc_pid_file(&self.session_name);
        if port_file.exists() {
            return Err(Error::AlreadyRunning(self.session_name.clone()));
        }

        let token = new_token()?;
        let info = self.info(backend.as_ref(), DaemonKind::Standalone);
        let (port_tx, port_rx) = tokio::sync::oneshot::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let mut join = tokio::spawn(server::run(
            backend,
            port_tx,
            shutdown_rx,
            info,
            token.clone(),
        ));
        let port = port_rx
            .await
            .map_err(|_| Error::StartFailed("daemon task exited before binding".into()))?;

        // The token file lands user-only before the port file: the port file
        // appearing means "ready".
        gitpoulpi_ipc::discovery::publish(
            &port_file,
            &gitpoulpi_ipc::Endpoint::new(port, Some(token)),
        )?;
        gitpoulpi_config::secure_fs::write_private(
            &pid_file,
            std::process::id().to_string().as_bytes(),
        )?;
        tracing::info!(
            session = %self.session_name,
            port,
            pid = std::process::id(),
            "gitpoulpi daemon listening"
        );

        // Races Ctrl-C against a client asking for shutdown over IPC
        // (which ends `server::run` without firing `shutdown_tx`). Either
        // way both discovery files come down together, so a stale one never
        // outlives the process that wrote it.
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                let _ = shutdown_tx.send(());
                let _ = (&mut join).await;
            }
            result = &mut join => {
                gitpoulpi_ipc::discovery::remove_files(&port_file);
                return result.map_err(|e| Error::StartFailed(e.to_string()))?;
            }
        }
        gitpoulpi_ipc::discovery::remove_files(&port_file);
        Ok(())
    }

    fn info(&self, backend: &dyn GitBackend, kind: DaemonKind) -> DaemonInfo {
        let backend_name = backend.name();
        DaemonInfo {
            session: self.session_name.clone(),
            pid: std::process::id(),
            kind,
            started_at: gitpoulpi_core::time::unix_now(),
            backend: backend_name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_sha: self.git_sha_override.clone().or_else(git_sha_best_effort),
        }
    }
}

fn new_token() -> Result<gitpoulpi_ipc::Token> {
    gitpoulpi_ipc::Token::generate()
        .map_err(|e| Error::StartFailed(format!("no random token: {e}")))
}

/// The short commit sha this daemon was built from (baked in at build
/// time, no spawn; `None` outside a git checkout, which clients treat as
/// matching).
fn git_sha_best_effort() -> Option<String> {
    gitpoulpi_ipc::build_git_sha().map(str::to_string)
}
