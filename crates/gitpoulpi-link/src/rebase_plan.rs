//! The interactive-rebase plan, frontend-neutral: the rows of the popup, what
//! a Pick / Reword / Squash / Fixup / Drop / Edit does to them, reordering,
//! the autosquash preview, the combined message a squash gives its receiver,
//! and how the plan becomes the daemon's action.
//!
//! Rows are shown newest-first; the plan sent to the backend is oldest-first
//! ([`RebasePlan::steps`] reverses). Squash / Fixup combine a row into the
//! older commit that follows it *below* it on screen, which the last (oldest)
//! row has none of: [`RebasePlan::apply_action`] refuses it. A frontend keeps
//! its own focus, dropdowns, mouse state and the message editor; it calls
//! these for every change of the plan and [`RebasePlan::action`] to submit.

use gitpoulpi_core::domain::{
    autosquash_order, normalize_reword_message, InteractiveRebase, Oid, RebaseAction, RebaseStep,
};

use crate::Action;

/// One row of the plan, in the popup's own (newest-first) display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseRow {
    pub commit: Oid,
    pub summary: String,
    pub action: RebaseAction,
    /// Set once a `Reword` row's message has been edited; `None` keeps
    /// the commit's own summary as the message.
    pub new_message: Option<String>,
    /// The commit's full message (summary and body), what a Reword opens
    /// on and what a squash into another row contributes.
    pub message: String,
    /// Set while the row is a Reword only because commits are squashed
    /// into it (see [`RebaseForm::sync_squash_receivers`]).
    pub auto: Option<AutoReword>,
    /// The user took the automatic Reword away by hand: it is not put
    /// back while commits are squashed into the row.
    pub no_auto: bool,
}

/// What an automatic Reword remembers: the action to give back when
/// nothing is squashed into the row any more, and the combined message
/// it last wrote (an edited message is no longer this text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoReword {
    pub previous: RebaseAction,
    pub text: String,
}

impl RebaseRow {
    /// A `Pick` row for a commit with its full `message`.
    pub fn pick(commit: Oid, summary: &str, message: &str) -> Self {
        Self {
            commit,
            summary: summary.to_string(),
            action: RebaseAction::Pick,
            new_message: None,
            message: message.to_string(),
            auto: None,
            no_auto: false,
        }
    }

    /// The message this row starts from: its full message, or its
    /// summary when only that is known.
    pub fn base_message(&self) -> &str {
        if self.message.trim().is_empty() {
            &self.summary
        } else {
            &self.message
        }
    }
}

/// The rows of an interactive rebase (or a multi-commit cherry-pick) and the
/// selection over them.
#[derive(Debug, Clone, PartialEq)]
pub struct RebasePlan {
    /// The rows in display (newest-first) order.
    pub rows: Vec<RebaseRow>,
    /// The selected row.
    pub cursor: usize,
    /// Show (and send) the plan with `fixup!` / `squash!` commits moved under
    /// their targets; baked into `rows` by [`Self::materialize`] before an edit.
    pub autosquash: bool,
    /// The rows as first given, what Reset goes back to.
    pub initial: Vec<RebaseRow>,
    /// The last refusal, worded for the user, until the next successful edit.
    pub error: Option<String>,
    /// `true` for a multi-commit cherry-pick: the plan replays the rows'
    /// (arbitrary) commits onto HEAD instead of rewriting a range.
    pub cherry_pick: bool,
}

impl RebasePlan {
    /// `rows` in newest-first order, exactly as they should first appear.
    pub fn new(rows: Vec<RebaseRow>) -> Self {
        Self {
            initial: rows.clone(),
            rows,
            cursor: 0,
            autosquash: false,
            error: None,
            cherry_pick: false,
        }
    }

