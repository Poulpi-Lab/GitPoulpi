//! Reading other tools' branching configuration (read-only, offered, never
//! silent): the classic `git flow init` settings and Git Town's.
//!
//! Inputs are `(key, value)` pairs of the repository-local git config, as
//! `git config --local --list` prints them (section and variable lower-cased,
//! subsection as written).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::classify::ParentSource;
use super::templates::WorkflowTemplate;
use super::{BaseBranch, FinishMode, FinishTarget, MergeStrategy, TopicType, Workflow};

/// The git config variable GitPoulpi records a branch's parent in
/// (`branch.<name>.gitpoulpi-parent`).
pub const RECORDED_PARENT_VARIABLE: &str = "gitpoulpi-parent";

/// Key prefixes the workflow reads from the repository config: what a
/// backend lists for [`import_git_flow`], [`import_git_town`] and
/// [`parse_recorded_parents`].
pub const WORKFLOW_CONFIG_PREFIXES: &[&str] =
    &["gitflow.", "git-town.", "git-town-branch.", "branch."];

/// A workflow read from another tool's configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imported {
    /// `git-flow` or `git-town`.
    pub source: String,
    pub workflow: Workflow,
    /// What could not be carried over, or was assumed.
    pub notes: Vec<String>,
}

type Entries<'a> = &'a [(String, String)];

/// The last value of `key` (git's own precedence for single-valued keys).
fn get<'a>(entries: Entries<'a>, key: &str) -> Option<&'a str> {
    entries
        .iter()
        .rev()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.trim().is_empty())
}

fn target(into: &str, merge: MergeStrategy, tag: Option<String>) -> FinishTarget {
    FinishTarget {
        into: into.into(),
        merge: Some(merge),
        tag,
    }
}

/// `git flow init`'s settings (`gitflow.branch.*`, `gitflow.prefix.*`).
/// `None` when the repository has none.
pub fn import_git_flow(entries: Entries) -> Option<Imported> {
    if !entries
        .iter()
        .any(|(k, _)| k.to_ascii_lowercase().starts_with("gitflow."))
    {
        return None;
    }
    let master = get(entries, "gitflow.branch.master").unwrap_or("master");
    let develop = get(entries, "gitflow.branch.develop").unwrap_or("develop");
    let prefix = |kind: &str, default: &str| {
        get(entries, &format!("gitflow.prefix.{kind}"))
            .unwrap_or(default)
            .to_string()
    };
    let versiontag = get(entries, "gitflow.prefix.versiontag").unwrap_or("");
    let tag = Some(format!("{versiontag}{{version}}"));
    let mut notes = Vec::new();

    let both = || {
        vec![
            target(master, MergeStrategy::Merge, tag.clone()),
            target(develop, MergeStrategy::Merge, None),
        ]
    };
    let topic = |id: &str,
                 label: &str,
                 pre: String,
                 leaf: &str,
                 parent: &str,
                 finish: Vec<FinishTarget>| TopicType {
        id: id.into(),
        label: label.into(),
        pattern: format!("{pre}{{{leaf}}}"),
        parent: Some(parent.into()),
        finish,
        finish_mode: Some(FinishMode::LocalMerge),
        delete_after_finish: true,
        ..Default::default()
    };
    let mut hotfix = topic(
        "hotfix",
        "Hotfix",
        prefix("hotfix", "hotfix/"),
        "version",
        master,
        both(),
    );
    hotfix.patterns = vec![format!("{}{{slug}}", prefix("hotfix", "hotfix/"))];
    let mut support = topic(
        "support",
        "Support",
        prefix("support", "support/"),
        "slug",
        master,
        Vec::new(),
    );
    support.finish_mode = Some(FinishMode::Ask);
    support.delete_after_finish = false;

    let workflow = Workflow {
        template: WorkflowTemplate::GitFlow,
        bases: vec![
            BaseBranch {
                name: master.into(),
                protected: true,
                ..Default::default()
            },
            BaseBranch {
                name: develop.into(),
                parent: Some(master.into()),
                merge: Some(MergeStrategy::Merge),
                ..Default::default()
            },
        ],
        topics: vec![
            topic(
                "feature",
                "Feature",
                prefix("feature", "feature/"),
                "slug",
                develop,
                vec![target(develop, MergeStrategy::Merge, None)],
            ),
            topic(
                "bugfix",
                "Bugfix",
                prefix("bugfix", "bugfix/"),
                "slug",
                develop,
                vec![target(develop, MergeStrategy::Merge, None)],
            ),
            topic(
                "release",
                "Release",
                prefix("release", "release/"),
                "version",
                develop,
                both(),
            ),
            hotfix,
            support,
        ],
        ..Default::default()
    };
    if get(entries, "gitflow.branch.master").is_none() {
        notes.push("gitflow.branch.master is not set: assumed `master`".to_string());
    }
    if get(entries, "gitflow.branch.develop").is_none() {
        notes.push("gitflow.branch.develop is not set: assumed `develop`".to_string());
    }
    Some(Imported {
        source: "git-flow".to_string(),
        workflow,
        notes,
    })
}

