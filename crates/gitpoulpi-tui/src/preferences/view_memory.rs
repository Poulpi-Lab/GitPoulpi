//! Where each page was left: the selected row and the scroll, per page (and
//! per sub-tab of a tabbed page), so switching back to a section, or
//! reopening Settings, lands where the user was instead of at the top.
//!
//! The memory lives on the state (`views`) and, for the process, in
//! `LAST_VIEWS` (written by `Drop`, read only by `PreferencesState::open`, so
//! unit tests stay independent). A remembered row that no longer exists
//! falls back to the page's first row, and the scroll is clamped by the
//! paint (never past the last full page), so a page whose content got
//! shorter is safe.

use super::*;

/// One page-and-tab's remembered `(field, scroll)`.
pub(super) type View = Option<(usize, usize)>;

/// The views of the last Settings screen, two slots per page (General, list).
static LAST_VIEWS: std::sync::Mutex<Vec<View>> = std::sync::Mutex::new(Vec::new());

/// The empty memory: nothing visited.
pub(super) fn blank() -> Vec<View> {
    vec![None; PrefSection::ALL.len() * 2]
}

/// Publishes `views` for the next `open`.
pub(super) fn store_last(views: &[View]) {
    if let Ok(mut last) = LAST_VIEWS.lock() {
        *last = views.to_vec();
    }
}

/// The views the previous Settings screen left (blank on the first open).
pub(super) fn load_last() -> Vec<View> {
    LAST_VIEWS
        .lock()
        .ok()
        .filter(|last| last.len() == PrefSection::ALL.len() * 2)
        .map(|last| last.clone())
        .unwrap_or_else(blank)
}

impl PreferencesState {
    /// The memory slot of the page in front and its tab.
    pub(super) fn view_slot(&self) -> usize {
        self.section.min(PrefSection::ALL.len() - 1) * 2 + (self.sub_tab() == SubTab::List) as usize
    }

    /// Remembers the selected row and the scroll of the page in front.
    pub(super) fn save_view(&mut self) {
        let slot = self.view_slot();
        if let Some(view) = self.views.get_mut(slot) {
            *view = Some((self.field, self.scroll));
        }
    }

    /// Brings back the row and scroll remembered for the page in front; a
    /// page never visited, or whose row is gone, starts on its first row.
    pub(super) fn restore_view(&mut self) {
        let remembered = self.views.get(self.view_slot()).copied().flatten();
        let fields = self.fields();
        match remembered {
            Some((field, scroll)) if fields.get(field).is_some_and(|f| f.selectable()) => {
                self.field = field;
                self.scroll = scroll;
                // The remembered scroll is not pulled back to the row.
                self.kept_in_view = Some((self.section, self.field, self.focus));
            }
            Some((_, scroll)) => {
                self.field = self.first_selectable();
                self.scroll = scroll;
                self.kept_in_view = None;
            }
            None => {
                self.field = self.first_selectable();
                self.scroll = 0;
                self.kept_in_view = None;
            }
        }
    }

    /// Goes to page `index` of the sidebar, leaving the page in front as it
    /// is remembered.
    pub(super) fn switch_section(&mut self, index: usize) {
        if index == self.section {
            return;
        }
        self.save_view();
        self.seq.in_steps = false;
        self.section = index;
        self.strip = false;
        self.restore_view();
    }
}
