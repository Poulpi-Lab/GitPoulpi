//! The undo/redo journal, pure: no I/O, no git. The daemon persists it
//! (`<git-dir>/gitpoulpi/journal.json`), captures the facts, and runs the
//! steps this module plans through `GitBackend`. See
//! the undo-redo design.
//!
//! Safety rules this module enforces (each has a test):
//! * a plan is only produced when everything the entry touched is exactly
//!   as the entry left it; otherwise a typed refusal and nothing happens;
//! * every value an undo or redo could need gets a keep-alive pin, and
//!   pins are released only when an entry falls off the bounded history;
//! * redo restores the recorded state, it never re-runs the operation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::{
    short, short_opt, ConflictEntry, Direction, EntrySource, EntryState, EntrySummary, HeadState,
    JournalState, ListedEntry, Oid, OperationKind, PathHash, RefSnapshot, RefUpdate, Snapshot,
    UndoRefusal, UNDO_PIN_PREFIX,
};

/// The undo depth when nothing is configured (`general.undo_depth`).
pub const DEFAULT_MAX_ENTRIES: usize = 100;
/// Entries dropped from the redo stack whose pins live on.
pub const MAX_RETIRED: usize = 20;
/// The on-disk format version.
pub const JOURNAL_VERSION: u32 = 1;

/// A keep-alive ref: `name` (under `refs/gitpoulpi/undo/`) holds `oid`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub name: String,
    pub oid: Oid,
}

/// One ref an operation changed. `None` means the ref did not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefChange {
    pub name: String,
    pub old: Option<Oid>,
    pub new: Option<Oid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadChange {
    pub old: HeadState,
    pub new: HeadState,
}

/// A config section (`branch.<name>`) before and after, as key/value
/// pairs with the full key (`branch.x.remote`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigChange {
    pub section: String,
    pub old: Vec<(String, String)>,
    pub new: Vec<(String, String)>,
}

/// What moving the entry's branch must do besides moving the ref.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// The ref alone moves (commit, reset --soft, ...).
    #[default]
    Refs,
    /// The ref moves and the index and files follow with `reset --keep`
    /// semantics (merge, rebase, cherry-pick, revert, pull, `reset
    /// --hard`); a hard reset over local changes also carries a file
    /// snapshot (`PathChange`) of them.
    Files,
}

/// Files an operation changed (discard, patch apply, delete): a pinned
/// safety snapshot of the touched paths before and after. Undo and redo
/// write one back, and only when the paths still hash as the other left
/// them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathChange {
    pub paths: Vec<String>,
    pub before: Snapshot,
    pub after: Snapshot,
}

/// The index (staged changes) before and after a staging operation, as
/// tree ids. Undo and redo read one back, and only when the index is
/// exactly the other tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexChange {
    pub before: Oid,
    pub after: Oid,
    /// A fact learnt for free: while the index file's trailing checksum is
    /// `.0`, the index is tree `.1`. It spares a `write-tree` (a process
    /// on the system backend) every time the state is read.
    #[serde(default)]
    pub known: Option<(String, Oid)>,
}

/// A "Save & Mark Resolved": the stages the path had before it was resolved.
/// The entry's [`PathChange`] holds the snapshots (the resolved index entry
/// and file are what redo writes back); undo puts the three stages and the
/// conflicted file back. Only valid while the operation that made the
/// conflict is still in progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictChange {
    pub entry: ConflictEntry,
}

/// The whole index file (conflict stages included) and the state files of
/// a stopped operation (`MERGE_HEAD`, `MERGE_MSG`, `MERGE_MODE`,
/// `AUTO_MERGE`, `CHERRY_PICK_HEAD`, ...), saved by the daemon under a
/// name before and after the entry's operation. An index tree cannot hold
/// unmerged entries, so this is how staging during a merge, and the abort
/// of a merge, are taken back exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpStateChange {
    /// Name of the saved state before / after.
    pub before: String,
    pub after: String,
    /// The index file's trailing checksum before / after: undo and redo
    /// only run while the index is exactly the other one.
    pub before_sum: Option<String>,
    pub after_sum: Option<String>,
    /// The entry is the abort of the operation: undo needs the operation
    /// *not* running and brings it back, redo needs it running. Otherwise
    /// the operation runs at both ends.
    pub aborts: bool,
}

/// One stash, as the stash list holds it: its commit (the identity; the
/// index in `stash@{n}` shifts) and its reflog message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashRecord {
    pub commit: Oid,
    pub message: String,
}

/// A symbolic ref (`refs/remotes/<r>/HEAD`) an operation created, moved or
/// deleted: where it pointed before and after (`None`: absent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymrefChange {
    pub name: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

/// What an operation did to the stash list (push, pop, drop): the stashes
/// that were on it afterwards and not before, and the other way round.
/// Entries are found by commit, never by position.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashChange {
    pub added: Vec<StashRecord>,
    pub removed: Vec<StashRecord>,
}

