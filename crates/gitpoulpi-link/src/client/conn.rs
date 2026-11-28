use super::*;

/// The requests on one connection still waiting for their response, and -
/// once the reader task has stopped -- why the connection is gone.
///
/// `closed` is what keeps a lost connection from hanging a caller
/// forever: the reader task is the only thing that completes a waiting
/// request, so when it exits it sets `closed` and drops every sender
/// still in `waiting` (each `call` then fails with the reason instead of
/// awaiting a response nobody will deliver), and a `call` made after
/// that fails at once rather than queueing behind a dead socket. Both
/// halves happen under the one lock, so no request can slip in between.
#[derive(Default)]
pub(crate) struct PendingRequests {
    pub(super) waiting: HashMap<u64, oneshot::Sender<Response>>,
    pub(super) closed: Option<String>,
}

impl PendingRequests {
    /// Marks the connection gone and fails every request still waiting.
    pub(super) fn close(&mut self, reason: String) {
        self.closed.get_or_insert(reason);
        self.waiting.clear();
    }

    pub(super) fn lost(&self) -> CoreError {
        CoreError::Backend(gitpoulpi_i18n::msg::link_error::connection_lost(
            self.closed
                .clone()
                .unwrap_or_else(|| gitpoulpi_i18n::msg::link_error::CLOSED.get().to_string()),
        ))
    }
}

pub(super) type PendingMap = Arc<std::sync::Mutex<PendingRequests>>;

/// Everything `call` needs, bundled for a task that makes many calls.
#[derive(Clone)]
pub(super) struct LinkConn {
    pub(super) write_half: Arc<AsyncMutex<OwnedWriteHalf>>,
    pub(super) pending: PendingMap,
    pub(super) next_id: Arc<AtomicU64>,
    pub(super) session: String,
    pub(super) repo: RepoRef,
    /// The last answer to each conditional read (see `refresh::call_cached`).
    pub(super) memo: Arc<std::sync::Mutex<refresh::ReadMemo>>,
}

impl LinkConn {
    pub(super) async fn call<T: serde::de::DeserializeOwned>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> GitResult<T> {
        call(
            &self.write_half,
            &self.pending,
            &self.next_id,
            &self.session,
            &self.repo,
            command,
            args,
        )
        .await
    }

    /// The commit's changed files without hunks where the backend can
    /// list more cheaply than it diffs.
    pub(super) async fn commit_list(&self, commit: Oid) -> GitResult<DiffFileList> {
        self.call(
            OperationKind::DiffFiles,
            serde_json::json!({ "target": DiffTarget::CommitVsParent(commit) }),
        )
        .await
    }

    pub(super) async fn commit_diff(&self, commit: Oid) -> GitResult<Vec<FileDiff>> {
        self.call(
            OperationKind::Diff,
            serde_json::json!({ "target": DiffTarget::CommitVsParent(commit) }),
        )
        .await
    }
}

pub(super) async fn call<T: serde::de::DeserializeOwned>(
    write_half: &Arc<AsyncMutex<OwnedWriteHalf>>,
    pending: &PendingMap,
    next_id: &Arc<AtomicU64>,
    session: &str,
    repo: &RepoRef,
    command: OperationKind,
    args: serde_json::Value,
) -> GitResult<T> {
    let value = call_raw(write_half, pending, next_id, session, repo, command, args).await?;
    serde_json::from_value(value)
        .map_err(|e| CoreError::Backend(gitpoulpi_i18n::msg::link_error::malformed_response(e)))
}

