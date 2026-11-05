//! The refs the Log's walk leaves out: what the hidden set (and, once there
//! is one, the hidden remotes) means to the daemon's `LogOptions::hidden`.
//!
//! Hiding a ref used to remove its pill only; the commits stayed. Now the
//! daemon walks from the visible tips, so the same facts that remove a pill
//! also remove the commits only that ref reaches. One function builds the
//! filter, sorted so equal states are equal values (the link compares them
//! to know when to reload, the daemon keys its read cache on them).

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use gitpoulpi_core::domain::RefFilter;

/// The filter for the hidden set, the hidden remotes and the soloed set.
///
/// `keep` is every soloed ref that is also hidden: solo shows that ref's
/// ancestry whatever hiding says (the solo filter itself stays client-side).
pub(crate) fn walk_filter(
    hidden: &HashSet<String>,
    hidden_remotes: &BTreeSet<String>,
    solo: &HashSet<String>,
) -> RefFilter {
    let sorted = |set: &HashSet<String>| {
        let mut v: Vec<String> = set.iter().cloned().collect();
        v.sort();
        v
    };
    let in_hidden_remote = |name: &str| {
        hidden_remotes.iter().any(|r| {
            name.strip_prefix(r.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
        })
    };
    let keep: HashSet<String> = solo
        .iter()
        .filter(|s| hidden.contains(*s) || in_hidden_remote(s))
        .cloned()
        .collect();
    RefFilter {
        names: sorted(hidden),
        remotes: hidden_remotes.iter().cloned().collect(),
        keep: sorted(&keep),
    }
}

/// The filter the repository's persisted visibility describes: what a link
/// starts with, so the first paint is already filtered.
pub(crate) fn load_walk_filter(repo_root: &Path) -> RefFilter {
    let (hidden, solo) = crate::refs_tree::load_ref_visibility(repo_root);
    walk_filter(&hidden, &BTreeSet::new(), &solo)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_filter_is_sorted_so_equal_states_are_equal() {
        let a = walk_filter(&set(&["b", "a/x", "c"]), &BTreeSet::new(), &set(&[]));
        let b = walk_filter(&set(&["c", "a/x", "b"]), &BTreeSet::new(), &set(&[]));
        assert_eq!(a, b);
        assert_eq!(a.names, vec!["a/x", "b", "c"]);
        assert!(a.keep.is_empty());
    }

    #[test]
    fn keep_is_the_soloed_refs_that_are_hidden() {
        let f = walk_filter(&set(&["a", "b"]), &BTreeSet::new(), &set(&["b", "c"]));
        assert_eq!(f.keep, vec!["b"]);
        let remotes: BTreeSet<String> = ["archive".to_string()].into();
        let f = walk_filter(&set(&[]), &remotes, &set(&["archive/old", "origin/main"]));
        assert_eq!(f.keep, vec!["archive/old"], "hidden through its remote");
        assert_eq!(f.remotes, vec!["archive"]);
    }

    #[test]
    fn nothing_hidden_is_the_empty_filter() {
        assert!(walk_filter(&set(&[]), &BTreeSet::new(), &set(&["x"])).is_empty());
    }
}
