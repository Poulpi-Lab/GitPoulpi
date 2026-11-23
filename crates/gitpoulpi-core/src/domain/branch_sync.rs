//! "Pull all branches": bring every local branch that tracks an upstream up
//! to date in one run - fetch, fast-forward the branches that are only
//! behind, integrate the checked-out branch like the Pull button does.
//!
//! Pure data and the planner, no I/O. The engine lives in the daemon
//! (`branch_sync.rs`); the CLI, the MCP server and the TUI read these types.
//!
//! The rules (the owner's decisions of the feature):
//!
//! * A branch that is not checked out is **never merged or rebased**: only
//!   behind (a fast-forward, a ref move) is applied; a diverged one is
//!   skipped and reported.
//! * The checked-out branch is integrated through the ordinary pull path, in
//!   the mode the caller chose (`head_mode`, else the configured pull mode).
//! * A behind branch checked out in another worktree is skipped or
//!   fast-forwarded in that worktree, as the caller asked
//!   ([`WorktreePolicy`]).
//! * Only the primary upstream of a branch counts; GitPoulpi's extra pull
//!   sources are ignored. A branch with no upstream is not part of the run.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{FetchOptions, Oid, PullMode};

/// What to do with a behind branch that is checked out in another worktree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreePolicy {
    /// Leave it, report it as `checked_out_elsewhere`.
    #[default]
    Skip,
    /// Fast-forward it inside that worktree (refused, and reported as
    /// `failed`, when local changes there would be overwritten).
    Update,
}

/// Arguments of `PullAllBranches`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PullAllBranchesRequest {
    /// Fetch the upstream remotes of the local branches first (the default).
    /// Off: work from the remote-tracking refs as they are.
    pub fetch: bool,
    /// Options of those fetches (`prune`).
    pub fetch_opts: FetchOptions,
    /// How the checked-out branch is integrated. `None`: the configured pull
    /// mode (what the Pull button uses).
    pub head_mode: Option<PullMode>,
    /// What to do with behind branches checked out in other worktrees.
    pub worktrees: WorktreePolicy,
}

impl Default for PullAllBranchesRequest {
    fn default() -> Self {
        PullAllBranchesRequest {
            fetch: true,
            fetch_opts: FetchOptions::default(),
            head_mode: None,
            worktrees: WorktreePolicy::Skip,
        }
    }
}

/// How one branch ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchSyncStatus {
    /// Moved to its upstream (a fast-forward, or the checked-out branch
    /// merged or rebased).
    Updated,
    UpToDate,
    /// Only ahead of its upstream: nothing to pull.
    Ahead,
    /// Has commits its upstream lacks and the reverse: not touched (a branch
    /// that is not checked out is never merged or rebased; a fast-forward
    /// only checked-out branch cannot fast-forward).
    Diverged,
    /// The remote-tracking branch it follows no longer exists.
    UpstreamGone,
    /// Behind, but checked out in another worktree and the run was told to
    /// skip those.
    CheckedOutElsewhere,
    /// Behind and checked out in another worktree, which was fast-forwarded.
    WorktreeUpdated,
    /// Its upstream is another local branch (`branch.<x>.remote = .`):
    /// nothing was downloaded for it.
    LocalUpstream,
    /// The fetch of its remote failed, so it was left alone.
    FetchFailed,
    /// The move was refused or failed (see `detail`).
    Failed,
    /// The checked-out branch's integration stopped on conflicts: the
    /// repository is in the same state a conflicting pull leaves.
    Conflicted,
}

impl BranchSyncStatus {
    /// The branch moved (counts for "N updated").
    pub fn moved(self) -> bool {
        matches!(
            self,
            BranchSyncStatus::Updated | BranchSyncStatus::WorktreeUpdated
        )
    }

    /// A status that makes the run's exit code non-zero.
    pub fn is_failure(self) -> bool {
        matches!(
            self,
            BranchSyncStatus::Failed | BranchSyncStatus::FetchFailed
        )
    }
}

