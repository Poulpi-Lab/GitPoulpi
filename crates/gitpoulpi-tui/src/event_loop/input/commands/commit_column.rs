//! The commit column's input: its palette commands, the keys while it has
//! the keyboard, the mouse on it. Every mouse action has its twin here:
//! picking a row is `Up` / `Down` (after `Ctrl+N` and its badge, or the
//! palette's Focus command), the splitter is Widen / Narrow, the cross is
//! Toggle.

use crate::commit_column::{
    viewed_commit, ColumnCmd, ColumnHit, ColumnMode, ColumnState, Pref, Side, PART_STEP, STEP,
};
use crate::event_loop::*;
use crate::keymap::term::TermKeymap;
use crate::keymap::Action as Bind;
use gitpoulpi_i18n::msg::commit_column as msg;

use super::{claim_diff_keys, release_diff_keys, CommandCtx, Flow};

/// What one key or wheel notch moves the rows sideways, in cells (the diff
/// view's step).
const SCROLL_STEP: isize = 4;

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: ColumnCmd) -> bool {
    match cmd {
        ColumnCmd::Toggle => toggle(cx),
        ColumnCmd::Set(state) => set_state(cx, state),
        ColumnCmd::Focus => focus(cx),
        ColumnCmd::Widen => resize(cx, i32::from(STEP)),
        ColumnCmd::Narrow => resize(cx, -i32::from(STEP)),
        ColumnCmd::ScrollLeft => scroll(cx, -SCROLL_STEP),
        ColumnCmd::ScrollRight => scroll(cx, SCROLL_STEP),
        ColumnCmd::NextPart => next_part(cx),
        ColumnCmd::MovePart(side) => move_part(cx, side),
        ColumnCmd::PartWider => resize_part(cx, i32::from(PART_STEP)),
        ColumnCmd::PartNarrower => resize_part(cx, -i32::from(PART_STEP)),
        ColumnCmd::ResetWidths => reset_widths(cx),
        ColumnCmd::Fit => {
            let part = cx.commit_col.part;
            fit_one(cx, part);
        }
        ColumnCmd::FitAll => fit_every(cx),
        ColumnCmd::ToggleShown(part) => toggle_shown(cx, part),
        ColumnCmd::ColumnsMenu => open_columns_menu(cx, None, None),
        ColumnCmd::Menu => open_menu_on_selection(cx, &std::collections::HashSet::new()),
        ColumnCmd::Show(op) => {
            match selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order) {
                Some(commit) => run_op(cx, op, commit),
                None => {
                    *cx.last_message = Some(
                        gitpoulpi_i18n::msg::input::SELECT_COMMIT_NOT_WIP
                            .get()
                            .into(),
                    )
                }
            }
        }
    }
    true
}

fn terminal_width() -> u16 {
    crossterm::terminal::size().map_or(0, |(w, _)| w)
}

fn save(cx: &mut CommandCtx<'_>) {
    session_store::save_commit_column(
        cx.session,
        session_store::CommitColumnSave {
            pref: cx.commit_col.pref.saved(),
            cols: cx.commit_col.cols,
            mode: cx.commit_col.mode.saved(),
        },
    );
    // The parts' choices are global settings, in `config.toml` beside the
    // Log's own columns.
    if let Some(path) = cx.diff_config_path {
        crate::commit_column::ColumnPrefs {
            visible: cx.commit_col.visible_names(),
            order: cx.commit_col.order_names(),
            widths: cx.commit_col.widths,
        }
        .save_in_background(path);
    }
}

