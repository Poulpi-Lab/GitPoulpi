use super::model::{
    AnnotBlock, AnnotRow, AnnotSide, Annotator, DiffRender, DiffRow, DiffText, RowChange,
};
use super::DiffViewMode;
use crate::cell_text::{cells, char_at_cell, char_cells, pad_to, slice_cells};
use crate::editor_buffer::tab_width;
use crate::th;

use gitpoulpi_core::domain::{DiffLine, FileDiff, Hunk, LineOrigin};
use ratatui::prelude::*;

use crate::text_select::{RowSegs, Seg};
use crate::truncate_with_ellipsis;

/// A hunk as the view shows it, plus the thread back to the hunk as the
/// backend sees it.
///
/// `origin[i]` is the position of `hunk.lines[i]` in the original
/// `FileDiff`'s hunk. The two numberings differ when the whitespace
/// toggle is off and `effective_hunks` drops, rewrites and reorders
/// lines; the backend resolves a `LineSelector` against its own diff, so
/// shipping the rewritten positions would stage lines the user never
/// pointed at.
pub(super) struct EffectiveHunk {
    pub(super) hunk: Hunk,
    pub(super) origin: Vec<u32>,
}

/// Applies the whitespace toggle. With it on, hunks pass through
/// untouched. With it off, each run of `-` lines followed by `+` lines is
/// paired positionally; a pair whose contents match after `trim_end` is a
/// whitespace-only change and collapses to one context line.
///
/// Within a change run the collapsed context lines are emitted ahead of
/// the remaining `-`/`+` lines, which reorders rows only inside that run.
/// Dropping them instead would lose lines that exist in the file.
pub(super) fn effective_hunks(file: &FileDiff, show_whitespace: bool) -> Vec<EffectiveHunk> {
    if show_whitespace {
        return file
            .hunks
            .iter()
            .map(|hunk| EffectiveHunk {
                origin: (0..hunk.lines.len() as u32).collect(),
                hunk: hunk.clone(),
            })
            .collect();
    }
    file.hunks
        .iter()
        .filter_map(|hunk| {
            let mut out: Vec<DiffLine> = Vec::with_capacity(hunk.lines.len());
            let mut origin: Vec<u32> = Vec::with_capacity(hunk.lines.len());
            let mut i = 0usize;
            while i < hunk.lines.len() {
                if hunk.lines[i].origin == LineOrigin::Context {
                    out.push(hunk.lines[i].clone());
                    origin.push(i as u32);
                    i += 1;
                    continue;
                }
                let del_start = i;
                while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Delete {
                    i += 1;
                }
                let dels = &hunk.lines[del_start..i];
                let add_start = i;
                while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Add {
                    i += 1;
                }
                let adds = &hunk.lines[add_start..i];

                let mut collapsed: Vec<(u32, DiffLine)> = Vec::new();
                let mut kept_dels: Vec<(u32, DiffLine)> = Vec::new();
                let mut kept_adds: Vec<(u32, DiffLine)> = Vec::new();
                for n in 0..dels.len().max(adds.len()) {
                    match (dels.get(n), adds.get(n)) {
                        (Some(d), Some(a)) if d.content.trim_end() == a.content.trim_end() => {
                            collapsed.push((
                                (add_start + n) as u32,
                                DiffLine {
                                    origin: LineOrigin::Context,
                                    content: a.content.clone(),
                                    staged: a.staged,
                                },
                            ));
                        }
                        (d, a) => {
                            if let Some(d) = d {
                                kept_dels.push(((del_start + n) as u32, d.clone()));
                            }
                            if let Some(a) = a {
                                kept_adds.push(((add_start + n) as u32, a.clone()));
                            }
                        }
                    }
                }
                for (idx, line) in collapsed.into_iter().chain(kept_dels).chain(kept_adds) {
                    origin.push(idx);
                    out.push(line);
                }
            }
            let still_changes = out.iter().any(|l| l.origin != LineOrigin::Context);
            still_changes.then(|| EffectiveHunk {
                hunk: Hunk {
                    lines: out,
                    ..hunk.clone()
                },
                origin,
            })
        })
        .collect()
}

