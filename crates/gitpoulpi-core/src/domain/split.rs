//! Split one commit into several (see the split-commit design).
//!
//! The types are shared by both backends, the daemon, the CLI/MCP and the
//! TUI editor. [`validate_split`] is the one pure check all of them run, so a
//! plan the editor accepts is a plan the backend accepts.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{normalize_reword_message, FileStatus, Hunk, HunkId, LineOrigin, Oid};

/// Why a file can only be assigned to a part as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WholeReason {
    Binary,
    ModeOnly,
    /// Added by the commit (a partial add breaks the `new file` header).
    Added,
    /// Deleted by the commit.
    Deleted,
    Submodule,
    Lfs,
}

/// One file changed by the commit, numbered for splitting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitFile {
    pub path: PathBuf,
    pub status: FileStatus,
    /// `Some` when the file can only be assigned whole; `hunks` is then empty.
    pub whole_only: Option<WholeReason>,
    pub hunks: Vec<Hunk>,
}

/// What `GitBackend::split_source` returns: the commit's changes, numbered
/// with `--no-renames` so hunk ids do not depend on `diff.renames`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitSource {
    pub commit: Oid,
    /// `None` for a root commit.
    pub parent: Option<Oid>,
    pub message: String,
    pub files: Vec<SplitFile>,
    /// The effective `commit.gpgSign` (repository, global and system config
    /// as git resolves it): what the editor's "Sign commits" starts as.
    #[serde(default)]
    pub sign_default: bool,
}

/// One unit assigned to a part. In JSON a piece is `path` plus an optional
/// `hunk` plus optional `lines` (indices into `Hunk.lines`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawPiece", into = "RawPiece")]
pub enum SplitPiece {
    /// A whole file: the only option for whole-only files; on a text file
    /// it means every hunk of it.
    File { path: PathBuf },
    /// Every changed line of a hunk.
    Hunk { path: PathBuf, hunk: HunkId },
    /// Some changed lines of a hunk (indices into `Hunk.lines`).
    Lines {
        path: PathBuf,
        hunk: HunkId,
        lines: Vec<u32>,
    },
}

#[derive(Serialize, Deserialize)]
struct RawPiece {
    path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hunk: Option<HunkId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lines: Option<Vec<u32>>,
}

impl TryFrom<RawPiece> for SplitPiece {
    type Error = String;
    fn try_from(raw: RawPiece) -> Result<Self, String> {
        match (raw.hunk, raw.lines) {
            (None, None) => Ok(Self::File { path: raw.path }),
            (Some(hunk), None) => Ok(Self::Hunk {
                path: raw.path,
                hunk,
            }),
            (Some(hunk), Some(lines)) => Ok(Self::Lines {
                path: raw.path,
                hunk,
                lines,
            }),
            (None, Some(_)) => Err("a piece with `lines` needs a `hunk`".into()),
        }
    }
}

impl From<SplitPiece> for RawPiece {
    fn from(piece: SplitPiece) -> Self {
        match piece {
            SplitPiece::File { path } => Self {
                path,
                hunk: None,
                lines: None,
            },
            SplitPiece::Hunk { path, hunk } => Self {
                path,
                hunk: Some(hunk),
                lines: None,
            },
            SplitPiece::Lines { path, hunk, lines } => Self {
                path,
                hunk: Some(hunk),
                lines: Some(lines),
            },
        }
    }
}

/// One resulting commit, oldest first in `SplitCommit::parts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitPart {
    pub message: String,
    pub pieces: Vec<SplitPiece>,
}

/// The request of `GitBackend::split_commit`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitCommit {
    pub commit: Oid,
    pub parts: Vec<SplitPart>,
    /// Sign the new commits (and the replayed descendants).
    #[serde(default)]
    pub sign: bool,
}

/// What `GitBackend::split_commit` did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitOutcome {
    /// The new commits, oldest first (the last carries the original tree).
    pub created: Vec<Oid>,
    /// HEAD before the rewrite.
    pub previous_head: Oid,
    /// HEAD after it.
    pub new_head: Oid,
    /// How many descendants were re-created on top.
    pub replayed: usize,
}

/// Why `GitBackend::split_commit` (or [`validate_split`]) refused, before
/// changing anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitRefusal {
    /// Not HEAD nor an ancestor of HEAD.
    NotOnCurrentBranch,
    MergeCommit,
    EmptyCommit,
    /// A merge, rebase, cherry-pick, revert or bisect is in progress.
    OperationInProgress,
    TooFewParts,
    EmptyPart,
    EmptyMessage,
    /// A changed line or whole file is in no part.
    Unassigned,
    /// A changed line or whole file is in two parts (or twice in one).
    AssignedTwice,
    /// An unknown path, hunk or line, or a line that is not an add/delete.
    UnknownPiece,
    /// A hunk or line selection on a whole-only file.
    WholeFileOnly,
    SigningFailed,
}

