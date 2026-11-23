use gitpoulpi_core::domain::{
    CheckoutTarget, CommitOptions, HeadState, Oid, OperationKind, RefScope, RefSnapshot, RepoRef,
};
use gitpoulpi_core::domain::{FileStatus, ResetMode};
use gitpoulpi_core::journal::Effect;
use gitpoulpi_core::{GitBackend, Result};
use gitpoulpi_ipc::Request;

use super::store::{branch_section, head_ref};

// ------------------------------------------------- recording operations

/// Whether `request` is an operation the journal records.
pub fn is_journaled(request: &Request) -> bool {
    if !crate::undo_settings::enabled() {
        return false;
    }
    if request.command.traits().journaled {
        return true;
    }
    match request.command {
        OperationKind::Checkout => {
            matches!(
                request
                    .args
                    .get("target")
                    .and_then(|t| serde_json::from_value::<CheckoutTarget>(t.clone()).ok()),
                Some(CheckoutTarget::Branch(_) | CheckoutTarget::Commit(_))
            ) || crate::journal_snap::is_snapshot_op(request)
        }
        // Staging, discard, patch apply, file delete: through snapshots.
        // Stashes, remotes, LFS tracking: `journal_stashes_remotes`.
        _ => {
            crate::journal_snap::is_snapshot_op(request)
                || crate::journal_stashes_remotes::handles(request)
        }
    }
}

/// What was captured before the operation ran.
pub(super) struct Before {
    pub(super) head: Option<HeadState>,
    pub(super) require_head: bool,
    pub(super) refs: Vec<(String, Option<Oid>)>,
    pub(super) snapshot: Option<RefSnapshot>,
    /// Oids to pin before the operation can orphan them.
    pub(super) pins: Vec<Oid>,
    pub(super) config: Vec<(String, String)>,
    /// The config sections the operation may change, as they were before
    /// it ran.
    pub(super) sections: Vec<(String, Vec<(String, String)>)>,
    /// The refs the operation may change; re-read after it ran.
    pub(super) scope: Option<RefScope>,
    /// What undoing a history-moving operation must do besides the ref.
    pub(super) effect: Effect,
    /// The operation cannot be undone exactly with what exists (a hard
    /// reset over local changes, a mixed reset over a staged index): run
    /// it unrecorded rather than record an undo that would lose work.
    pub(super) unrecordable: bool,
    /// The index tree or file snapshot taken before a staging, discard,
    /// patch or delete operation (see `journal_snap`).
    pub(super) snap: Option<crate::journal_snap::Captured>,
    /// What stash, remote and LFS operations read before (`journal_stashes_remotes`).
    pub(super) stash_remote: Option<crate::journal_stashes_remotes::Capture>,
    /// A branch or tag being deleted on a remote: what it held right before
    /// (asked of the server), recorded with the entry once the deletion
    /// succeeded.
    pub(super) remote_ref: Option<gitpoulpi_core::journal::RemoteRefChange>,
    /// The local deletion of the same name done in the same request (the
    /// "delete here and on the remote" gesture), as the request that alone
    /// would have made it: captured and recorded like it, in the same entry.
    pub(super) local_request: Option<Request>,
}

pub(super) fn arg_str(request: &Request, key: &str) -> Option<String> {
    request.args.get(key)?.as_str().map(str::to_string)
}

