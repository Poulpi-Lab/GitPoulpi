//! Comparing the rows the Log has selected ("select two commits and the right pane shows what differs").
//!
//! The comparison is a pure function of the selection and the loaded rows; nothing is stored but whether
//! the two ends were swapped. [`resolve`] turns the rows into one [`DiffTarget`] (the terminal UI's
//! `compare.rs` has the same table and keeps it private, so it is restated here):
//!
//! | selection | target |
//! |---|---|
//! | two commits / stashes | `CommitRange(older, newer)`, by Log order |
//! | the WIP row and one commit / stash | `CommitVsWorkingTree(it)` |
//! | three or more, newest a commit | `CommitSpan(oldest, newest)`: from the oldest's first parent |
//! | three or more, WIP among them | `CommitVsWorkingTree(oldest's parent)` |

use gitpoulpi_core::domain::{DiffTarget, Oid};
use gitpoulpi_i18n::msg;

use super::data::{LogData, RowKey};
use super::marks::Marks;
use crate::daemon::commit_ops::short;
use crate::strings;

/// Which end of the comparison a card is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndMark {
    From,
    To,
}

/// What kind of row a card stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    Wip,
    Commit,
    Stash,
}

/// One selected row as the comparison pane's card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// The row's display index in the Log (a click on the card moves the cursor there).
    pub row: usize,
    pub kind: CardKind,
    pub id: Option<Oid>,
    /// A commit's author, or a stash's `stash@{n}` handle.
    pub who: String,
    pub timestamp: i64,
    pub summary: String,
    pub mark: Option<EndMark>,
}

/// What the right pane compares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareSpec {
    /// Stands in for a commit id as the key of the changed files.
    pub id: Oid,
    pub target: DiffTarget,
    pub from_label: String,
    pub to_label: String,
    pub rows: usize,
    pub can_swap: bool,
    pub swapped: bool,
    /// Every selected row in Log order, newest first.
    pub cards: Vec<Card>,
}

impl CompareSpec {
    /// The comparison reads the working tree, so its file list goes stale when a file is saved.
    pub fn reads_worktree(&self) -> bool {
        matches!(self.target, DiffTarget::CommitVsWorkingTree(_))
    }
}

struct End {
    id: Option<Oid>,
    label: String,
    parent: Option<Oid>,
    stash: bool,
}

/// A stable stand-in `Oid` for a target, so the file list of one comparison never answers for another.
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

fn describe(key: &RowKey, data: &LogData) -> Option<End> {
    match key {
        RowKey::Wip => Some(End {
            id: None,
            label: msg::compare::WIP.get().to_string(),
            parent: None,
            stash: false,
        }),
        RowKey::Commit(id) => {
            let node = data.nodes.iter().find(|n| n.id == *id)?;
            Some(End {
                id: Some(*id),
                label: format!("{} {}", short(*id), node.summary),
                parent: node.parents.first().copied(),
                stash: false,
            })
        }
        RowKey::Stash(commit, _) => {
            let st = data.stashes.iter().find(|s| s.commit == *commit)?;
            Some(End {
                id: *commit,
                label: format!("{} {}", strings::stash_name(st.id.0), st.message),
                parent: None,
                stash: true,
            })
        }
    }
}

fn card(row: usize, key: &RowKey, data: &LogData) -> Option<Card> {
    Some(match key {
        RowKey::Wip => Card {
            row,
            kind: CardKind::Wip,
            id: None,
            who: msg::compare::WORKING_TREE.get().to_string(),
            timestamp: 0,
            summary: msg::compare::UNCOMMITTED.get().to_string(),
            mark: None,
        },
        RowKey::Commit(id) => {
            let node = data.nodes.iter().find(|n| n.id == *id)?;
            Card {
                row,
                kind: CardKind::Commit,
                id: Some(*id),
                who: node.author_name.clone(),
                timestamp: node.timestamp,
                summary: node.summary.clone(),
                mark: None,
            }
        }
        RowKey::Stash(commit, _) => {
            let st = data.stashes.iter().find(|s| s.commit == *commit)?;
            Card {
                row,
                kind: CardKind::Stash,
                id: *commit,
                who: strings::stash_name(st.id.0),
                timestamp: st.timestamp,
                summary: st.message.clone(),
                mark: None,
            }
        }
    })
}