/// Says why the column is not there, when it is not.
fn why_not_shown(cx: &CommandCtx<'_>) -> &'static str {
    let of_a_commit = viewed_commit(
        *cx.central_view,
        cx.diff_view_state.as_ref(),
        cx.file_insight.as_ref(),
    )
    .is_some();
    if of_a_commit && cx.commit_col.split(cx.commit_col.column).is_none() {
        msg::TOO_NARROW.get()
    } else {
        msg::NOT_OPEN.get()
    }
}

/// The state the column is in now: the File History's list while a History
/// is open (hidden by its cross: off), else the Log's commits when the
/// column is drawn, else off.
pub(in crate::event_loop) fn current_state(cx: &CommandCtx<'_>) -> ColumnState {
    if super::history::is_open(cx) {
        return if cx.history.as_ref().is_some_and(|h| h.hidden) {
            ColumnState::Off
        } else {
            ColumnState::File
        };
    }
    let of_a_commit = viewed_commit(
        *cx.central_view,
        cx.diff_view_state.as_ref(),
        cx.file_insight.as_ref(),
    )
    .is_some();
    if of_a_commit && cx.commit_col.wanted(terminal_width()) {
        ColumnState::Graph
    } else {
        ColumnState::Off
    }
}

/// `Alt+C`, the strip's click: hides the
/// column, or shows it again in the mode it was last in (the File History's
/// list while a History is open, which is that mode).
pub(in crate::event_loop) fn toggle(cx: &mut CommandCtx<'_>) {
    let history = super::history::is_open(cx);
    // What the key flips is what the width and the last choice make of the
    // column, not whether a view happens to be open right now.
    let on = if history {
        current_state(cx) == ColumnState::File
    } else {
        cx.commit_col.wanted(terminal_width())
    };
    let target = match (on, history) {
        (true, _) => ColumnState::Off,
        (false, true) => ColumnState::File,
        (false, false) => cx.commit_col.mode.state(),
    };
    set_state(cx, target);
}

/// A click on a segment of the bar's `File | All` selector: that mode,
/// shown (opening the column when it was closed); on the segment already
/// shown, the column hides and the list is remembered.
pub(in crate::event_loop) fn select(cx: &mut CommandCtx<'_>, mode: ColumnMode) {
    let target = if current_state(cx) == mode.state() {
        ColumnState::Off
    } else {
        mode.state()
    };
    set_state(cx, target);
}

/// Puts the column in `target` and remembers the choice. Off is one choice
/// for both lists (the commit column's saved preference, and the open
/// History's list) and leaves the mode alone; File and Graph are the two
/// modes, remembered (the File History itself is opened per file, so only the
/// mode is saved, not the list); Graph is also the saved preference "on".
pub(in crate::event_loop) fn set_state(cx: &mut CommandCtx<'_>, target: ColumnState) {
    let now = current_state(cx);
    match target {
        ColumnState::Off => {
            cx.commit_col.pref = Pref::Off;
            cx.commit_col.keys = false;
            super::history::hide_list(cx);
            claim_diff_keys(cx);
            save(cx);
        }
        ColumnState::File => {
            if !super::history::show_list(cx) {
                return;
            }
            cx.commit_col.mode = ColumnMode::File;
            save(cx);
        }
        ColumnState::Graph => {
            if super::history::is_open(cx) && !super::history::leave_for_graph(cx) {
                *cx.last_message = Some(gitpoulpi_i18n::msg::history_view::NO_ROW.get().into());
                return;
            }
            // Remembered even with nothing to show it beside (yet).
            if !super::history::is_open(cx)
                && viewed_commit(
                    *cx.central_view,
                    cx.diff_view_state.as_ref(),
                    cx.file_insight.as_ref(),
                )
                .is_none()
            {
                cx.commit_col.pref = Pref::On;
                cx.commit_col.mode = ColumnMode::All;
                save(cx);
                *cx.last_message = Some(format!(
                    "{} {}",
                    msg::state_changed(target.label()),
                    msg::NOT_OPEN.get()
                ));
                return;
            }
            cx.commit_col.pref = Pref::On;
            cx.commit_col.mode = ColumnMode::All;
            save(cx);
        }
    }
    if now == target {
        return;
    }
    let mut text = msg::state_changed(target.label());
    if target == ColumnState::Graph && cx.commit_col.split(cx.commit_col.column).is_none() {
        text = format!("{text} {}", msg::TOO_NARROW.get());
    }
    *cx.last_message = Some(text);
}

/// Gives the keyboard to the column: the arrows then pick a commit.
pub(in crate::event_loop) fn focus(cx: &mut CommandCtx<'_>) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    *cx.focus = Focus::Log;
    *cx.right_panel = RightPanel::CommitDetail;
    *cx.commit_file_focus = false;
    cx.commit_col.keys = true;
    release_diff_keys(cx);
    *cx.last_message = Some(msg::FOCUSED.get().into());
}

fn resize(cx: &mut CommandCtx<'_>, delta: i32) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    let now = i32::from(cx.commit_col.current_cols());
    cx.commit_col.set_cols((now + delta).max(0) as u16);
    save(cx);
    *cx.last_message = Some(msg::width(u64::from(cx.commit_col.cols)));
}

/// Scrolls the rows sideways (the key and the palette's twins of the wheel).
fn scroll(cx: &mut CommandCtx<'_>, delta: isize) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    cx.commit_col.scroll_h(delta);
    if cx.commit_col.max_h_offset() == 0 {
        *cx.last_message = Some(msg::NOTHING_TO_SCROLL.get().into());
    }
}

/// The next part the keys size (lanes, author, date), named in the status.
fn next_part(cx: &mut CommandCtx<'_>) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    cx.commit_col.part = cx.commit_col.part.next_sized(cx.commit_col.shown);
    say_part(cx);
}

/// Moves the part the keys size one place toward `side` among the shown
/// ones, and remembers the order (the keyboard's twin of dragging a header).
fn move_part(cx: &mut CommandCtx<'_>, side: Side) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    let part = if cx.commit_col.is_shown(cx.commit_col.part) {
        cx.commit_col.part
    } else {
        crate::commit_column::Part::Title
    };
    cx.commit_col.part = part;
    *cx.last_message = Some(if cx.commit_col.move_part(part, side) {
        save(cx);
        match side {
            Side::Left => msg::part_moved_left(part.title()),
            Side::Right => msg::part_moved_right(part.title()),
        }
    } else {
        msg::part_at_edge(part.title())
    });
}

