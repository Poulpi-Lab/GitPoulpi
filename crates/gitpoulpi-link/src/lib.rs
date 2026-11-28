//! The daemon client every frontend shares.
//!
//! A frontend is an IPC client of the session daemon (`gitpoulpi-ipc`): it
//! never touches a `GitBackend` itself. This crate is that client. It runs on
//! one dedicated thread per open repository with its own small async runtime,
//! off the frontend's render and input loop, so a slow request never stalls a
//! keypress or a frame. Results come back over plain `std::sync::mpsc`
//! channels the frontend drains whenever it draws; a request that must be
//! matched to its answer carries a [`Ticket`].
//!
//! There is no UI toolkit in here (`tests/dependency_guard.rs` fails if one
//! becomes a dependency). Everything the client needs from the frontend that
//! runs it goes through [`LinkHost`]; everything it hands back is plain data.
//!
//! # A tour for a frontend
//!
//! **1. Implement [`LinkHost`].** `wake` (a result was queued, draw again),
//! `before_exit`, `raise_window`, the hooks that feed the frontend's own
//! panels ([`JobEvent`], [`PushGroupEvent`], [`Report`], [`TimingKind`],
//! `ask_sensitive_stage`), and the few settings it owns (`initial_log_filter`,
//! `undo_key`). Hooks run on the thread that raised them and must return
//! quickly. Every event hook receives the [`LinkContext`] of the link it comes
//! from (its [`LinkId`] and repository root), so a frontend with several
//! repositories open routes by link instead of filtering by path. An undo or
//! redo becoming (un)available calls `wake` only when `wake_on_journal`
//! says so (off by default).
//!
//! **2. Start a link per repository.** [`start`]`(session, repo, host)`
//! returns a [`DaemonLink`] at once; the daemon of the session is started when
//! none runs. Each link keeps the host it was started with. The latest host
//! also becomes the process-wide fallback ([`host::install`]) for the few
//! hooks a frontend thread triggers.
//!
//! **3. Send work** with the request methods of [`DaemonLink`]:
//! `submit_action` and `submit_tracked` (which returns a [`Ticket`]),
//! `request_refresh`, the on-demand reads (commit files, file insight, merge,
//! log pages, commit stats) and the undo/redo calls. [`Action::activity_label`]
//! is the translated line to show while an action runs.
//!
//! **4. On every wake, drain.** [`DaemonLink::drain`] returns what arrived
//! as typed [`LinkEvent`]s: the refresh result ([`RefreshOutcome`], whose
//! `summary` ([`RefreshSummary`]) says which parts of the state changed),
//! action errors and messages, the answers to on-demand reads, the undo
//! journal, the flags other clients raise, and finally the link's failure.
//! The receivers `drain` is built on (`outcome_rx`, `action_error_rx`, ...)
//! stay public for a frontend that prefers to read them one by one, as the
//! terminal UI does; use one way or the other for a given link. The ticket
//! queue ([`take_done`], [`take_for`]) is process-wide and separate.
//!
//! ```ignore
//! let link = gitpoulpi_link::start(session, repo, host);   // host: Arc<dyn LinkHost>
//! // in the frontend's wake handler, for the link `link.id()`:
//! for event in link.drain() {
//!     match event {
//!         LinkEvent::Refreshed(outcome) if outcome.summary.working_tree => { /* ... */ }
//!         LinkEvent::ActionError(text) => { /* ... */ }
//!         LinkEvent::Failed(reason) => { /* offer a retry */ }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # The rest of the crate
//!
//! The shared pure helpers a frontend uses next to the link are public
//! modules: [`activity`] (what runs and the in-flight text), [`undo_ui`] and
//! [`undo_label`] (the undo journal rows and reasons), [`live_settings`],
//! [`commit_settings`], [`repo_state_path`], [`wip_view`], [`rename_pairs`],
//! [`binary_hint`], [`diff_recode`], [`lfs_ui`], [`ui_viewer`] (the agent
//! request mailbox), [`redraw`], [`oid_text`] and [`no_repository`]. The
//! client itself is private; what it exposes is re-exported below.
//!
//! The `test-support` feature adds `test_host` (a [`LinkHost`] that does
//! nothing) and `ticket_tests` (a link against an embedded daemon) for a
//! frontend's own tests; a normal build never has them.

mod build_features;
mod client;

pub mod activity;
pub mod binary_hint;
pub mod commit_settings;
pub mod diff_recode;
#[cfg(feature = "hosting")]
pub mod hosting_rows;
#[cfg(feature = "hosting")]
pub mod hosting_task;
pub mod lfs_ui;
pub mod live_settings;
pub mod no_repository;
pub mod oid_text;
pub mod redraw;
pub mod rename_pairs;
pub mod repo_state_path;
pub mod ui_viewer;
pub mod undo_label;
pub mod undo_ui;
pub mod wip_view;

pub(crate) mod commit_files_cache;
pub(crate) mod fetch_lock;
pub(crate) mod history_rename;

pub use client::error_text;
pub use client::host;
pub use client::{
    // Completion tickets (process-wide).
    arm,
    armed,
    cancel_ticket,
    disarm,
    // Typed requests of views, and the daemon of the session.
    ensure_daemon,
    issue,
    // Refresh scopes and the log page size.
    log_page_limit,
    remove_published_port_files,
    report_ticket,
    set_log_page_limit,
    set_origin,
    // The link and its identity.
    start,
    start_at,
    take_armed_for_dialog,
    take_done,
    take_for,
    take_split_created,
    ticket_running,
    // What a frontend submits.
    Action,
    AutoStashNeeded,
    // What comes back.
    CommitFilesOutcome,
    CommitStatsOutcome,
    ComposedCommit,
    DaemonLink,
    DiffContentSide,
    FileDiffRequest,
    FileInsightKind,
    FileInsightOutcome,
    FileInsightPayload,
    FileInsightRequest,
    // What the frontend implements and the events its hooks receive.
    JobEvent,
    LinkContext,
    LinkEvent,
    LinkHost,
    LinkId,
    LogPageOutcome,
    MergeReply,
    MergeRequest,
    PushGroupEvent,
    PushRejectedNotice,
    RawQueue,
    ReadChanges,
    ReadHashes,
    RefreshOutcome,
    RefreshSummary,
    Reply,
    Report,
    Requester,
    SplitSourceOutcome,
    StepDone,
    StepOutcome,
    StepPayload,
    Ticket,
    TicketGuard,
    TimingKind,
    UndoKey,
    UserWait,
    WorkingDiffOutcome,
    FIRST_LOAD_LATER,
    HISTORY_PAGE,
    SUBMODULES_SCOPE,
};
#[cfg(feature = "test-support")]
pub use client::{begin, test_host, ticket_tests};
