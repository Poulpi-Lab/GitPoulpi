//! A file's change inside one rewritten commit: what a content rule did to
//! one path, as the workbench's "diff of the diff" shows it. Pure: the
//! backend reads the two blobs, this turns them into hunks.

use std::ops::Range;

use imara_diff::{Algorithm, Diff, InternedInput};
use serde::{Deserialize, Serialize};

use crate::domain::Oid;

use super::RunId;

/// Lines of context around a change.
const CONTEXT: usize = 3;

/// What a secret rule writes in place of a secret when it names no text.
pub const REMOVED_TEXT: &str = "***REMOVED***";

/// Shown instead of a line that held a secret.
pub const REDACTED_LINE: &str = "(line hidden: it held a secret)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobChangeKind {
    /// The file stays, with other content.
    Modified,
    /// The rewrite removes the file from this commit.
    Removed,
    /// The rewrite leaves the file as it is.
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobLineKind {
    Context,
    Removed,
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobLine {
    pub kind: BlobLineKind,
    /// The line without its line ending.
    pub text: String,
}

/// A run of changed lines with context; line numbers start at 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<BlobLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobChange {
    pub run: RunId,
    pub commit: Oid,
    pub path: String,
    pub kind: BlobChangeKind,
    /// Binary content (or not text): sizes only, no hunks.
    pub binary: bool,
    /// The removed lines are hidden because a secret rule made the change.
    pub redacted: bool,
    pub old_size: u64,
    pub new_size: u64,
    pub hunks: Vec<BlobHunk>,
}

fn trim_eol(line: &str) -> &str {
    line.strip_suffix('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .unwrap_or(line)
}

/// The hunks that turn `old` into `new`. With `redact_old`, a removed line
/// is replaced by [`REDACTED_LINE`]: the old text held what a secret rule
/// removes.
pub fn blob_diff(old: &str, new: &str, redact_old: bool) -> Vec<BlobHunk> {
    let input = InternedInput::new(old, new);
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    let old_lines: Vec<&str> = old.split_inclusive('\n').collect();
    let new_lines: Vec<&str> = new.split_inclusive('\n').collect();

    // Changes whose context would touch share a hunk.
    let mut groups: Vec<Vec<(Range<usize>, Range<usize>)>> = Vec::new();
    for h in diff.hunks() {
        let before = h.before.start as usize..h.before.end as usize;
        let after = h.after.start as usize..h.after.end as usize;
        match groups.last_mut() {
            Some(g) if before.start <= g.last().map_or(0, |l| l.0.end) + 2 * CONTEXT => {
                g.push((before, after));
            }
            _ => groups.push(vec![(before, after)]),
        }
    }

    let context = |lines: &mut Vec<BlobLine>, from: usize, to: usize| {
        for l in &old_lines[from..to] {
            lines.push(BlobLine {
                kind: BlobLineKind::Context,
                text: trim_eol(l).to_string(),
            });
        }
    };
    groups
        .into_iter()
        .map(|group| {
            let first = &group[0];
            let start_old = first.0.start.saturating_sub(CONTEXT);
            let start_new = first.1.start.saturating_sub(CONTEXT);
            let mut lines = Vec::new();
            let mut at = start_old;
            for (before, after) in &group {
                context(&mut lines, at, before.start);
                for l in &old_lines[before.clone()] {
                    lines.push(BlobLine {
                        kind: BlobLineKind::Removed,
                        text: if redact_old {
                            REDACTED_LINE.to_string()
                        } else {
                            trim_eol(l).to_string()
                        },
                    });
                }
                for l in &new_lines[after.clone()] {
                    lines.push(BlobLine {
                        kind: BlobLineKind::Added,
                        text: trim_eol(l).to_string(),
                    });
                }
                at = before.end;
            }
            let last_end = group.last().map_or(at, |l| l.0.end);
            context(
                &mut lines,
                last_end,
                (last_end + CONTEXT).min(old_lines.len()),
            );
            BlobHunk {
                old_start: start_old as u32 + 1,
                new_start: start_new as u32 + 1,
                lines,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_changed_line_has_context_and_both_sides() {
        let old = "a\nb\nTOKEN\nc\nd\n";
        let new = "a\nb\n***\nc\nd\n";
        let hunks = blob_diff(old, new, false);
        assert_eq!(hunks.len(), 1);
        let kinds: Vec<_> = hunks[0].lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            [
                BlobLineKind::Context,
                BlobLineKind::Context,
                BlobLineKind::Removed,
                BlobLineKind::Added,
                BlobLineKind::Context,
                BlobLineKind::Context
            ]
        );
        assert_eq!(hunks[0].lines[2].text, "TOKEN");
        assert_eq!(hunks[0].old_start, 1);
    }

    #[test]
    fn redaction_hides_the_removed_lines_only() {
        let hunks = blob_diff("x\nSECRET\n", "x\n***\n", true);
        let text: Vec<_> = hunks[0].lines.iter().map(|l| l.text.as_str()).collect();
        assert!(!text.iter().any(|t| t.contains("SECRET")));
        assert!(text.contains(&"***"));
    }

    #[test]
    fn far_apart_changes_are_two_hunks_and_equal_texts_none() {
        let old: String = (0..30).map(|i| format!("l{i}\n")).collect();
        let new = old.replace("l2\n", "L2\n").replace("l27\n", "L27\n");
        assert_eq!(blob_diff(&old, &new, false).len(), 2);
        assert!(blob_diff(&old, &old, false).is_empty());
    }
}
