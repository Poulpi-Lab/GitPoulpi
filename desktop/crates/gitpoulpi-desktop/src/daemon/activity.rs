//! What the undo history, the Activity Log, the jobs panel and the grouped-push view read and ask of
//! the daemon.
//!
//! Three kinds of things live here:
//!
//! - typed requests on the link's connection ([`Backend::call_typed`] keeps the daemon's own error, so
//!   "needs confirmation" and "name taken" can be answered instead of printed);
//! - the *feed* the link host fills from what the link reports (operations with their duration,
//!   failures, messages): the Git operations tab of the Activity Log;
//! - the *group run*: the per-remote state of a grouped push, fed by the link's push-group events.
//!
//! The application log (the other tab) is `gitpoulpi-log`'s files, read through [`app_records`].

use std::collections::VecDeque;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use gitpoulpi_core::domain::{
    Direction, JournalListing, LostCommitsPage, Oid, OperationKind, ReflogPage, RemoteGroupOutcome,
    RemoteGroupRunRequest, RestoreOutcome, TargetStatus, UndoOutcome,
};
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_ipc::JobInfo;
use gitpoulpi_link::Reply;
use gitpoulpi_log::{Level, Record};
use serde::de::DeserializeOwned;

use super::host::Notifier;
use super::Backend;

/// A pending request whose answer keeps the daemon's own error type.
pub struct Typed<T> {
    reply: Reply<T>,
    notifier: Arc<Notifier>,
}

impl<T> Future for Typed<T> {
    type Output = Result<T, CoreError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.notifier.register(cx.waker());
        match self.reply.take() {
            Some(result) => Poll::Ready(result),
            None => Poll::Pending,
        }
    }
}

type TypedFut<T> = Typed<T>;

impl Backend {
    /// One request on the link's connection; the answer carries the daemon's typed error.
    pub fn call_typed<T: DeserializeOwned + Send + 'static>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Typed<T> {
        let reply = self.open().requester.call::<T>(command, args);
        Typed {
            reply,
            notifier: self.notifier.clone(),
        }
    }

    /// Every journal entry: the undo stack and the redo stack, with what each did.
    pub fn journal_listing(&self) -> TypedFut<JournalListing> {
        self.call_typed(OperationKind::JournalList, serde_json::json!({}))
    }

    /// Undoes or redoes the entry `expect` (the one the person was shown).
    pub fn undo_step(
        &self,
        direction: Direction,
        expect: u64,
        confirmed: bool,
    ) -> TypedFut<UndoOutcome> {
        let op = match direction {
            Direction::Undo => OperationKind::Undo,
            Direction::Redo => OperationKind::Redo,
        };
        self.call_typed(
            op,
            serde_json::json!({ "expect": expect, "dry_run": false, "confirmed": confirmed }),
        )
    }

    /// "Restore only this": the plan (`dry`) or the restore itself.
    pub fn restore_entry(
        &self,
        id: u64,
        name: Option<&str>,
        dry: bool,
    ) -> TypedFut<RestoreOutcome> {
        self.call_typed(
            OperationKind::JournalRestore,
            serde_json::json!({ "id": id, "name": name, "dry_run": dry }),
        )
    }

    /// Moves a branch (or HEAD) back to a reflog position: the plan (`dry`) or the move.
    pub fn reflog_restore(
        &self,
        refname: &str,
        oid: Oid,
        mode: Option<&str>,
        dry: bool,
    ) -> TypedFut<RestoreOutcome> {
        self.call_typed(
            OperationKind::ReflogRestore,
            serde_json::json!({ "ref": refname, "oid": oid, "mode": mode, "dry_run": dry }),
        )
    }

    /// One page of a reflog (`HEAD` or a full branch name).
    pub fn reflog_page(&self, refname: &str, skip: usize, limit: usize) -> TypedFut<ReflogPage> {
        self.call_typed(
            OperationKind::Reflog,
            serde_json::json!({ "ref": refname, "skip": skip, "limit": limit }),
        )
    }

    /// Commits no ref reaches: the cheap pass (`deep` off) or the scan of dangling objects.
    pub fn lost_commits(&self, deep: bool, limit: usize) -> TypedFut<LostCommitsPage> {
        self.call_typed(
            OperationKind::LostCommits,
            serde_json::json!({ "deep": deep, "limit": limit }),
        )
    }

    pub fn branch_at(&self, name: &str, at: Oid) -> TypedFut<()> {
        self.call_typed(
            OperationKind::BranchCreate,
            serde_json::json!({ "name": name, "at": at }),
        )
    }

    pub fn tag_at(&self, name: &str, at: Oid) -> TypedFut<()> {
        self.call_typed(
            OperationKind::TagCreate,
            serde_json::json!({ "name": name, "at": at, "annotated_message": null }),
        )
    }

    /// Replays what a commit changed onto the working tree.
    pub fn apply_commit(&self, commit: Oid) -> TypedFut<gitpoulpi_core::domain::PatchApplied> {
        self.call_typed(
            OperationKind::ApplyCommit,
            serde_json::json!({ "commit": commit }),
        )
    }

    /// Keeps what a commit changed as a stash, the files untouched.
    pub fn stash_from_commit(&self, commit: Oid) -> TypedFut<Oid> {
        self.call_typed(
            OperationKind::StashFromCommit,
            serde_json::json!({ "commit": commit }),
        )
    }

    /// Asks the daemon to cancel job `id`.
    pub fn cancel_job(&self, id: u64) -> TypedFut<JobInfo> {
        self.call_typed(OperationKind::JobCancel, serde_json::json!({ "id": id }))
    }
}

