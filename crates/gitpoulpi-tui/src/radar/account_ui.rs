//! The account-wide part of the Radar view: folding the worker's answers in,
//! the `[Turn on]` of a hint row and the two checkboxes.

use super::*;

impl RadarUi {
    /// Folds one account-wide answer in, like [`Self::apply_answer`]: a
    /// failed or paused one keeps the rows an earlier answer had.
    pub(super) fn apply_account(&mut self, mut new: AccountAnswer) {
        let Some(i) = self.account_answers.iter().position(|a| a.id == new.id) else {
            self.account_answers.push(new);
            return;
        };
        let prev = &self.account_answers[i];
        let failed = new.problem.is_some() || new.paused.is_some();
        if new.cached && !prev.cached && !failed {
            return;
        }
        if failed && new.items.prs.is_empty() && new.items.issues.is_empty() {
            new.items = prev.items.clone();
            new.fetched_at = prev.fetched_at;
        }
        self.account_answers[i] = new;
    }

    /// What a round starts with: which lists are asked, which hosts are
    /// offered, which clones exist.
    pub(super) fn apply_setup(&mut self, setup: Setup) {
        self.account_answers
            .retain(|a| setup.active.contains(&a.id));
        self.active = setup.active;
        self.offers = setup.offers;
        self.clones = setup.clones;
    }

    /// Turns on the review requests of the offered code host `index`, then
    /// asks again. Writes `integrations.radar.review_requests` in
    /// `config.toml` (a small file, like every other setting saved from here).
    pub(super) fn turn_on(&mut self, index: usize) -> Outcome {
        let Some(offer) = self.offers.get(index).cloned() else {
            return self.say(msg::radar::MSG_NOTHING_TO_TURN_ON.get().to_string());
        };
        let Some(dir) = self.config_dir.clone() else {
            return self.say(msg::radar::msg_turn_on_failed(
                msg::radar::WHY_NO_CONFIG.get(),
            ));
        };
        match account_fs::turn_on_review_requests(&dir, &offer.id) {
            Ok(()) => {
                self.offers.retain(|o| o.id != offer.id);
                self.force_account = true;
                self.refresh();
                self.say(msg::radar::msg_turned_on(offer.label))
            }
            Err(why) => self.say(msg::radar::msg_turn_on_failed(why)),
        }
    }

    /// `a` / the checkbox: account-wide rows of repositories with no known
    /// clone are listed or hidden.
    pub(super) fn toggle_not_cloned(&mut self) -> Outcome {
        self.state.show_not_cloned = !self.state.show_not_cloned;
        self.save_state();
        self.rebuild();
        self.say(if self.state.show_not_cloned {
            msg::radar::MSG_NOT_CLONED_SHOWN.get().to_string()
        } else {
            msg::radar::MSG_NOT_CLONED_HIDDEN.get().to_string()
        })
    }

    /// `d` / the checkbox: draft requests of the account-wide rows.
    pub(super) fn toggle_drafts(&mut self) -> Outcome {
        self.state.show_drafts = !self.state.show_drafts;
        self.save_state();
        self.rebuild();
        self.say(if self.state.show_drafts {
            msg::radar::MSG_DRAFTS_SHOWN.get().to_string()
        } else {
            msg::radar::MSG_DRAFTS_HIDDEN.get().to_string()
        })
    }
}
