//! Row alignment for the A / B (/ Base) panes (plan section 3.3). Display
//! only: nothing here decides what the output is.
//!
//! Each pane shows its side's *real* file. Git's merge is the common
//! frame: its lines, with every conflict opened into a block as tall as
//! its tallest side. A side's file is diffed (histogram) against that
//! side's *projection* of the frame, the merged text with every conflict
//! replaced by the side's own block. Conflict blocks match one to one; the
//! other side's auto-merged changes show up as lines the file lacks (or
//! has differently), which is what the `░ B` tint is for.

use std::ops::Range;

use imara_diff::{Algorithm, Diff, InternedInput};

use super::MergeRegion;
use crate::domain::ConflictSide;

/// How a side's line relates to the merged text on its row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowMark {
    /// The line is the merged line (or the conflict block's line).
    Same,
    /// The side has a line here and it differs from the merged one: the
    /// other side changed it and git merged that change in.
    Changed,
    /// A line only this side has, with no merged counterpart on the row:
    /// the other side removed it and git merged the removal in.
    Extra,
    /// No line of this side on the row (a filler, or a line the other side
    /// added).
    Absent,
}

/// One screen row across the panes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignedRow {
    /// Index into the regions this row belongs to.
    pub region: usize,
    /// The conflict number (0-based, among conflicts only), for a row
    /// inside a conflict block.
    pub conflict: Option<usize>,
    /// Per side, `[base, ours, theirs]`: the 0-based line of that side's
    /// file shown on this row.
    pub lines: [Option<usize>; 3],
    pub marks: [RowMark; 3],
}

/// The aligned panes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlignedRows {
    pub rows: Vec<AlignedRow>,
    /// A side whose conflict blocks could not be pinned to its file (it
    /// should not happen): that pane is better shown as the projection.
    pub degraded: [bool; 3],
}

fn slot(side: ConflictSide) -> usize {
    match side {
        ConflictSide::Base => 0,
        ConflictSide::Ours => 1,
        ConflictSide::Theirs => 2,
    }
}

const SIDES: [ConflictSide; 3] = [ConflictSide::Base, ConflictSide::Ours, ConflictSide::Theirs];

impl AlignedRows {
    /// The rows seen from one side.
    pub fn side(&self, side: ConflictSide) -> Vec<SideRow> {
        let s = slot(side);
        self.rows
            .iter()
            .map(|r| SideRow {
                line: r.lines[s],
                mark: r.marks[s],
                region: r.region,
                conflict: r.conflict,
            })
            .collect()
    }

    /// The lines of `side`'s file that conflict `n` covers, as a range of
    /// line indexes. An empty block is an empty range at the position the
    /// block sits at. `side_len` is the side file's line count.
    pub fn conflict_range(
        &self,
        n: usize,
        side: ConflictSide,
        side_len: usize,
    ) -> Option<Range<usize>> {
        let s = slot(side);
        let block: Vec<usize> = self
            .rows
            .iter()
            .filter(|r| r.conflict == Some(n))
            .filter_map(|r| r.lines[s])
            .collect();
        if let (Some(&first), Some(&last)) = (block.first(), block.last()) {
            return Some(first..last + 1);
        }
        // Empty block: where the next present line of the side starts.
        let after_block = self
            .rows
            .iter()
            .skip_while(|r| r.conflict != Some(n))
            .skip_while(|r| r.conflict == Some(n))
            .find_map(|r| r.lines[s]);
        let before_block = self
            .rows
            .iter()
            .take_while(|r| r.conflict != Some(n))
            .filter_map(|r| r.lines[s])
            .last();
        self.rows.iter().any(|r| r.conflict == Some(n)).then(|| {
            let at = after_block
                .or(before_block.map(|b| b + 1))
                .unwrap_or(0)
                .min(side_len);
            at..at
        })
    }
}

/// One side's view of [`AlignedRows`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideRow {
    /// The 0-based line of the side's file on this row; `None` is a filler.
    pub line: Option<usize>,
    pub mark: RowMark,
    pub region: usize,
    pub conflict: Option<usize>,
}

/// A row of the frame: one merged line, or one row of a conflict block.
struct FrameRow {
    region: usize,
    conflict: Option<usize>,
    /// Per side, the text this row holds in the side's projection.
    text: [Option<String>; 3],
}

