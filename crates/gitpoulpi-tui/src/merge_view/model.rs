//! The merge editor's state: picks per conflict, the Output document with
//! an anchor per conflict region, one undo stack, and the checks before a
//! save. Pure: no drawing, no keys, no IPC (plan §2.4, §2.5, §3, §7).
//!
//! # Shape
//!
//! A [`MergeModel`] is built from a core [`MergeDocument`] (git's own
//! 3-way merge cut into regions). The Output is a [`Doc`] holding the
//! merged text with every conflict resolved as its picks say. Each
//! conflict owns a byte range of that text, its **anchor**; the range
//! always covers whole lines, trailing `\n` included.
//!
//! * A **pick** (toggle a line or a chunk, take A / B / both, swap the
//!   order, clear, take all) recomputes its conflict's lines through the
//!   core's [`ConflictPicks::lines_for`] and replaces the anchor's text.
//! * **Typing** goes through [`MergeModel::edit`]: the `Doc` logs each
//!   change and the anchors follow it (an edit before a region shifts it,
//!   an edit touching it resizes it). A region whose text no longer equals
//!   what its picks produce is **Edited**: that is computed, never stored,
//!   so typing the original text back un-edits it, and a later pick
//!   replaces the edit (with a one-time hint).
//! * **Undo** is the `Doc`'s own stack. Every step, a pick or a run of
//!   typing, carries a tag, and the model keeps the picks and anchors from
//!   before and after each tag, so Ctrl+Z restores text, checkboxes and
//!   anchors together.
//!
//! # The final newline
//!
//! The Output text always ends with `\n` after its last line, so every
//! region is whole lines and a pick at the end of the file needs no
//! special case. The merge's own "no newline at end of file" is a flag
//! applied when the file is produced ([`MergeModel::output_text`],
//! [`MergeModel::output_file`]), not a missing byte in the editor.
//!
//! # Known edges (for the slices that drive this)
//!
//! * A conflict with nothing picked is an *empty* region. Typing into it
//!   with no newline joins the next merged line to it (the region then
//!   owns that line). The Output pane (S7) should insert a line break
//!   first when the caret sits on a placeholder.
//! * Typed lines *outside* any conflict are detected per gap between
//!   conflicts, not per line: [`MergeModel::origin_of_line`] reports
//!   `Edited` for every line of a gap whose text differs from git's merge.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use gitpoulpi_core::domain::{
    ConflictPicks, ConflictSide, Eol, MergeDocument, MergeRegion, Oid, Pick, WorktreeFileState,
};

use crate::editor_buffer::{Doc, TextChange};

/// Where a conflict stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConflictStatus {
    /// Nothing picked and nothing typed: it would save as empty.
    Unpicked,
    /// Resolved through picks (an explicit "none" counts).
    Picked,
    /// The Output region was typed in; the picks no longer describe it.
    Edited,
}

/// Where an Output line came from, for the gutter letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    /// Git merged it by itself (no letter).
    Common,
    /// Typed (the pencil).
    Edited,
    /// Picked from this side: `A` (Ours), `B` (Theirs) or `O` (Base).
    Side(ConflictSide),
}

/// How much of one side of a conflict is picked: the chunk checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChunkPick {
    /// `[ ]`
    None,
    /// `[-]`
    Some,
    /// `[x]`
    All,
}

/// A one-time status hint left by an operation. The text is the view's:
/// it names the undo key from the keymap, so it is not spelled here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hint {
    /// A pick replaced a typed edit: "replaced your edit · <undo> restores".
    ReplacedEdit,
}

/// What a save should ask about first (plan §2.4 "Saving").
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SaveChecks {
    /// Conflicts (0-based) with nothing picked and nothing typed.
    pub unpicked: Vec<usize>,
    /// Output lines (1-based) that look like conflict markers.
    pub marker_lines: Vec<usize>,
}

impl SaveChecks {
    pub(crate) fn is_clean(&self) -> bool {
        self.unpicked.is_empty() && self.marker_lines.is_empty()
    }
}

/// The "k / n · m left" counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counter {
    /// The current conflict, 1-based (0 when there are none).
    pub current: usize,
    pub total: usize,
    /// Conflicts still [`ConflictStatus::Unpicked`].
    pub left: usize,
}

/// One conflict's picks and where its resolution sits in the Output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConflictState {
    picks: ConflictPicks,
    /// Something was chosen on purpose, "none" included.
    decided: bool,
    /// Byte range of the region in the Output text, whole lines.
    start: usize,
    end: usize,
}

/// What to write to disk for the Output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputFile {
    /// The text with the file's line endings and final-newline choice
    /// applied, without a BOM.
    pub text: String,
    pub bom: bool,
    pub eol: Eol,
    /// The fallback encoding the sides were decoded with: the daemon
    /// encodes `text` back with it (`UseEncoded`). `None` is UTF-8.
    pub encoding: Option<String>,
}

