use super::layout::{
    annotate, effective_hunks, emit_split, emit_unified, push_split_row, push_unified_line,
    split_content_width, LayoutOpts, SideLine, Sink,
};
use super::{DiffBarButton, DiffViewMode, DiffViewState, DIFF_BAR_BUTTONS};
use crate::editor_buffer::tab_width;
use crate::th;
use gitpoulpi_app::diff::styled;
use gitpoulpi_i18n::msg::diff_view as msg;
use std::rc::Rc;

pub(crate) use gitpoulpi_app::diff::{
    hunk_at, selected_lines, AnnotSide, AnnotTag, DiffRow, DiffText, RowChange,
};

use gitpoulpi_core::domain::{FileDiff, Hunk, LineOrigin};
use ratatui::prelude::*;

use crate::text_select::{self, RowSegs};
use crate::{chip_overflow_style, display_width, fit_bar, BarItem, OVERFLOW_LABEL};

/// One screen line an [`Annotator`] adds. `text` is already wrapped to the
/// width the annotator was told.
#[derive(Debug, Clone)]
pub(crate) struct AnnotRow {
    pub(crate) tag: AnnotTag,
    pub(crate) text: String,
}

/// Annotation rows that belong under one code line, on one side.
#[derive(Debug, Clone)]
pub(crate) struct AnnotBlock {
    pub(crate) side: AnnotSide,
    pub(crate) rows: Vec<AnnotRow>,
}

/// The hook that lets another view put its own rows into the diff's row
/// model (the review's threads, pending comments and suggestions) instead of
/// copying the layout: the render calls it under every code line it draws,
/// in the Inline and Split layouts, in hunks-only and whole-file mode alike.
///
/// Implementations are asked in render order and may remember what they
/// placed (interior mutability): whatever is left when the diff ends is
/// asked for with [`Annotator::trailing`].
pub(crate) trait Annotator: std::fmt::Debug {
    /// The rows to draw under the code line showing `old` / `new` (the file
    /// line number on each side, `None` where the side has no line here).
    /// `text_width` is the room one row of text has.
    fn after_line(&self, old: Option<u32>, new: Option<u32>, text_width: usize) -> Vec<AnnotBlock>;
    /// The rows that found no line, after the last row of the file.
    fn trailing(&self, text_width: usize) -> Vec<AnnotRow>;
}

/// The rendered diff: the rows, the row index each surviving hunk's
/// header landed on (what ↑/↓ and `n`/`p` scroll to), and one `DiffRow`
/// per row.
///
/// `rows.len() == lines.len()` is an invariant every consumer relies on
/// and a test pins.
///
/// `rows` is an `Rc` because the render is cached across frames
/// (`diff_content::DiffContent`) and the event loop keeps the row map to
/// answer the next key or click; copying a large whole-file diff every
/// frame is the O(n) the cache avoids.
pub(crate) struct DiffRender {
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) hunk_starts: Vec<usize>,
    pub(crate) rows: Rc<[DiffRow]>,
    /// Per row, the text it shows on each side: what a mouse selection is
    /// mapped through. Same walk as `rows`, so `segs.len() == lines.len()`.
    pub(crate) segs: Rc<[RowSegs]>,
    /// The logical lines those segments index: side 0 is Inline's one
    /// column or Split's before column, side 1 Split's after column.
    pub(crate) text: Rc<DiffText>,
    /// Split only: the first content column (relative to the inner rect)
    /// of the after half, where a drag's side is decided.
    pub(crate) split_x: Option<u16>,
    /// The text of every annotation row, by row index (see
    /// `DiffRow::annot`): the owner paints these itself, in its own style,
    /// inside [`annot_region`].
    pub(crate) annot_text: Rc<std::collections::HashMap<usize, String>>,
}

/// A render's logical lines as a selectable text column.
pub(crate) trait DiffTextExt {
    /// One side as a selectable text column.
    fn side(&self, side: usize) -> text_select::Lines<'_>;
}

