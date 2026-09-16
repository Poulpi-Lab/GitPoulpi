use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{Oid, RebaseState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictSide {
    Ours,
    Theirs,
    Base,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictSet {
    pub path: PathBuf,
    pub ours: Option<String>,
    pub theirs: Option<String>,
    pub base: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// The whole file from one stage, without any auto-merge. Kind-aware:
    /// a side that deleted the path removes it, a submodule takes the
    /// side's commit as the gitlink (no checkout), a symlink or the exec
    /// bit stay what that side had.
    TakeSide(ConflictSide),
    /// Exactly this UTF-8 text, byte for byte (the caller restores the
    /// line terminator and BOM it wants).
    UseContent(String),
    /// `text` encoded as `encoding` (a WHATWG label such as
    /// `windows-1252`): for a file whose sides were not UTF-8. A
    /// character the encoding cannot represent is refused with
    /// [`ConflictRefusal::Unencodable`] and nothing is written.
    UseEncoded { text: String, encoding: String },
    /// Accept a deletion: the path is removed from the working tree and
    /// the index (`git rm`).
    Delete,
}

/// How [`crate::GitBackend::resolve_conflict`] writes its result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolveOptions {
    /// Stage the result (the path stops being conflicted). `false` is the
    /// editor's plain "Save": the working tree file changes and the stages
    /// stay.
    pub stage: bool,
    /// The three stage oids (base, ours, theirs; `None` for a missing
    /// stage) the resolution was made against. When the index no longer
    /// holds exactly these the call is refused with
    /// [`ConflictRefusal::Stale`] and nothing is written.
    pub expected: Option<[Option<Oid>; 3]>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            stage: true,
            expected: None,
        }
    }
}

impl ResolveOptions {
    /// Stage the result, no stale guard: what the old two-argument call did.
    pub fn staged() -> Self {
        Self::default()
    }

    /// The stage oids of `entry`, in the order of `expected`.
    pub fn oids_of(entry: &ConflictEntry) -> [Option<Oid>; 3] {
        [
            entry.base.map(|s| s.oid),
            entry.ours.map(|s| s.oid),
            entry.theirs.map(|s| s.oid),
        ]
    }
}

/// Why a conflict call refused before changing anything. Typed so every
/// frontend words each case itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictRefusal {
    /// The index no longer holds the stages the caller opened (another
    /// frontend resolved or re-conflicted the file meanwhile).
    Stale,
    /// The path is not conflicted (and the call needed it to be).
    NotConflicted,
    /// `unresolve_conflict` on a path that is still conflicted.
    AlreadyConflicted,
    /// `operation_abort` with no merge, cherry-pick, revert or rebase in
    /// progress.
    NoOperation,
    /// The text has a character the chosen encoding cannot represent, on
    /// this 1-based line.
    Unencodable { line: usize },
}

impl ConflictRefusal {
    pub fn message(&self) -> String {
        match self {
            ConflictRefusal::Stale => {
                "the file was resolved or changed elsewhere since it was opened".into()
            }
            ConflictRefusal::NotConflicted => "the file is not conflicted".into(),
            ConflictRefusal::AlreadyConflicted => "the file is still conflicted".into(),
            ConflictRefusal::NoOperation => {
                "no merge, rebase, cherry-pick or revert is in progress".into()
            }
            ConflictRefusal::Unencodable { line } => {
                format!("line {line} has a character the file's encoding cannot represent")
            }
        }
    }
}

/// The merge, cherry-pick, revert or rebase currently stopped in the
/// repository ([`crate::GitBackend::operation_state`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    Merge {
        /// The branch merged into, `None` for a detached HEAD.
        head_name: Option<String>,
        /// What is being merged: the branch name git's message gives, or
        /// the short commit id.
        their_label: String,
    },
    CherryPick {
        commit: Oid,
    },
    Revert {
        commit: Oid,
    },
    Rebase(RebaseState),
    /// A `git am` stopped on a patch (`rebase-apply/applying`). Not a
    /// rebase: `rebase --abort/--continue` refuse it, `am_*` handle it.
    Am {
        /// The patch it stopped on, 1-based.
        step: u32,
        /// How many patches the mailbox holds.
        total: u32,
    },
}

impl OperationState {
    /// `merge`, `cherry-pick`, `revert`, `rebase` or `am`.
    pub fn name(&self) -> &'static str {
        match self {
            OperationState::Merge { .. } => "merge",
            OperationState::CherryPick { .. } => "cherry-pick",
            OperationState::Revert { .. } => "revert",
            OperationState::Rebase(_) => "rebase",
            OperationState::Am { .. } => "am",
        }
    }
}