impl OutputFile {
    /// The text as `ConflictResolution::UseContent` takes it: the BOM,
    /// when there is one, as the character U+FEFF in front.
    pub(crate) fn content_string(&self) -> String {
        if self.bom {
            format!("\u{feff}{}", self.text)
        } else {
            self.text.clone()
        }
    }

    /// The bytes of a UTF-8 file (BOM first).
    pub(crate) fn utf8_bytes(&self) -> Vec<u8> {
        self.content_string().into_bytes()
    }
}

/// What is derived from the Output text and the anchors, kept until either
/// changes (the Output's version moves on every edit; [`MergeModel::touch`]
/// covers the anchors). Painting asks for a conflict's status and for a
/// gap's "is it as git merged it" once per visible line: without this a
/// gap of a hundred thousand lines is compared with its original once per
/// row, on every frame.
#[derive(Default)]
struct Derived {
    key: (u64, u64),
    status: Vec<Option<ConflictStatus>>,
    gap_ok: Vec<Option<bool>>,
    origins: Vec<Option<std::sync::Arc<Vec<ConflictSide>>>>,
}

type History = HashMap<u64, (Vec<ConflictState>, Vec<ConflictState>)>;

/// The editor's state for one conflicted file.
pub(crate) struct MergeModel {
    merge: MergeDocument,
    output: Doc,
    states: Vec<ConflictState>,
    /// The text git merged between the conflicts: `n + 1` gaps, as the
    /// Output text would hold them.
    gaps: Vec<String>,
    current: usize,
    history: History,
    next_tag: u64,
    eol: Eol,
    final_newline: bool,
    /// The Output was loaded from the file on disk (plan §2.5 case 2):
    /// there are no regions, so no picks.
    outside: bool,
    hint: Option<Hint>,
    /// The decisions as last saved, for [`MergeModel::is_dirty`].
    saved: Vec<(ConflictPicks, bool)>,
    /// Bumped whenever the anchors or picks change (see [`Derived`]).
    generation: u64,
    derived: RefCell<Derived>,
}

fn lines_text(lines: &[String]) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(l);
        s.push('\n');
    }
    s
}

/// `picks` with `order` holding exactly the sides that have a pick, the
/// ones already listed first (in their order), the rest in Ours, Theirs,
/// Base order.
fn normalize_order(picks: &mut ConflictPicks) {
    let has =
        |side: ConflictSide, picks: &ConflictPicks| picks.picks.iter().any(|p| p.side == side);
    let mut order: Vec<ConflictSide> = Vec::new();
    for &s in &picks.order {
        if has(s, picks) && !order.contains(&s) {
            order.push(s);
        }
    }
    for s in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
        if has(s, picks) && !order.contains(&s) {
            order.push(s);
        }
    }
    picks.order = order;
}

/// Moves the byte range `s..e` through the replacement of `at..at + rem`
/// by `ins` bytes: shifted when the change is before it, unchanged when
/// after, grown or shrunk when it touches it.
fn adjust(s: usize, e: usize, at: usize, rem: usize, ins: usize) -> (usize, usize) {
    let b = at + rem;
    let delta = ins as isize - rem as isize;
    let shift = |x: usize| (x as isize + delta).max(0) as usize;
    // A pure insertion at the very start is inside the region: typing on
    // its first line (or into an empty region) belongs to it.
    let inserts_at_start = rem == 0 && at == s;
    if b <= s && !inserts_at_start {
        return (shift(s), shift(e));
    }
    if at >= e && !inserts_at_start {
        return (s, e);
    }
    let new_start = s.min(at);
    let new_end = if b > e { at + ins } else { shift(e) };
    (new_start, new_end.max(new_start))
}

fn is_marker_line(line: &str) -> bool {
    ['<', '=', '>', '|'].into_iter().any(|c| {
        let run = line.chars().take_while(|&x| x == c).count();
        run == 7 && (line.len() == 7 || line.as_bytes()[7] == b' ')
    })
}

impl MergeModel {
    /// A fresh model: every conflict unpicked, the Output the merge with
    /// its conflicts empty.
    pub(crate) fn new(merge: MergeDocument) -> Self {
        let eol = merge.eol;
        let mut text = String::new();
        let mut states = Vec::new();
        let mut gaps = vec![String::new()];
        for region in &merge.regions {
            match region {
                MergeRegion::Merged { lines } => {
                    let t = lines_text(lines);
                    text.push_str(&t);
                    gaps.last_mut().expect("one gap").push_str(&t);
                }
                MergeRegion::Conflict { .. } => {
                    states.push(ConflictState {
                        picks: ConflictPicks::none(),
                        decided: false,
                        start: text.len(),
                        end: text.len(),
                    });
                    gaps.push(String::new());
                }
            }
        }
        let output = Doc::from_text(text, eol == Eol::Crlf, merge.bom);
        let final_newline = merge.final_newline;
        let mut m = Self {
            merge,
            output,
            states,
            gaps,
            current: 0,
            history: HashMap::new(),
            next_tag: 1,
            eol,
            final_newline,
            outside: false,
            hint: None,
            saved: Vec::new(),
            generation: 0,
            derived: RefCell::default(),
        };
        m.saved = m.decisions();
        m
    }

