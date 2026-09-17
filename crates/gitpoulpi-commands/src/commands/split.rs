//! `gitpoulpi split-source` and `gitpoulpi split-commit`.
//!
//! The plan JSON is `{"parts": [SplitPart]}` (a bare array is accepted too);
//! a piece is `path` plus an optional `hunk` plus optional `lines`, which the
//! core's `SplitPiece` serde mapping handles. Validation is the core's
//! `validate_split`, the same rule the daemon and the TUI run.

use gitpoulpi_core::domain::{
    short, unassigned, validate_split, OperationKind, SplitCommit, SplitOutcome, SplitPart,
    SplitPiece, SplitRefusal, SplitSource,
};
use gitpoulpi_core::TypedError;
use serde::Deserialize;
use serde_json::json;

use crate::client::Session;

use super::*;

#[derive(Deserialize)]
#[serde(untagged)]
enum PlanFile {
    Object { parts: Vec<SplitPart> },
    Bare(Vec<SplitPart>),
}

/// Reads a `--plan` argument: `-` is stdin, anything else a file path.
pub(crate) fn read_split_plan(plan: &str) -> Result<Vec<SplitPart>, String> {
    let raw = if plan == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read the plan from stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(plan).map_err(|e| format!("failed to read {plan}: {e}"))?
    };
    let parsed: PlanFile = serde_json::from_str(&raw)
        .map_err(|e| format!("plan is not a valid split plan (`{{\"parts\": [...]}}`): {e}"))?;
    Ok(match parsed {
        PlanFile::Object { parts } | PlanFile::Bare(parts) => parts,
    })
}

pub(super) async fn source(session: &mut Session, rev: &str) -> Result<Emitted, String> {
    let oid = resolve(session, rev).await?;
    let source: SplitSource = session
        .call(OperationKind::SplitSource, json!({ "commit": oid }))
        .await?;
    let hunks: usize = source.files.iter().map(|f| f.hunks.len()).sum();
    let human = format!(
        "{}: {} file(s), {hunks} hunk(s)",
        short(source.commit),
        source.files.len()
    );
    Ok(emitted(to_value(&source)?, human))
}

fn refused(refusal: SplitRefusal, unassigned: &[SplitPiece]) -> Emitted {
    let message = gitpoulpi_core::Error::SplitRefused(refusal).to_string();
    Emitted {
        json: json!({ "refused": { "reason": refusal, "message": message, "unassigned": unassigned } }),
        human: message,
        exit_code: 2,
    }
}

pub(super) async fn commit(
    session: &mut Session,
    rev: &str,
    plan: &str,
    rest_message: Option<String>,
    sign: bool,
    dry_run: bool,
) -> Result<Emitted, String> {
    let mut parts = read_split_plan(plan)?;
    let oid = resolve(session, rev).await?;
    if dry_run || rest_message.is_some() {
        let source: SplitSource = session
            .call(OperationKind::SplitSource, json!({ "commit": oid }))
            .await?;
        if let Some(message) = rest_message {
            let rest = unassigned(&source, &parts);
            if !rest.is_empty() {
                parts.push(SplitPart {
                    message,
                    pieces: rest,
                });
            }
        }
        if dry_run {
            let left = unassigned(&source, &parts);
            let verdict = validate_split(&source, &parts);
            let human = match verdict {
                Ok(()) => format!("plan is valid: {} commit(s)", parts.len()),
                Err(r) => gitpoulpi_core::Error::SplitRefused(r).to_string(),
            };
            return Ok(Emitted {
                json: json!({
                    "dry_run": true,
                    "valid": verdict.is_ok(),
                    "refusal": verdict.err(),
                    "parts": parts.len(),
                    "unassigned": left,
                }),
                human,
                exit_code: if verdict.is_ok() { 0 } else { 2 },
            });
        }
    }
    let req = SplitCommit {
        commit: oid,
        parts,
        sign,
    };
    let answer = session
        .call_typed::<SplitOutcome>(OperationKind::SplitCommit, json!({ "req": req }))
        .await?;
    match answer {
        Ok(outcome) => {
            let human = format!(
                "split {} into {} commits ({} descendant(s) replayed; was {}); `gitpoulpi undo` puts it back",
                short(oid),
                outcome.created.len(),
                outcome.replayed,
                short(outcome.previous_head),
            );
            Ok(emitted(to_value(&outcome)?, human))
        }
        Err(e) => match e.kind {
            Some(TypedError::SplitRefused(r)) => Ok(refused(r, &[])),
            _ => Err(e.message),
        },
    }
}
