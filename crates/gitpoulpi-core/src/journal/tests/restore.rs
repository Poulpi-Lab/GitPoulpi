//! "Restore only this": the pure planner (`journal::restore`).

use super::*;
use crate::domain::{RestoreItemKind, ZERO_OID};
use crate::journal::{plan_restore, readd_lines, RestoreAction, RestoreFacts};

fn top(j: &Journal) -> &JournalEntry {
    j.undo.last().unwrap()
}

fn never_exists(_: &str) -> bool {
    false
}

fn deleted_ref(name: &str, kind: OperationKind, old: u8) -> Journal {
    let mut j = Journal::default();
    let id = j.begin(
        kind,
        format!("Delete {name}"),
        1,
        None,
        false,
        vec![(name.into(), Some(oid(old)))],
    );
    j.finish(
        id,
        None,
        vec![RefChange {
            name: name.into(),
            old: Some(oid(old)),
            new: None,
        }],
        vec![],
        50,
    );
    j
}

fn facts_with(refs: &[(&str, u8)]) -> RestoreFacts {
    RestoreFacts {
        refs: refs.iter().map(|(n, o)| (n.to_string(), oid(*o))).collect(),
        ..RestoreFacts::default()
    }
}

#[test]
fn a_deleted_branch_comes_back_at_its_pinned_oid() {
    let j = deleted_ref("refs/heads/feature", OperationKind::BranchDelete, 5);
    let offer = top(&j).restore_offer();
    assert_eq!(offer.items[0].kind, RestoreItemKind::Branch);
    let plan = plan_restore(top(&j), &facts_with(&[]), None, &never_exists).unwrap();
    assert_eq!(
        plan.actions,
        vec![RestoreAction::CreateRef {
            name: "refs/heads/feature".into(),
            oid: oid(5),
            kind: RestoreItemKind::Branch
        }]
    );
}

#[test]
fn a_deleted_tag_comes_back_as_a_tag() {
    let j = deleted_ref("refs/tags/v1", OperationKind::TagDelete, 6);
    let plan = plan_restore(top(&j), &facts_with(&[]), None, &never_exists).unwrap();
    assert!(matches!(
        &plan.actions[0],
        RestoreAction::CreateRef { name, kind: RestoreItemKind::Tag, .. } if name == "refs/tags/v1"
    ));
}

#[test]
fn a_taken_name_is_refused_with_a_free_suggestion() {
    let j = deleted_ref("refs/heads/feature", OperationKind::BranchDelete, 5);
    let facts = facts_with(&[
        ("refs/heads/feature", 9),
        ("refs/heads/feature-restored", 9),
    ]);
    let refusal = plan_restore(top(&j), &facts, None, &never_exists).unwrap_err();
    assert_eq!(
        refusal,
        UndoRefusal::NameTaken {
            name: "feature".into(),
            suggested: "feature-restored-2".into()
        }
    );
}

#[test]
fn another_name_restores_under_that_name_and_a_second_taken_one_refuses() {
    let j = deleted_ref("refs/heads/feature", OperationKind::BranchDelete, 5);
    let facts = facts_with(&[("refs/heads/feature", 9)]);
    let plan = plan_restore(top(&j), &facts, Some("feature-2"), &never_exists).unwrap();
    assert!(matches!(
        &plan.actions[0],
        RestoreAction::CreateRef { name, .. } if name == "refs/heads/feature-2"
    ));
    let facts = facts_with(&[("refs/heads/feature", 9), ("refs/heads/feature-2", 9)]);
    assert!(matches!(
        plan_restore(top(&j), &facts, Some("feature-2"), &never_exists),
        Err(UndoRefusal::NameTaken { .. })
    ));
    assert!(matches!(
        plan_restore(top(&j), &facts, Some("bad name"), &never_exists),
        Err(UndoRefusal::NotRestorable { .. })
    ));
}

#[test]
fn a_branch_that_is_back_at_the_same_oid_is_already_restored() {
    let j = deleted_ref("refs/heads/feature", OperationKind::BranchDelete, 5);
    let facts = facts_with(&[("refs/heads/feature", 5)]);
    assert_eq!(
        plan_restore(top(&j), &facts, None, &never_exists).unwrap_err(),
        UndoRefusal::AlreadyRestored
    );
}

