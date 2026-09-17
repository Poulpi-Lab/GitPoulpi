//! What *Start over* (`autostash_start_over`) needs from the journal: the
//! entry of the operation that auto-stashed, and the removal of the plain
//! staging entries stacked on it.

use gitpoulpi_core::domain::{EntrySummary, OperationKind, RepoRef};
use gitpoulpi_core::GitBackend;

use super::record::release_pins;
use super::store::{journal_path, load, save};

fn is_staging(kind: OperationKind) -> bool {
    use OperationKind as K;
    matches!(
        kind,
        K::StageFile
            | K::StageHunk
            | K::StageLine
            | K::StageFolder
            | K::StageAll
            | K::UnstageFile
            | K::UnstageHunk
            | K::UnstageLine
            | K::UnstageFolder
            | K::UnstageAll
    )
}

/// The newest undo entry that is not plain staging, and what sits above it
/// (summaries, newest first). Staging only ever described an index that
/// Start over is about to throw away, so it does not count as "something
/// else was done since". `Ok(None)`: no journal entry at all.
pub fn start_over_target(
    repo: &RepoRef,
) -> Result<Option<(EntrySummary, Vec<EntrySummary>)>, String> {
    let Some(path) = journal_path(repo) else {
        return Ok(None);
    };
    let journal = load(&path)?;
    let mut above = Vec::new();
    for entry in journal.undo.iter().rev() {
        if is_staging(entry.kind) {
            above.push(entry.summary());
        } else {
            return Ok(Some((entry.summary(), above)));
        }
    }
    Ok(None)
}

/// Removes the staging entries above entry `id` (releasing their pins).
pub fn forget_staging_above(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    id: u64,
) -> Result<(), String> {
    let Some(path) = journal_path(repo) else {
        return Ok(());
    };
    let mut journal = load(&path)?;
    let Some(at) = journal.undo.iter().position(|e| e.id == id) else {
        return Ok(());
    };
    if journal.undo[at + 1..].iter().any(|e| !is_staging(e.kind)) {
        return Err("another operation was recorded meanwhile".into());
    }
    let dropped = journal.undo.split_off(at + 1);
    if dropped.is_empty() {
        return Ok(());
    }
    release_pins(backend, repo, &dropped);
    save(&path, &journal).map_err(|e| e.to_string())
}