    /// The Output starts from the file on disk, which was edited outside
    /// (plan §2.5 case 2). Picks are disabled; the counter reads "edited
    /// outside". `disk_text` is the file's decoded text, BOM removed.
    pub(crate) fn from_disk(merge: MergeDocument, disk_text: &str) -> Self {
        let crlf = matches!(
            merge.worktree,
            WorktreeFileState::EditedNoMarkers { crlf: true }
        );
        let lf = disk_text.replace("\r\n", "\n");
        let final_newline = lf.is_empty() || lf.ends_with('\n');
        let mut canonical = lf;
        if !canonical.is_empty() && !canonical.ends_with('\n') {
            canonical.push('\n');
        }
        let eol = if crlf { Eol::Crlf } else { Eol::Lf };
        let output = Doc::from_text(canonical, crlf, merge.bom);
        Self {
            merge,
            output,
            states: Vec::new(),
            gaps: vec![String::new()],
            current: 0,
            history: HashMap::new(),
            next_tag: 1,
            eol,
            final_newline,
            outside: true,
            hint: None,
            saved: Vec::new(),
            generation: 0,
            derived: RefCell::default(),
        }
    }

    /// Overrides the file's line ending (`editor.eol = lf|crlf`).
    pub(crate) fn with_eol(mut self, eol: Eol) -> Self {
        self.eol = eol;
        self
    }

    // -- reading -----------------------------------------------------------

    pub(crate) fn merge(&self) -> &MergeDocument {
        &self.merge
    }

    pub(crate) fn output(&self) -> &Doc {
        &self.output
    }

    /// Mutable access for moving the caret and selection only: text
    /// changes must go through [`MergeModel::edit`] so anchors follow.
    pub(crate) fn output_for_caret(&mut self) -> &mut Doc {
        &mut self.output
    }

    pub(crate) fn conflict_count(&self) -> usize {
        self.states.len()
    }

    pub(crate) fn is_outside(&self) -> bool {
        self.outside
    }

    /// The line ending the file will be written with.
    pub(crate) fn eol(&self) -> Eol {
        self.eol
    }

    /// Whether the written file ends with a line break (`false` shows the
    /// "no newline at end of file" note under the Output).
    pub(crate) fn final_newline(&self) -> bool {
        self.final_newline
    }

    fn region(&self, c: usize) -> Option<&MergeRegion> {
        self.merge.conflicts().nth(c)
    }

    /// The lines of `side`'s block of conflict `c` (for the A / B panes).
    pub(crate) fn block(&self, c: usize, side: ConflictSide) -> &[String] {
        self.region(c).map_or(&[], |r| r.lines_of(side))
    }

    pub(crate) fn picks(&self, c: usize) -> Option<&ConflictPicks> {
        self.states.get(c).map(|s| &s.picks)
    }

    fn expected_text(&self, c: usize) -> String {
        match (self.region(c), self.states.get(c)) {
            (Some(r), Some(s)) => lines_text(&s.picks.lines_for(r)),
            _ => String::new(),
        }
    }

    /// The Output text now in conflict `c`'s region.
    pub(crate) fn region_text(&self, c: usize) -> &str {
        self.states
            .get(c)
            .map_or("", |s| &self.output.text()[s.start..s.end])
    }

    /// The anchors or picks changed: what was derived from them is stale.
    fn touch(&mut self) {
        self.generation += 1;
    }

    /// Runs `f` over the derived cache, emptied first when the Output or
    /// the anchors moved since it was filled.
    fn derived<R>(&self, f: impl FnOnce(&mut Derived) -> R) -> R {
        let mut d = self.derived.borrow_mut();
        let key = (self.output.version(), self.generation);
        if d.key != key || d.status.len() != self.states.len() || d.gap_ok.len() != self.gaps.len()
        {
            d.key = key;
            d.status = vec![None; self.states.len()];
            d.gap_ok = vec![None; self.gaps.len()];
            d.origins = vec![None; self.states.len()];
        }
        f(&mut d)
    }

    pub(crate) fn status(&self, c: usize) -> Option<ConflictStatus> {
        let s = self.states.get(c)?;
        if let Some(cached) = self.derived(|d| d.status[c]) {
            return Some(cached);
        }
        let status = if self.region_text(c) != self.expected_text(c) {
            ConflictStatus::Edited
        } else if s.decided {
            ConflictStatus::Picked
        } else {
            ConflictStatus::Unpicked
        };
        self.derived(|d| d.status[c] = Some(status));
        Some(status)
    }

    pub(crate) fn is_edited(&self, c: usize) -> bool {
        self.status(c) == Some(ConflictStatus::Edited)
    }

    /// How many conflicts are in each state: `(unpicked, picked, edited)`.
    pub(crate) fn tally(&self) -> (usize, usize, usize) {
        let mut t = (0, 0, 0);
        for c in 0..self.states.len() {
            match self.status(c) {
                Some(ConflictStatus::Unpicked) => t.0 += 1,
                Some(ConflictStatus::Picked) => t.1 += 1,
                Some(ConflictStatus::Edited) => t.2 += 1,
                None => {}
            }
        }
        t
    }

