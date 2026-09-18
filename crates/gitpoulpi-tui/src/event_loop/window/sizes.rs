//! Sizes the user dragged that live outside the three-pane record: the
//! merge editor's A | Base | B weights and Output height, and the folder
//! picker's splitter. Both are remembered per session file
//! (`PaneLayout`), written when they change and never mid-drag (the B10
//! tick-diff pattern: one comparison per frame, a write only on a change).

use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// Adopts the remembered merge sizes on a freshly opened editor and
    /// saves the ones the user changed since.
    pub(in crate::event_loop) fn persist_sizes(&mut self) {
        if let Some(view) = self.merge.as_mut() {
            if !self.merge_layout_applied {
                self.merge_layout_applied = true;
                let layout = session_store::layout(&self.session);
                view.apply_layout_pref(&layout.merge_cols, layout.merge_output_pct);
                self.merge_layout_saved = view.layout_pref(layout.merge_output_pct);
            } else if !view.resizing() {
                let now = view.layout_pref(self.merge_layout_saved.1);
                if now != self.merge_layout_saved {
                    session_store::save_merge_layout(&self.session, now.0, now.1);
                    self.merge_layout_saved = now;
                }
            }
        } else {
            self.merge_layout_applied = false;
        }
        if let Some(permille) = crate::folder_picker::take_split_change() {
            session_store::save_picker_split(&self.session, permille);
        }
    }
}
