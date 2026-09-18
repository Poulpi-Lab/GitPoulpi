//! `ui.graph_ghost_ref`: which ref a commit with no ref of its own belongs to.
//!
//! The ghost pill shown on the hovered row names the ref whose first-parent
//! chain reaches the commit - the branch whose lane the commit sits on. It is
//! deduced once per layout build from the commits and refs already loaded, so
//! hovering costs a lookup and never a git call.
//!
//! Rules: a local branch beats a remote branch; tags are never ghosts (they
//! are drawn where they are). Within a kind the tip that is newer in the log
//! takes the lane (the checked-out branch first on one commit, then the
//! smaller name), so a fork point shared with a merged side branch stays on
//! the trunk it is drawn on.
//!
//! A commit no live branch reaches may still get a *deleted* ghost: the name
//! a standard merge commit subject gives the branch it merged
//! ([`parse_merge_branch`]). For each merge, newest first, the commits on its
//! second-parent side (first-parent chain from the second parent down to the
//! merge base of the two first-parent chains) that are still unclaimed take
//! that name. Squash merges, rebases and fast-forwards leave no such subject,
//! so those commits get nothing. Pure over the loaded nodes, O(commits).

use std::collections::HashMap;

use gitpoulpi_core::domain::{CommitNode, Oid, Ref, RefKind};

use crate::log_pills::RefPill;
use crate::refs_tree::RefRowKind;

/// The deduced ref, as the pill and the tooltip need it.
pub(crate) struct GhostRef {
    pub pill: RefPill,
    /// The remote's name, for a remote branch (the tooltip spells it out).
    pub remote: Option<String>,
    /// The merge commit that records the branch, for a ghost of a deleted
    /// branch (inferred from a merge subject); `None` for a live branch.
    pub deleted_in: Option<Oid>,
}

impl GhostRef {
    /// The ghost pill's tooltip: the ref and its kind, e.g. `origin -> update
    /// (Remote Branch)`.
    pub(crate) fn tooltip(&self) -> String {
        use gitpoulpi_i18n::msg::tooltip as t;
        if let Some(merge) = &self.deleted_in {
            let hex = merge.to_hex();
            return t::ghost_deleted(&self.pill.name, &hex[..hex.len().min(7)]);
        }
        match (&self.pill.kind, &self.remote) {
            (RefRowKind::Remote, Some(remote)) => t::ghost_remote(remote, &self.pill.name),
            _ => t::ghost_local(&self.pill.name),
        }
    }
}

/// One tip to walk from: its class (0 local, 1 remote), the sort
/// rank within the class, the node it points at and its `GhostRef`.
struct Tip {
    class: u8,
    rank: (usize, bool, String),
    node: usize,
    ghost: usize,
}

/// For each of `nodes` (same order) the index into the returned list of the
/// ref it belongs to, or `None`.
pub(super) fn deduce(nodes: &[CommitNode], refs: &[Ref]) -> (Vec<GhostRef>, Vec<Option<u32>>) {
    let index: HashMap<Oid, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let mut ghosts: Vec<GhostRef> = Vec::new();
    let mut tips: Vec<Tip> = Vec::new();
    for r in refs {
        let Some(&node) = index.get(&r.target) else {
            continue;
        };
        let (class, kind, name, remote) = match &r.kind {
            RefKind::LocalBranch { name } => (0, RefRowKind::Local, name.clone(), None),
            RefKind::RemoteBranch { remote, name } => {
                (1, RefRowKind::Remote, name.clone(), Some(remote.clone()))
            }
            RefKind::Tag { .. } => continue,
        };
        tips.push(Tip {
            class,
            // Newer tip first; on one commit the checked-out branch, then by name.
            rank: (node, !r.is_head, name.clone()),
            node,
            ghost: ghosts.len(),
        });
        ghosts.push(GhostRef {
            pill: RefPill {
                name,
                kind,
                checked_out: false,
                combined_remote: false,
                tag_where: None,
            },
            remote,
            deleted_in: None,
        });
    }
    tips.sort_by(|a, b| (a.class, &a.rank).cmp(&(b.class, &b.rank)));

    // Newest tip first within a class: it takes the lane, and a commit already
    // claimed (by this class or a stronger one) ends the walk, its tail being
    // claimed too.
    let mut owner: Vec<Option<u32>> = vec![None; nodes.len()];
    for tip in &tips {
        let mut at = Some(tip.node);
        while let Some(node) = at.filter(|&n| owner[n].is_none()) {
            owner[node] = Some(tip.ghost as u32);
            at = nodes[node]
                .parents
                .first()
                .and_then(|p| index.get(p).copied());
        }
    }
    infer_deleted(nodes, &index, &mut ghosts, &mut owner);
    (ghosts, owner)
}

