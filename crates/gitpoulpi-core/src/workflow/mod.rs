//! Branching workflows: which branches are long-lived bases, which are
//! topic branches of which type, how topic branches are named and where they
//! start and finish (git-flow, GitHub Flow, GitLab Flow, trunk-based, or
//! custom). Pure data and logic; reading and writing the files lives in
//! `gitpoulpi-session`, the daemon read in `gitpoulpi-daemon`.
//!
//! The serialized shape is the TOML of `.gitpoulpi/workflow.toml`:
//! `[[base]]`, `[[topic]]`, `[hooks]`, `[issue_flow]` (see
//! `docs/user/branching-workflows.md`).

mod classify;
mod detect;
mod flow;
mod import;
mod issue_flow;
mod pattern;
mod refname;
mod report;
mod templates;

use serde::{Deserialize, Serialize};

pub use classify::{Classification, CompiledWorkflow, ParentInfo, ParentSource};
pub use detect::{detect, Detection, FoundBase, FoundTopic};
pub use flow::{
    branch_title, plan_flow, FlowAction, FlowFacts, FlowOutcome, FlowPlan, FlowRequest, FlowResume,
    FlowState, FlowStatus, FlowStep, HookInfo, MergeRequestSpec, StepReport, StepStatus,
    SyncMethod,
};
pub use import::{
    import_git_flow, import_git_town, parse_recorded_parents, Imported, RECORDED_PARENT_VARIABLE,
    WORKFLOW_CONFIG_PREFIXES,
};
pub use issue_flow::{
    branches_for_issue, issue_branch_pattern, issue_draft, IssueDraft, WaitFor,
    DEFAULT_BRANCH_FROM_ISSUE, DEFAULT_WAIT_SECS,
};
pub use pattern::{BranchPattern, Fields, PatternError, Token};
pub use refname::{check_ref_format, slugify, RefFormatError, DEFAULT_SLUG_MAX};
pub use report::{
    build_report, BranchInput, BranchKind, BranchReport, ReportSources, WorkflowReport,
    MAX_COUNTED_BRANCHES,
};
pub use templates::{WorkflowTemplate, NAME_PRESETS};

fn is_false(b: &bool) -> bool {
    !*b
}

/// How a topic or base branch is merged into its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MergeStrategy {
    Merge,
    Squash,
    Rebase,
    FastForward,
}

/// What "finish" does with a topic branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishMode {
    LocalMerge,
    MergeRequest,
    Ask,
}

/// A long-lived branch (`main`, `develop`, `production`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BaseBranch {
    /// The name (the first candidate; with `names`, the preferred spelling).
    pub name: String,
    /// More candidate spellings of the same base (`master` for `main`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
    /// The base this one is merged into (a `name` or one of its `names`);
    /// none for a trunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// No direct commit or push warning, never deleted.
    #[serde(skip_serializing_if = "is_false")]
    pub protected: bool,
    /// How it is merged into its parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<MergeStrategy>,
    /// Tag the merge commit when merging into this base.
    #[serde(skip_serializing_if = "is_false")]
    pub tag_on_merge: bool,
    /// An environment branch (GitLab Flow): promotion merges downstream.
    #[serde(skip_serializing_if = "is_false")]
    pub environment: bool,
}

impl BaseBranch {
    /// Every spelling that names this base, preferred first.
    pub fn all_names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name.as_str())
            .chain(self.names.iter().map(String::as_str))
            .filter(|n| !n.is_empty())
    }

    pub fn is_named(&self, name: &str) -> bool {
        self.all_names().any(|n| n == name)
    }
}

/// One thing finishing a topic branch does: merge into `into`, maybe tag.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FinishTarget {
    pub into: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<MergeStrategy>,
    /// Tag to create, a pattern over the branch fields (`v{version}`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// A kind of topic branch (`feature`, `hotfix`, ...).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TopicType {
    pub id: String,
    /// Display label; the id when empty.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// The name pattern: what start renders and classification matches.
    pub pattern: String,
    /// More patterns that also match (`features/{slug}`); never rendered.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,
    /// A regular expression that also matches (never rendered); named
    /// groups (`(?P<slug>..)`) become fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    /// The start point and default finish target: a base name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finish: Vec<FinishTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_mode: Option<FinishMode>,
    #[serde(skip_serializing_if = "is_false")]
    pub delete_after_finish: bool,
}

impl TopicType {
    pub fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.id
        } else {
            &self.label
        }
    }
}

/// Opt-in shell commands (always shown before they run).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Hooks {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub after_start: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub before_finish: Vec<String>,
}

impl Hooks {
    pub fn is_empty(&self) -> bool {
        self.after_start.is_empty() && self.before_finish.is_empty()
    }
}

