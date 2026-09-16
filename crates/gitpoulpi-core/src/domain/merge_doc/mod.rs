//! The pure model behind the merge conflict editor (plan:
//! the merge-conflict-editor design, sections 3 and 4).
//!
//! No I/O: the backends read the three index stages and run git's own
//! 3-way merge (`git merge-file`) on them; everything from there on is
//! here, so the TUI's checkboxes, the CLI's `--pick 1=a,2=ba` and an
//! agent's proposal all reach the output through [`compose_resolution`]
//! and cannot disagree.
//!
//! * [`marker_size_for`] / [`parse_merge_file_output`]: git writes the
//!   merge with a marker size no input line can be mistaken for, and the
//!   output is parsed into [`MergeRegion`]s. Markers in the *worktree*
//!   file are never parsed.
//! * [`classify_sides`] / [`MergeDocument::assemble`]: the I/O-free halves
//!   of `GitBackend::merge_document`, shared by both backends so they
//!   build the same document from the same bytes.
//! * [`compose_resolution`]: picks to text.
//! * [`aligned_rows`] / [`side_rows`] (`align.rs`): the A/B/Base panes'
//!   row alignment.

mod align;
#[cfg(test)]
mod tests;
mod whitespace;

pub use align::{
    aligned_rows, side_changes, side_rows, AlignedRow, AlignedRows, LineChange, RowMark,
    SideChanges, SideRow,
};
pub use whitespace::{is_whitespace_only, whitespace_choice, whitespace_resolution};

use serde::{Deserialize, Serialize};

use super::{
    decode_text_lossy, fallback_encoding_name, BlobContent, ConflictEntry, ConflictKind,
    ConflictSide, StageEntry, MAX_TEXT_BLOB_BYTES, MODE_GITLINK,
};
use crate::{Error, Result};

/// The smallest conflict marker size (git's default).
pub const MIN_MARKER_SIZE: usize = 7;

/// A marker size for `git merge-file --marker-size` such that no line of
/// any of `sides` can be mistaken for a marker: one more than the longest
/// run of `<`, `=`, `>` or `|` any line starts with, and at least 7. A
/// Markdown file that legitimately contains `=======` therefore still
/// parses.
pub fn marker_size_for(sides: &[&[u8]]) -> usize {
    let mut longest = 0;
    for side in sides {
        for line in side.split(|&b| b == b'\n') {
            if let Some(&c) = line.first() {
                if matches!(c, b'<' | b'=' | b'>' | b'|') {
                    longest = longest.max(line.iter().take_while(|&&b| b == c).count());
                }
            }
        }
    }
    (longest + 1).max(MIN_MARKER_SIZE)
}

/// Whether `line` (without its `\n`) is a conflict marker of `size` and
/// kind `kind`: exactly `size` repeats of `kind`, then a space or the end
/// of the line. A trailing `\r` is ignored (git writes CRLF markers into a
/// CRLF file).
fn is_marker(line: &[u8], kind: u8, size: usize) -> bool {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    line.len() >= size
        && line[..size].iter().all(|&b| b == kind)
        && (line.len() == size || line[size] == b' ')
}

/// Whether `bytes` contain a line that looks like a git conflict marker
/// (`<<<<<<<` / `>>>>>>>` of at least the default size followed by a space
/// or the end of the line).
pub fn has_conflict_markers(bytes: &[u8]) -> bool {
    bytes.split(|&b| b == b'\n').any(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        b"<>".iter().copied().any(|kind| {
            let run = line.iter().take_while(|&&b| b == kind).count();
            run >= MIN_MARKER_SIZE && (line.len() == run || line[run] == b' ')
        })
    })
}

/// One stretch of the merge: text git merged by itself, or a conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeRegion {
    /// Git's auto-merge result: lines both sides agree on, or one side
    /// changed and the other did not.
    Merged { lines: Vec<String> },
    /// Both sides changed the same place. `base` is empty when there is no
    /// common ancestor (add/add) or the ancestor had nothing here.
    Conflict {
        base: Vec<String>,
        ours: Vec<String>,
        theirs: Vec<String>,
    },
}

