use super::*;
use gitpoulpi_core::domain::{RefVisibility, StashId, TagSync};

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn base(kind: RefKind, at: u8) -> Ref {
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

fn local(name: &str, at: u8, head: bool, upstream: Option<&str>) -> Ref {
    Ref {
        is_head: head,
        upstream: upstream.map(str::to_string),
        ..base(RefKind::LocalBranch { name: name.into() }, at)
    }
}

fn remote(r: &str, name: &str, at: u8) -> Ref {
    base(
        RefKind::RemoteBranch {
            remote: r.into(),
            name: name.into(),
        },
        at,
    )
}

fn tag(name: &str, sync: Option<TagSync>) -> Ref {
    Ref {
        tag_sync: sync,
        ..base(RefKind::Tag { name: name.into() }, 1)
    }
}

fn snap() -> Snapshot {
    Snapshot::new(
        vec![
            local("main", 1, true, Some("origin/main")),
            local("feature/x", 2, false, None),
            remote("origin", "main", 1),
            remote("origin", "topic", 3),
            tag("v1", None),
        ],
        vec![Stash {
            id: StashId(0),
            message: "wip".into(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: Some(oid(9)),
            base: None,
        }],
    )
}

#[test]
fn names_are_checked_like_the_terminal_ui_checks_them() {
    let refs = snap().refs;
    assert_eq!(
        validate_new_ref(NewRef::Branch, " feat/y ", &refs).as_deref(),
        Ok("feat/y")
    );
    for bad in ["", "a b", "a..b", "-x", "x/", "a.lock", "@", "a~1", "a//b"] {
        assert!(
            validate_new_ref(NewRef::Branch, bad, &refs).is_err(),
            "{bad}"
        );
    }
    assert!(validate_new_ref(NewRef::Branch, "main", &refs).is_err());
    assert!(
        validate_new_ref(NewRef::Branch, "feature", &refs).is_err(),
        "a branch named like a folder of another clashes"
    );
    assert!(validate_new_ref(NewRef::Branch, "feature/x/y", &refs).is_err());
    assert!(
        validate_new_ref(NewRef::Tag, "main", &refs).is_ok(),
        "tags and branches have their own namespaces"
    );
    assert!(validate_new_ref(NewRef::Tag, "v1", &refs).is_err());
    assert!(validate_rename("main", "main", &refs).is_ok());
    assert!(validate_rename("main", "feature/x", &refs).is_err());
}

#[test]
fn an_upstream_must_exist_and_empty_unsets() {
    let refs = snap().refs;
    assert_eq!(validate_upstream("", &refs), Ok(None));
    assert_eq!(
        validate_upstream("origin/topic", &refs),
        Ok(Some("origin/topic".into()))
    );
    assert!(validate_upstream("origin/nope", &refs).is_err());
}

#[test]
fn a_remote_branch_checks_out_through_a_tracking_branch() {
    let s = snap();
    match checkout_plan(&RefTarget::Remote("origin/topic".into()), &s) {
        Plan::Confirm { action, danger, .. } => {
            assert!(!danger);
            assert_eq!(
                action,
                Action::BranchCreateTracking {
                    local_name: "topic".into(),
                    upstream: "origin/topic".into()
                }
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        checkout_plan(&RefTarget::Remote("origin/main".into()), &s),
        Plan::Say(_)
    ));
    assert!(matches!(
        checkout_plan(&RefTarget::Local("feature/x".into()), &s),
        Plan::Submit {
            action: Action::CheckoutBranch(_),
            ..
        }
    ));
}

#[test]
fn deleting_the_checked_out_branch_is_refused_and_others_ask() {
    let s = snap();
    assert!(matches!(delete_branch_plan(&s, "main"), Plan::Say(_)));
    match delete_branch_plan(&s, "feature/x") {
        Plan::Confirm {
            danger,
            action: Action::BranchDelete(n),
            ..
        } => {
            assert!(danger);
            assert_eq!(n, "feature/x");
        }
        other => panic!("{other:?}"),
    }
    // main has a remote twin: both deletes are offered, but the local half is the checked-out one.
    assert!(matches!(
        delete_remote_plan(&s, &RefTarget::Local("main".into()), true),
        Plan::Say(_)
    ));
    assert!(matches!(
        delete_remote_plan(&s, &RefTarget::Local("main".into()), false),
        Plan::Confirm {
            action: Action::BranchDeleteRemote {
                and_local: None,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        delete_remote_plan(&s, &RefTarget::Local("feature/x".into()), false),
        Plan::Say(_)
    ));
    let by_remote = delete_targets(&s, &RefTarget::Remote("origin/topic".into()));
    assert_eq!(by_remote.local, None);
    assert_eq!(by_remote.remote, Some(("origin".into(), "topic".into())));
}

#[test]
fn a_fast_forward_needs_a_branch_and_a_difference() {
    let s = snap();
    assert!(matches!(
        ff_plan(&s, &RefTarget::Local("feature/x".into()), true),
        Plan::Submit {
            action: Action::FastForward { .. },
            ..
        }
    ));
    assert!(matches!(
        ff_plan(&s, &RefTarget::Remote("origin/main".into()), false),
        Plan::Say(_)
    ));
    let detached = Snapshot::new(vec![local("a", 1, false, None)], vec![]);
    assert!(matches!(
        ff_plan(&detached, &RefTarget::Local("a".into()), true),
        Plan::Say(_)
    ));
}

#[test]
fn the_tag_delete_question_follows_where_the_tag_lives() {
    let only_here = Snapshot::new(vec![tag("t", Some(TagSync::default()))], vec![]);
    assert!(matches!(
        tag_delete_start(&only_here, "t"),
        TagDelete::Plan(p) if matches!(*p, Plan::Confirm { .. })
    ));
    let both = Snapshot::new(
        vec![tag(
            "t",
            Some(TagSync {
                pushed_to: vec!["origin".into()],
                differs_on: vec![],
                local: true,
            }),
        )],
        vec![],
    );
    match tag_delete_start(&both, "t") {
        TagDelete::Ask(rows) => assert_eq!(
            rows,
            vec![
                TagScope::Local,
                TagScope::Remote("origin".into()),
                TagScope::Both("origin".into())
            ]
        ),
        _ => panic!("asks where"),
    }
    // Deleting here while a remote has it runs at once and says the remote still has it.
    assert!(matches!(
        tag_scope_plan(&both, "t", &TagScope::Local, &["origin".to_string()]),
        Plan::Submit {
            action: Action::TagDelete(_),
            ..
        }
    ));
    assert!(matches!(
        tag_scope_plan(&both, "t", &TagScope::Both("origin".into()), &[]),
        Plan::Confirm {
            danger: true,
            action: Action::TagDeleteBoth { .. },
            ..
        }
    ));
}

#[test]
fn pushing_a_tag_needs_a_default_remote() {
    let s = snap();
    assert!(matches!(
        push_tag_plan(&s, "v1"),
        Plan::Submit {
            action: Action::TagPush { .. },
            ..
        }
    ));
    let none = Snapshot::new(vec![local("a", 1, true, None)], vec![]);
    assert!(matches!(push_tag_plan(&none, "v1"), Plan::Say(_)));
}

#[test]
fn a_drop_offers_merge_rebase_and_fast_forward_and_checks_the_branch_out_first() {
    let s = snap();
    let src = RefTarget::Local("main".into());
    let dst = RefTarget::Local("feature/x".into());
    let rows = drop_rows(&s, &src, &dst);
    assert_eq!(rows.len(), 3);
    assert!(drop_rows(&s, &src, &src).is_empty());
    assert!(!drop_is_valid(&RefTarget::Tag("v1".into()), &dst));
    match drop_plan(&s, DropOp::Merge, &src, &dst) {
        Plan::Submit {
            action: Action::CheckoutThen { branch, then },
            ..
        } => {
            assert_eq!(branch, "feature/x");
            assert!(matches!(*then, Action::Merge(_)));
        }
        other => panic!("{other:?}"),
    }
    // Rebase of the checked-out source needs no checkout.
    assert!(matches!(
        drop_plan(&s, DropOp::Rebase, &src, &dst),
        Plan::Submit {
            action: Action::Rebase(_),
            ..
        }
    ));
}

#[test]
fn the_qualified_spelling_is_git_s() {
    assert_eq!(
        qualified_ref_name(&RefTarget::Remote("origin/x".into())).as_deref(),
        Some("refs/remotes/origin/x")
    );
    assert_eq!(qualified_ref_name(&RefTarget::Stash(1)), None);
}
