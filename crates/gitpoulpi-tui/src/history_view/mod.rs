//! The File History split view: the file's commits on the left, the
//! selected commit's diff of that file on the right.
//!
//! The right side is **the ordinary Diff view**, not a copy of it: the view
//! keeps `diff_view_state` open as a `DiffSource::Compare` state keyed by
//! [`HISTORY_KEY`], so every mode (hunk / inline / split, word diff,
//! minimap, wrap), the scrollbars and the keys come for free and the daemon
//! round trip is the one the Diff view already makes (`working_diff`,
//! latest request wins, stale answers ignored). This module owns what is
//! new: the paged list of commits, the selection (a row or a range), which
//! diff that selection stands for ([`HistoryView::diff_spec`]), a small
//! per-selection cache and a debounce so a held arrow key asks for one diff,
//! not one per repeat.
//!
//! The History *intent* (`FileInsightKind::History`, created by the ten
//! places that open a history) is promoted into this view at the next tick
//! ([`HistoryView::promote`]); nothing ever sends it to the daemon.

mod actions;
pub(crate) mod rename;
mod render;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{DiffTarget, FileChangeKind, FileDiff, FileHistoryEntry, Oid};
use ratatui::layout::Rect;

use crate::daemon_link::{
    FileDiffRequest, FileInsightKind, FileInsightOutcome, FileInsightPayload, FileInsightRequest,
};
use crate::diff_view::{DiffSource, DiffViewState};

pub(crate) use actions::{menu_entries, HistoryAction};
pub(crate) use rename::merge_rename;
pub(crate) use render::{render_list, render_strip, HistoryHit, ListPaint};

/// Commits asked for per page.
pub(crate) const PAGE: usize = 100;
/// The next page is asked for when the cursor is this close to the end.
const NEAR_END: usize = 15;
/// How long a selection must stand before its diff is asked for: a held
/// arrow key moves the cursor every ~30 ms and wants one diff at the end.
pub(crate) const DEBOUNCE: Duration = Duration::from_millis(40);
/// Diffs kept for the rows already visited.
const CACHE: usize = 32;
/// The diff state's synthetic commit key: how the rest of the window tells
/// the History's diff from a Log comparison's.
pub(crate) const HISTORY_KEY: Oid = Oid([0xA7; 20]);
/// The list's width, in percent of the central column, and its bounds.
pub(crate) const DEFAULT_SPLIT_PCT: u16 = 40;
pub(crate) const SPLIT_PCT_RANGE: (u16, u16) = (15, 75);
/// What one keyboard step of the splitter moves.
pub(crate) const SPLIT_STEP: u16 = 5;
/// Narrower than this the list has no room for the diff beside it.
const MIN_COLS: u16 = 12;
/// Terminal rows one commit takes in the list: the summary line and the
/// author line. Every row arithmetic (paint, clicks, the wheel, the
/// scrollbar, paging) goes through this one constant.
pub(crate) const ROW_H: u16 = 2;
/// Rows of the splitter lost to a header: none, the list is laid out under
/// the view's bar (which spans the whole central column), so the splitter
/// is as tall as the list.
pub(crate) const HEADER_ROWS: u16 = 0;

/// What the right side shows for the selected commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ViewMode {
    /// The commit's diff of the file (the ordinary Diff view).
    Diff,
    /// The file as it was at the commit.
    File,
    /// The file blamed as of the commit.
    Blame,
}

impl ViewMode {
    /// The File View / Blame request a mode stands for, `None` for the diff.
    pub(crate) fn kind(self) -> Option<FileInsightKind> {
        match self {
            ViewMode::Diff => None,
            ViewMode::File => Some(FileInsightKind::Content),
            ViewMode::Blame => Some(FileInsightKind::Blame),
        }
    }

    pub(crate) fn of_kind(kind: FileInsightKind) -> Option<ViewMode> {
        match kind {
            FileInsightKind::Content => Some(ViewMode::File),
            FileInsightKind::Blame => Some(ViewMode::Blame),
            _ => None,
        }
    }
}

/// Which half of the view has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pane {
    List,
    Diff,
}

/// What the right side shows, in the terms the Diff view asks the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffSpec {
    pub(crate) target: DiffTarget,
    pub(crate) path: PathBuf,
    /// The other name of the file, when the two ends differ by a rename.
    pub(crate) also: Option<PathBuf>,
}