impl MergeRegion {
    pub fn is_conflict(&self) -> bool {
        matches!(self, MergeRegion::Conflict { .. })
    }

    /// The lines `side` contributes to a conflict. A merged region has
    /// the same lines whichever side is asked for.
    pub fn lines_of(&self, side: ConflictSide) -> &[String] {
        match self {
            MergeRegion::Merged { lines } => lines,
            MergeRegion::Conflict { base, ours, theirs } => match side {
                ConflictSide::Base => base,
                ConflictSide::Ours => ours,
                ConflictSide::Theirs => theirs,
            },
        }
    }
}

fn bad_output(why: &str) -> Error {
    Error::Backend(format!("unexpected `git merge-file` output: {why}"))
}

/// Parses `git merge-file -p --diff3` / `--zdiff3` output (written with
/// `--marker-size=marker`) into regions. A merge without a `|||||||`
/// section (plain style) parses too, with empty bases.
///
/// Splits on raw `\n` *before* decoding each line, so a multi-byte
/// fallback encoding cannot confuse the markers. A line's `\r` stays in it.
/// Text git printed after the last `\n` is a final line with no newline.
pub fn parse_merge_file_output(bytes: &[u8], marker: usize) -> Result<Vec<MergeRegion>> {
    #[derive(Clone, Copy, PartialEq)]
    enum In {
        Merged,
        Ours,
        Base,
        Theirs,
    }
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if bytes.is_empty() || bytes.ends_with(b"\n") {
        lines.pop();
    }

    let decode = |l: &[u8]| decode_text_lossy(l).into_owned();
    let mut regions = Vec::new();
    let mut state = In::Merged;
    let mut merged: Vec<String> = Vec::new();
    let (mut base, mut ours, mut theirs) = (Vec::new(), Vec::new(), Vec::new());

    for line in lines {
        let kind = b"<|=>"
            .iter()
            .copied()
            .find(|&k| is_marker(line, k, marker));
        match (state, kind) {
            (In::Merged, Some(b'<')) => {
                if !merged.is_empty() {
                    regions.push(MergeRegion::Merged {
                        lines: std::mem::take(&mut merged),
                    });
                }
                state = In::Ours;
            }
            (In::Ours, Some(b'|')) => state = In::Base,
            (In::Ours | In::Base, Some(b'=')) => state = In::Theirs,
            (In::Theirs, Some(b'>')) => {
                regions.push(MergeRegion::Conflict {
                    base: std::mem::take(&mut base),
                    ours: std::mem::take(&mut ours),
                    theirs: std::mem::take(&mut theirs),
                });
                state = In::Merged;
            }
            (_, Some(k)) => {
                return Err(bad_output(&format!(
                    "a `{}` marker where none belongs",
                    k as char
                )))
            }
            (In::Merged, None) => merged.push(decode(line)),
            (In::Ours, None) => ours.push(decode(line)),
            (In::Base, None) => base.push(decode(line)),
            (In::Theirs, None) => theirs.push(decode(line)),
        }
    }
    if state != In::Merged {
        return Err(bad_output("a conflict that never closes"));
    }
    if !merged.is_empty() {
        regions.push(MergeRegion::Merged { lines: merged });
    }
    Ok(regions)
}

/// How to print regions back as a file with conflict markers.
#[derive(Debug, Clone)]
pub struct MarkerStyle<'a> {
    pub marker_size: usize,
    /// `true` writes the `|||||||` base section (diff3 / zdiff3 style).
    pub diff3: bool,
    pub ours_label: &'a str,
    pub base_label: &'a str,
    pub theirs_label: &'a str,
    /// The line terminator: `"\n"` or `"\r\n"`.
    pub eol: &'a str,
    pub final_newline: bool,
}

