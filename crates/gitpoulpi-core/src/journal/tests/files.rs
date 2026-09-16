use super::*;

#[test]
fn a_discard_undoes_to_its_snapshot_only_while_the_files_are_as_it_left_them() {
    let j = after_discard();
    let mut f = facts(&[], main());
    f.files.insert(1, vec![file("a.txt", Some(1), Some(1))]);
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert_eq!(
        plan.steps,
        vec![Step::RestorePaths {
            paths: vec!["a.txt".into(), "b.txt".into()],
            to: oid(50),
            from: oid(60)
        }]
    );
    assert_eq!(
        plan.steps[0].inverse(),
        Step::RestorePaths {
            paths: vec!["a.txt".into(), "b.txt".into()],
            to: oid(60),
            from: oid(50)
        }
    );

    // Edited since: refused, naming the file.
    f.files.insert(1, vec![file("a.txt", Some(9), Some(1))]);
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::PathChanged {
            path: "a.txt".into()
        }
    );
    // A file appeared under the paths: refused too.
    f.files.insert(
        1,
        vec![
            file("a.txt", Some(1), Some(1)),
            file("b.txt", Some(7), None),
        ],
    );
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::PathChanged {
            path: "b.txt".into()
        }
    );
    // Nothing known about the files at all: also refused.
    f.files.clear();
    assert!(matches!(
        j.plan(Direction::Undo, &f, None),
        Err(UndoRefusal::PathChanged { .. })
    ));
}

#[test]
fn a_discard_redoes_from_the_before_state() {
    let mut j = after_discard();
    let mut f = facts(&[], main());
    f.files.insert(1, vec![file("a.txt", Some(1), Some(1))]);
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    j.commit(&plan);
    // Redo needs the files as the *undo* left them (the before state).
    f.files.insert(
        1,
        vec![
            file("a.txt", Some(2), Some(1)),
            file("b.txt", Some(3), None),
        ],
    );
    let redo = j.plan(Direction::Redo, &f, None).unwrap();
    assert!(matches!(&redo.steps[0], Step::RestorePaths { to, .. } if *to == oid(60)));
    f.files.insert(1, vec![file("a.txt", Some(1), Some(1))]);
    assert!(j.plan(Direction::Redo, &f, None).is_err());
}

#[test]
fn a_staging_entry_needs_the_index_to_be_the_tree_it_left() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::StageFile,
        "Stage a.txt".into(),
        10,
        None,
        false,
        vec![],
    );
    j.entry_mut(id).unwrap().index = Some(IndexChange {
        before: oid(10),
        after: oid(11),
        known: None,
    });
    j.finish(id, None, vec![], vec![], 50);
    let mut f = facts(&[], main());
    f.index_tree = Some(oid(11));
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert_eq!(
        plan.steps,
        vec![Step::RestoreIndex {
            to: oid(10),
            from: oid(11)
        }]
    );
    f.index_tree = Some(oid(12));
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::IndexChanged
    );
    f.index_tree = None;
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::IndexChanged
    );
}
