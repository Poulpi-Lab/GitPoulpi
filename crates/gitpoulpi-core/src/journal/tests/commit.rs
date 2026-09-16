use super::*;

#[test]
fn undo_of_a_commit_moves_the_branch_back_and_pins_the_new_tip() {
    let j = after_commit();
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], main()),
            None,
        )
        .unwrap();
    assert_eq!(plan.new_pins.len(), 1, "the commit the undo un-references");
    let Step::UpdateRefs(updates) = &plan.steps[0] else {
        panic!("{:?}", plan.steps)
    };
    assert!(updates.contains(&RefUpdate::Update {
        name: "refs/heads/main".into(),
        old: oid(2),
        new: oid(1)
    }));
    assert!(updates
        .iter()
        .any(|u| matches!(u, RefUpdate::Create { name, new } if name.starts_with(UNDO_PIN_PREFIX) && *new == oid(2))));
    assert_eq!(plan.describe().len(), 1);
}

#[test]
fn an_undone_entry_is_redoable_and_redo_goes_forward_to_the_same_oid() {
    let mut j = after_commit();
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], main()),
            None,
        )
        .unwrap();
    j.commit(&plan);
    assert!(j.undo.is_empty());
    assert_eq!(j.redo.len(), 1);
    assert_eq!(j.redo[0].pins.len(), 1);
    let redo = j
        .plan(
            Direction::Redo,
            &facts(&[("refs/heads/main", 1)], main()),
            None,
        )
        .unwrap();
    // The value redo un-references (the parent) gets pinned too.
    assert_eq!(redo.new_pins.len(), 1);
    let Step::UpdateRefs(updates) = &redo.steps[0] else {
        panic!()
    };
    assert!(updates.contains(&RefUpdate::Update {
        name: "refs/heads/main".into(),
        old: oid(1),
        new: oid(2)
    }));
    j.commit(&redo);
    assert_eq!(j.undo[0].pins.len(), 2);
    assert_eq!(j.undo.len(), 1);
    assert!(j.redo.is_empty());
}

#[test]
fn nothing_to_undo_or_redo() {
    let j = Journal::default();
    let f = facts(&[], main());
    assert_eq!(
        j.plan(Direction::Undo, &f, None),
        Err(UndoRefusal::NothingToUndo)
    );
    assert_eq!(
        j.plan(Direction::Redo, &f, None),
        Err(UndoRefusal::NothingToRedo)
    );
}

#[test]
fn a_detached_head_commit_moves_head_with_a_set_head_update() {
    let mut j = Journal::default();
    let d = |n| HeadState::Detached(oid(n));
    let id = j.begin(
        OperationKind::Commit,
        "c".into(),
        1,
        Some(d(1)),
        true,
        vec![],
    );
    j.finish(id, Some(d(2)), vec![], vec![], 50);
    let plan = j.plan(Direction::Undo, &facts(&[], d(2)), None).unwrap();
    let Step::UpdateRefs(updates) = &plan.steps[0] else {
        panic!()
    };
    assert!(updates.contains(&RefUpdate::SetHead {
        old: d(2),
        new: d(1)
    }));
    assert!(plan.new_pins.iter().any(|p| p.oid == oid(2)));
}
