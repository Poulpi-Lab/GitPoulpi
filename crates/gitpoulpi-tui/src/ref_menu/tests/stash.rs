use super::*;

#[test]
fn stash_context_menu_offers_pop_apply_drop_and_worktrees_none() {
    let stash = ref_menu_entries(
        SidebarSection::Stashes,
        "0",
        &MenuContext::default(),
        false,
        false,
        false,
    );
    // Reveal in graph, a rule, Pop, Apply, a rule, Drop.
    assert_eq!(stash.len(), 6);
    assert!(matches!(
        stash[0],
        context_menu::Entry::Action {
            action: RefContextAction::RevealInGraph,
            ..
        }
    ));
    assert!(matches!(
        stash[2],
        context_menu::Entry::Action {
            action: RefContextAction::StashPop,
            ..
        }
    ));
    assert!(matches!(
        stash[5],
        context_menu::Entry::Action {
            action: RefContextAction::StashDrop,
            ..
        }
    ));
    // Read-only for now: no menu rather than dead entries.
    assert!(ref_menu_entries(
        SidebarSection::Worktrees,
        "wt",
        &MenuContext::default(),
        false,
        false,
        false
    )
    .is_empty());
}

#[test]
fn stash_row_identity_round_trips_through_the_context_menu() {
    assert_eq!(parse_stash_id("3").map(|id| id.0), Some(3));
    assert_eq!(parse_stash_id("not-a-stash"), None);
}

/// With the STASHES pane focused the palette's stash commands act on
/// the selected entry; anywhere else they keep meaning "the latest".
#[test]
fn target_stash_prefers_the_selection_only_when_that_pane_has_focus() {
    let stashes = Ok(vec![
        gitpoulpi_core::domain::Stash {
            id: gitpoulpi_core::domain::StashId(0),
            message: "newest".into(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: None,
            base: None,
        },
        gitpoulpi_core::domain::Stash {
            id: gitpoulpi_core::domain::StashId(1),
            message: "older".into(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: None,
            base: None,
        },
    ]);
    let mut state = ListState::default();
    state.select(Some(1));

    let picked = target_stash(&stashes, Focus::Refs, SidebarSection::Stashes, &state);
    assert_eq!(picked.unwrap().id.0, 1);

    // Focus elsewhere -> latest, regardless of the stale selection.
    let picked = target_stash(&stashes, Focus::Log, SidebarSection::Stashes, &state);
    assert_eq!(picked.unwrap().id.0, 0);
    let picked = target_stash(&stashes, Focus::Refs, SidebarSection::Local, &state);
    assert_eq!(picked.unwrap().id.0, 0);
}
