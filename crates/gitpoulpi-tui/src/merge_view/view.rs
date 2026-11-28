//! [`MergeView`]: the merge model plus what only the screen knows: scroll
//! offsets, the focused pane, the narrow layout's tab, the Output's
//! height, the per-pane highlighters and the aligned rows.
//!
//! The view never changes the model and the model never knows the screen;
//! the render (`render.rs`) reads both and the input slice (S6) drives
//! both.

use gitpoulpi_core::domain::{aligned_rows, side_changes, AlignedRows, ConflictSide, SideChanges};

use crate::editor_highlight::Highlight;

use super::layout::{self, Anchors, OutLayout, Split, TopRows};
use super::model::MergeModel;
use super::MergePane;

/// The slot of a side in the core's `[base, ours, theirs]` arrays.
pub(super) fn slot(side: ConflictSide) -> usize {
    match side {
        ConflictSide::Base => 0,
        ConflictSide::Ours => 1,
        ConflictSide::Theirs => 2,
    }
}

pub(super) const SIDES: [ConflictSide; 3] =
    [ConflictSide::Base, ConflictSide::Ours, ConflictSide::Theirs];

/// Two inputs closer than this are one held key (or one wheel spin).
pub(super) const BUSY: std::time::Duration = std::time::Duration::from_millis(150);

pub(crate) struct MergeView {
    pub(crate) model: MergeModel,
    /// `(k, n)` for the header's "file k of N" (1-based).
    pub(crate) file_pos: (usize, usize),
    /// The Base pane (diff3) is shown (wide layout).
    pub(crate) show_base: bool,
    /// "Full file": panes A and B are colored like the Split diff against
    /// the merge base. Remembered for the session by the flow.
    pub(crate) full_file: bool,
    /// Each side against the base, computed once per document: the sides
    /// never change while the view lives (a reloaded document is a new
    /// view), so this is the whole cache key.
    pub(super) changes: Option<[Option<SideChanges>; 3]>,
    /// The narrow layout's tab.
    pub(crate) tab: ConflictSide,
    pub(crate) focus: MergePane,
    pub(crate) split: Split,
    /// How the top panes share the width (dragged dividers, `Alt+arrows`).
    pub(crate) cols: super::layout::ColSplit,
    /// A one-off message for the hint row (a pick that replaced an edit,
    /// a save that failed); the key hints come back when it is `None`.
    pub(crate) note: Option<String>,
    /// First top display row and first Output row on screen.
    pub(crate) top_scroll: usize,
    /// How many columns the A / B / Base panes are scrolled sideways.
    pub(crate) h_scroll: usize,
    pub(crate) out_scroll: usize,
    pub(super) aligned: AlignedRows,
    pub(super) top: TopRows,
    /// Highlighters of `[base, ours, theirs]`, each over its side's file.
    pub(super) hl: [Highlight; 3],
    pub(super) hl_out: Highlight,
    /// The heights of the panes as last painted: `(top, output)`.
    pub(super) heights: (usize, usize),
    /// The terminal was short at the last paint (`z` toggles halves).
    pub(super) compact: bool,
    /// Highlighting did not reach what is on screen within the frame
    /// budget: the loop should keep drawing until it does.
    pub(super) pending: bool,
    /// When the last key or wheel step came, and whether it followed the one
    /// before within [`BUSY`]: a held key or a wheel spin. While that lasts
    /// the frames skip syntax highlighting (a line costs about 0.2 ms), as
    /// the Log's frames skip everything but the rows.
    pub(super) last_input: Option<std::time::Instant>,
    /// The widest line of the sides, for the sideways scroll's clamp: the
    /// sides never change while the view lives, and counting them is a pass
    /// over the whole file.
    pub(super) widest: std::cell::OnceCell<usize>,
    pub(super) repeating: bool,
    /// The Output's line count at the last paint, to move the highlighter's
    /// stale guesses when an edit adds or removes lines.
    pub(super) out_lines_seen: usize,
    /// The narrow (tabbed) layout was used at the last paint.
    pub(super) narrow: bool,
    /// What only the input knows (boxes, strip, drag, the line cursor).
    pub(super) ui: super::input::Ui,
    /// Where the panes were at the last paint, for the mouse.
    pub(super) areas: super::input::Areas,
    /// The conflict is not text: this card replaces the panes.
    pub(super) card: Option<super::cards::Card>,
    /// `ai.enabled`: the ✦ Suggest controls show (set by the window).
    pub(crate) ai_enabled: bool,
    /// An AI request is running (set by the window).
    pub(crate) ai_busy: bool,
    /// The vim count the window parsed for the key being handled (`3j`).
    pub(super) count: usize,
}

