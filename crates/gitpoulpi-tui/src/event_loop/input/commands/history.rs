//! The File History split view's input: its actions (keys, menu and palette
//! share `run_action`), the keys of the list, and the mouse on it.

use crate::commit_column::ColumnState;
use crate::event_loop::*;
use crate::history_view::{HistoryAction, HistoryHit, HistoryView, Pane, ViewMode, SPLIT_STEP};
use crate::keymap::term::TermKeymap;
use gitpoulpi_core::domain::short;
use gitpoulpi_i18n::msg::history_view as hmsg;

use super::{CommandCtx, Flow};

/// Whether the History view is the open central view: its diff, or (File /
/// Blame mode) the File View panel it re-points at the selected commit.
pub(in crate::event_loop) fn is_open(cx: &CommandCtx<'_>) -> bool {
    let Some(h) = cx.history.as_ref() else {
        return false;
    };
    cx.diff_view_state.as_ref().is_some_and(HistoryView::owns)
        && match *cx.central_view {
            CentralView::Diff => true,
            CentralView::File => cx
                .file_insight
                .as_ref()
                .is_some_and(|f| h.owns_insight(&f.request)),
            _ => false,
        }
}

/// The list's cross: hides it, the keyboard stays on the right side.
pub(in crate::event_loop) fn hide_list(cx: &mut CommandCtx<'_>) {
    if let Some(h) = cx.history.as_mut() {
        h.hidden = true;
        h.pane = Pane::Diff;
        h.dragging_split = false;
    }
}

/// The column's File state: brings the open History's list back, or opens
/// the History of the file the view shows. `false` (with a message) when
/// there is no file to open it for.
pub(in crate::event_loop) fn show_list(cx: &mut CommandCtx<'_>) -> bool {
    if is_open(cx) {
        if let Some(h) = cx.history.as_mut() {
            h.hidden = false;
            h.pane = Pane::List;
        }
        return true;
    }
    let refs = cx.refs_result.as_deref().unwrap_or(&[]);
    let request = match (
        *cx.central_view,
        cx.file_insight.as_ref(),
        cx.diff_view_state.as_ref(),
    ) {
        (CentralView::File, Some(f), _) => Some(FileInsightRequest {
            kind: FileInsightKind::History,
            ..f.request.clone()
        }),
        (CentralView::Diff, _, Some(s)) => {
            Some(diff_insight_request(s, FileInsightKind::History, refs))
        }
        _ => None,
    };
    match request {
        Some(request) => {
            open_insight_request(request, cx.link, &mut *cx.file_insight);
            true
        }
        None => {
            *cx.last_message = Some(hmsg::NO_FILE.get().into());
            false
        }
    }
}

/// The column's Graph state from the File History: the History closes and
/// the view stays on the selected row's commit (its diff, or the file or
/// blame at that commit), which the Log selects so the commit column and the
/// right panel follow. `false` when there is no row.
pub(in crate::event_loop) fn leave_for_graph(cx: &mut CommandCtx<'_>) -> bool {
    let Some(h) = cx.history.as_ref() else {
        return false;
    };
    let Some(entry) = h.current().cloned() else {
        return false;
    };
    let mode = h.mode;
    *cx.history = None;
    *cx.diff_view_state =
        Some(DiffViewState::for_commit(entry.commit, 0, entry.path.clone()).resume_last_used());
    if mode == ViewMode::Diff {
        *cx.file_insight = None;
        *cx.central_view = CentralView::Diff;
    }
    cx.reveal.request_commit_follow(entry.commit);
    true
}

