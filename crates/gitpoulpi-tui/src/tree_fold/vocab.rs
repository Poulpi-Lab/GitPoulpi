//! Expand and collapse operations of a tree.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// One expand/collapse operation. "Below" acts on a folder and everything
/// under it; the others on every folder of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TreeFold {
    ExpandAll,
    CollapseAll,
    ExpandBelow,
    CollapseBelow,
}
