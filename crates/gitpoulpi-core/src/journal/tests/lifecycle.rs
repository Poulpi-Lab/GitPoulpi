use super::*;

#[test]
fn a_pending_entry_is_interrupted_after_recovery_and_never_undone() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Commit,
        "c".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    assert!(j.recover());
    assert_eq!(j.undo[0].state, EntryState::Interrupted);
    assert_eq!(
        j.plan(Direction::Undo, &facts(&[], main()), None),
        Err(UndoRefusal::Interrupted { entry: id })
    );
    assert!(!j.recover());
}

#[test]
fn a_new_operation_retires_the_redo_stack_but_keeps_its_pins() {
    let mut j = after_commit();
    let plan = j
        .plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], main()),
            None,
        )
        .unwrap();
    j.commit(&plan);
    let id = j.begin(
        OperationKind::Commit,
        "d".into(),
        20,
        Some(main()),
        true,
        vec![],
    );
    j.finish(id, Some(main()), vec![], vec![], 50);
    assert!(j.redo.is_empty());
    assert_eq!(j.retired.len(), 1);
    assert_eq!(j.retired[0].pins.len(), 1, "still recoverable");
}

#[test]
fn the_history_is_bounded_and_dropped_entries_come_back_for_pin_release() {
    let mut j = Journal::default();
    let mut dropped_total = 0;
    for i in 0..(50 + 3) {
        let id = j.begin(
            OperationKind::Commit,
            format!("c{i}"),
            1,
            Some(main()),
            true,
            vec![],
        );
        dropped_total += j.finish(id, Some(main()), vec![], vec![], 50).len();
    }
    assert_eq!(j.undo.len(), 50);
    assert_eq!(dropped_total, 3);
    assert_eq!(j.undo[0].label, "c3");
}

