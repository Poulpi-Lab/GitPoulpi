//! Paints the commit column and returns the hit rects of that same walk.

use gitpoulpi_core::domain::{CommitNode, Oid};
use gitpoulpi_i18n::msg::commit_column as msg;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::picture::{self, ColumnPicture, Widths};
use super::{ColumnHit, CommitColumn, Part};
use crate::chrome::HitTarget;
use crate::graph::{self, LogLayout};
use crate::ui_kit::{clip_spans, clip_to_frame, display_width, fit};
use crate::{th, ClippedRender};

/// What the paint reads, gathered by the caller.
pub(crate) struct ColumnPaint<'a> {
    pub(crate) layout: &'a LogLayout,
    /// The Log's selected commit.
    pub(crate) selected: Option<Oid>,
    pub(crate) hovered: Option<HitTarget>,
    /// The keyboard is on the column.
    pub(crate) focused: bool,
    /// More commits are being loaded, or exist.
    pub(crate) more: bool,
    pub(crate) node_set: graph::NodeSet,
    /// The Log's Changes numbers, as the daemon has answered so far.
    pub(crate) changes: &'a std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
}

/// The date for a part `width` cells wide: the whole `ui.date_format` when it
/// fits, else with a leading year cut (`2026-10-27` -> `10-27`), whatever
/// pattern the user chose.
pub(crate) fn short_date(ts: i64, width: usize) -> String {
    let full = crate::live_settings::format_date(&crate::live_settings::date_short_pattern(), ts);
    if display_width(&full) <= width {
        return full;
    }
    let digits = full.chars().take_while(char::is_ascii_digit).count();
    let year_first = digits == 4
        && full
            .chars()
            .nth(4)
            .is_some_and(|c| matches!(c, '-' | '/' | '.'));
    let shown = if year_first {
        &full[5..]
    } else {
        full.as_str()
    };
    fit(shown, width)
}

/// The author as one short word.
pub(crate) fn short_author(name: &str, width: usize) -> String {
    let first = name.split_whitespace().next().unwrap_or(name);
    fit(first, width)
}

/// `spans` as the `width` cells from cell `skip` on (whole characters; a
/// wide character cut by the left edge leaves a blank).
fn slice_cells(spans: Vec<Span<'static>>, skip: usize, width: usize) -> Vec<Span<'static>> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut at = 0usize;
    let mut used = 0usize;
    for span in spans {
        let mut kept = String::new();
        for ch in span.content.chars() {
            let w = display_width(&ch.to_string());
            if at + w <= skip {
                at += w;
                continue;
            }
            if at < skip {
                // Cut through a wide character: a blank in its place.
                kept.push(' ');
                used += 1;
                at += w;
                continue;
            }
            if used + w > width {
                break;
            }
            kept.push(ch);
            used += w;
            at += w;
        }
        if !kept.is_empty() {
            out.push(Span::styled(kept, span.style));
        }
        if used >= width {
            break;
        }
    }
    out
}