/// Who and why, for an entry GitPoulpi did not make itself: the reflog's
/// reason and committer. Never a guess about shell versus AI agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutsideInfo {
    pub message: String,
    #[serde(default)]
    pub committer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub kind: OperationKind,
    pub label: String,
    pub at: i64,
    pub state: EntryState,
    /// HEAD before and after (`None` when the operation never looked at
    /// it). For a `Pending` entry `new` equals `old`.
    #[serde(default)]
    pub head: Option<HeadChange>,
    /// Undo and redo are only valid while HEAD is where the entry left
    /// it (commit, checkout); a branch create/delete does not care.
    pub require_head: bool,
    /// For a `Pending` entry: the refs the operation may change, with
    /// their old values and `new: None` (unknown yet).
    pub refs: Vec<RefChange>,
    #[serde(default)]
    pub config: Vec<ConfigChange>,
    #[serde(default)]
    pub pins: Vec<Pin>,
    #[serde(default)]
    pub effect: Effect,
    #[serde(default)]
    pub files: Option<PathChange>,
    #[serde(default)]
    pub index: Option<IndexChange>,
    #[serde(default)]
    pub stash: Option<StashChange>,
    #[serde(default)]
    pub symrefs: Vec<SymrefChange>,
    /// Set on a `ResolveConflict` entry: what undo puts back.
    #[serde(default)]
    pub conflict: Option<ConflictChange>,
    /// Set on a staging operation done while a merge, cherry-pick or
    /// revert runs, and on the abort of one: the index file and the
    /// operation's state files before and after.
    #[serde(default)]
    pub opstate: Option<OpStateChange>,
    /// Set on an `Outside` entry: the reflog's reason and committer.
    #[serde(default)]
    pub outside: Option<OutsideInfo>,
}

impl JournalEntry {
    pub fn summary(&self) -> EntrySummary {
        EntrySummary {
            id: self.id,
            kind: self.kind,
            label: self.label.clone(),
            at: self.at,
            state: self.state,
        }
    }

    pub fn listed(&self) -> ListedEntry {
        ListedEntry {
            source: if self.kind == OperationKind::Outside {
                EntrySource::Outside
            } else {
                EntrySource::Gitpoulpi
            },
            outside: self.outside.clone(),
            restore: self.restore_offer(),
            summary: self.summary(),
            changes: self
                .refs
                .iter()
                .map(|c| {
                    format!(
                        "{}: {} -> {}",
                        c.name,
                        short_opt(c.old),
                        if matches!(self.state, EntryState::Done | EntryState::Purged) {
                            short_opt(c.new)
                        } else {
                            "?".to_string()
                        }
                    )
                })
                .chain(
                    self.index
                        .iter()
                        .map(|i| format!("index: {} -> {}", short(i.before), short(i.after))),
                )
                .chain(
                    self.files
                        .iter()
                        .map(|f| format!("files: {}", paths_in_words(&f.paths))),
                )
                .chain(self.stash.iter().flat_map(|st| {
                    st.added
                        .iter()
                        .map(|r| format!("stash: + {}", short(r.commit)))
                        .chain(
                            st.removed
                                .iter()
                                .map(|r| format!("stash: - {}", short(r.commit))),
                        )
                }))
                .collect(),
        }
    }
}

/// `a.txt`, `a.txt, b.txt`, `a.txt, b.txt, c.txt and 4 more`.
pub fn paths_in_words(paths: &[String]) -> String {
    const SHOWN: usize = 3;
    let mut text = paths
        .iter()
        .take(SHOWN)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if paths.len() > SHOWN {
        text.push_str(&format!(" and {} more", paths.len() - SHOWN));
    }
    text
}

/// The pin ref name for entry `id`, slot `n`, of the journal whose pin
/// namespace is `ns` (`Journal::pin_ns`: empty for the main worktree,
/// `wt-<name>/` for a linked one, so two worktrees' entry ids never share a
/// ref).
pub fn pin_name(ns: &str, id: u64, n: usize) -> String {
    format!("{UNDO_PIN_PREFIX}{ns}{id}/{n}")
}

/// The pin namespace of the journal kept in `git_dir`: empty for a main
/// worktree, and `wt-<name>-<hash>/` for a linked one (`<common>/worktrees/<name>`),
/// the name made safe for a ref.
pub fn pin_namespace(git_dir: &std::path::Path) -> String {
    let linked = git_dir
        .parent()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n == "worktrees");
    let Some(name) = git_dir.file_name().filter(|_| linked) else {
        return String::new();
    };
    let name = name.to_string_lossy();
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let hash = name.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    format!("wt-{safe}-{:08x}/", hash as u32)
}

/// What the daemon captured about the repository right now.
#[derive(Debug, Clone)]
pub struct RepoFacts {
    pub snapshot: RefSnapshot,
    /// `rebase`, `merge`, `cherry-pick` or `revert` while one is running.
    pub in_progress: Option<String>,
    /// Current key/value pairs of the config sections the top entry
    /// touched.
    pub config: BTreeMap<String, Vec<(String, String)>>,
    /// The index's tree id now; only read when the top entry of a
    /// direction recorded an index change.
    pub index_tree: Option<Oid>,
    /// Current state of the files of the top entries that recorded a
    /// [`PathChange`], by entry id.
    pub files: BTreeMap<u64, Vec<PathHash>>,
    /// The commits on the stash list now; only read when the top entry of
    /// a direction recorded a [`StashChange`].
    pub stashes: Vec<Oid>,
    /// The index file's checksum now; only read when the top entry of a
    /// direction recorded an [`OpStateChange`].
    pub index_sum: Option<String>,
    /// Unmerged paths exist without a merge, rebase, cherry-pick or revert
    /// running (a conflicted `stash pop`/`apply`): a resolution made then
    /// can still be put back. Only read when a top entry is a resolution.
    pub unmerged: bool,
}

