//! Horizontal scrolling of the A / B / Base panes (owner: long lines are
//! scrolled, never wrapped). One offset for every top pane, so A and B stay
//! lined up exactly like the vertical scroll; the Output scrolls itself
//! with its caret.

use crossterm::event::{KeyModifiers, MouseEventKind};

use super::view::MergeView;

/// Columns per step of a key, and per wheel notch.
pub(super) const H_STEP: usize = 8;

impl MergeView {
    /// The widest line of any top pane, in characters (tabs count as one:
    /// an upper bound is all the clamp needs).
    pub(super) fn widest_line(&self) -> usize {
        *self.widest.get_or_init(|| {
            self.model
                .merge()
                .side_lines()
                .into_iter()
                .flatten()
                .flat_map(|lines| lines.iter())
                .map(|l| crate::cell_text::cells(l))
                .max()
                .unwrap_or(0)
        })
    }

    /// Scrolls the A / B / Base panes sideways by `delta` columns.
    pub(crate) fn scroll_h_by(&mut self, delta: isize) {
        let max = self.widest_line().saturating_sub(H_STEP);
        self.h_scroll = self.h_scroll.saturating_add_signed(delta).min(max);
    }

    /// The pointer's horizontal scrolling: a sideways wheel, or the wheel
    /// with Shift held, over the top panes. Returns whether it was one.
    pub(crate) fn hscroll_mouse(
        &mut self,
        kind: MouseEventKind,
        modifiers: KeyModifiers,
        col: u16,
        row: u16,
    ) -> bool {
        if self.ui.modal() || !self.areas.top.contains((col, row).into()) {
            return false;
        }
        let step = H_STEP as isize;
        let delta = match kind {
            MouseEventKind::ScrollLeft => -step,
            MouseEventKind::ScrollRight => step,
            MouseEventKind::ScrollUp if modifiers.contains(KeyModifiers::SHIFT) => -step,
            MouseEventKind::ScrollDown if modifiers.contains(KeyModifiers::SHIFT) => step,
            _ => return false,
        };
        self.scroll_h_by(delta);
        true
    }
}
