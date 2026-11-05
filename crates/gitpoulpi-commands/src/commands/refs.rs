//! `gitpoulpi refs` handlers.

use gitpoulpi_core::domain::{OperationKind, Ref, RefKind};
use serde_json::{json, Value};

use crate::cli::RefsAction;
use crate::client::Session;

use super::*;

/// `refs hide|show|solo`: the per-repo display filter, changed through the
/// daemon so it can tell every other frontend (`Event::RefVisibilityChanged`).
pub(super) async fn refs(session: &mut Session, action: RefsAction) -> Result<Emitted, String> {
    let (mode, names, remotes, clear) = match action {
        RefsAction::Hide {
            names,
            remote,
            clear,
        } => ("hide", names, remote, clear),
        RefsAction::Show {
            names,
            remote,
            clear,
        } => ("show", names, remote, clear),
        RefsAction::Solo { names, clear } => ("solo", names, Vec::new(), clear),
    };
    // Hiding or soloing something that does not exist would be remembered
    // forever and do nothing; showing it again must stay possible (a branch
    // can be deleted while hidden), so only the first two are checked.
    if mode != "show" && !names.is_empty() {
        let known = all_refs(session).await?;
        for name in &names {
            if !known.iter().any(|r| ref_name(&r.kind) == *name) {
                return Err(format!("no branch, remote branch or tag named `{name}`"));
            }
        }
    }
    // Same reasoning for remotes: hiding one that does not exist would be
    // remembered forever (showing one, e.g. after `remote remove`, stays
    // possible).
    if mode == "hide" && !remotes.is_empty() {
        let known: Vec<gitpoulpi_core::domain::Remote> =
            session.call(OperationKind::Remotes, json!({})).await?;
        for name in &remotes {
            if !known.iter().any(|r| r.name == *name) {
                return Err(format!("no such remote: {name}"));
            }
        }
    }
    let mut state: Value = Value::Null;
    // Names and `--clear` first, so `hide --clear --remote x` ends with only
    // `x` hidden.
    if !names.is_empty() || clear || remotes.is_empty() {
        state = session
            .call(
                OperationKind::SetRefVisibility,
                json!({ "mode": mode, "names": names, "clear": clear }),
            )
            .await?;
    }
    if !remotes.is_empty() {
        state = session
            .call(
                OperationKind::SetRefVisibility,
                json!({ "mode": format!("{mode}-remote"), "names": remotes, "clear": false }),
            )
            .await?;
    }
    let count = |key: &str| state[key].as_array().map_or(0, Vec::len);
    let human = format!(
        "{} hidden, {} soloed, {} remotes hidden",
        count("hidden"),
        count("solo"),
        count("hidden_remotes")
    );
    Ok(emitted(
        json!({
            "mode": mode,
            "names": names,
            "remotes": remotes,
            "clear": clear,
            "hidden": state["hidden"],
            "solo": state["solo"],
            "hidden_remotes": state["hidden_remotes"],
        }),
        human,
    ))
}

pub(super) async fn all_refs(session: &mut Session) -> Result<Vec<Ref>, String> {
    session.call(OperationKind::Refs, json!({})).await
}

/// `merge`/`rebase` take a whole `Ref`, not a name, because the backends
/// need its target oid to report a fast-forward. Looking it up in
/// `refs()` keeps the CLI from having to invent one.
pub(super) async fn find_ref(session: &mut Session, name: &str) -> Result<Ref, String> {
    let refs = all_refs(session).await?;
    refs.into_iter()
        .find(|r| ref_name(&r.kind) == name)
        .ok_or_else(|| format!("no branch, remote branch or tag named `{name}`"))
}

pub(super) fn ref_name(kind: &RefKind) -> String {
    match kind {
        RefKind::LocalBranch { name } => name.clone(),
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
        RefKind::Tag { name } => name.clone(),
    }
}
