//! The Refs sidebar sections.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

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
pub(crate) enum SidebarSection {
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
