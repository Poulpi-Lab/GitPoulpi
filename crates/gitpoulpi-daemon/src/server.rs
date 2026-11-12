use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use gitpoulpi_core::domain::{Operation, OperationKind, RepoId};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::transport::{read_message, write_message, write_raw_frame};
use gitpoulpi_ipc::{ChangeScope, DaemonInfo, Event, Request, Response, ServerMessage, Token};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::cache::ReadCache;
use crate::dispatch::{dispatch, dispatch_cached_reply, dispatch_network, repo_ref, CachedReply};
use crate::jobs::{self, JobHandle, JobResult, Registry, Submitted};
use crate::viewers::{self, ConnUi, Viewers};
use crate::Result;

/// Per-repo serialization of *mutations*: the backends shell out to real
/// `git`, which takes its own `.git/index.lock`, so a mutation running
/// concurrently with anything else on the *same* repo can collide on it
/// and fail with a spurious "Unable to create index.lock". Keyed by
/// `RepoId` so unrelated repos still dispatch concurrently.
///
/// A read/write lock rather than a mutex: reads
/// ([`OpTraits::read`](gitpoulpi_core::domain::OpTraits)) take it shared and
/// run concurrently, a mutation takes it exclusively, so the commit-files
/// request the Log selection makes is not queued behind the periodic
/// refresh (on Windows a `git` process for `conflicts` costs a second or
/// more). Reads only take git's *optional* locks, which git skips silently
/// when another process holds them.
///
/// Order is preserved: tokio's lock is fair, so a mutation waits for the
/// reads admitted before it and the reads that arrive after it wait for
/// the mutation - including the ones on its own connection.
///
/// Fetch and push take none of it (see [`TransferLocks`]): a mutation
/// waiting behind a transfer as a writer made every read that came after
/// it wait too, so one click during the open-time fetch froze every pane
/// for as long as the network took.
type RepoLocks = LockMap<RwLock<()>>;

/// Per-repo serialization of *transfers*: fetch and push hold only this,
/// so the reads and the local mutations of the repository go on while the
/// network is slow. They touch refs and objects, never the index or the
/// worktree, and git locks each ref it moves itself (and retries the
/// `packed-refs` lock for a second), so a branch created meanwhile cannot
/// tear anything. Two transfers do contend on the remote-tracking refs, so
/// they queue here, and so does every other network operation (pull,
/// clone, ...), which takes this before the exclusive repo lock.
type TransferLocks = LockMap<tokio::sync::Mutex<()>>;

type LockMap<L> = Arc<StdMutex<HashMap<RepoId, Arc<L>>>>;

fn repo_lock<L: Default>(locks: &LockMap<L>, repo: &RepoId) -> Arc<L> {
    let mut locks = locks.lock().unwrap();
    if let Some(lock) = locks.get(repo) {
        return Arc::clone(lock);
    }
    // A repo seen for the first time: drop the locks nobody holds (every
    // holder, queued or running, owns a clone taken under this mutex), so
    // the map follows the repos in use instead of every one ever opened.
    locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    let lock = Arc::new(L::default());
    locks.insert(repo.clone(), Arc::clone(&lock));
    lock
}

/// A cached read's answer, already serialized (see `cache.rs`). It has its
/// own channel to the writer so the bytes are never parsed into a `Value`.
struct RawResponse {
    request_id: u64,
    json: Arc<[u8]>,
}

impl RawResponse {
    /// The frame is exactly what `ServerMessage::Response(Response { request_id,
    /// result: Ok(..) })` serializes to.
    async fn write<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> gitpoulpi_ipc::transport::Result<()> {
        let head = format!(
            "{{\"Response\":{{\"request_id\":{},\"result\":{{\"Ok\":",
            self.request_id
        );
        write_raw_frame(writer, &[head.as_bytes(), &self.json, b"}}}"]).await
    }
}

/// Who the timing log names for a request (`GITPOULPI_TIMING`).
fn timing_label(request_id: u64, command: OperationKind) -> String {
    format!("#{request_id} {command:?}")
}

/// Runs the daemon's accept loop until `shutdown` resolves, *or* until a
/// client sends `OperationKind::Shutdown` (reaching this loop through the
/// `internal_shutdown` channel). Binds an ephemeral loopback port and
/// reports it via `port_tx` as soon as it is bound.
pub async fn run(
    backend: Arc<dyn GitBackend>,
    port_tx: tokio::sync::oneshot::Sender<u16>,
    mut shutdown: tokio::sync::oneshot::Receiver<()>,
    info: DaemonInfo,
    token: Token,
) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    let _ = port_tx.send(port);
    // Where a credential helper of this daemon's grants finds it, published
    // or not (ADR-0010, decision 21); gone when this function returns.
    let _endpoint = crate::grants::register_endpoint(
        &info.session,
        gitpoulpi_ipc::Endpoint::new(port, Some(token.clone())),
    );

    let (events_tx, _) = broadcast::channel::<Event>(256);
    let repo_locks: RepoLocks = Arc::new(StdMutex::new(HashMap::new()));
    let transfer_locks: TransferLocks = Arc::new(StdMutex::new(HashMap::new()));
    let read_cache = Arc::new(ReadCache::default());
    let viewers = Arc::new(Viewers::default());
    let jobs = Registry::new(events_tx.clone());
    let info = Arc::new(info);
    let token = Arc::new(token);
    let (internal_shutdown_tx, mut internal_shutdown_rx) = mpsc::channel::<()>(1);
    // Shutdown drains: connections stop reading when `stop` flips, and every
    // connection / admitted-request task holds a clone of `inflight_tx`, so
    // `inflight_rx` closes only once the last blocking dispatch has returned.
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let (inflight_tx, mut inflight_rx) = mpsc::channel::<()>(1);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                drain(&stop_tx, inflight_tx, &mut inflight_rx).await;
                return Ok(());
            }
            _ = internal_shutdown_rx.recv() => {
                drain(&stop_tx, inflight_tx, &mut inflight_rx).await;
                return Ok(());
            }
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                // Responses and pushed events are small frames; do not let Nagle batch them.
                let _ = stream.set_nodelay(true);
                let backend = Arc::clone(&backend);
                let events_tx = events_tx.clone();
                let locks = (Arc::clone(&repo_locks), Arc::clone(&transfer_locks));
                let read_cache = Arc::clone(&read_cache);
                let control = Control {
                    info: Arc::clone(&info),
                    token: Arc::clone(&token),
                    viewers: Arc::clone(&viewers),
                    jobs: Arc::clone(&jobs),
                    internal_shutdown_tx: internal_shutdown_tx.clone(),
                    stop: stop_rx.clone(),
                    inflight: inflight_tx.clone(),
                };
                tokio::spawn(handle_connection(stream, backend, events_tx, locks, read_cache, control));
            }
        }
    }
}

