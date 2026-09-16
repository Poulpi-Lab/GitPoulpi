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