/// One thing to do, in order. Each has an inverse for rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// An atomic ref transaction.
    UpdateRefs(Vec<RefUpdate>),
    /// `git checkout` of the previous HEAD (files follow; git refuses on
    /// a conflicting local change).
    Checkout { to: HeadState, from: HeadState },
    SetConfig {
        section: String,
        to: Vec<(String, String)>,
        from: Vec<(String, String)>,
    },
    /// The checked-out `branch` moves with its files (`reset --keep`).
    MoveHeadKeep { branch: String, from: Oid, to: Oid },
    /// The index becomes tree `to`; it is tree `from` now.
    RestoreIndex { to: Oid, from: Oid },
    /// The index and working tree of `paths` become snapshot `to`; they
    /// are snapshot `from` now.
    RestorePaths {
        paths: Vec<String>,
        to: Oid,
        from: Oid,
    },
    /// The stash commit goes (back) on top of the stash list.
    StashStore { commit: Oid, message: String },
    /// The stash whose commit this is leaves the stash list.
    StashDrop { commit: Oid, message: String },
    /// `path` goes back in conflict with the stages of `entry`. It was
    /// the snapshot `resolved` (tree) when this ran.
    Unresolve {
        path: String,
        entry: ConflictEntry,
        resolved: Oid,
    },
    /// The index file and the operation's state files become the saved
    /// state `to`; they are the saved state `from` now.
    RestoreOpState { to: String, from: String },
}

impl Step {
    pub fn inverse(&self) -> Step {
        match self {
            Step::UpdateRefs(updates) => {
                Step::UpdateRefs(updates.iter().rev().map(RefUpdate::inverse).collect())
            }
            Step::Checkout { to, from } => Step::Checkout {
                to: from.clone(),
                from: to.clone(),
            },
            Step::SetConfig { section, to, from } => Step::SetConfig {
                section: section.clone(),
                to: from.clone(),
                from: to.clone(),
            },
            Step::MoveHeadKeep { branch, from, to } => Step::MoveHeadKeep {
                branch: branch.clone(),
                from: *to,
                to: *from,
            },
            Step::RestoreIndex { to, from } => Step::RestoreIndex {
                to: *from,
                from: *to,
            },
            Step::RestorePaths { paths, to, from } => Step::RestorePaths {
                paths: paths.clone(),
                to: *from,
                from: *to,
            },
            Step::StashStore { commit, message } => Step::StashDrop {
                commit: *commit,
                message: message.clone(),
            },
            Step::StashDrop { commit, message } => Step::StashStore {
                commit: *commit,
                message: message.clone(),
            },
            Step::RestoreOpState { to, from } => Step::RestoreOpState {
                to: from.clone(),
                from: to.clone(),
            },
            // Back to the resolved file and its staged entry.
            Step::Unresolve { path, resolved, .. } => Step::RestorePaths {
                paths: vec![path.clone()],
                to: *resolved,
                from: *resolved,
            },
        }
    }

    fn describe(&self) -> Vec<String> {
        match self {
            Step::Checkout { to, .. } => vec![format!("check out {}", to.describe())],
            Step::MoveHeadKeep { branch, from, to } => vec![format!(
                "move {branch} from {} to {}, updating the working tree",
                short(*from),
                short(*to)
            )],
            Step::RestoreIndex { .. } => {
                vec!["restore the index (the staged changes)".to_string()]
            }
            Step::RestorePaths { paths, .. } => vec![format!(
                "restore {} from the safety snapshot (working tree and index): {}",
                if paths.len() == 1 {
                    "1 path".to_string()
                } else {
                    format!("{} paths", paths.len())
                },
                paths_in_words(paths)
            )],
            Step::Unresolve { path, .. } => vec![format!(
                "put {path} back in conflict (its three versions return to the index)"
            )],
            Step::RestoreOpState { .. } => vec![
                "restore the index with its conflicts and the operation's state (merge, \
                 cherry-pick or revert in progress)"
                    .to_string(),
            ],
            Step::StashStore { commit, message } => vec![format!(
                "put the stash \"{}\" ({}) back on the stash list",
                message,
                short(*commit)
            )],
            Step::StashDrop { commit, message } => vec![format!(
                "drop the stash \"{}\" ({})",
                message,
                short(*commit)
            )],
            Step::SetConfig { section, to, .. } => {
                if to.is_empty() {
                    vec![format!("remove the configuration section {section}")]
                } else {
                    vec![format!("restore the configuration section {section}")]
                }
            }
            Step::UpdateRefs(updates) => updates
                .iter()
                .filter_map(|u| match u {
                    RefUpdate::Create { name, new } if !name.starts_with(UNDO_PIN_PREFIX) => {
                        Some(format!("create {} at {}", display_ref(name), short(*new)))
                    }
                    RefUpdate::Update { name, old, new } => Some(format!(
                        "move {} from {} to {}",
                        display_ref(name),
                        short(*old),
                        short(*new)
                    )),
                    RefUpdate::Delete { name, old } if !name.starts_with(UNDO_PIN_PREFIX) => Some(
                        format!("delete {} (was {})", display_ref(name), short(*old)),
                    ),
                    RefUpdate::SetHead { new, .. } => {
                        Some(format!("point HEAD at {}", new.describe()))
                    }
                    _ => None,
                })
                .collect(),
        }
    }
}

