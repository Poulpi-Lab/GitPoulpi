//! Keeping the repository list's view in place when its content changes
//! under the user (the saved list replaced by the host's answer): the same
//! project stays highlighted and the viewport stays on the same projects,
//! even if repositories appeared above or vanished. Pure: rows in, indexes
//! out; the dialog captures before the swap and restores after it.

use super::groups::Group;

/// A row of the painted list, by identity rather than position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RowKey {
    /// A section header, by lower-cased owner.
    Header(String),
    /// A repository, by lower-cased full path.
    Repo(String),
}

/// The identities of the header and repository rows of `groups`, in
/// painted order.
pub(super) fn row_keys(groups: &[Group<'_>]) -> Vec<RowKey> {
    let mut keys = Vec::new();
    for g in groups {
        keys.push(RowKey::Header(g.owner.to_lowercase()));
        keys.extend(
            g.repos
                .iter()
                .map(|p| RowKey::Repo(p.full_name.to_lowercase())),
        );
    }
    keys
}

/// What the view was showing before a swap.
#[derive(Debug, Clone)]
pub(super) struct Anchor {
    /// The first row shown.
    top: Option<RowKey>,
    /// The highlighted repository (lower-cased full path).
    cursor: Option<String>,
    /// How many rows below the top the highlight was, when it was in view.
    cursor_slot: Option<usize>,
    old: Vec<RowKey>,
}

/// Notes the view: `rows` as painted, the highlighted repository's key,
/// the first row shown and the rows that fit.
pub(super) fn capture(
    rows: Vec<RowKey>,
    cursor: Option<String>,
    offset: usize,
    visible: usize,
) -> Anchor {
    let cursor_row = cursor
        .as_ref()
        .and_then(|c| rows.iter().position(|r| *r == RowKey::Repo(c.clone())));
    let cursor_slot = cursor_row
        .filter(|r| *r >= offset && *r < offset + visible)
        .map(|r| r - offset);
    Anchor {
        top: rows.get(offset).cloned(),
        cursor,
        cursor_slot,
        old: rows,
    }
}

/// The row of `old` nearest to `from` (after it first, then before it, at
/// each distance) that `accept`s and survives in `new`.
fn nearest_survivor(
    old: &[RowKey],
    from: usize,
    new: &[RowKey],
    accept: impl Fn(&RowKey) -> bool,
) -> Option<usize> {
    let survives = |i: usize| {
        old.get(i)
            .filter(|k| accept(k))
            .and_then(|k| new.iter().position(|n| n == k))
    };
    if let Some(row) = survives(from) {
        return Some(row);
    }
    (1..old.len())
        .find_map(|d| survives(from + d).or_else(|| from.checked_sub(d).and_then(survives)))
}

/// Where the highlight and the first row shown go in `new`:
/// `(cursor index among the repositories, offset)`.
pub(super) fn restore(anchor: &Anchor, new: &[RowKey], visible: usize) -> (usize, usize) {
    let is_repo = |k: &RowKey| matches!(k, RowKey::Repo(_));
    // 1. The highlight: the same repository, else its nearest surviving
    // neighbour, else the first one.
    let cursor_row = anchor
        .cursor
        .as_ref()
        .and_then(|c| {
            let key = RowKey::Repo(c.clone());
            new.iter().position(|n| *n == key).or_else(|| {
                let from = anchor.old.iter().position(|o| *o == key)?;
                nearest_survivor(&anchor.old, from, new, is_repo)
            })
        })
        .or_else(|| new.iter().position(is_repo));
    let Some(cursor_row) = cursor_row else {
        return (0, 0);
    };
    let cursor_index = new[..cursor_row].iter().filter(|k| is_repo(k)).count();
    // 2. The first row shown: the same row, else its nearest survivor.
    let mut offset = anchor
        .top
        .as_ref()
        .and_then(|top| {
            new.iter().position(|n| n == top).or_else(|| {
                let from = anchor.old.iter().position(|o| o == top)?;
                nearest_survivor(&anchor.old, from, new, |_| true)
            })
        })
        .unwrap_or(0);
    // 3. A highlight that was in view stays in view, in the same slot.
    if let Some(slot) = anchor.cursor_slot {
        if cursor_row < offset || cursor_row >= offset + visible {
            offset = cursor_row.saturating_sub(slot);
        }
    }
    // 4. Never past the last full page.
    (cursor_index, offset.min(new.len().saturating_sub(visible)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(o: &str) -> RowKey {
        RowKey::Header(o.into())
    }
    fn r(p: &str) -> RowKey {
        RowKey::Repo(p.into())
    }

    /// One section `a` with the repositories `names`.
    fn section(names: &[&str]) -> Vec<RowKey> {
        let mut rows = vec![h("a")];
        rows.extend(names.iter().map(|n| r(&format!("a/{n}"))));
        rows
    }

    fn view(old: &[RowKey], cursor: &str, offset: usize, visible: usize) -> Anchor {
        capture(old.to_vec(), Some(format!("a/{cursor}")), offset, visible)
    }

    #[test]
    fn a_repository_inserted_above_the_top_keeps_the_top_and_the_cursor() {
        let old = section(&["c", "d", "e", "f", "g", "h", "i", "j"]);
        // Top is row 2 (`a/d`), cursor on `a/f` (row 4).
        let anchor = view(&old, "f", 2, 4);
        let new = section(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let (cursor, offset) = restore(&anchor, &new, 4);
        assert_eq!(new[offset], r("a/d"));
        assert_eq!(cursor, 5, "`a/f` is the sixth repository");
    }

    #[test]
    fn a_removed_top_row_falls_back_to_its_nearest_neighbour() {
        let old = section(&["c", "d", "e", "f", "g", "h", "i", "j"]);
        let anchor = view(&old, "g", 3, 4); // top `a/e`
        let new = section(&["c", "d", "f", "g", "h", "i", "j"]); // `a/e` gone
        let (_, offset) = restore(&anchor, &new, 4);
        assert_eq!(new[offset], r("a/f"), "the row after it takes over");
    }

    #[test]
    fn a_removed_cursor_moves_to_its_nearest_neighbour() {
        let old = section(&["c", "d", "e", "f"]);
        let anchor = view(&old, "e", 0, 6);
        let new = section(&["c", "d", "f"]);
        let (cursor, _) = restore(&anchor, &new, 6);
        assert_eq!(cursor, 2, "`a/f`, the repository after the removed one");
        let anchor = view(&old, "f", 0, 6);
        let new = section(&["c", "d", "e"]);
        assert_eq!(restore(&anchor, &new, 6).0, 2, "else the one before");
    }

    #[test]
    fn rows_inserted_between_top_and_cursor_keep_the_cursor_in_its_slot() {
        let old = section(&["a", "b", "c"]);
        let anchor = view(&old, "c", 0, 4); // top is the header, cursor slot 3
        let mut names = vec!["a", "b"];
        let many: Vec<String> = (0..20).map(|i| format!("m{i:02}")).collect();
        names.extend(many.iter().map(String::as_str));
        names.push("c");
        let new = section(&names);
        let (cursor, offset) = restore(&anchor, &new, 4);
        let cursor_row = new.iter().position(|k| *k == r("a/c")).unwrap();
        assert_eq!(cursor, 22);
        assert_eq!(cursor_row - offset, 3, "still in the fourth slot");
    }

    #[test]
    fn the_offset_is_clamped_to_the_last_page() {
        let old = section(&["a", "b", "c", "d", "e", "f", "g", "h"]);
        let anchor = view(&old, "h", 5, 4);
        let new = section(&["a", "b", "c", "d", "e"]);
        let (_, offset) = restore(&anchor, &new, 4);
        assert_eq!(offset, new.len() - 4);
    }

    #[test]
    fn an_empty_new_list_resets_to_the_start() {
        let old = section(&["a", "b"]);
        let anchor = view(&old, "b", 1, 4);
        assert_eq!(restore(&anchor, &[], 4), (0, 0));
    }

    #[test]
    fn a_header_at_the_top_stays_the_top() {
        let mut old = section(&["a", "b", "c"]);
        old.push(h("z"));
        old.extend([r("z/x"), r("z/y"), r("z/w")]);
        let anchor = capture(old.clone(), Some("a/b".into()), 4, 4); // top = header z
        let mut new = vec![h("a"), r("a/a"), r("a/b"), r("a/c"), r("a/d"), r("a/e")];
        new.extend([h("z"), r("z/x"), r("z/y"), r("z/w")]);
        let (_, offset) = restore(&anchor, &new, 4);
        assert_eq!(new[offset], h("z"));
    }
}
