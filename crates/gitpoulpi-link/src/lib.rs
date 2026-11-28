//! The daemon client every frontend shares.
//!
//! A frontend is an IPC client of the session daemon (`gitpoulpi-ipc`): it
//! never touches a `GitBackend` itself. This crate is that client. It runs on
//! one dedicated thread with its own small async runtime, off the frontend's
//! render and input loop, so a slow request never stalls a keypress or a
//! frame. Results come back over plain `std::sync::mpsc` channels the
//! frontend drains whenever it draws; a request that must be matched to its
//! answer carries a [`Ticket`].
//!
//! There is no UI toolkit in here (`tests/dependency_guard.rs` fails if one
//! becomes a dependency). Everything the client needs from the frontend that
//! runs it goes through [`LinkHost`]; everything it hands back is plain data.
//!
//! # How a frontend uses this
//!
//! 1. Implement [`LinkHost`]: `wake` (a result was queued, draw again),
//!    `before_exit`, `raise_window`, the hooks that feed the frontend's own
//!    panels (`jobs`, `push_group`, `report`, `timing`, `ask_sensitive_stage`),
//!    and the few settings it owns (`initial_log_filter`, `undo_key`).
//!    Hooks run on the thread that raised them and must return quickly.
//! 2. Call [`start`]`(session, repo, host)` once per open repository. It
//!    returns a [`DaemonLink`] at once; the daemon of the session is started
//!    when none runs. The host is kept process-wide (one process has one
//!    frontend), installed by `start`: that is the single installation point.
//! 3. Send work with the request methods of [`DaemonLink`]: `submit_action`
//!    and `submit_tracked` (which returns a [`Ticket`]), `request_refresh`,
//!    the on-demand reads (commit files, file insight, merge, log pages,
//!    commit stats) and the undo/redo calls.
//! 4. On every wake, drain the receivers of the link (`outcome_rx` for
//!    refresh results, `action_error_rx`, `push_rejected_rx`, `autostash_rx`,
//!    ...) and the ticket queue (`take_done`, `take_for`), and apply what
//!    arrived to the frontend's own state.
//!
//! The shared pure helpers the client needs (activity tracking, the
//! undo journal rows, the commit-file cache, the per-repository preferences)
//! are public modules of this crate so the frontend uses the same ones.

mod build_features;
mod client;

pub mod activity;
pub mod binary_hint;
pub mod commit_files_cache;
pub mod commit_settings;
pub mod diff_recode;
pub mod fetch_lock;
pub mod history_rename;
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

pub use client::*;