fn display_ref(name: &str) -> String {
    name.strip_prefix("refs/heads/")
        .map(str::to_string)
        .or_else(|| name.strip_prefix("refs/tags/").map(|t| format!("tag {t}")))
        .unwrap_or_else(|| name.to_string())
}

/// What `Journal::plan` decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub direction: Direction,
    pub entry: EntrySummary,
    pub steps: Vec<Step>,
    /// Pins the plan's transaction creates; recorded on success.
    pub new_pins: Vec<Pin>,
}

impl Plan {
    pub fn describe(&self) -> Vec<String> {
        self.steps.iter().flat_map(Step::describe).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    pub version: u32,
    pub next_id: u64,
    /// Oldest first; the last is what `undo` takes.
    pub undo: Vec<JournalEntry>,
    /// The last is what `redo` takes.
    pub redo: Vec<JournalEntry>,
    /// No longer redoable, still pinned.
    pub retired: Vec<JournalEntry>,
    /// Where this journal's pins live under `refs/gitpoulpi/undo/`: see
    /// [`pin_name`]. Set from the journal's location when it is loaded.
    #[serde(default)]
    pub pin_ns: String,
}

impl Default for Journal {
    fn default() -> Self {
        Self {
            version: JOURNAL_VERSION,
            next_id: 1,
            undo: Vec::new(),
            redo: Vec::new(),
            retired: Vec::new(),
            pin_ns: String::new(),
        }
    }
}

/// An entry that fell off the history: its pins are to be deleted.
pub type Dropped = Vec<JournalEntry>;

impl Journal {
    /// Writes a `Pending` entry on top of the undo stack and returns its
    /// id. `before` lists the refs the operation may change with their
    /// current values.
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        &mut self,
        kind: OperationKind,
        label: String,
        at: i64,
        head: Option<HeadState>,
        require_head: bool,
        before: Vec<(String, Option<Oid>)>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.undo.push(JournalEntry {
            id,
            kind,
            label,
            at,
            state: EntryState::Pending,
            head: head.map(|head| HeadChange {
                old: head.clone(),
                new: head,
            }),
            require_head,
            refs: before
                .into_iter()
                .map(|(name, old)| RefChange {
                    name,
                    old,
                    new: None,
                })
                .collect(),
            config: Vec::new(),
            pins: Vec::new(),
            effect: Effect::Refs,
            files: None,
            index: None,
            stash: None,
            symrefs: Vec::new(),
            conflict: None,
            opstate: None,
            outside: None,
        });
        id
    }

    /// Records one outside change as a finished entry on top of the undo
    /// stack (it is a new operation: the redo stack retires), with pins
    /// already created under `pins`. Returns the entries that fell off.
    #[allow(clippy::too_many_arguments)]
    pub fn record_outside(
        &mut self,
        change: &OutsideChange,
        head_now: &HeadState,
        label: String,
        at: i64,
        pins: Vec<Pin>,
        max_entries: usize,
    ) -> (u64, Dropped) {
        let id = self.next_id;
        self.next_id += 1;
        let effect = change.effect(head_now);
        let head = change.head.clone().unwrap_or(HeadChange {
            old: head_now.clone(),
            new: head_now.clone(),
        });
        let head_moved = head.old != head.new;
        self.undo.push(JournalEntry {
            id,
            kind: OperationKind::Outside,
            label,
            at,
            state: EntryState::Done,
            head: Some(head),
            require_head: head_moved || effect == Effect::Files,
            refs: change.refs.clone(),
            config: Vec::new(),
            pins,
            effect,
            files: None,
            index: None,
            stash: change.stash.clone(),
            symrefs: Vec::new(),
            conflict: None,
            opstate: None,
            outside: Some(OutsideInfo {
                message: change.message.clone(),
                committer: change.committer.clone(),
            }),
        });
        self.retired.append(&mut self.redo);
        let mut dropped = Vec::new();
        while self.undo.len() > max_entries.max(1) {
            dropped.push(self.undo.remove(0));
        }
        while self.retired.len() > MAX_RETIRED {
            dropped.push(self.retired.remove(0));
        }
        (id, dropped)
    }

    pub fn entry_mut(&mut self, id: u64) -> Option<&mut JournalEntry> {
        self.undo.iter_mut().find(|e| e.id == id)
    }

    /// Completes a pending entry: records what changed, clears the redo
    /// stack into the retention ring, and trims to the bounds
    /// (`max_entries` is `general.undo_depth`; `usize::MAX` for unlimited).
    /// Returns the entries that fell off.
    pub fn finish(
        &mut self,
        id: u64,
        head_new: Option<HeadState>,
        refs: Vec<RefChange>,
        config: Vec<ConfigChange>,
        max_entries: usize,
    ) -> Dropped {
        if let Some(entry) = self.entry_mut(id) {
            entry.state = EntryState::Done;
            if let (Some(head), Some(new)) = (entry.head.as_mut(), head_new) {
                head.new = new;
            }
            entry.refs = refs;
            entry.config = config;
        }
        // A new operation makes the undone ones unreachable by redo.
        self.retired.append(&mut self.redo);
        let mut dropped = Vec::new();
        while self.undo.len() > max_entries.max(1) {
            dropped.push(self.undo.remove(0));
        }
        while self.retired.len() > MAX_RETIRED {
            dropped.push(self.retired.remove(0));
        }
        dropped
    }

