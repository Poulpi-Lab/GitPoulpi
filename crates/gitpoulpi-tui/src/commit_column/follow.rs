//! Which commit the open view is of, and what it becomes when the column
//! picks another one. Pure: the window applies the answer
//! (`event_loop/window/commit_follow.rs`).

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{FileDiff, Oid};

use crate::chrome::CentralView;
use crate::daemon_link::FileInsightKind;
use crate::diff_view::{DiffSource, DiffViewState};
use crate::file_view::FileInsight;
use crate::history_view::HistoryView;

/// What the open view shows of the file, for the retargeting rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewKind {
    /// The commit's diff of the file.
    Diff,
    /// The file as it was at the commit (View all files, or the File View
    /// button).
    Content,
    /// The file blamed as of the commit.
    Blame,
    /// The file as it was at the commit, even when the commit changed it
    /// (the column's menu asks for it: the diff is the other row).
    FileAsIs,
}

/// The commit the open central view is of, when it is a view of one commit's
/// file: a Diff (`DiffSource::Commit`), a File View or Blame at a commit, or
/// nothing else. `None` for the Editor (the working copy), for the working tree's views, a
/// comparison and the File History, which the column does not serve.
pub(crate) fn viewed_commit(
    central: CentralView,
    diff: Option<&DiffViewState>,
    insight: Option<&FileInsight>,
) -> Option<Oid> {
    if diff.is_some_and(HistoryView::owns) {
        return None;
    }
    let of_insight = || {
        insight
            .filter(|f| {
                !f.request.worktree
                    && matches!(
                        f.request.kind,
                        FileInsightKind::Content | FileInsightKind::Blame
                    )
            })
            .map(|f| f.request.at)
    };
    let of_diff = || {
        diff.filter(|d| d.source == DiffSource::Commit)
            .map(|d| d.commit)
    };
    match central {
        CentralView::Diff => of_diff(),
        CentralView::File => of_insight(),
        // The editor edits the working copy, not a commit's file: no column
        // (it is back when the editor closes onto the view under it).
        CentralView::Editor | CentralView::Log | CentralView::Merge => None,
    }
}

/// What the open view's file is called, and which kind of view it is.
pub(crate) fn open_file<'a>(
    central: CentralView,
    diff: Option<&'a DiffViewState>,
    insight: Option<&'a FileInsight>,
) -> Option<(ViewKind, &'a Path)> {
    let from_insight = || {
        insight.map(|f| {
            let kind = if f.request.kind == FileInsightKind::Blame {
                ViewKind::Blame
            } else {
                ViewKind::Content
            };
            (kind, f.request.path.as_path())
        })
    };
    let from_diff = || diff.map(|d| (ViewKind::Diff, d.path.as_path()));
    match central {
        CentralView::Diff => from_diff(),
        CentralView::File => from_insight(),
        CentralView::Editor => from_insight().or_else(from_diff),
        CentralView::Log | CentralView::Merge => None,
    }
}

/// What the view becomes at the commit that was picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Retarget {
    /// The commit changed the file: its diff there (`file_index` into the
    /// commit's changed files).
    Diff { file_index: usize, path: PathBuf },
    /// The commit did not touch the file: the file as it was there, with the
    /// "unchanged" notice. A file that does not exist there fails to read
    /// and says so (`FollowNote::Missing`).
    Content { path: PathBuf },
    /// Blame as of the commit; `changed` is the file's index when the commit
    /// changed it (so the bar's Diff View has something to show).
    Blame {
        path: PathBuf,
        changed: Option<usize>,
    },
}

/// The rule: a Diff or File View shows the file's diff when the commit
/// changed it and the file as it was otherwise; a Blame stays a Blame.
pub(crate) fn plan(kind: ViewKind, path: &Path, changed_files: Option<&[FileDiff]>) -> Retarget {
    let index = changed_files.and_then(|files| files.iter().position(|f| f.path == path));
    match (kind, index) {
        (ViewKind::Blame, changed) => Retarget::Blame {
            path: path.to_path_buf(),
            changed,
        },
        (ViewKind::FileAsIs, _) => Retarget::Content {
            path: path.to_path_buf(),
        },
        (_, Some(file_index)) => Retarget::Diff {
            file_index,
            path: path.to_path_buf(),
        },
        (_, None) => Retarget::Content {
            path: path.to_path_buf(),
        },
    }
}
