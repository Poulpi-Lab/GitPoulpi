//! The Jobs panel commands.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// A `Jobs: ...` palette command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobsCmd {
    /// `Jobs: Show background jobs`: opens the panel (`Ctrl+N Shift+J`).
    Show,
    /// `Jobs: Cancel <label>`: one row per job that can be canceled.
    Cancel(u64),
    /// `Jobs: Cancel all that can be canceled`.
    CancelAll,
}
