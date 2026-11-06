//! The daemon's job registry: every network request is a *job* from the
//! moment it is received, so a request waiting for a lock is visible
//! (`Queued`), a second identical one joins it instead of running again, and
//! any of them can be canceled. See the background-ops design`
//! (sections 1 and 2) for the design and the per-operation cancel policy this
//! implements.
//!
//! The registry knows nothing about git. The server decides when a job runs
//! (it owns the locks) and drops the future of a canceled one; dropping it is
//! what kills git and cleans up (`gitpoulpi-cli-backend`, `tree_kill`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{ClonePhase, OperationKind, RepoId};
use gitpoulpi_core::{Error, NotCancellableReason};
use gitpoulpi_ipc::{Event, JobInfo, JobState, Request, Response, ResponseError, ServerMessage};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, watch};

/// How long a finished job stays in the list.
const RETENTION: Duration = Duration::from_secs(30);
/// Progress-only changes are broadcast at most this often per job.
const PROGRESS_EVERY: Duration = Duration::from_millis(100);

/// What a job answered with, as each waiting caller receives it.
pub(crate) type JobResult = std::result::Result<Value, ResponseError>;

/// What `submit` decided about a request.
pub(crate) enum Submitted {
    /// A new job: the caller runs it.
    New(JobHandle),
    /// An identical job is queued or running: wait for its result.
    Joined {
        job: u64,
        done: watch::Receiver<Option<JobResult>>,
    },
    /// Refused: another job already does this (`Error::JobConflict`).
    Conflict(u64),
    /// An automatic fetch while the repository is busy: answered at once with
    /// an empty outcome, nothing runs.
    Skipped,
}

/// Whether `request` becomes a job: every network operation, and the two
/// history rewrite steps that take long (the preview and the apply).
pub(crate) fn is_job(request: &Request) -> bool {
    request.command.traits().network
        || matches!(
            request.command,
            OperationKind::RewritePreview
                | OperationKind::RewriteApply
                | OperationKind::RewriteScanSecrets
                | OperationKind::RewriteAnalyze
                | OperationKind::RewritePurge
        )
}

/// How far a running job may be stopped, per operation (the plan's table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// Fetches, pushes, remote deletes, LFS transfers and locks: stop at any
    /// moment (a push's outcome is then uncertain; the TUI says so).
    Free,
    /// A clone: stop at any moment and the destination goes back as it was,
    /// except in the LFS phase, where the clone is valid and is kept.
    Clone,
    /// Only while queued: running, it is past the point where stopping is
    /// safe.
    QueuedOnly(NotCancellableReason),
    /// Queued, and running until the work reaches its point of no return (a
    /// pull's merge, an LFS pull's checkout), which it announces with
    /// [`enter_point`]; from then on refused with the reason.
    UntilPoint(NotCancellableReason),
    /// Stops between steps only, on the job's own cooperation: the running
    /// step finishes (a workflow). The task is never aborted.
    Steps,
}

fn policy_for(kind: OperationKind) -> Policy {
    use OperationKind as K;
    match kind {
        K::Clone => Policy::Clone,
        // A pull is a download (`fetch`) then an integrate that rewrites the
        // index and the working tree: stoppable until the integrate starts.
        K::Pull => Policy::UntilPoint(NotCancellableReason::Integrating),
        // `git lfs fetch` then the checkout of the real files.
        K::LfsPull => Policy::UntilPoint(NotCancellableReason::RewritingFiles),
        K::LfsCheckout => Policy::QueuedOnly(NotCancellableReason::RewritingFiles),
        // Multi-step: the flow runner looks at the cancel request between
        // steps and stops there; the step running finishes.
        K::FlowRun => Policy::Steps,
        // A preview only writes a temporary pack: stop it at any moment, it
        // leaves nothing behind (the sink breaks, the run directory goes).
        K::RewritePreview => Policy::Free,
        // A scan only reads.
        K::RewriteScanSecrets => Policy::Free,
        K::RewriteAnalyze => Policy::Free,
        // Apply is queued, then past its point of no return: the refs move
        // in one transaction that must not be cut off.
        K::RewriteApply => Policy::UntilPoint(NotCancellableReason::RewritingFiles),
        // A purge deletes pins, reflogs and objects: queued, then not
        // stoppable (half a purge is an undo entry without its objects).
        K::RewritePurge => Policy::UntilPoint(NotCancellableReason::Purging),
        _ => Policy::Free,
    }
}

