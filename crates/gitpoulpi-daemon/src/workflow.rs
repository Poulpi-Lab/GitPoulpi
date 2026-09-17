//! The `Workflow` read: the repository's branching workflow (shared file plus
//! per-user override), every branch classified, with parents and counts.
//!
//! Pure classification lives in `gitpoulpi_core::workflow`; this gathers the
//! inputs (files, refs, local config) and hands them over.

use std::path::Path;

use gitpoulpi_core::domain::{RefKind, RepoRef};
use gitpoulpi_core::workflow::{
    build_report, import_git_flow, import_git_town, parse_recorded_parents, BranchInput,
    ReportSources, WORKFLOW_CONFIG_PREFIXES,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_session::LoadedWorkflow;
use serde_json::Value;

use crate::dispatch::Failure;

/// `session.per_repo_state_dir` of the ambient configuration.
pub(crate) fn state_dir() -> String {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string())
}

/// A fingerprint of the two files the answer depends on besides git's own
/// state (the read cache fingerprints that): their modification time and
/// size, so editing either invalidates a cached answer.
pub(crate) fn files_stamp(root: &Path) -> String {
    let dir = state_dir();
    let stamp = |p: std::path::PathBuf| match std::fs::metadata(&p)
        .and_then(|m| m.modified().map(|t| (t, m.len())))
    {
        Ok((t, len)) => format!("{:?}/{len}", t),
        Err(_) => "-".to_string(),
    };
    format!(
        "{}|{}",
        stamp(gitpoulpi_session::workflow_path_for(root, &dir)),
        stamp(gitpoulpi_session::RepoState::path_for(root, &dir)),
    )
}

/// `Failure` is the dispatcher's own (large) error tuple.
#[allow(clippy::result_large_err)]
pub(crate) fn read(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Value, Failure> {
    let fail = |message: String| -> Failure { (message, None, None) };
    let from_backend =
        |e: gitpoulpi_core::Error| -> Failure { (e.to_string(), e.typed(), e.detail().cloned()) };

    let loaded = LoadedWorkflow::load(&repo.root, &state_dir()).map_err(fail)?;
    let refs = backend.refs(repo).map_err(from_backend)?;
    let entries = backend
        .config_entries(repo, WORKFLOW_CONFIG_PREFIXES)
        .map_err(from_backend)?;

    let mut branches = Vec::new();
    for r in &refs {
        match &r.kind {
            RefKind::LocalBranch { name } => branches.push(BranchInput {
                name: name.clone(),
                remote: None,
                tip: r.target,
                is_head: r.is_head,
            }),
            RefKind::RemoteBranch { remote, name } if name != "HEAD" => {
                branches.push(BranchInput {
                    name: name.clone(),
                    remote: Some(remote.clone()),
                    tip: r.target,
                    is_head: false,
                })
            }
            _ => {}
        }
    }

    let imports = [import_git_flow(&entries), import_git_town(&entries)]
        .into_iter()
        .flatten()
        .collect();
    let sources = ReportSources {
        shared_file: loaded.shared.is_some(),
        user_override: loaded.user.is_some(),
    };
    let mut report = build_report(
        loaded.effective(),
        sources,
        imports,
        &branches,
        &parse_recorded_parents(&entries),
        &mut |tip, base| backend.ahead_behind(repo, tip, base).ok(),
    );
    if let Some(shared) = &loaded.shared {
        report.warnings.extend(
            shared
                .unknown_keys
                .iter()
                .map(|k| format!("workflow.toml: unknown key `{k}` (kept when saved)")),
        );
    }
    serde_json::to_value(report).map_err(|e| fail(format!("failed to encode result: {e}")))
}
