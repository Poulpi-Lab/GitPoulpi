//! Grouped push / pull over a branch's several remotes (see
//! `gitpoulpi_core::domain::remote_group`). This slice is the planning read
//! (`RemoteGroupPlan`); the run engine builds on the same module.

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

mod plan;

use gitpoulpi_core::domain::{RemoteGroupPlanRequest, RepoRef};
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
