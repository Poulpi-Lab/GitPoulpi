use super::*;

#[test]
fn a_checkout_plans_a_pin_step_then_a_checkout() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Checkout,
        "co".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    j.finish(id, Some(HeadState::Detached(oid(3))), vec![], vec![], 50);
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[], HeadState::Detached(oid(3))),
            None,
        )
        .unwrap();
    assert!(matches!(plan.steps[0], Step::UpdateRefs(_)));
    assert_eq!(
        plan.steps[1],
        Step::Checkout {
            to: main(),
            from: HeadState::Detached(oid(3))
        }
    );
    assert_eq!(plan.describe(), vec!["check out main".to_string()]);
}

#[test]
fn a_tracking_branch_is_undone_checkout_first_and_redone_checkout_last() {
    let work = HeadState::Branch("work".into());
    let section = |v: &[(&str, &str)]| -> Vec<(String, String)> {
        v.iter()
            .map(|(k, x)| (k.to_string(), x.to_string()))
            .collect()
    };
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::BranchCreateTracking,
        "t".into(),
        1,
        Some(main()),
        true,
        vec![("refs/heads/work".into(), None)],
    );
    j.finish(
        id,
        Some(work.clone()),
        vec![RefChange {
            name: "refs/heads/work".into(),
            old: None,
            new: Some(oid(5)),
        }],
        vec![ConfigChange {
            section: "branch.work".into(),
            old: vec![],
            new: section(&[("branch.work.remote", "origin")]),
        }],
        50,
    );
    let mut at = facts(&[("refs/heads/work", 5)], work.clone());
    at.config.insert(
        "branch.work".into(),
        section(&[("branch.work.remote", "origin")]),
    );
    let undo = j.plan(Direction::Undo, &at, None).unwrap();
    // The pin for the tip about to be un-referenced, then the way back.
    assert!(matches!(undo.steps[0], Step::UpdateRefs(_)));
    assert!(matches!(undo.steps[1], Step::Checkout { .. }));
    assert!(matches!(undo.steps[2], Step::UpdateRefs(_)));
    assert!(matches!(undo.steps[3], Step::SetConfig { .. }));
    j.commit(&undo);

    let mut gone = facts(&[], main());
    gone.config.insert("branch.work".into(), vec![]);
    let redo = j.plan(Direction::Redo, &gone, None).unwrap();
    assert!(matches!(redo.steps[0], Step::UpdateRefs(_)));
    assert!(matches!(redo.steps[1], Step::SetConfig { .. }));
    assert_eq!(
        redo.steps[2],
        Step::Checkout {
            to: work,
            from: main()
        }
    );
}

#[test]
fn a_fast_forward_moves_files_only_for_the_checked_out_branch() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::FastForward,
        "ff".into(),
        1,
        None,
        false,
        vec![],
    );
    j.finish(
        id,
        None,
        vec![RefChange {
            name: "refs/heads/main".into(),
            old: Some(oid(1)),
            new: Some(oid(2)),
        }],
        vec![],
        50,
    );
    // Checked out: a reset --keep step, after the pin.
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], main()),
            None,
        )
        .unwrap();
    assert_eq!(
        plan.steps.last(),
        Some(&Step::MoveHeadKeep {
            branch: "main".into(),
            from: oid(2),
            to: oid(1)
        })
    );
    assert!(plan.steps.iter().all(|s| match s {
        Step::UpdateRefs(u) => u.iter().all(|u| !matches!(u, RefUpdate::Update { .. })),
        _ => true,
    }));
    // Not checked out: the ref alone moves.
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], HeadState::Branch("other".into())),
            None,
        )
        .unwrap();
    assert!(!plan
        .steps
        .iter()
        .any(|s| matches!(s, Step::MoveHeadKeep { .. })));
    assert!(plan.steps.iter().any(|s| matches!(
        s,
        Step::UpdateRefs(u) if u.iter().any(|x| matches!(x, RefUpdate::Update { .. }))
    )));
}
