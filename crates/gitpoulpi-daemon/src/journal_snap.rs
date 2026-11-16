//! The journal's recording of operations that undo through **snapshots**:
//!
//! * staging and unstaging (10 operations) record the index as a tree id
//!   before and after, so undo reads the old tree back;
//! * discard (files, hunk, line, checkout of a file), patch apply and file
//!   delete record a pinned safety snapshot of the touched paths before
//!   and after, so nothing discarded is ever lost.
//!
//! Staging has to stay instant, so the "before" tree of a staging
//! operation is free when the previous staging entry already knows the
//! index file (its trailing checksum names the tree it holds); only the
//! "after" tree costs a `write-tree` on the system backend, and nothing on
//! libgit2.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    CheckoutTarget, DiscardTarget, Oid, OperationKind, RepoRef, StageTarget,
};
use gitpoulpi_core::domain::{ConflictEntry, ResolveOptions};
use gitpoulpi_core::journal::{
    ConflictChange, IndexChange, Journal, JournalEntry, PathChange, RepoFacts, Step,
};
use gitpoulpi_core::{GitBackend, Result};
use gitpoulpi_ipc::Request;

use crate::journal::git_dir;

/// Whether `request` is recorded through a snapshot (index or files).
pub fn is_snapshot_op(request: &Request) -> bool {
    match request.command {
        OperationKind::StageFile
        | OperationKind::StageHunk
        | OperationKind::StageLine
        | OperationKind::StageFolder
        | OperationKind::StageAll
        | OperationKind::UnstageFile
        | OperationKind::UnstageHunk
        | OperationKind::UnstageLine
        | OperationKind::UnstageFolder
        | OperationKind::UnstageAll
        | OperationKind::Discard
        | OperationKind::DiscardHunk
        | OperationKind::DiscardLine
        | OperationKind::PatchApply
        | OperationKind::ApplyCommit
        | OperationKind::DeleteFile
        | OperationKind::ConvertFiles
        | OperationKind::AddToGitignore
        | OperationKind::MailmapAdd
        | OperationKind::GitattributesTreat => true,
        OperationKind::Checkout => {
            matches!(
                checkout_target(request),
                Some(CheckoutTarget::File { .. } | CheckoutTarget::Files { .. })
            )
        }
        // "Save & Mark Resolved": journaled so that Ctrl+Z puts the
        // conflict back. A plain Save (no staging) changes nothing the
        // journal could undo.
        OperationKind::ResolveConflict => resolve_options(request).stage,
        // The abort of a merge, cherry-pick or revert: undone by putting
        // the whole stopped state back (`opstate`).
        OperationKind::OperationAbort => true,
        _ => false,
    }
}

fn resolve_options(request: &Request) -> ResolveOptions {
    request
        .args
        .get("opts")
        .and_then(|o| serde_json::from_value(o.clone()).ok())
        .unwrap_or_default()
}

fn resolve_path(request: &Request) -> Option<String> {
    let path = request.args.get("path")?.as_str()?;
    Some(path.replace('\\', "/"))
}

fn is_staging(command: OperationKind) -> bool {
    matches!(
        command,
        OperationKind::StageFile
            | OperationKind::StageHunk
            | OperationKind::StageLine
            | OperationKind::StageFolder
            | OperationKind::StageAll
            | OperationKind::UnstageFile
            | OperationKind::UnstageHunk
            | OperationKind::UnstageLine
            | OperationKind::UnstageFolder
            | OperationKind::UnstageAll
    )
}

/// A checkout of a branch or commit is a ref operation, handled by the
/// ref capture; only a file checkout is ours.
pub fn checkout_target(request: &Request) -> Option<CheckoutTarget> {
    serde_json::from_value(request.args.get("target")?.clone()).ok()
}

fn stage_target(request: &Request) -> Option<StageTarget> {
    serde_json::from_value(request.args.get("target")?.clone()).ok()
}

