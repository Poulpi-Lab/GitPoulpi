//! The read-only picture of a repository under a workflow: every branch
//! classified, with its parent and how far it is from it. Built from plain
//! inputs so it is testable without a repository.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::domain::Oid;

use super::classify::{Classification, ParentInfo, ParentSource};
use super::detect::{detect, Detection};
use super::import::Imported;
use super::pattern::Fields;
use super::templates::WorkflowTemplate;
use super::Workflow;

/// Ahead/behind counts are computed for at most this many local branches
/// (each is one graph walk); the rest report `None`.
pub const MAX_COUNTED_BRANCHES: usize = 300;

/// A branch to report on.
#[derive(Debug, Clone)]
pub struct BranchInput {
    /// The branch name, without the remote.
    pub name: String,
    /// The remote of a remote-tracking branch; `None` for a local one.
    pub remote: Option<String>,
    pub tip: Oid,
    pub is_head: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchKind {
    Base,
    Topic,
    Unclassified,
}

/// One classified branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchReport {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    pub is_head: bool,
    pub kind: BranchKind,
    /// The topic type id (topic branches only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_id: Option<String>,
    /// The topic type's label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_label: Option<String>,
    /// The base's configured name (base branches only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// What the pattern read from the name (`slug`, `issue`, ...).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: Fields,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<ParentInfo>,
    /// Commits on this branch that its parent lacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ahead: Option<u32>,
    /// Commits on its parent that this branch lacks ("behind its base").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behind: Option<u32>,
}

/// Where the effective workflow came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSources {
    /// `.gitpoulpi/workflow.toml` exists (and was read).
    pub shared_file: bool,
    /// The per-user override in the repo state is non-empty.
    pub user_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowReport {
    /// A workflow is defined (a file, an override); otherwise `workflow` is
    /// empty and every branch is unclassified.
    pub configured: bool,
    pub template: WorkflowTemplate,
    pub sources: ReportSources,
    /// The effective workflow: shared file with the user override on top.
    pub workflow: Workflow,
    pub branches: Vec<BranchReport>,
    /// Other tools' configuration found in the repository, as workflows
    /// `flow import` can offer (never applied by themselves).
    #[serde(default)]
    pub imports: Vec<Imported>,
    /// Templates ranked for these branches (best first).
    pub detections: Vec<Detection>,
    /// Problems in the definition (bad patterns, unknown parents).
    pub warnings: Vec<String>,
}

