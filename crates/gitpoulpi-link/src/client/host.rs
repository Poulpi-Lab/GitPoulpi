//! What the daemon client needs from the frontend that runs it.
//!
//! The client (this module tree) never calls the frontend's own modules: every
//! outbound call goes through [`LinkHost`], which the frontend implements
//! (the terminal UI's is `daemon_link_host.rs`). The events are plain data.
//!
//! The host is handed to [`super::start`]; the link keeps it and every hook
//! it raises reaches that host with the link's [`LinkContext`] (see
//! `link_ctx`). The host passed to the latest `start` is also the process-wide
//! fallback for a hook raised off every link's threads. A hook runs on the
//! thread that raised it, exactly as the call it replaced did: the reader
//! task's hooks must stay quick and never wait on the render loop.

use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use gitpoulpi_core::domain::{
    PushRejection, RefFilter, RemoteGroupOutcome, RemoteGroupRunRequest, TargetStatus,
};
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_core::{Error as CoreError, NotCancellableReason, OpContext};
use gitpoulpi_ipc::{HostingChangeArgs, JobInfo};

use super::link_ctx::{self, LinkContext};
use super::{Action, Requester, Ticket};

/// A change to the session's background jobs.
#[derive(Debug, Clone)]
pub enum JobEvent {
    /// The job list read when the link came up.
    Seed(Vec<JobInfo>),
    /// A job started or moved.
    Changed(JobInfo),
    /// A job left the daemon's list.
    Removed(u64),
    /// A one-line notice about jobs (an identical transfer already runs).
    Say(String),
}

/// Progress of a grouped push (`RemoteGroupRun`).
#[derive(Debug, Clone)]
pub enum PushGroupEvent<'a> {
    Started {
        root: &'a Path,
        request: &'a RemoteGroupRunRequest,
    },
    Outcome {
        root: &'a Path,
        outcome: &'a RemoteGroupOutcome,
    },
    Failed {
        root: &'a Path,
        why: &'a str,
    },
    /// One target moved (pushed by the daemon while the run goes on).
    Progress {
        run_id: &'a str,
        remote: &'a str,
        status: TargetStatus,
    },
}

/// What happened to an action or a message, for the frontend's activity log.
#[derive(Debug, Clone)]
pub enum Report<'a> {
    Success {
        action: &'a Action,
        repo: &'a Path,
        elapsed: Duration,
    },
    Failure {
        action: &'a Action,
        error: &'a CoreError,
        repo: &'a Path,
        elapsed: Duration,
        context: OpContext,
    },
    Rejection {
        action: &'a Action,
        rejection: &'a PushRejection,
        repo: &'a Path,
        elapsed: Duration,
    },
    /// A line the daemon's reply carried, worth a log entry of its own.
    Message { text: &'a str, repo: &'a Path },
    /// The window's own fetch met a git lock and was skipped silently.
    BackgroundFetchSkipped { repo: &'a Path },
}

/// Which timing line a duration belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingKind {
    /// Something the user waited for.
    Foreground,
    /// The poll and the refresh after a change.
    Background,
}

/// Which key hint a message names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoKey {
    /// The key of "undo the last action".
    Undo,
    /// The key of "restore the reflog position the blocked undo offers".
    UndoOffer,
}

