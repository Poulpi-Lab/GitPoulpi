//! "Restore only this" and "restore a ref to a reflog position": new
//! journaled operations (the undo-safety-net design). The
//! decisions are `gitpoulpi_core::journal::restore`; this module reads the
//! facts, runs the actions and records the entry that makes the restore
//! itself undoable.

use std::path::Path;

use gitpoulpi_core::domain::{
    reflog_refname, short, CheckoutTarget, EntrySummary, HeadState, Oid, OperationKind, RefScope,
    RefUpdate, ReflogPosition, RepoRef, RestoreOutcome, UndoRefusal, REMOTE_TAG_MIRROR_PREFIX,
};
use gitpoulpi_core::journal::{
    plan_restore, readd_lines, ConfigChange, Effect, JournalEntry, RefChange, RestoreAction,
    RestoreFacts, StashChange, StashRecord, SymrefChange,
};
use gitpoulpi_core::{Error, GitBackend, Result};
use gitpoulpi_ipc::Request;
use serde_json::{json, Value};

use super::manual::{run_recorded, Done, Spec};
use super::replay::refusal;
use super::state::{ok_json, state_with_facts};
use super::store::{journal_path, load};
use crate::dispatch::{err, err_kind, repo_ref, Dispatched};

fn flag(request: &Request, key: &str) -> bool {
    request
        .args
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn failure(request_id: u64, e: Error) -> Dispatched {
    match e.typed() {
        Some(kind) => err_kind(request_id, e.to_string(), false, Some(kind)),
        None => err(request_id, e.to_string(), false),
    }
}

fn restore_scope() -> RefScope {
    RefScope {
        prefixes: vec![
            "refs/heads/".into(),
            "refs/tags/".into(),
            "refs/remotes/".into(),
            REMOTE_TAG_MIRROR_PREFIX.into(),
        ],
    }
}

fn facts_for(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    entry: &JournalEntry,
) -> Result<RestoreFacts> {
    let snapshot = crate::journal_snap::ref_snapshot(backend, repo, &restore_scope())?;
    let sections: Vec<String> = entry.config.iter().map(|c| c.section.clone()).collect();
    let config = backend
        .config_sections(repo, &sections)?
        .into_iter()
        .collect();
    let stashes = if entry.stash.is_some() {
        crate::journal_snap::stashes(backend, repo)?
            .into_iter()
            .filter_map(|s| s.commit)
            .collect()
    } else {
        Vec::new()
    };
    let files = match &entry.files {
        Some(f) => backend.path_hashes(repo, &f.paths)?,
        None => Vec::new(),
    };
    Ok(RestoreFacts {
        refs: snapshot.refs,
        symrefs: snapshot.symrefs,
        config,
        stashes,
        files,
    })
}

// ------------------------------------------------------ restore only this

/// `JournalRestore { id, name?, dry_run?, confirmed? }`, under the
/// exclusive repo lock.
pub fn restore_entry(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    if !crate::undo_settings::enabled() {
        return refusal(request.id, UndoRefusal::Disabled);
    }
    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let Some(id) = request.args.get("id").and_then(Value::as_u64) else {
        return err(request.id, "JournalRestore needs an `id`".into(), false);
    };
    let name = request
        .args
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let dry_run = flag(request, "dry_run");
    let Some(path) = journal_path(&repo) else {
        return refusal(request.id, UndoRefusal::NoSuchEntry { id });
    };
    let journal = match load(&path) {
        Ok(j) => j,
        Err(m) => return err(request.id, m, false),
    };
    let Some(entry) = journal.undo.iter().find(|e| e.id == id).cloned() else {
        let undone = journal.redo.iter().any(|e| e.id == id);
        return refusal(
            request.id,
            if undone {
                UndoRefusal::NotRestorable {
                    reason: "it was undone: redo it instead".into(),
                }
            } else {
                UndoRefusal::NoSuchEntry { id }
            },
        );
    };
    let facts = match facts_for(backend, &repo, &entry) {
        Ok(f) => f,
        Err(e) => return failure(request.id, e),
    };
    let exists = |p: &str| repo.root.join(p).symlink_metadata().is_ok();
    let plan = match plan_restore(&entry, &facts, name.as_deref(), &exists) {
        Ok(p) => p,
        Err(r) => return refusal(request.id, r),
    };
    // Lines that are all there already are not an action.
    let mut plan = plan;
    plan.actions.retain(|a| match a {
        RestoreAction::ReAddLines {
            path,
            before,
            after,
        } => readd_would_change(backend, &repo, path, *before, *after),
        _ => true,
    });
    if plan.actions.is_empty() {
        return refusal(request.id, UndoRefusal::AlreadyRestored);
    }
    let created: Vec<String> = plan
        .actions
        .iter()
        .filter_map(|a| match a {
            RestoreAction::CreateRef { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let side_files: Vec<String> = plan
        .actions
        .iter()
        .filter_map(|a| match a {
            RestoreAction::SideWrite { dest, .. } => Some(dest.clone()),
            _ => None,
        })
        .collect();
    let outcome = |applied: bool, new_entry: Option<EntrySummary>| RestoreOutcome {
        entry: Some(plan.entry.clone()),
        steps: plan.describe(),
        applied,
        new_entry,
        refs: created.clone(),
        side_files: side_files.clone(),
    };
    if dry_run {
        return ok_json(request.id, &outcome(false, None));
    }
    let spec = spec_for(&plan.label, &plan.actions, &facts, &repo);
    let message = format!("gitpoulpi: {}", plan.label);
    let summary = match run_recorded(backend, &repo, spec, &|| {
        run_actions(backend, &repo, &plan.actions, &message)
    }) {
        Ok(s) => s,
        Err(e) => return failure(request.id, e),
    };
    let journal = load(&path).ok();
    let mut dispatched = ok_json(request.id, &outcome(true, Some(summary)));
    dispatched.mutated = true;
    dispatched.journal = journal.map(|j| state_with_facts(backend, &repo, &j));
    dispatched
}

fn spec_for(label: &str, actions: &[RestoreAction], facts: &RestoreFacts, _repo: &RepoRef) -> Spec {
    let mut refs_before = Vec::new();
    let mut pins = Vec::new();
    let mut file_paths: Vec<String> = Vec::new();
    for action in actions {
        match action {
            RestoreAction::CreateRef { name, oid, .. } => {
                refs_before.push((name.clone(), facts.refs.get(name).copied()));
                pins.push(*oid);
            }
            RestoreAction::StashStore { commit, .. } => pins.push(*commit),
            _ => {}
        }
        for p in action.written_paths() {
            if !file_paths.contains(&p) {
                file_paths.push(p);
            }
        }
    }
    Spec {
        kind: OperationKind::JournalRestore,
        label: label.to_string(),
        head: None,
        require_head: false,
        effect: Effect::Refs,
        refs_before,
        pins,
        file_paths,
    }
}

/// Runs the actions; on failure puts back whatever already changed.
fn run_actions(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    actions: &[RestoreAction],
    message: &str,
) -> Result<Done> {
    let mut done = Done::default();
    // The files first read for a rollback.
    let file_paths: Vec<String> = actions.iter().flat_map(|a| a.written_paths()).collect();
    let safety = if file_paths.is_empty() {
        None
    } else {
        Some(backend.snapshot_create(repo, &file_paths)?)
    };

    // 1. refs and symbolic refs, one atomic transaction.
    let mut updates: Vec<RefUpdate> = Vec::new();
    let mut symrefs: Vec<SymrefChange> = Vec::new();
    for action in actions {
        match action {
            RestoreAction::CreateRef { name, oid, .. } => {
                updates.push(RefUpdate::Create {
                    name: name.clone(),
                    new: *oid,
                });
                done.refs.push(RefChange {
                    name: name.clone(),
                    old: None,
                    new: Some(*oid),
                });
            }
            RestoreAction::SetSymref { name, target } => {
                symrefs.push(SymrefChange {
                    name: name.clone(),
                    old: None,
                    new: Some(target.clone()),
                });
            }
            _ => {}
        }
    }
    for s in &symrefs {
        updates.push(RefUpdate::SetSymbolic {
            name: s.name.clone(),
            old: None,
            new: s.new.clone(),
        });
    }
    if !updates.is_empty() {
        backend.update_refs(repo, &updates, message)?;
    }
    done.symrefs = symrefs;

    let rollback = |done: &Done, stash: &[Oid]| {
        for c in &done.config {
            let _ = backend.set_config_section(repo, &c.section, &c.old);
        }
        for commit in stash {
            let _ =
                crate::journal_stashes_remotes::run_stash_step(backend, repo, *commit, "", false);
        }
        if let Some(s) = &safety {
            let _ = backend.snapshot_restore(repo, s.tree, &file_paths);
        }
        let inverse: Vec<RefUpdate> = updates.iter().rev().map(RefUpdate::inverse).collect();
        if !inverse.is_empty() {
            let _ = backend.update_refs(repo, &inverse, "gitpoulpi: roll back a restore");
        }
    };

    let mut stashed: Vec<Oid> = Vec::new();
    let mut result: Result<()> = Ok(());
    for action in actions {
        let step = match action {
            RestoreAction::SetConfig { section, to, from } => {
                let r = backend.set_config_section(repo, section, to);
                if r.is_ok() {
                    done.config.push(ConfigChange {
                        section: section.clone(),
                        old: from.clone(),
                        new: to.clone(),
                    });
                }
                r
            }
            RestoreAction::StashStore { commit, message } => {
                let r = backend.stash_store(repo, *commit, message);
                if r.is_ok() {
                    stashed.push(*commit);
                    let st = done.stash.get_or_insert_with(StashChange::default);
                    st.added.push(StashRecord {
                        commit: *commit,
                        message: message.clone(),
                    });
                    done.pins.push(*commit);
                }
                r
            }
            RestoreAction::RestorePaths { paths, snapshot } => {
                backend.snapshot_restore(repo, *snapshot, paths)
            }
            RestoreAction::SideWrite { dest, file, .. } => side_write(backend, repo, dest, file),
            RestoreAction::ReAddLines {
                path,
                before,
                after,
            } => re_add_lines(backend, repo, path, *before, *after),
            RestoreAction::CreateRef { .. } | RestoreAction::SetSymref { .. } => Ok(()),
        };
        if let Err(e) = step {
            result = Err(e);
            break;
        }
    }
    if let Err(e) = result {
        rollback(&done, &stashed);
        return Err(e);
    }
    Ok(done)
}

fn side_write(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    dest: &str,
    file: &gitpoulpi_core::domain::FileHash,
) -> Result<()> {
    let full = repo.root.join(dest);
    if full.symlink_metadata().is_ok() {
        return Err(Error::Backend(format!(
            "{dest} appeared meanwhile; nothing was written over it"
        )));
    }
    let bytes = backend.blob_bytes(repo, file.oid)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Backend(format!("cannot create {}: {e}", parent.display())))?;
    }
    gitpoulpi_cli_backend::snapshot_write_file(&full, file, &bytes)
        .map_err(|e| Error::Backend(format!("cannot write {dest}: {e}")))
}

/// The file `path` as it would be with the entry's lines back, `None` when
/// they are all there.
fn re_added_text(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    path: &str,
    before: Option<gitpoulpi_core::domain::FileHash>,
    after: Option<gitpoulpi_core::domain::FileHash>,
) -> Result<Option<String>> {
    let text = |h: Option<gitpoulpi_core::domain::FileHash>| -> Result<String> {
        Ok(match h {
            Some(h) => String::from_utf8_lossy(&backend.blob_bytes(repo, h.oid)?).into_owned(),
            None => String::new(),
        })
    };
    let (before, after) = (text(before)?, text(after)?);
    let current = match std::fs::read(repo.root.join(path)) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(Error::Backend(format!("cannot read {path}: {e}"))),
    };
    Ok(readd_lines(&before, &after, &current))
}

fn readd_would_change(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    path: &str,
    before: Option<gitpoulpi_core::domain::FileHash>,
    after: Option<gitpoulpi_core::domain::FileHash>,
) -> bool {
    // Unreadable: let the run report the error.
    re_added_text(backend, repo, path, before, after).map_or(true, |t| t.is_some())
}

fn re_add_lines(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    path: &str,
    before: Option<gitpoulpi_core::domain::FileHash>,
    after: Option<gitpoulpi_core::domain::FileHash>,
) -> Result<()> {
    if let Some(updated) = re_added_text(backend, repo, path, before, after)? {
        write_atomic(&repo.root.join(path), updated.as_bytes())
            .map_err(|e| Error::Backend(format!("cannot write {path}: {e}")))?;
    }
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("restore-tmp{}", std::process::id()));
    std::fs::write(&tmp, bytes)?;
    // Keep the file's permissions when it exists.
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    std::fs::rename(&tmp, path)
}

