//! Where the Diff view's before / after panes go (plan D6, pure).
//!
//! ```text
//! header                                                    (1 row)
//! Before . parent        |  After . abc1234                 (captions)
//! +--------------------+ | +------------------------------+
//! |  picture           |b| |  picture                     |
//! +--------------------+ | +------------------------------+
//!                       ^ ^ the before pane's scrollbar column, then the divider
//! ```
//!
//! Split puts the two pictures side by side with a one-column divider and a
//! one-column gutter that holds the before pane's vertical scrollbar. Inline
//! shows one picture under a caption that says which side it is. A file with
//! only one side (added, deleted) has one pane and no toggle. When the split
//! would leave a pane under [`MIN_SPLIT_PANE`] columns the layout is inline
//! whatever was asked: the setting is not changed (`narrow` says so).

use ratatui::layout::Rect;

use crate::pane_split;

/// The narrowest a split pane may be before the view falls back to inline.
pub(crate) const MIN_SPLIT_PANE: u16 = 24;

/// Which of the two pictures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Side {
    Before,
    After,
}

impl Side {
    pub(crate) fn other(self) -> Side {
        match self {
            Side::Before => Side::After,
            Side::After => Side::Before,
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Side::Before => 0,
            Side::After => 1,
        }
    }
}

/// How the pictures are arranged this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arrangement {
    /// Before | After.
    Split,
    /// One of the two, flipped by `Tab`.
    Inline,
    /// The file has one side only.
    Single,
}

/// One drawn picture: its side, caption row and picture rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaneBox {
    pub(crate) side: Side,
    pub(crate) caption: Rect,
    pub(crate) picture: Rect,
}