/// Old/new line numbers for every line of a hunk, in order.
fn line_numbers(hunk: &Hunk) -> Vec<(Option<u32>, Option<u32>)> {
    let mut old = hunk.old_start;
    let mut new = hunk.new_start;
    hunk.lines
        .iter()
        .map(|l| match l.origin {
            LineOrigin::Context => {
                let pair = (Some(old), Some(new));
                old += 1;
                new += 1;
                pair
            }
            LineOrigin::Delete => {
                let pair = (Some(old), None);
                old += 1;
                pair
            }
            LineOrigin::Add => {
                let pair = (None, Some(new));
                new += 1;
                pair
            }
        })
        .collect()
}

fn gutter(old: Option<u32>, new: Option<u32>) -> String {
    if !crate::editor_prefs::line_numbers() {
        // Same width, so the selection geometry does not move.
        return " ".repeat(GUTTER_WIDTH);
    }
    let o = old.map(|n| n.to_string()).unwrap_or_default();
    let n = new.map(|n| n.to_string()).unwrap_or_default();
    format!("{o:>5} {n:>5} ")
}

pub(super) const GUTTER_WIDTH: usize = 12;

pub(crate) fn marker(origin: &LineOrigin) -> (&'static str, Style) {
    match origin {
        LineOrigin::Add => (
            "+",
            Style::default().fg(th().diff.add_fg).bg(th().diff.add_bg),
        ),
        LineOrigin::Delete => (
            "-",
            Style::default().fg(th().diff.del_fg).bg(th().diff.del_bg),
        ),
        LineOrigin::Context => (" ", Style::default().fg(th().ui.text)),
    }
}

/// The three knobs (wrap, width, scroll) every row emitter needs, grouped
/// to stay under clippy's `too_many_arguments`.
#[derive(Debug, Clone, Copy)]
pub(super) struct LayoutOpts<'a> {
    pub(super) word_wrap: bool,
    pub(super) width: u16,
    pub(super) h_offset: usize,
    pub(super) mode: DiffViewMode,
    /// Rows another view wants under the code lines (the review's threads).
    pub(super) annot: Option<&'a dyn Annotator>,
}

/// One hunk in the one-column layout, every line of it.
pub(super) fn emit_unified(sink: &mut Sink, eh: &EffectiveHunk, opts: LayoutOpts<'_>) {
    let numbers = line_numbers(&eh.hunk);
    for (i, line) in eh.hunk.lines.iter().enumerate() {
        let carries = line.origin != LineOrigin::Context;
        push_unified_line(
            sink,
            (&line.origin, &line.content),
            numbers[i],
            DiffRow::body(eh.hunk.id, [carries.then(|| eh.origin[i]), None])
                .with_change(RowChange::of(&line.origin)),
            opts,
        );
        annotate(sink, numbers[i].0, numbers[i].1, opts);
    }
}

/// One line in the one-column layout: one row, or several when word wrap
/// breaks it. `first` is the row-map entry of its first row; only that
/// row carries the line, or a selection over one wrapped line would look
/// like two selected lines.
pub(super) fn push_unified_line(
    sink: &mut Sink,
    (origin, text): (&LineOrigin, &str),
    (old, new): (Option<u32>, Option<u32>),
    first: DiffRow,
    opts: LayoutOpts<'_>,
) {
    let first = DiffRow {
        numbers: [old, new],
        ..first
    };
    let content_width = (opts.width as usize).saturating_sub(GUTTER_WIDTH + 1);
    let (mark, style) = marker(origin);
    let line = sink.add_text(0, text);
    let pieces = row_pieces(text, content_width, opts);
    for (n, (piece, span)) in pieces.into_iter().enumerate() {
        let g = if n == 0 {
            gutter(old, new)
        } else {
            " ".repeat(GUTTER_WIDTH)
        };
        let m = if n == 0 { mark } else { " " };
        let seg = span.seg(line, (GUTTER_WIDTH + 1) as u16, content_width);
        sink.push(
            Line::from(vec![
                Span::styled(g, Style::default().fg(th().diff.gutter_fg)),
                Span::styled(m.to_string(), style),
                Span::styled(piece, style),
            ]),
            if n == 0 {
                first
            } else {
                DiffRow {
                    header: false,
                    lines: [None, None],
                    ..first
                }
            },
            [Some(seg), None],
        );
    }
}

