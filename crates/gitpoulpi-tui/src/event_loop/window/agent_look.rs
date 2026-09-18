//! The look-view intents of an external agent (plan
//! the mcp-show-ui design, section 5.3): show a diff at a
//! line, a file / blame / history at a position, and tint a range of what
//! is open. [`agent_view`](super::agent_view) decides *when*;
//! [`agent_show`](super::agent_show) selects the commit; this places the
//! view.
//!
//! The chain is a [`ViewReveal`] ticked from `agent_view_tick`, one step at
//! a time, each with a 6 s deadline, the user's first key canceling it
//! (`Refused("user took over")`):
//!
//! - a diff of a commit: page the commit into the Log and select it, wait
//!   for its file list, open the diff by *path*, wait for the draw to say
//!   the rows are the real diff (`DiffViewState::ready`), find the row that
//!   shows the line (switching to the whole-file layout once when the line
//!   is outside every hunk), put the cursor and scroll there, tint it;
//! - a diff of the working changes: select the file in the Status pane
//!   (`show_wip`), then the same;
//! - a file: ask for its content / blame / history, wait for the answer
//!   (`FileInsight::accept` discards a stale one), select the row, scroll
//!   horizontally to the column, tint the range.
//!
//! View-only: nothing stages, edits or checks out; the layout it picks is
//! this view's, never the saved preference.

use std::path::PathBuf;

use crate::diff_view::{self, DiffHighlight, DiffViewMode};
use crate::event_loop::*;
use crate::file_view::FileInsightData;
use crate::ui_viewer::{CommitGoal, HighlightReveal, Reveal, ShowReveal, ViewReveal, REVEAL_POLL};
use gitpoulpi_ipc::ui::{
    DiffAt, DiffLayout, DiffSide, FileAt, FileMode, LineRange, UiIntent, UiOrigin, UiOutcome,
};
use init::WindowState;

use super::agent_view::{en, Applied};

/// How long one step may take.
const STEP_DEADLINE: Duration = Duration::from_secs(6);
/// Columns of left context kept when scrolling to a column.
const COLUMN_CONTEXT: usize = 8;

fn not_found(what: impl Into<String>) -> Applied {
    Applied::Not(Box::new(UiOutcome::NotFound { what: what.into() }))
}

fn refused(why: impl Into<String>) -> Applied {
    Applied::Not(Box::new(UiOutcome::refused(why)))
}

/// A plain agent reason, in English.
fn reason(m: gitpoulpi_i18n::M) -> String {
    en(|| m.get().to_string())
}

/// The side's index in `DiffRow::numbers`.
fn side_index(side: DiffSide) -> usize {
    match side {
        DiffSide::Old => 0,
        DiffSide::New => 1,
    }
}

/// The horizontal scroll that brings (1-based) `column` into view with a
/// little context to its left; `0` while it fits without scrolling.
pub(crate) fn h_offset_for(column: usize) -> usize {
    if column <= COLUMN_CONTEXT + 2 {
        0
    } else {
        column - COLUMN_CONTEXT
    }
}

/// The first scrolled row that shows `row` a third of the way down.
pub(crate) fn scroll_for(row: usize, height: usize) -> usize {
    row.saturating_sub(height / 3)
}

