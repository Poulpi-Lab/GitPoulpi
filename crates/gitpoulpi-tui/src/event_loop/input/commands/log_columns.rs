//! The Log's columns by keyboard. `Shift+C` (or any of the palette's Log
//! column commands) picks a column; while one is picked `Tab` / `Shift+Tab`
//! change it, `<` / `>` move it, `[` / `]` size it, `0` puts every width back
//! to automatic and `Esc` / `Enter` leave. Each is the twin of a header drag
//! (move) or a boundary drag (size); the layout is saved by the same tick
//! that saves a drag.

use crate::commit_column::Side;
use crate::event_loop::*;
use crate::keymap::term::TermKeymap;
use crate::keymap::Action as Bind;
use crate::log_table::{
    move_column, resize_widths, step_pick, visible_in_order, LogColumnCmd, KEY_STEP,
};
use gitpoulpi_i18n::msg::log_table as msg;

use super::CommandCtx;

fn visible(cx: &CommandCtx<'_>) -> Vec<LogColumn> {
    visible_in_order(&*cx.log_column_order, &*cx.log_column_hidden)
}

fn scrolling() -> bool {
    crate::live_settings::log_columns_layout() == gitpoulpi_config::LogColumnsLayout::Scroll
}

/// Scrolls the columns sideways by `delta` cells. `false` in the `fit` layout,
/// where there is nothing to scroll (the caller tells the user, or lets the
/// event go on): the wheel and the keys share this.
pub(in crate::event_loop) fn scroll_by(cx: &mut CommandCtx<'_>, delta: i32) -> bool {
    if !scrolling() {
        return false;
    }
    *cx.log_h_offset = crate::log_table::scrolled(&cx.log_geo(), delta);
    true
}

/// Puts the picked column in view (the `scroll` layout only).
fn show_picked(cx: &mut CommandCtx<'_>, col: LogColumn) {
    if scrolling() {
        *cx.log_h_offset = crate::log_table::offset_to_show(&cx.log_geo(), col);
    }
}

/// Stores every visible column's current width in the `scroll` layout's map
/// (`log_table::pin_scroll_widths`), so the one column a drag, key or fit
/// changes is the only one that moves. A no-op in the `fit` layout.
pub(in crate::event_loop) fn pin_scroll(cx: &mut CommandCtx<'_>) {
    if scrolling() {
        crate::log_table::pin_scroll_widths(
            *cx.log_area,
            &*cx.log_column_hidden,
            &mut *cx.log_column_scroll_widths,
            &*cx.log_column_overrides,
        );
    }
}

pub(in crate::event_loop) fn run(cx: &mut CommandCtx<'_>, cmd: LogColumnCmd) {
    if *cx.central_view != CentralView::Log || cx.log_area.width == 0 {
        *cx.last_message = Some(msg::COLUMNS_NOT_SHOWN.get().into());
        return;
    }
    *cx.focus = Focus::Log;
    *cx.commit_file_focus = false;
    match cmd {
        LogColumnCmd::Chooser => {
            let target = (*cx.log_col_pick).filter(|c| visible(cx).contains(c));
            *cx.open_menu = OpenMenu::Column(crate::log_table::ColumnMenu::new(target));
            return;
        }
        LogColumnCmd::ScrollLeft | LogColumnCmd::ScrollRight => {
            let step = i32::from(crate::log_table::SCROLL_STEP);
            let delta = if cmd == LogColumnCmd::ScrollLeft {
                -step
            } else {
                step
            };
            if !scroll_by(cx, delta) {
                *cx.last_message = Some(msg::COLUMNS_FIT_PANE.get().into());
            }
            return;
        }
        LogColumnCmd::ToggleLayout => {
            *cx.last_message = Some(crate::log_table::toggle_layout(cx.diff_config_path));
            return;
        }
        _ => {}
    }
    let order = visible(cx);
    let current = (*cx.log_col_pick).filter(|c| order.contains(c));
    let col = match cmd {
        LogColumnCmd::NextColumn => step_pick(&order, current, true),
        LogColumnCmd::PrevColumn => step_pick(&order, current, false),
        _ => current.or_else(|| step_pick(&order, None, true)),
    };
    let Some(col) = col else {
        return;
    };
    *cx.log_col_pick = Some(col);
    *cx.last_message = Some(match cmd {
        LogColumnCmd::Pick | LogColumnCmd::NextColumn | LogColumnCmd::PrevColumn => {
            show_picked(cx, col);
            msg::column_picked(col.title())
        }
        LogColumnCmd::Move(side) => {
            if move_column(&mut *cx.log_column_order, &*cx.log_column_hidden, col, side) {
                show_picked(cx, col);
                match side {
                    Side::Left => msg::column_moved_left(col.title()),
                    Side::Right => msg::column_moved_right(col.title()),
                }
            } else {
                msg::column_at_edge(col.title())
            }
        }
        LogColumnCmd::Wider => resize(cx, col, i32::from(KEY_STEP)),
        LogColumnCmd::Narrower => resize(cx, col, -i32::from(KEY_STEP)),
        LogColumnCmd::ResetWidth => reset_widths(cx),
        LogColumnCmd::FitToContent => fit_one(cx, col),
        LogColumnCmd::FitAllToContent => fit_every(cx),
        LogColumnCmd::Chooser
        | LogColumnCmd::ScrollLeft
        | LogColumnCmd::ScrollRight
        | LogColumnCmd::ToggleLayout => String::new(),
    });
}

