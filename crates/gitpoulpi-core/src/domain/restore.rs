//! "Restore only this" (the undo-safety-net design): what a
//! journal entry can bring back without undoing the entries after it.

use serde::{Deserialize, Serialize};

/// What kind of thing a restore brings back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreItemKind {
    /// A deleted (or renamed-away) local branch.
    Branch,
    /// A deleted tag.
    Tag,
    /// A remote-tracking branch (the removal of a remote).
    RemoteRef,
    /// A dropped or popped stash.
    Stash,
    /// A deleted or discarded file, hunk or lines (written back in place,
    /// or next to the file as `<name>.restored`).
    File,
    /// Lines of a `.gitignore` / `.gitattributes`.
    Lines,
    /// A configuration section: a remote, an upstream.
    Config,
    /// The old tip of a branch a history operation moved: restored as a new
    /// branch, the current branch is never moved.
    OldTip,
}

/// One thing a restore would bring back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreItem {
    pub kind: RestoreItemKind,
    /// `branch feature (a81b9e4)`, `stash "On main: wip"`, `a.txt`.
    pub label: String,
}

/// What "restore only this" would do for one entry, from the entry alone
/// (whether it can be done *now* is decided when it runs).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreOffer {
    pub items: Vec<RestoreItem>,
    /// Why there is nothing to restore, when `items` is empty.
    #[serde(default)]
    pub not_restorable: Option<String>,
}

impl RestoreOffer {
    pub fn restorable(&self) -> bool {
        !self.items.is_empty()
    }
}

/// What `JournalRestore` / `ReflogRestore` did, or would do (`dry_run`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreOutcome {
    /// The entry restored from (`JournalRestore`), when there is one.
    pub entry: Option<super::EntrySummary>,
    /// The steps, in words.
    pub steps: Vec<String>,
    /// False for a dry run.
    pub applied: bool,
    /// The journal entry the restore itself became: Ctrl+Z (or `undo`)
    /// takes it back. None for a dry run, or when the restore was done
    /// through an existing operation (a reset).
    #[serde(default)]
    pub new_entry: Option<super::EntrySummary>,
    /// Full names of the refs the restore created or moved.
    #[serde(default)]
    pub refs: Vec<String>,
    /// Paths written next to a file that changed (`<name>.restored`).
    #[serde(default)]
    pub side_files: Vec<String>,
}
