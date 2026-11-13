//! What runs before a mouse event reaches a pane: the main-window text fields,
//! the surfaces that own the mouse outright, and the list-bar helpers the
//! click, drag and wheel handlers share.

use super::commands::{CommandCtx, Flow};
use super::menu_actions;
use crate::event_loop::*;
use crossterm::event::MouseEvent;

/// The pointer position and the pane it is over, computed once per mouse event.
#[derive(Clone, Copy)]
pub(super) struct Geo {
    pub(super) col: u16,
    pub(super) row: u16,
    pub(super) in_refs: bool,
    pub(super) in_log: bool,
    pub(super) in_diff: bool,
    pub(super) in_status: bool,
    pub(super) in_commit_files: bool,
    pub(super) in_commit_meta: bool,
    pub(super) divider1_x: u16,
    pub(super) divider2_x: u16,
}

/// The mouse half of editing a main-window text field, and pasting into one.
/// Only when nothing modal sits over the fields (`fields_uncovered`) - a click
/// inside a dialog must not move the caret of the field drawn underneath it.
pub(in crate::event_loop) fn main_field_event(
    cx: &mut CommandCtx<'_>,
    event: &Event,
    fields_uncovered: bool,
) -> Flow {
    match event {
        // The remote form's own fields select by dragging too.
        Event::Mouse(m)
            if matches!(
                m.kind,
                MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
            ) && cx
                .remote_ui
                .form
                .as_ref()
                .is_some_and(|f| f.field_dragging()) =>
        {
            if let Some(form) = cx.remote_ui.form.as_mut() {
                form.field_drag(m.kind, m.column, m.row);
            }
            return Flow::Handled;
        }
        // An open sidebar dropdown scrolls with the wheel and its
        // scrollbar (the thumb drags); the rows stay clicks.
        Event::Mouse(m) if cx.hosting.picker_wants_mouse(m) => {
            cx.hosting.picker_mouse(*m);
            return Flow::Handled;
        }
        Event::Mouse(m)
            if m.kind == MouseEventKind::Down(MouseButton::Left) && fields_uncovered =>
        {
            *cx.pending_field_click = cx
                .field_editor
                .field_at(m.column, m.row)
                .filter(|id| {
                    *id == MainField::Palette
                        || !matches!(*cx.input_mode, InputMode::CommandPalette { .. })
                })
                .map(|id| (id, m.column, m.row, Instant::now()));
        }
        Event::Mouse(m)
            if m.kind == MouseEventKind::Drag(MouseButton::Left)
                && cx.field_editor.is_dragging() =>
        {
            if let Some(id) = cx.field_editor.dragging_field() {
                if let Some(text) = id.text(
                    &mut *cx.input_mode,
                    &mut *cx.commit_summary,
                    &mut *cx.commit_description,
                    &mut *cx.commit_files_filter,
                ) {
                    cx.field_editor.mouse_drag(text, m.column, m.row);
                }
            }
            return Flow::Handled;
        }
        Event::Mouse(m) if m.kind == MouseEventKind::Up(MouseButton::Left) => {
            cx.field_editor.mouse_up();
        }
        Event::Paste(pasted) if fields_uncovered => {
            if let Some(id) =
                MainField::active(&*cx.input_mode, *cx.composer_focus, *cx.commit_filter_focus)
            {
                let clean: String = pasted
                    .chars()
                    .filter(|c| !c.is_control() || (*c == '\n' && id.multiline()))
                    .collect();
                if let Some(text) = id.text(
                    &mut *cx.input_mode,
                    &mut *cx.commit_summary,
                    &mut *cx.commit_description,
                    &mut *cx.commit_files_filter,
                ) {
                    cx.field_editor.caret_mut(id).insert(text, &clean);
                }
                main_field_edited(id, &mut *cx.input_mode);
                return Flow::Handled;
            }
        }
        _ => {}
    }
    Flow::Continue
}