#[test]
fn a_deleted_branch_takes_its_upstream_settings_along() {
    let mut j = deleted_ref("refs/heads/feature", OperationKind::BranchDelete, 5);
    let up = vec![
        ("branch.feature.remote".to_string(), "origin".to_string()),
        (
            "branch.feature.merge".to_string(),
            "refs/heads/feature".to_string(),
        ),
    ];
    j.undo.last_mut().unwrap().config = vec![ConfigChange {
        section: "branch.feature".into(),
        old: up.clone(),
        new: vec![],
    }];
    let plan = plan_restore(top(&j), &facts_with(&[]), None, &never_exists).unwrap();
    assert!(plan
        .actions
        .iter()
        .any(|a| matches!(a, RestoreAction::SetConfig { section, to, .. } if section == "branch.feature" && *to == up)));
    // Under another name the old name's settings are not claimed.
    let plan = plan_restore(top(&j), &facts_with(&[]), Some("other"), &never_exists).unwrap();
    assert!(!plan
        .actions
        .iter()
        .any(|a| matches!(a, RestoreAction::SetConfig { .. })));
}

#[test]
fn a_dropped_stash_goes_back_on_top_unless_it_is_there() {
    let j = stash_drop_journal();
    let plan = plan_restore(top(&j), &RestoreFacts::default(), None, &never_exists).unwrap();
    assert_eq!(
        plan.actions,
        vec![RestoreAction::StashStore {
            commit: oid(7),
            message: "On main: wip".into()
        }]
    );
    let facts = RestoreFacts {
        stashes: vec![oid(7)],
        ..RestoreFacts::default()
    };
    assert_eq!(
        plan_restore(top(&j), &facts, None, &never_exists).unwrap_err(),
        UndoRefusal::AlreadyRestored
    );
}

fn now(files: Vec<PathHash>) -> RestoreFacts {
    RestoreFacts {
        files,
        ..RestoreFacts::default()
    }
}

#[test]
fn a_discarded_file_that_is_as_the_entry_left_it_is_written_back_in_place() {
    let j = after_discard();
    // a.txt is as the discard left it (worktree 1, index 1); b.txt is gone.
    let plan = plan_restore(
        top(&j),
        &now(vec![file("a.txt", Some(1), Some(1))]),
        None,
        &never_exists,
    )
    .unwrap();
    assert_eq!(
        plan.actions,
        vec![RestoreAction::RestorePaths {
            paths: vec!["a.txt".into(), "b.txt".into()],
            snapshot: oid(50)
        }]
    );
}

#[test]
fn a_file_changed_since_is_written_next_to_it_never_over_it() {
    let j = after_discard();
    // a.txt was edited after the discard (worktree 9); b.txt was re-created.
    let facts = now(vec![
        file("a.txt", Some(9), Some(1)),
        file("b.txt", Some(8), None),
    ]);
    let plan = plan_restore(top(&j), &facts, None, &never_exists).unwrap();
    let dests: Vec<_> = plan
        .actions
        .iter()
        .filter_map(|a| match a {
            RestoreAction::SideWrite { path, dest, file } => {
                Some((path.as_str(), dest.as_str(), file.oid))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        dests,
        vec![
            ("a.txt", "a.txt.restored", oid(2)),
            ("b.txt", "b.txt.restored", oid(3))
        ]
    );
    assert!(!plan
        .actions
        .iter()
        .any(|a| matches!(a, RestoreAction::RestorePaths { .. })));
}

#[test]
fn the_side_file_name_is_unique() {
    let j = after_discard();
    let facts = now(vec![file("a.txt", Some(9), Some(1))]);
    let taken = |p: &str| p == "a.txt.restored" || p == "a.txt.restored.2";
    let plan = plan_restore(top(&j), &facts, None, &taken).unwrap();
    assert!(plan
        .actions
        .iter()
        .any(|a| matches!(a, RestoreAction::SideWrite { dest, .. } if dest == "a.txt.restored.3")));
}

#[test]
fn a_file_already_holding_the_kept_bytes_is_already_restored() {
    let j = after_discard();
    let facts = now(vec![
        file("a.txt", Some(2), Some(1)),
        file("b.txt", Some(3), None),
    ]);
    assert_eq!(
        plan_restore(top(&j), &facts, None, &never_exists).unwrap_err(),
        UndoRefusal::AlreadyRestored
    );
}

#[test]
fn a_hard_reset_never_writes_files_in_place() {
    let mut j = after_discard();
    j.undo.last_mut().unwrap().effect = Effect::Files;
    let plan = plan_restore(
        top(&j),
        &now(vec![file("a.txt", Some(1), Some(1))]),
        None,
        &never_exists,
    )
    .unwrap();
    assert!(plan
        .actions
        .iter()
        .all(|a| matches!(a, RestoreAction::SideWrite { .. })));
}

#[test]
fn a_removed_remote_comes_back_with_tracking_refs_symref_and_upstreams() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::RemoteRemove,
        "Remove remote origin".into(),
        1,
        None,
        false,
        vec![],
    );
    let url = vec![("remote.origin.url".to_string(), "x".to_string())];
    let up = vec![("branch.main.remote".to_string(), "origin".to_string())];
    j.finish(
        id,
        None,
        vec![RefChange {
            name: "refs/remotes/origin/main".into(),
            old: Some(oid(3)),
            new: None,
        }],
        vec![
            ConfigChange {
                section: "remote.origin".into(),
                old: url.clone(),
                new: vec![],
            },
            ConfigChange {
                section: "branch.main".into(),
                old: up.clone(),
                new: vec![],
            },
        ],
        50,
    );
    j.undo.last_mut().unwrap().symrefs = vec![SymrefChange {
        name: "refs/remotes/origin/HEAD".into(),
        old: Some("refs/remotes/origin/main".into()),
        new: None,
    }];
    let facts = facts_with(&[("refs/heads/main", 1)]);
    let plan = plan_restore(top(&j), &facts, None, &never_exists).unwrap();
    let text = plan.describe().join("\n");
    assert!(text.contains("origin/main"), "{text}");
    assert!(plan.actions.iter().any(
        |a| matches!(a, RestoreAction::SetConfig { section, to, .. } if section == "remote.origin" && *to == url)
    ));
    assert!(plan.actions.iter().any(
        |a| matches!(a, RestoreAction::SetConfig { section, to, .. } if section == "branch.main" && *to == up)
    ));
    assert!(plan.actions.iter().any(
        |a| matches!(a, RestoreAction::SetSymref { name, .. } if name == "refs/remotes/origin/HEAD")
    ));
}

