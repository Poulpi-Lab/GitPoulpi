//! An agent opening the merge conflict editor (plan
//! the merge-conflict-editor design section 8): `ui show
//! conflict` (view only) and `ui propose resolution` (the editor opens with
//! the agent's picks or content filled in; the user's Save & Mark Resolved
//! is the only confirm).
//!
//! Both only ask the daemon for the file and let [`MergeFlow`] fill the
//! view when it arrives: nothing here can write or stage anything.
//!
//! [`MergeFlow`]: crate::merge_flow::MergeFlow

use super::agent_view::{en, Applied};
use crate::event_loop::*;
use crate::merge_flow::{AgentOpen, AgentProposal, Env};
use gitpoulpi_core::domain::{ConflictEntry, ConflictPicks, Oid, ResolveOptions};
use gitpoulpi_ipc::ui::UiOutcome;
use init::WindowState;

fn same_path(entry: &ConflictEntry, path: &str) -> bool {
    entry.path.to_string_lossy().replace('\\', "/") == path
}

impl WindowState {
    /// Why the editor cannot be taken over for an agent right now, if so:
    /// it holds the user's unsaved work.
    fn merge_editor_busy(&self) -> Option<String> {
        self.merge.as_ref().filter(|v| v.model.is_dirty()).map(|v| {
            en(|| gitpoulpi_i18n::msg::agent_reason::merge_editor_busy(v.path().display()))
        })
    }

    fn open_for_agent(&mut self, request: AgentOpen) {
        let mut env = Env {
            link: &self.link,
            conflicts: &self.conflicts,
            merge: &mut self.merge,
            root: &self.repo.root,
            last_message: &mut self.last_message,
        };
        self.merge_flow.open_for_agent(&mut env, request);
    }

    /// `ui show conflict`: open the editor on `path` at conflict `n`.
    pub(super) fn apply_show_conflict(&mut self, path: &str, conflict: Option<u32>) -> Applied {
        let Some(entry) = self.conflicts.iter().find(|e| same_path(e, path)) else {
            return Applied::Not(Box::new(UiOutcome::NotFound {
                what: en(|| gitpoulpi_i18n::msg::agent_reason::not_conflicted(path)),
            }));
        };
        if let Some(why) = self.merge_editor_busy() {
            return Applied::Not(Box::new(UiOutcome::refused(why)));
        }
        let request = AgentOpen {
            path: entry.path.clone(),
            conflict: conflict.map(|n| n as usize),
            proposal: None,
        };
        self.open_for_agent(request);
        Applied::Done
    }

    /// `ui propose resolution`: open the editor with the proposal in it.
    /// `Err` is the answer when it cannot.
    pub(super) fn propose_conflict(
        &mut self,
        path: &str,
        expected: [Option<Oid>; 3],
        picks: Option<Vec<ConflictPicks>>,
        content: Option<String>,
    ) -> Result<(), String> {
        let Some(entry) = self.conflicts.iter().find(|e| same_path(e, path)) else {
            return Err(en(|| {
                gitpoulpi_i18n::msg::agent_reason::not_conflicted(path)
            }));
        };
        if ResolveOptions::oids_of(entry) != expected {
            return Err(en(|| {
                gitpoulpi_i18n::msg::agent_reason::STAGES_MOVED
                    .get()
                    .to_string()
            }));
        }
        if let Some(why) = self.merge_editor_busy() {
            return Err(why);
        }
        let request = AgentOpen {
            path: entry.path.clone(),
            conflict: None,
            proposal: Some(AgentProposal {
                expected,
                picks,
                content,
            }),
        };
        self.open_for_agent(request);
        Ok(())
    }

    /// Whether the editor (or the request for it) is still the proposal's
    /// dialog.
    pub(super) fn conflict_proposal_open(&self) -> bool {
        self.merge.is_some() || self.merge_flow.agent_pending() || self.merge_flow.loading()
    }
}
