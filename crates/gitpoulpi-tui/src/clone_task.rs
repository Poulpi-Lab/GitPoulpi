//! A clone, on its own connection to the session's daemon.
//!
//! **Why not through `daemon_link`.** That link is bound to one
//! `RepoRef` for its whole life and multiplexes every request for it; a
//! clone has no repository (it is what produces one). Its own connection
//! also means a two-minute clone cannot delay a status refresh.
//!
//! **Process-wide, not per window** (see the background-ops design, B9). The
//! window restarts on every tab switch and the Clone dialog closes on
//! Submit, so the clone's outcome cannot belong to either: the worker
//! thread posts its end on a process-wide queue that whichever window is
//! current drains (`drain`). Progress is the daemon job's, shown by the
//! jobs indicator; nothing here streams it. Two clones in flight both end
//! up in the queue; a second clone into the same folder is refused before
//! it is sent (`is_running`), the daemon refuses it too.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{CloneOptions, OperationKind, RepoRef};
use gitpoulpi_core::TypedError;
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Request, ServerMessage};

/// How a clone ended when it did not produce a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneFailure {
    /// The user (or another client) canceled the job; the daemon removed
    /// the half folder. Not a failure: no card.
    Canceled,
    Failed(String),
}

/// What a finished clone tells the UI.
#[derive(Debug, Clone)]
pub struct CloneEnd {
    pub dest: PathBuf,
    pub result: Result<RepoRef, CloneFailure>,
}

static ENDED: std::sync::Mutex<Vec<CloneEnd>> = std::sync::Mutex::new(Vec::new());
static RUNNING: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

fn lock<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether a clone into `dest` is still going (started by this process).
pub fn is_running(dest: &Path) -> bool {
    lock(&RUNNING).iter().any(|d| same_dest(d, dest))
}

fn same_dest(a: &Path, b: &Path) -> bool {
    let (a, b) = (a.to_string_lossy(), b.to_string_lossy());
    if cfg!(windows) {
        a.eq_ignore_ascii_case(&b)
    } else {
        a == b
    }
}

/// Every clone that ended since the last call, oldest first.
pub fn drain() -> Vec<CloneEnd> {
    std::mem::take(&mut *lock(&ENDED))
}

/// Starts a clone in the background. The end is reported through
/// [`drain`], whichever window is current by then.
///
/// `dest` is already absolute (the caller resolves it against the TUI's
/// own working directory), because the daemon may have been started
/// somewhere else entirely.
pub fn start(session: String, url: String, dest: PathBuf, opts: CloneOptions) {
    lock(&RUNNING).push(dest.clone());
    std::thread::spawn(move || {
        let result = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime.block_on(run(session, url, dest.clone(), opts)),
            Err(e) => Err(CloneFailure::Failed(
                gitpoulpi_i18n::msg::task_error::clone_runtime(e),
            )),
        };
        lock(&RUNNING).retain(|d| !same_dest(d, &dest));
        lock(&ENDED).push(CloneEnd { dest, result });
        crate::redraw::poke();
    });
}

async fn run(
    session: String,
    url: String,
    dest: PathBuf,
    opts: CloneOptions,
) -> Result<RepoRef, CloneFailure> {
    let started = std::time::Instant::now();
    let failed = |e: String| CloneFailure::Failed(e);
    let endpoint = daemon_endpoint(&session).await.map_err(failed)?;
    let stream = gitpoulpi_ipc::discovery::connect(&endpoint)
        .await
        .map_err(|e| failed(gitpoulpi_i18n::msg::task_error::daemon_unreachable(e)))?;
    let (mut read_half, mut write_half) = stream.into_split();

    let request = Request {
        id: 1,
        session,
        // The one request with no repo target -- see
        // `dispatch_network`'s Clone arm.
        repo: None,
        command: OperationKind::Clone,
        args: serde_json::json!({ "url": url, "dest": dest, "opts": opts }),
    };
    if write_message(&mut write_half, &request).await.is_err() {
        return Err(failed(
            gitpoulpi_i18n::msg::task_error::CLONE_SEND.get().into(),
        ));
    }

    // Events (progress, job changes) arrive on this stream too and are
    // skipped: the jobs indicator has its own subscription.
    loop {
        match read_message::<_, ServerMessage>(&mut read_half).await {
            Ok(Some(ServerMessage::Response(response))) if response.request_id == request.id => {
                return match response.result {
                    Ok(value) => {
                        let repo = serde_json::from_value::<RepoRef>(value).map_err(|e| {
                            failed(gitpoulpi_i18n::msg::task_error::clone_malformed(e))
                        })?;
                        crate::activity_log::timing_record(
                            &dest,
                            gitpoulpi_i18n::msg::activity_log::OP_CLONE
                                .get()
                                .to_string(),
                            u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                        );
                        Ok(repo)
                    }
                    Err(e) => Err(match e.kind {
                        Some(TypedError::Canceled { .. }) => CloneFailure::Canceled,
                        Some(ref typed) => {
                            failed(crate::daemon_link::error_text::typed_text(typed))
                        }
                        None => failed(e.message),
                    }),
                };
            }
            Ok(Some(_)) => continue,
            // The connection closing before the response is a failed
            // clone, not a silently-still-running one.
            Ok(None) | Err(_) => {
                return Err(failed(
                    gitpoulpi_i18n::msg::task_error::CLONE_CLOSED.get().into(),
                ))
            }
        }
    }
}

async fn daemon_endpoint(session: &str) -> Result<gitpoulpi_ipc::Endpoint, String> {
    crate::daemon_link::ensure_daemon(session).await
}