/// Encodes `text` as `label` (`windows-1252`, `shift_jis`...). The error
/// is the 1-based line of the first character the encoding cannot hold.
/// UTF-16 is not written (no editor of ours produces it): a label naming
/// it is an error too, so is an unknown label.
pub fn encode_text(text: &str, label: &str) -> Result<Vec<u8>, EncodeError> {
    let encoding = encoding_rs::Encoding::for_label(label.trim().as_bytes())
        .ok_or_else(|| EncodeError::UnknownEncoding(label.to_string()))?;
    if encoding == encoding_rs::UTF_16LE || encoding == encoding_rs::UTF_16BE {
        return Err(EncodeError::UnknownEncoding(format!(
            "{label} (UTF-16 is not supported for writing)"
        )));
    }
    let (bytes, _, unmappable) = encoding.encode(text);
    if !unmappable {
        return Ok(bytes.into_owned());
    }
    // Find the line: the first one that does not round-trip on its own.
    let line = text
        .split('\n')
        .position(|l| encoding.encode(l).2)
        .map_or(1, |i| i + 1);
    Err(EncodeError::Unencodable { line })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    UnknownEncoding(String),
    Unencodable { line: usize },
}

/// What kind of conflict an unmerged path is, read off which of the
/// index's stages (1 = base, 2 = ours, 3 = theirs) exist. The same words
/// as `git status`'s `UU`/`AA`/`DD`/`DU`/`UD`/`AU`/`UA` codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictKind {
    /// Stages 1, 2 and 3: both sides changed the file.
    BothModified,
    /// Stages 2 and 3: both sides added a file at this path.
    BothAdded,
    /// Stage 1 only: both sides deleted it (a rename/rename leftover).
    BothDeleted,
    /// Stages 1 and 3: our side deleted it, theirs modified it.
    DeletedByUs,
    /// Stages 1 and 2: their side deleted it, ours modified it.
    DeletedByThem,
    /// Stage 2 only: only our side has a file here (rename/rename and
    /// rename/delete leftovers).
    AddedByUs,
    /// Stage 3 only: the mirror of `AddedByUs`.
    AddedByThem,
}

impl ConflictKind {
    /// The kind for the stages that exist. `None` when no stage does, which
    /// is not a conflict.
    pub fn from_stages(base: bool, ours: bool, theirs: bool) -> Option<Self> {
        Some(match (base, ours, theirs) {
            (_, true, true) if base => Self::BothModified,
            (_, true, true) => Self::BothAdded,
            (true, false, true) => Self::DeletedByUs,
            (true, true, false) => Self::DeletedByThem,
            (true, false, false) => Self::BothDeleted,
            (false, true, false) => Self::AddedByUs,
            (false, false, true) => Self::AddedByThem,
            (false, false, false) => return None,
        })
    }
}

/// One index stage of an unmerged path: its file mode (`0o100644`,
/// `0o100755`, `0o120000` symlink, `0o160000` gitlink, as git's octal
/// integer) and blob (or, for a gitlink, commit) id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageEntry {
    pub mode: u32,
    pub oid: Oid,
}

pub const MODE_SYMLINK: u32 = 0o120000;
pub const MODE_GITLINK: u32 = 0o160000;

/// An unmerged path as the index records it: cheap to list (no blob is
/// read), and enough to tell what kind of resolution it needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictEntry {
    pub path: PathBuf,
    pub kind: ConflictKind,
    pub base: Option<StageEntry>,
    pub ours: Option<StageEntry>,
    pub theirs: Option<StageEntry>,
}

impl ConflictEntry {
    /// An entry from the stages found, with `kind` derived from them.
    /// `None` when there are no stages.
    pub fn from_stages(
        path: PathBuf,
        base: Option<StageEntry>,
        ours: Option<StageEntry>,
        theirs: Option<StageEntry>,
    ) -> Option<Self> {
        let kind = ConflictKind::from_stages(base.is_some(), ours.is_some(), theirs.is_some())?;
        Some(Self {
            path,
            kind,
            base,
            ours,
            theirs,
        })
    }

    /// The stage of `side`.
    pub fn stage(&self, side: ConflictSide) -> Option<StageEntry> {
        match side {
            ConflictSide::Base => self.base,
            ConflictSide::Ours => self.ours,
            ConflictSide::Theirs => self.theirs,
        }
    }

    fn any_mode(&self, mode: u32) -> bool {
        [self.base, self.ours, self.theirs]
            .into_iter()
            .flatten()
            .any(|s| s.mode == mode)
    }

    /// A submodule (gitlink, mode 160000) is involved: the stages hold
    /// commit ids, not blobs.
    pub fn is_submodule(&self) -> bool {
        self.any_mode(MODE_GITLINK)
    }

    /// A symlink (mode 120000) is involved: the stage content is the
    /// link target.
    pub fn is_symlink(&self) -> bool {
        self.any_mode(MODE_SYMLINK)
    }

    /// Both sides exist and disagree on the file mode (the exec bit, say).
    pub fn mode_changed(&self) -> bool {
        matches!((self.ours, self.theirs), (Some(o), Some(t)) if o.mode != t.mode)
    }
}