#[test]
fn a_smaller_depth_drops_the_oldest_entries_and_never_below_one() {
    let mut j = Journal::default();
    for i in 0..3 {
        let id = j.begin(
            OperationKind::Commit,
            format!("c{i}"),
            1,
            Some(main()),
            true,
            vec![],
        );
        j.finish(id, Some(main()), vec![], vec![], usize::MAX);
    }
    assert_eq!(j.undo.len(), 3, "unlimited keeps all");
    let id = j.begin(
        OperationKind::Commit,
        "c3".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    let dropped = j.finish(id, Some(main()), vec![], vec![], 1);
    assert_eq!(j.undo.len(), 1);
    assert_eq!(j.undo[0].label, "c3");
    assert_eq!(dropped.len(), 3);
    let id = j.begin(
        OperationKind::Commit,
        "c4".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    j.finish(id, Some(main()), vec![], vec![], 0);
    assert_eq!(j.undo.len(), 1, "a depth of 0 entries is treated as 1");
}

#[test]
fn the_retention_ring_is_bounded_too() {
    let mut j = Journal::default();
    for round in 0..(MAX_RETIRED + 5) {
        let id = j.begin(
            OperationKind::Commit,
            format!("c{round}"),
            1,
            Some(main()),
            true,
            vec![],
        );
        j.finish(id, Some(main()), vec![], vec![], 50);
        let plan = Plan {
            direction: Direction::Undo,
            entry: j.undo.last().unwrap().summary(),
            steps: vec![],
            new_pins: vec![],
        };
        j.commit(&plan);
    }
    // Each new op retires the redo stack; the ring caps at MAX_RETIRED.
    assert!(j.retired.len() <= MAX_RETIRED);
}

#[test]
fn abort_removes_only_the_pending_entry() {
    let mut j = after_commit();
    let id = j.begin(
        OperationKind::Commit,
        "x".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    assert!(j.abort(id).is_some());
    assert_eq!(j.undo.len(), 1);
    assert!(j.abort(id).is_none());
}

#[test]
fn a_pending_entry_of_a_running_operation_says_in_progress_not_interrupted() {
    let mut j = Journal::default();
    j.begin(
        OperationKind::Rebase,
        "Rebase onto main".into(),
        1,
        Some(main()),
        true,
        vec![("refs/heads/main".into(), Some(oid(1)))],
    );
    let mut f = facts(&[("refs/heads/main", 1)], main());
    f.in_progress = Some("rebase".into());
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::InProgress {
            op: "rebase".into()
        }
    );
    f.in_progress = None;
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::Interrupted { entry: 1 }
    );
}

#[test]
fn steps_have_inverses_for_rollback() {
    let s = Step::UpdateRefs(vec![RefUpdate::Update {
        name: "refs/heads/m".into(),
        old: oid(1),
        new: oid(2),
    }]);
    assert_eq!(s.inverse().inverse(), s);
}

#[test]
fn state_reports_blocked_only_with_facts() {
    let j = after_commit();
    assert_eq!(j.state(None).blocked, None);
    let moved = facts(&[("refs/heads/main", 9)], main());
    assert!(j.state(Some(&moved)).blocked.is_some());
    let ok = facts(&[("refs/heads/main", 2)], main());
    assert_eq!(j.state(Some(&ok)).blocked, None);
    assert_eq!(j.state(Some(&ok)).undo.unwrap().id, 1);
}

#[test]
fn the_journal_round_trips_through_json() {
    let j = after_commit();
    let back: Journal = serde_json::from_str(&serde_json::to_string(&j).unwrap()).unwrap();
    assert_eq!(back, j);
}

#[test]
fn diff_snapshots_lists_created_moved_and_deleted_refs() {
    let snap = |refs: &[(&str, u8)]| RefSnapshot {
        head: main(),
        refs: refs.iter().map(|(n, o)| (n.to_string(), oid(*o))).collect(),
        symrefs: Default::default(),
    };
    let d = diff_snapshots(
        &snap(&[
            ("refs/heads/a", 1),
            ("refs/heads/b", 2),
            ("refs/heads/c", 3),
        ]),
        &snap(&[
            ("refs/heads/a", 1),
            ("refs/heads/b", 9),
            ("refs/heads/d", 4),
        ]),
    );
    assert_eq!(d.len(), 3);
    assert!(d.contains(&RefChange {
        name: "refs/heads/b".into(),
        old: Some(oid(2)),
        new: Some(oid(9))
    }));
    assert!(d.contains(&RefChange {
        name: "refs/heads/c".into(),
        old: Some(oid(3)),
        new: None
    }));
    assert!(d.contains(&RefChange {
        name: "refs/heads/d".into(),
        old: None,
        new: Some(oid(4))
    }));
}

#[test]
fn the_new_entry_fields_default_when_an_older_journal_is_read() {
    let mut value = serde_json::to_value(after_commit()).unwrap();
    for e in value["undo"].as_array_mut().unwrap() {
        e.as_object_mut().unwrap().remove("files");
        e.as_object_mut().unwrap().remove("index");
    }
    let j: Journal = serde_json::from_value(value).unwrap();
    assert!(j.undo[0].files.is_none() && j.undo[0].index.is_none());
}

#[test]
fn a_purged_entry_keeps_its_place_loses_its_pins_and_is_never_undone() {
    let mut j = after_commit();
    let id = j.undo[0].id;
    j.undo[0].pins.push(Pin {
        name: "refs/gitpoulpi/undo/1/0".into(),
        oid: Oid([1; 20]),
    });
    let pins = j.mark_purged(&[id]);
    assert_eq!(pins.len(), 1, "the pins come back to be deleted");
    assert_eq!(j.undo[0].state, EntryState::Purged);
    assert!(j.undo[0].pins.is_empty());
    assert_eq!(
        j.plan(
            Direction::Undo,
            &facts(&[("refs/heads/main", 2)], main()),
            None
        ),
        Err(UndoRefusal::Purged { entry: id })
    );
    assert!(UndoRefusal::Purged { entry: id }.forgettable());
    // Purging twice changes nothing.
    assert!(j.mark_purged(&[id]).is_empty());
}