/// The file git would have written for `regions`: the merged lines and
/// every conflict between markers.
pub fn render_with_markers(regions: &[MergeRegion], style: &MarkerStyle<'_>) -> String {
    let marker = |c: char, label: &str| {
        let mut m: String = c.to_string().repeat(style.marker_size);
        if !label.is_empty() {
            m.push(' ');
            m.push_str(label);
        }
        m
    };
    let mut out: Vec<String> = Vec::new();
    for region in regions {
        match region {
            MergeRegion::Merged { lines } => out.extend(lines.iter().cloned()),
            MergeRegion::Conflict { base, ours, theirs } => {
                out.push(marker('<', style.ours_label));
                out.extend(ours.iter().cloned());
                if style.diff3 {
                    out.push(marker('|', style.base_label));
                    out.extend(base.iter().cloned());
                }
                out.push(marker('=', ""));
                out.extend(theirs.iter().cloned());
                out.push(marker('>', style.theirs_label));
            }
        }
    }
    let mut text = out.join(style.eol);
    if !out.is_empty() && style.final_newline {
        text.push_str(style.eol);
    }
    text
}

/// One side of the conflict, read: which blob, and its content as the
/// rest of GitPoulpi reads blobs ([`BlobContent`]: text, binary, too
/// large, LFS pointer). A submodule side carries `Binary` content and
/// size 0: its oid is a commit in another repository, and there is no
/// blob to read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideBlob {
    pub oid: super::Oid,
    pub mode: u32,
    pub size: u64,
    pub content: BlobContent,
}

impl SideBlob {
    /// The side's lines, or an empty slice when it is not text.
    pub fn lines(&self) -> &[String] {
        self.content.lines()
    }
}

/// Line terminator of the file the editor writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eol {
    Lf,
    Crlf,
}

/// What the file in the working tree looks like right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorktreeFileState {
    /// Still carries git's conflict markers: the normal case.
    Markers,
    /// No markers, and the same text as the fresh merge (a merge with no
    /// conflict left in it).
    CleanMatchesMerge,
    /// No markers, and different from the fresh merge: resolved by hand,
    /// by rerere, or by an earlier save that did not stage.
    EditedNoMarkers { crlf: bool },
    /// No file at that path (deleted on one side).
    Missing,
}

/// Which operation stopped on the conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConflictOperation {
    /// No operation marker found (a stash apply, say).
    #[default]
    None,
    Merge,
    CherryPick,
    Revert,
    Rebase,
}

/// The names of the panes. Roles and refs, never a bare "ours" / "theirs":
/// during a rebase git swaps their meaning.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SideLabels {
    /// Stage 2, e.g. `Current · main` or `Current · onto 1a2b3c4`.
    pub current: String,
    /// Stage 3, e.g. `Incoming · feature/login`.
    pub incoming: String,
    /// Stage 1, e.g. `Base`.
    pub base: String,
    pub op: ConflictOperation,
    /// A one-line description for the title: `merging feature/login into
    /// main`, `rebasing onto 1a2b3c4 · step 3/5`. Empty for no operation.
    pub description: String,
    /// The branch or ref names git itself would put after the markers
    /// (`HEAD` and the merged branch) - what `git merge` wrote in the
    /// worktree file, for byte-for-byte comparison.
    pub git_ours_label: String,
    pub git_theirs_label: String,
}

/// Everything the editor needs about one conflicted file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeDocument {
    pub entry: ConflictEntry,
    pub base: Option<SideBlob>,
    pub ours: Option<SideBlob>,
    pub theirs: Option<SideBlob>,
    /// Git's own merge of the three stages. Empty unless every present
    /// side is text and the path is a plain content conflict.
    pub regions: Vec<MergeRegion>,
    pub labels: SideLabels,
    pub worktree: WorktreeFileState,
    /// The line terminator to write (see the plan's 3.4): the working
    /// tree file's when it is uniformly CRLF or LF, else stage 2's.
    pub eol: Eol,
    /// Stage 2 (or 3, with no stage 2) began with a UTF-8 BOM. The BOM is
    /// stripped from every line here and belongs back on the first line of
    /// the file that is written.
    pub bom: bool,
    /// The merged text ends with a line terminator.
    pub final_newline: bool,
    /// The fallback encoding the text was decoded with, when some side
    /// was not valid UTF-8.
    pub encoding: Option<String>,
    /// The `--marker-size` the merge ran with.
    pub marker_size: usize,
}

impl MergeDocument {
    /// How many conflict regions there are.
    pub fn conflict_count(&self) -> usize {
        self.regions.iter().filter(|r| r.is_conflict()).count()
    }

