//! Selecting several rows of the Log and what that selection does to the right pane: Ctrl+click and
//! Space toggle a row, Shift+click and Shift+Up / Down take a range, Esc (or a plain click) goes back to
//! the single row. Two or more rows turn the right pane into a comparison.

use std::collections::HashSet;
use std::sync::Arc;

use gitpoulpi_app::log_multiselect::{LogSelection, RowKey as MarkKey};
use gitpoulpi_i18n::msg;
use gpui_kit::{Context, Modifiers, ScrollStrategy, Window};

use super::compare::{self, CompareSpec};
use super::{LogEvent, LogPanel};

/// The display indices of the selected rows.
fn marked_rows(marks: &LogSelection, display: &[Option<MarkKey>]) -> HashSet<usize> {
    display
        .iter()
        .enumerate()
        .filter(|(_, k)| k.as_ref().is_some_and(|k| marks.contains(k)))
        .map(|(i, _)| i)
        .collect()
}

impl LogPanel {
    /// A click on row `ix` with the keys held: Ctrl toggles, Shift takes a range, nothing selects it alone.
    pub fn click_row(
        &mut self,
        ix: usize,
        mods: Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus.focus(window, cx);
        let toggle = mods.control || (cfg!(target_os = "macos") && mods.platform);
        if toggle || mods.shift {
            let Some(key) = self.data.mark_at(ix) else {
                return;
            };
            let cursor = self.data.mark_at(self.selected);
            if mods.shift {
                let display = self.data.display_marks();
                self.marks.extend_to(&display, ix, Some(self.selected));
            } else {
                self.marks.toggle(key, cursor);
            }
            self.move_cursor(ix);
            self.marks_changed(cx);
        } else {
            let key = self.data.mark_at(ix);
            self.select(ix, cx);
            self.marks.reset_to(key);
        }
    }

    /// Space: flips the cursor row in the selection.
    pub fn toggle_mark(&mut self, cx: &mut Context<Self>) {
        let Some(key) = self.data.mark_at(self.selected) else {
            return;
        };
        self.marks.toggle(key, Some(key));
        self.marks_changed(cx);
    }

    /// Shift+Up / Down: moves the cursor and takes the range from the anchor.
    pub fn extend_by(&mut self, delta: isize, cx: &mut Context<Self>) {
        let total = self.data.entry_count();
        if total == 0 {
            return;
        }
        let to = self.selected.saturating_add_signed(delta).min(total - 1);
        let display = self.data.display_marks();
        self.marks.extend_to(&display, to, Some(self.selected));
        self.move_cursor(to);
        self.marks_changed(cx);
    }

    /// Esc: back to the single row under the cursor.
    pub fn clear_marks(&mut self, cx: &mut Context<Self>) {
        self.drop_marks();
        self.sync_right(cx);
    }

    /// Forgets the selection without telling the right pane (the caller selects a row next).
    pub(super) fn drop_marks(&mut self) {
        self.marks.clear();
        self.swapped = false;
        self.marked_rows = Arc::default();
    }

    fn move_cursor(&mut self, ix: usize) {
        self.selected = ix;
        self.scroll.scroll_to_item(ix, ScrollStrategy::Nearest);
    }

    fn marks_changed(&mut self, cx: &mut Context<Self>) {
        self.swapped = false;
        self.recompute_marks();
        if self.marks.len() > 1 {
            let n = self.marks.len();
            self.say(msg::compare::rows_selected(n as u64), cx);
        }
        self.sync_right(cx);
    }

    /// Display rows of the marks, after the selection or the rows changed.
    pub(super) fn recompute_marks(&mut self) {
        let display = self.data.display_marks();
        self.marked_rows = Arc::new(marked_rows(&self.marks, &display));
    }

    /// Tells the right pane what to show: the comparison when two rows or more are selected, else the
    /// row under the cursor.
    pub(super) fn sync_right(&mut self, cx: &mut Context<Self>) {
        match self.compare() {
            Some(spec) => cx.emit(LogEvent::Compare(spec)),
            None => {
                if let Some(sel) = self.data.selection_at(self.selected) {
                    cx.emit(LogEvent::Selected(sel));
                }
            }
        }
        cx.notify();
    }

    /// The comparison the selected rows stand for.
    pub fn compare(&self) -> Option<CompareSpec> {
        compare::resolve(&self.marks, &self.data, self.swapped)
    }

    /// `x`: trades the two ends of a two-row comparison.
    pub fn swap_compare(&mut self, cx: &mut Context<Self>) {
        match self.compare() {
            Some(spec) if spec.can_swap => {
                self.swapped = !self.swapped;
                self.sync_right(cx);
            }
            Some(_) => self.say(msg::compare::SWAP_REFUSED.get(), cx),
            None => self.say(msg::compare::SELECT_MORE.get(), cx),
        }
    }

    /// `n` / a click on a card: the cursor goes to the next compared row (wrapping); the selection stays.
    pub fn next_compared(&mut self, cx: &mut Context<Self>) {
        let display = self.data.display_marks();
        let mut rows: Vec<usize> = marked_rows(&self.marks, &display).into_iter().collect();
        rows.sort_unstable();
        if rows.len() < 2 {
            return self.say(msg::compare::SELECT_MORE.get(), cx);
        }
        let next = rows
            .iter()
            .copied()
            .find(|r| *r > self.selected)
            .unwrap_or(rows[0]);
        self.go_to_row(next, cx);
    }

    /// Moves the cursor to display row `ix` without touching the selection.
    pub fn go_to_row(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.move_cursor(ix.min(self.data.entry_count().saturating_sub(1)));
        cx.notify();
    }
}
