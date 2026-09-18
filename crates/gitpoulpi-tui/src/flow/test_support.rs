//! Fixtures for the Flow tests: a workflow report over a list of branch
//! names, installed in the (thread-local) store.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::workflow::{
    build_report, BranchInput, ReportSources, Workflow, WorkflowReport, WorkflowTemplate,
};

use super::view::{self, FlowView};

/// The report of `workflow` over local `branches` (`head` is checked out);
/// every branch is `develop` ahead and behind by nothing unless `counts`
/// says otherwise.
pub(crate) fn report(
    workflow: Workflow,
    branches: &[&str],
    head: &str,
    counts: &[(&str, (u32, u32))],
) -> WorkflowReport {
    let inputs: Vec<BranchInput> = branches
        .iter()
        .enumerate()
        .map(|(i, name)| BranchInput {
            name: (*name).to_string(),
            remote: None,
            tip: Oid([i as u8 + 1; 20]),
            is_head: *name == head,
        })
        .collect();
    let by_tip: HashMap<Oid, &str> = inputs.iter().map(|b| (b.tip, b.name.as_str())).collect();
    let counts: BTreeMap<String, (u32, u32)> =
        counts.iter().map(|(n, c)| ((*n).to_string(), *c)).collect();
    build_report(
        workflow,
        ReportSources {
            shared_file: true,
            user_override: false,
        },
        Vec::new(),
        &inputs,
        &BTreeMap::new(),
        &mut |tip, _base| by_tip.get(&tip).and_then(|name| counts.get(*name)).copied(),
    )
}

/// A view of `template` over `branches`.
pub(crate) fn view_of(
    template: WorkflowTemplate,
    branches: &[&str],
    head: &str,
    counts: &[(&str, (u32, u32))],
) -> Arc<FlowView> {
    Arc::new(FlowView::new(report(
        template.workflow(),
        branches,
        head,
        counts,
    )))
}

/// Installs a git-flow view over `branches` in this thread's store.
pub(crate) fn install_git_flow(branches: &[&str], head: &str) -> Arc<FlowView> {
    let v = view_of(WorkflowTemplate::GitFlow, branches, head, &[]);
    view::install(Some(Arc::clone(&v)));
    v
}