impl WindowState {
    /// Applies `ShowDiff` or `ShowFile` (the window already is on the
    /// intent's repository).
    pub(super) fn apply_look(
        &mut self,
        id: Option<u64>,
        intent: &UiIntent,
        origin: &UiOrigin,
    ) -> Applied {
        let what = ui_intent::describe(intent);
        let now = Instant::now();
        let hold = std::env::var("GITPOULPI_UI_REVEAL_HOLD_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(|ms| now + Duration::from_millis(ms));
        let mut view = Box::new(ViewReveal {
            id,
            origin: origin.clone(),
            what,
            goal: intent.clone(),
            select: None,
            note: None,
            opened: false,
            widened: false,
            pushed: false,
            started: now,
            until: now + STEP_DEADLINE,
            poll_at: now + REVEAL_POLL,
            hold_until: hold,
        });
        // Whatever was open gives way; the Log is where a commit pages in.
        self.diff_view_state = None;
        self.file_insight = None;
        self.central_view = CentralView::Log;
        if let UiIntent::ShowDiff { target, path, .. } = intent {
            match target {
                DiffAt::Commit(oid) => {
                    view.select = Some(Box::new(ShowReveal {
                        id: None,
                        origin: origin.clone(),
                        what: String::new(),
                        goal: CommitGoal {
                            oid: *oid,
                            also: Vec::new(),
                            files: false,
                            focus: None,
                        },
                        note: None,
                        moved: false,
                        pushed: false,
                        started: now,
                        until: now + STEP_DEADLINE,
                        poll_at: now + REVEAL_POLL,
                        pages: 0,
                    }));
                }
                DiffAt::Unstaged | DiffAt::Staged => {
                    match self.show_wip(Some(path), *target == DiffAt::Staged, false) {
                        Applied::Done => {}
                        other => return other,
                    }
                }
            }
        }
        self.step_view(view)
    }

    /// One look at where the chain stands.
    pub(super) fn step_view(&mut self, mut view: Box<ViewReveal>) -> Applied {
        if view.hold_until.is_some_and(|at| Instant::now() < at) {
            return Applied::Viewing(view);
        }
        if let Some(select) = view.select.take() {
            match self.step_show(select) {
                Applied::Waiting(select) => {
                    view.select = Some(select);
                    return Applied::Viewing(view);
                }
                Applied::Done => {}
                Applied::Partial(note) => view.note = Some(note),
                other => return other,
            }
            view.until = Instant::now() + STEP_DEADLINE;
        }
        let goal = view.goal.clone();
        let step = match &goal {
            UiIntent::ShowDiff { .. } => self.step_diff(&mut view, &goal),
            UiIntent::ShowFile { .. } => self.step_file(&mut view, &goal),
            _ => Some(refused(reason(
                gitpoulpi_i18n::msg::agent_reason::NOT_SUPPORTED,
            ))),
        };
        match step {
            None => Applied::Viewing(view),
            Some(Applied::Done) => match view.note.take() {
                Some(note) => Applied::Partial(note),
                None => Applied::Done,
            },
            Some(other) => other,
        }
    }

    /// Ticks a pending diff / file reveal.
    pub(super) fn tick_view_reveal(&mut self, view: Box<ViewReveal>) {
        let (id, origin, what, pushed) =
            (view.id, view.origin.clone(), view.what.clone(), view.pushed);
        let result = if self.ui_viewer.input_since(view.started) {
            // The user always wins.
            refused(reason(gitpoulpi_i18n::msg::agent_reason::USER_TOOK_OVER))
        } else if Instant::now() >= view.until {
            let why = reason(gitpoulpi_i18n::msg::agent_reason::VIEW_TIMEOUT);
            if view.opened {
                Applied::Partial(why)
            } else {
                refused(why)
            }
        } else {
            let mut view = view;
            view.poll_at = Instant::now() + REVEAL_POLL;
            self.step_view(view)
        };
        self.conclude_agent_intent(id, &origin, &what, pushed, result);
    }

    // -- a diff --------------------------------------------------------------

    /// `None`: still waiting for the next thing to land.
    fn step_diff(&mut self, view: &mut ViewReveal, goal: &UiIntent) -> Option<Applied> {
        let UiIntent::ShowDiff {
            target,
            path,
            line,
            side,
            layout,
            whole_file,
            highlight,
            ..
        } = goal
        else {
            return None;
        };
        let wanted_line = line.or(highlight.map(|h| h.start_line));
        let sidx = side_index(*side);
        if !view.opened {
            let file = PathBuf::from(path);
            let mut state = match target {
                DiffAt::Commit(oid) => {
                    // The commit is selected: its file list is what the
                    // diff is opened from, by path.
                    let outcome = self.commit_files.as_ref().filter(|o| o.commit == *oid)?;
                    let Ok(files) = &outcome.files else {
                        return Some(not_found(reason(
                            gitpoulpi_i18n::msg::agent_reason::COMMIT_FILES_UNLISTED,
                        )));
                    };
                    let Some(index) = files.iter().position(|f| f.path == file) else {
                        return Some(not_found(en(|| {
                            gitpoulpi_i18n::msg::agent_reason::not_changed_in_commit(path)
                        })));
                    };
                    DiffViewState::for_commit(*oid, index, file).resume_last_used()
                }
                DiffAt::Unstaged => {
                    DiffViewState::for_working(DiffSource::Unstaged, file).resume_last_used()
                }
                DiffAt::Staged => {
                    DiffViewState::for_working(DiffSource::Staged, file).resume_last_used()
                }
            };
            // This view's layout, never the saved preference.
            match layout {
                Some(DiffLayout::Split) => state.mode = DiffViewMode::Split,
                Some(DiffLayout::Inline) => state.mode = DiffViewMode::Inline,
                None => {}
            }
            if let Some(whole) = whole_file {
                state.hunks_only = !whole;
            }
            state.highlight = highlight
                .map(|r| (r.start_line, r.end_line))
                .or(line.map(|l| (l, l)))
                .map(|(from, to)| DiffHighlight {
                    side: sidx,
                    from,
                    to: to.max(from),
                });
            self.file_insight = None;
            self.central_view = CentralView::Diff;
            self.diff_view_state = Some(state);
            view.opened = true;
            view.until = Instant::now() + STEP_DEADLINE;
            crate::redraw::poke();
            return None;
        }
        let Some(state) = self.diff_view_state.as_mut() else {
            return Some(refused(reason(
                gitpoulpi_i18n::msg::agent_reason::DIFF_CLOSED,
            )));
        };
        if !state.ready {
            return None;
        }
        let render = self.diff_render.clone()?;
        let Some(wanted) = wanted_line else {
            return Some(Applied::Done);
        };
        match diff_view::row_of_line(&render.rows, sidx, wanted) {
            Some(row) => {
                state.cursor = row;
                state.selection_anchor = None;
                state.h_offset = 0;
                state.scroll =
                    scroll_for(row, self.diff_body_area.height.saturating_sub(2) as usize);
                crate::redraw::poke();
                Some(Applied::Done)
            }
            None if state.hunks_only && whole_file.is_none() => {
                // Outside every hunk: the whole file has it.
                state.hunks_only = false;
                state.ready = false;
                view.widened = true;
                view.until = Instant::now() + STEP_DEADLINE;
                crate::redraw::poke();
                None
            }
            None if state.hunks_only => Some(Applied::Partial(en(|| {
                gitpoulpi_i18n::msg::agent_reason::line_not_in_hunk(wanted)
            }))),
            None => Some(Applied::Partial(en(|| {
                gitpoulpi_i18n::msg::agent_reason::line_not_in_side(
                    wanted,
                    if sidx == 0 {
                        gitpoulpi_i18n::msg::agent_reason::SIDE_OLD
                    } else {
                        gitpoulpi_i18n::msg::agent_reason::SIDE_NEW
                    },
                )
            }))),
        }
    }

    /// Ticks a `highlight` that widened the diff: once the whole-file rows
    /// are drawn, scroll to the line.
    pub(super) fn tick_highlight_reveal(&mut self, reveal: Box<HighlightReveal>) {
        let (id, origin, what, pushed) = (
            reveal.id,
            reveal.origin.clone(),
            reveal.what.clone(),
            reveal.pushed,
        );
        let height = self.diff_body_area.height.saturating_sub(2) as usize;
        let result = if self.ui_viewer.input_since(reveal.started) {
            Applied::Partial(reason(gitpoulpi_i18n::msg::agent_reason::USER_TOOK_OVER))
        } else if Instant::now() >= reveal.until {
            Applied::Partial(reason(gitpoulpi_i18n::msg::agent_reason::VIEW_TIMEOUT))
        } else if let Some(ready) = self.diff_view_state.as_ref().map(|s| s.ready) {
            let render = self.diff_render.clone().filter(|_| ready);
            let Some(render) = render else {
                let mut reveal = reveal;
                reveal.poll_at = Instant::now() + REVEAL_POLL;
                self.ui_viewer.set_reveal(Some(Reveal::Highlight(reveal)));
                return;
            };
            match diff_view::row_of_line(&render.rows, reveal.side, reveal.from) {
                Some(row) => {
                    if let Some(state) = self.diff_view_state.as_mut() {
                        state.cursor = row;
                        state.selection_anchor = None;
                        state.scroll = scroll_for(row, height);
                    }
                    crate::redraw::poke();
                    Applied::Done
                }
                None => Applied::Partial(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::line_not_in_side(
                        reveal.from,
                        if reveal.side == 0 {
                            gitpoulpi_i18n::msg::agent_reason::SIDE_OLD
                        } else {
                            gitpoulpi_i18n::msg::agent_reason::SIDE_NEW
                        },
                    )
                })),
            }
        } else {
            Applied::Partial(reason(gitpoulpi_i18n::msg::agent_reason::DIFF_CLOSED))
        };
        self.conclude_agent_intent(id, &origin, &what, pushed, result);
    }

    // -- a file --------------------------------------------------------------

    fn step_file(&mut self, view: &mut ViewReveal, goal: &UiIntent) -> Option<Applied> {
        let UiIntent::ShowFile {
            path,
            at,
            mode,
            line,
            column,
            highlight,
            ..
        } = goal
        else {
            return None;
        };
        if *mode == FileMode::History {
            return self.step_history(view, path, at);
        }
        if !view.opened {
            let head = head_oid(self.refs_result.as_deref().unwrap_or(&[]));
            let zero = Oid([0u8; 20]);
            let (oid, worktree, staged) = match at {
                FileAt::Rev(oid) => (*oid, false, false),
                FileAt::Worktree => (head.unwrap_or(zero), true, false),
                FileAt::Index => (head.unwrap_or(zero), true, true),
            };
            let request = FileInsightRequest {
                path: PathBuf::from(path),
                at: oid,
                kind: match mode {
                    FileMode::Content => FileInsightKind::Content,
                    FileMode::Blame => FileInsightKind::Blame,
                    FileMode::History => FileInsightKind::History,
                },
                worktree,
                staged,
            };
            self.link.request_file_insight(request.clone());
            self.diff_view_state = None;
            self.file_insight = Some(FileInsight::loading(request));
            self.central_view = CentralView::File;
            view.opened = true;
            view.until = Instant::now() + STEP_DEADLINE;
            crate::redraw::poke();
            return None;
        }
        let viewport = (self.file_insight_area.height.saturating_sub(2) as usize).max(1);
        let Some(insight) = self.file_insight.as_mut() else {
            return Some(refused(reason(
                gitpoulpi_i18n::msg::agent_reason::FILE_VIEW_CLOSED,
            )));
        };
        match &insight.data {
            FileInsightData::Loading => return None,
            FileInsightData::Failed(why) => return Some(not_found(why.clone())),
            _ => {}
        }
        let len = insight.len();
        let mut note = None;
        let row = line.or(highlight.map(|h| h.start_line)).map(|l| {
            let row = (l as usize).saturating_sub(1);
            if row >= len {
                note = Some(en(|| {
                    gitpoulpi_i18n::msg::agent_reason::line_past_end(len as u64, l)
                }));
            }
            row.min(len.saturating_sub(1))
        });
        if let Some(row) = row {
            insight.selected = row;
            insight.offset = scroll_for(row, viewport);
            insight.clamp(viewport);
        }
        if *mode != FileMode::History {
            if let Some(column) = column {
                let max = insight.longest_line().saturating_sub(1);
                insight.h_offset = h_offset_for(*column as usize).min(max);
            }
            insight.highlight = highlight
                .map(|r| (r.start_line, r.end_line))
                .or(line.map(|l| (l, l)))
                .filter(|_| len > 0)
                .map(|(a, b)| {
                    let last = len - 1;
                    (
                        (a as usize).saturating_sub(1).min(last),
                        (b.max(a) as usize).saturating_sub(1).min(last),
                    )
                });
        }
        crate::redraw::poke();
        if let Some(note) = note {
            view.note = Some(note);
        }
        Some(Applied::Done)
    }

    /// `ui show file ... history`: the History split, with the asked commit's
    /// row selected once it has loaded (paging as far back as it takes).
    fn step_history(&mut self, view: &mut ViewReveal, path: &str, at: &FileAt) -> Option<Applied> {
        if !view.opened {
            let head = head_oid(self.refs_result.as_deref().unwrap_or(&[]));
            let zero = Oid([0u8; 20]);
            // The whole walk from HEAD, with the asked commit selected in it,
            // rather than only what lies before that commit.
            let (worktree, staged) = match at {
                FileAt::Rev(_) | FileAt::Worktree => (matches!(at, FileAt::Worktree), false),
                FileAt::Index => (true, true),
            };
            let request = FileInsightRequest {
                path: PathBuf::from(path),
                at: head.unwrap_or(zero),
                kind: FileInsightKind::History,
                worktree,
                staged,
            };
            self.history = None;
            self.diff_view_state = None;
            self.file_insight = Some(FileInsight::loading(request));
            self.central_view = CentralView::File;
            view.opened = true;
            view.until = Instant::now() + STEP_DEADLINE;
            crate::redraw::poke();
            return None;
        }
        let Some(history) = self.history.as_mut() else {
            // Not promoted yet, or closed since.
            return self
                .file_insight
                .is_none()
                .then(|| refused(reason(gitpoulpi_i18n::msg::agent_reason::FILE_VIEW_CLOSED)));
        };
        if let Some(why) = &history.error {
            return Some(not_found(why.clone()));
        }
        if !history.first_loaded {
            return None;
        }
        if let FileAt::Rev(oid) = at {
            match history.entries.iter().position(|e| e.commit == *oid) {
                Some(row) => history.set_cursor(row, false),
                None if history.more => {
                    history.want = Some(*oid);
                    return None;
                }
                None => {
                    view.note = Some(reason(
                        gitpoulpi_i18n::msg::agent_reason::COMMIT_NOT_TOUCH_PATH,
                    ));
                }
            }
        }
        crate::redraw::poke();
        Some(Applied::Done)
    }

    // -- the highlight on what is open ----------------------------------------

    pub(super) fn apply_highlight(
        &mut self,
        id: Option<u64>,
        origin: &UiOrigin,
        range: LineRange,
        scroll: bool,
    ) -> Applied {
        let (from, to) = (range.start_line, range.end_line.max(range.start_line));
        match self.central_view {
            CentralView::Diff if self.diff_view_state.is_some() => {
                let height = self.diff_body_area.height.saturating_sub(2) as usize;
                let Some(state) = self.diff_view_state.as_mut() else {
                    return refused("no_text_view");
                };
                let side = state.highlight.map_or(1, |h| h.side);
                state.highlight = Some(DiffHighlight { side, from, to });
                if !scroll {
                    return Applied::Done;
                }
                let found = self
                    .diff_render
                    .as_ref()
                    .and_then(|r| diff_view::row_of_line(&r.rows, side, from));
                match found {
                    Some(row) => {
                        state.cursor = row;
                        state.selection_anchor = None;
                        state.scroll = scroll_for(row, height);
                        Applied::Done
                    }
                    None if state.hunks_only && self.diff_render.is_some() => {
                        // Outside every hunk: the whole file has it, as for
                        // `ShowDiff`; the draw says when its rows are there.
                        state.hunks_only = false;
                        state.ready = false;
                        let now = Instant::now();
                        crate::redraw::poke();
                        Applied::Widening(Box::new(HighlightReveal {
                            id,
                            origin: origin.clone(),
                            what: "a highlight".into(),
                            side,
                            from,
                            pushed: false,
                            started: now,
                            until: now + STEP_DEADLINE,
                            poll_at: now + REVEAL_POLL,
                        }))
                    }
                    None => Applied::Partial(en(|| {
                        gitpoulpi_i18n::msg::agent_reason::line_not_in_side(
                            from,
                            if side == 0 {
                                gitpoulpi_i18n::msg::agent_reason::SIDE_OLD
                            } else {
                                gitpoulpi_i18n::msg::agent_reason::SIDE_NEW
                            },
                        )
                    })),
                }
            }
            CentralView::File if self.file_insight.is_some() => {
                let viewport = (self.file_insight_area.height.saturating_sub(2) as usize).max(1);
                let Some(insight) = self.file_insight.as_mut() else {
                    return refused("no_text_view");
                };
                let len = insight.len();
                let text = matches!(
                    insight.data,
                    FileInsightData::Content(_) | FileInsightData::Blame(_)
                );
                if !text || len == 0 {
                    return refused(reason(
                        gitpoulpi_i18n::msg::agent_reason::NO_TEXT_VIEW_LINES,
                    ));
                }
                let last = len - 1;
                let (a, b) = (
                    (from as usize).saturating_sub(1).min(last),
                    (to as usize).saturating_sub(1).min(last),
                );
                insight.highlight = Some((a, b));
                if scroll {
                    insight.selected = a;
                    insight.offset = scroll_for(a, viewport);
                    insight.clamp(viewport);
                    if let Some(column) = range.start_col {
                        let max = insight.longest_line().saturating_sub(1);
                        insight.h_offset = h_offset_for(column as usize).min(max);
                    }
                }
                Applied::Done
            }
            _ => refused(reason(gitpoulpi_i18n::msg::agent_reason::NO_TEXT_VIEW_OPEN)),
        }
    }
}