/// Lines to one string with one `\n` per line, so a `&str` token source
/// yields exactly one token per line.
fn joined(lines: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(l.as_ref());
        s.push('\n');
    }
    s
}

/// What the diff of one side's file against its projection found, per
/// projection element.
struct SideAlignment {
    /// Per projection element: the file line shown with it and its mark.
    per_elem: Vec<Option<(usize, RowMark)>>,
    /// Extra file lines, keyed by the projection index they sit before.
    gaps: Vec<(usize, Vec<usize>)>,
}

fn align_side(file: &[String], projection: &[&str]) -> SideAlignment {
    let before = joined(file);
    let after = joined(projection);
    let input = InternedInput::new(before.as_str(), after.as_str());
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);

    let mut per_elem: Vec<Option<(usize, RowMark)>> = vec![None; projection.len()];
    let mut gaps: Vec<(usize, Vec<usize>)> = Vec::new();
    let (mut f, mut p) = (0usize, 0usize);
    fn match_up_to(
        per_elem: &mut [Option<(usize, RowMark)>],
        f: &mut usize,
        p: &mut usize,
        f_end: usize,
        p_end: usize,
    ) {
        while *f < f_end && *p < p_end {
            per_elem[*p] = Some((*f, RowMark::Same));
            *f += 1;
            *p += 1;
        }
    }
    for hunk in diff.hunks() {
        let (b, a) = (
            hunk.before.start as usize..hunk.before.end as usize,
            hunk.after.start as usize..hunk.after.end as usize,
        );
        match_up_to(&mut per_elem, &mut f, &mut p, b.start, a.start);
        let paired = b.len().min(a.len());
        for t in 0..paired {
            per_elem[a.start + t] = Some((b.start + t, RowMark::Changed));
        }
        if b.len() > paired {
            let at = if a.is_empty() { a.start } else { a.end };
            gaps.push((at, (b.start + paired..b.end).collect()));
        }
        f = b.end;
        p = a.end;
    }
    match_up_to(&mut per_elem, &mut f, &mut p, file.len(), projection.len());
    SideAlignment { per_elem, gaps }
}

/// How a line of a side's file differs from the merge base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineChange {
    /// The line is in the base too.
    Same,
    /// The line replaces a base line (the diff view's old / new pair).
    Changed,
    /// The line has no base counterpart.
    Added,
}

/// A side's file diffed against the merge base: the data behind the
/// "Full file" coloring. The side's lines are what is shown, so a base
/// line the side removed is recorded as a position, not a row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SideChanges {
    /// Per line of the side's file.
    pub lines: Vec<LineChange>,
    /// Per line of the side's file (and one past the end): base lines the
    /// side removed sit just before it.
    pub removed_before: Vec<bool>,
}

/// Diffs `side` against `base` (histogram, line by line, the same
/// algorithm the alignment uses).
pub fn side_changes(base: &[String], side: &[String]) -> SideChanges {
    let before = joined(base);
    let after = joined(side);
    let input = InternedInput::new(before.as_str(), after.as_str());
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    let mut out = SideChanges {
        lines: vec![LineChange::Same; side.len()],
        removed_before: vec![false; side.len() + 1],
    };
    for hunk in diff.hunks() {
        let (b, a) = (
            hunk.before.start as usize..hunk.before.end as usize,
            hunk.after.start as usize..hunk.after.end as usize,
        );
        let paired = b.len().min(a.len());
        for (t, i) in a.clone().enumerate() {
            out.lines[i] = if t < paired {
                LineChange::Changed
            } else {
                LineChange::Added
            };
        }
        if b.len() > paired {
            let at = if a.is_empty() { a.start } else { a.end };
            out.removed_before[at.min(side.len())] = true;
        }
    }
    out
}

