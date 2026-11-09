//! Changes made outside GitPoulpi (a shell, an AI agent), found by diffing
//! two snapshots of the repository's refs (the outside-operations design).
//!
//! The snapshot is read through `GitBackend`'s own primitives (`ref_snapshot`,
//! `stashes`, `worktrees`), so both backends see the same thing, and the diff
//! is pure. The reflog (`GitBackend::reflog`) only names *why* a ref moved
//! ("commit: fix", "rebase (finish)", "reset: moving to HEAD~1") so that a
//! burst of moves becomes one coherent change, never twenty.
//!
//! What is never captured: edits of files in the working tree (no snapshot
//! of them exists), and fetch-only moves of remote-tracking refs (counted,
//! not journaled).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Effect, HeadChange, RefChange, StashChange, StashRecord};
use crate::domain::{HeadState, Oid, RefScope, RefSnapshot, RepoRef};
use crate::{GitBackend, Result};

/// What the daemon last knew of the repository's refs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutsideSnapshot {
    pub version: u32,
    /// HEAD of the worktree this snapshot (and its journal) belongs to.
    pub head: HeadState,
    /// Local branches, tags and remote-tracking refs by full name (raw
    /// values: an annotated tag is its tag object).
    pub refs: BTreeMap<String, Oid>,
    /// The stash list, newest first.
    pub stashes: Vec<StashRecord>,
    /// The repository's worktrees (the main one included), by path.
    pub worktrees: Vec<WorktreeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeRecord {
    pub path: String,
    pub branch: Option<String>,
    #[serde(default)]
    pub is_main: bool,
}

/// Which of the two optional reads `capture` may skip. A read that is
/// skipped is left empty; the daemon skips one only when it knows there is
/// nothing to read (no linked worktree, no stash), because on the system
/// backend each read is a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureScope {
    pub stashes: bool,
    pub worktrees: bool,
}

impl CaptureScope {
    pub const ALL: CaptureScope = CaptureScope {
        stashes: true,
        worktrees: true,
    };
}

pub const OUTSIDE_SNAPSHOT_VERSION: u32 = 1;

fn tracked(name: &str) -> bool {
    name.starts_with("refs/heads/")
        || name.starts_with("refs/tags/")
        || name.starts_with("refs/remotes/")
}

fn undoable(name: &str) -> bool {
    name.starts_with("refs/heads/") || name.starts_with("refs/tags/")
}

impl OutsideSnapshot {
    /// Reads the refs, HEAD, the stash list and the worktree list now.
    pub fn capture(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Self> {
        Self::capture_scoped(backend, repo, CaptureScope::ALL)
    }

    /// [`capture`](Self::capture), skipping the reads `scope` leaves out.
    pub fn capture_scoped(
        backend: &dyn GitBackend,
        repo: &RepoRef,
        scope_: CaptureScope,
    ) -> Result<Self> {
        let scope = RefScope {
            prefixes: vec![
                "refs/heads/".into(),
                "refs/tags/".into(),
                "refs/remotes/".into(),
            ],
        };
        let snap = backend.ref_snapshot(repo, &scope)?;
        let stashes = if scope_.stashes {
            backend.stashes(repo)?
        } else {
            Vec::new()
        };
        let stashes = stashes
            .into_iter()
            .filter_map(|s| {
                s.commit.map(|commit| StashRecord {
                    commit,
                    message: s.message,
                })
            })
            .collect();
        // Not built (no `worktree` feature) or a failed read: no list.
        let listed: Vec<crate::domain::Worktree> = if scope_.worktrees {
            backend.worktrees(repo).unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut worktrees: Vec<WorktreeRecord> = listed
            .into_iter()
            .map(|w| WorktreeRecord {
                path: w.path.to_string_lossy().replace('\\', "/"),
                branch: w.branch,
                is_main: w.is_main,
            })
            .collect();
        worktrees.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self {
            version: OUTSIDE_SNAPSHOT_VERSION,
            head: snap.head,
            // A symbolic ref (`origin/HEAD`) is not a value of its own, and
            // only one backend lists it among the refs.
            refs: snap
                .refs
                .into_iter()
                .filter(|(n, _)| tracked(n) && !snap.symrefs.contains_key(n))
                .collect(),
            stashes,
            worktrees,
        })
    }

    fn ref_snapshot(&self) -> RefSnapshot {
        RefSnapshot {
            head: self.head.clone(),
            refs: self.refs.clone(),
            symrefs: Default::default(),
        }
    }
}

/// One coherent change: becomes one journal entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutsideChange {
    /// The reflog's reason (`commit: fix login`), or a synthesized one
    /// (`deleted branch x`).
    pub message: String,
    /// `Name <email>` from the reflog, when it named one.
    pub committer: Option<String>,
    /// Unix seconds from the reflog, when it named one.
    pub time: Option<i64>,
    pub refs: Vec<RefChange>,
    pub head: Option<HeadChange>,
    pub stash: Option<StashChange>,
    /// `worktree added /path` / `worktree removed /path`: said in the label,
    /// not undone (undo moves refs only).
    pub notes: Vec<String>,
}