/// One column of a split-view row: the line it shows and the number to
/// print beside it. `None` in its place is a blank half (the "before"
/// side of a pure addition, the "after" side of a deletion).
#[derive(Clone, Copy)]
pub(super) struct SideLine<'a> {
    pub(super) origin: &'a LineOrigin,
    pub(super) content: &'a str,
    pub(super) number: u32,
}

/// Side-by-side rows. Deletes are paired positionally with the adds that
/// follow them; an unmatched side renders blank, so a pure addition (or a
/// deleted file) still reads correctly.
pub(super) fn emit_split(sink: &mut Sink, eh: &EffectiveHunk, opts: LayoutOpts<'_>) {
    let hunk = &eh.hunk;
    let numbers = line_numbers(hunk);
    // A side as (index into the hunk, number to print).
    type SplitSide = Option<(usize, u32)>;
    let mut split_rows: Vec<(SplitSide, SplitSide)> = Vec::new();
    let mut i = 0usize;
    while i < hunk.lines.len() {
        if hunk.lines[i].origin == LineOrigin::Context {
            let (o, n) = numbers[i];
            split_rows.push((Some((i, o.unwrap_or(0))), Some((i, n.unwrap_or(0)))));
            i += 1;
            continue;
        }
        let del_start = i;
        while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Delete {
            i += 1;
        }
        let del_range: Vec<usize> = (del_start..i).collect();
        let add_start = i;
        while i < hunk.lines.len() && hunk.lines[i].origin == LineOrigin::Add {
            i += 1;
        }
        let add_range: Vec<usize> = (add_start..i).collect();
        for n in 0..del_range.len().max(add_range.len()) {
            let left = del_range
                .get(n)
                .map(|&idx| (idx, numbers[idx].0.unwrap_or(0)));
            let right = add_range
                .get(n)
                .map(|&idx| (idx, numbers[idx].1.unwrap_or(0)));
            split_rows.push((left, right));
        }
    }

    for (left, right) in split_rows {
        // A split row shows a delete and its paired add, so it carries
        // both: selecting it selects the whole change.
        let carried = |side: SplitSide| {
            side.filter(|&(idx, _)| hunk.lines[idx].origin != LineOrigin::Context)
                .map(|(idx, _)| eh.origin[idx])
        };
        let side_line = |side: SplitSide| {
            side.map(|(idx, number)| SideLine {
                origin: &hunk.lines[idx].origin,
                content: &hunk.lines[idx].content,
                number,
            })
        };
        push_split_row(
            sink,
            side_line(left),
            side_line(right),
            DiffRow::body(hunk.id, [carried(left), carried(right)]).with_change(
                match (
                    left.map(|(i, _)| RowChange::of(&hunk.lines[i].origin)),
                    right.map(|(i, _)| RowChange::of(&hunk.lines[i].origin)),
                ) {
                    (Some(RowChange::Del), Some(RowChange::Add)) => RowChange::Both,
                    (Some(RowChange::Del), _) => RowChange::Del,
                    (_, Some(RowChange::Add)) => RowChange::Add,
                    _ => RowChange::None,
                },
            ),
            opts,
        );
        annotate(
            sink,
            left.map(|(_, number)| number),
            right.map(|(_, number)| number),
            opts,
        );
    }
}