fn say_part(cx: &mut CommandCtx<'_>) {
    let part = cx.commit_col.part;
    say_part_of(cx, part);
}

fn say_part_of(cx: &mut CommandCtx<'_>, part: crate::commit_column::Part) {
    let cells = cx.commit_col.part_width(part);
    *cx.last_message = Some(msg::part_width(part.title(), u64::from(cells)));
}

/// Widens or narrows the part the keys size, and remembers it.
fn resize_part(cx: &mut CommandCtx<'_>, delta: i32) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    cx.commit_col.resize_part(delta);
    save(cx);
    say_part(cx);
}

/// Fits one part to its content (a double click on its boundary, `=` on the
/// part the keys size, the header menu's row, the palette) and remembers it.
/// Says what came of it.
pub(in crate::event_loop) fn fit_one(cx: &mut CommandCtx<'_>, part: crate::commit_column::Part) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    cx.commit_col.fit_part(part);
    save(cx);
    say_part_of(cx, part);
}

/// Fits every shown part to its content.
pub(in crate::event_loop) fn fit_every(cx: &mut CommandCtx<'_>) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    cx.commit_col.fit_all();
    save(cx);
    *cx.last_message = Some(
        gitpoulpi_i18n::msg::log_table::COLUMNS_FITTED_ALL
            .get()
            .into(),
    );
}

fn reset_widths(cx: &mut CommandCtx<'_>) {
    cx.commit_col.reset_widths();
    save(cx);
    *cx.last_message = Some(msg::WIDTHS_RESET.get().into());
}

/// Shows or hides one part of the rows and remembers it (the palette's twin
/// of a click in the header menu). The Log's columns are not touched: the
/// column keeps a list of its own.
pub(in crate::event_loop) fn toggle_shown(
    cx: &mut CommandCtx<'_>,
    part: crate::commit_column::Part,
) {
    if !cx.commit_col.toggle_shown(part) {
        *cx.last_message = Some(msg::COLUMN_ALWAYS_SHOWN.get().into());
        return;
    }
    save(cx);
    *cx.last_message = Some(if cx.commit_col.is_shown(part) {
        msg::column_shown(part.title())
    } else {
        msg::column_hidden(part.title())
    });
}

