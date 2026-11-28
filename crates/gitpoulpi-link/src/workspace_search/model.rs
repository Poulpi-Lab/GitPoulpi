//! The workspace search's pure half: which kinds a query asks for, how the
//! answers of every repository merge into one grouped list, and how the
//! highlight skips the group headings. No terminal, no daemon.

use gitpoulpi_core::domain::{parse_query, SearchHit, SearchKind, SearchTarget};
use gitpoulpi_core::fuzzy::{self, Anchor};
use gitpoulpi_hosting::cache_search::{self, CachedItems};

use super::scope::Scope;

/// The kinds in the order of the chips (`Alt+1` .. `Alt+8`) and of the groups.
pub const ORDER: [SearchKind; 8] = [
    SearchKind::Repo,
    SearchKind::Branch,
    SearchKind::Tag,
    SearchKind::Worktree,
    SearchKind::Stash,
    SearchKind::Commit,
    SearchKind::PullRequest,
    SearchKind::Issue,
];

/// The best this many of each kind are listed: a repository with a thousand
/// matching commits must not push every other kind out of the window.
pub const PER_GROUP: usize = 30;

/// Where `kind` sits in [`ORDER`].
pub fn slot(kind: SearchKind) -> usize {
    ORDER.iter().position(|k| *k == kind).unwrap_or(ORDER.len())
}

/// The kinds `query` asks for given the chips: a prefix (`b:`, `pr:`, `#`)
/// replaces the chips, like typing it in the CLI.
pub fn effective_kinds(query: &str, chips: &[bool; 8]) -> Vec<SearchKind> {
    let parsed = parse_query(query);
    if let Some(kind) = parsed.kind {
        return vec![kind];
    }
    if parsed.hosting_only {
        return vec![SearchKind::PullRequest, SearchKind::Issue];
    }
    ORDER
        .iter()
        .zip(chips)
        .filter(|(_, on)| **on)
        .map(|(k, _)| *k)
        .collect()
}

/// One row of the result list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: SearchKind,
    pub label: String,
    pub detail: String,
    pub target: SearchTarget,
    pub score: i32,
    pub positions: Vec<u32>,
    /// Index into the scope's repositories.
    pub repo: usize,
}

impl Item {
    pub fn from_hit(hit: SearchHit, repo: usize) -> Self {
        Item {
            kind: hit.kind,
            label: hit.label,
            detail: hit.detail,
            target: hit.target,
            score: hit.score,
            positions: hit.positions,
            repo,
        }
    }

    /// What identifies the row across a rebuild (the selection follows it).
    pub fn same_as(&self, other: &Item) -> bool {
        self.kind == other.kind && self.repo == other.repo && self.target == other.target
    }
}

/// A line of the list: a group heading or an item (index into the items).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Heading(SearchKind, usize),
    Item(usize),
}

/// Groups `items` by kind (in [`ORDER`]), best first inside a group (ties by
/// label, then repository), at most [`PER_GROUP`] each. Returns the items in
/// list order and the rows (headings included) that index them.
pub fn group(mut items: Vec<Item>) -> (Vec<Item>, Vec<Row>) {
    items.sort_by(|a, b| {
        slot(a.kind)
            .cmp(&slot(b.kind))
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.repo.cmp(&b.repo))
    });
    let mut kept: Vec<Item> = Vec::with_capacity(items.len().min(PER_GROUP * ORDER.len()));
    let mut rows: Vec<Row> = Vec::new();
    let mut current: Option<(SearchKind, usize)> = None;
    for item in items {
        match &mut current {
            Some((kind, n)) if *kind == item.kind => {
                if *n >= PER_GROUP {
                    continue;
                }
                *n += 1;
            }
            _ => {
                current = Some((item.kind, 1));
                rows.push(Row::Heading(item.kind, 0));
            }
        }
        rows.push(Row::Item(kept.len()));
        kept.push(item);
    }
    // Headings carry their group's count.
    let mut at = None;
    let mut count = 0;
    for i in 0..rows.len() {
        match rows[i] {
            Row::Heading(..) => {
                if let Some(h) = at {
                    set_count(&mut rows, h, count);
                }
                at = Some(i);
                count = 0;
            }
            Row::Item(_) => count += 1,
        }
    }
    if let Some(h) = at {
        set_count(&mut rows, h, count);
    }
    (kept, rows)
}

fn set_count(rows: &mut [Row], at: usize, count: usize) {
    if let Row::Heading(kind, _) = rows[at] {
        rows[at] = Row::Heading(kind, count);
    }
}

/// The first row that is an item, if any.
pub fn first_item_row(rows: &[Row]) -> Option<usize> {
    rows.iter().position(|r| matches!(r, Row::Item(_)))
}

