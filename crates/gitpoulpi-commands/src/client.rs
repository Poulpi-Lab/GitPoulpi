//! The CLI's side of the IPC link: find (or start) this session's daemon,
//! then send it one `GitBackend`-mirroring request per subcommand.
//!
//! Architecture principle 3: the CLI never touches a `GitBackend` itself,
//! so a `gitpoulpi commit` in a terminal and a commit in an open TUI go
//! through the same daemon and repo lock, and the TUI redraws without
//! being told.

use std::path::{Path, PathBuf};

use gitpoulpi_config::DefaultBackend;
use gitpoulpi_core::domain::{OperationKind, RepoId, RepoRef};
use gitpoulpi_ipc::ui::UiOrigin;
use gitpoulpi_ipc::{Client, Request};
use serde::de::DeserializeOwned;

/// The details of the last few failed calls, by the message they came with.
/// `call` flattens an error to its message (`Result<_, String>` is what every
/// command returns), so the facts travel on the side: the CLI and the MCP
/// server look them up with [`error_detail_for`] when they report a failure.
static DETAILS: std::sync::Mutex<Vec<(String, gitpoulpi_core::ErrorDetail)>> =
    std::sync::Mutex::new(Vec::new());

fn remember_detail(message: &str, detail: gitpoulpi_core::ErrorDetail) {
    let mut all = DETAILS.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|(m, _)| m != message);
    all.push((message.to_string(), detail));
    if all.len() > 16 {
        all.remove(0);
    }
}

/// The facts of a failure whose message is `message` (or which `message`
/// wraps), when the daemon sent any.
pub fn error_detail_for(message: &str) -> Option<gitpoulpi_core::ErrorDetail> {
    let all = DETAILS.lock().unwrap_or_else(|e| e.into_inner());
    all.iter()
        .rev()
        .find(|(m, _)| message.contains(m.as_str()))
        .map(|(_, d)| d.clone())
}

/// A connected client plus the repo every request targets.
///
/// `repo` is `None` only outside a repository, which is legitimate for
/// `clone` (it *creates* a repo); every other command turns that into an
/// error rather than acting on the wrong repo.
pub struct Session {
    client: Client,
    session: String,
    repo: Option<RepoRef>,
    next_id: u64,
    /// Who the view intents (`ui show ...`) sent through this session come
    /// from, and how far that caller lets them move the user's window.
    ui_origin: UiOrigin,
    /// This process is short-lived (the CLI): the embedded daemon dies with
    /// it, so the undo journal's write-behind must be on disk before the
    /// command returns (see `send`).
    flush_journal: bool,
    /// Network requests carry `cancel_on_disconnect`: the daemon cancels the
    /// job they started when this connection drops (Ctrl+C on the CLI).
    cancel_on_disconnect: bool,
    /// A person is at the keyboard of this process (the CLI): it may trust an
    /// external history rewrite script. Off for everything else (the MCP
    /// server), so an agent cannot grant trust.
    may_trust: bool,
}

impl Session {
    /// `publish` controls whether an embedded daemon this call has to
    /// start publishes its port to the session's discovery file (see
    /// `ensure_daemon`). The CLI passes `false` (one short-lived process
    /// per command); a long-lived frontend such as the MCP server passes
    /// `true`, so an open TUI on the same session joins the daemon it
    /// started, and vice versa.
    pub async fn connect(
        session: &str,
        repo: Option<RepoRef>,
        backend: DefaultBackend,
        publish: bool,
    ) -> Result<Self, String> {
        let port = ensure_daemon(session, backend, publish).await?;
        let client = Client::connect_port(port)
            .await
            .map_err(|e| format!("could not reach this session's daemon: {e}"))?;
        Ok(Self {
            client,
            session: session.to_string(),
            repo,
            next_id: 1,
            ui_origin: UiOrigin::cli(),
            flush_journal: !publish,
            cancel_on_disconnect: false,
            may_trust: false,
        })
    }

