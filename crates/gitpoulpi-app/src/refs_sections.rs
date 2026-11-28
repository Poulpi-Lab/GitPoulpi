//! Which sections of the Refs pane exist and which are shown: the pure half
//! of the sidebar's section model (LOCAL, REMOTE, WORKTREES, STASHES, TAGS,
//! PULL REQUESTS, ISSUES, SUBMODULES), in their fixed order.
//!
//! The inputs are plain counts and flags (linked worktrees, stashes, whether
//! a hosting profile lists pull requests and issues, submodules) and the
//! user's hidden set, so a second frontend lists the same sections in the
//! same order. The rows *inside* a section are the frontend's to build from
//! the daemon's lists; see `gitpoulpi_app::refs_tree` for the ref sections.

use std::collections::HashSet;

use crate::palette::vocab::refs_sidebar::SidebarSection;

/// The sections to render this frame, in order. Worktrees and stashes are
/// elided entirely when empty.
///
/// `linked_worktree_count` excludes the main worktree on purpose: every repo
/// has one, so keying on the raw count would show a WORKTREES pane holding a
/// single "main" row in *every* repo. The pane appears once you actually use
/// worktrees, and then lists the main one too, for context.
pub fn visible_sections(linked_worktree_count: usize, stash_count: usize) -> Vec<SidebarSection> {
    visible_sections_with(
        linked_worktree_count,
        stash_count,
        (false, false),
        0,
        &HashSet::new(),
    )
}

/// [`visible_sections`] plus the hosting sections: `(pull requests, issues)`
/// are shown only when a hosting profile answers for this repository and can
/// list them.
pub fn visible_sections_with(
    linked_worktree_count: usize,
    stash_count: usize,
    hosting: (bool, bool),
    submodule_count: usize,
    hidden: &HashSet<SidebarSection>,
) -> Vec<SidebarSection> {
    let available =
        available_sections(linked_worktree_count, stash_count, hosting, submodule_count);
    let marked = hidden.contains(&SidebarSection::WorktreesEmpty);
    let hidden = effective_hidden(hidden, linked_worktree_count);
    let shown: Vec<SidebarSection> = available
        .iter()
        .copied()
        .filter(|s| !hidden.contains(s))
        .collect();
    // The pane never goes blank: a config that hides everything that exists
    // here (or sections that vanished under the hidden ones) shows them all.
    if shown.is_empty() {
        // Everything is shown, except WORKTREES where it is empty and was not
        // asked for: that default is not the user's hiding.
        let default_hidden = effective_hidden(&HashSet::new(), linked_worktree_count);
        available
            .into_iter()
            .filter(|s| !default_hidden.contains(s) || marked)
            .collect()
    } else {
        shown
    }
}

/// Every section that exists in this repository right now, whatever the user
/// hid: what the show/hide menu can act on.
pub fn available_sections(
    _linked_worktree_count: usize,
    stash_count: usize,
    hosting: (bool, bool),
    submodule_count: usize,
) -> Vec<SidebarSection> {
    SidebarSection::ORDER
        .into_iter()
        .filter(|s| match s {
            // Always offered (the show/hide list and "Show Worktrees
            // section" reach it); `effective_hidden` keeps it out of a
            // repository without linked worktrees until the user shows it.
            SidebarSection::Worktrees => crate::features::WORKTREE,
            SidebarSection::Stashes => stash_count > 0,
            SidebarSection::PullRequests => hosting.0,
            SidebarSection::Issues => hosting.1,
            SidebarSection::Submodules => crate::features::SUBMODULE && submodule_count > 0,
            _ => true,
        })
        .collect()
}

