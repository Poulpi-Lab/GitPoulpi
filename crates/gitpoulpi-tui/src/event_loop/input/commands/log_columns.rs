//! The Log's columns by keyboard. `Shift+C` (or any of the palette's Log
//! column commands) picks a column; while one is picked `Tab` / `Shift+Tab`
//! change it, `<` / `>` move it, `[` / `]` size it, `0` puts every width back
//! to automatic and `Esc` / `Enter` leave. Each is the twin of a header drag
//! (move) or a boundary drag (size); the layout is saved by the same tick
//! that saves a drag.

use crate::commit_column::Side;
use crate::event_loop::*;
use crate::keymap::Action as Bind;
use crate::log_table::{
    column_x_offsets, move_column, resize_widths, step_pick, visible_in_order, LogColumnCmd,
    KEY_STEP,
};
use gitpoulpi_i18n::msg::log_table as msg;

use super::CommandCtx;

fn visible(cx: &CommandCtx<'_>) -> Vec<LogColumn> {
    visible_in_order(&*cx.log_column_order, &*cx.log_column_hidden)
}

fn widths(cx: &CommandCtx<'_>) -> LogColumnWidths {
    LogColumnWidths::for_area_width(
        cx.log_area.width,
        &*cx.log_column_hidden,
        &*cx.log_column_overrides,
    )
}

pub(in crate::event_loop) fn run(cx: &mut CommandCtx<'_>, cmd: LogColumnCmd) {
    if *cx.central_view != CentralView::Log || cx.log_area.width == 0 {
        *cx.last_message = Some(msg::COLUMNS_NOT_SHOWN.get().into());
        return;
    }
    *cx.focus = Focus::Log;
    *cx.commit_file_focus = false;
    if cmd == LogColumnCmd::Chooser {
        *cx.open_menu = OpenMenu::Column(crate::log_table::first_hideable());
        return;
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
            msg::column_picked(col.title())
        }
        LogColumnCmd::Move(side) => {
            if move_column(&mut *cx.log_column_order, &*cx.log_column_hidden, col, side) {
                match side {
                    Side::Left => msg::column_moved_left(col.title()),
                    Side::Right => msg::column_moved_right(col.title()),
                }
            } else {
                msg::column_at_edge(col.title())
            }
        }
        LogColumnCmd::Chooser => String::new(),
        LogColumnCmd::Wider => resize(cx, col, i32::from(KEY_STEP)),
        LogColumnCmd::Narrower => resize(cx, col, -i32::from(KEY_STEP)),
        LogColumnCmd::ResetWidth => {
            cx.log_column_overrides.clear();
            msg::COLUMN_WIDTH_RESET.get().into()
        }
    });
}

fn resize(cx: &mut CommandCtx<'_>, col: LogColumn, delta: i32) -> String {
    let offsets = column_x_offsets(cx.log_area.x, &visible(cx), &widths(cx));
    for (c, w) in resize_widths(&offsets, col, delta) {
        cx.log_column_overrides.insert(c, w);
    }
    msg::column_width(col.title(), u64::from(widths(cx).width_of(col)))
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
    false
}