    /// Drops the `ResolveConflict` entries, from the undo, redo and retired
    /// stacks, and returns them (their pins are to be deleted). They are
    /// only undoable while their operation runs, so the daemon calls this
    /// once it ended or moved on. With `keep_newest`, only the oldest ones
    /// beyond that many go (a long merge must not push its own pending
    /// entry off the bounded history).
    pub fn drop_resolutions(&mut self, keep_newest: Option<usize>) -> Dropped {
        let mut dropped = Vec::new();
        // Staging done during the operation is as short-lived as a
        // resolution; the abort's own entry is not (it undoes *after* the
        // operation).
        let is_resolution = |e: &JournalEntry| {
            e.kind == OperationKind::ResolveConflict
                || e.opstate.as_ref().is_some_and(|o| !o.aborts)
        };
        let keep = keep_newest.unwrap_or(0);
        let total = self.undo.iter().filter(|e| is_resolution(e)).count();
        let mut excess = total.saturating_sub(keep);
        // Redo and retired entries are older than nothing: they go first.
        for stack in [&mut self.redo, &mut self.retired] {
            let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(stack)
                .into_iter()
                .partition(|e| is_resolution(e) && keep_newest.is_none());
            *stack = kept;
            dropped.extend(gone);
        }
        let mut kept = Vec::new();
        for entry in std::mem::take(&mut self.undo) {
            if is_resolution(&entry) && excess > 0 {
                excess -= 1;
                dropped.push(entry);
            } else {
                kept.push(entry);
            }
        }
        self.undo = kept;
        dropped
    }

    /// Every entry of the three stacks: undo, then redo, then retired.
    pub fn all_entries(&self) -> impl Iterator<Item = &JournalEntry> {
        self.undo
            .iter()
            .chain(self.redo.iter())
            .chain(self.retired.iter())
    }

    /// Marks the `Done` entries `ids` as `Purged` (a rewrite whose old
    /// objects were deleted): their pins are forgotten and returned, since
    /// the refs are deleted with the objects. Entries that are not `Done`
    /// are left as they are.
    pub fn mark_purged(&mut self, ids: &[u64]) -> Vec<Pin> {
        let mut pins = Vec::new();
        for entry in self
            .undo
            .iter_mut()
            .chain(self.redo.iter_mut())
            .chain(self.retired.iter_mut())
        {
            if ids.contains(&entry.id) && entry.state == EntryState::Done {
                entry.state = EntryState::Purged;
                pins.append(&mut entry.pins);
            }
        }
        pins
    }

    /// Removes a pending entry (the operation failed or changed nothing).
    pub fn abort(&mut self, id: u64) -> Option<JournalEntry> {
        let at = self.undo.iter().position(|e| e.id == id)?;
        Some(self.undo.remove(at))
    }

    /// Marks every `Pending` entry `Interrupted`; true if any changed.
    /// Called when the journal is loaded: nothing is running yet.
    pub fn recover(&mut self) -> bool {
        let mut changed = false;
        for entry in self.undo.iter_mut().chain(self.redo.iter_mut()) {
            if entry.state == EntryState::Pending {
                entry.state = EntryState::Interrupted;
                changed = true;
            }
        }
        changed
    }

    pub fn top(&self, direction: Direction) -> Option<&JournalEntry> {
        match direction {
            Direction::Undo => self.undo.last(),
            Direction::Redo => self.redo.last(),
        }
    }

    /// Config sections the top entry of `direction` touched, so the
    /// daemon knows which to read.
    pub fn top_config_sections(&self, direction: Direction) -> Vec<String> {
        self.top(direction)
            .map(|e| e.config.iter().map(|c| c.section.clone()).collect())
            .unwrap_or_default()
    }

    /// Decides whether the top entry of `direction` can run against
    /// `facts`, and if so what to do. Pure: changes nothing.
    pub fn plan(
        &self,
        direction: Direction,
        facts: &RepoFacts,
        expect: Option<u64>,
    ) -> Result<Plan, UndoRefusal> {
        let entry = self.top(direction).ok_or(match direction {
            Direction::Undo => UndoRefusal::NothingToUndo,
            Direction::Redo => UndoRefusal::NothingToRedo,
        })?;
        check_preconditions(entry, direction, facts, expect)?;
        Ok(self.build_steps(entry, direction, facts))
    }