impl MergeView {
    pub(crate) fn new(model: MergeModel) -> MergeView {
        let merge = model.merge();
        let card = super::cards::Card::detect(merge);
        let sides = merge.side_lines();
        let aligned = if card.is_some() {
            AlignedRows::default()
        } else {
            aligned_rows(&merge.regions, sides)
        };
        let mut top = layout::top_rows(&aligned);
        // A side whose blocks could not be pinned to its file shows the
        // conflict blocks themselves, row by row (plan §3.3).
        for (s, degraded) in aligned.degraded.iter().enumerate() {
            if !degraded {
                continue;
            }
            let side = SIDES[s];
            let mut run: Option<(usize, usize)> = None;
            for (i, row) in aligned.rows.iter().enumerate() {
                top.block_idx[i][s] = None;
                let Some(c) = row.conflict else {
                    run = None;
                    continue;
                };
                let k = match run {
                    Some((rc, k)) if rc == c => k,
                    _ => 0,
                };
                run = Some((c, k + 1));
                if k < model.block(c, side).len() {
                    top.block_idx[i][s] = Some(k);
                }
            }
        }
        let path = merge.entry.path.clone();
        let first = |lines: Option<&[String]>| lines.and_then(|l| l.first().cloned());
        let hl = [
            crate::editor_highlight::for_file(&path, first(sides[0]).as_deref().unwrap_or("")),
            crate::editor_highlight::for_file(&path, first(sides[1]).as_deref().unwrap_or("")),
            crate::editor_highlight::for_file(&path, first(sides[2]).as_deref().unwrap_or("")),
        ];
        let hl_out = crate::editor_highlight::for_file(&path, model.output().line(0));
        MergeView {
            model,
            file_pos: (1, 1),
            show_base: false,
            full_file: false,
            changes: None,
            tab: ConflictSide::Ours,
            focus: MergePane::A,
            split: Split::DEFAULT,
            cols: Default::default(),
            note: None,
            top_scroll: 0,
            h_scroll: 0,
            out_scroll: 0,
            aligned,
            top,
            hl,
            hl_out,
            heights: (0, 0),
            compact: false,
            pending: false,
            last_input: None,
            widest: std::cell::OnceCell::new(),
            repeating: false,
            out_lines_seen: 0,
            narrow: false,
            ui: Default::default(),
            areas: Default::default(),
            card,
            ai_enabled: false,
            ai_busy: false,
            count: 1,
        }
    }

    /// Pays what the first frame would otherwise: the grammars' lazy
    /// compile. Called on the worker that builds a big file's view.
    /// Also what the first Full file toggle and the first sideways scroll
    /// would compute (a pass over the whole file each).
    pub(crate) fn warm_up(&mut self) {
        for hl in &self.hl {
            hl.warm();
        }
        self.hl_out.warm();
        self.widest_line();
        if self.card.is_none() {
            self.side_diffs();
        }
    }

    /// Swaps in a new model (the file on disk was loaded, or the view
    /// restarts from the merge) and keeps the screen state.
    pub(crate) fn replace_model(&mut self, model: MergeModel) {
        let mut fresh = MergeView::new(model);
        fresh.file_pos = self.file_pos;
        fresh.show_base = self.show_base;
        fresh.full_file = self.full_file;
        fresh.tab = self.tab;
        fresh.focus = self.focus;
        fresh.split = self.split;
        fresh.cols = self.cols;
        fresh.h_scroll = self.h_scroll;
        fresh.narrow = self.narrow;
        fresh.ai_enabled = self.ai_enabled;
        fresh.ai_busy = self.ai_busy;
        fresh.areas = self.areas;
        fresh.ui = std::mem::take(&mut self.ui);
        fresh.ui.cursor = 0;
        fresh.ui.prompt = None;
        fresh.ui.menu = None;
        *self = fresh;
    }

