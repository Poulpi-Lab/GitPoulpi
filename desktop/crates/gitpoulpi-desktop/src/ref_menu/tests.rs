use super::*;
use gitpoulpi_core::domain::{Oid, Ref, RefKind, RefVisibility, Stash, StashId, TagSync};

fn base(kind: RefKind, at: u8) -> Ref {
    Ref {
        kind,
        target: Oid([at; 20]),
        visibility: RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    }
}

fn snap() -> Snapshot {
    Snapshot::new(
        vec![
            Ref {
                is_head: true,
                upstream: Some("origin/main".into()),
                ..base(
                    RefKind::LocalBranch {
                        name: "main".into(),
                    },
                    1,
                )
            },
            base(
                RefKind::LocalBranch {
                    name: "topic".into(),
                },
                2,
            ),
            base(
                RefKind::RemoteBranch {
                    remote: "origin".into(),
                    name: "main".into(),
                },
                1,
            ),
            Ref {
                tag_sync: Some(TagSync::default()),
                ..base(RefKind::Tag { name: "v1".into() }, 1)
            },
        ],
        vec![Stash {
            id: StashId(0),
            message: "wip".into(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: None,
            base: None,
        }],
    )
}

fn labels(items: &[Item]) -> Vec<String> {
    items.iter().map(|i| i.label().to_string()).collect()
}

#[test]
fn the_checked_out_branch_has_no_checkout_merge_rebase_or_delete_target_confusion() {
    let s = snap();
    let rows = items_for(&RefTarget::Local("main".into()), &s, MenuFacts::default());
    let l = labels(&rows);
    assert!(l[0].contains("Reveal"), "{l:?}");
    assert!(!l.iter().any(|x| x.contains("Check out")));
    assert!(!l.iter().any(|x| x.contains("Merge")));
    assert!(l.iter().any(|x| x.contains("Rename")));
    // main has a remote twin: three delete rows.
    assert_eq!(
        l.iter().filter(|x| x.contains("Delete")).count(),
        3,
        "{l:?}"
    );
    assert!(matches!(rows.last(), Some(Item::Submenu { .. })));
}

#[test]
fn another_branch_gets_checkout_merge_rebase_and_both_fast_forwards() {
    let s = snap();
    let rows = items_for(&RefTarget::Local("topic".into()), &s, MenuFacts::default());
    let l = labels(&rows);
    let at = |needle: &str| l.iter().position(|x| x.contains(needle)).unwrap();
    assert!(at("Check out") < at("Merge") && at("Merge") < at("Rebase"));
    assert_eq!(l.iter().filter(|x| x.contains("Fast-forward")).count(), 2);
    assert!(
        !l.iter().any(|x| x.contains("Delete origin")),
        "no remote twin"
    );
}

#[test]
fn a_stash_menu_is_pop_apply_drop() {
    let s = snap();
    let rows = items_for(&RefTarget::Stash(0), &s, MenuFacts::default());
    let l = labels(&rows);
    assert!(l.iter().any(|x| x.contains("Pop")) && l.iter().any(|x| x.contains("Apply")));
    let drop = rows.iter().find(|i| i.label().contains("Drop")).unwrap();
    assert!(matches!(drop, Item::Action { danger: true, .. }));
}

#[test]
fn a_tag_only_here_offers_a_push() {
    let s = snap();
    let rows = items_for(&RefTarget::Tag("v1".into()), &s, MenuFacts::default());
    assert!(labels(&rows).iter().any(|x| x.contains("default remote")));
}

#[test]
fn rows_the_desktop_cannot_run_are_dimmed_with_a_reason() {
    let s = snap();
    let rows = items_for(&RefTarget::Local("topic".into()), &s, MenuFacts::default());
    let dimmed = rows
        .iter()
        .filter(|i| {
            matches!(
                i,
                Item::Action {
                    disabled: Some(_),
                    ..
                }
            )
        })
        .count();
    assert!(dimmed >= 3);
}
