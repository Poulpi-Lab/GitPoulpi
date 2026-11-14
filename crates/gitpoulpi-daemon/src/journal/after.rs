use gitpoulpi_core::domain::{
    short, HeadState, JournalState, Oid, OperationKind, RefScope, RepoRef,
};
use gitpoulpi_core::domain::{EntryState, FastForwardOutcome, RewordOutcome, SplitOutcome};
use gitpoulpi_core::journal::{diff_snapshots, ConfigChange, Journal, RefChange};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;
use serde_json::Value;

use super::capture::{arg_str, read_ref, Before};
use super::labels::label_for;
use super::record::release_pins;
use super::store::{
    branch_section, git_dir, head_ref, in_progress, journal_path, load, save, undo_depth,
};

pub(super) struct After {
    pub(super) head: Option<HeadState>,
    pub(super) refs: Vec<RefChange>,
    pub(super) config: Vec<ConfigChange>,
    pub(super) label: Option<String>,
}

impl After {
    pub(super) fn head_moved(&self, before: &Before) -> bool {
        self.head.is_some() && self.head != before.head
    }
}

/// What the operation changed. Commit, branch create and branch delete
/// are deterministic from the request and the answer (no spawn); a
/// checkout can create refs (git's DWIM), so it re-reads them.
pub(super) fn after_state(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
    before: &Before,
    answer: &Value,
) -> Option<After> {
    match request.command {
        OperationKind::Commit
        | OperationKind::Reword
        | OperationKind::AmendCommit
        | OperationKind::SplitCommit => {
            let new: Oid = if matches!(
                request.command,
                OperationKind::Reword | OperationKind::AmendCommit
            ) {
                serde_json::from_value::<RewordOutcome>(answer.clone())
                    .ok()?
                    .new_head
            } else if request.command == OperationKind::SplitCommit {
                serde_json::from_value::<SplitOutcome>(answer.clone())
                    .ok()?
                    .new_head
            } else {
                serde_json::from_value(answer.clone()).ok()?
            };
            let head = before.head.clone()?;
            let base = label_for(request);
            let label = Some(format!("{base} ({})", short(new)));
            match &head {
                HeadState::Branch(name) => {
                    let old = before.refs.first().and_then(|(_, tip)| *tip);
                    let refs = if old == Some(new) {
                        vec![]
                    } else {
                        vec![RefChange {
                            name: head_ref(name),
                            old,
                            new: Some(new),
                        }]
                    };
                    Some(After {
                        head: Some(head),
                        refs,
                        config: vec![],
                        label,
                    })
                }
                HeadState::Detached(old) => Some(After {
                    head: Some(HeadState::Detached(new)),
                    refs: vec![],
                    config: vec![],
                    label: (*old != new).then_some(label).flatten(),
                }),
            }
        }
        OperationKind::Checkout => {
            let after =
                crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads()).ok()?;
            let refs = diff_snapshots(before.snapshot.as_ref()?, &after);
            Some(After {
                head: Some(after.head),
                refs,
                config: vec![],
                label: None,
            })
        }
        OperationKind::BranchCreate => {
            let name = arg_str(request, "name")?;
            let at: Oid = serde_json::from_value(request.args.get("at")?.clone()).ok()?;
            Some(After {
                head: None,
                refs: vec![RefChange {
                    name: head_ref(&name),
                    old: None,
                    new: Some(at),
                }],
                config: vec![],
                label: Some(format!("Create branch {name} ({})", short(at))),
            })
        }
        OperationKind::BranchDelete => {
            let name = arg_str(request, "name")?;
            let tip = before.refs.first().and_then(|(_, tip)| *tip)?;
            Some(After {
                head: None,
                refs: vec![RefChange {
                    name: head_ref(&name),
                    old: Some(tip),
                    new: None,
                }],
                config: vec![ConfigChange {
                    section: branch_section(&name),
                    old: before.config.clone(),
                    new: vec![],
                }],
                label: Some(format!("Delete branch {name} ({})", short(tip))),
            })
        }
        OperationKind::FastForward => {
            let FastForwardOutcome::FastForwarded { from, to } =
                serde_json::from_value(answer.clone()).ok()?
            else {
                return None;
            };
            let branch = arg_str(request, "branch")?;
            Some(After {
                head: None,
                refs: vec![RefChange {
                    name: head_ref(&branch),
                    old: Some(from),
                    new: Some(to),
                }],
                config: vec![],
                label: Some(format!("Fast-forward {branch} to {}", short(to))),
            })
        }
        OperationKind::TagCreate | OperationKind::TagDelete | OperationKind::TagAnnotate => {
            let (name, old) = before.refs.first().cloned()?;
            let new = match request.command {
                OperationKind::TagDelete => None,
                // A lightweight tag is its commit; anything else is a tag
                // object only a read can name.
                OperationKind::TagCreate
                    if request
                        .args
                        .get("annotated_message")
                        .is_none_or(Value::is_null) =>
                {
                    Some(serde_json::from_value(request.args.get("at")?.clone()).ok()?)
                }
                _ => read_ref(backend, repo, &name).ok()?,
            };
            let refs = if old == new {
                vec![]
            } else {
                vec![RefChange { name, old, new }]
            };
            Some(After {
                head: None,
                refs,
                config: vec![],
                label: None,
            })
        }
        OperationKind::BranchRename => {
            let old_name = arg_str(request, "old_name")?;
            let new_name = arg_str(request, "new_name")?;
            let tip = before
                .refs
                .iter()
                .find(|(n, _)| *n == head_ref(&old_name))
                .and_then(|(_, tip)| *tip)?;
            let old_section = branch_section(&old_name);
            let old_config = before
                .sections
                .iter()
                .find(|(s, _)| *s == old_section)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            // git carries the section along under the new name.
            let moved: Vec<(String, String)> = old_config
                .iter()
                .map(|(k, v)| {
                    (
                        k.replacen(&old_section, &branch_section(&new_name), 1),
                        v.clone(),
                    )
                })
                .collect();
            let mut config = Vec::new();
            if !old_config.is_empty() {
                config.push(ConfigChange {
                    section: old_section,
                    old: old_config,
                    new: vec![],
                });
                config.push(ConfigChange {
                    section: branch_section(&new_name),
                    old: vec![],
                    new: moved,
                });
            }
            Some(After {
                head: before.head.as_ref().map(|head| match head {
                    HeadState::Branch(b) if *b == old_name => HeadState::Branch(new_name.clone()),
                    other => other.clone(),
                }),
                refs: vec![
                    RefChange {
                        name: head_ref(&old_name),
                        old: Some(tip),
                        new: None,
                    },
                    RefChange {
                        name: head_ref(&new_name),
                        old: None,
                        new: Some(tip),
                    },
                ],
                config,
                label: None,
            })
        }
        OperationKind::BranchCreateTracking
        | OperationKind::BranchSetUpstream
        | OperationKind::BranchSetRemotes
        | OperationKind::SetRepoConfig
        | OperationKind::FetchRefAsBranch => {
            let mut refs = Vec::new();
            let mut head = None;
            if let (Some(scope), Some(snapshot)) = (&before.scope, &before.snapshot) {
                let after = crate::journal_snap::ref_snapshot(backend, repo, scope).ok()?;
                // Only the refs this operation names: a scope can be
                // wider (every branch), and another client's concurrent
                // change to an unrelated one is not ours to record.
                let named: Vec<&String> = before.refs.iter().map(|(n, _)| n).collect();
                refs = diff_snapshots(snapshot, &after)
                    .into_iter()
                    .filter(|c| named.contains(&&c.name))
                    .collect();
                if before.head.is_some() {
                    head = Some(after.head);
                }
            }
            let mut config = Vec::new();
            for (section, old) in &before.sections {
                let new = backend.config_section(repo, section).ok()?;
                let key = |v: &[(String, String)]| {
                    let mut v = v.to_vec();
                    v.sort();
                    v
                };
                if key(old) != key(&new) {
                    config.push(ConfigChange {
                        section: section.clone(),
                        old: old.clone(),
                        new,
                    });
                }
            }
            Some(After {
                head,
                refs,
                config,
                label: None,
            })
        }
        OperationKind::FlowRun | OperationKind::RemoteGroupRun => {
            let scope = if request.command == OperationKind::FlowRun {
                RefScope::default()
            } else {
                RefScope::heads()
            };
            let after = crate::journal_snap::ref_snapshot(backend, repo, &scope).ok()?;
            let refs = diff_snapshots(before.snapshot.as_ref()?, &after);
            let mut config = Vec::new();
            for (section, old) in &before.sections {
                let new = backend.config_section(repo, section).ok()?;
                let key = |v: &[(String, String)]| {
                    let mut v = v.to_vec();
                    v.sort();
                    v
                };
                if key(old) != key(&new) {
                    config.push(ConfigChange {
                        section: section.clone(),
                        old: old.clone(),
                        new,
                    });
                }
            }
            // A start does not name its branch in the request: the one it
            // created is in the diff.
            let mut label = if request.command == OperationKind::RemoteGroupRun {
                crate::remote_group::label(request)
            } else {
                crate::flow::label(request)
            };
            if request.command == OperationKind::FlowRun && arg_str(request, "branch").is_none() {
                if let Some(created) = refs
                    .iter()
                    .find(|c| c.old.is_none() && c.name.starts_with("refs/heads/"))
                {
                    let name = created.name.trim_start_matches("refs/heads/");
                    label = format!("{label} {name}");
                }
            }
            Some(After {
                head: Some(after.head),
                refs,
                config,
                label: Some(label),
            })
        }
        OperationKind::Merge
        | OperationKind::CherryPick
        | OperationKind::Revert
        | OperationKind::Rebase
        | OperationKind::RebaseInteractive
        | OperationKind::Reset
        | OperationKind::Pull
        | OperationKind::ComposeCommits => history_after(backend, repo, before),
        OperationKind::RewriteApply => rewrite_after(before, answer),
        _ => None,
    }
}