/// [`call`] up to the daemon's reply, undecoded.
pub(super) async fn call_raw(
    write_half: &Arc<AsyncMutex<OwnedWriteHalf>>,
    pending: &PendingMap,
    next_id: &Arc<AtomicU64>,
    session: &str,
    repo: &RepoRef,
    command: OperationKind,
    args: serde_json::Value,
) -> GitResult<serde_json::Value> {
    // A sequence run's mutations carry the run (`origin`), for the journal.
    let args = super::origin::attach(command, args);
    let id = next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    {
        let mut pending = pending.lock().unwrap();
        if pending.closed.is_some() {
            return Err(pending.lost());
        }
        pending.waiting.insert(id, tx);
    }

    let request = Request {
        id,
        session: session.to_string(),
        repo: Some(repo.id.clone()),
        command,
        args,
    };
    {
        let mut write_half = write_half.lock().await;
        if let Err(e) = write_message(&mut *write_half, &request).await {
            let mut pending = pending.lock().unwrap();
            pending.waiting.remove(&id);
            return Err(CoreError::Backend(
                gitpoulpi_i18n::msg::link_error::send_failed(e),
            ));
        }
    }

    // Completed by the reader task, or failed by it when it stops (see
    // `PendingRequests::close`) -- never left waiting on a dead socket.
    // A response the daemon dropped is bounded by the same timeout the
    // other IPC clients use, so no worker waits forever.
    let response = match tokio::time::timeout(gitpoulpi_ipc::CALL_TIMEOUT, rx).await {
        Ok(Ok(r)) => r,
        Ok(Err(_)) => return Err(pending.lock().unwrap().lost()),
        Err(_) => {
            pending.lock().unwrap().waiting.remove(&id);
            return Err(CoreError::Backend(
                gitpoulpi_i18n::msg::link_error::TIMEOUT.get().to_string(),
            ));
        }
    };
    match response.result {
        Ok(value) => Ok(value),
        // The typed half first: a diverged push must stay a
        // `PushRejected` all the way to the render loop.
        Err(e) => {
            if let Some(context) = &e.context {
                super::failed_op::note(context);
            }
            Err(match e.kind {
                Some(kind) => CoreError::from_typed(kind),
                None => CoreError::from_wire(e.message, e.detail.map(|d| *d)),
            })
        }
    }
}

/// This window's own short commit sha, baked in at build time (the same
/// value the daemon reports, no spawn). `None` for a build outside a git
/// checkout; the build-mismatch check above treats that as inconclusive.
pub(super) fn this_build_git_sha() -> Option<String> {
    gitpoulpi_ipc::build_git_sha().map(str::to_string)
}

/// The runtime an embedded daemon lives on: one per process, created on
/// first use and never dropped (a `static` is never destructed).
///
/// It must not be the runtime of a link thread: every window has its own
/// link thread whose runtime is dropped when the window ends, so a daemon
/// spawned there would die with the window while the discovery file still
/// pointed at it, and the next window's `ensure_daemon` could connect
/// mid-shutdown and wait forever for a first response (see
/// `session-and-state.md`, "The daemon died with the window").
pub(super) fn embedded_daemon_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("gitpoulpi-embedded-daemon")
            .enable_all()
            .build()
            .expect("failed to start the embedded daemon's tokio runtime")
    })
}

/// Discovery files this process published, with the port each held.
static PUBLISHED: std::sync::Mutex<Vec<(PathBuf, u16)>> = std::sync::Mutex::new(Vec::new());

fn record_published(port_file: &std::path::Path, port: u16) {
    let mut published = PUBLISHED.lock().unwrap_or_else(|e| e.into_inner());
    published.retain(|(p, _)| p != port_file);
    published.push((port_file.to_path_buf(), port));
}

/// Takes down the discovery files this process published, each only while it
/// still holds our port (a newer process may have replaced it). The embedded
/// daemon dies with the process, so a file left behind makes the next start
/// connect to a dead port (about 2 s to be refused on Windows). Idempotent:
/// called on every way out, including the panic hook.
pub fn remove_published_port_files() {
    let published = std::mem::take(&mut *PUBLISHED.lock().unwrap_or_else(|e| e.into_inner()));
    for (port_file, port) in published {
        gitpoulpi_ipc::discovery::remove_if_ours(&port_file, port);
    }
}

/// Finds this session's running daemon via its discovery file
/// (`gitpoulpi_config::ConfigPaths::ipc_port_file`) and connects to confirm
/// it is alive; if none is found (or it is stale), starts one in-process
/// and publishes its port there, so this TUI and any client launched
/// afterwards find the same daemon. The embedded daemon runs on
/// `embedded_daemon_runtime`, **not** the caller's runtime, so it lives as
/// long as the TUI process, though it still dies with that process rather
/// than surviving independently (see `session-and-state.md`).
pub async fn ensure_daemon(session: &str) -> Result<gitpoulpi_ipc::Endpoint, String> {
    ensure_daemon_at(session_port_file(session).as_deref(), session).await
}

