use super::model::{hunk_at, DiffRender, DiffRow, RowChange};
use super::{DiffHighlight, DiffStageAction, DiffViewState};
use crate::th;
use gitpoulpi_i18n::msg::diff_view as msg;

use gitpoulpi_core::domain::HunkId;
use ratatui::prelude::*;

use crate::text_select::{self, LineSource};

/// Draws the per-hunk `Stage Hunk` / `Unstage Hunk` button at the right
/// end of a hunk header row and returns its `(x_offset, width)` from the
/// content's left edge, so the caller registers the hit rect from the
/// walk that painted it.
///
/// Returns `None` (and paints nothing) when the pane is too narrow: a
/// control drawn past the right edge is invisible yet clickable.
///
/// `lines` is the slice of rendered rows being drawn and `row` an index
/// into it.
#[cfg(test)]
pub(crate) fn hunk_button(
    lines: &mut [Line<'static>],
    row: usize,
    width: u16,
    label: &str,
) -> Option<(u16, u16)> {
    row_buttons(lines, row, width, &[(label, th().ui.accent)], false, None).map(|b| b[0])
}

/// Label of the Unstaged half's second hunk button.
pub(crate) fn discard_hunk_label() -> &'static str {
    gitpoulpi_i18n::msg::diff_actions::HUNK_DISCARD.get()
}
/// The per-row buttons revealed on a hovered changed row: short, because
/// they sit over the row's code.
pub(crate) fn line_stage_label() -> String {
    format!("+ {}", gitpoulpi_i18n::msg::status_pane::FA_STAGE.get())
}
pub(crate) fn line_unstage_label() -> String {
    format!(
        "\u{2212} {}",
        gitpoulpi_i18n::msg::status_pane::FA_UNSTAGE.get()
    )
}
pub(crate) fn line_discard_label() -> String {
    format!("\u{2717} {}", msg::LINE_DISCARD.get())
}

