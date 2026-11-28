//! "Split commit": the editor that recomposes one existing commit into
//! several. This module is the model and its keys; `render.rs` paints it and
//! maps the mouse onto the same methods.
//!
//! The editor holds the commit's numbered changes (`SplitSource`, read once
//! by `GitBackend::split_source`) and a list of parts, oldest first. Every
//! assignable unit - a whole file that cannot be cut, or one changed line of
//! a hunk - is owned by at most one part, so a unit can never be assigned
//! twice by construction. A hunk is "all its changed lines", a whole text
//! file "all the lines of all its hunks": the model keeps lines, and
//! [`SplitForm::build_parts`] folds them back into the coarsest pieces
//! (`Hunk` when a part owns every changed line of a hunk, `Lines` otherwise,
//! `File` only for whole-only files). What the backend would refuse is
//! answered by the same pure check, `validate_split`, so the Split button
//! is enabled exactly when the backend will accept the plan.
//!
//! Applying runs [`Action::SplitCommit`]. The rewrite never touches the
//! index or the working tree and is one journaled step, so nothing here can
//! lose work (the split-commit design).

mod render;

pub(crate) use render::{split_hit_at, SplitHit};

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::keymap::term::TermKeymap;
use crate::keymap::Action as Bind;
use crate::reword_ui::{RewordForm, RewordFormOutcome};
use gitpoulpi_core::domain::{
    unassigned, validate_split, Hunk, LineOrigin, SplitCommit, SplitPart, SplitPiece, SplitRefusal,
    SplitSource,
};
use std::collections::HashMap;

/// The rebase popups that asked for a split editor, by the commit asked for:
/// the editor opens when the daemon answers, long after the popup was left.
static RETURNS: std::sync::Mutex<Vec<(Oid, Oid)>> = std::sync::Mutex::new(Vec::new());

/// Notes that the editor about to open on `commit` comes from the rebase
/// popup on `upstream`.
pub(crate) fn expect_return(commit: Oid, upstream: Oid) {
    if let Ok(mut held) = RETURNS.lock() {
        held.retain(|(c, _)| *c != commit);
        held.push((commit, upstream));
    }
}

/// The upstream of the rebase popup the editor on `commit` came from.
pub(crate) fn take_return(commit: Oid) -> Option<Oid> {
    let mut held = RETURNS.lock().ok()?;
    let at = held.iter().position(|(c, _)| *c == commit)?;
    Some(held.remove(at).1)
}

/// One selectable row of the changes pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SplitRow {
    /// File `.0` of the source.
    File(usize),
    /// Hunk `.1` of file `.0`.
    Hunk(usize, usize),
    /// Line `.2` (an index into `Hunk::lines`, always an add or a delete) of
    /// hunk `.1` of file `.0`. Only the hunk opened by line mode lists them.
    Line(usize, usize, u32),
}

/// The pane that has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitPane {
    /// The commits to create, oldest first.
    Parts,
    /// The files, hunks and lines of the original commit.
    Changes,
}

/// Who owns what a row stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowOwner {
    Unassigned,
    /// Part `.0` (0-based).
    Part(usize),
    /// A file or hunk whose lines are split over several parts or only
    /// partly assigned.
    Mixed,
}

/// What a key asks of the caller.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SplitFormOutcome {
    Stay,
    Close,
    Submit(Action),
}

