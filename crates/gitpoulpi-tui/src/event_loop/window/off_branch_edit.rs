//! The edit that waited for a checkout (`crate::off_branch_edit`): opens the
//! Edit commit message / Amend popup once the branch is checked out.

use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// One step per tick.
    pub(in crate::event_loop) fn follow_off_branch_edit(&mut self) {
        if !crate::off_branch_edit::waiting() {
            return;
        }
        // Another dialog is up: the edit waits behind it.
        if !matches!(self.input_mode, InputMode::Normal) {
            return;
        }
        match crate::off_branch_edit::step(&self.log_result, &self.refs_result) {
            Some(crate::off_branch_edit::Step::Open(form)) => {
                self.input_mode = InputMode::EditingCommitMessage(form);
            }
            Some(crate::off_branch_edit::Step::GiveUp(message)) => {
                self.last_message = Some(message);
            }
            Some(crate::off_branch_edit::Step::Wait) | None => {}
        }
    }
}
