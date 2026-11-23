use super::*;

#[test]
fn a_history_move_undoes_with_reset_keep_and_redoes_the_same_way() {
    let mut j = history_move(Effect::Files);
    let at = facts(&[("refs/heads/main", 2)], main());
    let plan = j.plan(Direction::Undo, &at, None).unwrap();
    assert_eq!(
        plan.steps.last(),
        Some(&Step::MoveHeadKeep {
            branch: "main".into(),
            from: oid(2),
            to: oid(1)
        })
    );
    j.commit(&plan);
    let back = facts(&[("refs/heads/main", 1)], main());
    let plan = j.plan(Direction::Redo, &back, None).unwrap();
    assert_eq!(
        plan.steps.last(),
        Some(&Step::MoveHeadKeep {
            branch: "main".into(),
            from: oid(1),
            to: oid(2)
        })
    );
}

#[test]
fn a_ref_effect_moves_the_ref_alone() {
    let at = facts(&[("refs/heads/main", 2)], main());
    let plan = history_move(Effect::Refs)
        .plan(Direction::Undo, &at, None)
        .unwrap();
    assert!(!plan
        .steps
        .iter()
        .any(|s| matches!(s, Step::MoveHeadKeep { .. })));
}

#[test]
fn pull_all_branches_undoes_the_checked_out_branch_with_reset_keep_and_the_others_by_ref() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::PullAllBranches,
        "Pull all branches".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    j.finish(
        id,
        Some(main()),
        vec![
            RefChange {
                name: "refs/heads/main".into(),
                old: Some(oid(1)),
                new: Some(oid(2)),
            },
            RefChange {
                name: "refs/heads/other".into(),
                old: Some(oid(3)),
                new: Some(oid(4)),
            },
        ],
        vec![],
        50,
    );
    let at = facts(&[("refs/heads/main", 2), ("refs/heads/other", 4)], main());
    let plan = j.plan(Direction::Undo, &at, None).unwrap();
    // The checked-out branch's files follow (reset --keep) ...
    assert_eq!(
        plan.steps.last(),
        Some(&Step::MoveHeadKeep {
            branch: "main".into(),
            from: oid(2),
            to: oid(1)
        })
    );
    // ... the other branch moves by ref, in one update, never the head's.
    let updates: Vec<&RefUpdate> = plan
        .steps
        .iter()
        .filter_map(|s| match s {
            Step::UpdateRefs(u) => Some(u.iter()),
            _ => None,
        })
        .flatten()
        .filter(|u| matches!(u, RefUpdate::Update { .. }))
        .collect();
    assert_eq!(updates.len(), 1);
    assert!(matches!(
        updates[0],
        RefUpdate::Update { name, old, new }
            if name == "refs/heads/other" && *old == oid(4) && *new == oid(3)
    ));
}
