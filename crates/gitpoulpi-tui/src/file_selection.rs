//! Multi-select in the right pane's file lists (Unstaged, Staged, the
//! commit's files, View all files), with the same gestures as the Log
//! (`log_multiselect`): Ctrl+click or Space toggles a row, Shift+click (Alt+click
//! for terminals that keep Shift) or Shift+Up/Down takes the range from the
//! anchor, a plain click starts over, Esc clears.
//!
//! The selection is a set of paths kept *next to* each list's own single
//! cursor, never instead of it: the cursor keeps driving the diff, and a plain
//! click resets this set exactly as it does in the Log. A folder row selects
//! the files beneath it (collapsed or not: they come from the whole list, not
//! the rows on screen). One selection at a time, tagged with the list it
//! belongs to, so a click in another list starts over.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Which file list a selection belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileList {
    Unstaged,
    Staged,
    /// The selected commit's changed files (and its View all files tree).
    Commit,
    /// The working tree's View all files list.
    WipAll,
}

/// One row of a list as the selection sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelRow {
    pub(crate) path: PathBuf,
    pub(crate) folder: bool,
}

/// What the selection reads from a list: its rows in display order and
/// every file it holds (folders select files that are not on screen).
#[derive(Debug, Clone, Default)]
pub(crate) struct ListRows {
    pub(crate) rows: Vec<SelRow>,
    pub(crate) files: Vec<PathBuf>,
}