    /// The conflict regions, in file order (conflict `n` is the `n`th).
    pub fn conflicts(&self) -> impl Iterator<Item = &MergeRegion> {
        self.regions.iter().filter(|r| r.is_conflict())
    }

    /// The blob of `side`.
    pub fn blob(&self, side: ConflictSide) -> Option<&SideBlob> {
        match side {
            ConflictSide::Base => self.base.as_ref(),
            ConflictSide::Ours => self.ours.as_ref(),
            ConflictSide::Theirs => self.theirs.as_ref(),
        }
    }

    /// The text git merged, with every conflict rendered as `style` says.
    pub fn render_markers(&self, style: &MarkerStyle<'_>) -> String {
        render_with_markers(&self.regions, style)
    }

    /// The three sides' lines, indexed `[base, ours, theirs]`, for
    /// [`aligned_rows`]: `None` for a side that is absent or not text.
    pub fn side_lines(&self) -> [Option<&[String]>; 3] {
        fn text(b: &Option<SideBlob>) -> Option<&[String]> {
            b.as_ref()
                .filter(|b| matches!(b.content, BlobContent::Text { .. }))
                .map(|b| b.lines())
        }
        [text(&self.base), text(&self.ours), text(&self.theirs)]
    }
}

/// An index stage's bytes, as a backend read them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawStage {
    pub entry: StageEntry,
    /// The blob's byte length (git2 `read_header`, `cat-file` header),
    /// known even when `bytes` was not read.
    pub size: u64,
    /// `None` when the blob was not read: over the size cap, or a gitlink.
    pub bytes: Option<Vec<u8>>,
}

/// The bytes `git merge-file` runs on, and how to run it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeJob {
    pub marker: usize,
    /// Empty for add/add.
    pub base: Vec<u8>,
    pub ours: Vec<u8>,
    pub theirs: Vec<u8>,
}

/// Turns the raw stages into [`SideBlob`]s (`[base, ours, theirs]`) and
/// decides whether there is a text merge to run: only a plain
/// both-modified or both-added path whose present sides are all text.
pub fn classify_sides(
    entry: &ConflictEntry,
    raws: [Option<RawStage>; 3],
) -> ([Option<SideBlob>; 3], Option<MergeJob>) {
    let blobs = raws.each_ref().map(|raw| {
        raw.as_ref().map(|raw| {
            let content = match (&raw.bytes, raw.entry.mode) {
                (_, MODE_GITLINK) => BlobContent::Binary,
                (Some(bytes), _) if raw.size <= MAX_TEXT_BLOB_BYTES => {
                    BlobContent::from_bytes(bytes)
                }
                _ => BlobContent::TooLarge,
            };
            let size = if raw.entry.mode == MODE_GITLINK {
                0
            } else {
                raw.size
            };
            SideBlob {
                oid: raw.entry.oid,
                mode: raw.entry.mode,
                size,
                content,
            }
        })
    });

    let is_text = |b: &Option<SideBlob>| {
        b.as_ref()
            .map_or(true, |b| matches!(b.content, BlobContent::Text { .. }))
    };
    let mergeable = matches!(
        entry.kind,
        ConflictKind::BothModified | ConflictKind::BothAdded
    ) && !entry.is_submodule()
        && !entry.is_symlink()
        && blobs[1].is_some()
        && blobs[2].is_some()
        && blobs.iter().all(is_text);
    let job = if mergeable {
        let [base, ours, theirs] = raws;
        let bytes = |r: Option<RawStage>| r.and_then(|r| r.bytes).unwrap_or_default();
        let (base, ours, theirs) = (bytes(base), bytes(ours), bytes(theirs));
        Some(MergeJob {
            marker: marker_size_for(&[&base, &ours, &theirs]),
            base,
            ours,
            theirs,
        })
    } else {
        None
    };
    (blobs, job)
}

/// `Some(Crlf)` when every `\n` of `bytes` follows a `\r`, `Some(Lf)` when
/// none does, `None` when mixed or with no newline at all.
fn uniform_eol(bytes: &[u8]) -> Option<Eol> {
    let newlines = bytes.iter().filter(|&&b| b == b'\n').count();
    if newlines == 0 {
        return None;
    }
    let crlf = bytes.windows(2).filter(|w| w == b"\r\n").count();
    match crlf {
        0 => Some(Eol::Lf),
        n if n == newlines => Some(Eol::Crlf),
        _ => None,
    }
}

