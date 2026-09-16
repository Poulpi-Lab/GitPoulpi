use super::*;

#[test]
fn undoing_a_stash_drop_stores_the_commit_and_redo_drops_it_again() {
    let mut j = stash_drop_journal();
    let mut f = facts(&[], main());
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert_eq!(
        plan.steps,
        vec![Step::StashStore {
            commit: oid(7),
            message: "On main: wip".into()
        }]
    );
    // The stash is on the list already (stored by hand): refused.
    f.stashes = vec![oid(7)];
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::StashChanged { commit: oid(7) }
    );
    f.stashes = vec![];
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    j.commit(&plan);
    // Redo needs it on the list, and drops it by commit.
    assert_eq!(
        j.plan(Direction::Redo, &f, None).unwrap_err(),
        UndoRefusal::StashChanged { commit: oid(7) }
    );
    f.stashes = vec![oid(1), oid(7)];
    let plan = j.plan(Direction::Redo, &f, None).unwrap();
    assert_eq!(
        plan.steps,
        vec![Step::StashDrop {
            commit: oid(7),
            message: "On main: wip".into()
        }]
    );
    // Each step's inverse is the other one (rollback).
    assert_eq!(
        plan.steps[0].inverse(),
        Step::StashStore {
            commit: oid(7),
            message: "On main: wip".into()
        }
    );
}

#[test]
fn a_stash_push_undo_drops_the_stash_the_redo_stores_it() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::StashPush,
        "Stash".into(),
        0,
        None,
        false,
        vec![],
    );
    j.entry_mut(id).unwrap().stash = Some(StashChange {
        added: vec![StashRecord {
            commit: oid(9),
            message: "wip".into(),
        }],
        removed: vec![],
    });
    j.finish(id, None, vec![], vec![], 10);
    let mut f = facts(&[], main());
    f.stashes = vec![oid(9)];
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert!(matches!(plan.steps[..], [Step::StashDrop { .. }]));
    j.commit(&plan);
    f.stashes = vec![];
    let plan = j.plan(Direction::Redo, &f, None).unwrap();
    assert!(matches!(plan.steps[..], [Step::StashStore { .. }]));
}
