//! Maps an IPC `Request` onto a `GitBackend` call and back into a JSON
//! `Response`.
//!
//! Every `OperationKind` is routed here, so the frontends are limited only
//! by what they expose, never by what the daemon can carry. Anything not
//! wired returns `ResponseError { unsupported: true }` rather than
//! panicking.

/// Extracts and deserializes `args[$field]`, returning early with an error
/// `Dispatched` (via the enclosing function's `return`) if it's missing or
/// malformed. A macro because the early-return needs to unwind out of
/// `dispatch`'s `match`, not just this helper.
macro_rules! arg_or_err {
    ($args:expr, $field:expr, $request_id:expr) => {
        match $args.get($field) {
            Some(v) => match serde_json::from_value(v.clone()) {
                Ok(parsed) => parsed,
                Err(e) => return err($request_id, format!("invalid `{}`: {e}", $field), false),
            },
            None => return err($request_id, format!("missing `{}`", $field), false),
        }
    };
}

mod failure_log;
mod network;
mod plain;

use network::dispatch_network_plain;
use plain::dispatch_plain;

use std::path::PathBuf;
use std::sync::Arc;

use gitpoulpi_core::domain::{ChangeScope, OperationKind, RepoId, RepoRef};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::{Event, Request, Response, ResponseError};
use serde_json::Value;
use tokio::sync::broadcast;

/// The cargo feature a request needs (custom builds). One table, so a build
/// without the feature rejects the request early with a typed `NotBuilt`
/// instead of reaching a backend stub.
pub(crate) fn requires_feature(request: &Request) -> Option<&'static str> {
    use OperationKind as K;
    match request.command {
        K::LfsStatus
        | K::LfsFiles
        | K::LfsInstall
        | K::LfsTrack
        | K::LfsUntrack
        | K::LfsFetch
        | K::LfsPull
        | K::LfsCheckout
        | K::LfsPush
        | K::LfsPrune
        | K::LfsLocks
        | K::LfsLock
        | K::LfsUnlock => Some("lfs"),
        K::Worktrees | K::WorktreeAdd => Some("worktree"),
        K::Submodules | K::SubmoduleInit | K::SubmoduleUpdate => Some("submodule"),
        _ => None,
    }
}

fn feature_built(feature: &str) -> bool {
    (feature != "lfs" || cfg!(feature = "lfs"))
        && (feature != "worktree" || cfg!(feature = "worktree"))
        && (feature != "submodule" || cfg!(feature = "submodule"))
}

/// The early rejection for a request that needs a feature this daemon was
/// built without.
pub(crate) fn refuse_unbuilt(request: &Request) -> Option<Dispatched> {
    let feature = requires_feature(request).filter(|f| !feature_built(f))?;
    let error = gitpoulpi_core::Error::NotBuilt { feature };
    Some(err_kind(
        request.id,
        error.to_string(),
        false,
        error.typed(),
    ))
}

/// Whether the operation this request ran (if it ran at all) mutated repo
/// state, so the caller knows whether to broadcast a `RepoChanged` event.
pub struct Dispatched {
    pub response: Response,
    pub mutated: bool,
    /// Set by the undo journal when this request recorded an entry or ran
    /// an undo/redo: the server broadcasts it as `Event::JournalChanged`.
    pub journal: Option<gitpoulpi_core::domain::JournalState>,
    /// Set when this request found changes made outside GitPoulpi and
    /// recorded them: the server broadcasts it as `Event::JournalChanged`
    /// even when the request itself was a read.
    pub outside: Option<gitpoulpi_core::domain::JournalState>,
    /// Set by `autostash` when the repo's pending auto-stash record was
    /// written, changed or cleared: the server broadcasts it as
    /// `Event::PendingAutoStashChanged` (even when the request failed).
    pub pending_auto_stash_changed: bool,
    /// Set by `autostash::reconcile` when the pending record's stash had
    /// disappeared from the stash list (dropped or popped outside
    /// GitPoulpi) and the record was cleared: the server broadcasts it as
    /// `Event::PendingAutoStashGone` so clients can say so.
    pub pending_auto_stash_gone: Option<gitpoulpi_core::domain::StashId>,
}