/// Opens the header's parts menu: at the pointer for the right-click
/// (`anchor`), under the header's left end for the keyboard and the palette.
pub(in crate::event_loop) fn open_columns_menu(
    cx: &mut CommandCtx<'_>,
    anchor: Option<(u16, u16)>,
    target: Option<crate::commit_column::Part>,
) {
    if cx.commit_col.area.width == 0 {
        *cx.last_message = Some(why_not_shown(cx).into());
        return;
    }
    let (col, row) = anchor.unwrap_or((cx.commit_col.rows_area.x, cx.commit_col.rows_area.y));
    // The "fit this part" row acts on the part right-clicked, else on the one
    // the keys size.
    let target = target.unwrap_or(cx.commit_col.part);
    *cx.focus = Focus::Log;
    *cx.open_menu = OpenMenu::CommitColumns(
        ContextMenu::new(
            msg::COLUMNS_MENU_TITLE.get(),
            cx.commit_col.columns_menu_entries(Some(target)),
            col,
            row,
        )
        .closable(),
    );
}

/// A part of the parts menu was chosen.
pub(in crate::event_loop) fn run_columns_menu(
    cx: &mut CommandCtx<'_>,
    act: crate::commit_column::ColumnsAct,
) {
    use crate::commit_column::ColumnsAct;
    match act {
        ColumnsAct::Toggle(part) => toggle_shown(cx, part),
        ColumnsAct::Fit(part) => fit_one(cx, part),
        ColumnsAct::FitAll => fit_every(cx),
    }
}

/// The menu of the commit `commit`, anchored at `(col, row)`: what a
/// diff or file view of that commit can do (`log_menu::column_menu_entries`),
/// then the Log's own menu for it. The mouse twin is a right-click on its
/// row; the keys' is the menu key.
pub(in crate::event_loop) fn open_menu_for(
    cx: &mut CommandCtx<'_>,
    commit: Oid,
    col: u16,
    row: u16,
    unpushed_tags: &std::collections::HashSet<String>,
) {
    use crate::context_menu::Entry;
    // Inside the multi-selection the menu acts on all of it; outside, on
    // the commit alone (the Log's rule).
    if !cx.log_multi.contains_id(&commit) {
        cx.log_multi.clear();
    }
    let selected_count = cx.log_multi.newest_first(&*cx.log_result).len();
    let built = commit_menu(
        &MenuSources {
            refs_result: &*cx.refs_result,
            log_result: &*cx.log_result,
            remotes: &cx.remote_ui.remotes,
            hidden: &*cx.hidden_from_graph,
            solo: &*cx.solo_refs,
            unpushed_tags,
            selected_count,
            ai_enabled: cx.ai_ui.enabled(),
            merge_ff: cx.commit_settings.merge_ff,
        },
        commit,
    );
    let mut entries = Vec::new();
    if selected_count <= 1 {
        entries.extend(crate::log_menu::column_menu_entries());
        entries.push(Entry::Separator);
    }
    entries.extend(built.entries);
    *cx.log_menu_refs = built.table;
    *cx.focus = Focus::Log;
    *cx.open_menu = OpenMenu::Log((
        commit,
        ContextMenu::new(&commit.to_hex()[..SHORT_SHA_LEN], entries, col, row),
    ));
}