impl OutsideChange {
    /// Files follow (`reset --keep`) only when the checked-out branch itself
    /// moved and HEAD stayed on it, or a detached HEAD moved; every other
    /// ref moves alone.
    pub fn effect(&self, head_now: &HeadState) -> Effect {
        let head_stayed = self.head.as_ref().is_none_or(|h| h.old == h.new);
        let moves_checked_out = match head_now {
            HeadState::Branch(b) => {
                let full = format!("refs/heads/{b}");
                self.refs
                    .iter()
                    .any(|r| r.name == full && r.old.is_some() && r.new.is_some())
            }
            HeadState::Detached(_) => false,
        };
        let detached_move = matches!(
            &self.head,
            Some(HeadChange {
                old: HeadState::Detached(_),
                new: HeadState::Detached(_)
            })
        ) && self.refs.is_empty();
        if (head_stayed && moves_checked_out) || detached_move {
            Effect::Files
        } else {
            Effect::Refs
        }
    }

    /// The label the undo history shows: the reason, then what it touched.
    pub fn label(&self) -> String {
        let mut label = self.message.clone();
        // `branch: Created from HEAD` does not say which branch.
        let names: Vec<&str> = self.refs.iter().map(|r| short_ref(&r.name)).collect();
        if !names.is_empty() && !names.iter().all(|n| label.contains(n)) {
            label.push_str(" - ");
            label.push_str(&names.iter().take(3).copied().collect::<Vec<_>>().join(", "));
            if names.len() > 3 {
                label.push_str(&format!(" and {} more", names.len() - 3));
            }
        }
        for note in &self.notes {
            label.push_str(" (");
            label.push_str(note);
            label.push(')');
        }
        label
    }

    /// Every object the entry's undo and redo may need alive.
    pub fn pin_oids(&self) -> Vec<Oid> {
        let mut out: Vec<Oid> = Vec::new();
        let mut add = |oid: Oid| {
            if !out.contains(&oid) {
                out.push(oid);
            }
        };
        for r in &self.refs {
            for oid in [r.old, r.new].into_iter().flatten() {
                add(oid);
            }
        }
        if let Some(st) = &self.stash {
            for r in st.removed.iter().chain(&st.added) {
                add(r.commit);
            }
        }
        if let Some(h) = &self.head {
            for s in [&h.old, &h.new] {
                if let HeadState::Detached(oid) = s {
                    add(*oid);
                }
            }
        }
        out
    }
}

/// What changed between two snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutsideDiff {
    pub changes: Vec<OutsideChange>,
    /// Remote-tracking refs that moved, appeared or vanished (a fetch, a
    /// push, a prune): counted, never journaled.
    pub fetched: usize,
    pub worktrees_added: Vec<String>,
    pub worktrees_removed: Vec<String>,
}

impl OutsideDiff {
    /// True when nothing GitPoulpi would journal changed.
    pub fn is_quiet(&self) -> bool {
        self.changes.is_empty()
    }
}

/// How many changed refs get a reflog lookup; the rest keep a generic
/// reason (a mass prune must not cost thousands of reads).
const MAX_REFLOG_LOOKUPS: usize = 50;

/// The part of a reflog message that names the kind of move.
fn verb(message: &str) -> &str {
    message.split(':').next().unwrap_or(message).trim()
}

