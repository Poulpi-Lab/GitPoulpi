//! A file's own history: the commits that changed one path, following it
//! across renames (`GitBackend::file_history`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Oid;

/// What a commit did to the file, against its first parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeKind {
    Added,
    Modified,
    /// The file ceased to exist. [`FileHistoryEntry::path`] is the path it
    /// had just before.
    Deleted,
    /// The file arrived at [`FileHistoryEntry::path`] from `from` (rename
    /// detection at 50% similarity, as `git log --follow` does).
    Renamed {
        from: PathBuf,
    },
}

/// One commit in a file's history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHistoryEntry {
    pub commit: Oid,
    pub parents: Vec<Oid>,
    pub author_name: String,
    pub author_email: String,
    /// Author time, seconds since the epoch.
    pub timestamp: i64,
    pub committer_timestamp: i64,
    pub summary: String,
    /// The file's path **at this commit** (after the change; before it for
    /// `Deleted`). Differs between entries when the file was renamed.
    pub path: PathBuf,
    pub change: FileChangeKind,
    /// Lines added to the file by this commit against its first parent.
    /// `None` for a binary file.
    pub added: Option<u32>,
    /// Lines removed; `None` for a binary file.
    pub removed: Option<u32>,
}

/// One page of a file's history, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHistoryPage {
    pub entries: Vec<FileHistoryEntry>,
    /// True when older entries exist beyond this page.
    pub more: bool,
}
