//! The branching-workflow actions (`FlowPlan`, `FlowRun`): the planner is
//! pure (`gitpoulpi_core::workflow::plan_flow`); this gathers its facts,
//! runs the steps through `GitBackend` and keeps what a conflict needs to
//! resume. The whole run is one undo-journal entry (`journal/capture.rs` and
//! `journal/after.rs` special-case `FlowRun`), so the steps themselves are
//! plain backend calls, never nested journaled requests.
//!
//! A step that cannot finish stops the flow:
//! * a merge or rebase on conflicts saves a [`FlowState`] and answers
//!   `conflicted`; `resume: continue` completes that step (commits a resolved
//!   merge, continues a rebase) and runs the rest, `resume: abort` aborts the
//!   operation and returns to the branch the flow started on;
//! * any other failure after something changed (a hook exiting non-zero)
//!   saves the state too, so `continue` retries that step. A failure before
//!   anything changed is a plain error (the auto-stash layer needs to see
//!   it, and nothing has to be resumed).

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

mod exec;
mod facts;
mod hooks;
mod state;
mod trust;

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::workflow::{plan_flow, FlowPlan, FlowRequest, FlowResume, FlowState, FlowStep};
use gitpoulpi_core::GitBackend;
use serde_json::Value;

use crate::dispatch::Failure;
use facts::{gather, plain_failure};

fn parse(args: &Value) -> Result<FlowRequest, Failure> {
    serde_json::from_value(args.clone())
        .map_err(|e| plain_failure(format!("invalid flow request: {e}")))
}

fn encode<T: serde::Serialize>(value: &T) -> Result<Value, Failure> {
    serde_json::to_value(value).map_err(|e| plain_failure(format!("failed to encode result: {e}")))
}

/// The plan for `req` as the answer of `FlowPlan`: for a resume, the steps
/// the saved flow still has to do.
#[allow(clippy::result_large_err)]
pub(crate) fn plan_of(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &FlowRequest,
) -> Result<FlowPlan, Failure> {
    if let Some(resume) = req.resume {
        let state = state::load(repo)
            .ok_or_else(|| plain_failure("no flow is waiting to be continued or aborted"))?;
        return Ok(resume_plan(&state, resume));
    }
    let (workflow, facts) = gather(backend, repo)?;
    let mut plan = plan_flow(&workflow, &facts, req, &mut |tip, base| {
        backend.ahead_behind(repo, tip, base).ok()
    });
    plan.hooks_trust = trust::classify(repo, &facts, &plan);
    Ok(plan)
}

pub(crate) use trust::trust_hooks;

fn resume_plan(state: &FlowState, resume: FlowResume) -> FlowPlan {
    let mut plan = FlowPlan {
        action: state.request.action,
        title: match resume {
            FlowResume::Continue => format!("Continue {}", state.request.action.word()),
            FlowResume::Abort => format!("Abort {}", state.request.action.word()),
        },
        branch: state.request.branch.clone(),
        resumed: true,
        undoable: true,
        merge_request: state.merge_request.clone(),
        ..FlowPlan::default()
    };
    match resume {
        FlowResume::Continue => {
            plan.steps = state.steps[state.next.min(state.steps.len())..].to_vec();
        }
        FlowResume::Abort => {
            plan.warnings.push(
                "The git operation in progress is aborted and the branch the flow started on is \
                 checked out again; steps already completed stay (Undo takes them back)"
                    .to_string(),
            );
        }
    }
    plan
}

/// `FlowPlan`.
#[allow(clippy::result_large_err)]
pub(crate) fn plan_read(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    args: &Value,
) -> Result<Value, Failure> {
    let req = parse(args)?;
    encode(&plan_of(backend, repo, &req)?)
}

/// `FlowRun`.
#[allow(clippy::result_large_err)]
pub(crate) async fn run(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    args: &Value,
) -> Result<Value, Failure> {
    let req = parse(args)?;
    let outcome = match req.resume {
        Some(FlowResume::Continue) => exec::resume_continue(backend, repo, &req).await?,
        Some(FlowResume::Abort) => exec::resume_abort(backend, repo, &req)?,
        None => exec::start_run(backend, repo, &req).await?,
    };
    encode(&outcome)
}

/// The branches whose refs or config a run may change, for the journal's
/// write-ahead capture (pins and config sections).
pub(crate) fn involved_branches(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    args: &Value,
) -> Vec<String> {
    let Ok(req) = parse(args) else {
        return Vec::new();
    };
    let steps = match req.resume {
        Some(_) => state::load(repo).map(|s| s.steps).unwrap_or_default(),
        None => plan_of(backend, repo, &req)
            .map(|p| p.steps)
            .unwrap_or_default(),
    };
    let mut out: Vec<String> = Vec::new();
    let mut add = |name: &str| {
        if !out.iter().any(|n| n == name) {
            out.push(name.to_string());
        }
    };
    for step in &steps {
        match step {
            FlowStep::CreateBranch { name, .. } | FlowStep::DeleteBranch { name } => add(name),
            FlowStep::RecordParent { branch, .. }
            | FlowStep::SetUpstream { branch, .. }
            | FlowStep::Rebase { branch, .. } => add(branch),
            FlowStep::Merge { into, .. } => add(into),
            _ => {}
        }
    }
    // An abort goes back to the branch the flow started on.
    if let Some(state) = state::load(repo).filter(|_| req.resume.is_some()) {
        if let Some(head) = &state.original_head {
            add(head);
        }
    }
    out
}

/// The label of the journal entry.
pub(crate) fn label(request: &gitpoulpi_ipc::Request) -> String {
    let req = parse(&request.args).unwrap_or_default();
    let what = match (&req.branch, &req.topic) {
        (Some(b), _) => format!(" {b}"),
        (None, Some(t)) => format!(" {t}"),
        _ => String::new(),
    };
    match req.resume {
        Some(FlowResume::Continue) => format!("Flow {} (continued){what}", req.action.word()),
        Some(FlowResume::Abort) => format!("Flow {} (aborted){what}", req.action.word()),
        None => format!("Flow {}{what}", req.action.word()),
    }
}
