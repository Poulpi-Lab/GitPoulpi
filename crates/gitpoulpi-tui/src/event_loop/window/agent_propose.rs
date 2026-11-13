//! An external agent's proposals (see the mcp-show-ui design
//! section 11): an existing dialog opened pre-filled, marked "✦ Proposed by
//! <client>", for the user to confirm, edit or cancel.
//!
//! Opening only sets what a click on the matching menu entry sets (the
//! Reword popup, the composer's fields, the inline New Branch / New Tag box);
//! nothing here can submit an `Action`. The dialogs' own confirm paths tell
//! [`proposal::confirmed`] what the user confirmed, and
//! [`WindowState::agent_propose_tick`] settles the proposal once its dialog
//! has closed.

use super::agent_view::{en, Applied};
use crate::event_loop::*;
use crate::ui_intent::proposal::{self, Active, Dialog};
use gitpoulpi_ipc::ui::{sanitize_display, Proposal, ProposalStatus, UiOrigin, UiOutcome};
use init::WindowState;

fn refused(id: &str, kind: &str, client: &str, reason: String) -> Applied {
    proposal::record(id, kind, client, ProposalStatus::Failed);
    Applied::Not(Box::new(UiOutcome::Refused { reason }))
}

fn not_found(id: &str, kind: &str, client: &str, what: String) -> Applied {
    proposal::record(id, kind, client, ProposalStatus::Failed);
    Applied::Not(Box::new(UiOutcome::NotFound { what }))
}

impl WindowState {
    /// Opens `proposal`'s dialog pre-filled (the window already is on its
    /// repository).
    pub(super) fn apply_propose(
        &mut self,
        id: &str,
        proposal: &Proposal,
        origin: &UiOrigin,
    ) -> Applied {
        let client = sanitize_display(&origin.client);
        let kind = proposal.kind();
        if let Some(open) = proposal::active() {
            return refused(
                id,
                kind,
                &client,
                en(|| gitpoulpi_i18n::msg::agent_reason::proposal_pending(&open.id)),
            );
        }
        let mut proposal = proposal.clone();
        // Defence in depth: the daemon and the commands layer did it too.
        proposal.sanitize();
        let (dialog, shown) = match &proposal {
            Proposal::ConflictResolution {
                path,
                expected,
                picks,
                content,
            } => {
                if let Err(why) =
                    self.propose_conflict(path, *expected, picks.clone(), content.clone())
                {
                    return refused(id, kind, &client, why);
                }
                (Dialog::Conflict, String::new())
            }
            Proposal::Reword { commit, message } => {
                let form = match reword_form_for(&self.log_result, &self.refs_result, *commit) {
                    Ok(form) => form.proposed(message, &client),
                    Err(why) => return not_found(id, kind, &client, why),
                };
                select_log_commit(
                    &self.log_result,
                    &self.log_order,
                    &mut self.log_state,
                    *commit,
                );
                self.central_view = CentralView::Log;
                let shown = form.text();
                self.input_mode = InputMode::EditingCommitMessage(Box::new(form));
                (Dialog::Reword(*commit), shown)
            }
            Proposal::SplitCommit { commit, parts } => {
                if !matches!(self.input_mode, InputMode::Normal) {
                    return refused(
                        id,
                        kind,
                        &client,
                        en(|| {
                            gitpoulpi_i18n::msg::split_ui::AI_DIALOG_OPEN
                                .get()
                                .to_string()
                        }),
                    );
                }
                if let Err(why) = reword_form_for(&self.log_result, &self.refs_result, *commit) {
                    return not_found(id, kind, &client, why);
                }
                select_log_commit(
                    &self.log_result,
                    &self.log_order,
                    &mut self.log_state,
                    *commit,
                );
                self.central_view = CentralView::Log;
                // The editor opens when the daemon has numbered the changes.
                self.pending_split_proposal = Some((*commit, parts.clone()));
                self.link.request_split_source(*commit);
                (Dialog::Split(*commit), String::new())
            }
            Proposal::Rewrite { recipe } => {
                if !crate::build_features::HISTORY_REWRITE {
                    return refused(
                        id,
                        kind,
                        &client,
                        en(|| gitpoulpi_i18n::msg::typed_error::not_built("history-rewrite")),
                    );
                }
                if let Some(why) = proposal.problem() {
                    return refused(id, kind, &client, why);
                }
                if !matches!(self.input_mode, InputMode::Normal) {
                    return refused(
                        id,
                        kind,
                        &client,
                        en(|| {
                            gitpoulpi_i18n::msg::split_ui::AI_DIALOG_OPEN
                                .get()
                                .to_string()
                        }),
                    );
                }
                let text = proposal.text();
                input::open_rewrite_proposed(&mut self.command_ctx(), (**recipe).clone(), &client);
                (Dialog::Rewrite, text)
            }
            Proposal::CommitMessage { summary, body } => {
                if !self.commit_summary.trim().is_empty()
                    || !self.commit_description.trim().is_empty()
                {
                    return refused(
                        id,
                        kind,
                        &client,
                        en(|| {
                            gitpoulpi_i18n::msg::agent_reason::COMPOSER_DRAFT
                                .get()
                                .to_string()
                        }),
                    );
                }
                if has_wip_row(&self.status_result) {
                    // The WIP row and the Status pane, as `ui show wip`.
                    let _ = self.show_wip(None, false, false);
                } else {
                    self.right_panel = RightPanel::Status;
                    self.right_pane_collapsed = false;
                    self.focus = Focus::Status;
                }
                self.commit_summary = summary.clone();
                self.commit_description = body.clone();
                self.composer_focus = Some(ComposerField::Summary);
                self.field_editor.caret_mut(MainField::Summary).set_cursor(
                    &self.commit_summary,
                    self.commit_summary.len(),
                    false,
                );
                (Dialog::CommitMessage, proposal.text())
            }
            Proposal::Branch { name, start_point } => {
                self.open_ref_name(RefNameOp::NewBranchAt(*start_point), name, *start_point);
                (Dialog::Branch(*start_point), name.clone())
            }
            Proposal::Tag {
                name,
                target,
                message,
            } => {
                let op = if message.is_some() {
                    RefNameOp::NewAnnotatedTagAt(*target)
                } else {
                    RefNameOp::NewTagAt(*target)
                };
                self.open_ref_name(op, name, *target);
                (Dialog::Tag(*target), proposal.text())
            }
        };
        let tag_message = match &proposal {
            Proposal::Tag { message, .. } => message.clone(),
            _ => None,
        };
        proposal::open(Active {
            id: id.to_string(),
            client: client.clone(),
            what: crate::ui_intent::describe_proposal(&proposal),
            dialog,
            repo: self.repo_root.clone(),
            proposed: shown,
            tag_message,
            opened_at: Instant::now(),
            confirmed: None,
        });
        proposal::record(id, kind, &client, ProposalStatus::Open);
        Applied::Proposed(id.to_string())
    }

