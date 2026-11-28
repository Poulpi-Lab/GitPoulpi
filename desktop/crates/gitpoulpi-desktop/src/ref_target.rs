//! What a ref command acts on: one row of the Refs sidebar or one ref pill of the Log, as plain data, and
//! the facts about the current selection the palette's availability rules read.

use gitpoulpi_app::refs_tree::RefRowKind;

/// A ref (or stash) a command acts on. The names are the sidebar's own: a local branch or tag by its
/// name, a remote branch as `remote/name`, a stash by its number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefTarget {
    Local(String),
    Remote(String),
    Tag(String),
    Stash(u32),
}

impl RefTarget {
    /// The kind of ref, `None` for a stash.
    pub fn kind(&self) -> Option<RefRowKind> {
        match self {
            RefTarget::Local(_) => Some(RefRowKind::Local),
            RefTarget::Remote(_) => Some(RefRowKind::Remote),
            RefTarget::Tag(_) => Some(RefRowKind::Tag),
            RefTarget::Stash(_) => None,
        }
    }

    /// The name as shown: the ref's, or `stash@{n}`.
    pub fn name(&self) -> String {
        match self {
            RefTarget::Local(n) | RefTarget::Remote(n) | RefTarget::Tag(n) => n.clone(),
            RefTarget::Stash(id) => crate::strings::stash_name(*id),
        }
    }

    pub fn of_kind(kind: RefRowKind, name: &str) -> RefTarget {
        match kind {
            RefRowKind::Local => RefTarget::Local(name.to_string()),
            RefRowKind::Remote => RefTarget::Remote(name.to_string()),
            RefRowKind::Tag => RefTarget::Tag(name.to_string()),
        }
    }
}

/// What is selected in the Refs sidebar, for the palette's availability rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionFacts {
    pub local_branch: bool,
    pub remote_branch: bool,
    pub tag: bool,
    pub stash: bool,
    pub folder: bool,
    pub tag_unpushed: bool,
    pub tag_remote_only: bool,
    pub tag_differs: bool,
    /// The repository has at least one stash.
    pub has_stash: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_the_sidebars_spelling() {
        assert_eq!(RefTarget::Local("main".into()).name(), "main");
        assert_eq!(RefTarget::Stash(2).name(), "stash@{2}");
        assert_eq!(RefTarget::Stash(0).kind(), None);
        assert_eq!(
            RefTarget::of_kind(RefRowKind::Remote, "origin/x"),
            RefTarget::Remote("origin/x".into())
        );
    }
}
