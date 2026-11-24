//! `gitpoulpi undo` handlers.

use gitpoulpi_core::domain::{
    Direction, EntryState, JournalListing, ListedEntry, OperationKind, UndoOutcome,
};
use gitpoulpi_core::TypedError;
use serde_json::json;

use crate::client::Session;

use super::*;

/// `undo` / `redo`: one request; a refusal is a normal answer (exit code
/// 2, the typed reason in `{"refused": ...}`), not a failure.
pub(super) async fn undo_or_redo(
    session: &mut Session,
    command: OperationKind,
    dry_run: bool,
    expect: Option<u64>,
    confirmed: bool,
) -> Result<Emitted, String> {
    run(session, command, false, dry_run, expect, confirmed).await
}

/// `undo-forget`: drop the blocked top entry of the journal.
pub(super) async fn forget(
    session: &mut Session,
    command: OperationKind,
    dry_run: bool,
    expect: Option<u64>,
    confirmed: bool,
) -> Result<Emitted, String> {
    run(session, command, true, dry_run, expect, confirmed).await
}

async fn run(
    session: &mut Session,
    command: OperationKind,
    forget: bool,
    dry_run: bool,
    expect: Option<u64>,
    confirmed: bool,
) -> Result<Emitted, String> {
    let answer = session
        .call_typed::<UndoOutcome>(
            command,
            json!({ "expect": expect, "dry_run": dry_run, "confirmed": confirmed, "forget": forget }),
        )
        .await?;
    match answer {
        Ok(outcome) => {
            if forget {
                let verb = if outcome.applied {
                    "forgot"
                } else {
                    "would forget"
                };
                return Ok(emitted(
                    to_value(&outcome)?,
                    format!("{verb} {}", outcome.entry.label),
                ));
            }
            let verb = match (outcome.direction, outcome.applied) {
                (Direction::Undo, true) => "undid",
                (Direction::Redo, true) => "redid",
                (Direction::Undo, false) => "would undo",
                (Direction::Redo, false) => "would redo",
            };
            let mut human = format!("{verb} {}", outcome.entry.label);
            for step in &outcome.steps {
                human.push_str(&format!("\n  - {step}"));
            }
            Ok(emitted(to_value(&outcome)?, human))
        }
        Err(e) => match e.kind {
            Some(TypedError::UndoRefused(reason)) => {
                let message = reason.message();
                let forgettable = reason.forgettable();
                let human = if forgettable && command == OperationKind::Undo {
                    format!(
                        "{message}\nTo reach the entries below it, drop this one from the \
                         journal with `gitpoulpi undo-forget` (the repository is not touched)."
                    )
                } else {
                    message.clone()
                };
                Ok(Emitted {
                    json: json!({ "refused": { "reason": reason, "message": message,
                                                "forgettable": forgettable } }),
                    human,
                    exit_code: 2,
                })
            }
            Some(TypedError::NeedsConfirmation(prompt)) => Ok(Emitted {
                json: json!({ "needs_confirmation": { "prompt": prompt } }),
                human: format!("{prompt}\nNothing was changed; repeat with --yes to go ahead."),
                exit_code: 3,
            }),
            _ => Err(e.message),
        },
    }
}