    /// The ordered steps, and the pins they need, that move `entry` in
    /// `direction`. Assumes [`check_preconditions`] passed.
    fn build_steps(&self, entry: &JournalEntry, direction: Direction, facts: &RepoFacts) -> Plan {
        let undo = direction == Direction::Undo;
        let heads = head_ends(entry, undo);
        let head_move = heads.filter(|(at, to)| at != to);
        let (stash_drops, stash_stores) = stash_ends(entry, undo);

        // Pin every value the move is about to un-reference, unless the
        // entry already pins it; the pins ride in the same transaction.
        let mut new_pins: Vec<Pin> = Vec::new();
        let want_pin = |oid: Oid, new_pins: &mut Vec<Pin>| {
            let known = entry
                .pins
                .iter()
                .chain(new_pins.iter())
                .any(|p| p.oid == oid);
            if !known {
                new_pins.push(Pin {
                    name: pin_name(&self.pin_ns, entry.id, entry.pins.len() + new_pins.len()),
                    oid,
                });
            }
        };
        let mut updates: Vec<RefUpdate> = Vec::new();
        for change in &entry.refs {
            let (from, to) = oriented(undo, change.old, change.new);
            if let Some(oid) = from {
                want_pin(oid, &mut new_pins);
            }
            match (from, to) {
                (None, Some(new)) => updates.push(RefUpdate::Create {
                    name: change.name.clone(),
                    new,
                }),
                (Some(old), Some(new)) => updates.push(RefUpdate::Update {
                    name: change.name.clone(),
                    old,
                    new,
                }),
                (Some(old), None) => updates.push(RefUpdate::Delete {
                    name: change.name.clone(),
                    old,
                }),
                (None, None) => {}
            }
        }
        for change in &entry.symrefs {
            let (at, to) = oriented(undo, &change.old, &change.new);
            updates.push(RefUpdate::SetSymbolic {
                name: change.name.clone(),
                old: at.clone(),
                new: to.clone(),
            });
        }
        if let Some((HeadState::Detached(oid), _)) = heads {
            if entry.require_head || head_move.is_some() {
                want_pin(*oid, &mut new_pins);
            }
        }

        // A fast-forward of the branch that is checked out right now moves
        // its files too (`reset --keep`); any other ref moves alone.
        let mut keeps: Vec<(String, Oid, Oid)> = Vec::new();
        let moves_files = entry.effect == Effect::Files
            || matches!(
                entry.kind,
                OperationKind::FastForward | OperationKind::FetchRefAsBranch
            );
        if moves_files {
            if let HeadState::Branch(branch) = &facts.snapshot.head {
                let full = format!("refs/heads/{branch}");
                updates.retain(|u| match u {
                    RefUpdate::Update { name, old, new } if *name == full => {
                        keeps.push((branch.clone(), *old, *new));
                        false
                    }
                    _ => true,
                });
            }
        }

        let mut steps: Vec<Step> = Vec::new();
        let pin_creates: Vec<RefUpdate> = new_pins
            .iter()
            .map(|p| RefUpdate::Create {
                name: p.name.clone(),
                new: p.oid,
            })
            .collect();
        // Operations that check a branch out: undo checks the old HEAD
        // out first and then removes what the operation created; redo
        // has to create first, then check out.
        // A history operation on a detached HEAD moves HEAD alone, with
        // its files: undoing it checks the old commit out again.
        let detached_history = entry.effect == Effect::Files
            && entry.refs.is_empty()
            && matches!(
                heads,
                Some((HeadState::Detached(_), HeadState::Detached(_)))
            );
        let checks_out = detached_history
            || matches!(
                entry.kind,
                OperationKind::Checkout | OperationKind::BranchCreateTracking
            )
            // A checkout made outside moves HEAD (and the files) the same way.
            || (entry.kind == OperationKind::Outside && head_move.is_some())
            // So does a workflow action (a start, a finish that ends on the parent).
            || (entry.kind == OperationKind::FlowRun && head_move.is_some());
        // A workflow action can create the branch it checks out and delete the
        // one it leaves: undo must create before it checks out, and delete
        // after; redo must delete after it checked out.
        let flow_order = entry.kind == OperationKind::FlowRun && checks_out;
        let mut late_updates: Vec<RefUpdate> = Vec::new();
        let mut checkout_step = None;
        if checks_out {
            if let Some((head_at, head_to)) = head_move {
                checkout_step = Some(Step::Checkout {
                    to: head_to.clone(),
                    from: head_at.clone(),
                });
            }
        }
        if checks_out && undo {
            let mut early_creates = pin_creates;
            if flow_order {
                let (early, late): (Vec<_>, Vec<_>) = std::mem::take(&mut updates)
                    .into_iter()
                    .partition(|u| matches!(u, RefUpdate::Create { .. }));
                early_creates.extend(early);
                updates = late;
            }
            if !early_creates.is_empty() {
                steps.push(Step::UpdateRefs(early_creates));
            }
            steps.extend(checkout_step.take());
            if !updates.is_empty() {
                steps.push(Step::UpdateRefs(updates));
            }
        } else {
            if flow_order {
                let (late, early): (Vec<_>, Vec<_>) = std::mem::take(&mut updates)
                    .into_iter()
                    .partition(|u| matches!(u, RefUpdate::Delete { .. }));
                late_updates = late;
                updates = early;
            }
            if !checks_out {
                if let Some((head_at, head_to)) = head_move {
                    updates.push(RefUpdate::SetHead {
                        old: head_at.clone(),
                        new: head_to.clone(),
                    });
                }
            }
            updates.extend(pin_creates);
            if !updates.is_empty() {
                steps.push(Step::UpdateRefs(updates));
            }
        }
        // The snapshot of a hard reset's lost edits: undo writes it back
        // after the branch moved; redo first puts the (clean) state of the
        // new tip in place, so that `reset --keep` has no local change
        // left to refuse over.
        // A resolution undoes by putting the conflict back (its stages,
        // not a snapshot: an index entry cannot be unmerged by one); redo
        // writes the resolved snapshot.
        let unresolve = match (&entry.conflict, &entry.files) {
            (Some(c), Some(f)) if undo => f.paths.first().map(|path| Step::Unresolve {
                path: path.clone(),
                entry: c.entry.clone(),
                resolved: f.after.tree,
            }),
            _ => None,
        };
        let files_step = unresolve.or_else(|| {
            entry.files.as_ref().map(|f| {
                let (at, to) = oriented(undo, &f.before, &f.after);
                Step::RestorePaths {
                    paths: f.paths.clone(),
                    to: to.tree,
                    from: at.tree,
                }
            })
        });
        let files_first = !undo && entry.effect == Effect::Files;
        if files_first {
            steps.extend(files_step.clone());
        }
        for (branch, from, to) in keeps {
            steps.push(Step::MoveHeadKeep { branch, from, to });
        }
        for change in &entry.config {
            let (at, to) = oriented(undo, &change.old, &change.new);
            steps.push(Step::SetConfig {
                section: change.section.clone(),
                to: to.clone(),
                from: at.clone(),
            });
        }
        steps.extend(checkout_step);
        if !late_updates.is_empty() {
            steps.push(Step::UpdateRefs(late_updates));
        }
        if let Some(ix) = &entry.index {
            let (at, to) = oriented(undo, ix.before, ix.after);
            steps.push(Step::RestoreIndex { to, from: at });
        }
        if !files_first {
            steps.extend(files_step);
        }
        // Last: replaces the whole index file, which the steps above
        // (they only know stage-0 entries) must not have the final word on.
        if let Some(os) = &entry.opstate {
            let (at, to) = oriented(undo, &os.before, &os.after);
            steps.push(Step::RestoreOpState {
                to: to.clone(),
                from: at.clone(),
            });
        }
        for r in stash_drops {
            steps.push(Step::StashDrop {
                commit: r.commit,
                message: r.message.clone(),
            });
        }
        for r in stash_stores {
            steps.push(Step::StashStore {
                commit: r.commit,
                message: r.message.clone(),
            });
        }
        Plan {
            direction,
            entry: entry.summary(),
            steps,
            new_pins,
        }
    }

