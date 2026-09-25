//! What the Radar asked of this repository's window: it ended the Home
//! window to open this one and handed the action over in
//! [`crate::radar::pending`]. Run once the refs are in (a checkout needs
//! to know the branches) and the window shows a repository.
//!
//! Every case reuses what the sidebar already does: the checkout effect, the
//! Create worktree dialog, the PULL REQUESTS section's own provider choice,
//! the New branch prompt. Nothing here is a second implementation.

use crate::event_loop::*;
use crate::hosting_ui::{HostingEffect, HostingList, RevealOutcome};
use crate::radar::pending::{self, Pending};
use gitpoulpi_core::domain::{RefKind, WorktreeStart};
use gitpoulpi_i18n::msg;
use init::WindowState;

impl WindowState {
    /// Per frame: runs the carried action once the window is ready for it.
    pub(in crate::event_loop) fn radar_pending_tick(&mut self) {
        if self.no_repository || self.still_loading || self.refs_result.is_err() {
            return;
        }
        if !pending::waiting() {
            return;
        }
        // A review sent from the workspace search names its repository: the
        // window of another one leaves it for the right window.
        if let Some(Pending::Review {
            repo: Some(repo), ..
        }) = pending::peek()
        {
            if repo != self.repo.root {
                return;
            }
        }
        // A carried review waits for the request list to arrive.
        #[cfg(feature = "hosting")]
        if let Some(Pending::Review { number, .. }) = pending::peek() {
            if !self.hosting.review_ready(number) {
                return;
            }
        }
        let Some(action) = pending::take() else {
            return;
        };
        self.run_radar_pending(action);
    }

    fn run_radar_pending(&mut self, action: Pending) {
        match action {
            Pending::CheckoutPr { number } => {
                let refs = self.refs_result.as_deref().unwrap_or(&[]);
                let effect = self
                    .hosting
                    .checkout_pr_effect(number, refs, &mut self.clipboard);
                commands::apply_hosting_effect(&mut self.command_ctx(), effect);
            }
            Pending::StartWork { branch } => {
                commands::apply_hosting_effect(
                    &mut self.command_ctx(),
                    HostingEffect::NewBranch(branch),
                );
            }
            Pending::Link => {
                commands::apply_hosting_effect(
                    &mut self.command_ctx(),
                    HostingEffect::Focus(HostingList::Pulls),
                );
                self.last_message = Some(msg::radar::MSG_LINK.get().to_string());
            }
            Pending::Show { pr, number } => {
                let list = if pr {
                    HostingList::Pulls
                } else {
                    HostingList::Issues
                };
                match self.hosting.reveal(list, number) {
                    RevealOutcome::Row(row) => self.reveal_sidebar_row(list.section(), row),
                    RevealOutcome::NotLinked | RevealOutcome::NotListed => {
                        commands::apply_hosting_effect(
                            &mut self.command_ctx(),
                            HostingEffect::Focus(list),
                        );
                        self.last_message = Some(msg::radar::msg_not_listed(format!("#{number}")));
                    }
                }
            }
            Pending::Worktree { branch } => self.radar_worktree(&branch),
            Pending::EditMeta { pr, number, act } => self.radar_edit_meta(pr, number, act),
            Pending::Review { number, .. } => self.radar_review(number),
        }
    }

    /// The metadata picker for a request or issue the Radar named, once
    /// this window's lists know it.
    #[cfg(feature = "hosting")]
    fn radar_edit_meta(&mut self, pr: bool, number: u64, act: crate::radar::Act) {
        use crate::meta_picker::MetaField;
        use crate::radar::Act;
        let field = match act {
            Act::EditReviewers => MetaField::Reviewers,
            Act::EditLabels => MetaField::Labels,
            Act::SetMilestone => MetaField::Milestone,
            _ => MetaField::Assignees,
        };
        let effect = self.hosting.meta_edit_effect(field, pr, number);
        commands::apply_hosting_effect(&mut self.command_ctx(), effect);
    }

    #[cfg(not(feature = "hosting"))]
    fn radar_edit_meta(&mut self, _pr: bool, _number: u64, _act: crate::radar::Act) {}

    /// The in-app review of a request the Radar named, once this
    /// window's lists know it.
    #[cfg(feature = "hosting")]
    fn radar_review(&mut self, number: u64) {
        let effect = self.hosting.review_effect(number);
        commands::apply_hosting_effect(&mut self.command_ctx(), effect);
    }

    #[cfg(not(feature = "hosting"))]
    fn radar_review(&mut self, _number: u64) {}

    /// The Create worktree dialog for a pull request's branch: the local
    /// branch when there is one, else a new local branch started at the
    /// remote one.
    fn radar_worktree(&mut self, branch: &str) {
        let refs = self.refs_result.as_deref().unwrap_or(&[]);
        let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]);
        let local = refs
            .iter()
            .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch));
        let remote = refs.iter().find_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name } if name == branch => {
                Some((remote.clone(), r.target))
            }
            _ => None,
        });
        self.worktree_seed = match (local, remote) {
            (true, _) => Some(WorktreeSeed {
                start: WorktreeStart::Branch(branch.to_string()),
                label: branch.to_string(),
                new_branch: String::new(),
                branch_busy: worktrees
                    .iter()
                    .any(|w| w.branch.as_deref() == Some(branch))
                    || refs.iter().any(|r| {
                        r.is_head
                            && matches!(&r.kind, RefKind::LocalBranch { name } if name == branch)
                    }),
            }),
            (false, Some((remote, target))) => Some(WorktreeSeed {
                start: WorktreeStart::Commit(target),
                label: format!("{remote}/{branch}"),
                new_branch: branch.to_string(),
                branch_busy: false,
            }),
            (false, None) => {
                self.last_message = Some(msg::input::branch_not_here(
                    branch,
                    keymap::paren(&keymap::current(), Bind::GitFetchAll),
                ));
                None
            }
        };
    }
}