impl DiffSpec {
    /// Whether the answer can change under us (it reads the disk).
    fn is_live(&self) -> bool {
        matches!(self.target, DiffTarget::CommitVsWorkingTree(_))
    }
}

pub(crate) struct HistoryView {
    /// The file's name at the newest end of the walk.
    pub(crate) path: PathBuf,
    /// The commit the walk starts from (HEAD from the working tree).
    pub(crate) start: Oid,
    pub(crate) entries: Vec<FileHistoryEntry>,
    /// Older commits exist beyond `entries`.
    pub(crate) more: bool,
    /// The first page has answered (or failed, or HEAD is unborn).
    pub(crate) first_loaded: bool,
    /// The `skip` of the page in flight.
    inflight: Option<u32>,
    pub(crate) error: Option<String>,
    /// The row with the cursor, and the other end of a Shift range.
    pub(crate) cursor: usize,
    pub(crate) anchor: Option<usize>,
    pub(crate) offset: usize,
    /// Whether the offset follows the cursor (a key moved it) or stays where
    /// the wheel or the scrollbar left it.
    pub(crate) follow: bool,
    pub(crate) pane: Pane,
    /// The list's width, percent of the central column.
    pub(crate) split_pct: u16,
    pub(crate) dragging_split: bool,
    /// The list was hidden with its cross (remembered in the session layout,
    /// like the commit column's choice): a thin strip at the left of the
    /// right side brings it back, and the keyboard stays on the right side.
    pub(crate) hidden: bool,
    /// The reopen strip as last painted (empty while the list is shown).
    pub(crate) strip: Rect,
    /// The list's bordered rect and the rows inside it, as last painted
    /// (empty when not drawn): what the mouse and the scrollbar read.
    pub(crate) list_area: Rect,
    pub(crate) rows_area: Rect,
    /// The central column the view was last laid out in.
    pub(crate) column: Rect,
    /// A commit to put the cursor on once its row has loaded (`ui show
    /// file ... history` with a revision).
    pub(crate) want: Option<Oid>,
    /// `true`: the right side compares the cursor row with the working tree.
    pub(crate) vs_worktree: bool,
    /// What the right side shows. Kept while the cursor moves between rows.
    /// `File` and `Blame` live in the window's `file_insight`, which this
    /// view re-points at the cursor row (`insight_request`). The view bar's
    /// File View / Diff View / Blame buttons (and their keys) set it.
    pub(crate) mode: ViewMode,
    changed_at: Instant,
    shown: Option<DiffSpec>,
    cache: Vec<(DiffSpec, Vec<FileDiff>)>,
}

impl HistoryView {
    /// The view an open History request stands for.
    pub(crate) fn promote(request: &FileInsightRequest, split_pct: u16) -> Self {
        let now = Instant::now();
        HistoryView {
            path: request.path.clone(),
            start: request.at,
            entries: Vec::new(),
            more: false,
            first_loaded: request.is_unborn(),
            inflight: None,
            error: None,
            cursor: 0,
            anchor: None,
            offset: 0,
            follow: true,
            pane: Pane::List,
            split_pct: clamp_pct(split_pct),
            dragging_split: false,
            hidden: false,
            strip: Rect::default(),
            list_area: Rect::default(),
            rows_area: Rect::default(),
            column: Rect::default(),
            want: None,
            vs_worktree: false,
            mode: ViewMode::Diff,
            // Long past: the first diff is asked for at once.
            changed_at: now - DEBOUNCE,
            shown: None,
            cache: Vec::new(),
        }
    }

    // ---- paging ---------------------------------------------------------

    /// The next page to ask for, marked in flight: the first one, then one
    /// whenever the cursor nears the end of what is loaded. At most one at a
    /// time, and none after a failure.
    pub(crate) fn next_request(&mut self, viewport: usize) -> Option<FileInsightRequest> {
        if self.inflight.is_some() || self.error.is_some() {
            return None;
        }
        let skip = if !self.first_loaded {
            0
        } else if self.more && (self.want.is_some() || self.near_end(viewport)) {
            self.entries.len()
        } else {
            return None;
        };
        self.inflight = Some(skip as u32);
        Some(FileInsightRequest {
            path: self.path.clone(),
            at: self.start,
            kind: FileInsightKind::HistoryPage(skip as u32),
            worktree: false,
            staged: false,
        })
    }