    /// The Output lines of conflict `c`'s region: `(first, end)`, `end`
    /// exclusive; equal for an empty region (where the placeholder row
    /// goes).
    pub(crate) fn region_lines(&self, c: usize) -> Option<(usize, usize)> {
        let s = self.states.get(c)?;
        let first = self.output.line_of(s.start);
        let text = self.output.text();
        let end = if s.end == s.start {
            first
        } else if s.end == text.len() && !text.ends_with('\n') {
            self.output.line_of(s.end) + 1
        } else {
            self.output.line_of(s.end)
        };
        Some((first, end))
    }

    /// The conflict whose region holds byte `offset` of the Output (an
    /// empty region holds just its own start).
    pub(crate) fn conflict_at(&self, offset: usize) -> Option<usize> {
        self.states.iter().position(|s| {
            (s.start <= offset && offset < s.end) || (s.start == s.end && offset == s.start)
        })
    }

    /// Whether one of the checkboxes: how much of `side`'s block is picked.
    pub(crate) fn chunk_pick(&self, c: usize, side: ConflictSide) -> ChunkPick {
        let n = self.block(c, side).len();
        let picked = (0..n).filter(|&i| self.line_picked(c, side, i)).count();
        match picked {
            0 => ChunkPick::None,
            p if p == n => ChunkPick::All,
            _ => ChunkPick::Some,
        }
    }

    pub(crate) fn line_picked(&self, c: usize, side: ConflictSide, line: usize) -> bool {
        self.picks(c)
            .is_some_and(|p| p.picks.iter().any(|x| x.side == side && x.line == line))
    }

    /// `1` or `2` (or `3`) when more than one side contributes: the
    /// `1st` / `2nd` badge. `None` for a single side or none.
    pub(crate) fn order_badge(&self, c: usize, side: ConflictSide) -> Option<usize> {
        let order = self.picks(c)?.effective_order();
        if order.len() < 2 {
            return None;
        }
        order.iter().position(|&s| s == side).map(|i| i + 1)
    }

    /// Where the `‹ Conflict n · nothing picked yet ›` row sits: the
    /// Output line of an unpicked, untouched, empty region.
    pub(crate) fn placeholder_line(&self, c: usize) -> Option<usize> {
        (self.status(c)? == ConflictStatus::Unpicked && self.states[c].start == self.states[c].end)
            .then(|| self.output.line_of(self.states[c].start))
    }

    /// The gutter letter of an Output line.
    pub(crate) fn origin_of_line(&self, line: usize) -> Origin {
        let off = self.output.line_start(line);
        if let Some(c) = self
            .states
            .iter()
            .position(|s| s.start <= off && off < s.end)
        {
            if self.is_edited(c) {
                return Origin::Edited;
            }
            let first = self.output.line_of(self.states[c].start);
            let idx = line.saturating_sub(first);
            let origins = match self.derived(|d| d.origins[c].clone()) {
                Some(o) => o,
                None => {
                    let o = std::sync::Arc::new(self.region_origins(c));
                    self.derived(|d| d.origins[c] = Some(o.clone()));
                    o
                }
            };
            return origins
                .get(idx)
                .copied()
                .map_or(Origin::Edited, Origin::Side);
        }
        let k = self.states.partition_point(|s| s.end <= off);
        let same = match self.derived(|d| d.gap_ok[k]) {
            Some(same) => same,
            None => {
                let same = self.gap_text(k) == self.gaps[k];
                self.derived(|d| d.gap_ok[k] = Some(same));
                same
            }
        };
        if same {
            Origin::Common
        } else {
            Origin::Edited
        }
    }

    /// The side of each line a conflict's picks produce, in output order.
    fn region_origins(&self, c: usize) -> Vec<ConflictSide> {
        let Some(picks) = self.picks(c) else {
            return Vec::new();
        };
        let n = |side| self.block(c, side).len();
        let mut out = Vec::new();
        for side in picks.effective_order() {
            let mut idx: Vec<usize> = picks
                .picks
                .iter()
                .filter(|p| p.side == side && p.line < n(side))
                .map(|p| p.line)
                .collect();
            idx.sort_unstable();
            idx.dedup();
            out.extend(idx.iter().map(|_| side));
        }
        out
    }

    /// The Output text of gap `k` (between conflicts `k - 1` and `k`).
    fn gap_text(&self, k: usize) -> &str {
        let start = if k == 0 { 0 } else { self.states[k - 1].end };
        let end = self
            .states
            .get(k)
            .map_or(self.output.text().len(), |s| s.start);
        &self.output.text()[start.min(end)..end]
    }

    // -- navigation --------------------------------------------------------

    pub(crate) fn current(&self) -> usize {
        self.current
    }

    pub(crate) fn go_to(&mut self, c: usize) -> bool {
        if c < self.states.len() {
            self.current = c;
            true
        } else {
            false
        }
    }

