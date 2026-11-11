//! The journal's recording of stashes, remotes and LFS tracking. Three
//! facts beyond the other journal modules:
//!
//! * **stashes** are found by their commit, never by `stash@{n}` (the index
//!   shifts): a push adds one, a drop or pop removes one, and undo or redo
//!   stores or drops that very commit. Push, apply and pop also change files,
//!   so they carry a safety snapshot of the paths they touch;
//! * **remotes** (add, remove, rename, set-url, set-push-url) are config:
//!   the `remote.<name>` section, every `branch.<b>` section that names the
//!   remote, and the remote-tracking refs a remove deletes or a rename moves
//!   (those tips are pinned). `refs/remotes/<r>/HEAD` is a symbolic ref
//!   `ref_snapshot` lists in `symrefs`, not `refs`: a remove or rename
//!   compares the two snapshots and the entry restores it with
//!   `RefUpdate::SetSymbolic`;
//! * **LFS track / untrack** change `.gitattributes` (a path snapshot) and,
//!   with `renormalize`, the index (its tree before and after).
//!
//! Nothing that touches a server is recorded (push, remote branch delete, LFS
//! push, lock).

use gitpoulpi_core::domain::{
    short, FileStatus, LfsTrackOptions, Oid, OperationKind, RefScope, RefSnapshot, RepoRef,
    StashId, REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::journal::{
    diff_snapshots, pin_name, ConfigChange, IndexChange, Journal, JournalEntry, Pin, RefChange,
    StashChange, StashRecord,
};
use gitpoulpi_core::{GitBackend, Result};
use gitpoulpi_ipc::Request;
use serde_json::Value;

use crate::journal_snap::Captured;

/// Whether `request` is recorded by this module.
pub fn handles(request: &Request) -> bool {
    matches!(
        request.command,
        OperationKind::StashPush
            | OperationKind::StashFromCommit
            | OperationKind::StashPop
            | OperationKind::StashApply
            | OperationKind::StashDrop
            | OperationKind::RemoteAdd
            | OperationKind::RemoteRemove
            | OperationKind::RemoteRename
            | OperationKind::RemoteSetUrl
            | OperationKind::RemoteSetPushUrl
            | OperationKind::LfsTrack
            | OperationKind::LfsUntrack
    )
}

fn arg_str(request: &Request, key: &str) -> String {
    request
        .args
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn patterns(request: &Request) -> Vec<String> {
    request
        .args
        .get("patterns")
        .and_then(|p| serde_json::from_value(p.clone()).ok())
        .unwrap_or_default()
}

/// `On main: wip` becomes `wip`; a message git did not write stays.
fn stash_title(message: &str) -> &str {
    for prefix in ["On ", "WIP on "] {
        if let Some(rest) = message.strip_prefix(prefix) {
            if let Some((_, title)) = rest.split_once(": ") {
                return title;
            }
        }
    }
    message
}

/// The words the toolbar shows before the outcome refines them.
pub fn label(request: &Request) -> Option<String> {
    if !handles(request) {
        return None;
    }
    let name = arg_str(request, "name");
    Some(match request.command {
        OperationKind::StashPush => "Stash changes".to_string(),
        OperationKind::StashFromCommit => "Save changes as a stash".to_string(),
        OperationKind::StashPop => "Pop stash".to_string(),
        OperationKind::StashApply => "Apply stash".to_string(),
        OperationKind::StashDrop => "Drop stash".to_string(),
        OperationKind::RemoteAdd => format!("Add remote {name}"),
        OperationKind::RemoteRemove => format!("Remove remote {name}"),
        OperationKind::RemoteRename => format!(
            "Rename remote {} to {}",
            arg_str(request, "old_name"),
            arg_str(request, "new_name")
        ),
        OperationKind::RemoteSetUrl => format!("Set the URL of remote {name}"),
        OperationKind::RemoteSetPushUrl => format!("Set the push URL of remote {name}"),
        OperationKind::LfsTrack => format!("Track {} with LFS", patterns(request).join(", ")),
        OperationKind::LfsUntrack => {
            format!("Stop tracking {} with LFS", patterns(request).join(", "))
        }
        _ => return None,
    })
}

/// What was read before the operation, on top of the snapshot.
pub enum Capture {
    Stash {
        before: Vec<StashRecord>,
        /// The stash a pop, apply or drop works on.
        target: Option<StashRecord>,
    },
    Remote {
        names: Vec<String>,
        sections: Vec<String>,
        config: Vec<(String, Vec<(String, String)>)>,
        refs: Option<RefSnapshot>,
    },
    Lfs {
        index_before: Option<Oid>,
    },
}

/// The result of [`capture`]: the capture itself, the oids to pin before
/// anything changes, the path snapshot to take care of, and whether the
/// operation cannot be recorded exactly (it then runs unrecorded).
pub struct StashRemoteCapture {
    pub capture: Capture,
    pub pins: Vec<Oid>,
    pub snap: Option<Captured>,
    pub unrecordable: bool,
}

fn records(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Vec<(StashId, StashRecord)>> {
    Ok(crate::journal_snap::stashes(backend, repo)?
        .into_iter()
        .filter_map(|s| {
            Some((
                s.id,
                StashRecord {
                    commit: s.commit?,
                    message: s.message,
                },
            ))
        })
        .collect())
}

fn remote_scope(names: &[String]) -> RefScope {
    // Remote-tracking branches and our tag mirror; `refs/heads/` rides
    // along so that git names HEAD without one more spawn.
    let mut prefixes = vec!["refs/heads/".to_string()];
    for n in names {
        prefixes.push(format!("refs/remotes/{n}/"));
        prefixes.push(format!("{REMOTE_TAG_MIRROR_PREFIX}{n}/"));
    }
    RefScope { prefixes }
}

/// Drops what a ref snapshot lists that the journal cannot restore or
/// does not own: HEAD aliases (`refs/remotes/<r>/HEAD` is symbolic) and
/// every branch.
fn remote_refs_only(snapshot: &RefSnapshot) -> RefSnapshot {
    RefSnapshot {
        head: snapshot.head.clone(),
        refs: snapshot
            .refs
            .iter()
            .filter(|(name, _)| !name.starts_with("refs/heads/") && !name.ends_with("/HEAD"))
            .map(|(n, o)| (n.clone(), *o))
            .collect(),
        symrefs: snapshot
            .symrefs
            .iter()
            .filter(|(name, _)| name.starts_with("refs/remotes/"))
            .map(|(n, t)| (n.clone(), t.clone()))
            .collect(),
    }
}

pub fn capture(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
) -> Result<StashRemoteCapture> {
    let mut out = StashRemoteCapture {
        capture: Capture::Lfs { index_before: None },
        pins: Vec::new(),
        snap: None,
        unrecordable: false,
    };
    match request.command {
        OperationKind::StashPush => {
            let before: Vec<StashRecord> = records(backend, repo)?
                .into_iter()
                .map(|(_, r)| r)
                .collect();
            out.capture = Capture::Stash {
                before,
                target: None,
            };
            let named: Vec<String> = request
                .args
                .get("paths")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();
            let untracked = request
                .args
                .get("include_untracked")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            // The auto-stash already scanned the tree for exactly this list.
            let hinted: Option<Vec<String>> = request
                .args
                .get("paths_hint")
                .and_then(|p| serde_json::from_value(p.clone()).ok());
            let paths = if let (true, Some(hinted)) = (named.is_empty(), hinted) {
                hinted
            } else if named.is_empty() {
                let status = backend.status(repo)?;
                if status.files.iter().any(|f| {
                    f.index == Some(FileStatus::Conflicted)
                        || f.worktree == Some(FileStatus::Conflicted)
                }) {
                    out.unrecordable = true;
                    return Ok(out);
                }
                status
                    .files
                    .iter()
                    .filter(|f| {
                        f.index.is_some()
                            || f.worktree
                                .is_some_and(|w| w != FileStatus::Untracked || untracked)
                    })
                    .map(|f| f.path.to_string_lossy().replace('\\', "/"))
                    .collect()
            } else {
                named
            };
            if paths.is_empty() {
                // "No local changes to save": nothing happens.
                out.unrecordable = true;
                return Ok(out);
            }
            let snapshot = crate::journal_snap::snapshot_create(backend, repo, &paths)?;
            out.pins.push(snapshot.tree);
            out.snap = Some(Captured::Files {
                paths,
                before: snapshot,
                conflict: None,
            });
        }
        OperationKind::StashFromCommit => {
            let before: Vec<StashRecord> = records(backend, repo)?
                .into_iter()
                .map(|(_, r)| r)
                .collect();
            out.capture = Capture::Stash {
                before,
                target: None,
            };
        }
        OperationKind::StashPop | OperationKind::StashApply | OperationKind::StashDrop => {
            let id = request
                .args
                .get("id")
                .and_then(|v| serde_json::from_value::<StashId>(v.clone()).ok());
            let all = records(backend, repo)?;
            let target = all
                .iter()
                .find(|(i, _)| Some(*i) == id)
                .map(|(_, r)| r.clone());
            let Some(target) = target else {
                out.unrecordable = true;
                return Ok(out);
            };
            // The stash commit outlives its entry on the list.
            out.pins.push(target.commit);
            if request.command != OperationKind::StashDrop {
                let paths = backend.stash_paths(repo, target.commit)?;
                if paths.is_empty() {
                    out.unrecordable = true;
                } else {
                    let snapshot = crate::journal_snap::snapshot_create(backend, repo, &paths)?;
                    out.pins.push(snapshot.tree);
                    out.snap = Some(Captured::Files {
                        paths,
                        before: snapshot,
                        conflict: None,
                    });
                }
            }
            out.capture = Capture::Stash {
                before: all.into_iter().map(|(_, r)| r).collect(),
                target: Some(target),
            };
        }
        OperationKind::RemoteAdd
        | OperationKind::RemoteRemove
        | OperationKind::RemoteRename
        | OperationKind::RemoteSetUrl
        | OperationKind::RemoteSetPushUrl => {
            let (names, wide) = match request.command {
                OperationKind::RemoteRename => (
                    vec![arg_str(request, "old_name"), arg_str(request, "new_name")],
                    true,
                ),
                OperationKind::RemoteRemove => (vec![arg_str(request, "name")], true),
                _ => (vec![arg_str(request, "name")], false),
            };
            let mut sections: Vec<String> = names.iter().map(|n| format!("remote.{n}")).collect();
            let mut refs = None;
            if wide {
                // The branches that track it lose or rewrite their
                // `remote` key; the refs are deleted or moved.
                sections.extend(backend.remote_dependents(repo, &names[0])?);
                let snapshot =
                    crate::journal_snap::ref_snapshot(backend, repo, &remote_scope(&names))?;
                let only = remote_refs_only(&snapshot);
                // Tracking branches only: a tag mirror can hold thousands
                // of refs, and its tag objects are still held by the tags.
                if request.command == OperationKind::RemoteRemove {
                    out.pins.extend(
                        only.refs
                            .iter()
                            .filter(|(n, _)| n.starts_with("refs/remotes/"))
                            .map(|(_, o)| *o),
                    );
                }
                refs = Some(only);
            }
            // An add fails when the name is taken, so what it replaces is
            // nothing, and that needs no read.
            let config = if request.command == OperationKind::RemoteAdd {
                sections.iter().map(|s| (s.clone(), Vec::new())).collect()
            } else {
                backend.config_sections(repo, &sections)?
            };
            out.capture = Capture::Remote {
                names,
                sections,
                config,
                refs,
            };
        }
        OperationKind::LfsTrack | OperationKind::LfsUntrack => {
            let renormalize = request.command == OperationKind::LfsTrack
                && request
                    .args
                    .get("opts")
                    .and_then(|o| serde_json::from_value::<LfsTrackOptions>(o.clone()).ok())
                    .is_some_and(|o| o.renormalize);
            let mut index_before = None;
            if renormalize {
                match backend.index_tree(repo) {
                    Ok(tree) => {
                        out.pins.push(tree);
                        index_before = Some(tree);
                    }
                    // Unresolved conflicts: no tree to restore.
                    Err(_) => out.unrecordable = true,
                }
            }
            let paths = vec![".gitattributes".to_string()];
            let snapshot = crate::journal_snap::snapshot_create(backend, repo, &paths)?;
            out.pins.push(snapshot.tree);
            out.snap = Some(Captured::Files {
                paths,
                before: snapshot,
                conflict: None,
            });
            out.capture = Capture::Lfs { index_before };
        }
        _ => {}
    }
    Ok(out)
}

/// What the operation changed, besides the files (`journal_snap::after`).
#[derive(Default)]
pub struct Done {
    pub label: Option<String>,
    pub refs: Vec<RefChange>,
    pub config: Vec<ConfigChange>,
    pub stash: Option<StashChange>,
    pub index: Option<IndexChange>,
    /// Remote HEAD aliases the operation moved.
    pub symrefs: Vec<gitpoulpi_core::journal::SymrefChange>,
    /// Objects the "after" state needs kept (redo).
    pub pins: Vec<Oid>,
}

/// `None`: the operation did not leave a state the journal can restore
/// exactly (a conflicted apply, a pop that stopped), so it is not recorded.
pub fn after(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
    capture: &Capture,
) -> Option<Done> {
    match capture {
        Capture::Stash { before, target } => {
            let now: Vec<StashRecord> = records(backend, repo)
                .ok()?
                .into_iter()
                .map(|(_, r)| r)
                .collect();
            let listed =
                |r: &StashRecord, list: &[StashRecord]| list.iter().any(|x| x.commit == r.commit);
            match request.command {
                OperationKind::StashPush | OperationKind::StashFromCommit => {
                    let added: Vec<StashRecord> =
                        now.iter().filter(|r| !listed(r, before)).cloned().collect();
                    let [new] = added.as_slice() else {
                        return None;
                    };
                    Some(Done {
                        label: Some(format!(
                            "{} \"{}\" ({})",
                            if request.command == OperationKind::StashFromCommit {
                                "Save changes as stash"
                            } else {
                                "Stash"
                            },
                            stash_title(&new.message),
                            short(new.commit)
                        )),
                        pins: vec![new.commit],
                        stash: Some(StashChange {
                            added,
                            removed: vec![],
                        }),
                        ..Done::default()
                    })
                }
                OperationKind::StashDrop | OperationKind::StashPop => {
                    let target = target.as_ref()?;
                    // A pop that hit a conflict keeps the stash: not done.
                    if listed(target, &now) {
                        return None;
                    }
                    let verb = if request.command == OperationKind::StashPop {
                        "Pop"
                    } else {
                        "Drop"
                    };
                    Some(Done {
                        label: Some(format!(
                            "{verb} stash \"{}\" ({})",
                            stash_title(&target.message),
                            short(target.commit)
                        )),
                        stash: Some(StashChange {
                            added: vec![],
                            removed: vec![target.clone()],
                        }),
                        ..Done::default()
                    })
                }
                _ => {
                    // Apply: the list is untouched; conflicts cannot be
                    // restored from a snapshot (the index has stages).
                    let target = target.as_ref()?;
                    let conflicted = backend.status(repo).ok()?.files.iter().any(|f| {
                        f.index == Some(FileStatus::Conflicted)
                            || f.worktree == Some(FileStatus::Conflicted)
                    });
                    (!conflicted).then(|| Done {
                        label: Some(format!(
                            "Apply stash \"{}\" ({})",
                            stash_title(&target.message),
                            short(target.commit)
                        )),
                        ..Done::default()
                    })
                }
            }
        }
        Capture::Remote {
            names,
            sections,
            config,
            refs,
        } => {
            let now = backend.config_sections(repo, sections).ok()?;
            let key = |v: &[(String, String)]| {
                let mut v = v.to_vec();
                v.sort();
                v
            };
            let changes: Vec<ConfigChange> = config
                .iter()
                .zip(&now)
                .filter(|((_, old), (_, new))| key(old) != key(new))
                .map(|((section, old), (_, new))| ConfigChange {
                    section: section.clone(),
                    old: old.clone(),
                    new: new.clone(),
                })
                .collect();
            let mut symref_changes = Vec::new();
            let ref_changes = match refs {
                Some(before) => {
                    let after = remote_refs_only(
                        &crate::journal_snap::ref_snapshot(backend, repo, &remote_scope(names))
                            .ok()?,
                    );
                    // `refs/remotes/<r>/HEAD`, which `diff_snapshots` does not see.
                    let aliases: std::collections::BTreeSet<&String> =
                        before.symrefs.keys().chain(after.symrefs.keys()).collect();
                    for name in aliases {
                        let (old, new) = (before.symrefs.get(name), after.symrefs.get(name));
                        if old != new {
                            symref_changes.push(gitpoulpi_core::journal::SymrefChange {
                                name: name.clone(),
                                old: old.cloned(),
                                new: new.cloned(),
                            });
                        }
                    }
                    diff_snapshots(before, &after)
                }
                None => vec![],
            };
            Some(Done {
                refs: ref_changes,
                config: changes,
                symrefs: symref_changes,
                ..Done::default()
            })
        }
        Capture::Lfs { index_before } => {
            let index = match index_before {
                Some(before) => {
                    let tree = backend.index_tree(repo).ok()?;
                    (tree != *before).then_some(IndexChange {
                        before: *before,
                        after: tree,
                        known: None,
                    })
                }
                None => None,
            };
            Some(Done {
                pins: index.iter().map(|i| i.after).collect(),
                index,
                ..Done::default()
            })
        }
    }
}

/// Pins `oids` (the "after" state, which redo needs) under entry `id`'s
/// namespace, after the pins it already has.
pub fn pin_extra(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    ns: &str,
    entry: &mut JournalEntry,
    oids: &[Oid],
) {
    let mut fresh: Vec<Oid> = Vec::new();
    for oid in oids {
        if !entry.pins.iter().any(|p| p.oid == *oid) && !fresh.contains(oid) {
            fresh.push(*oid);
        }
    }
    if fresh.is_empty() {
        return;
    }
    let base = entry.pins.len();
    let extra: Vec<Pin> = fresh
        .iter()
        .enumerate()
        .map(|(n, oid)| Pin {
            name: pin_name(ns, entry.id, base + n),
            oid: *oid,
        })
        .collect();
    let creates: Vec<_> = extra
        .iter()
        .map(|p| gitpoulpi_core::domain::RefUpdate::Create {
            name: p.name.clone(),
            new: p.oid,
        })
        .collect();
    match backend.update_refs(repo, &creates, "gitpoulpi: pin for redo") {
        Ok(()) => entry.pins.extend(extra),
        Err(e) => tracing::warn!("could not pin the state after the operation: {e}"),
    }
}

/// The ref prefixes outside `refs/heads/` and `refs/tags/` that the top
/// undo and redo entries changed, for the facts the preconditions compare.
pub fn extra_prefixes(journal: &Journal) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for entry in journal.undo.last().into_iter().chain(journal.redo.last()) {
        for change in &entry.refs {
            let name = &change.name;
            if name.starts_with("refs/heads/") || name.starts_with("refs/tags/") {
                continue;
            }
            let prefix = [REMOTE_TAG_MIRROR_PREFIX, "refs/remotes/"]
                .iter()
                .find_map(|base| {
                    let rest = name.strip_prefix(base)?;
                    let (remote, _) = rest.split_once('/')?;
                    Some(format!("{base}{remote}/"))
                })
                .unwrap_or_else(|| name.clone());
            if !out.contains(&prefix) {
                out.push(prefix);
            }
        }
    }
    out
}

/// What the preconditions read about the stash list: the commits on it,
/// only when the top entry of a direction changed it.
pub fn stash_facts(backend: &dyn GitBackend, repo: &RepoRef, journal: &Journal) -> Vec<Oid> {
    let wanted = journal
        .undo
        .last()
        .into_iter()
        .chain(journal.redo.last())
        .any(|e| e.stash.is_some());
    if !wanted {
        return Vec::new();
    }
    records(backend, repo)
        .map(|all| all.into_iter().map(|(_, r)| r.commit).collect())
        .unwrap_or_default()
}

/// Runs a stash step of a plan. Entries are found by commit.
pub fn run_stash_step(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    commit: Oid,
    message: &str,
    store: bool,
) -> Result<()> {
    if store {
        return backend.stash_store(repo, commit, message);
    }
    let id = records(backend, repo)?
        .into_iter()
        .find(|(_, r)| r.commit == commit)
        .map(|(id, _)| id)
        .ok_or({
            gitpoulpi_core::Error::UndoRefused(gitpoulpi_core::domain::UndoRefusal::StashChanged {
                commit,
            })
        })?;
    backend.stash_drop(repo, id)
}