/// What a history rewrite did, from its own outcome: every ref that moved
/// (the checked-out branch included) as one change each, and a detached
/// HEAD as the head change. No re-read: the outcome is exact, and the daemon
/// refreshes its outside-change snapshot itself once the request ends.
fn rewrite_after(before: &Before, answer: &Value) -> Option<After> {
    let outcome: gitpoulpi_core::domain::rewrite::RewriteOutcome =
        serde_json::from_value(answer.clone()).ok()?;
    // An extraction into a clone moves nothing here.
    if outcome.moved.is_empty() {
        return None;
    }
    let mut head = before.head.clone();
    let mut refs = Vec::new();
    for moved in &outcome.moved {
        if moved.name == "HEAD" {
            if let Some(new) = moved.new {
                head = Some(HeadState::Detached(new));
            }
            continue;
        }
        if moved.new == Some(moved.old) {
            continue;
        }
        refs.push(RefChange {
            name: moved.name.clone(),
            old: Some(moved.old),
            new: moved.new,
        });
    }
    let names: Vec<&str> = outcome.moved.iter().map(|m| m.name.as_str()).collect();
    Some(After {
        head,
        refs,
        config: vec![],
        label: Some(super::labels::rewrite_label(
            outcome.commits_changed,
            &names,
        )),
    })
}

