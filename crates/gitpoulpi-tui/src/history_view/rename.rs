//! A rename as one diff.
//!
//! The backends never detect renames (`--no-renames`: a rename is a delete
//! plus an add, see `gitpoulpi-cli-backend`'s `diff.rs`), so a diff asked
//! for a renamed file's two paths comes back as two files. The History view
//! wants the rename as git shows it with `-M`: one file, the lines that
//! changed while it moved. The two halves carry the whole of both versions,
//! so the merge needs no extra read.

use std::ops::Range;
use std::path::Path;

use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};
use imara_diff::{Algorithm, Diff, InternedInput};

/// Context lines around each change, as git's own diff.
const CONTEXT: usize = 3;

/// Replaces the deleted `from` and the added `to` of `files` by one
/// `Renamed` file at `to`. Anything else (a binary file has no hunks to
/// merge, a missing half) is returned untouched.
pub(crate) fn merge_rename(mut files: Vec<FileDiff>, from: &Path, to: &Path) -> Vec<FileDiff> {
    if from == to {
        return files;
    }
    let old = files
        .iter()
        .position(|f| f.path == from && f.status == FileStatus::Deleted);
    let new = files
        .iter()
        .position(|f| f.path == to && f.status == FileStatus::Added);
    let (Some(old), Some(new)) = (old, new) else {
        return files;
    };
    let old_lines = side(&files[old], LineOrigin::Delete);
    let new_lines = side(&files[new], LineOrigin::Add);
    if old_lines.is_empty() && new_lines.is_empty() {
        return files;
    }
    files[new] = FileDiff {
        renamed_from: None,
        similarity: None,
        path: to.to_path_buf(),
        status: FileStatus::Renamed,
        hunks: hunks(&old_lines, &new_lines),
    };
    files.remove(old);
    files
}

/// One version's lines, from the diff half that carries all of them.
fn side(file: &FileDiff, origin: LineOrigin) -> Vec<String> {
    file.hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .filter(|l| l.origin == origin)
        .map(|l| l.content.clone())
        .collect()
}

fn joined(lines: &[String]) -> String {
    let mut out = String::new();
    for l in lines {
        out.push_str(l);
        out.push('\n');
    }
    out
}

fn line(origin: LineOrigin, content: &str) -> DiffLine {
    DiffLine {
        origin,
        content: content.to_string(),
        staged: false,
    }
}

/// The unified hunks turning `old` into `new`.
pub(crate) fn hunks(old: &[String], new: &[String]) -> Vec<Hunk> {
    let (before, after) = (joined(old), joined(new));
    let input = InternedInput::new(before.as_str(), after.as_str());
    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);
    let changes: Vec<(Range<usize>, Range<usize>)> = diff
        .hunks()
        .map(|h| {
            (
                h.before.start as usize..h.before.end as usize,
                h.after.start as usize..h.after.end as usize,
            )
        })
        .collect();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (i, (b, _)) in changes.iter().enumerate() {
        match groups.last_mut() {
            Some(g) if b.start - changes[g[g.len() - 1]].0.end <= 2 * CONTEXT => g.push(i),
            _ => groups.push(vec![i]),
        }
    }
    groups
        .iter()
        .enumerate()
        .map(|(n, g)| {
            let first = &changes[g[0]];
            let last = &changes[g[g.len() - 1]];
            let pre = CONTEXT.min(first.0.start);
            let post = CONTEXT.min(old.len() - last.0.end);
            let mut lines = Vec::new();
            let context = |lines: &mut Vec<DiffLine>, range: Range<usize>| {
                for l in &old[range] {
                    lines.push(line(LineOrigin::Context, l));
                }
            };
            context(&mut lines, first.0.start - pre..first.0.start);
            for (k, idx) in g.iter().enumerate() {
                let (b, a) = &changes[*idx];
                if k > 0 {
                    context(&mut lines, changes[g[k - 1]].0.end..b.start);
                }
                for l in &old[b.clone()] {
                    lines.push(line(LineOrigin::Delete, l));
                }
                for l in &new[a.clone()] {
                    lines.push(line(LineOrigin::Add, l));
                }
            }
            context(&mut lines, last.0.end..last.0.end + post);
            let (old_start, new_start) = (first.0.start - pre, first.1.start - pre);
            let (old_end, new_end) = (last.0.end + post, last.1.end + post);
            Hunk {
                id: HunkId(n as u32),
                old_start: old_start as u32 + 1,
                old_lines: (old_end - old_start) as u32,
                new_start: new_start as u32 + 1,
                new_lines: (new_end - new_start) as u32,
                lines,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    fn half(path: &str, status: FileStatus, origin: LineOrigin, body: &[String]) -> FileDiff {
        FileDiff {
            renamed_from: None,
            similarity: None,
            path: PathBuf::from(path),
            status,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 0,
                old_lines: 0,
                new_start: 0,
                new_lines: 0,
                lines: body.iter().map(|c| line(origin.clone(), c)).collect(),
            }],
        }
    }

    #[test]
    fn a_delete_and_an_add_become_one_renamed_file_with_only_the_changed_line() {
        let old = lines("a\nb\nc\nd\ne\nf\ng\nh");
        let mut new = old.clone();
        new[3] = "D".into();
        let files = vec![
            half("new.txt", FileStatus::Added, LineOrigin::Add, &new),
            half("old.txt", FileStatus::Deleted, LineOrigin::Delete, &old),
        ];
        let out = merge_rename(files, Path::new("old.txt"), Path::new("new.txt"));
        assert_eq!(out.len(), 1, "{out:?}");
        assert_eq!(out[0].status, FileStatus::Renamed);
        assert_eq!(out[0].path, PathBuf::from("new.txt"));
        let changed: Vec<(&LineOrigin, &str)> = out[0].hunks[0]
            .lines
            .iter()
            .filter(|l| l.origin != LineOrigin::Context)
            .map(|l| (&l.origin, l.content.as_str()))
            .collect();
        assert_eq!(
            changed,
            [(&LineOrigin::Delete, "d"), (&LineOrigin::Add, "D")]
        );
        assert_eq!(out[0].hunks[0].lines.len(), 8, "3 + 1 + 1 + 3");
    }

    #[test]
    fn a_pure_rename_has_no_hunks_and_other_files_are_left_alone() {
        let body = lines("x\ny");
        let files = vec![
            half("n", FileStatus::Added, LineOrigin::Add, &body),
            half("o", FileStatus::Deleted, LineOrigin::Delete, &body),
        ];
        let out = merge_rename(files, Path::new("o"), Path::new("n"));
        assert!(out[0].hunks.is_empty() && out[0].status == FileStatus::Renamed);
        let lone = vec![half("n", FileStatus::Added, LineOrigin::Add, &body)];
        assert_eq!(
            merge_rename(lone.clone(), Path::new("o"), Path::new("n")),
            lone
        );
    }
}
