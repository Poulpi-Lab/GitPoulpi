//! The Log table columns and their commands.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use super::commit_column::Side;

/// Identifies one of the Log table's six columns - one enum for every
/// concern that needs to name a column: width allocation, on-screen
/// display order (drag-to-reorder), show/hide (right-click header menu),
/// and which column a hover/click lands in (drives per-column tooltip
/// content and header/boundary hit-testing). A single shared identity
/// instead of a near-duplicate enum per concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogColumn {
    Ref,
    Graph,
    Message,
    Author,
    Date,
    /// Files changed and lines added/removed (`3 files +12 -4`). Hidden by
    /// default, and filled lazily for the rows on screen only.
    Changes,
    Sha,
}

/// What the Log column keys and palette commands do to the picked column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogColumnCmd {
    /// Starts picking: the first visible column, or the one already picked.
    Pick,
    NextColumn,
    PrevColumn,
    /// Moves the picked column one place among the visible ones (the twin of
    /// dragging its header).
    Move(Side),
    /// Widens or narrows the picked column (the twin of dragging its
    /// boundary).
    Wider,
    Narrower,
    /// Every column back to its automatic width (the commit column's `0`).
    ResetWidth,
    /// Opens the columns chooser (the header's right-click menu).
    Chooser,
    /// Fits the picked column to its content (the twin of double-clicking its
    /// boundary).
    FitToContent,
    /// Fits every visible column to its content.
    FitAllToContent,
    /// Scrolls the columns sideways by one key step (`scroll` layout).
    ScrollLeft,
    ScrollRight,
    /// Switches between the `fit` and `scroll` layouts.
    ToggleLayout,
}
