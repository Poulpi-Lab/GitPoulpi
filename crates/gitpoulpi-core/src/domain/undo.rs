//! The vocabulary of undo/redo (the undo-redo design): ref
//! snapshots and compare-and-swap ref updates (the backend primitives),
//! plus the typed refusal and the summaries a client shows. The journal
//! state machine that uses them is `crate::journal`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Oid, OperationKind, ReflogPosition, RestoreOffer};

/// Where HEAD points: a branch (by short name, which may be unborn) or a
/// detached commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadState {
    Branch(String),
    Detached(Oid),
}

impl HeadState {
    pub fn describe(&self) -> String {
        match self {
            HeadState::Branch(name) => name.clone(),
            HeadState::Detached(oid) => format!("detached at {}", short(*oid)),
        }
    }
}

/// The first 7 characters of an object id.
pub fn short(oid: Oid) -> String {
    oid.to_hex()[..7].to_string()
}

/// `short`, or `none` for a ref that does not exist.
pub fn short_opt(oid: Option<Oid>) -> String {
    oid.map(short).unwrap_or_else(|| "nothing".to_string())
}

/// Raw ref values and HEAD at one instant. Tags are the *tag object's*
/// oid (not peeled), so an annotated tag can be restored byte for byte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefSnapshot {
    pub head: HeadState,
    /// Full ref name (`refs/heads/main`) to its raw oid.
    pub refs: BTreeMap<String, Oid>,
    /// Symbolic refs in scope (`refs/remotes/origin/HEAD`) to the full name
    /// of the ref they point at. Their resolved value also sits in `refs`
    /// when the backend lists it.
    #[serde(default)]
    pub symrefs: BTreeMap<String, String>,
}

/// Which ref namespaces a [`RefSnapshot`] lists. The default is every
/// branch and tag; a narrower scope is cheaper in a repository with
/// thousands of tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefScope {
    pub prefixes: Vec<String>,
}

impl RefScope {
    pub fn heads() -> Self {
        Self {
            prefixes: vec!["refs/heads/".into()],
        }
    }
}

impl Default for RefScope {
    fn default() -> Self {
        Self {
            prefixes: vec!["refs/heads/".into(), "refs/tags/".into()],
        }
    }
}

/// One step of an atomic ref transaction (`GitBackend::update_refs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefUpdate {
    /// The ref must not exist.
    Create {
        name: String,
        new: Oid,
    },
    Update {
        name: String,
        old: Oid,
        new: Oid,
    },
    Delete {
        name: String,
        old: Oid,
    },
    /// Fails the whole transaction unless the ref holds `old`.
    Verify {
        name: String,
        old: Oid,
    },
    /// Re-points HEAD without touching the index or any file. `old` is
    /// verified first (`UndoRefusal::HeadMoved`).
    SetHead {
        old: HeadState,
        new: HeadState,
    },
    /// Makes the symbolic ref `name` (`refs/remotes/<r>/HEAD`) point at
    /// the ref `new`, or deletes it when `new` is `None`. Applied after the
    /// rest of the transaction (its target may be created in it), and not
    /// verified: `old` only exists so that the update can be inverted.
    SetSymbolic {
        name: String,
        old: Option<String>,
        new: Option<String>,
    },
}

impl RefUpdate {
    /// The update that undoes this one, run against the state this one
    /// left.
    pub fn inverse(&self) -> RefUpdate {
        match self {
            RefUpdate::Create { name, new } => RefUpdate::Delete {
                name: name.clone(),
                old: *new,
            },
            RefUpdate::Update { name, old, new } => RefUpdate::Update {
                name: name.clone(),
                old: *new,
                new: *old,
            },
            RefUpdate::Delete { name, old } => RefUpdate::Create {
                name: name.clone(),
                new: *old,
            },
            RefUpdate::Verify { name, old } => RefUpdate::Verify {
                name: name.clone(),
                old: *old,
            },
            RefUpdate::SetHead { old, new } => RefUpdate::SetHead {
                old: new.clone(),
                new: old.clone(),
            },
            RefUpdate::SetSymbolic { name, old, new } => RefUpdate::SetSymbolic {
                name: name.clone(),
                old: new.clone(),
                new: old.clone(),
            },
        }
    }
}

