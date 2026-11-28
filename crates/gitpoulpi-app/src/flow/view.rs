//! What a frontend knows about the repository's workflow: the daemon's
//! `Workflow` report with what is derived from it, the grouped sidebar rows
//! and the branch tooltips built from it.
//!
//! Pure model: the process-wide store the terminal UI keeps the report in
//! (and the hovered-branch state) stays in that frontend and passes the view
//! to these functions.

use std::collections::HashMap;

use gitpoulpi_core::workflow::{
    BranchKind, BranchReport, CompiledWorkflow, TopicType, WorkflowReport,
};
use gitpoulpi_i18n::msg::flow as msg;

use crate::refs_tree::{CollapsedFolders, RefRowKind, RefTreeRow};

/// A folder path that is a workflow group, not a `/`-derived folder.
pub const GROUP_PREFIX: &str = "\u{1f}flow:";

/// The report plus what is derived from it once.
pub struct FlowView {
    pub report: WorkflowReport,
    by_name: HashMap<String, usize>,
    compiled: CompiledWorkflow,
}

/// Where a branch sits in the grouped sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Group {
    /// A base branch, by its index in the workflow.
    Base(usize),
    /// A topic type, by id.
    Topic(String),
    Other,
}

impl FlowView {
    pub fn new(report: WorkflowReport) -> Self {
        let mut by_name = HashMap::new();
        for (i, b) in report.branches.iter().enumerate() {
            let key = match &b.remote {
                Some(remote) => format!("{remote}/{}", b.name),
                None => b.name.clone(),
            };
            by_name.insert(key, i);
        }
        let compiled = report.workflow.compile();
        Self {
            report,
            by_name,
            compiled,
        }
    }

    /// A workflow is defined for this repository.
    pub fn configured(&self) -> bool {
        self.report.configured
    }

    pub fn topics(&self) -> &[TopicType] {
        &self.report.workflow.topics
    }

    pub fn compiled(&self) -> &CompiledWorkflow {
        &self.compiled
    }

    /// The report of a branch by its full ref name (`name`, `origin/name`).
    pub fn branch(&self, full_name: &str) -> Option<&BranchReport> {
        self.by_name
            .get(full_name)
            .and_then(|i| self.report.branches.get(*i))
    }

    pub fn group_of(&self, full_name: &str) -> Group {
        match self.branch(full_name) {
            Some(b) if b.kind == BranchKind::Base => {
                let at = b.base.as_deref().and_then(|name| {
                    self.report
                        .workflow
                        .bases
                        .iter()
                        .position(|x| x.name == name)
                });
                Group::Base(at.unwrap_or(0))
            }
            Some(b) if b.kind == BranchKind::Topic => match &b.type_id {
                Some(id) => Group::Topic(id.clone()),
                None => Group::Other,
            },
            _ => Group::Other,
        }
    }

