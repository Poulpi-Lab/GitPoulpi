//! The workspace search's hosting kinds: pull/merge requests and issues
//! found in a repository's on-disk hosting cache (`hosting-cache.json`, the
//! snapshot the TUI keeps beside `repo-state.toml`), never from a host.
//!
//! The cache is the TUI's file and the search is read-only over it, so this
//! reads a deliberately small, lenient mirror of the snapshot (only what is
//! searched; every other field is ignored) instead of the TUI's own type: a
//! cache written by another version, rewritten while it is read, or
//! truncated is "no hits", never an error. Always compiled (no hosting
//! feature): without one, no cache exists and nothing is found.

use std::path::Path;

use gitpoulpi_core::domain::{SearchHit, SearchKind, SearchTarget};
use gitpoulpi_core::fuzzy::{self, Anchor};
use serde::Deserialize;

/// A hit found only through the author or the branch ranks under every
/// title match.
const SECONDARY_PENALTY: i32 = 8_000;
/// A merged or closed item ranks under open ones of the same match.
const SETTLED_PENALTY: i32 = 60;

/// One cached request or issue, reduced to what is searched.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CachedItem {
    #[serde(default)]
    pub number: u64,
    /// `#12` (GitHub, GitLab issues) or `!53` (GitLab merge requests).
    #[serde(default)]
    pub iid_label: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    /// A request's source branch; empty for an issue.
    #[serde(default)]
    pub source_branch: String,
    /// `open`, `draft`, `merged`, `closed` as the cache spells it.
    #[serde(default)]
    pub state: String,
}

/// What a repository's cache holds.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CachedItems {
    #[serde(default)]
    pub prs: Vec<CachedItem>,
    #[serde(default)]
    pub issues: Vec<CachedItem>,
}