/// The prefix of the hidden keep-alive refs. Excluded from every listing.
pub const UNDO_PIN_PREFIX: &str = "refs/gitpoulpi/undo/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Undo,
    Redo,
}

impl Direction {
    pub fn verb(self) -> &'static str {
        match self {
            Direction::Undo => "undo",
            Direction::Redo => "redo",
        }
    }
}

/// What every surface says when `general.undo_enabled` is off.
pub const UNDO_DISABLED_MESSAGE: &str = "Undo is turned off (Settings > General)";

/// Why an undo or redo did not happen. Nothing was changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoRefusal {
    NothingToUndo,
    NothingToRedo,
    /// A merge, rebase, cherry-pick or revert is in progress.
    InProgress {
        op: String,
    },
    RefMoved {
        name: String,
        expected: Option<Oid>,
        actual: Option<Oid>,
        /// Set by the daemon: where the ref's reflog holds `expected`, so
        /// a client can offer "restore main to where Commit left it".
        #[serde(default, skip_serializing_if = "Option::is_none")]
        restore: Option<Box<ReflogPosition>>,
    },
    HeadMoved {
        expected: HeadState,
        actual: HeadState,
    },
    /// A config section the entry touched is no longer as it left it.
    ConfigChanged {
        section: String,
    },
    /// The operation never completed (the daemon stopped half way).
    Interrupted {
        entry: u64,
    },
    /// Moving a checked-out branch would overwrite local changes in the
    /// working tree (`reset --keep` semantics); nothing was changed.
    WouldOverwrite {
        detail: String,
    },
    /// A file the entry touched is no longer as the operation left it
    /// (edited, staged or deleted since). Nothing was changed.
    PathChanged {
        path: String,
    },
    /// The index is no longer as the staging operation left it.
    IndexChanged,
    /// The stash list is no longer as the operation left it: the stash
    /// `commit` was dropped, popped or stored outside GitPoulpi.
    StashChanged {
        commit: Oid,
    },
    /// The client named an entry that is not the top one any more.
    StaleEntry {
        expected_id: u64,
        top_id: u64,
    },
    /// `general.undo_enabled` is off: nothing is recorded and nothing runs.
    Disabled,
    /// A resolved file can only be put back in conflict while the merge,
    /// rebase, cherry-pick or revert it belongs to is still going.
    OperationEnded,
    /// "Restore only this": the entry has nothing to bring back, or not
    /// without guessing. `reason` says why.
    NotRestorable {
        reason: String,
    },
    /// "Restore only this": the name the branch or tag had is taken now.
    /// Run again with `name` set (`suggested` is free).
    NameTaken {
        name: String,
        suggested: String,
    },
    /// "Restore only this": everything the entry removed is back already.
    AlreadyRestored,
    /// No such entry in the undo history.
    NoSuchEntry {
        id: u64,
    },
    /// The entry's old history was purged (`rewrite purge`): the objects it
    /// would put back are gone.
    Purged {
        entry: u64,
    },
    /// Undoing or redoing a deletion on a remote: the branch or tag there is
    /// no longer as the operation left it (recreated, moved or deleted by
    /// someone else), so pushing or deleting would overwrite that. Nothing was
    /// changed; `name` is the full ref (`refs/heads/x`).
    RemoteRefMoved(Box<RemoteRefMoved>),
    /// Undoing the deletion of a remote branch or tag needs its commit or tag
    /// object, and it is no longer in this repository (it was garbage
    /// collected), so it cannot be pushed back.
    RemoteObjectMissing {
        oid: Oid,
    },
}

/// What [`UndoRefusal::RemoteRefMoved`] says (boxed there: the refusal travels
/// in every error, and strings would make all of them bigger).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteRefMoved {
    pub remote: String,
    /// The full ref name (`refs/heads/x`, `refs/tags/v1`).
    pub name: String,
    /// What the entry expected the remote to hold (`None`: nothing).
    pub expected: Option<Oid>,
    pub actual: Option<Oid>,
}