fn strip_cr(line: &mut String) {
    if line.ends_with('\r') {
        line.pop();
    }
}

fn strip_bom(line: &mut String) {
    if line.starts_with('\u{feff}') {
        line.remove(0);
    }
}

impl MergeDocument {
    /// Builds the document from what the backends gathered.
    ///
    /// * `blobs` / `merged`: from [`classify_sides`] and `git merge-file`'s
    ///   stdout (`None` when there was no job);
    /// * `worktree`: the bytes of the file on disk, if any.
    ///
    /// `\r` is stripped from every line and the BOM from the first, so
    /// regions and blobs index the same lines; [`Eol`] and `bom` carry what
    /// was taken out.
    pub fn assemble(
        entry: ConflictEntry,
        blobs: [Option<SideBlob>; 3],
        merged: Option<(usize, Vec<u8>)>,
        labels: SideLabels,
        worktree: Option<&[u8]>,
    ) -> Result<MergeDocument> {
        let [mut base, mut ours, mut theirs] = blobs;

        let mut encoding = None;
        let mut raw_text_seen = false;
        for blob in [&base, &ours, &theirs].into_iter().flatten() {
            raw_text_seen |= matches!(blob.content, BlobContent::Text { .. });
        }
        if raw_text_seen {
            encoding = fallback_encoding_name().map(str::to_string);
        }

        // Stage 2 decides the BOM and the fallback line ending.
        let first_line_bom = |b: &Option<SideBlob>| {
            b.as_ref()
                .is_some_and(|b| b.lines().first().is_some_and(|l| l.starts_with('\u{feff}')))
        };
        let bom = if ours.is_some() {
            first_line_bom(&ours)
        } else {
            first_line_bom(&theirs)
        };
        let stage_crlf = |b: &Option<SideBlob>| {
            b.as_ref()
                .is_some_and(|b| matches!(b.content, BlobContent::Text { crlf: true, .. }))
        };
        let stage_eol = if stage_crlf(if ours.is_some() { &ours } else { &theirs }) {
            Eol::Crlf
        } else {
            Eol::Lf
        };

        for blob in [&mut base, &mut ours, &mut theirs].into_iter().flatten() {
            if let BlobContent::Text { lines, .. } = &mut blob.content {
                if let Some(first) = lines.first_mut() {
                    strip_bom(first);
                }
            }
        }

        let (marker_size, mut regions, final_newline) = match merged {
            Some((marker, bytes)) => {
                let regions = parse_merge_file_output(&bytes, marker)?;
                let final_newline = bytes.is_empty() || bytes.ends_with(b"\n");
                (marker, regions, final_newline)
            }
            None => {
                let from_ours =
                    [&ours, &theirs]
                        .into_iter()
                        .flatten()
                        .find_map(|b| match b.content {
                            BlobContent::Text { final_newline, .. } => Some(final_newline),
                            _ => None,
                        });
                (MIN_MARKER_SIZE, Vec::new(), from_ours.unwrap_or(true))
            }
        };
        let mut first_line = true;
        for region in &mut regions {
            let each = |lines: &mut Vec<String>| {
                for line in lines.iter_mut() {
                    strip_cr(line);
                }
                if first_line {
                    if let Some(l) = lines.first_mut() {
                        strip_bom(l);
                    }
                }
            };
            match region {
                MergeRegion::Merged { lines } => {
                    each(lines);
                    first_line = false;
                }
                MergeRegion::Conflict { base, ours, theirs } => {
                    // Only the very first line of the file can carry a BOM,
                    // wherever it landed in this region.
                    each(base);
                    each(ours);
                    each(theirs);
                    first_line = false;
                }
            }
        }

        let eol = worktree.and_then(uniform_eol).unwrap_or(stage_eol);

        let worktree_state = match worktree {
            None => WorktreeFileState::Missing,
            Some(bytes) if has_conflict_markers(bytes) => WorktreeFileState::Markers,
            Some(bytes) => {
                let clean = !regions.is_empty() && regions.iter().all(|r| !r.is_conflict());
                let on_disk: Vec<String> = decode_text_lossy(bytes)
                    .split('\n')
                    .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
                    .collect();
                let mut on_disk = on_disk;
                if on_disk.last().is_some_and(String::is_empty) {
                    on_disk.pop();
                }
                if let Some(first) = on_disk.first_mut() {
                    strip_bom(first);
                }
                let merged_lines: Vec<&String> = regions
                    .iter()
                    .flat_map(|r| r.lines_of(ConflictSide::Ours))
                    .collect();
                if clean && merged_lines.iter().copied().eq(on_disk.iter()) {
                    WorktreeFileState::CleanMatchesMerge
                } else {
                    WorktreeFileState::EditedNoMarkers {
                        crlf: uniform_eol(bytes) == Some(Eol::Crlf),
                    }
                }
            }
        };

        Ok(MergeDocument {
            entry,
            base,
            ours,
            theirs,
            regions,
            labels,
            worktree: worktree_state,
            eol,
            bom,
            final_newline,
            encoding,
            marker_size,
        })
    }
}