/// The branch name a standard merge commit subject records for the branch it
/// merged, or `None`. Understood: `Merge branch 'x'` (with `into y`, `into
/// 'y'`, `of host:path`), `Merge remote-tracking branch 'origin/x'`, `Merge
/// pull request #N from owner/x`, and `Merged in x (pull request #N)`.
pub(crate) fn parse_merge_branch(subject: &str) -> Option<String> {
    // The subjects git and the hosts write; not UI text.
    const BRANCH: &str = "Merge branch "; // i18n-english
    const REMOTE_TRACKING: &str = "Merge remote-tracking branch "; // i18n-english
    const PULL_REQUEST: &str = "Merge pull request #"; // i18n-english
    const FROM: &str = " from "; // i18n-english
    const MERGED_IN: &str = "Merged in "; // i18n-english
    const PR_SUFFIX: &str = " (pull request"; // i18n-english
    let subject = subject.trim();
    let name = if let Some(rest) = subject.strip_prefix(BRANCH) {
        quoted(rest)?
    } else if let Some(rest) = subject.strip_prefix(REMOTE_TRACKING) {
        quoted(rest)?.split_once('/')?.1.to_string()
    } else if let Some(rest) = subject.strip_prefix(PULL_REQUEST) {
        let from = rest.split_once(FROM)?.1;
        from.split_whitespace()
            .next()?
            .split_once('/')?
            .1
            .to_string()
    } else {
        let rest = subject.strip_prefix(MERGED_IN)?;
        rest.split(PR_SUFFIX).next()?.trim().to_string()
    };
    (!name.is_empty() && !name.contains(char::is_whitespace)).then_some(name)
}

/// The text of the leading quoted word of `s` (`'x' into y` -> `x`).
fn quoted(s: &str) -> Option<String> {
    let quote = s.chars().next().filter(|c| matches!(c, '\'' | '"'))?;
    let inner = &s[1..];
    Some(inner[..inner.find(quote)?].to_string())
}

/// Longest first-parent walk per merge, so a pathological log stays cheap.
const MAX_SIDE_WALK: usize = 5000;

