//! The three-way merge editor (see the merge-conflict-editor design).
//!
//! * [`model`] (S3): the pure state: picks, the Output document, undo.
//! * [`layout`] (S5): which layout fits, the geometry, the toolbar's
//!   degradation, the row bookkeeping behind the scroll sync.
//! * [`view`] (S5): [`MergeView`], the model plus what only the screen
//!   knows (scroll, focus, tab, highlighters).
//! * [`render`] (S5): the paint walk, which returns the hit rects.
//!
//! * [`cards`] (S8): what shows instead of the panes when the conflict is
//!   not text.
//! * [`input`] (S6): keys, mouse and the command every surface runs.
//! * [`overlay`] (S6): the popup menu, the boxes and the strip.
//!
//! The IPC lives in the window (`merge_flow.rs`); Output editing beyond
//! typing is slice S7.

mod agent;
pub(crate) mod cards;
mod hscroll;
pub(crate) mod input;
mod labels;
#[allow(dead_code)]
pub(crate) mod layout;
#[allow(dead_code)]
pub(crate) mod model;
mod overlay;
#[allow(dead_code)]
mod render;
#[allow(dead_code)]
pub(crate) mod view;

#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod perf_tests;
#[cfg(test)]
pub(crate) mod render_tests;

use gitpoulpi_core::domain::{ConflictResolution, ConflictSide, Oid};

pub(crate) use input::{Effect, MergeCmd, Outcome, Step, Then};
pub(crate) use view::MergeView;

/// What writing the Output asks of the daemon (`ResolveConflict`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SaveRequest {
    pub path: std::path::PathBuf,
    pub resolution: ConflictResolution,
    pub stage: bool,
    /// The stage oids the editor opened on (the stale guard).
    pub expected: [Option<Oid>; 3],
}

mod cmd;
pub(crate) use cmd::MergePane;

/// Which scrollbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeScroll {
    /// The A / B (/ Base) panes, which scroll together.
    Top,
    Output,
}

/// Everything clickable the merge view draws, registered from the paint
/// walk (maintainer rule 6) and carried by `HitTarget::Merge`. The pointer
/// handling is slice S6; the targets and their hover styles are here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeHit {
    /// `✕`.
    Close,
    /// `‹` / `›` in the header: the previous / next conflicted file.
    PrevFile,
    NextFile,
    /// `▲` / `▼`.
    PrevConflict,
    NextConflict,
    /// Take all A / all B (the whole file, every conflict).
    TakeAllA,
    TakeAllB,
    /// Reset: every conflict back to unresolved.
    ResetAll,
    /// A conflict's mark on a scrollbar: jumps to that conflict.
    ConflictMark(usize),
    /// `[Base]`: show / hide the Base pane.
    ToggleBase,
    /// `[Full file]`: whole-file diff coloring in A and B.
    ToggleFullFile,
    /// `✦ Suggest`: the AI suggests a resolution for the current conflict.
    AiSuggest,
    /// `…`: the menu the narrow toolbar moves Take all and Base into.
    More,
    /// Write the file, stay conflicted.
    Save,
    /// Write the file and stage it.
    SaveResolve,
    /// A tab of the narrow layout's strip.
    Tab(ConflictSide),
    /// The checkbox on a conflict frame's header.
    Chunk {
        conflict: usize,
        side: ConflictSide,
    },
    /// A conflict line of a side: its `+` / `✓` gutter and the row.
    Line {
        conflict: usize,
        side: ConflictSide,
        line: usize,
    },
    /// The `1st` / `2nd` badge: swaps the order.
    Badge {
        conflict: usize,
    },
    /// The Output's title rule: drag to resize.
    Splitter,
    /// The separator after top pane `n` (0 = between the first two):
    /// drag to resize the panes either side of it.
    Divider(u8),
    /// A document line of the Output.
    OutputLine(usize),
    /// The `‹ Conflict n · nothing picked yet ›` row.
    Placeholder {
        conflict: usize,
    },
    /// A scrollbar's thumb.
    Thumb(MergeScroll),
    /// The rest of a scrollbar: end arrows and the track around the thumb.
    Bar(MergeScroll),
    /// A row of the open popup menu.
    MenuRow(usize),
    /// A button of the card (`cards.rs`), by its index in the card.
    Choice(usize),
    /// The unsaved-changes box (also the confirm box): its primary
    /// button, its second one and Cancel.
    PromptSave,
    PromptDiscard,
    PromptCancel,
    /// The box's close cross: Cancel, like Esc.
    PromptClose,
    /// The two buttons of the strip that replaces the key hints.
    StripPrimary,
    StripSecondary,
}
