//! The diff's row model: what each logical row stands for, so a screen row
//! can be mapped back to something stageable. No layout, no styling.

use gitpoulpi_core::domain::{HunkId, LineOrigin};

/// What one rendered row shows, so a screen row can be mapped back to
/// something stageable.
///
/// Built by the same walk that produces the rows themselves: a second
/// walk would drift, and here the cost is a `StageTarget` naming the
/// wrong lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiffRow {
    /// The hunk this row belongs to, `None` only for the two "nothing to
    /// show here" notices.
    pub hunk: Option<HunkId>,
    /// Whether this is the hunk's `@@ ... @@` header row.
    pub header: bool,
    /// The changed (non-context) lines this row shows, numbered as the
    /// backend numbers them: indices into the original `Hunk::lines`
    /// (`LineSelector::line_indices`).
    ///
    /// One entry in the unified layouts; up to two in Split, where a
    /// delete and its paired add share a row. Empty on a header, a
    /// context row, a word-wrap continuation and an elision marker.
    /// Context is not selectable: `filter_hunk_to_selection` keeps it
    /// either way, so "selected lines" means "selected changes", as in
    /// `git add -p`.
    pub lines: [Option<u32>; 2],
    /// The file line numbers this row shows, `[old side, new side]`.
    /// `None` where the side has no line here, and on headers and notices;
    /// wrapped continuation rows repeat them. What `ui show diff --line`
    /// and the highlight band find their rows by.
    pub numbers: [Option<u32>; 2],
    /// What kind of change the row is, for the scrollbar's overview ruler.
    /// Unlike `lines` it is also set on rows staging cannot act on.
    pub change: RowChange,
    /// Set on a row an annotator added under a code line (a review's
    /// comment thread, say): what it is and which side of the diff it
    /// belongs to. Such a row belongs to no hunk and carries no changed line.
    pub annot: Option<AnnotTag>,
}

/// What an annotation row is, as its owner (the review view) names it:
/// `kind` and `index` are the maintainer's vocabulary, `side` says where the row
/// sits (see the frontend annotation region).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnnotTag {
    pub kind: u8,
    pub index: u32,
    pub side: AnnotSide,
}

/// Where an annotation row is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotSide {
    /// Under the old (left) half in Split, the whole text column in Inline.
    Old,
    /// Under the new (right) half in Split, the whole text column in Inline.
    New,
    /// The whole text column in both layouts (a list after the diff).
    Span,
    /// The whole row, from the first column (a heading).
    Full,
}

/// The change a diff row stands for (see `DiffRow::change`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowChange {
    #[default]
    None,
    Add,
    Del,
    /// A Split row pairing a deletion with an addition.
    Both,
}

impl RowChange {
    pub fn of(origin: &LineOrigin) -> Self {
        match origin {
            LineOrigin::Add => RowChange::Add,
            LineOrigin::Delete => RowChange::Del,
            _ => RowChange::None,
        }
    }
}

impl DiffRow {
    pub fn header(hunk: HunkId) -> Self {
        Self {
            hunk: Some(hunk),
            header: true,
            lines: [None, None],
            numbers: [None, None],
            change: RowChange::None,
            annot: None,
        }
    }

    pub fn body(hunk: HunkId, lines: [Option<u32>; 2]) -> Self {
        Self {
            hunk: Some(hunk),
            header: false,
            lines,
            numbers: [None, None],
            change: RowChange::None,
            annot: None,
        }
    }

    pub fn with_change(mut self, change: RowChange) -> Self {
        self.change = change;
        self
    }
}

/// The raw lines a diff render shows, per side (see `DiffRender::text`).
#[derive(Debug, Default)]
pub struct DiffText {
    pub sides: [Vec<String>; 2],
}

/// The hunk the row at `row` belongs to.
///
/// A free function over `&[DiffRow]`, not a method on `DiffRender`: the
/// render loop keeps only the row map between frames.
pub fn hunk_at(rows: &[DiffRow], row: usize) -> Option<HunkId> {
    rows.get(row).and_then(|r| r.hunk)
}

/// Which lines an inclusive row range covers, for a line-level
/// `StageTarget`.
///
/// Returns the hunk the range starts in, that hunk's selected line
/// indices, and whether the range spilled into another hunk. A
/// `LineSelector` names one hunk, and a second patch would be wrong: the
/// first `git apply --cached` changes the diff the second hunk's ids were
/// taken from. So the spill is reported and dropped, never applied.
pub fn selected_lines(
    rows: &[DiffRow],
    from: usize,
    to: usize,
) -> Option<(HunkId, Vec<u32>, bool)> {
    let rows = rows.get(from..=to.min(rows.len().saturating_sub(1)))?;
    let hunk = rows.iter().find_map(|r| r.hunk)?;
    let spilled = rows.iter().any(|r| r.hunk.is_some_and(|h| h != hunk));
    let mut lines: Vec<u32> = rows
        .iter()
        .filter(|r| r.hunk == Some(hunk))
        .flat_map(|r| r.lines.into_iter().flatten())
        .collect();
    lines.sort_unstable();
    lines.dedup();
    Some((hunk, lines, spilled))
}