/// How long a shutdown waits for in-flight dispatches before giving up (a
/// long clone must not hold the process hostage).
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Stops every connection's reader, then waits (bounded) for the connection
/// and request tasks to finish their in-flight blocking dispatch. Without it
/// the daemon "shut down" while a read was still running, and a test
/// deleting its fixture raced that read.
async fn drain(
    stop_tx: &tokio::sync::watch::Sender<bool>,
    inflight_tx: mpsc::Sender<()>,
    inflight_rx: &mut mpsc::Receiver<()>,
) {
    let _ = stop_tx.send(true);
    drop(inflight_tx);
    let _ = tokio::time::timeout(DRAIN_TIMEOUT, inflight_rx.recv()).await;
    crate::journal::flush_pending();
}

/// What a connection needs for the daemon-control requests (`Hello`,
/// `Shutdown`, `Ui*`), plus the shutdown drain's handles.
struct Control {
    /// Flips to `true` when the daemon shuts down: the reader stops.
    stop: tokio::sync::watch::Receiver<bool>,
    /// Held by the connection and each admitted request until it is done.
    inflight: mpsc::Sender<()>,
    info: Arc<DaemonInfo>,
    /// What a connection's first `Hello` must carry (see `gitpoulpi_ipc::auth`).
    token: Arc<Token>,
    viewers: Arc<Viewers>,
    jobs: Arc<Registry>,
    internal_shutdown_tx: mpsc::Sender<()>,
}

/// A request the connection's reader hands to its processing loop, with the
/// job a network request became.
struct Work {
    request: Request,
    job: Option<JobHandle>,
}

fn job_error(error: &gitpoulpi_core::Error) -> gitpoulpi_ipc::ResponseError {
    gitpoulpi_ipc::ResponseError {
        message: error.to_string(),
        unsupported: false,
        kind: error.typed(),
        detail: None,
        context: None,
        auth_required: false,
    }
}

/// Answers `JobList` / `JobCancel` (they never wait for admission).
async fn answer_job_control(
    jobs: &Arc<Registry>,
    request: &Request,
    out_tx: &mpsc::Sender<ServerMessage>,
) -> bool {
    let result: std::result::Result<serde_json::Value, gitpoulpi_ipc::ResponseError> =
        match request.command {
            OperationKind::JobList => {
                Ok(serde_json::to_value(jobs.list()).unwrap_or(serde_json::Value::Null))
            }
            _ => match request.args.get("id").and_then(serde_json::Value::as_u64) {
                Some(id) => match jobs.cancel(id) {
                    Ok(canceled) => {
                        if let Some((tx, response)) = canceled.respond {
                            let _ = tx.send(ServerMessage::Response(response)).await;
                        }
                        Ok(serde_json::to_value(canceled.info).unwrap_or(serde_json::Value::Null))
                    }
                    Err(error) => Err(job_error(&error)),
                },
                None => Err(gitpoulpi_ipc::ResponseError {
                    message: "missing `id`".into(),
                    unsupported: false,
                    kind: None,
                    detail: None,
                    context: None,
                    auth_required: false,
                }),
            },
        };
    out_tx
        .send(ServerMessage::Response(Response {
            request_id: request.id,
            result,
        }))
        .await
        .is_ok()
}

/// Reads a connection's requests. Job requests are answered right here, so a
/// pull running on the processing loop never delays them; a network request
/// becomes a job *now* (so it is visible while it waits, and a duplicate joins
/// it); everything else is handed to the loop in arrival order.
async fn read_requests(
    read_half: tokio::net::tcp::OwnedReadHalf,
    work_tx: mpsc::UnboundedSender<Work>,
    out_tx: mpsc::Sender<ServerMessage>,
    jobs: Arc<Registry>,
    stop: tokio::sync::watch::Receiver<bool>,
) {
    let mut owned = Vec::new();
    read_loop(read_half, work_tx, out_tx, &jobs, &mut owned, stop).await;
    // The connection is gone: the jobs that asked to die with it (a CLI
    // command interrupted with Ctrl+C) are canceled. One past its point of no
    // return answers `NotCancellable` and simply finishes.
    for id in owned {
        if let Ok(canceled) = jobs.cancel(id) {
            tracing::debug!(job = id, "canceled: its client disconnected");
            drop(canceled);
        }
    }
}