    /// Records a successfully applied plan: the entry moves to the other
    /// stack and keeps the pins the transaction created.
    pub fn commit(&mut self, plan: &Plan) {
        let (from, to) = match plan.direction {
            Direction::Undo => (&mut self.undo, &mut self.redo),
            Direction::Redo => (&mut self.redo, &mut self.undo),
        };
        if let Some(mut entry) = from.pop() {
            entry.pins.extend(plan.new_pins.iter().cloned());
            to.push(entry);
        }
    }

    /// The state a client shows. `facts` is optional: without it nothing
    /// is reported as blocked (the cheap case, right after a mutation).
    pub fn state(&self, facts: Option<&RepoFacts>) -> JournalState {
        let blocked =
            |direction: Direction| facts.and_then(|f| self.plan(direction, f, None).err());
        JournalState {
            undo: self.undo.last().map(JournalEntry::summary),
            redo: self.redo.last().map(JournalEntry::summary),
            blocked: self.undo.last().and_then(|_| blocked(Direction::Undo)),
            redo_blocked: self.redo.last().and_then(|_| blocked(Direction::Redo)),
            disabled: false,
            undo_count: self.undo.len(),
            redo_count: self.redo.len(),
        }
    }
}

/// The first path whose state differs between `expected` and `actual`
/// (both sorted by path), or `None` when they are the same.
fn first_difference(expected: &[PathHash], actual: &[PathHash]) -> Option<String> {
    let actual_by_path: BTreeMap<&str, &PathHash> =
        actual.iter().map(|h| (h.path.as_str(), h)).collect();
    for want in expected {
        if actual_by_path.get(want.path.as_str()) != Some(&want) {
            return Some(want.path.clone());
        }
    }
    let expected_paths: std::collections::BTreeSet<&str> =
        expected.iter().map(|h| h.path.as_str()).collect();
    actual
        .iter()
        .find(|h| !expected_paths.contains(h.path.as_str()))
        .map(|h| h.path.clone())
}