/// Classifies `branches` under `workflow`. `recorded` maps a branch to its
/// recorded parent; `counts(tip, base_tip)` answers `(ahead, behind)` or
/// `None` when it cannot.
pub fn build_report(
    workflow: Workflow,
    sources: ReportSources,
    imports: Vec<Imported>,
    branches: &[BranchInput],
    recorded: &BTreeMap<String, (String, ParentSource)>,
    counts: &mut dyn FnMut(Oid, Oid) -> Option<(u32, u32)>,
) -> WorkflowReport {
    let configured = !workflow.is_empty();
    let compiled = workflow.compile();
    let local: BTreeMap<&str, Oid> = branches
        .iter()
        .filter(|b| b.remote.is_none())
        .map(|b| (b.name.as_str(), b.tip))
        .collect();
    let exists = |n: &str| local.contains_key(n);

    let mut rows: Vec<(usize, usize, BranchReport)> = Vec::new();
    let mut counted = 0usize;
    for b in branches {
        let class = if configured {
            compiled.classify(&b.name)
        } else {
            Classification::Unclassified
        };
        let rec = recorded
            .get(&b.name)
            .map(|(n, s)| (n.as_str(), *s))
            .filter(|_| b.remote.is_none());
        let parent = compiled.resolve_parent(&class, rec, &exists);
        let (mut ahead, mut behind) = (None, None);
        if b.remote.is_none() {
            if let Some(base_tip) = parent.as_ref().and_then(|p| local.get(p.name.as_str())) {
                if counted < MAX_COUNTED_BRANCHES {
                    counted += 1;
                    if let Some((a, bh)) = counts(b.tip, *base_tip) {
                        ahead = Some(a);
                        behind = Some(bh);
                    }
                }
            }
        }
        let (kind, rank, type_id, type_label, base, fields) = match &class {
            Classification::Base { name } => {
                let idx = workflow
                    .bases
                    .iter()
                    .position(|x| x.name == *name)
                    .unwrap_or(0);
                (
                    BranchKind::Base,
                    idx,
                    None,
                    None,
                    Some(name.clone()),
                    Fields::new(),
                )
            }
            Classification::Topic { type_id, fields } => {
                let idx = workflow.topics.iter().position(|t| t.id == *type_id);
                (
                    BranchKind::Topic,
                    1000 + idx.unwrap_or(0),
                    Some(type_id.clone()),
                    idx.map(|i| workflow.topics[i].display_label().to_string()),
                    None,
                    fields.clone(),
                )
            }
            Classification::Unclassified => (
                BranchKind::Unclassified,
                100_000,
                None,
                None,
                None,
                Fields::new(),
            ),
        };
        rows.push((
            rank,
            usize::from(b.remote.is_some()),
            BranchReport {
                name: b.name.clone(),
                remote: b.remote.clone(),
                is_head: b.is_head,
                kind,
                type_id,
                type_label,
                base,
                fields,
                parent,
                ahead,
                behind,
            },
        ));
    }
    rows.sort_by(|a, b| {
        (a.0, &a.2.name, a.1, &a.2.remote).cmp(&(b.0, &b.2.name, b.1, &b.2.remote))
    });

    let mut names: BTreeSet<String> = BTreeSet::new();
    names.extend(branches.iter().map(|b| b.name.clone()));
    let names: Vec<String> = names.into_iter().collect();
    WorkflowReport {
        configured,
        template: workflow.template,
        sources,
        warnings: compiled.warnings().to_vec(),
        imports,
        detections: detect(&names),
        branches: rows.into_iter().map(|r| r.2).collect(),
        workflow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn input(name: &str, remote: Option<&str>, n: u8) -> BranchInput {
        BranchInput {
            name: name.into(),
            remote: remote.map(str::to_string),
            tip: oid(n),
            is_head: false,
        }
    }

    fn report(branches: &[BranchInput]) -> WorkflowReport {
        build_report(
            WorkflowTemplate::GitFlow.workflow(),
            ReportSources::default(),
            Vec::new(),
            branches,
            &BTreeMap::new(),
            &mut |a, b| Some((u32::from(a.0[0]), u32::from(b.0[0]))),
        )
    }

    #[test]
    fn classifies_orders_and_counts() {
        let r = report(&[
            input("zzz", None, 9),
            input("feature/b", None, 4),
            input("feature/a", None, 3),
            input("feature/a", Some("origin"), 3),
            input("develop", None, 2),
            input("main", None, 1),
            input("release/1.0", None, 5),
        ]);
        assert!(r.configured);
        let order: Vec<_> = r
            .branches
            .iter()
            .map(|b| (b.name.as_str(), b.remote.as_deref()))
            .collect();
        assert_eq!(
            order,
            [
                ("main", None),
                ("develop", None),
                ("feature/a", None),
                ("feature/a", Some("origin")),
                ("feature/b", None),
                ("release/1.0", None),
                ("zzz", None)
            ]
        );
        let a = &r.branches[2];
        assert_eq!(a.kind, BranchKind::Topic);
        assert_eq!(a.type_id.as_deref(), Some("feature"));
        assert_eq!(a.type_label.as_deref(), Some("Feature"));
        assert_eq!(a.fields["slug"], "a");
        assert_eq!(a.parent.as_ref().unwrap().name, "develop");
        assert_eq!((a.ahead, a.behind), (Some(3), Some(2)));
        // remote branches get a parent but no counts
        let ra = &r.branches[3];
        assert!(ra.parent.is_some());
        assert_eq!((ra.ahead, ra.behind), (None, None));
        // develop sits on main
        let dev = &r.branches[1];
        assert_eq!(dev.parent.as_ref().unwrap().source, ParentSource::Base);
        assert_eq!(dev.base.as_deref(), Some("develop"));
        // trunk and unclassified have none
        assert!(r.branches[0].parent.is_none());
        assert_eq!(r.branches[6].kind, BranchKind::Unclassified);
        assert!(r.branches[6].parent.is_none());
        assert_eq!(r.detections[0].template, WorkflowTemplate::GitFlow);
    }

    #[test]
    fn a_recorded_parent_beats_the_type_and_must_exist() {
        let mut rec = BTreeMap::new();
        rec.insert(
            "feature/a".to_string(),
            ("feature/base".to_string(), ParentSource::Recorded),
        );
        rec.insert(
            "feature/b".to_string(),
            ("gone".to_string(), ParentSource::Recorded),
        );
        let r = build_report(
            WorkflowTemplate::GitFlow.workflow(),
            ReportSources::default(),
            Vec::new(),
            &[
                input("main", None, 1),
                input("develop", None, 2),
                input("feature/base", None, 3),
                input("feature/a", None, 4),
                input("feature/b", None, 5),
            ],
            &rec,
            &mut |_, _| None,
        );
        let by = |n: &str| r.branches.iter().find(|b| b.name == n).unwrap();
        assert_eq!(
            by("feature/a").parent.as_ref().unwrap().name,
            "feature/base"
        );
        assert_eq!(
            by("feature/a").parent.as_ref().unwrap().source,
            ParentSource::Recorded
        );
        assert_eq!(by("feature/b").parent.as_ref().unwrap().name, "develop");
        assert_eq!(by("feature/a").ahead, None);
    }

    #[test]
    fn no_workflow_means_everything_is_unclassified_but_detection_still_runs() {
        let r = build_report(
            Workflow::default(),
            ReportSources::default(),
            Vec::new(),
            &[input("main", None, 1), input("develop", None, 2)],
            &BTreeMap::new(),
            &mut |_, _| Some((0, 0)),
        );
        assert!(!r.configured);
        assert!(r
            .branches
            .iter()
            .all(|b| b.kind == BranchKind::Unclassified));
        assert_eq!(r.detections[0].template, WorkflowTemplate::GitFlow);
    }

    #[test]
    fn counting_is_capped() {
        let mut branches = vec![input("main", None, 1), input("develop", None, 2)];
        for i in 0..(MAX_COUNTED_BRANCHES + 20) {
            branches.push(input(&format!("feature/f{i}"), None, 3));
        }
        let mut calls = 0usize;
        let r = build_report(
            WorkflowTemplate::GitFlow.workflow(),
            ReportSources::default(),
            Vec::new(),
            &branches,
            &BTreeMap::new(),
            &mut |_, _| {
                calls += 1;
                Some((1, 1))
            },
        );
        assert_eq!(calls, MAX_COUNTED_BRANCHES);
        assert!(r
            .branches
            .iter()
            .any(|b| b.ahead.is_none() && b.parent.is_some()));
    }
}
