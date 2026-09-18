//! The merge editor's geometry: which layout fits, where every pane
//! sits, how the toolbar degrades, and the row bookkeeping the scroll sync
//! rests on (plan §2.2, §2.3, §2.9). Pure: no drawing.

use std::sync::atomic::{AtomicU16, Ordering};

use gitpoulpi_core::domain::{AlignedRows, ConflictSide};
use ratatui::layout::Rect;

use gitpoulpi_i18n::msg::merge as tr;

use crate::display_width;

use super::MergeHit;

/// The central-column width from which A | B are side by side (setting
/// `merge.side_by_side_min_width`, default 100, `0` meaning always; long
/// lines scroll sideways); narrower, one side at a time behind tabs.
/// Published process-wide like the other installed settings
/// (`editor_prefs`).
static SIDE_BY_SIDE_MIN: AtomicU16 =
    AtomicU16::new(gitpoulpi_config::DEFAULT_MERGE_SIDE_BY_SIDE_MIN_WIDTH);

/// Publishes the `[merge]` settings (at start, and after Settings changes
/// one).
pub(crate) fn install(config: &gitpoulpi_config::MergeConfig) {
    SIDE_BY_SIDE_MIN.store(config.side_by_side_min_width_clamped(), Ordering::Relaxed);
}

/// The width A | B need to be side by side.
pub(crate) fn side_by_side_min() -> u16 {
    #[cfg(test)]
    if let Some(w) = tests_override::get() {
        return w;
    }
    SIDE_BY_SIDE_MIN.load(Ordering::Relaxed)
}

/// Per-thread override of the setting, so tests that need another width do
/// not race on the process-wide one.
#[cfg(test)]
pub(crate) mod tests_override {
    use std::cell::Cell;

    thread_local! {
        static MIN: Cell<Option<u16>> = const { Cell::new(None) };
    }

    pub(crate) fn get() -> Option<u16> {
        MIN.with(Cell::get)
    }

    /// Runs `f` with `merge.side_by_side_min_width` as `width` (this
    /// thread only).
    pub(crate) fn with<R>(width: u16, f: impl FnOnce() -> R) -> R {
        let before = MIN.with(|m| m.replace(Some(width)));
        let out = f();
        MIN.with(|m| m.set(before));
        out
    }
}
/// A terminal shorter than this is "compact": `z` toggles between "top
/// only" and "Output only" instead of cycling the Output's height.
pub(crate) const COMPACT_BELOW_ROWS: u16 = 24;
/// Below this the view shows a notice and nothing else.
pub(crate) const MIN_COLS: u16 = 40;
pub(crate) const MIN_ROWS: u16 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// A / B (and Base) side by side over the Output.
    Wide,
    /// One side at a time behind a tab strip, over the Output.
    Narrow,
    /// Too small for either: a notice.
    TooSmall,
}

pub(crate) fn choose_mode(area: Rect) -> Mode {
    if area.width < MIN_COLS || area.height < MIN_ROWS {
        Mode::TooSmall
    } else if area.width >= side_by_side_min() {
        Mode::Wide
    } else {
        Mode::Narrow
    }
}

/// The share of the body the Output takes (`z` cycles it), or one half
/// alone (the short-terminal toggle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Split {
    /// The Output's height in percent of the body.
    Percent(u16),
    OutputOnly,
    TopOnly,
}

impl Split {
    pub(crate) const DEFAULT: Split = Split::Percent(40);

    /// The next state of `z`: 40% to 65% to 25% on a tall terminal, "top
    /// only" and "Output only" on a short one (plan §2.3).
    pub(crate) fn cycled(self, compact: bool) -> Split {
        if compact {
            match self {
                Split::TopOnly => Split::OutputOnly,
                _ => Split::TopOnly,
            }
        } else {
            match self {
                Split::Percent(40) => Split::Percent(65),
                Split::Percent(65) => Split::Percent(25),
                _ => Split::Percent(40),
            }
        }
    }
}

/// The smallest a top pane may be dragged to: its gutter and some text.
pub(crate) const MIN_PANE_W: u16 = crate::pane_split::MIN_PANE_W;
/// What the keyboard moves a divider by, in per mille of the panes' width (as a share of the usable width).
pub(crate) const KEY_STEP_PERMILLE: u32 = crate::pane_split::KEY_STEP_PERMILLE;

