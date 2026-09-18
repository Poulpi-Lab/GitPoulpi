//! Tabs opened or closed from outside (`gitpoulpi repo open|close` rewrites
//! the session file, with no event to the window): picked up at the next
//! frame, so the tab bar, the marked tab and the repository on screen
//! cannot drift apart.

use crate::event_loop::*;
use crate::tabs::repo_ref_for;
use init::WindowState;

impl WindowState {
    /// Re-reads the tab list when the session file changed under us. The
    /// window keeps the repository it shows; when that repository's tab was
    /// closed it moves to the session's active tab (else the first one).
    pub(in crate::event_loop) fn sync_external_tabs(&mut self) {
        // `general.group_worktree_tabs` just turned on: group the open tabs;
        // the rewrite is then picked up by the stamp check below.
        if crate::live_settings::take_regroup_request() {
            session_store::regroup_worktree_tabs(&self.session);
        }
        // The management window and a window with no repository have no
        // tab of their own to keep in step.
        if self.management.is_some() || self.no_repository {
            return;
        }
        let stamp = session_store::session_stamp(&self.session);
        if stamp == self.session_stamp {
            return;
        }
        self.session_stamp = stamp;
        let tabs = session_store::tabs(&self.session);
        if tabs == self.repo_tabs {
            return;
        }
        self.repo_tabs = tabs;
        match self.repo_tabs.iter().position(|t| t.path == self.repo.root) {
            Some(index) => self.active_tab = index,
            None => {
                self.active_tab = session_store::active_tab_index(&self.session)
                    .filter(|i| *i < self.repo_tabs.len())
                    .unwrap_or(0);
                if let Some(next) = self.repo_tabs.get(self.active_tab) {
                    self.switch_to = Some(repo_ref_for(next.path.clone()));
                }
            }
        }
    }
}