// ------------------------------------------------ restore to a reflog position

/// `ReflogRestore { ref, oid, mode?, dry_run? }`, under the exclusive repo
/// lock. `mode`: absent or `keep` moves the branch like `reset --keep`
/// (local changes stay, the move refuses if one is in the way; with
/// uncommitted changes and no mode it refuses and names the modes);
/// `soft`, `mixed`, `hard` are the existing reset modes, journaled as a
/// reset. A branch that is not checked out just moves.
pub fn reflog_restore(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    if !crate::undo_settings::enabled() {
        return refusal(request.id, UndoRefusal::Disabled);
    }
    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let args = &request.args;
    let name = args.get("ref").and_then(Value::as_str).unwrap_or("HEAD");
    let Some(oid) = args
        .get("oid")
        .and_then(|v| serde_json::from_value::<Oid>(v.clone()).ok())
    else {
        return err(request.id, "ReflogRestore needs an `oid`".into(), false);
    };
    let mode = args.get("mode").and_then(Value::as_str).map(str::to_string);
    let dry_run = flag(request, "dry_run");

    let snapshot = match crate::journal_snap::ref_snapshot(backend, &repo, &RefScope::heads()) {
        Ok(s) => s,
        Err(e) => return failure(request.id, e),
    };
    let refname = reflog_refname(name);
    let head_branch = match &snapshot.head {
        HeadState::Branch(b) => Some(b.clone()),
        HeadState::Detached(_) => None,
    };
    let (target, detached) = if refname == "HEAD" {
        match &snapshot.head {
            HeadState::Branch(b) => (Some(format!("refs/heads/{b}")), false),
            HeadState::Detached(_) => (None, true),
        }
    } else if refname.starts_with("refs/heads/") {
        (Some(refname.clone()), false)
    } else {
        return refusal(
            request.id,
            UndoRefusal::NotRestorable {
                reason: format!("only HEAD and local branches can be restored, not {refname}"),
            },
        );
    };
    let checked_out = match (&target, &head_branch) {
        (Some(t), Some(b)) => *t == format!("refs/heads/{b}"),
        _ => false,
    };
    let current = match &target {
        Some(t) => snapshot.refs.get(t).copied(),
        None => match &snapshot.head {
            HeadState::Detached(o) => Some(*o),
            _ => None,
        },
    };
    if current == Some(oid) {
        return refusal(request.id, UndoRefusal::AlreadyRestored);
    }
    let display = target
        .as_deref()
        .and_then(|t| t.strip_prefix("refs/heads/"))
        .unwrap_or("HEAD")
        .to_string();
    let label = format!("Restore {display} to {}", short(oid));
    let outcome = |applied: bool, steps: Vec<String>, new_entry| RestoreOutcome {
        entry: None,
        steps,
        applied,
        new_entry,
        refs: target.iter().cloned().collect(),
        side_files: Vec::new(),
    };

    // Not checked out: the ref alone moves.
    if !checked_out && !detached {
        let full = target.clone().expect("a branch target");
        let steps = vec![format!(
            "move {display} from {} to {}",
            gitpoulpi_core::domain::short_opt(current),
            short(oid)
        )];
        if dry_run {
            return ok_json(request.id, &outcome(false, steps, None));
        }
        let message = format!("gitpoulpi: {label}");
        let spec = Spec {
            kind: OperationKind::ReflogRestore,
            label: label.clone(),
            head: None,
            require_head: false,
            effect: Effect::Refs,
            refs_before: vec![(full.clone(), current)],
            pins: current.into_iter().chain([oid]).collect(),
            file_paths: Vec::new(),
        };
        let result = run_recorded(backend, &repo, spec, &|| {
            let update = match current {
                Some(old) => RefUpdate::Update {
                    name: full.clone(),
                    old,
                    new: oid,
                },
                None => RefUpdate::Create {
                    name: full.clone(),
                    new: oid,
                },
            };
            backend.update_refs(&repo, &[update], &message)?;
            Ok(Done {
                refs: vec![RefChange {
                    name: full.clone(),
                    old: current,
                    new: Some(oid),
                }],
                ..Done::default()
            })
        });
        return finish(backend, request, &repo, result, |s| {
            outcome(true, steps.clone(), Some(s))
        });
    }

    // Checked out: the working tree must stay safe.
    let dirty = match super::capture::has_tracked_changes(backend, &repo) {
        Ok(d) => d,
        Err(e) => return failure(request.id, e),
    };
    if dirty && mode.is_none() {
        return refusal(
            request.id,
            UndoRefusal::WouldOverwrite {
                detail: "there are uncommitted changes; run again with a mode: keep (move and \
                         keep them, refusing if one is in the way), soft, mixed or hard \
                         (journaled, so Undo brings the changes back)"
                    .into(),
            },
        );
    }
    match mode.as_deref() {
        None | Some("keep") => {}
        Some(m @ ("soft" | "mixed" | "hard")) => {
            let mode_name = match m {
                "soft" => "Soft",
                "mixed" => "Mixed",
                _ => "Hard",
            };
            if dry_run {
                return ok_json(
                    request.id,
                    &outcome(
                        false,
                        vec![format!("reset --{m} {display} to {}", short(oid))],
                        None,
                    ),
                );
            }
            let reset = Request {
                id: request.id,
                session: request.session.clone(),
                repo: request.repo.clone(),
                command: OperationKind::Reset,
                args: json!({ "target": oid, "mode": mode_name }),
            };
            let mut d = crate::dispatch::dispatch(backend, &reset);
            if d.response.result.is_ok() {
                d.response.result = Ok(serde_json::to_value(outcome(
                    true,
                    vec![format!("reset --{m} {display} to {}", short(oid))],
                    None,
                ))
                .unwrap_or(Value::Null));
            }
            return d;
        }
        Some(other) => {
            return err(
                request.id,
                format!("unknown mode {other}: use keep, soft, mixed or hard"),
                false,
            )
        }
    }

    let steps = vec![format!(
        "move {display} from {} to {}, keeping local changes",
        gitpoulpi_core::domain::short_opt(current),
        short(oid)
    )];
    if dry_run {
        return ok_json(request.id, &outcome(false, steps, None));
    }
    let tip = current.expect("a checked-out branch or a detached HEAD has a value");
    let message = format!("gitpoulpi: {label}");
    let (head_before, head_after, refs_before, full) = match (&target, &snapshot.head) {
        (Some(t), HeadState::Branch(b)) => (
            HeadState::Branch(b.clone()),
            HeadState::Branch(b.clone()),
            vec![(t.clone(), Some(tip))],
            Some(t.clone()),
        ),
        _ => (
            HeadState::Detached(tip),
            HeadState::Detached(oid),
            Vec::new(),
            None,
        ),
    };
    let spec = Spec {
        kind: OperationKind::ReflogRestore,
        label: label.clone(),
        head: Some(head_before),
        require_head: true,
        effect: Effect::Files,
        refs_before,
        pins: vec![tip, oid],
        file_paths: Vec::new(),
    };
    let result = run_recorded(backend, &repo, spec, &|| match (&full, &head_branch) {
        (Some(t), Some(b)) => {
            backend.move_head_keep(&repo, b, tip, oid)?;
            Ok(Done {
                head_after: Some(head_after.clone()),
                refs: vec![RefChange {
                    name: t.clone(),
                    old: Some(tip),
                    new: Some(oid),
                }],
                ..Done::default()
            })
        }
        _ => {
            backend.checkout(&repo, &CheckoutTarget::Commit(oid))?;
            let _ = &message;
            Ok(Done {
                head_after: Some(head_after.clone()),
                ..Done::default()
            })
        }
    });
    finish(backend, request, &repo, result, |s| {
        outcome(true, steps.clone(), Some(s))
    })
}

