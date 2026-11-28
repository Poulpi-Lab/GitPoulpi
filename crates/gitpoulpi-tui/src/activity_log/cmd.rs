//! The Activity Log panel commands the palette and keys run.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use gitpoulpi_log::Level;

/// What the display toggles can be asked to do (keys, mouse and palette).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LevelCmd {
    Toggle(Level),
    All,
    Clear,
    /// Opens the confirmation that clears the application log files.
    ClearLog,
    /// From the selected line of the Application tab, opens the Git
    /// operations entry of the operation that produced it.
    ShowOperation,
    /// Selects the next line (further down) that names a failed operation.
    NextProblem,
    /// Selects the previous line (further up) that names a failed operation.
    PrevProblem,
    /// Opens the Scope box's list.
    Scope,
    /// Shows the Git operations tab.
    GitTab,
    /// Shows the Application tab.
    AppTab,
    /// Scrolls the lines of the tab in front one step to the left.
    ScrollLeft,
    /// Scrolls the lines of the tab in front one step to the right.
    ScrollRight,
}