    /// The next conflict, wrapping past the last to the first.
    pub(crate) fn next_conflict(&mut self) -> Option<usize> {
        let n = self.states.len();
        (n > 0).then(|| {
            self.current = (self.current + 1) % n;
            self.current
        })
    }

    /// The previous conflict, wrapping past the first to the last.
    pub(crate) fn prev_conflict(&mut self) -> Option<usize> {
        let n = self.states.len();
        (n > 0).then(|| {
            self.current = (self.current + n - 1) % n;
            self.current
        })
    }

    /// The next conflict after the current one that is still unpicked,
    /// wrapping; the current one is left alone when none is.
    pub(crate) fn next_unpicked(&mut self) -> Option<usize> {
        let n = self.states.len();
        let found = (1..=n)
            .map(|d| (self.current + d) % n.max(1))
            .find(|&c| self.status(c) == Some(ConflictStatus::Unpicked))?;
        self.current = found;
        Some(found)
    }

    /// Makes the conflict under the Output caret the current one.
    pub(crate) fn follow_caret(&mut self) {
        if let Some(c) = self.conflict_at(self.output.cursor()) {
            self.current = c;
        }
    }

    pub(crate) fn counter(&self) -> Counter {
        Counter {
            current: if self.states.is_empty() {
                0
            } else {
                self.current + 1
            },
            total: self.states.len(),
            left: self.tally().0,
        }
    }

    /// `Conflict 2 / 5 · 3 left`.
    pub(crate) fn counter_label(&self) -> String {
        if self.outside {
            return gitpoulpi_i18n::msg::merge::EDITED_OUTSIDE.get().to_string();
        }
        let c = self.counter();
        if c.total == 0 {
            return gitpoulpi_i18n::msg::merge_view::COUNTER_NONE
                .get()
                .to_string();
        }
        gitpoulpi_i18n::msg::merge_view::counter(c.left as u64, c.current, c.total)
    }

    // -- picking -----------------------------------------------------------

    /// One line of one side, on or off.
    pub(crate) fn toggle_line(&mut self, c: usize, side: ConflictSide, line: usize) -> bool {
        let Some(mut picks) = self.picks(c).cloned() else {
            return false;
        };
        if line >= self.block(c, side).len() {
            return false;
        }
        let pick = Pick { side, line };
        if let Some(i) = picks.picks.iter().position(|p| *p == pick) {
            picks.picks.remove(i);
        } else {
            picks.picks.push(pick);
            if !picks.order.contains(&side) {
                picks.order.push(side);
            }
        }
        normalize_order(&mut picks);
        let decided = !picks.picks.is_empty();
        self.apply(vec![(c, picks, decided)])
    }

    /// A whole side of a conflict: all of it off when all is picked, else
    /// all of it on (appended to the order when the side was absent).
    pub(crate) fn toggle_chunk(&mut self, c: usize, side: ConflictSide) -> bool {
        let Some(mut picks) = self.picks(c).cloned() else {
            return false;
        };
        let n = self.block(c, side).len();
        if n == 0 {
            return false;
        }
        if self.chunk_pick(c, side) == ChunkPick::All {
            picks.picks.retain(|p| p.side != side);
        } else {
            for line in 0..n {
                let pick = Pick { side, line };
                if !picks.picks.contains(&pick) {
                    picks.picks.push(pick);
                }
            }
            if !picks.order.contains(&side) {
                picks.order.push(side);
            }
        }
        normalize_order(&mut picks);
        let decided = !picks.picks.is_empty();
        self.apply(vec![(c, picks, decided)])
    }

    /// Takes all of each of `sides`, in that order, replacing any picks
    /// and any typed edit of the conflict.
    pub(crate) fn take(&mut self, c: usize, sides: &[ConflictSide]) -> bool {
        let Some(region) = self.region(c) else {
            return false;
        };
        let picks = ConflictPicks::take(region, sides);
        self.apply(vec![(c, picks, true)])
    }

    pub(crate) fn take_a(&mut self, c: usize) -> bool {
        self.take(c, &[ConflictSide::Ours])
    }
    pub(crate) fn take_b(&mut self, c: usize) -> bool {
        self.take(c, &[ConflictSide::Theirs])
    }
    pub(crate) fn take_a_then_b(&mut self, c: usize) -> bool {
        self.take(c, &[ConflictSide::Ours, ConflictSide::Theirs])
    }
    pub(crate) fn take_b_then_a(&mut self, c: usize) -> bool {
        self.take(c, &[ConflictSide::Theirs, ConflictSide::Ours])
    }
    pub(crate) fn take_base(&mut self, c: usize) -> bool {
        self.take(c, &[ConflictSide::Base])
    }

    /// Nothing from either side, on purpose: the conflict resolves to no
    /// lines and counts as handled.
    pub(crate) fn clear(&mut self, c: usize) -> bool {
        if c >= self.states.len() {
            return false;
        }
        self.apply(vec![(c, ConflictPicks::none(), true)])
    }

