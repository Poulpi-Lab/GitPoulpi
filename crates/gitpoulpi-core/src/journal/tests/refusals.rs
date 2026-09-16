use super::*;

#[test]
fn a_moved_ref_refuses_and_names_it() {
    let j = after_commit();
    let err = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 9)], main()),
            None,
        )
        .unwrap_err();
    assert_eq!(
        err,
        UndoRefusal::RefMoved {
            name: "refs/heads/main".into(),
            expected: Some(oid(2)),
            actual: Some(oid(9)),
            restore: None
        }
    );
    assert!(err.message().contains("main is at 0909090"));
    assert!(err.message().contains("outside GitPoulpi"));
}

#[test]
fn a_moved_head_refuses_only_when_the_entry_requires_it() {
    let j = after_commit();
    let elsewhere = HeadState::Branch("other".into());
    let err = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], elsewhere.clone()),
            None,
        )
        .unwrap_err();
    assert!(matches!(err, UndoRefusal::HeadMoved { .. }));

    // A branch create does not care where HEAD is.
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::BranchCreate,
        "b".into(),
        1,
        Some(main()),
        false,
        vec![],
    );
    j.finish(
        id,
        Some(main()),
        vec![RefChange {
            name: "refs/heads/b".into(),
            old: None,
            new: Some(oid(1)),
        }],
        vec![],
        50,
    );
    assert!(j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/b", 1)], elsewhere),
            None
        )
        .is_ok());
}

#[test]
fn a_change_to_an_unrelated_ref_does_not_block() {
    let j = after_commit();
    let f = facts(&[("refs/heads/main", 2), ("refs/heads/other", 7)], main());
    assert!(j.plan(Direction::Undo, &f, None).is_ok());
}

#[test]
fn in_progress_operations_block() {
    let j = after_commit();
    let mut f = facts(&[("refs/heads/main", 2)], main());
    f.in_progress = Some("rebase".into());
    let err = j.plan(Direction::Undo, &f, None).unwrap_err();
    assert_eq!(
        err,
        UndoRefusal::InProgress {
            op: "rebase".into()
        }
    );
    assert!(err.message().contains("finish or abort the rebase"));
}

#[test]
fn a_stale_expectation_refuses() {
    let j = after_commit();
    let f = facts(&[("refs/heads/main", 2)], main());
    assert_eq!(
        j.plan(Direction::Undo, &f, Some(99)),
        Err(UndoRefusal::StaleEntry {
            expected_id: 99,
            top_id: 1
        })
    );
    assert!(j.plan(Direction::Undo, &f, Some(1)).is_ok());
}

#[test]
fn config_must_be_as_the_entry_left_it() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::BranchDelete,
        "d".into(),
        1,
        Some(main()),
        false,
        vec![],
    );
    let kv = |v: &str| vec![("branch.x.remote".to_string(), v.to_string())];
    j.finish(
        id,
        Some(main()),
        vec![RefChange {
            name: "refs/heads/x".into(),
            old: Some(oid(5)),
            new: None,
        }],
        vec![ConfigChange {
            section: "branch.x".into(),
            old: kv("origin"),
            new: vec![],
        }],
        50,
    );
    let mut f = facts(&[], main());
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert!(matches!(plan.steps.last(), Some(Step::SetConfig { to, .. }) if *to == kv("origin")));
    f.config.insert("branch.x".into(), kv("elsewhere"));
    assert_eq!(
        j.plan(Direction::Undo, &f, None),
        Err(UndoRefusal::ConfigChanged {
            section: "branch.x".into()
        })
    );
}