// -- `ui state` -----------------------------------------------------------

/// The open diff, for `ui state` (free, over the fields it reads, so the
/// render loop can call it while the terminal's buffer is borrowed).
pub(in crate::event_loop) fn diff_state_json(
    central_view: CentralView,
    diff_view_state: &Option<DiffViewState>,
    diff_render: &Option<std::rc::Rc<diff_view::DiffRender>>,
    body: Rect,
) -> Option<serde_json::Value> {
    let state = diff_view_state
        .as_ref()
        .filter(|_| central_view == CentralView::Diff)?;
    let target = match state.source {
        DiffSource::Commit => serde_json::json!({ "commit": state.commit.to_hex() }),
        DiffSource::Compare => serde_json::json!({ "compare": format!("{:?}", state.compare) }),
        DiffSource::Unstaged => serde_json::json!("unstaged"),
        DiffSource::Staged => serde_json::json!("staged"),
    };
    let rows = diff_render.as_ref().map(|r| &r.rows[..]).unwrap_or(&[]);
    let height = body.height.saturating_sub(2) as usize;
    let cursor_line = rows
        .get(state.cursor)
        .and_then(|r| r.numbers[1].or(r.numbers[0]));
    let visible = diff_view::line_span(rows, state.scroll, state.scroll + height);
    Some(serde_json::json!({
        "target": target,
        "path": state.path.to_string_lossy().replace('\\', "/"),
        "layout": match state.mode {
            DiffViewMode::Inline => "inline",
            DiffViewMode::Split => "split",
        },
        "whole_file": !state.hunks_only,
        "cursor_line": cursor_line,
        "visible_lines": visible.map(|(a, b)| [a, b]),
        "highlight": state.highlight.map(|h| serde_json::json!({
            "side": if h.side == 0 { "old" } else { "new" },
            "lines": [h.from, h.to],
        })),
    }))
}