    /// Back to unresolved: no picks, nothing decided, and any typed edit of
    /// the conflict gone (the region is empty again, the placeholder row
    /// shows). One undo step. `false` when it already is so.
    pub(crate) fn unpick(&mut self, c: usize) -> bool {
        if c >= self.states.len() {
            return false;
        }
        self.apply(vec![(c, ConflictPicks::none(), false)])
    }

    /// Whether the Output holds anything the picks and the base merge do
    /// not explain: a typed edit in a conflict, or in the text between
    /// them. A whole-file reset would throw that away.
    pub(crate) fn has_manual_edits(&self) -> bool {
        if self.outside {
            return true;
        }
        let mut text = String::new();
        let mut c = 0;
        for region in &self.merge.regions {
            match region {
                MergeRegion::Merged { lines } => text.push_str(&lines_text(lines)),
                MergeRegion::Conflict { .. } => {
                    text.push_str(&self.expected_text(c));
                    c += 1;
                }
            }
        }
        text != self.output.text()
    }

    /// Reverses the order the picked sides come out in (`s`). Needs two
    /// or more sides picked.
    pub(crate) fn swap_order(&mut self, c: usize) -> bool {
        let Some(picks) = self.picks(c) else {
            return false;
        };
        let mut order = picks.effective_order();
        if order.len() < 2 {
            return false;
        }
        order.reverse();
        let mut picks = picks.clone();
        picks.order = order;
        self.apply(vec![(c, picks, true)])
    }

    /// Whole-file "take all": every conflict takes `sides` (one side, or
    /// two in order). The merged text between conflicts stays as git
    /// merged it. One undo step.
    pub(crate) fn take_all(&mut self, sides: &[ConflictSide]) -> bool {
        let changes: Vec<_> = (0..self.states.len())
            .filter_map(|c| {
                self.region(c)
                    .map(|r| (c, ConflictPicks::take(r, sides), true))
            })
            .collect();
        self.apply(changes)
    }

    pub(crate) fn take_all_a(&mut self) -> bool {
        self.take_all(&[ConflictSide::Ours])
    }
    pub(crate) fn take_all_b(&mut self) -> bool {
        self.take_all(&[ConflictSide::Theirs])
    }

    /// An agent's proposal: one choice per conflict, in file order, as one
    /// undo step. Returns whether anything changed.
    pub(crate) fn set_picks(&mut self, picks: &[ConflictPicks]) -> bool {
        self.apply(
            picks
                .iter()
                .enumerate()
                .map(|(c, p)| (c, p.clone(), true))
                .collect(),
        )
    }

    /// Starts over: every conflict unpicked and the Output the fresh
    /// merge, as one undo step ("Reset Output to Merge Result").
    pub(crate) fn reset(&mut self) -> bool {
        let before = self.states.clone();
        if self.outside || before.is_empty() && self.output.text().is_empty() {
            return false;
        }
        let mut text = String::new();
        let mut states = Vec::new();
        let mut conflict = 0;
        for region in &self.merge.regions {
            match region {
                MergeRegion::Merged { lines } => text.push_str(&lines_text(lines)),
                MergeRegion::Conflict { .. } => {
                    states.push(ConflictState {
                        picks: ConflictPicks::none(),
                        decided: false,
                        start: text.len(),
                        end: text.len(),
                    });
                    conflict += 1;
                }
            }
        }
        debug_assert_eq!(conflict, states.len());
        if text == self.output.text() && states == before {
            return false;
        }
        let tag = self.fresh_tag();
        let len = self.output.text().len();
        self.output.replace_tagged(0, len, &text, tag);
        self.output.take_text_changes();
        self.history.insert(tag, (before, states.clone()));
        self.states = states;
        self.touch();
        true
    }

    /// Puts `lines` into conflict `c`'s region as one undo step (the AI's
    /// suggestion). The picks stay as they were, so the region reads as
    /// edited. `false` when there is no such region or nothing changes.
    pub(crate) fn set_region_lines(&mut self, c: usize, lines: &[String]) -> bool {
        if self.outside || c >= self.states.len() {
            return false;
        }
        let new_text = lines_text(lines);
        if self.region_text(c) == new_text {
            return false;
        }
        let before = self.states.clone();
        let mut after = before.clone();
        let (start, end) = (before[c].start, before[c].end);
        after[c].end = start + new_text.len();
        let delta = new_text.len() as isize - (end - start) as isize;
        for s in after.iter_mut().skip(c + 1) {
            s.start = (s.start as isize + delta) as usize;
            s.end = (s.end as isize + delta) as usize;
        }
        let tag = self.fresh_tag();
        self.output.replace_tagged(start, end, &new_text, tag);
        self.output.take_text_changes();
        self.history.insert(tag, (before, after.clone()));
        self.states = after;
        true
    }

    fn fresh_tag(&mut self) -> u64 {
        let t = self.next_tag;
        self.next_tag += 1;
        t
    }