    /// Connects to a daemon already listening on `port` (found through a
    /// `sessions/*.port` file) without ever starting one: the cross-session
    /// routing of `ui show`.
    pub async fn connect_port(
        session: &str,
        port: u16,
        repo: Option<RepoRef>,
    ) -> Result<Self, String> {
        let client = Client::connect_port(port)
            .await
            .map_err(|e| format!("could not reach session `{session}`'s daemon: {e}"))?;
        Ok(Self {
            client,
            session: session.to_string(),
            repo,
            next_id: 1,
            ui_origin: UiOrigin::cli(),
            flush_journal: false,
            cancel_on_disconnect: false,
            may_trust: false,
        })
    }

    /// Ask the daemon to cancel the network jobs this connection starts if
    /// the connection drops before they end. The CLI sets it (Ctrl+C stops
    /// what the command started); the TUI and the MCP server never do.
    pub fn set_cancel_on_disconnect(&mut self, on: bool) {
        self.cancel_on_disconnect = on;
    }

    /// The `gitpoulpi` binary marks its own session as a person's: only then
    /// can `rewrite script trust` and `--trust` remember a script as trusted.
    pub fn set_may_trust(&mut self, on: bool) {
        self.may_trust = on;
    }

    /// Whether a person may trust scripts through this session.
    pub fn may_trust(&self) -> bool {
        self.may_trust
    }

    /// The session this client talks to.
    pub fn session_name(&self) -> &str {
        &self.session
    }

    /// Retargets this already-connected session at another repository (or
    /// none), so a long-lived caller - the MCP server - can serve many
    /// tool calls, each possibly against a different repository, over one
    /// daemon connection instead of reconnecting per call.
    pub fn set_repo(&mut self, repo: Option<RepoRef>) {
        self.repo = repo;
    }

    /// Names the caller of the `ui` commands (the MCP server sets the agent's
    /// `clientInfo.name` and its configured `ui_control` mode).
    pub fn set_ui_origin(&mut self, origin: UiOrigin) {
        self.ui_origin = origin;
    }

    pub fn ui_origin(&self) -> &UiOrigin {
        &self.ui_origin
    }

    /// The targeted repository, when there is one.
    pub fn repo_ref(&self) -> Option<&RepoRef> {
        self.repo.as_ref()
    }

    /// The targeted repository's working-tree root, when there is one --
    /// for per-repository state (`.gitpoulpi/repo-state.toml`) that lives
    /// beside the repository rather than in the daemon.
    pub fn repo_root(&self) -> Option<&std::path::Path> {
        self.repo.as_ref().map(|r| r.root.as_path())
    }

    /// Sends one request and deserializes the daemon's reply into `T`.
    /// Use `()` for the operations that return nothing.
    pub async fn call<T: DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Result<T, String> {
        let repo = self.target_repo()?;
        self.call_for(command, args, Some(repo)).await
    }

    /// Like `call`, but hands back the daemon's whole error, so the caller
    /// can match on its typed part (an undo refusal) instead of a string.
    pub async fn call_typed<T: DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Result<Result<T, gitpoulpi_ipc::ResponseError>, String> {
        let repo = self.target_repo()?;
        self.send(command, args, Some(repo)).await
    }

    /// Like `call`, but for `Clone` - the one operation with no repo to
    /// target, since it is what produces one.
    pub async fn call_without_repo<T: DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Result<T, String> {
        self.call_for(command, args, None).await
    }