async fn read_loop(
    mut read_half: tokio::net::tcp::OwnedReadHalf,
    work_tx: mpsc::UnboundedSender<Work>,
    out_tx: mpsc::Sender<ServerMessage>,
    jobs: &Arc<Registry>,
    owned: &mut Vec<u64>,
    mut stop: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        let request: Request = tokio::select! {
            read = read_message(&mut read_half) => match read {
                Ok(Some(request)) => request,
                _ => return,
            },
            _ = stop.wait_for(|stopped| *stopped) => return,
        };
        if request.command.is_job_control() {
            if !answer_job_control(jobs, &request, &out_tx).await {
                return;
            }
            continue;
        }
        let mut job = None;
        if jobs::is_job(&request) {
            let immediate: Option<std::result::Result<serde_json::Value, _>> =
                match jobs.submit(&request, out_tx.clone()) {
                    Submitted::New(handle) => {
                        if Registry::cancels_on_disconnect(&request) {
                            owned.push(handle.id());
                        }
                        job = Some(handle);
                        None
                    }
                    Submitted::Joined { job, done } => {
                        tracing::debug!(job, request = request.id, "joined a running job");
                        let (out_tx, id) = (out_tx.clone(), request.id);
                        tokio::spawn(async move {
                            let result = jobs::wait_done(done).await;
                            let _ = out_tx
                                .send(ServerMessage::Response(Response {
                                    request_id: id,
                                    result,
                                }))
                                .await;
                        });
                        continue;
                    }
                    Submitted::Conflict(existing) => {
                        Some(Err(job_error(&gitpoulpi_core::Error::JobConflict {
                            existing,
                        })))
                    }
                    Submitted::Skipped => Some(Ok(serde_json::json!({ "updated_refs": [] }))),
                };
            if let Some(result) = immediate {
                let sent = out_tx
                    .send(ServerMessage::Response(Response {
                        request_id: request.id,
                        result,
                    }))
                    .await;
                if sent.is_err() {
                    return;
                }
                continue;
            }
        }
        if work_tx.send(Work { request, job }).is_err() {
            return;
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    backend: Arc<dyn GitBackend>,
    events_tx: broadcast::Sender<Event>,
    (repo_locks, transfer_locks): (RepoLocks, TransferLocks),
    read_cache: Arc<ReadCache>,
    control: Control,
) {
    let Control {
        info,
        token,
        viewers,
        jobs,
        internal_shutdown_tx,
        stop,
        inflight,
    } = control;
    // Nothing is read or dispatched before the client proved it read the
    // user-only token file: its first request is `Hello` carrying the token.
    let Some(client) = gitpoulpi_ipc::auth::authenticate_named(&mut stream, &token, &info).await
    else {
        return;
    };
    let mut conn_ui = ConnUi::new(viewers.next_conn());
    let (read_half, mut write_half) = stream.into_split();
    let mut events_rx = events_tx.subscribe();

    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<ServerMessage>(64);
    let (raw_tx, mut raw_rx) = tokio::sync::mpsc::channel::<RawResponse>(64);

    // Forwards this connection's own responses plus every broadcast event
    // out over the wire, interleaved, so a client sees live updates from
    // *other* clients' mutations without polling.
    let writer_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = out_rx.recv() => match msg {
                    Some(mut msg) => {
                        // The frontend this connection said it is goes into
                        // the context of every failure it is answered with,
                        // and into the log line of that failure.
                        crate::dispatch::stamp_failure(&mut msg, client.as_deref());
                        if write_message(&mut write_half, &msg).await.is_err() {
                            return;
                        }
                    }
                    None => return,
                },
                raw = raw_rx.recv() => match raw {
                    Some(raw) => {
                        if raw.write(&mut write_half).await.is_err() {
                            return;
                        }
                    }
                    None => return,
                },
                event = events_rx.recv() => {
                    if let Ok(event) = event {
                        if write_message(&mut write_half, &ServerMessage::Event(event)).await.is_err() {
                            return;
                        }
                    }
                }
            }
        }
    });

    let (work_tx, mut work_rx) = mpsc::unbounded_channel::<Work>();
    let reader = tokio::spawn(read_requests(
        read_half,
        work_tx,
        out_tx.clone(),
        Arc::clone(&jobs),
        stop,
    ));

    while let Some(Work { mut request, job }) = work_rx.recv().await {
        // The session name a transfer's credential grant is served under is
        // this daemon's own, never one the client wrote: a client could name
        // another session's daemon, and the helper's lookup trusts it.
        request.session.clone_from(&info.session);
        // Canceled while it waited behind an earlier request of this
        // connection: already answered, run nothing.
        if job.as_ref().is_some_and(|job| !job.claim()) {
            continue;
        }

        let request_id = request.id;
        let repo = request.repo.clone();
        let command = request.command;

        // Daemon-control pseudo-operations: answered here, never dispatched,
        // never under a repo lock.
        if command.is_ui_control() {
            let response = viewers::handle(&viewers, &mut conn_ui, &out_tx, &request).await;
            if out_tx
                .send(ServerMessage::Response(response))
                .await
                .is_err()
            {
                break;
            }
            continue;
        }
        if let Some(flow) =
            handle_control(&request, &info, &out_tx, &events_tx, &internal_shutdown_tx).await
        {
            match flow {
                Flow::Continue => continue,
                Flow::Stop => break,
            }
        }

        // A conflict check that asks to be debounced waits for the repository
        // to be quiet: on its own task, under no repository lock.
        if command == OperationKind::PredictConflicts {
            if let Some(debounce) = crate::conflict_prevention::debounce_of(&request) {
                crate::conflict_prevention::spawn_debounced(
                    Arc::clone(&backend),
                    request,
                    debounce,
                    out_tx.clone(),
                );
                continue;
            }
        }

        let backend = Arc::clone(&backend);
        let received = std::time::Instant::now();

        // Reads: admitted under the shared lock *here*, in arrival order,
        // then run on their own task so this loop goes straight back to
        // reading. That ordering keeps a later mutation on this connection
        // behind them (see `RepoLocks`). A read never broadcasts, never
        // probes for conflicts and is never a network operation.
        //
        // Transfers (fetch, push) are not admitted under the repo lock at
        // all: they queue on the transfer lock, on their own task, so
        // neither this loop nor any read or mutation waits for the network
        // (see `TransferLocks`). Pull and Clone stay exclusive.
        let traits = command.traits();
        let admission = match &repo {
            Some(repo) if traits.read => Some(Admission::Read(
                repo_lock(&repo_locks, repo).read_owned().await,
            )),
            Some(repo) if traits.shared_network => {
                Some(Admission::Transfer(repo_lock(&transfer_locks, repo)))
            }
            _ => None,
        };
        if let (Some(admission), Some(repo)) = (admission, &repo) {
            run_admitted(
                Admitted {
                    request,
                    repo: repo.clone(),
                    admission,
                    received,
                    job,
                },
                backend,
                (out_tx.clone(), raw_tx.clone()),
                events_tx.clone(),
                Arc::clone(&read_cache),
                inflight.clone(),
            );
            continue;
        }

        // Only meaningful for the operations that can leave unmerged
        // paths behind, so it is captured alongside the request rather
        // than re-derived from it after `request` has been moved into
        // `spawn_blocking`.
        let repo_ref_of_for_journal = repo_ref(&request).ok();
        let conflict_probe = command
            .traits()
            .can_conflict
            .then(|| repo_ref(&request).ok())
            .flatten();

        let dispatched = {
            // Held across the blocking dispatch call so no other request
            // against this same repo, read or write, from any connection,
            // can start until this one finishes -- see `RepoLocks`.
            // A network operation queues behind the transfers first, so it
            // never holds the repo lock while it waits for one.
            //
            // A job waiting here can be canceled: the wait is dropped.
            let acquire = acquire_exclusive(
                &repo,
                command.traits().network,
                &transfer_locks,
                &repo_locks,
            );
            let acquired = match &job {
                Some(job) => tokio::select! {
                    () = job.cancel_requested() => None,
                    held = acquire => Some(held),
                },
                None => Some(acquire.await),
            };
            let admitted = received.elapsed();
            let started = std::time::Instant::now();
            let dispatched = if acquired.is_none() {
                Ok(canceled_dispatch(
                    job.as_ref(),
                    &CanceledCtx::of(&request),
                    &events_tx,
                ))
            } else if command.traits().network {
                if let Some(job) = &job {
                    job.running();
                }
                network_cancellable(
                    job.as_ref(),
                    Arc::clone(&backend),
                    request,
                    events_tx.clone(),
                )
                .await
            } else if let (OperationKind::RewriteApply | OperationKind::RewritePurge, Some(job)) =
                (command, &job)
            {
                job.running();
                let ctl = job.ctl();
                let backend = Arc::clone(&backend);
                // A purge reports its steps through the job's progress.
                let purging = command == OperationKind::RewritePurge;
                let control = if purging {
                    repo.clone().map(|repo| {
                        crate::rewrite_runs::Control::new(job.ctl(), repo, events_tx.clone())
                    })
                } else {
                    None
                };
                let point = if purging { "Purging" } else { "Updating" };
                // Past this point the refs move in one transaction (an
                // apply) or the old history is deleted (a purge): cancel is
                // refused from here on.
                tokio::task::spawn_blocking(move || {
                    if ctl.enter_point(point) {
                        match control {
                            Some(control) => control.run(|| dispatch(backend.as_ref(), &request)),
                            None => dispatch(backend.as_ref(), &request),
                        }
                    } else {
                        crate::rewrite_runs::canceled_apply(&request, ctl.id())
                    }
                })
                .await
            } else {
                let backend = Arc::clone(&backend);
                tokio::task::spawn_blocking(move || dispatch(backend.as_ref(), &request)).await
            };
            gitpoulpi_ipc::timing::record("daemon", || {
                format!(
                    "{} exclusive: waited {} ran {}",
                    timing_label(request_id, command),
                    gitpoulpi_ipc::timing::ms(admitted),
                    gitpoulpi_ipc::timing::ms(started.elapsed()),
                )
            });
            // A mutation that recorded nothing may still have moved a ref
            // the top undo entry depends on: re-derive its blocked state
            // (a read, under the same lock) so every client's buttons follow.
            let mut dispatched = dispatched;
            if let (Ok(d), Some(repo_ref)) = (dispatched.as_mut(), repo_ref_of_for_journal.clone())
            {
                if d.mutated
                    && d.journal.is_none()
                    && !matches!(command, OperationKind::Undo | OperationKind::Redo)
                {
                    let backend = Arc::clone(&backend);
                    d.journal = tokio::task::spawn_blocking(move || {
                        crate::journal::state_after_mutation(backend.as_ref(), &repo_ref)
                    })
                    .await
                    .ok()
                    .flatten();
                }
            }
            // Probed while the per-repo lock is still held, so the answer
            // describes the repo as this operation left it and not as
            // some other connection's next mutation has since made it.
            if let Some(repo_ref) = conflict_probe.filter(|_| !was_canceled(&dispatched)) {
                let backend = Arc::clone(&backend);
                if let Ok(Ok(paths)) = tokio::task::spawn_blocking(move || {
                    backend.conflicts(&repo_ref).map(|sets| {
                        sets.into_iter()
                            .map(|c| c.path.to_string_lossy().into_owned())
                            .collect::<Vec<_>>()
                    })
                })
                .await
                {
                    if !paths.is_empty() {
                        if let Some(repo) = repo.clone() {
                            let _ = events_tx.send(Event::ConflictDetected { repo, paths });
                        }
                    }
                }
            }
            dispatched
        };

        let mut dispatched = match dispatched {
            Ok(dispatched) => dispatched,
            Err(_) => {
                // The operation panicked: answer with an error (the lock
                // guard was released with the block above), never leave
                // the client waiting.
                let response = crate::dispatch::panicked(request_id, command);
                if let Some(job) = job {
                    job.finish(&response.result);
                }
                if out_tx
                    .send(ServerMessage::Response(response))
                    .await
                    .is_err()
                {
                    break;
                }
                continue;
            }
        };
        if let Some(job) = job {
            note_job(&mut dispatched.response, job.id());
            job.finish(&dispatched.response.result);
        }
        if is_canceled_response(&dispatched) && command != OperationKind::Clone {
            if let Some(repo) = repo.clone() {
                announce_canceled(&events_tx, &read_cache, repo, command);
            }
        }
        let journal_state = dispatched.journal.take();
        let outside_state = dispatched.outside.take();
        let pending_changed = dispatched.pending_auto_stash_changed;
        let pending_gone = dispatched.pending_auto_stash_gone;

        if out_tx
            .send(ServerMessage::Response(dispatched.response))
            .await
            .is_err()
        {
            break;
        }

        if pending_changed {
            if let Some(repo) = repo.clone() {
                if let Some(stash) = pending_gone {
                    let _ = events_tx.send(Event::PendingAutoStashGone {
                        repo: repo.clone(),
                        stash,
                    });
                }
                let _ = events_tx.send(Event::PendingAutoStashChanged { repo });
            }
        }
        if let (Some(state), Some(repo)) = (outside_state, repo.clone()) {
            announce_outside(&events_tx, &read_cache, repo, state);
        }
        if dispatched.mutated {
            if let Some(repo) = repo {
                after_mutation(&events_tx, &read_cache, repo.clone(), command);
                if let Some(state) = journal_state {
                    let _ = events_tx.send(Event::JournalChanged { repo, state });
                }
            }
        }
    }

    reader.abort();
    writer_task.abort();

    // Windows this connection attached stay known for a short grace, so a
    // TUI restarting into another tab keeps the intents it had not answered.
    for viewer in viewers.detach_conn(conn_ui.id) {
        let viewers = Arc::clone(&viewers);
        let conn = conn_ui.id;
        tokio::spawn(async move {
            tokio::time::sleep(viewers.grace()).await;
            viewers.expire(&viewer, conn);
        });
    }
}

