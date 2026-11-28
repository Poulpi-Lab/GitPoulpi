//! Rows of the changed-files lists: flat (Path) or folded into folders (Tree).

use std::collections::HashSet;

use gitpoulpi_core::domain::{FileDiff, FileStatus, LineOrigin};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub status: FileStatus,
    pub adds: usize,
    pub dels: usize,
    /// Tree mode: nesting depth.
    pub depth: usize,
    pub folder: bool,
    /// The old name of a moved file (the row stands for both names).
    pub renamed_from: Option<String>,
    /// Index into the diff set (commit mode) or status list (working mode); `usize::MAX` for folders.
    pub ix: usize,
}

impl FileRow {
    pub fn file(path: String, status: FileStatus, adds: usize, dels: usize, ix: usize) -> Self {
        Self {
            path,
            status,
            adds,
            dels,
            depth: 0,
            folder: false,
            renamed_from: None,
            ix,
        }
    }

    /// The label drawn: the last path segment in Tree mode, the whole path otherwise.
    pub fn label(&self) -> String {
        if self.folder || self.depth > 0 {
            let name = self
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&self.path)
                .to_string();
            if self.folder {
                name + "/"
            } else {
                name
            }
        } else if let Some(old) = &self.renamed_from {
            format!("{} <- {old}", self.path)
        } else {
            self.path.clone()
        }
    }
}

/// Added and removed line counts of one file.
pub fn count_lines(f: &FileDiff) -> (usize, usize) {
    let (mut adds, mut dels) = (0, 0);
    for h in &f.hunks {
        for l in &h.lines {
            match l.origin {
                LineOrigin::Add => adds += 1,
                LineOrigin::Delete => dels += 1,
                LineOrigin::Context => {}
            }
        }
    }
    (adds, dels)
}

/// Colour class of a status: 0 success, 1 warning, 2 danger, 3 info.
pub fn status_glyph(s: FileStatus) -> (&'static str, u8) {
    match s {
        FileStatus::Added | FileStatus::Untracked | FileStatus::Staged => ("+", 0),
        FileStatus::Deleted => ("\u{2212}", 2),
        FileStatus::Renamed => ("R", 3),
        FileStatus::Conflicted => ("!", 2),
        FileStatus::Modified => ("\u{270E}", 1),
    }
}

/// Inserts a folder row before the first file under it.
pub fn tree_rows(rows: Vec<FileRow>) -> Vec<FileRow> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for r in rows {
        let parts: Vec<&str> = r.path.split('/').collect();
        let mut prefix = String::new();
        for (depth, part) in parts.iter().enumerate().take(parts.len().saturating_sub(1)) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            if seen.insert(prefix.clone()) {
                out.push(FileRow {
                    path: prefix.clone(),
                    status: FileStatus::Modified,
                    adds: 0,
                    dels: 0,
                    depth,
                    folder: true,
                    renamed_from: None,
                    ix: usize::MAX,
                });
            }
        }
        let depth = parts.len().saturating_sub(1);
        out.push(FileRow { depth, ..r });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(path: &str, ix: usize) -> FileRow {
        FileRow::file(path.into(), FileStatus::Modified, 1, 0, ix)
    }

    #[test]
    fn tree_inserts_each_folder_once_before_its_files() {
        let rows = tree_rows(vec![
            row("a/b/c.rs", 0),
            row("a/b/d.rs", 1),
            row("a/e.rs", 2),
            row("top.rs", 3),
        ]);
        let shape: Vec<(String, usize, bool)> = rows
            .iter()
            .map(|r| (r.path.clone(), r.depth, r.folder))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("a".into(), 0, true),
                ("a/b".into(), 1, true),
                ("a/b/c.rs".into(), 2, false),
                ("a/b/d.rs".into(), 2, false),
                ("a/e.rs".into(), 1, false),
                ("top.rs".into(), 0, false),
            ]
        );
    }

    #[test]
    fn labels_are_the_last_segment_in_tree_mode() {
        let rows = tree_rows(vec![row("src/lib.rs", 0)]);
        assert_eq!(rows[0].label(), "src/");
        assert_eq!(rows[1].label(), "lib.rs");
        assert_eq!(row("src/lib.rs", 0).label(), "src/lib.rs");
    }
}
