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
pub enum CommitField {
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
pub enum ColumnOp {
    /// Leave the view for the Log, on that commit.
    ShowInLog,
    /// The commit's diff of the file.
    Diff,
    /// The file as it was at the commit.
    File,
    /// The file blamed as of the commit.
    Blame,
}

impl ColumnOp {
    pub const ALL: [ColumnOp; 4] = [
        ColumnOp::ShowInLog,
        ColumnOp::Diff,
        ColumnOp::File,
        ColumnOp::Blame,
    ];

    /// The menu row: the File History's own wording for the same four.
    pub fn label(self) -> &'static str {
        use gitpoulpi_i18n::msg::history_view as h;
        match self {
            ColumnOp::ShowInLog => h::ACTION_SHOW_IN_LOG.get(),
            ColumnOp::Diff => h::ACTION_SHOW_DIFF.get(),
            ColumnOp::File => h::ACTION_OPEN_FILE.get(),
            ColumnOp::Blame => h::ACTION_BLAME.get(),
        }
    }
}
