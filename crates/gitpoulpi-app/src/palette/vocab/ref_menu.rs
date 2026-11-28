//! The fields of a ref the menus can copy.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use gitpoulpi_i18n::msg::menu_ref as msg;

/// One cell of a ref, as its Copy submenu offers it.
///
/// Three, not six: a ref has a name, a spelling of that name git itself
/// uses, and a commit it points at. Anything else on the row (ahead/
/// behind, the checked-out marker) is computed, not a value to paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefField {
    /// What the sidebar shows: `main`, `origin/feature`, `v1.2.0`.
    Name,
    /// The fully-qualified spelling: `refs/heads/main`. What a script,
    /// a `git push` refspec or a CI config wants.
    FullRef,
    /// The commit the ref points at, in full.
    TargetSha,
}

impl RefField {
    pub const fn all() -> [RefField; 3] {
        [RefField::Name, RefField::FullRef, RefField::TargetSha]
    }

    pub fn label(self) -> &'static str {
        match self {
            RefField::Name => msg::NOUN_NAME.get(),
            RefField::FullRef => msg::NOUN_FULL_REF.get(),
            RefField::TargetSha => msg::NOUN_TARGET_SHA.get(),
        }
    }

    /// The Copy submenu's own wording, which is shorter than `label`
    /// because the parent row already says "Copy".
    pub fn row_label(self) -> &'static str {
        match self {
            RefField::Name => msg::COPY_NAME.get(),
            RefField::FullRef => msg::COPY_FULL_REF.get(),
            RefField::TargetSha => msg::COPY_TARGET_SHA.get(),
        }
    }
}