    /// Sets the picks of several conflicts at once as one undo step: the
    /// span from the first to the last is rebuilt (text between them
    /// untouched), later anchors shift.
    fn apply(&mut self, mut changes: Vec<(usize, ConflictPicks, bool)>) -> bool {
        if self.outside {
            return false;
        }
        changes.retain(|(c, ..)| *c < self.states.len());
        // Skip conflicts the change would leave exactly as they are.
        changes.retain(|(c, picks, decided)| {
            let s = &self.states[*c];
            !(s.picks == *picks && s.decided == *decided && !self.is_edited(*c))
        });
        if changes.is_empty() {
            return false;
        }
        changes.sort_by_key(|(c, ..)| *c);
        let lo = changes[0].0;
        let hi = changes[changes.len() - 1].0;
        let before = self.states.clone();
        let mut after = before.clone();
        let text = self.output.text().to_string();
        let span_start = before[lo].start;
        let span_end = before[hi].end;
        let mut new_text = String::new();
        let mut cursor = span_start;
        let mut replaced_edit = false;
        for k in lo..=hi {
            new_text.push_str(&text[cursor..before[k].start]);
            let start = span_start + new_text.len();
            if let Some((_, picks, decided)) = changes.iter().find(|(c, ..)| *c == k) {
                replaced_edit |= self.is_edited(k);
                let region = self.region(k).expect("conflict exists");
                new_text.push_str(&lines_text(&picks.lines_for(region)));
                after[k].picks = picks.clone();
                after[k].decided = *decided;
            } else {
                new_text.push_str(&text[before[k].start..before[k].end]);
            }
            after[k].start = start;
            after[k].end = span_start + new_text.len();
            cursor = before[k].end;
        }
        let delta = new_text.len() as isize - (span_end - span_start) as isize;
        for s in after.iter_mut().skip(hi + 1) {
            s.start = (s.start as isize + delta) as usize;
            s.end = (s.end as isize + delta) as usize;
        }
        let tag = self.fresh_tag();
        self.output
            .replace_tagged(span_start, span_end, &new_text, tag);
        self.output.take_text_changes();
        self.history.insert(tag, (before, after.clone()));
        self.states = after;
        self.touch();
        if replaced_edit {
            self.hint = Some(Hint::ReplacedEdit);
        }
        true
    }

    /// The one-time status hint a pick left behind (it replaced a typed
    /// edit), taken so it shows once.
    pub(crate) fn take_hint(&mut self) -> Option<Hint> {
        self.hint.take()
    }

    // -- typing ------------------------------------------------------------

    /// Runs `f` over the Output document (keys, paste, indent) and makes
    /// the anchors follow what it changed. Typing is one undo stack with
    /// the picks.
    pub(crate) fn edit<R>(&mut self, f: impl FnOnce(&mut Doc) -> R) -> R {
        let before = self.states.clone();
        let steps = self.output.undo_len();
        let out = f(&mut self.output);
        let changes = self.output.take_text_changes();
        if changes.is_empty() {
            return out;
        }
        self.follow(&changes);
        if self.output.undo_len() > steps {
            let tag = self.fresh_tag();
            self.output.set_last_tag(tag);
            self.history.insert(tag, (before, self.states.clone()));
        } else if let Some(entry) = self
            .output
            .last_tag()
            .and_then(|t| self.history.get_mut(&t))
        {
            entry.1 = self.states.clone();
        }
        out
    }

    fn follow(&mut self, changes: &[TextChange]) {
        for ch in changes {
            for s in &mut self.states {
                (s.start, s.end) = adjust(s.start, s.end, ch.at, ch.removed, ch.inserted);
            }
        }
        self.normalize();
        self.touch();
    }

    /// Keeps the anchors ordered, inside the text and on whole lines.
    fn normalize(&mut self) {
        let text = self.output.text();
        let len = text.len();
        let mut prev_end = 0;
        for s in &mut self.states {
            let mut start = s.start.max(prev_end).min(len);
            start = text[..start].rfind('\n').map_or(0, |i| i + 1).max(prev_end);
            let mut end = s.end.max(start).min(len);
            if end > start && end < len && text.as_bytes()[end - 1] != b'\n' {
                end = text[end..].find('\n').map_or(len, |i| end + i + 1);
            }
            s.start = start;
            s.end = end;
            prev_end = end;
        }
    }

    // -- undo --------------------------------------------------------------

    pub(crate) fn can_undo(&self) -> bool {
        self.output.can_undo()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.output.can_redo()
    }

    /// Undoes the last pick or run of typing: text, picks and anchors
    /// together.
    pub(crate) fn undo(&mut self) -> bool {
        match self.output.undo_tagged() {
            Some(tag) => {
                self.restore(tag, true);
                true
            }
            None => false,
        }
    }

    pub(crate) fn redo(&mut self) -> bool {
        match self.output.redo_tagged() {
            Some(tag) => {
                self.restore(tag, false);
                true
            }
            None => false,
        }
    }

    fn restore(&mut self, tag: Option<u64>, undo: bool) {
        let changes = self.output.take_text_changes();
        match tag.and_then(|t| self.history.get(&t)) {
            Some((before, after)) => {
                let snap = if undo { before } else { after };
                self.states = snap.clone();
                self.touch();
            }
            None => self.follow(&changes),
        }
    }

