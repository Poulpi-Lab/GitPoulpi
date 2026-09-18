//! The pure part of the show intents (see the mcp-show-ui design
//! sections 5.1 and 5.2): what to do next about a commit that may not be
//! loaded yet, and which ref a name means. No terminal, no window state, so
//! it is unit-tested here; `event_loop/window/agent_show.rs` applies it.

use gitpoulpi_core::domain::{Oid, Ref, RefKind};

use crate::refs_sidebar::ref_full_name;
use crate::refs_tree::{RefRowKind, RefTreeRow};

/// Past this many pages paged in for one intent, the commit is "older than
/// the first N commits loaded" (plan section 5.2).
pub(crate) const MAX_PAGES: u32 = 20;
/// ... or past this many commits held, whichever comes first.
pub(crate) const MAX_COMMITS: usize = 10_000;

/// Where a commit is, as far as the loaded Log can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    /// A row of the Log as drawn (the display index).
    Shown(usize),
    /// Loaded, but the soloed refs hide it: never unhidden for an agent.
    Filtered,
    /// Not among the loaded commits (yet).
    Missing,
}

/// What the window knows about the lazy Log.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogFacts {
    pub(crate) presence: Presence,
    /// Commits held.
    pub(crate) loaded: usize,
    /// Pages this intent already asked for.
    pub(crate) pages: u32,
    /// A page request is in flight.
    pub(crate) loading: bool,
    /// The lazy loader would ask for one more page now (lazy loading is on,
    /// history is not exhausted, the configured ceiling is not reached).
    pub(crate) can_page: bool,
}

/// The next step towards showing a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommitStep {
    /// Select this display row.
    Select(usize),
    /// Loaded but hidden by the soloed refs: answer `Partial`.
    Hidden,
    /// A page is on its way.
    Wait,
    /// Ask for one more page.
    RequestPage,
    /// Give up: the text of the `NotFound`.
    GiveUp(String),
}

pub(crate) fn commit_step(f: &LogFacts) -> CommitStep {
    match f.presence {
        Presence::Shown(row) => CommitStep::Select(row),
        Presence::Filtered => CommitStep::Hidden,
        Presence::Missing if f.loading => CommitStep::Wait,
        Presence::Missing if f.pages >= MAX_PAGES || f.loaded >= MAX_COMMITS => {
            CommitStep::GiveUp(format!(
                "commit is older than the first {} commits loaded", // i18n-english
                f.loaded
            ))
        }
        Presence::Missing if !f.can_page => CommitStep::GiveUp(format!(
            "commit is not among the {} commits the Log holds", // i18n-english
            f.loaded
        )),
        Presence::Missing => CommitStep::RequestPage,
    }
}

/// A ref a name stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefTarget {
    pub(crate) kind: RefRowKind,
    /// The name the sidebar's rows key on (`ref_full_name`): a branch or
    /// tag's bare name, a remote branch's `remote/name`.
    pub(crate) name: String,
    pub(crate) target: Oid,
}

/// Finds the ref called `wanted`: a full name (`refs/heads/x`,
/// `refs/remotes/origin/x`, `refs/tags/v1`) or a short one, tried as a local
/// branch, then a remote branch, then a tag.
pub(crate) fn find_ref(refs: &[Ref], wanted: &str) -> Option<RefTarget> {
    let (only, short) = if let Some(rest) = wanted.strip_prefix("refs/heads/") {
        (Some(RefRowKind::Local), rest)
    } else if let Some(rest) = wanted.strip_prefix("refs/remotes/") {
        (Some(RefRowKind::Remote), rest)
    } else if let Some(rest) = wanted.strip_prefix("refs/tags/") {
        (Some(RefRowKind::Tag), rest)
    } else {
        (None, wanted)
    };
    let kind_of = |r: &Ref| match r.kind {
        RefKind::LocalBranch { .. } => RefRowKind::Local,
        RefKind::RemoteBranch { .. } => RefRowKind::Remote,
        RefKind::Tag { .. } => RefRowKind::Tag,
    };
    [RefRowKind::Local, RefRowKind::Remote, RefRowKind::Tag]
        .into_iter()
        .filter(|k| only.map_or(true, |o| o == *k))
        .find_map(|kind| {
            refs.iter()
                .find(|r| kind_of(r) == kind && ref_full_name(r) == short)
                .map(|r| RefTarget {
                    kind,
                    name: short.to_string(),
                    target: r.target,
                })
        })
}

