//! `gitpoulpi stash` handlers.

use gitpoulpi_core::domain::{OperationKind, Stash, StashId};
use serde_json::json;

use crate::cli::StashAction;
use crate::client::Session;

use super::*;

pub(super) async fn stash(session: &mut Session, action: StashAction) -> Result<Emitted, String> {
    match action {
        StashAction::Push {
            message,
            ai,
            send_all,
        } => {
            if send_all && !ai {
                return Err("--send-all goes with --ai: it sends the whole diff to the AI".into());
            }
            let mut truncated = Vec::new();
            let message = if ai {
                let (message, cuts) =
                    crate::ai_cmd::generate_stash_message(session, send_all).await?;
                truncated = cuts;
                Some(message)
            } else {
                message
            };
            let id: Option<StashId> = session
                .call(
                    OperationKind::StashPush,
                    json!({ "message": message, "include_untracked": false }),
                )
                .await?;
            let (mut out, mut human) = match id {
                Some(id) => (
                    json!({ "stash": id }),
                    format!("stashed as stash@{{{}}}", id.0),
                ),
                None => (
                    json!({ "stash": null }),
                    "No local changes to save".to_string(),
                ),
            };
            crate::ai_cmd::report_truncated(&mut out, &mut human, &truncated);
            Ok(emitted(out, human))
        }
        StashAction::Pop { id } => {
            let id = stash_id(id)?;
            session
                .call::<()>(OperationKind::StashPop, json!({ "id": id }))
                .await?;
            Ok(emitted(
                json!({ "popped": id }),
                format!("popped stash@{{{}}}", id.0),
            ))
        }
        StashAction::Apply { id } => {
            let id = stash_id(id)?;
            session
                .call::<()>(OperationKind::StashApply, json!({ "id": id }))
                .await?;
            Ok(emitted(
                json!({ "applied": id }),
                format!("applied stash@{{{}}}", id.0),
            ))
        }
        StashAction::Drop { id } => {
            let id = stash_id(id)?;
            session
                .call::<()>(OperationKind::StashDrop, json!({ "id": id }))
                .await?;
            Ok(emitted(
                json!({ "dropped": id }),
                format!("dropped stash@{{{}}}", id.0),
            ))
        }
        StashAction::FromCommit { commit, message } => {
            let oid = resolve(session, &commit).await?;
            let stash: Oid = session
                .call(
                    OperationKind::StashFromCommit,
                    json!({ "commit": oid, "message": message }),
                )
                .await?;
            Ok(emitted(
                json!({ "stash": StashId(0), "commit": stash, "from": oid }),
                format!("saved the changes of {} as stash@{{0}}", short(oid)),
            ))
        }
        StashAction::List => {
            let stashes: Vec<Stash> = session.call(OperationKind::Stashes, json!({})).await?;
            let human = stashes
                .iter()
                .map(|s| format!("stash@{{{}}} {}", s.id.0, s.message))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&stashes)?, human))
        }
    }
}

/// Accepts both spellings a user is likely to paste: `2` and `stash@{2}`.
pub(super) fn stash_id(raw: Option<String>) -> Result<StashId, String> {
    let Some(raw) = raw else {
        return Ok(StashId(0));
    };
    let trimmed = raw
        .trim()
        .trim_start_matches("stash@{")
        .trim_end_matches('}');
    trimmed
        .parse::<u32>()
        .map(StashId)
        .map_err(|_| format!("`{raw}` is not a stash id"))
}
