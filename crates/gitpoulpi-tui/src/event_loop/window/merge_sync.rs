//! The merge editor's replies and the refreshes that touch its file,
//! applied once per iteration (`merge_flow.rs` holds the logic).

use crate::event_loop::*;
use crate::merge_flow::Env;
use init::WindowState;

impl WindowState {
    /// The conflicts or the operation changed: the open editor may have to
    /// say its file moved, and a finished operation clears what the flow
    /// kept.
    pub(in crate::event_loop) fn after_conflicts_changed(&mut self) {
        let mut env = Env {
            link: &self.link,
            conflicts: &self.conflicts,
            merge: &mut self.merge,
            root: &self.repo.root,
            last_message: &mut self.last_message,
        };
        self.merge_flow.on_conflicts(&mut env);
        if self.operation_state.is_none() && self.conflicts.is_empty() {
            self.merge_flow.operation_ended();
        }
    }

    /// Conflicts appeared with no merge, rebase, cherry-pick or revert in
    /// progress - a stash pop (an auto-stash's included). No operation
    /// banner says so, and the right pane shows the selected commit: select
    /// the WIP row so the conflicted files are on screen, and say why.
    pub(in crate::event_loop) fn reveal_stray_conflicts(&mut self) {
        if !has_wip_row(&self.status_result) {
            return;
        }
        self.central_view = CentralView::Log;
        self.log_state.select(Some(0));
        self.log_follow = true;
        self.right_panel = RightPanel::Status;
        self.right_pane_collapsed = false;
        self.active_status_section = StatusSection::Unstaged;
        self.last_message = Some(
            gitpoulpi_i18n::msg::autostash::STRAY_CONFLICTS
                .get()
                .to_string(),
        );
    }

    /// Answers of the daemon to what the editor asked.
    pub(in crate::event_loop) fn apply_merge_replies(&mut self) {
        while let Ok(reply) = self.link.merge_rx.try_recv() {
            let mut env = Env {
                link: &self.link,
                conflicts: &self.conflicts,
                merge: &mut self.merge,
                root: &self.repo.root,
                last_message: &mut self.last_message,
            };
            self.merge_flow.on_reply(&mut env, reply);
        }
        let mut env = Env {
            link: &self.link,
            conflicts: &self.conflicts,
            merge: &mut self.merge,
            root: &self.repo.root,
            last_message: &mut self.last_message,
        };
        self.merge_flow.poll_built(&mut env);
    }
}

impl WindowState {
    /// ✦ The AI's suggestion for conflict `conflict` of `path` arrived: it
    /// goes into that conflict's Output region as one undoable edit (marked
    /// edited), never saved. Dropped when the editor is on another file.
    pub(in crate::event_loop) fn put_conflict_suggestion(
        &mut self,
        path: &str,
        conflict: usize,
        lines: &[String],
    ) {
        use gitpoulpi_i18n::msg::merge as tr;
        let line = match self.merge.as_mut() {
            Some(view) if view.path().to_string_lossy().replace('\\', "/") == path => {
                if view.put_suggestion(conflict, lines) {
                    tr::msg_ai_done(conflict + 1)
                } else {
                    tr::MSG_AI_DROPPED.get().to_string()
                }
            }
            _ => tr::MSG_AI_DROPPED.get().to_string(),
        };
        if let Some(view) = self.merge.as_mut() {
            view.note = Some(line.clone());
        }
        self.last_message = Some(line);
    }
}
