//! The workflow templates and the name presets the editor offers.

use serde::{Deserialize, Serialize};

use super::{BaseBranch, FinishMode, FinishTarget, MergeStrategy, TopicType, Workflow};

/// The base of a [`Workflow`] (informative: the definition is what counts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkflowTemplate {
    #[serde(rename = "git-flow")]
    GitFlow,
    #[serde(rename = "github-flow")]
    GithubFlow,
    #[serde(rename = "gitlab-flow")]
    GitlabFlow,
    #[serde(rename = "trunk")]
    Trunk,
    #[default]
    #[serde(rename = "custom", other)]
    Custom,
}

/// Names the editor offers, as groups of alternatives: long-lived branches,
/// then topic prefixes.
pub const NAME_PRESETS: &[&[&str]] = &[
    &["main", "master"],
    &["develop", "dev"],
    &["staging", "pre-production"],
    &["production", "prod"],
    &["feature/", "features/"],
    &["bugfix/", "bugfixes/"],
    &["hotfix/", "hotfixes/"],
    &["release/", "releases/"],
    &["support/"],
];

impl WorkflowTemplate {
    /// Every template, in the order detection breaks ties.
    pub const ALL: [WorkflowTemplate; 5] = [
        WorkflowTemplate::GitFlow,
        WorkflowTemplate::GithubFlow,
        WorkflowTemplate::GitlabFlow,
        WorkflowTemplate::Trunk,
        WorkflowTemplate::Custom,
    ];

