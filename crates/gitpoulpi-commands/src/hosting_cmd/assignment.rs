//! `gitpoulpi hosting assignment`: shows, blocks or allows assigning people
//! in the current repository's issues and pull requests.
//!
//! Blocking is always allowed; allowing again is a person's decision (the MCP
//! server cannot), because the block is what keeps a bot-driven workflow from
//! naming a maintainer.

use serde_json::json;

use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::*;

pub(super) fn run_assignment(ipc: &Session, block: bool, allow: bool) -> Result<Emitted, String> {
    if allow && !ipc.may_trust() {
        return Err(TRUST_REFUSAL.to_string());
    }
    let root = ipc
        .repo_root()
        .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())?
        .to_path_buf();
    let path = crate::commands::repo_state_path(&root);
    let mut state = gitpoulpi_session::RepoState::load(&path).map_err(|e| e.to_string())?;
    if (block || allow) && state.assignment_blocked != block {
        state.assignment_blocked = block;
        state.save(&path).map_err(|e| e.to_string())?;
    }
    let human = if state.assignment_blocked {
        "assignment is blocked in this repository (existing assignees stay visible)".to_string()
    } else {
        "assignment is allowed in this repository".to_string()
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "assignment": if state.assignment_blocked { "blocked" } else { "allowed" } }),
        human,
    })
}

/// A hosting error that refused an assignment, as the CLI answers it: exit
/// code 2 and a `refused` object with the reason `assignment_blocked`.
pub(crate) fn assignment_refusal(message: &str) -> Emitted {
    json_refusal(message)
}

fn json_refusal(message: &str) -> Emitted {
    Emitted {
        exit_code: 2,
        json: json!({
            "refused": {
                "reason": "assignment_blocked",
                "message": message,
                "hint": "this repository blocks assignment: `gitpoulpi hosting assignment --allow` (a person) lifts it",
            }
        }),
        human: format!("refused: {message}"),
    }
}
