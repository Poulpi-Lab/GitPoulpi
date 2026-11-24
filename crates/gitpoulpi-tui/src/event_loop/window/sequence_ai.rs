//! The Sequences page's "Create with AI" / "Edit with AI": what the dialog
//! asked of the window (send, cancel) and the answer coming back. The dialog
//! lives in `preferences/sequences_ai.rs`; the request runs through the
//! ordinary `AiUi` like every other ✦ action.

use crate::event_loop::*;
use crate::preferences::SeqAiWish;
use init::WindowState;

impl WindowState {
    /// Called every tick: tells the dialog whether AI is on and busy, and
    /// runs what it asked for.
    pub(in crate::event_loop) fn drive_sequence_ai(&mut self) {
        let enabled = self.ai_ui.enabled();
        let running = self.ai_ui.running();
        let Some(prefs) = self.preferences.as_mut() else {
            return;
        };
        prefs.seq_ai_sync(enabled, running);
        while let Some(wish) = prefs.take_seq_ai_wish() {
            match wish {
                SeqAiWish::Cancel => self.ai_ui.cancel(),
                SeqAiWish::Send(mut request) => {
                    if request.context {
                        request.remotes = self
                            .remote_ui
                            .remotes
                            .iter()
                            .map(|r| r.name.clone())
                            .collect();
                    }
                    let mut line = None;
                    self.ai_ui.start_into(
                        &self.session,
                        &self.repo,
                        AiRequest::SequenceDraft(request),
                        &mut line,
                    );
                    if let Some(why) = line {
                        prefs.seq_ai_stopped(why);
                    }
                }
            }
        }
    }

    /// The finished request: the proposal (or why there is none) goes back
    /// to the dialog. Never applied here.
    pub(in crate::event_loop) fn sequence_ai_outcome(&mut self, outcome: AiOutcome) {
        let result = match outcome {
            AiOutcome::Done(_, Ok(AiAnswer::SequenceDraft(answer))) => Ok(*answer),
            AiOutcome::Done(_, Ok(_)) => return,
            AiOutcome::Done(_, Err(why)) => Err(gitpoulpi_i18n::msg::window::ai_failed(&why)),
            AiOutcome::Canceled(_) => {
                Err(gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string())
            }
        };
        self.last_message = Some(String::new());
        match self.preferences.as_mut() {
            Some(prefs) => prefs.seq_ai_answer(result),
            None => {
                self.last_message = Some(
                    gitpoulpi_i18n::msg::sequence_ai::DLG_CLOSED
                        .get()
                        .to_string(),
                )
            }
        }
    }
}
