//! Whitespace-only conflicts resolve themselves (maintainer answer 4 of
//! the merge-conflict-editor design): when a merge, cherry-pick, revert or
//! rebase step stops on conflicts, every file whose conflicts all differ in
//! whitespace only, and unambiguously so (see
//! `gitpoulpi_core::domain::whitespace_resolution`), is resolved right
//! away. Each goes through the ordinary `ResolveConflict` request, so it is
//! journaled and Ctrl+Z puts the conflict back, and its journal label says
//! "Auto-resolve (whitespace only) <path>". The operation itself stays
//! open for the person to conclude. Files with anything else in them are
//! left alone, whole.

use gitpoulpi_core::domain::{
    compose_resolution, whitespace_resolution, with_fallback_encoding, ConflictKind,
    ConflictResolution, Eol, OperationKind, ResolveOptions,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Request;
use serde_json::json;

use crate::dispatch::{dispatch, repo_ref, Dispatched};

/// What the journal label and the resolved record say.
pub const WHY: &str = "whitespace only";

fn can_leave_conflicts(command: OperationKind) -> bool {
    matches!(
        command,
        OperationKind::Merge
            | OperationKind::CherryPick
            | OperationKind::Revert
            | OperationKind::Rebase
            | OperationKind::RebaseInteractive
            | OperationKind::RebaseContinue
            | OperationKind::RebaseSkip
            | OperationKind::AmContinue
            | OperationKind::AmSkip
    )
}

/// Runs after `request` (already dispatched into `dispatched`): resolves
/// what can be resolved, and hands the journal state of the last
/// resolution to `dispatched` so every client's undo button follows.
pub(crate) fn after_operation(
    backend: &dyn GitBackend,
    request: &Request,
    dispatched: &mut Dispatched,
) {
    if !dispatched.mutated
        || dispatched.response.result.is_err()
        || !can_leave_conflicts(request.command)
    {
        return;
    }
    let Ok(repo) = repo_ref(request) else {
        return;
    };
    // One cheap listing; most operations leave nothing unmerged.
    let Ok(entries) = backend.conflict_entries(&repo) else {
        return;
    };
    let candidates = entries.into_iter().filter(|e| {
        matches!(e.kind, ConflictKind::BothModified | ConflictKind::BothAdded)
            && !e.mode_changed()
            && !e.is_symlink()
            && !e.is_submodule()
    });
    // The document decodes with the same fallback encoding the editor will.
    let show = Request {
        command: OperationKind::ConflictShow,
        ..request.clone()
    };
    let encoding = crate::encoding::for_request(&show);
    for entry in candidates {
        let read = || backend.merge_document(&repo, &entry.path);
        let Ok(doc) = with_fallback_encoding(encoding.as_deref(), read) else {
            continue;
        };
        let Some(picks) = whitespace_resolution(&doc) else {
            continue;
        };
        let mut text = compose_resolution(&doc, &picks);
        if doc.eol == Eol::Crlf {
            text = text.replace('\n', "\r\n");
        }
        if doc.bom {
            text.insert(0, '\u{feff}');
        }
        let resolution = match &doc.encoding {
            Some(label) => ConflictResolution::UseEncoded {
                text,
                encoding: label.clone(),
            },
            None => ConflictResolution::UseContent(text),
        };
        let opts = ResolveOptions {
            stage: true,
            expected: Some(ResolveOptions::oids_of(&entry)),
        };
        let resolve = Request {
            command: OperationKind::ResolveConflict,
            args: json!({
                "path": entry.path,
                "resolution": resolution,
                "opts": opts,
                "auto": WHY,
            }),
            ..request.clone()
        };
        let outcome = dispatch(backend, &resolve);
        match outcome.response.result {
            Ok(_) => {
                tracing::info!(
                    repo = request.repo.as_ref().map_or("", |r| r.0.as_str()),
                    "auto-resolved {} ({WHY}) after {:?}",
                    entry.path.display(),
                    request.command
                );
                if outcome.journal.is_some() {
                    dispatched.journal = outcome.journal;
                }
            }
            Err(e) => tracing::warn!(
                repo = request.repo.as_ref().map_or("", |r| r.0.as_str()),
                "could not auto-resolve {}: {}",
                entry.path.display(),
                e.message
            ),
        }
    }
}