/// One split row - or several when word wrap breaks either side. `first`
/// is the row-map entry of the first; continuation rows carry no line
/// (see `push_unified_line`).
pub(super) fn push_split_row(
    sink: &mut Sink,
    left: Option<SideLine>,
    right: Option<SideLine>,
    first: DiffRow,
    opts: LayoutOpts<'_>,
) {
    let first = DiffRow {
        numbers: [left.map(|l| l.number), right.map(|r| r.number)],
        ..first
    };
    // Each side: 5-col number + marker + content, and a 1-col separator.
    let content_width = split_content_width(opts.width);
    let mut side_rows = |side: usize, line: Option<SideLine>| match line {
        Some(l) => {
            let id = sink.add_text(side, l.content);
            let x = if side == 0 { 7 } else { content_width + 15 };
            row_pieces(l.content, content_width, opts)
                .into_iter()
                .map(|(piece, span)| (piece, span.seg(id, x as u16, content_width)))
                .collect::<Vec<_>>()
        }
        None => Vec::new(),
    };
    let l_rows = side_rows(0, left);
    let r_rows = side_rows(1, right);
    let height = l_rows.len().max(r_rows.len()).max(1);
    for n in 0..height {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let l = l_rows.get(n);
        let r = r_rows.get(n);
        push_side(&mut spans, left, l.map(|(p, _)| p), n, content_width);
        spans.push(Span::styled(
            "\u{2502}",
            Style::default().fg(th().diff.gutter_fg),
        ));
        push_side(&mut spans, right, r.map(|(p, _)| p), n, content_width);
        sink.push(
            Line::from(spans),
            if n == 0 {
                first
            } else {
                DiffRow {
                    header: false,
                    lines: [None, None],
                    ..first
                }
            },
            [l.map(|(_, s)| *s), r.map(|(_, s)| *s)],
        );
    }
}

/// Columns of text one row can show at inner `width`, without word wrap:
/// the unified layout loses its gutter, Split shows one side at a time.
pub(crate) fn text_viewport(mode: DiffViewMode, width: u16) -> usize {
    match mode {
        DiffViewMode::Inline => (width as usize).saturating_sub(GUTTER_WIDTH + 1),
        DiffViewMode::Split => split_content_width(width),
    }
}

/// The largest useful horizontal offset: 0 while no line overflows the
/// text column (no scrollbar), else the offset at which the longest
/// line's last character is drawn. Scrolled rows spend one column on the
/// left-guillemet marker, hence the `- 1`. Measured against the text
/// column, not the pane width: the gutter (and in Split the other side)
/// eats columns, and the pane width would stop the drag short of the end.
pub(crate) fn h_scroll_max(longest: usize, mode: DiffViewMode, width: u16) -> usize {
    let view = text_viewport(mode, width);
    if longest <= view {
        0
    } else {
        longest - view.saturating_sub(1).max(1)
    }
}

/// The content length to hand `scrollbar::Track::horizontal` (whose
/// viewport is the pane's inner width) so its thumb range is exactly
/// `0..=h_scroll_max`.
pub(crate) fn h_track_content(
    longest: usize,
    mode: DiffViewMode,
    width: u16,
    track_width: u16,
) -> usize {
    match h_scroll_max(longest, mode, width) {
        0 => 0,
        max => max + track_width as usize,
    }
}

/// Row `i` of the render as the minimap sees it: the text without the
/// line-number gutter, and the change the row stands for.
pub(crate) fn minimap_row(
    render: &DiffRender,
    i: usize,
) -> (
    std::borrow::Cow<'static, str>,
    Option<crate::scroll_marks::Mark>,
) {
    use crate::scroll_marks::Mark;
    let text: String = render.lines[i]
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    let text: String = text.chars().skip(GUTTER_WIDTH + 1).collect();
    let mark = match render.rows[i].change {
        RowChange::None => None,
        RowChange::Add => Some(Mark::Add),
        RowChange::Del => Some(Mark::Del),
        RowChange::Both => Some(Mark::Modified),
    };
    (std::borrow::Cow::Owned(text), mark)
}