    fn near_end(&self, viewport: usize) -> bool {
        let last_seen = (self.offset + viewport).max(self.cursor + 1);
        last_seen + NEAR_END >= self.entries.len()
    }

    /// Takes the answer to one of this view's page reads; hands anything
    /// else back. A page that is not the one in flight is dropped.
    pub(crate) fn offer(&mut self, outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
        let FileInsightKind::HistoryPage(skip) = outcome.request.kind else {
            return Some(outcome);
        };
        if outcome.request.path != self.path
            || outcome.request.at != self.start
            || self.inflight != Some(skip)
        {
            return None;
        }
        self.inflight = None;
        match outcome.result {
            Ok(FileInsightPayload::HistoryPage(page)) => {
                self.more = page.more;
                self.entries.extend(page.entries);
                self.first_loaded = true;
                if let Some(want) = self.want {
                    if let Some(i) = self.entries.iter().position(|e| e.commit == want) {
                        self.want = None;
                        self.set_cursor(i, false);
                    } else if !self.more {
                        self.want = None;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                self.first_loaded = true;
                self.error = Some(e.to_string());
            }
        }
        None
    }

    /// A page is being read: the list shows its loading row.
    pub(crate) fn loading(&self) -> bool {
        self.inflight.is_some() || !self.first_loaded
    }

    /// Rows the list scrolls over: the commits, and the loading row.
    pub(crate) fn rows(&self) -> usize {
        self.entries.len() + usize::from(self.more || self.loading())
    }

    // ---- selection ------------------------------------------------------

    pub(crate) fn current(&self) -> Option<&FileHistoryEntry> {
        self.entries.get(self.cursor)
    }

    /// The rows the selection covers, newest first: one, or a range.
    pub(crate) fn range(&self) -> (usize, usize) {
        match self.anchor {
            Some(a) => (a.min(self.cursor), a.max(self.cursor)),
            None => (self.cursor, self.cursor),
        }
    }

    pub(crate) fn is_range(&self) -> bool {
        self.anchor.is_some_and(|a| a != self.cursor)
    }

    pub(crate) fn in_selection(&self, row: usize) -> bool {
        let (lo, hi) = self.range();
        (lo..=hi).contains(&row)
    }

    /// Moves the cursor by `delta` rows; `extend` keeps (or starts) a range.
    pub(crate) fn move_by(&mut self, delta: isize, extend: bool) {
        let last = self.entries.len().saturating_sub(1) as isize;
        let next = (self.cursor as isize + delta).clamp(0, last.max(0)) as usize;
        self.set_cursor(next, extend);
    }

    pub(crate) fn set_cursor(&mut self, row: usize, extend: bool) {
        if self.entries.is_empty() {
            return;
        }
        let row = row.min(self.entries.len() - 1);
        if extend {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
        if row != self.cursor || !extend {
            self.vs_worktree = false;
        }
        if row != self.cursor {
            self.cursor = row;
            self.changed_at = Instant::now();
        }
        self.follow = true;
    }

    /// Switches the right side between the row's own diff and its diff
    /// against the working tree; `true` when it now compares.
    pub(crate) fn toggle_worktree(&mut self) -> bool {
        self.vs_worktree = !self.vs_worktree;
        self.anchor = None;
        self.changed_at = Instant::now();
        self.vs_worktree
    }

    /// Esc: drops a range or a working-tree comparison before closing.
    pub(crate) fn clear_extras(&mut self) -> bool {
        let had = self.anchor.take().is_some() || std::mem::take(&mut self.vs_worktree);
        if had {
            self.changed_at = Instant::now();
        }
        had
    }

    /// The wheel and the scrollbar: moves the viewport, not the cursor.
    pub(crate) fn scroll_to(&mut self, offset: usize, viewport: usize) {
        let max = self.rows().saturating_sub(viewport.max(1));
        self.offset = offset.min(max);
        self.follow = false;
    }

    /// Keeps the offset legal, and the cursor in view while it is followed.
    pub(crate) fn clamp(&mut self, viewport: usize) {
        let viewport = viewport.max(1);
        let max = self.rows().saturating_sub(viewport);
        if self.follow {
            if self.cursor < self.offset {
                self.offset = self.cursor;
            } else if self.cursor >= self.offset + viewport {
                self.offset = self.cursor + 1 - viewport;
            }
        }
        self.offset = self.offset.min(max);
    }

    // ---- the split --------------------------------------------------------

    /// Forgets what was painted: the list is not on screen this frame.
    pub(crate) fn forget_list(&mut self) {
        self.list_area = Rect::default();
        self.rows_area = Rect::default();
        self.strip = Rect::default();
    }

    /// The splitter's cells, along the right side's left edge under its bar
    /// row, for a list that `split` laid out in `column`.
    pub(crate) fn divider_rect(&self, column: Rect) -> Rect {
        let (list, _) = self.split(column);
        Rect::new(
            list.right(),
            column.y + HEADER_ROWS,
            1,
            column.height.saturating_sub(HEADER_ROWS),
        )
    }

    pub(crate) fn set_split(&mut self, pct: u16) {
        self.split_pct = clamp_pct(pct);
    }

    /// The list's rect and the diff's, in `column`. Too narrow for both:
    /// the diff alone is not an option (the list is the view), so the list
    /// keeps `MIN_COLS` and the diff takes what is left, down to nothing.
    pub(crate) fn split(&self, column: Rect) -> (Rect, Rect) {
        let width = column.width;
        let list = ((width as u32 * self.split_pct as u32) / 100) as u16;
        let list = list.clamp(MIN_COLS.min(width), width);
        let list = if width.saturating_sub(list) < MIN_COLS {
            width.saturating_sub(MIN_COLS).max(MIN_COLS.min(width))
        } else {
            list
        };
        (
            Rect::new(column.x, column.y, list, column.height),
            Rect::new(
                column.x + list,
                column.y,
                width.saturating_sub(list),
                column.height,
            ),
        )
    }

    /// The percentage a pointer at `col` stands for.
    pub(crate) fn pct_at(&self, col: u16) -> u16 {
        let width = self.column.width.max(1) as u32;
        let from = col.saturating_sub(self.column.x) as u32;
        clamp_pct((from * 100 / width) as u16)
    }

    // ---- the diff on the right -------------------------------------------

    /// What the selection asks the daemon for.
    pub(crate) fn diff_spec(&self) -> Option<DiffSpec> {
        let entry = self.current()?;
        if self.vs_worktree {
            return Some(DiffSpec {
                target: DiffTarget::CommitVsWorkingTree(entry.commit),
                path: self.path.clone(),
                also: (entry.path != self.path).then(|| entry.path.clone()),
            });
        }
        if self.is_range() {
            let (lo, hi) = self.range();
            let (newer, older) = (self.entries.get(lo)?, self.entries.get(hi)?);
            return Some(DiffSpec {
                target: DiffTarget::CommitRange(older.commit, newer.commit),
                path: newer.path.clone(),
                also: (older.path != newer.path).then(|| older.path.clone()),
            });
        }
        Some(DiffSpec {
            target: DiffTarget::CommitVsParent(entry.commit),
            path: entry.path.clone(),
            also: match &entry.change {
                FileChangeKind::Renamed { from } => Some(from.clone()),
                _ => None,
            },
        })
    }

    /// Whether `state` is this view's diff.
    pub(crate) fn owns(state: &DiffViewState) -> bool {
        state.source == DiffSource::Compare && state.commit == HISTORY_KEY
    }

    /// The diff state to start with: before the first page lands there is no
    /// selection, so it follows the walk's own commit.
    pub(crate) fn initial_state(&self) -> DiffViewState {
        let target = DiffTarget::CommitVsParent(self.start);
        let mut state = DiffViewState::for_compare(HISTORY_KEY, target, 0, self.path.clone())
            .resume_last_used();
        state.also_path = None;
        state
    }

    /// Points `state` at the selection once it has stood for the debounce
    /// (or its diff is already cached). Returns when a later look is due.
    pub(crate) fn sync_diff(
        &mut self,
        state: &mut Option<DiffViewState>,
        now: Instant,
    ) -> Option<Instant> {
        let spec = self.diff_spec()?;
        let intact = state.as_ref().is_some_and(Self::owns);
        if intact && self.shown.as_ref() == Some(&spec) {
            return None;
        }
        let due = self.changed_at + DEBOUNCE;
        if intact && now < due && !self.is_cached(&spec) {
            return Some(due);
        }
        let mut next =
            DiffViewState::for_compare(HISTORY_KEY, spec.target.clone(), 0, spec.path.clone())
                .resume_last_used();
        next.also_path = spec.also.clone();
        *state = Some(next);
        self.shown = Some(spec);
        None
    }

    fn is_cached(&self, spec: &DiffSpec) -> bool {
        !spec.is_live() && self.cache.iter().any(|(s, _)| s == spec)
    }

    fn spec_of(request: &FileDiffRequest) -> Option<DiffSpec> {
        Some(DiffSpec {
            target: request.target.clone(),
            path: request.path.clone()?,
            also: request.also.clone(),
        })
    }

    /// The remembered answer for `request`, most recently used last.
    pub(crate) fn cached(&mut self, request: &FileDiffRequest) -> Option<Vec<FileDiff>> {
        let spec = Self::spec_of(request)?;
        if spec.is_live() {
            return None;
        }
        let at = self.cache.iter().position(|(s, _)| *s == spec)?;
        let hit = self.cache.remove(at);
        let files = hit.1.clone();
        self.cache.push(hit);
        Some(files)
    }

    /// Remembers an answer for the next visit of its row.
    pub(crate) fn remember(&mut self, request: &FileDiffRequest, files: &[FileDiff]) {
        let Some(spec) = Self::spec_of(request) else {
            return;
        };
        if spec.is_live() || self.cache.iter().any(|(s, _)| *s == spec) {
            return;
        }
        if self.cache.len() >= CACHE {
            self.cache.remove(0);
        }
        self.cache.push((spec, files.to_vec()));
    }

    /// When the loop must look again without input: a selection waiting out
    /// its debounce.
    pub(crate) fn next_deadline(&self, now: Instant) -> Option<Instant> {
        let due = self.changed_at + DEBOUNCE;
        (due > now && (self.mode != ViewMode::Diff || self.shown != self.diff_spec()))
            .then_some(due)
    }

    // ---- rows -------------------------------------------------------------

    pub(crate) fn row_at(&self, row: usize) -> Option<&FileHistoryEntry> {
        self.entries.get(row)
    }

    /// Whole rows the list shows (at least one).
    pub(crate) fn viewport_rows(&self) -> usize {
        usize::from(self.rows_area.height / ROW_H).max(1)
    }

    /// The screen row `row` starts on, as last painted.
    pub(crate) fn row_y(&self, row: usize) -> u16 {
        self.rows_area.y + (row.saturating_sub(self.offset) as u16) * ROW_H
    }

    // ---- the File / Blame side ------------------------------------------------

    /// The commit to read the file *at* for an entry: its own, except for a
    /// deletion, where the file is only in the parent.
    pub(crate) fn read_at(entry: &FileHistoryEntry) -> Oid {
        match entry.change {
            FileChangeKind::Deleted => entry.parents.first().copied().unwrap_or(entry.commit),
            _ => entry.commit,
        }
    }

    /// The File View / Blame read the cursor row asks for in `mode`.
    pub(crate) fn insight_request(&self, mode: ViewMode) -> Option<FileInsightRequest> {
        let entry = self.current()?;
        Some(FileInsightRequest {
            path: entry.path.clone(),
            at: Self::read_at(entry),
            kind: mode.kind()?,
            worktree: false,
            staged: false,
        })
    }

    /// Whether a File View / Blame request is this view's own right side
    /// (one of the file's commits), not some other file opened meanwhile.
    pub(crate) fn owns_insight(&self, request: &FileInsightRequest) -> bool {
        ViewMode::of_kind(request.kind).is_some()
            && !request.worktree
            && (request.path == self.path || self.entries.iter().any(|e| e.path == request.path))
    }

    /// When the selection last changed, for the debounce of the File /
    /// Blame read.
    pub(crate) fn changed_at(&self) -> Instant {
        self.changed_at
    }
}

pub(crate) fn clamp_pct(pct: u16) -> u16 {
    let (lo, hi) = SPLIT_PCT_RANGE;
    if pct == 0 {
        DEFAULT_SPLIT_PCT
    } else {
        pct.clamp(lo, hi)
    }
}

#[cfg(test)]
mod tests;