/// A press on a repository tab arms a drag (`tab_drag`) instead of focusing at
/// once, so the same press can become a reorder; this is the drag and the
/// release. `true` when the event was consumed. A release that never became a
/// drag is the click: it focuses the tab exactly as the press used to.
fn tab_bar_drag(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> bool {
    use crate::tab_drag::{update_bar_target, DragFrom};
    let (col, row) = (mouse.column, mouse.row);
    let Some(drag) = cx
        .tab_drag
        .as_mut()
        .filter(|d| d.origin_kind == DragFrom::Bar)
    else {
        return false;
    };
    match mouse.kind {
        MouseEventKind::Drag(MouseButton::Left) => {
            drag.at = (col, row);
            if !drag.started && drag.passes_threshold(col, row) {
                drag.started = true;
                drag.source_rect = cx
                    .tab_hits
                    .iter()
                    .find(|h| h.index == drag.from)
                    .map(|h| h.tab);
            }
            if drag.started {
                let vertical = crate::tab_column::area().is_some();
                if !vertical {
                    // Carried onto a scroll arrow, the bar scrolls (and keeps
                    // scrolling while the tab rests there: `tab_scroll`).
                    crate::tab_scroll::drag_over(col, row, std::time::Instant::now());
                }
                // The source tab may have scrolled in or out of view.
                drag.source_rect = cx
                    .tab_hits
                    .iter()
                    .find(|h| h.index == drag.from)
                    .map(|h| h.tab);
                update_bar_target(
                    drag,
                    &*cx.tab_hits,
                    cx.repo_tabs.len(),
                    (col, row),
                    vertical,
                );
            }
            true
        }
        MouseEventKind::Up(MouseButton::Left) => {
            crate::tab_scroll::drag_end();
            let Some(mut drag) = cx.tab_drag.take() else {
                return true;
            };
            // The bar may have scrolled under a pointer that did not move:
            // the drop lands where the last painted frame says it is.
            if drag.started {
                update_bar_target(
                    &mut drag,
                    &*cx.tab_hits,
                    cx.repo_tabs.len(),
                    (col, row),
                    crate::tab_column::area().is_some(),
                );
            }
            if !drag.started {
                if let Some(message) = apply_switcher_action(
                    repo_switcher::SwitcherAction::Focus(drag.from),
                    cx.session,
                    cx.link,
                    cx.repo,
                    &mut *cx.repo_dialogs,
                    &mut *cx.repo_tabs,
                    &mut *cx.active_tab,
                    &mut *cx.switch_to,
                    &mut *cx.close_window,
                ) {
                    *cx.last_message = Some(message);
                }
                *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
            } else if let Some(over) = drag.over {
                *cx.last_message = crate::tabs::reorder_tab(
                    cx.session,
                    &mut *cx.repo_tabs,
                    &mut *cx.active_tab,
                    drag.from,
                    over.to,
                );
            }
            true
        }
        _ => false,
    }
}

/// The surfaces that own the mouse ahead of the panes: the AI popup, help,
/// Preferences, the editor and File View bodies, dialogs, tab bar and the
/// Repository Management tab. `None` when one consumed the event; otherwise
/// the pointer geometry the click handlers share.
pub(super) fn prelude(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Option<Geo>> {
    // "What is sent" before an AI request is modal over everything, the
    // explanation popup included (an Explain shows it while it works).
    if crate::ai_sent_dialog::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        crate::ai_sent_dialog::on_mouse(mouse);
        return Ok(None);
    }
    // "Over the AI limit" is modal the same way.
    if crate::ai_limit_dialog::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        crate::ai_limit_dialog::on_mouse(mouse);
        super::commands::open_limit_setting(cx);
        return Ok(None);
    }
    if tab_bar_drag(cx, mouse) {
        return Ok(None);
    }
    // The integration security check owns the mouse above everything: a click
    // never reaches what it covers, and a click away from it is swallowed.
    if crate::trust_dialog::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(outcome) = crate::trust_dialog::on_mouse(mouse) {
            crate::trust_dialog::apply(
                outcome,
                cx.repo_root,
                &mut *cx.preferences,
                &mut *cx.last_message,
            );
        }
        return Ok(None);
    }
    // The AI popup is modal: the wheel scrolls it, its cross or a click
    // outside closes it (cancelling a running request), a click inside
    // does nothing.
    if cx.ai_ui.popup_open() {
        let closes = cx
            .ai_ui
            .popup_mut()
            .is_some_and(|p| p.on_mouse(mouse) == crate::ai_ui::PopupMouse::Close);
        if closes {
            if cx.ai_ui.popup_mut().is_some_and(|p| p.is_working()) {
                cx.ai_ui.cancel();
            } else {
                cx.ai_ui.close_popup();
            }
        }
        return Ok(None);
    }
    // The help overlay owns the mouse while open, ahead of Preferences and
    // everything below (the same precedence as its key arm); otherwise a
    // wheel notch or click falls through to whatever it covers.
    if *cx.show_help {
        let (col, row) = (mouse.column, mouse.row);
        let scroll = usize::from(*cx.help_scroll);
        let step = crate::popup_list::WHEEL_STEP as u16;
        match mouse.kind {
            MouseEventKind::ScrollUp => *cx.help_scroll = cx.help_scroll.saturating_sub(step),
            MouseEventKind::ScrollDown => *cx.help_scroll = cx.help_scroll.saturating_add(step),
            MouseEventKind::Down(MouseButton::Left) => {
                if crate::help_view::on_cross(col, row) {
                    *cx.show_help = false;
                    cx.help_filter.clear();
                    *cx.help_scroll = 0;
                } else if crate::popup_list::View::bar_press(col, row, scroll) {
                    if let Some(to) = crate::popup_list::View::free_offset() {
                        *cx.help_scroll = to as u16;
                    }
                } else if let Some(tab) = crate::help_view::tab_at(col, row) {
                    crate::help_view::set_tab(tab);
                    *cx.help_scroll = 0;
                } else if crate::help_view::on_link(col, row) {
                    *cx.last_message = Some(crate::help_view::open_docs());
                } else if !crate::help_view::contains(col, row) {
                    // A click away closes it, the mouse twin of `?`.
                    *cx.show_help = false;
                    cx.help_filter.clear();
                    *cx.help_scroll = 0;
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                crate::popup_list::View::bar_drag(row, scroll);
                if let Some(to) = crate::popup_list::View::free_offset() {
                    *cx.help_scroll = to as u16;
                }
            }
            MouseEventKind::Up(MouseButton::Left) => crate::popup_list::View::bar_release(),
            // Nothing behind the popup hovers; its tab headers do.
            MouseEventKind::Moved => crate::help_view::hover(col, row),
            _ => {}
        }
        return Ok(None);
    }
    // A binary file's "Open in" popup is modal the same way.
    if crate::open_with::popup::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some((pick, subject)) = crate::open_with::popup::on_mouse(mouse) {
            *cx.last_message = Some(crate::open_with::popup::apply(
                pick,
                &subject,
                cx.repo_root,
                cx.link,
            ));
        }
        return Ok(None);
    }
    // The Diff view's encoding dropdown is modal the same way.
    if crate::diff_encoding_menu::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some(pick) = crate::diff_encoding_menu::on_mouse(mouse) {
            super::commands::apply_diff_encoding_pick(cx, pick);
        }
        return Ok(None);
    }
    // The image header's "Edit in" popup is modal the same way.
    if crate::image_view::edit_in::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some(pick) = crate::image_view::edit_in::on_mouse(mouse) {
            super::commands::apply_image_edit_pick(cx, pick);
        }
        return Ok(None);
    }
    // The About window: a click closes it, nothing behind it hovers.
    if crate::about::is_open() {
        *cx.hovered_target = None;
        crate::about::on_mouse(mouse);
        return Ok(None);
    }
    // The toolbar's profile popup, modal like "Open in" below.
    if crate::profile_bar::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some(id) = crate::profile_bar::on_mouse(mouse) {
            super::commands::apply_profile_pick(cx, id);
        }
        return Ok(None);
    }
    // The tab bar's workspace combo, modal the same way.
    if crate::workspace_ui::chip::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some(pick) = crate::workspace_ui::chip::on_mouse(mouse) {
            super::commands::apply_workspace_pick(cx, pick);
        }
        return Ok(None);
    }
    // The tab bar's "Open in" popup is modal: it owns the mouse (a click away
    // closes it and is consumed), and nothing behind it hovers.
    if crate::open_in_bar::is_open() {
        if mouse.kind == MouseEventKind::Moved {
            *cx.hovered_target = find_hit_target(&*cx.hit_targets, mouse.column, mouse.row);
            *cx.hovered_tab_index = None;
            *cx.hovered_close_tab = None;
            *cx.hovered_status_row = None;
        }
        if let Some(pick) = crate::open_in_bar::on_mouse(mouse) {
            super::commands::apply_open_in_pick(cx, pick);
        }
        return Ok(None);
    }
    // Copy as Mermaid gitGraph is modal the same way.
    if crate::mermaid_popup::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        let outcome = crate::mermaid_popup::on_mouse(mouse, cx.diff_config_path);
        super::modal::copy_mermaid_outcome(cx, outcome);
        return Ok(None);
    }
    // The predicted conflicts popup is modal the same way.
    if crate::conflict_popup::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(action) = crate::conflict_popup::on_mouse(mouse) {
            crate::conflict_badge::apply_popup_action(
                action,
                &*cx.refs_result,
                cx.link,
                &mut *cx.last_message,
            );
        }
        return Ok(None);
    }
    // The merge-revert parent dropdown is modal the same way.
    if crate::revert_mainline::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(pick) = crate::revert_mainline::on_mouse(mouse) {
            crate::log_menu::apply_revert_mainline(pick, &mut *cx.input_mode);
        }
        return Ok(None);
    }
    // The tag delete scope dialog is modal the same way.
    if crate::tag_delete_dialog::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(pick) = crate::tag_delete_dialog::on_mouse(mouse) {
            crate::tag_delete_dialog::apply(
                pick,
                cx.link,
                &mut *cx.input_mode,
                &mut *cx.last_message,
            );
        }
        return Ok(None);
    }
    // The sensitive-files question of a refused stage is modal the same way.
    if crate::stage_guard_dialog::is_open() {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(staged) = crate::stage_guard_dialog::on_mouse(mouse) {
            crate::stage_guard_dialog::apply(staged, cx.link, cx.repo_root);
        }
        return Ok(None);
    }
    // An editor status menu (encoding, line breaks, indentation) is modal
    // the same way: it owns the mouse, a click away closes it and is
    // consumed, and nothing behind it hovers.
    if *cx.central_view == CentralView::Editor && cx.editor.as_ref().is_some_and(|e| e.menu_open())
    {
        *cx.hovered_target = None;
        *cx.hovered_tab_index = None;
        *cx.hovered_close_tab = None;
        *cx.hovered_status_row = None;
        if let Some(ed) = cx.editor.as_mut() {
            let out = ed.menu_mouse(mouse);
            crate::file_panel::apply_editor_outcome(
                out,
                &mut *cx.editor,
                &mut *cx.file_insight,
                cx.link,
                &mut *cx.last_message,
                cx.diff_config_path,
            );
        }
        return Ok(None);
    }
    let (col, row) = (mouse.column, mouse.row);
    let in_refs = cx.refs_area.contains((col, row).into());
    // `in_log` means "over the central column's *log table*", not merely the
    // central column: every handler behind it (row selection, column
    // resize/reorder, the header's right-click menu) assumes the table is
    // drawn there, which is false once the Diff view takes the column.
    let in_central = cx.log_area.contains((col, row).into());
    let in_log = in_central && *cx.central_view == CentralView::Log;
    let in_diff = in_central && *cx.central_view == CentralView::Diff;
    let in_status = cx.status_area.contains((col, row).into());
    let in_commit_files = *cx.right_panel == RightPanel::CommitDetail
        && cx.commit_files_area.contains((col, row).into());
    // The non-interactive "Commit details" half of the
    // right column. Tracked separately because it's a
    // sub-rect of `status_area`: without it, a click here
    // falls through to the Status branch and destroys the
    // commit view (see that branch's comment).
    let in_commit_meta = *cx.right_panel == RightPanel::CommitDetail
        && cx.commit_meta_area.contains((col, row).into());
    let (divider1_x, divider2_x) = pane_divider_xs(
        *cx.panes_area,
        *cx.divider1_pct,
        *cx.divider2_pct,
        *cx.left_pane_collapsed,
        *cx.right_pane_collapsed,
    );

    // The Preferences screen is modal and on top of everything, so it gets
    // the mouse first, even before the File Insight block below. It
    // hit-tests its own rows against rects captured while drawing, not
    // through `hit_targets`.
    if let Some(prefs) = cx.preferences.as_mut() {
        let stay_open = prefs.on_mouse_shift(
            mouse.kind,
            col,
            row,
            mouse.modifiers.contains(KeyModifiers::SHIFT),
        );
        if let Some(message) = prefs.take_message() {
            *cx.last_message = Some(message);
        }
        if prefs.take_open_about() {
            crate::about::open();
        }
        if !stay_open {
            *cx.preferences = None;
            cx.link.request_refresh();
            if let Some(load) = apply_live_settings(
                &mut *cx.last_message,
                &mut *cx.hierarchical_refs,
                &mut *cx.graph_show_stashes,
                &mut *cx.ref_connector,
                &mut *cx.avatar_placement,
                &mut *cx.avatar_live,
                &mut *cx.ui_lane_background,
                &mut *cx.ui_graph_theme,
                &mut *cx.ui_tint_strength,
            ) {
                cx.lazy_log.apply_settings(load);
            }
        }
        return Ok(None);
    }

    // The editor's body: clicks, drags,
    // the wheel and its scrollbars. Its bar, its `✕ Esc` and
    // the Save / Discard / Cancel prompt are hit targets
    // (`HitTarget::Editor`) and go through the click path
    // below; so does everything outside it. A drag the body
    // started keeps coming here wherever the pointer goes.
    if *cx.central_view == CentralView::Editor && !matches!(mouse.kind, MouseEventKind::Moved) {
        if let Some(ed) = cx.editor.as_mut() {
            let on_control = matches!(
                find_hit_target(&*cx.hit_targets, col, row),
                Some(HitTarget::Editor(h))
                    if !matches!(
                        h,
                        crate::editor::EditorHit::VThumb
                            | crate::editor::EditorHit::HThumb
                            | crate::editor::EditorHit::Minimap
                    )
            );
            let dragging = ed.dragging()
                && matches!(mouse.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_));
            // A `Drag` / `Up` belongs to the editor only when the editor's
            // own press started it (`dragging`): another gesture's drag (a
            // pane divider) that crosses the body would otherwise be eaten
            // here, and its release never reach the drag handler (the pane
            // stuck, the resize pointer never reset).
            let is_drag_or_up =
                matches!(mouse.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_));
            if dragging || (!on_control && !is_drag_or_up && ed.owns_point(col, row)) {
                ed.on_mouse(mouse.kind, mouse.modifiers, col, row);
                return Ok(None);
            }
        }
    }

    // The merge editor's body: the wheel over a pane, a press on the
    // splitter or a scrollbar thumb starting a drag, the drag itself.
    // Everything else (buttons, checkboxes, line gutters) is a hit target
    // and goes through the click path below.
    if *cx.central_view == CentralView::Merge && !matches!(mouse.kind, MouseEventKind::Moved) {
        if let Some(view) = cx.merge.as_mut() {
            let hit = find_hit_target(&*cx.hit_targets, col, row);
            if let (MouseEventKind::Down(MouseButton::Left), Some(HitTarget::Merge(h))) =
                (mouse.kind, &hit)
            {
                if !view.has_popup() && view.begin_drag(*h, row) {
                    return Ok(None);
                }
                if let crate::merge_view::MergeHit::OutputLine(line) = h {
                    if view.output_press(*line, mouse.modifiers, col, row) {
                        return Ok(None);
                    }
                }
            }
            // A press that lands on no row of an open menu only closes it.
            if matches!(mouse.kind, MouseEventKind::Down(_)) && view.has_popup() {
                let on_popup = matches!(
                    hit,
                    Some(HitTarget::Merge(
                        crate::merge_view::MergeHit::MenuRow(_)
                            | crate::merge_view::MergeHit::PromptSave
                            | crate::merge_view::MergeHit::PromptDiscard
                            | crate::merge_view::MergeHit::PromptCancel
                            | crate::merge_view::MergeHit::PromptClose
                    ))
                );
                if !on_popup {
                    view.dismiss_popup();
                    return Ok(None);
                }
            }
            if view.hscroll_mouse(mouse.kind, mouse.modifiers, col, row)
                || view.on_mouse(mouse.kind, col, row)
            {
                return Ok(None);
            }
        }
    }

    // A binary file's "Open in" button (Diff view or File View).
    if matches!(*cx.central_view, CentralView::Diff | CentralView::File)
        && mouse.kind == MouseEventKind::Down(MouseButton::Left)
        && matches!(
            find_hit_target(&*cx.hit_targets, col, row),
            Some(HitTarget::OpenWith)
        )
    {
        if let Some(subject) = crate::open_with::view::shown() {
            crate::open_with::popup::open(subject);
        }
        return Ok(None);
    }
    // The LFS card's "Open in default program": the key `o`'s twin.
    if matches!(*cx.central_view, CentralView::Diff | CentralView::File)
        && mouse.kind == MouseEventKind::Down(MouseButton::Left)
        && matches!(
            find_hit_target(&*cx.hit_targets, col, row),
            Some(HitTarget::OpenWithDefault)
        )
    {
        if let Some(subject) = crate::open_with::view::shown() {
            *cx.last_message = Some(crate::open_with::popup::run_default(
                &subject,
                cx.repo_root,
                cx.link,
            ));
        }
        return Ok(None);
    }

    // A picture in the File View: the header's zoom buttons, the wheel, a pan
    // drag and the scrollbars are the image pane's; nothing of it reaches the
    // text rows' selection. A drag that began in the pane stays its own wherever
    // the pointer goes.
    if *cx.central_view == CentralView::File
        && crate::image_view::pane::active()
        && !matches!(mouse.kind, MouseEventKind::Moved)
        && cx
            .file_insight
            .as_ref()
            .is_some_and(crate::image_view::pane::shows)
        && (crate::image_view::pane::dragging()
            || (cx.file_insight_area.contains((col, row).into()) && row > cx.file_insight_area.y))
    {
        if let (MouseEventKind::Down(MouseButton::Left), Some(HitTarget::Image(h))) =
            (mouse.kind, find_hit_target(&*cx.hit_targets, col, row))
        {
            crate::image_view::pane::click(h);
        } else {
            crate::image_view::pane::on_mouse(mouse.kind, col, row, mouse.modifiers);
        }
        return Ok(None);
    }

    // A picture in the Diff view: before / after panes. The header's buttons,
    // the divider, the wheel, a pan drag and the scrollbars are the image
    // view's; the layout toggle (a bar button) and the close cross keep their
    // own paths. A drag that began in the view stays its own.
    if *cx.central_view == CentralView::Diff
        && crate::image_view::split::active()
        && !matches!(mouse.kind, MouseEventKind::Moved)
    {
        let dragging = crate::image_view::split::dragging();
        let inside = cx.diff_body_area.contains((col, row).into()) && row > cx.diff_body_area.y;
        let target = find_hit_target(&*cx.hit_targets, col, row);
        let bar_button = matches!(
            target,
            Some(HitTarget::DiffBarButton(_) | HitTarget::CloseDiff)
        );
        if dragging || (inside && !bar_button) {
            match (mouse.kind, target) {
                (MouseEventKind::Down(MouseButton::Left), Some(HitTarget::Image(h))) => {
                    use crate::image_view::pane::ImageHit;
                    match h {
                        ImageHit::Flip(side) => crate::image_view::split::show_side(side),
                        ImageHit::Divider => {
                            crate::image_view::split::on_mouse(
                                mouse.kind,
                                col,
                                row,
                                mouse.modifiers,
                            );
                        }
                        ImageHit::EditIn => crate::image_view::pane::click(ImageHit::EditIn),
                        zoom => {
                            if let Some(op) = zoom.op() {
                                crate::image_view::split::run_zoom(op);
                            }
                        }
                    }
                }
                _ => {
                    crate::image_view::split::on_mouse(mouse.kind, col, row, mouse.modifiers);
                }
            }
            return Ok(None);
        }
    }

    // The Blame view: its border (drag to resize), the card and the menu are
    // hit targets of their own; a border drag keeps coming here wherever the
    // pointer goes. Everything else falls through to the File View's body.
    if *cx.central_view == CentralView::File && !matches!(mouse.kind, MouseEventKind::Moved) {
        let hit = match find_hit_target(&*cx.hit_targets, col, row) {
            Some(HitTarget::Blame(h)) => Some(h),
            _ => None,
        };
        // Right-click on a hunk (its annotation or its code) opens the menu
        // of that hunk's commit; on the card or menu itself it does nothing.
        if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
            let area = *cx.file_insight_area;
            if let Some(insight) = cx.file_insight.as_mut().filter(|f| f.blame.is_some()) {
                if matches!(
                    hit,
                    Some(
                        crate::blame_view::BlameHit::Card
                            | crate::blame_view::BlameHit::CardClose
                            | crate::blame_view::BlameHit::Op(_)
                    )
                ) {
                    return Ok(None);
                }
                let inside = area.contains((col, row).into())
                    && row > area.y
                    && row + 1 < area.y + area.height;
                if inside {
                    let viewport = area.height.saturating_sub(2) as usize;
                    let line = insight.offset + (row - area.y - 1) as usize;
                    if line < insight.len() && !insight.blame_open_menu(line, col, row, viewport) {
                        *cx.last_message = Some(
                            gitpoulpi_i18n::msg::blame_view::NOT_COMMITTED_YET
                                .get()
                                .into(),
                        );
                    }
                    return Ok(None);
                }
            }
        }
        if let Some(insight) = cx.file_insight.as_mut().filter(|f| f.blame.is_some()) {
            match insight.blame_mouse(mouse.kind, col, hit) {
                crate::blame_view::MouseOutcome::Handled => return Ok(None),
                crate::blame_view::MouseOutcome::Run(op) => {
                    super::commands::run_blame_op(cx, op);
                    return Ok(None);
                }
                crate::blame_view::MouseOutcome::Width(width) => {
                    super::commands::save_blame_width(cx, width);
                    return Ok(None);
                }
                crate::blame_view::MouseOutcome::Pass => {}
            }
        }
    }

    // The File View's body (the central column). Only events *inside* it are
    // its own; everything else, its bar and the panes around it, goes through
    // the normal paths below, like the Diff view's surroundings.
    let in_file_body = *cx.central_view == CentralView::File
                        && cx.file_insight_area.contains((col, row).into())
                        // The top border carries the `✕ Esc` hit target.
                        && row > cx.file_insight_area.y
                        && !matches!(mouse.kind, MouseEventKind::Moved);
    if let Some(insight) = cx.file_insight.as_mut().filter(|_| in_file_body) {
        let viewport = cx.file_insight_area.height.saturating_sub(2) as usize;
        // The scrollbars take priority over row selection: the vertical bar's
        // column and the horizontal bar's row are border cells a content click
        // never reaches (`scrollbar.rs`'s module doc), so this can't steal a row
        // click. `Moved` events are excluded from `in_file_body` above, so they
        // reach the general `Moved` arm, which drives `hovered_target` from
        // `hit_targets`.
        let v_track =
            scrollbar::Track::vertical(*cx.file_insight_area, insight.len(), insight.offset);
        let h_track = scrollbar::Track::horizontal(
            *cx.file_insight_area,
            insight.longest_line(),
            insight.h_offset,
        );
        let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
        // The Blame view's minimap (`ui.minimap`): a click centers the view
        // on the line it shows there, and dragging along it scrubs.
        let blame_map = (insight.blame_lines().is_some()
            && crate::minimap::padding_for(*cx.file_insight_area, crate::live_settings::minimap())
                > 0)
        .then(|| crate::minimap::rect_in(*cx.file_insight_area))
        .flatten();
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if blame_map.is_some_and(|r| r.contains((col, row).into())) =>
            {
                if let Some(rect) = blame_map {
                    let line = crate::minimap::line_at(rect, insight.len(), row);
                    let top = crate::minimap::top_for(line, insight.len(), viewport);
                    insight.scroll(top as isize - insight.offset as isize, viewport);
                    *cx.dragging_scrollbar = Some((ScrollBar::BlameMinimap, 0));
                }
            }
            MouseEventKind::Down(MouseButton::Left)
                if v_track.is_some_and(|t| t.rect.contains((col, row).into())) =>
            {
                let Some(t) = v_track else {
                    return Ok(None);
                };
                let pos = (row as i32 - t.rect.y as i32).max(0) as u16;
                match t.press(pos) {
                    scrollbar::TrackPress::Arrow(step) => insight.scroll(step, viewport),
                    scrollbar::TrackPress::Grab { grab, jump } => {
                        *cx.dragging_scrollbar = Some((ScrollBar::FileInsightVertical, grab));
                        if jump {
                            insight.offset =
                                t.drag_offset(insight.len(), viewport, grab, i32::from(pos));
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left)
                if h_track.is_some_and(|t| t.rect.contains((col, row).into())) =>
            {
                let Some(t) = h_track else {
                    return Ok(None);
                };
                let pos = (col as i32 - t.rect.x as i32).max(0) as u16;
                match t.press(pos) {
                    scrollbar::TrackPress::Arrow(step) => insight.scroll_h(step),
                    scrollbar::TrackPress::Grab { grab, jump } => {
                        *cx.dragging_scrollbar = Some((ScrollBar::FileInsightHorizontal, grab));
                        if jump {
                            let width = t.rect.width as usize;
                            insight.h_offset =
                                t.drag_offset(insight.longest_line(), width, grab, i32::from(pos));
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // +1 for the body's top border. Derived here rather than via
                // `row_to_index`, whose third argument is an absolute offset from
                // `area.y` (it has changed meaning once before).
                let top = cx.file_insight_area.y + 1;
                let index = insight.offset + (row - top) as usize;
                if index < insight.len() {
                    insight.select(index, viewport);
                }
                // A click still picks the
                // row; a drag from here selects text, a
                // double-click the word under the pointer.
                let now = Instant::now();
                let double = cx.last_click.is_some_and(|(t, c, r)| {
                    now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
                });
                *cx.last_click = Some((now, col, row));
                let pos = file_view::text_pos_at(insight, *cx.file_insight_area, col, row);
                let mut sel = None;
                *cx.text_drag = text_select::press(
                    text_select::TextView::File,
                    0,
                    pos,
                    double,
                    &*insight,
                    &mut sel,
                );
                insight.text_sel = sel;
            }
            MouseEventKind::Drag(MouseButton::Left) if cx.dragging_scrollbar.is_none() => {
                if let Some(drag) = cx
                    .text_drag
                    .filter(|d| d.view == text_select::TextView::File)
                {
                    if let Some(head) =
                        file_view::text_pos_at(insight, *cx.file_insight_area, col, row)
                    {
                        insight.text_sel = Some(drag.to(head));
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some((kind, grab)) = *cx.dragging_scrollbar {
                    match kind {
                        ScrollBar::FileInsightVertical => {
                            if let Some(t) = v_track {
                                let pos = row as i32 - t.rect.y as i32;
                                insight.offset = t.drag_offset(insight.len(), viewport, grab, pos);
                            }
                        }
                        ScrollBar::FileInsightHorizontal => {
                            if let Some(t) = h_track {
                                let pos = col as i32 - t.rect.x as i32;
                                let width = t.rect.width as usize;
                                insight.h_offset =
                                    t.drag_offset(insight.longest_line(), width, grab, pos);
                            }
                        }
                        ScrollBar::BlameMinimap => {
                            if let Some(rect) = blame_map {
                                let line = crate::minimap::line_at(
                                    rect,
                                    insight.len(),
                                    row.clamp(rect.y, rect.bottom().saturating_sub(1)),
                                );
                                let top = crate::minimap::top_for(line, insight.len(), viewport);
                                insight.scroll(top as isize - insight.offset as isize, viewport);
                            }
                        }
                        ScrollBar::DiffVertical
                        | ScrollBar::DiffHorizontal
                        | ScrollBar::LogHorizontal
                        | ScrollBar::DiffMinimap
                        | ScrollBar::CommitDescription => {}
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                *cx.dragging_scrollbar = None;
                *cx.text_drag = None;
            }
            // Shift+wheel is horizontal scroll. Crossterm reports it two ways
            // depending on the terminal: `ScrollLeft`/`ScrollRight`, or plain
            // `ScrollUp`/`ScrollDown` with `SHIFT` set; both are handled.
            MouseEventKind::ScrollLeft => insight.scroll_h(-4),
            MouseEventKind::ScrollRight => insight.scroll_h(4),
            MouseEventKind::ScrollDown if shift => insight.scroll_h(4),
            MouseEventKind::ScrollUp if shift => insight.scroll_h(-4),
            MouseEventKind::ScrollDown => insight.scroll(3, viewport),
            MouseEventKind::ScrollUp => insight.scroll(-3, viewport),
            _ => {}
        }
        return Ok(None);
    }

    // The management tab's dialogs are modal for the
    // mouse, the way the Preferences screen is: while one
    // is open every event -- click, wheel, hover -- goes
    // to it (it hit-tests the rects it recorded while
    // drawing), and nothing underneath sees it.
    if cx.repo_dialogs.is_open() && !matches!(*cx.input_mode, InputMode::CommandPalette { .. }) {
        if let Some(effect) = cx.repo_dialogs.on_mouse(mouse) {
            menu_actions::apply_effect(cx, effect);
        }
        if !cx.repo_dialogs.pull_request_open() && cx.ai_ui.running_pull_request() {
            cx.ai_ui.cancel();
        }
        if !cx.repo_dialogs.is_open() {
            refresh_management(cx.session, &mut *cx.management);
        }
        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
        return Ok(None);
    }

    // The repository tab bar, tested before anything else: it is the topmost
    // row and `hit_targets` knows nothing about it (the bar carries its own
    // rects, so the list that drew the glyphs resolves the clicks). Registered
    // close-button-first inside `tab_bar_click` (the inner-first rule): a tab
    // band covers the `x` inside it, and the wrong order makes the button
    // unreachable.
    // The repository menu (tab menu / a management row's
    // menu) is modal for the mouse: a click runs the row
    // under it or dismisses the menu, and nothing
    // underneath sees either.
    if cx.repo_menu.is_some() {
        if let MouseEventKind::Down(button) = mouse.kind {
            let row_hit = cx
                .repo_menu_rects
                .iter()
                .position(|r| r.contains((col, row).into()));
            match (button, row_hit) {
                (MouseButton::Left, Some(index)) => {
                    let outcome = cx.repo_menu.as_mut().map(|s| s.menu.activate(index));
                    match outcome {
                        Some(MenuOutcome::Run(action)) => {
                            if let Some(state) = cx.repo_menu.take() {
                                if let Some(message) =
                                    apply_repo_menu_action(action, state.target, &mut tab_ctx!(cx))
                                {
                                    *cx.last_message = Some(message.clone());
                                    if let Some(m) = cx.management.as_mut() {
                                        m.message = Some(message);
                                    }
                                }
                            }
                        }
                        Some(MenuOutcome::Close) => *cx.repo_menu = None,
                        _ => {}
                    }
                }
                _ => *cx.repo_menu = None,
            }
            *cx.should_quit |=
                cx.switch_to.is_some() || *cx.close_window || cx.show_management.is_some();
        }
        return Ok(None);
    }

    // Right-click on a repository tab: its menu,
    // hanging under the click. `Ctrl+T` is the keyboard
    // half, and every row is also a `Tab: ...` palette
    // command.
    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Right))
        && matches!(*cx.input_mode, InputMode::Normal)
    {
        if let Some(index) = repo_switcher::tab_at(&*cx.tab_hits, col, row) {
            if let Some(path) = cx.repo_tabs.get(index).map(|t| t.path.clone()) {
                open_repo_menu(&path, true, (col, row + 1), &mut tab_ctx!(cx));
            }
            return Ok(None);
        }
    }

    // The wheel over the vertical tab column scrolls its list of tabs.
    if matches!(*cx.input_mode, InputMode::Normal) {
        let delta = match mouse.kind {
            MouseEventKind::ScrollUp => Some(-1),
            MouseEventKind::ScrollDown => Some(1),
            _ => None,
        };
        if let (Some(delta), Some(area)) = (delta, crate::tab_column::area()) {
            if area.contains((col, row).into()) {
                crate::tab_column::scroll_by(delta);
                return Ok(None);
            }
        }
        // Over the horizontal bar (which has no other wheel action), the
        // wheel scrolls its tabs: up or left shows the earlier ones.
        let sideways = match mouse.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => Some(-1),
            MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => Some(1),
            _ => None,
        };
        if let Some(delta) = sideways {
            if crate::tab_scroll::wheel(col, row, delta) {
                return Ok(None);
            }
        }
    }

    // Middle-click closes a tab, through the same confirmation as its `x`
    // and `Ctrl+W`; on the New tab it just closes it.
    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Middle))
        && matches!(*cx.input_mode, InputMode::Normal)
    {
        let on_radar = cx.hit_targets.iter().any(|(r, t)| {
            matches!(t, HitTarget::RadarTab | HitTarget::RadarClose)
                && r.contains((col, row).into())
        });
        if on_radar {
            close_radar_tab(&mut tab_ctx!(cx));
            return Ok(None);
        }
        let on_workspaces = cx.hit_targets.iter().any(|(r, t)| {
            matches!(t, HitTarget::WorkspacesTab | HitTarget::WorkspacesClose)
                && r.contains((col, row).into())
        });
        if on_workspaces {
            close_workspaces_tab(&mut tab_ctx!(cx));
            return Ok(None);
        }
        let on_notes = cx.hit_targets.iter().any(|(r, t)| {
            matches!(t, HitTarget::NotesTab | HitTarget::NotesClose)
                && r.contains((col, row).into())
        });
        if on_notes {
            close_notes_tab(&mut tab_ctx!(cx));
            return Ok(None);
        }
        if let Some(index) = repo_switcher::tab_at(&*cx.tab_hits, col, row) {
            if index == cx.repo_tabs.len() {
                if let Some(message) = close_new_tab(&mut tab_ctx!(cx)) {
                    *cx.last_message = Some(message);
                }
                *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
            } else if let Some(prompt) = close_tab_prompt(&*cx.repo_tabs, index) {
                *cx.input_mode = InputMode::Confirm {
                    prompt,
                    action: ConfirmAction::CloseTab(index),
                };
            }
            return Ok(None);
        }
    }

    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
        *cx.tab_drag = None;
        if let Some(action) = repo_switcher::tab_bar_click(&*cx.tab_hits, col, row) {
            // A press on a repository tab (not its `x`, not the New tab)
            // is a click or the start of a reorder: decided on release.
            if let repo_switcher::SwitcherAction::Focus(index) = action {
                if let Some(tab) = cx.repo_tabs.get(index) {
                    *cx.tab_drag = Some(crate::tab_drag::TabDrag::armed(
                        index,
                        tab.name.clone(),
                        crate::tab_drag::DragFrom::Bar,
                        col,
                        row,
                    ));
                    return Ok(None);
                }
            }
            // The transient New tab is the one past the last repository
            // tab: its `x` closes it, a click on it is already there.
            if let repo_switcher::SwitcherAction::Close(index)
            | repo_switcher::SwitcherAction::Focus(index) = action
            {
                if index == cx.repo_tabs.len() {
                    if matches!(action, repo_switcher::SwitcherAction::Close(_)) {
                        if let Some(message) = close_new_tab(&mut tab_ctx!(cx)) {
                            *cx.last_message = Some(message);
                        }
                        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
                    }
                    return Ok(None);
                }
            }
            // Closing asks first; focusing does not.
            // The question goes through the app's one
            // confirm modal rather than a second dialog
            // language of its own.
            if let repo_switcher::SwitcherAction::Close(index) = action {
                if let Some(prompt) = close_tab_prompt(&*cx.repo_tabs, index) {
                    *cx.input_mode = InputMode::Confirm {
                        prompt,
                        action: ConfirmAction::CloseTab(index),
                    };
                }
                return Ok(None);
            }
            if let Some(message) = apply_switcher_action(
                action,
                cx.session,
                cx.link,
                cx.repo,
                &mut *cx.repo_dialogs,
                &mut *cx.repo_tabs,
                &mut *cx.active_tab,
                &mut *cx.switch_to,
                &mut *cx.close_window,
            ) {
                *cx.last_message = Some(message);
            }
            *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
            return Ok(None);
        }
    }

    // The gear, checked ahead of the Repository Management body handling
    // below: that block claims every event inside `body`, where the gear is
    // drawn on this tab, so left unguarded it would swallow clicks and hover
    // that the ordinary toolbar path never reaches on this tab.
    if cx.management.is_some() && matches!(*cx.input_mode, InputMode::Normal) {
        let on_gear = find_hit_target(&*cx.hit_targets, col, row) == Some(HitTarget::Gear);
        if on_gear {
            match mouse.kind {
                MouseEventKind::Moved => *cx.hovered_target = Some(HitTarget::Gear),
                MouseEventKind::Down(MouseButton::Left) => {
                    *cx.preferences = Some(PreferencesState::open(cx.repo_root));
                }
                _ => {}
            }
            return Ok(None);
        } else if matches!(mouse.kind, MouseEventKind::Moved) && hover(&*cx.hovered_target).gear() {
            *cx.hovered_target = None;
        }
    }

    // The Release notes tab, when it shows over the Home tab, owns the body:
    // clicks, the wheel and the hover go to it, nothing underneath.
    if matches!(*cx.input_mode, InputMode::Normal) {
        let body = cx.management_panel.as_ref().map(|(body, _)| *body);
        if let (Some(body), Some(manager)) = (body, cx.management.as_mut()) {
            if manager.notes.is_open() && body.contains((col, row).into()) {
                let event = manager.notes.on_mouse(mouse);
                commands::apply_notes_event(cx, event);
                return Ok(None);
            }
        }
    }

    // The Radar, when it shows over the Home tab, owns the body: its own
    // hit rects (popups first), nothing underneath it. A click above the
    // body (the tab bar) goes on to the ordinary path.
    if matches!(*cx.input_mode, InputMode::Normal) {
        let body = cx.management_panel.as_ref().map(|(body, _)| *body);
        if let (Some(body), Some(manager)) = (body, cx.management.as_mut()) {
            if manager.radar.is_open()
                && (body.contains((col, row).into())
                    || manager.radar.popup_open()
                    || manager.radar.sizing_column())
            {
                let mut double = false;
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                    let now = Instant::now();
                    double = cx.last_click.is_some_and(|(t, c, r)| {
                        now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
                    });
                    *cx.last_click = Some((now, col, row));
                }
                let outcome = manager.radar.on_mouse(mouse, double);
                // The resize pointer over a separator, back to the default
                // off it; written on a change only.
                let shape = if manager.radar.resize_pointer() {
                    CursorShape::ColResize
                } else if manager.radar.pointer_text_at() {
                    CursorShape::Text
                } else if manager.radar.pointer_hand_at(col, row) {
                    CursorShape::Pointer
                } else {
                    CursorShape::Default
                };
                cursor::set_pointer_shape(cx.terminal.backend_mut(), cx.last_cursor_shape, shape)
                    .map_err(exit_log::io_context("setting the pointer shape"))?;
                commands::apply_radar_outcome(cx, outcome);
                return Ok(None);
            }
            // A move out of the body (the tab bar, the status bar) is not the
            // Radar's to hear: its hover (a lit separator) would stay.
            if manager.radar.is_open() && matches!(mouse.kind, MouseEventKind::Moved) {
                manager.radar.pointer_left();
            }
        }
    }

    // The Repository Management tab owns everything under
    // the tab bar: clicks, right clicks, the wheel and
    // hover. It hit-tests the rects its own render walk
    // produced (`ManagerRender::hit`); the empty panes
    // underneath never see an event. Skipped while a
    // modal (the confirm popup) is up, which then gets
    // the click through the ordinary path below.
    if matches!(*cx.input_mode, InputMode::Normal) {
        if let (Some(manager), Some((body, render))) =
            (cx.management.as_mut(), cx.management_panel.as_ref())
        {
            let inside = body.contains((col, row).into());
            let hit = if inside { render.hit(col, row) } else { None };
            // A drag that started in the search box keeps
            // selecting wherever the pointer goes.
            if manager.search_dragging()
                && matches!(
                    mouse.kind,
                    MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
                )
            {
                manager.search_mouse(mouse.kind, col, row);
                return Ok(None);
            }
            if matches!(mouse.kind, MouseEventKind::Moved) {
                manager.hover = hit;
            }
            // A click above the body -- the tab bar --
            // isn't "inside" and falls through to the
            // ordinary path below (the search
            // box loses its typing focus like every other
            // field does on an outside click; the query
            // itself is untouched).
            if !inside && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                manager.search_focused = false;
            }
            // The list's scrollbar: thumb grab and drag (even off the bar), a
            // press on the track jumping the thumb, the end arrows.
            let (len, top) = manager.list_bar_state();
            let bar = manager.bar;
            if let Some(top) = manager.bar_drag.on_mouse(bar, mouse, len, top) {
                manager.set_list_scroll(top);
                return Ok(None);
            }
            if inside {
                if hover(&*cx.hovered_target).folder() {
                    *cx.hovered_target = None;
                }
                let mut action = repo_manager::ManagerAction::Handled;
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        let now = Instant::now();
                        let double = cx.last_click.is_some_and(|(t, c, r)| {
                            now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
                        });
                        *cx.last_click = Some((now, col, row));
                        if let Some(hit) = hit {
                            action = repo_manager::click(manager, hit, double);
                            if hit == repo_manager::ManagerHit::Search {
                                manager.search_mouse(mouse.kind, col, row);
                            }
                        } else {
                            manager.search_focused = false;
                        }
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        if let Some(repo_manager::ManagerHit::Ws(ws_hit)) = hit {
                            if let Some(menu) = repo_manager::ws_menu_for_hit(manager, ws_hit) {
                                action = menu;
                            }
                        } else if let Some((index, repo)) =
                            hit.and_then(|h| repo_manager::menu_target(manager, h))
                        {
                            manager.selected = index;
                            action = repo_manager::ManagerAction::Menu { repo };
                        }
                    }
                    MouseEventKind::ScrollDown => manager.scroll_by(3),
                    MouseEventKind::ScrollUp => manager.scroll_by(-3),
                    _ => {}
                }
                if let Some(message) =
                    apply_manager_action(action, (col, row + 1), &mut tab_ctx!(cx))
                {
                    *cx.last_message = Some(message.clone());
                    if let Some(m) = cx.management.as_mut() {
                        m.message = Some(message);
                    }
                }
                *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
                return Ok(None);
            }
        }
    }

    Ok(Some(Geo {
        col,
        row,
        in_refs,
        in_log,
        in_diff,
        in_status,
        in_commit_files,
        in_commit_meta,
        divider1_x,
        divider2_x,
    }))
}