/// Paints the column into `area`: a header row (the parts' names, between
/// them the boundaries a drag resizes) over the rows. Hit rects: the close
/// cross, the header's boundaries, the rows (by commit id), then the
/// splitter along the view's left border.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut CommitColumn,
    paint: &ColumnPaint<'_>,
    picture: Option<&mut ColumnPicture<'_>>,
) -> Vec<(Rect, HitTarget)> {
    let border = if paint.focused {
        th().ui.accent
    } else {
        th().ui.border
    };
    let block = Block::default()
        .title(format!(" {} ", msg::TITLE.get()))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_clipped(block, area);
    state.area = area;
    // The header row, then the rows.
    let header = Rect::new(inner.x, inner.y, inner.width, inner.height.min(1));
    let rows_area = Rect::new(
        inner.x,
        inner.y + header.height,
        inner.width,
        inner.height - header.height,
    );
    state.rows_area = rows_area;
    state.parts.clear();
    state.boundaries.clear();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();

    let close_hot = matches!(
        paint.hovered,
        Some(HitTarget::CommitColumn(ColumnHit::Close))
    );
    if let Some(cross) = crate::chrome::close_cross::paint(frame.buffer_mut(), area, close_hot) {
        hits.push((cross, HitTarget::CommitColumn(ColumnHit::Close)));
    }

    let nodes: &[CommitNode] = paint.layout.nodes.as_deref().unwrap_or(&[]);
    state.len = nodes.len();
    let selected_row = paint
        .selected
        .and_then(|id| nodes.iter().position(|n| n.id == id));
    let mut content_w = 0usize;
    if inner.height > 0 && inner.width > 0 {
        let viewport = rows_area.height as usize;
        state.settle(selected_row, nodes.len(), viewport);
        let dim = Style::default().fg(th().ui.muted);
        if nodes.is_empty() && rows_area.height > 0 {
            frame.render_clipped(
                Paragraph::new(Line::styled(msg::EMPTY.get(), dim)),
                Rect::new(rows_area.x, rows_area.y, rows_area.width, 1),
            );
        }
        let width = inner.width as usize;
        let shown = nodes.len().saturating_sub(state.offset).min(viewport);
        // The lanes take what they need (`picture::plan_widths`), unless
        // the user sized a part (`picture::plan_widths_with`).
        let needed = picture::lanes_needed(paint.layout, state.offset, shown);
        // The Changes part asks the daemon for the numbers of the rows it shows.
        state.visible_ids.clear();
        if state.shown[Part::Changes.slot()] {
            state
                .visible_ids
                .extend(nodes.iter().skip(state.offset).take(viewport).map(|n| n.id));
        }
        let widths = picture::plan_custom(width, needed, state.widths, state.shown, state.order);
        state.plan = widths;
        state.natural = natural_widths(
            nodes.iter().skip(state.offset).take(viewport),
            paint,
            needed,
        );
        content_w = widths.content;
        state.content_w = widths.content;
        state.inner_w = width;
        state.h_offset = state.h_offset.min(state.max_h_offset());
        let h = state.h_offset;
        // Where each part is on screen, and the header's boundaries.
        let screen = |rel: usize, w: usize| -> Option<(u16, u16)> {
            let x0 = rel.max(h);
            let x1 = (rel + w).min(h + width);
            (x0 < x1).then(|| (inner.x + (x0 - h) as u16, inner.x + (x1 - h) as u16))
        };
        let spans = widths.spans();
        for (part, rel, w) in &spans {
            if let Some((x0, x1)) = screen(*rel, *w) {
                state.parts.push((*part, x0, x1));
            }
        }
        // The Log's rule (`column_resize`): a part's boundary is its right
        // edge. `ui.column_resize = "header"`: the grab is the first letter of
        // the next part's header; `"line"`: the separator cell, down the
        // header and every row. Lit while hovered or dragged by
        // `hovered_resize_strip`. The last part has no next letter: its grab is
        // the cell right after it, which is the column's own border when the
        // row fills the room.
        if header.height > 0 {
            let mode = crate::column_resize::mode();
            let bottom = rows_area.y + rows_area.height;
            for (i, (part, rel, w)) in spans.iter().enumerate() {
                let next = spans.get(i + 1);
                let edge = next.map_or(rel + w, |next| next.1);
                let Some(x) = edge.checked_sub(h).filter(|x| *x <= width) else {
                    continue;
                };
                // A separator scrolled off the left has nothing to grab.
                if next.is_some() && x == 0 {
                    continue;
                }
                let sx = inner.x + x as u16;
                let e = if next.is_some() {
                    crate::column_resize::Edge::before(sx)
                } else {
                    crate::column_resize::Edge::after(sx)
                };
                let rect = crate::column_resize::grab_rect(mode, e, header.y, bottom);
                state.boundaries.push((*part, rect));
                hits.push((rect, HitTarget::CommitColumn(ColumnHit::Boundary(*part))));
            }
            // After the boundaries (the first rect that holds the pointer
            // wins): a name drags its column to another place.
            for (part, x0, x1) in &state.parts {
                hits.push((
                    Rect::new(*x0, header.y, x1 - x0, 1),
                    HitTarget::CommitColumn(ColumnHit::Header(*part)),
                ));
            }
            state.header = header;
            let line = header_line(&spans, paint.focused.then_some(state.part));
            let line = slice_cells(line, h, width);
            frame.render_clipped(Paragraph::new(Line::from(line)), header);
            // The resting ticks, in the separators between two parts.
            for (i, (_, rel, w)) in spans.iter().enumerate() {
                if i + 1 < spans.len() {
                    if let Some(x) = (rel + w).checked_sub(h).filter(|x| *x < width) {
                        crate::column_resize::paint_tick(
                            frame.buffer_mut(),
                            crate::column_resize::mode(),
                            inner.x + x as u16,
                            header.y,
                        );
                    }
                }
            }
        }
        // Pixels over blank cells, or characters: the Log's choice. The
        // picture covers the part of the Graph part that is on screen,
        // wherever the user put it: the row scrolled sideways, or a Graph
        // part wider than the room, cuts the lanes at the edge and they stay
        // pixels, with their avatars. Characters only on a terminal with no
        // image protocol.
        let graph_at = spans
            .iter()
            .find(|(p, _, _)| *p == Part::Graph)
            .map_or(0, |(_, rel, _)| *rel);
        let graph_on_screen = if widths.graph > 0 {
            screen(graph_at, widths.graph)
        } else {
            None
        };
        let as_picture = graph_on_screen.is_some() && picture.as_ref().is_some_and(|p| p.draws());
        for (slot, i) in (state.offset..nodes.len()).take(viewport).enumerate() {
            let node = &nodes[i];
            let y = rows_area.y + slot as u16;
            let hovered = paint
                .hovered
                .as_ref()
                .is_some_and(|h| *h == HitTarget::CommitColumn(ColumnHit::Row(node.id)));
            let line = row_line(
                node,
                paint.layout.grid.get(i),
                RowFacts {
                    pills: paint.layout.pills_by_oid.get(&node.id).map(Vec::as_slice),
                    lane_color: paint.layout.rows.get(i).map(|(_, c)| *c),
                    tinted: paint.layout.rows.get(i).map(|(l, _)| l),
                    node_set: paint.node_set,
                    changes: paint.changes.get(&node.id),
                },
                widths,
                !as_picture,
                selected_row == Some(i),
                hovered,
            );
            let line = slice_cells(line, h, width);
            frame.render_clipped(
                Paragraph::new(Line::from(line)),
                Rect::new(rows_area.x, y, rows_area.width, 1),
            );
            hits.push((
                Rect::new(rows_area.x, y, rows_area.width, 1),
                HitTarget::CommitColumn(ColumnHit::Row(node.id)),
            ));
        }
        // After the rows, like the Log's: the picture sits on the blanks
        // the rows just drew for it, and on the rows' own arithmetic
        // (`state.offset` is settled above, the window is the one painted).
        if let (true, Some(picture), Some((x0, x1))) = (as_picture, picture, graph_on_screen) {
            let mut rows = picture.rows_for(
                paint.layout,
                nodes,
                state.offset,
                shown as u16,
                selected_row,
            );
            if graph::is_compact(widths.graph) {
                graph::compact_raster_rows(&mut rows);
            } else {
                // Cells scrolled off at the left are not drawn.
                let cut = h.saturating_sub(graph_at);
                if cut > 0 {
                    for row in &mut rows {
                        row.cells.drain(..cut.min(row.cells.len()));
                    }
                }
            }
            // The band runs through the spacer that follows the Graph part
            // (the boundary cell), so no plain cell sits between it and the
            // next part, as in the Log.
            let spacer = u16::from(
                spans.last().is_some_and(|(p, _, _)| *p != Part::Graph) && x1 < inner.right(),
            );
            let at = clip_to_frame(
                frame.area(),
                Rect::new(x0, rows_area.y, x1 - x0 + spacer, shown as u16),
            );
            picture.image.render(frame.buffer_mut(), at, &rows);
        }
        // The Log's lazy loading is asked from the frame; the column only
        // says there is more behind the last row.
        let shown_end = (state.offset + viewport).min(nodes.len());
        if paint.more && shown_end == nodes.len() && slot_free(rows_area, nodes.len(), state.offset)
        {
            let y = rows_area.y + (nodes.len() - state.offset) as u16;
            frame.render_clipped(
                Paragraph::new(Line::styled(msg::LOADING_MORE.get(), dim)),
                Rect::new(rows_area.x, y, rows_area.width, 1),
            );
        }
    }

    if let Some(track) = crate::scrollbar::Track::vertical(area, nodes.len(), state.offset) {
        crate::scrollbar::render(frame, track, true, false);
    }
    // Sideways: the row is wider than the room the user left it.
    if let Some(track) = crate::scrollbar::Track::horizontal(area, content_w, state.h_offset) {
        crate::scrollbar::render(frame, track, false, false);
    }

    // The splitter: the view's left border, beside the column (which starts
    // under the view's own bar row). Its highlight is the cell a press grabs.
    if area.height > 0 {
        let rect = Rect::new(area.right(), area.y, 1, area.height);
        let hot = state.dragging
            || matches!(
                paint.hovered,
                Some(HitTarget::CommitColumn(ColumnHit::Divider))
            );
        if hot {
            let buf = frame.buffer_mut();
            for y in rect.y..rect.bottom() {
                if let Some(cell) = buf.cell_mut((rect.x, y)) {
                    cell.set_style(Style::default().fg(th().ui.accent));
                }
            }
        }
        hits.push((rect, HitTarget::CommitColumn(ColumnHit::Divider)));
    }
    hits
}