impl DiffTextExt for DiffText {
    fn side(&self, side: usize) -> text_select::Lines<'_> {
        text_select::Lines {
            lines: &self.sides[side.min(1)],
            tabs: text_select::Tabs::Expand(tab_width()),
        }
    }
}

impl DiffRender {
    /// A single italic line and no hunk: the "nothing to show" notices.
    pub(crate) fn notice(text: impl Into<String>) -> Self {
        Self {
            lines: vec![super::term::line_of(&styled::notice(text))],
            hunk_starts: Vec::new(),
            rows: Rc::from(vec![DiffRow::default()]),
            segs: Rc::from(vec![[None, None]]),
            text: Rc::new(DiffText::default()),
            split_x: None,
            annot_text: Rc::default(),
        }
    }

    /// The same render with one notice row on top (every row index and
    /// hunk start shifted by one, so the row map stays true).
    pub(crate) fn with_notice_on_top(self, text: impl Into<String>) -> Self {
        let mut top = Self::notice(text);
        top.lines.extend(self.lines);
        let mut rows = vec![DiffRow::default()];
        rows.extend(self.rows.iter().copied());
        let mut segs: Vec<RowSegs> = vec![[None, None]];
        segs.extend(self.segs.iter().copied());
        Self {
            lines: top.lines,
            hunk_starts: self.hunk_starts.iter().map(|h| h + 1).collect(),
            rows: Rc::from(rows),
            segs: Rc::from(segs),
            text: self.text,
            split_x: self.split_x,
            annot_text: Rc::new(
                self.annot_text
                    .iter()
                    .map(|(row, text)| (row + 1, text.clone()))
                    .collect(),
            ),
        }
    }

    /// Several files' renders one under the other, each with a `header` row
    /// of its own (the review's Commits page): row indices, hunk starts and
    /// the logical lines the text segments point at are all re-based, so the
    /// row map, the selection and the ruler stay true over the whole stack.
    /// `header` makes the row for a file's name.
    #[cfg_attr(not(all(feature = "hosting", feature = "review")), allow(dead_code))]
    pub(crate) fn stack(
        parts: Vec<(AnnotRow, DiffRender)>,
        mode: DiffViewMode,
        width: u16,
    ) -> Self {
        let mut out = Self {
            lines: Vec::new(),
            hunk_starts: Vec::new(),
            rows: Rc::default(),
            segs: Rc::default(),
            text: Rc::default(),
            split_x: (mode == DiffViewMode::Split).then(|| split_content_width(width) as u16 + 8),
            annot_text: Rc::default(),
        };
        let mut rows: Vec<DiffRow> = Vec::new();
        let mut segs: Vec<RowSegs> = Vec::new();
        let mut text = DiffText::default();
        let mut annots = std::collections::HashMap::new();
        for (header, part) in parts {
            annots.insert(rows.len(), header.text.clone());
            out.lines.push(Line::from(header.text));
            rows.push(DiffRow {
                annot: Some(header.tag),
                ..DiffRow::default()
            });
            segs.push([None, None]);
            let base = rows.len();
            let line_base = [text.sides[0].len(), text.sides[1].len()];
            out.hunk_starts
                .extend(part.hunk_starts.iter().map(|h| h + base));
            out.lines.extend(part.lines.iter().cloned());
            rows.extend(part.rows.iter().copied());
            for pair in part.segs.iter() {
                let mut pair = *pair;
                for (side, seg) in pair.iter_mut().enumerate() {
                    if let Some(seg) = seg {
                        seg.line += line_base[side];
                    }
                }
                segs.push(pair);
            }
            for (row, t) in part.annot_text.iter() {
                annots.insert(row + base, t.clone());
            }
            for side in 0..2 {
                text.sides[side].extend(part.text.sides[side].iter().cloned());
            }
        }
        out.rows = Rc::from(rows);
        out.segs = Rc::from(segs);
        out.text = Rc::new(text);
        out.annot_text = Rc::new(annots);
        out
    }
}

