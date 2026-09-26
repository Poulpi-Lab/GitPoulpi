//! "Push for review": a Gerrit change is created by pushing `HEAD` to
//! `refs/for/<branch>`, never by an API call. With the PULL REQUESTS
//! section linked to a Gerrit project, the section's `+` button, the palette
//! rows *Hosting: Create Pull Request...* and *Hosting: Push for Review...*
//! all land here: a menu of the branches a change can target (the ones the
//! listed changes already target, else `main` / `master`), and the pick
//! starts the push as an ordinary action (spinner, activity log, failure
//! card). The push is not undoable from the app: abandon the change in
//! Gerrit.

use super::*;

impl HostingUi {
    /// The PULL REQUESTS section is linked to a Gerrit project.
    pub(super) fn gerrit_pulls(&self) -> bool {
        self.linked_snapshot()
            .and_then(|s| s.kind)
            .is_some_and(|k| k.family() == gitpoulpi_config::ApiFamily::Gerrit)
    }

    /// The branches a change can target, most used first.
    fn review_branches(&self) -> Vec<String> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        if let Some(snap) = self.linked_snapshot() {
            for pr in &snap.prs {
                match counts.iter_mut().find(|(b, _)| *b == pr.target_branch) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((pr.target_branch.clone(), 1)),
                }
            }
        }
        counts.sort_by_key(|c| std::cmp::Reverse(c.1));
        let mut out: Vec<String> = counts.into_iter().map(|(b, _)| b).collect();
        if out.is_empty() {
            out = vec!["main".to_string(), "master".to_string()];
        }
        out
    }

    /// The menu of target branches.
    pub(super) fn push_review_menu(&mut self, anchor: (u16, u16)) -> HostingEffect {
        let entries: Vec<Entry<HostingAction>> = self
            .review_branches()
            .into_iter()
            .map(|branch| Entry::action(branch.clone(), HostingAction::PushForReview { branch }))
            .collect();
        self.menu = Some(ContextMenu::new(
            msg::hosting_ui::MENU_PUSH_REVIEW.get(),
            entries,
            anchor.0,
            anchor.1,
        ));
        HostingEffect::None
    }

    /// The pick: push to that branch's review ref on the project's remote.
    pub(super) fn push_review_effect(&self, branch: String) -> HostingEffect {
        let Some(snap) = self.linked_snapshot() else {
            return HostingEffect::Message(msg::hosting_ui::WHY_NO_PUSH_REVIEW.get().to_string());
        };
        let remote = match snap.remote.as_str() {
            "" => "origin".to_string(),
            r => r.to_string(),
        };
        HostingEffect::PushForReview { remote, branch }
    }
}