/// The set the pane is painted from: the stored one, where WORKTREES counts
/// as hidden in a repository with no linked worktree unless the user showed
/// it there ([`SidebarSection::WorktreesEmpty`] in the set). The mark never
/// leaves the stored set.
pub fn effective_hidden(
    hidden: &HashSet<SidebarSection>,
    linked_worktree_count: usize,
) -> HashSet<SidebarSection> {
    let mut out = hidden.clone();
    if linked_worktree_count == 0 && !hidden.contains(&SidebarSection::WorktreesEmpty) {
        out.insert(SidebarSection::Worktrees);
    }
    out.remove(&SidebarSection::WorktreesEmpty);
    out
}

/// Flips WORKTREES in the stored set the way the user sees it: with linked
/// worktrees the section's own entry, without any the "shown while empty"
/// mark. `show` is what the user asked for.
pub fn set_worktrees_shown(
    hidden: &mut HashSet<SidebarSection>,
    linked_worktree_count: usize,
    show: bool,
) {
    if linked_worktree_count == 0 {
        if show {
            hidden.insert(SidebarSection::WorktreesEmpty);
        } else {
            hidden.remove(&SidebarSection::WorktreesEmpty);
        }
        // The plain entry would hide it the day a worktree exists.
        hidden.remove(&SidebarSection::Worktrees);
    } else if show {
        hidden.remove(&SidebarSection::Worktrees);
    } else {
        hidden.insert(SidebarSection::Worktrees);
    }
}

/// The hidden set stored in `[ui]`; unknown names are ignored.
pub fn hidden_from_names(names: &[String]) -> HashSet<SidebarSection> {
    names
        .iter()
        .filter_map(|n| SidebarSection::from_config_name(n))
        .collect()
}

/// The hidden set as names, in section order, for the config writer.
pub fn hidden_to_names(hidden: &HashSet<SidebarSection>) -> Vec<String> {
    SidebarSection::ORDER
        .into_iter()
        .chain([SidebarSection::WorktreesEmpty])
        .filter(|s| hidden.contains(s))
        .map(|s| s.config_name().to_string())
        .collect()
}

/// Whether `section` may be hidden now: another available section must stay
/// shown (the last one cannot be unchecked).
pub fn can_hide(
    section: SidebarSection,
    hidden: &HashSet<SidebarSection>,
    available: &[SidebarSection],
) -> bool {
    available
        .iter()
        .any(|s| *s != section && !hidden.contains(s))
}

