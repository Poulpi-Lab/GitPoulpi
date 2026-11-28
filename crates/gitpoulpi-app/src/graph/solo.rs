//! The Solo filter's pure half: which commit a soloed ref points at, and
//! which commits are reachable from the soloed refs.
//!
//! Solo accumulates (soloing a second branch adds its ancestry instead of
//! replacing the first one's), so the reachability walk seeds its stack with
//! every target at once.

use gitpoulpi_core::domain::{CommitNode, Oid, Ref};

use crate::refs_tree::ref_full_name;

/// Resolves one soloed ref (its full name, as stored in the solo set) down
/// to the commit oid it points at right now - `None` if the ref no longer
/// exists (e.g. it was deleted or renamed while soloed).
pub fn solo_target_oid(solo_ref: &str, refs: &[Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| ref_full_name(r) == solo_ref)
        .map(|r| r.target)
}

/// Every commit reachable from any oid in `targets` by walking `parents`
/// (both a first-parent mainline and every merge parent) - the ancestry
/// Solo mode shows, unioned across every currently-soloed ref, in the order
/// of `nodes`.
pub fn solo_filter_nodes(nodes: &[CommitNode], targets: &[Oid]) -> Vec<CommitNode> {
    let by_id: std::collections::HashMap<Oid, &CommitNode> =
        nodes.iter().map(|c| (c.id, c)).collect();
    let mut reachable: std::collections::HashSet<Oid> = std::collections::HashSet::new();
    let mut stack: Vec<Oid> = targets.to_vec();
    while let Some(id) = stack.pop() {
        if !reachable.insert(id) {
            continue;
        }
        if let Some(c) = by_id.get(&id) {
            stack.extend(c.parents.iter().copied());
        }
    }
    nodes
        .iter()
        .filter(|c| reachable.contains(&c.id))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{RefKind, RefVisibility};

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn commit(id: u8, parents: &[u8]) -> CommitNode {
        CommitNode {
            id: oid(id),
            parents: parents.iter().map(|&p| oid(p)).collect(),
            summary: String::new(),
            message: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }

    fn branch(kind: RefKind, at: u8) -> Ref {
        Ref {
            kind,
            target: oid(at),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    #[test]
    fn a_soloed_ref_resolves_by_its_full_name_and_a_missing_one_to_none() {
        let refs = vec![
            branch(
                RefKind::LocalBranch {
                    name: "main".into(),
                },
                1,
            ),
            branch(
                RefKind::RemoteBranch {
                    remote: "origin".into(),
                    name: "dev".into(),
                },
                2,
            ),
        ];
        assert_eq!(solo_target_oid("main", &refs), Some(oid(1)));
        assert_eq!(solo_target_oid("origin/dev", &refs), Some(oid(2)));
        assert_eq!(solo_target_oid("gone", &refs), None);
    }

    #[test]
    fn solo_keeps_the_union_of_the_targets_ancestry_in_node_order() {
        // 4 merges 2 and 3; 3 and 2 both sit on 1; 5 is a side line.
        let nodes = vec![
            commit(5, &[1]),
            commit(4, &[2, 3]),
            commit(3, &[1]),
            commit(2, &[1]),
            commit(1, &[]),
        ];
        let ids = |v: Vec<CommitNode>| v.iter().map(|c| c.id).collect::<Vec<_>>();
        assert_eq!(
            ids(solo_filter_nodes(&nodes, &[oid(4)])),
            vec![oid(4), oid(3), oid(2), oid(1)]
        );
        assert_eq!(
            ids(solo_filter_nodes(&nodes, &[oid(3), oid(5)])),
            vec![oid(5), oid(3), oid(1)]
        );
        assert!(solo_filter_nodes(&nodes, &[]).is_empty());
    }
}
