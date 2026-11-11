//! The maths of a row of resizable panes: how a row's width is shared by
//! weights, and how a drag or a key moves the divider between two of them.
//! Shared by the merge editor (`merge_view::layout::ColSplit`) and the image
//! view's before / after split (`image_view::split`), so both drag, step and
//! clamp the same way.
//!
//! A weight is a share of 10 000. `None` weights mean an equal share. The
//! functions are pure: the callers hold the weights as session state (a
//! ratio, so a resize of the terminal keeps the proportions) and paint what
//! these return.

/// The smallest a pane may be dragged to: its gutter and some content.
pub(crate) const MIN_PANE_W: u16 = 14;
/// What the keyboard moves a divider by, in per mille of the panes' width.
pub(crate) const KEY_STEP_PERMILLE: u32 = 50;

/// The widths (in columns) of `n` panes sharing `usable` columns, left to
/// right, summing to `usable`. Every pane keeps [`MIN_PANE_W`] when the room
/// allows it; a narrower row falls back to the equal share.
pub(crate) fn widths(weights: Option<&[u16]>, usable: u16, n: usize) -> Vec<u16> {
    widths_min(weights, usable, n, MIN_PANE_W)
}

/// [`widths`] with another floor per pane: the rows of a column of panes
/// (a few lines each) share a height the same way.
pub(crate) fn widths_min(
    weights: Option<&[u16]>,
    usable: u16,
    n: usize,
    min_pane: u16,
) -> Vec<u16> {
    let n16 = n.max(1) as u16;
    let equal = |usable: u16| -> Vec<u16> {
        let each = usable / n16;
        let mut extra = usable % n16;
        (0..n16)
            .map(|_| {
                let w = each + u16::from(extra > 0);
                extra = extra.saturating_sub(1);
                w
            })
            .collect()
    };
    let Some(weights) = weights.filter(|w| w.len() == n) else {
        return equal(usable);
    };
    if usable < n16 * min_pane {
        return equal(usable);
    }
    let ws: Vec<u32> = weights.iter().map(|w| u32::from(*w).max(1)).collect();
    let total: u32 = ws.iter().sum();
    // Rounded cumulative edges, so a drag lands on the cell it was dropped on.
    let mut out: Vec<u16> = Vec::new();
    let (mut cum, mut prev) = (0u32, 0u32);
    for w in &ws {
        cum += w;
        let edge = (u32::from(usable) * cum * 2 + total) / (total * 2);
        out.push(((edge - prev) as u16).max(min_pane));
        prev = edge;
    }
    // Rounding and the floors: settle on exactly `usable`.
    loop {
        let sum: u16 = out.iter().sum();
        if sum == usable {
            break;
        }
        if sum < usable {
            // Give the rest to the widest pane.
            let i = (0..out.len()).max_by_key(|i| out[*i]).unwrap_or(0);
            out[i] += usable - sum;
        } else {
            let i = (0..out.len()).max_by_key(|i| out[*i]).unwrap_or(0);
            let take = (sum - usable).min(out[i].saturating_sub(min_pane));
            if take == 0 {
                return equal(usable);
            }
            out[i] -= take;
        }
    }
    out
}

/// The weights that give the panes `widths` (what a drag or a key settles on).
pub(crate) fn weights_for(widths: &[u16]) -> Vec<u16> {
    let total: u32 = widths.iter().map(|w| u32::from(*w)).sum::<u32>().max(1);
    widths
        .iter()
        .map(|w| (u32::from(*w) * 10_000 / total).max(1) as u16)
        .collect()
}

/// The widths after moving divider `i` (between pane `i` and `i + 1`) so that
/// pane `i` is `left` columns wide, the pair keeping its total and each pane
/// at least [`MIN_PANE_W`]. `None` when the move is not possible (no such
/// divider, or the pair is too narrow to hold two minimum panes).
pub(crate) fn move_divider(current: &[u16], i: usize, left: i32) -> Option<Vec<u16>> {
    move_divider_min(current, i, left, MIN_PANE_W)
}

/// [`move_divider`] with another floor per pane.
pub(crate) fn move_divider_min(
    current: &[u16],
    i: usize,
    left: i32,
    min_pane: u16,
) -> Option<Vec<u16>> {
    if i + 1 >= current.len() {
        return None;
    }
    let pair = i32::from(current[i]) + i32::from(current[i + 1]);
    let min = i32::from(min_pane);
    if pair < 2 * min {
        return None;
    }
    let left = left.clamp(min, pair - min);
    let mut next = current.to_vec();
    next[i] = left as u16;
    next[i + 1] = (pair - left) as u16;
    Some(next)
}

#[cfg_attr(not(feature = "image-view"), allow(dead_code))]
/// How many columns a key step moves a divider by, for panes `usable`
/// columns wide in all: [`KEY_STEP_PERMILLE`] of it, at least 2.
pub(crate) fn key_step(usable: u16) -> i32 {
    ((u32::from(usable) * KEY_STEP_PERMILLE / 1000).max(2)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_weights_share_the_row_equally_and_exactly() {
        assert_eq!(widths(None, 99, 2), vec![50, 49]);
        assert_eq!(widths(None, 100, 3), vec![34, 33, 33]);
    }

    #[test]
    fn weights_share_the_row_in_proportion_and_sum_to_the_usable_width() {
        let w = widths(Some(&[7_500, 2_500]), 80, 2);
        assert_eq!(w, vec![60, 20]);
        for usable in [28, 29, 57, 100, 201] {
            let w = widths(Some(&[9_000, 1_000]), usable, 2);
            assert_eq!(w.iter().sum::<u16>(), usable, "{w:?}");
            assert!(w.iter().all(|x| *x >= MIN_PANE_W), "{w:?}");
        }
    }

    #[test]
    fn a_row_too_narrow_for_the_minimums_falls_back_to_the_equal_share() {
        assert_eq!(widths(Some(&[9_000, 1_000]), 20, 2), vec![10, 10]);
    }

    #[test]
    fn weights_of_the_wrong_length_are_ignored() {
        assert_eq!(widths(Some(&[1, 2, 3]), 60, 2), vec![30, 30]);
    }

    #[test]
    fn dragging_a_divider_keeps_the_pair_total_and_the_minimums() {
        let cur = [50, 49];
        let far_left = move_divider(&cur, 0, -40).unwrap();
        assert_eq!(far_left, vec![MIN_PANE_W, 99 - MIN_PANE_W]);
        let far_right = move_divider(&cur, 0, 500).unwrap();
        assert_eq!(far_right, vec![99 - MIN_PANE_W, MIN_PANE_W]);
        assert_eq!(move_divider(&cur, 0, 30).unwrap(), vec![30, 69]);
        assert_eq!(move_divider(&cur, 1, 30), None, "no such divider");
        assert_eq!(move_divider(&[10, 10], 0, 12), None, "pair too narrow");
    }

    #[test]
    fn a_middle_divider_only_trades_with_its_neighbor() {
        let w = move_divider(&[34, 33, 33], 1, 20).unwrap();
        assert_eq!(w, vec![34, 20, 46]);
    }

    #[test]
    fn the_weights_of_widths_round_trip_through_the_widths() {
        let w = weights_for(&[60, 20]);
        assert_eq!(widths(Some(&w), 80, 2), vec![60, 20]);
    }

    #[test]
    fn a_key_step_is_five_percent_and_never_under_two_columns() {
        assert_eq!(key_step(200), 10);
        assert_eq!(key_step(10), 2);
    }
}