/// Where an annotation row's text may go: `(first column, columns)` inside
/// the inner width. Inline gives every side the text column (the old/new
/// gutters stay clear); Split gives Old the left half and New the right
/// half, each from its marker column, so a comment sits under the side it is
/// about. The one geometry both the render and the maintainer's paint walk use.
pub(crate) fn annot_region(mode: DiffViewMode, width: u16, side: AnnotSide) -> (usize, usize) {
    let width = width as usize;
    let column = GUTTER_WIDTH.min(width);
    let text = (column, width - column);
    match (mode, side) {
        (_, AnnotSide::Full) => (0, width),
        (DiffViewMode::Inline, _) | (_, AnnotSide::Span) => text,
        (DiffViewMode::Split, AnnotSide::Old) => {
            let cw = split_content_width(width as u16);
            (6.min(width), (cw + 1).min(width.saturating_sub(6)))
        }
        (DiffViewMode::Split, AnnotSide::New) => {
            let cw = split_content_width(width as u16);
            let start = (cw + 14).min(width);
            (start, (cw + 1).min(width - start))
        }
    }
}

/// The annotator's rows for the code line showing `old` / `new`, pushed right
/// under the rows just emitted.
pub(super) fn annotate(sink: &mut Sink, old: Option<u32>, new: Option<u32>, opts: LayoutOpts<'_>) {
    let Some(annotator) = opts.annot else { return };
    let text_width = annot_region(opts.mode, opts.width, AnnotSide::Span)
        .1
        .saturating_sub(2);
    let split_width = annot_region(opts.mode, opts.width, AnnotSide::New)
        .1
        .saturating_sub(2);
    let text_width = if opts.mode == DiffViewMode::Split {
        split_width
    } else {
        text_width
    };
    for AnnotBlock { side, rows } in annotator.after_line(old, new, text_width) {
        for row in rows {
            sink.push_annot(side, row, opts.mode, opts.width);
        }
    }
}

/// A Split side's content width at a given inner width.
pub(super) fn split_content_width(width: u16) -> usize {
    let side_total = (width as usize).saturating_sub(1) / 2;
    side_total.saturating_sub(7)
}

/// Everything the row emitters append to, kept in lockstep: the drawn
/// line, its row-map entry (staging) and its text segments (selection).
#[derive(Default)]
pub(super) struct Sink {
    pub(super) lines: Vec<Line<'static>>,
    pub(super) rows: Vec<DiffRow>,
    pub(super) segs: Vec<RowSegs>,
    pub(super) text: DiffText,
    /// The text of the annotation rows, by row index (`DiffRender::annot_text`).
    pub(super) annot_text: std::collections::HashMap<usize, String>,
}

impl Sink {
    pub(super) fn push(&mut self, line: Line<'static>, row: DiffRow, segs: RowSegs) {
        self.lines.push(line);
        self.rows.push(row);
        self.segs.push(segs);
    }

    /// One annotation row: blank space in the code columns, the maintainer's text
    /// inside its region (the owner repaints it in its own style), and the
    /// Split separator carried through so the column stays unbroken.
    fn push_annot(&mut self, side: AnnotSide, row: AnnotRow, mode: DiffViewMode, width: u16) {
        let (start, len) = annot_region(mode, width, side);
        let total = width as usize;
        let mut cells: Vec<char> = vec![' '; total];
        // The owner draws a bar or a glyph in the first two columns of its
        // region and the text after them; a heading takes the whole row.
        let indent = if side == AnnotSide::Full { 0 } else { 2 };
        let shown = slice_cells(&row.text, 0, len.saturating_sub(indent));
        let mut at = start + indent;
        for c in shown.chars() {
            let w = char_cells(c);
            if w == 0 || at + w > total.min(start + len) {
                break;
            }
            cells[at] = c;
            for filler in cells.iter_mut().skip(at + 1).take(w - 1) {
                *filler = '\u{0}';
            }
            at += w;
        }
        let text: String = cells.iter().filter(|c| **c != '\u{0}').collect();
        let sep = (mode == DiffViewMode::Split && side != AnnotSide::Full)
            .then(|| split_content_width(width) + 7)
            .filter(|c| *c < total && cells[*c] == ' ');
        let line = match sep {
            Some(col) => {
                let before: String = cells[..col].iter().filter(|c| **c != '\u{0}').collect();
                let after: String = cells[col + 1..].iter().filter(|c| **c != '\u{0}').collect();
                Line::from(vec![
                    Span::raw(before),
                    Span::styled("\u{2502}", Style::default().fg(th().diff.gutter_fg)),
                    Span::raw(after),
                ])
            }
            None => Line::from(text),
        };
        self.annot_text.insert(self.lines.len(), row.text);
        self.push(
            line,
            DiffRow {
                annot: Some(row.tag),
                ..DiffRow::default()
            },
            [None, None],
        );
    }