/// How the width of the top panes is shared: a weight per side (a share of 10,000), in
/// `[A, Base, B]` order (the two of them shown when Base is hidden).
/// `None` is the default, an equal share. Session state of the view: a
/// ratio, so it survives a resize of the terminal. The maths is
/// `pane_split.rs`'s, shared with the image view's before / after split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ColSplit(pub Option<[u16; 3]>);

impl ColSplit {
    /// The index of a side in the weights.
    fn slot(side: ConflictSide) -> usize {
        match side {
            ConflictSide::Ours => 0,
            ConflictSide::Base => 1,
            ConflictSide::Theirs => 2,
        }
    }

    /// The widths (in columns) of the panes for `sides`, left to right,
    /// summing to `usable`. Every pane keeps [`MIN_PANE_W`] when the room
    /// allows it; a narrower row falls back to the equal share.
    pub(crate) fn widths(self, usable: u16, sides: &[ConflictSide]) -> Vec<u16> {
        let weights: Option<Vec<u16>> = self
            .0
            .map(|w| sides.iter().map(|s| w[Self::slot(*s)]).collect());
        crate::pane_split::widths(weights.as_deref(), usable, sides.len())
    }

    /// The split that gives the panes `widths` (what the drag and the
    /// keys settle on): the weights of the shown sides become their
    /// shares, a hidden Base keeps what it had.
    pub(crate) fn with_widths(self, sides: &[ConflictSide], widths: &[u16]) -> ColSplit {
        let shares = crate::pane_split::weights_for(widths);
        let mut w = self.0.unwrap_or([5000, 5000, 5000]);
        for (side, share) in sides.iter().zip(shares) {
            w[Self::slot(*side)] = share;
        }
        ColSplit(Some(w))
    }

    /// Moves divider `i` (between pane `i` and `i + 1`) so that pane `i`
    /// is `left` columns wide, the pair keeping its total and each pane
    /// at least [`MIN_PANE_W`]. `current` are the widths painted now.
    pub(crate) fn with_divider(
        self,
        sides: &[ConflictSide],
        current: &[u16],
        i: usize,
        left: i32,
    ) -> ColSplit {
        if current.len() != sides.len() {
            return self;
        }
        match crate::pane_split::move_divider(current, i, left) {
            Some(widths) => self.with_widths(sides, &widths),
            None => self,
        }
    }
}

/// Where everything is for one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Geometry {
    pub mode: Mode,
    /// The bordered block (the whole area but the hint row).
    pub outer: Rect,
    /// The key-hint row under the block.
    pub hint: Rect,
    pub toolbar: Rect,
    /// The row above the top panes: pane titles (wide) or the tab strip
    /// (narrow). Also the rule the pane separators hang from.
    pub top_title: Rect,
    /// Each visible top pane: its side and its content rect (below its
    /// title). Wide shows A and B (and Base between them when asked);
    /// narrow shows the one tab.
    pub panes: Vec<(ConflictSide, Rect)>,
    /// The union of the pane contents.
    pub top_content: Rect,
    /// The draggable separators between wide panes, left to right: one
    /// column wide, from the title rule down to the Output's.
    pub dividers: Vec<Rect>,
    /// The Output's title row, which is also the splitter handle.
    pub out_title: Rect,
    pub out_content: Rect,
}