    /// The id used in files and on the command line.
    pub fn id(self) -> &'static str {
        match self {
            WorkflowTemplate::GitFlow => "git-flow",
            WorkflowTemplate::GithubFlow => "github-flow",
            WorkflowTemplate::GitlabFlow => "gitlab-flow",
            WorkflowTemplate::Trunk => "trunk",
            WorkflowTemplate::Custom => "custom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WorkflowTemplate::GitFlow => "Git Flow",
            WorkflowTemplate::GithubFlow => "GitHub Flow",
            WorkflowTemplate::GitlabFlow => "GitLab Flow",
            WorkflowTemplate::Trunk => "Trunk-based",
            WorkflowTemplate::Custom => "Custom",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            WorkflowTemplate::GitFlow => {
                "main + develop; feature, bugfix, release, hotfix and support branches"
            }
            WorkflowTemplate::GithubFlow => {
                "main only; short-lived branches merged by pull request"
            }
            WorkflowTemplate::GitlabFlow => {
                "main plus environment branches (pre-production, production) promoted downstream"
            }
            WorkflowTemplate::Trunk => {
                "main only; very short-lived <user>/<slug> branches, rebased"
            }
            WorkflowTemplate::Custom => "nothing predefined: define your own bases and topic types",
        }
    }

    pub fn from_id(id: &str) -> Option<WorkflowTemplate> {
        let id = id.trim().to_ascii_lowercase();
        WorkflowTemplate::ALL.into_iter().find(|t| t.id() == id)
    }

    /// The preset definition (editable afterwards).
    pub fn workflow(self) -> Workflow {
        match self {
            WorkflowTemplate::GitFlow => git_flow(),
            WorkflowTemplate::GithubFlow => github_flow(),
            WorkflowTemplate::GitlabFlow => gitlab_flow(),
            WorkflowTemplate::Trunk => trunk(),
            WorkflowTemplate::Custom => Workflow {
                template: WorkflowTemplate::Custom,
                ..Default::default()
            },
        }
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn main_base() -> BaseBranch {
    BaseBranch {
        name: "main".into(),
        names: names(&["master"]),
        protected: true,
        ..Default::default()
    }
}

fn target(into: &str, merge: MergeStrategy, tag: Option<&str>) -> FinishTarget {
    FinishTarget {
        into: into.into(),
        merge: Some(merge),
        tag: tag.map(str::to_string),
    }
}

#[allow(clippy::too_many_arguments)]
fn topic(
    id: &str,
    label: &str,
    prefixes: &[&str],
    leaf: &str,
    parent: &str,
    finish: Vec<FinishTarget>,
    mode: FinishMode,
    delete: bool,
) -> TopicType {
    let mut patterns = prefixes.iter().map(|p| format!("{p}{{{leaf}}}"));
    TopicType {
        id: id.into(),
        label: label.into(),
        pattern: patterns.next().unwrap_or_default(),
        patterns: patterns.collect(),
        regex: None,
        parent: Some(parent.into()),
        finish,
        finish_mode: Some(mode),
        delete_after_finish: delete,
    }
}

fn git_flow() -> Workflow {
    use MergeStrategy::Merge;
    let both = |tag: bool| {
        vec![
            target("main", Merge, tag.then_some("v{version}")),
            target("develop", Merge, None),
        ]
    };
    let mut hotfix = topic(
        "hotfix",
        "Hotfix",
        &["hotfix/"],
        "version",
        "main",
        both(true),
        FinishMode::LocalMerge,
        true,
    );
    // Hotfix branches are often named after the problem, not the version.
    hotfix.patterns = names(&["hotfix/{slug}", "hotfixes/{version}", "hotfixes/{slug}"]);
    Workflow {
        template: WorkflowTemplate::GitFlow,
        bases: vec![
            main_base(),
            BaseBranch {
                name: "develop".into(),
                names: names(&["dev"]),
                parent: Some("main".into()),
                merge: Some(Merge),
                ..Default::default()
            },
        ],
        topics: vec![
            topic(
                "feature",
                "Feature",
                &["feature/", "features/"],
                "slug",
                "develop",
                vec![target("develop", Merge, None)],
                FinishMode::LocalMerge,
                true,
            ),
            topic(
                "bugfix",
                "Bugfix",
                &["bugfix/", "bugfixes/"],
                "slug",
                "develop",
                vec![target("develop", Merge, None)],
                FinishMode::LocalMerge,
                true,
            ),
            topic(
                "release",
                "Release",
                &["release/", "releases/"],
                "version",
                "develop",
                both(true),
                FinishMode::LocalMerge,
                true,
            ),
            hotfix,
            topic(
                "support",
                "Support",
                &["support/"],
                "slug",
                "main",
                Vec::new(),
                FinishMode::Ask,
                false,
            ),
        ],
        ..Default::default()
    }
}

fn github_flow() -> Workflow {
    let mut feature = topic(
        "feature",
        "Feature",
        &["feature/", "features/"],
        "slug",
        "main",
        vec![target("main", MergeStrategy::Squash, None)],
        FinishMode::MergeRequest,
        true,
    );
    // GitHub Flow: any name will do.
    feature.patterns.push("{slug}".into());
    Workflow {
        template: WorkflowTemplate::GithubFlow,
        bases: vec![main_base()],
        topics: vec![feature],
        ..Default::default()
    }
}

fn gitlab_flow() -> Workflow {
    let mut feature = topic(
        "feature",
        "Feature",
        &["feature/", "features/"],
        "slug",
        "main",
        vec![target("main", MergeStrategy::Merge, None)],
        FinishMode::MergeRequest,
        true,
    );
    feature.patterns.push("{slug}".into());
    Workflow {
        template: WorkflowTemplate::GitlabFlow,
        bases: vec![
            main_base(),
            BaseBranch {
                name: "pre-production".into(),
                names: names(&["preprod", "staging"]),
                parent: Some("main".into()),
                protected: true,
                merge: Some(MergeStrategy::Merge),
                environment: true,
                ..Default::default()
            },
            BaseBranch {
                name: "production".into(),
                names: names(&["prod"]),
                parent: Some("pre-production".into()),
                protected: true,
                merge: Some(MergeStrategy::Merge),
                environment: true,
                ..Default::default()
            },
        ],
        topics: vec![
            feature,
            // Stable-branch style releases: a cherry-pick target.
            topic(
                "release",
                "Release",
                &["release/", "releases/"],
                "version",
                "main",
                Vec::new(),
                FinishMode::Ask,
                false,
            ),
        ],
        ..Default::default()
    }
}

fn trunk() -> Workflow {
    Workflow {
        template: WorkflowTemplate::Trunk,
        bases: vec![main_base()],
        topics: vec![TopicType {
            id: "topic".into(),
            label: "Short-lived branch".into(),
            pattern: "{user}/{slug}".into(),
            parent: Some("main".into()),
            finish: vec![target("main", MergeStrategy::Rebase, None)],
            finish_mode: Some(FinishMode::LocalMerge),
            delete_after_finish: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::super::classify::Classification;
    use super::*;

    fn kind(w: &Workflow, b: &str) -> String {
        match w.classify(b) {
            Classification::Base { name } => format!("base:{name}"),
            Classification::Topic { type_id, .. } => format!("topic:{type_id}"),
            Classification::Unclassified => "-".into(),
        }
    }

    #[test]
    fn every_template_compiles_without_warnings_and_round_trips_its_id() {
        for t in WorkflowTemplate::ALL {
            let w = t.workflow();
            assert!(
                w.compile().warnings().is_empty(),
                "{t:?}: {:?}",
                w.compile().warnings()
            );
            assert_eq!(WorkflowTemplate::from_id(t.id()), Some(t));
            assert_eq!(w.template, t);
        }
        assert_eq!(
            WorkflowTemplate::from_id(" Git-Flow "),
            Some(WorkflowTemplate::GitFlow)
        );
        assert_eq!(WorkflowTemplate::from_id("nope"), None);
    }

    #[test]
    fn git_flow_classifies_a_typical_branch_set() {
        let w = WorkflowTemplate::GitFlow.workflow();
        let cases = [
            ("main", "base:main"),
            ("master", "base:main"),
            ("develop", "base:develop"),
            ("dev", "base:develop"),
            ("feature/login", "topic:feature"),
            ("features/login", "topic:feature"),
            ("bugfix/crash", "topic:bugfix"),
            ("release/1.2.0", "topic:release"),
            ("releases/v2.0", "topic:release"),
            ("hotfix/1.2.1", "topic:hotfix"),
            ("hotfix/fix-crash", "topic:hotfix"),
            ("hotfixes/fix-crash", "topic:hotfix"),
            ("support/1.x", "topic:support"),
            ("release/next", "-"),
            ("random", "-"),
            ("user/thing", "-"),
        ];
        for (b, want) in cases {
            assert_eq!(kind(&w, b), want, "{b}");
        }
    }

    #[test]
    fn git_flow_release_and_hotfix_tag_and_finish_into_both() {
        let w = WorkflowTemplate::GitFlow.workflow();
        for id in ["release", "hotfix"] {
            let t = w.topics.iter().find(|t| t.id == id).unwrap();
            assert_eq!(t.finish.len(), 2, "{id}");
            assert_eq!(t.finish[0].into, "main");
            assert_eq!(t.finish[0].tag.as_deref(), Some("v{version}"));
            assert_eq!(t.finish[1].into, "develop");
        }
        assert_eq!(w.topics[0].finish_mode, Some(FinishMode::LocalMerge));
    }

    #[test]
    fn github_flow_takes_any_name_as_a_topic() {
        let w = WorkflowTemplate::GithubFlow.workflow();
        for (b, want) in [
            ("main", "base:main"),
            ("master", "base:main"),
            ("feature/x", "topic:feature"),
            ("anything-goes", "topic:feature"),
            ("a/b/c", "topic:feature"),
        ] {
            assert_eq!(kind(&w, b), want, "{b}");
        }
        let f = &w.topics[0];
        assert_eq!(f.finish_mode, Some(FinishMode::MergeRequest));
        assert_eq!(f.finish[0].merge, Some(MergeStrategy::Squash));
        assert!(f.delete_after_finish);
    }

    #[test]
    fn gitlab_flow_chains_environment_branches() {
        let w = WorkflowTemplate::GitlabFlow.workflow();
        let parent_of = |n: &str| {
            w.bases
                .iter()
                .find(|b| b.is_named(n))
                .and_then(|b| b.parent.clone())
        };
        assert_eq!(parent_of("main"), None);
        assert_eq!(parent_of("pre-production").as_deref(), Some("main"));
        assert_eq!(parent_of("production").as_deref(), Some("pre-production"));
        assert!(w.bases[1].environment && w.bases[2].environment && !w.bases[0].environment);
        for (b, want) in [
            ("main", "base:main"),
            ("pre-production", "base:pre-production"),
            ("staging", "base:pre-production"),
            ("production", "base:production"),
            ("feature/a", "topic:feature"),
            ("release/1.0", "topic:release"),
            ("my-fix", "topic:feature"),
        ] {
            assert_eq!(kind(&w, b), want, "{b}");
        }
    }

    #[test]
    fn trunk_has_user_prefixed_short_lived_branches() {
        let w = WorkflowTemplate::Trunk.workflow();
        assert_eq!(kind(&w, "main"), "base:main");
        assert_eq!(kind(&w, "ana/fix-login"), "topic:topic");
        assert_eq!(kind(&w, "plain"), "-");
        assert_eq!(w.topics[0].finish[0].merge, Some(MergeStrategy::Rebase));
    }

    #[test]
    fn custom_is_empty() {
        let w = WorkflowTemplate::Custom.workflow();
        assert!(w.is_empty());
        assert_eq!(kind(&w, "main"), "-");
    }

    #[test]
    fn every_preset_name_is_a_valid_branch_name_or_prefix() {
        for group in NAME_PRESETS {
            for n in *group {
                let probe = format!("{n}x");
                assert!(super::super::check_ref_format(&probe).is_ok(), "{n}");
            }
        }
    }

    #[test]
    fn unknown_template_ids_in_a_file_fall_back_to_custom() {
        #[derive(Deserialize)]
        struct W {
            template: WorkflowTemplate,
        }
        let w: W = toml::from_str("template = \"something-new\"").unwrap();
        assert_eq!(w.template, WorkflowTemplate::Custom);
        let w: W = toml::from_str("template = \"git-flow\"").unwrap();
        assert_eq!(w.template, WorkflowTemplate::GitFlow);
    }
}
