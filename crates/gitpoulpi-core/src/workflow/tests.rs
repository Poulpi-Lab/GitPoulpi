use super::*;

/// The example of the design plan, as written there.
const PLAN_TOML: &str = r#"
template = "git-flow"

[[base]]
name = "main"
protected = true
[[base]]
name = "develop"
names = ["develop", "dev"]
parent = "main"
merge = "merge"
tag_on_merge = false

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "develop"
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"
delete_after_finish = true
[[topic]]
id = "hotfix"
pattern = "hotfix/{version}"
parent = "main"
finish = [{ into = "main", merge = "merge", tag = "v{version}" }, { into = "develop", merge = "merge" }]

[hooks]
after_start = []
before_finish = ["cargo test"]

[issue_flow]
mode = "bot"
issue_title = "[{type}] {title}"
issue_labels = ["workflow::{type}"]
branch_from_issue = "{type}/{issue}-{slug}"
wait_for = "branch-or-mr"
"#;

#[test]
fn the_plans_toml_parses() {
    let w: Workflow = toml::from_str(PLAN_TOML).unwrap();
    assert_eq!(w.template, WorkflowTemplate::GitFlow);
    assert_eq!(w.bases.len(), 2);
    assert!(w.bases[0].protected);
    assert_eq!(w.bases[1].merge, Some(MergeStrategy::Merge));
    assert_eq!(w.topics[0].finish_mode, Some(FinishMode::MergeRequest));
    assert_eq!(w.topics[1].finish.len(), 2);
    assert_eq!(w.topics[1].finish[0].tag.as_deref(), Some("v{version}"));
    assert_eq!(w.hooks.before_finish, ["cargo test"]);
    assert_eq!(w.issue_flow.as_ref().unwrap().mode, IssueFlowMode::Bot);
    assert!(w.compile().warnings().is_empty());
    assert!(w.bases[1].is_named("dev"));
}

#[test]
fn every_template_round_trips_through_toml() {
    for t in WorkflowTemplate::ALL {
        let w = t.workflow();
        let text = toml::to_string_pretty(&w).unwrap();
        let back: Workflow = toml::from_str(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
        assert_eq!(back, w, "{t:?}\n{text}");
    }
}

#[test]
fn an_empty_file_is_an_empty_custom_workflow() {
    let w: Workflow = toml::from_str("").unwrap();
    assert_eq!(w, Workflow::default());
    assert!(w.is_empty());
}

#[test]
fn a_user_override_replaces_adds_and_disables() {
    let shared = WorkflowTemplate::GitFlow.workflow();
    let over = WorkflowOverride {
        bases: vec![BaseBranch {
            name: "develop".into(),
            names: vec!["integration".into()],
            parent: Some("main".into()),
            ..Default::default()
        }],
        topics: vec![
            TopicType {
                id: "feature".into(),
                pattern: "feat/{slug}".into(),
                parent: Some("develop".into()),
                ..Default::default()
            },
            TopicType {
                id: "spike".into(),
                pattern: "spike/{slug}".into(),
                ..Default::default()
            },
        ],
        disable_topics: vec!["support".into()],
        hooks: Some(Hooks {
            after_start: vec!["echo hi".into()],
            before_finish: vec![],
        }),
        ..Default::default()
    };
    let w = shared.merged_with(&over);
    assert_eq!(w.bases.len(), 2);
    assert!(w.bases[1].is_named("integration"));
    let ids: Vec<_> = w.topics.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["feature", "bugfix", "release", "hotfix", "spike"]);
    assert_eq!(w.topics[0].pattern, "feat/{slug}");
    assert_eq!(w.hooks.after_start, ["echo hi"]);
    assert_eq!(shared.merged_with(&WorkflowOverride::default()), shared);
    let w2 = shared.merged_with(&WorkflowOverride {
        disable_bases: vec!["dev".into()],
        ..Default::default()
    });
    assert_eq!(w2.bases.len(), 1, "a base is disabled by any of its names");
    assert!(WorkflowOverride::default().is_empty());
}
