//! The fields of a ref the menus can copy.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// One cell of a ref, as its Copy submenu offers it.
///
/// Three, not six: a ref has a name, a spelling of that name git itself
/// uses, and a commit it points at. Anything else on the row (ahead/
/// behind, the checked-out marker) is computed, not a value to paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefField {
    /// What the sidebar shows: `main`, `origin/feature`, `v1.2.0`.
    Name,
    /// The fully-qualified spelling: `refs/heads/main`. What a script,
    /// a `git push` refspec or a CI config wants.
    FullRef,
    /// The commit the ref points at, in full.
    TargetSha,
}