/// The cache at `path`; missing, unreadable, corrupt or half-written is empty.
pub fn load(path: &Path) -> CachedItems {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn label_of(item: &CachedItem, fallback_mark: char) -> String {
    let mark = if item.iid_label.is_empty() {
        format!("{fallback_mark}{}", item.number)
    } else {
        item.iid_label.clone()
    };
    format!("{mark} {}", item.title)
}

fn hit(item: &CachedItem, kind: SearchKind, text_lower: &str) -> Option<SearchHit> {
    let label = label_of(
        item,
        if kind == SearchKind::PullRequest {
            '!'
        } else {
            '#'
        },
    );
    let lower = label.to_lowercase();
    let digits = text_lower.trim_start_matches(['#', '!']);
    let by_number = (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
        .then(|| {
            let n = item.number.to_string();
            if n == digits {
                Some(40_000)
            } else if n.starts_with(digits) {
                Some(35_000 - n.len() as i32)
            } else {
                None
            }
        })
        .flatten();
    let primary = fuzzy::matched(&lower, text_lower, Anchor::Whole);
    let branch_lower = item.source_branch.to_lowercase();
    let author_lower = item.author.to_lowercase();
    let secondary = [&author_lower, &branch_lower]
        .into_iter()
        .filter(|t| !t.is_empty())
        .filter_map(|t| fuzzy::score_lower(t, text_lower, Anchor::Whole))
        .max()
        .map(|s| s - SECONDARY_PENALTY);
    let (mut score, positions) = match (by_number, primary, secondary) {
        (Some(n), Some(m), _) if m.score <= n => (n, Vec::new()),
        (Some(n), None, _) => (n, Vec::new()),
        (_, Some(m), Some(s)) if s > m.score => (s, Vec::new()),
        (_, Some(m), _) => (m.score, m.positions),
        (_, None, Some(s)) => (s, Vec::new()),
        (None, None, None) => return None,
    };
    if matches!(item.state.as_str(), "merged" | "closed") {
        score -= SETTLED_PENALTY;
    }
    let mut detail = item.author.clone();
    if !item.source_branch.is_empty() {
        if !detail.is_empty() {
            detail.push_str(" \u{b7} ");
        }
        detail.push_str(&item.source_branch);
    }
    if matches!(item.state.as_str(), "merged" | "closed" | "draft") {
        if !detail.is_empty() {
            detail.push_str(" \u{b7} ");
        }
        detail.push_str(&item.state);
    }
    Some(SearchHit {
        kind,
        label,
        detail,
        target: match kind {
            SearchKind::PullRequest => SearchTarget::PullRequest {
                number: item.number,
            },
            _ => SearchTarget::Issue {
                number: item.number,
            },
        },
        score,
        positions,
        repo: None,
    })
}

/// The best `limit` requests and issues of `items` for `text` (a query with
/// its prefix already removed), restricted to `kinds` (only the hosting
/// kinds count; empty = both). Best first, then by number, newest first.
pub fn search(
    items: &CachedItems,
    kinds: &[SearchKind],
    text: &str,
    limit: usize,
) -> Vec<SearchHit> {
    let text_lower = text.trim().to_lowercase();
    let wants = |k: SearchKind| kinds.is_empty() || kinds.contains(&k);
    let mut hits: Vec<SearchHit> = Vec::new();
    if wants(SearchKind::PullRequest) {
        hits.extend(
            items
                .prs
                .iter()
                .filter_map(|i| hit(i, SearchKind::PullRequest, &text_lower)),
        );
    }
    if wants(SearchKind::Issue) {
        hits.extend(
            items
                .issues
                .iter()
                .filter_map(|i| hit(i, SearchKind::Issue, &text_lower)),
        );
    }
    let number = |h: &SearchHit| match h.target {
        SearchTarget::PullRequest { number } | SearchTarget::Issue { number } => number,
        _ => 0,
    };
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| number(b).cmp(&number(a)))
    });
    if limit > 0 {
        hits.truncate(limit);
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> CachedItems {
        serde_json::from_str(
            r##"{
              "profile_id": "p", "unknown_field": [1, 2],
              "prs": [
                {"number": 12, "iid_label": "#12", "title": "Fix login redirect", "author": "ana",
                 "source_branch": "fix/login", "state": "open", "extra": 1},
                {"number": 40, "iid_label": "!40", "title": "Bump deps", "author": "bob",
                 "source_branch": "chore/deps", "state": "merged"}
              ],
              "issues": [
                {"number": 7, "iid_label": "#7", "title": "Login page is slow", "author": "cy",
                 "state": "open"}
              ]
            }"##,
        )
        .unwrap()
    }

    #[test]
    fn a_title_matches_and_highlights_after_the_number() {
        let hits = search(&items(), &[], "login", 10);
        // The earlier match wins: "#7 Login..." before "#12 Fix login...".
        let kinds: Vec<_> = hits.iter().map(|h| h.kind).collect();
        assert_eq!(kinds, [SearchKind::Issue, SearchKind::PullRequest]);
        assert_eq!(hits[1].label, "#12 Fix login redirect");
        // "login" starts at char 8 of the label.
        assert_eq!(hits[1].positions, [8, 9, 10, 11, 12]);
        assert_eq!(hits[1].detail, "ana \u{b7} fix/login");
    }

    #[test]
    fn a_number_finds_its_item_first() {
        let hits = search(&items(), &[], "12", 10);
        assert_eq!(hits[0].target, SearchTarget::PullRequest { number: 12 });
        let hits = search(&items(), &[], "!40", 10);
        assert_eq!(hits[0].target, SearchTarget::PullRequest { number: 40 });
    }

    #[test]
    fn author_and_branch_match_under_the_titles() {
        let by_author = search(&items(), &[], "bob", 10);
        assert_eq!(by_author.len(), 1);
        assert_eq!(by_author[0].label, "!40 Bump deps");
        assert!(by_author[0].positions.is_empty());
        let by_branch = search(&items(), &[], "chore", 10);
        assert_eq!(
            by_branch[0].target,
            SearchTarget::PullRequest { number: 40 }
        );
    }

    #[test]
    fn kinds_restrict_and_an_empty_query_lists_everything() {
        let only_issues = search(&items(), &[SearchKind::Issue], "", 10);
        assert_eq!(only_issues.len(), 1);
        assert_eq!(search(&items(), &[], "", 10).len(), 3);
        assert_eq!(search(&items(), &[], "", 2).len(), 2);
    }

    #[test]
    fn a_missing_or_corrupt_cache_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(&dir.path().join("nope.json")).prs.is_empty());
        let bad = dir.path().join("hosting-cache.json");
        std::fs::write(&bad, "{\"prs\": [{\"number\": 1,").unwrap();
        let cache = load(&bad);
        assert!(cache.prs.is_empty() && cache.issues.is_empty());
        std::fs::write(
            &bad,
            serde_json::to_string(&serde_json::json!({"prs": 3})).unwrap(),
        )
        .unwrap();
        assert!(load(&bad).prs.is_empty());
    }
}