pub(crate) fn ok(request_id: u64, value: Value, mutated: bool) -> Dispatched {
    Dispatched {
        response: Response {
            request_id,
            result: Ok(value),
        },
        mutated,
        journal: None,
        outside: None,
        pending_auto_stash_changed: false,
        pending_auto_stash_gone: None,
    }
}

/// The answer for a request whose operation panicked: an error, so the
/// client is never left waiting for a response that will not come.
pub(crate) fn panicked(request_id: u64, command: OperationKind) -> Response {
    err(
        request_id,
        format!("internal error: the daemon's {command:?} operation panicked"),
        false,
    )
    .response
}

pub(crate) fn err(request_id: u64, message: String, unsupported: bool) -> Dispatched {
    err_kind(request_id, message, unsupported, None)
}

/// `err`, carrying the typed half of a backend error (see
/// `ResponseError::kind`).
pub(crate) fn err_kind(
    request_id: u64,
    message: String,
    unsupported: bool,
    kind: Option<gitpoulpi_core::TypedError>,
) -> Dispatched {
    err_full(request_id, message, unsupported, kind, None)
}

/// A failed backend call as the dispatchers carry it: the message, the typed
/// half and the facts (`ResponseError::detail`).
pub(crate) type Failure = (
    String,
    Option<gitpoulpi_core::TypedError>,
    Option<gitpoulpi_core::ErrorDetail>,
);

/// The answer for `request` whose backend call failed. The detail is named
/// after the operation the user asked for, and the failure goes to the
/// application log (the Activity Log's second tab).
pub(crate) fn failed(
    request: &Request,
    message: String,
    kind: Option<gitpoulpi_core::TypedError>,
    detail: Option<gitpoulpi_core::ErrorDetail>,
) -> Dispatched {
    let operation = format!("{:?}", request.command);
    let detail = detail.map(|d| d.with_operation(operation.clone()));
    // A poll failing the same way is one line, not one per poll.
    let repo = request.repo.as_ref().map(|r| r.0.as_str());
    if let Some(held_back) =
        failure_log::global().observe(&operation, repo, &message, std::time::Instant::now())
    {
        let redacted = gitpoulpi_core::redact(&message);
        if held_back == 0 {
            tracing::warn!(target: "daemon", repo = repo.unwrap_or(""), "{operation} failed: {redacted}");
        } else {
            tracing::warn!(
                target: "daemon",
                repo = repo.unwrap_or(""),
                "{operation} failed: {redacted} (repeated {held_back} more times since the last line)"
            );
        }
    }
    if let Some(d) = &detail {
        tracing::debug!(
            target: "daemon",
            repo = repo.unwrap_or(""),
            "{operation} detail: {}",
            d.lines().join(" | ")
        );
    }
    err_full(request.id, message, false, kind, detail)
}

pub(crate) fn err_full(
    request_id: u64,
    message: String,
    unsupported: bool,
    kind: Option<gitpoulpi_core::TypedError>,
    detail: Option<gitpoulpi_core::ErrorDetail>,
) -> Dispatched {
    Dispatched {
        response: Response {
            request_id,
            result: Err(ResponseError {
                message,
                unsupported,
                kind,
                detail: detail.map(Box::new),
                auth_required: false,
            }),
        },
        mutated: false,
        journal: None,
        outside: None,
        pending_auto_stash_changed: false,
        pending_auto_stash_gone: None,
    }
}

pub(crate) fn repo_ref(request: &Request) -> Result<RepoRef, String> {
    let repo: &RepoId = request
        .repo
        .as_ref()
        .ok_or_else(|| "request has no repo target".to_string())?;
    Ok(RepoRef {
        id: repo.clone(),
        root: PathBuf::from(&repo.0),
    })
}