impl UndoRefusal {
    /// The journal's top entry is stuck for good (the repository moved on,
    /// or the entry never finished) and can be dropped from the journal
    /// (`forget`) to reach the entries under it.
    pub fn forgettable(&self) -> bool {
        matches!(
            self,
            UndoRefusal::Interrupted { .. }
                | UndoRefusal::RefMoved { .. }
                | UndoRefusal::HeadMoved { .. }
                | UndoRefusal::ConfigChanged { .. }
                | UndoRefusal::PathChanged { .. }
                | UndoRefusal::IndexChanged
                | UndoRefusal::StashChanged { .. }
                | UndoRefusal::OperationEnded
                | UndoRefusal::Purged { .. }
                | UndoRefusal::RemoteRefMoved { .. }
                | UndoRefusal::RemoteObjectMissing { .. }
        )
    }

    /// A sentence naming what moved, for a human.
    pub fn message(&self) -> String {
        match self {
            UndoRefusal::NothingToUndo => "nothing to undo (a push, a fetch or anything else \
                 done on a remote server is never undone, except deleting a remote branch \
                 or tag)"
                .into(),
            UndoRefusal::NothingToRedo => "nothing to redo".into(),
            UndoRefusal::Disabled => UNDO_DISABLED_MESSAGE.into(),
            UndoRefusal::NotRestorable { reason } => format!("cannot restore: {reason}"),
            UndoRefusal::NameTaken { name, suggested } => format!(
                "the name {name} is taken now; restore under another name (for example {suggested})"
            ),
            UndoRefusal::AlreadyRestored => {
                "everything this operation removed is already back".to_string()
            }
            UndoRefusal::NoSuchEntry { id } => {
                format!("there is no operation #{id} in the undo history")
            }
            UndoRefusal::OperationEnded => "the merge, rebase, cherry-pick or revert this file \
                 was resolved in has ended, so it cannot be put back in conflict"
                .into(),
            UndoRefusal::InProgress { op } => {
                format!("finish or abort the {op} first")
            }
            UndoRefusal::RefMoved {
                name,
                expected,
                actual,
                ..
            } => format!(
                "{} is at {}, not at {} where the operation left it; it was changed outside GitPoulpi",
                name.strip_prefix("refs/heads/")
                    .or_else(|| name.strip_prefix("refs/tags/"))
                    .unwrap_or(name),
                short_opt(*actual),
                short_opt(*expected),
            ),
            UndoRefusal::HeadMoved { expected, actual } => format!(
                "HEAD is {}, not {} where the operation left it; it was changed outside GitPoulpi",
                actual.describe(),
                expected.describe()
            ),
            UndoRefusal::ConfigChanged { section } => format!(
                "the configuration section {section} was changed outside GitPoulpi since the operation"
            ),
            UndoRefusal::Interrupted { entry } => format!(
                "operation #{entry} never completed (GitPoulpi stopped while it ran); \
                 inspect it with `gitpoulpi undo --list` and fix the repository by hand"
            ),
            UndoRefusal::WouldOverwrite { detail } => format!(
                "local changes in the working tree would be overwritten, so nothing was moved: {}",
                detail.trim()
            ),
            UndoRefusal::PathChanged { path } => format!(
                "{path} was changed since the operation (outside GitPoulpi, or by a later \
                 operation), so restoring it would overwrite that change; nothing was restored"
            ),
            UndoRefusal::IndexChanged => "the index (staged changes) was changed since the \
                 operation, outside GitPoulpi; nothing was restored"
                .to_string(),
            UndoRefusal::StashChanged { commit } => format!(
                "the stash {} was dropped, applied or stored outside GitPoulpi since the \
                 operation; the stash list was left alone",
                short(*commit)
            ),
            UndoRefusal::Purged { entry } => format!(
                "the old history of operation #{entry} was purged, so it cannot be undone or redone; \
                 `gitpoulpi undo-forget` drops it from the undo history"
            ),
            UndoRefusal::RemoteRefMoved(moved) => {
                let RemoteRefMoved {
                    remote,
                    name,
                    expected,
                    actual,
                } = &**moved;
                format!(
                "{} on {remote} is at {}, not at {} where the operation left it: it was changed \
                 on the remote since, so nothing was changed there",
                display_ref_name(name),
                actual.map_or_else(|| "nothing".to_string(), short),
                expected.map_or_else(|| "nothing".to_string(), short),
                )
            }
            UndoRefusal::RemoteObjectMissing { oid } => format!(
                "{} is no longer in this repository (it was garbage collected), so the branch or \
                 tag cannot be restored on the remote with a push",
                short(*oid)
            ),
            UndoRefusal::StaleEntry {
                expected_id,
                top_id,
            } => format!(
                "the operation you saw (#{expected_id}) is no longer the latest (#{top_id} is)"
            ),
        }
    }
}