/// `undo --sequence`: the newest entries of one sequence run, undone newest
/// first with one ordinary `Undo` each (`expect` pins the entry, so another
/// client's operation in between is refused as stale, never undone). The
/// first refusal (exit 2) or entry that needs `--yes` (exit 3) stops it and
/// says which step; what was undone stays undone, the rest stays in the
/// journal. A run that also acted in another repository (a worktree) is
/// undone here only as far as this repository's journal goes.
pub(super) async fn undo_sequence(
    session: &mut Session,
    dry_run: bool,
    confirmed: bool,
) -> Result<Emitted, String> {
    let state: gitpoulpi_core::domain::JournalState =
        session.call(OperationKind::JournalState, json!({})).await?;
    let Some(group) = state.sequence.map(|g| *g) else {
        let message = "the newest journal entries are not one sequence run".to_string();
        return Ok(Emitted {
            json: json!({ "no_sequence": { "message": message } }),
            human: message,
            exit_code: 2,
        });
    };
    let sequence = json!({ "name": group.name, "run_id": group.run_id, "of": group.of });
    let scope = "Only this repository's journal is undone: steps done in another repository \
                 (a worktree) stay.";
    if dry_run {
        let mut human = format!(
            "would undo the sequence \"{}\" ({} entries, newest first):",
            group.name,
            group.entries.len()
        );
        for e in &group.entries {
            human.push_str(&format!("\n  - step {}/{}: {}", e.step, group.of, e.label));
        }
        human.push_str(&format!("\n{scope}"));
        return Ok(emitted(
            json!({ "sequence": sequence, "applied": false, "entries": group.entries, "note": scope }),
            human,
        ));
    }
    let mut undone: Vec<UndoOutcome> = Vec::new();
    for (at, member) in group.entries.iter().enumerate() {
        let left = &group.entries[at..];
        let answer = session
            .call_typed::<UndoOutcome>(
                OperationKind::Undo,
                json!({ "expect": member.id, "dry_run": false, "confirmed": confirmed }),
            )
            .await?;
        let e = match answer {
            Ok(outcome) => {
                undone.push(outcome);
                continue;
            }
            Err(e) => e,
        };
        let at_step = format!(
            "step {}/{} ({}) of \"{}\"",
            member.step, group.of, member.label, group.name
        );
        let (key, detail, exit_code) = match e.kind {
            Some(TypedError::UndoRefused(reason)) => {
                let message = reason.message();
                (
                    "refused",
                    json!({ "reason": reason, "message": message }),
                    2,
                )
            }
            Some(TypedError::NeedsConfirmation(prompt)) => {
                ("needs_confirmation", json!({ "prompt": prompt }), 3)
            }
            _ => ("error", json!({ "message": e.message }), 1),
        };
        let why = detail["message"]
            .as_str()
            .or_else(|| detail["prompt"].as_str())
            .unwrap_or_default()
            .to_string();
        let mut human = format!("stopped at {at_step}: {why}");
        if exit_code == 3 {
            human.push_str("\nNothing more was changed; repeat with --yes to go ahead.");
        }
        human.push_str(&format!(
            "\nundone so far: {}; left in the journal: {}",
            undone.len(),
            left.len()
        ));
        let mut stopped = json!({ "id": member.id, "step": member.step, "label": member.label });
        stopped[key] = detail;
        return Ok(Emitted {
            json: json!({
                "sequence": sequence, "applied": !undone.is_empty(), "undone": undone,
                "stopped": stopped, "left": left, "note": scope,
            }),
            human,
            exit_code,
        });
    }
    let mut human = format!(
        "undid the sequence \"{}\" ({} entries)",
        group.name,
        undone.len()
    );
    for outcome in &undone {
        human.push_str(&format!("\n  - {}", outcome.entry.label));
    }
    human.push_str(&format!("\n{scope}"));
    Ok(emitted(
        json!({ "sequence": sequence, "applied": true, "undone": undone, "note": scope }),
        human,
    ))
}

pub(super) async fn journal_listing(session: &mut Session, all: bool) -> Result<Emitted, String> {
    let listing: JournalListing = session
        .call(OperationKind::JournalList, json!({ "all": all }))
        .await?;
    let line = |e: &ListedEntry, mark: &str| {
        let state = match e.summary.state {
            EntryState::Done => String::new(),
            EntryState::Pending => " [pending]".to_string(),
            EntryState::Interrupted => " [interrupted]".to_string(),
            EntryState::Purged => " [purged]".to_string(),
        };
        let mut text = format!("{mark} #{} {}{state}", e.summary.id, e.summary.label);
        for change in &e.changes {
            text.push_str(&format!("\n      {change}"));
        }
        text
    };
    let mut lines: Vec<String> = Vec::new();
    lines.extend(listing.undo.iter().map(|e| line(e, "undo ")));
    lines.extend(listing.redo.iter().rev().map(|e| line(e, "redo ")));
    lines.extend(listing.retired.iter().map(|e| line(e, "kept ")));
    let human = if lines.is_empty() {
        "the journal is empty".to_string()
    } else {
        lines.join("\n")
    };
    Ok(emitted(to_value(&listing)?, human))
}

// ------------------------------------------------- the undo safety net

/// A typed refusal as the CLI answers it: exit code 2, the reason in JSON.
fn refused_answer(reason: gitpoulpi_core::domain::UndoRefusal) -> Emitted {
    let message = reason.message();
    Emitted {
        json: json!({ "refused": { "reason": reason, "message": message,
                                    "forgettable": reason.forgettable() } }),
        human: message,
        exit_code: 2,
    }
}

