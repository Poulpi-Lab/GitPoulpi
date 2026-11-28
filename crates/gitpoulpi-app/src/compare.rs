//! Comparing what the Log's multi-select holds ("select two
//! commits and the right panel shows the differences between them").
//!
//! The comparison is a pure function of the selection (`log_multiselect`),
//! the Log's rows and the graph; nothing is stored but whether the user
//! swapped the two ends. [`resolve`] turns the selected rows into one
//! [`gitpoulpi_core::domain::DiffTarget`]:
//!
//! | selection | target |
//! |---|---|
//! | two commits / stashes | `CommitRange(older, newer)`, by Log order |
//! | the WIP row and one commit / stash | `CommitVsWorkingTree(it)` |
//! | three or more, newest a commit | `CommitSpan(oldest, newest)`: from the oldest's first parent |
//! | three or more, WIP among them | `CommitVsWorkingTree(oldest's parent)` |
//!
//! The WIP row is always the newest end and a stash sits above its base, so
//! "older to newer" is just the Log order read from the bottom. The changed
//! files travel the commit-files channel keyed by a synthetic [`Oid`] (a
//! hash of the target), which is why the right pane's list, filter, tree
//! view and click-to-open need no second implementation.

use gitpoulpi_core::domain::{CommitGraph, CommitNode, DiffTarget, Oid, Stash};
use gitpoulpi_i18n::msg;

use crate::log_multiselect::{LogSelection, RowKey};
use crate::ui_kit::short_oid;

/// One end of a comparison, as the Log row it came from.
struct End {
    /// `None` for the WIP row.
    id: Option<Oid>,
    label: String,
    /// The commit's first parent (a stash or the WIP row has none here).
    parent: Option<Oid>,
    stash: bool,
}

/// Which end of the comparison a card is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EndMark {
    From,
    To,
}

/// What kind of Log row a card stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardKind {
    Wip,
    Commit,
    Stash,
}

/// One selected Log row as the comparison panel's card (each compared
/// commit has its own zone): who, when, which sha, what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Card {
    /// The row's display index in the Log (the click target's destination).
    pub(crate) row: usize,
    pub(crate) kind: CardKind,
    /// The commit (or stash commit); `None` for the WIP row.
    pub(crate) id: Option<Oid>,
    /// A commit's author, or a stash's `stash@{n}` handle.
    pub(crate) author_name: String,
    pub(crate) author_email: String,
    pub(crate) timestamp: i64,
    pub(crate) summary: String,
    pub(crate) mark: Option<EndMark>,
}

/// What the right pane compares and how the center diff reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompareSpec {
    /// The key the changed files are published under (`CommitFilesOutcome::commit`).
    pub(crate) id: Oid,
    pub(crate) target: DiffTarget,
    /// "from" and "to" ends, ready to show.
    pub(crate) from_label: String,
    pub(crate) to_label: String,
    /// How many Log rows are selected.
    pub(crate) rows: usize,
    /// Whether the two ends can trade places (two rows, no WIP).
    pub(crate) can_swap: bool,
    pub(crate) swapped: bool,
    /// Every selected row in Log order (newest first), one per card.
    pub(crate) cards: Vec<Card>,
}

impl CompareSpec {
    /// The comparison reads the working tree, so its file list goes stale
    /// whenever a file is saved and must be asked again on a refresh.
    pub(crate) fn reads_worktree(&self) -> bool {
        matches!(self.target, DiffTarget::CommitVsWorkingTree(_))
    }

    /// Lines for the message block: both ends, and for a range what it means.
    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            msg::compare::from_line(&self.from_label),
            msg::compare::to_line(&self.to_label),
        ];
        if self.rows > 2 {
            lines.push(msg::compare::range_note(self.rows as u64));
        }
        lines
    }

    /// The pseudo-commit the right pane's detail code is handed in place of
    /// a selected commit: only its `id` matters to the file list.
    pub(crate) fn node(&self) -> CommitNode {
        CommitNode {
            id: self.id,
            parents: Vec::new(),
            summary: self.from_label.clone(),
            message: self.lines().join("\n"),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }
}

/// A stable stand-in `Oid` for a target, so the file list of one
/// comparison never answers for another.
fn synthetic_id(target: &DiffTarget) -> Oid {
    use std::hash::{Hash, Hasher};
    let text = format!("{target:?}");
    let mut bytes = [0u8; 20];
    for (i, chunk) in bytes.chunks_mut(8).enumerate() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (i, &text).hash(&mut hasher);
        let hash = hasher.finish().to_le_bytes();
        let n = chunk.len();
        chunk.copy_from_slice(&hash[..n]);
    }
    Oid(bytes)
}

fn stash_handle(stash: &Stash) -> String {
    format!("stash@{{{}}}", stash.id.0)
}