/// Gives the unclaimed commits of each merge's second-parent side the branch
/// name the merge subject records. `nodes` are newest first, so a parent's
/// index is always larger than its child's: the two first-parent chains are
/// walked newest-first until they meet at the merge base; the side chain's
/// commits before that are the branch's. A commit a live branch (or an
/// earlier merge) already claimed ends the walk; chains that never meet claim
/// nothing.
fn infer_deleted(
    nodes: &[CommitNode],
    index: &HashMap<Oid, usize>,
    ghosts: &mut Vec<GhostRef>,
    owner: &mut [Option<u32>],
) {
    let first_parent = |n: usize| nodes[n].parents.first().and_then(|p| index.get(p).copied());
    for (m, merge) in nodes.iter().enumerate() {
        if merge.parents.len() < 2 {
            continue;
        }
        let Some(name) = parse_merge_branch(&merge.summary) else {
            continue;
        };
        let (Some(&side), Some(&trunk)) =
            (index.get(&merge.parents[1]), index.get(&merge.parents[0]))
        else {
            continue;
        };
        let (mut a, mut b) = (Some(trunk), Some(side));
        let mut claimed: Vec<usize> = Vec::new();
        let mut met = false;
        for _ in 0..MAX_SIDE_WALK * 2 {
            match (a, b) {
                (Some(x), Some(y)) if x == y => {
                    met = true;
                    break;
                }
                (Some(x), Some(y)) if x < y => a = first_parent(x),
                (Some(_), Some(y)) => {
                    if owner[y].is_some() {
                        break;
                    }
                    claimed.push(y);
                    b = first_parent(y);
                }
                _ => break,
            }
        }
        if !met || claimed.is_empty() {
            continue;
        }
        let ghost = ghosts.len() as u32;
        ghosts.push(GhostRef {
            pill: RefPill {
                name,
                kind: RefRowKind::Local,
                checked_out: false,
                combined_remote: false,
                tag_where: None,
            },
            remote: None,
            deleted_in: Some(nodes[m].id),
        });
        for n in claimed {
            owner[n] = Some(ghost);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn name_of(nodes: &[CommitNode], refs: &[Ref], id: u8) -> Option<String> {
        let (ghosts, owner) = deduce(nodes, refs);
        let i = nodes.iter().position(|n| n.id == oid(id))?;
        owner[i].map(|g| ghosts[g as usize].pill.name.clone())
    }

    /// Newest first: 1 (main tip) -> 2 -> 4 and 3 (feature tip) -> 4.
    fn forked() -> Vec<CommitNode> {
        vec![
            commit(1, &[2]),
            commit(2, &[4]),
            commit(3, &[4]),
            commit(4, &[]),
        ]
    }

    #[test]
    fn a_commit_belongs_to_the_branch_whose_chain_reaches_it() {
        let refs = vec![
            local_branch_at("main", oid(1)),
            local_branch_at("feature", oid(3)),
        ];
        let nodes = forked();
        assert_eq!(name_of(&nodes, &refs, 2).as_deref(), Some("main"));
        assert_eq!(name_of(&nodes, &refs, 3).as_deref(), Some("feature"));
    }

    #[test]
    fn a_merged_side_branch_is_reached_through_the_second_parent_only_by_its_own_tip() {
        // 1 merges 2 (first parent) and 3 (second); 3 -> 4, 2 -> 4.
        let nodes = vec![
            commit(1, &[2, 3]),
            commit(2, &[4]),
            commit(3, &[4]),
            commit(4, &[]),
        ];
        let refs = vec![
            local_branch_at("main", oid(1)),
            local_branch_at("feature", oid(3)),
        ];
        assert_eq!(name_of(&nodes, &refs, 3).as_deref(), Some("feature"));
        assert_eq!(name_of(&nodes, &refs, 2).as_deref(), Some("main"));
        // The shared fork point stays on the trunk: the newer tip takes it.
        assert_eq!(name_of(&nodes, &refs, 4).as_deref(), Some("main"));
    }

    #[test]
    fn the_newer_tip_takes_the_lane_and_the_checked_out_branch_wins_one_commit() {
        // 1 -> 2 -> 3; `new` at 1 and `old` at 2 share the tail.
        let nodes = vec![commit(1, &[2]), commit(2, &[3]), commit(3, &[])];
        let refs = vec![
            local_branch_at("old", oid(2)),
            local_branch_at("new", oid(1)),
        ];
        assert_eq!(name_of(&nodes, &refs, 3).as_deref(), Some("new"));
        // Two branches on the same commit: the checked-out one.
        let mut head = local_branch_at("zz-head", oid(1));
        head.is_head = true;
        let refs = vec![local_branch_at("aa", oid(1)), head];
        assert_eq!(name_of(&nodes, &refs, 3).as_deref(), Some("zz-head"));
    }

    #[test]
    fn local_beats_remote_even_when_farther() {
        let nodes = vec![commit(1, &[2]), commit(2, &[3]), commit(3, &[])];
        let refs = vec![
            local_branch_at("loc", oid(1)),
            remote_branch_at("origin", "rem", oid(2)),
        ];
        assert_eq!(name_of(&nodes, &refs, 3).as_deref(), Some("loc"));
        let only_remote = vec![remote_branch_at("origin", "rem", oid(1))];
        let (ghosts, owner) = deduce(&nodes, &only_remote);
        let g = &ghosts[owner[2].unwrap() as usize];
        assert_eq!(g.pill.name, "rem");
        assert_eq!(g.remote.as_deref(), Some("origin"));
    }

    #[test]
    fn a_tag_is_never_a_ghost_and_an_unreached_commit_gets_nothing() {
        let nodes = vec![commit(1, &[2]), commit(2, &[])];
        let refs = vec![tag_at("v1.2", oid(2))];
        assert_eq!(name_of(&nodes, &refs, 2), None);
        // A tag in front of a branch does not take its commits either.
        let refs = vec![tag_at("v1", oid(1)), local_branch_at("main", oid(1))];
        assert_eq!(name_of(&nodes, &refs, 2).as_deref(), Some("main"));
        assert_eq!(name_of(&nodes, &[], 1), None);
    }

    #[test]
    fn the_tooltip_names_the_ref_and_its_kind() {
        let nodes = vec![commit(1, &[2]), commit(2, &[])];
        let cases = [
            (
                remote_branch_at("origin", "update", oid(1)),
                "origin -> update (Remote Branch)",
            ),
            (local_branch_at("update", oid(1)), "update (Local Branch)"),
        ];
        for (r, text) in cases {
            let (ghosts, owner) = deduce(&nodes, &[r]);
            assert_eq!(ghosts[owner[1].unwrap() as usize].tooltip(), text);
        }
    }

    #[test]
    fn merge_subjects_name_the_merged_branch() {
        let cases = [
            ("Merge branch 'feature'", Some("feature")),
            (
                "Merge branch 'feature/login' into main",
                Some("feature/login"),
            ),
            ("Merge branch 'x' into 'main'", Some("x")),
            ("Merge branch \"dq\" into main", Some("dq")),
            ("Merge branch 'x' of github.com:o/r into y", Some("x")),
            ("Merge pull request #42 from owner/fix-bug", Some("fix-bug")),
            ("Merge pull request #7 from o/feat/deep", Some("feat/deep")),
            ("Merge remote-tracking branch 'origin/topic'", Some("topic")),
            (
                "Merge remote-tracking branch 'origin/a/b' into main",
                Some("a/b"),
            ),
            ("Merged in topic (pull request #3)", Some("topic")),
            ("Merge tag 'v1'", None),
            ("Merge branch ''", None),
            ("Merge pull request #1", None),
            ("Merge branches 'a' and 'b'", None),
            ("Fix the merge branch 'x'", None),
        ];
        for (subject, want) in cases {
            assert_eq!(parse_merge_branch(subject).as_deref(), want, "{subject}");
        }
    }

    /// 1 merges 2 (trunk, first parent) and 4 (side); 4 -> 5 and 2 -> 3 -> 5;
    /// only `main` exists, at 1.
    fn merged_then_deleted(subject: &str) -> Vec<CommitNode> {
        let mut nodes = vec![
            commit(1, &[2, 4]),
            commit(2, &[3]),
            commit(4, &[5]),
            commit(3, &[5]),
            commit(5, &[6]),
            commit(6, &[]),
        ];
        nodes[0].summary = subject.into();
        nodes
    }

    #[test]
    fn a_deleted_branch_is_inferred_from_the_merge_that_recorded_it() {
        let nodes = merged_then_deleted("Merge branch 'feature'");
        let refs = vec![local_branch_at("main", oid(1))];
        let (ghosts, owner) = deduce(&nodes, &refs);
        // The side commit (4) is feature's, with the merge it was merged in.
        let g = &ghosts[owner[2].unwrap() as usize];
        assert_eq!(g.pill.name, "feature");
        assert_eq!(g.deleted_in, Some(oid(1)));
        assert!(g
            .tooltip()
            .starts_with("Branch feature (deleted), via merge "));
        // The trunk, the merge base and the merge itself stay on main.
        for id in [1, 2, 3, 5, 6] {
            assert_eq!(name_of(&nodes, &refs, id).as_deref(), Some("main"), "{id}");
        }
    }

    #[test]
    fn a_fast_forward_or_an_unnamed_merge_infers_nothing() {
        // Fast-forward: no merge commit at all.
        let nodes = vec![commit(1, &[2]), commit(2, &[3]), commit(3, &[])];
        assert_eq!(name_of(&nodes, &[], 2), None);
        // A merge whose subject names no branch.
        let nodes = merged_then_deleted("Combine the work");
        assert_eq!(name_of(&nodes, &[], 4), None);
        // A branch that still exists is not replaced by an inferred one.
        let nodes = merged_then_deleted("Merge branch 'feature'");
        let refs = vec![
            local_branch_at("main", oid(1)),
            local_branch_at("feature", oid(4)),
        ];
        let (ghosts, owner) = deduce(&nodes, &refs);
        assert_eq!(ghosts[owner[2].unwrap() as usize].deleted_in, None);
    }

    fn tag_at(name: &str, target: Oid) -> Ref {
        Ref {
            kind: RefKind::Tag { name: name.into() },
            ..local_branch_at("x", target)
        }
    }
}
