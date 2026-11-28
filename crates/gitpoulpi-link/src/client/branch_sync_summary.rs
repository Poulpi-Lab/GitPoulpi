//! The status line (and the Activity Log lines) after "Pull all branches":
//! "updated 3 branches, skipped 2 (1 diverged, 1 checked out in another
//! worktree)". Pure, so the wording is tested without a daemon.

use gitpoulpi_core::domain::{BranchSyncItem, BranchSyncStatus, PullAllBranchesOutcome};
use gitpoulpi_i18n::msg::link as l;

use crate::oid_text::short_oid as short;

/// Whether the branch was left alone for a reason the user may want to act on
/// (everything but moved, up to date, ahead and the conflict, which has its
/// own line).
fn skipped(status: BranchSyncStatus) -> bool {
    matches!(
        status,
        BranchSyncStatus::Diverged
            | BranchSyncStatus::UpstreamGone
            | BranchSyncStatus::CheckedOutElsewhere
            | BranchSyncStatus::LocalUpstream
            | BranchSyncStatus::FetchFailed
            | BranchSyncStatus::Failed
    )
}

/// "1 diverged, 1 checked out in another worktree", reasons in a fixed order.
fn why(outcome: &PullAllBranchesOutcome) -> String {
    let count = |status: BranchSyncStatus| {
        outcome
            .branches
            .iter()
            .filter(|b| b.status == status)
            .count() as u64
    };
    let mut parts = Vec::new();
    type Reason = (BranchSyncStatus, fn(u64) -> String);
    let reasons: [Reason; 6] = [
        (BranchSyncStatus::Diverged, l::skip_diverged),
        (BranchSyncStatus::CheckedOutElsewhere, l::skip_worktree),
        (BranchSyncStatus::UpstreamGone, l::skip_upstream_gone),
        (BranchSyncStatus::LocalUpstream, l::skip_local_upstream),
        (BranchSyncStatus::FetchFailed, l::skip_not_downloaded),
        (BranchSyncStatus::Failed, l::skip_branch_failed),
    ];
    for (status, word) in reasons {
        let n = count(status);
        if n > 0 {
            parts.push(word(n));
        }
    }
    parts.join(", ")
}

/// The one status line of a finished run.
pub(super) fn text(outcome: PullAllBranchesOutcome) -> String {
    let updated = outcome.updated() as u64;
    let left = outcome
        .branches
        .iter()
        .filter(|b| skipped(b.status))
        .count() as u64;
    let line = if let Some(item) = outcome
        .branches
        .iter()
        .find(|b| b.status == BranchSyncStatus::Conflicted)
    {
        l::pulled_all_conflict(updated, &item.branch)
    } else if left == 0 {
        if updated == 0 {
            l::PULLED_ALL_NOTHING.get().to_string()
        } else {
            l::pulled_all_updated(updated)
        }
    } else if updated == 0 {
        l::pulled_all_none_skipped(left, why(&outcome))
    } else {
        l::pulled_all_updated_skipped(updated, left, why(&outcome))
    };
    if outcome.fetch_failed.is_empty() {
        line
    } else {
        let remotes: Vec<&str> = outcome
            .fetch_failed
            .iter()
            .map(|f| f.remote.as_str())
            .collect();
        l::pulled_all_fetch_failed(line, remotes.join(", "))
    }
}

/// One Activity Log line per branch that has something to say (a branch that
/// is up to date or only ahead is one short line too, so the log shows that
/// it was looked at).
pub(super) fn lines(outcome: &PullAllBranchesOutcome) -> Vec<String> {
    outcome.branches.iter().map(line).collect()
}