fn describe(
    key: RowKey,
    graph: &CommitGraph,
    stashes: &gitpoulpi_core::Result<Vec<Stash>>,
) -> Option<End> {
    let RowKey::Id(id) = key else {
        return Some(End {
            id: None,
            label: msg::compare::WIP.get().to_string(),
            parent: None,
            stash: false,
        });
    };
    if let Some(node) = graph.nodes.iter().find(|n| n.id == id) {
        return Some(End {
            id: Some(id),
            label: format!("{} {}", short_oid(id), node.summary),
            parent: node.parents.first().copied(),
            stash: false,
        });
    }
    let stash = stashes
        .as_ref()
        .ok()?
        .iter()
        .find(|s| s.commit == Some(id))?;
    Some(End {
        id: Some(id),
        label: format!("{} {}", stash_handle(stash), stash.message),
        parent: None,
        stash: true,
    })
}

fn card(
    row: usize,
    key: RowKey,
    graph: &CommitGraph,
    stashes: &gitpoulpi_core::Result<Vec<Stash>>,
) -> Option<Card> {
    let RowKey::Id(id) = key else {
        return Some(Card {
            row,
            kind: CardKind::Wip,
            id: None,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            summary: String::new(),
            mark: None,
        });
    };
    if let Some(node) = graph.nodes.iter().find(|n| n.id == id) {
        return Some(Card {
            row,
            kind: CardKind::Commit,
            id: Some(id),
            author_name: node.author_name.clone(),
            author_email: node.author_email.clone(),
            timestamp: node.timestamp,
            summary: node.summary.clone(),
            mark: None,
        });
    }
    let stash = stashes
        .as_ref()
        .ok()?
        .iter()
        .find(|s| s.commit == Some(id))?;
    Some(Card {
        row,
        kind: CardKind::Stash,
        id: Some(id),
        author_name: stash_handle(stash),
        author_email: String::new(),
        timestamp: stash.timestamp,
        summary: stash.message.clone(),
        mark: None,
    })
}

/// How long ago `ts` was at `now`, in the app's relative wording.
pub(crate) fn relative_age(now: i64, ts: i64) -> String {
    use msg::reltime as r;
    let secs = (now - ts).max(0);
    let n = |d: i64| d as u64;
    match secs {
        0..=59 => r::JUST_NOW.get().to_string(),
        60..=3599 => r::minutes_ago(n(secs / 60)),
        3600..=86_399 => r::hours_ago(n(secs / 3600)),
        86_400..=604_799 => r::days_ago(n(secs / 86_400)),
        604_800..=2_591_999 => r::weeks_ago(n(secs / 604_800)),
        2_592_000..=31_535_999 => r::months_ago(n(secs / 2_592_000)),
        _ => r::years_ago(n(secs / 31_536_000)),
    }
}

