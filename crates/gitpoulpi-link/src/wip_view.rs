//! Blame for a file as it is in the WIP (working tree or index).
//!
//! The backend blames committed revisions only. For the WIP the committed
//! blame at HEAD is matched against the WIP's own lines: a line that is
//! the same in both keeps its HEAD attribution, every other line is
//! "Not committed yet" -- git's own wording and its zero-oid pseudo
//! commit, which is how the rest of the UI recognises it.
//!
//! Pure on purpose, so the matching is unit-tested without a daemon.

use std::path::PathBuf;

use gitpoulpi_core::domain::{
    BlameLine, DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin, Oid,
};

/// The author column of an uncommitted line.
pub const NOT_COMMITTED: gitpoulpi_i18n::M = gitpoulpi_i18n::msg::wip::NOT_COMMITTED;

/// `true` for the pseudo commit uncommitted lines are attributed to.
pub fn is_uncommitted(commit: Oid) -> bool {
    commit == Oid([0u8; 20])
}

/// Past this many cells the middle of the diff is not worth a quadratic
/// table: everything in it is reported as uncommitted rather than risking
/// a long stall (or a huge allocation) on a generated file.
const MAX_CELLS: usize = 4_000_000;

fn uncommitted(final_line: u32, content: &str) -> BlameLine {
    BlameLine {
        commit: Oid([0u8; 20]),
        author_name: NOT_COMMITTED.get().to_string(),
        author_email: String::new(),
        timestamp: 0,
        summary: String::new(),
        final_line,
        orig_line: final_line,
        committer_time: 0,
        orig_path: Default::default(),
        previous: None,
        content: content.to_string(),
    }
}

/// Attributes `wip` (the WIP file's lines): `head` is HEAD's blame of the
/// same path, or `None` when HEAD has no such file (new or untracked).
pub fn attribute(head: Option<&[BlameLine]>, wip: &[String]) -> Vec<BlameLine> {
    let head = head.unwrap_or(&[]);
    let mut out: Vec<Option<&BlameLine>> = vec![None; wip.len()];

    // Common prefix and suffix first: most edits touch a small middle.
    let max_common = head.len().min(wip.len());
    let mut prefix = 0;
    while prefix < max_common && head[prefix].content == wip[prefix] {
        out[prefix] = Some(&head[prefix]);
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < max_common - prefix
        && head[head.len() - 1 - suffix].content == wip[wip.len() - 1 - suffix]
    {
        out[wip.len() - 1 - suffix] = Some(&head[head.len() - 1 - suffix]);
        suffix += 1;
    }

    let h = &head[prefix..head.len() - suffix];
    let w = &wip[prefix..wip.len() - suffix];
    if !h.is_empty() && !w.is_empty() && h.len().saturating_mul(w.len()) <= MAX_CELLS {
        // Longest common subsequence over the middle.
        let cols = w.len() + 1;
        let mut table = vec![0u32; (h.len() + 1) * cols];
        for i in (0..h.len()).rev() {
            for j in (0..w.len()).rev() {
                table[i * cols + j] = if h[i].content == w[j] {
                    table[(i + 1) * cols + j + 1] + 1
                } else {
                    table[(i + 1) * cols + j].max(table[i * cols + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < h.len() && j < w.len() {
            if h[i].content == w[j] {
                out[prefix + j] = Some(&h[i]);
                i += 1;
                j += 1;
            } else if table[(i + 1) * cols + j] >= table[i * cols + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }

    out.into_iter()
        .enumerate()
        .map(|(idx, owner)| {
            let final_line = idx as u32 + 1;
            match owner {
                Some(line) => BlameLine {
                    final_line,
                    ..line.clone()
                },
                None => uncommitted(final_line, &wip[idx]),
            }
        })
        .collect()
}

/// The Unstaged diff of an untracked file: every line added. The
/// backend's working-tree diff does not list untracked files, so the Diff
/// view would otherwise say "nothing left to show" for a file the
/// Unstaged list shows. An empty file has no hunk, like any empty add.
pub fn all_added_diff(path: PathBuf, lines: &[String]) -> FileDiff {
    let hunks = if lines.is_empty() {
        Vec::new()
    } else {
        vec![Hunk {
            id: HunkId(0),
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: lines.len() as u32,
            lines: lines
                .iter()
                .map(|content| DiffLine {
                    origin: LineOrigin::Add,
                    content: content.clone(),
                    staged: false,
                })
                .collect(),
        }]
    };
    FileDiff {
        renamed_from: None,
        similarity: None,
        path,
        status: FileStatus::Added,
        hunks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untracked_file_is_one_all_added_hunk() {
        let diff = all_added_diff(PathBuf::from("TODO.md"), &lines(&["a", "b"]));
        assert_eq!(diff.hunks.len(), 1);
        let hunk = &diff.hunks[0];
        assert_eq!((hunk.old_lines, hunk.new_start, hunk.new_lines), (0, 1, 2));
        assert!(hunk.lines.iter().all(|l| l.origin == LineOrigin::Add));
        assert_eq!(hunk.lines[1].content, "b");
        assert!(all_added_diff(PathBuf::from("e"), &[]).hunks.is_empty());
    }

    fn head_line(n: u32, content: &str) -> BlameLine {
        BlameLine {
            commit: Oid([n as u8; 20]),
            author_name: "Ada".into(),
            author_email: "ada@example.com".into(),
            timestamp: 1_700_000_000,
            summary: "a commit".into(),
            final_line: n,
            orig_line: n,
            committer_time: 0,
            orig_path: Default::default(),
            previous: None,
            content: content.into(),
        }
    }

    fn lines(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_new_file_is_entirely_uncommitted() {
        let blame = attribute(None, &lines(&["a", "b"]));
        assert_eq!(blame.len(), 2);
        assert!(blame.iter().all(|l| is_uncommitted(l.commit)));
        assert_eq!(blame[1].final_line, 2);
        assert_eq!(blame[1].content, "b");
        assert_eq!(blame[0].author_name, NOT_COMMITTED.get());
    }

    #[test]
    fn unchanged_lines_keep_their_commit_and_edited_ones_do_not() {
        let head = vec![
            head_line(1, "one"),
            head_line(2, "two"),
            head_line(3, "three"),
        ];
        let blame = attribute(Some(&head), &lines(&["one", "TWO", "three"]));
        assert_eq!(blame[0].commit, Oid([1; 20]));
        assert!(is_uncommitted(blame[1].commit));
        assert_eq!(blame[1].content, "TWO");
        assert_eq!(blame[2].commit, Oid([3; 20]));
    }

    #[test]
    fn an_inserted_line_shifts_the_rest_without_losing_their_commits() {
        let head = vec![head_line(1, "a"), head_line(2, "b"), head_line(3, "c")];
        let blame = attribute(Some(&head), &lines(&["a", "x", "b", "y", "c"]));
        let owners: Vec<bool> = blame.iter().map(|l| is_uncommitted(l.commit)).collect();
        assert_eq!(owners, [false, true, false, true, false]);
        assert_eq!(blame[2].commit, Oid([2; 20]));
        assert_eq!(blame[2].final_line, 3, "numbered by its place in the WIP");
        assert_eq!(blame[2].orig_line, 2, "orig line stays HEAD's");
    }

    #[test]
    fn a_wholly_rewritten_file_is_uncommitted() {
        let head = vec![head_line(1, "a")];
        let blame = attribute(Some(&head), &lines(&["z"]));
        assert!(is_uncommitted(blame[0].commit));
    }
}
