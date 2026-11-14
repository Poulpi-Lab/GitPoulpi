use super::*;

fn head_on(name: &str, upstream: Option<&str>) -> Vec<Ref> {
    let mut r = local_branch_at(name, oid(1));
    r.is_head = true;
    r.upstream = upstream.map(str::to_string);
    vec![r]
}

#[test]
fn the_default_remote_follows_the_head_upstream_before_origin() {
    let remotes = vec!["origin".to_string(), "fork".to_string()];
    assert_eq!(
        default_remote(None, &head_on("main", Some("fork/main")), &remotes).as_deref(),
        Some("fork")
    );
    assert_eq!(
        default_remote(
            Some("origin"),
            &head_on("main", Some("fork/main")),
            &remotes
        )
        .as_deref(),
        Some("origin"),
        "a chosen default wins over the upstream"
    );
    assert_eq!(
        default_remote(None, &head_on("main", None), &remotes).as_deref(),
        Some("origin")
    );
}

#[test]
fn the_pull_button_does_what_its_mode_says() {
    let tracked = HeadBranch {
        name: "main".into(),
        upstream: Some("origin/main".into()),
    };
    let untracked = HeadBranch {
        name: "feature".into(),
        upstream: None,
    };
    assert_eq!(
        pull_action(PullButtonMode::Rebase, Some(&tracked), None)
            .unwrap()
            .0,
        Action::Pull(PullOptions {
            mode: PullMode::Rebase,
            remote: None,
            ..PullOptions::default()
        })
    );
    assert_eq!(
        pull_action(PullButtonMode::FfOnly, Some(&untracked), Some("fork"))
            .unwrap()
            .0,
        Action::Pull(PullOptions {
            mode: PullMode::FastForwardOnly,
            remote: Some("fork".into()),
            ..PullOptions::default()
        }),
        "no upstream: the same-named branch of the default remote"
    );
    assert!(pull_action(PullButtonMode::Merge, Some(&untracked), None).is_err());
    assert!(pull_action(PullButtonMode::Merge, None, Some("origin")).is_err());
}

#[test]
fn push_goes_direct_asks_or_refuses() {
    let new_branch = HeadBranch {
        name: "feature".into(),
        upstream: None,
    };
    match push_plan(
        PushBehavior::Direct,
        Some(&new_branch),
        Some("origin"),
        true,
    ) {
        PushPlan::Direct(Action::Push { remote, opts }, _) => {
            assert_eq!(remote, "origin");
            assert!(opts.set_upstream, "a first push sets the upstream");
            assert!(!opts.force_with_lease);
            assert_eq!(opts.branch.as_deref(), Some("feature"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        push_plan(PushBehavior::Ask, Some(&new_branch), Some("origin"), true),
        PushPlan::Ask
    );
    assert_eq!(
        push_plan(PushBehavior::Direct, Some(&new_branch), None, true),
        PushPlan::Ask,
        "no default remote: ask rather than guess"
    );
    assert!(matches!(
        push_plan(PushBehavior::Direct, None, Some("origin"), true),
        PushPlan::Refuse(_)
    ));
    assert!(matches!(
        push_plan(PushBehavior::Direct, Some(&new_branch), None, false),
        PushPlan::Refuse(_)
    ));
}

#[test]
fn the_push_menu_lists_the_default_remote_first() {
    let entries = push_to_entries(
        &["a".to_string(), "b".to_string(), "c".to_string()],
        Some("b"),
    );
    assert!(matches!(
        &entries[0],
        Entry::Action { action: RemoteUiAction::PushTo(r), label, .. }
            if r == "b" && label.contains("default")
    ));
}

#[test]
fn the_pull_dropdown_runs_its_items_and_marks_none() {
    let entries = pull_mode_entries();
    let actions: Vec<_> = entries
        .iter()
        .filter_map(|e| match e {
            Entry::Action { label, action, .. } => {
                assert!(!label.starts_with('\u{25C9}') && !label.starts_with('\u{25CB}'));
                Some(action.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        actions,
        vec![
            RemoteUiAction::FetchAllRemotes,
            RemoteUiAction::RunPull(PullButtonMode::Merge),
            RemoteUiAction::RunPull(PullButtonMode::FfOnly),
            RemoteUiAction::RunPull(PullButtonMode::Rebase),
        ]
    );
}

#[test]
fn the_diverged_dialog_offers_pull_force_with_lease_and_cancel() {
    let rejection = PushRejection {
        remote: "origin".into(),
        branch: "main".into(),
        reason: PushRejectReason::NonFastForward,
        detail: None,
    };
    let actions: Vec<RemoteUiAction> = diverged_entries(&rejection)
        .into_iter()
        .filter_map(|e| match e {
            Entry::Action { action, .. } => Some(action),
            _ => None,
        })
        .collect();
    assert_eq!(
        actions,
        vec![
            RemoteUiAction::PullThenPush {
                remote: "origin".into(),
                branch: "main".into()
            },
            RemoteUiAction::ForcePush {
                remote: "origin".into(),
                branch: "main".into(),
                after_fetch: true
            },
            RemoteUiAction::Cancel,
        ]
    );
}