fn finish(
    backend: &dyn GitBackend,
    request: &Request,
    repo: &RepoRef,
    result: Result<EntrySummary>,
    outcome: impl FnOnce(EntrySummary) -> RestoreOutcome,
) -> Dispatched {
    match result {
        Ok(summary) => {
            let state = journal_path(repo)
                .and_then(|p| load(&p).ok())
                .map(|j| state_with_facts(backend, repo, &j));
            let mut d = ok_json(request.id, &outcome(summary));
            d.mutated = true;
            d.journal = state;
            d
        }
        Err(e) => failure(request.id, e),
    }
}

// ----------------------------------------------------- blocked undo hint

/// A `RefMoved` refusal with the reflog position that holds the value the
/// entry expects, so a client can offer "restore main to where Commit left
/// it". Every other refusal is returned as it is.
pub(super) fn with_restore_hint(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refusal: UndoRefusal,
) -> UndoRefusal {
    let UndoRefusal::RefMoved {
        name,
        expected: Some(expected),
        actual,
        ..
    } = &refusal
    else {
        return refusal;
    };
    let restore = reflog_position(backend, repo, name, *expected).map(Box::new);
    UndoRefusal::RefMoved {
        name: name.clone(),
        expected: Some(*expected),
        actual: *actual,
        restore,
    }
}

/// How far back `refname`'s reflog holds `oid`, newest match first.
pub(super) fn reflog_position(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refname: &str,
    oid: Oid,
) -> Option<ReflogPosition> {
    if !refname.starts_with("refs/heads/") {
        return None;
    }
    let page = backend.reflog(repo, refname, 0, 1000).ok()?;
    let (index, entry) = page
        .entries
        .iter()
        .enumerate()
        .find(|(_, e)| e.new == oid)?;
    Some(ReflogPosition {
        refname: refname.to_string(),
        oid,
        index,
        time: entry.time,
        message: entry.message.clone(),
    })
}

/// [`with_restore_hint`] on the `blocked` side of a state.
pub(super) fn enrich_state(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    mut state: gitpoulpi_core::domain::JournalState,
) -> gitpoulpi_core::domain::JournalState {
    state.blocked = state
        .blocked
        .take()
        .map(|r| with_restore_hint(backend, repo, r));
    state.redo_blocked = state
        .redo_blocked
        .take()
        .map(|r| with_restore_hint(backend, repo, r));
    state
}
