//! Keyboard notches of the pane dividers.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// The keyboard's way to drag a pane divider: one notch of the Refs or the
/// Status pane's width (`PANE_NUDGE` percent of the window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneResize {
    RefsWider,
    RefsNarrower,
    StatusWider,
    StatusNarrower,
}

impl PaneResize {
    /// The two dividers after the notch, each kept inside the same limits a
    /// drag keeps them in (`compute_divider1_pct` / `compute_divider2_pct`).
    pub fn apply(self, divider1_pct: u16, divider2_pct: u16) -> (u16, u16) {
        match self {
            PaneResize::RefsWider => (
                (divider1_pct + PANE_NUDGE).min(divider2_pct.saturating_sub(MIN_PANE_PERCENT)),
                divider2_pct,
            ),
            PaneResize::RefsNarrower => (
                divider1_pct
                    .saturating_sub(PANE_NUDGE)
                    .max(MIN_PANE_PERCENT),
                divider2_pct,
            ),
            // The Status pane is what is right of divider 2.
            PaneResize::StatusWider => (
                divider1_pct,
                divider2_pct
                    .saturating_sub(PANE_NUDGE)
                    .max(divider1_pct + MIN_PANE_PERCENT),
            ),
            PaneResize::StatusNarrower => (
                divider1_pct,
                (divider2_pct + PANE_NUDGE).min(100 - MIN_PANE_PERCENT),
            ),
        }
    }
}

/// Neither of the two side panes shrinks below this share of the width.
pub const MIN_PANE_PERCENT: u16 = 10;
/// A keyboard notch of a pane width, in percent.
pub const PANE_NUDGE: u16 = 3;
