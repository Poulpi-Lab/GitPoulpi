//! `gitpoulpi pull --all-branches`: one `PullAllBranches` request, the
//! daemon's `PullAllBranchesOutcome` as the JSON, one line per branch for a
//! human.
//!
//! Exit codes: 0 when every branch is updated, up to date, ahead or
//! deliberately skipped (a conflict on the checked-out branch is 0 like a
//! conflicting `pull`: the JSON says `conflicted`), 4 when a branch move or a
//! fetch failed, 2 when the daemon refused (a pending auto-stash, or local
//! changes in the way with the setting on `ask`).

use gitpoulpi_core::domain::{
    BranchSyncItem, BranchSyncStatus, FetchOptions, OperationKind, PullAllBranchesOutcome,
    PullAllBranchesRequest, PullMode, WorktreePolicy,
};
use gitpoulpi_core::TypedError;
use serde_json::json;

use crate::cli::{PullModeArg, PullWorktreesArg};
use crate::client::Session;

use super::*;

/// The flags of `pull --all-branches`.
pub(super) struct AllBranches {
    pub mode: Option<PullModeArg>,
    pub no_fetch: bool,
    pub prune: bool,
    pub worktrees: Option<PullWorktreesArg>,
    pub autostash: bool,
}

fn refused(reason: &str, message: String) -> Emitted {
    let mut out = emitted(
        json!({ "refused": { "reason": reason, "message": message } }),
        format!("refused: {message}"),
    );
    out.exit_code = 2;
    out
}

fn status_word(status: BranchSyncStatus) -> &'static str {
    match status {
        BranchSyncStatus::Updated => "updated",
        BranchSyncStatus::UpToDate => "up to date",
        BranchSyncStatus::Ahead => "ahead of its upstream, nothing to pull",
        BranchSyncStatus::Diverged => "diverged, skipped",
        BranchSyncStatus::UpstreamGone => "upstream is gone",
        BranchSyncStatus::CheckedOutElsewhere => "checked out in another worktree, skipped",
        BranchSyncStatus::WorktreeUpdated => "updated in its worktree",
        BranchSyncStatus::LocalUpstream => "tracks a local branch, skipped",
        BranchSyncStatus::FetchFailed => "fetch failed, left alone",
        BranchSyncStatus::Failed => "failed",
        BranchSyncStatus::Conflicted => "stopped on conflicts",
    }
}

fn line(item: &BranchSyncItem) -> String {
    let mut line = format!("{}: {}", item.branch, status_word(item.status));
    if item.status.moved() {
        if let (Some(from), Some(to)) = (item.from, item.to) {
            line.push_str(&format!(" ({} -> {})", short(from), short(to)));
        }
    }
    if let Some(path) = &item.worktree {
        if matches!(
            item.status,
            BranchSyncStatus::CheckedOutElsewhere | BranchSyncStatus::WorktreeUpdated
        ) {
            line.push_str(&format!(" [{}]", path.display()));
        }
    }
    if let Some(detail) = &item.detail {
        line.push_str(&format!(": {detail}"));
    }
    line
}

fn human(outcome: &PullAllBranchesOutcome) -> String {
    let mut lines: Vec<String> = Vec::new();
    for failed in &outcome.fetch_failed {
        lines.push(format!("fetch {} failed: {}", failed.remote, failed.error));
    }
    lines.extend(outcome.branches.iter().map(line));
    let mut summary = match outcome.updated() {
        0 => "no branch needed an update".to_string(),
        1 => "1 branch updated".to_string(),
        n => format!("{n} branches updated"),
    };
    if outcome.conflicted() {
        summary.push_str("; the checked-out branch stopped on conflicts: resolve them or abort");
    }
    lines.push(summary);
    lines.join("\n")
}

pub(super) async fn pull_all_branches(
    session: &mut Session,
    flags: AllBranches,
) -> Result<Emitted, String> {
    let request = PullAllBranchesRequest {
        fetch: !flags.no_fetch,
        fetch_opts: FetchOptions {
            prune: flags.prune,
            ..FetchOptions::default()
        },
        head_mode: flags.mode.map(|m| match m {
            PullModeArg::Merge => PullMode::Merge,
            PullModeArg::FfOnly => PullMode::FastForwardOnly,
            PullModeArg::Rebase => PullMode::Rebase,
        }),
        worktrees: match flags.worktrees {
            Some(PullWorktreesArg::Update) => WorktreePolicy::Update,
            _ => WorktreePolicy::Skip,
        },
    };
    let args = with_autostash(
        serde_json::to_value(&request).map_err(|e| e.to_string())?,
        flags.autostash,
    );
    match session
        .call_typed::<PullAllBranchesOutcome>(OperationKind::PullAllBranches, args)
        .await?
    {
        Ok(outcome) => {
            let mut out = emitted(to_value(&outcome)?, human(&outcome));
            if outcome.has_failures() {
                out.exit_code = 4;
            }
            Ok(out)
        }
        Err(e) => match &e.kind {
            Some(TypedError::NeedsAutoStash { .. }) => Ok(refused(
                "needs_autostash",
                format!("{} (pass --autostash to stash and restore them)", e.message),
            )),
            Some(TypedError::PendingAutoStash { .. }) => {
                Ok(refused("pending_autostash", e.message))
            }
            _ => Err(e.message),
        },
    }
}
