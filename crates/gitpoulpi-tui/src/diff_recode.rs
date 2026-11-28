//! The Diff view's re-read in a chosen encoding. The cache, the decoding and
//! the diff live in `gitpoulpi_link::diff_recode`; this file is the view's
//! half: which two sides an open diff is made of.

pub(crate) use gitpoulpi_link::diff_recode::*;

use crate::editor_charset::Charset;

/// The sides of a Diff view: a commit against its parent, one half of
/// the working tree, or a comparison.
pub(crate) fn want_of_state(state: &crate::diff_view::DiffViewState, charset: Charset) -> Want {
    use crate::diff_view::DiffSource;
    use gitpoulpi_core::domain::DiffTarget as T;
    use gitpoulpi_link::DiffContentSide as S;
    let (old, new, at) = match state.source {
        DiffSource::Commit => (S::Parent, S::Commit, state.commit),
        DiffSource::Unstaged => (S::Index, S::WorkingTree, state.commit),
        DiffSource::Staged => (S::Head, S::Index, state.commit),
        DiffSource::Compare => match state.compare {
            Some(T::CommitRange(a, b)) => (S::CommitAt(a), S::CommitAt(b), state.commit),
            Some(T::CommitSpan(o, n)) => (S::ParentOf(o), S::CommitAt(n), state.commit),
            Some(T::CommitVsWorkingTree(c)) => (S::CommitAt(c), S::WorkingTree, state.commit),
            Some(T::CommitVsParent(c)) => (S::ParentOf(c), S::CommitAt(c), state.commit),
            Some(T::WorkingTreeVsIndex) => (S::Index, S::WorkingTree, state.commit),
            Some(T::IndexVsHead) => (S::Head, S::Index, state.commit),
            None => (S::Parent, S::Commit, state.commit),
        },
    };
    Want {
        path: state.path.clone(),
        old,
        new,
        at,
        charset,
    }
}