/// What each part needs for the rows in `rows` (by `Part::slot`): the widest
/// content, never under the part's own header word. The same functions the
/// row is drawn with, given room for everything.
fn natural_widths<'a>(
    rows: impl Iterator<Item = &'a CommitNode>,
    paint: &ColumnPaint<'_>,
    lanes: usize,
) -> [usize; super::PARTS] {
    let mut out = [0usize; super::PARTS];
    for part in Part::ORDER {
        out[part.slot()] = display_width(part.title());
    }
    let mut grow = |part: Part, cells: usize| {
        let at = part.slot();
        out[at] = out[at].max(cells);
    };
    let dim = Style::default();
    for node in rows {
        grow(Part::Title, display_width(&node.summary));
        grow(
            Part::Author,
            display_width(&short_author(&node.author_name, usize::MAX / 2)),
        );
        grow(
            Part::Date,
            display_width(&short_date(node.timestamp, usize::MAX / 2)),
        );
        grow(Part::Sha, 7);
        if let Some(pills) = paint.layout.pills_by_oid.get(&node.id) {
            let spans =
                crate::log_pills::ref_pill_spans(pills, usize::MAX / 2, th().ui.accent, None);
            grow(
                Part::Refs,
                spans.iter().map(|s| display_width(&s.content)).sum(),
            );
        }
        let cell =
            crate::log_table::changes_spans(paint.changes.get(&node.id), usize::MAX / 2, dim);
        grow(
            Part::Changes,
            cell.iter().map(|s| display_width(&s.content)).sum(),
        );
    }
    grow(Part::Graph, lanes);
    out
}