/// What a history-moving operation did: the checked-out branch's tip
/// moved (or did not). HEAD must be the same branch as before: a rebase
/// ends on it, a merge never leaves it.
pub(super) fn history_after(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    before: &Before,
) -> Option<After> {
    if let Some(HeadState::Detached(old)) = before.head.as_ref() {
        let after = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads()).ok()?;
        // A rebase or merge on a detached HEAD ends detached; anything
        // else (a branch got checked out meanwhile) is not ours to record.
        let HeadState::Detached(new) = after.head else {
            return None;
        };
        return (new != *old).then(|| After {
            head: Some(HeadState::Detached(new)),
            refs: vec![],
            config: vec![],
            label: None,
        });
    }
    let HeadState::Branch(branch) = before.head.as_ref()? else {
        return None;
    };
    let old = before.refs.first()?.1?;
    let after = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads()).ok()?;
    if after.head != HeadState::Branch(branch.clone()) {
        return None;
    }
    let new = after.refs.get(&head_ref(branch)).copied()?;
    let refs = if old == new {
        vec![]
    } else {
        vec![RefChange {
            name: head_ref(branch),
            old: Some(old),
            new: Some(new),
        }]
    };
    Some(After {
        head: Some(after.head),
        refs,
        config: vec![],
        label: None,
    })
}