fn line(item: &BranchSyncItem) -> String {
    let branch = item.branch.as_str();
    let upstream = item.upstream.as_deref().unwrap_or("");
    let path = item
        .worktree
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    match item.status {
        BranchSyncStatus::Updated => l::pulled_all_line_updated(
            branch,
            item.from.map(short).unwrap_or_default(),
            item.to.map(short).unwrap_or_default(),
        ),
        BranchSyncStatus::UpToDate => l::pulled_all_line_up_to_date(branch),
        BranchSyncStatus::Ahead => l::pulled_all_line_ahead(branch, upstream),
        BranchSyncStatus::Diverged => l::pulled_all_line_diverged(branch, upstream),
        BranchSyncStatus::UpstreamGone => l::pulled_all_line_upstream_gone(branch, upstream),
        BranchSyncStatus::CheckedOutElsewhere => l::pulled_all_line_elsewhere(branch, path),
        BranchSyncStatus::WorktreeUpdated => l::pulled_all_line_worktree(branch, path),
        BranchSyncStatus::LocalUpstream => l::pulled_all_line_local_upstream(branch, upstream),
        BranchSyncStatus::FetchFailed => l::pulled_all_line_not_downloaded(branch),
        BranchSyncStatus::Failed => {
            l::pulled_all_line_failed(branch, item.detail.as_deref().unwrap_or(""))
        }
        BranchSyncStatus::Conflicted => l::pulled_all_line_conflicted(branch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, RemoteFetchFailure};

    fn item(branch: &str, status: BranchSyncStatus) -> BranchSyncItem {
        BranchSyncItem {
            branch: branch.into(),
            upstream: Some(format!("origin/{branch}")),
            status,
            from: Some(Oid([1; 20])),
            to: Some(Oid([2; 20])),
            ahead: 0,
            behind: 0,
            worktree: Some("/work/orbit".into()),
            detail: Some("local changes".into()),
            is_head: false,
        }
    }

    fn outcome(items: Vec<BranchSyncItem>) -> PullAllBranchesOutcome {
        PullAllBranchesOutcome {
            branches: items,
            ..Default::default()
        }
    }

    #[test]
    fn nothing_moved_and_nothing_left_says_everything_is_up_to_date() {
        let o = outcome(vec![
            item("main", BranchSyncStatus::UpToDate),
            item("feature/x", BranchSyncStatus::Ahead),
        ]);
        assert_eq!(text(o), "all branches are up to date");
        assert_eq!(text(outcome(vec![])), "all branches are up to date");
    }

    #[test]
    fn the_count_of_updated_branches_is_plural_aware() {
        assert_eq!(
            text(outcome(vec![item("a", BranchSyncStatus::Updated)])),
            "updated 1 branch"
        );
        assert_eq!(
            text(outcome(vec![
                item("a", BranchSyncStatus::Updated),
                item("b", BranchSyncStatus::WorktreeUpdated),
            ])),
            "updated 2 branches"
        );
    }

    #[test]
    fn skipped_branches_are_counted_with_their_reasons_in_a_fixed_order() {
        let o = outcome(vec![
            item("a", BranchSyncStatus::Updated),
            item("b", BranchSyncStatus::CheckedOutElsewhere),
            item("c", BranchSyncStatus::Diverged),
            item("d", BranchSyncStatus::Diverged),
        ]);
        assert_eq!(
            text(o),
            "updated 1 branch, skipped 3 (2 diverged, 1 checked out in another worktree)"
        );
        let none = outcome(vec![item("b", BranchSyncStatus::UpstreamGone)]);
        assert_eq!(
            text(none),
            "nothing updated, skipped 1 (1 with the upstream gone)"
        );
    }

    #[test]
    fn a_conflict_on_the_checked_out_branch_is_named_with_the_branches_updated_before() {
        let mut head = item("main", BranchSyncStatus::Conflicted);
        head.is_head = true;
        let o = outcome(vec![head, item("a", BranchSyncStatus::Updated)]);
        assert_eq!(
            text(o),
            "conflicts on main: resolve them or abort, as for a normal pull (1 other branch updated)"
        );
    }

    #[test]
    fn a_failed_download_is_appended_with_the_remotes() {
        let mut o = outcome(vec![item("a", BranchSyncStatus::Updated)]);
        o.fetch_failed = vec![
            RemoteFetchFailure {
                remote: "origin".into(),
                error: "offline".into(),
            },
            RemoteFetchFailure {
                remote: "acme".into(),
                error: "offline".into(),
            },
        ];
        assert_eq!(text(o), "updated 1 branch; fetch failed for origin, acme");
    }

    #[test]
    fn every_branch_gets_one_activity_line() {
        let o = outcome(vec![
            item("a", BranchSyncStatus::Updated),
            item("b", BranchSyncStatus::UpToDate),
            item("c", BranchSyncStatus::CheckedOutElsewhere),
            item("d", BranchSyncStatus::Failed),
        ]);
        let lines = lines(&o);
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("a: updated "), "{}", lines[0]);
        assert_eq!(lines[1], "b: up to date");
        assert_eq!(lines[2], "c: checked out in /work/orbit, skipped");
        assert_eq!(lines[3], "d: failed (local changes)");
    }
}