/// Worktrees excluding the main one - what `visible_sections` keys on.
pub fn linked_worktree_count(worktrees: &[gitpoulpi_core::domain::Worktree]) -> usize {
    worktrees.iter().filter(|w| !w.is_main).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_worktree_and_stash_sections_are_hidden_entirely() {
        // A repo with neither shows just the three ref sections.
        assert_eq!(
            visible_sections(0, 0),
            vec![
                SidebarSection::Local,
                SidebarSection::Remote,
                SidebarSection::Tags
            ]
        );
    }

    #[cfg(all(feature = "worktree", feature = "submodule"))]
    #[test]
    fn non_empty_worktree_and_stash_sections_appear_in_a_fixed_order() {
        assert_eq!(
            visible_sections(2, 1),
            vec![
                SidebarSection::Local,
                SidebarSection::Remote,
                SidebarSection::Worktrees,
                SidebarSection::Stashes,
                SidebarSection::Tags
            ]
        );
        // Independently: stashes present, worktrees not.
        assert_eq!(
            visible_sections(0, 3),
            vec![
                SidebarSection::Local,
                SidebarSection::Remote,
                SidebarSection::Stashes,
                SidebarSection::Tags
            ]
        );
    }

    /// Every repo has a main worktree, so the pane must key on *linked* ones,
    /// or a WORKTREES pane holding a single "main" row appears in every repo.
    #[cfg(feature = "worktree")]
    #[test]
    fn a_repo_with_only_its_main_worktree_shows_no_worktrees_pane() {
        let main_only = vec![gitpoulpi_core::domain::Worktree {
            path: "/repo".into(),
            branch: Some("main".into()),
            is_main: true,
            is_locked: false,
        }];
        assert_eq!(linked_worktree_count(&main_only), 0);
        assert!(!visible_sections(linked_worktree_count(&main_only), 0)
            .contains(&SidebarSection::Worktrees));

        let with_linked = vec![
            main_only[0].clone(),
            gitpoulpi_core::domain::Worktree {
                path: "/wt".into(),
                branch: Some("feature".into()),
                is_main: false,
                is_locked: false,
            },
        ];
        assert_eq!(linked_worktree_count(&with_linked), 1);
        assert!(visible_sections(linked_worktree_count(&with_linked), 0)
            .contains(&SidebarSection::Worktrees));
    }

    /// A build without `worktree` / `submodule` never shows those panes,
    /// whatever the lists hold.
    #[test]
    fn the_worktree_and_submodule_panes_follow_their_features() {
        let sections = visible_sections_with(2, 0, (false, false), 3, &HashSet::new());
        assert_eq!(
            sections.contains(&SidebarSection::Worktrees),
            crate::features::WORKTREE
        );
        assert_eq!(
            sections.contains(&SidebarSection::Submodules),
            crate::features::SUBMODULE
        );
    }

    #[test]
    fn hidden_sections_are_not_visible_but_the_pane_never_goes_blank() {
        let hidden = HashSet::from([SidebarSection::Local, SidebarSection::Tags]);
        let shown = visible_sections_with(0, 0, (false, false), 0, &hidden);
        assert_eq!(shown, vec![SidebarSection::Remote]);
        // Everything that exists is hidden (a hand-edited config): all show.
        let all = HashSet::from([
            SidebarSection::Local,
            SidebarSection::Remote,
            SidebarSection::Tags,
        ]);
        assert_eq!(
            visible_sections_with(0, 0, (false, false), 0, &all),
            vec![
                SidebarSection::Local,
                SidebarSection::Remote,
                SidebarSection::Tags
            ]
        );
    }

    #[test]
    #[cfg(feature = "worktree")]
    fn worktrees_stay_hidden_without_linked_ones_until_shown_on_purpose() {
        let none = HashSet::new();
        assert!(!visible_sections_with(0, 0, (false, false), 0, &none)
            .contains(&SidebarSection::Worktrees));
        let mut shown = HashSet::new();
        set_worktrees_shown(&mut shown, 0, true);
        assert!(visible_sections_with(0, 0, (false, false), 0, &shown)
            .contains(&SidebarSection::Worktrees));
        // The mark survives the file: stored as `worktrees_empty`.
        assert_eq!(hidden_to_names(&shown), vec!["worktrees_empty"]);
        assert_eq!(hidden_from_names(&["worktrees_empty".to_string()]), shown);
        // With linked worktrees the plain rule applies and the mark is inert.
        assert!(visible_sections_with(1, 0, (false, false), 0, &shown)
            .contains(&SidebarSection::Worktrees));
        let mut hidden = HashSet::new();
        set_worktrees_shown(&mut hidden, 1, false);
        assert!(!visible_sections_with(1, 0, (false, false), 0, &hidden)
            .contains(&SidebarSection::Worktrees));
        // Hiding it again where it is empty drops the mark.
        set_worktrees_shown(&mut shown, 0, false);
        assert!(shown.is_empty());
    }

    #[test]
    fn the_last_shown_section_cannot_be_hidden() {
        let available = [SidebarSection::Local, SidebarSection::Remote];
        let none = HashSet::new();
        assert!(can_hide(SidebarSection::Local, &none, &available));
        let one = HashSet::from([SidebarSection::Remote]);
        assert!(!can_hide(SidebarSection::Local, &one, &available));
    }

    #[test]
    fn hidden_names_round_trip_and_unknown_names_are_ignored() {
        let hidden = hidden_from_names(&["tags".into(), "nope".into(), "issues".into()]);
        assert_eq!(
            hidden,
            HashSet::from([SidebarSection::Tags, SidebarSection::Issues])
        );
        assert_eq!(hidden_to_names(&hidden), vec!["tags", "issues"]);
    }
}
