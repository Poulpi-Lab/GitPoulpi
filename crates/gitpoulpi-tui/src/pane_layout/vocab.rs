//! Keyboard notches of the pane dividers.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// The keyboard's way to drag a pane divider: one notch of the Refs or the
/// Status pane's width (`PANE_NUDGE` percent of the window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PaneResize {
    RefsWider,
    RefsNarrower,
    StatusWider,
    StatusNarrower,
}
