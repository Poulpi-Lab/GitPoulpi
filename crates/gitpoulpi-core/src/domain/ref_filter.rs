//! Which refs a graph walk starts from, when some are hidden.
//!
//! The Log walks every branch, tag and remote-tracking ref (`all_refs`).
//! Hiding a ref in the graph must also drop the commits only that ref
//! reaches, so the walk cannot be "everything, then filter pills": the
//! backends list the refs, drop the hidden ones with [`visible_tips`] and
//! walk from what is left. The decision lives here so `Git2Backend` and
//! `SystemGitBackend` cannot disagree about it.

use serde::{Deserialize, Serialize};

use super::Oid;

/// What a walk leaves out. Names are the shorthands the persisted
/// visibility uses (`RepoState::ref_visibility`): a local branch `x`, a
/// remote-tracking branch `origin/x`, a tag `v1`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RefFilter {
    /// Hidden refs, as shorthands.
    #[serde(default)]
    pub names: Vec<String>,
    /// Hidden remotes by name: every `refs/remotes/<remote>/...`, the ones
    /// fetched later included.
    #[serde(default)]
    pub remotes: Vec<String>,
    /// Shorthands never excluded: soloed refs that are also hidden.
    #[serde(default)]
    pub keep: Vec<String>,
}

impl RefFilter {
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.remotes.is_empty()
    }

    /// Whether the ref with this full name (`refs/heads/x`,
    /// `refs/remotes/origin/x`, `refs/tags/v1`) is left out of the walk.
    pub fn hides(&self, full_name: &str) -> bool {
        let (short, remote_ref) = if let Some(s) = full_name.strip_prefix("refs/heads/") {
            (s, false)
        } else if let Some(s) = full_name.strip_prefix("refs/remotes/") {
            (s, true)
        } else if let Some(s) = full_name.strip_prefix("refs/tags/") {
            (s, false)
        } else {
            return false;
        };
        if self.keep.iter().any(|k| k == short) {
            return false;
        }
        if self.names.iter().any(|n| n == short) {
            return true;
        }
        remote_ref
            && self.remotes.iter().any(|r| {
                short
                    .strip_prefix(r.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
            })
    }
}

/// The commits a filtered walk starts from: every ref of `refs` (full name
/// and the commit it peels to) the filter keeps, plus `head` (the walk
/// always includes the checked-out commit while a filter is active: the
/// working-tree row, commits and amends refer to it). Sorted by ref name,
/// then deduplicated, so a paged walk sees the same input every time.
pub fn visible_tips(filter: &RefFilter, refs: &[(String, Oid)], head: Option<Oid>) -> Vec<Oid> {
    let mut kept: Vec<&(String, Oid)> = refs.iter().filter(|(n, _)| !filter.hides(n)).collect();
    kept.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out: Vec<Oid> = Vec::with_capacity(kept.len() + 1);
    let mut seen = std::collections::HashSet::new();
    for id in kept.into_iter().map(|(_, id)| *id).chain(head) {
        if seen.insert(id) {
            out.push(id);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn refs() -> Vec<(String, Oid)> {
        vec![
            ("refs/remotes/archive/old-b".into(), oid(3)),
            ("refs/heads/main".into(), oid(1)),
            ("refs/remotes/archive/grp/deep".into(), oid(4)),
            ("refs/remotes/origin/main".into(), oid(1)),
            ("refs/tags/v1".into(), oid(5)),
            ("refs/remotes/archive2/x".into(), oid(6)),
        ]
    }

    #[test]
    fn empty_filter_keeps_everything_and_sorts() {
        let tips = visible_tips(&RefFilter::default(), &refs(), None);
        assert_eq!(tips, vec![oid(1), oid(4), oid(3), oid(6), oid(5)]);
        assert!(RefFilter::default().is_empty());
    }

    #[test]
    fn a_hidden_remote_is_a_prefix_on_a_path_boundary() {
        let f = RefFilter {
            remotes: vec!["archive".into()],
            ..RefFilter::default()
        };
        assert!(f.hides("refs/remotes/archive/old-b"));
        assert!(f.hides("refs/remotes/archive/grp/deep"));
        assert!(!f.hides("refs/remotes/archive2/x"), "no partial name match");
        assert!(
            !f.hides("refs/heads/archive/old-b"),
            "local branches are not remote refs"
        );
        assert_eq!(
            visible_tips(&f, &refs(), None),
            vec![oid(1), oid(6), oid(5)]
        );
    }

    #[test]
    fn names_hide_by_shorthand_and_keep_overrides() {
        let f = RefFilter {
            names: vec!["main".into(), "origin/main".into(), "v1".into()],
            keep: vec!["origin/main".into()],
            ..RefFilter::default()
        };
        assert!(f.hides("refs/heads/main"));
        assert!(!f.hides("refs/remotes/origin/main"));
        assert!(f.hides("refs/tags/v1"));
    }

    #[test]
    fn head_is_always_added_once() {
        let f = RefFilter {
            names: vec!["main".into(), "origin/main".into()],
            ..RefFilter::default()
        };
        let tips = visible_tips(&f, &refs(), Some(oid(1)));
        assert_eq!(tips.iter().filter(|t| **t == oid(1)).count(), 1);
        assert_eq!(*tips.last().unwrap(), oid(1));
        // Already a tip through another ref: not duplicated.
        let tips = visible_tips(&RefFilter::default(), &refs(), Some(oid(1)));
        assert_eq!(tips.len(), 5);
    }
}
