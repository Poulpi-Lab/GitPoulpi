use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Untracked,
    Modified,
    Staged,
    Conflicted,
    Renamed,
    Deleted,
    Added,
}

/// One `git status` row. `index`/`worktree` are independent because a file
/// can be staged *and* have further unstaged changes at the same time
/// (porcelain v2's XY columns) - collapsing that into a single FileStatus
/// would lose information the staging UI needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEntry {
    pub path: PathBuf,
    pub index: Option<FileStatus>,
    pub worktree: Option<FileStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineOrigin {
    Add,
    Delete,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub origin: LineOrigin,
    pub content: String,
    pub staged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HunkId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hunk {
    pub id: HunkId,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: PathBuf,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
}

/// What `GitBackend::diff_files` answers: the files `diff` lists for the
/// same target, in the same order with the same statuses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFileList {
    pub files: Vec<FileDiff>,
    /// `true`: `files` *is* the `diff` answer, hunks and all - the
    /// backend could not list more cheaply than it diffs, so it did both
    /// at once. `false`: every `hunks` is empty and the hunks are still
    /// to be asked of `diff`. Never inferred from empty `hunks`: a binary
    /// file or a mode-only change has none in the full diff either.
    pub hunks_included: bool,
}

/// What one commit changed, in numbers (the Log's "Changes" column): the
/// commit against its first parent (a root commit against nothing), the
/// same comparison as `DiffTarget::CommitVsParent`. Renames are not
/// detected, so a rename counts as two files; a binary file counts as a
/// file with no lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitStat {
    pub id: super::Oid,
    pub files: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTarget {
    WorkingTreeVsIndex,
    IndexVsHead,
    CommitVsParent(super::Oid),
    CommitRange(super::Oid, super::Oid),
    /// The commit's tree against the working tree: what a user would have
    /// to do to the commit to get what is on disk (staged and unstaged
    /// changes of tracked files; untracked files are not part of any diff
    /// target). The "WIP against a commit" comparison.
    CommitVsWorkingTree(super::Oid),
    /// From the *first parent of* `.0` (the empty tree for a root commit)
    /// to `.1`: the combined changes of a run of commits whose oldest is
    /// `.0` and whose newest is `.1`.
    CommitSpan(super::Oid, super::Oid),
}

/// Which lines within a hunk a line-level stage/unstage selection covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineSelector {
    pub hunk: HunkId,
    pub line_indices: Vec<u32>,
}

/// Unifies every staging granularity (line, hunk, file, folder, all) behind
/// one call so frontends never need separate stage_file/stage_hunk/... methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageTarget {
    Line(PathBuf, LineSelector),
    Hunk(PathBuf, HunkId),
    File(PathBuf),
    Folder(PathBuf),
    All,
}

/// Part of one file's *unstaged* changes to throw away:
/// The "Discard Hunk" and "Discard this line" actions in the diff.
///
/// Numbered exactly like [`StageTarget::Hunk`] / [`StageTarget::Line`]
/// against the working-tree-vs-index diff (`DiffTarget::WorkingTreeVsIndex`):
/// the hunk ids that diff assigns, and indices into that hunk's `lines`.
/// Whole files, folders and "everything" are not here on purpose - that
/// is [`crate::GitBackend::discard`], which also deletes untracked files;
/// this only ever edits the working-tree copy of a tracked file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscardTarget {
    Hunk(PathBuf, HunkId),
    Line(PathBuf, LineSelector),
}
