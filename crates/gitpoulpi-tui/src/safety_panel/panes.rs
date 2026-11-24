//! The panes of the Reflog and Lost work views: the commit list on the left,
//! and on the right the commit's header, its changed files (the shared
//! `file_rows` list, Path or Tree) and the selected file's hunks.
//!
//! This file is state and geometry only (no painting): which pane has the
//! keyboard, where the two dividers are (`pane_split` weights, so a resize of
//! the dialog keeps the proportions), the files' tree rows, and the diff
//! reads. A diff is asked of the daemon off the render thread
//! (`Diff { CommitVsParent(oid), paths: [path] }`), one request at a time with
//! the latest wanted file asked next (a selection that moves on is never
//! waited for), and the last answers are kept so going back is free.

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{DiffTarget, FileDiff, Oid, OperationKind};
use ratatui::layout::Rect;
use ratatui::text::Line;

use super::ListState;
use crate::daemon_link::{error_text, Reply, Requester};
use crate::diff_view::{render_diff_with, DiffRenderOpts, DiffViewMode};
use crate::pane_split;
use crate::status_pane::{build_tree_rows, FileListMode, TreeRow};

/// The panes `Tab` walks through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pane {
    List,
    Files,
    Diff,
}

impl Pane {
    pub(crate) fn next(self) -> Self {
        match self {
            Pane::List => Pane::Files,
            Pane::Files => Pane::Diff,
            Pane::Diff => Pane::List,
        }
    }

    pub(crate) fn prev(self) -> Self {
        match self {
            Pane::List => Pane::Diff,
            Pane::Files => Pane::List,
            Pane::Diff => Pane::Files,
        }
    }
}

/// The two dividers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Divider {
    /// Between the commit list and the right pane.
    Vertical,
    /// Between the file list and the diff.
    Horizontal,
}

/// Default shares of the row (list | right) and of the right pane's column
/// (files | diff), per ten thousand.
pub(super) const DEFAULT_COLS: [u16; 2] = [5_000, 5_000];
pub(super) const DEFAULT_ROWS: [u16; 2] = [3_500, 6_500];
/// The least a box on the right keeps: two borders and two lines.
pub(super) const MIN_BOX_H: u16 = 4;
/// Two presses on a divider this close in time reset it.
const DOUBLE_CLICK: Duration = Duration::from_millis(500);
/// How many diffs are kept.
const DIFF_CACHE: usize = 16;

/// Where everything of the panes is, as the last frame painted it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PaneLayout {
    /// The commit list's bordered box.
    pub(crate) list: Rect,
    /// The commit header (no border).
    pub(crate) header: Rect,
    /// The file list's bordered box.
    pub(crate) files: Rect,
    /// The diff's bordered box.
    pub(crate) diff: Rect,
    /// What a press on the vertical divider grabs: the gap and the left
    /// border of the right pane.
    pub(crate) grab_v: Rect,
    /// What a press on the horizontal divider grabs: the file list's bottom
    /// border (the diff's top border stays the diff's, so its pane badge has
    /// a cell).
    pub(crate) grab_h: Rect,
}

impl PaneLayout {
    /// The area the list and the right pane share (the gap included).
    pub(crate) fn body(&self) -> Rect {
        Rect::new(
            self.list.x,
            self.list.y,
            (self.files.right()).saturating_sub(self.list.x),
            self.list.height,
        )
    }
}

/// Splits `body` into the panes. `header_lines` is what the commit header
/// would like (it shrinks so both boxes keep [`MIN_BOX_H`] rows when it can).
pub(super) fn layout(
    body: Rect,
    cols: Option<&[u16]>,
    rows: Option<&[u16]>,
    header_lines: u16,
) -> PaneLayout {
    // One column between the panes: the divider's gap.
    let usable = body.width.saturating_sub(1);
    let widths = pane_split::widths(Some(cols.unwrap_or(&DEFAULT_COLS)), usable, 2);
    let list_w = widths[0];
    let side_x = body.x + list_w + 1;
    let side_w = widths[1];
    let spare = body.height.saturating_sub(2 * MIN_BOX_H);
    let header_h = header_lines.min(spare.max(1)).min(body.height);
    let boxes_h = body.height - header_h;
    let heights =
        pane_split::widths_min(Some(rows.unwrap_or(&DEFAULT_ROWS)), boxes_h, 2, MIN_BOX_H);
    let files = Rect::new(side_x, body.y + header_h, side_w, heights[0]);
    let diff = Rect::new(side_x, files.bottom(), side_w, heights[1]);
    PaneLayout {
        list: Rect::new(body.x, body.y, list_w, body.height),
        header: Rect::new(side_x, body.y, side_w, header_h),
        files,
        diff,
        grab_v: Rect::new(side_x.saturating_sub(1), body.y + header_h, 2, boxes_h),
        grab_h: Rect::new(side_x, files.bottom().saturating_sub(1), side_w, 1),
    }
}