impl Geometry {
    /// Lays out `area`. `sides` are the top panes to show (wide), `tab`
    /// the one shown (narrow).
    pub(crate) fn compute(
        area: Rect,
        split: Split,
        cols: ColSplit,
        sides: &[ConflictSide],
        tab: ConflictSide,
    ) -> Geometry {
        let mode = choose_mode(area);
        let hint = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
        let outer = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
        let inner = Rect::new(
            outer.x + 1,
            outer.y + 1,
            outer.width.saturating_sub(2),
            outer.height.saturating_sub(2),
        );
        let toolbar = Rect::new(inner.x, inner.y, inner.width, 1.min(inner.height));
        let top_title = Rect::new(inner.x, inner.y + 1, inner.width, 1.min(inner.height - 1));
        // Rows left for the two contents: the toolbar and the two title
        // rows are fixed.
        let avail = inner.height.saturating_sub(3);
        let out_h = match split {
            Split::OutputOnly => avail,
            Split::TopOnly => 0,
            Split::Percent(p) => {
                let want = (u32::from(avail) * u32::from(p) / 100) as u16;
                // Keep at least 3 rows for each half when there is room.
                want.clamp(3.min(avail), avail.saturating_sub(3).max(3.min(avail)))
            }
        };
        let top_h = avail - out_h;
        let top_y = top_title.bottom();
        let top_content = Rect::new(inner.x, top_y, inner.width, top_h);
        let out_title = Rect::new(inner.x, top_content.bottom(), inner.width, 1);
        let out_content = Rect::new(inner.x, out_title.bottom(), inner.width, out_h);

        let shown: Vec<ConflictSide> = match mode {
            Mode::Wide => sides.to_vec(),
            _ => vec![tab],
        };
        let n = shown.len().max(1) as u16;
        // One separator column between panes.
        let usable = top_content.width.saturating_sub(n - 1);
        let widths = cols.widths(usable, &shown);
        let mut x = top_content.x;
        let mut panes = Vec::new();
        let mut dividers = Vec::new();
        for (side, w) in shown.into_iter().zip(widths) {
            panes.push((side, Rect::new(x, top_content.y, w, top_content.height)));
            if mode == Mode::Wide && top_content.height > 0 {
                dividers.push(Rect::new(
                    x + w,
                    top_title.y,
                    1,
                    top_content.height + (top_content.y - top_title.y),
                ));
            }
            x += w + 1;
        }
        dividers.pop();
        Geometry {
            mode,
            outer,
            hint,
            toolbar,
            top_title,
            panes,
            top_content,
            dividers,
            out_title,
            out_content,
        }
    }

    /// The title cell of a top pane (the row above its content).
    pub(crate) fn pane_title(&self, content: Rect) -> Rect {
        Rect::new(content.x, self.top_title.y, content.width, 1)
    }
}

// -- the toolbar ---------------------------------------------------------

/// One thing on the toolbar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolItem {
    Button {
        label: String,
        hit: MergeHit,
        kind: ButtonKind,
    },
    Text(String),
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonKind {
    Normal,
    /// The primary action.
    Primary,
    /// A bare glyph with no fill at rest (the close cross).
    Plain,
    /// A toggle that is on.
    Toggled,
    /// An AI control: drawn with the sparkle in AI purple.
    Ai,
}

impl ToolItem {
    pub(crate) fn width(&self) -> u16 {
        match self {
            ToolItem::Button { label, .. } => display_width(label) as u16 + 2,
            ToolItem::Text(t) => display_width(t) as u16,
            ToolItem::Separator => 3,
        }
    }
}

/// The toolbar at one degradation level: items from the left, and the
/// right-aligned group.
pub(crate) struct ToolbarPlan {
    pub left: Vec<ToolItem>,
    pub right: Vec<ToolItem>,
}

/// What the toolbar needs to know about the model.
pub(crate) struct ToolbarInput {
    /// `Conflict 2 / 5 · 3 left`.
    pub counter_long: String,
    /// `2/5`.
    pub counter_short: String,
    pub base_on: bool,
    /// Whether a Base pane can be toggled (the file has a base).
    pub base_available: bool,
    /// The whole-file diff coloring is on.
    pub full_file: bool,
    /// AI is enabled: the ✦ Suggest button shows.
    pub ai: bool,
}

const GAP: u16 = 1;

fn btn(label: &str, hit: MergeHit, kind: ButtonKind) -> ToolItem {
    ToolItem::Button {
        label: label.to_string(),
        hit,
        kind,
    }
}