/// This session's discovery file, if there is a config directory at all.
pub(super) fn session_port_file(session: &str) -> Option<PathBuf> {
    gitpoulpi_config::ConfigPaths::discover().map(|p| p.ipc_port_file(session))
}

/// [`ensure_daemon`] against an explicit discovery file - what the tests
/// drive, rather than repointing the process-wide `GITPOULPI_CONFIG_DIR`
/// (see `session_store.rs`'s `config_dir` for why that is wrong).
pub(super) async fn ensure_daemon_at(
    port_file: Option<&std::path::Path>,
    session: &str,
) -> Result<gitpoulpi_ipc::Endpoint, String> {
    let paths = gitpoulpi_config::ConfigPaths::discover();
    ensure_daemon_in(port_file, session, paths.as_ref()).await
}

/// Refuses while an update is replacing the files this process would run
/// from: a daemon started now would hold the install folder open, or run
/// half-swapped binaries. `Err` carries the status-line text.
pub(super) fn refuse_while_updating(
    paths: Option<&gitpoulpi_config::ConfigPaths>,
    install_dir: Option<&std::path::Path>,
    now: u64,
) -> Result<(), String> {
    match paths {
        Some(paths)
            if gitpoulpi_config::update_lock::update_in_progress(paths, install_dir, now) =>
        {
            Err(gitpoulpi_i18n::msg::link_error::UPDATING.get().to_string())
        }
        _ => Ok(()),
    }
}

/// [`ensure_daemon_at`] with the config paths given (the update lock lives
/// there), for the tests.
pub(super) async fn ensure_daemon_in(
    port_file: Option<&std::path::Path>,
    session: &str,
    paths: Option<&gitpoulpi_config::ConfigPaths>,
) -> Result<gitpoulpi_ipc::Endpoint, String> {
    if let Some(port_file) = port_file {
        // A file left by a process that is gone is deleted, not connected to:
        // on Windows a refused loopback connect takes ~2 s. The connect
        // itself is bounded too (`discovery::CONNECT_TIMEOUT`).
        if let Some(endpoint) = gitpoulpi_ipc::discovery::live_endpoint(port_file).await {
            return Ok(endpoint);
        }
    }

    // A daemon that is already up was handled above; starting a new one
    // waits for the update.
    let install_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
    refuse_while_updating(
        paths,
        install_dir.as_deref(),
        gitpoulpi_config::update_lock::now_secs(),
    )?;

    // `general.default_backend` (plus the `GITPOULPI_*` env layer), resolved
    // through the same `gitpoulpi_daemon` factory the CLI uses: otherwise
    // which backend served a repo would depend on whether a TUI or a CLI
    // started its daemon first. The fallback is defensive only: `run`
    // resolves the same settings before the alternate screen and refuses to
    // start on a config that will not parse.
    let backend: Arc<dyn GitBackend> = gitpoulpi_daemon::backend_from_ambient_config()
        .unwrap_or_else(|_| gitpoulpi_daemon::backend_for(Default::default()));
    let daemon = gitpoulpi_daemon::Daemon::new(session);
    // Spawned *on the process-lifetime runtime*: `Daemon::spawn` puts its
    // accept loop on whatever runtime it is polled from, so awaiting it
    // here directly would tie the daemon to this caller's runtime again.
    let endpoint = embedded_daemon_runtime()
        .spawn(async move {
            let handle = daemon.spawn(backend).await.map_err(|e| e.to_string())?;
            let endpoint = handle.endpoint();
            // Leaked on purpose: dropping the handle would not stop the
            // server, but it is the one thing that could, so nothing
            // holds it.
            std::mem::forget(handle);
            Ok::<gitpoulpi_ipc::Endpoint, String>(endpoint)
        })
        .await
        .map_err(gitpoulpi_i18n::msg::link_error::embedded_failed)?
        .map_err(gitpoulpi_i18n::msg::link_error::embedded_failed)?;

    if let Some(port_file) = port_file {
        // The token file lands user-only before the port file.
        let _ = gitpoulpi_ipc::discovery::publish(port_file, &endpoint);
        // Who owns the port, so the next start can tell a stale file from a
        // live one without a connect (`remove_port_file` takes both down).
        gitpoulpi_ipc::discovery::write_host_file(port_file);
        record_published(port_file, endpoint.port);
    }

    Ok(endpoint)
}