/// Puts the right side in `mode`: the diff, or the File View / Blame of the
/// cursor row's commit, which stay in the split and follow the selection.
pub(in crate::event_loop) fn set_mode(cx: &mut CommandCtx<'_>, mode: ViewMode) {
    let Some(h) = cx.history.as_mut() else {
        return;
    };
    match h.insight_request(mode) {
        None if mode == ViewMode::Diff => {
            h.mode = ViewMode::Diff;
            *cx.file_insight = None;
        }
        None => *cx.last_message = Some(hmsg::NO_ROW.get().into()),
        Some(request) => {
            if cx.file_insight.as_ref().map(|f| &f.request) != Some(&request) {
                cx.link.request_file_insight(request.clone());
                *cx.file_insight = Some(FileInsight::loading(request));
            }
            h.mode = mode;
        }
    }
}

/// Runs one action on the row under the cursor.
pub(in crate::event_loop) fn run_action(cx: &mut CommandCtx<'_>, action: HistoryAction) {
    let Some(h) = cx.history.as_mut() else {
        *cx.last_message = Some(hmsg::NOT_OPEN.get().into());
        return;
    };
    let Some(entry) = h.current().cloned() else {
        // The pane and split actions need no row.
        if !matches!(
            action,
            HistoryAction::SwitchFocus
                | HistoryAction::WidenList
                | HistoryAction::NarrowList
                | HistoryAction::ResetSplit
        ) {
            *cx.last_message = Some(hmsg::NO_ROW.get().into());
            return;
        }
        return run_pane_action(cx, action);
    };
    match action {
        HistoryAction::ShowInLog => {
            cx.reveal.request_commit_select(entry.commit);
            *cx.last_message = None;
        }
        HistoryAction::CopySha => {
            *cx.last_message = Some(
                cx.clipboard
                    .copy(hmsg::WHAT_SHA.get(), &entry.commit.to_hex()),
            );
        }
        HistoryAction::CopySummary => {
            *cx.last_message = Some(cx.clipboard.copy(hmsg::WHAT_SUMMARY.get(), &entry.summary));
        }
        HistoryAction::ShowDiff => set_mode(cx, ViewMode::Diff),
        HistoryAction::OpenFile => set_mode(cx, ViewMode::File),
        HistoryAction::Blame => set_mode(cx, ViewMode::Blame),
        HistoryAction::CompareWorkingTree => {
            let on = h.toggle_worktree();
            *cx.last_message = Some(if on {
                hmsg::comparing_worktree(short(entry.commit))
            } else {
                hmsg::compare_off(short(entry.commit))
            });
        }
        HistoryAction::Restore => {
            let source = HistoryView::read_at(&entry);
            *cx.input_mode = InputMode::Confirm {
                prompt: gitpoulpi_i18n::msg::pane_menu::restore_prompt(
                    entry.path.display(),
                    short(source),
                ),
                action: ConfirmAction::Destructive(PendingDestructive::RestoreFiles {
                    paths: vec![entry.path.clone()],
                    source,
                }),
            };
        }
        HistoryAction::RangeUp => h.move_by(-1, true),
        HistoryAction::RangeDown => h.move_by(1, true),
        HistoryAction::SwitchFocus
        | HistoryAction::WidenList
        | HistoryAction::NarrowList
        | HistoryAction::ResetSplit => run_pane_action(cx, action),
    }
}

/// The actions that need no row: the focus and the splitter.
fn run_pane_action(cx: &mut CommandCtx<'_>, action: HistoryAction) {
    let Some(h) = cx.history.as_mut() else {
        return;
    };
    match action {
        HistoryAction::SwitchFocus => {
            h.pane = match h.pane {
                Pane::List => Pane::Diff,
                // A hidden list cannot take the keyboard.
                Pane::Diff if !h.hidden => Pane::List,
                Pane::Diff => Pane::Diff,
            };
        }
        HistoryAction::WidenList | HistoryAction::NarrowList if h.hidden => {}
        HistoryAction::WidenList | HistoryAction::NarrowList => {
            let step = if action == HistoryAction::WidenList {
                SPLIT_STEP as i32
            } else {
                -(SPLIT_STEP as i32)
            };
            h.set_split((h.split_pct as i32 + step).max(1) as u16);
            session_store::save_history_split(cx.session, h.split_pct);
            *cx.last_message = Some(hmsg::split_width(h.split_pct as u64));
        }
        HistoryAction::ResetSplit => {
            h.set_split(0);
            session_store::save_history_split(cx.session, 0);
            *cx.last_message = Some(hmsg::split_width(h.split_pct as u64));
        }
        _ => {}
    }
}