    /// Notes an input event for [`MergeView::input_busy`].
    pub(super) fn note_input(&mut self) {
        let now = std::time::Instant::now();
        self.repeating = self
            .last_input
            .is_some_and(|t| now.duration_since(t) < BUSY);
        self.last_input = Some(now);
    }

    /// Keys or wheel steps are coming in a stream: frames stay cheap.
    pub(super) fn input_busy(&self) -> bool {
        self.repeating && self.last_input.is_some_and(|t| t.elapsed() < BUSY)
    }

    /// A box (menu or prompt) is up and owns the pointer and the keys.
    pub(crate) fn has_popup(&self) -> bool {
        self.ui.modal()
    }

    /// The keys are text (the Output has the focus, or its find bar is
    /// up): digits are typed, not a vim count.
    pub(crate) fn types_text(&self) -> bool {
        self.focus == MergePane::Output || self.ui.find.is_some() || self.ui.modal()
    }

    /// A save is in flight.
    pub(crate) fn is_saving(&self) -> bool {
        self.ui.saving.is_some()
    }

    /// Marks a save as started (or finished, with `None`); returns what
    /// it was to do once done.
    pub(crate) fn set_saving(&mut self, then: Option<super::Then>) -> Option<super::Then> {
        std::mem::replace(&mut self.ui.saving, then)
    }

    /// Keeps what the person set up when the editor moves to another file:
    /// the Base pane, the split, the focus and the layout the mouse maps.
    pub(crate) fn carry_screen_state(&mut self, old: &MergeView) {
        self.show_base = old.show_base;
        self.full_file = old.full_file;
        self.split = old.split;
        self.cols = old.cols;
        self.h_scroll = old.h_scroll;
        self.tab = old.tab;
        self.focus = old.focus;
        self.narrow = old.narrow;
        self.areas = old.areas;
    }

    /// The sides diffed against the base (slot 0), computed on first use.
    /// `None` for a side with no file or when there is no base.
    pub(super) fn side_diffs(&mut self) -> &[Option<SideChanges>; 3] {
        if self.changes.is_none() {
            let sides = self.model.merge().side_lines();
            let mut out: [Option<SideChanges>; 3] = Default::default();
            if let Some(base) = sides[0] {
                for s in 1..3 {
                    out[s] = sides[s].map(|lines| side_changes(base, lines));
                }
            }
            self.changes = Some(out);
        }
        self.changes.as_ref().expect("just set")
    }

    /// The strip replacing the key hints, if any.
    pub(crate) fn strip(&self) -> Option<&super::input::Strip> {
        self.ui.strip.as_ref()
    }

    pub(crate) fn set_strip(&mut self, strip: Option<super::input::Strip>) {
        self.ui.strip = strip;
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.model.merge().entry.path
    }

    pub(crate) fn aligned(&self) -> &AlignedRows {
        &self.aligned
    }

    /// The loop should keep drawing: highlighting is still catching up.
    pub(crate) fn highlight_pending(&self) -> bool {
        self.pending
    }

    /// The panes the wide layout shows, left to right.
    pub(crate) fn wide_sides(&self) -> Vec<ConflictSide> {
        if self.show_base {
            vec![ConflictSide::Ours, ConflictSide::Base, ConflictSide::Theirs]
        } else {
            vec![ConflictSide::Ours, ConflictSide::Theirs]
        }
    }

    /// The size state worth remembering: the column weights and the Output
    /// share (`0` = the default). `prev_pct` is what is stored now, kept
    /// while the short-terminal halves (never stored) are showing.
    pub(crate) fn layout_pref(&self, prev_pct: u16) -> (Option<[u16; 3]>, u16) {
        let pct = match self.split {
            Split::Percent(p) if self.split != Split::DEFAULT => p,
            Split::Percent(_) => 0,
            _ => prev_pct,
        };
        (self.cols.0, pct)
    }

