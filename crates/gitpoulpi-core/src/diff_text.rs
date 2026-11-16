//! Unified-diff text to [`FileDiff`]s, with no I/O: the one parser behind
//! the system-git backend (`git diff` output) and the hosting API diffs (a
//! host's per-file `patch`, which has hunks but no `diff --git` header).
//!
//! Lines are split with [`str::lines`], so a `\r\n` terminator is dropped
//! like a `\n`: a CRLF patch parses as the same lines as an LF one. A
//! `\ No newline at end of file` marker is not a line of the file and is
//! skipped.

use std::iter::Peekable;
use std::path::PathBuf;
use std::str::Lines;

use crate::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};

/// Parses a whole `git diff` output: one [`FileDiff`] per `diff --git`
/// header, its status read from the extended header lines (`new file mode`,
/// `deleted file mode`, `rename from|to`).
pub fn parse_unified(raw: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        let Some(header) = line.strip_prefix("diff --git ") else {
            continue;
        };
        let path = extract_path(header);
        let mut status = FileStatus::Modified;
        let mut renamed_from: Option<PathBuf> = None;
        let mut similarity: Option<u8> = None;
        let mut renamed_to: Option<PathBuf> = None;

        // Consume everything up to (and including) the first hunk header
        // or the next file's diff header, tracking status markers.
        while let Some(&peeked) = lines.peek() {
            if peeked.starts_with("diff --git ") || peeked.starts_with("@@ ") {
                break;
            }
            let l = lines.next().unwrap();
            if l.starts_with("new file mode") {
                status = FileStatus::Added;
            } else if l.starts_with("deleted file mode") {
                status = FileStatus::Deleted;
            } else if let Some(from) = l
                .strip_prefix("rename from ")
                .or_else(|| l.strip_prefix("copy from "))
            {
                status = FileStatus::Renamed;
                renamed_from = Some(PathBuf::from(from));
            } else if let Some(to) = l
                .strip_prefix("rename to ")
                .or_else(|| l.strip_prefix("copy to "))
            {
                status = FileStatus::Renamed;
                renamed_to = Some(PathBuf::from(to));
            } else if let Some(pct) = l.strip_prefix("similarity index ") {
                similarity = pct.trim_end_matches('%').parse().ok();
            }
        }

        let hunks = read_hunks(&mut lines);
        files.push(FileDiff {
            path: renamed_to.unwrap_or(path),
            status,
            hunks,
            renamed_from,
            similarity,
        });
    }

    files
}

/// Parses the hunks of one file's `patch` (what a hosting API returns per
/// file: `@@ -a,b +c,d @@` sections, no `diff --git` / `---` / `+++`
/// header) into a [`FileDiff`] for `path` with the given `status`. Anything
/// before the first `@@` line is skipped, so a patch that does carry
/// `---`/`+++` lines parses the same. An empty patch (a binary file, a
/// mode change, a rename without edits) has no hunks.
pub fn parse_hunks(path: impl Into<PathBuf>, status: FileStatus, patch: &str) -> FileDiff {
    let mut lines = patch.lines().peekable();
    while lines.next_if(|l| !l.starts_with("@@ ")).is_some() {}
    FileDiff {
        renamed_from: None,
        similarity: None,
        path: path.into(),
        status,
        hunks: read_hunks(&mut lines),
    }
}

/// Reads consecutive hunks (`@@ ` header plus body) from `lines`, stopping
/// at the next file header or when the input ends.
fn read_hunks(lines: &mut Peekable<Lines<'_>>) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    let mut next_hunk_id = 0u32;
    while let Some(&peeked) = lines.peek() {
        if !peeked.starts_with("@@ ") {
            break;
        }
        let hunk_header = lines.next().unwrap();
        let Some((old_start, old_lines, new_start, new_lines)) = parse_hunk_header(hunk_header)
        else {
            continue;
        };

        let mut hunk_lines = Vec::new();
        while let Some(&peeked) = lines.peek() {
            if peeked.starts_with("@@ ") || peeked.starts_with("diff --git ") {
                break;
            }
            let l = lines.next().unwrap();
            if let Some(content) = l.strip_prefix('+') {
                hunk_lines.push(DiffLine {
                    origin: LineOrigin::Add,
                    content: content.to_string(),
                    staged: false,
                });
            } else if let Some(content) = l.strip_prefix('-') {
                hunk_lines.push(DiffLine {
                    origin: LineOrigin::Delete,
                    content: content.to_string(),
                    staged: false,
                });
            } else if let Some(content) = l.strip_prefix(' ') {
                hunk_lines.push(DiffLine {
                    origin: LineOrigin::Context,
                    content: content.to_string(),
                    staged: false,
                });
            }
            // lines like "\ No newline at end of file" are ignored.
        }

        hunks.push(Hunk {
            id: HunkId(next_hunk_id),
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines: hunk_lines,
        });
        next_hunk_id += 1;
    }
    hunks
}