/// The header: each part's name over its cells, the separator cell blank (the
/// grab of a boundary is the next name's first letter, lit by the hover, as in
/// the Log). `focused` underlines the part the keyboard sizes.
fn header_line(spans: &[(Part, usize, usize)], focused: Option<Part>) -> Vec<Span<'static>> {
    let dim = Style::default()
        .fg(th().ui.muted)
        .add_modifier(Modifier::BOLD);
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut at = 0usize;
    for (part, rel, w) in spans {
        if *rel > at {
            // The separator before the part: blank.
            out.push(Span::raw(" ".repeat(rel - at)));
        }
        let style = if focused == Some(*part) {
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            dim
        };
        let label = fit(part.title(), *w);
        let pad = w.saturating_sub(display_width(&label));
        out.push(Span::styled(label, style));
        out.push(Span::raw(" ".repeat(pad)));
        at = rel + w;
    }
    out
}

/// The strip that brings a hidden column back, in the first `STRIP_COLS`
/// cells of `area` (the central column's left). Why a strip and not a button
/// on the border or the bar: the view's left border *is* the pane seam (the
/// resize grab of `pane_divider_grab_rect` owns that cell), and the bars are
/// packed with buttons that fold behind a `...` when narrow; a strip is the
/// one place that cannot collide with either, and the app already has the
/// pattern (`render_collapsed_pane_strip`: a thin strip with a chevron, the
/// whole strip is the target). The first cell stays the seam, the second is
/// the tab. Returns the hit rects of this walk.
pub(crate) fn render_strip(
    frame: &mut Frame,
    area: Rect,
    state: &mut CommitColumn,
    hovered: bool,
) -> Vec<(Rect, HitTarget)> {
    state.strip = paint_strip(frame, area, hovered);
    if state.strip.is_empty() {
        return Vec::new();
    }
    vec![(state.strip, HitTarget::CommitColumn(ColumnHit::Reopen))]
}

