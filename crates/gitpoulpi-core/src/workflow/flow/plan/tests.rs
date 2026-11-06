use std::collections::BTreeMap;

use super::*;
use crate::workflow::{FlowAction, ParentSource, WorkflowTemplate};

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn facts(branches: &[&str], head: &str) -> FlowFacts {
    FlowFacts {
        head: Some(head.to_string()),
        branches: branches
            .iter()
            .enumerate()
            .map(|(i, b)| (b.to_string(), oid(i as u8 + 1)))
            .collect(),
        remotes: vec!["origin".into()],
        user: Some("Ann".into()),
        today: "2026-10-16".into(),
        ..FlowFacts::default()
    }
}

fn plan(w: &Workflow, f: &FlowFacts, r: &FlowRequest) -> FlowPlan {
    plan_flow(w, f, r, &mut |_, _| Some((1, 0)))
}

fn ops(p: &FlowPlan) -> Vec<String> {
    p.steps
        .iter()
        .map(|s| {
            let text = format!("{s:?}");
            text.split([' ', '{'])
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}

fn git_flow() -> Workflow {
    WorkflowTemplate::GitFlow.workflow()
}

fn start(topic: &str, fields: &[(&str, &str)]) -> FlowRequest {
    FlowRequest {
        action: FlowAction::Start,
        topic: Some(topic.into()),
        fields: fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..FlowRequest::default()
    }
}

#[test]
fn a_start_names_the_branch_from_the_pattern_and_records_the_parent() {
    let f = facts(&["main", "develop"], "main");
    let p = plan(
        &git_flow(),
        &f,
        &start("feature", &[("slug", "Login Form!")]),
    );
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(p.branch.as_deref(), Some("feature/login-form"));
    assert_eq!(ops(&p), ["CreateBranch", "Checkout", "RecordParent"]);
    assert_eq!(
        p.steps[0],
        FlowStep::CreateBranch {
            name: "feature/login-form".into(),
            from: "develop".into()
        }
    );
    assert_eq!(
        p.steps[2],
        FlowStep::RecordParent {
            branch: "feature/login-form".into(),
            parent: Some("develop".into())
        }
    );
    assert!(p.undoable && p.irreversible.is_empty());
}

#[test]
fn user_and_date_tokens_default_and_a_missing_field_blocks() {
    let w = WorkflowTemplate::Trunk.workflow();
    let f = facts(&["main"], "main");
    let p = plan(&w, &f, &start("topic", &[("slug", "fix")]));
    assert_eq!(p.branch.as_deref(), Some("ann/fix"), "{:?}", p.blockers);
    let p = plan(&git_flow(), &f, &start("release", &[]));
    assert!(p
        .blockers
        .iter()
        .any(|b| b.contains("cannot name the branch")));
    assert!(plan(&git_flow(), &f, &start("ghost", &[])).blockers[0].contains("unknown"));
}

#[test]
fn a_start_from_a_commit_ish_records_the_types_parent_and_publish_is_irreversible() {
    let mut f = facts(&["main", "develop"], "develop");
    f.tags.insert("v1.0".into());
    let mut r = start("hotfix", &[("version", "1.0.1")]);
    r.from = Some("v1.0".into());
    r.publish = true;
    let p = plan(&git_flow(), &f, &r);
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(
        p.steps[2],
        FlowStep::RecordParent {
            branch: "hotfix/1.0.1".into(),
            parent: Some("main".into())
        }
    );
    assert!(matches!(
        p.steps[3],
        FlowStep::Push {
            set_upstream: true,
            ..
        }
    ));
    assert!(!p.undoable);
    assert!(p.irreversible[0].contains("Pushing"));
}

#[test]
fn hooks_are_listed_always_and_become_steps_only_when_asked() {
    let mut w = git_flow();
    w.hooks.after_start = vec!["echo hi".into()];
    let f = facts(&["main", "develop"], "main");
    let mut r = start("feature", &[("slug", "a")]);
    let p = plan(&w, &f, &r);
    assert_eq!(p.hooks.len(), 1);
    assert!(!p.hooks_run);
    assert!(!ops(&p).contains(&"Hook".to_string()));
    assert!(p.warnings.iter().any(|w| w.contains("hooks")));
    r.run_hooks = true;
    let p = plan(&w, &f, &r);
    assert!(p.hooks_run);
    assert_eq!(ops(&p).last().unwrap(), "Hook");
    assert!(!p.undoable);
}

fn finish(branch: Option<&str>) -> FlowRequest {
    FlowRequest {
        action: FlowAction::Finish,
        branch: branch.map(str::to_string),
        ..FlowRequest::default()
    }
}

#[test]
fn finishing_a_hotfix_merges_into_both_bases_tags_once_and_goes_back_to_the_parent() {
    let f = facts(&["main", "develop", "hotfix/1.0.1"], "hotfix/1.0.1");
    let p = plan(&git_flow(), &f, &finish(None));
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(
        ops(&p),
        [
            "Checkout",
            "Merge",
            "Tag",
            "Checkout",
            "Merge",
            "Checkout",
            "DeleteBranch"
        ]
    );
    assert_eq!(
        p.steps[2],
        FlowStep::Tag {
            name: "v1.0.1".into(),
            at: "main".into(),
            message: "Hotfix v1.0.1".into()
        }
    );
    // The merge into main always makes a merge commit (git-flow's --no-ff).
    assert!(matches!(
        &p.steps[1],
        FlowStep::Merge {
            strategy: MergeStrategy::Merge,
            no_ff: true,
            ..
        }
    ));
}

#[test]
fn an_existing_tag_and_a_target_behind_its_remote_block() {
    let mut f = facts(&["main", "develop", "release/2.0.0"], "release/2.0.0");
    f.tags.insert("v2.0.0".into());
    let p = plan(&git_flow(), &f, &finish(None));
    assert!(p
        .blockers
        .iter()
        .any(|b| b.contains("v2.0.0 already exists")));

    let mut f = facts(&["main", "develop", "feature/x"], "feature/x");
    f.upstreams
        .insert("develop".into(), "origin/develop".into());
    f.remote_branches.insert("origin/develop".into(), oid(9));
    let mut behind = |_: Oid, _: Oid| Some((0, 3));
    let p = plan_flow(&git_flow(), &f, &finish(None), &mut behind);
    assert!(p
        .blockers
        .iter()
        .any(|b| b.contains("3 commits behind origin/develop")));
    let mut r = finish(None);
    r.allow_behind = true;
    let p = plan_flow(&git_flow(), &f, &r, &mut behind);
    assert!(p.can_run());
}

#[test]
fn rebase_and_squash_strategies_make_their_own_steps() {
    let mut w = git_flow();
    w.topics[0].finish[0].merge = Some(MergeStrategy::Rebase);
    let f = facts(&["main", "develop", "feature/x"], "feature/x");
    let p = plan(&w, &f, &finish(None));
    assert_eq!(ops(&p), ["Rebase", "Checkout", "Merge", "DeleteBranch"]);
    assert!(matches!(
        &p.steps[2],
        FlowStep::Merge {
            strategy: MergeStrategy::FastForward,
            ..
        }
    ));
    w.topics[0].finish[0].merge = Some(MergeStrategy::Squash);
    let mut r = finish(None);
    r.title = Some("Add things".into());
    let p = plan(&w, &f, &r);
    assert!(matches!(
        &p.steps[1],
        FlowStep::Merge { strategy: MergeStrategy::Squash, message: Some(m), .. } if m == "Add things"
    ));
}

#[test]
fn merge_request_mode_pushes_and_leaves_the_request_to_the_client() {
    let w = WorkflowTemplate::GithubFlow.workflow();
    let f = facts(&["main", "feature/dark-mode"], "feature/dark-mode");
    let p = plan(&w, &f, &finish(None));
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(ops(&p), ["Push", "OpenMergeRequest"]);
    let spec = p.merge_request.clone().unwrap();
    assert_eq!(
        (
            spec.source.as_str(),
            spec.target.as_str(),
            spec.title.as_str()
        ),
        ("feature/dark-mode", "main", "Dark mode")
    );
    assert!(!p.undoable);
    // `ask` needs a choice; asking for the mode makes it a plan.
    let mut r = finish(None);
    r.finish_mode = Some(FinishMode::LocalMerge);
    let p = plan(&w, &f, &r);
    assert_eq!(ops(&p)[0], "Checkout");
    let mut ask = git_flow();
    ask.topics[0].finish_mode = Some(FinishMode::Ask);
    let f = facts(&["main", "develop", "feature/x"], "feature/x");
    let p = plan(&ask, &f, &finish(None));
    assert!(p.blockers.iter().any(|b| b.contains("--mode")));
}

#[test]
fn a_recorded_parent_redirects_the_first_finish_target() {
    let mut f = facts(
        &["main", "develop", "release/1.0.0", "feature/x"],
        "feature/x",
    );
    f.recorded.insert(
        "feature/x".into(),
        ("release/1.0.0".into(), ParentSource::Recorded),
    );
    let p = plan(&git_flow(), &f, &finish(None));
    assert_eq!(
        p.steps[0],
        FlowStep::Checkout {
            branch: "release/1.0.0".into()
        }
    );
    assert!(p.steps.iter().any(|s| matches!(
        s,
        FlowStep::Merge { into, .. } if into == "release/1.0.0"
    )));
}

#[test]
fn finish_refuses_what_it_cannot_finish_and_warns_about_a_stack() {
    let f = facts(&["main", "develop", "feature/a", "feature/b"], "main");
    assert!(plan(&git_flow(), &f, &finish(None)).blockers[0].contains("base branch"));
    assert!(plan(&git_flow(), &f, &finish(Some("ghost"))).blockers[0].contains("no local branch"));
    let mut f = f;
    f.recorded.insert(
        "feature/b".into(),
        ("feature/a".into(), ParentSource::Recorded),
    );
    let p = plan(&git_flow(), &f, &finish(Some("feature/a")));
    assert!(
        p.warnings.iter().any(|w| w.contains("feature/b")),
        "{:?}",
        p.warnings
    );
    // No workflow at all.
    let p = plan(&Workflow::default(), &f, &finish(None));
    assert!(p.blockers[0].contains("no branching workflow"));
    // An operation in progress.
    let mut busy = facts(&["main", "develop", "feature/a"], "feature/a");
    busy.in_progress = Some("merge".into());
    assert!(plan(&git_flow(), &busy, &finish(None))
        .blockers
        .iter()
        .any(|b| b.contains("merge is in progress")));
}

fn sync(branch: &str, stack: bool, method: Option<SyncMethod>) -> FlowRequest {
    FlowRequest {
        action: FlowAction::Sync,
        branch: Some(branch.into()),
        stack,
        method,
        ..FlowRequest::default()
    }
}

#[test]
fn sync_updates_the_branch_then_its_stack_in_order_and_returns() {
    let mut f = facts(
        &["main", "develop", "feature/a", "feature/b", "feature/c"],
        "main",
    );
    f.recorded.insert(
        "feature/b".into(),
        ("feature/a".into(), ParentSource::Recorded),
    );
    f.recorded.insert(
        "feature/c".into(),
        ("feature/b".into(), ParentSource::Recorded),
    );
    let mut behind = |_: Oid, _: Oid| Some((1, 2));
    let p = plan_flow(&git_flow(), &f, &sync("feature/a", true, None), &mut behind);
    assert!(p.can_run(), "{:?}", p.blockers);
    let merged: Vec<(String, String)> = p
        .steps
        .iter()
        .filter_map(|s| match s {
            FlowStep::Merge { branch, into, .. } => Some((branch.clone(), into.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        merged,
        [
            ("develop".to_string(), "feature/a".to_string()),
            ("feature/a".to_string(), "feature/b".to_string()),
            ("feature/b".to_string(), "feature/c".to_string()),
        ]
    );
    assert_eq!(
        p.steps.last().unwrap(),
        &FlowStep::Checkout {
            branch: "main".into()
        },
        "back where it started"
    );
    // Rebase by request.
    let p = plan_flow(
        &git_flow(),
        &f,
        &sync("feature/a", false, Some(SyncMethod::Rebase)),
        &mut behind,
    );
    assert_eq!(ops(&p)[0], "Rebase");
    // Up to date: nothing to do.
    let mut uptodate = |_: Oid, _: Oid| Some((1, 0));
    let p = plan_flow(
        &git_flow(),
        &f,
        &sync("feature/a", false, None),
        &mut uptodate,
    );
    assert!(p.steps.is_empty());
    assert!(p.warnings.iter().any(|w| w.contains("already up to date")));
}

#[test]
fn promote_merges_the_parent_into_the_environment_branch() {
    let w = WorkflowTemplate::GitlabFlow.workflow();
    let f = facts(&["main", "pre-production", "production"], "main");
    let r = FlowRequest {
        action: FlowAction::Promote,
        branch: Some("production".into()),
        ..FlowRequest::default()
    };
    let p = plan(&w, &f, &r);
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(ops(&p), ["Checkout", "Merge", "Checkout"]);
    assert!(matches!(
        &p.steps[1],
        FlowStep::Merge { branch, into, .. } if branch == "pre-production" && into == "production"
    ));
    let none = FlowRequest {
        action: FlowAction::Promote,
        ..FlowRequest::default()
    };
    assert!(plan(&w, &f, &none).blockers[0].contains("environment branch"));
}

#[test]
fn set_parent_validates_and_refuses_a_cycle() {
    let mut f = facts(&["main", "develop", "feature/a", "feature/b"], "main");
    f.recorded.insert(
        "feature/b".into(),
        ("feature/a".into(), ParentSource::Recorded),
    );
    let set = |branch: &str, parent: &str| FlowRequest {
        action: FlowAction::SetParent,
        branch: Some(branch.into()),
        parent: Some(parent.into()),
        ..FlowRequest::default()
    };
    let p = plan(&git_flow(), &f, &set("feature/a", "develop"));
    assert_eq!(ops(&p), ["RecordParent"]);
    assert!(plan(&git_flow(), &f, &set("feature/a", "feature/b")).blockers[0].contains("cycle"));
    assert!(
        plan(&git_flow(), &f, &set("feature/a", "feature/a")).blockers[0].contains("own parent")
    );
    assert!(
        plan(&git_flow(), &f, &set("feature/a", "ghost")).blockers[0].contains("no local branch")
    );
    let clear = FlowRequest {
        action: FlowAction::SetParent,
        branch: Some("feature/b".into()),
        clear: true,
        ..FlowRequest::default()
    };
    assert_eq!(
        plan(&git_flow(), &f, &clear).steps,
        [FlowStep::RecordParent {
            branch: "feature/b".into(),
            parent: None
        }]
    );
}

#[test]
fn titles_read_like_words_with_the_issue() {
    let mut fields = BTreeMap::new();
    fields.insert("slug".to_string(), "fix-login-bug".to_string());
    assert_eq!(
        branch_title(&fields, "feature/fix-login-bug"),
        "Fix login bug"
    );
    fields.insert("issue".to_string(), "#12".to_string());
    assert_eq!(
        branch_title(&fields, "feature/12-fix-login-bug"),
        "Fix login bug (#12)"
    );
    fields.insert("issue_key".to_string(), "PROJ-9".to_string());
    assert_eq!(branch_title(&fields, "x"), "PROJ-9: Fix login bug");
    assert_eq!(branch_title(&BTreeMap::new(), "release/1.0"), "release/1.0");
}

#[test]
fn every_step_describes_itself_and_names_its_git_commands() {
    let steps = [
        FlowStep::Checkout { branch: "a".into() },
        FlowStep::Merge {
            branch: "a".into(),
            into: "b".into(),
            strategy: MergeStrategy::Squash,
            no_ff: false,
            message: Some("m".into()),
        },
        FlowStep::Tag {
            name: "v1".into(),
            at: "b".into(),
            message: "t".into(),
        },
    ];
    assert_eq!(steps[0].git(), ["git checkout a"]);
    assert_eq!(steps[1].git().len(), 2);
    assert!(steps[2].describe().contains("v1"));
    // The wire shape W3 builds on.
    assert_eq!(serde_json::to_value(&steps[0]).unwrap()["op"], "checkout");
}

fn pick_up(branch: &str, parent: Option<&str>) -> FlowRequest {
    FlowRequest {
        action: FlowAction::PickUp,
        branch: Some(branch.into()),
        parent: parent.map(String::from),
        ..FlowRequest::default()
    }
}

#[test]
fn a_pick_up_tracks_the_remote_branch_and_records_the_merge_request_target() {
    let mut f = facts(&["main", "develop"], "main");
    f.remote_branches
        .insert("origin/feature/12-login".into(), oid(9));
    let p = plan(
        &git_flow(),
        &f,
        &pick_up("feature/12-login", Some("develop")),
    );
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(
        ops(&p),
        ["CreateBranch", "SetUpstream", "Checkout", "RecordParent"]
    );
    assert_eq!(
        p.steps[1],
        FlowStep::SetUpstream {
            branch: "feature/12-login".into(),
            upstream: "origin/feature/12-login".into()
        }
    );
    assert_eq!(
        p.steps[3],
        FlowStep::RecordParent {
            branch: "feature/12-login".into(),
            parent: Some("develop".into())
        }
    );
    assert!(p.undoable);
}

#[test]
fn a_pick_up_of_an_unfetched_branch_is_blocked_and_one_already_local_is_only_checked_out() {
    let f = facts(&["main", "develop"], "main");
    let p = plan(&git_flow(), &f, &pick_up("feature/9-x", None));
    assert!(!p.can_run());
    assert!(p.blockers[0].contains("fetch"), "{:?}", p.blockers);

    let f = facts(&["main", "develop", "feature/9-x"], "main");
    let p = plan(&git_flow(), &f, &pick_up("feature/9-x", None));
    assert!(p.can_run());
    // The type's parent (develop) is recorded when no target is given.
    assert_eq!(ops(&p), ["Checkout", "RecordParent"]);
    assert!(!p.warnings.is_empty());
}

#[test]
fn a_pick_up_needs_no_workflow() {
    let mut f = facts(&["main"], "main");
    f.remote_branches.insert("origin/bot-1".into(), oid(9));
    let p = plan(&Workflow::default(), &f, &pick_up("bot-1", Some("main")));
    assert!(p.can_run(), "{:?}", p.blockers);
    assert_eq!(
        ops(&p),
        ["CreateBranch", "SetUpstream", "Checkout", "RecordParent"]
    );
}

#[test]
fn hook_steps_carry_whether_the_hooks_are_the_users_own() {
    let mut w = git_flow();
    w.hooks.after_start = vec!["echo hi".into()];
    let mut f = facts(&["main", "develop"], "main");
    let mut r = start("feature", &[("slug", "a")]);
    r.run_hooks = true;
    let p = plan(&w, &f, &r);
    assert!(matches!(
        p.steps.last(),
        Some(FlowStep::Hook {
            personal: false,
            ..
        })
    ));
    assert!(!p.hooks[0].personal);
    f.hooks_personal = true;
    let p = plan(&w, &f, &r);
    assert!(matches!(
        p.steps.last(),
        Some(FlowStep::Hook { personal: true, .. })
    ));
    assert!(p.hooks[0].personal);
    // A saved state from before the field existed is not personal.
    let old: FlowStep =
        serde_json::from_str(r#"{"op":"hook","phase":"after_start","command":"x"}"#).unwrap();
    assert!(matches!(
        old,
        FlowStep::Hook {
            personal: false,
            ..
        }
    ));
}