impl ListRows {
    fn index_of(&self, path: &Path) -> Option<usize> {
        self.rows.iter().position(|r| r.path == path)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FileSelection {
    list: Option<FileList>,
    files: HashSet<PathBuf>,
    folders: HashSet<PathBuf>,
    anchor: Option<PathBuf>,
}

impl FileSelection {
    /// How many files are selected (folders count their files).
    pub(crate) fn len(&self) -> usize {
        self.files.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.files.is_empty() && self.folders.is_empty()
    }

    /// The list the selection belongs to, when there is one.
    pub(crate) fn list(&self) -> Option<FileList> {
        self.list.filter(|_| !self.is_empty())
    }

    /// Whether the row `path` of `list` is highlighted as selected.
    pub(crate) fn contains(&self, list: FileList, path: &Path, folder: bool) -> bool {
        self.list == Some(list)
            && if folder {
                self.folders.contains(path)
            } else {
                self.files.contains(path)
            }
    }

    pub(crate) fn clear(&mut self) {
        self.files.clear();
        self.folders.clear();
        self.anchor = None;
        self.list = None;
    }

    fn scope(&mut self, list: FileList) {
        if self.list != Some(list) {
            self.clear();
            self.list = Some(list);
        }
    }

    fn insert(&mut self, row: &SelRow, rows: &ListRows) {
        if row.folder {
            self.folders.insert(row.path.clone());
            self.files.extend(
                rows.files
                    .iter()
                    .filter(|f| f.starts_with(&row.path))
                    .cloned(),
            );
        } else {
            self.files.insert(row.path.clone());
        }
    }

    fn remove(&mut self, row: &SelRow) -> bool {
        if row.folder {
            let had = self.folders.remove(&row.path);
            let before = self.files.len();
            self.files.retain(|f| !f.starts_with(&row.path));
            had || before != self.files.len()
        } else {
            self.files.remove(&row.path)
        }
    }

    /// A plain click: forget the selection, remember where a following
    /// Shift+click should start from.
    pub(crate) fn reset_to(&mut self, list: FileList, anchor: Option<PathBuf>) {
        self.clear();
        self.list = Some(list);
        self.anchor = anchor;
    }

    /// Ctrl+click / Space: flips row `at`. The row the cursor was on joins
    /// the selection the moment a second one is added, the way Ctrl+click
    /// extends what is already highlighted.
    pub(crate) fn toggle(
        &mut self,
        list: FileList,
        rows: &ListRows,
        at: usize,
        cursor: Option<usize>,
    ) {
        let Some(row) = rows.rows.get(at) else {
            return;
        };
        self.scope(list);
        if self.is_empty() {
            if let Some(cursor_row) = cursor.filter(|c| *c != at).and_then(|c| rows.rows.get(c)) {
                self.insert(cursor_row, rows);
            }
        }
        if !self.remove(row) {
            self.insert(row, rows);
        }
        self.anchor = Some(row.path.clone());
    }

    /// Shift+click / Shift+Up/Down: every row between the anchor (or the
    /// cursor when there is none yet) and `target`.
    pub(crate) fn extend_to(
        &mut self,
        list: FileList,
        rows: &ListRows,
        target: usize,
        cursor: Option<usize>,
    ) {
        self.scope(list);
        let anchor_index = self
            .anchor
            .as_deref()
            .and_then(|a| rows.index_of(a))
            .or(cursor);
        let Some(anchor_index) = anchor_index else {
            return;
        };
        if self.anchor.is_none() {
            self.anchor = rows.rows.get(anchor_index).map(|r| r.path.clone());
        }
        let (lo, hi) = if anchor_index <= target {
            (anchor_index, target)
        } else {
            (target, anchor_index)
        };
        self.files.clear();
        self.folders.clear();
        for row in rows.rows.iter().skip(lo).take(hi - lo + 1) {
            self.insert(row, rows);
        }
    }

    /// Selects every file of the list.
    pub(crate) fn select_all(&mut self, list: FileList, rows: &ListRows) {
        self.clear();
        self.list = Some(list);
        for row in &rows.rows {
            self.insert(row, rows);
        }
        self.anchor = rows.rows.first().map(|r| r.path.clone());
    }

    /// The selected files, in the list's own order (the files the list
    /// holds first, so a path that left the list is dropped).
    pub(crate) fn files_in_order(&self, rows: &ListRows) -> Vec<PathBuf> {
        rows.files
            .iter()
            .filter(|f| self.files.contains(*f))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> ListRows {
        // src/ (folder), src/a.rs, src/b.rs, c.txt
        let f = |p: &str| SelRow {
            path: p.into(),
            folder: false,
        };
        ListRows {
            rows: vec![
                SelRow {
                    path: "src".into(),
                    folder: true,
                },
                f("src/a.rs"),
                f("src/b.rs"),
                f("c.txt"),
            ],
            files: vec!["src/a.rs".into(), "src/b.rs".into(), "c.txt".into()],
        }
    }

    #[test]
    fn ctrl_click_adds_the_cursor_row_then_toggles() {
        let mut sel = FileSelection::default();
        let r = rows();
        sel.toggle(FileList::Unstaged, &r, 3, Some(1));
        assert_eq!(sel.len(), 2);
        sel.toggle(FileList::Unstaged, &r, 3, Some(1));
        assert_eq!(sel.len(), 1);
        assert!(sel.contains(FileList::Unstaged, Path::new("src/a.rs"), false));
    }

    #[test]
    fn a_folder_row_selects_its_files_even_when_collapsed() {
        let mut sel = FileSelection::default();
        // Collapsed: only the folder row and c.txt are on screen.
        let mut r = rows();
        r.rows.remove(2);
        r.rows.remove(1);
        sel.toggle(FileList::Staged, &r, 0, None);
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(FileList::Staged, Path::new("src"), true));
        sel.toggle(FileList::Staged, &r, 0, None);
        assert!(sel.is_empty());
    }

    #[test]
    fn shift_takes_the_range_from_the_anchor() {
        let mut sel = FileSelection::default();
        let r = rows();
        sel.reset_to(FileList::Commit, Some("src/a.rs".into()));
        sel.extend_to(FileList::Commit, &r, 3, Some(1));
        assert_eq!(
            sel.files_in_order(&r),
            vec![
                PathBuf::from("src/a.rs"),
                PathBuf::from("src/b.rs"),
                PathBuf::from("c.txt")
            ]
        );
        // Back up to the folder row: the folder and its files.
        sel.extend_to(FileList::Commit, &r, 0, Some(3));
        assert_eq!(sel.len(), 2, "the folder row and both its files, not c.txt");
        assert!(sel.contains(FileList::Commit, Path::new("src"), true));
    }

    #[test]
    fn shift_without_an_anchor_starts_at_the_cursor() {
        let mut sel = FileSelection::default();
        sel.extend_to(FileList::WipAll, &rows(), 2, Some(1));
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn another_list_starts_over_and_a_plain_click_clears() {
        let mut sel = FileSelection::default();
        let r = rows();
        sel.toggle(FileList::Unstaged, &r, 1, None);
        sel.toggle(FileList::Staged, &r, 3, None);
        assert_eq!(sel.list(), Some(FileList::Staged));
        assert_eq!(sel.len(), 1);
        sel.reset_to(FileList::Staged, Some("c.txt".into()));
        assert!(sel.is_empty());
        assert_eq!(sel.list(), None);
    }

    #[test]
    fn select_all_takes_every_file() {
        let mut sel = FileSelection::default();
        sel.select_all(FileList::Unstaged, &rows());
        assert_eq!(sel.len(), 3);
    }
}