impl PaneBox {
    /// A rectangle whose border ring is the one just outside the picture, so
    /// that `scrollbar::Track::{vertical, horizontal}` (which live on a
    /// bordered block's border) land on the column right of the picture and
    /// the row below it.
    pub(crate) fn scroll_frame(&self) -> Rect {
        let p = self.picture;
        Rect::new(
            p.x.saturating_sub(1),
            p.y.saturating_sub(1),
            p.width + 2,
            p.height + 2,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) header: Rect,
    pub(crate) arrangement: Arrangement,
    /// Split was asked for but the room is too small: inline instead.
    pub(crate) narrow: bool,
    pub(crate) panes: Vec<PaneBox>,
    /// The draggable column between the panes (split only), caption row
    /// included.
    pub(crate) divider: Option<Rect>,
}

/// Lays `inner` (the bordered block's inside) out.
///
/// * `want_split`: the layout setting.
/// * `has`: which sides exist `[before, after]`.
/// * `flip`: the side an inline view shows.
/// * `weights`: the split's shares (`None`: equal).
pub(crate) fn compute(
    inner: Rect,
    want_split: bool,
    has: [bool; 2],
    flip: Side,
    weights: Option<[u16; 2]>,
) -> Layout {
    let header = Rect::new(inner.x, inner.y, inner.width, inner.height.min(1));
    let mut out = Layout {
        header,
        arrangement: Arrangement::Single,
        narrow: false,
        panes: Vec::new(),
        divider: None,
    };
    if inner.height < 3 || inner.width < 3 {
        return out;
    }
    let caption_y = inner.y + 1;
    let pic_y = inner.y + 2;
    let pic_h = inner.height - 2;
    let boxed = |side: Side, x: u16, w: u16| PaneBox {
        side,
        caption: Rect::new(x, caption_y, w, 1),
        picture: Rect::new(x, pic_y, w, pic_h),
    };
    let both = has[0] && has[1];
    if !both {
        let side = if has[0] { Side::Before } else { Side::After };
        out.panes.push(boxed(side, inner.x, inner.width));
        return out;
    }
    let usable = inner.width.saturating_sub(2);
    if want_split && usable < 2 * MIN_SPLIT_PANE {
        out.narrow = true;
    }
    if !want_split || out.narrow {
        out.arrangement = Arrangement::Inline;
        out.panes.push(boxed(flip, inner.x, inner.width));
        return out;
    }
    out.arrangement = Arrangement::Split;
    let w = pane_split::widths(weights.as_ref().map(|w| &w[..]), usable, 2);
    let (w0, w1) = (w[0], w[1]);
    out.panes.push(boxed(Side::Before, inner.x, w0));
    // One column for the before pane's scrollbar, one for the divider.
    let divider_x = inner.x + w0 + 1;
    out.panes.push(boxed(Side::After, divider_x + 1, w1));
    out.divider = Some(Rect::new(divider_x, caption_y, 1, inner.height - 1));
    out
}

/// The share of the split `[before, after]` after moving the divider so the
/// before pane is `left` columns wide, given the widths painted now.
pub(crate) fn drag_to(current: [u16; 2], left: i32) -> Option<[u16; 2]> {
    let next = pane_split::move_divider(&current, 0, left)?;
    let w = pane_split::weights_for(&next);
    Some([w[0], w[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    const INNER: Rect = Rect::new(1, 1, 100, 30);

    fn split(weights: Option<[u16; 2]>) -> Layout {
        compute(INNER, true, [true, true], Side::After, weights)
    }

    #[test]
    fn split_gives_the_panes_the_row_less_the_gutter_and_the_divider() {
        let l = split(None);
        assert_eq!(l.arrangement, Arrangement::Split);
        assert!(!l.narrow);
        let (b, a) = (l.panes[0], l.panes[1]);
        assert_eq!((b.side, a.side), (Side::Before, Side::After));
        assert_eq!(b.picture.width + a.picture.width, 98);
        let div = l.divider.unwrap();
        assert_eq!(div.x, b.picture.right() + 1, "after the scrollbar gutter");
        assert_eq!(a.picture.x, div.right());
        assert_eq!(a.picture.right(), INNER.right());
        // The divider spans the caption row and the pictures.
        assert_eq!(div.y, b.caption.y);
        assert_eq!(div.bottom(), b.picture.bottom());
    }

    #[test]
    fn the_header_is_the_first_row_then_the_caption_then_the_picture() {
        let l = split(None);
        assert_eq!(l.header, Rect::new(1, 1, 100, 1));
        assert_eq!(l.panes[0].caption.y, 2);
        assert_eq!(l.panes[0].picture.y, 3);
        assert_eq!(l.panes[0].picture.height, 28);
    }

    #[test]
    fn weights_move_the_divider() {
        let even = split(None);
        let wide = split(Some([7_000, 3_000]));
        assert!(wide.panes[0].picture.width > even.panes[0].picture.width);
        assert!(wide.divider.unwrap().x > even.divider.unwrap().x);
    }

    #[test]
    fn a_room_too_small_for_two_panes_falls_back_to_inline_and_says_so() {
        let narrow = Rect::new(0, 0, 40, 20);
        let l = compute(narrow, true, [true, true], Side::Before, None);
        assert_eq!(l.arrangement, Arrangement::Inline);
        assert!(l.narrow);
        assert_eq!(l.panes.len(), 1);
        assert_eq!(l.panes[0].side, Side::Before, "the flipped side");
        assert_eq!(l.panes[0].picture.width, 40);
        assert!(l.divider.is_none());
    }

    #[test]
    fn inline_by_choice_is_not_narrow() {
        let l = compute(INNER, false, [true, true], Side::After, None);
        assert_eq!(l.arrangement, Arrangement::Inline);
        assert!(!l.narrow);
    }

    #[test]
    fn a_file_with_one_side_has_one_full_width_pane() {
        let added = compute(INNER, true, [false, true], Side::Before, None);
        assert_eq!(added.arrangement, Arrangement::Single);
        assert_eq!(added.panes.len(), 1);
        assert_eq!(added.panes[0].side, Side::After);
        assert_eq!(added.panes[0].picture.width, 100);
        let deleted = compute(INNER, false, [true, false], Side::After, None);
        assert_eq!(deleted.panes[0].side, Side::Before);
        assert!(!added.narrow && added.divider.is_none());
    }

    #[test]
    fn the_scroll_frame_puts_the_bars_on_the_ring_around_the_picture() {
        let l = split(None);
        let b = l.panes[0];
        let f = b.scroll_frame();
        let v = crate::scrollbar::Track::vertical(f, 1000, 0).unwrap();
        assert_eq!(v.rect.x, b.picture.right(), "the gutter column");
        assert_eq!((v.rect.y, v.rect.height), (b.picture.y, b.picture.height));
        let h = crate::scrollbar::Track::horizontal(f, 1000, 0).unwrap();
        assert_eq!(h.rect.y, b.picture.bottom());
        assert_eq!((h.rect.x, h.rect.width), (b.picture.x, b.picture.width));
        // The after pane's bars are the block's own border.
        let a = l.panes[1];
        let v = crate::scrollbar::Track::vertical(a.scroll_frame(), 1000, 0).unwrap();
        assert_eq!(v.rect.x, INNER.right());
    }

    #[test]
    fn dragging_clamps_to_the_minimum_pane_and_keeps_the_total() {
        let cur = [49, 49];
        assert_eq!(drag_to(cur, 1).map(|w| w[0] < w[1]), Some(true));
        let far = drag_to(cur, 10_000).unwrap();
        let widths = pane_split::widths(Some(&far[..]), 98, 2);
        assert_eq!(widths[1], pane_split::MIN_PANE_W);
        assert_eq!(widths.iter().sum::<u16>(), 98);
    }

    #[test]
    fn a_degenerate_area_draws_nothing() {
        let l = compute(
            Rect::new(0, 0, 50, 2),
            true,
            [true, true],
            Side::After,
            None,
        );
        assert!(l.panes.is_empty());
    }
}