    /// The inline New Branch / New Tag box on `at`'s row, named `name`.
    fn open_ref_name(&mut self, op: RefNameOp, name: &str, at: Oid) {
        self.central_view = CentralView::Log;
        select_log_commit(&self.log_result, &self.log_order, &mut self.log_state, at);
        self.ref_name_error = None;
        self.input_mode = InputMode::InlineName {
            op,
            value: name.to_string(),
        };
        self.field_editor
            .caret_mut(MainField::RefName)
            .set_cursor(name, name.len(), false);
    }

    /// Whether `dialog` is still open in this window, pre-filled or edited.
    fn proposal_dialog_open(&self, dialog: Dialog) -> bool {
        match (dialog, &self.input_mode) {
            (Dialog::Reword(oid), InputMode::EditingCommitMessage(form)) => form.commit == oid,
            // An annotated tag's message step is a dialog.
            (Dialog::Tag(oid), InputMode::Prompt(dialog)) => matches!(
                &dialog.kind,
                crate::prompt_dialog::PromptKind::TagMessage { at, .. } if *at == oid
            ),
            (Dialog::Split(oid), mode) => {
                self.pending_split_proposal
                    .as_ref()
                    .is_some_and(|(at, _)| *at == oid)
                    || matches!(mode, InputMode::SplitCommit(form) if form.commit() == oid)
            }
            (Dialog::Conflict, _) => self.conflict_proposal_open(),
            (Dialog::Rewrite, mode) => matches!(mode, InputMode::Rewrite(_)),
            (Dialog::CommitMessage, _) => {
                !self.commit_summary.trim().is_empty() || !self.commit_description.trim().is_empty()
            }
            (Dialog::Branch(oid), InputMode::InlineName { op, .. }) => {
                matches!(op, RefNameOp::NewBranchAt(at) if *at == oid)
            }
            (Dialog::Tag(oid), InputMode::InlineName { op, .. }) => matches!(
                op,
                RefNameOp::NewTagAt(at)
                    | RefNameOp::NewAnnotatedTagAt(at) if *at == oid
            ),
            _ => false,
        }
    }

    /// Once per tick: settles the open proposal when its dialog has closed
    /// (confirmed, edited then confirmed, or canceled), or when this window
    /// is not the one it was opened in (the tab changed under it).
    pub(super) fn agent_propose_tick(&mut self) {
        let Some(active) = proposal::active() else {
            return;
        };
        let here = self.management.is_none() && active.repo == self.repo_root;
        if here && active.confirmed.is_none() && self.proposal_dialog_open(active.dialog) {
            return;
        }
        let Some(active) = proposal::close() else {
            return;
        };
        let status = match &active.confirmed {
            Some(text) => proposal::verdict(&active.proposed, text),
            None => ProposalStatus::Rejected,
        };
        proposal::settle(&active.id, status);
        crate::redraw::poke();
    }
}