// ---- the Git operations feed -----------------------------------------------------------------------

/// How many entries the feed keeps; the oldest go first.
pub const FEED_CAPACITY: usize = 300;

/// One line of the Git operations tab: an operation that ran, failed or said something.
#[derive(Debug, Clone, PartialEq)]
pub struct FeedEntry {
    pub id: u64,
    pub ts_ms: i64,
    pub level: Level,
    /// "Push", "Fetch", "Push failed".
    pub title: String,
    /// The details lines: the error as the daemon worded it, the operation and its arguments.
    pub details: Vec<String>,
    pub repo: Option<PathBuf>,
    pub duration_ms: Option<u64>,
}

impl FeedEntry {
    /// The entry as the text Copy puts on the clipboard.
    pub fn to_text(&self) -> String {
        let mut out = format!(
            "{} {} {}",
            gitpoulpi_log::timestamp(self.ts_ms),
            self.level.as_str(),
            self.title
        );
        if let Some(ms) = self.duration_ms {
            out.push_str(&format!(" ({ms} ms)"));
        }
        if let Some(repo) = &self.repo {
            out.push_str(&format!("\n{}", repo.display()));
        }
        for line in &self.details {
            out.push_str(&format!("\n{line}"));
        }
        out
    }

    /// An error or a warning: what "next failed operation" stops at.
    pub fn is_problem(&self) -> bool {
        self.level <= Level::Warn
    }
}

#[derive(Default)]
pub struct Feed {
    next: u64,
    entries: VecDeque<FeedEntry>,
}

impl Feed {
    pub fn push(
        &mut self,
        level: Level,
        title: String,
        details: Vec<String>,
        repo: Option<PathBuf>,
        duration_ms: Option<u64>,
    ) {
        self.next += 1;
        if self.entries.len() >= FEED_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(FeedEntry {
            id: self.next,
            ts_ms: gitpoulpi_log::now_ms(),
            level,
            title,
            details,
            repo,
            duration_ms,
        });
    }

    /// Oldest first.
    pub fn snapshot(&self) -> Vec<FeedEntry> {
        self.entries.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The newest error, for "copy the last error" and "open the last error".
    pub fn last_error(&self) -> Option<FeedEntry> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.level == Level::Error)
            .cloned()
    }
}

// ---- the application log ---------------------------------------------------------------------------

/// How many application log lines the tab reads.
pub const APP_RECORDS: usize = 2000;

/// The application log's lines, oldest first, and the folder of its files (`None`: kept in memory).
/// Blocks on the log writer: call it off the main thread.
pub fn app_records(limit: usize) -> (Vec<Record>, Option<PathBuf>) {
    let Some(logger) = gitpoulpi_log::global() else {
        return (Vec::new(), None);
    };
    logger.flush();
    match logger.dir().map(std::path::Path::to_path_buf) {
        Some(dir) => (gitpoulpi_log::read_records(&dir, limit), Some(dir)),
        None => (logger.recent(), None),
    }
}

/// Empties the application log (the files of every kept day); how many files that touched.
pub fn clear_app_log() -> usize {
    gitpoulpi_log::global().map_or(0, |l| l.clear())
}

// ---- the grouped push run --------------------------------------------------------------------------

/// One remote of a grouped push.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupRow {
    pub remote: String,
    pub branch: String,
    /// `None` until the first progress report: waiting.
    pub status: Option<TargetStatus>,
    pub detail: Option<String>,
    pub integrated: Option<gitpoulpi_core::domain::Integration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupPhase {
    Going,
    Ended(gitpoulpi_core::domain::GroupRunStatus),
    /// The run request itself was refused.
    Failed(String),
}

/// A grouped push (or pull) as the link last reported it.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupRun {
    pub branch: String,
    pub mode: gitpoulpi_core::domain::GroupMode,
    pub run_id: Option<String>,
    pub request: RemoteGroupRunRequest,
    pub rows: Vec<GroupRow>,
    pub phase: GroupPhase,
    pub message: String,
}