#[test]
fn a_config_section_that_changed_since_is_left_alone_with_a_refusal() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::BranchSetUpstream,
        "Unset upstream of main".into(),
        1,
        None,
        false,
        vec![],
    );
    let old = vec![("branch.main.remote".to_string(), "origin".to_string())];
    j.finish(
        id,
        None,
        vec![],
        vec![ConfigChange {
            section: "branch.main".into(),
            old: old.clone(),
            new: vec![],
        }],
        50,
    );
    let mut facts = facts_with(&[("refs/heads/main", 1)]);
    // Unchanged since: re-added.
    let plan = plan_restore(top(&j), &facts, None, &never_exists).unwrap();
    assert!(matches!(&plan.actions[0], RestoreAction::SetConfig { to, .. } if *to == old));
    // Someone set another upstream since: refuse, change nothing.
    facts.config.insert(
        "branch.main".into(),
        vec![("branch.main.remote".to_string(), "upstream".to_string())],
    );
    assert!(matches!(
        plan_restore(top(&j), &facts, None, &never_exists),
        Err(UndoRefusal::ConfigChanged { .. })
    ));
}

#[test]
fn a_reset_is_restored_as_a_new_branch_at_the_old_tip() {
    let j = history_move(Effect::Files);
    let offer = top(&j).restore_offer();
    assert_eq!(offer.items[0].kind, RestoreItemKind::OldTip);
    let plan = plan_restore(
        top(&j),
        &facts_with(&[("refs/heads/main", 2)]),
        None,
        &never_exists,
    )
    .unwrap();
    match &plan.actions[0] {
        RestoreAction::CreateRef { name, oid: o, kind } => {
            assert_eq!(*o, oid(1));
            assert_eq!(*kind, RestoreItemKind::OldTip);
            assert!(name.starts_with("refs/heads/restore/main-"), "{name}");
            assert_ne!(name, "refs/heads/main");
        }
        other => panic!("{other:?}"),
    }
    // The branch name is the caller's choice.
    let plan = plan_restore(top(&j), &facts_with(&[]), Some("rescue"), &never_exists).unwrap();
    assert!(matches!(
        &plan.actions[0],
        RestoreAction::CreateRef { name, .. } if name == "refs/heads/rescue"
    ));
}

#[test]
fn a_commit_amend_or_rebase_restores_the_old_tip_the_same_way() {
    let j = after_commit();
    assert_eq!(
        top(&j).restore_offer().items[0].kind,
        RestoreItemKind::OldTip
    );
}