/// The menu of the selected commit, hanging under its row (the keys' and the
/// palette's way in).
pub(in crate::event_loop) fn open_menu_on_selection(
    cx: &mut CommandCtx<'_>,
    unpushed_tags: &std::collections::HashSet<String>,
) {
    let Some(commit) = selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order) else {
        *cx.last_message = Some(
            gitpoulpi_i18n::msg::input::SELECT_COMMIT_NOT_WIP
                .get()
                .into(),
        );
        return;
    };
    let nodes = cx.log_result.as_ref().ok().map(|g| &g.nodes);
    let at = nodes
        .and_then(|n| n.iter().position(|c| c.id == commit))
        .map_or(0, |i| i.saturating_sub(cx.commit_col.offset));
    let area = cx.commit_col.rows_area;
    let row = area.y + (at as u16).min(area.height.saturating_sub(1));
    open_menu_for(cx, commit, area.x + 2, row, unpushed_tags);
}

/// A row of the column's own part of the menu: the view of that commit's
/// file the menu names, or the Log.
pub(in crate::event_loop) fn run_op(
    cx: &mut CommandCtx<'_>,
    op: crate::log_menu::ColumnOp,
    commit: Oid,
) {
    use crate::commit_column::ViewKind;
    use crate::log_menu::ColumnOp;
    let kind = match op {
        ColumnOp::ShowInLog => {
            cx.reveal.request_commit_select(commit);
            *cx.last_message = None;
            return;
        }
        ColumnOp::Diff => ViewKind::Diff,
        ColumnOp::File => ViewKind::FileAsIs,
        ColumnOp::Blame => ViewKind::Blame,
    };
    // The follow step re-points the open view at the picked commit in that
    // kind of view, once the commit's file list has landed.
    cx.commit_col.force = Some(kind);
    cx.log_multi.clear();
    if select_log_commit(&*cx.log_result, &*cx.log_order, &mut *cx.log_state, commit) {
        *cx.log_follow = true;
        cx.commit_col.follow = true;
        *cx.right_panel = RightPanel::CommitDetail;
        *cx.commit_file_focus = false;
    } else {
        cx.commit_col.force = None;
        *cx.last_message = Some(gitpoulpi_i18n::msg::input::not_in_loaded_log(
            &commit.to_hex()[..8],
        ));
    }
}

/// The commits of the Log, as display rows: how many, and the display
/// index of the first and last one (the WIP and stash rows between are not
/// the column's).
fn commit_rows(cx: &CommandCtx<'_>) -> Option<(usize, usize)> {
    let len = *cx.log_len;
    let first = (0..len).find(|&d| matches!(cx.log_order.entry(d), graph::LogEntry::Commit(_)))?;
    let last = (0..len)
        .rev()
        .find(|&d| matches!(cx.log_order.entry(d), graph::LogEntry::Commit(_)))?;
    Some((first, last))
}

/// Picks the commit on display row `display` (a plain click, the arrows).
fn pick_display(cx: &mut CommandCtx<'_>, display: usize) {
    cx.log_state.select(Some(display));
    *cx.log_follow = true;
    cx.commit_col.follow = true;
    cx.log_multi.clear();
    *cx.right_panel = RightPanel::CommitDetail;
    *cx.focus = Focus::Log;
    *cx.commit_file_focus = false;
}

/// Moves the selection `delta` commits, skipping the WIP and stash rows the
/// Log interleaves: they are not commits of the column.
fn move_by(cx: &mut CommandCtx<'_>, delta: isize) {
    let Some((first, last)) = commit_rows(cx) else {
        return;
    };
    // Nothing selected: the first commit, whichever way it is asked.
    let Some(mut display) = cx.log_state.selected() else {
        pick_display(cx, first);
        return;
    };
    let step: isize = delta.signum();
    let mut left = delta.unsigned_abs();
    while left > 0 {
        let next = display as isize + step;
        if next < first as isize || next > last as isize {
            break;
        }
        display = next as usize;
        if matches!(cx.log_order.entry(display), graph::LogEntry::Commit(_)) {
            left -= 1;
        }
    }
    // Landing on a WIP or stash row (the selection started there and no
    // step reached a commit): the nearest commit from there on.
    let is_commit = |d: usize| matches!(cx.log_order.entry(d), graph::LogEntry::Commit(_));
    if !is_commit(display) {
        display = (display.clamp(first, last)..=last)
            .find(|&d| is_commit(d))
            .unwrap_or(first);
    }
    pick_display(cx, display.clamp(first, last));
}

