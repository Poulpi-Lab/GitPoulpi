//! The Kanban board's entry from the ISSUES section (slice L2): the header
//! button, the palette command and the effect that opens the view. The view
//! itself is `board_view`; this file only says what it opens on.

use super::*;
use crate::board_view::BoardTarget;

impl HostingUi {
    /// What the board opens on: the linked ISSUES source, the provider
    /// profiles (the token source reads them all) and the repository's saved
    /// status labels. `None` while the section is unlinked.
    pub(crate) fn board_target(&self) -> Option<BoardTarget> {
        #[cfg(feature = "board")]
        {
            let seed = self.issue_seed()?;
            let query = self
                .issues_source
                .as_ref()
                .and_then(|s| s.filter.clone())
                .filter(|f| !f.trim().is_empty());
            let state_path = crate::remotes_ui::repo_state_path(&self.repo_root);
            let labels = gitpoulpi_session::RepoState::load(&state_path)
                .map(|s| s.board_labels)
                .unwrap_or_default();
            Some(BoardTarget {
                seed,
                all: (self.integrations)(),
                query,
                repo_root: self.repo_root.clone(),
                labels,
            })
        }
        #[cfg(not(feature = "board"))]
        {
            None
        }
    }

    /// The effect of "open the board" (the header button, the palette).
    pub(crate) fn board_effect(&self) -> HostingEffect {
        match self.board_target() {
            Some(target) => HostingEffect::Board(Box::new(target)),
            None => HostingEffect::Message(msg::hosting_ui::ISSUES_PICK_BOTH.get().into()),
        }
    }
}
