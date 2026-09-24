//! `gitpoulpi submodule` handlers.

use gitpoulpi_core::domain::{OperationKind, Submodule, SubmoduleState, SubmoduleUpdateOptions};
use serde_json::json;

use crate::cli::SubmoduleAction;
use crate::client::Session;

use super::*;

fn state_word(state: SubmoduleState) -> &'static str {
    match state {
        SubmoduleState::Uninitialized => "uninitialized",
        SubmoduleState::UpToDate => "up to date",
        SubmoduleState::Modified => "modified",
        SubmoduleState::Conflicted => "conflicted",
    }
}

pub(super) async fn submodule(
    session: &mut Session,
    action: SubmoduleAction,
) -> Result<Emitted, String> {
    match action {
        SubmoduleAction::List => {
            let list: Vec<Submodule> = session.call(OperationKind::Submodules, json!({})).await?;
            let human = if list.is_empty() {
                "no submodules".to_string()
            } else {
                list.iter()
                    .map(|s| format!("{}  {}  {}", s.path, state_word(s.state), s.url))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(to_value(&list)?, human))
        }
        SubmoduleAction::Init { paths } => {
            session
                .call::<()>(OperationKind::SubmoduleInit, json!({ "paths": paths }))
                .await?;
            let list: Vec<Submodule> = session.call(OperationKind::Submodules, json!({})).await?;
            Ok(emitted(
                to_value(&list)?,
                "submodules registered (use `submodule update --init` to clone them)".to_string(),
            ))
        }
        SubmoduleAction::Update {
            paths,
            init,
            recursive,
        } => {
            let opts = SubmoduleUpdateOptions {
                paths,
                init,
                recursive,
            };
            session
                .call::<()>(OperationKind::SubmoduleUpdate, json!({ "opts": opts }))
                .await?;
            let list: Vec<Submodule> = session.call(OperationKind::Submodules, json!({})).await?;
            Ok(emitted(to_value(&list)?, "submodules updated".to_string()))
        }
    }
}