pub(super) fn has_pending_history(journal: &Journal) -> bool {
    journal
        .undo
        .iter()
        .any(|e| e.state == EntryState::Pending && e.kind.traits().history)
}

/// Finishes the `Pending` entry of a merge, rebase, cherry-pick, revert
/// or pull that stopped on a conflict, once nothing is in progress any
/// more: the continue, skip or concluding commit completed it (the entry
/// becomes `Done` with the tip it left), an abort left the branch where
/// it was (the entry is dropped with its pins). Runs after every
/// mutation; it costs a journal read, and a ref read only while such an
/// entry exists. Returns the journal state when it changed anything.
pub fn settle(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    command: OperationKind,
) -> Option<JournalState> {
    let path = journal_path(repo)?;
    if !path.exists() {
        return None;
    }
    let mut journal = load(&path).ok()?;
    let running = in_progress(&git_dir(&repo.root)?).is_some();
    // "Save & Mark Resolved" entries can only be undone while the operation
    // they belong to runs, and only until something else happens to the
    // repository: the next operation (a continue, a commit, an abort) ends
    // them, as does the operation ending on its own.
    let own = matches!(
        command,
        OperationKind::ResolveConflict
            | OperationKind::ConflictUnresolve
            | OperationKind::Undo
            | OperationKind::Redo
            | OperationKind::StageFile
            | OperationKind::StageHunk
            | OperationKind::StageLine
            | OperationKind::StageFolder
            | OperationKind::StageAll
            | OperationKind::UnstageFile
            | OperationKind::UnstageHunk
            | OperationKind::UnstageLine
            | OperationKind::UnstageFolder
            | OperationKind::UnstageAll
            // Reading the state settles but never ends a resolution.
            | OperationKind::JournalState
    );
    let mut pruned = false;
    // A conflicted stash pop leaves unmerged paths with no operation
    // running: the resolutions made then live while those are pending.
    let alive = own
        && (running
            || (journal.undo.iter().any(|e| e.conflict.is_some())
                && crate::journal_snap::resolutions_pending(backend, repo, &journal)));
    if !alive {
        let gone = journal.drop_resolutions(None);
        pruned = !gone.is_empty();
        release_pins(backend, repo, &gone);
    }
    let Some(entry) = journal
        .undo
        .iter()
        .find(|e| e.state == EntryState::Pending && e.kind.traits().history)
        .cloned()
    else {
        if pruned {
            save(&path, &journal).ok()?;
            return Some(journal.state(None));
        }
        return None;
    };
    if running {
        if pruned {
            save(&path, &journal).ok()?;
            return Some(journal.state(None));
        }
        return None;
    }
    let after = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads()).ok()?;
    let old_head = entry.head.as_ref().map(|h| h.old.clone());
    let change = entry.refs.first();
    // A detached HEAD that moved: the entry records HEAD alone.
    if let (Some(HeadState::Detached(old)), None, HeadState::Detached(new)) =
        (&old_head, change, &after.head)
    {
        if old != new {
            let dropped = journal.finish(entry.id, Some(after.head), vec![], vec![], undo_depth());
            release_pins(backend, repo, &dropped);
            save(&path, &journal).ok()?;
            return Some(journal.state(None));
        }
    }
    let moved = match (&old_head, change) {
        (Some(HeadState::Branch(branch)), Some(change)) if after.head == old_head.clone()? => {
            let new = after.refs.get(&head_ref(branch)).copied();
            (new.is_some() && new != change.old).then(|| RefChange {
                name: head_ref(branch),
                old: change.old,
                new,
            })
        }
        _ => None,
    };
    match moved {
        Some(change) => {
            let dropped = journal.finish(
                entry.id,
                Some(after.head),
                vec![change],
                vec![],
                undo_depth(),
            );
            release_pins(backend, repo, &dropped);
        }
        None => {
            if let Some(gone) = journal.abort(entry.id) {
                release_pins(backend, repo, std::slice::from_ref(&gone));
            }
        }
    }
    save(&path, &journal).ok()?;
    Some(journal.state(None))
}
