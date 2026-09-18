//! Where the Blame view's columns sit, as a pure function of the pane's
//! width: the line numbers, the code, the annotation column and the three
//! one-cell strips between them. The painter, the hit rects and the text
//! selection all read this one answer.

use ratatui::layout::Rect;

/// The annotation column is never narrower than this when it is shown;
/// below it only the heat and author bars remain (the card via `i` still
/// gives the details).
pub(crate) const MIN_ANN: u16 = 12;
/// Narrower than this the annotation switches to its compact form: one line
/// per hunk start (avatar, author, age, then the summary if there is room).
pub(crate) const FULL_ANN: u16 = 30;
/// The code keeps at least this many cells when the annotation is shown.
pub(crate) const MIN_TEXT: u16 = 40;
/// Display-only share of the available width the annotation may take, in
/// percent. The user's width (`ui.blame_width`) is never rewritten.
pub(crate) const MAX_SHARE: u16 = 40;
/// Cells between the code and the annotation: the border (resize handle),
/// the age heat strip and the author bar.
pub(crate) const STRIP: u16 = 3;
/// Narrower than this (the pane's body) the Blame view draws no minimap: it
/// would take its columns from the code.
pub(crate) const MIN_MAP_BODY: u16 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) num_x: u16,
    pub(crate) num_w: u16,
    pub(crate) text_x: u16,
    pub(crate) text_w: u16,
    /// The border, a one-cell resize handle. Meaningless when `ann_w` is 0.
    pub(crate) handle_x: u16,
    pub(crate) heat_x: u16,
    pub(crate) bar_x: u16,
    pub(crate) ann_x: u16,
    /// 0 when the pane is too narrow to show annotations at all.
    pub(crate) ann_w: u16,
}

/// The widest the annotation column can be in a pane `width` wide with
/// `lines` file lines (the number gutter's width), `want` being the user's
/// width: capped to a share of the pane and so that the code keeps
/// [`MIN_TEXT`] cells. 0 when less than [`MIN_ANN`] is left.
pub(crate) fn effective_ann(width: u16, lines: usize, want: u16) -> u16 {
    let fixed = digits(lines) + 1 + STRIP;
    let room = width.saturating_sub(fixed);
    let share = room * MAX_SHARE / 100;
    let eff = want.min(share).min(room.saturating_sub(MIN_TEXT));
    if eff >= MIN_ANN {
        eff
    } else {
        0
    }
}

pub(crate) fn digits(n: usize) -> u16 {
    n.max(1).to_string().len() as u16
}

/// The columns of `inner`, the annotation `want` cells wide (clamped to
/// what fits) on the left or the right.
pub(crate) fn layout(inner: Rect, lines: usize, left: bool, want: u16) -> Layout {
    let num_w = digits(lines);
    let ann_w = effective_ann(inner.width, lines, want);
    let text_w = inner.width.saturating_sub(num_w + 1 + STRIP + ann_w);
    if left {
        let ann_x = inner.x;
        let bar_x = ann_x + ann_w;
        let heat_x = bar_x + 1;
        let handle_x = heat_x + 1;
        let num_x = handle_x + 1;
        Layout {
            num_x,
            num_w,
            text_x: num_x + num_w + 1,
            text_w,
            handle_x,
            heat_x,
            bar_x,
            ann_x,
            ann_w,
        }
    } else {
        let num_x = inner.x;
        let text_x = num_x + num_w + 1;
        let handle_x = text_x + text_w;
        let heat_x = handle_x + 1;
        let bar_x = heat_x + 1;
        Layout {
            num_x,
            num_w,
            text_x,
            text_w,
            handle_x,
            heat_x,
            bar_x,
            ann_x: bar_x + 1,
            ann_w,
        }
    }
}

/// The annotation width a pointer at column `col` asks for while the
/// border is dragged.
pub(crate) fn width_for_pointer(inner: Rect, left: bool, col: u16) -> u16 {
    if left {
        col.saturating_sub(inner.x + 2)
    } else {
        (inner.x + inner.width).saturating_sub(col + 3)
    }
}