/// A key while the History view is open. `true` when it was taken; the
/// diff's own keys and the global chords get everything else.
pub(in crate::event_loop) fn on_key(
    cx: &mut CommandCtx<'_>,
    km: &keymap::Keymap,
    key: KeyEvent,
) -> bool {
    if !is_open(cx) || cx.composer_focus.is_some() || *cx.commit_filter_focus {
        return false;
    }
    use HistoryAction as A;
    // Valid in either half.
    for a in [A::SwitchFocus, A::WidenList, A::NarrowList, A::ResetSplit] {
        if km.is(a.bind(), &key) {
            run_action(cx, a);
            return true;
        }
    }
    let Some(pane) = cx.history.as_ref().map(|h| h.pane) else {
        return false;
    };
    // The mode keys also work from the right half, as Alt chords (its own
    // keys are plain letters).
    if key.modifiers.contains(KeyModifiers::ALT) {
        for a in [A::ShowDiff, A::OpenFile, A::Blame] {
            if km.is(a.bind(), &key) {
                run_action(cx, a);
                return true;
            }
        }
    }
    // Esc from a File / Blame side goes back to the diff (the blame view's
    // own popups take their Esc first).
    if key.code == KeyCode::Esc && key.modifiers.is_empty() && leave_insight_mode(cx) {
        return true;
    }
    if pane == Pane::Diff {
        return false;
    }
    // The keymap is lenient about extra modifiers: `Alt+C` would be the
    // row's plain `c` (compare with the working tree), and it is the
    // column's key. Chords of the list are plain or Shift.
    let chorded = key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    for a in A::ALL {
        if !chorded
            && !matches!(
                a,
                A::SwitchFocus | A::WidenList | A::NarrowList | A::ResetSplit
            )
            && km.is(a.bind(), &key)
        {
            run_action(cx, a);
            return true;
        }
    }
    if km.is(keymap::Action::GlobalContextMenu, &key) {
        open_menu_on_cursor(cx);
        return true;
    }
    let plain = !key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
    let viewport = cx
        .history
        .as_ref()
        .map_or(1, |h| h.viewport_rows() as isize)
        .max(1);
    let Some(h) = cx.history.as_mut() else {
        return false;
    };
    match key.code {
        KeyCode::Down | KeyCode::Char('j') if plain => h.move_by(1, false),
        KeyCode::Up | KeyCode::Char('k') if plain => h.move_by(-1, false),
        KeyCode::PageDown if plain => h.move_by(viewport, false),
        KeyCode::PageUp if plain => h.move_by(-viewport, false),
        KeyCode::Home if plain => h.move_by(isize::MIN / 2, false),
        KeyCode::End if plain => h.move_by(isize::MAX / 2, false),
        KeyCode::Char('G') => h.move_by(isize::MAX / 2, false),
        KeyCode::Right | KeyCode::Char('l') if plain => h.pane = Pane::Diff,
        KeyCode::Esc if h.clear_extras() => {}
        _ => return false,
    }
    true
}

/// Esc on a File / Blame side: back to the diff, unless the Blame view has a
/// card, a menu or a prompt open (they close first). `true` when it left.
fn leave_insight_mode(cx: &mut CommandCtx<'_>) -> bool {
    let Some(h) = cx.history.as_ref() else {
        return false;
    };
    if h.mode == ViewMode::Diff {
        return false;
    }
    let busy = cx.file_insight.as_ref().is_some_and(|f| {
        f.blame
            .as_ref()
            .is_some_and(|b| b.menu.is_some() || b.pinned || b.goto.is_some())
    });
    if busy {
        return false;
    }
    set_mode(cx, ViewMode::Diff);
    true
}