    fn target_repo(&self) -> Result<RepoId, String> {
        self.repo
            .as_ref()
            .map(|r| r.id.clone())
            .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())
    }

    async fn call_for<T: DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: serde_json::Value,
        repo: Option<RepoId>,
    ) -> Result<T, String> {
        self.send(command, args, repo).await?.map_err(|e| {
            if let Some(detail) = e.detail {
                remember_detail(&e.message, *detail);
            }
            e.message
        })
    }

    async fn send<T: DeserializeOwned>(
        &mut self,
        command: OperationKind,
        args: serde_json::Value,
        repo: Option<RepoId>,
    ) -> Result<Result<T, gitpoulpi_ipc::ResponseError>, String> {
        let id = self.next_id;
        self.next_id += 1;
        let mut args = args;
        if self.cancel_on_disconnect && command.traits().network {
            if let Some(map) = args.as_object_mut() {
                map.insert(
                    gitpoulpi_ipc::CANCEL_ON_DISCONNECT.into(),
                    serde_json::Value::Bool(true),
                );
            }
        }
        let request = Request {
            id,
            session: self.session.clone(),
            repo,
            command,
            args,
        };
        let response = self
            .client
            .call(&request)
            .await
            .map_err(|e| format!("daemon connection lost: {e}"))?;
        if self.flush_journal {
            // A staging operation's journal entry is written behind the
            // response on a background thread; this process exits right
            // after the command, which would drop it (and the next `undo`
            // would find nothing). Daemon and client share this process, so
            // flushing here flushes the daemon's.
            gitpoulpi_daemon::flush_undo_journal();
        }
        match response.result {
            Ok(value) => serde_json::from_value(value)
                .map(Ok)
                .map_err(|e| format!("malformed daemon response for {command:?}: {e}")),
            Err(e) => Ok(Err(e)),
        }
    }
}

/// Connects to this session's already-running daemon, or embeds one for
/// the lifetime of this command.
///
/// Unlike the TUI's twin (`gitpoulpi-tui`'s `daemon_link::ensure_daemon`),
/// a short-lived caller passes `publish = false`: a CLI process lives for
/// milliseconds, so a published discovery file would point at a port that
/// is already gone and make every other client pay a failed connection.
/// The TUI and the MCP server run for hours, so publishing lets later
/// clients join them.
///
/// `backend` (from `general.default_backend` plus the env and `--backend`
/// layers) applies only to a daemon this call *starts*; a running one
/// keeps the `GitBackend` it was built with, because its other clients
/// are already subscribed to it.
async fn ensure_daemon(
    session: &str,
    backend: DefaultBackend,
    publish: bool,
) -> Result<u16, String> {
    let port_file = gitpoulpi_config::ConfigPaths::discover().map(|p| p.ipc_port_file(session));
    if let Some(port_file) = &port_file {
        if let Some(port) = gitpoulpi_ipc::discovery::live_port(port_file).await {
            return Ok(port);
        }
    }

    // An update is replacing the files this daemon would run from: starting
    // one now would hold the install folder open (or run half-swapped
    // binaries). A daemon that is already up was handled above.
    if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
        let install_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
        if gitpoulpi_config::update_lock::update_in_progress(
            &paths,
            install_dir.as_deref(),
            gitpoulpi_config::update_lock::now_secs(),
        ) {
            return Err(gitpoulpi_config::update_lock::UPDATING_MESSAGE.to_string());
        }
    }

    let handle = gitpoulpi_daemon::Daemon::new(session)
        .spawn(gitpoulpi_daemon::backend_for(backend))
        .await
        .map_err(|e| format!("failed to start this session's daemon: {e}"))?;
    let port = handle.port;
    // The accept loop has to outlive this function; the process exiting is
    // what stops it.
    std::mem::forget(handle);

    if publish {
        if let Some(port_file) = &port_file {
            if let Some(parent) = port_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(port_file, port.to_string());
            gitpoulpi_ipc::discovery::write_host_file(port_file);
        }
    }
    Ok(port)
}

/// The repo a command targets: `--repo` if given, otherwise the enclosing
/// repository of the current directory.
///
/// `RepoId` is built from the repository *root*, so `gitpoulpi status` in
/// a subdirectory and one at the top address the same repo (the daemon
/// keys its per-repo lock on that id; see `dispatch::repo_ref`).
pub fn discover_repo(explicit: Option<&str>) -> Result<RepoRef, String> {
    let start = match explicit {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()
            .map_err(|e| format!("could not read the current directory: {e}"))?,
    };
    let root = find_repo_root(&start)
        .ok_or_else(|| format!("{} is not inside a git repository", start.display()))?;
    Ok(RepoRef::from_root(&root))
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}