    // -- the file ----------------------------------------------------------

    /// The Output as LF text, with the merge's final-newline choice
    /// applied (the same string as the core's `compose_resolution` while
    /// nothing is typed).
    pub(crate) fn output_text(&self) -> String {
        let mut t = self.output.text().to_string();
        if !self.final_newline && t.ends_with('\n') {
            t.pop();
        }
        t
    }

    /// What to write: line endings, BOM and encoding decisions restored.
    pub(crate) fn output_file(&self) -> OutputFile {
        let lf = self.output_text();
        let text = match self.eol {
            Eol::Lf => lf,
            Eol::Crlf => lf.replace('\n', "\r\n"),
        };
        OutputFile {
            text,
            bom: self.merge.bom,
            eol: self.eol,
            encoding: self.merge.encoding.clone(),
        }
    }

    /// Unpicked conflicts and marker-like lines still in the Output.
    pub(crate) fn save_checks(&self) -> SaveChecks {
        let unpicked = (0..self.states.len())
            .filter(|&c| self.status(c) == Some(ConflictStatus::Unpicked))
            .collect();
        let text = self.output.text();
        let marker_lines = text
            .split('\n')
            .enumerate()
            .filter(|(_, l)| is_marker_line(l))
            .map(|(i, _)| i + 1)
            .collect();
        SaveChecks {
            unpicked,
            marker_lines,
        }
    }

    fn decisions(&self) -> Vec<(ConflictPicks, bool)> {
        self.states
            .iter()
            .map(|s| (s.picks.clone(), s.decided))
            .collect()
    }

    /// The Output or a decision differs from what was last saved (or
    /// from the fresh merge, before any save).
    pub(crate) fn is_dirty(&self) -> bool {
        self.output.is_dirty() || self.decisions() != self.saved
    }

    pub(crate) fn mark_saved(&mut self) {
        self.output.mark_saved();
        self.saved = self.decisions();
    }

    // -- the per-session cache ---------------------------------------------

    fn cache_key(&self) -> CacheKey {
        let e = &self.merge.entry;
        CacheKey {
            path: e.path.clone(),
            oids: [e.base, e.ours, e.theirs].map(|s| s.map(|s| s.oid)),
        }
    }

    fn snapshot(&self) -> Saved {
        Saved {
            states: self.states.clone(),
            text: self.output.text().to_string(),
            current: self.current,
        }
    }

    /// Puts a cached snapshot back (no undo history is kept across a
    /// close). Ignored when it does not fit this document.
    fn restore_snapshot(&mut self, saved: &Saved) {
        if saved.states.len() != self.states.len() || self.outside {
            return;
        }
        let fresh = self.output.text().to_string();
        let mut output = Doc::from_text(saved.text.clone(), self.eol == Eol::Crlf, self.merge.bom);
        output.set_baseline(&fresh);
        self.output = output;
        self.states = saved.states.clone();
        self.current = saved.current.min(self.states.len().saturating_sub(1));
        self.normalize();
        self.touch();
    }
}

/// The identity of a conflicted file's contents: the path and the three
/// stage oids. A snapshot only applies while they are unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    path: PathBuf,
    oids: [Option<Oid>; 3],
}

#[derive(Debug, Clone)]
struct Saved {
    states: Vec<ConflictState>,
    text: String,
    current: usize,
}

/// The picks and Output of files closed during an operation, so
/// reopening one restores them (plan §2.5 case 3). Cleared when the
/// operation ends.
#[derive(Default)]
pub(crate) struct PicksCache {
    map: HashMap<CacheKey, Saved>,
}

impl PicksCache {
    /// Remembers `model`'s state, or forgets the file when it has none
    /// worth restoring (never touched).
    pub(crate) fn store(&mut self, model: &MergeModel) {
        if model.outside {
            return;
        }
        if model.is_dirty() {
            self.map.insert(model.cache_key(), model.snapshot());
        } else {
            self.map.remove(&model.cache_key());
        }
    }

    /// A model for `merge`: the remembered state when the file's stages
    /// are the ones it was remembered for, else a fresh one.
    pub(crate) fn open(&self, merge: MergeDocument) -> MergeModel {
        self.prepare(merge)()
    }

    /// [`PicksCache::open`] as a job that owns what it needs, so a big
    /// file's model can be built on a worker thread.
    pub(crate) fn prepare(&self, merge: MergeDocument) -> impl FnOnce() -> MergeModel + Send {
        let key = CacheKey {
            path: merge.entry.path.clone(),
            oids: [merge.entry.base, merge.entry.ours, merge.entry.theirs]
                .map(|s| s.map(|s| s.oid)),
        };
        let saved = self.map.get(&key).cloned();
        move || {
            let mut model = MergeModel::new(merge);
            if let Some(saved) = &saved {
                model.restore_snapshot(saved);
            }
            model
        }
    }

    /// Drops what was remembered for `model`'s file (it was discarded,
    /// or resolved).
    pub(crate) fn forget(&mut self, model: &MergeModel) {
        self.map.remove(&model.cache_key());
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