/// Every list bar drawn *right now*, resolved from the same numbers the render pass
/// used (see `ui_kit::ListBarRegion`) - one place, so a list added to the drag
/// later is one line here and one arm in `list_bar_offset`/`set_list_bar_offset`.
pub(super) fn list_bar_regions(cx: &mut CommandCtx<'_>) -> Vec<ListBarRegion> {
    let mut regions: Vec<ListBarRegion> = Vec::new();
    if *cx.central_view == CentralView::Log && cx.log_area.height > 0 {
        regions.push(ListBarRegion {
            bar: ListBar::Log,
            area: *cx.log_area,
            len: *cx.log_len,
            visible: log_visible_rows(*cx.log_area),
        });
    }
    for (section, area) in cx.sidebar_section_areas.iter().copied() {
        regions.push(ListBarRegion {
            bar: ListBar::Refs(section),
            area,
            len: ref_section_row_count(
                &*cx.refs_result,
                &*cx.refs_filter,
                section,
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
                cx.worktrees_result.as_deref().unwrap_or(&[]).len(),
                cx.stashes_result.as_deref().unwrap_or(&[]).len(),
                cx.hosting.row_counts(),
                cx.submodules_result.len(),
            ),
            visible: area.height.saturating_sub(2) as usize,
        });
    }
    if *cx.right_panel == RightPanel::CommitDetail && cx.commit_files_area.height > 0 {
        regions.push(ListBarRegion {
            bar: ListBar::CommitFiles,
            area: commit_detail::files_list_body(*cx.commit_files_area),
            len: *cx.commit_files_len,
            visible: *cx.commit_files_visible_rows,
        });
    }
    if *cx.right_panel == RightPanel::CommitDetail && cx.commit_message_area.height > 0 {
        regions.push(ListBarRegion {
            bar: ListBar::CommitMessage,
            area: *cx.commit_message_area,
            len: *cx.commit_message_total,
            visible: (*cx.commit_message_rows_now as usize).max(1),
        });
    }
    // The composer's Description bar, from the track the paint walk
    // registered (a bordered-body stand-in two columns wide whose
    // right column is that track).
    if let Some(track) = cx.hit_targets.iter().find_map(|(rect, target)| {
        matches!(
            target,
            HitTarget::ScrollbarTrack(ScrollBar::CommitDescription)
        )
        .then_some(*rect)
    }) {
        regions.push(ListBarRegion {
            bar: ListBar::CommitDescription,
            area: Rect::new(
                track.x.saturating_sub(1),
                track.y.saturating_sub(1),
                2,
                track.height + 2,
            ),
            len: cx.commit_description.split('\n').count(),
            visible: track.height as usize,
        });
    }
    if let Some(h) = cx.history.as_ref().filter(|h| h.list_area.height > 0) {
        regions.push(ListBarRegion {
            bar: ListBar::History,
            area: h.list_area,
            len: h.rows() * usize::from(crate::history_view::ROW_H),
            visible: h.rows_area.height as usize,
        });
    }
    if cx.commit_col.area.height > 0 {
        regions.push(ListBarRegion {
            bar: ListBar::CommitColumn,
            area: cx.commit_col.area,
            len: cx.commit_col.len,
            visible: cx.commit_col.viewport(),
        });
    }
    if cx.wip_all.shown() {
        regions.push(ListBarRegion {
            bar: ListBar::WipAll,
            area: cx.wip_all.area,
            len: cx.wip_all.rows.len(),
            visible: cx.wip_all.area.height.saturating_sub(2) as usize,
        });
    } else {
        for (section, area) in cx.status_section_areas.iter().copied() {
            regions.push(ListBarRegion {
                bar: ListBar::Status(section),
                area,
                len: section_lines(&*cx.status_section_lines, section).len(),
                visible: area.height.saturating_sub(2) as usize,
            });
        }
    }
    regions
}