/// Runs the async `GitBackend` network call for `request`: every
/// operation `OpTraits::network` marks (fetch, pull, push, clone, the tag and
/// LFS transfers, remote branch deletion).
///
/// The one network operation the undo journal records is
/// `FetchRefAsBranch` (its local branch); the write-ahead capture runs
/// before the await and the outcome after it.
pub async fn dispatch_network(
    backend: &dyn GitBackend,
    request: &Request,
    events: &broadcast::Sender<Event>,
) -> Dispatched {
    /// Counts the transfer for outside detection, whatever way it ends.
    struct Transfer;
    impl Drop for Transfer {
        fn drop(&mut self) {
            crate::journal::transfer_finished();
        }
    }
    let repo = repo_ref(request).ok();
    let busy = repo
        .as_ref()
        .and_then(|repo| crate::journal::Busy::enter(&repo.root));
    let repo = repo.filter(|_| busy.is_some());
    // What happened outside before this transfer is recorded first; what
    // the transfer itself moves (tags, a pulled branch) is adopted after.
    let outside = repo
        .as_ref()
        .and_then(|repo| crate::journal::outside_sync(backend, repo, crate::journal::Caller::Read));
    crate::journal::transfer_started();
    let transfer = Transfer;
    let mut dispatched = dispatch_network_inner(backend, request, events).await;
    drop(transfer);
    if let Some(repo) = &repo {
        crate::journal::outside_adopt(backend, repo);
    }
    drop(busy);
    if outside.is_some() {
        dispatched.outside = outside;
    }
    dispatched
}

/// A journal entry written ahead of a network operation. If the operation's
/// future is dropped before it ends (the job was canceled), the entry would
/// stay `Pending` for ever and a later start would report it `Interrupted`:
/// dropping the guard settles it as a failed operation, which removes the entry
/// and its pins, so a canceled operation leaves no trace in the journal.
struct AbandonedRecording<'a> {
    backend: &'a dyn GitBackend,
    request: &'a Request,
    recording: Option<crate::journal::Recording>,
}

impl<'a> AbandonedRecording<'a> {
    fn begin(backend: &'a dyn GitBackend, request: &'a Request) -> Self {
        Self {
            backend,
            request,
            recording: crate::journal::begin_record(backend, request),
        }
    }

    /// The operation ended: hands the recording back to be finished normally.
    fn finished(&mut self) -> Option<crate::journal::Recording> {
        self.recording.take()
    }
}

impl Drop for AbandonedRecording<'_> {
    fn drop(&mut self) {
        if let Some(recording) = self.recording.take() {
            let mut canceled = err(self.request.id, "canceled".into(), false);
            crate::journal::finish_record(self.backend, self.request, recording, &mut canceled);
        }
    }
}

/// An auto-stash a pull made and has not put back yet. If the pull's future
/// is dropped (the job was canceled during the second download) the changes
/// go back at once: a canceled pull leaves the working tree as it found it.
struct ArmedStash<'a> {
    backend: &'a dyn GitBackend,
    request: &'a Request,
    events: broadcast::Sender<Event>,
    armed: Option<crate::autostash::Armed>,
}

impl ArmedStash<'_> {
    /// The pull ended: the stash is handled by the normal path.
    fn disarm(&mut self) -> crate::autostash::Armed {
        self.armed.take().expect("armed until the pull ends")
    }
}

impl Drop for ArmedStash<'_> {
    fn drop(&mut self) {
        let Some(armed) = self.armed.take() else {
            return;
        };
        let canceled = err(self.request.id, "canceled".into(), false);
        let _ = crate::autostash::finish(self.backend, self.request, armed, canceled);
        if let Ok(repo) = repo_ref(self.request) {
            let _ = self
                .events
                .send(Event::PendingAutoStashChanged { repo: repo.id });
        }
    }
}

