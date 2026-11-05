//! Reveal in graph: scroll the Log to a Refs-sidebar ref's commit and flash
//! its row, without touching the commit selection or the right panel.
//!
//! One request lives in [`RevealState`]. The window ticks it
//! (`event_loop/window/reveal.rs`): a commit the lazy Log has not loaded yet is
//! paged in one page at a time, with a footer note once it takes longer than
//! [`FEEDBACK_AFTER`]. A ref the graph hides (hide-in-graph, solo) is
//! reported in the footer instead of failing silently.

use super::*;
use gitpoulpi_i18n::msg;

/// How long the revealed row stays highlighted.
pub(crate) const FLASH_FOR: Duration = Duration::from_millis(1500);

/// What to bring into view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevealTarget {
    Commit(Oid),
    /// A stash's own row, falling back to the commit it was made on.
    Stash {
        commit: Oid,
        base: Oid,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct PendingReveal {
    pub(crate) target: RevealTarget,
    pub(crate) label: String,
    pub(crate) started: Instant,
    pub(crate) pages: u32,
    pub(crate) noted: bool,
    /// Also select the commit and leave the view that asked (the File
    /// History's "Show in Log"), rather than only scroll to it.
    pub(crate) select: bool,
    /// With `select`: stay in the view that asked (the column's switch from
    /// the File History to the commit graph), selecting the commit under it.
    pub(crate) keep_view: bool,
}

/// The one reveal in progress and the row flashing after it.
#[derive(Debug, Default)]
pub(crate) struct RevealState {
    pub(crate) pending: Option<PendingReveal>,
    pub(crate) flash: Option<(usize, Instant)>,
}

/// What a sidebar row's reveal reads: refs and stashes for the commit, the
/// hidden set for the "hidden from the graph" note.
pub(crate) struct RevealSources<'a> {
    pub(crate) refs: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(crate) stashes: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(crate) hidden: &'a std::collections::HashSet<String>,
}

impl RevealState {
    /// Starts revealing the sidebar row `name` of `section`. `Some(note)`
    /// when there is nothing to reveal (shown in the footer).
    pub(crate) fn request_ref(
        &mut self,
        section: SidebarSection,
        name: &str,
        src: &RevealSources<'_>,
    ) -> Option<String> {
        let (target, label) = match section.ref_kind() {
            Some(kind) => {
                if crate::refs_tree::hidden_in_graph(
                    src.hidden,
                    name,
                    kind == crate::refs_tree::RefRowKind::Remote,
                ) {
                    return Some(msg::input::reveal_hidden(name));
                }
                let found = ref_menu::ref_by_display_name(src.refs, kind, name)?;
                (RevealTarget::Commit(found.target), name.to_string())
            }
            None if section == SidebarSection::Stashes => {
                let stash = src
                    .stashes
                    .as_ref()
                    .ok()?
                    .iter()
                    .find(|s| ref_menu::parse_stash_id(name) == Some(s.id))?;
                let label = format!("stash@{{{}}}", stash.id.0);
                match (stash.commit, stash.base) {
                    (Some(commit), Some(base)) => (RevealTarget::Stash { commit, base }, label),
                    _ => return Some(msg::input::reveal_no_commit(&label)),
                }
            }
            None => return Some(msg::input::reveal_no_commit(name)),
        };
        self.pending = Some(PendingReveal {
            target,
            label,
            started: Instant::now(),
            pages: 0,
            noted: false,
            select: false,
            keep_view: false,
        });
        None
    }

    /// Starts bringing `commit` into the Log and selecting it, paging the
    /// Log in as far as it takes.
    pub(crate) fn request_commit_select(&mut self, commit: Oid) {
        self.pending = Some(PendingReveal {
            target: RevealTarget::Commit(commit),
            label: commit.to_hex()[..8].to_string(),
            started: Instant::now(),
            pages: 0,
            noted: false,
            select: true,
            keep_view: false,
        });
    }

