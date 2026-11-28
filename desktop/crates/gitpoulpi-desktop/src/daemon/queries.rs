//! Typed reads and diffs on top of [`Backend::call`].

use std::future::Future;

use gitpoulpi_core::domain::{
    CommitGraph, CommitNode, DiffTarget, FileDiff, LogOptions, Oid, OperationKind, Ref, Stash,
    WorkingTree, Worktree,
};

use super::Backend;

type Fut<T> = std::pin::Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'static>>;

/// Which commits a Log walk starts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
    /// Every branch, remote branch and tag (`git log --branches --tags --remotes`).
    AllRefs,
    /// Only what HEAD reaches; the walk for a detached HEAD.
    Head,
}

impl LogSource {
    pub(super) fn options(self, skip: u32, limit: Option<u32>) -> LogOptions {
        match self {
            LogSource::AllRefs => LogOptions {
                limit,
                skip,
                all_refs: true,
                ..LogOptions::default()
            },
            LogSource::Head => LogOptions {
                limit,
                skip,
                start_ref: Some("HEAD".into()),
                ..LogOptions::default()
            },
        }
    }
}

impl Backend {
    pub fn log_page(&self, source: LogSource, skip: u32, limit: u32) -> Fut<Vec<CommitNode>> {
        let opts = source.options(skip, Some(limit));
        let fut = self.call::<CommitGraph>(OperationKind::Log, serde_json::json!({ "opts": opts }));
        Box::pin(async move { fut.await.map(|g| g.nodes) })
    }

    pub fn count(&self, source: LogSource) -> Fut<u64> {
        let opts = source.options(0, None);
        Box::pin(self.call(
            OperationKind::CountCommits,
            serde_json::json!({ "opts": opts }),
        ))
    }

    /// The commit HEAD points at; an error on a repository without commits.
    pub fn head(&self) -> Fut<Oid> {
        Box::pin(self.call(OperationKind::Resolve, serde_json::json!({ "rev": "HEAD" })))
    }

    pub fn refs(&self) -> Fut<Vec<Ref>> {
        Box::pin(self.call(OperationKind::Refs, serde_json::json!({})))
    }

    pub fn stashes(&self) -> Fut<Vec<Stash>> {
        Box::pin(self.call(OperationKind::Stashes, serde_json::json!({})))
    }

    pub fn worktrees(&self) -> Fut<Vec<Worktree>> {
        Box::pin(self.call(OperationKind::Worktrees, serde_json::json!({})))
    }

    pub fn status(&self) -> Fut<WorkingTree> {
        Box::pin(self.call(OperationKind::Status, serde_json::json!({})))
    }

    /// Working tree diff: staged (index against HEAD) or unstaged (working tree against index).
    pub fn working_diff(&self, staged: bool) -> Fut<Vec<FileDiff>> {
        let target = if staged {
            DiffTarget::IndexVsHead
        } else {
            DiffTarget::WorkingTreeVsIndex
        };
        Box::pin(self.call(OperationKind::Diff, serde_json::json!({ "target": target })))
    }

    pub fn commit_diff(&self, oid: Oid) -> Fut<Vec<FileDiff>> {
        let target = DiffTarget::CommitVsParent(oid);
        Box::pin(self.call(OperationKind::Diff, serde_json::json!({ "target": target })))
    }
}