/// Paints the strip and returns its hit rect (the tab cell column, over the
/// whole height); empty when `area` is too narrow. Shared with the File
/// History's list, which hides the same way.
pub(crate) fn paint_strip(frame: &mut Frame, area: Rect, hovered: bool) -> Rect {
    let strip = Rect::new(
        area.x,
        area.y,
        super::STRIP_COLS.min(area.width),
        area.height,
    );
    let tab_rect = Rect::new(
        strip.x + 1,
        strip.y,
        strip.width.saturating_sub(1),
        strip.height,
    );
    if tab_rect.is_empty() {
        return tab_rect;
    }
    let seam = Style::default().fg(th().ui.border);
    let tab = Style::default()
        .bg(crate::theme::button_bg(hovered))
        .fg(th().ui.on_button);
    let buf = frame.buffer_mut();
    let mid = strip.y + strip.height / 2;
    for y in strip.y..strip.bottom() {
        if let Some(cell) = buf.cell_mut((strip.x, y)) {
            cell.set_symbol("\u{2502}").set_style(seam);
        }
        if let Some(cell) = buf.cell_mut((tab_rect.x, y)) {
            // `›`: points toward where the list comes back.
            cell.set_symbol(if y == mid { "\u{203A}" } else { " " })
                .set_style(tab);
        }
    }
    tab_rect
}

/// Whether the row under the last commit exists.
fn slot_free(inner: Rect, len: usize, offset: usize) -> bool {
    len.saturating_sub(offset) < inner.height as usize
}