    /// Selects `commit` in the Log (paging it in as far as it takes) without
    /// leaving the view that asks: the commit column follows it.
    pub(crate) fn request_commit_follow(&mut self, commit: Oid) {
        self.request_commit_select(commit);
        if let Some(p) = self.pending.as_mut() {
            p.keep_view = true;
        }
    }

    /// A reveal is still looking for its row: the loop polls at the frame
    /// rate meanwhile.
    pub(crate) fn pending(&self) -> bool {
        self.pending.is_some()
    }

    /// The display row that flashes now.
    pub(crate) fn flash_row(&self) -> Option<usize> {
        self.flash
            .filter(|(_, until)| *until > Instant::now())
            .map(|(row, _)| row)
    }

    /// When the flash ends, for the idle loop's deadline.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.flash.map(|(_, until)| until)
    }

    /// Drops a finished flash.
    pub(crate) fn expire(&mut self, now: Instant) {
        if self.flash.is_some_and(|(_, until)| until <= now) {
            self.flash = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use gitpoulpi_core::domain::{Stash, StashId};

    fn stash(n: u32, commit: Option<u8>, base: Option<u8>) -> Stash {
        Stash {
            id: StashId(n),
            message: String::new(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: commit.map(oid),
            base: base.map(oid),
        }
    }

    fn sources<'a>(
        refs: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
        stashes: &'a gitpoulpi_core::Result<Vec<Stash>>,
        hidden: &'a std::collections::HashSet<String>,
    ) -> RevealSources<'a> {
        RevealSources {
            refs,
            stashes,
            hidden,
        }
    }

    #[test]
    fn a_branch_becomes_a_pending_reveal_of_its_commit() {
        let refs = Ok(vec![local_branch_at("feature", oid(7))]);
        let stashes = Ok(vec![]);
        let hidden = Default::default();
        let mut state = RevealState::default();
        let note = state.request_ref(
            SidebarSection::Local,
            "feature",
            &sources(&refs, &stashes, &hidden),
        );
        assert_eq!(note, None);
        let pending = state.pending.expect("a pending reveal");
        assert_eq!(pending.target, RevealTarget::Commit(oid(7)));
        assert_eq!(pending.label, "feature");
    }

    #[test]
    fn a_hidden_ref_is_reported_not_revealed() {
        let refs = Ok(vec![local_branch_at("feature", oid(7))]);
        let stashes = Ok(vec![]);
        let hidden = ["feature".to_string()].into_iter().collect();
        let mut state = RevealState::default();
        let note = state.request_ref(
            SidebarSection::Local,
            "feature",
            &sources(&refs, &stashes, &hidden),
        );
        assert!(note.is_some_and(|n| n.contains("feature")));
        assert!(!state.pending());
    }

    #[test]
    fn a_stash_reveals_its_own_row_and_falls_back_to_its_base() {
        let refs = Ok(vec![]);
        let stashes = Ok(vec![stash(0, Some(9), Some(3)), stash(1, None, None)]);
        let hidden = Default::default();
        let mut state = RevealState::default();
        assert_eq!(
            state.request_ref(
                SidebarSection::Stashes,
                "0",
                &sources(&refs, &stashes, &hidden)
            ),
            None
        );
        let pending = state.pending.take().unwrap();
        assert_eq!(
            pending.target,
            RevealTarget::Stash {
                commit: oid(9),
                base: oid(3)
            }
        );
        assert_eq!(pending.label, "stash@{0}");
        // A stash the backend gave no commit for has nothing to reveal.
        let note = state.request_ref(
            SidebarSection::Stashes,
            "1",
            &sources(&refs, &stashes, &hidden),
        );
        assert!(note.is_some());
        assert!(!state.pending());
    }

    #[test]
    fn the_flash_row_lasts_until_it_expires() {
        let mut state = RevealState::default();
        let now = Instant::now();
        state.flash = Some((4, now + FLASH_FOR));
        assert_eq!(state.flash_row(), Some(4));
        state.expire(now + FLASH_FOR);
        assert_eq!(state.flash_row(), None);
        assert_eq!(state.next_deadline(), None);
    }
}