    /// What the annotator could not place under a line, after the last row.
    pub(super) fn trailing(
        &mut self,
        annotator: Option<&dyn Annotator>,
        mode: DiffViewMode,
        width: u16,
    ) {
        let Some(a) = annotator else { return };
        let text_width = annot_region(mode, width, AnnotSide::Span)
            .1
            .saturating_sub(2);
        for row in a.trailing(text_width) {
            let side = row.tag.side;
            self.push_annot(side, row, mode, width);
        }
    }

    /// Records a logical line on `side` and returns its index there.
    fn add_text(&mut self, side: usize, raw: &str) -> usize {
        let lines = &mut self.text.sides[side];
        lines.push(raw.to_string());
        lines.len() - 1
    }
}

/// Where one drawn row sits in its logical line (displayed chars).
#[derive(Debug, Clone, Copy)]
struct PieceSpan {
    start: usize,
    len: usize,
    last: bool,
    /// Leading cells of the row that map to no char (the `\u{ab}` marker
    /// standing for a wide char cut by the scroll edge).
    shift: u16,
}

impl PieceSpan {
    fn seg(self, line: usize, x: u16, width: usize) -> Seg {
        Seg {
            line,
            start: self.start,
            len: self.len,
            x: x + self.shift,
            width: (width as u16).saturating_sub(self.shift),
            last: self.last,
        }
    }
}

/// The row(s) one content string occupies, each with the stretch of the
/// line it stands for: with wrap off one scrolled, ellipsis-truncated
/// row standing for the rest of the line; with it on, the word-wrapped
/// rows.
fn row_pieces(raw: &str, width: usize, opts: LayoutOpts<'_>) -> Vec<(String, PieceSpan)> {
    let shown = expand_tabs(raw);
    let len = shown.chars().count();
    if !opts.word_wrap {
        let (content, start, shift) = scroll_content(&shown, opts.h_offset);
        return vec![(
            truncate_with_ellipsis(&content, width),
            PieceSpan {
                start,
                len: len.saturating_sub(start),
                last: true,
                shift,
            },
        )];
    }
    let chars: Vec<char> = shown.chars().collect();
    wrap_spans(&chars, width)
        .into_iter()
        .map(|(start, end, next)| {
            (
                chars[start..end].iter().collect(),
                PieceSpan {
                    start,
                    len: next - start,
                    last: next >= len,
                    shift: 0,
                },
            )
        })
        .collect()
}