/// One assignable unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Unit {
    /// A whole-only file (binary, mode change, added, deleted, submodule, LFS).
    File(usize),
    /// One changed line: file, hunk (position in the file), line index.
    Line(usize, usize, u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartState {
    message: String,
}

fn is_change(hunk: &Hunk, index: u32) -> bool {
    hunk.lines
        .get(index as usize)
        .is_some_and(|l| l.origin != LineOrigin::Context)
}

fn changed_lines(hunk: &Hunk) -> Vec<u32> {
    (0..hunk.lines.len() as u32)
        .filter(|i| is_change(hunk, *i))
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SplitForm {
    source: SplitSource,
    parts: Vec<PartState>,
    owner: HashMap<Unit, usize>,
    initial: (Vec<PartState>, HashMap<Unit, usize>),
    pane: SplitPane,
    part_cursor: usize,
    /// Position in [`Self::rows`].
    row_cursor: usize,
    /// The hunk whose changed lines are listed (line mode), as (file, hunk).
    lines_open: Option<(usize, usize)>,
    /// Where a Shift+arrow range started, as a position in [`Self::rows`].
    anchor: Option<usize>,
    /// The message editor, open on part `.0`.
    editing: Option<(usize, RewordForm)>,
    sign: bool,
    /// Set when the editor was opened from the interactive-rebase popup:
    /// the upstream the popup is rebuilt on when the editor is left.
    return_to: Option<Oid>,
    error: Option<String>,
    /// The plan was sent to the daemon and its answer is awaited: the editor
    /// stays open (a refusal keeps the plan), closes on success.
    applying: bool,
    /// What the last paint learned, for the mouse and the list scrolling.
    view: render::View,
}

impl SplitForm {
    /// The editor for `source`: two parts, the first holding everything with
    /// the original message, the second empty. Moving changes into the
    /// second is the first thing a user does; starting from "everything in
    /// part 1" keeps the plan valid except for the empty part.
    pub(crate) fn new(source: SplitSource) -> Self {
        let message = source.message.clone();
        Self::build(
            source,
            vec![
                PartState { message },
                PartState {
                    message: String::new(),
                },
            ],
            None,
        )
    }

    /// The editor prefilled with `parts` (an AI proposal, or a plan from
    /// elsewhere). Pieces that do not exist in `source` are ignored; a unit
    /// named twice stays with its first part (the editor cannot hold it
    /// twice). Fewer than two parts are padded with empty ones.
    pub(crate) fn with_parts(source: SplitSource, parts: Vec<SplitPart>) -> Self {
        let mut states: Vec<PartState> = parts
            .iter()
            .map(|p| PartState {
                message: p.message.clone(),
            })
            .collect();
        while states.len() < 2 {
            states.push(PartState {
                message: String::new(),
            });
        }
        Self::build(source, states, Some(parts))
    }

    /// Remembers the rebase popup this editor was opened from (by its
    /// upstream), so leaving the editor returns to it.
    pub(crate) fn returning_to_rebase(mut self, upstream: Oid) -> Self {
        self.return_to = Some(upstream);
        self
    }

    /// The upstream of the rebase popup to reopen on leaving, if any.
    pub(crate) fn return_to(&self) -> Option<Oid> {
        self.return_to
    }

    /// Shows `reason` under the editor (the AI could not plan the split).
    pub(crate) fn note(&mut self, reason: String) {
        self.error = Some(reason);
    }

    fn build(source: SplitSource, parts: Vec<PartState>, plan: Option<Vec<SplitPart>>) -> Self {
        let mut owner = HashMap::new();
        match plan {
            None => {
                for unit in Self::all_units(&source) {
                    owner.insert(unit, 0);
                }
            }
            Some(plan) => {
                for (k, part) in plan.iter().enumerate() {
                    for piece in &part.pieces {
                        for unit in Self::piece_units(&source, piece) {
                            owner.entry(unit).or_insert(k);
                        }
                    }
                }
            }
        }
        let sign = source.sign_default;
        Self {
            source,
            initial: (parts.clone(), owner.clone()),
            parts,
            owner,
            pane: SplitPane::Changes,
            part_cursor: 0,
            row_cursor: 0,
            lines_open: None,
            anchor: None,
            editing: None,
            sign,
            error: None,
            return_to: None,
            applying: false,
            view: render::View::default(),
        }
    }

    // ---- what the source holds -------------------------------------------

    fn all_units(source: &SplitSource) -> Vec<Unit> {
        let mut out = Vec::new();
        for (fi, file) in source.files.iter().enumerate() {
            if file.whole_only.is_some() {
                out.push(Unit::File(fi));
                continue;
            }
            for (hi, hunk) in file.hunks.iter().enumerate() {
                for li in changed_lines(hunk) {
                    out.push(Unit::Line(fi, hi, li));
                }
            }
        }
        out
    }

    fn file_units(source: &SplitSource, fi: usize) -> Vec<Unit> {
        let Some(file) = source.files.get(fi) else {
            return Vec::new();
        };
        if file.whole_only.is_some() {
            return vec![Unit::File(fi)];
        }
        (0..file.hunks.len())
            .flat_map(|hi| Self::hunk_units(source, fi, hi))
            .collect()
    }

    fn hunk_units(source: &SplitSource, fi: usize, hi: usize) -> Vec<Unit> {
        source
            .files
            .get(fi)
            .and_then(|f| f.hunks.get(hi))
            .map(|h| {
                changed_lines(h)
                    .into_iter()
                    .map(|li| Unit::Line(fi, hi, li))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn piece_units(source: &SplitSource, piece: &SplitPiece) -> Vec<Unit> {
        let find = |path: &std::path::Path| source.files.iter().position(|f| f.path == path);
        let hunk_at = |fi: usize, id| source.files[fi].hunks.iter().position(|h| h.id == id);
        match piece {
            SplitPiece::File { path } => find(path)
                .map(|fi| Self::file_units(source, fi))
                .unwrap_or_default(),
            SplitPiece::Hunk { path, hunk } => find(path)
                .and_then(|fi| hunk_at(fi, *hunk).map(|hi| Self::hunk_units(source, fi, hi)))
                .unwrap_or_default(),
            SplitPiece::Lines { path, hunk, lines } => find(path)
                .and_then(|fi| {
                    let hi = hunk_at(fi, *hunk)?;
                    let h = &source.files[fi].hunks[hi];
                    Some(
                        lines
                            .iter()
                            .filter(|li| is_change(h, **li))
                            .map(|li| Unit::Line(fi, hi, *li))
                            .collect(),
                    )
                })
                .unwrap_or_default(),
        }
    }

    // ---- reading the state (what the renderer draws from) ------------------

    /// The commit being split.
    pub(crate) fn commit(&self) -> Oid {
        self.source.commit
    }

    pub(crate) fn pane(&self) -> SplitPane {
        self.pane
    }

    pub(crate) fn part_count(&self) -> usize {
        self.parts.len()
    }

    pub(crate) fn part_message(&self, part: usize) -> &str {
        self.parts.get(part).map_or("", |p| p.message.as_str())
    }

    /// The part the commits pane has selected.
    pub(crate) fn selected_part(&self) -> usize {
        self.part_cursor
    }

    pub(crate) fn sign(&self) -> bool {
        self.sign
    }

    /// Sets the "Sign commits" checkbox (it starts as the effective
    /// `commit.gpgSign`, which `SplitSource::sign_default` carries).
    #[cfg(test)]
    pub(crate) fn set_sign(&mut self, sign: bool) {
        self.sign = sign;
    }

    #[cfg(test)]
    pub(crate) fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    /// The message editor open on a part, for the renderer.
    pub(crate) fn editing(&self) -> Option<(usize, &RewordForm)> {
        self.editing.as_ref().map(|(k, f)| (*k, f))
    }

    /// Line mode is on: a hunk lists its changed lines.
    pub(crate) fn line_mode(&self) -> bool {
        self.lines_open.is_some()
    }

    /// Every row of the changes pane, in display order: a file, then its
    /// hunks; the hunk opened by line mode is followed by its changed lines.
    pub(crate) fn rows(&self) -> Vec<SplitRow> {
        let mut rows = Vec::new();
        for (fi, file) in self.source.files.iter().enumerate() {
            rows.push(SplitRow::File(fi));
            if file.whole_only.is_some() {
                continue;
            }
            for (hi, hunk) in file.hunks.iter().enumerate() {
                rows.push(SplitRow::Hunk(fi, hi));
                if self.lines_open == Some((fi, hi)) {
                    rows.extend(
                        changed_lines(hunk)
                            .into_iter()
                            .map(|li| SplitRow::Line(fi, hi, li)),
                    );
                }
            }
        }
        rows
    }

    /// The row the changes pane's cursor is on.
    pub(crate) fn cursor_row(&self) -> Option<SplitRow> {
        self.rows().get(self.row_cursor).copied()
    }

    /// Moves the changes cursor onto `row` (a click); no-op if it is not listed.
    pub(crate) fn select_row(&mut self, row: SplitRow) {
        if let Some(at) = self.rows().iter().position(|r| *r == row) {
            self.pane = SplitPane::Changes;
            self.row_cursor = at;
            self.anchor = None;
        }
    }

    /// The rows an assignment applies to: the Shift-extended line range, else
    /// the cursor row.
    pub(crate) fn selection(&self) -> Vec<SplitRow> {
        let rows = self.rows();
        let Some(cursor) = rows.get(self.row_cursor).copied() else {
            return Vec::new();
        };
        if let (Some(anchor), SplitRow::Line(..)) = (self.anchor, cursor) {
            let (lo, hi) = (anchor.min(self.row_cursor), anchor.max(self.row_cursor));
            let range: Vec<SplitRow> = rows[lo..=hi]
                .iter()
                .copied()
                .filter(|r| matches!(r, SplitRow::Line(..)))
                .collect();
            if !range.is_empty() {
                return range;
            }
        }
        vec![cursor]
    }

    fn row_units(&self, row: SplitRow) -> Vec<Unit> {
        match row {
            SplitRow::File(fi) => Self::file_units(&self.source, fi),
            SplitRow::Hunk(fi, hi) => Self::hunk_units(&self.source, fi, hi),
            SplitRow::Line(fi, hi, li) => vec![Unit::Line(fi, hi, li)],
        }
    }

    fn owner_of(&self, units: &[Unit]) -> RowOwner {
        let mut owners = units.iter().map(|u| self.owner.get(u).copied());
        let Some(first) = owners.next() else {
            return RowOwner::Unassigned;
        };
        if owners.any(|o| o != first) {
            return RowOwner::Mixed;
        }
        first.map_or(RowOwner::Unassigned, RowOwner::Part)
    }

    /// Who owns `row` (the `[N]` of a hunk's combo, the gutter colour).
    pub(crate) fn row_owner(&self, row: SplitRow) -> RowOwner {
        self.owner_of(&self.row_units(row))
    }

    /// How many pieces part `part` will carry (the number next to a commit).
    pub(crate) fn part_pieces(&self, part: usize) -> usize {
        self.build_parts().get(part).map_or(0, |p| p.pieces.len())
    }

    /// The plan as the backend takes it: per part, a `File` for each
    /// whole-only file, a `Hunk` for each hunk the part owns completely and
    /// `Lines` for the rest.
    pub(crate) fn build_parts(&self) -> Vec<SplitPart> {
        let mut out: Vec<SplitPart> = self
            .parts
            .iter()
            .map(|p| SplitPart {
                message: p.message.clone(),
                pieces: Vec::new(),
            })
            .collect();
        for (fi, file) in self.source.files.iter().enumerate() {
            if file.whole_only.is_some() {
                if let Some(&k) = self.owner.get(&Unit::File(fi)) {
                    out[k].pieces.push(SplitPiece::File {
                        path: file.path.clone(),
                    });
                }
                continue;
            }
            for (hi, hunk) in file.hunks.iter().enumerate() {
                let changed = changed_lines(hunk);
                for (k, part) in out.iter_mut().enumerate() {
                    let mine: Vec<u32> = changed
                        .iter()
                        .copied()
                        .filter(|li| self.owner.get(&Unit::Line(fi, hi, *li)) == Some(&k))
                        .collect();
                    if mine.is_empty() {
                        continue;
                    }
                    part.pieces.push(if mine.len() == changed.len() {
                        SplitPiece::Hunk {
                            path: file.path.clone(),
                            hunk: hunk.id,
                        }
                    } else {
                        SplitPiece::Lines {
                            path: file.path.clone(),
                            hunk: hunk.id,
                            lines: mine,
                        }
                    });
                }
            }
        }
        out
    }

    /// The request [`Action::SplitCommit`] carries.
    pub(crate) fn request(&self) -> SplitCommit {
        SplitCommit {
            commit: self.source.commit,
            parts: self.build_parts(),
            sign: self.sign,
        }
    }

    /// The first thing `validate_split` (the backend's own check) would refuse.
    pub(crate) fn validation(&self) -> Result<(), SplitRefusal> {
        validate_split(&self.source, &self.build_parts())
    }

    /// How many pieces (whole files, hunks, or runs of lines) no part owns.
    pub(crate) fn unassigned_count(&self) -> usize {
        unassigned(&self.source, &self.build_parts()).len()
    }

    /// Why Split is disabled, in the UI language; `None` when it is not.
    pub(crate) fn validation_text(&self) -> Option<String> {
        self.validation().err().map(refusal_text)
    }

    /// What the footer says: the last refusal of an action, else the reason
    /// Split is disabled.
    pub(crate) fn status_text(&self) -> Option<String> {
        self.error.clone().or_else(|| self.validation_text())
    }

    // ---- editing the plan ---------------------------------------------------

    /// Gives the selected rows to `part` (`None` takes them back). A part
    /// that does not exist changes nothing.
    pub(crate) fn assign_selection(&mut self, part: Option<usize>) {
        if part.is_some_and(|k| k >= self.parts.len()) {
            return;
        }
        let units: Vec<Unit> = self
            .selection()
            .into_iter()
            .flat_map(|row| self.row_units(row))
            .collect();
        for unit in units {
            match part {
                Some(k) => {
                    self.owner.insert(unit, k);
                }
                None => {
                    self.owner.remove(&unit);
                }
            }
        }
        self.error = None;
    }

    /// Puts the selected rows into a new last part, which becomes selected.
    pub(crate) fn new_part_from_selection(&mut self) {
        let k = self.add_part();
        self.assign_selection(Some(k));
        self.pane = SplitPane::Changes;
        self.part_cursor = k;
    }

    /// Appends an empty part (empty message) and selects it.
    pub(crate) fn add_part(&mut self) -> usize {
        self.parts.push(PartState {
            message: String::new(),
        });
        self.part_cursor = self.parts.len() - 1;
        self.part_cursor
    }

    /// Deletes part `k`: its changes become unassigned. A split needs two
    /// parts, so the last two cannot be deleted.
    pub(crate) fn remove_part(&mut self, k: usize) {
        if k >= self.parts.len() {
            return;
        }
        if self.parts.len() <= 2 {
            self.error = Some(refusal_text(SplitRefusal::TooFewParts));
            return;
        }
        self.parts.remove(k);
        self.owner.retain(|_, o| *o != k);
        for o in self.owner.values_mut() {
            if *o > k {
                *o -= 1;
            }
        }
        self.part_cursor = self.part_cursor.min(self.parts.len() - 1);
        self.error = None;
    }

    /// Moves part `k` up (`-1`) or down (`+1`) the list; its changes go with it.
    pub(crate) fn move_part(&mut self, k: usize, delta: isize) {
        let j = k as isize + delta;
        if k >= self.parts.len() || j < 0 || j as usize >= self.parts.len() {
            return;
        }
        let j = j as usize;
        self.parts.swap(k, j);
        for o in self.owner.values_mut() {
            if *o == k {
                *o = j;
            } else if *o == j {
                *o = k;
            }
        }
        self.part_cursor = j;
    }

    /// Back to what the editor opened with.
    pub(crate) fn reset(&mut self) {
        self.parts = self.initial.0.clone();
        self.owner = self.initial.1.clone();
        self.part_cursor = 0;
        self.row_cursor = 0;
        self.lines_open = None;
        self.anchor = None;
        self.editing = None;
        self.error = None;
    }

    /// Line mode: the hunk under the cursor lists its changed lines (each can
    /// go to a part on its own); again, back to hunks. Whole files and
    /// whole-only files have no lines.
    pub(crate) fn toggle_line_mode(&mut self) {
        self.anchor = None;
        if let Some((fi, hi)) = self.lines_open.take() {
            self.select_row(SplitRow::Hunk(fi, hi));
            return;
        }
        if let Some(SplitRow::Hunk(fi, hi) | SplitRow::Line(fi, hi, _)) = self.cursor_row() {
            self.lines_open = Some((fi, hi));
            if let Some(first) = changed_lines(&self.source.files[fi].hunks[hi]).first() {
                self.select_row(SplitRow::Line(fi, hi, *first));
            }
        }
    }

    fn move_cursor(&mut self, delta: isize, extend: bool) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        if extend && self.lines_open.is_some() {
            self.anchor.get_or_insert(self.row_cursor);
        } else {
            self.anchor = None;
        }
        self.row_cursor =
            (self.row_cursor as isize + delta).clamp(0, (len as isize - 1).max(0)) as usize;
    }

    fn move_part_cursor(&mut self, delta: isize) {
        self.part_cursor = (self.part_cursor as isize + delta)
            .clamp(0, (self.parts.len() as isize - 1).max(0)) as usize;
    }

    pub(crate) fn toggle_pane(&mut self) {
        self.pane = match self.pane {
            SplitPane::Parts => SplitPane::Changes,
            SplitPane::Changes => SplitPane::Parts,
        };
    }

    pub(crate) fn select_part(&mut self, k: usize) {
        if k < self.parts.len() {
            self.pane = SplitPane::Parts;
            self.part_cursor = k;
        }
    }

    // ---- the assignment list (the keyboard twin of the per-row `[N v]`) -------

    /// Opens the list of commits the selected rows can be given to.
    pub(crate) fn open_assign_combo(&mut self) {
        let Some(row) = self.cursor_row() else { return };
        let mut combo = Combo::new("").with_click_picks();
        let mut options = vec![ComboOption::new(
            crate::split_ui::msg::COMBO_UNASSIGNED.get(),
            "-",
        )];
        for k in 0..self.parts.len() {
            options.push(ComboOption::new(self.part_label(k), k.to_string()));
        }
        options.push(ComboOption::new(
            crate::split_ui::msg::COMBO_NEW.get(),
            "new",
        ));
        combo.set_options(options);
        match self.row_owner(row) {
            RowOwner::Part(k) => combo.select(Some(k.to_string())),
            RowOwner::Unassigned => combo.select(Some("-".into())),
            RowOwner::Mixed => {}
        }
        combo.open();
        *self.view.combo.get_mut() = Some(combo);
    }

    /// "2 feat: the second part": the number and the summary of a message.
    pub(crate) fn part_label(&self, k: usize) -> String {
        let (summary, _) = crate::reword_ui::split_message(self.part_message(k));
        if summary.trim().is_empty() {
            format!("{} {}", k + 1, crate::split_ui::msg::NO_MESSAGE.get())
        } else {
            format!("{} {}", k + 1, summary)
        }
    }

    pub(crate) fn assign_combo_open(&self) -> bool {
        self.view.combo.borrow().is_some()
    }

    fn finish_combo(&mut self, outcome: ComboOutcome) {
        let open = self
            .view
            .combo
            .get_mut()
            .as_ref()
            .is_some_and(Combo::is_open);
        match outcome {
            ComboOutcome::Picked(value) => {
                *self.view.combo.get_mut() = None;
                match value.as_str() {
                    "-" => self.assign_selection(None),
                    "new" => self.new_part_from_selection(),
                    v => {
                        if let Ok(k) = v.parse::<usize>() {
                            self.assign_selection(Some(k));
                        }
                    }
                }
            }
            _ if !open => *self.view.combo.get_mut() = None,
            _ => {}
        }
    }

    /// A mouse event while the list is open: it is modal.
    pub(crate) fn on_combo_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        let Some(combo) = self.view.combo.get_mut().as_mut() else {
            return;
        };
        let outcome = combo.on_mouse(mouse);
        self.finish_combo(outcome);
    }

    fn on_combo_key(&mut self, key: KeyEvent) {
        let Some(combo) = self.view.combo.get_mut().as_mut() else {
            return;
        };
        let outcome = combo.on_key(key);
        self.finish_combo(outcome);
    }

    /// Extends the selection from the cursor row to `row` (a mouse drag over
    /// lines); only lines of the open hunk can be ranged.
    pub(crate) fn extend_selection_to(&mut self, row: SplitRow) {
        if !matches!(row, SplitRow::Line(..))
            || !matches!(self.cursor_row(), Some(SplitRow::Line(..)))
        {
            return;
        }
        if let Some(at) = self.rows().iter().position(|r| *r == row) {
            self.anchor.get_or_insert(self.row_cursor);
            self.row_cursor = at;
        }
    }

    /// The changes list's scrollbar took this mouse event.
    pub(crate) fn changes_bar_mouse(&self, mouse: crossterm::event::MouseEvent) -> bool {
        self.view.bar_mouse(mouse)
    }

    /// Scrolls the changes list by `delta` rows (the wheel).
    pub(crate) fn scroll_changes(&mut self, delta: isize) {
        self.view.scroll(delta);
    }

    // ---- closing and applying -----------------------------------------------

    /// The plan differs from what the editor opened with.
    pub(crate) fn is_dirty(&self) -> bool {
        self.parts != self.initial.0 || self.owner != self.initial.1
    }

    /// A close request from the cross, Cancel or a click outside: one rule
    /// for all (`dialog_kit::dismiss`). An outside click on an edited plan
    /// is swallowed and makes the cross flash.
    pub(crate) fn request_dismiss(
        &mut self,
        kind: crate::dialog_kit::DismissKind,
    ) -> SplitFormOutcome {
        use crate::dialog_kit::{dismiss, DismissAction};
        match dismiss(&render::Probe::new(self.is_dirty()), kind) {
            DismissAction::Close => SplitFormOutcome::Close,
            DismissAction::Swallow | DismissAction::Confirm => {
                self.view.flash_cross();
                SplitFormOutcome::Stay
            }
        }
    }

    /// The plan was sent: keep the editor, show the work in flight.
    pub(crate) fn begin_applying(&mut self) {
        self.applying = true;
        self.error = None;
    }

    pub(crate) fn is_applying(&self) -> bool {
        self.applying
    }

    /// The daemon refused or failed: the plan stays, the reason shows.
    pub(crate) fn refused(&mut self, text: String) {
        self.applying = false;
        self.error = Some(text);
    }

    /// The message editor, for the mouse (drag, release).
    pub(crate) fn editing_mut(&mut self) -> Option<&mut RewordForm> {
        self.editing.as_mut().map(|(_, f)| f)
    }

    // ---- the message editor (the Reword popup, embedded) ----------------------

    pub(crate) fn start_edit(&mut self, k: usize) {
        if let Some(p) = self.parts.get(k) {
            let form = RewordForm::new(self.source.commit, &p.message, None).embedded();
            self.editing = Some((k, form));
        }
    }

    fn finish_editing(&mut self, outcome: RewordFormOutcome) -> SplitFormOutcome {
        match outcome {
            RewordFormOutcome::Stay => {}
            RewordFormOutcome::Close | RewordFormOutcome::ModifyDeeper => self.editing = None,
            RewordFormOutcome::Submit(Action::Reword { message, .. }) => {
                if let Some((k, _)) = self.editing.take() {
                    if let Some(p) = self.parts.get_mut(k) {
                        p.message = message;
                    }
                }
            }
            RewordFormOutcome::Submit(_) => self.editing = None,
        }
        SplitFormOutcome::Stay
    }

    pub(crate) fn on_paste(&mut self, text: &str) {
        if let Some((_, form)) = self.editing.as_mut() {
            form.on_paste(text);
        }
    }

    // ---- apply ---------------------------------------------------------------

    /// Submits the split when the plan is valid; otherwise says why not.
    pub(crate) fn try_apply(&mut self) -> SplitFormOutcome {
        if self.applying {
            return SplitFormOutcome::Stay;
        }
        match self.validation() {
            Ok(()) => SplitFormOutcome::Submit(Action::SplitCommit {
                req: self.request(),
            }),
            Err(why) => {
                self.error = Some(refusal_text(why));
                SplitFormOutcome::Stay
            }
        }
    }

    // ---- keys ----------------------------------------------------------------

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> SplitFormOutcome {
        if self.applying {
            // Waiting for the daemon: only Esc leaves (the answer is then
            // reported on the status line, as for any action).
            return if key.code == KeyCode::Esc {
                SplitFormOutcome::Close
            } else {
                SplitFormOutcome::Stay
            };
        }
        self.error = None;
        if self.assign_combo_open() {
            self.on_combo_key(key);
            return SplitFormOutcome::Stay;
        }
        if let Some((_, form)) = self.editing.as_mut() {
            let outcome = form.on_key(key);
            return self.finish_editing(outcome);
        }
        let km = crate::keymap::current();
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => return SplitFormOutcome::Close,
            _ if km.is(Bind::ComposeApply, &key) => return self.try_apply(),
            KeyCode::Tab | KeyCode::BackTab => self.toggle_pane(),
            _ if km.is(Bind::SplitReset, &key) => self.reset(),
            _ if km.is(Bind::SplitMoveUp, &key) => self.move_part(self.part_cursor, -1),
            _ if km.is(Bind::SplitMoveDown, &key) => self.move_part(self.part_cursor, 1),
            _ if km.is(Bind::SplitLineMode, &key) => {
                if self.pane == SplitPane::Changes {
                    self.toggle_line_mode();
                }
            }
            _ => match self.pane {
                SplitPane::Parts => self.on_parts_key(key, plain),
                SplitPane::Changes => self.on_changes_key(key, plain),
            },
        }
        SplitFormOutcome::Stay
    }

    fn on_parts_key(&mut self, key: KeyEvent, plain: bool) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_part_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_part_cursor(1),
            KeyCode::Char('e') | KeyCode::Enter if plain => self.start_edit(self.part_cursor),
            KeyCode::Char('d') | KeyCode::Delete if plain => self.remove_part(self.part_cursor),
            KeyCode::Char('n') if plain => {
                self.add_part();
            }
            KeyCode::Char('s') if plain => self.sign = !self.sign,
            KeyCode::Char(c @ '1'..='9') if plain => self.select_part(c as usize - '1' as usize),
            _ => {}
        }
    }

    fn on_changes_key(&mut self, key: KeyEvent, plain: bool) {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Up if !key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_cursor(-1, shift);
            }
            KeyCode::Down if !key.modifiers.contains(KeyModifiers::ALT) => {
                self.move_cursor(1, shift);
            }
            KeyCode::Char('k') if plain => self.move_cursor(-1, false),
            KeyCode::Char('j') if plain => self.move_cursor(1, false),
            KeyCode::Char(c @ '1'..='9') if plain => {
                self.assign_selection(Some(c as usize - '1' as usize));
            }
            KeyCode::Char('0') if plain => self.assign_selection(None),
            KeyCode::Char('n') if plain => self.new_part_from_selection(),
            KeyCode::Char('s') if plain => self.sign = !self.sign,
            KeyCode::Enter | KeyCode::Char('m') if plain => self.open_assign_combo(),
            _ => {}
        }
    }
}

pub(crate) use gitpoulpi_i18n::msg::split_ui as msg;

/// A refusal in the UI language: the words the typed error uses.
pub(crate) fn refusal_text(refusal: SplitRefusal) -> String {
    crate::daemon_link::error_text::split_refusal(refusal)
        .get()
        .to_string()
}

#[cfg(test)]
mod tests;
