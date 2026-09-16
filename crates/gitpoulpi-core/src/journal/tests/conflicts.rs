use super::*;
use crate::domain::{ConflictEntry, StageEntry};

fn stage(n: u8) -> Option<StageEntry> {
    Some(StageEntry {
        mode: 0o100644,
        oid: oid(n),
    })
}

fn conflicted_entry() -> ConflictEntry {
    ConflictEntry::from_stages("c.txt".into(), stage(1), stage(2), stage(3)).unwrap()
}

/// A finished "Save & mark resolved" of `c.txt`: it was conflicted with a
/// marker file (worktree 2, no stage-0 entry) and is now resolved.
fn after_resolve() -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::ResolveConflict,
        "Resolve c.txt".into(),
        10,
        None,
        false,
        vec![],
    );
    let entry = j.entry_mut(id).unwrap();
    entry.files = Some(PathChange {
        paths: vec!["c.txt".into()],
        before: Snapshot {
            tree: oid(50),
            hashes: vec![file("c.txt", Some(2), None)],
        },
        after: Snapshot {
            tree: oid(60),
            hashes: vec![file("c.txt", Some(9), Some(9))],
        },
    });
    entry.conflict = Some(ConflictChange {
        entry: conflicted_entry(),
    });
    j.finish(id, None, vec![], vec![], usize::MAX);
    j
}

fn merging(mut f: RepoFacts) -> RepoFacts {
    f.in_progress = Some("merge".into());
    f
}

#[test]
fn a_resolution_undoes_by_putting_the_conflict_back_while_the_operation_runs() {
    let j = after_resolve();
    let mut f = merging(facts(&[], main()));
    f.files.insert(1, vec![file("c.txt", Some(9), Some(9))]);
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    assert_eq!(
        plan.steps,
        vec![Step::Unresolve {
            path: "c.txt".into(),
            entry: conflicted_entry(),
            resolved: oid(60),
        }]
    );
    assert!(plan.describe()[0].contains("c.txt"));
    // A failed plan rolls back to the resolved snapshot.
    assert_eq!(
        plan.steps[0].inverse(),
        Step::RestorePaths {
            paths: vec!["c.txt".into()],
            to: oid(60),
            from: oid(60)
        }
    );
}

#[test]
fn a_resolution_is_refused_once_edited_since_or_after_the_operation_ended() {
    let j = after_resolve();
    let mut f = merging(facts(&[], main()));
    f.files.insert(1, vec![file("c.txt", Some(5), Some(9))]);
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::PathChanged {
            path: "c.txt".into()
        }
    );
    // The merge was concluded (or aborted): nothing to put back.
    let mut f = facts(&[], main());
    f.files.insert(1, vec![file("c.txt", Some(9), Some(9))]);
    assert_eq!(
        j.plan(Direction::Undo, &f, None).unwrap_err(),
        UndoRefusal::OperationEnded
    );
}

#[test]
fn a_resolution_redoes_from_the_snapshot_and_other_entries_still_refuse_during_an_operation() {
    let mut j = after_resolve();
    let mut f = merging(facts(&[], main()));
    f.files.insert(1, vec![file("c.txt", Some(9), Some(9))]);
    let plan = j.plan(Direction::Undo, &f, None).unwrap();
    j.commit(&plan);
    // After the undo the daemon records what the unresolve left.
    j.redo
        .last_mut()
        .unwrap()
        .files
        .as_mut()
        .unwrap()
        .before
        .hashes = vec![file("c.txt", Some(3), None)];
    f.files.insert(1, vec![file("c.txt", Some(3), None)]);
    let redo = j.plan(Direction::Redo, &f, None).unwrap();
    assert_eq!(
        redo.steps,
        vec![Step::RestorePaths {
            paths: vec!["c.txt".into()],
            to: oid(60),
            from: oid(50)
        }]
    );

    // The ordinary rule is untouched: a commit does not undo mid-merge.
    let j = after_commit();
    let f = merging(facts(&[("refs/heads/main", 2)], main()));
    assert!(matches!(
        j.plan(Direction::Undo, &f, None),
        Err(UndoRefusal::InProgress { .. })
    ));
}

#[test]
fn dropping_resolutions_leaves_every_other_entry_and_can_keep_the_newest() {
    let mut j = after_commit();
    for _ in 0..3 {
        let mut r = after_resolve();
        let mut entry = r.undo.pop().unwrap();
        entry.id = j.next_id;
        j.next_id += 1;
        j.undo.push(entry);
    }
    assert_eq!(j.undo.len(), 4);
    let gone = j.drop_resolutions(Some(2));
    assert_eq!(gone.len(), 1);
    assert_eq!(j.undo.len(), 3);
    assert_eq!(j.undo[0].kind, OperationKind::Commit, "oldest stays first");
    let gone = j.drop_resolutions(None);
    assert_eq!(gone.len(), 2);
    assert_eq!(j.undo.len(), 1);
    assert_eq!(j.undo[0].kind, OperationKind::Commit);
}