/// The comparison the selection stands for, if it holds two rows or more
/// that the Log can still name.
pub(crate) fn resolve(
    selection: &LogSelection,
    display: &[Option<RowKey>],
    log_result: &gitpoulpi_core::Result<CommitGraph>,
    stashes: &gitpoulpi_core::Result<Vec<Stash>>,
    swapped: bool,
) -> Option<CompareSpec> {
    let keys = selection.in_display_order(display);
    if keys.len() < 2 {
        return None;
    }
    let graph = log_result.as_ref().ok()?;
    let newest = describe(keys[0], graph, stashes)?;
    let oldest = describe(*keys.last()?, graph, stashes)?;
    let rows = keys.len();
    let can_swap = rows == 2 && newest.id.is_some();
    let swapped = swapped && can_swap;

    let (target, from_label, to_label) = match (newest.id, oldest.id) {
        // The WIP row on top: the oldest end against what is on disk.
        (None, Some(oldest_id)) => {
            // From the oldest commit's parent, so the commit's own change is
            // part of a range. (A root commit has none: it stands as it is.)
            let from = if rows > 2 {
                oldest.parent.unwrap_or(oldest_id)
            } else {
                oldest_id
            };
            let from_label = if rows > 2 && oldest.parent.is_some() {
                msg::compare::before(&oldest.label)
            } else {
                oldest.label.clone()
            };
            (
                DiffTarget::CommitVsWorkingTree(from),
                from_label,
                newest.label.clone(),
            )
        }
        (Some(newest_id), Some(oldest_id)) => {
            if swapped {
                (
                    DiffTarget::CommitRange(newest_id, oldest_id),
                    newest.label.clone(),
                    oldest.label.clone(),
                )
            } else if rows > 2 && !oldest.stash {
                (
                    DiffTarget::CommitSpan(oldest_id, newest_id),
                    msg::compare::before(&oldest.label),
                    newest.label.clone(),
                )
            } else {
                (
                    DiffTarget::CommitRange(oldest_id, newest_id),
                    oldest.label.clone(),
                    newest.label.clone(),
                )
            }
        }
        // The WIP row cannot be the oldest: nothing sits below it.
        (_, None) => return None,
    };
    let mut cards = keys
        .iter()
        .map(|k| {
            let row = display.iter().position(|d| *d == Some(*k))?;
            card(row, *k, graph, stashes)
        })
        .collect::<Option<Vec<_>>>()?;
    // The older end is the last card, the newer the first; a swap trades them.
    let (from_i, to_i) = if swapped {
        (0, cards.len() - 1)
    } else {
        (cards.len() - 1, 0)
    };
    cards[from_i].mark = Some(EndMark::From);
    cards[to_i].mark = Some(EndMark::To);
    Some(CompareSpec {
        id: synthetic_id(&target),
        target,
        from_label,
        to_label,
        rows,
        can_swap,
        swapped,
        cards,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn graph() -> gitpoulpi_core::Result<CommitGraph> {
        Ok(CommitGraph {
            nodes: vec![
                commit(4, &[3]),
                commit(3, &[2]),
                commit(2, &[1]),
                commit(1, &[]),
            ],
        })
    }

    fn display() -> Vec<Option<RowKey>> {
        vec![
            Some(RowKey::Wip),
            Some(RowKey::Id(oid(4))),
            Some(RowKey::Id(oid(3))),
            Some(RowKey::Id(oid(2))),
            Some(RowKey::Id(oid(1))),
        ]
    }

    fn selection(rows: &[usize]) -> LogSelection {
        let mut sel = LogSelection::default();
        for r in rows {
            sel.toggle(display()[*r].unwrap(), None);
        }
        sel
    }

    fn spec(rows: &[usize], swapped: bool) -> Option<CompareSpec> {
        resolve(
            &selection(rows),
            &display(),
            &graph(),
            &Ok(Vec::new()),
            swapped,
        )
    }

    #[test]
    fn two_commits_compare_older_to_newer_whatever_the_click_order() {
        let a = spec(&[4, 2], false).unwrap();
        assert_eq!(a.target, DiffTarget::CommitRange(oid(1), oid(3)));
        let b = spec(&[2, 4], false).unwrap();
        assert_eq!(a.target, b.target);
        assert!(a.can_swap);
    }

    #[test]
    fn swapping_reverses_two_commits_and_not_a_wip_comparison() {
        let s = spec(&[4, 2], true).unwrap();
        assert_eq!(s.target, DiffTarget::CommitRange(oid(3), oid(1)));
        assert!(s.swapped);
        let w = spec(&[0, 2], true).unwrap();
        assert!(!w.swapped && !w.can_swap);
    }

    #[test]
    fn the_wip_row_and_a_commit_compare_the_commit_with_the_working_tree() {
        let s = spec(&[0, 3], false).unwrap();
        assert_eq!(s.target, DiffTarget::CommitVsWorkingTree(oid(2)));
        assert!(s.reads_worktree());
    }

    #[test]
    fn three_commits_are_combined_from_the_oldests_parent() {
        let s = spec(&[1, 2, 3], false).unwrap();
        assert_eq!(s.target, DiffTarget::CommitSpan(oid(2), oid(4)));
        assert_eq!(s.rows, 3);
        let w = spec(&[0, 2, 3], false).unwrap();
        assert_eq!(
            w.target,
            DiffTarget::CommitVsWorkingTree(oid(1)),
            "WIP over commits 3 and 2: from the parent of the oldest, commit 1"
        );
    }

    #[test]
    fn cards_follow_the_log_order_and_mark_the_ends_through_a_swap() {
        let s = spec(&[0, 3, 1], false).unwrap();
        let rows: Vec<usize> = s.cards.iter().map(|c| c.row).collect();
        assert_eq!(rows, [0, 1, 3], "newest first, with the display row");
        assert_eq!(s.cards[0].kind, CardKind::Wip);
        assert_eq!(s.cards[0].mark, Some(EndMark::To));
        assert_eq!(s.cards[1].mark, None);
        assert_eq!(s.cards[2].mark, Some(EndMark::From));
        let two = spec(&[1, 3], true).unwrap();
        assert_eq!(two.cards[0].mark, Some(EndMark::From));
        assert_eq!(two.cards[1].mark, Some(EndMark::To));
        assert_eq!(two.cards[0].summary, "commit 4");
    }

    #[test]
    fn relative_ages_use_the_largest_whole_unit() {
        assert_eq!(relative_age(100, 100), "just now");
        assert_eq!(relative_age(10_000, 10_000 - 7200), "2 hours ago");
        assert_eq!(
            relative_age(10_000_000, 10_000_000 - 86_400 * 3),
            "3 days ago"
        );
    }

    #[test]
    fn one_row_is_no_comparison_and_each_target_has_its_own_key() {
        assert!(spec(&[2], false).is_none());
        assert_ne!(
            spec(&[1, 2], false).unwrap().id,
            spec(&[1, 3], false).unwrap().id
        );
    }
}