    /// The actions a row's dropdown offers: four for a
    /// cherry-pick, plus Fixup and Edit for a rebase.
    pub fn choices(&self) -> &'static [RebaseAction] {
        if self.cherry_pick {
            &[
                RebaseAction::Pick,
                RebaseAction::Reword,
                RebaseAction::Squash,
                RebaseAction::Drop,
            ]
        } else {
            &[
                RebaseAction::Pick,
                RebaseAction::Reword,
                RebaseAction::Squash,
                RebaseAction::Fixup,
                RebaseAction::Drop,
                RebaseAction::Edit,
            ]
        }
    }

    /// Bakes the autosquash preview into `rows` before any edit, so the
    /// row a click or key refers to is the row that was drawn.
    pub fn materialize(&mut self) {
        if self.autosquash {
            self.rows = self.display_rows();
            self.autosquash = false;
        }
    }

    /// Moves the row at `from` to position `to` (a drag), the cursor
    /// following it.
    pub fn move_row_to(&mut self, from: usize, to: usize) {
        if from >= self.rows.len() || to >= self.rows.len() || from == to {
            return;
        }
        let row = self.rows.remove(from);
        self.rows.insert(to, row);
        self.cursor = to;
        self.sync_squash_receivers();
    }

    /// The plan the backend would run, oldest first - the reverse of
    /// display order, with `req.autosquash` handled by the backend
    /// itself rather than pre-applied here (so the preview and the
    /// actually-run plan can never disagree).
    pub fn steps(&self) -> Vec<RebaseStep> {
        self.rows
            .iter()
            .rev()
            .map(|r| RebaseStep {
                commit: r.commit,
                action: r.action,
                message: if r.action == RebaseAction::Reword {
                    Some(
                        r.new_message
                            .clone()
                            .unwrap_or_else(|| r.base_message().to_string()),
                    )
                } else {
                    None
                },
            })
            .collect()
    }

    /// The rows as they would be drawn once `autosquash` reordering is
    /// applied - a pure preview, computed on demand rather than kept in
    /// sync as a second copy of `rows`.
    pub fn display_rows(&self) -> Vec<RebaseRow> {
        if !self.autosquash {
            return self.rows.clone();
        }
        // `autosquash_order` reasons oldest-first; reverse in, reverse
        // out.
        let mut oldest_first: Vec<(Oid, String, RebaseAction)> = self
            .rows
            .iter()
            .rev()
            .map(|r| (r.commit, r.summary.clone(), r.action))
            .collect();
        autosquash_order(&mut oldest_first);
        let by_commit: std::collections::HashMap<Oid, &RebaseRow> =
            self.rows.iter().map(|r| (r.commit, r)).collect();
        oldest_first
            .into_iter()
            .rev()
            .map(|(commit, _, action)| {
                let base = by_commit.get(&commit).expect("row came from self.rows");
                RebaseRow {
                    commit,
                    summary: base.summary.clone(),
                    action,
                    new_message: base.new_message.clone(),
                    message: base.message.clone(),
                    auto: base.auto.clone(),
                    no_auto: base.no_auto,
                }
            })
            .collect()
    }

    /// Keeps every row that commits are squashed into a Reword carrying
    /// their combined message.
    ///
    /// The receiver of a Squash is the first row below it (older) that
    /// is not itself a Squash / Fixup. When at least one Squash folds
    /// into a `Pick` (or an unedited `Reword`), that row becomes a
    /// Reword whose message is its own followed by the full messages of
    /// the squashed commits, oldest first - git's combined squash
    /// message without the `# This is a combination` comments. Fixup
    /// drops its message as in git and never triggers this. When nothing
    /// is squashed into the row any more it gets its previous action
    /// back, unless the user edited the text meanwhile.
    pub fn sync_squash_receivers(&mut self) {
        for r in 0..self.rows.len() {
            if matches!(
                self.rows[r].action,
                RebaseAction::Squash | RebaseAction::Fixup
            ) {
                continue;
            }
            let mut squashed: Vec<String> = Vec::new();
            let mut k = r;
            while k > 0
                && matches!(
                    self.rows[k - 1].action,
                    RebaseAction::Squash | RebaseAction::Fixup
                )
            {
                k -= 1;
                if self.rows[k].action == RebaseAction::Squash {
                    squashed.push(self.rows[k].base_message().trim().to_string());
                }
            }
            let row = &mut self.rows[r];
            if squashed.is_empty() {
                if let Some(auto) = row.auto.take() {
                    let untouched = row.new_message.is_none()
                        || row.new_message.as_deref() == Some(auto.text.as_str());
                    if row.action == RebaseAction::Reword && untouched {
                        row.action = auto.previous;
                        row.new_message = None;
                    }
                }
                continue;
            }
            let mut combined = row.base_message().trim().to_string();
            for message in &squashed {
                combined.push_str("\n\n");
                combined.push_str(message);
            }
            match &mut row.auto {
                Some(auto) => {
                    if row.action != RebaseAction::Reword {
                        row.auto = None;
                    } else if row.new_message.is_none()
                        || row.new_message.as_deref() == Some(auto.text.as_str())
                    {
                        auto.text = combined.clone();
                        row.new_message = Some(combined);
                    }
                }
                None => {
                    if !row.no_auto
                        && (row.action == RebaseAction::Pick
                            || (row.action == RebaseAction::Reword && row.new_message.is_none()))
                    {
                        row.auto = Some(AutoReword {
                            previous: row.action,
                            text: combined.clone(),
                        });
                        row.action = RebaseAction::Reword;
                        row.new_message = Some(combined);
                    }
                }
            }
        }
    }

    /// Stores the message the popup returned for the focused row. Text
    /// that is still the automatic combined message stays "unedited".
    pub fn set_new_message(&mut self, message: String) {
        let row = &mut self.rows[self.cursor];
        if let Some(auto) = &row.auto {
            if normalize_reword_message(&message) == normalize_reword_message(&auto.text) {
                row.new_message = Some(auto.text.clone());
                return;
            }
        }
        row.new_message = Some(message);
    }

    /// Whether `index` is the oldest row (no predecessor to squash into).
    pub fn is_last_row(&self, index: usize) -> bool {
        index + 1 == self.rows.len()
    }

    /// Sets the focused row's action, refusing Squash/Fixup on the last
    /// (oldest, no predecessor) row. `true` when applied: a frontend then
    /// asks for the new message when `action` is Reword.
    pub fn apply_action(&mut self, action: RebaseAction) -> bool {
        if self.rows.is_empty() {
            return false;
        }
        self.materialize();
        if matches!(action, RebaseAction::Squash | RebaseAction::Fixup)
            && self.is_last_row(self.cursor)
        {
            self.error = Some(gitpoulpi_i18n::msg::rebase_ui::ERR_OLDEST.get().into());
            return false;
        }
        self.error = None;
        self.rows[self.cursor].action = action;
        // The row was chosen by hand: it is no longer an automatic Reword
        // (unless Reword itself was chosen), and an unedited combined
        // message goes with it.
        if action != RebaseAction::Reword {
            let row = &mut self.rows[self.cursor];
            if let Some(auto) = row.auto.take() {
                row.no_auto = true;
                if row.new_message.as_deref() == Some(auto.text.as_str()) {
                    row.new_message = None;
                }
            }
        } else {
            self.rows[self.cursor].no_auto = false;
        }
        self.sync_squash_receivers();
        true
    }

    /// Moves the selected row `delta` places (the keyboard twin of a drag).
    pub fn move_row(&mut self, delta: isize) {
        self.materialize();
        let n = self.rows.len();
        if n < 2 {
            return;
        }
        let at = self.cursor as isize;
        let to = at + delta;
        if to < 0 || to >= n as isize {
            return;
        }
        self.rows.swap(at as usize, to as usize);
        self.cursor = to as usize;
        self.sync_squash_receivers();
    }

    /// Back to the rows as first given.
    pub fn reset(&mut self) {
        self.rows = self.initial.clone();
        self.sync_squash_receivers();
        self.cursor = 0;
        self.autosquash = false;
        self.error = None;
    }

    /// Whether the plan differs from the one first shown.
    pub fn is_edited(&self) -> bool {
        self.autosquash || self.rows != self.initial
    }

    /// The daemon action that runs the plan: bakes the autosquash preview in so
    /// the receivers of the squashes it creates carry their combined messages.
    /// With `checkout` the action first checks that branch out and stops if
    /// that is refused, so the plan can never be replayed on the wrong branch.
    pub fn action(
        &mut self,
        upstream: Option<Oid>,
        onto: Option<Oid>,
        checkout: Option<&str>,
    ) -> Action {
        self.materialize();
        self.sync_squash_receivers();
        let rebase = Action::RebaseInteractive(InteractiveRebase {
            upstream,
            onto,
            steps: self.steps(),
            autosquash: self.autosquash,
            cherry_pick: self.cherry_pick,
        });
        match checkout {
            Some(branch) => Action::CheckoutThen {
                branch: branch.to_string(),
                then: Box::new(rebase),
            },
            None => rebase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    /// Newest first: third, second, first.
    fn plan() -> RebasePlan {
        RebasePlan::new(vec![
            RebaseRow::pick(oid(3), "third", "third\n\nbody three"),
            RebaseRow::pick(oid(2), "second", "second"),
            RebaseRow::pick(oid(1), "first", "first"),
        ])
    }

    #[test]
    fn steps_are_oldest_first() {
        let steps = plan().steps();
        let ids: Vec<Oid> = steps.iter().map(|s| s.commit).collect();
        assert_eq!(ids, vec![oid(1), oid(2), oid(3)]);
    }

    #[test]
    fn squash_on_the_oldest_row_is_refused() {
        let mut p = plan();
        p.cursor = 2;
        assert!(!p.apply_action(RebaseAction::Squash));
        assert_eq!(p.rows[2].action, RebaseAction::Pick);
        assert!(p.error.is_some());
    }

    #[test]
    fn squashing_into_a_row_makes_it_a_reword_with_the_combined_message() {
        let mut p = plan();
        p.cursor = 0;
        assert!(p.apply_action(RebaseAction::Squash));
        // The receiver is the row below: second, now a Reword.
        assert_eq!(p.rows[1].action, RebaseAction::Reword);
        assert_eq!(
            p.rows[1].new_message.as_deref(),
            Some("second\n\nthird\n\nbody three")
        );
        // Taking the squash back gives the row its action back.
        assert!(p.apply_action(RebaseAction::Pick));
        assert_eq!(p.rows[1].action, RebaseAction::Pick);
        assert_eq!(p.rows[1].new_message, None);
    }

    #[test]
    fn moving_and_reset() {
        let mut p = plan();
        p.move_row(1);
        assert_eq!(p.rows[1].commit, oid(3));
        assert_eq!(p.cursor, 1);
        assert!(p.is_edited());
        p.reset();
        assert_eq!(p.rows[0].commit, oid(3));
        assert!(!p.is_edited());
    }

    #[test]
    fn the_plan_becomes_a_rebase_action_and_a_branch_switch_wraps_it() {
        let mut p = plan();
        let plain = p.action(Some(oid(9)), None, None);
        assert!(matches!(plain, Action::RebaseInteractive(r) if r.steps.len() == 3));
        let wrapped = p.action(Some(oid(9)), None, Some("feature"));
        assert!(matches!(wrapped, Action::CheckoutThen { branch, .. } if branch == "feature"));
    }
}
