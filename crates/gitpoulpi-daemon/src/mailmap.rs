//! `log.use_mailmap` (slice S16): the Log, blame and file history show each
//! author as the repository's `.mailmap` names them.
//!
//! The daemon owns the setting (it is read on every request, like
//! `git.default_encoding`), so a TUI, the CLI and MCP all show the same
//! names and a change in Settings reaches the others on their next read. The
//! Log asks the backend for `%aN`/`%aE` (`LogOptions::use_mailmap`, no extra
//! process); blame and file history carry few distinct people, so their
//! answers are mapped here with one `mailmap_resolve` call.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::rewrite::Ident;
use gitpoulpi_core::domain::{Blame, BlameContent, FileHistoryPage, RepoRef};
use gitpoulpi_core::GitBackend;

/// `log.use_mailmap` (default on when the configuration cannot be read).
pub(crate) fn enabled() -> bool {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().log.use_mailmap)
        .unwrap_or(true)
}

/// Part of a cached read's key: nothing while the setting is off, else the
/// stamp of the worktree's `.mailmap`, so editing the file by hand shows up
/// on the next read. (`mailmap.file` is not stamped.)
pub(crate) fn stamp(root: &str) -> String {
    if !enabled() {
        return String::new();
    }
    let meta = std::fs::metadata(std::path::Path::new(root).join(".mailmap")).ok();
    let modified = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("mailmap:{modified}:{}", meta.map_or(0, |m| m.len()))
}

/// The canonical identity of each distinct `idents` entry; an identity the
/// backend could not resolve stays as it is.
fn resolved(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    idents: impl Iterator<Item = Ident>,
) -> HashMap<Ident, Ident> {
    let mut seen = HashSet::new();
    let distinct: Vec<Ident> = idents.filter(|i| seen.insert(i.clone())).collect();
    match backend.mailmap_resolve(repo, &distinct) {
        Ok(out) if out.len() == distinct.len() => distinct.into_iter().zip(out).collect(),
        _ => HashMap::new(),
    }
}

/// Blame through the `.mailmap`.
pub(crate) fn map_blame(backend: &dyn GitBackend, repo: &RepoRef, mut blame: Blame) -> Blame {
    if !enabled() {
        return blame;
    }
    let BlameContent::Lines(lines) = &mut blame.content else {
        return blame;
    };
    let map = resolved(
        backend,
        repo,
        lines
            .iter()
            .map(|l| Ident::new(&l.author_name, &l.author_email)),
    );
    for l in lines.iter_mut() {
        if let Some(to) = map.get(&Ident::new(&l.author_name, &l.author_email)) {
            l.author_name.clone_from(&to.name);
            l.author_email.clone_from(&to.email);
        }
    }
    blame
}

/// A file's history through the `.mailmap`.
pub(crate) fn map_file_history(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    mut page: FileHistoryPage,
) -> FileHistoryPage {
    if !enabled() {
        return page;
    }
    let map = resolved(
        backend,
        repo,
        page.entries
            .iter()
            .map(|e| Ident::new(&e.author_name, &e.author_email)),
    );
    for e in &mut page.entries {
        if let Some(to) = map.get(&Ident::new(&e.author_name, &e.author_email)) {
            e.author_name.clone_from(&to.name);
            e.author_email.clone_from(&to.email);
        }
    }
    page
}