/// One assignable unit: a whole file, or one changed line of a hunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Unit {
    File(PathBuf),
    Line(PathBuf, HunkId, u32),
}

fn is_change(hunk: &Hunk, index: u32) -> bool {
    hunk.lines
        .get(index as usize)
        .is_some_and(|l| l.origin != LineOrigin::Context)
}

/// The units a piece covers, or why it is invalid for `source`.
fn units(source: &SplitSource, piece: &SplitPiece) -> Result<Vec<Unit>, SplitRefusal> {
    let path = match piece {
        SplitPiece::File { path }
        | SplitPiece::Hunk { path, .. }
        | SplitPiece::Lines { path, .. } => path,
    };
    let file = source
        .files
        .iter()
        .find(|f| &f.path == path)
        .ok_or(SplitRefusal::UnknownPiece)?;
    if file.whole_only.is_some() {
        return match piece {
            SplitPiece::File { .. } => Ok(vec![Unit::File(path.clone())]),
            _ => Err(SplitRefusal::WholeFileOnly),
        };
    }
    let (hunk_id, selected) = match piece {
        // A whole text file: every changed line of every hunk.
        SplitPiece::File { .. } => {
            return Ok(file
                .hunks
                .iter()
                .flat_map(|h| {
                    (0..h.lines.len() as u32)
                        .filter(|i| is_change(h, *i))
                        .map(|i| Unit::Line(path.clone(), h.id, i))
                })
                .collect());
        }
        SplitPiece::Hunk { hunk, .. } => (*hunk, None),
        SplitPiece::Lines { hunk, lines, .. } => (*hunk, Some(lines)),
    };
    let hunk = file
        .hunks
        .iter()
        .find(|h| h.id == hunk_id)
        .ok_or(SplitRefusal::UnknownPiece)?;
    let indices: Vec<u32> = match selected {
        None => (0..hunk.lines.len() as u32)
            .filter(|i| is_change(hunk, *i))
            .collect(),
        Some(lines) => {
            if lines.is_empty() || lines.iter().any(|i| !is_change(hunk, *i)) {
                return Err(SplitRefusal::UnknownPiece);
            }
            lines.clone()
        }
    };
    Ok(indices
        .into_iter()
        .map(|i| Unit::Line(path.clone(), hunk_id, i))
        .collect())
}

/// Checks `parts` against `source`, the one rule set behind the backend, the
/// CLI and the TUI. The first refusal in this order wins: part count, empty
/// parts, empty messages, each piece in part order (unknown, whole-only,
/// assigned twice), then anything left unassigned.
pub fn validate_split(source: &SplitSource, parts: &[SplitPart]) -> Result<(), SplitRefusal> {
    if parts.len() < 2 {
        return Err(SplitRefusal::TooFewParts);
    }
    if parts.iter().any(|p| p.pieces.is_empty()) {
        return Err(SplitRefusal::EmptyPart);
    }
    if parts
        .iter()
        .any(|p| normalize_reword_message(&p.message).is_none())
    {
        return Err(SplitRefusal::EmptyMessage);
    }
    let mut seen: HashSet<Unit> = HashSet::new();
    for piece in parts.iter().flat_map(|p| &p.pieces) {
        for unit in units(source, piece)? {
            if !seen.insert(unit) {
                return Err(SplitRefusal::AssignedTwice);
            }
        }
    }
    if unassigned_in(source, &seen).is_empty() {
        Ok(())
    } else {
        Err(SplitRefusal::Unassigned)
    }
}

fn unassigned_in(source: &SplitSource, seen: &HashSet<Unit>) -> Vec<SplitPiece> {
    let mut out = Vec::new();
    for file in &source.files {
        if file.whole_only.is_some() {
            if !seen.contains(&Unit::File(file.path.clone())) {
                out.push(SplitPiece::File {
                    path: file.path.clone(),
                });
            }
            continue;
        }
        for hunk in &file.hunks {
            let changed: Vec<u32> = (0..hunk.lines.len() as u32)
                .filter(|i| is_change(hunk, *i))
                .collect();
            let missing: Vec<u32> = changed
                .iter()
                .copied()
                .filter(|i| !seen.contains(&Unit::Line(file.path.clone(), hunk.id, *i)))
                .collect();
            if missing.is_empty() {
                continue;
            }
            if missing.len() == changed.len() {
                out.push(SplitPiece::Hunk {
                    path: file.path.clone(),
                    hunk: hunk.id,
                });
            } else {
                out.push(SplitPiece::Lines {
                    path: file.path.clone(),
                    hunk: hunk.id,
                    lines: missing,
                });
            }
        }
    }
    out
}

