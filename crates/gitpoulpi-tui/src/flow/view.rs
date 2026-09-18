//! What the window knows about the repository's workflow: the daemon's
//! `Workflow` report, kept process-wide so the sidebar's row builder
//! (`refs_tree::ref_display_rows`, called from a dozen places that must agree
//! on the rows) can group branches without every caller being handed it.
//!
//! One store per process: a window installs its report when it arrives and
//! clears it when it opens (a repository switch restarts the window). In unit
//! tests the store is **thread-local**, like the process-wide settings in
//! `live_settings`, so tests running in parallel never see each other's.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_core::workflow::{
    BranchKind, BranchReport, CompiledWorkflow, TopicType, WorkflowReport,
};
use gitpoulpi_i18n::msg::flow as msg;

use crate::refs_tree::{CollapsedFolders, RefRowKind, RefTreeRow};

/// A folder path that is a workflow group, not a `/`-derived folder.
pub(crate) const GROUP_PREFIX: &str = "\u{1f}flow:";

/// The report plus what is derived from it once.
pub(crate) struct FlowView {
    pub(crate) report: WorkflowReport,
    by_name: HashMap<String, usize>,
    compiled: CompiledWorkflow,
}

/// Where a branch sits in the grouped sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Group {
    /// A base branch, by its index in the workflow.
    Base(usize),
    /// A topic type, by id.
    Topic(String),
    Other,
}

impl FlowView {
    pub(crate) fn new(report: WorkflowReport) -> Self {
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
    pub(crate) fn configured(&self) -> bool {
        self.report.configured
    }

    pub(crate) fn topics(&self) -> &[TopicType] {
        &self.report.workflow.topics
    }

    pub(crate) fn compiled(&self) -> &CompiledWorkflow {
        &self.compiled
    }

    /// The report of a branch by its full ref name (`name`, `origin/name`).
    pub(crate) fn branch(&self, full_name: &str) -> Option<&BranchReport> {
        self.by_name
            .get(full_name)
            .and_then(|i| self.report.branches.get(*i))
    }

    pub(crate) fn group_of(&self, full_name: &str) -> Group {
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
    pub(crate) fn base_names(&self) -> Vec<String> {
        self.report
            .workflow
            .bases
            .iter()
            .map(|b| b.name.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------

#[cfg(not(test))]
mod store {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::RwLock;

    static VIEW: RwLock<Option<Arc<FlowView>>> = RwLock::new(None);
    static PLAIN: AtomicBool = AtomicBool::new(false);
    static VERSION: AtomicU64 = AtomicU64::new(0);

    pub(super) fn current() -> Option<Arc<FlowView>> {
        VIEW.read().ok().and_then(|v| v.clone())
    }
    pub(super) fn install(view: Option<Arc<FlowView>>) {
        if let Ok(mut slot) = VIEW.write() {
            *slot = view;
        }
        VERSION.fetch_add(1, Ordering::SeqCst);
    }
    pub(super) fn version() -> u64 {
        VERSION.load(Ordering::SeqCst)
    }
    pub(super) fn plain() -> bool {
        PLAIN.load(Ordering::SeqCst)
    }
    pub(super) fn set_plain(on: bool) {
        PLAIN.store(on, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod store {
    use super::*;
    use std::cell::{Cell, RefCell};

    thread_local! {
        static VIEW: RefCell<Option<Arc<FlowView>>> = const { RefCell::new(None) };
        static PLAIN: Cell<bool> = const { Cell::new(false) };
        static VERSION: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn current() -> Option<Arc<FlowView>> {
        VIEW.with(|v| v.borrow().clone())
    }
    pub(super) fn install(view: Option<Arc<FlowView>>) {
        VIEW.with(|v| *v.borrow_mut() = view);
        VERSION.with(|v| v.set(v.get() + 1));
    }
    pub(super) fn version() -> u64 {
        VERSION.with(Cell::get)
    }
    pub(super) fn plain() -> bool {
        PLAIN.with(Cell::get)
    }
    pub(super) fn set_plain(on: bool) {
        PLAIN.with(|p| p.set(on));
    }
}

/// The window's current report, if one arrived.
pub(crate) fn current() -> Option<Arc<FlowView>> {
    store::current()
}

/// The report when a workflow is configured (the toolbar button, the
/// palette commands and the grouping all need exactly that).
pub(crate) fn configured() -> Option<Arc<FlowView>> {
    current().filter(|v| v.configured())
}

pub(crate) fn install(view: Option<Arc<FlowView>>) {
    store::install(view);
}

/// Changes whenever a report is installed or cleared (the Settings page
/// refreshes its copy when it moves).
pub(crate) fn version() -> u64 {
    store::version()
}

/// The sidebar shows the plain LOCAL / REMOTE trees instead of the groups.
pub(crate) fn plain() -> bool {
    store::plain()
}

pub(crate) fn set_plain(on: bool) {
    store::set_plain(on);
}

// ---------------------------------------------------------------------
// The grouped sidebar rows
// ---------------------------------------------------------------------

/// Whether a folder path is a workflow group.
pub(crate) fn is_group_path(path: &str) -> bool {
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
pub(crate) fn grouped_rows(
    names: &[String],
    kind: RefRowKind,
    collapsed: &CollapsedFolders,
) -> Option<Vec<RefTreeRow>> {
    if kind == RefRowKind::Tag || plain() {
        return None;
    }
    let view = configured()?;
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
pub(crate) fn group_members(names: &[String], path: &str) -> Vec<String> {
    let Some(id) = path.strip_prefix(GROUP_PREFIX) else {
        return Vec::new();
    };
    let Some(view) = configured() else {
        return Vec::new();
    };
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
pub(crate) fn branch_tooltip(full_name: &str) -> Option<Vec<String>> {
    let view = configured()?;
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

// ---------------------------------------------------------------------
// The hovered sidebar branch (its tooltip)
// ---------------------------------------------------------------------

#[cfg(not(test))]
static HOVER: std::sync::Mutex<Option<(String, u16, u16)>> = std::sync::Mutex::new(None);
#[cfg(test)]
thread_local! {
    static HOVER: std::cell::RefCell<Option<(String, u16, u16)>> =
        const { std::cell::RefCell::new(None) };
}

/// The pointer is over sidebar branch `name` at `(col, row)`, or over none.
pub(crate) fn set_hover(hover: Option<(String, u16, u16)>) {
    #[cfg(not(test))]
    if let Ok(mut slot) = HOVER.lock() {
        *slot = hover;
    }
    #[cfg(test)]
    HOVER.with(|h| *h.borrow_mut() = hover);
}

fn hover() -> Option<(String, u16, u16)> {
    #[cfg(not(test))]
    {
        HOVER.lock().ok().and_then(|h| h.clone())
    }
    #[cfg(test)]
    {
        HOVER.with(|h| h.borrow().clone())
    }
}

/// The tooltip key of the hovered branch (a change of key is a change of
/// hover target), `None` when it has no tooltip.
pub(crate) fn hover_key() -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let (name, _, _) = hover()?;
    branch_tooltip(&name)?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut h);
    Some(h.finish())
}

/// `(key, lines, col, row)` of the hovered branch's tooltip.
pub(crate) fn hover_tooltip() -> Option<(u64, Vec<String>, u16, u16)> {
    let key = hover_key()?;
    let (name, col, row) = hover()?;
    Some((key, branch_tooltip(&name)?, col, row))
}