/// A restore outcome, or the refusal that stopped it.
async fn restore_call(
    session: &mut Session,
    command: OperationKind,
    args: serde_json::Value,
) -> Result<Emitted, String> {
    let answer = session
        .call_typed::<gitpoulpi_core::domain::RestoreOutcome>(command, args)
        .await?;
    match answer {
        Ok(outcome) => {
            let verb = if outcome.applied {
                "restored"
            } else {
                "would restore"
            };
            let what = outcome
                .entry
                .as_ref()
                .map(|e| format!(" {}", e.label))
                .unwrap_or_default();
            let mut human = format!("{verb}{what}");
            for step in &outcome.steps {
                human.push_str(&format!("\n  - {step}"));
            }
            if outcome.applied {
                human.push_str("\nUndo it with `gitpoulpi undo`.");
            }
            Ok(emitted(to_value(&outcome)?, human))
        }
        Err(e) => match e.kind {
            Some(TypedError::UndoRefused(reason)) => Ok(refused_answer(reason)),
            _ => Err(e.message),
        },
    }
}

/// `undo-restore <id>`: "restore only this".
pub(super) async fn restore(
    session: &mut Session,
    id: u64,
    name: Option<String>,
    dry_run: bool,
) -> Result<Emitted, String> {
    restore_call(
        session,
        OperationKind::JournalRestore,
        json!({ "id": id, "name": name, "dry_run": dry_run }),
    )
    .await
}

/// `reflog-restore <ref> <oid>`.
pub(super) async fn reflog_restore(
    session: &mut Session,
    reference: String,
    oid: gitpoulpi_core::domain::Oid,
    mode: Option<&str>,
    dry_run: bool,
) -> Result<Emitted, String> {
    restore_call(
        session,
        OperationKind::ReflogRestore,
        json!({ "ref": reference, "oid": oid, "mode": mode, "dry_run": dry_run }),
    )
    .await
}

/// `reflog [<ref>]`.
pub(super) async fn reflog(
    session: &mut Session,
    reference: Option<String>,
    skip: usize,
    limit: usize,
) -> Result<Emitted, String> {
    let page: gitpoulpi_core::domain::ReflogPage = session
        .call(
            OperationKind::Reflog,
            json!({ "ref": reference.unwrap_or_else(|| "HEAD".into()), "skip": skip, "limit": limit }),
        )
        .await?;
    let mut lines: Vec<String> = page
        .entries
        .iter()
        .enumerate()
        .map(|(n, e)| {
            format!(
                "{:>3}  {}  {}  {}",
                skip + n,
                gitpoulpi_core::domain::short(e.new),
                e.time,
                e.message
            )
        })
        .collect();
    if page.more {
        lines.push(format!(
            "... more: use --skip {}",
            skip + page.entries.len()
        ));
    }
    let human = if lines.is_empty() {
        format!("no reflog for {}", page.refname)
    } else {
        lines.join("\n")
    };
    Ok(emitted(to_value(&page)?, human))
}

/// `lost-commits`.
pub(super) async fn lost_commits(
    session: &mut Session,
    skip: usize,
    limit: usize,
    deep: bool,
    budget_ms: u64,
) -> Result<Emitted, String> {
    let page: gitpoulpi_core::domain::LostCommitsPage = session
        .call(
            OperationKind::LostCommits,
            json!({ "skip": skip, "limit": limit, "deep": deep, "budget_ms": budget_ms }),
        )
        .await?;
    let mut lines: Vec<String> = page
        .commits
        .iter()
        .map(|c| {
            format!(
                "{}  {:?}  {}  {}",
                gitpoulpi_core::domain::short(c.oid),
                c.source,
                c.author,
                c.summary
            )
        })
        .collect();
    if page.more {
        lines.push(format!(
            "... more: use --skip {}",
            skip + page.commits.len()
        ));
    }
    if page.partial {
        lines.push("(partial: the scan stopped at its time budget)".to_string());
    }
    let human = if lines.is_empty() {
        "no lost commits".to_string()
    } else {
        lines.join("\n")
    };
    Ok(emitted(to_value(&page)?, human))
}