/// The Diff view's top button bar, laid out for one width.
pub(crate) struct DiffBarRender {
    pub(crate) line: Line<'static>,
    /// Per drawn button, its `(x_offset_from_bar_start, width)`, so the
    /// caller registers hit targets from the exact walk that drew them.
    pub(crate) buttons: Vec<(u16, u16, DiffBarButton)>,
    /// `(x_offset, width)` of the `⋯`, when some buttons did not fit.
    pub(crate) overflow: Option<(u16, u16)>,
    /// The buttons behind the `⋯`, in bar order.
    pub(crate) hidden: Vec<DiffBarButton>,
}

/// Builds the Diff view's top button bar.
/// Buttons that do not fit collapse behind a `⋯` at the end, least
/// important first (`DiffBarButton::priority`). One row, nothing wraps.
/// `overflow_hovered` is whether the pointer is on the `⋯`.
pub(crate) fn diff_bar(
    state: &DiffViewState,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    width: u16,
) -> DiffBarRender {
    diff_bar_of(DIFF_BAR_BUTTONS, state, hovered, overflow_hovered, width)
}

/// [`diff_bar`] for another set of buttons in the same look (the review's
/// diff pane keeps the display toggles and drops the view tabs).
pub(crate) fn diff_bar_of(
    buttons: &[DiffBarButton],
    state: &DiffViewState,
    hovered: Option<DiffBarButton>,
    overflow_hovered: bool,
    width: u16,
) -> DiffBarRender {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut hits: Vec<(u16, u16, DiffBarButton)> = Vec::new();
    // Edit in Working Directory is always offered, on a commit's diff too:
    // it opens the working-tree copy of the path. A missing copy is
    // refused at click time (`edit_target`), not grayed here.
    // The same priorities in every state of the commit column (and in the
    // File History): which buttons fit, and where they are, never depends on
    // what the column beside the view is doing.
    let items: Vec<BarItem<DiffBarButton>> = buttons
        .iter()
        .map(|button| BarItem {
            key: *button,
            width: display_width(&format!(" {} ", button.label_for(state))) as u16,
            gap_before: 1,
            priority: button.priority(),
        })
        .collect();
    let fit = fit_bar(&items, width, 1);
    let mut x: u16 = 0;

    for (bx, item) in &fit.visible {
        let button = &item.key;
        let label = button.label_for(state);
        if *bx > x {
            spans.push(Span::raw(" ".repeat((bx - x) as usize)));
        }
        if *button == DiffBarButton::History {
            // The column's `File | All` selector: two hits, one control.
            let (segments, segment_hits) = crate::commit_column::selector(hovered);
            spans.extend(segments);
            for (dx, w, b) in segment_hits {
                hits.push((*bx + dx, w, b));
            }
            x = bx + item.width;
            continue;
        }
        let text = format!(" {label} ");
        let w = item.width;
        let enabled = button.enabled();
        let active = button.is_active(state);
        let style = if !enabled {
            // Dimmed: clearly not live, still legible.
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::DIM)
        } else if active {
            Style::default()
                .bg(th().ui.accent)
                .fg(th().ui.on_accent)
                .add_modifier(Modifier::BOLD)
        } else if hovered == Some(*button) {
            Style::default()
                .bg(th().ui.button_hover)
                .fg(th().ui.on_button)
        } else {
            Style::default().bg(th().ui.button).fg(th().ui.on_button)
        };
        spans.push(Span::styled(text, style));
        hits.push((*bx, w, *button));
        x = bx + w;
    }
    if let Some((ox, _)) = fit.overflow_button {
        if ox > x {
            spans.push(Span::raw(" ".repeat((ox - x) as usize)));
        }
        spans.push(Span::styled(
            OVERFLOW_LABEL,
            chip_overflow_style(overflow_hovered),
        ));
    }
    DiffBarRender {
        line: Line::from(spans),
        buttons: hits,
        overflow: fit.overflow_button,
        hidden: fit.overflow,
    }
}

