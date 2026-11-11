//! The rows of a changed-files list, built in one place: the main window's
//! right pane (`event_loop/draw/right_panel/commit_detail.rs`) and the Undo
//! history / Reflog / Lost work dialog (`safety_panel`) both list a commit's
//! files, so a file looks the same (status marker, indent, folder chevron,
//! LFS tag, picked tint) wherever it is shown.
//!
//! Pure: this turns `TreeRow`s (`status_pane::build_tree_rows`, Path or Tree
//! mode) into `ListItem`s; the owner paints the `List`, its scrollbar and its
//! hit rects.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::FileStatus;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::status_pane::{status_marker, TreeRow};
use crate::th;
use crate::theme::file_status_color;

/// What decides how a row looks. The closures keep the owner's own state
/// (the multi-selection, the LFS index, the commit's file statuses) out of
/// here.
pub(crate) struct FileRows<'a> {
    pub(crate) rows: &'a [TreeRow],
    /// The paths `TreeRow::File::index` points into.
    pub(crate) paths: &'a [PathBuf],
    /// The commit's status of a path (`None`: unchanged, a View all files row).
    pub(crate) status_of: &'a dyn Fn(&Path) -> Option<FileStatus>,
    pub(crate) is_lfs: &'a dyn Fn(&Path) -> bool,
    /// A file row that is picked (multi-selection).
    pub(crate) file_picked: &'a dyn Fn(&Path) -> bool,
    /// A folder row whose files are all picked.
    pub(crate) folder_picked: &'a dyn Fn(&Path) -> bool,
}

impl FileRows<'_> {
    pub(crate) fn items(&self) -> Vec<ListItem<'static>> {
        self.rows
            .iter()
            .map(|row| match row {
                TreeRow::Folder {
                    depth,
                    label,
                    collapsed,
                    path,
                } => {
                    let chevron = crate::fold_mark::glyph(*collapsed);
                    let item = ListItem::new(Line::from(Span::styled(
                        format!("{}{chevron} {label}/", "  ".repeat(*depth)),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::BOLD),
                    )));
                    if (self.folder_picked)(path) {
                        item.style(Style::default().bg(th().ui.selection))
                    } else {
                        item
                    }
                }
                TreeRow::File {
                    depth,
                    label,
                    index,
                } => {
                    let path = self.paths.get(*index);
                    let status = path.and_then(|p| (self.status_of)(p));
                    let marker = status.map(status_marker).unwrap_or(" ");
                    let color = status.map(file_status_color).unwrap_or(th().ui.muted);
                    let mut spans = vec![
                        Span::styled(
                            format!("{}{marker} ", "  ".repeat(*depth)),
                            Style::default().fg(color),
                        ),
                        Span::styled(label.clone(), Style::default().fg(th().ui.text)),
                    ];
                    // Tagged by whether the path is an LFS file in the
                    // *current* index: the file almost certainly was one when
                    // the commit was made, and asking per commit would cost a
                    // read per selection.
                    if path.is_some_and(|p| (self.is_lfs)(p)) {
                        spans.push(Span::styled(
                            " LFS",
                            Style::default()
                                .fg(th().ui.accent)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    let item = ListItem::new(Line::from(spans));
                    // Multi-selected rows (Ctrl/Shift+click) read as picked.
                    if path.is_some_and(|p| (self.file_picked)(p)) {
                        item.style(Style::default().bg(th().ui.selection))
                    } else {
                        item
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status_pane::{build_tree_rows, FileListMode};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{List, Widget};
    use std::collections::HashSet;

    /// The commit-detail pane's own row building as it was before the rows
    /// moved here, kept as the reference the shared one must equal.
    fn reference(
        rows: &[TreeRow],
        paths: &[PathBuf],
        status: &dyn Fn(&Path) -> Option<FileStatus>,
        lfs: &dyn Fn(&Path) -> bool,
        picked: &dyn Fn(&Path) -> bool,
    ) -> Vec<ListItem<'static>> {
        rows.iter()
            .map(|row| match row {
                TreeRow::Folder {
                    depth,
                    label,
                    collapsed,
                    ..
                } => {
                    let chevron = crate::fold_mark::glyph(*collapsed);
                    ListItem::new(Line::from(Span::styled(
                        format!("{}{chevron} {label}/", "  ".repeat(*depth)),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::BOLD),
                    )))
                }
                TreeRow::File {
                    depth,
                    label,
                    index,
                } => {
                    let status = paths.get(*index).and_then(|p| status(p));
                    let marker = status.map(status_marker).unwrap_or(" ");
                    let color = status.map(file_status_color).unwrap_or(th().ui.muted);
                    let mut spans = vec![
                        Span::styled(
                            format!("{}{marker} ", "  ".repeat(*depth)),
                            Style::default().fg(color),
                        ),
                        Span::styled(label.clone(), Style::default().fg(th().ui.text)),
                    ];
                    if paths.get(*index).is_some_and(|p| lfs(p)) {
                        spans.push(Span::styled(
                            " LFS",
                            Style::default()
                                .fg(th().ui.accent)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    let item = ListItem::new(Line::from(spans));
                    if paths.get(*index).is_some_and(|p| picked(p)) {
                        item.style(Style::default().bg(th().ui.selection))
                    } else {
                        item
                    }
                }
            })
            .collect()
    }

    fn paint(items: Vec<ListItem<'static>>) -> Buffer {
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        List::new(items).render(area, &mut buf);
        buf
    }

    #[test]
    fn shared_rows_match_the_commit_detail_rows_in_path_and_tree_mode() {
        let paths: Vec<PathBuf> = ["src/a.rs", "src/deep/b.rs", "README.md", "big.bin"]
            .iter()
            .map(PathBuf::from)
            .collect();
        let status = |p: &Path| match p.to_str() {
            Some("src/a.rs") => Some(FileStatus::Modified),
            Some("src/deep/b.rs") => Some(FileStatus::Added),
            Some("README.md") => Some(FileStatus::Deleted),
            _ => None,
        };
        let lfs = |p: &Path| p.to_str() == Some("big.bin");
        let picked = |p: &Path| p.to_str() == Some("README.md");
        let none = |_: &Path| false;
        for mode in [FileListMode::Path, FileListMode::Tree] {
            let rows = build_tree_rows(&paths, mode, &HashSet::new(), "");
            let shared = FileRows {
                rows: &rows,
                paths: &paths,
                status_of: &status,
                is_lfs: &lfs,
                file_picked: &picked,
                folder_picked: &none,
            }
            .items();
            let old = reference(&rows, &paths, &status, &lfs, &picked);
            assert_eq!(paint(shared), paint(old), "{mode:?}");
            // Not a vacuous pass: the rows carry the markers.
            let text = crate::dialog_kit::buffer_text(&paint(
                FileRows {
                    rows: &rows,
                    paths: &paths,
                    status_of: &status,
                    is_lfs: &lfs,
                    file_picked: &picked,
                    folder_picked: &none,
                }
                .items(),
            ));
            assert!(text.contains("a.rs") && text.contains("LFS"), "{text}");
        }
    }
}