    /// Puts remembered sizes on a freshly opened view (values are clamped
    /// again here: the file is user-editable).
    pub(crate) fn apply_layout_pref(&mut self, cols: &[u16], pct: u16) {
        self.cols = match cols {
            [a, b, c] => super::layout::ColSplit(Some([
                (*a).clamp(1, 10_000),
                (*b).clamp(1, 10_000),
                (*c).clamp(1, 10_000),
            ])),
            _ => super::layout::ColSplit(None),
        };
        self.split = if pct == 0 {
            Split::DEFAULT
        } else {
            Split::Percent(pct.clamp(10, 85))
        };
    }

    /// A splitter or divider drag is in progress (its size is saved when
    /// it ends, not per frame).
    pub(crate) fn resizing(&self) -> bool {
        self.ui.drag.is_some()
    }

    /// `z`: the next Output height (or half, on a short terminal).
    pub(crate) fn cycle_split(&mut self) {
        self.split = self.split.cycled(self.compact);
    }

    // -- the Output's rows ---------------------------------------------------

    pub(super) fn out_layout(&self) -> OutLayout {
        let doc = self.model.output();
        let count = doc.line_count();
        let text = doc.text();
        // The empty "line" after the final newline is not a line of the
        // file: hidden unless the caret sits on it.
        let mut lines = if text.is_empty() || text.ends_with('\n') {
            count - 1
        } else {
            count
        };
        let caret_line = doc.cursor_line_col().0;
        if caret_line >= lines {
            lines = caret_line + 1;
        }
        let mut placeholders: Vec<(usize, usize)> = (0..self.model.conflict_count())
            .filter_map(|c| self.model.placeholder_line(c).map(|l| (l, c)))
            .collect();
        placeholders.sort_unstable();
        OutLayout {
            placeholders,
            lines,
            no_eol: !self.model.final_newline(),
        }
    }

    /// The region map: matching rows of the top panes and the Output.
    pub(super) fn anchors(&self, out: &OutLayout) -> Anchors {
        let mut a: Anchors = Vec::new();
        let mut push = |t: usize, o: usize| {
            let (pt, po) = a.last().copied().unwrap_or((0, 0));
            a.push((t.max(pt), o.max(po)));
        };
        for c in 0..self.model.conflict_count() {
            let (Some(&h), Some(&f)) = (self.top.header_at.get(c), self.top.footer_at.get(c))
            else {
                continue;
            };
            let Some((first, end)) = self.model.region_lines(c) else {
                continue;
            };
            let (start, len) = match out.row_of_placeholder(c) {
                Some(r) => (r, 1),
                None => (out.row_of_line(first), end - first),
            };
            push(h, start);
            push(f + 1, start + len);
        }
        push(self.top.rows.len(), out.total());
        a
    }

    fn max_top(&self) -> usize {
        self.top.rows.len().saturating_sub(self.heights.0.max(1))
    }

    fn max_out(&self, out: &OutLayout) -> usize {
        out.total().saturating_sub(self.heights.1.max(1))
    }

    /// Scrolls the A / B panes; the Output follows through the region map.
    pub(crate) fn scroll_top_by(&mut self, delta: isize) {
        let out = self.out_layout();
        self.top_scroll = self
            .top_scroll
            .saturating_add_signed(delta)
            .min(self.max_top());
        let anchors = self.anchors(&out);
        self.out_scroll =
            layout::map_through(&anchors, self.top_scroll, true).min(self.max_out(&out));
    }

    /// Scrolls the Output; the A / B panes follow.
    pub(crate) fn scroll_output_by(&mut self, delta: isize) {
        let out = self.out_layout();
        self.out_scroll = self
            .out_scroll
            .saturating_add_signed(delta)
            .min(self.max_out(&out));
        let anchors = self.anchors(&out);
        self.top_scroll = layout::map_through(&anchors, self.out_scroll, false).min(self.max_top());
    }

    /// Brings the current conflict into view in every pane, its header and
    /// its Output region one row from the top.
    pub(crate) fn reveal_current(&mut self) {
        let c = self.model.current();
        let Some(&h) = self.top.header_at.get(c) else {
            return;
        };
        let out = self.out_layout();
        let anchors = self.anchors(&out);
        self.top_scroll = h.saturating_sub(1).min(self.max_top());
        let start = layout::map_through(&anchors, h, true);
        self.out_scroll = start.saturating_sub(1).min(self.max_out(&out));
    }
}