/// Who creates the branch of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueFlowMode {
    #[default]
    Local,
    HostCreatesBranch,
    Bot,
}

/// Host-driven flows (slice W4): an issue made from a template, a branch
/// and merge request made by the host or a bot, picked up locally.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct IssueFlow {
    pub mode: IssueFlowMode,
    /// `{type}` and `{title}` are replaced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_title: Option<String>,
    /// Label names; `{type}` is replaced.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issue_labels: Vec<String>,
    /// The name the host or bot gives the branch; `{type}` is the topic id,
    /// the rest are the pattern tokens (`{issue}`, `{slug}`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_from_issue: Option<String>,
    /// What to wait for: `branch`, `mr` or `branch-or-mr` (default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for: Option<String>,
    /// A Markdown file of the repository used as the issue description;
    /// `{type}`, `{title}`, `{user}` and `{date}` are replaced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_template: Option<String>,
    /// How long `wait` polls before giving up (seconds; default 600).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// `host-creates-branch`: also open the merge request (default true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_request: Option<bool>,
    /// People the issue is assigned to.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assignees: Vec<String>,
}

/// A repository's branching workflow.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Workflow {
    /// The template it started from (informative).
    pub template: WorkflowTemplate,
    #[serde(rename = "base", skip_serializing_if = "Vec::is_empty")]
    pub bases: Vec<BaseBranch>,
    #[serde(rename = "topic", skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<TopicType>,
    #[serde(skip_serializing_if = "Hooks::is_empty")]
    pub hooks: Hooks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_flow: Option<IssueFlow>,
}

impl Workflow {
    /// Whether nothing is defined at all.
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty() && self.topics.is_empty()
    }

    /// Prepares matching (compiles patterns and regexes).
    pub fn compile(&self) -> CompiledWorkflow {
        CompiledWorkflow::new(self)
    }

    /// One-shot [`CompiledWorkflow::classify`].
    pub fn classify(&self, branch: &str) -> Classification {
        self.compile().classify(branch)
    }

    /// This workflow with the per-user `over` layered on top: a base with the
    /// same name or a topic with the same id is replaced, new ones are
    /// appended, `disable_*` entries are removed.
    pub fn merged_with(&self, over: &WorkflowOverride) -> Workflow {
        let mut out = self.clone();
        if let Some(t) = over.template {
            out.template = t;
        }
        out.bases
            .retain(|b| !over.disable_bases.iter().any(|d| b.is_named(d)));
        out.topics.retain(|t| !over.disable_topics.contains(&t.id));
        for b in &over.bases {
            match out.bases.iter_mut().find(|e| e.name == b.name) {
                Some(slot) => *slot = b.clone(),
                None => out.bases.push(b.clone()),
            }
        }
        for t in &over.topics {
            match out.topics.iter_mut().find(|e| e.id == t.id) {
                Some(slot) => *slot = t.clone(),
                None => out.topics.push(t.clone()),
            }
        }
        // A reorder is kept as the full list of names, first to last.
        if !over.base_order.is_empty() {
            let at = |b: &BaseBranch| over.base_order.iter().position(|n| b.is_named(n));
            out.bases.sort_by_key(|b| at(b).unwrap_or(usize::MAX));
        }
        if !over.topic_order.is_empty() {
            let at = |t: &TopicType| over.topic_order.iter().position(|n| *n == t.id);
            out.topics.sort_by_key(|t| at(t).unwrap_or(usize::MAX));
        }
        if let Some(h) = &over.hooks {
            out.hooks = h.clone();
        }
        if let Some(i) = &over.issue_flow {
            out.issue_flow = Some(i.clone());
        }
        out
    }
}

/// The per-user tweaks stored in the repo state (`[workflow]` of
/// `.gitpoulpi/repo-state.toml`), layered over the shared file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkflowOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<WorkflowTemplate>,
    #[serde(rename = "base", skip_serializing_if = "Vec::is_empty")]
    pub bases: Vec<BaseBranch>,
    #[serde(rename = "topic", skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<TopicType>,
    /// Shared bases (by any name) this user does not want.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disable_bases: Vec<String>,
    /// Shared topic ids this user does not want.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disable_topics: Vec<String>,
    /// The order of the bases (by name), when this user reordered them.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub base_order: Vec<String>,
    /// The order of the topic ids, when this user reordered them.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub topic_order: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_flow: Option<IssueFlow>,
}

impl WorkflowOverride {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Problems with a workflow definition, for display (never fatal: a broken
/// topic is skipped by classification).
pub fn validate(workflow: &Workflow) -> Vec<String> {
    CompiledWorkflow::new(workflow).warnings().to_vec()
}

#[cfg(test)]
mod tests;