fn short_ref(name: &str) -> &str {
    name.strip_prefix("refs/heads/")
        .or_else(|| name.strip_prefix("refs/tags/"))
        .unwrap_or(name)
}

fn kind_of(name: &str) -> &'static str {
    if name.starts_with("refs/tags/") {
        "tag"
    } else {
        "branch"
    }
}

fn plain(head: &HeadState) -> String {
    match head {
        HeadState::Branch(b) => b.clone(),
        HeadState::Detached(oid) => crate::domain::short(*oid),
    }
}

type Why = (String, String, i64);

/// The newest reflog line of `refname` that left it at `new`.
fn why(backend: &dyn GitBackend, repo: &RepoRef, refname: &str, new: Oid) -> Option<Why> {
    let page = backend.reflog(repo, refname, 0, 20).ok()?;
    page.entries
        .into_iter()
        .find(|e| e.new == new)
        // `update-ref` without `-m` leaves no reason: say what moved instead.
        .filter(|e| !e.message.trim().is_empty())
        .map(|e| (e.message, e.committer, e.time))
}

struct Group {
    message: String,
    committer: Option<String>,
    time: Option<i64>,
    refs: Vec<RefChange>,
    head: Option<HeadChange>,
}

/// Diffs `before` against `after`, naming each coherent change: refs are
/// grouped by the kind of move the reflog names, vanished refs together,
/// HEAD joins the change that created the branch it moved to, the stash
/// list is one change.
pub fn diff(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    before: &OutsideSnapshot,
    after: &OutsideSnapshot,
) -> Result<OutsideDiff> {
    let mut out = OutsideDiff::default();
    let mut moved: Vec<RefChange> = Vec::new();
    for c in super::diff_snapshots(&before.ref_snapshot(), &after.ref_snapshot()) {
        if undoable(&c.name) {
            moved.push(c);
        } else {
            out.fetched += 1;
        }
    }
    moved.sort_by(|a, b| a.name.cmp(&b.name));

    let mut groups: Vec<(String, Group)> = Vec::new();
    let mut deleted: Vec<RefChange> = Vec::new();
    let mut lookups = 0usize;
    for change in moved {
        let Some(new) = change.new else {
            deleted.push(change);
            continue;
        };
        let reflog = if change.name.starts_with("refs/heads/") && lookups < MAX_REFLOG_LOOKUPS {
            lookups += 1;
            why(backend, repo, &change.name, new)
        } else {
            None
        };
        let (key, message, committer, time) = match reflog {
            Some((message, committer, time)) => (
                format!("reflog:{}", verb(&message)),
                message,
                Some(committer),
                Some(time),
            ),
            None => {
                let k = kind_of(&change.name);
                let what = if change.old.is_none() {
                    format!("created {k} {}", short_ref(&change.name))
                } else {
                    format!("moved {k} {}", short_ref(&change.name))
                };
                (format!("plain:{what}"), what, None, None)
            }
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, g)) => g.refs.push(change),
            None => groups.push((
                key,
                Group {
                    message,
                    committer,
                    time,
                    refs: vec![change],
                    head: None,
                },
            )),
        }
    }
    if !deleted.is_empty() {
        let names: Vec<String> = deleted
            .iter()
            .take(3)
            .map(|c| format!("{} {}", kind_of(&c.name), short_ref(&c.name)))
            .collect();
        let mut message = format!("deleted {}", names.join(", "));
        if deleted.len() > 3 {
            message.push_str(&format!(" and {} more", deleted.len() - 3));
        }
        groups.push((
            "deleted".into(),
            Group {
                message,
                committer: None,
                time: None,
                refs: deleted,
                head: None,
            },
        ));
    }

    // HEAD joins the group that holds the branch it moved to
    // (`checkout -b x`), else it is a change of its own.
    let head_change = (before.head != after.head).then(|| HeadChange {
        old: before.head.clone(),
        new: after.head.clone(),
    });
    let mut lone_head = None;
    if let Some(h) = head_change {
        let target = match &h.new {
            HeadState::Branch(b) => Some(format!("refs/heads/{b}")),
            HeadState::Detached(_) => None,
        };
        let host = target.and_then(|full| {
            groups
                .iter_mut()
                .find(|(_, g)| g.refs.iter().any(|r| r.name == full))
        });
        match host {
            Some((_, g)) => {
                g.message = format!(
                    "checkout: moving from {} to {}",
                    plain(&h.old),
                    plain(&h.new)
                );
                g.head = Some(h);
            }
            None => lone_head = Some(h),
        }
    }

    let mut changes: Vec<OutsideChange> = groups
        .into_iter()
        .map(|(_, g)| OutsideChange {
            message: g.message,
            committer: g.committer,
            time: g.time,
            refs: g.refs,
            head: g.head,
            stash: None,
            notes: Vec::new(),
        })
        .collect();
    if let Some(h) = lone_head {
        let reason = backend
            .reflog(repo, "HEAD", 0, 1)
            .ok()
            .and_then(|p| p.entries.into_iter().next())
            // Only a line that fits what we saw.
            .filter(|e| match &h.new {
                HeadState::Detached(oid) => e.new == *oid,
                HeadState::Branch(_) => e.message.starts_with("checkout"),
            });
        let (message, committer, time) = match reason {
            Some(e) => (e.message, Some(e.committer), Some(e.time)),
            None => (
                format!(
                    "checkout: moving from {} to {}",
                    plain(&h.old),
                    plain(&h.new)
                ),
                None,
                None,
            ),
        };
        changes.push(OutsideChange {
            message,
            committer,
            time,
            refs: Vec::new(),
            head: Some(h),
            stash: None,
            notes: Vec::new(),
        });
    }

    // The stash list: entries are found by commit, never by position.
    let added: Vec<StashRecord> = after
        .stashes
        .iter()
        .filter(|s| !before.stashes.iter().any(|b| b.commit == s.commit))
        .cloned()
        .collect();
    let removed: Vec<StashRecord> = before
        .stashes
        .iter()
        .filter(|s| !after.stashes.iter().any(|a| a.commit == s.commit))
        .cloned()
        .collect();
    let message = match (added.first(), removed.first()) {
        (Some(a), None) => Some(format!("stash: {}", a.message)),
        (None, Some(r)) => Some(format!("stash dropped: {}", r.message)),
        (Some(a), Some(_)) => Some(format!("stash changed: {}", a.message)),
        (None, None) => None,
    };
    if let Some(message) = message {
        changes.push(OutsideChange {
            message,
            committer: None,
            time: None,
            refs: Vec::new(),
            head: None,
            stash: Some(StashChange { added, removed }),
            notes: Vec::new(),
        });
    }

    // Worktrees: said in the label of the change that created or deleted
    // the branch they carry; a list change alone is not an entry.
    // An empty list is a read that was skipped (no linked worktree then):
    // the main worktree is the only one there was.
    let before_worktrees: Vec<WorktreeRecord> = if before.worktrees.is_empty() {
        after
            .worktrees
            .iter()
            .filter(|w| w.is_main)
            .cloned()
            .collect()
    } else {
        before.worktrees.clone()
    };
    if !after.worktrees.is_empty() {
        for w in &after.worktrees {
            if !before_worktrees.iter().any(|b| b.path == w.path) {
                out.worktrees_added.push(w.path.clone());
            }
        }
        for w in &before_worktrees {
            if !after.worktrees.iter().any(|a| a.path == w.path) {
                out.worktrees_removed.push(w.path.clone());
            }
        }
    }
    for (paths, side, said) in [
        (&out.worktrees_added, &after.worktrees, "worktree added"),
        (
            &out.worktrees_removed,
            &before.worktrees,
            "worktree removed",
        ),
    ] {
        for path in paths {
            let full = side
                .iter()
                .find(|w| &w.path == path)
                .and_then(|w| w.branch.as_ref())
                .map(|b| format!("refs/heads/{b}"));
            let host = changes.iter_mut().find(|c| {
                full.as_ref()
                    .is_some_and(|f| c.refs.iter().any(|r| &r.name == f))
            });
            if let Some(change) = host {
                change.notes.push(format!("{said} {path}"));
            }
        }
    }

    changes.sort_by_key(|c| c.time.unwrap_or(i64::MAX));
    out.changes = changes;
    Ok(out)
}
