//! The Refs sidebar sections.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use crate::refs_tree::RefRowKind;

/// A sub-pane of the left sidebar, in display order.
///
/// Deliberately *not* folded into `RefRowKind`: that enum also names "what
/// color/icon a Log-row pill gets", and stashes/worktrees are not refs and
/// must never grow a pill. The two overlap only for the three ref-backed
/// sections, which `ref_kind()` bridges.
///
/// `Worktrees` and `Stashes` are only *shown* when non-empty (see
/// `visible_sections`), so navigation and hit-testing must work off the
/// visible list rather than this full one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarSection {
    Local,
    Remote,
    Worktrees,
    Stashes,
    Tags,
    /// Pull/merge requests of the repository's hosting profile
    /// (`hosting_ui`). Shown only when a profile answers for the repo.
    PullRequests,
    /// Issues of the same project.
    Issues,
    /// The repository's submodules, with their state. Shown only when it
    /// has some. Last in [`SidebarSection::ORDER`] so the persisted
    /// section heights (keyed by position) of the sections before it keep
    /// their meaning.
    Submodules,
    /// Not a pane: the mark "show WORKTREES although the repository has no
    /// linked worktree yet" in the hidden set (persisted as
    /// `worktrees_empty`). Absent from [`SidebarSection::ORDER`]. Without
    /// the mark the section stays hidden in such a repository, as before.
    WorktreesEmpty,
}

impl SidebarSection {
    pub const ORDER: [SidebarSection; 8] = [
        SidebarSection::Local,
        SidebarSection::Remote,
        SidebarSection::Worktrees,
        SidebarSection::Stashes,
        SidebarSection::Tags,
        SidebarSection::PullRequests,
        SidebarSection::Issues,
        SidebarSection::Submodules,
    ];
    /// Position in [`Self::ORDER`] - the key of the persisted section
    /// heights (`PaneLayout::sidebar_heights`).
    pub fn index(self) -> usize {
        Self::ORDER
            .iter()
            .position(|s| *s == self)
            .expect("every section is in ORDER")
    }
    /// The stable English name: the key of the persisted fold state, never shown.
    // i18n-scan: off
    pub fn title(self) -> &'static str {
        match self {
            SidebarSection::Local => "LOCAL",
            SidebarSection::Remote => "REMOTE",
            SidebarSection::Worktrees => "WORKTREES",
            SidebarSection::Stashes => "STASHES",
            SidebarSection::Tags => "TAGS",
            SidebarSection::PullRequests => "PULL REQUESTS",
            SidebarSection::Issues => "ISSUES",
            SidebarSection::Submodules => "SUBMODULES",
            SidebarSection::WorktreesEmpty => "WORKTREES (EMPTY)",
        }
    }
    // i18n-scan: on
    /// The section header's text in the current language (the stable key is
    /// [`Self::title`]).
    pub fn display_title(self) -> &'static str {
        use gitpoulpi_i18n::msg::sidebar as m;
        match self {
            SidebarSection::Local => m::SECTION_LOCAL,
            SidebarSection::Remote => m::SECTION_REMOTE,
            SidebarSection::Worktrees => m::SECTION_WORKTREES,
            SidebarSection::Stashes => m::SECTION_STASHES,
            SidebarSection::Tags => m::SECTION_TAGS,
            SidebarSection::PullRequests => m::SECTION_PULL_REQUESTS,
            SidebarSection::Issues => m::SECTION_ISSUES,
            SidebarSection::Submodules => m::SECTION_SUBMODULES,
            SidebarSection::WorktreesEmpty => m::SECTION_WORKTREES,
        }
        .get()
    }
    /// The inverse of `ref_kind`: which sidebar section a ref of this kind lives
    /// in. A ref chip in the Log carries a `RefRowKind` while the menu is keyed
    /// by section, and the two must not be mapped by hand at each call site.
    pub fn from_ref_kind(kind: RefRowKind) -> Self {
        match kind {
            RefRowKind::Local => SidebarSection::Local,
            RefRowKind::Remote => SidebarSection::Remote,
            RefRowKind::Tag => SidebarSection::Tags,
        }
    }
    /// The ref kind this section lists, or `None` for the sections that don't
    /// hold refs at all (worktrees, stashes).
    pub fn ref_kind(self) -> Option<RefRowKind> {
        match self {
            SidebarSection::Local => Some(RefRowKind::Local),
            SidebarSection::Remote => Some(RefRowKind::Remote),
            SidebarSection::Tags => Some(RefRowKind::Tag),
            SidebarSection::Worktrees
            | SidebarSection::Stashes
            | SidebarSection::PullRequests
            | SidebarSection::Issues
            | SidebarSection::Submodules
            | SidebarSection::WorktreesEmpty => None,
        }
    }
    /// The stable lowercase name persisted in `ui.sidebar_sections_hidden`.
    // i18n-scan: off
    pub fn config_name(self) -> &'static str {
        match self {
            SidebarSection::Local => "local",
            SidebarSection::Remote => "remote",
            SidebarSection::Worktrees => "worktrees",
            SidebarSection::Stashes => "stashes",
            SidebarSection::Tags => "tags",
            SidebarSection::PullRequests => "pull_requests",
            SidebarSection::Issues => "issues",
            SidebarSection::Submodules => "submodules",
            SidebarSection::WorktreesEmpty => "worktrees_empty",
        }
    }
    // i18n-scan: on
    /// Whether this build has the feature the section needs.
    pub fn compiled_in(self) -> bool {
        match self {
            SidebarSection::Worktrees => crate::features::WORKTREE,
            SidebarSection::Submodules => crate::features::SUBMODULE,
            SidebarSection::PullRequests | SidebarSection::Issues => crate::features::HOSTING,
            _ => true,
        }
    }
    pub fn from_config_name(name: &str) -> Option<Self> {
        Self::ORDER
            .into_iter()
            .chain([SidebarSection::WorktreesEmpty])
            .find(|s| s.config_name() == name)
    }
}
