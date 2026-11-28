//! Typed reads and diffs on top of [`Backend::call`].

use std::future::Future;

use gitpoulpi_core::domain::{
    DiffTarget, FileDiff, Oid, OperationKind, Ref, Stash, Submodule, WorkingTree, Worktree,
};

use super::Backend;

type Fut<T> = std::pin::Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'static>>;

impl Backend {
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

    pub fn submodules(&self) -> Fut<Vec<Submodule>> {
        Box::pin(self.call(OperationKind::Submodules, serde_json::json!({})))
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
