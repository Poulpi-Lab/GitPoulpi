use gitpoulpi_core::domain::{
    Direction, JournalListing, JournalState, OperationKind, RefScope, RepoRef,
};
use gitpoulpi_core::journal::{Journal, JournalEntry, RepoFacts};
use gitpoulpi_core::{GitBackend, Result};
use gitpoulpi_ipc::Request;
use serde_json::Value;

use super::store::{git_dir, in_progress, journal_path, load};
use crate::dispatch::{err, ok, repo_ref, Dispatched};

pub(super) fn gather(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
    direction: Direction,
) -> Result<RepoFacts> {
    let mut scope = RefScope::default();
    // Remote-tracking refs and the tag mirror, when the top entries moved
    // some.
    scope
        .prefixes
        .extend(crate::journal_stashes_remotes::extra_prefixes(journal));
    let snapshot = crate::journal_snap::ref_snapshot(backend, repo, &scope)?;
    let config = backend
        .config_sections(repo, &journal.top_config_sections(direction))?
        .into_iter()
        .collect();
    let mut facts = RepoFacts {
        snapshot,
        in_progress: git_dir(&repo.root).and_then(|d| in_progress(&d)),
        config,
        index_tree: None,
        files: Default::default(),
        stashes: crate::journal_stashes_remotes::stash_facts(backend, repo, journal),
        index_sum: None,
        unmerged: false,
    };
    crate::journal_snap::fill_facts(backend, repo, journal, &mut facts);
    Ok(facts)
}

// ---------------------------------------------------------------- reads

/// `JournalState`: the top entries and why they are blocked.
pub fn read_state(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    if !crate::undo_settings::enabled() {
        return ok_json(request.id, &disabled_state());
    }
    let Some(path) = journal_path(&repo) else {
        return ok_json(request.id, &JournalState::default());
    };
    // A conflicted merge or rebase finished outside the app leaves its
    // entry pending until someone settles it; reading is that moment too.
    let _ = super::after::settle(backend, &repo, OperationKind::JournalState);
    let journal = match load(&path) {
        Ok(j) => j,
        Err(m) => return err(request.id, m, false),
    };
    ok_json(request.id, &state_with_facts(backend, &repo, &journal))
}

pub(super) fn state_with_facts(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
) -> JournalState {
    if journal.undo.is_empty() && journal.redo.is_empty() {
        return JournalState::default();
    }
    // Which stack decides the facts: they differ only in the config
    // sections read, so read for the undo side and add the redo side's.
    match gather(backend, repo, journal, Direction::Undo) {
        Ok(mut facts) => {
            if let Ok(redo) =
                backend.config_sections(repo, &journal.top_config_sections(Direction::Redo))
            {
                facts.config.extend(redo);
            }
            super::restore::enrich_state(backend, repo, journal.state(Some(&facts)))
        }
        Err(_) => journal.state(None),
    }
}

/// The journal state after a mutation that did not itself record an
/// entry (a push, a reset, a merge...): it may have moved a ref the top
/// entry depends on, flipping its blocked state. `None` when there is no
/// journal to report on.
pub fn state_after_mutation(backend: &dyn GitBackend, repo: &RepoRef) -> Option<JournalState> {
    if !crate::undo_settings::enabled() {
        return None;
    }
    let path = journal_path(repo)?;
    if !path.exists() {
        return None;
    }
    let _ = super::after::settle(backend, repo, OperationKind::JournalState);
    let journal = load(&path).ok()?;
    Some(state_with_facts(backend, repo, &journal))
}

/// `JournalList { all }`.
pub fn read_list(request: &Request) -> Dispatched {
    let repo = match repo_ref(request) {
        Ok(r) => r,
        Err(m) => return err(request.id, m, false),
    };
    let all = request
        .args
        .get("all")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !crate::undo_settings::enabled() {
        return ok_json(request.id, &JournalListing::default());
    }
    let Some(path) = journal_path(&repo) else {
        return ok_json(request.id, &JournalListing::default());
    };
    let journal = match load(&path) {
        Ok(j) => j,
        Err(m) => return err(request.id, m, false),
    };
    let listed = |entries: &[JournalEntry]| entries.iter().map(JournalEntry::listed).collect();
    ok_json(
        request.id,
        &JournalListing {
            undo: listed(&journal.undo),
            redo: listed(&journal.redo),
            retired: if all {
                listed(&journal.retired)
            } else {
                vec![]
            },
        },
    )
}

/// What `JournalState` answers while undo is turned off.
pub(super) fn disabled_state() -> JournalState {
    JournalState {
        disabled: true,
        ..JournalState::default()
    }
}

pub(super) fn ok_json<T: serde::Serialize>(request_id: u64, value: &T) -> Dispatched {
    match serde_json::to_value(value) {
        Ok(json) => ok(request_id, json, false),
        Err(e) => err(request_id, format!("failed to encode result: {e}"), false),
    }
}