/// Whether `entry` can run in `direction` against `facts`: the repository
/// must look exactly as the entry left it.
fn check_preconditions(
    entry: &JournalEntry,
    direction: Direction,
    facts: &RepoFacts,
    expect: Option<u64>,
) -> Result<(), UndoRefusal> {
    if let Some(expected_id) = expect {
        if expected_id != entry.id {
            return Err(UndoRefusal::StaleEntry {
                expected_id,
                top_id: entry.id,
            });
        }
    }
    if let (EntryState::Pending, Some(op)) = (entry.state, &facts.in_progress) {
        // A conflicted merge or rebase of GitPoulpi's own: not done
        // yet, so say what is in the way rather than "interrupted".
        return Err(UndoRefusal::InProgress { op: op.clone() });
    }
    if entry.state == EntryState::Purged {
        return Err(UndoRefusal::Purged { entry: entry.id });
    }
    if entry.state != EntryState::Done {
        return Err(UndoRefusal::Interrupted { entry: entry.id });
    }
    // A resolution is the one thing undone *during* an operation, and only
    // then: once the operation ended its stages are gone for good.
    // Staging during an operation, and the abort of one, are undone against
    // the operation's own state: running at both ends, or (an abort) only
    // before it.
    if let Some(os) = &entry.opstate {
        let undo = direction == Direction::Undo;
        let running_at = !os.aborts || !undo;
        match (&facts.in_progress, running_at) {
            (Some(op), false) => return Err(UndoRefusal::InProgress { op: op.clone() }),
            (None, true) => return Err(UndoRefusal::OperationEnded),
            _ => {}
        }
        let (at_sum, _) = oriented(undo, &os.before_sum, &os.after_sum);
        if facts.index_sum != *at_sum {
            return Err(UndoRefusal::IndexChanged);
        }
    } else {
        match (&facts.in_progress, entry.conflict.is_some()) {
            (Some(op), false) => return Err(UndoRefusal::InProgress { op: op.clone() }),
            (None, true) if !facts.unmerged => return Err(UndoRefusal::OperationEnded),
            _ => {}
        }
    }

    // What the repository must look like now (`at`).
    let undo = direction == Direction::Undo;
    let heads = head_ends(entry, undo);
    if let (true, Some((head_at, _))) = (entry.require_head, heads) {
        if &facts.snapshot.head != head_at {
            return Err(UndoRefusal::HeadMoved {
                expected: head_at.clone(),
                actual: facts.snapshot.head.clone(),
            });
        }
    }
    for change in &entry.refs {
        let (at, _) = oriented(undo, change.old, change.new);
        let actual = facts.snapshot.refs.get(&change.name).copied();
        if actual != at {
            return Err(UndoRefusal::RefMoved {
                restore: None,
                name: change.name.clone(),
                expected: at,
                actual,
            });
        }
    }
    for change in &entry.config {
        let (at, _) = oriented(undo, &change.old, &change.new);
        let actual = facts
            .config
            .get(&change.section)
            .cloned()
            .unwrap_or_default();
        if sorted(&actual) != sorted(at) {
            return Err(UndoRefusal::ConfigChanged {
                section: change.section.clone(),
            });
        }
    }

    if let Some(ix) = &entry.index {
        let (at, _) = oriented(undo, ix.before, ix.after);
        if facts.index_tree != Some(at) {
            return Err(UndoRefusal::IndexChanged);
        }
    }
    // The stashes: the ones to drop are on the list, the ones to put
    // back are not (anything else was changed outside GitPoulpi).
    let (stash_drops, stash_stores) = stash_ends(entry, undo);
    for r in stash_drops {
        if !facts.stashes.contains(&r.commit) {
            return Err(UndoRefusal::StashChanged { commit: r.commit });
        }
    }
    for r in stash_stores {
        if facts.stashes.contains(&r.commit) {
            return Err(UndoRefusal::StashChanged { commit: r.commit });
        }
    }
    if let Some(f) = &entry.files {
        let (at, _) = oriented(undo, &f.before, &f.after);
        let now = facts.files.get(&entry.id).map(Vec::as_slice).unwrap_or(&[]);
        if let Some(path) = first_difference(&at.hashes, now) {
            return Err(UndoRefusal::PathChanged { path });
        }
    }
    Ok(())
}

/// `(at, to)` for the two ends of a record in a direction: undo goes from
/// `new` back to `old`, redo from `old` to `new`.
fn oriented<T>(undo: bool, old: T, new: T) -> (T, T) {
    if undo {
        (new, old)
    } else {
        (old, new)
    }
}

fn head_ends(entry: &JournalEntry, undo: bool) -> Option<(&HeadState, &HeadState)> {
    entry.head.as_ref().map(|h| oriented(undo, &h.old, &h.new))
}

/// `(to drop, to put back)`: the stashes the move removes and re-creates.
fn stash_ends(entry: &JournalEntry, undo: bool) -> (&[StashRecord], &[StashRecord]) {
    match &entry.stash {
        Some(st) => oriented(undo, st.removed.as_slice(), st.added.as_slice()),
        None => (&[], &[]),
    }
}

fn sorted(pairs: &[(String, String)]) -> Vec<(String, String)> {
    let mut v = pairs.to_vec();
    v.sort();
    v
}

/// Diffs two snapshots into the refs that changed.
pub fn diff_snapshots(before: &RefSnapshot, after: &RefSnapshot) -> Vec<RefChange> {
    let mut out = Vec::new();
    for (name, old) in &before.refs {
        match after.refs.get(name) {
            Some(new) if new == old => {}
            new => out.push(RefChange {
                name: name.clone(),
                old: Some(*old),
                new: new.copied(),
            }),
        }
    }
    for (name, new) in &after.refs {
        if !before.refs.contains_key(name) {
            out.push(RefChange {
                name: name.clone(),
                old: None,
                new: Some(*new),
            });
        }
    }
    out
}

mod outside;
pub use outside::{
    diff as diff_outside, CaptureScope, OutsideChange, OutsideDiff, OutsideSnapshot,
    WorktreeRecord, OUTSIDE_SNAPSHOT_VERSION,
};
mod restore;
pub use restore::{
    plan_restore, readd_lines, valid_ref_name, RestoreAction, RestoreFacts, RestorePlan,
};
#[cfg(test)]
mod tests;