/// Greedy word wrap as char ranges: `(start, end_drawn, next_start)` per
/// row, each row at most `width` **cells** wide. A row breaks at the last
/// blank that fits (the blanks at the break are not drawn but belong to
/// the row they end, so a selection copies them); a token too long to ever
/// fit is hard-broken, never between a char and its combining marks.
/// Ranges, not strings, so every row knows which chars of the line it is
/// (what a text selection maps through). Leading indentation stays on the
/// first row.
fn wrap_spans(chars: &[char], width: usize) -> Vec<(usize, usize, usize)> {
    let n = chars.len();
    if width == 0 {
        return vec![(0, 0, n)];
    }
    // cum[i] = cells before char i.
    let mut cum = Vec::with_capacity(n + 1);
    cum.push(0usize);
    for &c in chars {
        cum.push(cum[cum.len() - 1] + char_cells(c));
    }
    if cum[n] <= width {
        return vec![(0, n, n)];
    }
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < n {
        if cum[n] - cum[pos] <= width {
            out.push((pos, n, n));
            break;
        }
        // The furthest end whose cells fit, at least one char, and never
        // between a char and the marks after it.
        let mut limit = pos + 1;
        while limit < n && cum[limit + 1] - cum[pos] <= width {
            limit += 1;
        }
        while limit > pos + 1 && char_cells(chars[limit]) == 0 {
            limit -= 1;
        }
        let first_word = (pos..limit).find(|&i| !chars[i].is_whitespace());
        let brk =
            first_word.and_then(|fw| (fw + 1..=limit).rev().find(|&w| chars[w].is_whitespace()));
        match brk {
            Some(w) => {
                let mut end = w;
                while end > pos && chars[end - 1].is_whitespace() {
                    end -= 1;
                }
                let mut next = w;
                while next < n && chars[next].is_whitespace() {
                    next += 1;
                }
                out.push((pos, end, next));
                pos = next;
            }
            None => {
                out.push((pos, limit, limit));
                pos = limit;
            }
        }
    }
    out
}

/// Drops the first `h_offset` **cells** of a content string and, if
/// anything was dropped, marks the row with `\u{ab}`: the fixed gutter
/// never scrolls, so the marker is the only sign a row is not showing
/// column 0. A wide char cut by the edge leaves a blank for its other half.
/// Returns the text, the char offset the marker cell stands for (what
/// `Seg::start` wants) and how many leading cells map to no char.
/// Mirrors `file_view::clip_scrolled`.
fn scroll_content(s: &str, h_offset: usize) -> (String, usize, u16) {
    if h_offset == 0 {
        return (s.to_string(), 0, 0);
    }
    let rest = slice_cells(s, h_offset, usize::MAX);
    if rest.is_empty() {
        return ("\u{ab}".to_string(), h_offset.saturating_sub(1), 0);
    }
    let (first, straddled) = char_at_cell(s, h_offset);
    // The marker takes the place of the char before the first visible
    // one when that is one cell wide (the common case); otherwise the
    // marker cell maps to nothing and the row starts at the first char.
    let before = first
        .checked_sub(1)
        .and_then(|i| s.chars().nth(i))
        .map_or(0, char_cells);
    let (start, shift) = if straddled || before == 1 {
        (first.saturating_sub(usize::from(!straddled)), 0)
    } else {
        (first, 1)
    };
    (format!("\u{ab}{rest}"), start, shift)
}

fn push_side(
    spans: &mut Vec<Span<'static>>,
    side: Option<SideLine>,
    piece: Option<&String>,
    n: usize,
    content_width: usize,
) {
    match side {
        Some(SideLine { origin, number, .. }) => {
            let (mark, style) = marker(origin);
            let num = if n == 0 && crate::editor_prefs::line_numbers() {
                format!("{number:>5} ")
            } else {
                " ".repeat(6)
            };
            let m = if n == 0 { mark } else { " " };
            let body = piece.cloned().unwrap_or_default();
            spans.push(Span::styled(num, Style::default().fg(th().diff.gutter_fg)));
            spans.push(Span::styled(m.to_string(), style));
            spans.push(Span::styled(pad_to(&body, content_width), style));
        }
        None => {
            spans.push(Span::raw(" ".repeat(content_width + 7)));
        }
    }
}

/// Renders each tab as spaces up to the next tab stop: a raw `\t` in a
/// `Span` advances to the terminal's own 8-column stop and shears indented
/// lines out of alignment with the gutter.
pub(super) fn expand_tabs(s: &str) -> String {
    if !s.contains('\t') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch == '\t' {
            let pad = tab_width() - (cells(&out) % tab_width());
            out.push_str(&" ".repeat(pad));
        } else {
            out.push(ch);
        }
    }
    out
}
