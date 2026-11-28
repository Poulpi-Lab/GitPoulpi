//! The merge editor panes and commands.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// The panes that can have the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergePane {
    A,
    B,
    Base,
    Output,
}

/// Every action the editor has, whatever surface asked for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeCmd {
    NextConflict,
    PrevConflict,
    NextFile,
    PrevFile,
    /// The chunk of one side of the current conflict, on or off.
    ChunkA,
    ChunkB,
    TakeA,
    TakeB,
    TakeBoth,
    TakeBothReversed,
    TakeBase,
    Clear,
    /// The current conflict back to unresolved (no picks, nothing decided).
    Unpick,
    SwapOrder,
    TakeAllA,
    TakeAllB,
    ToggleBase,
    ToggleFullFile,
    Undo,
    Redo,
    Save,
    SaveResolve,
    Reset,
    CycleHeight,
    /// Widen the A (current) pane / the B (incoming) pane by a step.
    WidenCurrent,
    WidenIncoming,
    /// Back to the default pane widths and Output height.
    ResetPaneSizes,
    Close,
    FocusNext,
    FocusPrev,
    /// Jump to a pane, on the current conflict.
    FocusPane(MergePane),
    /// The neighboring pane (A, Base, B, Output), on the same conflict.
    PaneLeft,
    PaneRight,
    EditInOutput,
    /// The "edited outside" strip: load the text on disk.
    ContinueFromDisk,
    /// The "edited outside" strip: keep the fresh merge.
    StartOver,
    /// The "resolved elsewhere" strip: put the conflict back and reopen.
    Unresolve,
    /// The "changed elsewhere" strip: read the file again.
    Reload,
    /// Stage a side's whole file as it is (no merge at all).
    UseFileA,
    UseFileB,
    UseFileBase,
    /// A card: keep the file that is left (delete / modify, rename).
    Keep,
    /// A card: accept the deletion (`git rm`).
    Delete,
    /// A card: hand the file to the external merge tool.
    ExternalTool,
    /// Show or hide the Refs and Status panels around the editor.
    TogglePanes,
    /// Scroll the A / B / Base panes sideways.
    ScrollLeft,
    ScrollRight,
    /// ✦ Ask the AI for a resolution of the current conflict.
    AiSuggest,
}