impl GroupRun {
    pub fn from_request(request: &RemoteGroupRunRequest) -> Self {
        Self {
            branch: request.branch.clone(),
            mode: request.mode,
            run_id: None,
            request: request.clone(),
            rows: request
                .targets
                .iter()
                .map(|t| GroupRow {
                    remote: t.remote.clone(),
                    branch: t.branch.clone(),
                    status: None,
                    detail: None,
                    integrated: None,
                })
                .collect(),
            phase: GroupPhase::Going,
            message: String::new(),
        }
    }

    pub fn going(&self) -> bool {
        self.phase == GroupPhase::Going
    }

    /// Some target failed, was refused or never ran: Retry has something to do.
    pub fn has_retryable(&self) -> bool {
        self.rows.iter().any(|r| {
            matches!(
                r.status,
                Some(
                    TargetStatus::Failed { .. }
                        | TargetStatus::Rejected { .. }
                        | TargetStatus::Unknown
                        | TargetStatus::NotAttempted
                )
            )
        })
    }

    /// A new run, a retry or a continue started.
    pub fn on_started(slot: &mut Option<GroupRun>, request: &RemoteGroupRunRequest) {
        let fresh = request.resume.is_none() && !request.retry;
        if fresh {
            if slot
                .as_ref()
                .is_none_or(|r| &r.request != request || !r.going())
            {
                *slot = Some(GroupRun::from_request(request));
            }
            return;
        }
        let Some(run) = slot.as_mut() else {
            return;
        };
        if request.resume == Some(gitpoulpi_core::domain::GroupResume::Abort) {
            return;
        }
        run.phase = GroupPhase::Going;
        if request.retry {
            for row in run.rows.iter_mut().filter(|r| {
                matches!(
                    r.status,
                    Some(
                        TargetStatus::Failed { .. }
                            | TargetStatus::Rejected { .. }
                            | TargetStatus::Unknown
                            | TargetStatus::NotAttempted
                    )
                )
            }) {
                row.status = None;
                row.detail = None;
            }
        }
    }

    /// One target moved.
    pub fn on_progress(
        slot: &mut Option<GroupRun>,
        run_id: &str,
        remote: &str,
        status: TargetStatus,
    ) {
        let Some(run) = slot.as_mut().filter(|r| r.going()) else {
            return;
        };
        match &run.run_id {
            None => run.run_id = Some(run_id.to_string()),
            Some(id) if id != run_id => return,
            Some(_) => {}
        }
        if let Some(row) = run.rows.iter_mut().find(|r| r.remote == remote) {
            row.status = Some(status);
        }
    }

    /// The run's answer: the final rows.
    pub fn on_outcome(slot: &mut Option<GroupRun>, outcome: &RemoteGroupOutcome) {
        let Some(run) = slot.as_mut() else {
            return;
        };
        run.run_id = Some(outcome.run_id.clone());
        run.message = outcome.message.clone();
        for t in &outcome.targets {
            let at = run
                .rows
                .iter()
                .position(|r| r.remote == t.remote && r.branch == t.branch);
            let row = match at {
                Some(i) => &mut run.rows[i],
                None => {
                    run.rows.push(GroupRow {
                        remote: t.remote.clone(),
                        branch: t.branch.clone(),
                        status: None,
                        detail: None,
                        integrated: None,
                    });
                    run.rows.last_mut().expect("just pushed")
                }
            };
            row.status = Some(t.status.clone());
            row.detail = t.detail.clone();
            row.integrated = t.integrated;
        }
        run.phase = GroupPhase::Ended(outcome.status);
    }

    pub fn on_failed(slot: &mut Option<GroupRun>, why: &str) {
        if let Some(run) = slot.as_mut() {
            run.phase = GroupPhase::Failed(why.to_string());
        }
    }
}

// ---- process-wide facts the palette's generated rows read -------------------------------------------

/// What the palette's families (fn pointers, no state) can read: the jobs going and whether the Git
/// operations feed holds an error. The link host keeps them current.
#[derive(Default)]
struct Process {
    jobs: Vec<JobInfo>,
    error: bool,
}

static PROCESS: std::sync::Mutex<Process> = std::sync::Mutex::new(Process {
    jobs: Vec::new(),
    error: false,
});

fn process() -> std::sync::MutexGuard<'static, Process> {
    PROCESS.lock().unwrap_or_else(|e| e.into_inner())
}

/// The daemon's jobs, as the newest link host knows them.
pub fn publish_jobs(jobs: Vec<JobInfo>) {
    process().jobs = jobs;
}

/// The jobs that can be cancelled right now, oldest first.
pub fn cancellable_jobs() -> Vec<JobInfo> {
    process()
        .jobs
        .iter()
        .filter(|j| j.is_cancellable())
        .cloned()
        .collect()
}

/// Whether the feed holds an error ("copy the last error" has something to copy).
pub fn set_error_present(present: bool) {
    process().error = present;
}

pub fn error_present() -> bool {
    process().error
}