/// The open file / blame / history, for `ui state`.
pub(in crate::event_loop) fn file_state_json(
    central_view: CentralView,
    file_insight: &Option<FileInsight>,
    area: Rect,
    history: Option<&crate::history_view::HistoryView>,
) -> Option<serde_json::Value> {
    if let (CentralView::Diff, Some(h)) = (central_view, history) {
        let rows = h.viewport_rows();
        let last_visible = (h.offset + rows).min(h.entries.len());
        return Some(serde_json::json!({
            "mode": "history",
            "path": h.path.to_string_lossy().replace('\\', "/"),
            "at": serde_json::json!({ "rev": h.start.to_hex() }),
            "selected_line": (!h.entries.is_empty()).then_some(h.cursor + 1),
            "visible_lines": (!h.entries.is_empty()).then_some([h.offset + 1, last_visible]),
            "h_offset": 0,
            "highlight": serde_json::Value::Null,
        }));
    }
    let insight = file_insight
        .as_ref()
        .filter(|_| central_view == CentralView::File)?;
    let request = &insight.request;
    let viewport = (area.height.saturating_sub(2) as usize).max(1);
    let len = insight.len();
    let at = if request.worktree {
        if request.staged {
            serde_json::json!("index")
        } else {
            serde_json::json!("worktree")
        }
    } else {
        serde_json::json!({ "rev": request.at.to_hex() })
    };
    let last_visible = (insight.offset + viewport).min(len);
    Some(serde_json::json!({
        "mode": match request.kind {
            FileInsightKind::Blame => "blame",
            FileInsightKind::History => "history",
            _ => "content",
        },
        "path": request.path.to_string_lossy().replace('\\', "/"),
        "at": at,
        "selected_line": (len > 0).then_some(insight.selected + 1),
        "visible_lines": (len > 0).then_some([insight.offset + 1, last_visible]),
        "h_offset": insight.h_offset,
        "highlight": insight.highlight.map(|(a, b)| [a + 1, b + 1]),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_scrolls_with_left_context_and_a_near_one_does_not() {
        assert_eq!(h_offset_for(1), 0);
        assert_eq!(h_offset_for(10), 0);
        assert_eq!(h_offset_for(90), 82);
    }

    #[test]
    fn a_row_lands_a_third_of_the_way_down() {
        assert_eq!(scroll_for(2, 30), 0);
        assert_eq!(scroll_for(120, 30), 110);
    }
}