pub(super) fn capture_before(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
) -> Result<Before> {
    let mut before = Before {
        head: None,
        require_head: false,
        refs: Vec::new(),
        snapshot: None,
        pins: Vec::new(),
        config: Vec::new(),
        sections: Vec::new(),
        scope: None,
        effect: Effect::Refs,
        unrecordable: false,
        snap: None,
        stash_remote: None,
        remote_ref: None,
        local_request: None,
    };
    if crate::journal_stashes_remotes::handles(request) {
        let captured = crate::journal_stashes_remotes::capture(backend, repo, request)?;
        before.pins.extend(captured.pins);
        before.snap = captured.snap;
        before.unrecordable = captured.unrecordable;
        before.stash_remote = Some(captured.capture);
        return Ok(before);
    }
    match request.command {
        OperationKind::Commit
        | OperationKind::Reword
        | OperationKind::AmendCommit
        | OperationKind::SplitCommit => {
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads())?;
            // A reword or a split replaces the tip like an amend does.
            let amend = matches!(
                request.command,
                OperationKind::Reword | OperationKind::AmendCommit | OperationKind::SplitCommit
            ) || request
                .args
                .get("opts")
                .and_then(|o| serde_json::from_value::<CommitOptions>(o.clone()).ok())
                .is_some_and(|o| o.amend);
            match &snap.head {
                HeadState::Branch(name) => {
                    let tip = snap.refs.get(&head_ref(name)).copied();
                    before.refs.push((head_ref(name), tip));
                    if amend {
                        before.pins.extend(tip);
                    }
                }
                HeadState::Detached(oid) => {
                    if amend {
                        before.pins.push(*oid);
                    }
                }
            }
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.snapshot = Some(snap);
        }
        // A file checkout is a snapshot operation (`journal_snap`).
        OperationKind::Checkout if !crate::journal_snap::is_snapshot_op(request) => {
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads())?;
            if let HeadState::Detached(oid) = &snap.head {
                before.pins.push(*oid);
            }
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.snapshot = Some(snap);
        }
        OperationKind::BranchCreate => {
            let name = arg_str(request, "name").unwrap_or_default();
            before.refs.push((head_ref(&name), None));
        }
        OperationKind::BranchDelete => {
            let name = arg_str(request, "name").unwrap_or_default();
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads())?;
            let tip = snap.refs.get(&head_ref(&name)).copied();
            before.refs.push((head_ref(&name), tip));
            before.pins.extend(tip);
            before.config = backend.config_section(repo, &branch_section(&name))?;
            before.snapshot = Some(snap);
        }
        OperationKind::BranchCreateTracking => {
            let name = arg_str(request, "local_name").unwrap_or_default();
            capture_scoped(
                backend,
                repo,
                &mut before,
                RefScope::heads(),
                &[head_ref(&name)],
            )?;
            capture_sections(backend, repo, &mut before, &[branch_section(&name)])?;
            // Checks the new branch out: undo goes back to this HEAD.
            if let Some(snap) = &before.snapshot {
                if let HeadState::Detached(oid) = &snap.head {
                    before.pins.push(*oid);
                }
                before.head = Some(snap.head.clone());
                before.require_head = true;
            }
        }
        OperationKind::BranchRename => {
            let old = arg_str(request, "old_name").unwrap_or_default();
            let new = arg_str(request, "new_name").unwrap_or_default();
            capture_scoped(
                backend,
                repo,
                &mut before,
                RefScope::heads(),
                &[head_ref(&old), head_ref(&new)],
            )?;
            // git moves `branch.<old>.*` to `branch.<new>.*`; the old
            // section is all there is to read.
            capture_sections(backend, repo, &mut before, &[branch_section(&old)])?;
            // HEAD follows a renamed checked-out branch.
            before.head = before.snapshot.as_ref().map(|s| s.head.clone());
        }
        OperationKind::SetRepoConfig => {
            let key = arg_str(request, "key").unwrap_or_default();
            capture_sections(backend, repo, &mut before, &[key])?;
        }
        OperationKind::BranchSetUpstream | OperationKind::BranchSetRemotes => {
            let branch = arg_str(request, "branch").unwrap_or_default();
            capture_sections(backend, repo, &mut before, &[branch_section(&branch)])?;
        }
        OperationKind::TagCreate | OperationKind::TagDelete | OperationKind::TagAnnotate => {
            let name = arg_str(request, "name").unwrap_or_default();
            let full = format!("refs/tags/{name}");
            if request.command == OperationKind::TagCreate {
                // A create of an existing tag fails, so it starts absent.
                before.refs.push((full, None));
            } else {
                // An annotated tag's own object (not the commit) must
                // outlive a delete or an annotate-in-place.
                let old = read_ref(backend, repo, &full)?;
                before.refs.push((full, old));
                before.pins.extend(old);
            }
        }
        OperationKind::FetchRefAsBranch => {
            let branch = arg_str(request, "branch").unwrap_or_default();
            capture_scoped(
                backend,
                repo,
                &mut before,
                RefScope::heads(),
                &[head_ref(&branch)],
            )?;
            capture_sections(backend, repo, &mut before, &[branch_section(&branch)])?;
        }
        OperationKind::Merge
        | OperationKind::CherryPick
        | OperationKind::Revert
        | OperationKind::Rebase
        | OperationKind::RebaseInteractive
        | OperationKind::Reset
        | OperationKind::Pull
        | OperationKind::ComposeCommits => {
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads())?;
            match &snap.head {
                HeadState::Branch(name) => {
                    let tip = snap.refs.get(&head_ref(name)).copied();
                    before.refs.push((head_ref(name), tip));
                    before.pins.extend(tip);
                    before.unrecordable = tip.is_none();
                }
                // No branch moves: HEAD alone does, and the entry records
                // that (undo checks the old commit out again).
                HeadState::Detached(oid) => before.pins.push(*oid),
            }
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.snapshot = Some(snap);
            before.effect = Effect::Refs;
            match request.command {
                OperationKind::Reset => {
                    let mode = request
                        .args
                        .get("mode")
                        .and_then(|m| serde_json::from_value::<ResetMode>(m.clone()).ok());
                    match mode {
                        Some(ResetMode::Soft) => {}
                        // The index before and after (the tree capture),
                        // so undo gives the staged changes back exactly.
                        Some(ResetMode::Mixed) => capture_index(backend, repo, &mut before),
                        _ => {
                            before.effect = Effect::Files;
                            capture_changed_files(backend, repo, &mut before)?;
                        }
                    }
                }
                OperationKind::ComposeCommits => capture_index(backend, repo, &mut before),
                _ => before.effect = Effect::Files,
            }
        }
        // A workflow action: the refs (heads and tags) before, HEAD, and the
        // config of every branch it may touch; the run's own result is
        // diffed against this.
        OperationKind::FlowRun | OperationKind::RemoteGroupRun => {
            let (branches, scope) = if request.command == OperationKind::FlowRun {
                (
                    crate::flow::involved_branches(backend, repo, &request.args),
                    RefScope::default(),
                )
            } else {
                // A grouped run moves local branches only: the fetches of a
                // pull must not be "undone" as ref changes.
                (
                    crate::remote_group::involved_branches(repo, &request.args),
                    RefScope::heads(),
                )
            };
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &scope)?;
            for branch in &branches {
                let tip = snap.refs.get(&head_ref(branch)).copied();
                before.refs.push((head_ref(branch), tip));
                before.pins.extend(tip);
            }
            if let HeadState::Detached(oid) = &snap.head {
                before.pins.push(*oid);
            }
            let sections: Vec<String> = branches.iter().map(|b| branch_section(b)).collect();
            capture_sections(backend, repo, &mut before, &sections)?;
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.scope = Some(scope);
            before.snapshot = Some(snap);
        }
        // Pull all branches: every local branch tip is read (a snapshot of
        // the heads, never the remote-tracking refs the fetch moves). Only
        // the checked-out branch can lose commits (a rebase), so only its tip
        // is pinned: a fast-forwarded branch's old tip stays reachable from
        // the new one.
        OperationKind::PullAllBranches => {
            let scope = RefScope::heads();
            let snap = crate::journal_snap::ref_snapshot(backend, repo, &scope)?;
            match &snap.head {
                HeadState::Branch(name) => {
                    let tip = snap.refs.get(&head_ref(name)).copied();
                    before.refs.push((head_ref(name), tip));
                    before.pins.extend(tip);
                }
                HeadState::Detached(oid) => before.pins.push(*oid),
            }
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.scope = Some(scope);
            before.snapshot = Some(snap);
        }
        // One entry for the whole run: every ref the plan moves (branches,
        // tags, notes) with its old value, and a pin per distinct old tip,
        // so the whole old history stays reachable through the pins.
        OperationKind::RewriteApply => capture_rewrite(backend, repo, request, &mut before)?,
        // The outcome carries everything: nothing to read before.
        OperationKind::FastForward => {}
        _ => {}
    }
    Ok(before)
}