/// Paints `buttons` (label, color) side by side, one blank column apart,
/// ending at the right edge of row `row`, and returns each one's
/// `(x_offset, width)` in the same order (`hunk_button`'s contract for
/// several buttons).
///
/// `overlay` decides what happens when the row's text leaves no room: a
/// header row (`false`) gets no buttons; a changed row (`true`) has its
/// text cut short under them, since the buttons show only while the
/// pointer is on that row.
pub(crate) fn row_buttons(
    lines: &mut [Line<'static>],
    row: usize,
    width: u16,
    buttons: &[(&str, Color)],
    overlay: bool,
    lit: Option<usize>,
) -> Option<Vec<(u16, u16)>> {
    let line = lines.get_mut(row)?;
    let widths: Vec<usize> = buttons
        .iter()
        .map(|(l, _)| crate::display_width(l) + 2)
        .collect();
    let total = widths.iter().sum::<usize>() + widths.len().saturating_sub(1);
    let width = width as usize;
    let mut head: usize = line
        .spans
        .iter()
        .map(|s| crate::display_width(&s.content))
        .sum();
    if head + total + 1 > width {
        if !overlay || total + 1 > width {
            return None;
        }
        truncate_spans(line, width - total - 1);
        head = width - total - 1;
    }
    let start = width - total;
    line.spans.push(Span::raw(" ".repeat(start - head)));
    let mut out = Vec::with_capacity(buttons.len());
    let mut x = start;
    for (i, ((label, fg), bw)) in buttons.iter().zip(&widths).enumerate() {
        if i > 0 {
            line.spans.push(Span::raw(" "));
            x += 1;
        }
        line.spans.push(Span::styled(
            format!(" {label} "),
            body_button_style(*fg, lit == Some(i)),
        ));
        out.push((x as u16, *bw as u16));
        x += bw;
    }
    Some(out)
}

/// Cuts a line's spans down to `max` characters, styles kept.
fn truncate_spans(line: &mut Line<'static>, max: usize) {
    let mut used = 0;
    let mut kept = Vec::with_capacity(line.spans.len());
    for span in line.spans.drain(..) {
        let n = crate::display_width(&span.content);
        if used + n <= max {
            used += n;
            kept.push(span);
        } else {
            let take = max - used;
            if take > 0 {
                let text = crate::take_width(&span.content, take);
                kept.push(Span::styled(text, span.style));
            }
            break;
        }
    }
    line.spans = kept;
}

/// Paints the keyboard cursor and, when one is active, the line
/// selection, as full-width bands.
///
/// A post-pass, so `render_diff` keeps its pure `FileDiff -> rows` shape.
/// Every span of a highlighted row has its background replaced,
/// including the `ADD_BG`/`DEL_BG` tints, which are too close to the
/// terminal background to read as selected.
///
/// `lines` is the drawn window, starting at absolute row `first_row`;
/// `cursor` and `selection` are absolute rows. Only the window is
/// painted: the whole-file layout can be tens of thousands of rows.
pub(crate) fn apply_cursor(
    lines: &mut [Line<'static>],
    first_row: usize,
    width: u16,
    cursor: usize,
    selection: Option<(usize, usize)>,
) {
    let window = first_row..first_row + lines.len();
    let paint = |line: &mut Line<'static>, bg: Color| paint_band(line, width, bg);
    if let Some((from, to)) = selection {
        let from = from.max(window.start);
        let to = to.min(window.end.saturating_sub(1));
        for row in from..=to {
            if let Some(line) = lines.get_mut(row - window.start) {
                paint(line, th().diff.range_bg);
            }
        }
    }
    if window.contains(&cursor) {
        if let Some(line) = lines.get_mut(cursor - window.start) {
            paint(line, th().ui.selection);
        }
    }
}

/// Recolors every span of `line` to `bg` and pads it to `width`, so the
/// band runs the whole row instead of ending where the text does.
pub(crate) fn paint_band(line: &mut Line<'static>, width: u16, bg: Color) {
    let used: usize = line
        .spans
        .iter()
        .map(|s| crate::display_width(&s.content))
        .sum();
    for span in &mut line.spans {
        span.style = span.style.bg(bg);
    }
    if (width as usize) > used {
        line.spans.push(Span::styled(
            " ".repeat(width as usize - used),
            Style::default().bg(bg),
        ));
    }
}

/// Paints the agent's highlight band over the drawn window (`lines` start
/// at absolute row `first_row`), over the cursor's band so it stays
/// visible there.
pub(crate) fn apply_highlight(
    lines: &mut [Line<'static>],
    first_row: usize,
    width: u16,
    rows: &[DiffRow],
    highlight: Option<DiffHighlight>,
) {
    let Some(highlight) = highlight else { return };
    for (i, line) in lines.iter_mut().enumerate() {
        if rows
            .get(first_row + i)
            .is_some_and(|row| highlight.covers(row))
        {
            paint_band(line, width, th().diff.search_hit_bg);
        }
    }
}

/// A control drawn inside the Diff view's body, registered as
/// `HitTarget::DiffBodyButton` from the walk that painted it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffBodyButton {
    /// `Stage hunk` / `Unstage hunk` in a hunk header, whichever the open
    /// half means (like `DiffStageAction::Hunk`).
    Hunk(HunkId),
    /// `Discard hunk` in a hunk header, Unstaged half only.
    DiscardHunk(HunkId),
    /// `+ Stage` / `− Unstage` on a hovered changed row: that row's
    /// changed lines (two in Split, where a removal and its addition
    /// share a row). Carries the row, like `HitTarget::DiffRow`.
    Line(usize),
    /// `✗ Discard` on a hovered changed row, Unstaged half only.
    DiscardLine(usize),
}

impl DiffBodyButton {
    /// What clicking it does, in `diff_actions::diff_stage_action_at`'s
    /// terms: the action, the hunk it names, and the row range for the
    /// line-level ones.
    pub(crate) fn action(self) -> (DiffStageAction, Option<HunkId>, Option<(usize, usize)>) {
        match self {
            DiffBodyButton::Hunk(h) => (DiffStageAction::Hunk, Some(h), None),
            DiffBodyButton::DiscardHunk(h) => (DiffStageAction::DiscardHunk, Some(h), None),
            DiffBodyButton::Line(r) => (DiffStageAction::Lines, None, Some((r, r))),
            DiffBodyButton::DiscardLine(r) => (DiffStageAction::DiscardLines, None, Some((r, r))),
        }
    }
}

/// The rows the Diff view draws this frame: a copy of the visible window
/// of the cached render, with the body's buttons and the cursor/selection
/// bands painted on it, plus each button's hit rect.
///
/// Two kinds of button, only on the two working-tree halves (the only
/// diffs a hunk id can be resolved against):
///
/// - the hunk header's `Stage hunk` / `Unstage hunk` (and, Unstaged half,
///   `Discard hunk`) on the hunk the mouse is in, else the one the cursor
///   is in. The header is found through `hunk_starts`, not by scanning
///   every row of a whole-file layout each frame.
/// - on the changed row under the mouse (never the keyboard cursor: the
///   keys already act on it), `+ Stage` / `− Unstage` and, Unstaged half
///   only, `✗ Discard`.
///
/// `inner` is the diff block's inner rect; `state.scroll`/`cursor` have
/// already been clamped against `render`.
pub(crate) fn visible_window(
    render: &DiffRender,
    state: &DiffViewState,
    inner: Rect,
    hovered_row: Option<usize>,
    hovered_button: Option<DiffBodyButton>,
) -> (Vec<Line<'static>>, Vec<(DiffBodyButton, Rect)>) {
    let first = state.scroll.min(render.lines.len());
    let last = (first + inner.height as usize).min(render.lines.len());
    let mut lines: Vec<Line<'static>> = render.lines[first..last].to_vec();

    let mut buttons = Vec::new();
    if let Some(label) = state
        .source
        .hunk_label()
        .filter(|_| state.encoding.is_none())
    {
        let discards = state.source.discards();
        let hunk_of = |row: usize| render.rows.get(row).and_then(|r| r.hunk);
        let target = hovered_row
            .and_then(hunk_of)
            .or_else(|| hunk_of(state.cursor));
        let header_row = target.and_then(|h| {
            render
                .hunk_starts
                .iter()
                .copied()
                .find(|&s| render.rows.get(s).is_some_and(|r| r.hunk == Some(h)))
        });
        if let (Some(header_row), Some(hunk)) = (header_row, target) {
            if (first..last).contains(&header_row) {
                // Discard first, the everyday Stage at the right edge
                // (the order the buttons keep everywhere).
                let mut row: Vec<(&str, Color, DiffBodyButton)> = Vec::new();
                if discards {
                    row.push((
                        discard_hunk_label(),
                        th().status.danger,
                        DiffBodyButton::DiscardHunk(hunk),
                    ));
                }
                row.push((label, th().ui.accent, DiffBodyButton::Hunk(hunk)));
                paint_buttons(
                    &mut lines,
                    header_row - first,
                    inner,
                    &row,
                    false,
                    hovered_button,
                    &mut buttons,
                );
            }
        }
        let changed = |r: usize| {
            render
                .rows
                .get(r)
                .is_some_and(|d| !d.header && d.lines.iter().any(Option::is_some))
        };
        if let Some(r) = hovered_row
            .filter(|&r| (first..last).contains(&r) && changed(r) && state.encoding.is_none())
        {
            let stage = if state.source.staging() == Some(true) {
                line_stage_label()
            } else {
                line_unstage_label()
            };
            let discard_line = line_discard_label();
            let mut row: Vec<(&str, Color, DiffBodyButton)> = Vec::new();
            if discards {
                row.push((
                    &discard_line,
                    th().status.danger,
                    DiffBodyButton::DiscardLine(r),
                ));
            }
            row.push((&stage, th().ui.accent, DiffBodyButton::Line(r)));
            paint_buttons(
                &mut lines,
                r - first,
                inner,
                &row,
                true,
                hovered_button,
                &mut buttons,
            );
        }
    }

    apply_cursor(
        &mut lines,
        first,
        inner.width,
        state.cursor,
        state
            .selection_anchor
            .map(|a| (a.min(state.cursor), a.max(state.cursor))),
    );
    apply_highlight(
        &mut lines,
        first,
        inner.width,
        &render.rows,
        state.highlight,
    );
    // The bands recolored the cursor row's buttons too: restore their
    // own (hover-aware) fill.
    for (button, rect) in &buttons {
        let fg = match button {
            DiffBodyButton::DiscardHunk(_) | DiffBodyButton::DiscardLine(_) => th().status.danger,
            DiffBodyButton::Hunk(_) | DiffBodyButton::Line(_) => th().ui.accent,
        };
        if let Some(line) = lines.get_mut((rect.y - inner.y) as usize) {
            restyle_cells(
                line,
                rect.x - inner.x,
                rect.width,
                body_button_style(fg, hovered_button == Some(*button)),
            );
        }
    }
    (lines, buttons)
}

/// A body button's style: its color on the hover tint at rest, filled
/// with that color under the pointer.
fn body_button_style(fg: Color, lit: bool) -> Style {
    if lit {
        Style::default()
            .fg(th().ui.on_accent)
            .bg(fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(fg).bg(th().ui.hover)
    }
}

/// Sets `style` on the spans exactly covering columns `x..x + w` of `line`
/// (the button spans `row_buttons` pushed).
fn restyle_cells(line: &mut Line<'static>, x: u16, w: u16, style: Style) {
    let mut at = 0u16;
    for span in &mut line.spans {
        let n = crate::display_width(&span.content) as u16;
        if at >= x && at + n <= x + w && n > 0 {
            span.style = style;
        }
        at += n;
    }
}

/// `row_buttons` for `visible_window`: paints one row's buttons and turns
/// each reported offset into a screen rect.
fn paint_buttons(
    lines: &mut [Line<'static>],
    window_row: usize,
    inner: Rect,
    row: &[(&str, Color, DiffBodyButton)],
    overlay: bool,
    hovered: Option<DiffBodyButton>,
    out: &mut Vec<(DiffBodyButton, Rect)>,
) {
    let labels: Vec<(&str, Color)> = row.iter().map(|(l, c, _)| (*l, *c)).collect();
    if let Some(placed) = row_buttons(
        lines,
        window_row,
        inner.width,
        &labels,
        overlay,
        hovered.and_then(|h| row.iter().position(|(_, _, b)| *b == h)),
    ) {
        let y = inner.y + window_row as u16;
        for ((dx, bw), (_, _, button)) in placed.into_iter().zip(row) {
            out.push((*button, Rect::new(inner.x + dx, y, bw, 1)));
        }
    }
}

/// Paints the mouse text selection over the diff rows already drawn into
/// `buf`: a buffer post-pass like the scrollbars, leaving the cached
/// render and `visible_window`'s bands untouched. `inner` is the diff
/// block's inner rect and `state.scroll` has been clamped against
/// `render`.
pub(crate) fn paint_text_selection(
    buf: &mut ratatui::buffer::Buffer,
    inner: Rect,
    render: &DiffRender,
    state: &DiffViewState,
) {
    let Some(sel) = state.text_sel else {
        return;
    };
    let first = state.scroll.min(render.segs.len());
    let last = (first + inner.height as usize).min(render.segs.len());
    let src = render.text.side(sel.side);
    // Every occurrence of the selected text, softly (the ruler's needle).
    text_select::paint_occurrences(
        buf,
        inner,
        &render.segs[first..last],
        sel.side,
        &text_select::selected_text(&sel, &src),
        &src,
    );
    text_select::paint(buf, inner, &render.segs[first..last], &sel, &src);
}

/// One tick per changed row: what the overview ruler shows of the diff
/// itself, for the Diff view and the review's diff pane alike.
pub(crate) fn change_marks(render: &DiffRender) -> Vec<(usize, crate::scroll_marks::Mark)> {
    use crate::scroll_marks::Mark;
    render
        .rows
        .iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let mark = match row.change {
                RowChange::None => return None,
                RowChange::Add => Mark::Add,
                RowChange::Del => Mark::Del,
                RowChange::Both => Mark::Modified,
            };
            Some((i, mark))
        })
        .collect()
}

/// The ticks of the vertical scrollbar's overview ruler: every changed
/// row, the cursor row, and every row showing the text the mouse
/// selection covers (see `scroll_marks`). Only whole-file mode gets one;
/// in hunks-only mode every row is near a change.
pub(crate) fn ruler_marks(
    render: &DiffRender,
    state: &DiffViewState,
) -> Vec<(usize, crate::scroll_marks::Mark)> {
    use crate::scroll_marks::{occurrences, Mark};
    if state.hunks_only {
        return Vec::new();
    }
    let mut marks = change_marks(render);
    if let Some(sel) = state.text_sel {
        let src = render.text.side(sel.side);
        let needle = text_select::selected_text(&sel, &src);
        let lines = &render.text.sides[sel.side.min(1)];
        // Logical line -> the first screen row that draws it.
        let mut first_row = vec![usize::MAX; lines.len()];
        for (row, segs) in render.segs.iter().enumerate() {
            if let Some(seg) = segs[sel.side.min(1)] {
                if let Some(slot) = first_row.get_mut(seg.line) {
                    *slot = (*slot).min(row);
                }
            }
        }
        for line in occurrences(&needle, lines.iter().map(String::as_str)) {
            if let Some(&row) = first_row.get(line) {
                if row != usize::MAX {
                    marks.push((row, Mark::Occurrence));
                }
            }
        }
    }
    marks.push((state.cursor, Mark::Cursor));
    marks
}

/// The text position under screen cell `(col, row)` of the diff body on
/// `side`, the row clamped into the drawn rows so a drag past the top or
/// bottom keeps extending to the edge row. `inner` is the diff block's
/// inner rect.
pub(crate) fn text_pos_at(
    render: &DiffRender,
    inner: Rect,
    scroll: usize,
    side: usize,
    col: u16,
    row: u16,
) -> Option<text_select::TextPos> {
    if inner.height == 0 || render.segs.is_empty() {
        return None;
    }
    let row = row.clamp(inner.y, inner.y + inner.height - 1);
    let index = (scroll + (row - inner.y) as usize).min(render.segs.len() - 1);
    text_select::pos_at(
        &render.segs,
        index,
        side,
        col.saturating_sub(inner.x),
        &render.text.side(side),
    )
}

/// Which side of the diff screen column `col` is on.
pub(crate) fn text_side_at(render: &DiffRender, inner: Rect, col: u16) -> usize {
    text_select::side_at(render.split_x, col.saturating_sub(inner.x))
}

/// What a copy from the Diff view takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffCopy {
    /// The mouse text selection.
    Selection,
    /// The line under the cursor, or the lines a `v` selection covers.
    Line,
    /// Every line of the hunk under the cursor.
    Hunk,
}

/// The text a copy takes, with the words the status line names it by, or
/// why there is nothing to copy. It comes from the logical lines
/// (`DiffRender::text`), so decorations are never part of it.
///
/// Inline takes every line in the range, `-` and `+` alike, in order.
/// Split takes the after column (the file as the change leaves it),
/// falling back to the before column only when the range has no
/// after-side line (a pure deletion).
pub(crate) fn copy_text(
    render: &DiffRender,
    state: &DiffViewState,
    what: DiffCopy,
) -> Result<(&'static str, String), &'static str> {
    match what {
        DiffCopy::Selection => {
            let sel = state.text_sel.ok_or(msg::NO_TEXT_SELECTED.get())?;
            Ok((
                msg::WHAT_SELECTION.get(),
                text_select::selected_text(&sel, &render.text.side(sel.side)),
            ))
        }
        DiffCopy::Line => {
            let (from, to) = state.selected_rows();
            let what = if from == to {
                msg::WHAT_LINE.get()
            } else {
                msg::WHAT_LINES.get()
            };
            rows_text(render, from, to)
                .map(|t| (what, t))
                .ok_or(msg::NO_TEXT_ROW.get())
        }
        DiffCopy::Hunk => {
            let hunk = hunk_at(&render.rows, state.cursor)
                .ok_or(gitpoulpi_i18n::msg::diff_actions::NO_HUNK.get())?;
            let from = render.rows.iter().position(|r| r.hunk == Some(hunk));
            let to = render.rows.iter().rposition(|r| r.hunk == Some(hunk));
            match (from, to) {
                (Some(from), Some(to)) => rows_text(render, from, to)
                    .map(|t| (msg::WHAT_HUNK.get(), t))
                    .ok_or(msg::NO_TEXT_HUNK.get()),
                _ => Err(gitpoulpi_i18n::msg::diff_actions::NO_HUNK.get()),
            }
        }
    }
}

/// `y` / `Ctrl+C`: the text selection when there is one, else the line
/// (or `v` line range) under the cursor.
pub(crate) fn yank(
    render: &DiffRender,
    state: &DiffViewState,
) -> Result<(&'static str, String), &'static str> {
    let what = if state.text_sel.is_some() {
        DiffCopy::Selection
    } else {
        DiffCopy::Line
    };
    copy_text(render, state, what)
}

/// The whole logical lines rows `from..=to` show (see `copy_text` for
/// which side), each once even when word wrap spread it over rows.
fn rows_text(render: &DiffRender, from: usize, to: usize) -> Option<String> {
    let segs = render
        .segs
        .get(from..=to.min(render.segs.len().checked_sub(1)?))?;
    let lines_on = |side: usize| {
        let mut lines: Vec<usize> = Vec::new();
        for seg in segs.iter().filter_map(|s| s[side]) {
            if lines.last() != Some(&seg.line) {
                lines.push(seg.line);
            }
        }
        lines
    };
    let after = lines_on(1);
    let (side, lines) = if after.is_empty() {
        (0, lines_on(0))
    } else {
        (1, after)
    };
    if lines.is_empty() {
        return None;
    }
    let src = render.text.side(side);
    Some(text_select::lines_text(&lines, &src as &dyn LineSource))
}

/// Maps a terminal row to an index into the rendered diff rows, or `None`
/// for the block's own border rows and for anything past the end of the
/// diff.
///
/// `body` is the diff view's **outer** rect (the bordered block), so the
/// first content row is `body.y + 1`; forgetting it is the `top_offset`
/// bug of `docs/developer/where-we-are.md` and would stage the hunk below
/// the one clicked.
pub(crate) fn diff_row_at(
    body: Rect,
    scroll: usize,
    rendered_len: usize,
    row: u16,
) -> Option<usize> {
    let top = body.y.checked_add(1)?;
    if row < top || row + 1 >= body.y + body.height {
        return None;
    }
    let index = (row - top) as usize + scroll;
    (index < rendered_len).then_some(index)
}

/// Moves the cursor by `delta` rows, clamped to the diff, and scrolls the
/// minimum amount needed to keep it on screen.
///
/// `view_height` is the inner height (rows of diff drawn): the caller
/// subtracts the block's two border rows.
pub(crate) fn move_cursor(state: &mut DiffViewState, delta: isize, len: usize, view_height: u16) {
    if len == 0 {
        state.cursor = 0;
        return;
    }
    let target = (state.cursor as isize + delta).clamp(0, (len as isize - 1).max(0)) as usize;
    state.cursor = target;
    follow_cursor(state, len, view_height);
}

/// Scrolls just far enough that the cursor row is visible.
pub(crate) fn follow_cursor(state: &mut DiffViewState, len: usize, view_height: u16) {
    let height = view_height.max(1) as usize;
    state.cursor = state.cursor.min(len.saturating_sub(1));
    if state.cursor < state.scroll {
        state.scroll = state.cursor;
    } else if state.cursor >= state.scroll + height {
        state.scroll = state.cursor + 1 - height;
    }
}