/// Aligns the panes. `sides` holds the real file lines of `[base, ours,
/// theirs]`, `None` for a pane not shown; `regions` is git's merge of
/// them. Every line of every shown side appears on exactly one row.
pub fn aligned_rows(regions: &[MergeRegion], sides: [Option<&[String]>; 3]) -> AlignedRows {
    // The frame, and per side the projection's frame row for each element.
    let mut frame: Vec<FrameRow> = Vec::new();
    let mut proj_rows: [Vec<usize>; 3] = Default::default();
    let mut conflict_no = 0;
    for (r, region) in regions.iter().enumerate() {
        match region {
            MergeRegion::Merged { lines } => {
                for line in lines {
                    let row = frame.len();
                    let mut text: [Option<String>; 3] = Default::default();
                    for s in 0..3 {
                        if sides[s].is_some() {
                            text[s] = Some(line.clone());
                            proj_rows[s].push(row);
                        }
                    }
                    frame.push(FrameRow {
                        region: r,
                        conflict: None,
                        text,
                    });
                }
            }
            MergeRegion::Conflict { .. } => {
                let height = SIDES
                    .iter()
                    .filter(|&&side| sides[slot(side)].is_some())
                    .map(|&side| region.lines_of(side).len())
                    .max()
                    .unwrap_or(0);
                let start = frame.len();
                for k in 0..height {
                    let mut text: [Option<String>; 3] = Default::default();
                    for &side in &SIDES {
                        let s = slot(side);
                        if sides[s].is_some() {
                            if let Some(line) = region.lines_of(side).get(k) {
                                text[s] = Some(line.clone());
                                proj_rows[s].push(start + k);
                            }
                        }
                    }
                    frame.push(FrameRow {
                        region: r,
                        conflict: Some(conflict_no),
                        text,
                    });
                }
                conflict_no += 1;
            }
        }
    }

    // Per side: diff the file against the projection.
    let mut shown: Vec<[Option<(usize, RowMark)>; 3]> = vec![Default::default(); frame.len()];
    let mut gap_at: Vec<[Vec<usize>; 3]> = vec![Default::default(); frame.len() + 1];
    let mut degraded = [false; 3];
    for s in 0..3 {
        let Some(file) = sides[s] else { continue };
        let projection: Vec<&str> = proj_rows[s]
            .iter()
            .map(|&row| frame[row].text[s].as_deref().unwrap_or_default())
            .collect();
        let aligned = align_side(file, &projection);
        for (j, elem) in aligned.per_elem.iter().enumerate() {
            let row = proj_rows[s][j];
            shown[row][s] = *elem;
            if frame[row].conflict.is_some() && !matches!(elem, Some((_, RowMark::Same))) {
                degraded[s] = true;
            }
        }
        for (before_elem, lines) in aligned.gaps {
            let key = proj_rows[s]
                .get(before_elem)
                .copied()
                .unwrap_or(frame.len());
            gap_at[key][s].extend(lines);
        }
    }

    let mut rows = Vec::with_capacity(frame.len());
    for (f, fr) in frame.iter().enumerate().map(|(f, fr)| (f, Some(fr))).chain(
        // The gap after the last frame row.
        std::iter::once((frame.len(), None)),
    ) {
        let region = fr
            .map(|fr| fr.region)
            .unwrap_or_else(|| regions.len().saturating_sub(1));
        let n = gap_at[f].iter().map(Vec::len).max().unwrap_or(0);
        for t in 0..n {
            let mut row = AlignedRow {
                region,
                conflict: None,
                lines: [None; 3],
                marks: [RowMark::Absent; 3],
            };
            for (s, gap) in gap_at[f].iter().enumerate() {
                if let Some(&line) = gap.get(t) {
                    row.lines[s] = Some(line);
                    row.marks[s] = RowMark::Extra;
                }
            }
            rows.push(row);
        }
        if let Some(fr) = fr {
            let mut row = AlignedRow {
                region: fr.region,
                conflict: fr.conflict,
                lines: [None; 3],
                marks: [RowMark::Absent; 3],
            };
            for (s, cell) in shown[f].iter().enumerate() {
                if let Some((line, mark)) = *cell {
                    row.lines[s] = Some(line);
                    row.marks[s] = mark;
                }
            }
            rows.push(row);
        }
    }
    AlignedRows { rows, degraded }
}

/// The rows of one side alone, over the merged frame: the delete/modify
/// card shows the surviving side this way, and a pane that is the only
/// one alive has no other side to align with.
pub fn side_rows(
    side_lines: &[String],
    regions: &[MergeRegion],
    side: ConflictSide,
) -> Vec<SideRow> {
    let mut sides: [Option<&[String]>; 3] = [None; 3];
    sides[slot(side)] = Some(side_lines);
    aligned_rows(regions, sides).side(side)
}