/// Reads the refs named in `track` (within `scope`) before the operation.
pub(super) fn capture_scoped(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    before: &mut Before,
    scope: RefScope,
    track: &[String],
) -> Result<()> {
    let snap = crate::journal_snap::ref_snapshot(backend, repo, &scope)?;
    for name in track {
        before
            .refs
            .push((name.clone(), snap.refs.get(name).copied()));
    }
    before.scope = Some(scope);
    before.snapshot = Some(snap);
    Ok(())
}

/// The raw value of one ref, in one spawn (the `refs/heads/` in the
/// scope makes git name HEAD for free, so no second spawn resolves it).
pub(super) fn read_ref(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    name: &str,
) -> Result<Option<Oid>> {
    let snap = crate::journal_snap::ref_snapshot(
        backend,
        repo,
        &RefScope {
            prefixes: vec![name.to_string(), "refs/heads/".to_string()],
        },
    )?;
    Ok(snap.refs.get(name).copied())
}

/// Staged or modified tracked files (untracked ones survive a hard reset).
pub(super) fn tracked_changes(
    backend: &dyn GitBackend,
    repo: &RepoRef,
) -> Result<Vec<gitpoulpi_core::domain::StatusEntry>> {
    Ok(backend
        .status(repo)?
        .files
        .into_iter()
        .filter(|f| f.index.is_some() || f.worktree.is_some_and(|w| w != FileStatus::Untracked))
        .collect())
}

pub(super) fn has_tracked_changes(backend: &dyn GitBackend, repo: &RepoRef) -> Result<bool> {
    Ok(!tracked_changes(backend, repo)?.is_empty())
}

