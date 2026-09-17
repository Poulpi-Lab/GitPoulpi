//! `gitpoulpi mailmap list|check|add` (slice S16): the identities of a
//! scope, how the `.mailmap` shows some, and "Merge into..." from a script.

use gitpoulpi_core::domain::rewrite::{Ident, IdentityList, MailmapAdded, Scope, ScopeOptions};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_ipc::mailmap::{AddArgs, IdentitiesArgs, ResolveArgs};
use serde_json::json;

use crate::cli::MailmapAction;
use crate::client::Session;

use super::*;

/// `Name <email>` or `<email>`.
fn identity(text: &str) -> Result<Ident, String> {
    let bad = || format!("{text:?} is not `Name <email>`");
    let open = text.find('<').ok_or_else(bad)?;
    let close = text.rfind('>').filter(|c| *c > open).ok_or_else(bad)?;
    if !text[close + 1..].trim().is_empty() {
        return Err(bad());
    }
    let email = text[open + 1..close].trim();
    if email.is_empty() {
        return Err(bad());
    }
    Ok(Ident::new(text[..open].trim(), email))
}

fn show(i: &Ident) -> String {
    if i.name.is_empty() {
        format!("<{}>", i.email)
    } else {
        format!("{} <{}>", i.name, i.email)
    }
}

pub(super) async fn mailmap(
    session: &mut Session,
    action: MailmapAction,
) -> Result<Emitted, String> {
    match action {
        MailmapAction::List {
            all_local,
            refs,
            mapped,
            unmapped,
        } => {
            let scope = if all_local {
                Scope::AllLocal
            } else if !refs.is_empty() {
                Scope::Refs { refs }
            } else {
                Scope::CurrentBranch { since: None }
            };
            let mut list: IdentityList = session
                .call(
                    OperationKind::MailmapIdentities,
                    to_value(&IdentitiesArgs {
                        scope,
                        options: ScopeOptions::default(),
                    })?,
                )
                .await?;
            if mapped {
                list.identities.retain(|i| i.mapped_to.is_some());
            } else if unmapped {
                list.identities.retain(|i| i.mapped_to.is_none());
            }
            let mut human = format!(
                "{} identities in {} commits{}",
                list.identities.len(),
                list.commits,
                if list.has_mailmap {
                    ""
                } else {
                    " (no .mailmap)"
                }
            );
            for i in &list.identities {
                human.push_str(&format!(
                    "\n{:>6} author {:>6} committer  {}{}",
                    i.as_author,
                    i.as_committer,
                    show(&i.ident()),
                    i.mapped_to
                        .as_ref()
                        .map(|t| format!(" -> {}", show(t)))
                        .unwrap_or_default()
                ));
            }
            Ok(emitted(to_value(&list)?, human))
        }
        MailmapAction::Check { identities } => {
            let idents: Vec<Ident> = identities
                .iter()
                .map(|t| identity(t))
                .collect::<Result<_, _>>()?;
            let resolved: Vec<Ident> = session
                .call(
                    OperationKind::MailmapResolve,
                    to_value(&ResolveArgs {
                        idents: idents.clone(),
                    })?,
                )
                .await?;
            let rows: Vec<serde_json::Value> = idents
                .iter()
                .zip(&resolved)
                .map(|(from, to)| json!({ "identity": from, "shown_as": to, "mapped": from != to }))
                .collect();
            let human = idents
                .iter()
                .zip(&resolved)
                .map(|(from, to)| format!("{} -> {}", show(from), show(to)))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(json!({ "identities": rows }), human))
        }
        MailmapAction::Add {
            canonical,
            from,
            file,
        } => {
            let canonical = identity(&canonical)?;
            let from: Vec<Ident> = from.iter().map(|t| identity(t)).collect::<Result<_, _>>()?;
            let done: MailmapAdded = session
                .call(
                    OperationKind::MailmapAdd,
                    to_value(&AddArgs {
                        canonical,
                        from,
                        file,
                    })?,
                )
                .await?;
            let human = format!(
                "{} line(s) added to {} (not committed)",
                done.added, done.file
            );
            Ok(emitted(json!({ "mailmap": done }), human))
        }
    }
}