/// The toolbar at `level` (0 = everything, 3 = the least that still
/// works). Take all and Base move into the `…` menu from level 2 on.
pub(crate) fn toolbar_at(level: u8, input: &ToolbarInput) -> ToolbarPlan {
    let mut left = vec![
        btn("\u{25B2}", MergeHit::PrevConflict, ButtonKind::Normal),
        btn("\u{25BC}", MergeHit::NextConflict, ButtonKind::Normal),
        ToolItem::Text(format!(
            " {} ",
            if level >= 3 {
                &input.counter_short
            } else {
                &input.counter_long
            }
        )),
    ];
    if level <= 1 {
        left.push(ToolItem::Separator);
        left.push(ToolItem::Text(
            if level == 0 {
                tr::TAKE_ALL.get()
            } else {
                tr::TAKE_ALL_SHORT.get()
            }
            .to_string(),
        ));
        left.push(btn(
            if level == 0 { tr::TAKE_A.get() } else { "A" },
            MergeHit::TakeAllA,
            ButtonKind::Normal,
        ));
        left.push(btn(
            if level == 0 { tr::TAKE_B.get() } else { "B" },
            MergeHit::TakeAllB,
            ButtonKind::Normal,
        ));
        left.push(btn(
            tr::RESET_ALL.get(),
            MergeHit::ResetAll,
            ButtonKind::Normal,
        ));
        left.push(ToolItem::Separator);
        if input.base_available {
            left.push(btn(
                tr::BASE.get(),
                MergeHit::ToggleBase,
                if input.base_on {
                    ButtonKind::Toggled
                } else {
                    ButtonKind::Normal
                },
            ));
        }
        left.push(btn(
            tr::FULL_FILE.get(),
            MergeHit::ToggleFullFile,
            if input.full_file {
                ButtonKind::Toggled
            } else {
                ButtonKind::Normal
            },
        ));
        if input.ai {
            left.push(btn(
                &super::input::ai_label(tr::AI_SUGGEST.get()),
                MergeHit::AiSuggest,
                ButtonKind::Ai,
            ));
        }
    } else {
        left.push(btn("\u{2026}", MergeHit::More, ButtonKind::Normal));
    }
    let mut right = Vec::new();
    if level <= 1 {
        right.push(btn(tr::SAVE.get(), MergeHit::Save, ButtonKind::Normal));
    }
    right.push(btn(
        &match level {
            0 => tr::SAVE_RESOLVE_LONG.get().to_string(),
            1 | 2 => tr::SAVE_RESOLVE.get().to_string(),
            _ => format!("\u{2713} {}", tr::SAVE_SHORT.get()),
        },
        MergeHit::SaveResolve,
        ButtonKind::Primary,
    ));
    right.push(btn("\u{2715}", MergeHit::Close, ButtonKind::Plain));
    ToolbarPlan { left, right }
}

fn plan_width(items: &[ToolItem]) -> u16 {
    let mut w = 0;
    let mut prev_button = false;
    for it in items {
        let is_button = matches!(it, ToolItem::Button { .. });
        if prev_button && is_button {
            w += GAP;
        }
        w += it.width();
        prev_button = is_button;
    }
    w
}

/// The first level whose two groups fit in `width` (the last one when
/// none does).
pub(crate) fn toolbar_for(width: u16, input: &ToolbarInput) -> ToolbarPlan {
    let mut plan = toolbar_at(3, input);
    for level in 0..=3 {
        let p = toolbar_at(level, input);
        let need = 1 + plan_width(&p.left) + 2 + plan_width(&p.right) + 1;
        if need <= width {
            plan = p;
            break;
        }
        plan = p;
    }
    plan
}

/// Lays `items` out left to right from `x`; returns each item with its
/// rect. Buttons separated by one blank column.
pub(crate) fn place_items(items: &[ToolItem], mut x: u16, y: u16) -> Vec<(Rect, ToolItem)> {
    let mut out = Vec::new();
    let mut prev_button = false;
    for it in items {
        let is_button = matches!(it, ToolItem::Button { .. });
        if prev_button && is_button {
            x += GAP;
        }
        let w = it.width();
        out.push((Rect::new(x, y, w, 1), it.clone()));
        x += w;
        prev_button = is_button;
    }
    out
}

/// The width of a placed group.
pub(crate) fn group_width(items: &[ToolItem]) -> u16 {
    plan_width(items)
}

// -- display rows ----------------------------------------------------------

/// One row of the top panes. The aligned rows plus a header above and a
/// footer below each conflict block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DRow {
    Header {
        conflict: usize,
    },
    /// An index into [`AlignedRows::rows`].
    Line {
        aligned: usize,
    },
    Footer {
        conflict: usize,
    },
}

/// The display rows and, per aligned row, the block line index it holds
/// for each side (`[base, ours, theirs]`): the line number *within the
/// conflict's block*, which is what picks are keyed on.
pub(crate) struct TopRows {
    pub rows: Vec<DRow>,
    pub block_idx: Vec<[Option<usize>; 3]>,
    /// Per conflict: the display index of its header and of its footer.
    pub header_at: Vec<usize>,
    pub footer_at: Vec<usize>,
}

