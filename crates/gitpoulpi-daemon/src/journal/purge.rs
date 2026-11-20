//! Purging the old history of history rewrites (the history-rewrite design,
//! "Purge (S15)"): the journal side. The backend deletes pins, reflogs and
//! objects ([`GitBackend::rewrite_purge`]); this module picks the journal
//! entries, builds the confirmation, and marks the entries `Purged` so undo
//! refuses them with the typed reason.
//!
//! Order matters: the entries are marked `Purged` (and saved) **before** the
//! backend deletes anything, so a crash half way never leaves an entry that
//! says it can be undone while its pins are gone. If the real run finds a
//! blocker the dry run did not (a ref moved in between), the journal is put
//! back as it was.

use gitpoulpi_core::domain::rewrite::{
    confirmation_text, HostChecklist, PurgeBlockerKind, PurgeCheck, PurgeEntry, PurgeEntryInfo,
    PurgeOutcome, PurgeRequest, PurgeStatus, RewriteRefusal,
};
use gitpoulpi_core::domain::{EntryState, OperationKind, RepoRef, UNDO_PIN_PREFIX};
use gitpoulpi_core::journal::{Journal, JournalEntry};
use gitpoulpi_core::{Error, GitBackend, Result};
use gitpoulpi_ipc::rewrite::PurgeArgs;
use gitpoulpi_ipc::Request;

use super::state::ok_json;
use super::store::{journal_path, load, save};
use crate::dispatch::{err, err_kind, repo_ref, Dispatched};

fn refused(detail: impl Into<String>) -> Error {
    Error::RewriteRefused(RewriteRefusal::Stale, detail.into())
}

fn failure(request_id: u64, e: Error) -> Dispatched {
    match e.typed() {
        Some(kind) => err_kind(request_id, e.to_string(), false, Some(kind)),
        None => err(request_id, e.to_string(), false),
    }
}

/// The journal entries a purge request names, and the chosen ones that were
/// purged already.
struct Chosen {
    entries: Vec<JournalEntry>,
    /// Other operations whose undo pins keep the old history alive: their
    /// pins go and their undo goes with them.
    extra: Vec<JournalEntry>,
    already_purged: Vec<u64>,
}

impl Chosen {
    fn request(&self, dry_run: bool) -> PurgeRequest {
        PurgeRequest {
            entries: self
                .entries
                .iter()
                .map(to_purge_entry)
                .chain(self.extra.iter().map(|e| PurgeEntry {
                    // Their refs are history the rewrite entries already
                    // name; only their pins matter here.
                    refs: Vec::new(),
                    ..to_purge_entry(e)
                }))
                .collect(),
            dry_run,
        }
    }

    fn infos(&self) -> Vec<PurgeEntryInfo> {
        let mut all: Vec<PurgeEntryInfo> = self
            .entries
            .iter()
            .map(|e| PurgeEntryInfo {
                id: e.id,
                label: e.label.clone(),
                rewrite: true,
            })
            .chain(self.extra.iter().map(|e| PurgeEntryInfo {
                id: e.id,
                label: e.label.clone(),
                rewrite: false,
            }))
            .collect();
        all.sort_by_key(|e| e.id);
        all
    }

    fn ids(&self) -> Vec<u64> {
        self.entries
            .iter()
            .chain(&self.extra)
            .map(|e| e.id)
            .collect()
    }
}

fn choose(journal: &Journal, args: &PurgeArgs) -> Result<Chosen> {
    let rewrites: Vec<&JournalEntry> = journal
        .all_entries()
        .filter(|e| e.kind == OperationKind::RewriteApply)
        .collect();
    let mut wanted: Vec<u64> = args.entries.clone();
    if args.all {
        wanted = rewrites.iter().map(|e| e.id).collect();
    } else if wanted.is_empty() {
        // The newest rewrite that still has something to purge.
        let latest = rewrites
            .iter()
            .filter(|e| e.state == EntryState::Done)
            .map(|e| e.id)
            .max()
            .ok_or_else(|| refused("there is no history rewrite in the undo history to purge"))?;
        wanted.push(latest);
    }
    let mut entries = Vec::new();
    let mut already = Vec::new();
    for id in wanted {
        let entry = rewrites.iter().find(|e| e.id == id).ok_or_else(|| {
            refused(format!(
                "operation #{id} is not a history rewrite in the undo history"
            ))
        })?;
        match entry.state {
            EntryState::Purged => already.push(id),
            EntryState::Done => {
                if !entries.iter().any(|e: &JournalEntry| e.id == id) {
                    entries.push((*entry).clone());
                }
            }
            _ => {
                return Err(refused(format!(
                    "operation #{id} never completed: there is nothing to purge for it"
                )))
            }
        }
    }
    Ok(Chosen {
        entries,
        extra: Vec::new(),
        already_purged: already,
    })
}

/// The entry an undo pin belongs to, for this journal (`None` for another
/// worktree's pin).
fn pin_owner(journal: &Journal, pin: &str) -> Option<u64> {
    let rest = pin.strip_prefix(UNDO_PIN_PREFIX)?;
    if let Some(rest) = rest.strip_prefix(journal.pin_ns.as_str()) {
        return rest.split('/').next()?.parse().ok();
    }
    // A pin adopted from the legacy journal (`store::adopt_legacy`) keeps its
    // legacy name: the entry that lists it owns it.
    journal
        .all_entries()
        .find(|e| e.pins.iter().any(|p| p.name == pin))
        .map(|e| e.id)
}

