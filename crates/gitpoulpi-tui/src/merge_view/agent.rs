//! What an agent sees of, and puts into, the merge editor (plan
//! the merge-conflict-editor design section 8): the
//! `ui state` section, opening on a conflict, and a proposal's picks or
//! content. A proposal only fills the view: the user's Save & Mark Resolved
//! is the one thing that writes or stages anything.

use gitpoulpi_core::domain::ConflictPicks;
use serde_json::{json, Value};

use super::model::MergeModel;
use super::view::MergeView;

impl MergeView {
    /// The `merge` section of `ui state`.
    pub(crate) fn agent_state(&self) -> Value {
        let counter = self.model.counter();
        json!({
            "path": self.path().to_string_lossy().replace('\\', "/"),
            "card": self.card().map(|c| format!("{:?}", c.kind).to_lowercase()),
            "conflict": counter.current,
            "conflicts": counter.total,
            "left": counter.left,
            "dirty": self.model.is_dirty(),
            "proposed": crate::ui_intent::proposal::active()
                .is_some_and(|a| a.dialog == crate::ui_intent::proposal::Dialog::Conflict),
        })
    }

    /// Brings conflict `n` (1-based) into view. `false` when the file has
    /// no such conflict.
    pub(crate) fn show_conflict(&mut self, n: usize) -> bool {
        if n == 0 || !self.model.go_to(n - 1) {
            return false;
        }
        self.ui.cursor = 0;
        self.reveal_current();
        true
    }

    /// Fills the Output with an agent's picks. `false` when the picks do
    /// not fit the file (a different number of conflicts) or this is a card.
    pub(crate) fn propose_picks(&mut self, picks: &[ConflictPicks]) -> bool {
        if self.card().is_some()
            || self.model.is_outside()
            || picks.len() != self.model.conflict_count()
        {
            return false;
        }
        self.model.set_picks(picks);
        self.reveal_current();
        true
    }

    /// ✦ Puts the AI's suggestion into conflict `c`'s region (one undo step,
    /// the region reads as edited) and brings it into view. `false` when
    /// the region is gone or nothing changes.
    pub(crate) fn put_suggestion(&mut self, c: usize, lines: &[String]) -> bool {
        if self.card().is_some() || !self.model.set_region_lines(c, lines) {
            return false;
        }
        self.model.go_to(c);
        self.ui.cursor = 0;
        self.reveal_current();
        true
    }

    /// Fills the Output with the whole file an agent proposes (picks are off
    /// until the user starts over).
    pub(crate) fn propose_content(&mut self, text: &str) -> bool {
        if self.card().is_some() {
            return false;
        }
        let merge = self.model.merge().clone();
        let eol = merge.eol;
        self.replace_model(MergeModel::from_disk(merge, text).with_eol(eol));
        true
    }
}