/// Git Town's settings (`git-town.main-branch`, `git-town.perennial-branches`).
/// `None` when no main branch is configured. Each branch's own parent
/// (`git-town-branch.<b>.parent`) is not part of the workflow: it is a
/// recorded parent, see [`parse_recorded_parents`].
pub fn import_git_town(entries: Entries) -> Option<Imported> {
    let main = get(entries, "git-town.main-branch")?;
    let mut notes = Vec::new();
    let mut bases = vec![BaseBranch {
        name: main.into(),
        protected: true,
        ..Default::default()
    }];
    for p in get(entries, "git-town.perennial-branches")
        .unwrap_or("")
        .split_whitespace()
        .filter(|p| *p != main)
    {
        bases.push(BaseBranch {
            name: p.into(),
            protected: true,
            ..Default::default()
        });
    }
    if get(entries, "git-town.perennial-regex").is_some() {
        notes.push(
            "git-town.perennial-regex is not imported: add those branches as bases by hand"
                .to_string(),
        );
    }
    notes.push(
        "Git Town branches take their parent from git-town-branch.<branch>.parent".to_string(),
    );
    let workflow = Workflow {
        template: WorkflowTemplate::Custom,
        bases,
        topics: vec![TopicType {
            id: "feature".into(),
            label: "Feature branch".into(),
            pattern: "{slug}".into(),
            parent: Some(main.into()),
            finish: vec![target(main, MergeStrategy::Squash, None)],
            finish_mode: Some(FinishMode::Ask),
            delete_after_finish: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    Some(Imported {
        source: "git-town".to_string(),
        workflow,
        notes,
    })
}

/// The recorded parent of each branch: GitPoulpi's own
/// `branch.<b>.gitpoulpi-parent`, else Git Town's `git-town-branch.<b>.parent`.
pub fn parse_recorded_parents(entries: Entries) -> BTreeMap<String, (String, ParentSource)> {
    let mut out: BTreeMap<String, (String, ParentSource)> = BTreeMap::new();
    let suffix = format!(".{RECORDED_PARENT_VARIABLE}");
    for (key, value) in entries {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let lower = key.to_ascii_lowercase();
        // The subsection (the branch name) keeps its case.
        if lower.starts_with("branch.") && lower.ends_with(&suffix) && key.len() > 7 + suffix.len()
        {
            let name = &key[7..key.len() - suffix.len()];
            out.insert(
                name.to_string(),
                (value.to_string(), ParentSource::Recorded),
            );
        } else if lower.starts_with("git-town-branch.")
            && lower.ends_with(".parent")
            && key.len() > 16 + 7
        {
            let name = &key[16..key.len() - 7];
            out.entry(name.to_string())
                .or_insert_with(|| (value.to_string(), ParentSource::GitTown));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::classify::Classification;
    use super::*;

    fn e(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn kind(w: &Workflow, b: &str) -> String {
        match w.classify(b) {
            Classification::Base { name } => format!("base:{name}"),
            Classification::Topic { type_id, .. } => type_id,
            Classification::Unclassified => "-".into(),
        }
    }

    #[test]
    fn nothing_to_import_without_the_keys() {
        assert_eq!(import_git_flow(&e(&[("core.bare", "false")])), None);
        assert_eq!(import_git_town(&e(&[("core.bare", "false")])), None);
        assert_eq!(
            import_git_town(&e(&[("git-town.perennial-branches", "dev")])),
            None
        );
    }

    #[test]
    fn git_flow_defaults_when_only_some_keys_exist() {
        let i = import_git_flow(&e(&[("gitflow.branch.master", "main")])).unwrap();
        assert_eq!(i.source, "git-flow");
        let w = i.workflow;
        assert!(
            w.compile().warnings().is_empty(),
            "{:?}",
            w.compile().warnings()
        );
        assert_eq!(kind(&w, "main"), "base:main");
        assert_eq!(kind(&w, "develop"), "base:develop");
        assert_eq!(kind(&w, "feature/x"), "feature");
        assert!(i.notes.iter().any(|n| n.contains("develop")));
    }

    #[test]
    fn git_flow_custom_prefixes_and_version_tag() {
        let i = import_git_flow(&e(&[
            ("gitflow.branch.master", "prod"),
            ("gitflow.branch.develop", "next"),
            ("gitflow.prefix.feature", "feat-"),
            ("gitflow.prefix.bugfix", "fix-"),
            ("gitflow.prefix.release", "rel/"),
            ("gitflow.prefix.hotfix", "hot/"),
            ("gitflow.prefix.support", "sup/"),
            ("gitflow.prefix.versiontag", "v"),
        ]))
        .unwrap();
        let w = i.workflow;
        assert!(i.notes.is_empty());
        assert_eq!(w.compile().warnings(), Vec::<String>::new());
        for (b, want) in [
            ("prod", "base:prod"),
            ("next", "base:next"),
            ("feat-login", "feature"),
            ("fix-x", "bugfix"),
            ("rel/1.0", "release"),
            ("hot/1.0.1", "hotfix"),
            ("hot/fast", "hotfix"),
            ("sup/1.x", "support"),
            ("feature/x", "-"),
        ] {
            assert_eq!(kind(&w, b), want, "{b}");
        }
        let rel = w.topics.iter().find(|t| t.id == "release").unwrap();
        assert_eq!(rel.finish[0].into, "prod");
        assert_eq!(rel.finish[0].tag.as_deref(), Some("v{version}"));
        assert_eq!(rel.finish[1].into, "next");
        let feature = &w.topics[0];
        assert_eq!(feature.parent.as_deref(), Some("next"));
    }

    #[test]
    fn git_flow_empty_prefix_values_fall_back_to_defaults() {
        let i = import_git_flow(&e(&[("gitflow.prefix.feature", "  ")])).unwrap();
        assert_eq!(kind(&i.workflow, "feature/x"), "feature");
    }

    #[test]
    fn git_town_main_and_perennials() {
        let i = import_git_town(&e(&[
            ("git-town.main-branch", "main"),
            ("git-town.perennial-branches", "develop  staging main"),
            ("git-town.perennial-regex", "^release-"),
        ]))
        .unwrap();
        assert_eq!(i.source, "git-town");
        let names: Vec<_> = i.workflow.bases.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["main", "develop", "staging"]);
        assert!(i.notes.iter().any(|n| n.contains("perennial-regex")));
        assert_eq!(kind(&i.workflow, "anything"), "feature");
        assert_eq!(kind(&i.workflow, "staging"), "base:staging");
        assert!(i.workflow.compile().warnings().is_empty());
    }

    #[test]
    fn recorded_parents_read_both_sources_and_ours_wins() {
        let p = parse_recorded_parents(&e(&[
            ("branch.feature/a.gitpoulpi-parent", "develop"),
            ("git-town-branch.feature/a.parent", "main"),
            ("git-town-branch.fix.b.parent", "main"),
            ("branch.x.merge", "refs/heads/x"),
            ("git-town-branch.late.parent", "main"),
            ("branch.late.gitpoulpi-parent", "dev"),
            ("branch..gitpoulpi-parent", "x"),
            ("branch.empty.gitpoulpi-parent", " "),
        ]));
        assert_eq!(
            p["feature/a"],
            ("develop".to_string(), ParentSource::Recorded)
        );
        assert_eq!(p["fix.b"], ("main".to_string(), ParentSource::GitTown));
        assert_eq!(p["late"], ("dev".to_string(), ParentSource::Recorded));
        assert!(!p.contains_key("x") && !p.contains_key("empty") && !p.contains_key(""));
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn branch_names_keep_their_case() {
        let p = parse_recorded_parents(&e(&[("branch.Feature/My.gitpoulpi-parent", "Main")]));
        assert_eq!(p["Feature/My"].0, "Main");
    }
}