/// One branch of the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchSyncItem {
    pub branch: String,
    /// The upstream's shorthand name (`origin/feature/x`).
    pub upstream: Option<String>,
    pub status: BranchSyncStatus,
    /// The tip before the run.
    pub from: Option<Oid>,
    /// The tip after it (equal to `from` when nothing moved).
    pub to: Option<Oid>,
    /// Commits the branch has that its upstream lacks / the reverse, as
    /// the run found them (before any move).
    pub ahead: u32,
    pub behind: u32,
    /// The other worktree the branch is checked out in, if any.
    #[serde(default)]
    pub worktree: Option<PathBuf>,
    /// A sentence for `failed`, `fetch_failed` and `conflicted`.
    #[serde(default)]
    pub detail: Option<String>,
    /// The branch checked out in the repository the run was started in.
    #[serde(default)]
    pub is_head: bool,
}

/// A remote whose fetch failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteFetchFailure {
    pub remote: String,
    pub error: String,
}

/// Answer of `PullAllBranches`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullAllBranchesOutcome {
    /// The remotes fetched successfully.
    pub fetched: Vec<String>,
    pub fetch_failed: Vec<RemoteFetchFailure>,
    /// Every branch that has an upstream, checked-out one first, then by
    /// name.
    pub branches: Vec<BranchSyncItem>,
    /// The mode the checked-out branch was integrated in; `None` when there
    /// was no such step (detached HEAD, no upstream, nothing behind).
    #[serde(default)]
    pub head_mode: Option<PullMode>,
}

impl PullAllBranchesOutcome {
    /// Branches that moved.
    pub fn updated(&self) -> usize {
        self.branches.iter().filter(|b| b.status.moved()).count()
    }

    /// The checked-out branch's integration stopped on conflicts.
    pub fn conflicted(&self) -> bool {
        self.branches
            .iter()
            .any(|b| b.status == BranchSyncStatus::Conflicted)
    }

    /// Something failed (a branch move or a fetch).
    pub fn has_failures(&self) -> bool {
        !self.fetch_failed.is_empty() || self.branches.iter().any(|b| b.status.is_failure())
    }
}

// ------------------------------------------------------------------ planner

/// Where a branch's upstream lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamKind {
    /// A remote-tracking branch.
    Remote,
    /// Another local branch (`branch.<x>.remote = .`).
    Local,
}

/// What the daemon read about a branch's upstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamFacts {
    pub name: String,
    pub kind: UpstreamKind,
    /// The upstream's tip; `None` when the tracking ref is gone.
    pub target: Option<Oid>,
    /// `(ahead, behind)` of the branch against the upstream (0, 0 when the
    /// tracking ref is gone).
    pub ahead: u32,
    pub behind: u32,
    /// The fetch of its remote failed.
    pub fetch_failed: bool,
}

/// What the daemon read about a local branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchFacts {
    pub branch: String,
    pub tip: Oid,
    /// Checked out in the worktree the run was started in.
    pub is_head: bool,
    /// Checked out in another linked worktree, at this path.
    pub other_worktree: Option<PathBuf>,
    pub upstream: Option<UpstreamFacts>,
}

/// What the run does with a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedAction {
    /// Nothing to do: this is its final status.
    Report(BranchSyncStatus),
    /// Move the ref to `to` (not checked out anywhere).
    FastForward { to: Oid },
    /// Fast-forward inside the other worktree.
    FastForwardInWorktree { to: Oid, path: PathBuf },
    /// The checked-out branch: integrate through the pull path.
    IntegrateHead,
}

/// One planned branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedBranch {
    pub branch: String,
    pub upstream: String,
    pub from: Oid,
    pub ahead: u32,
    pub behind: u32,
    pub worktree: Option<PathBuf>,
    pub is_head: bool,
    pub action: PlannedAction,
}

