//! Expand and collapse operations of a tree.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// One expand/collapse operation. "Below" acts on a folder and everything
/// under it; the others on every folder of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeFold {
    ExpandAll,
    CollapseAll,
    ExpandBelow,
    CollapseBelow,
}

use crate::keymap::Action;
use gitpoulpi_i18n::msg::tree_fold as msg;

/// Which pane's key table a fold operation is looked up in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldCtx {
    Refs,
    Status,
    LogFiles,
}

impl TreeFold {
    pub const ALL: [TreeFold; 4] = [
        TreeFold::ExpandAll,
        TreeFold::CollapseAll,
        TreeFold::ExpandBelow,
        TreeFold::CollapseBelow,
    ];

    pub fn collapses(self) -> bool {
        matches!(self, TreeFold::CollapseAll | TreeFold::CollapseBelow)
    }

    pub fn below(self) -> bool {
        matches!(self, TreeFold::ExpandBelow | TreeFold::CollapseBelow)
    }

    pub fn label(self) -> &'static str {
        match self {
            TreeFold::ExpandAll => msg::EXPAND_ALL.get(),
            TreeFold::CollapseAll => msg::COLLAPSE_ALL.get(),
            TreeFold::ExpandBelow => msg::EXPAND_BELOW.get(),
            TreeFold::CollapseBelow => msg::COLLAPSE_BELOW.get(),
        }
    }

    /// The registry action of this operation in `ctx`'s pane.
    pub fn key(self, ctx: FoldCtx) -> Action {
        use FoldCtx as C;
        use TreeFold as T;
        match (ctx, self) {
            (C::Refs, T::ExpandAll) => Action::RefsExpandAll,
            (C::Refs, T::CollapseAll) => Action::RefsCollapseAll,
            (C::Refs, T::ExpandBelow) => Action::RefsExpandBelow,
            (C::Refs, T::CollapseBelow) => Action::RefsCollapseBelow,
            (C::Status, T::ExpandAll) => Action::StatusExpandAll,
            (C::Status, T::CollapseAll) => Action::StatusCollapseAllFolders,
            (C::Status, T::ExpandBelow) => Action::StatusExpandBelow,
            (C::Status, T::CollapseBelow) => Action::StatusCollapseBelow,
            (C::LogFiles, T::ExpandAll) => Action::LogFilesExpandAll,
            (C::LogFiles, T::CollapseAll) => Action::LogFilesCollapseAllFolders,
            (C::LogFiles, T::ExpandBelow) => Action::LogFilesExpandBelow,
            (C::LogFiles, T::CollapseBelow) => Action::LogFilesCollapseBelow,
        }
    }
}