async fn dispatch_network_inner(
    backend: &dyn GitBackend,
    request: &Request,
    events: &broadcast::Sender<Event>,
) -> Dispatched {
    // `credentials.https_mode`, read live: the next transfer obeys a
    // change made in Settings.
    if let Some(refused) = refuse_unbuilt(request) {
        return refused;
    }
    match repo_ref(request) {
        // The repository's own authentication profile comes with the global
        // setting; a profile that cannot be used refuses the transfer.
        Ok(repo) => {
            if let Err(error) = crate::credentials::apply_for(backend, &repo) {
                return failed(request, error.to_string(), error.typed(), None);
            }
        }
        // A clone has no repository yet.
        Err(_) => crate::credentials::apply_from_config(),
    }
    if let Some(refused) = crate::autostash::refuse_while_pending(request) {
        return refused;
    }
    #[cfg(feature = "lfs")]
    if let Ok(repo) = repo_ref(request) {
        crate::lfs_preflight::run(backend, &repo, request.command, events);
    }
    if matches!(
        request.command,
        OperationKind::BranchDeleteRemote | OperationKind::TagDeleteRemote
    ) {
        return dispatch_remote_delete(backend, request, events).await;
    }
    if !matches!(
        request.command,
        OperationKind::FetchRefAsBranch | OperationKind::Pull | OperationKind::FlowRun
    ) {
        return dispatch_network_plain(backend, request, events).await;
    }
    let mut recording = AbandonedRecording::begin(backend, request);
    let mut dispatched = dispatch_network_plain(backend, request, events).await;
    if let Some(recording) = recording.finished() {
        crate::journal::finish_record(backend, request, recording, &mut dispatched);
    }
    // A pull blocked by local changes: stash, pull again (recorded on its
    // own, after the stash's entry), pop. See `autostash`.
    if let Some(armed) = crate::autostash::after_first(backend, request, &mut dispatched) {
        let mut stash = ArmedStash {
            backend,
            request,
            events: events.clone(),
            armed: Some(armed),
        };
        let mut recording = AbandonedRecording::begin(backend, request);
        let mut second = dispatch_network_plain(backend, request, events).await;
        if let Some(recording) = recording.finished() {
            crate::journal::finish_record(backend, request, recording, &mut second);
        }
        dispatched = crate::autostash::finish(backend, request, stash.disarm(), second);
    }
    // `general.keep_submodules_up_to_date`.
    crate::submodules::keep_up_to_date(backend, request, &mut dispatched);
    settle_pending(backend, request, &mut dispatched);
    dispatched
}

/// Deleting a branch or tag on a remote, journaled: the server is asked what
/// the ref holds right now, that exact value is recorded (and pinned: undo
/// pushes it back), and the deletion is leased on it, so what is recorded is
/// what is deleted. `and_local` also deletes the local branch or tag of the
/// same name, in the same entry.
///
/// A ref the server does not have, or a server that does not answer, is not
/// journaled: the plain deletion runs and fails with its own message.
async fn dispatch_remote_delete(
    backend: &dyn GitBackend,
    request: &Request,
    events: &broadcast::Sender<Event>,
) -> Dispatched {
    let is_tag = request.command == OperationKind::TagDeleteRemote;
    let text = |key: &str| request.args.get(key).and_then(|v| v.as_str());
    let (Ok(repo), Some(remote), Some(name)) = (repo_ref(request), text("remote"), text("name"))
    else {
        return dispatch_network_plain(backend, request, events).await;
    };
    let full_ref = if is_tag {
        format!("refs/tags/{name}")
    } else {
        format!("refs/heads/{name}")
    };
    let Ok(Some(old)) = backend.remote_ref_value(&repo, remote, &full_ref).await else {
        return dispatch_network_plain(backend, request, events).await;
    };
    let local_request = network::local_twin(&request.args, name).map(|local| Request {
        command: if is_tag {
            OperationKind::TagDelete
        } else {
            OperationKind::BranchDelete
        },
        args: serde_json::json!({ "name": local }),
        ..request.clone()
    });
    let mut leased = request.clone();
    if let Some(args) = leased.args.as_object_mut() {
        args.insert("expect".into(), serde_json::json!(old));
    }
    let change = gitpoulpi_core::journal::RemoteRefChange {
        remote: remote.to_string(),
        name: full_ref,
        old: Some(old),
        new: None,
    };
    let mut recording = crate::journal::begin_record_remote(
        backend,
        request,
        crate::journal::RemoteDelete {
            change,
            local_request,
        },
    )
    .map(|recording| AbandonedRecording {
        backend,
        request,
        recording: Some(recording),
    });
    let mut dispatched = dispatch_network_plain(backend, &leased, events).await;
    if let Some(recording) = recording.as_mut().and_then(AbandonedRecording::finished) {
        crate::journal::finish_record(backend, request, recording, &mut dispatched);
    }
    dispatched
}