/// Moves a selection that `popup_list::nav` put on `to` (coming from
/// `from`) off a heading: onward in the direction of travel, back the other
/// way at the end of the list.
pub fn skip_headings(rows: &[Row], from: usize, to: usize) -> usize {
    if rows.is_empty() {
        return 0;
    }
    let to = to.min(rows.len() - 1);
    if matches!(rows[to], Row::Item(_)) {
        return to;
    }
    let forward = to >= from;
    let ahead = if forward {
        (to..rows.len()).find(|&i| matches!(rows[i], Row::Item(_)))
    } else {
        (0..=to).rev().find(|&i| matches!(rows[i], Row::Item(_)))
    };
    ahead
        .or_else(|| {
            if forward {
                (0..to).rev().find(|&i| matches!(rows[i], Row::Item(_)))
            } else {
                (to..rows.len()).find(|&i| matches!(rows[i], Row::Item(_)))
            }
        })
        .unwrap_or(from.min(rows.len() - 1))
}

/// The repositories of the scope as `r:` hits: name first, path behind it.
pub fn repo_items(scope: &Scope, text: &str) -> Vec<Item> {
    let text = text.to_lowercase();
    scope
        .repos
        .iter()
        .enumerate()
        .filter_map(|(i, repo)| {
            let by_name = fuzzy::matched(&repo.name.to_lowercase(), &text, Anchor::Whole);
            let path = repo.path.to_string_lossy().into_owned();
            let (score, positions) = match by_name {
                Some(m) => (m.score, m.positions),
                // A path match ranks under every name match.
                None => (
                    fuzzy::score(&path, &text, Anchor::Whole)? - 20_000,
                    Vec::new(),
                ),
            };
            Some(Item {
                kind: SearchKind::Repo,
                label: repo.name.clone(),
                detail: path.clone(),
                target: SearchTarget::Repo { path },
                score,
                positions,
                repo: i,
            })
        })
        .collect()
}

/// The pull/merge requests and issues of every repository's cache for `text`.
pub fn hosting_items(caches: &[CachedItems], kinds: &[SearchKind], text: &str) -> Vec<Item> {
    caches
        .iter()
        .enumerate()
        .flat_map(|(i, cache)| {
            cache_search::search(cache, kinds, text, PER_GROUP)
                .into_iter()
                .map(move |hit| Item::from_hit(hit, i))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace_search::scope::{Scope, ScopeRepo};

    fn item(kind: SearchKind, label: &str, score: i32, repo: usize) -> Item {
        Item {
            kind,
            label: label.into(),
            detail: String::new(),
            target: SearchTarget::Repo {
                path: label.to_string(),
            },
            score,
            positions: Vec::new(),
            repo,
        }
    }

    #[test]
    fn groups_follow_the_chip_order_best_first_and_are_capped() {
        let mut items = vec![
            item(SearchKind::Commit, "c1", 5, 0),
            item(SearchKind::Branch, "low", 1, 0),
            item(SearchKind::Branch, "high", 9, 1),
        ];
        for n in 0..(PER_GROUP + 5) {
            items.push(item(SearchKind::Tag, &format!("t{n:03}"), 3, 0));
        }
        let (items, rows) = group(items);
        let order: Vec<SearchKind> = rows
            .iter()
            .filter_map(|r| match r {
                Row::Heading(k, _) => Some(*k),
                Row::Item(_) => None,
            })
            .collect();
        assert_eq!(
            order,
            [SearchKind::Branch, SearchKind::Tag, SearchKind::Commit]
        );
        assert_eq!(items[0].label, "high");
        let tags = items.iter().filter(|i| i.kind == SearchKind::Tag).count();
        assert_eq!(tags, PER_GROUP);
    }

    #[test]
    fn the_highlight_never_rests_on_a_heading() {
        let (_, rows) = group(vec![
            item(SearchKind::Branch, "a", 1, 0),
            item(SearchKind::Tag, "b", 1, 0),
        ]);
        // rows: heading, a, heading, b
        assert_eq!(first_item_row(&rows), Some(1));
        assert_eq!(skip_headings(&rows, 1, 2), 3);
        assert_eq!(skip_headings(&rows, 3, 2), 1);
    }

    #[test]
    fn a_prefix_replaces_the_chips() {
        let mut chips = [false; 8];
        chips[0] = true;
        assert_eq!(effective_kinds("x", &chips), [SearchKind::Repo]);
        assert_eq!(effective_kinds("b: x", &chips), [SearchKind::Branch]);
    }

    #[test]
    fn repositories_match_by_name_before_path() {
        let scope = Scope {
            workspace: None,
            repos: ["alpha", "bravo"]
                .iter()
                .map(|n| ScopeRepo::new(std::path::Path::new(&format!("/nowhere/{n}"))))
                .collect(),
        };
        let items = repo_items(&scope, "brav");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "bravo");
        assert_eq!(items[0].repo, 1);
    }
}