#[test]
fn staging_a_checkout_and_a_created_branch_have_typed_reasons() {
    // A checkout only moved HEAD.
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::Checkout,
        "Checkout dev".into(),
        1,
        Some(main()),
        true,
        vec![],
    );
    j.finish(
        id,
        Some(HeadState::Branch("dev".into())),
        vec![],
        vec![],
        50,
    );
    let offer = top(&j).restore_offer();
    assert!(!offer.restorable());
    assert!(offer.not_restorable.unwrap().contains("HEAD"));
    assert!(matches!(
        plan_restore(top(&j), &RestoreFacts::default(), None, &never_exists),
        Err(UndoRefusal::NotRestorable { .. })
    ));
    // A staging entry only changed the index.
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::StageAll,
        "Stage all".into(),
        1,
        None,
        false,
        vec![],
    );
    j.undo.last_mut().unwrap().index = Some(IndexChange {
        before: oid(1),
        after: oid(2),
        known: None,
    });
    j.finish(id, None, vec![], vec![], 50);
    assert!(top(&j)
        .restore_offer()
        .not_restorable
        .unwrap()
        .contains("index"));
    // A created branch removed nothing.
    let j = {
        let mut j = Journal::default();
        let id = j.begin(
            OperationKind::BranchCreate,
            "Create branch x".into(),
            1,
            None,
            false,
            vec![("refs/heads/x".into(), None)],
        );
        j.finish(
            id,
            None,
            vec![RefChange {
                name: "refs/heads/x".into(),
                old: None,
                new: Some(oid(1)),
            }],
            vec![],
            50,
        );
        j
    };
    assert!(!top(&j).restore_offer().restorable());
}

#[test]
fn an_entry_that_never_completed_is_not_restorable() {
    let mut j = Journal::default();
    j.begin(
        OperationKind::BranchDelete,
        "Delete x".into(),
        1,
        None,
        false,
        vec![("refs/heads/x".into(), Some(oid(1)))],
    );
    assert!(matches!(
        plan_restore(top(&j), &RestoreFacts::default(), None, &never_exists),
        Err(UndoRefusal::NotRestorable { .. })
    ));
    assert!(!top(&j).restore_offer().restorable());
}

#[test]
fn ignore_lines_are_re_added_line_by_line() {
    // The entry removed "*.log" and "tmp/".
    assert_eq!(
        readd_lines("*.log\ntmp/\nkeep\n", "keep\n", "keep\nother\n").as_deref(),
        Some("keep\nother\n*.log\ntmp/\n")
    );
    // A line that is there already is not doubled; nothing to add: None.
    assert_eq!(readd_lines("*.log\n", "", "*.log\n"), None);
    // An entry that only added a line: re-add it when it is gone.
    assert_eq!(
        readd_lines("a\n", "a\n*.tmp\n", "a\n").as_deref(),
        Some("a\n*.tmp\n")
    );
    // A file without a final newline gets one before the appended line.
    assert_eq!(readd_lines("x\n", "", "y").as_deref(), Some("y\nx\n"));
}

#[test]
fn a_gitignore_entry_plans_re_added_lines_not_a_file_overwrite() {
    let mut j = Journal::default();
    let id = j.begin(
        OperationKind::AddToGitignore,
        "Ignore *.log".into(),
        1,
        None,
        false,
        vec![],
    );
    j.undo.last_mut().unwrap().files = Some(PathChange {
        paths: vec![".gitignore".into()],
        before: Snapshot {
            tree: oid(50),
            hashes: vec![file(".gitignore", Some(1), None)],
        },
        after: Snapshot {
            tree: oid(60),
            hashes: vec![file(".gitignore", Some(2), None)],
        },
    });
    j.finish(id, None, vec![], vec![], 50);
    assert_eq!(
        top(&j).restore_offer().items[0].kind,
        RestoreItemKind::Lines
    );
    let facts = now(vec![file(".gitignore", Some(7), None)]);
    let plan = plan_restore(top(&j), &facts, None, &never_exists).unwrap();
    assert!(
        matches!(&plan.actions[0], RestoreAction::ReAddLines { path, .. } if path == ".gitignore")
    );
}

#[test]
fn zero_oid_is_the_all_zero_id() {
    assert_eq!(ZERO_OID.to_hex(), "0".repeat(40));
}