/// How a cached read is answered: the serialized result (shared with the
/// cache, written to the wire as is), or a ready [`Dispatched`] for
/// everything that did not go through the cache or failed.
pub enum CachedReply {
    /// `pending_auto_stash_*`: what reconciling the pending auto-stash record
    /// found while computing a miss (`Refs` reconciles like `Status` does).
    /// A hit computes nothing, so these are then unset. They must reach the
    /// server: whichever read reconciles first clears the record, and a
    /// later one finds nothing to report, so a dropped notice is lost for
    /// good (the banner stays until the next restart).
    Json {
        request_id: u64,
        json: Arc<[u8]>,
        pending_auto_stash_changed: bool,
        pending_auto_stash_gone: Option<gitpoulpi_core::domain::StashId>,
    },
    Dispatched(Box<Dispatched>),
}

/// [`dispatch`], answering `Log`, `Refs` and `Workflow` from `cache` while the refs are
/// unchanged (see `cache.rs`). Everything else goes straight through.
pub fn dispatch_cached_reply(
    cache: &crate::cache::ReadCache,
    backend: &dyn GitBackend,
    request: &Request,
) -> CachedReply {
    if let (Some(repo), OperationKind::CommitStats) = (&request.repo, request.command) {
        return CachedReply::Dispatched(Box::new(dispatch_commit_stats(
            cache, backend, request, repo,
        )));
    }
    let (Some(repo), true) = (
        &request.repo,
        matches!(
            request.command,
            OperationKind::Log
                | OperationKind::CountCommits
                | OperationKind::Refs
                | OperationKind::Workflow
        ),
    ) else {
        return CachedReply::Dispatched(Box::new(dispatch(backend, request)));
    };
    // The decoded text depends on the encoding preference, so it is part of
    // the key: changing the setting must not serve the old decoding.
    let encoding = crate::encoding::for_request(request).unwrap_or_default();
    // The client's etag is not part of what identifies the read.
    let if_none_match = gitpoulpi_ipc::etag::if_none_match(&request.args);
    let args = if if_none_match.is_some() {
        gitpoulpi_ipc::etag::without_if_none_match(&request.args)
    } else {
        request.args.clone()
    };
    // The workflow also depends on two files outside git's own state.
    let files = match request.command {
        OperationKind::Workflow => crate::workflow::files_stamp(std::path::Path::new(&repo.0)),
        // The authors shown depend on `log.use_mailmap` and the `.mailmap`.
        OperationKind::Log => crate::mailmap::stamp(&repo.0),
        _ => String::new(),
    };
    let key = (
        request.command,
        format!("{encoding}\u{0}{args}\u{0}{files}"),
    );
    let mut failure = None;
    let mut pending = (false, None);
    let cached = cache.get_or_compute_if_none_match(repo, key, if_none_match, || {
        let dispatched = dispatch(backend, request);
        pending = (
            dispatched.pending_auto_stash_changed,
            dispatched.pending_auto_stash_gone,
        );
        match dispatched.response.result {
            Ok(value) => Ok(value),
            Err(e) => Err(failure.insert(e).message.clone()),
        }
    });
    match cached {
        Ok(crate::cache::Cached::Json(json)) => CachedReply::Json {
            request_id: request.id,
            json,
            pending_auto_stash_changed: pending.0,
            pending_auto_stash_gone: pending.1,
        },
        Ok(crate::cache::Cached::NotModified) => CachedReply::Json {
            request_id: request.id,
            json: Arc::from(gitpoulpi_ipc::etag::NOT_MODIFIED_JSON),
            pending_auto_stash_changed: pending.0,
            pending_auto_stash_gone: pending.1,
        },
        Err(_) => CachedReply::Dispatched(Box::new(Dispatched {
            response: Response {
                request_id: request.id,
                result: Err(failure.expect("a failed compute recorded its error")),
            },
            mutated: false,
            journal: None,
            outside: None,
            pending_auto_stash_changed: pending.0,
            pending_auto_stash_gone: pending.1,
        })),
    }
}