/// The collapse keys of the folders above the leaf `leaf` in `rows` (the
/// section's rows with nothing collapsed): what to open so the row shows.
pub(crate) fn folders_to_open(rows_all: &[RefTreeRow], leaf: &str) -> Vec<String> {
    let Some(at) = rows_all.iter().position(|r| r.leaf_name() == Some(leaf)) else {
        return Vec::new();
    };
    let mut depth = rows_all[at].depth();
    let mut out = Vec::new();
    for row in rows_all[..at].iter().rev() {
        if depth == 0 {
            break;
        }
        if let RefTreeRow::Folder { path, depth: d, .. } = row {
            if *d < depth {
                out.push(path.clone());
                depth = *d;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refs_tree::ref_display_rows;
    use gitpoulpi_core::domain::RefVisibility;
    use std::collections::HashSet;

    fn facts(presence: Presence) -> LogFacts {
        LogFacts {
            presence,
            loaded: 100,
            pages: 0,
            loading: false,
            can_page: true,
        }
    }

    #[test]
    fn a_loaded_commit_is_selected_and_a_filtered_one_is_never_unhidden() {
        assert_eq!(
            commit_step(&facts(Presence::Shown(7))),
            CommitStep::Select(7)
        );
        assert_eq!(commit_step(&facts(Presence::Filtered)), CommitStep::Hidden);
    }

    #[test]
    fn a_missing_commit_pages_in_one_page_at_a_time() {
        assert_eq!(
            commit_step(&facts(Presence::Missing)),
            CommitStep::RequestPage
        );
        let waiting = LogFacts {
            loading: true,
            ..facts(Presence::Missing)
        };
        assert_eq!(commit_step(&waiting), CommitStep::Wait);
    }

    #[test]
    fn paging_stops_at_the_page_cap_the_commit_cap_and_the_end_of_history() {
        let capped = LogFacts {
            pages: MAX_PAGES,
            ..facts(Presence::Missing)
        };
        assert!(matches!(commit_step(&capped), CommitStep::GiveUp(m) if m.contains("older")));
        let big = LogFacts {
            loaded: MAX_COMMITS,
            ..facts(Presence::Missing)
        };
        assert!(matches!(commit_step(&big), CommitStep::GiveUp(_)));
        let end = LogFacts {
            can_page: false,
            ..facts(Presence::Missing)
        };
        assert!(matches!(commit_step(&end), CommitStep::GiveUp(_)));
        // A page in flight is waited for even at the end of the budget.
        let last = LogFacts {
            pages: MAX_PAGES,
            loading: true,
            ..facts(Presence::Missing)
        };
        assert_eq!(commit_step(&last), CommitStep::Wait);
    }

    fn r(kind: RefKind, byte: u8) -> Ref {
        Ref {
            kind,
            target: Oid([byte; 20]),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
        }
    }

    fn refs() -> Vec<Ref> {
        vec![
            r(
                RefKind::LocalBranch {
                    name: "main".into(),
                },
                1,
            ),
            r(
                RefKind::LocalBranch {
                    name: "feature/deep/x".into(),
                },
                2,
            ),
            r(
                RefKind::RemoteBranch {
                    remote: "origin".into(),
                    name: "main".into(),
                },
                3,
            ),
            r(RefKind::Tag { name: "v1".into() }, 4),
        ]
    }

    #[test]
    fn a_ref_is_found_by_full_or_short_name() {
        let refs = refs();
        let main = find_ref(&refs, "main").unwrap();
        assert_eq!((main.kind, main.target), (RefRowKind::Local, Oid([1; 20])));
        assert_eq!(
            find_ref(&refs, "refs/heads/feature/deep/x").unwrap().name,
            "feature/deep/x"
        );
        let remote = find_ref(&refs, "origin/main").unwrap();
        assert_eq!(
            (remote.kind, remote.target),
            (RefRowKind::Remote, Oid([3; 20]))
        );
        assert_eq!(
            find_ref(&refs, "refs/remotes/origin/main").unwrap().kind,
            RefRowKind::Remote
        );
        assert_eq!(find_ref(&refs, "v1").unwrap().kind, RefRowKind::Tag);
        assert!(find_ref(&refs, "refs/tags/main").is_none());
        assert!(find_ref(&refs, "nope").is_none());
    }

    #[test]
    fn the_folders_above_a_leaf_are_the_ones_to_open() {
        let names = vec!["feature/deep/x".to_string(), "main".to_string()];
        let rows = ref_display_rows(&names, RefRowKind::Local, true, &HashSet::new(), &[]);
        assert_eq!(
            folders_to_open(&rows, "feature/deep/x"),
            vec!["feature/deep".to_string(), "feature".to_string()]
        );
        assert!(folders_to_open(&rows, "main").is_empty());
        assert!(folders_to_open(&rows, "absent").is_empty());
    }
}