/// `diff --git a/<path> b/<path>` - paths can contain spaces, so this
/// trusts the `a/`/`b/` markers rather than splitting on whitespace.
fn extract_path(header: &str) -> PathBuf {
    if let Some(b_idx) = header.rfind(" b/") {
        return PathBuf::from(&header[b_idx + 3..]);
    }
    PathBuf::from(header)
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let body = line.strip_prefix("@@ -")?;
    let (ranges, _) = body.split_once(" @@")?;
    let (old_range, new_range) = ranges.split_once(" +")?;
    let (old_start, old_lines) = parse_range(old_range);
    let (new_start, new_lines) = parse_range(new_range);
    Some((old_start, old_lines, new_start, new_lines))
}

fn parse_range(range: &str) -> (u32, u32) {
    match range.split_once(',') {
        Some((start, len)) => (start.parse().unwrap_or(0), len.parse().unwrap_or(0)),
        None => (range.parse().unwrap_or(0), 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contents(file: &FileDiff) -> Vec<(LineOrigin, &str)> {
        file.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .map(|l| (l.origin.clone(), l.content.as_str()))
            .collect()
    }

    #[test]
    fn a_git_diff_gives_one_file_per_header_with_its_status() {
        let raw = "diff --git a/old name.txt b/new name.txt\nsimilarity index 90%\nrename from old name.txt\nrename to new name.txt\n@@ -1,2 +1,2 @@\n keep\n-a\n+b\ndiff --git a/n.txt b/n.txt\nnew file mode 100644\n--- /dev/null\n+++ b/n.txt\n@@ -0,0 +1 @@\n+x\n";
        let files = parse_unified(raw);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, PathBuf::from("new name.txt"));
        assert_eq!(files[0].status, FileStatus::Renamed);
        assert_eq!(files[1].status, FileStatus::Added);
        assert_eq!(
            contents(&files[0]),
            vec![
                (LineOrigin::Context, "keep"),
                (LineOrigin::Delete, "a"),
                (LineOrigin::Add, "b")
            ]
        );
        let h = &files[0].hunks[0];
        assert_eq!(
            (h.old_start, h.old_lines, h.new_start, h.new_lines),
            (1, 2, 1, 2)
        );
        // A one-line range has no `,len`: it is 1.
        let h = &files[1].hunks[0];
        assert_eq!(
            (h.old_start, h.old_lines, h.new_start, h.new_lines),
            (0, 0, 1, 1)
        );
    }

    #[test]
    fn the_no_newline_marker_is_not_a_line() {
        let patch =
            "@@ -1 +1 @@\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n";
        let file = parse_hunks("a.txt", FileStatus::Modified, patch);
        assert_eq!(
            contents(&file),
            vec![(LineOrigin::Delete, "old"), (LineOrigin::Add, "new")]
        );
    }

    #[test]
    fn a_crlf_patch_parses_like_an_lf_one() {
        let lf = "@@ -1,2 +1,2 @@\n a\n-b\n+c\n";
        let crlf = lf.replace('\n', "\r\n");
        assert_eq!(
            parse_hunks("a", FileStatus::Modified, &crlf),
            parse_hunks("a", FileStatus::Modified, lf)
        );
    }

    #[test]
    fn a_host_patch_without_headers_has_its_hunks_numbered() {
        let patch = "@@ -1,2 +1,3 @@\n a\n+b\n c\n@@ -10 +11,2 @@\n-x\n+y\n+z";
        let file = parse_hunks("src/x.rs", FileStatus::Modified, patch);
        assert_eq!(file.path, PathBuf::from("src/x.rs"));
        assert_eq!(file.hunks.len(), 2);
        assert_eq!(file.hunks[1].id, HunkId(1));
        assert_eq!(file.hunks[1].old_start, 10);
        assert_eq!(file.hunks[1].new_lines, 2);
        assert_eq!(file.hunks[1].lines.len(), 3);
    }

    #[test]
    fn leading_header_lines_and_empty_patches_are_tolerated() {
        let with_headers = "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n";
        assert_eq!(
            parse_hunks("x", FileStatus::Modified, with_headers)
                .hunks
                .len(),
            1
        );
        assert!(parse_hunks("img.png", FileStatus::Modified, "")
            .hunks
            .is_empty());
    }
}