/// A key while the column has the keyboard. `true` when taken; everything
/// else (the palette, the prefixes, the toggle) goes on to the window.
pub(in crate::event_loop) fn on_key(
    cx: &mut CommandCtx<'_>,
    km: &keymap::Keymap,
    key: KeyEvent,
    unpushed_tags: &std::collections::HashSet<String>,
) -> bool {
    if cx.commit_col.area.width == 0 {
        cx.commit_col.keys = false;
        return false;
    }
    let plain = !key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    if !plain {
        return false;
    }
    // The keys of the sideways scroll and the parts' widths, before the
    // arrows below (Shift+Right is not a step into the file list).
    for (bind, cmd) in [
        (Bind::CommitColumnScrollLeft, ColumnCmd::ScrollLeft),
        (Bind::CommitColumnScrollRight, ColumnCmd::ScrollRight),
        (Bind::CommitColumnNextPart, ColumnCmd::NextPart),
        (Bind::CommitColumnMoveLeft, ColumnCmd::MovePart(Side::Left)),
        (
            Bind::CommitColumnMoveRight,
            ColumnCmd::MovePart(Side::Right),
        ),
        (Bind::CommitColumnWiderPart, ColumnCmd::PartWider),
        (Bind::CommitColumnNarrowerPart, ColumnCmd::PartNarrower),
        (Bind::CommitColumnResetWidths, ColumnCmd::ResetWidths),
        (Bind::CommitColumnFit, ColumnCmd::Fit),
        (Bind::CommitColumnFitAll, ColumnCmd::FitAll),
    ] {
        if km.is(bind, &key) {
            run(cx, cmd);
            return true;
        }
    }
    if km.is(Bind::GlobalContextMenu, &key) {
        open_menu_on_selection(cx, unpushed_tags);
        return true;
    }
    let page = cx.commit_col.viewport().max(1) as isize;
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => move_by(cx, 1),
        KeyCode::Up | KeyCode::Char('k') => move_by(cx, -1),
        KeyCode::PageDown => move_by(cx, page),
        KeyCode::PageUp => move_by(cx, -page),
        KeyCode::Home => move_by(cx, isize::MIN / 2),
        KeyCode::End => move_by(cx, isize::MAX / 2),
        // Deeper, as in the Log: on to the picked commit's file list, where
        // another file of it opens in the view.
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            cx.commit_col.keys = false;
            *cx.focus = Focus::Log;
            *cx.right_panel = RightPanel::CommitDetail;
            release_diff_keys(cx);
            if *cx.commit_files_len > 0 {
                *cx.commit_file_focus = true;
                move_selection(&mut *cx.commit_file_state, *cx.commit_files_len, 0);
                *cx.commit_files_follow = true;
            }
        }
        // Back to the view.
        KeyCode::Esc => {
            cx.commit_col.keys = false;
            claim_diff_keys(cx);
        }
        _ => return false,
    }
    true
}

