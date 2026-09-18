use super::*;

#[test]
fn with_several_commits_selected_the_menu_offers_one_multi_cherry_pick() {
    let ctx = MenuContext::default().with_selected(3);
    let entries = commit_menu_entries(&ctx, false);
    // The cherry-pick, Copy as Mermaid gitGraph for the same selection
    // (when this build has it) and the Rewrite history submenu (likewise).
    assert_eq!(
        entries.len(),
        1 + usize::from(crate::build_features::MERMAID)
            + usize::from(crate::build_features::HISTORY_REWRITE)
    );
    let context_menu::Entry::Action { label, action, .. } = &entries[0] else {
        panic!("expected an action row");
    };
    assert!(label.contains("Cherry pick 3 commits"), "{label}");
    assert_eq!(*action, LogContextAction::CherryPick);
}

#[test]
fn the_cherry_pick_form_lists_the_selection_newest_first_and_refuses_a_merge() {
    let graph = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(3, &[2]), commit(2, &[1]), commit(1, &[])],
    });
    let refs = Ok(Vec::new());
    let form = cherry_pick_form_for(&graph, &refs, &[oid(3), oid(1)]).unwrap();
    assert!(form.cherry_pick);
    assert_eq!(
        form.steps().iter().map(|s| s.commit).collect::<Vec<_>>(),
        vec![oid(1), oid(3)]
    );
    let merged = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: vec![commit(3, &[2, 1]), commit(2, &[]), commit(1, &[])],
    });
    assert!(cherry_pick_form_for(&merged, &refs, &[oid(3), oid(1)]).is_err());
    assert!(cherry_pick_form_for(&graph, &refs, &[oid(9), oid(1)]).is_err());
}