/// Decides, per branch with an upstream, what the run does. Branches
/// without one are not part of the run and are left out. The checked-out
/// branch comes first, the others by name.
pub fn plan(facts: &[BranchFacts], worktrees: WorktreePolicy) -> Vec<PlannedBranch> {
    use BranchSyncStatus as S;
    let mut planned: Vec<PlannedBranch> = facts
        .iter()
        .filter_map(|f| {
            let up = f.upstream.as_ref()?;
            let action = if up.kind == UpstreamKind::Local {
                PlannedAction::Report(S::LocalUpstream)
            } else if up.fetch_failed {
                PlannedAction::Report(S::FetchFailed)
            } else if let Some(target) = up.target {
                match (up.ahead, up.behind) {
                    (0, 0) => PlannedAction::Report(S::UpToDate),
                    (_, 0) => PlannedAction::Report(S::Ahead),
                    _ if f.is_head => PlannedAction::IntegrateHead,
                    (ahead, _) if ahead > 0 => PlannedAction::Report(S::Diverged),
                    _ => match (&f.other_worktree, worktrees) {
                        (Some(path), WorktreePolicy::Update) => {
                            PlannedAction::FastForwardInWorktree {
                                to: target,
                                path: path.clone(),
                            }
                        }
                        (Some(_), WorktreePolicy::Skip) => {
                            PlannedAction::Report(S::CheckedOutElsewhere)
                        }
                        (None, _) => PlannedAction::FastForward { to: target },
                    },
                }
            } else {
                PlannedAction::Report(S::UpstreamGone)
            };
            Some(PlannedBranch {
                branch: f.branch.clone(),
                upstream: up.name.clone(),
                from: f.tip,
                ahead: up.ahead,
                behind: up.behind,
                worktree: f.other_worktree.clone(),
                is_head: f.is_head,
                action,
            })
        })
        .collect();
    planned.sort_by(|a, b| {
        b.is_head
            .cmp(&a.is_head)
            .then_with(|| a.branch.cmp(&b.branch))
    });
    planned
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(n: u8) -> Oid {
        Oid::from_hex(&format!("{n:02x}").repeat(20)).unwrap()
    }

    fn facts(name: &str, ahead: u32, behind: u32) -> BranchFacts {
        BranchFacts {
            branch: name.into(),
            tip: oid(1),
            is_head: false,
            other_worktree: None,
            upstream: Some(UpstreamFacts {
                name: format!("origin/{name}"),
                kind: UpstreamKind::Remote,
                target: Some(oid(2)),
                ahead,
                behind,
                fetch_failed: false,
            }),
        }
    }

    fn action_of(f: BranchFacts, w: WorktreePolicy) -> PlannedAction {
        plan(&[f], w).remove(0).action
    }

    #[test]
    fn a_branch_without_an_upstream_is_not_part_of_the_run() {
        let mut f = facts("a", 0, 0);
        f.upstream = None;
        assert!(plan(&[f], WorktreePolicy::Skip).is_empty());
    }

    #[test]
    fn equal_and_ahead_branches_are_reported_not_touched() {
        use BranchSyncStatus as S;
        assert_eq!(
            action_of(facts("a", 0, 0), WorktreePolicy::Skip),
            PlannedAction::Report(S::UpToDate)
        );
        assert_eq!(
            action_of(facts("a", 3, 0), WorktreePolicy::Skip),
            PlannedAction::Report(S::Ahead)
        );
    }

    #[test]
    fn a_behind_branch_fast_forwards_to_its_upstream_tip() {
        assert_eq!(
            action_of(facts("a", 0, 2), WorktreePolicy::Skip),
            PlannedAction::FastForward { to: oid(2) }
        );
    }

    #[test]
    fn a_diverged_branch_that_is_not_checked_out_is_skipped() {
        assert_eq!(
            action_of(facts("a", 1, 2), WorktreePolicy::Update),
            PlannedAction::Report(BranchSyncStatus::Diverged)
        );
    }

    #[test]
    fn the_checked_out_branch_is_integrated_when_behind_even_if_diverged() {
        let mut f = facts("a", 0, 2);
        f.is_head = true;
        assert_eq!(
            action_of(f.clone(), WorktreePolicy::Skip),
            PlannedAction::IntegrateHead
        );
        f.upstream.as_mut().unwrap().ahead = 4;
        assert_eq!(
            action_of(f, WorktreePolicy::Skip),
            PlannedAction::IntegrateHead
        );
    }

    #[test]
    fn a_branch_in_another_worktree_follows_the_policy() {
        let mut f = facts("a", 0, 1);
        f.other_worktree = Some(PathBuf::from("/w/a"));
        assert_eq!(
            action_of(f.clone(), WorktreePolicy::Skip),
            PlannedAction::Report(BranchSyncStatus::CheckedOutElsewhere)
        );
        assert_eq!(
            action_of(f.clone(), WorktreePolicy::Update),
            PlannedAction::FastForwardInWorktree {
                to: oid(2),
                path: PathBuf::from("/w/a")
            }
        );
        // Diverged there: never touched, whatever the policy.
        f.upstream.as_mut().unwrap().ahead = 1;
        assert_eq!(
            action_of(f, WorktreePolicy::Update),
            PlannedAction::Report(BranchSyncStatus::Diverged)
        );
    }

    #[test]
    fn a_gone_local_or_unfetched_upstream_is_reported() {
        use BranchSyncStatus as S;
        let mut gone = facts("a", 0, 0);
        gone.upstream.as_mut().unwrap().target = None;
        assert_eq!(
            action_of(gone, WorktreePolicy::Skip),
            PlannedAction::Report(S::UpstreamGone)
        );
        let mut local = facts("a", 0, 3);
        local.upstream.as_mut().unwrap().kind = UpstreamKind::Local;
        assert_eq!(
            action_of(local, WorktreePolicy::Skip),
            PlannedAction::Report(S::LocalUpstream)
        );
        let mut failed = facts("a", 0, 3);
        failed.upstream.as_mut().unwrap().fetch_failed = true;
        assert_eq!(
            action_of(failed, WorktreePolicy::Skip),
            PlannedAction::Report(S::FetchFailed)
        );
    }

    #[test]
    fn the_checked_out_branch_comes_first_then_by_name() {
        let mut head = facts("zeta", 0, 0);
        head.is_head = true;
        let planned = plan(
            &[facts("b", 0, 0), head, facts("a", 0, 0)],
            WorktreePolicy::Skip,
        );
        let names: Vec<_> = planned.iter().map(|p| p.branch.as_str()).collect();
        assert_eq!(names, ["zeta", "a", "b"]);
    }

    #[test]
    fn the_request_defaults_to_fetching_and_skipping_worktrees() {
        let req: PullAllBranchesRequest = serde_json::from_str("{}").unwrap();
        assert!(req.fetch);
        assert_eq!(req.head_mode, None);
        assert_eq!(req.worktrees, WorktreePolicy::Skip);
        let req: PullAllBranchesRequest =
            serde_json::from_str(r#"{"fetch":false,"head_mode":"rebase","worktrees":"update"}"#)
                .unwrap();
        assert!(!req.fetch);
        assert_eq!(req.head_mode, Some(PullMode::Rebase));
        assert_eq!(req.worktrees, WorktreePolicy::Update);
    }

    #[test]
    fn the_outcome_counts_moves_and_failures() {
        let item = |status| BranchSyncItem {
            branch: "a".into(),
            upstream: None,
            status,
            from: None,
            to: None,
            ahead: 0,
            behind: 0,
            worktree: None,
            detail: None,
            is_head: false,
        };
        let mut out = PullAllBranchesOutcome {
            branches: vec![
                item(BranchSyncStatus::Updated),
                item(BranchSyncStatus::WorktreeUpdated),
                item(BranchSyncStatus::Diverged),
            ],
            ..PullAllBranchesOutcome::default()
        };
        assert_eq!(out.updated(), 2);
        assert!(!out.has_failures());
        assert!(!out.conflicted());
        out.branches.push(item(BranchSyncStatus::Failed));
        assert!(out.has_failures());
        out.branches.push(item(BranchSyncStatus::Conflicted));
        assert!(out.conflicted());
    }
}