/// [`dispatch_cached_reply`] as a plain [`Dispatched`] (parses a cached
/// answer back into a `Value`): what the in-process callers and tests use.
#[cfg(test)]
pub fn dispatch_cached(
    cache: &crate::cache::ReadCache,
    backend: &dyn GitBackend,
    request: &Request,
) -> Dispatched {
    match dispatch_cached_reply(cache, backend, request) {
        CachedReply::Dispatched(d) => *d,
        CachedReply::Json {
            request_id, json, ..
        } => ok(
            request_id,
            serde_json::from_slice(&json).expect("the cache holds valid JSON"),
            false,
        ),
    }
}

/// `CommitStats` through the daemon's stats cache: ids already known are
/// answered from memory, only the rest go to the backend (as one batch),
/// and what it returns is remembered. A commit's numbers never change, so
/// nothing is ever invalidated.
fn dispatch_commit_stats(
    cache: &crate::cache::ReadCache,
    backend: &dyn GitBackend,
    request: &Request,
    repo: &gitpoulpi_core::domain::RepoId,
) -> Dispatched {
    use gitpoulpi_core::domain::{CommitStat, Oid};
    let ids: Vec<Oid> = arg_or_err!(&request.args, "ids", request.id);
    let (known, missing) = cache.stats_lookup(repo, &ids);
    let mut fetched: Vec<CommitStat> = Vec::new();
    if !missing.is_empty() {
        let sub = Request {
            args: serde_json::json!({ "ids": missing }),
            ..request.clone()
        };
        let dispatched = dispatch(backend, &sub);
        match dispatched.response.result {
            Ok(value) => match serde_json::from_value::<Vec<CommitStat>>(value) {
                Ok(stats) => fetched = stats,
                Err(e) => return err(request.id, format!("bad commit stats: {e}"), false),
            },
            Err(e) => {
                return Dispatched {
                    response: Response {
                        request_id: request.id,
                        result: Err(e),
                    },
                    mutated: false,
                    journal: None,
                    outside: None,
                    pending_auto_stash_changed: false,
                    pending_auto_stash_gone: None,
                }
            }
        }
        cache.stats_store(repo, &fetched);
    }
    let by_id: std::collections::HashMap<Oid, CommitStat> = known
        .into_iter()
        .chain(fetched.into_iter().map(|s| (s.id, s)))
        .collect();
    let answer: Vec<CommitStat> = ids.iter().filter_map(|id| by_id.get(id).copied()).collect();
    match serde_json::to_value(answer) {
        Ok(json) => ok(request.id, json, false),
        Err(e) => err(request.id, format!("failed to encode result: {e}"), false),
    }
}