/// The row menu, at the cursor row (the keyboard twin of a right-click).
fn open_menu_on_cursor(cx: &mut CommandCtx<'_>) {
    let Some(h) = cx.history.as_ref() else {
        return;
    };
    let row = h.cursor;
    let y = h.row_y(row);
    open_menu(cx, row, h.rows_area.x + 2, y);
}

/// Opens the right-click menu of row `row`.
pub(in crate::event_loop) fn open_menu(cx: &mut CommandCtx<'_>, row: usize, col: u16, y: u16) {
    let Some(h) = cx.history.as_mut() else {
        return;
    };
    if h.row_at(row).is_none() {
        return;
    }
    // A right-click inside the selected range keeps it; elsewhere it moves.
    if !h.in_selection(row) {
        h.set_cursor(row, false);
    }
    let Some(entry) = h.row_at(row) else {
        return;
    };
    let title = entry.commit.to_hex()[..8].to_string();
    let menu = ContextMenu::new(
        title,
        crate::history_view::menu_entries(h.is_range()),
        col,
        y,
    );
    *cx.open_menu = OpenMenu::History((row, menu));
}

/// A left press while the History view is open: which half takes the
/// keyboard, and a click on a row. `Some(flow)` when the press was taken.
pub(in crate::event_loop) fn on_press(
    cx: &mut CommandCtx<'_>,
    hit: Option<&HitTarget>,
    col: u16,
    row: u16,
    shift: bool,
) -> Option<Flow> {
    if !is_open(cx) {
        return None;
    }
    let h = cx.history.as_mut()?;
    let inside_list = h.list_area.contains((col, row).into());
    // The bar spans the whole central column, above the list: a click on one
    // of its buttons (the view tabs, the toggles) is the right side's. The
    // column selector decides for itself (`hide_list` gives the right side
    // the keys).
    let on_bar = match hit {
        Some(HitTarget::DiffBarButton(button)) => !matches!(
            button,
            DiffBarButton::History | DiffBarButton::ColumnFile | DiffBarButton::ColumnAll
        ),
        Some(HitTarget::BarOverflow(_)) => true,
        _ => false,
    };
    if inside_list {
        h.pane = Pane::List;
    } else if h.column.contains((col, row).into()) || on_bar {
        h.pane = Pane::Diff;
    }
    match hit {
        Some(HitTarget::History(HistoryHit::Row(i))) => {
            let i = *i;
            let now = Instant::now();
            let double = cx.last_click.is_some_and(|(t, c, r)| {
                now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
            });
            *cx.last_click = Some((now, col, row));
            h.set_cursor(i, shift);
            if double && !shift {
                run_action(cx, HistoryAction::ShowInLog);
            }
            Some(Flow::Handled)
        }
        Some(HitTarget::History(HistoryHit::Divider)) => None,
        Some(HitTarget::History(HistoryHit::Close)) => {
            super::commit_column::set_state(cx, ColumnState::Off);
            Some(Flow::Handled)
        }
        Some(HitTarget::History(HistoryHit::Reopen)) => {
            super::commit_column::set_state(cx, ColumnState::File);
            Some(Flow::Handled)
        }
        // The list's own empty cells and border: nothing behind them.
        None if inside_list => Some(Flow::Handled),
        _ => None,
    }
}

/// The wheel over the list moves the viewport, three rows a notch.
pub(in crate::event_loop) fn on_wheel(
    cx: &mut CommandCtx<'_>,
    col: u16,
    row: u16,
    delta: isize,
) -> bool {
    if !is_open(cx) {
        return false;
    }
    let Some(h) = cx.history.as_mut() else {
        return false;
    };
    if !h.list_area.contains((col, row).into()) {
        return false;
    }
    let viewport = h.viewport_rows();
    let to = (h.offset as isize + delta * 3).max(0) as usize;
    h.scroll_to(to, viewport);
    true
}