pub(super) fn list_bar_offset(cx: &mut CommandCtx<'_>, bar: ListBar) -> usize {
    match bar {
        ListBar::Log => cx.log_state.offset(),
        ListBar::CommitFiles => cx.commit_file_state.offset(),
        ListBar::CommitMessage => *cx.commit_message_scroll,
        ListBar::AboutCrates | ListBar::ActivityLog | ListBar::JobsPanel | ListBar::SafetyPanel => {
            0
        }
        ListBar::History => cx
            .history
            .as_ref()
            .map_or(0, |h| h.offset * usize::from(crate::history_view::ROW_H)),
        ListBar::CommitColumn => cx.commit_col.offset,
        ListBar::WipAll => cx.wip_all.state.offset(),
        ListBar::CommitDescription => cx
            .field_editor
            .caret(MainField::Description)
            .map_or(0, |c| c.top.get()),
        ListBar::Refs(section) => cx.sidebar_states.state(section).offset(),
        ListBar::Status(section) => cx.status_states.state(section).offset(),
    }
}

/// Like the wheel: moves the viewport only, and stops the offset following the
/// selection back on the next render.
pub(super) fn set_list_bar_offset(cx: &mut CommandCtx<'_>, bar: ListBar, new_offset: usize) {
    match bar {
        ListBar::Log => {
            *cx.log_state.offset_mut() = new_offset;
            *cx.log_follow = false;
        }
        ListBar::CommitFiles => {
            *cx.commit_file_state.offset_mut() = new_offset;
            *cx.commit_files_follow = false;
        }
        ListBar::CommitMessage => *cx.commit_message_scroll = new_offset,
        ListBar::AboutCrates | ListBar::ActivityLog | ListBar::JobsPanel | ListBar::SafetyPanel => {
        }
        ListBar::History => {
            if let Some(h) = cx.history.as_mut() {
                // The bar counts cells, two per commit.
                let row = new_offset.div_ceil(usize::from(crate::history_view::ROW_H));
                let viewport = h.viewport_rows();
                h.scroll_to(row, viewport);
            }
        }
        ListBar::CommitColumn => cx.commit_col.scroll_to(new_offset),
        ListBar::WipAll => {
            *cx.wip_all.state.offset_mut() = new_offset;
            cx.wip_all.follow = false;
        }
        ListBar::CommitDescription => {
            // Moves the caret into view too, so the next render
            // does not scroll straight back to it.
            let rows = list_bar_regions(cx)
                .into_iter()
                .find(|r| r.bar == ListBar::CommitDescription)
                .map_or(1, |r| r.visible);
            cx.field_editor
                .caret_mut(MainField::Description)
                .scroll_lines_to(&*cx.commit_description, new_offset, rows);
        }
        ListBar::Refs(section) => {
            *cx.sidebar_states.state_mut(section).offset_mut() = new_offset;
            *cx.sidebar_states.follow_mut(section) = false;
        }
        ListBar::Status(section) => {
            *cx.status_states.state_mut(section).offset_mut() = new_offset;
            *cx.status_states.follow_mut(section) = false;
        }
    }
}