pub(crate) fn top_rows(aligned: &AlignedRows) -> TopRows {
    let mut rows = Vec::with_capacity(aligned.rows.len() + 8);
    let mut block_idx = vec![[None; 3]; aligned.rows.len()];
    let mut header_at = Vec::new();
    let mut footer_at = Vec::new();
    let mut counts = [0usize; 3];
    let mut open: Option<usize> = None;
    for (i, row) in aligned.rows.iter().enumerate() {
        if open != row.conflict {
            if let Some(c) = open.take() {
                footer_at.resize(c + 1, 0);
                footer_at[c] = rows.len();
                rows.push(DRow::Footer { conflict: c });
            }
            if let Some(c) = row.conflict {
                header_at.resize(c + 1, 0);
                header_at[c] = rows.len();
                rows.push(DRow::Header { conflict: c });
                counts = [0; 3];
                open = Some(c);
            }
        }
        if row.conflict.is_some() {
            for s in 0..3 {
                if row.lines[s].is_some() {
                    block_idx[i][s] = Some(counts[s]);
                    counts[s] += 1;
                }
            }
        }
        rows.push(DRow::Line { aligned: i });
    }
    if let Some(c) = open {
        footer_at.resize(c + 1, 0);
        footer_at[c] = rows.len();
        rows.push(DRow::Footer { conflict: c });
    }
    TopRows {
        rows,
        block_idx,
        header_at,
        footer_at,
    }
}

// -- the Output's rows --------------------------------------------------------

/// What an Output screen row shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ORow {
    Line(usize),
    /// The virtual `‹ Conflict n · nothing picked yet ›` row.
    Placeholder(usize),
    /// The note under a file with no final newline.
    NoEol,
}

/// The Output's rows: the document's lines with the placeholders of
/// unpicked conflicts woven in, and the "no newline" note at the end.
pub(crate) struct OutLayout {
    /// `(line, conflict)`, sorted by line: the placeholder sits before
    /// that line.
    pub placeholders: Vec<(usize, usize)>,
    /// Document lines shown (the empty one after the final newline is
    /// hidden unless the caret is on it).
    pub lines: usize,
    pub no_eol: bool,
}

impl OutLayout {
    pub(crate) fn total(&self) -> usize {
        self.lines + self.placeholders.len() + usize::from(self.no_eol)
    }

    /// The row of document line `line`.
    pub(crate) fn row_of_line(&self, line: usize) -> usize {
        line + self.placeholders.partition_point(|(l, _)| *l <= line)
    }

    /// The row of conflict `c`'s placeholder.
    pub(crate) fn row_of_placeholder(&self, c: usize) -> Option<usize> {
        // Sorted by line, and the conflicts come in file order: a binary
        // search (a file with thousands of conflicts asks for every one
        // of them on every frame).
        let j = self.placeholders.partition_point(|(_, pc)| *pc < c);
        self.placeholders
            .get(j)
            .filter(|(_, pc)| *pc == c)
            .map(|(l, _)| l + j)
    }

    pub(crate) fn row_at(&self, row: usize) -> Option<ORow> {
        let mut shift = 0;
        for (j, (l, c)) in self.placeholders.iter().enumerate() {
            let vr = l + j;
            match vr.cmp(&row) {
                std::cmp::Ordering::Equal => return Some(ORow::Placeholder(*c)),
                std::cmp::Ordering::Less => shift = j + 1,
                std::cmp::Ordering::Greater => break,
            }
        }
        let line = row - shift;
        if line < self.lines {
            Some(ORow::Line(line))
        } else if self.no_eol && row + 1 == self.total() {
            Some(ORow::NoEol)
        } else {
            None
        }
    }
}

// -- scroll sync -----------------------------------------------------------

/// The region map: matching rows of the top panes and the Output, as
/// `(top_row, out_row)` points, nondecreasing in both. Between two points
/// the rows are paired linearly.
pub(crate) type Anchors = Vec<(usize, usize)>;