/// A short English description of a request.
fn label_for(request: &Request) -> String {
    use OperationKind as K;
    let arg = |name: &str| {
        request
            .args
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let remote = arg("remote").unwrap_or_else(|| "the remote".into());
    match request.command {
        K::Clone => format!(
            "Cloning into {}",
            arg("dest").unwrap_or_else(|| "a new folder".into())
        ),
        K::Fetch => format!("Fetching {remote}"),
        K::FetchAll => "Fetching all remotes".into(),
        K::FetchRefspec | K::FetchRefAsBranch => format!("Fetching from {remote}"),
        K::Pull => "Pulling".into(),
        K::Push => {
            let branch = request
                .args
                .pointer("/opts/branch")
                .and_then(Value::as_str)
                .unwrap_or("the current branch");
            format!("Pushing {branch} to {remote}")
        }
        K::PushForReview => format!("Pushing for review to {remote}"),
        K::TagPush => format!("Pushing tags to {remote}"),
        K::TagDeleteRemote | K::BranchDeleteRemote => format!("Deleting on {remote}"),
        K::TagFetch => format!("Fetching a tag from {remote}"),
        K::LfsFetch => "Fetching LFS objects".into(),
        K::LfsPull => "Pulling LFS objects".into(),
        K::LfsPush => "Pushing LFS objects".into(),
        K::LfsCheckout => "Checking out LFS files".into(),
        K::LfsLocks | K::LfsLock | K::LfsUnlock => "Talking to the LFS server".into(),
        K::FlowRun => "Running a workflow".into(),
        K::RewritePreview => "Previewing a history rewrite".into(),
        K::RewriteScanSecrets => "Scanning history for secrets".into(),
        K::RewriteAnalyze => "Analysing history sizes".into(),
        K::RewriteApply => "Rewriting history".into(),
        K::RewritePurge => "Purging the old history".into(),
        K::RewritePush => "Pushing the rewritten history".into(),
        other => format!("{other:?}"),
    }
}

/// Which other job a new request joins or is refused by.
enum Dedupe {
    /// Runs on its own (queues behind the others as before).
    No,
    /// Joins a queued or running job with `key` (or with `also`).
    Join { key: String, also: Option<String> },
    /// Refused while a job with `key` is queued or running.
    Reject { key: String },
}

/// The destination of a clone as one string, equal for the spellings of the
/// same folder (separators, a trailing one; case on Windows).
fn normalize_dest(dest: &str) -> String {
    let text = dest.replace('\\', "/");
    let text = text.trim_end_matches('/').to_string();
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

fn dedupe_for(request: &Request) -> Dedupe {
    use OperationKind as K;
    let repo = request.repo.as_ref().map_or("", |r| r.0.as_str());
    // What changes the outcome is the arguments; `auto` only says who asked.
    let mut args = request.args.clone();
    if let Some(map) = args.as_object_mut() {
        map.remove("auto");
        map.remove(gitpoulpi_ipc::CANCEL_ON_DISCONNECT);
    }
    let key = |kind: K, args: &Value| format!("{kind:?}|{repo}|{args}");
    match request.command {
        K::Clone => match request.args.get("dest").and_then(Value::as_str) {
            Some(dest) => Dedupe::Reject {
                key: format!("Clone|{}", normalize_dest(dest)),
            },
            None => Dedupe::No,
        },
        // A fetch of one remote is covered by a fetch of all of them.
        K::Fetch => {
            let opts = args.get("opts").cloned().unwrap_or(Value::Null);
            Dedupe::Join {
                key: key(K::Fetch, &args),
                also: Some(key(K::FetchAll, &serde_json::json!({ "opts": opts }))),
            }
        }
        // One live preview and one apply per repository.
        K::RewritePreview
        | K::RewriteApply
        | K::RewriteScanSecrets
        | K::RewriteAnalyze
        | K::RewritePush
        | K::RewritePurge => Dedupe::Reject {
            key: format!("{:?}|{repo}", request.command),
        },
        K::FetchAll
        | K::FetchRefspec
        | K::Pull
        | K::Push
        | K::PushForReview
        | K::LfsFetch
        | K::LfsPull
        | K::LfsPush => Dedupe::Join {
            key: key(request.command, &args),
            also: None,
        },
        _ => Dedupe::No,
    }
}

struct Entry {
    info: JobInfo,
    key: Option<String>,
    policy: Policy,
    cancel: watch::Sender<bool>,
    done: watch::Sender<Option<JobResult>>,
    /// Where to answer the originator if the job is canceled before the
    /// server ever picks it up (it may be queued behind a running request of
    /// its own connection).
    unclaimed: Option<(mpsc::Sender<ServerMessage>, u64)>,
    started: Option<Instant>,
    last_emit: Instant,
    /// A clone canceled in its LFS phase keeps the (valid) clone.
    keep_on_cancel: bool,
    /// The work announced its point of no return ([`Policy::UntilPoint`]).
    past_point: bool,
}

impl Entry {
    fn refresh(&mut self) {
        self.info.not_cancellable = match (self.info.state, self.policy) {
            (JobState::Running, Policy::QueuedOnly(reason)) => Some(reason),
            (JobState::Running, Policy::UntilPoint(reason)) if self.past_point => Some(reason),
            (state, _) if state.is_finished() => Some(NotCancellableReason::Finished),
            _ => None,
        };
    }
}

struct Inner {
    next: u64,
    jobs: HashMap<u64, Entry>,
}

/// See the module docs.
pub(crate) struct Registry {
    inner: Mutex<Inner>,
    events: broadcast::Sender<Event>,
}

/// The effect of a `JobCancel` the caller still has to carry out.
pub(crate) struct Canceled {
    pub info: JobInfo,
    /// A job nobody had started: its originator's answer, to send now.
    pub respond: Option<(mpsc::Sender<ServerMessage>, Response)>,
}

pub(crate) fn canceled_error(job: u64) -> ResponseError {
    let error = Error::Canceled { job };
    ResponseError {
        message: error.to_string(),
        unsupported: false,
        kind: error.typed(),
        detail: None,
        auth_required: false,
    }
}

impl Registry {
    /// A registry broadcasting on `events`, and following the clone and LFS
    /// progress events on it to keep its jobs' progress current.
    pub(crate) fn new(events: broadcast::Sender<Event>) -> Arc<Self> {
        let registry = Arc::new(Self {
            inner: Mutex::new(Inner {
                next: 1,
                jobs: HashMap::new(),
            }),
            events: events.clone(),
        });
        let weak: Weak<Registry> = Arc::downgrade(&registry);
        let mut rx = events.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        let Some(registry) = weak.upgrade() else {
                            return;
                        };
                        registry.observe(&event);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        registry
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn emit(&self, info: JobInfo) {
        let _ = self.events.send(Event::JobChanged { job: info });
    }

    /// Progress events of a clone or an LFS transfer reach their job.
    fn observe(&self, event: &Event) {
        match event {
            Event::CloneProgress { dest, progress, .. } => {
                let dest = normalize_dest(&dest.to_string_lossy());
                let mut inner = self.lock();
                let Some(entry) = inner.jobs.values_mut().find(|e| {
                    e.info.kind == OperationKind::Clone
                        && !e.info.state.is_finished()
                        && e.info
                            .dest
                            .as_ref()
                            .is_some_and(|d| normalize_dest(&d.to_string_lossy()) == dest)
                }) else {
                    return;
                };
                let phase = format!("{:?}", progress.phase);
                let phase_changed = entry.info.phase.as_deref() != Some(phase.as_str());
                entry.info.phase = Some(phase);
                entry.info.progress = Some(progress.percent);
                entry.keep_on_cancel = progress.phase == ClonePhase::LfsObjects;
                if phase_changed || entry.last_emit.elapsed() >= PROGRESS_EVERY {
                    entry.last_emit = Instant::now();
                    let info = entry.info.clone();
                    drop(inner);
                    self.emit(info);
                }
            }
            Event::LfsProgress {
                repo,
                operation,
                progress,
            } => {
                let mut inner = self.lock();
                let Some(entry) = inner.jobs.values_mut().find(|e| {
                    e.info.kind == *operation
                        && !e.info.state.is_finished()
                        && e.info.repo.as_ref() == Some(repo)
                }) else {
                    return;
                };
                entry.info.progress = Some(progress.percent());
                if entry.last_emit.elapsed() >= PROGRESS_EVERY {
                    entry.last_emit = Instant::now();
                    let info = entry.info.clone();
                    drop(inner);
                    self.emit(info);
                }
            }
            _ => {}
        }
    }

    /// Registers `request` as a job, or finds the one it joins / conflicts
    /// with. `responder` is where the originator's answer goes if the job is
    /// canceled before the server picks it up.
    pub(crate) fn submit(
        self: &Arc<Self>,
        request: &Request,
        responder: mpsc::Sender<ServerMessage>,
    ) -> Submitted {
        let dedupe = dedupe_for(request);
        let mut inner = self.lock();
        let live = |e: &&mut Entry| !e.info.state.is_finished();

        // An automatic fetch yields to anything already moving data in the
        // same repository.
        let automatic = request.args.get("auto").and_then(Value::as_bool) == Some(true);
        if automatic
            && matches!(
                request.command,
                OperationKind::Fetch | OperationKind::FetchAll
            )
            && inner
                .jobs
                .values_mut()
                .filter(live)
                .any(|e| e.info.repo == request.repo)
        {
            return Submitted::Skipped;
        }

        let key = match dedupe {
            Dedupe::No => None,
            Dedupe::Reject { key } => {
                if let Some(existing) = inner
                    .jobs
                    .values_mut()
                    .filter(live)
                    .find(|e| e.key.as_deref() == Some(key.as_str()))
                {
                    return Submitted::Conflict(existing.info.id);
                }
                Some(key)
            }
            Dedupe::Join { key, also } => {
                let found = inner.jobs.values_mut().filter(live).find(|e| {
                    e.key.as_deref() == Some(key.as_str()) || (also.is_some() && e.key == also)
                });
                if let Some(entry) = found {
                    entry.info.joiners += 1;
                    let (job, done, info) =
                        (entry.info.id, entry.done.subscribe(), entry.info.clone());
                    drop(inner);
                    self.emit(info);
                    return Submitted::Joined { job, done };
                }
                Some(key)
            }
        };

        let id = inner.next;
        inner.next += 1;
        let repo: Option<RepoId> = request.repo.clone();
        // The job whose lock this one waits for: the oldest live job of the
        // same repository.
        let queued_after = repo.as_ref().and_then(|repo| {
            inner
                .jobs
                .values()
                .filter(|e| !e.info.state.is_finished() && e.info.repo.as_ref() == Some(repo))
                .map(|e| e.info.id)
                .min()
        });
        let (cancel, cancel_rx) = watch::channel(false);
        let (done, _) = watch::channel(None);
        let mut entry = Entry {
            info: JobInfo {
                id,
                kind: request.command,
                repo,
                dest: request
                    .args
                    .get("dest")
                    .and_then(Value::as_str)
                    .filter(|_| request.command == OperationKind::Clone)
                    .map(PathBuf::from),
                label: label_for(request),
                state: JobState::Queued,
                phase: None,
                progress: None,
                started_at: None,
                joiners: 0,
                queued_after,
                not_cancellable: None,
                error: None,
            },
            key,
            policy: policy_for(request.command),
            cancel,
            done,
            unclaimed: Some((responder, request.id)),
            started: None,
            last_emit: Instant::now(),
            keep_on_cancel: false,
            past_point: false,
        };
        entry.refresh();
        let info = entry.info.clone();
        inner.jobs.insert(id, entry);
        drop(inner);
        self.emit(info);
        Submitted::New(JobHandle {
            id,
            registry: Arc::clone(self),
            cancel: cancel_rx,
            finished: false,
        })
    }

    /// Every job, oldest first.
    pub(crate) fn list(&self) -> Vec<JobInfo> {
        let mut jobs: Vec<JobInfo> = self.lock().jobs.values().map(|e| e.info.clone()).collect();
        jobs.sort_by_key(|j| j.id);
        jobs
    }

    /// Whether `request` asked for its job to die with its connection.
    pub(crate) fn cancels_on_disconnect(request: &Request) -> bool {
        request
            .args
            .get(gitpoulpi_ipc::CANCEL_ON_DISCONNECT)
            .and_then(Value::as_bool)
            == Some(true)
    }

    /// Asks job `id` to stop, if its policy allows it right now.
    pub(crate) fn cancel(self: &Arc<Self>, id: u64) -> Result<Canceled, Error> {
        let mut inner = self.lock();
        let Some(entry) = inner.jobs.get_mut(&id) else {
            return Err(Error::NotCancellable {
                reason: NotCancellableReason::UnknownJob,
            });
        };
        if let Some(reason) = entry.info.not_cancellable {
            return Err(Error::NotCancellable { reason });
        }
        if entry.info.state == JobState::Cancelling {
            return Ok(Canceled {
                info: entry.info.clone(),
                respond: None,
            });
        }
        // Nobody started it (still behind a request of its own connection):
        // end it here and answer its originator now.
        if let Some((tx, request_id)) = entry.unclaimed.take() {
            let result: JobResult = Err(canceled_error(id));
            let response = Response {
                request_id,
                result: result.clone(),
            };
            let info = finish_entry(entry, result);
            drop(inner);
            self.emit(info.clone());
            self.schedule_removal(id);
            return Ok(Canceled {
                info,
                respond: Some((tx, response)),
            });
        }
        entry.info.state = JobState::Cancelling;
        entry.refresh();
        let _ = entry.cancel.send(true);
        let info = entry.info.clone();
        drop(inner);
        self.emit(info.clone());
        Ok(Canceled {
            info,
            respond: None,
        })
    }

    /// The job reports the part of the work it is in.
    fn set_phase(&self, id: u64, phase: &str) {
        let mut inner = self.lock();
        let Some(entry) = inner.jobs.get_mut(&id) else {
            return;
        };
        if entry.info.phase.as_deref() == Some(phase) {
            return;
        }
        entry.info.phase = Some(phase.to_string());
        let info = entry.info.clone();
        drop(inner);
        self.emit(info);
    }

    /// A job reports the part of the work it is in and how far it got
    /// (progress-only changes are throttled like a clone's).
    fn set_progress(&self, id: u64, phase: &str, percent: Option<u8>) {
        let mut inner = self.lock();
        let Some(entry) = inner.jobs.get_mut(&id) else {
            return;
        };
        if entry.info.state.is_finished() {
            return;
        }
        let phase_changed = entry.info.phase.as_deref() != Some(phase);
        entry.info.phase = Some(phase.to_string());
        if percent.is_some() {
            entry.info.progress = percent;
        }
        if phase_changed || entry.last_emit.elapsed() >= PROGRESS_EVERY {
            entry.last_emit = Instant::now();
            let info = entry.info.clone();
            drop(inner);
            self.emit(info);
        }
    }

    /// The work is about to pass its point of no return. `false`: cancel was
    /// requested first, so the work must not go on.
    fn enter_point(&self, id: u64, phase: &str) -> bool {
        let mut inner = self.lock();
        let Some(entry) = inner.jobs.get_mut(&id) else {
            return true;
        };
        if entry.info.state == JobState::Cancelling {
            return false;
        }
        entry.past_point = true;
        entry.info.phase = Some(phase.to_string());
        entry.refresh();
        let info = entry.info.clone();
        drop(inner);
        self.emit(info);
        true
    }

    /// The step after the point of no return changed nothing and failed: the
    /// job can be stopped again (a pull the working tree's changes blocked,
    /// before its auto-stash and second download).
    fn leave_point(&self, id: u64, phase: &str) {
        let mut inner = self.lock();
        let Some(entry) = inner.jobs.get_mut(&id) else {
            return;
        };
        entry.past_point = false;
        entry.info.phase = Some(phase.to_string());
        entry.refresh();
        let info = entry.info.clone();
        drop(inner);
        self.emit(info);
    }

    fn schedule_removal(self: &Arc<Self>, id: u64) {
        let registry = Arc::downgrade(self);
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        runtime.spawn(async move {
            tokio::time::sleep(RETENTION).await;
            if let Some(registry) = registry.upgrade() {
                registry.lock().jobs.remove(&id);
                let _ = registry.events.send(Event::JobRemoved { id });
            }
        });
    }
}

/// Marks `entry` finished with `result` and returns its new info.
fn finish_entry(entry: &mut Entry, result: JobResult) -> JobInfo {
    // A workflow asked to stop answers with its (partial) report; the job is
    // `Canceled` all the same.
    let canceled = matches!(
        &result,
        Err(e) if matches!(e.kind, Some(gitpoulpi_core::TypedError::Canceled { .. }))
    ) || (entry.info.state == JobState::Cancelling
        && matches!(entry.policy, Policy::Steps));
    entry.info.state = match (&result, canceled) {
        (_, true) => JobState::Canceled,
        (Ok(_), _) => JobState::Done,
        (Err(_), _) => JobState::Failed,
    };
    if let (Err(e), false) = (&result, canceled) {
        entry.info.error = Some(e.message.clone());
    }
    if entry.info.state == JobState::Done {
        entry.info.progress = Some(100);
    }
    entry.unclaimed = None;
    entry.refresh();
    let _ = entry.done.send(Some(result));
    entry.info.clone()
}

/// The runner's end of a job.
pub(crate) struct JobHandle {
    id: u64,
    registry: Arc<Registry>,
    cancel: watch::Receiver<bool>,
    finished: bool,
}

impl JobHandle {
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// The server takes the job up. `false`: it was canceled while it sat in
    /// a queue and has already been answered; run nothing.
    pub(crate) fn claim(&self) -> bool {
        let mut inner = self.registry.lock();
        match inner.jobs.get_mut(&self.id) {
            Some(entry) if entry.unclaimed.is_some() => {
                entry.unclaimed = None;
                true
            }
            _ => false,
        }
    }

    /// What the running work needs to talk to its job from inside the task
    /// (see [`run_in`]).
    pub(crate) fn ctl(&self) -> JobCtl {
        JobCtl {
            registry: Arc::clone(&self.registry),
            id: self.id,
            cancel: self.cancel.clone(),
        }
    }

    /// Resolves when the running work must be dropped: cancel was requested
    /// and the job is not one that stops itself between steps.
    pub(crate) async fn abort_requested(&self) {
        let cooperative = self
            .registry
            .lock()
            .jobs
            .get(&self.id)
            .is_some_and(|e| matches!(e.policy, Policy::Steps));
        if cooperative {
            std::future::pending::<()>().await;
        }
        self.cancel_requested().await;
    }

    /// Resolves when cancel has been requested (never, if it is not).
    pub(crate) async fn cancel_requested(&self) {
        let mut rx = self.cancel.clone();
        let _ = rx.wait_for(|requested| *requested).await;
        // A dropped sender means the registry is gone: never resolve.
        if self.registry.lock().jobs.contains_key(&self.id) {
            return;
        }
        std::future::pending::<()>().await;
    }

    /// The job got its lock and starts working.
    pub(crate) fn running(&self) {
        let mut inner = self.registry.lock();
        let Some(entry) = inner.jobs.get_mut(&self.id) else {
            return;
        };
        if entry.info.state != JobState::Queued {
            return;
        }
        entry.info.state = JobState::Running;
        entry.info.queued_after = None;
        entry.info.started_at = Some(gitpoulpi_core::time::unix_now());
        entry.started = Some(Instant::now());
        entry.refresh();
        let info = entry.info.clone();
        drop(inner);
        self.registry.emit(info);
    }

    /// Whether the clone, if canceled now, keeps what it has (LFS phase).
    pub(crate) fn keeps_clone_on_cancel(&self) -> bool {
        self.registry
            .lock()
            .jobs
            .get(&self.id)
            .is_some_and(|e| e.keep_on_cancel)
    }

    /// The job's result, for every caller waiting on it.
    pub(crate) fn finish(mut self, result: &JobResult) {
        self.finished = true;
        self.finish_with(result.clone());
    }

    fn finish_with(&self, result: JobResult) {
        let mut inner = self.registry.lock();
        let Some(entry) = inner.jobs.get_mut(&self.id) else {
            return;
        };
        if entry.info.state.is_finished() {
            return;
        }
        let info = finish_entry(entry, result);
        drop(inner);
        self.registry.emit(info);
        self.registry.schedule_removal(self.id);
    }
}

impl Drop for JobHandle {
    /// A runner that vanished (panic, dropped connection task) must not leave
    /// a job `Running` forever, with callers waiting on it.
    fn drop(&mut self) {
        if !self.finished {
            self.finish_with(Err(ResponseError {
                message: "the job was abandoned".into(),
                unsupported: false,
                kind: None,
                detail: None,
                auth_required: false,
            }));
        }
    }
}

/// The running work's line to its job.
#[derive(Clone)]
pub(crate) struct JobCtl {
    registry: Arc<Registry>,
    id: u64,
    cancel: watch::Receiver<bool>,
}

impl JobCtl {
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// Cancel was requested (a blocking job looks between its steps).
    pub(crate) fn canceled(&self) -> bool {
        *self.cancel.borrow()
    }

    /// The blocking work reports where it is.
    pub(crate) fn progress(&self, phase: &str, percent: Option<u8>) {
        self.registry.set_progress(self.id, phase, percent);
    }

    /// The blocking work reaches its point of no return; `false`: cancel
    /// came first and the work must not go on.
    pub(crate) fn enter_point(&self, phase: &str) -> bool {
        self.registry.enter_point(self.id, phase)
    }
}

tokio::task_local! {
    static CURRENT: JobCtl;
}

/// Runs `work` as the job `ctl` belongs to: [`set_phase`], [`enter_point`],
/// [`leave_point`] and [`cancel_requested_now`] called from inside it reach
/// that job. Outside a job (the tests that call `dispatch_network` directly)
/// they do nothing.
pub(crate) async fn run_in<F: std::future::Future>(ctl: JobCtl, work: F) -> F::Output {
    CURRENT.scope(ctl, work).await
}

/// The work says which part it is in (`fetch`, `integrate`, ...).
pub(crate) fn set_phase(phase: &str) {
    let _ = CURRENT.try_with(|c| c.registry.set_phase(c.id, phase));
}

/// The work reaches its point of no return: from here the job refuses
/// cancel. If cancel came first this never returns, and the task is dropped
/// at once.
pub(crate) async fn enter_point(phase: &str) {
    let proceed = CURRENT
        .try_with(|c| c.registry.enter_point(c.id, phase))
        .unwrap_or(true);
    if !proceed {
        std::future::pending::<()>().await;
    }
}

/// The step after the point of no return failed having changed nothing.
pub(crate) fn leave_point(phase: &str) {
    let _ = CURRENT.try_with(|c| c.registry.leave_point(c.id, phase));
}

/// Cancel was requested for the job this work runs as (a workflow looks
/// between its steps).
pub(crate) fn cancel_requested_now() -> bool {
    CURRENT.try_with(|c| *c.cancel.borrow()).unwrap_or(false)
}

/// Waits for a job another request started and returns its result.
pub(crate) async fn wait_done(mut done: watch::Receiver<Option<JobResult>>) -> JobResult {
    match done.wait_for(Option::is_some).await {
        Ok(result) => result.clone().expect("waited for Some"),
        Err(_) => Err(ResponseError {
            message: "the job was abandoned".into(),
            unsupported: false,
            kind: None,
            detail: None,
            auth_required: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(command: OperationKind, args: Value) -> Request {
        Request {
            id: 1,
            session: "s".into(),
            repo: Some(RepoId("/r".into())),
            command,
            args,
        }
    }

    fn registry() -> (Arc<Registry>, mpsc::Sender<ServerMessage>) {
        let (events, _) = broadcast::channel(64);
        let (tx, _rx) = mpsc::channel(8);
        (Registry::new(events), tx)
    }

    #[tokio::test]
    async fn clone_destinations_compare_by_folder_not_spelling() {
        assert_eq!(normalize_dest("C:\\a\\b\\"), normalize_dest("C:/a/b"));
        let (registry, tx) = registry();
        let first = |dest: &str| Request {
            repo: None,
            ..request(OperationKind::Clone, serde_json::json!({ "dest": dest }))
        };
        let Submitted::New(_job) = registry.submit(&first("/tmp/x"), tx.clone()) else {
            panic!("the first clone is a new job");
        };
        assert!(matches!(
            registry.submit(&first("/tmp/x/"), tx.clone()),
            Submitted::Conflict(1)
        ));
        assert!(matches!(
            registry.submit(&first("/tmp/y"), tx),
            Submitted::New(_)
        ));
    }

    #[tokio::test]
    async fn only_identical_arguments_join() {
        let (registry, tx) = registry();
        let push = |branch: &str, force: bool| {
            request(
                OperationKind::Push,
                serde_json::json!({
                    "remote": "origin",
                    "opts": { "branch": branch, "force_with_lease": force },
                }),
            )
        };
        let Submitted::New(_a) = registry.submit(&push("main", false), tx.clone()) else {
            panic!();
        };
        assert!(matches!(
            registry.submit(&push("main", false), tx.clone()),
            Submitted::Joined { job: 1, .. }
        ));
        assert!(matches!(
            registry.submit(&push("other", false), tx.clone()),
            Submitted::New(_)
        ));
        assert!(matches!(
            registry.submit(&push("main", true), tx),
            Submitted::New(_)
        ));
        assert_eq!(registry.list()[0].joiners, 1);
    }

    #[tokio::test]
    async fn a_fetch_of_all_remotes_covers_a_fetch_of_one() {
        let (registry, tx) = registry();
        let all = request(
            OperationKind::FetchAll,
            serde_json::json!({ "opts": { "prune": false } }),
        );
        let one = request(
            OperationKind::Fetch,
            serde_json::json!({ "remote": "origin", "opts": { "prune": false } }),
        );
        let Submitted::New(_all) = registry.submit(&all, tx.clone()) else {
            panic!();
        };
        assert!(matches!(
            registry.submit(&one, tx),
            Submitted::Joined { job: 1, .. }
        ));
    }

    #[tokio::test]
    async fn the_cancel_policy_is_per_operation() {
        let (registry, tx) = registry();
        let new = |command, args| {
            let Submitted::New(job) = registry.submit(&request(command, args), tx.clone()) else {
                panic!("{command:?} should be a new job");
            };
            assert!(job.claim());
            job.running();
            job
        };
        let checkout = new(OperationKind::LfsCheckout, serde_json::json!({}));
        let fetch = new(OperationKind::Fetch, serde_json::json!({ "remote": "o" }));
        let by_id = |id| registry.list().into_iter().find(|j| j.id == id).unwrap();

        // Rewriting files in place: never while it runs.
        assert_eq!(
            by_id(checkout.id()).not_cancellable,
            Some(NotCancellableReason::RewritingFiles)
        );
        assert!(matches!(
            registry.cancel(checkout.id()),
            Err(Error::NotCancellable {
                reason: NotCancellableReason::RewritingFiles
            })
        ));
        assert_eq!(by_id(checkout.id()).state, JobState::Running, "unchanged");

        // A fetch: any time.
        assert!(by_id(fetch.id()).not_cancellable.is_none());
        assert_eq!(
            registry.cancel(fetch.id()).unwrap().info.state,
            JobState::Cancelling
        );
        assert!(matches!(
            registry.cancel(99),
            Err(Error::NotCancellable {
                reason: NotCancellableReason::UnknownJob
            })
        ));
    }

    #[tokio::test]
    async fn a_pull_and_an_lfs_pull_can_be_stopped_until_their_point_of_no_return() {
        let (registry, tx) = registry();
        for (command, reason, phase) in [
            (
                OperationKind::Pull,
                NotCancellableReason::Integrating,
                "integrate",
            ),
            (
                OperationKind::LfsPull,
                NotCancellableReason::RewritingFiles,
                "checkout",
            ),
        ] {
            let Submitted::New(job) = registry.submit(
                &Request {
                    id: 7,
                    ..request(command, serde_json::json!({ "remote": command as u8 }))
                },
                tx.clone(),
            ) else {
                panic!();
            };
            assert!(job.claim());
            job.running();
            let info = || {
                registry
                    .list()
                    .into_iter()
                    .find(|j| j.id == job.id())
                    .unwrap()
            };
            assert!(info().not_cancellable.is_none(), "{command:?} downloading");

            registry.enter_point(job.id(), phase);
            assert_eq!(info().not_cancellable, Some(reason), "{command:?} past it");
            assert_eq!(info().phase.as_deref(), Some(phase));
            assert!(matches!(
                registry.cancel(job.id()),
                Err(Error::NotCancellable { reason: r }) if r == reason
            ));
            assert_eq!(info().state, JobState::Running, "a refusal changes nothing");

            // The step after it failed having changed nothing: stoppable again.
            registry.leave_point(job.id(), "fetch");
            assert!(info().not_cancellable.is_none());
            assert_eq!(
                registry.cancel(job.id()).unwrap().info.state,
                JobState::Cancelling
            );
            // Cancel came first: the work must not go past the point.
            assert!(!registry.enter_point(job.id(), phase));
            assert!(
                info().not_cancellable.is_none(),
                "{command:?} never passed it"
            );
        }
    }

    #[tokio::test]
    async fn a_workflow_is_stopped_by_its_own_cooperation_and_ends_canceled() {
        let (registry, tx) = registry();
        let Submitted::New(flow) =
            registry.submit(&request(OperationKind::FlowRun, serde_json::json!({})), tx)
        else {
            panic!();
        };
        assert!(flow.claim());
        flow.running();
        let id = flow.id();
        let info = || registry.list().into_iter().find(|j| j.id == id).unwrap();
        assert!(info().not_cancellable.is_none(), "stoppable between steps");
        assert_eq!(
            registry.cancel(id).unwrap().info.state,
            JobState::Cancelling
        );
        // Its task is not dropped: only the work itself looks at the request.
        let waited = tokio::time::timeout(Duration::from_millis(100), flow.abort_requested()).await;
        assert!(waited.is_err(), "a workflow is never aborted from outside");
        flow.finish(&Ok(serde_json::json!({ "status": "failed" })));
        assert_eq!(
            info().state,
            JobState::Canceled,
            "a partial report, a canceled job"
        );
    }

    #[tokio::test]
    async fn an_unclaimed_job_is_canceled_and_answered_by_the_registry() {
        let (events, _) = broadcast::channel(64);
        let (tx, mut rx) = mpsc::channel(8);
        let registry = Registry::new(events);
        let Submitted::New(job) =
            registry.submit(&request(OperationKind::Pull, serde_json::json!({})), tx)
        else {
            panic!();
        };
        let canceled = registry.cancel(job.id()).unwrap();
        assert_eq!(canceled.info.state, JobState::Canceled);
        let (_, response) = canceled.respond.expect("the originator is answered");
        assert_eq!(response.request_id, 1);
        assert!(matches!(
            response.result.unwrap_err().kind,
            Some(gitpoulpi_core::TypedError::Canceled { .. })
        ));
        assert!(!job.claim(), "the server must not run it afterwards");
        assert!(rx.try_recv().is_err(), "the caller sends, not the registry");
    }

    #[tokio::test]
    async fn a_clone_canceled_in_its_lfs_phase_is_kept_before_it_is_not() {
        use gitpoulpi_core::domain::CloneProgress;
        let (registry, tx) = registry();
        let dest = std::path::PathBuf::from("/tmp/lfs-clone");
        let Submitted::New(job) = registry.submit(
            &Request {
                repo: None,
                ..request(
                    OperationKind::Clone,
                    serde_json::json!({ "url": "u", "dest": dest }),
                )
            },
            tx,
        ) else {
            panic!();
        };
        let progress = |phase| Event::CloneProgress {
            url: "u".into(),
            dest: dest.clone(),
            progress: CloneProgress::new(phase, 40),
        };
        registry.observe(&progress(ClonePhase::Receiving));
        assert!(!job.keeps_clone_on_cancel(), "mid-transfer: remove it");
        registry.observe(&progress(ClonePhase::LfsObjects));
        assert!(job.keeps_clone_on_cancel(), "the clone is valid: keep it");
        assert_eq!(registry.list()[0].progress, Some(40));
    }
}