/// The comparison the marked rows stand for, when there are two or more the Log can still name.
pub fn resolve(marks: &Marks, data: &LogData, swapped: bool) -> Option<CompareSpec> {
    let display = data.display_keys();
    let keys = marks.in_display_order(&display);
    if keys.len() < 2 {
        return None;
    }
    let newest = describe(&keys[0], data)?;
    let oldest = describe(keys.last()?, data)?;
    let rows = keys.len();
    let can_swap = rows == 2 && newest.id.is_some();
    let swapped = swapped && can_swap;

    let (target, from_label, to_label) = match (newest.id, oldest.id) {
        // The WIP row on top: the oldest end against what is on disk.
        (None, Some(oldest_id)) => {
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
            let row = display.iter().position(|d| d == k)?;
            card(row, k, data)
        })
        .collect::<Option<Vec<_>>>()?;
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
    use crate::graph::{GraphInput, GraphLayout};
    use gitpoulpi_core::domain::CommitNode;
    use std::sync::Arc;

    fn node(n: u8, parent: Option<u8>) -> CommitNode {
        CommitNode {
            id: Oid([n; 20]),
            parents: parent.map(|p| Oid([p; 20])).into_iter().collect(),
            summary: format!("c{n}"),
            message: format!("c{n}"),
            author_name: "Ada".into(),
            author_email: String::new(),
            timestamp: 0,
            refs: vec![],
        }
    }

    /// WIP, then c3 -> c2 -> c1.
    fn data() -> LogData {
        let nodes = Arc::new(vec![node(3, Some(2)), node(2, Some(1)), node(1, None)]);
        let layout = GraphLayout::build(&GraphInput {
            nodes: &nodes,
            refs: &[],
            stashes: &[],
            dirty: true,
            head: Some(Oid([3; 20])),
            palette_len: 4,
        });
        LogData::new(
            nodes.clone(),
            nodes,
            Arc::default(),
            Arc::default(),
            vec![],
            None,
            Some(Oid([3; 20])),
            layout,
        )
    }

    fn marked(keys: &[RowKey]) -> Marks {
        let mut m = Marks::default();
        for (i, k) in keys.iter().enumerate() {
            m.toggle(k.clone(), (i > 0).then(|| keys[0].clone()));
        }
        m
    }

    fn c(n: u8) -> RowKey {
        RowKey::Commit(Oid([n; 20]))
    }

    #[test]
    fn two_commits_compare_older_against_newer_whatever_the_click_order() {
        let d = data();
        let spec = resolve(&marked(&[c(3), c(1)]), &d, false).unwrap();
        assert_eq!(
            spec.target,
            DiffTarget::CommitRange(Oid([1; 20]), Oid([3; 20]))
        );
        assert!(spec.can_swap);
        let swapped = resolve(&marked(&[c(1), c(3)]), &d, true).unwrap();
        assert_eq!(
            swapped.target,
            DiffTarget::CommitRange(Oid([3; 20]), Oid([1; 20]))
        );
        assert_eq!(swapped.cards[0].mark, Some(EndMark::From));
    }

    #[test]
    fn three_commits_combine_and_the_wip_row_reads_the_working_tree() {
        let d = data();
        let span = resolve(&marked(&[c(3), c(2), c(1)]), &d, false).unwrap();
        assert_eq!(
            span.target,
            DiffTarget::CommitSpan(Oid([1; 20]), Oid([3; 20]))
        );
        assert!(!span.can_swap);
        let wip = resolve(&marked(&[RowKey::Wip, c(2)]), &d, true).unwrap();
        assert_eq!(wip.target, DiffTarget::CommitVsWorkingTree(Oid([2; 20])));
        assert!(wip.reads_worktree() && !wip.can_swap);
        assert!(resolve(&marked(&[c(2)]), &d, false).is_none());
    }
}