/// A diff that is wanted, or one that came.
pub(crate) type DiffKey = (Oid, PathBuf);

/// What the diff box shows.
pub(crate) enum DiffShown<'a> {
    /// No file is selected (or the selection is a folder).
    Nothing,
    Loading,
    Failed(&'a str),
    Ready(&'a FileDiff),
}

#[derive(Default)]
pub(crate) struct DiffPane {
    kept: Vec<(DiffKey, Result<FileDiff, String>)>,
    pending: Option<(DiffKey, Reply<Vec<FileDiff>>)>,
    /// What was last laid out: the key and width it was for.
    laid: Option<(DiffKey, u16, Rc<[Line<'static>]>)>,
}

impl DiffPane {
    /// Drains the answer and asks for `want` when nothing else is running.
    pub(crate) fn poll(&mut self, requester: &Requester, want: Option<&DiffKey>) {
        if let Some((key, reply)) = self.pending.as_ref() {
            if let Some(result) = reply.take() {
                let key = key.clone();
                self.pending = None;
                let file = result
                    .map(|files| {
                        files.into_iter().next().unwrap_or(FileDiff {
                            renamed_from: None,
                            similarity: None,
                            path: key.1.clone(),
                            status: gitpoulpi_core::domain::FileStatus::Modified,
                            hunks: Vec::new(),
                        })
                    })
                    .map_err(|e| error_text::error_text(&e));
                self.kept.retain(|(k, _)| *k != key);
                self.kept.push((key, file));
                if self.kept.len() > DIFF_CACHE {
                    let _ = self.kept.remove(0);
                }
            }
        }
        let Some(want) = want else {
            return;
        };
        if self.pending.is_some() || self.kept.iter().any(|(k, _)| k == want) {
            return;
        }
        self.pending = Some((
            want.clone(),
            requester.call(
                OperationKind::Diff,
                serde_json::json!({
                    "target": DiffTarget::CommitVsParent(want.0),
                    "paths": [want.1],
                }),
            ),
        ));
    }

    /// Keeps an answer, as if the read had landed.
    #[cfg(test)]
    pub(crate) fn keep(&mut self, key: DiffKey, file: FileDiff) {
        self.kept.push((key, Ok(file)));
    }

    /// Whether a read is on its way.
    pub(crate) fn pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(crate) fn shown(&self, want: Option<&DiffKey>) -> DiffShown<'_> {
        let Some(want) = want else {
            return DiffShown::Nothing;
        };
        match self.kept.iter().find(|(k, _)| k == want) {
            Some((_, Ok(file))) => DiffShown::Ready(file),
            Some((_, Err(why))) => DiffShown::Failed(why),
            None => DiffShown::Loading,
        }
    }

    /// The hunks of `file` as lines `width` cells wide (hunks only, wrapped),
    /// laid out once per file and width.
    pub(crate) fn lines(
        &mut self,
        key: &DiffKey,
        file: &FileDiff,
        width: u16,
    ) -> Rc<[Line<'static>]> {
        if let Some((k, w, lines)) = &self.laid {
            if k == key && *w == width {
                return Rc::clone(lines);
            }
        }
        let opts = DiffRenderOpts {
            mode: DiffViewMode::Inline,
            hunks_only: true,
            show_whitespace: true,
            word_wrap: true,
            width,
            h_offset: 0,
            full_file: None,
            lfs_raw: false,
            lfs_local: None,
            intraline: crate::live_settings::diff_intraline(),
        };
        let lines: Rc<[Line<'static>]> = render_diff_with(file, &opts).lines.into();
        self.laid = Some((key.clone(), width, Rc::clone(&lines)));
        lines
    }
}

/// The state of the panes.
pub(crate) struct Panes {
    pub(crate) focus: Pane,
    /// Shares of the row (list | right); `None`: the default.
    pub(crate) cols: Option<Vec<u16>>,
    /// Shares of the right column (files | diff); `None`: the default.
    pub(crate) rows: Option<Vec<u16>>,
    pub(crate) mode: FileListMode,
    pub(crate) collapsed: HashSet<PathBuf>,
    /// The file list (`selected` is a row of `tree`).
    pub(crate) files: ListState,
    /// The diff's scroll (`offset` is the first row shown).
    pub(crate) scroll: ListState,
    /// The commit whose files are listed.
    pub(crate) commit: Option<Oid>,
    /// The file list's rows: rebuilt when the commit, the mode or a folder
    /// changes.
    pub(crate) tree: Vec<TreeRow>,
    tree_key: Option<(Option<Oid>, FileListMode, u64, usize, bool)>,
    revision: u64,
    pub(crate) diff: DiffPane,
    /// Lines the diff box has (set by the paint, for its scrollbar).
    pub(crate) diff_len: usize,
    pub(crate) layout: PaneLayout,
    /// A divider being dragged.
    pub(crate) drag: Option<Divider>,
    last_press: Option<(Instant, Divider)>,
}

impl Default for Panes {
    fn default() -> Self {
        Self {
            focus: Pane::List,
            cols: None,
            rows: None,
            mode: FileListMode::Path,
            collapsed: HashSet::new(),
            files: ListState::default(),
            scroll: ListState::default(),
            commit: None,
            tree: Vec::new(),
            tree_key: None,
            revision: 0,
            diff: DiffPane::default(),
            diff_len: 0,
            layout: PaneLayout::default(),
            drag: None,
            last_press: None,
        }
    }
}

impl Panes {
    /// The commit under the list's cursor changed: the file list and the diff
    /// start over.
    pub(crate) fn follow(&mut self, commit: Option<Oid>) {
        if self.commit != commit {
            self.commit = commit;
            self.collapsed.clear();
            self.files = ListState::default();
            self.scroll = ListState::default();
            self.tree_key = None;
            self.tree.clear();
        }
    }

    /// Rebuilds the file rows when something they depend on changed, and puts
    /// the cursor on the first file after a new commit.
    pub(crate) fn rebuild(&mut self, paths: &[PathBuf]) {
        // The folders-first setting is part of the key: Settings can change it
        // while the panel is open, and the cached rows carry the old order.
        let key = (
            self.commit,
            self.mode,
            self.revision,
            paths.len(),
            crate::live_settings::tree_folders_first(),
        );
        if self.tree_key == Some(key) {
            return;
        }
        let fresh = self.tree.is_empty() || self.tree_key.is_none_or(|k| k.0 != key.0);
        self.tree = build_tree_rows(paths, self.mode, &self.collapsed, "");
        self.tree_key = Some(key);
        let len = self.tree.len();
        if fresh {
            self.files.selected = self
                .tree
                .iter()
                .position(|r| matches!(r, TreeRow::File { .. }))
                .unwrap_or(0);
            self.files.offset = 0;
        }
        self.files.clamp_selected(len);
    }

    /// The path of the file row under the cursor (`None` on a folder).
    pub(crate) fn selected_path<'a>(&self, paths: &'a [PathBuf]) -> Option<&'a PathBuf> {
        match self.tree.get(self.files.selected)? {
            TreeRow::File { index, .. } => paths.get(*index),
            TreeRow::Folder { .. } => None,
        }
    }

    /// Flips Path / Tree.
    pub(crate) fn toggle_mode(&mut self) {
        self.set_mode(self.mode.toggled());
    }

    pub(crate) fn set_mode(&mut self, mode: FileListMode) {
        if self.mode != mode {
            self.mode = mode;
            self.revision += 1;
            self.files = ListState::default();
            self.tree_key = None;
        }
    }

    /// Folds or unfolds the folder row at `row`; `true` when it was one.
    pub(crate) fn toggle_folder(&mut self, row: usize) -> bool {
        let Some(TreeRow::Folder { path, .. }) = self.tree.get(row) else {
            return false;
        };
        let path = path.clone();
        if !self.collapsed.remove(&path) {
            self.collapsed.insert(path);
        }
        self.revision += 1;
        true
    }

    /// Moves the file cursor; the diff starts at its top for the new file.
    pub(crate) fn move_file(&mut self, delta: isize) {
        let len = self.tree.len();
        let before = self.files.selected;
        self.files.move_by(delta, len);
        if self.files.selected != before {
            self.scroll = ListState::default();
        }
    }

    /// Puts the file cursor on `row` (a click).
    pub(crate) fn select_file(&mut self, row: usize) {
        let len = self.tree.len();
        let before = self.files.selected;
        self.files.selected = row.min(len.saturating_sub(1));
        self.files.keep_visible();
        if self.files.selected != before {
            self.scroll = ListState::default();
        }
    }

    // -- dividers ---------------------------------------------------------

    /// A press on a divider: `true` when it is the second of a double click
    /// (the caller resets it).
    pub(crate) fn press_divider(&mut self, which: Divider) -> bool {
        let now = Instant::now();
        let double = self
            .last_press
            .is_some_and(|(t, d)| d == which && now.duration_since(t) < DOUBLE_CLICK);
        self.last_press = (!double).then_some((now, which));
        if double {
            self.reset_divider(which);
            self.drag = None;
        } else {
            self.drag = Some(which);
        }
        double
    }

    pub(crate) fn reset_divider(&mut self, which: Divider) {
        match which {
            Divider::Vertical => self.cols = None,
            Divider::Horizontal => self.rows = None,
        }
    }

    pub(crate) fn reset_dividers(&mut self) {
        self.cols = None;
        self.rows = None;
    }

    /// The pointer is at `(col, row)` while a divider is held.
    pub(crate) fn drag_to(&mut self, col: u16, row: u16, body: Rect) {
        let Some(which) = self.drag else {
            return;
        };
        match which {
            Divider::Vertical => {
                let usable = body.width.saturating_sub(1);
                let current = pane_split::widths(
                    Some(self.cols.as_deref().unwrap_or(&DEFAULT_COLS)),
                    usable,
                    2,
                );
                let left = i32::from(col) - i32::from(body.x);
                if let Some(next) = pane_split::move_divider(&current, 0, left) {
                    self.cols = Some(pane_split::weights_for(&next));
                }
            }
            Divider::Horizontal => {
                let l = self.layout;
                let total = l.files.height + l.diff.height;
                let current = pane_split::widths_min(
                    Some(self.rows.as_deref().unwrap_or(&DEFAULT_ROWS)),
                    total,
                    2,
                    MIN_BOX_H,
                );
                let top = i32::from(row) - i32::from(l.files.y) + 1;
                if let Some(next) = pane_split::move_divider_min(&current, 0, top, MIN_BOX_H) {
                    self.rows = Some(pane_split::weights_for(&next));
                }
            }
        }
    }

    /// The keyboard twin: moves a divider one step. `towards` is `-1` (left /
    /// up) or `1` (right / down).
    pub(crate) fn nudge_divider(&mut self, which: Divider, towards: i32, body: Rect) {
        match which {
            Divider::Vertical => {
                let usable = body.width.saturating_sub(1);
                let current = pane_split::widths(
                    Some(self.cols.as_deref().unwrap_or(&DEFAULT_COLS)),
                    usable,
                    2,
                );
                let step = pane_split::key_step(usable);
                if let Some(next) =
                    pane_split::move_divider(&current, 0, i32::from(current[0]) + towards * step)
                {
                    self.cols = Some(pane_split::weights_for(&next));
                }
            }
            Divider::Horizontal => {
                let l = self.layout;
                let total = l.files.height + l.diff.height;
                let current = pane_split::widths_min(
                    Some(self.rows.as_deref().unwrap_or(&DEFAULT_ROWS)),
                    total,
                    2,
                    MIN_BOX_H,
                );
                let step = pane_split::key_step(total).min(3);
                if let Some(next) = pane_split::move_divider_min(
                    &current,
                    0,
                    i32::from(current[0]) + towards * step,
                    MIN_BOX_H,
                ) {
                    self.rows = Some(pane_split::weights_for(&next));
                }
            }
        }
    }

    /// Whether anything is still being read.
    pub(crate) fn busy(&self) -> bool {
        self.diff.pending()
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    const BODY: Rect = Rect::new(4, 5, 121, 30);

    #[test]
    fn the_panes_tile_the_body_without_overlap() {
        let l = layout(BODY, None, None, 4);
        assert_eq!(l.list.width + 1 + l.files.width, BODY.width);
        assert_eq!(l.list.height, BODY.height);
        assert_eq!(
            l.header.height + l.files.height + l.diff.height,
            BODY.height
        );
        assert_eq!(l.files.bottom(), l.diff.y);
        assert_eq!(l.files.x, l.diff.x);
        assert!(l.grab_v.contains((l.files.x - 1, l.files.y).into()));
        assert!(l.grab_h.contains((l.files.x, l.diff.y - 1).into()));
    }

    #[test]
    fn both_boxes_keep_their_minimum_in_a_short_body() {
        let l = layout(Rect::new(0, 0, 100, 9), None, None, 4);
        assert!(l.files.height >= MIN_BOX_H && l.diff.height >= MIN_BOX_H);
        assert_eq!(l.header.height, 1);
    }

    #[test]
    fn a_vertical_drag_moves_the_divider_and_keeps_the_proportion_on_resize() {
        let mut p = Panes::default();
        p.layout = layout(BODY, None, None, 4);
        p.press_divider(Divider::Vertical);
        p.drag_to(BODY.x + 30, BODY.y + 3, BODY);
        let moved = layout(BODY, p.cols.as_deref(), None, 4);
        assert_eq!(moved.list.width, 30);
        let wider = layout(Rect::new(4, 5, 241, 30), p.cols.as_deref(), None, 4);
        assert!(
            (59..=61).contains(&wider.list.width),
            "{}",
            wider.list.width
        );
    }

    #[test]
    fn a_double_press_resets_the_divider() {
        let mut p = Panes::default();
        p.layout = layout(BODY, None, None, 4);
        assert!(!p.press_divider(Divider::Vertical));
        p.drag_to(BODY.x + 30, BODY.y, BODY);
        assert!(p.cols.is_some());
        assert!(p.press_divider(Divider::Vertical));
        assert!(p.cols.is_none());
    }

    #[test]
    fn the_keyboard_twin_moves_the_horizontal_divider_and_stops_at_the_minimum() {
        let mut p = Panes::default();
        p.layout = layout(BODY, None, None, 4);
        let before = p.layout.files.height;
        p.nudge_divider(Divider::Horizontal, 1, BODY);
        let after = layout(BODY, p.rows.as_deref(), None, 4);
        let _ = after;
        let l = layout(BODY, None, p.rows.as_deref(), 4);
        assert!(l.files.height > before, "{} > {before}", l.files.height);
        for _ in 0..50 {
            p.nudge_divider(Divider::Horizontal, -1, BODY);
            p.layout = layout(BODY, None, p.rows.as_deref(), 4);
        }
        assert!(p.layout.files.height >= MIN_BOX_H);
    }

    #[test]
    fn the_file_cursor_starts_on_the_first_file_and_a_new_commit_starts_over() {
        let paths: Vec<PathBuf> = ["a/b.rs", "c.rs"].iter().map(PathBuf::from).collect();
        let mut p = Panes::default();
        p.mode = FileListMode::Tree;
        p.follow(Some(Oid([1; 20])));
        p.rebuild(&paths);
        assert_eq!(p.selected_path(&paths), Some(&paths[0]), "{:?}", p.tree);
        p.move_file(1);
        assert_eq!(p.selected_path(&paths), Some(&paths[1]));
        p.follow(Some(Oid([2; 20])));
        p.rebuild(&paths);
        assert_eq!(p.selected_path(&paths), Some(&paths[0]));
    }
}
