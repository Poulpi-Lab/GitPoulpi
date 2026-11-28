//! How a diff is laid out.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// The layout axis. The other axis, whole file or only the hunks, is
/// `DiffViewState::hunks_only`; every combination is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffViewMode {
    /// `+`/`-`/context interleaved in one column.
    Inline,
    /// Before and after side by side, deletes paired with adds.
    Split,
}
