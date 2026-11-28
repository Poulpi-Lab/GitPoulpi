//! Several rows selected in the Log: Ctrl+click or Space toggles a row, Shift+click or Shift+Up / Down
//! takes a range from the anchor, Esc clears. The set lives next to the cursor (the row the right pane
//! follows), never instead of it, and is keyed by what a row is (`RowKey`) so a reload keeps it.
//!
//! The rules are the terminal UI's (its `log_multiselect.rs` keeps them private, so they are restated).

use std::collections::HashSet;

use super::data::RowKey;

#[derive(Debug, Default, Clone)]
pub struct Marks {
    set: HashSet<RowKey>,
    /// Where a Shift range starts: the last row toggled or clicked.
    anchor: Option<RowKey>,
}

impl Marks {
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn contains(&self, key: &RowKey) -> bool {
        self.set.contains(key)
    }

    pub fn clear(&mut self) {
        self.set.clear();
        self.anchor = None;
    }

    /// A plain click: forget the set, remember where a following Shift+click starts.
    pub fn reset_to(&mut self, anchor: Option<RowKey>) {
        self.set.clear();
        self.anchor = anchor;
    }

    /// Ctrl+click / Space: flips `key`. The row the cursor was on joins the set the moment a second one
    /// is added, the way Ctrl+click extends what is already highlighted.
    pub fn toggle(&mut self, key: RowKey, cursor: Option<RowKey>) {
        if self.set.is_empty() {
            if let Some(cursor) = cursor.filter(|c| *c != key) {
                self.set.insert(cursor);
            }
        }
        if !self.set.remove(&key) {
            self.set.insert(key.clone());
        }
        self.anchor = Some(key);
    }

    /// Shift+click / Shift+Up / Down: every row between the anchor (or `cursor_index` when there is none
    /// yet) and `target`, both display indices into `display`.
    pub fn extend_to(&mut self, display: &[RowKey], target: usize, cursor_index: Option<usize>) {
        let anchor_index = self
            .anchor
            .as_ref()
            .and_then(|a| display.iter().position(|d| d == a))
            .or(cursor_index);
        let Some(anchor_index) = anchor_index else {
            return;
        };
        if self.anchor.is_none() {
            self.anchor = display.get(anchor_index).cloned();
        }
        let (lo, hi) = if anchor_index <= target {
            (anchor_index, target)
        } else {
            (target, anchor_index)
        };
        self.set.clear();
        for key in display.iter().skip(lo).take(hi - lo + 1) {
            self.set.insert(key.clone());
        }
    }

    /// The set in the Log's own order, newest (top row) first.
    pub fn in_display_order(&self, display: &[RowKey]) -> Vec<RowKey> {
        display
            .iter()
            .filter(|k| self.set.contains(k))
            .cloned()
            .collect()
    }

    /// Display indices of the marked rows.
    pub fn rows(&self, display: &[RowKey]) -> HashSet<usize> {
        display
            .iter()
            .enumerate()
            .filter(|(_, k)| self.set.contains(k))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn k(n: u8) -> RowKey {
        RowKey::Commit(Oid([n; 20]))
    }

    fn display() -> Vec<RowKey> {
        vec![RowKey::Wip, k(1), k(2), k(3), k(4)]
    }

    #[test]
    fn ctrl_click_adds_the_cursor_row_with_the_second_one() {
        let mut m = Marks::default();
        m.toggle(k(3), Some(k(1)));
        assert_eq!(m.in_display_order(&display()), vec![k(1), k(3)]);
        m.toggle(k(3), Some(k(3)));
        assert_eq!(m.in_display_order(&display()), vec![k(1)]);
    }

    #[test]
    fn shift_takes_the_range_from_the_anchor_either_way() {
        let d = display();
        let mut m = Marks::default();
        m.extend_to(&d, 3, Some(1));
        assert_eq!(m.in_display_order(&d), vec![k(1), k(2), k(3)]);
        m.extend_to(&d, 0, None);
        assert_eq!(m.in_display_order(&d), vec![RowKey::Wip, k(1)]);
    }

    #[test]
    fn a_plain_click_starts_over_and_sets_the_anchor() {
        let d = display();
        let mut m = Marks::default();
        m.toggle(k(2), Some(k(1)));
        m.reset_to(Some(k(4)));
        assert!(m.is_empty());
        m.extend_to(&d, 2, None);
        assert_eq!(m.in_display_order(&d), vec![k(2), k(3), k(4)]);
        assert_eq!(m.rows(&d).len(), 3);
    }
}