/// Every width of the layout in force back to automatic (the other layout's
/// stay).
fn reset_widths(cx: &mut CommandCtx<'_>) -> String {
    if scrolling() {
        cx.log_column_scroll_widths.clear();
    } else {
        cx.log_column_overrides.clear();
    }
    msg::COLUMN_WIDTH_RESET.get().into()
}

/// Runs one row of the chooser: a checkbox toggles and leaves the chooser
/// open, an action runs and closes it (`ChooserRow::closes`).
pub(in crate::event_loop) fn run_chooser_row(
    cx: &mut CommandCtx<'_>,
    row: crate::log_table::ChooserRow,
) {
    use crate::log_table::ChooserRow;
    if !row.actionable(crate::live_settings::log_columns_layout()) {
        return;
    }
    match row {
        ChooserRow::Column(col) => {
            if !cx.log_column_hidden.remove(&col) {
                cx.log_column_hidden.insert(col);
            }
        }
        ChooserRow::Layout => {
            *cx.last_message = Some(crate::log_table::toggle_layout(cx.diff_config_path));
        }
        ChooserRow::FitColumn(col) => *cx.last_message = Some(fit_one(cx, col)),
        ChooserRow::FitAll => *cx.last_message = Some(fit_every(cx)),
        ChooserRow::Reset => *cx.last_message = Some(reset_widths(cx)),
        ChooserRow::Separator => {}
    }
    if row.closes() {
        cx.open_menu.close_column_menu();
    }
}

/// Fits one column to its content (`log_table::fit_column`); the status line
/// it returns says what came of it.
pub(in crate::event_loop) fn fit_one(cx: &mut CommandCtx<'_>, col: LogColumn) -> String {
    let ctx = crate::log_table::FitCtx {
        log_area: *cx.log_area,
        all_order: &*cx.log_column_order,
        hidden: &*cx.log_column_hidden,
        layout: crate::live_settings::log_columns_layout(),
    };
    crate::log_table::fit_column(
        &ctx,
        &mut crate::log_table::FitMaps {
            fit: &mut *cx.log_column_overrides,
            scroll: &mut *cx.log_column_scroll_widths,
        },
        cx.log_natural_widths,
        col,
    )
}

/// Fits every column to its content (`log_table::fit_all`).
pub(in crate::event_loop) fn fit_every(cx: &mut CommandCtx<'_>) -> String {
    let ctx = crate::log_table::FitCtx {
        log_area: *cx.log_area,
        all_order: &*cx.log_column_order,
        hidden: &*cx.log_column_hidden,
        layout: crate::live_settings::log_columns_layout(),
    };
    crate::log_table::fit_all(
        &ctx,
        &mut crate::log_table::FitMaps {
            fit: &mut *cx.log_column_overrides,
            scroll: &mut *cx.log_column_scroll_widths,
        },
        cx.log_natural_widths,
    )
}

fn resize(cx: &mut CommandCtx<'_>, col: LogColumn, delta: i32) -> String {
    if scrolling() {
        // Only this column changes: the others are pinned where they are.
        pin_scroll(cx);
        let width = crate::log_table::scroll_resized(&cx.log_geo(), col, delta);
        cx.log_column_scroll_widths.insert(col, width);
        show_picked(cx, col);
        return msg::column_width(col.title(), u64::from(width));
    }
    for (c, w) in resize_widths(&cx.log_geo().offsets, col, delta) {
        cx.log_column_overrides.insert(c, w);
    }
    msg::column_width(col.title(), u64::from(cx.log_geo().widths.width_of(col)))
}

/// A key while a column is picked. `true` when taken; everything else (the
/// arrows, the palette, the prefixes) goes on to the window. Leaving the Log
/// ends the mode.
pub(in crate::event_loop) fn on_key(
    cx: &mut CommandCtx<'_>,
    km: &keymap::Keymap,
    key: KeyEvent,
) -> bool {
    if *cx.focus != Focus::Log || *cx.central_view != CentralView::Log {
        *cx.log_col_pick = None;
        return false;
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return false;
    }
    for (bind, cmd) in [
        (Bind::LogColumnsNext, LogColumnCmd::NextColumn),
        (Bind::LogColumnsPrev, LogColumnCmd::PrevColumn),
        (Bind::LogColumnsMoveLeft, LogColumnCmd::Move(Side::Left)),
        (Bind::LogColumnsMoveRight, LogColumnCmd::Move(Side::Right)),
        (Bind::LogColumnsWider, LogColumnCmd::Wider),
        (Bind::LogColumnsNarrower, LogColumnCmd::Narrower),
        (Bind::LogColumnsResetWidth, LogColumnCmd::ResetWidth),
        (Bind::LogColumnsFit, LogColumnCmd::FitToContent),
        (Bind::LogColumnsFitAll, LogColumnCmd::FitAllToContent),
        (Bind::LogColumnsLayout, LogColumnCmd::ToggleLayout),
    ] {
        if km.is(bind, &key) {
            run(cx, cmd);
            return true;
        }
    }
    if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
        *cx.log_col_pick = None;
        return true;
    }
    // The header's right-click menu, from the keyboard: opens on the picked
    // column (its "fit" row acts on it).
    if crate::context_menu::is_menu_key(key.code, key.modifiers) {
        run(cx, LogColumnCmd::Chooser);
        return true;
    }
    false
}
