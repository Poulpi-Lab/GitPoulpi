use super::*;

/// Fast-forward verdicts from the loaded graph: 1 <- 2 <- 3 on main,
/// 1 <- 4 on a side branch. A walk leaving the loaded graph is
/// `Unknown`, never a guess.
#[test]
fn fast_forward_verdicts_come_from_the_loaded_graph() {
    let graph = gitpoulpi_core::domain::CommitGraph {
        nodes: vec![
            commit(3, &[2]),
            commit(4, &[1]),
            commit(2, &[1]),
            commit(1, &[]),
        ],
    };
    assert_eq!(ff_verdict(&graph, oid(1), oid(3)), FfVerdict::Possible);
    assert_eq!(ff_verdict(&graph, oid(3), oid(1)), FfVerdict::Ahead);
    assert_eq!(ff_verdict(&graph, oid(3), oid(3)), FfVerdict::AlreadyThere);
    assert_eq!(ff_verdict(&graph, oid(4), oid(3)), FfVerdict::Diverged);
    let partial = gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(3, &[2]), commit(4, &[9])],
    };
    assert_eq!(ff_verdict(&partial, oid(4), oid(3)), FfVerdict::Unknown);
}

/// Both Fast-forward rows on another local branch, the one that is
/// not a fast-forward grayed out with its reason; the remote row
/// offers only `<current> to <remote>`; nothing on a detached HEAD.
#[test]
fn fast_forward_rows_are_offered_or_grayed_with_their_reason() {
    let ctx = MenuContext {
        current_branch: Some("main".into()),
        fast_forward: FastForwardCheck {
            this_to_head: FfVerdict::Possible,
            head_to_this: FfVerdict::Ahead,
        },
        pr_reason: None,
        selected_count: 0,
        ai_enabled: false,
        local_twin: None,
        remote_twin: None,
        merge_ff: Default::default(),
    };
    let local = ref_menu_entries(SidebarSection::Local, "feature", &ctx, false, false, false);
    assert_eq!(
        row_state(&local, "Fast-forward feature to main"),
        Some(Ok(RefContextAction::FastForwardToCurrent))
    );
    let grayed = row_state(&local, "Fast-forward main to feature")
        .unwrap()
        .unwrap_err();
    assert!(grayed.contains("main is already ahead"), "{grayed}");

    let remote = ref_menu_entries(
        SidebarSection::Remote,
        "origin/main",
        &MenuContext {
            current_branch: Some("main".into()),
            ..Default::default()
        },
        false,
        false,
        false,
    );
    assert_eq!(
        row_state(&remote, "Fast-forward main to origin/main"),
        Some(Ok(RefContextAction::FastForwardCurrentToThis)),
        "an unknown verdict leaves the row live for the backend to judge"
    );
    assert!(row_state(&remote, "Fast-forward origin/main").is_none());

    let detached = ref_menu_entries(
        SidebarSection::Local,
        "feature",
        &MenuContext::default(),
        false,
        false,
        false,
    );
    assert!(row_state(&detached, "Fast-forward").is_none());
    // The checked-out branch fast-forwards from the other rows' menus.
    let own = ref_menu_entries(SidebarSection::Local, "main", &ctx, false, false, false);
    assert!(row_state(&own, "Fast-forward").is_none());
}
