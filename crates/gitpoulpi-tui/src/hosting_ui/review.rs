//! "Review": the entry points of the in-app code review that belong to the
//! sidebar's PULL REQUESTS list - the row menu entry, Enter on a row, and
//! the palette's `Review Pull Request...` (which lists the requests to
//! choose from, the same menu shape as `Open Issue...`). GitHub and GitLab
//! only (`review_view::supports`).

use super::*;
use crate::review_view::{self, ReviewTarget};

/// How many requests the palette list offers (the sidebar is the place for
/// more).
const LIST_LIMIT: usize = 40;

impl HostingUi {
    /// The pull request list is on a provider the review view supports.
    pub(super) fn review_prs(&self) -> bool {
        self.linked_snapshot()
            .and_then(|s| s.kind)
            .is_some_and(review_view::supports)
    }

    /// What the review of request `number` opens on, from the list on
    /// screen.
    pub(crate) fn review_target(&self, number: u64) -> Option<ReviewTarget> {
        #[cfg(feature = "review")]
        {
            let snap = self.linked_snapshot()?;
            let pr = snap.prs.iter().find(|p| p.number == number)?;
            let profile = snap.profile_id.clone()?;
            let all = (self.integrations)();
            let integration = all.iter().find(|i| i.id == profile)?.clone();
            review_view::supports(integration.kind).then_some(())?;
            let remote = match snap.remote.as_str() {
                "" => "origin".to_string(),
                r => r.to_string(),
            };
            Some(ReviewTarget {
                integration,
                all,
                project: snap.project.clone(),
                number,
                iid_label: pr.iid_label.clone(),
                title: pr.title.clone(),
                remote,
                web_url: pr.web_url.clone(),
                source_branch: pr.source_branch.clone(),
                target_branch: pr.target_branch.clone(),
                patch_set: None,
                compare_patch_set: None,
                repo_root: Some(self.repo_root.clone()),
            })
        }
        #[cfg(not(feature = "review"))]
        {
            let _ = number;
            None
        }
    }

    /// Whether a carried "review request `number`" can run: the list knows
    /// the request, or it is not coming (nothing is loading any more).
    pub(crate) fn review_ready(&self, number: u64) -> bool {
        self.review_target(number).is_some() || (self.snapshot.is_some() && !self.in_flight)
    }

    /// The effect of "review request `number`" (a menu row, Enter, the
    /// Radar's carried action).
    pub(crate) fn review_effect(&self, number: u64) -> HostingEffect {
        match self.review_target(number) {
            Some(target) => HostingEffect::Review(Box::new(target)),
            None => HostingEffect::Message(msg::hosting_ui::WHY_NO_REVIEW.get().to_string()),
        }
    }

    /// The palette command: a list of the requests to choose from.
    pub(super) fn review_command(&mut self, anchor: (u16, u16)) -> HostingEffect {
        let entries: Vec<Entry<HostingAction>> = self
            .linked_snapshot()
            .map(|snap| {
                snap.prs
                    .iter()
                    .take(LIST_LIMIT)
                    .map(|pr| {
                        Entry::action(
                            format!("{}  {}", pr.iid_label, pr.title),
                            HostingAction::Review { number: pr.number },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.menu = Some(ContextMenu::new(
            msg::hosting_ui::MENU_PICK_REVIEW.get(),
            entries,
            anchor.0,
            anchor.1,
        ));
        HostingEffect::None
    }
}
