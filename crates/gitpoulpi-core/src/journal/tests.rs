use super::*;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn main() -> HeadState {
    HeadState::Branch("main".into())
}

fn facts(refs: &[(&str, u8)], head: HeadState) -> RepoFacts {
    RepoFacts {
        snapshot: RefSnapshot {
            head,
            refs: refs.iter().map(|(n, o)| (n.to_string(), oid(*o))).collect(),
            symrefs: Default::default(),
        },
        in_progress: None,
        config: BTreeMap::new(),
        index_tree: None,
        files: BTreeMap::new(),
        stashes: Vec::new(),
        index_sum: None,
        unmerged: false,
    }
}

/// A journal with one finished commit: main 1 -> 2.
fn after_commit() -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Commit,
        "Commit \"x\"".into(),
        10,
        Some(main()),
        true,
        vec![("refs/heads/main".into(), Some(oid(1)))],
    );
    j.finish(
        id,
        Some(main()),
        vec![RefChange {
            name: "refs/heads/main".into(),
            old: Some(oid(1)),
            new: Some(oid(2)),
        }],
        vec![],
        50,
    );
    j
}

/// A finished history move of `main`, 1 -> 2, with `effect`.
fn history_move(effect: Effect) -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Merge,
        "Merge x".into(),
        1,
        Some(main()),
        true,
        vec![("refs/heads/main".into(), Some(oid(1)))],
    );
    j.finish(
        id,
        Some(main()),
        vec![RefChange {
            name: "refs/heads/main".into(),
            old: Some(oid(1)),
            new: Some(oid(2)),
        }],
        vec![],
        50,
    );
    j.entry_mut(id).unwrap().effect = effect;
    j
}

fn file(path: &str, wt: Option<u8>, ix: Option<u8>) -> PathHash {
    let hash = |n: u8| crate::domain::FileHash {
        mode: 0o100644,
        oid: oid(n),
    };
    PathHash {
        path: path.into(),
        worktree: wt.map(hash),
        index: ix.map(hash),
    }
}

/// A finished discard of `a.txt` (modified) and `b.txt` (untracked).
fn after_discard() -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Discard,
        "Discard 2 files".into(),
        10,
        None,
        false,
        vec![],
    );
    let entry = j.entry_mut(id).unwrap();
    entry.files = Some(PathChange {
        paths: vec!["a.txt".into(), "b.txt".into()],
        before: Snapshot {
            tree: oid(50),
            hashes: vec![
                file("a.txt", Some(2), Some(1)),
                file("b.txt", Some(3), None),
            ],
        },
        after: Snapshot {
            tree: oid(60),
            hashes: vec![file("a.txt", Some(1), Some(1))],
        },
    });
    j.finish(id, None, vec![], vec![], 50);
    j
}

fn stash_drop_journal() -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::StashDrop,
        "Drop stash".into(),
        0,
        None,
        false,
        vec![],
    );
    j.entry_mut(id).unwrap().stash = Some(StashChange {
        added: vec![],
        removed: vec![StashRecord {
            commit: oid(7),
            message: "On main: wip".into(),
        }],
    });
    j.finish(id, None, vec![], vec![], 10);
    j
}

mod checkout;
mod commit;
mod conflicts;
mod depth;
mod files;
mod history;
mod lifecycle;
mod refusals;
mod restore;
mod stash;
