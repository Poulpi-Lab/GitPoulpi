//! `.mailmap` (slice S16), the system-git side: resolving identities through
//! `git check-mailmap`, the Identities list of a scope (one `git log`), and
//! the journaled edit "Merge into..." makes. `Git2Backend` resolves with
//! libgit2's `Mailmap` and delegates the other two here; the contract suite
//! pins that both resolve alike.

use std::collections::HashMap;
use std::path::Path;

use gitpoulpi_core::domain::rewrite::{
    mailmap, Ident, IdentityCount, IdentityList, MailmapAdded, RewriteRefusal, Scope, ScopeOptions,
};
use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::{Error, Result};

use super::refs::{self, Active};
use super::walk::{self, read_git};
use crate::process::{failed_with, run_bytes_with_stdin, spawn_failed};
use crate::rule_files::{read_or_empty, write};
use crate::snapshot_ops::normalize;

/// `git check-mailmap --stdin`: each identity as the `.mailmap` shows it.
pub(crate) fn resolve(repo: &RepoRef, idents: &[Ident]) -> Result<Vec<Ident>> {
    if idents.is_empty() {
        return Ok(Vec::new());
    }
    let mut input = String::new();
    for i in idents {
        if i.name.is_empty() {
            input.push_str(&format!("<{}>\n", i.email));
        } else {
            input.push_str(&format!("{} <{}>\n", i.name, i.email));
        }
    }
    let out = run_bytes_with_stdin(&repo.root, &["check-mailmap", "--stdin"], input.as_bytes())?;
    let text = String::from_utf8_lossy(&out);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != idents.len() {
        return Err(Error::Backend(
            "git check-mailmap answered a different number of lines".into(),
        ));
    }
    Ok(lines
        .iter()
        .zip(idents)
        .map(|(line, old)| {
            let Some(open) = line.rfind('<') else {
                return old.clone();
            };
            let name = line[..open].trim();
            let email = line[open + 1..].trim_end().trim_end_matches('>');
            Ident::new(name, email)
        })
        .collect())
}

/// Every identity of the scope with its counts (see
/// `GitBackend::mailmap_identities`).
pub(crate) fn identities(
    repo: &RepoRef,
    scope: &Scope,
    options: &ScopeOptions,
) -> Result<IdentityList> {
    let root: &Path = &repo.root;
    let head = walk::head_of(root)?;
    let all = refs::list(root)?;
    let resolved = match refs::resolve(scope, options, &all, &head) {
        Ok(r) => r,
        // No commit yet: nothing to list.
        Err(Error::RewriteRefused(RewriteRefusal::EmptyScope, _)) => {
            return Ok(IdentityList {
                has_mailmap: root.join(".mailmap").is_file(),
                ..IdentityList::default()
            })
        }
        Err(e) => return Err(e),
    };
    let mut args: Vec<String> = vec!["log".into(), "--format=%an%x01%ae%x01%cn%x01%ce".into()];
    match &resolved.active {
        Active::Set(set) => {
            args.push("--no-walk=unsorted".into());
            args.extend(set.iter().map(|o| o.to_hex()));
        }
        Active::UpTo(tip) => {
            args.push(tip.to_hex());
            if !resolved.not.is_empty() {
                args.push("--not".into());
                args.extend(resolved.not.iter().cloned());
            }
        }
        Active::All => {
            args.extend(resolved.tips.iter().cloned());
            if !resolved.not.is_empty() {
                args.push("--not".into());
                args.extend(resolved.not.iter().cloned());
            }
        }
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = read_git(root, &arg_refs).output().map_err(spawn_failed)?;
    if !out.status.success() {
        return Err(failed_with(
            &arg_refs,
            out.status.code(),
            &String::from_utf8_lossy(&out.stderr),
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut counts: HashMap<Ident, (u64, u64)> = HashMap::new();
    let mut commits = 0;
    for line in text.lines() {
        let mut f = line.split('\u{1}');
        let (Some(an), Some(ae), Some(cn), Some(ce)) = (f.next(), f.next(), f.next(), f.next())
        else {
            continue;
        };
        commits += 1;
        counts.entry(Ident::new(an, ae)).or_default().0 += 1;
        counts.entry(Ident::new(cn, ce)).or_default().1 += 1;
    }
    let mut rows: Vec<(Ident, (u64, u64))> = counts.into_iter().collect();
    rows.sort_by(|a, b| {
        (b.1 .0 + b.1 .1)
            .cmp(&(a.1 .0 + a.1 .1))
            .then_with(|| a.0.name.cmp(&b.0.name))
            .then_with(|| a.0.email.cmp(&b.0.email))
    });
    let idents: Vec<Ident> = rows.iter().map(|(i, _)| i.clone()).collect();
    let mapped = resolve(repo, &idents)?;
    let identities = rows
        .into_iter()
        .zip(mapped)
        .map(|((ident, (as_author, as_committer)), to)| IdentityCount {
            mapped_to: (to != ident).then_some(to),
            name: ident.name,
            email: ident.email,
            as_author,
            as_committer,
        })
        .collect();
    Ok(IdentityList {
        identities,
        commits,
        has_mailmap: root.join(".mailmap").is_file(),
    })
}

/// Appends the lines to the `.mailmap` (see `GitBackend::mailmap_add`).
pub(crate) fn add(
    repo: &RepoRef,
    canonical: &Ident,
    from: &[Ident],
    file: Option<&str>,
) -> Result<MailmapAdded> {
    if let Some(problem) = mailmap::problem(canonical, from) {
        return Err(Error::Backend(problem.into()));
    }
    let rel = match file {
        None => ".mailmap".to_string(),
        Some(f) => normalize(f)?,
    };
    if rel.rsplit('/').next() != Some(".mailmap") {
        return Err(Error::Backend(format!("{rel} is not a .mailmap file")));
    }
    let full = repo.root.join(&rel);
    let existing = read_or_empty(&full, &rel)?;
    let (next, added) = mailmap::merged(&existing, canonical, from);
    if added > 0 {
        write(&full, &rel, next)?;
    }
    Ok(MailmapAdded { added, file: rel })
}