/// Runs the blocking `GitBackend` call for `request`. Must be called from
/// inside `spawn_blocking` - every `GitBackend` method here is
/// synchronous I/O (process spawns or libgit2 calls).
///
/// The undo journal sits here: `Undo`/`Redo`/`JournalState`/`JournalList`
/// are answered by it, and the operations it records (commit, checkout,
/// branch create/delete) run inside its write-ahead capture. Everything
/// else is [`dispatch_plain`].
pub fn dispatch(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    use crate::journal::Caller;
    let traits = request.command.traits();
    let watched = repo_ref(request).ok().filter(|_| {
        !traits.network
            && (!traits.read
                || matches!(
                    request.command,
                    OperationKind::JournalState | OperationKind::JournalList
                ))
            // A mutation that cannot move a ref, HEAD or the stash (staging,
            // config edits) needs no snapshot work.
            && (traits.read
                || traits.scope.intersects(
                    ChangeScope::REFS | ChangeScope::LOG | ChangeScope::STASHES,
                ))
    });
    // A step of a request that already owns detection (an auto-stash pop)
    // does not look for outside changes in the middle of it.
    let busy = watched
        .as_ref()
        .and_then(|repo| crate::journal::Busy::enter(&repo.root));
    let watched = watched.filter(|_| busy.is_some());
    let caller = if traits.read {
        Caller::Read
    } else {
        Caller::Mutation
    };
    // What a shell or an AI agent did since last time is recorded before
    // this request looks at the journal or adds its own entry.
    let outside = watched
        .as_ref()
        .and_then(|repo| crate::journal::outside_sync(backend, repo, caller));
    let mut dispatched = dispatch_inner(backend, request);
    // Whatever this request moved is its own: never an outside change.
    if let (Some(repo), Caller::Mutation) = (&watched, caller) {
        crate::journal::outside_adopt(backend, repo);
    }
    drop(busy);
    if outside.is_some() {
        dispatched.outside = outside;
    }
    dispatched
}

fn dispatch_inner(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    crate::test_hook::maybe_panic(request.command);
    if let Some(refused) = refuse_unbuilt(request) {
        return refused;
    }
    // The pending auto-stash record is brought in line with the stash list
    // first: a stash dropped outside must not keep refusing operations.
    let before = crate::autostash::before(backend, request);
    if let Some(refused) = before.refused {
        return refused;
    }
    let request = before.request.as_ref().unwrap_or(request);
    // *Start over*, and Undo while it is what Undo means (see its module).
    let start_over = match request.command {
        OperationKind::AutoStashStartOver => {
            Some(crate::autostash_start_over::start_over(backend, request))
        }
        // `forget` drops a blocked journal entry: never Start over.
        OperationKind::Undo
            if request.args.get("forget").and_then(|v| v.as_bool()) != Some(true) =>
        {
            crate::autostash_start_over::route_undo(backend, request)
        }
        _ => None,
    };
    if let Some(mut done) = start_over {
        before.apply(&mut done);
        return done;
    }
    if let Some(mut refused) = crate::autostash::refuse_while_pending(request) {
        before.apply(&mut refused);
        return refused;
    }
    let mut dispatched = dispatch_journaled(backend, request);
    // Blocked by local changes? `general.auto_stash` decides (see `autostash`).
    if let Some(armed) = crate::autostash::after_first(backend, request, &mut dispatched) {
        let second = dispatch_journaled(backend, request);
        dispatched = crate::autostash::finish(backend, request, armed, second);
    }
    // `general.keep_submodules_up_to_date`.
    crate::submodules::keep_up_to_date(backend, request, &mut dispatched);
    settle_pending(backend, request, &mut dispatched);
    // A merge, cherry-pick, revert or rebase that stopped on conflicts
    // that differ in whitespace only: resolved here, journaled. After the
    // settling, which ends the resolutions of an earlier operation.
    crate::auto_resolve::after_operation(backend, request, &mut dispatched);
    if let Ok(repo) = repo_ref(request) {
        if dispatched.mutated {
            crate::resolved_record::prune_when_idle(&repo);
        }
    }
    crate::autostash::settle_ended(backend, request, &mut dispatched);
    crate::autostash::settle_manual(backend, request, &mut dispatched);
    before.apply(&mut dispatched);
    dispatched
}

