//! `gitpoulpi tag` handlers.

use gitpoulpi_core::domain::{
    short, OperationKind, PushOutcome, Ref, RefKind, TagSelection, TagWhere,
};
use gitpoulpi_core::TypedError;
use serde_json::json;

use crate::cli::TagAction;
use crate::client::Session;

use super::*;

pub(super) async fn tag(session: &mut Session, action: TagAction) -> Result<Emitted, String> {
    match action {
        TagAction::Create { name, at, message } => {
            let at = resolve(session, at.as_deref().unwrap_or("HEAD")).await?;
            session
                .call::<()>(
                    OperationKind::TagCreate,
                    json!({ "name": name, "at": at, "annotated_message": message }),
                )
                .await?;
            Ok(emitted(
                json!({ "tag": name, "at": at, "annotated": message.is_some() }),
                format!("tagged {} as {name}", short(at)),
            ))
        }
        TagAction::Annotate { name, message } => {
            session
                .call::<()>(
                    OperationKind::TagAnnotate,
                    json!({ "name": name, "message": message }),
                )
                .await?;
            Ok(emitted(
                json!({ "annotated": name }),
                format!("annotated tag {name}"),
            ))
        }
        TagAction::TakeRemote { name, remote } => {
            let remote = differing_remote(session, &name, remote).await?;
            session
                .call::<()>(
                    OperationKind::TagTakeRemote,
                    json!({ "remote": remote, "name": name }),
                )
                .await?;
            Ok(emitted(
                json!({ "took": name, "remote": remote }),
                format!("tag {name} now is {remote}'s version (`undo` puts yours back)"),
            ))
        }
        TagAction::ForcePush { name, remote, yes } => {
            let remote = differing_remote(session, &name, remote).await?;
            let answer = session
                .call_typed::<()>(
                    OperationKind::TagForcePush,
                    json!({ "remote": remote, "name": name, "confirmed": yes }),
                )
                .await?;
            match answer {
                Ok(()) => Ok(emitted(
                    json!({ "force_pushed": name, "remote": remote }),
                    format!("force pushed tag {name} to {remote} (`undo` pushes the old one back)"),
                )),
                Err(e) => match e.kind {
                    Some(TypedError::NeedsConfirmation(prompt)) => Ok(Emitted {
                        json: json!({ "needs_confirmation": { "prompt": prompt } }),
                        human: format!(
                            "{prompt}
Nothing was changed; repeat with --yes to go ahead."
                        ),
                        exit_code: 3,
                    }),
                    _ => Err(e.message),
                },
            }
        }
        // Empty `names` means `AllUnpushed`, resolved in the backend at
        // push time -- not a list this command built by filtering
        // `refs()`, which would miss a tag created since the last
        // refresh and would turn one gesture into N pushes.
        TagAction::Push { names, remote } => {
            let selection = if names.is_empty() {
                TagSelection::AllUnpushed
            } else {
                TagSelection::Named(names)
            };
            let outcome = session
                .call::<PushOutcome>(
                    OperationKind::TagPush,
                    json!({ "remote": remote, "selection": selection }),
                )
                .await?;
            let human = if outcome.pushed_refs.is_empty() {
                format!("every tag is already on {remote}")
            } else {
                format!("pushed {} to {remote}", outcome.pushed_refs.join(", "))
            };
            Ok(emitted(to_value(&outcome)?, human))
        }
        TagAction::DeleteRemote { name, remote } => {
            session
                .call::<()>(
                    OperationKind::TagDeleteRemote,
                    json!({ "remote": remote, "name": name }),
                )
                .await?;
            Ok(emitted(
                json!({ "deleted": name, "remote": remote }),
                format!("deleted tag {name} from {remote}"),
            ))
        }
        TagAction::Delete {
            name,
            local: _,
            remote,
            both,
        } => delete(session, name, remote, both).await,
        TagAction::Fetch { name, remote } => {
            let refs = all_refs(session).await?;
            let on = remotes_with(&refs, &name);
            let remote = match remote {
                Some(r) => r,
                None => match on.as_slice() {
                    [only] => only.clone(),
                    [] => return Err(format!("no remote is known to have a tag named `{name}`")),
                    many => {
                        return Err(format!(
                            "tag `{name}` is on {}: pick one with --remote",
                            many.join(", ")
                        ))
                    }
                },
            };
            session
                .call::<()>(
                    OperationKind::TagFetch,
                    json!({ "remote": remote, "name": name }),
                )
                .await?;
            Ok(emitted(
                json!({ "fetched": name, "remote": remote }),
                format!("fetched tag {name} from {remote}"),
            ))
        }
        TagAction::List { unpushed } => {
            let refs = all_refs(session).await?;
            let tags: Vec<&Ref> = refs
                .iter()
                .filter(|r| matches!(r.kind, RefKind::Tag { .. }))
                .filter(|r| !unpushed || r.is_unpushed_tag())
                .collect();
            // Each row is the `Ref` plus the location spelled out, so a
            // script reads `where` instead of re-deriving it from
            // `tag_sync`: `local` (bool: the tag exists here), `remote`
            // (every remote that has it), `differs_on` and `where`.
            let mut rows = Vec::new();
            for r in &tags {
                let mut row = to_value(r)?;
                if let (Some(obj), Some(sync)) = (row.as_object_mut(), r.tag_sync.as_ref()) {
                    obj.insert("name".into(), json!(ref_name(&r.kind)));
                    obj.insert("local".into(), json!(sync.local));
                    obj.insert("remote".into(), json!(sync.on_remotes()));
                    obj.insert("differs_on".into(), json!(sync.differs_on));
                    obj.insert("where".into(), json!(sync.where_().word()));
                }
                rows.push(row);
            }
            // The marker the TUI draws, spelled out in one column: `+`
            // not on any remote, `>` remote only (not fetched), `=` both,
            // `!` differs.
            let human = tags
                .iter()
                .map(|r| {
                    format!(
                        "{} {} {}",
                        match r.tag_where() {
                            Some(TagWhere::Local) => "+",
                            Some(TagWhere::Remote) => ">",
                            Some(TagWhere::Differs) => "!",
                            _ => "=",
                        },
                        ref_name(&r.kind),
                        short(r.target)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(serde_json::Value::Array(rows), human))
        }
    }
}

/// The remote a `differs` tag is acted on: the one given, else the only
/// remote holding it at another commit.
async fn differing_remote(
    session: &mut Session,
    name: &str,
    remote: Option<String>,
) -> Result<String, String> {
    if let Some(remote) = remote {
        return Ok(remote);
    }
    let refs = all_refs(session).await?;
    let found = refs.iter().find(|r| r.tag_name() == Some(name));
    let Some(found) = found else {
        return Err(format!("no tag named `{name}`"));
    };
    let on = found
        .tag_sync
        .as_ref()
        .map(|s| s.differs_on.clone())
        .unwrap_or_default();
    match on.as_slice() {
        [only] => Ok(only.clone()),
        [] => Err(format!("tag `{name}` does not differ from any remote")),
        many => Err(format!(
            "tag `{name}` differs on {}: pick one with --remote",
            many.join(", ")
        )),
    }
}

/// The remotes known (from the mirror) to have tag `name`.
fn remotes_with(refs: &[Ref], name: &str) -> Vec<String> {
    refs.iter()
        .find(|r| r.tag_name() == Some(name))
        .and_then(|r| r.tag_sync.as_ref())
        .map(|s| s.on_remotes())
        .unwrap_or_default()
}

/// `tag delete`: local (default), `--remote R`, or `--both`.
///
/// The remote goes **first** in a combined delete: if the push is refused
/// (no rights, offline) nothing local was lost; the reverse would leave a
/// tag that is still on the remote and gone here, which a fetch brings
/// straight back. It is one request, so one undo entry: undo puts the tag
/// back on the remote (the very same object, never forced) and here, redo
/// deletes both again.
async fn delete(
    session: &mut Session,
    name: String,
    remote: Option<String>,
    both: bool,
) -> Result<Emitted, String> {
    let refs = all_refs(session).await?;
    let found = refs.iter().find(|r| r.tag_name() == Some(name.as_str()));
    let Some(found) = found else {
        return Err(format!("no tag named `{name}`"));
    };
    let is_local = !found.is_remote_only_tag();
    let on = remotes_with(&refs, &name);
    let target_remote = match (&remote, both) {
        (Some(r), _) => Some(r.clone()),
        (None, true) => match on.as_slice() {
            [only] => Some(only.clone()),
            [] => None,
            many => {
                return Err(format!(
                    "tag `{name}` is on {}: pick one with --remote",
                    many.join(", ")
                ))
            }
        },
        (None, false) => None,
    };
    if both && target_remote.is_none() {
        return Err(format!(
            "no remote is known to have `{name}`: use --local to delete it here"
        ));
    }
    let remote_only_request = remote.is_some() && !both;
    if !remote_only_request && !is_local {
        return Err(format!(
            "`{name}` is only on {}: there is no local tag to delete (use --remote to delete it there)",
            on.join(", ")
        ));
    }
    if let Some(r) = target_remote.as_deref() {
        // `both`: the daemon deletes the local tag in the same request, once
        // the remote one is gone.
        session
            .call::<()>(
                OperationKind::TagDeleteRemote,
                json!({ "remote": r, "name": name, "and_local": !remote_only_request }),
            )
            .await?;
    } else if !remote_only_request {
        session
            .call::<()>(OperationKind::TagDelete, json!({ "name": name }))
            .await?;
    }
    Ok(match (remote_only_request, target_remote) {
        (true, Some(r)) => emitted(
            json!({ "deleted": name, "remote": r, "scope": "remote" }),
            format!(
                "deleted tag {name} on {r} (the local tag stays; `undo` pushes it back on {r})"
            ),
        ),
        (false, Some(r)) => emitted(
            json!({ "deleted": name, "remote": r, "scope": "both" }),
            format!("deleted tag {name} here and on {r} (`undo` restores it in both places)"),
        ),
        _ => {
            let still: Vec<String> = on;
            let human = if still.is_empty() {
                format!("deleted tag {name}")
            } else {
                format!(
                    "deleted tag {name} here; it is still on {}: the next fetch brings it back unless you also delete it there (--remote or --both)",
                    still.join(", ")
                )
            };
            emitted(
                json!({ "deleted": name, "scope": "local", "still_on": still }),
                human,
            )
        }
    })
}
