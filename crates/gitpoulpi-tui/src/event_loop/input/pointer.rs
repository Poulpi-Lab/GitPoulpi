//! The OS pointer over the surfaces that own the mouse outright (Settings,
//! dialogs, the Home body, popups, help...). The ordinary path sets it in
//! `mouse_moved`; those surfaces return before it, so without this their
//! buttons and rows kept whatever shape the pointer had on the way in.
//!
//! Each surface answers "does a click here do something?" from the rects its
//! own paint walk produced (the ones its clicks resolve against); the answer
//! becomes `CursorShape::Pointer`, the rest `Default`. Written on a change
//! only.

use super::commands::CommandCtx;
use crate::event_loop::*;
use ratatui::layout::Position;

/// Sets the pointer shape for a move over a surface that owns the mouse.
pub(super) fn owned_surface(cx: &mut CommandCtx<'_>, col: u16, row: u16) -> io::Result<()> {
    let shape = shape_over_owned_surface(cx, col, row);
    if shape != *cx.last_cursor_shape {
        cursor::emit_cursor_shape(cx.terminal.backend_mut(), shape)
            .map_err(exit_log::io_context("setting the pointer shape"))?;
        *cx.last_cursor_shape = shape;
    }
    Ok(())
}

/// Whether the paint registered a control (a scrollbar, a button) here.
fn registered_hand(col: u16, row: u16) -> bool {
    crate::pointer_map::at(col, row) == Some(crate::pointer_map::Kind::Hand)
}

/// A status bar control that is not a hit target: the jobs indicator, the
/// Activity button and the status line's error, the version chip.
pub(super) fn status_bar_hand(col: u16, row: u16) -> bool {
    crate::jobs_ui::pointer_hand(col, row)
        || crate::activity_log::pointer_hand(col, row)
        || crate::updates::chip::hit(col, row)
}

/// Whether the paint registered a text field here.
pub(super) fn registered_field(col: u16, row: u16) -> bool {
    crate::pointer_map::at(col, row) == Some(crate::pointer_map::Kind::Field)
}

fn hand(yes: bool) -> CursorShape {
    if yes {
        CursorShape::Pointer
    } else {
        CursorShape::Default
    }
}

/// [`hand`], and the I-beam over a text field the paint registered.
fn hand_or_text(yes: bool, col: u16, row: u16) -> CursorShape {
    if let Some(shape) = crate::pointer_map::at(col, row).and_then(|k| k.resize_shape()) {
        return shape;
    }
    match hand(yes) {
        CursorShape::Default if registered_field(col, row) => CursorShape::Text,
        shape => shape,
    }
}

fn shape_over_owned_surface(cx: &mut CommandCtx<'_>, col: u16, row: u16) -> CursorShape {
    let at = Position::new(col, row);
    // Covers the whole screen: nothing behind it counts.
    if crate::about::is_open() {
        return hand(crate::about::pointer_hand(col, row) || registered_hand(col, row));
    }
    if cx.ai_ui.popup_open() {
        return hand_or_text(registered_hand(col, row), col, row);
    }
    if *cx.show_help {
        let control = crate::help_view::on_cross(col, row)
            || crate::help_view::tab_at(col, row).is_some()
            || crate::help_view::on_link(col, row);
        if !control && crate::help_view::pointer_text(col, row) {
            return CursorShape::Text;
        }
        return hand(control);
    }
    if crate::activity_log::is_open() {
        if crate::activity_log::pointer_text(col, row) {
            return CursorShape::Text;
        }
        return hand(crate::activity_log::pointer_hand(col, row));
    }
    if crate::jobs_ui::panel_open() {
        return hand(crate::jobs_ui::pointer_hand(col, row) || registered_hand(col, row));
    }
    if crate::updates::panel::is_open() {
        return hand(crate::updates::panel::pointer_hand(col, row) || registered_hand(col, row));
    }
    if let Some(prefs) = cx.preferences.as_ref() {
        if prefs.pointer_text_at(col, row) {
            return CursorShape::Text;
        }
        return hand_or_text(prefs.pointer_hand_at(col, row), col, row);
    }
    if cx.repo_dialogs.is_open() && !matches!(*cx.input_mode, InputMode::CommandPalette { .. }) {
        return hand_or_text(cx.repo_dialogs.pointer_hand_at(col, row), col, row);
    }
    if cx.repo_menu.is_some() {
        return hand(cx.repo_menu_rects.iter().any(|r| r.contains(at)));
    }
    // The Home body: every control and row, but not the search box.
    if matches!(*cx.input_mode, InputMode::Normal) {
        if let (Some(manager), Some((body, _))) =
            (cx.management.as_ref(), cx.management_panel.as_ref())
        {
            // The Radar sets its own shape (separators, rows, popups) in the
            // prelude; leave it as it is.
            if manager.radar.is_open() && (body.contains(at) || manager.radar.popup_open()) {
                return *cx.last_cursor_shape;
            }
            if manager.notes.is_open() && body.contains(at) {
                return hand_or_text(
                    manager.notes.pointer_hand_at(col, row) || registered_hand(col, row),
                    col,
                    row,
                );
            }
            if body.contains(at) {
                if matches!(manager.hover, Some(repo_manager::ManagerHit::Search)) {
                    return CursorShape::Text;
                }
                return hand(manager.hover.is_some());
            }
        }
    }
    // A modal list (palette, pickers, global search) covers the screen: only
    // what its own paint registered answers (its text field, its chips and
    // cross), never the toolbar or the panes behind it. Its rows light on
    // hover and keep the arrow.
    if matches!(
        &*cx.input_mode,
        InputMode::CommandPalette { .. }
            | InputMode::TabPicker { .. }
            | InputMode::ThemePicker(_)
            | InputMode::LanguagePicker(_)
            | InputMode::AuthorFilter(_)
            | InputMode::WorkspaceSearch(_)
            | InputMode::Guide(_)
    ) {
        return crate::pointer_map::shape_at(col, row);
    }
    // What the paint registered (a combo list, a button, a text field), then
    // the popups and forms that register their rows as hit targets.
    match crate::pointer_map::at(col, row) {
        Some(crate::pointer_map::Kind::Field) => return CursorShape::Text,
        Some(crate::pointer_map::Kind::Hand) => return CursorShape::Pointer,
        Some(crate::pointer_map::Kind::Plain) => return CursorShape::Default,
        Some(kind) => {
            if let Some(shape) = kind.resize_shape() {
                return shape;
            }
        }
        None => {}
    }
    desired_cursor_shape(
        find_hit_target(&*cx.hit_targets, col, row).as_ref(),
        None,
        None,
        None,
        false,
        None,
    )
}
