//! The Solo mapping (`solo_shown_indices`) is asked every frame: it must run
//! its reachability pass only when its inputs change.

use super::*;
use crate::test_support::*;

fn computes() -> usize {
    SHOWN_COMPUTES.with(|n| n.get())
}

#[test]
fn the_solo_mapping_is_computed_once_until_its_inputs_change() {
    // 4 -> 3 -> 1 and 2 -> 1: soloing `a` (at 4) shows 4, 3, 1.
    let nodes = vec![
        commit(4, &[3]),
        commit(3, &[1]),
        commit(2, &[1]),
        commit(1, &[]),
    ];
    let refs = Ok(vec![
        local_branch_at("a", oid(4)),
        local_branch_at("b", oid(2)),
    ]);
    let solo: std::collections::HashSet<String> = ["a".to_string()].into();
    let before = computes();

    let first = solo_shown_indices(&nodes, &solo, &refs).expect("filtered");
    assert_eq!(*first, vec![0, 1, 3]);
    for _ in 0..50 {
        let again = solo_shown_indices(&nodes, &solo, &refs).expect("filtered");
        assert!(std::sync::Arc::ptr_eq(&first, &again), "the cached answer");
    }
    assert_eq!(computes(), before + 1, "one pass for fifty frames");

    // Another soloed ref: a new answer, computed once.
    let both: std::collections::HashSet<String> = ["a".to_string(), "b".to_string()].into();
    let wider = solo_shown_indices(&nodes, &both, &refs).expect("filtered");
    assert_eq!(*wider, vec![0, 1, 2, 3]);
    solo_shown_indices(&nodes, &both, &refs);
    assert_eq!(computes(), before + 2);

    // A soloed ref that moved: recomputed.
    let moved = Ok(vec![
        local_branch_at("a", oid(3)),
        local_branch_at("b", oid(2)),
    ]);
    let narrower = solo_shown_indices(&nodes, &solo, &moved).expect("filtered");
    assert_eq!(*narrower, vec![1, 3]);
    assert_eq!(computes(), before + 3);

    // A different held list (a refresh): recomputed.
    let fresh = nodes.clone();
    solo_shown_indices(&fresh, &solo, &moved);
    assert_eq!(computes(), before + 4);

    // No Solo: nothing to map, nothing computed.
    assert!(solo_shown_indices(&nodes, &Default::default(), &refs).is_none());
    assert_eq!(computes(), before + 4);
}