/// One entry as a client shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrySummary {
    pub id: u64,
    pub kind: OperationKind,
    /// `Commit "fix login" (a81b9e4)`.
    pub label: String,
    /// Unix seconds.
    pub at: i64,
    pub state: EntryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryState {
    /// Written before the operation ran.
    Pending,
    Done,
    /// Pending, and the operation is no longer running: it never
    /// finished. Never undone automatically.
    Interrupted,
    /// A history rewrite whose old objects, pins and reflogs were purged:
    /// kept in the list, greyed, never undone or redone.
    Purged,
}

/// The answer to `OperationKind::JournalState`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalState {
    pub undo: Option<EntrySummary>,
    pub redo: Option<EntrySummary>,
    /// Why the top undo entry cannot run right now, if it cannot.
    pub blocked: Option<UndoRefusal>,
    /// Same, for the top redo entry.
    pub redo_blocked: Option<UndoRefusal>,
    /// `general.undo_enabled` is off: no entries are recorded and the
    /// buttons say so.
    #[serde(default)]
    pub disabled: bool,
    /// How many entries the undo stack holds (what the Undo button says
    /// "you can go back this far" with).
    #[serde(default)]
    pub undo_count: usize,
    /// How many undone entries a redo can bring back.
    #[serde(default)]
    pub redo_count: usize,
}

/// What an undo or redo did, or would do (`dry_run`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoOutcome {
    pub direction: Direction,
    pub entry: EntrySummary,
    /// The steps, in words.
    pub steps: Vec<String>,
    /// False for a dry run.
    pub applied: bool,
    /// Set on a dry run when the real run needs `confirmed`: why.
    #[serde(default)]
    pub needs_confirmation: Option<String>,
}

/// The answer to `OperationKind::JournalList`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalListing {
    /// Oldest first; `redo` entries are listed newest-undone last.
    pub undo: Vec<ListedEntry>,
    pub redo: Vec<ListedEntry>,
    /// With `all`: entries no longer redoable whose data is still pinned.
    pub retired: Vec<ListedEntry>,
}

/// Who made the operation an entry records.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntrySource {
    /// GitPoulpi itself (a window, the CLI, an MCP tool).
    #[default]
    Gitpoulpi,
    /// A shell or an AI agent, found by the ref snapshot diff.
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListedEntry {
    /// `gitpoulpi`, or `outside` for a change made outside GitPoulpi.
    #[serde(default)]
    pub source: EntrySource,
    /// For an outside entry: the reflog's reason and committer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outside: Option<crate::journal::OutsideInfo>,
    pub summary: EntrySummary,
    /// `refs/heads/main: a81b9e4 -> 3f2a1c0`, one per changed ref.
    pub changes: Vec<String>,
    /// What "restore only this" would bring back (empty `items` and a
    /// `not_restorable` reason when nothing).
    #[serde(default)]
    pub restore: RestoreOffer,
}

/// One file's recorded state on one side (index or working tree): the git
/// file mode (`0o100644`, `0o100755`, `0o120000`) and the blob id of the
/// **raw** bytes (no clean filter: CRLF and LFS pointers stay as they are).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    pub mode: u32,
    pub oid: Oid,
}

/// The index and working-tree state of one file, `None` where it is
/// absent. Compared with `==` for the undo/redo preconditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathHash {
    pub path: String,
    pub worktree: Option<FileHash>,
    pub index: Option<FileHash>,
}

/// A safety snapshot (`GitBackend::snapshot_create`): a tree holding
/// `index/<path>` and `worktree/<path>` for every file under the paths it
/// was made for, plus the same information as `hashes`, sorted by path.
/// Keep `tree` alive with a pin ref (any ref may point at a tree).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tree: Oid,
    pub hashes: Vec<PathHash>,
}

/// `refs/heads/x` as `x`, `refs/tags/v1` as `tag v1`.
fn display_ref_name(name: &str) -> String {
    name.strip_prefix("refs/heads/")
        .map(str::to_string)
        .or_else(|| name.strip_prefix("refs/tags/").map(|t| format!("tag {t}")))
        .unwrap_or_else(|| name.to_string())
}