/// One commit as one line, the whole row (`widths.content` cells), its parts
/// in the user's order with a cell of air between two. The caller cuts the
/// window out of it.
fn row_line(
    node: &CommitNode,
    grid: Option<&graph::GraphRow>,
    facts: RowFacts<'_>,
    widths: Widths,
    text_lanes: bool,
    selected: bool,
    hovered: bool,
) -> Vec<Span<'static>> {
    let dim = Style::default().fg(th().ui.muted);
    let bg = if selected {
        Some(th().ui.selection)
    } else if hovered {
        Some(th().ui.hover)
    } else {
        None
    };
    let mut spans: Vec<Span<'static>> = Vec::new();
    // The spans of the Refs part: the Log's pills keep their own colors.
    let mut pills_at = 0..0;
    let mut first = true;
    // The row tint of the Graph part, carried through the spacer after it:
    // the band ends on the cell the boundary line replaces, as in the Log.
    let mut spacer_tint: Option<ratatui::style::Color> = None;
    for part in widths.order {
        let w = widths.of(part);
        if w == 0 {
            continue;
        }
        if !first {
            spans.push(match spacer_tint {
                Some(bg) => Span::styled(" ", Style::default().bg(bg)),
                None => Span::raw(" "),
            });
        }
        spacer_tint = None;
        first = false;
        let start = spans.len();
        // The part's text, then blanks up to its width.
        let used: usize = match part {
            Part::Refs => {
                let pills = facts.pills.unwrap_or(&[]);
                let pill = if pills.is_empty() {
                    Vec::new()
                } else {
                    crate::log_pills::ref_pill_spans(
                        pills,
                        w,
                        facts.lane_color.unwrap_or(th().ui.muted),
                        None,
                    )
                };
                let pill = clip_spans(pill, w);
                let used = pill.iter().map(|s| display_width(&s.content)).sum();
                spans.extend(pill);
                pills_at = start..spans.len();
                used
            }
            Part::Graph => {
                let lanes = match grid {
                    Some(row) if text_lanes && graph::is_compact(w) => {
                        graph::compact_line(&row.cells, facts.node_set).spans
                    }
                    Some(row) if text_lanes => graph::text_line(row, facts.node_set).spans,
                    _ => Vec::new(),
                };
                let mut lanes = clip_spans(lanes, w);
                let mut used: usize = lanes.iter().map(|s| display_width(&s.content)).sum();
                // Text mode: the Log's row tint, from the node to the part's
                // end and on through the spacer (the picture's own band does
                // the same, `render`).
                if text_lanes && !graph::is_compact(w) {
                    if let Some(line) = facts.tinted {
                        for (span, tinted) in lanes.iter_mut().zip(&line.spans) {
                            span.style.bg = tinted.style.bg;
                        }
                        spacer_tint = line.spans.last().and_then(|s| s.style.bg);
                        if let (Some(bg), true) = (spacer_tint, used < w) {
                            lanes.push(Span::styled(" ".repeat(w - used), Style::default().bg(bg)));
                            used = w;
                        }
                    }
                }
                spans.extend(lanes);
                used
            }
            Part::Title => {
                let title = fit(&node.summary, w);
                let style = if selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let used = display_width(&title);
                spans.push(Span::styled(title, style));
                used
            }
            Part::Author => {
                let author = short_author(&node.author_name, w);
                let used = display_width(&author);
                spans.push(Span::styled(author, dim));
                used
            }
            Part::Date => {
                let date = short_date(node.timestamp, w);
                let used = display_width(&date);
                spans.push(Span::styled(date, dim));
                used
            }
            // The Log's own cell: `3 files +12 -4`, shortened to fit.
            Part::Changes => {
                let cell = crate::log_table::changes_spans(facts.changes, w, dim);
                let used = cell.iter().map(|s| display_width(&s.content)).sum();
                spans.extend(cell);
                used
            }
            Part::Sha => {
                let sha = node.id.to_hex()[..w.min(7)].to_string();
                let used = display_width(&sha);
                spans.push(Span::styled(sha, dim));
                used
            }
        };
        spans.push(Span::raw(" ".repeat(w.saturating_sub(used))));
    }
    if let Some(bg) = bg {
        // The pills keep their own colors, as in the Log (the band skips
        // the Ref column there too).
        for (i, span) in spans.iter_mut().enumerate() {
            if !pills_at.contains(&i) {
                span.style = span.style.bg(bg);
            }
        }
    }
    spans
}

/// What a row is made of besides the commit: the pills of its Refs part (the
/// Log's own `ref_pills_for_commit`, cached with the layout), its lane color
/// and the node glyphs of its lanes.
#[derive(Clone, Copy)]
struct RowFacts<'a> {
    pills: Option<&'a [crate::log_pills::RefPill]>,
    lane_color: Option<ratatui::style::Color>,
    /// The Log's own Graph cell of the row, with its row tint (text mode).
    tinted: Option<&'a Line<'static>>,
    node_set: graph::NodeSet,
    changes: Option<&'a gitpoulpi_core::domain::CommitStat>,
}
