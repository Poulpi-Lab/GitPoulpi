//! Grouped push / pull over a branch's several remotes (see
//! `gitpoulpi_core::domain::remote_group`). This slice is the planning read
//! (`RemoteGroupPlan`); the run engine builds on the same module.

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

mod exec;
mod plan;
mod state;

use gitpoulpi_core::domain::{
    GroupMode, GroupResume, PendingGroup, RemoteGroupPlanRequest, RemoteGroupRunRequest, RepoRef,
};
use gitpoulpi_core::GitBackend;
use serde_json::Value;

use crate::dispatch::Failure;

/// The `RemoteGroupPlan` read: ask each target, fetch what is needed,
/// classify, suggest.
pub(crate) async fn plan_read(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    args: &Value,
) -> Result<Value, Failure> {
    let req: RemoteGroupPlanRequest = serde_json::from_value(args.clone()).map_err(|e| {
        (
            format!("invalid remote group plan request: {e}"),
            None,
            None,
        )
    })?;
    let plan = plan::build(backend, repo, &req)
        .await
        .map_err(|e| (e.to_string(), e.typed(), e.detail().cloned()))?;
    serde_json::to_value(&plan).map_err(|e| (format!("failed to encode result: {e}"), None, None))
}

/// `RemoteGroupRun`: a fresh run from per-target decisions, or `resume`
/// (continue, abort) / `retry` of the saved one. Progress goes out as
/// `Event::RemoteGroupProgress`.
pub(crate) async fn run(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    events: &tokio::sync::broadcast::Sender<gitpoulpi_ipc::Event>,
    args: &Value,
) -> Result<Value, Failure> {
    exec::run(backend, repo, events, args).await
}

fn parse(args: &Value) -> RemoteGroupRunRequest {
    serde_json::from_value(args.clone()).unwrap_or_default()
}

/// The saved run of this worktree as a read shows it, with the readiness
/// flag computed now (never acted on: a read starts nothing).
pub(crate) fn pending_of(backend: &dyn GitBackend, repo: &RepoRef) -> Option<PendingGroup> {
    let st = state::load(repo)?;
    let ready = exec::ready_to_continue(backend, repo, &st);
    Some(PendingGroup {
        run_id: st.run_id,
        branch: st.branch,
        mode: st.mode,
        started_tip: st.started_tip,
        targets: st.targets.into_iter().map(|t| t.result).collect(),
        stopped: st.stopped,
        ready_to_continue: ready,
    })
}

/// The branches whose refs a run may change, for the journal's write-ahead
/// capture: the branch of the request, or of the saved run on a resume.
pub(crate) fn involved_branches(repo: &RepoRef, args: &Value) -> Vec<String> {
    let req = parse(args);
    let branch = if req.resume.is_some() || req.retry {
        state::load(repo).map(|s| s.branch)
    } else {
        Some(req.branch)
    };
    branch.filter(|b| !b.is_empty()).into_iter().collect()
}

/// The label of the journal entry.
pub(crate) fn label(request: &gitpoulpi_ipc::Request) -> String {
    let req = parse(&request.args);
    let what = if req.branch.is_empty() {
        String::new()
    } else {
        format!(" {}", req.branch)
    };
    let word = match req.mode {
        GroupMode::Push => "Grouped push",
        GroupMode::Pull => "Grouped pull",
    };
    match req.resume {
        Some(GroupResume::Continue) => format!("{word} (continued){what}"),
        Some(GroupResume::Abort) => format!("{word} (aborted){what}"),
        None => format!("{word}{what}"),
    }
}
