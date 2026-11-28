//! Comparing the rows the Log has selected ("select two commits and the right pane shows what differs").
//!
//! The rules are the shared ones (`gitpoulpi_app::compare`, with its table of which selection gives
//! which `DiffTarget`); this file only hands them the rows the desktop's Log holds.

use gitpoulpi_core::domain::CommitGraph;

use super::data::LogData;
pub use gitpoulpi_app::compare::{Card, CardKind, CompareSpec, EndMark};
use gitpoulpi_app::log_multiselect::{LogSelection, RowKey};

/// The comparison the selected rows stand for, when there are two or more the Log can still name.
pub fn resolve(marks: &LogSelection, data: &LogData, swapped: bool) -> Option<CompareSpec> {
    let display = data.display_marks();
    let keys = marks.in_display_order(&display);
    if keys.len() < 2 {
        return None;
    }
    // The shared rule looks the selected ends up by id, so only those commits are handed over.
    let nodes = data
        .nodes
        .iter()
        .filter(|n| keys.contains(&RowKey::Id(n.id)))
        .cloned()
        .collect();
    gitpoulpi_app::compare::resolve(
        marks,
        &display,
        &Ok(CommitGraph { nodes }),
        &Ok(data.stashes.as_ref().clone()),
        swapped,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphInput, GraphLayout};
    use gitpoulpi_core::domain::{CommitNode, DiffTarget, Oid};
    use std::sync::Arc;

    fn node(n: u8, parent: Option<u8>) -> CommitNode {
        CommitNode {
            id: Oid([n; 20]),
            parents: parent.map(|p| Oid([p; 20])).into_iter().collect(),
            summary: format!("c{n}"),
            message: format!("c{n}"),
            author_name: "Ada".into(),
            author_email: String::new(),
            timestamp: 0,
            refs: vec![],
        }
    }

    /// WIP, then c3 -> c2 -> c1.
    fn data() -> LogData {
        let nodes = Arc::new(vec![node(3, Some(2)), node(2, Some(1)), node(1, None)]);
        let layout = GraphLayout::build(&GraphInput {
            nodes: &nodes,
            refs: &[],
            stashes: &[],
            dirty: true,
            head: Some(Oid([3; 20])),
            palette_len: 4,
        });
        LogData::new(
            nodes.clone(),
            nodes,
            Arc::default(),
            Arc::default(),
            vec![],
            None,
            Some(Oid([3; 20])),
            layout,
        )
    }

    fn marked(keys: &[RowKey]) -> LogSelection {
        let mut m = LogSelection::default();
        for (i, k) in keys.iter().enumerate() {
            m.toggle(*k, (i > 0).then(|| keys[0]));
        }
        m
    }

    fn c(n: u8) -> RowKey {
        RowKey::Id(Oid([n; 20]))
    }

    #[test]
    fn two_commits_compare_older_against_newer_whatever_the_click_order() {
        let d = data();
        let spec = resolve(&marked(&[c(3), c(1)]), &d, false).unwrap();
        assert_eq!(
            spec.target,
            DiffTarget::CommitRange(Oid([1; 20]), Oid([3; 20]))
        );
        assert!(spec.can_swap);
        let swapped = resolve(&marked(&[c(1), c(3)]), &d, true).unwrap();
        assert_eq!(
            swapped.target,
            DiffTarget::CommitRange(Oid([3; 20]), Oid([1; 20]))
        );
        assert_eq!(swapped.cards[0].mark, Some(EndMark::From));
    }

    #[test]
    fn three_commits_combine_and_the_wip_row_reads_the_working_tree() {
        let d = data();
        let span = resolve(&marked(&[c(3), c(2), c(1)]), &d, false).unwrap();
        assert_eq!(
            span.target,
            DiffTarget::CommitSpan(Oid([1; 20]), Oid([3; 20]))
        );
        assert!(!span.can_swap);
        let wip = resolve(&marked(&[RowKey::Wip, c(2)]), &d, true).unwrap();
        assert_eq!(wip.target, DiffTarget::CommitVsWorkingTree(Oid([2; 20])));
        assert!(wip.reads_worktree() && !wip.can_swap);
        assert!(resolve(&marked(&[c(2)]), &d, false).is_none());
    }
}