/// The frontend's side of the client.
///
/// Every event hook takes the [`LinkContext`] of the link it comes from (see
/// `link_ctx`): a frontend with several repositories open routes by
/// `link.id()` or `link.root()` instead of filtering. The pure queries
/// (labels, key hints, settings) carry none. An event the daemon sends to the
/// whole session (a job moved) reaches the host once per open link, each time
/// with that link's context; a frontend that wants it once keeps the first.
pub trait LinkHost: Send + Sync {
    /// A result was queued for the frontend: draw again. Called from any
    /// thread, after the send.
    fn wake(&self, link: &LinkContext);
    /// Whether an undo or redo becoming (un)available, as the daemon reports
    /// it, also calls [`LinkHost::wake`]. Off by default: the terminal UI
    /// already redraws on the journal cell's own poke and must not draw twice;
    /// a frontend that waits on `wake` alone turns it on.
    fn wake_on_journal(&self) -> bool {
        false
    }
    /// The process ends for an update: hand the terminal (or window) back.
    fn before_exit(&self, link: &LinkContext);
    /// A launcher click asks this window to come to the front. Runs programs;
    /// the client calls it off its reader task. The answer says how, or why not.
    fn raise_window(&self, link: &LinkContext) -> Result<String, String>;
    /// A stage was refused because of sensitive files: ask the user.
    fn ask_sensitive_stage(
        &self,
        link: &LinkContext,
        root: std::path::PathBuf,
        action: Action,
        files: Vec<Sensitive>,
        ticket: Option<Ticket>,
    );
    /// The link is connected: `requester` sends raw requests over it.
    fn on_connected(&self, link: &LinkContext, requester: Requester);
    fn jobs(&self, link: &LinkContext, event: JobEvent);
    fn push_group(&self, link: &LinkContext, event: PushGroupEvent<'_>);
    fn report(&self, link: &LinkContext, report: Report<'_>);
    fn timing(&self, link: &LinkContext, kind: TimingKind, repo: &Path, label: String, ms: u64);
    /// The window starts a fetch of its own (auto-fetch, fetch on open).
    fn expect_background_fetch(&self, link: &LinkContext);
    /// Whether such a fetch is still going.
    fn background_fetch_pending(&self, link: &LinkContext) -> bool;
    /// What a job is called (`#id` for one never heard of).
    fn job_label(&self, id: u64) -> String;
    /// Why a job cannot be canceled.
    fn not_cancellable(&self, reason: NotCancellableReason) -> &'static str;
    /// The refs the Log's walk leaves out for `root` (what the frontend
    /// persisted), so a link starts with its first page already filtered.
    fn initial_log_filter(&self, root: &Path) -> RefFilter;
    /// How the frontend shows the key of `which`, empty when it has none.
    fn undo_key(&self, which: UndoKey) -> String;
    /// A daemon job is going and the frontend's jobs indicator names it, so
    /// the status line's in-flight text for the same transfer stays off.
    fn daemon_job_live(&self) -> bool;
    /// Whether `change` is this process's own hosting notification coming
    /// back (the frontend already shows what it wrote).
    fn is_own_hosting_change(&self, change: &HostingChangeArgs) -> bool;
}

static HOST: RwLock<Option<Arc<dyn LinkHost>>> = RwLock::new(None);

/// Makes `host` the process-wide fallback host: the one a hook raised off
/// every link's threads reaches (the queries a frontend thread triggers, a
/// test of a piece that reads a hook without a link). [`super::start`] does
/// it; a link itself keeps the host it was started with, so links started with
/// different hosts each reach their own.
pub fn install(host: Arc<dyn LinkHost>) {
    *HOST.write().unwrap_or_else(|e| e.into_inner()) = Some(host);
}

fn installed() -> Option<Arc<dyn LinkHost>> {
    HOST.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// The host a hook raised on this thread reaches, with the link it is for:
/// the thread's link and that link's host, else the installed host and the
/// detached context.
fn current() -> Option<(Arc<dyn LinkHost>, LinkContext)> {
    match link_ctx::current() {
        Some(link) => {
            let host = link.host().or_else(installed)?;
            Some((host, link))
        }
        None => installed().map(|host| (host, LinkContext::detached())),
    }
}

pub(super) fn wake() {
    if let Some((host, link)) = current() {
        host.wake(&link);
    }
}

/// Whether the host asks for a wake when the undo journal changes.
pub(super) fn wake_on_journal() -> bool {
    current().is_some_and(|(host, _)| host.wake_on_journal())
}

pub(super) fn before_exit() {
    if let Some((host, link)) = current() {
        host.before_exit(&link);
    }
}

pub(super) fn raise_window() -> Result<String, String> {
    match current() {
        Some((host, link)) => host.raise_window(&link),
        None => Err("no host".to_string()),
    }
}

pub(super) fn ask_sensitive_stage(
    root: std::path::PathBuf,
    action: Action,
    files: Vec<Sensitive>,
    ticket: Option<Ticket>,
) {
    if let Some((host, link)) = current() {
        host.ask_sensitive_stage(&link, root, action, files, ticket);
    }
}

pub(super) fn on_connected(requester: Requester) {
    if let Some((host, link)) = current() {
        host.on_connected(&link, requester);
    }
}

pub(super) fn jobs(event: JobEvent) {
    if let Some((host, link)) = current() {
        host.jobs(&link, event);
    }
}

pub(super) fn push_group(event: PushGroupEvent<'_>) {
    if let Some((host, link)) = current() {
        host.push_group(&link, event);
    }
}

pub(super) fn report(report: Report<'_>) {
    if let Some((host, link)) = current() {
        host.report(&link, report);
    }
}

pub(super) fn timing(kind: TimingKind, repo: &Path, label: String, ms: u64) {
    if let Some((host, link)) = current() {
        host.timing(&link, kind, repo, label, ms);
    }
}

pub(super) fn expect_background_fetch() {
    if let Some((host, link)) = current() {
        host.expect_background_fetch(&link);
    }
}

pub(super) fn background_fetch_pending() -> bool {
    current().is_some_and(|(host, link)| host.background_fetch_pending(&link))
}

pub(super) fn job_label(id: u64) -> String {
    current().map_or_else(|| format!("#{id}"), |(host, _)| host.job_label(id))
}

pub(super) fn not_cancellable(reason: NotCancellableReason) -> &'static str {
    current().map_or("", |(host, _)| host.not_cancellable(reason))
}

pub(super) fn initial_log_filter(root: &Path) -> RefFilter {
    current().map_or_else(RefFilter::default, |(host, _)| {
        host.initial_log_filter(root)
    })
}

pub(crate) fn undo_key(which: UndoKey) -> String {
    current().map_or_else(String::new, |(host, _)| host.undo_key(which))
}

pub(crate) fn daemon_job_live() -> bool {
    current().is_some_and(|(host, _)| host.daemon_job_live())
}

pub(crate) fn is_own_hosting_change(change: &HostingChangeArgs) -> bool {
    current().is_some_and(|(host, _)| host.is_own_hosting_change(change))
}

/// `send`, then [`wake`]: the way a worker answers the render loop. The wake
/// comes after the send, so a loop that wakes on it always finds the value.
pub(super) trait NotifySend<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>>;
}

impl<T> NotifySend<T> for std::sync::mpsc::Sender<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        let sent = self.send(value);
        wake();
        sent
    }
}

impl<T> NotifySend<T> for std::sync::mpsc::SyncSender<T> {
    fn send_notify(&self, value: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        let sent = self.send(value);
        wake();
        sent
    }
}
