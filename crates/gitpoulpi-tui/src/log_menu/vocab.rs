//! Fields and operations of the Log menus.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

/// One cell of a commit, as the Copy submenu offers it.
///
/// The six conventional actions of a commit context menu, in their usual order.
/// A separate enum rather than six `LogContextAction` variants so that
/// "which field" and "how to read it" live together and the menu, the
/// clipboard call and the status line cannot name different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitField {
    Message,
    AuthorName,
    AuthorEmail,
    Date,
    ShortSha,
    LongSha,
}

/// What the commit column's menu adds to the Log's: the file the view beside
/// the column shows, at the commit picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ColumnOp {
    /// Leave the view for the Log, on that commit.
    ShowInLog,
    /// The commit's diff of the file.
    Diff,
    /// The file as it was at the commit.
    File,
    /// The file blamed as of the commit.
    Blame,
}
