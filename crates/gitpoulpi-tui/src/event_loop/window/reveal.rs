//! Ticking a pending reveal (`ref_reveal.rs`): page the commit in, scroll the
//! Log to it, flash the row.

use crate::event_loop::*;
use crate::ref_reveal::{RevealTarget, FLASH_FOR};
use crate::ui_intent::plan;
use gitpoulpi_i18n::msg;
use init::WindowState;

const FEEDBACK_AFTER: Duration = Duration::from_millis(150);
const GIVE_UP_AFTER: Duration = Duration::from_secs(20);

/// Where `target` is in the Log as drawn.
fn presence(w: &WindowState, target: RevealTarget) -> plan::Presence {
    match target {
        RevealTarget::Commit(oid) => w.commit_presence(oid),
        RevealTarget::Stash { commit, base } => {
            let stashes = w.stashes_result.as_deref().unwrap_or(&[]);
            let placed = graph::GraphStash::from_stashes(stashes);
            let row = placed
                .iter()
                .position(|s| s.commit == commit)
                .and_then(|idx| {
                    w.log_order
                        .slots()
                        .iter()
                        .position(|slot| slot.stash == idx)
                })
                .and_then(|k| w.log_order.stash_display_index(k));
            match row {
                Some(row) => plan::Presence::Shown(row),
                None => w.commit_presence(base),
            }
        }
    }
}

impl WindowState {
    /// Advances the pending reveal and ends a finished flash.
    pub(in crate::event_loop) fn tick_reveal(&mut self) {
        let now = Instant::now();
        self.reveal.expire(now);
        let Some(mut p) = self.reveal.pending.take() else {
            return;
        };
        if now.duration_since(p.started) > GIVE_UP_AFTER {
            self.last_message = Some(msg::input::reveal_not_loaded(&p.label));
            return;
        }
        let visible = log_visible_rows(self.log_area);
        let facts = plan::LogFacts {
            presence: presence(self, p.target),
            loaded: self.lazy_log.loaded(),
            pages: p.pages,
            loading: self.lazy_log.is_loading(),
            can_page: self.lazy_log.next_page(0, visible).is_some(),
        };
        match plan::commit_step(&facts) {
            plan::CommitStep::Select(row) => {
                let target = match (p.select, p.target) {
                    (true, RevealTarget::Commit(commit)) => Some(commit),
                    _ => None,
                };
                if let (true, Some(commit)) = (p.keep_view, target) {
                    // The column's switch from the File History to the
                    // graph: select the commit, stay in the view.
                    select_log_commit(
                        &self.log_result,
                        &self.log_order,
                        &mut self.log_state,
                        commit,
                    );
                    self.log_follow = true;
                    return;
                }
                if let Some(commit) = target {
                    self.leave_history_for_log(commit);
                }
                self.scroll_log_to(row);
                self.reveal.flash = Some((row, now + FLASH_FOR));
                if p.noted {
                    self.last_message = None;
                }
            }
            plan::CommitStep::Hidden => {
                self.last_message = Some(msg::input::reveal_filtered(&p.label));
            }
            plan::CommitStep::GiveUp(_) => {
                self.last_message = Some(msg::input::reveal_not_loaded(&p.label));
            }
            step => {
                if let plan::CommitStep::RequestPage = step {
                    if let Some(request) = self.lazy_log.next_page(0, visible) {
                        self.lazy_log.mark_requested(request);
                        self.link.request_log_page(request);
                    }
                    p.pages += 1;
                }
                if !p.noted && now.duration_since(p.started) > FEEDBACK_AFTER {
                    p.noted = true;
                    self.last_message = Some(msg::input::reveal_loading(&p.label));
                }
                self.reveal.pending = Some(p);
            }
        }
    }

    /// Brings display row `row` into the Log's window, only moving it when
    /// it is out of view. The selection stays where it is: the Log keeps
    /// the viewport, not the cursor (`log_follow` off).
    fn scroll_log_to(&mut self, row: usize) {
        self.central_view = CentralView::Log;
        let visible = log_visible_rows(self.log_area).max(1);
        let offset = self.log_state.offset();
        if row < offset || row >= offset + visible {
            let max = self.log_len.saturating_sub(visible);
            *self.log_state.offset_mut() = row.saturating_sub(visible / 3).min(max);
        }
        self.log_follow = false;
    }
}