/// Everything `render_diff_with` needs besides the `FileDiff`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DiffRenderOpts<'a> {
    pub(crate) mode: DiffViewMode,
    pub(crate) hunks_only: bool,
    pub(crate) show_whitespace: bool,
    pub(crate) word_wrap: bool,
    /// The content width available (the block's inner width). A width of
    /// 0 still returns well-formed rows: ratatui hands over degenerate
    /// rects during resizes.
    pub(crate) width: u16,
    /// The first character column of every row's content span, never the
    /// fixed gutter. Ignored (as if 0) whenever `word_wrap` is true.
    pub(crate) h_offset: usize,
    /// The new side's whole content, one entry per line, for the
    /// whole-file layout (`hunks_only == false`). `None` there renders
    /// the hunks alone; `diff_content.rs` says why.
    pub(crate) full_file: Option<&'a [String]>,
    /// Show a Git LFS pointer diff as its raw text (`DiffViewState::lfs_raw`).
    pub(crate) lfs_raw: bool,
    /// See `DiffViewState::lfs_local`.
    pub(crate) lfs_local: Option<bool>,
    /// Paint the changed words of a changed line stronger (`ui.diff_intraline`).
    pub(crate) intraline: bool,
}

impl<'a> DiffRenderOpts<'a> {
    /// The options the open view's own toggles ask for.
    pub(crate) fn for_state(
        state: &DiffViewState,
        width: u16,
        full_file: Option<&'a [String]>,
    ) -> Self {
        Self {
            mode: state.mode,
            hunks_only: state.hunks_only,
            show_whitespace: state.show_whitespace,
            word_wrap: state.word_wrap,
            width,
            h_offset: state.h_offset,
            full_file,
            lfs_raw: state.lfs_raw,
            lfs_local: state.lfs_local,
            intraline: crate::live_settings::diff_intraline(),
        }
    }
}

/// Hunks-only convenience over `render_diff_with` for tests that do not
/// care about horizontal scroll or the whole-file layout.
#[cfg(test)]
pub(crate) fn render_diff(
    file: &FileDiff,
    mode: DiffViewMode,
    show_whitespace: bool,
    word_wrap: bool,
    width: u16,
) -> DiffRender {
    render_diff_scrolled(file, mode, show_whitespace, word_wrap, width, 0)
}

/// `render_diff` plus the horizontal scroll (test convenience).
#[cfg(test)]
pub(crate) fn render_diff_scrolled(
    file: &FileDiff,
    mode: DiffViewMode,
    show_whitespace: bool,
    word_wrap: bool,
    width: u16,
    h_offset: usize,
) -> DiffRender {
    render_diff_with(
        file,
        &DiffRenderOpts {
            mode,
            hunks_only: true,
            show_whitespace,
            word_wrap,
            width,
            h_offset,
            full_file: None,
            lfs_raw: false,
            lfs_local: None,
            intraline: false,
        },
    )
}

/// The first line a hunk covers on one side. git spells an empty side as
/// `+N,0` meaning "after line N", so that region starts one line later:
/// a pure deletion at the top of a file is `+0,0` and the gap before it
/// is empty.
fn hunk_first_line(start: u32, count: u32) -> u32 {
    if count == 0 {
        start + 1
    } else {
        start.max(1)
    }
}

/// The most changed lines (removed plus added) a file may have and still get
/// changed-word emphasis (`diff_view/intraline.rs`).
const MAX_EMPHASIS_LINES: usize = 20_000;

fn changed_lines(hunks: &[gitpoulpi_app::diff::EffectiveHunk]) -> usize {
    hunks
        .iter()
        .flat_map(|eh| &eh.hunk.lines)
        .filter(|l| l.origin != LineOrigin::Context)
        .count()
}