/// How an admitted request holds its lock.
enum Admission {
    /// A read: the shared repo lock, already taken in arrival order;
    /// answered from the read cache when it can.
    Read(tokio::sync::OwnedRwLockReadGuard<()>),
    /// A transfer touching only refs and objects: the repository's transfer
    /// lock, taken on the request's own task, and the change events once it
    /// has answered.
    Transfer(Arc<tokio::sync::Mutex<()>>),
}

/// A request admitted to run on its own task.
struct Admitted {
    request: Request,
    repo: RepoId,
    admission: Admission,
    received: std::time::Instant,
    /// The job a transfer is (always set for one).
    job: Option<JobHandle>,
}

/// Runs an admitted request on its own task: dispatch, release the lock,
/// log the timing, answer the client, and for a transfer that changed
/// something announce it.
fn run_admitted(
    admitted: Admitted,
    backend: Arc<dyn GitBackend>,
    (out_tx, raw_tx): (mpsc::Sender<ServerMessage>, mpsc::Sender<RawResponse>),
    events_tx: broadcast::Sender<Event>,
    read_cache: Arc<ReadCache>,
    inflight: mpsc::Sender<()>,
) {
    let Admitted {
        request,
        repo,
        admission,
        received,
        job,
    } = admitted;
    let (request_id, command) = (request.id, request.command);
    let transfer = matches!(admission, Admission::Transfer(_));
    tokio::spawn(async move {
        let _inflight = inflight;
        let (label, started, dispatched) = match admission {
            Admission::Read(guard) => {
                let started = std::time::Instant::now();
                let cache = Arc::clone(&read_cache);
                // A history rewrite preview is a job: it reports progress and
                // stops when the job is canceled.
                let preview = job.as_ref().map(|job| {
                    job.running();
                    crate::rewrite_runs::Control::new(job.ctl(), repo.clone(), events_tx.clone())
                });
                let result = tokio::task::spawn_blocking(move || {
                    if request.command == OperationKind::Search {
                        return crate::search::reply(&cache, &backend, &request);
                    }
                    if let Some(control) = preview {
                        return CachedReply::Dispatched(Box::new(
                            control.run(|| dispatch(backend.as_ref(), &request)),
                        ));
                    }
                    dispatch_cached_reply(&cache, backend.as_ref(), &request)
                })
                .await;
                if let Some(job) = job {
                    let outcome: JobResult = match &result {
                        Ok(CachedReply::Dispatched(d)) => d.response.result.clone(),
                        Ok(CachedReply::Json { .. }) => Ok(serde_json::Value::Null),
                        Err(_) => crate::dispatch::panicked(request_id, command).result,
                    };
                    job.finish(&outcome);
                }
                drop(guard);
                ("read", started, result)
            }
            Admission::Transfer(lock) => {
                // A transfer waiting for the lock is a `Queued` job and can
                // be canceled: the wait is dropped and git never starts.
                let held = match &job {
                    Some(job) => tokio::select! {
                        () = job.cancel_requested() => None,
                        held = lock.lock_owned() => Some(held),
                    },
                    None => Some(lock.lock_owned().await),
                };
                let started = std::time::Instant::now();
                let mut result = if held.is_some() {
                    if let Some(job) = &job {
                        job.running();
                    }
                    network_cancellable(job.as_ref(), backend, request, events_tx.clone()).await
                } else {
                    Ok(canceled_dispatch(
                        job.as_ref(),
                        &CanceledCtx::of(&request),
                        &events_tx,
                    ))
                };
                if let (Ok(d), Some(job)) = (&mut result, &job) {
                    note_job(&mut d.response, job.id());
                }
                if let Some(job) = job {
                    let outcome: JobResult = match &result {
                        Ok(d) => d.response.result.clone(),
                        Err(_) => crate::dispatch::panicked(request_id, command).result,
                    };
                    job.finish(&outcome);
                }
                if matches!(&result, Ok(d) if is_canceled_response(d)) {
                    announce_canceled(&events_tx, &read_cache, repo.clone(), command);
                }
                drop(held);
                (
                    "transfer",
                    started,
                    result.map(|d| CachedReply::Dispatched(Box::new(d))),
                )
            }
        };
        let waited = started - received;
        gitpoulpi_ipc::timing::record("daemon", || {
            format!(
                "{} {label}: waited {} ran {}",
                timing_label(request_id, command),
                gitpoulpi_ipc::timing::ms(waited),
                gitpoulpi_ipc::timing::ms(started.elapsed()),
            )
        });
        let mut outside = None;
        let (response, mutated, pending_changed, pending_gone) = match dispatched {
            Ok(CachedReply::Json {
                request_id,
                json,
                pending_auto_stash_changed,
                pending_auto_stash_gone,
            }) => {
                let _ = raw_tx.send(RawResponse { request_id, json }).await;
                if pending_auto_stash_changed {
                    if let Some(stash) = pending_auto_stash_gone {
                        let _ = events_tx.send(Event::PendingAutoStashGone {
                            repo: repo.clone(),
                            stash,
                        });
                    }
                    let _ = events_tx.send(Event::PendingAutoStashChanged { repo });
                }
                return;
            }
            Ok(CachedReply::Dispatched(d)) => {
                outside = d.outside;
                (
                    d.response,
                    d.mutated,
                    d.pending_auto_stash_changed,
                    d.pending_auto_stash_gone,
                )
            }
            Err(_) => (
                crate::dispatch::panicked(request_id, command),
                false,
                false,
                None,
            ),
        };
        let sent = out_tx.send(ServerMessage::Response(response)).await;
        if let Some(state) = outside {
            announce_outside(&events_tx, &read_cache, repo.clone(), state);
        }
        if pending_changed {
            if let Some(stash) = pending_gone {
                let _ = events_tx.send(Event::PendingAutoStashGone {
                    repo: repo.clone(),
                    stash,
                });
            }
            let _ = events_tx.send(Event::PendingAutoStashChanged { repo: repo.clone() });
        }
        if transfer && mutated && sent.is_ok() {
            after_mutation(&events_tx, &read_cache, repo, command);
        }
    });
}

