//! The daemon's side of undo/redo: persistence, write-ahead capture
//! around the journaled operations, and running a plan through
//! `GitBackend`. The decisions live in `gitpoulpi_core::journal`; this
//! module only does I/O.
//!
//! Everything here runs under the repository's write lock (or its shared
//! lock for the two reads), and loads the journal fresh from disk each
//! time: the file, not a copy in memory, is the truth, so an embedded CLI
//! daemon and a TUI daemon on one repository cannot disagree.

mod after;
mod capture;
mod housekeeping;
mod labels;
mod manual;
mod outside;
mod purge;
mod record;
mod replay;
mod restore;
mod risk;
mod start_over;
mod state;
mod store;

pub use after::settle;
pub use capture::is_journaled;
pub use housekeeping::{flush_pending, housekeeping};
pub use outside::{
    adopt as outside_adopt, sync as outside_sync, transfer_finished, transfer_started, Busy, Caller,
};
pub use purge::{purge_check, purge_run};
pub use record::{
    begin_record, begin_record_remote, finish_record, record, Recording, RemoteDelete,
};
pub use replay::undo_redo;
pub use restore::{reflog_restore, restore_entry};
pub use start_over::{forget_staging_above, start_over_target};
pub use state::{read_list, read_state, state_after_mutation};
pub(crate) use store::in_progress;
pub use store::{git_dir, set_env};
