use gitpoulpi_core::domain::{Direction, RepoRef};
use gitpoulpi_core::journal::{Effect, Journal, JournalEntry};
use gitpoulpi_core::GitBackend;

use super::capture::{has_tracked_changes, read_ref};
use super::store::branch_section;

/// Why undoing or redoing the top entry is risky, in words, or `None`
/// when it is cheap to take back (every ref-only move: the redo is one
/// key away).
///
/// - the undone commits are already on the branch's upstream, so the
///   history being rewritten was published (undo only);
/// - the move touches the working tree or the index while local changes
///   are around: a history move (`Effect::Files`) keeps uncommitted
///   changes but stops, changing nothing, when one of them is in a file
pub(super) fn risk(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    journal: &Journal,
    d: Direction,
) -> Option<String> {
    let entry = journal.top(d)?;
    let mut reasons: Vec<String> = Vec::new();
    match entry.effect {
        // A hard reset's own entry owns the changes it will write back.
        Effect::Files if entry.files.is_none() && has_tracked_changes(backend, repo).ok()? => {
            reasons.push(format!(
                "You have uncommitted changes. {} keeps them, but stops if one of \
             them is in a file it rewrites.",
                capitalized(d.verb())
            ));
        }
        _ => {}
    }
    for change in &entry.remote_refs {
        let what = match change.name.strip_prefix("refs/tags/") {
            Some(tag) => format!("tag {tag}"),
            None => format!(
                "branch {}",
                change
                    .name
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&change.name)
            ),
        };
        if let (Some(old), Some(new)) = (change.old, change.new) {
            // A tag moved on the remote (a force push), not created or deleted.
            let (verb, to) = match d {
                Direction::Undo => ("moves", old),
                Direction::Redo => ("force pushes", new),
            };
            reasons.push(format!(
                "{} this {verb} {what} on {} to {} (the remote server, leased on what it \
                 holds now: refused if {} moved again).",
                capitalized(d.verb()),
                change.remote,
                gitpoulpi_core::domain::short(to),
                change.remote
            ));
            continue;
        }
        reasons.push(match (d, change.old) {
            (Direction::Undo, Some(old)) => format!(
                "Undoing this re-creates {what} on {} at {}: it is pushed to the remote \
                 server, where everyone sees it again (it is never forced, and refused \
                 if {} has something else under that name now).",
                change.remote,
                gitpoulpi_core::domain::short(old),
                change.remote
            ),
            _ => format!(
                "Redoing this deletes {what} on {} again (the remote server).",
                change.remote
            ),
        });
    }
    if d == Direction::Undo {
        reasons.extend(pushed_risk(backend, repo, entry));
    }
    (!reasons.is_empty()).then(|| reasons.join(" "))
}

pub(super) fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

pub(super) fn pushed_risk(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    entry: &JournalEntry,
) -> Option<String> {
    if !entry.kind.traits().pushed_risk {
        return None;
    }
    // A rewrite moves many refs: any branch of it that was pushed counts.
    if entry.kind == gitpoulpi_core::domain::OperationKind::RewriteApply {
        // A rewrite pushed back with `push_rewritten` left a record, whether
        // or not the branch has an upstream configured.
        if let Some(note) = rewrite_pushed_note(repo, entry) {
            return Some(note);
        }
        return entry
            .refs
            .iter()
            .find_map(|change| pushed_risk_of(backend, repo, entry, change));
    }
    pushed_risk_of(backend, repo, entry, entry.refs.first()?)
}

/// The undo warning of a rewrite whose refs were pushed (`pushed.json` in the
/// rewrite directory, written by the push): undoing it changes only this
/// repository, the remote keeps the new history.
fn rewrite_pushed_note(repo: &RepoRef, entry: &JournalEntry) -> Option<String> {
    let file = crate::journal::git_dir(&repo.root)?
        .join("gitpoulpi")
        .join("rewrite")
        .join(gitpoulpi_core::domain::rewrite::PUSHED_FILE);
    let pushed: Vec<gitpoulpi_core::domain::rewrite::PushedRef> =
        serde_json::from_slice(&std::fs::read(file).ok()?).ok()?;
    let hit = entry.refs.iter().find_map(|change| {
        let new = change.new?;
        pushed
            .iter()
            .find(|p| p.name == change.name && p.new == new)
    })?;
    Some(format!(
        "{} was pushed to {}. Undoing it only changes this repository: {} keeps \
         the rewritten history, so the old history would have to be force-pushed \
         again, and everyone who already fetched the new one has to recover.",
        entry.label, hit.remote, hit.remote
    ))
}

fn pushed_risk_of(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    entry: &JournalEntry,
    change: &gitpoulpi_core::journal::RefChange,
) -> Option<String> {
    let new = change.new?;
    // A flow that created the branch (a start, a pick-up of a branch a bot
    // pushed) is undone by deleting it: no history is rewritten, whatever
    // the remote has.
    if change.old.is_none() && entry.kind == gitpoulpi_core::domain::OperationKind::FlowRun {
        return None;
    }
    let branch = change.name.strip_prefix("refs/heads/")?;
    // Two cheap reads (the branch's config, its tracking ref) rather than
    // the whole ref list, and none at all for a branch with no upstream.
    let cfg = backend.config_section(repo, &branch_section(branch)).ok()?;
    let value = |key: &str| {
        cfg.iter()
            .find(|(k, _)| *k == format!("branch.{branch}.{key}"))
            .map(|(_, v)| v.clone())
    };
    let remote = value("remote").filter(|r| r != ".")?;
    let merge = value("merge")?;
    let upstream = format!("{remote}/{}", merge.strip_prefix("refs/heads/")?);
    let on_remote = read_ref(backend, repo, &format!("refs/remotes/{upstream}")).ok()? == Some(new);
    on_remote.then(|| {
        format!(
            "{} is already on {upstream}. Undoing it rewrites history you pushed: \
             you would have to force-push to publish the result.",
            entry.label
        )
    })
}