/// Dry-runs the purge; every undo pin of another entry that holds the old
/// history joins the request (its undo cannot work without the objects), until
/// nothing new turns up. What is left in `blockers` is not the journal's to
/// fix.
fn plan(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
    chosen: &mut Chosen,
) -> Result<PurgeOutcome> {
    loop {
        let outcome = backend.rewrite_purge(repo, &chosen.request(true), &mut |_| {})?;
        let before = chosen.extra.len();
        for blocker in outcome
            .blockers
            .iter()
            .filter(|b| b.kind == PurgeBlockerKind::UndoPin)
        {
            let Some(id) = pin_owner(journal, &blocker.name) else {
                continue;
            };
            let known = chosen.ids().contains(&id);
            let entry = journal
                .all_entries()
                .find(|e| e.id == id && e.state == EntryState::Done);
            if let (false, Some(entry)) = (known, entry) {
                chosen.extra.push(entry.clone());
            }
        }
        if chosen.extra.len() == before {
            return Ok(outcome);
        }
    }
}

fn to_purge_entry(entry: &JournalEntry) -> PurgeEntry {
    PurgeEntry {
        id: entry.id,
        label: entry.label.clone(),
        refs: entry.refs.clone(),
        pins: entry.pins.iter().map(|p| p.name.clone()).collect(),
    }
}

fn hosts(backend: &dyn GitBackend, repo: &RepoRef) -> Vec<HostChecklist> {
    let mut remotes = backend.remotes(repo).unwrap_or_default();
    if remotes.is_empty() {
        return Vec::new();
    }
    remotes.sort_by(|a, b| a.name.cmp(&b.name));
    remotes
        .iter()
        .map(|r| HostChecklist::new(&r.name, r.push_url.as_deref().unwrap_or(&r.fetch_url)))
        .collect()
}

fn args_of(request: &Request) -> std::result::Result<PurgeArgs, Box<Dispatched>> {
    serde_json::from_value(request.args.clone())
        .map_err(|e| Box::new(err(request.id, format!("invalid arguments: {e}"), false)))
}

fn journal_of(request: &Request, repo: &RepoRef) -> std::result::Result<Journal, Box<Dispatched>> {
    if !crate::undo_settings::enabled() {
        return Err(Box::new(failure(
            request.id,
            refused("undo is turned off, so there is no list of rewrites to purge from"),
        )));
    }
    let Some(path) = journal_path(repo) else {
        return Err(Box::new(failure(
            request.id,
            refused("this is not a repository"),
        )));
    };
    load(&path).map_err(|m| Box::new(err(request.id, m, false)))
}

/// `RewritePurgeCheck`: what a purge would do and lose, what blocks it, and
/// the host-side steps. Changes nothing.
pub fn purge_check(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    let (repo, args) = match (repo_ref(request), args_of(request)) {
        (Ok(repo), Ok(args)) => (repo, args),
        (Err(m), _) => return err(request.id, m, false),
        (_, Err(d)) => return *d,
    };
    let journal = match journal_of(request, &repo) {
        Ok(j) => j,
        Err(d) => return *d,
    };
    let mut chosen = match choose(&journal, &args) {
        Ok(c) => c,
        Err(e) => return failure(request.id, e),
    };
    let outcome = match plan(backend, &repo, &journal, &mut chosen) {
        Ok(o) => o,
        Err(e) => return failure(request.id, e),
    };
    let entries = chosen.infos();
    let prompt = confirmation_text(&entries, &outcome);
    ok_json(
        request.id,
        &PurgeCheck {
            entries,
            already_purged: chosen.already_purged,
            outcome,
            hosts: hosts(backend, &repo),
            prompt,
        },
    )
}

/// `RewritePurge`, under the exclusive repo lock and as a job that cannot be
/// stopped once it runs. Answers with the `PurgeOutcome`; `Blocked` means
/// nothing was changed.
pub fn purge_run(backend: &dyn GitBackend, request: &Request) -> Dispatched {
    let (repo, args) = match (repo_ref(request), args_of(request)) {
        (Ok(repo), Ok(args)) => (repo, args),
        (Err(m), _) => return err(request.id, m, false),
        (_, Err(d)) => return *d,
    };
    let journal = match journal_of(request, &repo) {
        Ok(j) => j,
        Err(d) => return *d,
    };
    let Some(path) = journal_path(&repo) else {
        return failure(request.id, refused("this is not a repository"));
    };
    let mut chosen = match choose(&journal, &args) {
        Ok(c) => c,
        Err(e) => return failure(request.id, e),
    };

    // The check first, so a blocker is reported before the journal changes.
    match plan(backend, &repo, &journal, &mut chosen) {
        Ok(o) if o.status == PurgeStatus::Blocked => return ok_json(request.id, &o),
        Ok(_) => {}
        Err(e) => return failure(request.id, e),
    }
    let purge = chosen.request(false);

    // Mark first, then delete (see the module comment).
    let before = journal.clone();
    let mut marked = journal;
    marked.mark_purged(&chosen.ids());
    if let Err(e) = save(&path, &marked) {
        return err(
            request.id,
            format!("cannot write the undo journal: {e}"),
            false,
        );
    }

    let outcome = backend.rewrite_purge(&repo, &purge, &mut |step| {
        crate::rewrite_runs::purge_step(step);
    });
    match outcome {
        Ok(o) if o.status == PurgeStatus::Blocked => {
            // Something moved between the check and the run: nothing was
            // deleted, so the entries are as undoable as before.
            let _ = save(&path, &before);
            ok_json(request.id, &o)
        }
        Ok(o) => {
            let mut done = ok_json(request.id, &o);
            done.mutated = true;
            done
        }
        Err(e) => failure(request.id, e),
    }
}