/// One picked line: line `line` (0-based, within the conflict's block) of
/// `side`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pick {
    pub side: ConflictSide,
    pub line: usize,
}

/// What is taken for one conflict, and in which order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictPicks {
    pub picks: Vec<Pick>,
    /// The sides in the order they were first picked: the first comes
    /// first in the output. Within a side lines keep their file order.
    pub order: Vec<ConflictSide>,
}

impl ConflictPicks {
    /// Nothing picked: the conflict resolves to no lines.
    pub fn none() -> Self {
        Self::default()
    }

    /// Every line of each of `sides`, in that order, taken from `region`
    /// (`take(region, &[Ours])` is "take A", `&[Theirs, Ours]` is "B then
    /// A"). A merged region has nothing to pick.
    pub fn take(region: &MergeRegion, sides: &[ConflictSide]) -> Self {
        let mut out = Self::default();
        if !region.is_conflict() {
            return out;
        }
        for &side in sides {
            if out.order.contains(&side) {
                continue;
            }
            let n = region.lines_of(side).len();
            out.picks.extend((0..n).map(|line| Pick { side, line }));
            out.order.push(side);
        }
        out
    }

    /// The sides in the order they come out: `order`, then any side that
    /// has picks but is not listed, in Ours, Theirs, Base order.
    pub fn effective_order(&self) -> Vec<ConflictSide> {
        let mut order = self.order.clone();
        for side in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
            if !order.contains(&side) && self.picks.iter().any(|p| p.side == side) {
                order.push(side);
            }
        }
        order
    }

    /// The lines these picks yield for `region`.
    pub fn lines_for(&self, region: &MergeRegion) -> Vec<String> {
        let mut out = Vec::new();
        for side in self.effective_order() {
            let block = region.lines_of(side);
            let mut idx: Vec<usize> = self
                .picks
                .iter()
                .filter(|p| p.side == side && p.line < block.len())
                .map(|p| p.line)
                .collect();
            idx.sort_unstable();
            idx.dedup();
            out.extend(idx.into_iter().map(|i| block[i].clone()));
        }
        out
    }
}

/// The resolved file text (`\n` line endings, no BOM: the writer adds
/// both back from `doc.eol` / `doc.bom`): merged regions as git merged
/// them, conflict `n` as `picks[n]` says. A conflict with no entry in
/// `picks` resolves to nothing, like an empty pick.
pub fn compose_resolution(doc: &MergeDocument, picks: &[ConflictPicks]) -> String {
    let empty = ConflictPicks::none();
    let mut out = String::new();
    let mut n = 0;
    for region in &doc.regions {
        let lines = match region {
            MergeRegion::Merged { lines } => lines.clone(),
            conflict => {
                let lines = picks.get(n).unwrap_or(&empty).lines_for(conflict);
                n += 1;
                lines
            }
        };
        for line in lines {
            out.push_str(&line);
            out.push('\n');
        }
    }
    if !doc.final_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}