/// A failed request that became job `id` says so in its context: the number
/// the Jobs list shows.
fn note_job(response: &mut Response, id: u64) {
    if let Err(error) = &mut response.result {
        if let Some(context) = &mut error.context {
            context.job = Some(id);
        }
    }
}

/// What the connection loop does after a control request.
enum Flow {
    Continue,
    Stop,
}

/// Daemon-control pseudo-operations, intercepted before they ever reach
/// `dispatch`/`GitBackend` - see `OperationKind::{Hello, Shutdown}`'s doc
/// comments. Neither takes a repo lock. `None`: not a control request.
async fn handle_control(
    request: &Request,
    info: &DaemonInfo,
    out_tx: &mpsc::Sender<ServerMessage>,
    events_tx: &broadcast::Sender<Event>,
    internal_shutdown_tx: &mpsc::Sender<()>,
) -> Option<Flow> {
    let request_id = request.id;
    match request.command {
        OperationKind::Hello => {
            let value = serde_json::to_value(info).unwrap_or(serde_json::Value::Null);
            let sent = out_tx
                .send(ServerMessage::Response(Response {
                    request_id,
                    result: Ok(value),
                }))
                .await
                .is_ok();
            Some(if sent { Flow::Continue } else { Flow::Stop })
        }
        OperationKind::CredentialGrant => {
            let result = match serde_json::from_value::<
                gitpoulpi_ipc::git_credential::CredentialGrantRequest,
            >(request.args.clone())
            {
                Ok(grant) => Ok(serde_json::to_value(crate::grants::answer(&grant).await)
                    .unwrap_or(serde_json::Value::Null)),
                Err(e) => Err(gitpoulpi_ipc::ResponseError {
                    message: format!("invalid arguments: {e}"),
                    unsupported: false,
                    kind: None,
                    detail: None,
                    context: None,
                    auth_required: false,
                }),
            };
            let sent = out_tx
                .send(ServerMessage::Response(Response { request_id, result }))
                .await
                .is_ok();
            Some(if sent { Flow::Continue } else { Flow::Stop })
        }
        OperationKind::Shutdown => {
            let sent = out_tx
                .send(ServerMessage::Response(Response {
                    request_id,
                    result: Ok(serde_json::Value::Null),
                }))
                .await
                .is_ok();
            // A reason (the updater's `update`) is announced first so a frontend
            // can tell "replaced by an update, do not restart a daemon" from a
            // plain stop; `DaemonStopping` always follows.
            if let Some(reason) = gitpoulpi_ipc::ShutdownArgs::from_value(&request.args).reason {
                let _ = events_tx.send(Event::ShuttingDown {
                    session: info.session.clone(),
                    reason,
                });
            }
            let _ = events_tx.send(Event::DaemonStopping {
                session: info.session.clone(),
            });
            if sent {
                // Give the writer task a moment to flush the response and
                // the broadcast event before this connection (and, via
                // `internal_shutdown_tx`, the whole daemon) tears down.
                // Best-effort.
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
            let _ = internal_shutdown_tx.send(()).await;
            Some(Flow::Stop)
        }
        _ => None,
    }
}

/// The locks an exclusive request holds: the transfer lock for a network
/// operation (taken first, so it never holds the repository lock while it
/// waits for a transfer), then the repository lock, exclusively.
async fn acquire_exclusive(
    repo: &Option<RepoId>,
    network: bool,
    transfer_locks: &TransferLocks,
    repo_locks: &RepoLocks,
) -> (
    Option<tokio::sync::OwnedMutexGuard<()>>,
    Option<tokio::sync::OwnedRwLockWriteGuard<()>>,
) {
    let transfer = match repo {
        Some(repo) if network => Some(repo_lock(transfer_locks, repo).lock_owned().await),
        _ => None,
    };
    let guard = match repo {
        Some(repo) => Some(repo_lock(repo_locks, repo).write_owned().await),
        None => None,
    };
    (transfer, guard)
}

/// What a canceled request needs to say about itself.
struct CanceledCtx {
    id: u64,
    command: OperationKind,
    url: Option<String>,
    dest: Option<std::path::PathBuf>,
}

impl CanceledCtx {
    fn of(request: &Request) -> Self {
        let text = |name: &str| {
            request
                .args
                .get(name)
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        };
        Self {
            id: request.id,
            command: request.command,
            url: text("url"),
            dest: text("dest").map(std::path::PathBuf::from),
        }
    }
}

/// The answer of a request whose job was canceled: the typed
/// `Error::Canceled` for everyone waiting on it. A clone canceled in its LFS
/// phase is the exception: the clone is valid and stays, so it answers as
/// done (the LFS objects are the part that was not downloaded).
fn canceled_dispatch(
    job: Option<&JobHandle>,
    ctx: &CanceledCtx,
    events: &broadcast::Sender<Event>,
) -> crate::dispatch::Dispatched {
    if let (OperationKind::Clone, Some(dest)) = (ctx.command, &ctx.dest) {
        let url = ctx.url.clone().unwrap_or_default();
        if job.is_some_and(JobHandle::keeps_clone_on_cancel) {
            let repo = gitpoulpi_core::domain::RepoRef::from_root(dest);
            let _ = events.send(Event::CloneFinished {
                url,
                dest: dest.clone(),
                result: Ok(repo.id.clone()),
            });
            if let Ok(json) = serde_json::to_value(repo) {
                return crate::dispatch::ok(ctx.id, json, false);
            }
        } else {
            let _ = events.send(Event::CloneFinished {
                url,
                dest: dest.clone(),
                result: Err("canceled".into()),
            });
        }
    }
    let error = gitpoulpi_core::Error::Canceled {
        job: job.map_or(0, JobHandle::id),
    };
    crate::dispatch::err_kind(ctx.id, error.to_string(), false, error.typed())
}

fn is_canceled_response(dispatched: &crate::dispatch::Dispatched) -> bool {
    matches!(
        &dispatched.response.result,
        Err(e) if matches!(e.kind, Some(gitpoulpi_core::TypedError::Canceled { .. }))
    )
}

fn was_canceled(
    dispatched: &std::result::Result<crate::dispatch::Dispatched, tokio::task::JoinError>,
) -> bool {
    matches!(dispatched, Ok(d) if is_canceled_response(d))
}

/// A transfer was stopped part-way: it may have moved refs before it died (or
/// a push may have reached the remote), so every client re-reads.
fn announce_canceled(
    events_tx: &broadcast::Sender<Event>,
    read_cache: &ReadCache,
    repo: RepoId,
    command: OperationKind,
) {
    read_cache.invalidate(&repo);
    let _ = events_tx.send(Event::RepoChanged {
        repo,
        scope: command.traits().scope,
    });
}

/// [`dispatch_network`] on its own task, so a panic inside it surfaces as
/// a `JoinError` the caller answers, instead of unwinding the task that
/// owes the client a response. When `job` is canceled the task is aborted and
/// awaited: its futures are dropped, which kills git and everything it
/// started and cleans up after it (`gitpoulpi-cli-backend`), before this
/// returns the canceled answer.
async fn network_cancellable(
    job: Option<&JobHandle>,
    backend: Arc<dyn GitBackend>,
    request: Request,
    events: broadcast::Sender<Event>,
) -> std::result::Result<crate::dispatch::Dispatched, tokio::task::JoinError> {
    let ctx = CanceledCtx::of(&request);
    let task_events = events.clone();
    let ctl = job.map(JobHandle::ctl);
    let mut task = tokio::spawn(async move {
        let work = async {
            crate::test_hook::maybe_panic(request.command);
            crate::test_hook::transfer_delay().await;
            dispatch_network(backend.as_ref(), &request, &task_events).await
        };
        match ctl {
            Some(ctl) => jobs::run_in(ctl, work).await,
            None => work.await,
        }
    });
    let Some(job) = job else {
        return task.await;
    };
    tokio::select! {
        done = &mut task => done,
        () = job.abort_requested() => {
            task.abort();
            let _ = task.await;
            Ok(canceled_dispatch(Some(job), &ctx, &events))
        }
    }
}

/// Changes made outside GitPoulpi were just recorded as journal entries:
/// every client re-reads (the refs moved) and sees the new entries.
fn announce_outside(
    events_tx: &broadcast::Sender<Event>,
    read_cache: &ReadCache,
    repo: RepoId,
    state: gitpoulpi_core::domain::JournalState,
) {
    read_cache.invalidate(&repo);
    let _ = events_tx.send(Event::RepoChanged {
        repo: repo.clone(),
        scope: ChangeScope::ALL,
    });
    let _ = events_tx.send(Event::JournalChanged { repo, state });
}

/// Drops the read cache entries a mutation can have made stale, then tells
/// every client. Done *after* the operation, so a read admitted while a
/// shared-lock transfer was running cannot leave a pre-transfer answer
/// behind (the cache's ref fingerprint also changes; this is the belt).
fn after_mutation(
    events_tx: &broadcast::Sender<Event>,
    read_cache: &ReadCache,
    repo: RepoId,
    command: OperationKind,
) {
    let scope = command.traits().scope;
    if scope.intersects(ChangeScope::REFS | ChangeScope::LOG | ChangeScope::CONFIG) {
        read_cache.invalidate(&repo);
    } else if scope.intersects(ChangeScope::STASHES | ChangeScope::WORKTREES) {
        // The search index lists stashes and worktrees.
        read_cache.search.invalidate(&repo);
    }
    if command == OperationKind::SetRefVisibility {
        let _ = events_tx.send(Event::RefVisibilityChanged { repo: repo.clone() });
    }
    if matches!(
        command,
        OperationKind::RepoAuthSet | OperationKind::RepoAuthClear
    ) {
        let _ = events_tx.send(Event::RepoAuthChanged { repo: repo.clone() });
    }
    let _ = events_tx.send(Event::RepoChanged {
        repo: repo.clone(),
        scope,
    });
    let _ = events_tx.send(Event::OperationCompleted {
        repo,
        operation: Operation {
            kind: command,
            description: format!("{command:?}"),
            timestamp: gitpoulpi_core::time::unix_now(),
            auto_stashed: false,
            succeeded: true,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RepoRef;

    fn locks_for_one_repo() -> (RepoLocks, RepoId) {
        (
            Arc::new(StdMutex::new(HashMap::new())),
            RepoRef::from_root(&std::env::temp_dir()).id,
        )
    }

    /// The pre-serialized frame must be byte-for-byte what serializing the
    /// `ServerMessage` would have produced, or clients would see a different
    /// wire format.
    #[tokio::test]
    async fn a_cached_response_frame_matches_the_serialized_message() {
        let value = serde_json::json!({"nodes": [{"summary": "a \"b\""}], "n": 1.5});
        let json: Arc<[u8]> = serde_json::to_vec(&value).unwrap().into();
        let mut raw = Vec::new();
        RawResponse {
            request_id: 42,
            json,
        }
        .write(&mut raw)
        .await
        .unwrap();
        let mut expected = Vec::new();
        write_message(
            &mut expected,
            &ServerMessage::Response(Response {
                request_id: 42,
                result: Ok(value),
            }),
        )
        .await
        .unwrap();
        assert_eq!(raw, expected);
    }

    #[test]
    fn repo_locks_of_repos_nobody_uses_are_pruned() {
        let (locks, first) = locks_for_one_repo();
        let held = repo_lock(&locks, &first);
        let other = RepoId("/some/other/repo".into());
        drop(repo_lock(&locks, &other));
        let third = RepoId("/a/third/repo".into());
        let _ = repo_lock(&locks, &third);
        let map = locks.lock().unwrap();
        assert!(map.contains_key(&first), "a held lock stays");
        assert!(!map.contains_key(&other), "an idle one is dropped");
        drop(held);
    }

    #[test]
    fn a_stage_broadcast_is_index_only() {
        let scope = OperationKind::StageFile.traits().scope;
        assert!(scope.contains(ChangeScope::INDEX));
        for wide in [
            ChangeScope::LOG,
            ChangeScope::REFS,
            ChangeScope::STASHES,
            ChangeScope::CONFIG,
            ChangeScope::LFS,
        ] {
            assert!(!scope.intersects(wide), "{wide:?} must not be in {scope:?}");
        }
        assert!(!scope.touches_lfs());
        assert_eq!(OperationKind::Merge.traits().scope, ChangeScope::ALL);
        assert!(OperationKind::BranchCreate
            .traits()
            .scope
            .contains(ChangeScope::LOG));
    }

    /// A read admitted while another read is running starts at once.
    #[tokio::test]
    async fn two_reads_of_one_repository_hold_its_lock_together() {
        let (locks, repo) = locks_for_one_repo();
        let _first = repo_lock(&locks, &repo).read_owned().await;
        let second = tokio::time::timeout(
            Duration::from_millis(200),
            repo_lock(&locks, &repo).read_owned(),
        )
        .await;
        assert!(second.is_ok(), "a second read must not wait for the first");
    }

    #[tokio::test]
    async fn a_mutation_waits_for_the_reads_before_it_and_holds_off_the_ones_after() {
        let (locks, repo) = locks_for_one_repo();
        let read = repo_lock(&locks, &repo).read_owned().await;

        let lock = repo_lock(&locks, &repo);
        let mut write = tokio::spawn(async move { lock.write_owned().await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!write.is_finished(), "the mutation waits for the read");

        // A read arriving while the mutation is queued goes after it --
        // otherwise a stream of reads (the 2-second refresh) could keep
        // a commit waiting forever.
        let lock = repo_lock(&locks, &repo);
        let later_read = tokio::spawn(async move { lock.read_owned().await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!later_read.is_finished(), "the later read queues behind it");

        drop(read);
        let write_guard = (&mut write).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !later_read.is_finished(),
            "still behind the running mutation"
        );
        drop(write_guard);
        later_read.await.unwrap();
    }
}
