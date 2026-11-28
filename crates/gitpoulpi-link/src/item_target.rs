//! What the conversation view of an issue or a pull / merge request opens
//! on: the item and where it lives. A frontend builds one from a list row (or
//! from a [`crate::review_model::ReviewTarget`]) and hands it to its own view.

use std::path::PathBuf;

use gitpoulpi_config::Integration;
use gitpoulpi_hosting::conversation::{ItemKey, ItemKind};
use gitpoulpi_hosting::refs::RefContext;

/// What the view opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTarget {
    pub integration: Integration,
    /// Every profile (the token source reads them all).
    pub all: Vec<Integration>,
    pub project: String,
    pub key: ItemKey,
    /// What a human types for it (`#142`, `ABC-12`): the title until the
    /// header arrives.
    pub reference: String,
    pub title: String,
    pub web_url: String,
    /// The repository the work tree belongs to; `None`: no clone (a commit
    /// link then opens on the host).
    pub repo_root: Option<PathBuf>,
    /// The repository's linked ISSUES and PULLS sources: what `#N` means.
    pub bases: RefContext,
}

impl ItemTarget {
    /// The key the "seen" state of this item is stored under.
    pub fn seen_key(&self) -> String {
        gitpoulpi_session::conversation_key(
            &self.integration.id,
            &self.project,
            match self.key.kind {
                ItemKind::Issue => "issue",
                ItemKind::PullRequest => "request",
            },
            self.key.number,
        )
    }

    /// Whether a change the host announced is about this item.
    pub fn is(&self, change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        use gitpoulpi_ipc::HostingItemKind as K;
        change.integration_id == self.integration.id
            && change.project == self.project
            && change.number == self.key.number
            && matches!(
                (change.kind, self.key.kind),
                (K::Issue, ItemKind::Issue) | (K::PullRequest, ItemKind::PullRequest)
            )
            && (change.id.is_empty() || self.key.id.is_empty() || change.id == self.key.id)
    }
}
