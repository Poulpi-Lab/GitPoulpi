//! The conversation's entries from the sidebar: Enter or a double click on
//! an issue row, the row menu's "Open conversation", Enter on a request of a
//! host with no review view, and the palette's `Open Issue Conversation...`.
//! The view itself is `item_view`; this file only says what it opens on.

use super::*;
use crate::item_view::ItemTarget;

/// How many issues the palette list offers (the sidebar is the place for
/// more).
const LIST_LIMIT: usize = 40;

impl HostingUi {
    /// The ISSUES list is on a provider whose issues have a conversation to
    /// read here.
    pub(crate) fn conversation_issues(&self) -> bool {
        self.snapshot
            .as_ref()
            .and_then(|s| s.issues_target.as_ref())
            .is_some_and(|t| crate::item_view::supports_issues(t.profile.kind))
    }

    /// The PULL REQUESTS list is on a host with no review view but with a
    /// conversation to read (Azure DevOps, Bitbucket).
    pub(super) fn conversation_prs(&self) -> bool {
        !self.review_prs()
            && self
                .linked_snapshot()
                .and_then(|s| s.kind)
                .is_some_and(crate::item_view::supports_requests)
    }

    /// What the conversation of issue `number` opens on, from the list on
    /// screen.
    pub(crate) fn issue_item_target(&self, number: u64) -> Option<ItemTarget> {
        #[cfg(feature = "conversation")]
        {
            let seed = self.issue_seed()?;
            let issue = self
                .snapshot
                .as_ref()?
                .issues
                .iter()
                .find(|i| i.number == number)?;
            crate::item_view::supports_issues(seed.integration.kind).then_some(())?;
            let key = crate::item_view::issue_key(seed.integration.kind, issue);
            Some(ItemTarget {
                integration: seed.integration,
                all: (self.integrations)(),
                project: seed.project,
                key,
                reference: issue.iid_label.clone(),
                title: issue.title.clone(),
                web_url: issue.web_url.clone(),
                repo_root: Some(self.repo_root.clone()),
                bases: self.token_bases(),
            })
        }
        #[cfg(not(feature = "conversation"))]
        {
            let _ = number;
            None
        }
    }

    /// What the conversation of request `number` opens on.
    pub(crate) fn request_item_target(&self, number: u64) -> Option<ItemTarget> {
        #[cfg(feature = "conversation")]
        {
            let snap = self.linked_snapshot()?;
            let pr = snap.prs.iter().find(|p| p.number == number)?;
            let profile = snap.profile_id.clone()?;
            let all = (self.integrations)();
            let integration = all.iter().find(|i| i.id == profile)?.clone();
            crate::item_view::supports_requests(integration.kind).then_some(())?;
            Some(ItemTarget {
                integration,
                all,
                project: snap.project.clone(),
                key: gitpoulpi_hosting::conversation::ItemKey::pull_request(number),
                reference: pr.iid_label.clone(),
                title: pr.title.clone(),
                web_url: pr.web_url.clone(),
                repo_root: Some(self.repo_root.clone()),
                bases: self.token_bases(),
            })
        }
        #[cfg(not(feature = "conversation"))]
        {
            let _ = number;
            None
        }
    }

    /// Whether a carried "open the conversation of #`number`" can run: the
    /// list knows the issue, or it is not coming (nothing is loading any more).
    pub(crate) fn item_ready(&self, number: u64) -> bool {
        self.issue_item_target(number).is_some() || (self.snapshot.is_some() && !self.in_flight)
    }

    /// The effect of "open the conversation of #`number`" (a menu row, Enter,
    /// the palette). A request opens its conversation only where no review
    /// view exists; the issue list keeps the browser when it cannot be read.
    pub(crate) fn item_effect(&self, number: u64, pr: bool) -> HostingEffect {
        let target = if pr {
            self.request_item_target(number)
        } else {
            self.issue_item_target(number)
        };
        match target {
            Some(target) => HostingEffect::Item(Box::new(target)),
            None => HostingEffect::Message(msg::item_view::WHY_NO_CONVERSATION.get().to_string()),
        }
    }

    /// The palette command: a list of the issues to choose from.
    pub(super) fn item_command(&mut self, anchor: (u16, u16)) -> HostingEffect {
        let entries: Vec<Entry<HostingAction>> = self
            .snapshot
            .as_ref()
            .map(|s| {
                s.issues
                    .iter()
                    .take(LIST_LIMIT)
                    .map(|i| {
                        Entry::action(
                            format!("{}  {}", i.iid_label, i.title),
                            HostingAction::Conversation {
                                number: i.number,
                                pr: false,
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.menu = Some(ContextMenu::new(
            msg::item_view::MENU_PICK_CONVERSATION.get(),
            entries,
            anchor.0,
            anchor.1,
        ));
        HostingEffect::None
    }
}