    /// The configured names of every base (preferred spelling first).
    pub fn base_names(&self) -> Vec<String> {
        self.report
            .workflow
            .bases
            .iter()
            .map(|b| b.name.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------
// The grouped sidebar rows
// ---------------------------------------------------------------------

/// Whether a folder path is a workflow group.
pub fn is_group_path(path: &str) -> bool {
    path.starts_with(GROUP_PREFIX)
}

fn group_path(id: &str) -> String {
    format!("{GROUP_PREFIX}{id}")
}

const OTHER_ID: &str = "\u{2}other";

/// The rows of LOCAL / REMOTE grouped by the workflow, or `None` when the
/// plain tree applies (no workflow, the toggle is off, tags).
///
/// Base branches first (a badge, a marker when protected), then one folder
/// per topic type in the workflow's order (label and count), then what no
/// type claims. A folder's collapse key is `(kind, GROUP_PREFIX + id)`.
pub fn grouped_rows(
    view: &FlowView,
    names: &[String],
    kind: RefRowKind,
    collapsed: &CollapsedFolders,
) -> Option<Vec<RefTreeRow>> {
    if kind == RefRowKind::Tag || !view.configured() {
        return None;
    }
    let mut bases: Vec<(usize, &String)> = Vec::new();
    let mut topics: Vec<Vec<&String>> = vec![Vec::new(); view.topics().len()];
    let mut other: Vec<&String> = Vec::new();
    for name in names {
        match view.group_of(name) {
            Group::Base(i) => bases.push((i, name)),
            Group::Topic(id) => match view.topics().iter().position(|t| t.id == id) {
                Some(i) => topics[i].push(name),
                None => other.push(name),
            },
            Group::Other => other.push(name),
        }
    }
    bases.sort_by_key(|(i, n)| (*i, (*n).clone()));
    let mut out = Vec::new();
    for (i, name) in bases {
        let protected = view
            .report
            .workflow
            .bases
            .get(i)
            .is_some_and(|b| b.protected);
        let badge = if protected {
            msg::BADGE_BASE_PROTECTED.get()
        } else {
            msg::BADGE_BASE.get()
        };
        out.push(RefTreeRow::Leaf {
            full_name: name.clone(),
            display_name: format!("{name}  {badge}"),
            depth: 0,
        });
    }
    let mut push_group = |id: String, label: String, members: &[&String]| {
        if members.is_empty() {
            return;
        }
        let path = group_path(&id);
        let open = !collapsed.contains(&(kind, path.clone()));
        out.push(RefTreeRow::Folder {
            path,
            label: format!("{label} ({})", members.len()),
            depth: 0,
            is_remote_root: false,
        });
        if open {
            for name in members {
                out.push(RefTreeRow::Leaf {
                    full_name: (*name).clone(),
                    display_name: (*name).clone(),
                    depth: 1,
                });
            }
        }
    };
    for (topic, members) in view.topics().iter().zip(&topics) {
        push_group(topic.id.clone(), topic.display_label().to_string(), members);
    }
    push_group(OTHER_ID.into(), msg::GROUP_OTHER.get().into(), &other);
    Some(out)
}

/// The branches of `names` that sit under the group folder `path` (what
/// "hide / solo this folder" acts on).
pub fn group_members(view: &FlowView, names: &[String], path: &str) -> Vec<String> {
    let Some(id) = path.strip_prefix(GROUP_PREFIX) else {
        return Vec::new();
    };
    if !view.configured() {
        return Vec::new();
    }
    names
        .iter()
        .filter(|n| match (view.group_of(n), id) {
            (Group::Topic(t), id) => t == id,
            (Group::Other, id) => id == OTHER_ID,
            (Group::Base(_), _) => false,
        })
        .cloned()
        .collect()
}

/// The tooltip lines of a sidebar branch: its kind, its parent and how far
/// behind it the branch is. `None` when no workflow is configured or the
/// branch is unknown to the report.
pub fn branch_tooltip(view: &FlowView, full_name: &str) -> Option<Vec<String>> {
    if !view.configured() {
        return None;
    }
    let b = view.branch(full_name)?;
    let mut lines = vec![full_name.to_string()];
    match b.kind {
        BranchKind::Base => lines.push(msg::TIP_BASE.get().into()),
        BranchKind::Topic => {
            if let Some(label) = &b.type_label {
                lines.push(msg::tip_type(label));
            }
        }
        BranchKind::Unclassified => lines.push(msg::TIP_UNCLASSIFIED.get().into()),
    }
    if let Some(parent) = &b.parent {
        lines.push(msg::tip_parent(&parent.name));
        match (b.behind, b.ahead) {
            (Some(0), Some(0)) => lines.push(msg::TIP_UP_TO_DATE.get().into()),
            (Some(behind), ahead) => {
                lines.push(msg::tip_behind(u64::from(behind), &parent.name));
                if let Some(ahead) = ahead.filter(|a| *a > 0) {
                    lines.push(msg::tip_ahead(u64::from(ahead)));
                }
            }
            _ => {}
        }
    }
    Some(lines)
}