/// What `parts` leave unassigned, in file and hunk order: a whole file, a
/// hunk nothing touches, or the remaining lines of a partly assigned hunk.
/// Invalid pieces are ignored here ([`validate_split`] reports them). Used
/// by the CLI's `--rest-message` and the TUI counter.
pub fn unassigned(source: &SplitSource, parts: &[SplitPart]) -> Vec<SplitPiece> {
    let seen: HashSet<Unit> = parts
        .iter()
        .flat_map(|p| &p.pieces)
        .filter_map(|piece| units(source, piece).ok())
        .flatten()
        .collect();
    unassigned_in(source, &seen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DiffLine;

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn line(origin: LineOrigin) -> DiffLine {
        DiffLine {
            origin,
            content: "x\n".into(),
            staged: false,
        }
    }

    fn hunk(id: u32) -> Hunk {
        use LineOrigin::{Add, Context, Delete};
        Hunk {
            id: HunkId(id),
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 4,
            // indices: 0 ctx, 1 add, 2 add, 3 delete, 4 ctx
            lines: vec![
                line(Context),
                line(Add),
                line(Add),
                line(Delete),
                line(Context),
            ],
        }
    }

    fn source() -> SplitSource {
        SplitSource {
            commit: oid(1),
            parent: Some(oid(2)),
            message: "feat: both".into(),
            sign_default: false,
            files: vec![
                SplitFile {
                    path: "src/a.rs".into(),
                    status: FileStatus::Modified,
                    whole_only: None,
                    hunks: vec![hunk(0), hunk(1)],
                },
                SplitFile {
                    path: "logo.png".into(),
                    status: FileStatus::Added,
                    whole_only: Some(WholeReason::Binary),
                    hunks: vec![],
                },
            ],
        }
    }

    fn file(p: &str) -> SplitPiece {
        SplitPiece::File { path: p.into() }
    }
    fn hk(p: &str, h: u32) -> SplitPiece {
        SplitPiece::Hunk {
            path: p.into(),
            hunk: HunkId(h),
        }
    }
    fn lines(p: &str, h: u32, l: &[u32]) -> SplitPiece {
        SplitPiece::Lines {
            path: p.into(),
            hunk: HunkId(h),
            lines: l.to_vec(),
        }
    }
    fn part(msg: &str, pieces: Vec<SplitPiece>) -> SplitPart {
        SplitPart {
            message: msg.into(),
            pieces,
        }
    }

    #[test]
    fn hunks_and_whole_files_validate() {
        let parts = [
            part("one", vec![hk("src/a.rs", 0), file("logo.png")]),
            part("two", vec![hk("src/a.rs", 1)]),
        ];
        assert_eq!(validate_split(&source(), &parts), Ok(()));
        assert!(unassigned(&source(), &parts).is_empty());
    }

    #[test]
    fn a_whole_text_file_piece_means_every_hunk() {
        let parts = [
            part("one", vec![file("src/a.rs")]),
            part("two", vec![file("logo.png")]),
        ];
        assert_eq!(validate_split(&source(), &parts), Ok(()));
        assert!(unassigned(&source(), &parts).is_empty());
        // Mixed with a hunk of the same file, in any part: twice.
        let twice = [
            part("one", vec![file("src/a.rs"), file("logo.png")]),
            part("two", vec![hk("src/a.rs", 1)]),
        ];
        assert_eq!(
            validate_split(&source(), &twice),
            Err(SplitRefusal::AssignedTwice)
        );
        // Untouched text files are still reported per hunk.
        let only_logo = [part("one", vec![file("logo.png")])];
        assert_eq!(
            unassigned(&source(), &only_logo),
            vec![hk("src/a.rs", 0), hk("src/a.rs", 1)]
        );
    }

    #[test]
    fn a_hunk_split_by_lines_across_three_parts_validates() {
        let parts = [
            part("one", vec![lines("src/a.rs", 0, &[1]), file("logo.png")]),
            part("two", vec![lines("src/a.rs", 0, &[2, 3])]),
            part("three", vec![hk("src/a.rs", 1)]),
        ];
        assert_eq!(validate_split(&source(), &parts), Ok(()));
    }

    #[test]
    fn refusals_in_order() {
        let s = source();
        let ok_a = part("a", vec![hk("src/a.rs", 0), file("logo.png")]);
        let ok_b = part("b", vec![hk("src/a.rs", 1)]);
        let with_a = |second: SplitPart| validate_split(&s, &[ok_a.clone(), second]);

        assert_eq!(
            validate_split(&s, std::slice::from_ref(&ok_a)),
            Err(SplitRefusal::TooFewParts)
        );
        assert_eq!(validate_split(&s, &[]), Err(SplitRefusal::TooFewParts));
        assert_eq!(with_a(part("b", vec![])), Err(SplitRefusal::EmptyPart));
        assert_eq!(
            with_a(part("  \n ", vec![hk("src/a.rs", 1)])),
            Err(SplitRefusal::EmptyMessage)
        );
        assert_eq!(
            with_a(part("b", vec![hk("src/a.rs", 1), hk("src/a.rs", 0)])),
            Err(SplitRefusal::AssignedTwice)
        );
        assert_eq!(
            with_a(part(
                "b",
                vec![hk("src/a.rs", 1), lines("src/a.rs", 0, &[2])]
            )),
            Err(SplitRefusal::AssignedTwice)
        );
        assert_eq!(
            with_a(part("b", vec![lines("src/a.rs", 1, &[1, 1])])),
            Err(SplitRefusal::AssignedTwice)
        );
        assert_eq!(
            with_a(part("b", vec![hk("src/other.rs", 0)])),
            Err(SplitRefusal::UnknownPiece)
        );
        assert_eq!(
            with_a(part("b", vec![hk("src/a.rs", 9)])),
            Err(SplitRefusal::UnknownPiece)
        );
        // out of range, a context line, and an empty selection
        for bad in [&[7][..], &[0], &[]] {
            assert_eq!(
                with_a(part("b", vec![lines("src/a.rs", 1, bad)])),
                Err(SplitRefusal::UnknownPiece)
            );
        }
        // a whole-file piece on a text file overlaps any hunk of it
        assert_eq!(
            with_a(part("b", vec![file("src/a.rs")])),
            Err(SplitRefusal::AssignedTwice)
        );
        // selection on a whole-only file
        assert_eq!(
            validate_split(&s, &[part("a", vec![hk("logo.png", 0)]), ok_b.clone()]),
            Err(SplitRefusal::WholeFileOnly)
        );
        assert_eq!(
            with_a(part("b", vec![lines("logo.png", 0, &[1])])),
            Err(SplitRefusal::WholeFileOnly)
        );
        // unassigned: part of hunk 1 left out, or the whole file
        assert_eq!(
            with_a(part("b", vec![lines("src/a.rs", 1, &[1])])),
            Err(SplitRefusal::Unassigned)
        );
        assert_eq!(
            validate_split(
                &s,
                &[
                    part("a", vec![hk("src/a.rs", 0)]),
                    part("b", vec![hk("src/a.rs", 1)])
                ]
            ),
            Err(SplitRefusal::Unassigned)
        );
        assert_eq!(validate_split(&s, &[ok_a, ok_b]), Ok(()));
    }

    #[test]
    fn unassigned_lists_files_hunks_and_remaining_lines_in_order() {
        let parts = [
            part("a", vec![lines("src/a.rs", 0, &[1])]),
            part("b", vec![lines("src/a.rs", 1, &[2])]),
        ];
        assert_eq!(
            unassigned(&source(), &parts),
            vec![
                lines("src/a.rs", 0, &[2, 3]),
                lines("src/a.rs", 1, &[1, 3]),
                file("logo.png"),
            ]
        );
        assert_eq!(
            unassigned(&source(), &[]),
            vec![hk("src/a.rs", 0), hk("src/a.rs", 1), file("logo.png")]
        );
    }

    #[test]
    fn unassigned_ignores_invalid_pieces() {
        let parts = [part("a", vec![hk("nope", 0), hk("src/a.rs", 0)])];
        assert_eq!(
            unassigned(&source(), &parts),
            vec![hk("src/a.rs", 1), file("logo.png")]
        );
    }

    #[test]
    fn pieces_round_trip_as_path_hunk_lines_json() {
        let json = r#"[{"path":"a"},{"path":"b","hunk":1},{"path":"c","hunk":2,"lines":[3,4]}]"#;
        let pieces: Vec<SplitPiece> = serde_json::from_str(json).unwrap();
        assert_eq!(pieces, vec![file("a"), hk("b", 1), lines("c", 2, &[3, 4])]);
        assert_eq!(serde_json::to_string(&pieces).unwrap(), json);
        assert!(serde_json::from_str::<SplitPiece>(r#"{"path":"a","lines":[1]}"#).is_err());
    }

    #[test]
    fn a_request_defaults_to_unsigned() {
        let req: SplitCommit = serde_json::from_value(serde_json::json!({
            "commit": serde_json::to_value(oid(3)).unwrap(),
            "parts": [],
        }))
        .unwrap();
        assert!(!req.sign);
    }
}