/// Turns a `FileDiff` into display rows: the hunks alone (`hunks_only`),
/// or the whole file with the hunks in place, the unchanged stretches
/// between them read from `full_file` and numbered on both sides.
///
/// Every hunk keeps its `@@` header row in both states: it carries the
/// Stage/Unstage hunk button and is what ↑/↓ land on. Unchanged rows
/// between hunks belong to no hunk (`DiffRow::default()`), so staging
/// from one reports "no hunk under the cursor".
///
/// O(rows): one walk, no per-row search. The caller caches the result
/// (`diff_content::DiffContent`).
pub(crate) fn render_diff_with(file: &FileDiff, opts: &DiffRenderOpts) -> DiffRender {
    render_diff_annotated(file, opts, None)
}

/// [`render_diff_with`] with an [`Annotator`]: its rows are put under the
/// code lines it asks for, and whatever it could not place after the last row.
pub(crate) fn render_diff_annotated(
    file: &FileDiff,
    opts: &DiffRenderOpts,
    annotator: Option<&dyn Annotator>,
) -> DiffRender {
    if !opts.lfs_raw {
        if let Some(change) = gitpoulpi_core::domain::lfs_change(file) {
            return lfs_card(&change, opts.lfs_local).with_trailing(annotator, opts);
        }
    }
    let h_offset = if opts.word_wrap { 0 } else { opts.h_offset };
    let hunks = effective_hunks(file, opts.show_whitespace);
    let full = opts.full_file.filter(|_| !opts.hunks_only);

    if file.hunks.is_empty() {
        return DiffRender::notice(msg::NOTICE_NO_CHANGES.get()).with_trailing(annotator, opts);
    }
    if hunks.is_empty() && full.is_none() {
        return DiffRender::notice(msg::NOTICE_WHITESPACE_ONLY.get())
            .with_trailing(annotator, opts);
    }

    let mut sink = Sink::default();
    let mut hunk_starts: Vec<usize> = Vec::new();

    let layout = LayoutOpts {
        word_wrap: opts.word_wrap,
        width: opts.width,
        h_offset,
        mode: opts.mode,
        // Past this many changed lines in one file the emphasis is dropped
        // wholesale: the render runs on the UI thread.
        intraline: opts.intraline && changed_lines(&hunks) <= MAX_EMPHASIS_LINES,
        annot: annotator,
    };
    // The next unchanged line on each side, 1-based.
    let (mut next_old, mut next_new) = (1u32, 1u32);
    for eh in &hunks {
        if let Some(content) = full {
            let first_new = hunk_first_line(eh.hunk.new_start, eh.hunk.new_lines);
            emit_gap(
                &mut sink,
                content,
                (next_old, next_new),
                first_new,
                opts.mode,
                layout,
            );
            next_old = hunk_first_line(eh.hunk.old_start, eh.hunk.old_lines) + eh.hunk.old_lines;
            next_new = first_new + eh.hunk.new_lines;
        }
        hunk_starts.push(sink.lines.len());
        sink.push(
            hunk_header_line(&eh.hunk),
            DiffRow::header(eh.hunk.id),
            [None, None],
        );
        match opts.mode {
            DiffViewMode::Inline => emit_unified(&mut sink, eh, layout),
            DiffViewMode::Split => emit_split(&mut sink, eh, layout),
        }
    }
    if let Some(content) = full {
        let end = content.len() as u32 + 1;
        emit_gap(
            &mut sink,
            content,
            (next_old, next_new),
            end,
            opts.mode,
            layout,
        );
    }

    sink.trailing(annotator, opts.mode, opts.width);
    debug_assert_eq!(sink.lines.len(), sink.rows.len());
    debug_assert_eq!(sink.lines.len(), sink.segs.len());
    let split_x =
        (opts.mode == DiffViewMode::Split).then(|| split_content_width(opts.width) as u16 + 8);
    DiffRender {
        lines: sink.lines,
        hunk_starts,
        rows: Rc::from(sink.rows),
        segs: Rc::from(sink.segs),
        text: Rc::new(sink.text),
        split_x,
        annot_text: Rc::new(sink.annot_text),
    }
}

