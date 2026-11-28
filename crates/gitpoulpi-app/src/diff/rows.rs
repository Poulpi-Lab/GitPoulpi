//! The diff's logical rows: which hunks survive the whitespace toggle, the
//! file line numbers of each line, and how Split pairs deletions with
//! additions.

use super::model::RowChange;
use gitpoulpi_core::domain::{DiffLine, FileDiff, Hunk, LineOrigin};

/// A hunk as the view shows it, plus the thread back to the hunk as the
/// backend sees it.
///
/// `origin[i]` is the position of `hunk.lines[i]` in the original
/// `FileDiff`'s hunk. The two numberings differ when the whitespace
/// toggle is off and `effective_hunks` drops, rewrites and reorders
/// lines; the backend resolves a `LineSelector` against its own diff, so
/// shipping the rewritten positions would stage lines the user never
/// pointed at.
pub struct EffectiveHunk {
    pub hunk: Hunk,
    pub origin: Vec<u32>,
}

/// Applies the whitespace toggle. With it on, hunks pass through
/// untouched. With it off, each run of `-` lines followed by `+` lines is
/// paired positionally; a pair whose contents match after `trim_end` is a
/// whitespace-only change and collapses to one context line.
///
/// Within a change run the collapsed context lines are emitted ahead of
/// the remaining `-`/`+` lines, which reorders rows only inside that run.
/// Dropping them instead would lose lines that exist in the file.
pub fn effective_hunks(file: &FileDiff, show_whitespace: bool) -> Vec<EffectiveHunk> {
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
pub fn line_numbers(hunk: &Hunk) -> Vec<(Option<u32>, Option<u32>)> {
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

/// One side of a split row: the index of the line in the hunk and the file
/// line number to print beside it.
pub type SplitSide = Option<(usize, u32)>;

/// Side-by-side pairing of a hunk's lines: a context line shows on both
/// sides, each run of deletions is paired positionally with the additions
/// that follow it, and an unmatched side is `None` (a blank half).
pub fn split_pairs(hunk: &Hunk) -> Vec<(SplitSide, SplitSide)> {
    let numbers = line_numbers(hunk);
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
    split_rows
}

/// What a split row stands for, from the change on each side.
pub fn split_change(hunk: &Hunk, left: SplitSide, right: SplitSide) -> RowChange {
    match (
        left.map(|(i, _)| RowChange::of(&hunk.lines[i].origin)),
        right.map(|(i, _)| RowChange::of(&hunk.lines[i].origin)),
    ) {
        (Some(RowChange::Del), Some(RowChange::Add)) => RowChange::Both,
        (Some(RowChange::Del), _) => RowChange::Del,
        (_, Some(RowChange::Add)) => RowChange::Add,
        _ => RowChange::None,
    }
}