/// A left press while the column is drawn. `Some(flow)` when taken. Any
/// press outside it gives the keyboard back to the view.
pub(in crate::event_loop) fn on_press(
    cx: &mut CommandCtx<'_>,
    hit: Option<&HitTarget>,
    col: u16,
    row: u16,
) -> Option<Flow> {
    // The strip of a hidden column: the cross's way back (`toggle`'s other
    // half, Alt+C and the palette command being the keyboard twins).
    if let Some(HitTarget::CommitColumn(ColumnHit::Reopen)) = hit {
        toggle_on(cx);
        return Some(Flow::Handled);
    }
    if cx.commit_col.area.width == 0 {
        return None;
    }
    let inside = cx.commit_col.area.contains((col, row).into());
    match hit {
        Some(HitTarget::CommitColumn(ColumnHit::Row(id))) => {
            let id = *id;
            cx.log_multi.clear();
            if select_log_commit(&*cx.log_result, &*cx.log_order, &mut *cx.log_state, id) {
                *cx.log_follow = true;
                cx.commit_col.follow = true;
                *cx.right_panel = RightPanel::CommitDetail;
                *cx.focus = Focus::Log;
                *cx.commit_file_focus = false;
                cx.commit_col.keys = true;
                release_diff_keys(cx);
            }
            Some(Flow::Handled)
        }
        Some(HitTarget::CommitColumn(ColumnHit::Close)) => {
            toggle_off(cx);
            Some(Flow::Handled)
        }
        // A name in the header: dragging it moves the part, as the Log's.
        Some(HitTarget::CommitColumn(ColumnHit::Header(part))) => {
            cx.commit_col.dragging_header = Some(*part);
            Some(Flow::Handled)
        }
        // The splitter and the header's boundaries arm their drag where the
        // resize grab is read.
        Some(HitTarget::CommitColumn(
            ColumnHit::Divider
            | ColumnHit::Boundary(_)
            | ColumnHit::ColumnsMenuItem(_)
            | ColumnHit::ColumnsMenuClose,
        )) => None,
        // The column's own empty cells and border: nothing behind them.
        None if inside => Some(Flow::Handled),
        _ => {
            // A press anywhere else: the keyboard goes back to the view.
            if cx.commit_col.keys {
                cx.commit_col.keys = false;
                claim_diff_keys(cx);
            }
            None
        }
    }
}

/// The strip: show, remembered. The strip is only painted where the column
/// then fits (`draw_central`), so there is nothing to explain.
fn toggle_on(cx: &mut CommandCtx<'_>) {
    set_state(cx, cx.commit_col.mode.state());
}

/// The cross: hide, remembered. (`toggle` flips; the cross only closes.)
fn toggle_off(cx: &mut CommandCtx<'_>) {
    set_state(cx, ColumnState::Off);
}

/// The wheel over the column moves its viewport, three rows a notch.
pub(in crate::event_loop) fn on_wheel(
    cx: &mut CommandCtx<'_>,
    col: u16,
    row: u16,
    delta: isize,
) -> bool {
    if cx.commit_col.area.width == 0 || !cx.commit_col.area.contains((col, row).into()) {
        return false;
    }
    let to = (cx.commit_col.offset as isize + delta * 3).max(0) as usize;
    cx.commit_col.scroll_to(to);
    true
}

/// Shift+wheel and a horizontal wheel over the column move the rows sideways
/// (the diff view's way; `Shift+Left` / `Shift+Right` are the keys').
pub(in crate::event_loop) fn on_hwheel(
    cx: &mut CommandCtx<'_>,
    col: u16,
    row: u16,
    direction: isize,
) -> bool {
    if cx.commit_col.area.width == 0 || !cx.commit_col.area.contains((col, row).into()) {
        return false;
    }
    cx.commit_col.scroll_h(direction * SCROLL_STEP);
    true
}

/// The splitter and the header's boundaries follow the pointer while dragged.
pub(in crate::event_loop) fn drag(cx: &mut CommandCtx<'_>, col: u16, row: u16) {
    if cx.commit_col.dragging {
        let cols = cx.commit_col.cols_at(col);
        cx.commit_col.set_cols(cols);
    }
    if let Some(boundary) = cx.commit_col.dragging_part {
        cx.commit_col.drag_boundary(boundary, col);
    }
    // The ghost and the insertion bar follow the pointer; the part takes the
    // place of the one it is over on release.
    if let Some(dragged) = cx.commit_col.dragging_header {
        cx.commit_col.drag_header(dragged, col, row);
    }
}

/// The drag ended: the width is written once, not per motion event.
pub(in crate::event_loop) fn end_drag(cx: &mut CommandCtx<'_>) {
    if cx.commit_col.dragging {
        cx.commit_col.dragging = false;
        save(cx);
    }
    if cx.commit_col.dragging_part.take().is_some() {
        save(cx);
    }
    if cx.commit_col.dragging_header.is_some() && cx.commit_col.drop_header() {
        save(cx);
    }
}
