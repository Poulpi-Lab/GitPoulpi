//! The TUI's connection to its session daemon: a `gitpoulpi-ipc` client
//! rather than a direct `GitBackend`. Reads go through a `Request`/`Response`
//! round trip, and a `RepoChanged` event pushed by the daemon (because *any*
//! client changed the repo) triggers an immediate refresh, not just the 2 s
//! poll.
//!
//! Everything runs on one dedicated OS thread with its own small tokio
//! runtime, off the render/input loop, so a slow request never stalls
//! keypresses. Results come back to the synchronous render loop over plain
//! `std::sync::mpsc` channels it polls every frame.
//! `docs/architecture/session-and-state.md` ("Daemon/IPC implementation
//! status") records what is real and what is planned.
//!
//! Layout:
//! - `handle`: `DaemonLink`, the render loop's end, and `start`.
//! - `messages` and [`action`]: the types that cross between the two.
//! - `link_loop`: the select loop on the link thread; `reader` is the
//!   socket reader task, where an inbound daemon event kind is added.
//! - `refresher` and [`refresh`]: the single-flight refresh and its reads.
//! - `run_action`: performs an `Action`; `Action::request` covers the
//!   one-request ones.
//! - `file_insight` and [`workers`]: on-demand reads (file content, blame,
//!   history, commit files, commit stats).
//! - `conn`: the connection, `LinkConn::call`, and starting the daemon.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc as std_mpsc, Arc};
use std::time::Duration;

use gitpoulpi_core::domain::{
    short, Blame, BlameContent, Blob, BlobContent, BlobSource, CheckoutTarget, CherryPickOptions,
    CherryPickOutcome, CommitGraph, CommitOptions, CommitStat, ConflictEntry, ConflictResolution,
    ConflictSide, DiffFileList, DiffTarget, DiscardTarget, FastForwardOutcome, FetchOptions,
    FileDiff, FileStatus, HunkId, InteractiveRebase, LineSelector, LogOptions, MergeOptions, Oid,
    OperationKind, OperationState, PatchApplied, PatchCreated, PatchSource, PullOptions,
    PushOptions, PushRejection, RebaseOptions, RebaseOutcome, RebaseState, Ref, Remote, RepoRef,
    ResetMode, RevertOutcome, RewordOutcome, SplitOutcome, StageTarget, Stash, StashId, Submodule,
    SubmoduleUpdateOptions, TagSelection, WorkingTree, Worktree, WorktreeAddOptions,
};
use gitpoulpi_core::{Error as CoreError, GitBackend, Result as GitResult};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{ChangeScope, Event, Request, Response, ServerMessage};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{oneshot, Mutex as AsyncMutex};

use crate::commit_files_cache::{
    CommitFilesCache, CommitFilesPurpose, CommitFilesQueue, QueuedFetch,
};

mod action;
mod conn;
mod convert_summary;
pub(crate) mod error_text;
mod file_insight;
mod handle;
mod link_loop;
mod merge;
mod messages;
mod outcome;
mod raw;
mod reader;
mod refresh;
mod refresher;
mod run_action;
mod takeover;
mod workers;

pub use self::action::*;
pub(crate) use self::conn::*;
use self::file_insight::*;
pub use self::handle::*;
use self::link_loop::*;
pub use self::merge::*;
pub use self::messages::*;
pub(crate) use self::outcome::*;
use self::raw::spawn_raw;
pub(crate) use self::raw::{RawQueue, Reply, Requester};
use self::reader::*;
pub(crate) use self::refresh::*;
use self::refresher::*;
use self::run_action::*;
use self::takeover::*;
use self::workers::*;

#[cfg(test)]
mod action_requests_tests;
#[cfg(test)]
mod commit_files_tests;
#[cfg(test)]
mod link_lifetime_tests;
#[cfg(test)]
mod outcome_tests;
#[cfg(test)]
mod takeover_tests;
#[cfg(test)]
mod tests;
