//! Hand-built histories shared by the golden test: merges, an octopus, stashes, a WIP row on a
//! detached HEAD. Ids are one repeated byte, so a fixture reads as a small table.

use gitpoulpi_core::domain::{CommitNode, Oid, Stash, StashId};

pub struct Fixture {
    pub name: &'static str,
    pub nodes: Vec<CommitNode>,
    pub ref_targets: Vec<Oid>,
    pub stashes: Vec<Stash>,
    pub dirty: bool,
    pub head: Option<Oid>,
}

pub fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn commit(id: u8, parents: &[u8], timestamp: i64) -> CommitNode {
    CommitNode {
        id: oid(id),
        parents: parents.iter().map(|p| oid(*p)).collect(),
        summary: format!("c{id}"),
        message: format!("c{id}"),
        author_name: String::new(),
        author_email: String::new(),
        timestamp,
        refs: vec![],
    }
}

fn stash(n: u32, commit: u8, base: u8, timestamp: i64) -> Stash {
    Stash {
        id: StashId(n),
        message: format!("s{n}"),
        branch_at_creation: String::new(),
        timestamp,
        commit: Some(oid(commit)),
        base: Some(oid(base)),
    }
}

/// Newest first, children before parents, as the daemon returns them.
pub fn all() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "linear",
            nodes: vec![commit(1, &[2], 30), commit(2, &[3], 20), commit(3, &[], 10)],
            ref_targets: vec![oid(1)],
            stashes: vec![],
            dirty: false,
            head: Some(oid(1)),
        },
        // A branch forked at 5 and merged back at 1, a second branch crossing it.
        Fixture {
            name: "merges",
            nodes: vec![
                commit(1, &[2, 3], 90),
                commit(2, &[4], 80),
                commit(3, &[5], 70),
                commit(4, &[6, 7], 60),
                commit(5, &[6], 50),
                commit(7, &[8], 40),
                commit(6, &[8], 30),
                commit(8, &[], 10),
            ],
            ref_targets: vec![oid(1), oid(3), oid(7)],
            stashes: vec![],
            dirty: false,
            head: Some(oid(1)),
        },
        Fixture {
            name: "octopus",
            nodes: vec![
                commit(1, &[2, 3, 4, 5], 90),
                commit(2, &[6], 80),
                commit(3, &[6], 70),
                commit(4, &[6], 60),
                commit(5, &[6], 50),
                commit(6, &[], 10),
            ],
            ref_targets: vec![oid(1), oid(4)],
            stashes: vec![],
            dirty: false,
            head: Some(oid(1)),
        },
        // Stashes on the tip, in the middle and on an old commit; one whose base is not loaded.
        Fixture {
            name: "stashes",
            nodes: vec![
                commit(1, &[2], 100),
                commit(2, &[3, 4], 90),
                commit(3, &[5], 80),
                commit(4, &[5], 70),
                commit(5, &[], 10),
            ],
            ref_targets: vec![oid(1), oid(4)],
            stashes: vec![
                stash(0, 10, 1, 120),
                stash(1, 11, 3, 85),
                stash(2, 12, 5, 95),
                stash(3, 13, 99, 50),
            ],
            dirty: false,
            head: Some(oid(1)),
        },
        Fixture {
            name: "wip_on_branch_tip",
            nodes: vec![commit(1, &[2], 30), commit(2, &[3], 20), commit(3, &[], 10)],
            ref_targets: vec![oid(1)],
            stashes: vec![stash(0, 10, 2, 25)],
            dirty: true,
            head: Some(oid(1)),
        },
        // Detached HEAD on a commit no ref points at: the WIP row hangs from it, below the tip.
        Fixture {
            name: "detached_head_wip",
            nodes: vec![
                commit(1, &[3], 50),
                commit(2, &[3], 40),
                commit(3, &[4], 30),
                commit(4, &[], 10),
            ],
            ref_targets: vec![oid(1)],
            stashes: vec![stash(0, 10, 2, 45)],
            dirty: true,
            head: Some(oid(2)),
        },
        Fixture {
            name: "empty_dirty",
            nodes: vec![],
            ref_targets: vec![],
            stashes: vec![],
            dirty: true,
            head: None,
        },
    ]
}