/// `x` mapped through `anchors` from one axis to the other (`forward`:
/// top to Output).
pub(crate) fn map_through(anchors: &[(usize, usize)], x: usize, forward: bool) -> usize {
    let key = |p: &(usize, usize)| if forward { p.0 } else { p.1 };
    let val = |p: &(usize, usize)| if forward { p.1 } else { p.0 };
    let mut prev = (0usize, 0usize);
    for p in anchors {
        if key(p) >= x {
            let (k0, v0) = (key(&prev), val(&prev));
            let (k1, v1) = (key(p), val(p));
            if k1 <= k0 {
                return v1;
            }
            return v0 + (x - k0) * v1.saturating_sub(v0) / (k1 - k0);
        }
        prev = *p;
    }
    // Past the last point: keep the same distance.
    val(&prev) + (x - key(&prev))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> ToolbarInput {
        ToolbarInput {
            counter_long: "Conflict 2 / 5 \u{b7} 3 left".into(),
            counter_short: "2/5".into(),
            base_on: false,
            base_available: true,
            full_file: false,
            ai: false,
        }
    }

    #[test]
    fn mode_thresholds() {
        let m = |w, h| choose_mode(Rect::new(0, 0, w, h));
        // The default threshold (setting `merge.side_by_side_min_width`,
        // 100): side by side from there, tabs below, the notice under 40
        // columns or 14 rows whatever the setting.
        tests_override::with(100, || {
            assert_eq!(m(100, 24), Mode::Wide);
            assert_eq!(m(100, 16), Mode::Wide);
            assert_eq!(m(99, 24), Mode::Narrow);
            assert_eq!(m(60, 14), Mode::Narrow);
            assert_eq!(m(40, 14), Mode::Narrow);
            assert_eq!(m(39, 30), Mode::TooSmall);
            assert_eq!(m(120, 13), Mode::TooSmall);
        });
        // A lower threshold, and `0` meaning always side by side.
        tests_override::with(60, || {
            assert_eq!(m(60, 14), Mode::Wide);
            assert_eq!(m(59, 30), Mode::Narrow);
        });
        tests_override::with(0, || {
            assert_eq!(m(40, 14), Mode::Wide);
            assert_eq!(m(39, 14), Mode::TooSmall);
        });
    }

    #[test]
    fn split_cycles_ratios_or_halves() {
        let s = Split::DEFAULT;
        assert_eq!(s.cycled(false), Split::Percent(65));
        assert_eq!(s.cycled(false).cycled(false), Split::Percent(25));
        assert_eq!(s.cycled(false).cycled(false).cycled(false), Split::DEFAULT);
        assert_eq!(s.cycled(true), Split::TopOnly);
        assert_eq!(s.cycled(true).cycled(true), Split::OutputOnly);
        assert_eq!(s.cycled(true).cycled(true).cycled(true), Split::TopOnly);
    }

    #[test]
    fn geometry_tiles_the_block_without_overlap() {
        let area = Rect::new(0, 0, 120, 30);
        let g = Geometry::compute(
            area,
            Split::DEFAULT,
            ColSplit::default(),
            &[ConflictSide::Ours, ConflictSide::Theirs],
            ConflictSide::Ours,
        );
        assert_eq!(g.mode, Mode::Wide);
        assert_eq!(g.outer.height + g.hint.height, 30);
        // toolbar, title, content, out title, out content, inside the border.
        assert_eq!(g.toolbar.y, 1);
        assert_eq!(g.top_title.y, 2);
        assert_eq!(g.top_content.y, 3);
        assert_eq!(g.out_title.y, g.top_content.bottom());
        assert_eq!(g.out_content.bottom(), g.outer.bottom() - 1);
        assert_eq!(g.panes.len(), 2);
        let (a, b) = (g.panes[0].1, g.panes[1].1);
        assert_eq!(a.width + 1 + b.width, g.top_content.width);
        assert_eq!(b.x, a.right() + 1);
    }

    #[test]
    fn geometry_narrow_has_one_pane_and_base_adds_a_third_when_wide() {
        let area = Rect::new(0, 0, 50, 24);
        let g = Geometry::compute(
            area,
            Split::DEFAULT,
            ColSplit::default(),
            &[ConflictSide::Ours, ConflictSide::Theirs],
            ConflictSide::Theirs,
        );
        assert_eq!(g.mode, Mode::Narrow);
        assert_eq!(g.panes.len(), 1);
        assert_eq!(g.panes[0].0, ConflictSide::Theirs);
        let wide = Geometry::compute(
            Rect::new(0, 0, 130, 30),
            Split::DEFAULT,
            ColSplit::default(),
            &[ConflictSide::Ours, ConflictSide::Base, ConflictSide::Theirs],
            ConflictSide::Ours,
        );
        assert_eq!(wide.panes.len(), 3);
        let widths: u16 = wide.panes.iter().map(|(_, r)| r.width).sum();
        assert_eq!(widths + 2, wide.top_content.width);
    }

    #[test]
    fn output_only_and_top_only_give_the_whole_body_to_one_half() {
        let a = Rect::new(0, 0, 80, 20);
        let g = Geometry::compute(
            a,
            Split::OutputOnly,
            ColSplit::default(),
            &[],
            ConflictSide::Ours,
        );
        assert_eq!(g.top_content.height, 0);
        assert!(g.out_content.height > 8);
        let g = Geometry::compute(
            a,
            Split::TopOnly,
            ColSplit::default(),
            &[],
            ConflictSide::Ours,
        );
        assert_eq!(g.out_content.height, 0);
    }

    #[test]
    fn toolbar_degrades_with_width_and_keeps_save_and_close() {
        let widths = |w| {
            let p = toolbar_for(w, &input());
            let labels: Vec<String> = p
                .left
                .iter()
                .chain(&p.right)
                .filter_map(|i| match i {
                    ToolItem::Button { label, .. } => Some(label.clone()),
                    _ => None,
                })
                .collect();
            labels
        };
        let full = widths(140);
        assert!(full.contains(&"Save & mark resolved".to_string()));
        assert!(full.contains(&"A Current".to_string()));
        let mid = widths(112);
        assert!(mid.contains(&"Save & resolve".to_string()), "{mid:?}");
        assert!(mid.contains(&"A".to_string()));
        let small = widths(60);
        assert!(small.contains(&"\u{2026}".to_string()), "{small:?}");
        assert!(!small.contains(&"A".to_string()));
        for w in [60, 80, 100, 112, 140] {
            let l = widths(w);
            assert!(l.contains(&"\u{2715}".to_string()), "close at {w}");
            assert!(
                l.iter().any(|b| b.contains("Save")),
                "a save button at {w}: {l:?}"
            );
        }
    }

    const AB: [ConflictSide; 2] = [ConflictSide::Ours, ConflictSide::Theirs];
    const ABC: [ConflictSide; 3] = [ConflictSide::Ours, ConflictSide::Base, ConflictSide::Theirs];

    #[test]
    fn default_column_split_is_equal_and_exact() {
        assert_eq!(ColSplit::default().widths(99, &AB), vec![50, 49]);
        assert_eq!(ColSplit::default().widths(100, &ABC), vec![34, 33, 33]);
    }

    #[test]
    fn dragging_a_divider_clamps_to_the_minimum_width() {
        let cur = [50u16, 49];
        let far_left = ColSplit::default().with_divider(&AB, &cur, 0, -40);
        let w = far_left.widths(99, &AB);
        assert_eq!(w, vec![MIN_PANE_W, 99 - MIN_PANE_W]);
        let far_right = ColSplit::default().with_divider(&AB, &cur, 0, 500);
        let w = far_right.widths(99, &AB);
        assert_eq!(w, vec![99 - MIN_PANE_W, MIN_PANE_W]);
    }

    #[test]
    fn column_split_is_a_ratio_and_sums_to_the_room() {
        let s = ColSplit::default().with_divider(&AB, &[50, 49], 0, 30);
        for usable in [60u16, 99, 150, 297] {
            let w = s.widths(usable, &AB);
            assert_eq!(w.iter().sum::<u16>(), usable, "{usable}");
            assert!(w[0] < w[1] && w.iter().all(|x| *x >= MIN_PANE_W), "{w:?}");
        }
        // Too narrow for minimum widths: the equal share.
        assert_eq!(s.widths(20, &AB), vec![10, 10]);
    }

    #[test]
    fn a_hidden_base_keeps_its_weight_and_three_panes_sum() {
        let s = ColSplit::default().with_divider(&ABC, &[34, 33, 33], 1, 20);
        let w = s.widths(100, &ABC);
        assert_eq!(w.iter().sum::<u16>(), 100);
        assert!(w[1] < w[2], "{w:?}");
        let two = s.widths(100, &AB);
        assert_eq!(two.iter().sum::<u16>(), 100);
    }

    #[test]
    fn wide_geometry_has_one_divider_per_gap_and_none_when_narrow() {
        let area = Rect::new(0, 0, 130, 30);
        let g = Geometry::compute(
            area,
            Split::DEFAULT,
            ColSplit::default(),
            &AB,
            ConflictSide::Ours,
        );
        assert_eq!(g.dividers.len(), 1);
        assert_eq!(g.dividers[0].x, g.panes[0].1.right());
        assert_eq!(g.dividers[0].y, g.top_title.y);
        assert_eq!(g.dividers[0].bottom(), g.top_content.bottom());
        let g = Geometry::compute(
            area,
            Split::DEFAULT,
            ColSplit::default(),
            &ABC,
            ConflictSide::Ours,
        );
        assert_eq!(g.dividers.len(), 2);
        let narrow = Geometry::compute(
            Rect::new(0, 0, 80, 30),
            Split::DEFAULT,
            ColSplit::default(),
            &AB,
            ConflictSide::Ours,
        );
        assert!(narrow.dividers.is_empty());
    }

    #[test]
    fn toolbar_never_overflows_when_a_level_fits() {
        for w in 60..160 {
            let p = toolbar_for(w, &input());
            let need = 2 + group_width(&p.left) + group_width(&p.right);
            assert!(need <= w, "level overflows at {w}: {need}");
        }
    }

    #[test]
    fn top_rows_frame_each_conflict_and_number_block_lines_per_side() {
        use gitpoulpi_core::domain::{AlignedRow, RowMark};
        let row = |c: Option<usize>, lines: [Option<usize>; 3]| AlignedRow {
            region: 0,
            conflict: c,
            lines,
            marks: [RowMark::Same; 3],
        };
        let aligned = AlignedRows {
            rows: vec![
                row(None, [Some(0), Some(0), Some(0)]),
                row(Some(0), [None, Some(1), Some(1)]),
                row(Some(0), [None, Some(2), None]),
                row(None, [Some(1), Some(3), Some(2)]),
                row(Some(1), [None, None, Some(3)]),
            ],
            degraded: [false; 3],
        };
        let t = top_rows(&aligned);
        assert_eq!(
            t.rows,
            vec![
                DRow::Line { aligned: 0 },
                DRow::Header { conflict: 0 },
                DRow::Line { aligned: 1 },
                DRow::Line { aligned: 2 },
                DRow::Footer { conflict: 0 },
                DRow::Line { aligned: 3 },
                DRow::Header { conflict: 1 },
                DRow::Line { aligned: 4 },
                DRow::Footer { conflict: 1 },
            ]
        );
        assert_eq!(t.block_idx[1], [None, Some(0), Some(0)]);
        assert_eq!(t.block_idx[2], [None, Some(1), None]);
        assert_eq!(t.block_idx[4], [None, None, Some(0)]);
        assert_eq!(t.header_at, vec![1, 6]);
        assert_eq!(t.footer_at, vec![4, 8]);
    }

    #[test]
    fn out_layout_weaves_placeholders_and_finds_rows_both_ways() {
        let l = OutLayout {
            placeholders: vec![(2, 0), (4, 1)],
            lines: 5,
            no_eol: true,
        };
        assert_eq!(l.total(), 8);
        // lines 0 1, [ph0], 2 3, [ph1], 4, note
        let rows: Vec<_> = (0..8).map(|r| l.row_at(r)).collect();
        assert_eq!(
            rows,
            vec![
                Some(ORow::Line(0)),
                Some(ORow::Line(1)),
                Some(ORow::Placeholder(0)),
                Some(ORow::Line(2)),
                Some(ORow::Line(3)),
                Some(ORow::Placeholder(1)),
                Some(ORow::Line(4)),
                Some(ORow::NoEol),
            ]
        );
        assert_eq!(l.row_at(8), None);
        assert_eq!(l.row_of_line(2), 3);
        assert_eq!(l.row_of_line(4), 6);
        assert_eq!(l.row_of_placeholder(1), Some(5));
        assert_eq!(l.row_of_placeholder(7), None);
    }

    #[test]
    fn region_map_is_piecewise_linear_in_both_directions() {
        // top rows 0..10 map to output rows 0..6; the conflict block is
        // top 3..7 and output 2..4.
        let anchors = vec![(3, 2), (7, 4), (10, 6)];
        assert_eq!(map_through(&anchors, 0, true), 0);
        assert_eq!(map_through(&anchors, 3, true), 2);
        assert_eq!(map_through(&anchors, 5, true), 3);
        assert_eq!(map_through(&anchors, 7, true), 4);
        assert_eq!(map_through(&anchors, 10, true), 6);
        assert_eq!(map_through(&anchors, 2, false), 3);
        assert_eq!(map_through(&anchors, 4, false), 7);
        assert_eq!(map_through(&anchors, 6, false), 10);
        // Past the end keeps the distance.
        assert_eq!(map_through(&anchors, 12, true), 8);
    }
}
