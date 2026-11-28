//! The questions the Log asks before it opens "Edit commit message..." or
//! "Amend...", and how a message is split into the popup's two fields.
//!
//! A frontend asks these of the loaded commit graph and refs: is the commit on
//! the checked-out branch at all ([`on_current_branch`]), which other branches
//! hold it ([`other_branches_holding`]) and has it already been pushed
//! ([`pushed_upstream`], which means a force push will be needed). The popup
//! itself (text widgets, focus) stays in the frontend.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{CommitGraph, Oid, Ref, RefKind};

/// The commit HEAD's branch points at, when a branch is checked out.
fn head_tip(refs: &[Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }))
        .map(|r| r.target)
}

/// Whether `commit` is `from` or one of its ancestors, walking only the
/// commits the Log has loaded. `None` when the walk falls off the loaded
/// page before it can tell.
fn reaches(graph: &CommitGraph, from: Oid, commit: Oid) -> Option<bool> {
    let parents: HashMap<Oid, &[Oid]> = graph
        .nodes
        .iter()
        .map(|n| (n.id, n.parents.as_slice()))
        .collect();
    let mut seen = HashSet::new();
    let mut stack = vec![from];
    let mut incomplete = false;
    while let Some(oid) = stack.pop() {
        if oid == commit {
            return Some(true);
        }
        if !seen.insert(oid) {
            continue;
        }
        match parents.get(&oid) {
            Some(ps) => stack.extend(ps.iter().copied()),
            None => incomplete = true,
        }
    }
    if incomplete {
        None
    } else {
        Some(false)
    }
}

/// `Some(false)` when the loaded graph shows `commit` is *not* on the
/// checked-out branch - the backend would refuse, so the Log says so
/// without opening a popup. `Some(true)` or `None` (detached HEAD, or a
/// history deeper than what is loaded) let the popup open; the backend
/// has the last word either way.
pub fn on_current_branch(graph: &CommitGraph, refs: &[Ref], commit: Oid) -> Option<bool> {
    reaches(graph, head_tip(refs)?, commit)
}

/// The local branches (other than the checked-out one) whose history, as far
/// as the loaded graph shows, holds `commit`: where it could be edited after
/// a checkout. Sorted by name.
pub fn other_branches_holding(graph: &CommitGraph, refs: &[Ref], commit: Oid) -> Vec<String> {
    let mut names: Vec<String> = refs
        .iter()
        .filter(|r| !r.is_head)
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => {
                (reaches(graph, r.target, commit) == Some(true)).then(|| name.clone())
            }
            _ => None,
        })
        .collect();
    names.sort();
    names
}

/// The upstream (`origin/main`) of the checked-out branch when `commit`
/// is already on it - i.e. rewording it will need a force push. `None`
/// when the branch has no upstream, the upstream is not a known remote
/// branch, or the commit is not on it.
///
/// When the loaded graph cannot tell (the upstream's history leaves the
/// loaded page before reaching the commit), the answer is the upstream:
/// warning about a force push that turns out unnecessary costs a click,
/// not warning about one that is costs a rejected push.
pub fn pushed_upstream(graph: &CommitGraph, refs: &[Ref], commit: Oid) -> Option<String> {
    let upstream = refs
        .iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }))?
        .upstream
        .clone()?;
    let tip = refs.iter().find_map(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == upstream => {
            Some(r.target)
        }
        _ => None,
    })?;
    match reaches(graph, tip, commit) {
        Some(false) => None,
        Some(true) | None => Some(upstream),
    }
}

/// Splits a commit message into its one-line summary and its
/// description (everything after the blank line that follows the
/// summary), line endings normalized to `\n`.
pub fn split_message(message: &str) -> (String, String) {
    let unix = message.replace("\r\n", "\n");
    let trimmed = unix.trim_end();
    match trimmed.split_once('\n') {
        Some((summary, rest)) => (
            summary.trim_end().to_string(),
            rest.trim_start_matches('\n').to_string(),
        ),
        None => (trimmed.to_string(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{CommitNode, RefVisibility};

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn commit(id: u8, parents: &[u8]) -> CommitNode {
        CommitNode {
            id: oid(id),
            parents: parents.iter().map(|&p| oid(p)).collect(),
            summary: format!("commit {id}"),
            message: format!("commit {id}"),
            author_name: "a".into(),
            author_email: "a@example.com".into(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }

    fn branch(name: &str, target: u8, is_head: bool, upstream: Option<&str>) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: oid(target),
            visibility: RefVisibility::Shown,
            is_head,
            upstream: upstream.map(str::to_string),
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn remote(remote: &str, name: &str, target: u8) -> Ref {
        Ref {
            kind: RefKind::RemoteBranch {
                remote: remote.into(),
                name: name.into(),
            },
            target: oid(target),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    /// 1 <- 2 <- 3 <- 4 (main, HEAD); origin/main is at 2; 9 is on
    /// another branch, off 1.
    fn graph() -> CommitGraph {
        CommitGraph {
            nodes: vec![
                commit(4, &[3]),
                commit(3, &[2]),
                commit(9, &[1]),
                commit(2, &[1]),
                commit(1, &[]),
            ],
        }
    }

    #[test]
    fn a_commit_on_the_upstream_is_pushed_and_one_above_it_is_not() {
        let refs = vec![
            branch("main", 4, true, Some("origin/main")),
            remote("origin", "main", 2),
        ];
        let g = graph();
        assert_eq!(
            pushed_upstream(&g, &refs, oid(2)).as_deref(),
            Some("origin/main")
        );
        assert_eq!(
            pushed_upstream(&g, &refs, oid(1)).as_deref(),
            Some("origin/main")
        );
        assert_eq!(pushed_upstream(&g, &refs, oid(3)), None);
        assert_eq!(pushed_upstream(&g, &refs, oid(4)), None);
        // No upstream: nothing to force push.
        let unset = vec![branch("main", 4, true, None), remote("origin", "main", 2)];
        assert_eq!(pushed_upstream(&g, &unset, oid(2)), None);
    }

    #[test]
    fn only_head_and_its_ancestors_are_on_the_current_branch() {
        let refs = vec![branch("main", 4, true, None)];
        let g = graph();
        assert_eq!(on_current_branch(&g, &refs, oid(4)), Some(true));
        assert_eq!(on_current_branch(&g, &refs, oid(1)), Some(true));
        assert_eq!(on_current_branch(&g, &refs, oid(9)), Some(false));
        // Detached HEAD: the Log cannot tell, the backend decides.
        let detached = vec![branch("main", 4, false, None)];
        assert_eq!(on_current_branch(&g, &detached, oid(9)), None);
    }
}