fn discard_target(request: &Request) -> Option<DiscardTarget> {
    serde_json::from_value(request.args.get("target")?.clone()).ok()
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The label of a multi-path stage or unstage. A rename (`"rename": true`
/// in the arguments, the paths `[new, old]`) reads `Stage rename old -> new`.
fn files_label(request: &Request, verb: &str, paths: &[PathBuf]) -> String {
    let rename = request
        .args
        .get("rename")
        .and_then(|r| r.as_bool())
        .unwrap_or(false);
    match paths {
        [] => format!("{verb} nothing"),
        [one] => format!("{verb} {}", slash(one)),
        [new, old] if rename => format!("{verb} rename {} -> {}", slash(old), slash(new)),
        [a, b] => format!("{verb} {} and {}", slash(a), slash(b)),
        many => format!("{verb} {} paths", many.len()),
    }
}

/// `Stage a.txt`, `Discard 3 files`, ... The words the toolbar shows.
pub fn label(request: &Request) -> Option<String> {
    if !is_snapshot_op(request) {
        return None;
    }
    let verb = match request.command {
        OperationKind::StageFile
        | OperationKind::StageHunk
        | OperationKind::StageLine
        | OperationKind::StageFolder
        | OperationKind::StageAll => "Stage",
        OperationKind::UnstageFile
        | OperationKind::UnstageHunk
        | OperationKind::UnstageLine
        | OperationKind::UnstageFolder
        | OperationKind::UnstageAll => "Unstage",
        _ => "",
    };
    if is_staging(request.command) {
        return Some(match stage_target(request) {
            Some(StageTarget::File(p)) => format!("{verb} {}", slash(&p)),
            Some(StageTarget::Folder(p)) => format!("{verb} folder {}", slash(&p)),
            Some(StageTarget::Files(paths)) => files_label(request, verb, &paths),
            Some(StageTarget::Hunk(p, _)) => format!("{verb} a hunk of {}", slash(&p)),
            Some(StageTarget::Line(p, _)) => format!("{verb} lines of {}", slash(&p)),
            Some(StageTarget::All) | None => format!("{verb} all changes"),
        });
    }
    if request.command == OperationKind::OperationAbort {
        return Some("Abort the operation".into());
    }
    let files = |n: usize, first: Option<&str>| match (n, first) {
        (1, Some(p)) => p.to_string(),
        (n, _) => format!("{n} files"),
    };
    Some(match request.command {
        OperationKind::Discard => {
            let paths = paths_arg(request, "paths");
            format!(
                "Discard {}",
                files(paths.len(), paths.first().map(String::as_str))
            )
        }
        OperationKind::DiscardHunk => match discard_target(request) {
            Some(DiscardTarget::Hunk(p, _) | DiscardTarget::Line(p, _)) => {
                format!("Discard a hunk of {}", slash(&p))
            }
            None => "Discard a hunk".into(),
        },
        OperationKind::DiscardLine => match discard_target(request) {
            Some(DiscardTarget::Hunk(p, _) | DiscardTarget::Line(p, _)) => {
                format!("Discard lines of {}", slash(&p))
            }
            None => "Discard lines".into(),
        },
        OperationKind::Checkout => match checkout_target(request) {
            Some(CheckoutTarget::File { path, .. }) => format!("Discard {path}"),
            Some(CheckoutTarget::Files { paths, .. }) => {
                format!(
                    "Restore {}",
                    files(paths.len(), paths.first().map(String::as_str))
                )
            }
            _ => "Checkout".into(),
        },
        OperationKind::PatchApply => match request.args.get("patch").and_then(|p| p.as_str()) {
            Some(p) => format!(
                "Apply patch {}",
                Path::new(p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            ),
            None => "Apply patch".into(),
        },
        OperationKind::ApplyCommit => match request
            .args
            .get("commit")
            .and_then(|c| serde_json::from_value::<Oid>(c.clone()).ok())
        {
            Some(c) => format!("Apply changes of {}", gitpoulpi_core::domain::short(c)),
            None => "Apply changes of a commit".into(),
        },
        OperationKind::DeleteFile => {
            let paths = paths_arg(request, "paths");
            format!(
                "Delete {}",
                files(paths.len(), paths.first().map(String::as_str))
            )
        }
        OperationKind::ConvertFiles => {
            let paths = paths_arg(request, "paths");
            format!(
                "Convert {}",
                files(paths.len(), paths.first().map(String::as_str))
            )
        }
        OperationKind::AddToGitignore => {
            let pattern = request
                .args
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            format!("Ignore {pattern}")
        }
        OperationKind::MailmapAdd => {
            let name = request
                .args
                .get("canonical")
                .and_then(|c| c.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            format!("Merge identities into {name}")
        }
        OperationKind::GitattributesTreat => {
            let treat = request
                .args
                .get("treat")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_lowercase();
            let paths = paths_arg(request, "paths");
            format!(
                "Treat {} as {treat}",
                files(paths.len(), paths.first().map(String::as_str))
            )
        }
        OperationKind::ResolveConflict => {
            let path = resolve_path(request).unwrap_or_default();
            // The daemon's own whitespace pass says so in the label, so
            // the undo button and `undo --list` report it.
            match request.args.get("auto").and_then(|a| a.as_str()) {
                Some(why) => format!("Auto-resolve ({why}) {path}"),
                None => format!("Resolve {path}"),
            }
        }
        _ => return None,
    })
}

fn paths_arg(request: &Request, key: &str) -> Vec<String> {
    request
        .args
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// The trailing checksum of the index file (git's own SHA-1 over its
/// content), read without a process. `None` when there is no index, or
/// when `index.skipHash` left the trailer zeroed.
pub fn index_checksum(git_dir: &Path) -> Option<String> {
    let mut file = std::fs::File::open(git_dir.join("index")).ok()?;
    let len = file.metadata().ok()?.len();
    if len < 32 {
        return None;
    }
    file.seek(SeekFrom::End(-20)).ok()?;
    let mut tail = [0u8; 20];
    file.read_exact(&mut tail).ok()?;
    if tail.iter().all(|b| *b == 0) {
        return None;
    }
    Some(tail.iter().map(|b| format!("{b:02x}")).collect())
}

/// What the index holds, conflict stages included (see
/// `gitpoulpi_git2_backend::native_index_fingerprint`).
pub(crate) fn index_fingerprint(repo: &RepoRef) -> Option<String> {
    gitpoulpi_git2_backend::native_index_fingerprint(repo).ok()
}

fn checksum_of(repo: &RepoRef) -> Option<String> {
    git_dir(&repo.root).and_then(|d| index_checksum(&d))
}

/// The index's tree, without a process when libgit2 can read it (it can,
/// bar conflicts and exotic index formats; then the backend decides).
fn index_tree(backend: &dyn GitBackend, repo: &RepoRef) -> Result<Oid> {
    gitpoulpi_git2_backend::native_index_tree(repo).or_else(|_| backend.index_tree(repo))
}

pub(crate) fn snapshot_create(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    paths: &[String],
) -> Result<gitpoulpi_core::domain::Snapshot> {
    gitpoulpi_git2_backend::native_snapshot_create(repo, paths)
        .or_else(|_| backend.snapshot_create(repo, paths))
}

/// The refs under `scope`, without a process when libgit2 can read them.
pub(crate) fn ref_snapshot(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    scope: &gitpoulpi_core::domain::RefScope,
) -> Result<gitpoulpi_core::domain::RefSnapshot> {
    gitpoulpi_git2_backend::native_ref_snapshot(repo, scope)
        .or_else(|_| backend.ref_snapshot(repo, scope))
}

/// The stash list, likewise (the system `git stash list` is two processes).
pub(crate) fn stashes(
    backend: &dyn GitBackend,
    repo: &RepoRef,
) -> Result<Vec<gitpoulpi_core::domain::Stash>> {
    gitpoulpi_git2_backend::native_stashes(repo).or_else(|_| backend.stashes(repo))
}

fn path_hashes(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    paths: &[String],
) -> Result<Vec<gitpoulpi_core::domain::PathHash>> {
    gitpoulpi_git2_backend::native_path_hashes(repo, paths)
        .or_else(|_| backend.path_hashes(repo, paths))
}

/// What was captured before the operation.
pub enum Captured {
    Index {
        before: Oid,
    },
    Files {
        paths: Vec<String>,
        before: gitpoulpi_core::domain::Snapshot,
        /// A resolution: the conflict the path was in.
        conflict: Option<ConflictEntry>,
    },
    /// Staging during a merge, cherry-pick or revert, or the abort of one
    /// (`files`: the paths the abort may rewrite, snapshotted): the index
    /// file and the state files, saved as `before`.
    OpState {
        before: String,
        before_sum: Option<String>,
        /// `Some(op)` for an abort of `op`.
        aborts: Option<String>,
        files: Option<(Vec<String>, gitpoulpi_core::domain::Snapshot)>,
    },
}

impl Captured {
    /// Objects the journal must keep alive before the operation runs.
    pub fn pins(&self) -> Vec<Oid> {
        match self {
            Captured::Index { before } => vec![*before],
            Captured::Files { before, .. } => vec![before.tree],
            Captured::OpState { files, .. } => files.iter().map(|(_, s)| s.tree).collect(),
        }
    }
}

/// What the operation did, ready for the entry.
pub struct Done {
    pub opstate: Option<gitpoulpi_core::journal::OpStateChange>,
    /// Replaces the entry's label (the abort names its operation).
    pub label: Option<String>,
    pub conflict: Option<ConflictChange>,
    pub files: Option<PathChange>,
    pub index: Option<IndexChange>,
    /// Objects to pin besides the "before" ones.
    pub pins: Vec<Oid>,
}

/// The "before" half. `Ok(None)`: this operation is not recorded (a
/// merge is in progress, or the index has no tree because of conflicts).
pub fn capture(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    request: &Request,
    journal: &Journal,
) -> Result<Option<Captured>> {
    let running = git_dir(&repo.root).and_then(|d| crate::journal::in_progress(&d));
    if request.command == OperationKind::OperationAbort
        || (is_staging(request.command) && running.is_some())
    {
        // Rebase keeps its state in a directory of its own and moves
        // branches: not recorded (and so not undoable).
        let Some(op) = running.filter(|op| op != "rebase") else {
            return Ok(None);
        };
        let Some(dir) = git_dir(&repo.root) else {
            return Ok(None);
        };
        let aborts = (request.command == OperationKind::OperationAbort).then_some(op);
        let files = if aborts.is_some() {
            // Every tracked path the operation touched: conflicted, merged
            // in cleanly, or edited since.
            let mut paths: Vec<String> = backend
                .status(repo)?
                .files
                .into_iter()
                .filter(|e| {
                    !matches!(
                        (e.index, e.worktree),
                        (None, Some(gitpoulpi_core::domain::FileStatus::Untracked))
                    )
                })
                .map(|e| slash(&e.path))
                .collect();
            paths.sort();
            paths.dedup();
            let snapshot = snapshot_create(backend, repo, &paths)?;
            Some((paths, snapshot))
        } else {
            None
        };
        let name = crate::opstate::fresh_name();
        if crate::opstate::save(&dir, &name).is_err() {
            return Ok(None);
        }
        return Ok(Some(Captured::OpState {
            before: name,
            before_sum: index_fingerprint(repo),
            aborts,
            files,
        }));
    }
    if is_staging(request.command) {
        // The previous staging entry may already know the index file.
        let now = checksum_of(repo);
        let known = journal
            .undo
            .last()
            .filter(|e| e.state == gitpoulpi_core::domain::EntryState::Done)
            .and_then(|e| e.index.as_ref())
            .and_then(|i| i.known.as_ref())
            .filter(|(sum, _)| Some(sum) == now.as_ref())
            .map(|(_, tree)| *tree);
        let before = match known {
            Some(tree) => tree,
            None => match index_tree(backend, repo) {
                Ok(tree) => tree,
                // Unresolved conflicts: no tree, nothing to record.
                Err(_) => return Ok(None),
            },
        };
        return Ok(Some(Captured::Index { before }));
    }

    let paths: Vec<String> = match request.command {
        OperationKind::Discard | OperationKind::DeleteFile | OperationKind::ConvertFiles => {
            paths_arg(request, "paths")
        }
        OperationKind::AddToGitignore
        | OperationKind::GitattributesTreat
        | OperationKind::MailmapAdd => {
            let default = match request.command {
                OperationKind::AddToGitignore => ".gitignore",
                OperationKind::MailmapAdd => ".mailmap",
                _ => ".gitattributes",
            };
            let file = request
                .args
                .get("file")
                .and_then(|f| f.as_str())
                .map_or_else(|| default.to_string(), |f| f.replace('\\', "/"));
            // The per-clone files live in `.git`, outside what a snapshot
            // can hold: those writes are explicitly not undoable.
            if file.starts_with(".git/") {
                vec![]
            } else {
                vec![file]
            }
        }
        OperationKind::DiscardHunk | OperationKind::DiscardLine => match discard_target(request) {
            Some(DiscardTarget::Hunk(p, _) | DiscardTarget::Line(p, _)) => vec![slash(&p)],
            None => vec![],
        },
        OperationKind::Checkout => match checkout_target(request) {
            Some(CheckoutTarget::File { path, .. }) => vec![path],
            Some(CheckoutTarget::Files { paths, .. }) => {
                paths.iter().map(|p| p.replace('\\', "/")).collect()
            }
            _ => vec![],
        },
        OperationKind::PatchApply => match request.args.get("patch").and_then(|p| p.as_str()) {
            Some(p) => backend.patch_files(repo, &PathBuf::from(p))?,
            None => vec![],
        },
        OperationKind::ApplyCommit => {
            match request
                .args
                .get("commit")
                .and_then(|c| serde_json::from_value::<Oid>(c.clone()).ok())
            {
                Some(commit) => backend
                    .diff_files(
                        repo,
                        gitpoulpi_core::domain::DiffTarget::CommitVsParent(commit),
                    )?
                    .files
                    .iter()
                    .map(|f| slash(&f.path))
                    .collect(),
                None => vec![],
            }
        }
        OperationKind::ResolveConflict => resolve_path(request).into_iter().collect(),
        _ => vec![],
    };
    if paths.is_empty() {
        return Ok(None);
    }
    // A resolution is only recorded for a path that is conflicted now.
    let conflict = if request.command == OperationKind::ResolveConflict {
        match backend
            .conflict_entries(repo)?
            .into_iter()
            .find(|e| slash(&e.path) == paths[0])
        {
            Some(entry) => Some(entry),
            None => return Ok(None),
        }
    } else {
        None
    };
    let before = snapshot_create(backend, repo, &paths)?;
    Ok(Some(Captured::Files {
        paths,
        before,
        conflict,
    }))
}

/// The "after" half. `None`: the operation changed nothing the journal
/// could restore (so no entry).
pub fn after(backend: &dyn GitBackend, repo: &RepoRef, captured: &Captured) -> Option<Done> {
    match captured {
        Captured::OpState {
            before,
            before_sum,
            aborts,
            files,
        } => {
            let dir = git_dir(&repo.root)?;
            let after_sum = index_fingerprint(repo);
            if aborts.is_none() && *before_sum == after_sum {
                // Staging changed nothing.
                crate::opstate::remove(&dir, before);
                return None;
            }
            let name = crate::opstate::fresh_name();
            if crate::opstate::save(&dir, &name).is_err() {
                crate::opstate::remove(&dir, before);
                return None;
            }
            let (files, pins) = match files {
                Some((paths, before_snap)) => {
                    let after_snap = snapshot_create(backend, repo, paths).ok()?;
                    let pin = after_snap.tree;
                    (
                        Some(PathChange {
                            paths: paths.clone(),
                            before: before_snap.clone(),
                            after: after_snap,
                        }),
                        vec![pin],
                    )
                }
                None => (None, vec![]),
            };
            Some(Done {
                opstate: Some(gitpoulpi_core::journal::OpStateChange {
                    before: before.clone(),
                    after: name,
                    before_sum: before_sum.clone(),
                    after_sum,
                    aborts: aborts.is_some(),
                }),
                label: aborts.as_ref().map(|op| format!("Abort {op}")),
                conflict: None,
                files,
                index: None,
                pins,
            })
        }
        Captured::Index { before } => {
            let tree = index_tree(backend, repo).ok()?;
            if tree == *before {
                return None;
            }
            // Read after `index_tree`: `git write-tree` rewrites the index.
            let known = checksum_of(repo).map(|sum| (sum, tree));
            Some(Done {
                opstate: None,
                label: None,
                conflict: None,
                files: None,
                index: Some(IndexChange {
                    before: *before,
                    after: tree,
                    known,
                }),
                pins: vec![tree],
            })
        }
        Captured::Files {
            paths,
            before,
            conflict,
        } => {
            let after = snapshot_create(backend, repo, paths).ok()?;
            if after.hashes == before.hashes {
                return None;
            }
            let pin = after.tree;
            Some(Done {
                opstate: None,
                label: None,
                conflict: conflict.clone().map(|entry| ConflictChange { entry }),
                files: Some(PathChange {
                    paths: paths.clone(),
                    before: before.clone(),
                    after,
                }),
                index: None,
                pins: vec![pin],
            })
        }
    }
}

/// A resolution recorded outside any merge/rebase (a conflicted
/// `stash pop` or `apply`) can still be put back: some path is still
/// unmerged, or a resolved path is still staged (not yet committed).
/// Costs one status read; callers ask only when such an entry exists.
pub fn resolutions_pending(backend: &dyn GitBackend, repo: &RepoRef, journal: &Journal) -> bool {
    if backend
        .conflict_entries(repo)
        .is_ok_and(|entries| !entries.is_empty())
    {
        return true;
    }
    let resolved: Vec<String> = journal
        .undo
        .iter()
        .filter_map(|e| e.conflict.as_ref())
        .map(|c| slash(&c.entry.path))
        .collect();
    if resolved.is_empty() {
        return false;
    }
    backend.status(repo).is_ok_and(|status| {
        status
            .files
            .iter()
            .any(|f| f.index.is_some() && resolved.contains(&slash(&f.path)))
    })
}

/// Adds to `facts` what the top undo and redo entries need to be checked:
/// the index tree and the files' current state.
pub fn fill_facts(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
    facts: &mut RepoFacts,
) {
    let tops: Vec<&JournalEntry> = journal
        .undo
        .last()
        .into_iter()
        .chain(journal.redo.last())
        .filter(|e| e.state == gitpoulpi_core::domain::EntryState::Done)
        .collect();
    let mut index_wanted = false;
    if facts.in_progress.is_none() && tops.iter().any(|e| e.conflict.is_some()) {
        facts.unmerged = resolutions_pending(backend, repo, journal);
    }
    if tops.iter().any(|e| e.opstate.is_some()) {
        facts.index_sum = index_fingerprint(repo);
    }
    for entry in &tops {
        if let Some(files) = &entry.files {
            if let Ok(now) = path_hashes(backend, repo, &files.paths) {
                facts.files.insert(entry.id, now);
            }
        }
        index_wanted |= entry.index.is_some();
    }
    if !index_wanted {
        return;
    }
    let sum = checksum_of(repo);
    let known = tops
        .iter()
        .filter_map(|e| e.index.as_ref()?.known.as_ref())
        .find(|(s, _)| Some(s) == sum.as_ref())
        .map(|(_, tree)| *tree);
    facts.index_tree = known.or_else(|| index_tree(backend, repo).ok());
}

/// After an undo or redo ran its steps: the index is now exactly the
/// tree of the `RestoreIndex` step, and the file's checksum names it.
pub fn note_restored(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    entry: Option<&mut JournalEntry>,
    steps: &[Step],
) {
    let Some(entry) = entry else {
        return;
    };
    if let Some(change) = entry.index.as_mut() {
        for step in steps {
            if let Step::RestoreIndex { to, .. } = step {
                change.known = checksum_of(repo).map(|sum| (sum, *to));
            }
        }
    }
    // An unresolve rewrote the conflicted file itself: what redo must
    // find is the file as the unresolve left it, not as it was before the
    // resolution.
    for step in steps {
        if let (Step::Unresolve { path, .. }, Some(files)) = (step, entry.files.as_mut()) {
            if let Ok(now) = path_hashes(backend, repo, std::slice::from_ref(path)) {
                files.before.hashes = now;
            }
        }
    }
}