/// After a mutation, finishes the journal entry of a conflicted merge or
/// rebase whose operation has ended (see `journal::settle`).
fn settle_pending(backend: &dyn GitBackend, request: &Request, dispatched: &mut Dispatched) {
    if !dispatched.mutated
        || matches!(
            request.command,
            OperationKind::Undo | OperationKind::Redo | OperationKind::Pull
        ) && dispatched.journal.is_some()
    {
        return;
    }
    if let Ok(repo) = repo_ref(request) {
        if let Some(state) = crate::journal::settle(backend, &repo, request.command) {
            dispatched.journal = Some(state);
        }
    }
}

fn dispatch_journaled(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    use gitpoulpi_core::domain::Direction;
    crate::journal::housekeeping(backend, request);
    match request.command {
        OperationKind::Undo => crate::journal::undo_redo(backend, request, Direction::Undo),
        OperationKind::Redo => crate::journal::undo_redo(backend, request, Direction::Redo),
        OperationKind::JournalState => crate::journal::read_state(backend, request),
        OperationKind::JournalList => crate::journal::read_list(request),
        OperationKind::JournalRestore => crate::journal::restore_entry(backend, request),
        OperationKind::RewritePurgeCheck => crate::journal::purge_check(backend, request),
        OperationKind::RewritePurge => crate::journal::purge_run(backend, request),
        OperationKind::ReflogRestore => crate::journal::reflog_restore(backend, request),
        OperationKind::SetRefVisibility => crate::ref_visibility::set(request),
        _ if crate::journal::is_journaled(request) => {
            crate::journal::record(backend, request, &|| dispatch_plain(backend, request))
        }
        _ => match crate::encoding::for_request(request) {
            // `git.default_encoding`: decode non-UTF-8 text for display.
            Some(label) => gitpoulpi_core::domain::with_fallback_encoding(Some(&label), || {
                dispatch_plain(backend, request)
            }),
            None => dispatch_plain(backend, request),
        },
    }
}

#[cfg(all(test, not(feature = "lfs")))]
mod not_built_tests {
    use super::*;

    #[test]
    fn an_lfs_request_is_refused_with_not_built() {
        let backend = gitpoulpi_git2_backend::Git2Backend::new();
        let request = Request {
            id: 7,
            session: "t".into(),
            command: OperationKind::LfsStatus,
            repo: Some(RepoId("/nowhere".into())),
            args: serde_json::json!({}),
        };
        let dispatched = dispatch(&backend, &request);
        let error = dispatched.response.result.unwrap_err();
        assert!(matches!(
            error.kind,
            Some(gitpoulpi_core::TypedError::NotBuilt { feature: ref f }) if f == "lfs"
        ));
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    fn request(command: OperationKind) -> Request {
        Request {
            id: 1,
            session: "t".into(),
            repo: None,
            command,
            args: serde_json::json!({}),
        }
    }

    /// Worktree and submodule operations are in the `requires_feature`
    /// table and are refused with a typed `NotBuilt` exactly when the
    /// feature is off.
    #[test]
    fn worktree_and_submodule_requests_follow_their_features() {
        for (kind, feature, built) in [
            (
                OperationKind::Worktrees,
                "worktree",
                cfg!(feature = "worktree"),
            ),
            (
                OperationKind::WorktreeAdd,
                "worktree",
                cfg!(feature = "worktree"),
            ),
            (
                OperationKind::Submodules,
                "submodule",
                cfg!(feature = "submodule"),
            ),
            (
                OperationKind::SubmoduleInit,
                "submodule",
                cfg!(feature = "submodule"),
            ),
            (
                OperationKind::SubmoduleUpdate,
                "submodule",
                cfg!(feature = "submodule"),
            ),
        ] {
            let req = request(kind);
            assert_eq!(requires_feature(&req), Some(feature));
            let refused = refuse_unbuilt(&req);
            assert_eq!(refused.is_none(), built, "{kind:?}");
            if let Some(refused) = refused {
                let e = refused.response.result.unwrap_err();
                assert!(matches!(
                    e.kind,
                    Some(gitpoulpi_core::TypedError::NotBuilt { .. })
                ));
            }
        }
    }
}
