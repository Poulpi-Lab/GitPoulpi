//! `Ctrl+N` badges over the Settings screen.
//!
//! A badge is a click: [`PreferencesState::nav_targets`] lists, from the rects
//! the last paint captured for the mouse, a point on each control that is on
//! screen, and [`PreferencesState::nav_press`] sends a left click there. The
//! badge and the mouse can therefore never disagree: a text field opens for
//! editing, a checkbox toggles, a combobox opens, a button runs, a list row
//! takes the keyboard, a sub-tab comes to the front.
//!
//! Nothing is badged while a surface of Settings' own owns the keyboard (a
//! field being typed into, an open combobox, the search popup, a modal editor
//! or a confirmation): those keep their own `Ctrl+N`.

use super::*;

/// Where a badge clicks: a screen cell.
pub(crate) type NavPoint = (u16, u16);

impl PreferencesState {
    /// Whether one of Settings' own modal surfaces is up.
    fn nav_blocked(&self) -> bool {
        self.confirm_action.is_some()
            || self.close_ask.is_some()
            || self.confirm_undo_off.is_some()
            || self.search.is_some()
            || self.choice.is_some()
            || self.edit.is_some()
            || self.ai.prompt_open()
            || self.wf_editor_open()
            || self.rp_editor_open()
            || (self.list_shown() && self.list.owns_pointer())
    }

    /// What `Ctrl+N` badges, in the order the badges are handed out: the
    /// search box, the section list (one badge, on the current section), the
    /// sub-tabs, the list pane and its buttons, the page's controls top to
    /// bottom, then the close cross. Only what the last paint put on screen.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, NavPoint)> {
        if self.nav_blocked() {
            return Vec::new();
        }
        // A button's badge starts on the gap before it, so it hides fewer
        // letters of the label.
        let before = |r: Rect| Rect::new(r.x.saturating_sub(1), r.y, r.width + 1, 1);
        let mut out: Vec<(Rect, NavPoint)> = Vec::new();
        if let Some(r) = self.search_box {
            out.push((before(r), (r.x + r.width / 2, r.y)));
        }
        if let Some((y, _)) = self.sidebar_rows.iter().find(|(_, i)| *i == self.section) {
            let area = self.sidebar_area;
            out.push((
                Rect::new(
                    area.x.saturating_sub(1),
                    *y,
                    area.width.saturating_sub(1),
                    1,
                ),
                (area.x + 2, *y),
            ));
        }
        for (r, _) in &self.tab_rects {
            out.push((
                Rect::new(r.x.saturating_sub(2), r.y, r.width + 2, 1),
                (r.x + 1, r.y),
            ));
        }
        if self.list_shown() && self.list_area.width > 0 {
            for (r, hit) in self.list.nav_hits() {
                if matches!(hit, list_editor::ListHit::Row(_)) {
                    out.push((r, (r.x + r.width / 2, r.y)));
                } else {
                    out.push((before(r), (r.x, r.y)));
                }
            }
        }
        // The page's controls, top to bottom. A row of chips or inline
        // buttons is badged control by control; any other row once, on its
        // first visible line.
        let mut controls: Vec<(u16, u16, Rect, NavPoint)> = Vec::new();
        let with_cells: std::collections::HashSet<usize> = self
            .chip_rects
            .iter()
            .chain(self.btn_rects.iter())
            .map(|(_, field, _)| *field)
            .collect();
        for (r, _, _) in self.chip_rects.iter().chain(self.btn_rects.iter()) {
            controls.push((r.y, r.x, before(*r), (r.x, r.y)));
        }
        let left = self.fields_x;
        let width = self.body_area.right().saturating_sub(left);
        let mut seen = std::collections::HashSet::new();
        for (y, index) in &self.field_rows {
            if with_cells.contains(index) || !seen.insert(*index) {
                continue;
            }
            controls.push((
                *y,
                left,
                Rect::new(left.saturating_sub(2), *y, width, 1),
                (left + 1, *y),
            ));
        }
        controls.sort_by_key(|(y, x, _, _)| (*y, *x));
        out.extend(controls.into_iter().map(|(_, _, r, p)| (r, p)));
        if let Some(r) = self.close_rect {
            out.push((before(r), (r.x + 1, r.y)));
        }
        out
    }

    /// Whether the point is on a text input box (a text, number or area
    /// field, the search box, a field being edited): the I-beam. Only the box
    /// itself, never the row's label or help; what the paint registered.
    pub(crate) fn pointer_text_at(&self, col: u16, row: u16) -> bool {
        crate::pointer_map::at(col, row) == Some(crate::pointer_map::Kind::Field)
    }

    /// Whether a click at the point is on a compact control that acts on its
    /// own: the pointing hand (a checkbox, a combobox field, a button, a chip,
    /// a tab, a cross). A row (the section list, a field row, a list row) is
    /// the arrow and lights on hover, and clicking its label or help still
    /// acts on its control. What the paint registered.
    pub(crate) fn pointer_hand_at(&self, col: u16, row: u16) -> bool {
        crate::pointer_map::at(col, row) == Some(crate::pointer_map::Kind::Hand)
    }

    /// The pending prefix's hint (`None` when none waits).
    pub(crate) fn set_mode_hint(&mut self, hint: Option<String>) {
        self.mode_hint = hint;
    }

    /// A pressed badge: the left click at its point. Whether the screen stays
    /// open (`false` when the click closed it).
    pub(crate) fn nav_press(&mut self, point: NavPoint) -> bool {
        self.on_mouse_shift(
            MouseEventKind::Down(MouseButton::Left),
            point.0,
            point.1,
            false,
        )
    }
}