/// The index as a tree before the operation (a mixed reset, a compose).
/// No tree (unresolved conflicts) means the operation is not recorded.
pub(super) fn capture_index(backend: &dyn GitBackend, repo: &RepoRef, before: &mut Before) {
    match backend.index_tree(repo) {
        Ok(tree) => {
            before.pins.push(tree);
            before.snap = Some(crate::journal_snap::Captured::Index { before: tree });
        }
        Err(_) => before.unrecordable = true,
    }
}

/// A hard reset throws the tracked changes away: snapshot them first,
/// so undo can write them back. Conflicted paths cannot be.
pub(super) fn capture_changed_files(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    before: &mut Before,
) -> Result<()> {
    let changed = tracked_changes(backend, repo)?;
    if changed.is_empty() {
        return Ok(());
    }
    if changed.iter().any(|f| {
        f.index == Some(FileStatus::Conflicted) || f.worktree == Some(FileStatus::Conflicted)
    }) {
        before.unrecordable = true;
        return Ok(());
    }
    let paths: Vec<String> = changed
        .iter()
        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
        .collect();
    let snapshot = backend.snapshot_create(repo, &paths)?;
    before.pins.push(snapshot.tree);
    before.snap = Some(crate::journal_snap::Captured::Files {
        paths,
        before: snapshot,
        conflict: None,
    });
    Ok(())
}

pub(super) fn capture_sections(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    before: &mut Before,
    sections: &[String],
) -> Result<()> {
    for section in sections {
        before
            .sections
            .push((section.clone(), backend.config_section(repo, section)?));
    }
    Ok(())
}

/// The live plan an apply request names, if there is one (an apply without
/// one is refused by the dispatcher and records nothing).
pub(super) fn rewrite_plan(
    backend: Option<&dyn GitBackend>,
    repo: &RepoRef,
    request: &Request,
) -> Option<std::sync::Arc<gitpoulpi_core::domain::rewrite::RewritePlan>> {
    let apply: gitpoulpi_ipc::rewrite::ApplyArgs =
        serde_json::from_value(request.args.clone()).ok()?;
    match backend {
        Some(backend) => crate::rewrite_runs::live_or_load(backend, repo, &apply.run),
        None => crate::rewrite_runs::live(repo, &apply.run),
    }
}

fn capture_rewrite(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
    before: &mut Before,
) -> Result<()> {
    let Some(plan) = rewrite_plan(Some(backend), repo, request) else {
        return Ok(());
    };
    if plan.refs.is_empty() {
        return Ok(());
    }
    let snap = crate::journal_snap::ref_snapshot(backend, repo, &RefScope::heads())?;
    let extracting =
        serde_json::from_value::<gitpoulpi_ipc::rewrite::ApplyArgs>(request.args.clone())
            .is_ok_and(|a| a.target.is_some());
    if extracting {
        // A new repository elsewhere: here only the current branch can move
        // (the submodule commit), and undo moves it back with its files.
        if let HeadState::Branch(name) = &snap.head {
            let tip = snap.refs.get(&head_ref(name)).copied();
            before.refs.push((head_ref(name), tip));
            before.pins.extend(tip);
            before.unrecordable = tip.is_none();
            before.effect = Effect::Files;
            before.head = Some(snap.head.clone());
            before.require_head = true;
            before.snapshot = Some(snap);
        }
        return Ok(());
    }
    for change in &plan.refs {
        // A detached HEAD moves HEAD alone: the entry records it as the head
        // change and pins the commit; every other ref is a ref change.
        if change.name != "HEAD" {
            before.refs.push((change.name.clone(), Some(change.old)));
        }
        if !before.pins.contains(&change.old) {
            before.pins.push(change.old);
        }
    }
    // The content of the checked-out branch changes with its tip: undo
    // moves the files with it (`reset --keep`), as other history operations.
    let head_old = match &snap.head {
        HeadState::Branch(name) => plan
            .refs
            .iter()
            .find(|c| c.name == head_ref(name) || c.name == "HEAD")
            .map(|c| c.old),
        HeadState::Detached(_) => plan.refs.iter().find(|c| c.name == "HEAD").map(|c| c.old),
    };
    if let Some(old) = head_old {
        // The plan says so itself (a plan reloaded after a restart has no
        // rows); a row that changes the tree says it too.
        let tree_changes = plan.counts.head_tree_changed
            || plan.rows.iter().any(|row| {
                row.old == old
                    && row
                        .fields
                        .iter()
                        .any(|f| f.field == gitpoulpi_core::domain::rewrite::RewriteField::Tree)
            });
        if tree_changes {
            before.effect = Effect::Files;
        }
    }
    before.head = Some(snap.head.clone());
    before.require_head = true;
    before.snapshot = Some(snap);
    Ok(())
}