impl DiffRender {
    /// This render with the annotator's leftover rows after it (the early
    /// returns: a card, a notice).
    fn with_trailing(self, annotator: Option<&dyn Annotator>, opts: &DiffRenderOpts) -> Self {
        let Some(a) = annotator else { return self };
        let mut sink = Sink::default();
        sink.trailing(Some(a), opts.mode, opts.width);
        if sink.lines.is_empty() {
            return self;
        }
        let base = self.lines.len();
        let mut lines = self.lines;
        lines.extend(sink.lines);
        let mut rows: Vec<DiffRow> = self.rows.iter().copied().collect();
        rows.extend(sink.rows);
        let mut segs: Vec<RowSegs> = self.segs.iter().copied().collect();
        segs.extend(sink.segs);
        let mut annots = (*self.annot_text).clone();
        annots.extend(sink.annot_text.into_iter().map(|(r, t)| (r + base, t)));
        Self {
            lines,
            hunk_starts: self.hunk_starts,
            rows: Rc::from(rows),
            segs: Rc::from(segs),
            text: self.text,
            split_x: self.split_x,
            annot_text: Rc::new(annots),
        }
    }
}

/// A Git LFS file's diff as a card instead of pointer text: the object's
/// old and new size and short oid, and whether the new object is in the
/// local store. Rows belong to no hunk, so only the file-level Stage /
/// Unstage in the Status pane applies.
pub(crate) fn lfs_card(
    change: &gitpoulpi_core::domain::LfsChange,
    local: Option<bool>,
) -> DiffRender {
    let lines: Vec<Line<'static>> = styled::lfs_card(change, local)
        .iter()
        .map(super::term::line_of)
        .collect();
    let n = lines.len();
    DiffRender {
        lines,
        hunk_starts: Vec::new(),
        rows: Rc::from(vec![DiffRow::default(); n]),
        segs: Rc::from(vec![[None, None]; n]),
        text: Rc::new(DiffText::default()),
        split_x: None,
        annot_text: Rc::default(),
    }
}

/// The unchanged lines `new_from .. to_new` (new-side numbers, 1-based,
/// end exclusive) of the whole file, as context rows numbered on both
/// sides. Clamped to the content: a file that changed between the diff
/// and the read renders short for one refresh.
fn emit_gap(
    sink: &mut Sink,
    content: &[String],
    (old_from, new_from): (u32, u32),
    to_new: u32,
    mode: DiffViewMode,
    layout: LayoutOpts,
) {
    let end = (to_new as usize).saturating_sub(1).min(content.len());
    let start = (new_from as usize).saturating_sub(1).min(end);
    for (k, text) in content[start..end].iter().enumerate() {
        let old = old_from + k as u32;
        let new = new_from + k as u32;
        match mode {
            DiffViewMode::Inline => {
                push_unified_line(
                    sink,
                    (&LineOrigin::Context, text),
                    (Some(old), Some(new)),
                    DiffRow::default(),
                    &[],
                    layout,
                );
                annotate(sink, Some(old), Some(new), layout);
            }
            DiffViewMode::Split => {
                let side = |number| {
                    Some(SideLine {
                        origin: &LineOrigin::Context,
                        content: text,
                        number,
                        marked: &[],
                    })
                };
                push_split_row(sink, side(old), side(new), DiffRow::default(), layout);
                annotate(sink, Some(old), Some(new), layout);
            }
        }
    }
}

fn hunk_header_line(hunk: &Hunk) -> Line<'static> {
    super::term::line_of(&styled::hunk_header(hunk))
}
