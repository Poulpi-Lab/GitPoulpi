//! The mouse wheel.

use super::commands::{self, CommandCtx, Flow};
use super::mouse_prelude::{list_bar_offset, list_bar_regions, Geo};
use crate::event_loop::*;
use crossterm::event::MouseEvent;

/// The wheel (and Shift+wheel / horizontal scroll) over the pane under the pointer.
pub(super) fn handle(cx: &mut CommandCtx<'_>, g: &Geo, mouse: MouseEvent) -> io::Result<Flow> {
    let Geo {
        col,
        row,
        in_refs,
        in_log,
        in_diff,
        in_status,
        in_commit_files,
        ..
    } = *g;
    // The commit column and the File History list scroll on their own: up
    // and down by the wheel, sideways by Shift+wheel or a horizontal wheel
    // (the diff view's gestures).
    let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
    match mouse.kind {
        MouseEventKind::ScrollLeft if commands::commit_column::on_hwheel(cx, col, row, -1) => {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollRight if commands::commit_column::on_hwheel(cx, col, row, 1) => {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollDown
            if shift && commands::commit_column::on_hwheel(cx, col, row, 1) =>
        {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollUp
            if shift && commands::commit_column::on_hwheel(cx, col, row, -1) =>
        {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollDown if commands::commit_column::on_wheel(cx, col, row, 1) => {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollUp if commands::commit_column::on_wheel(cx, col, row, -1) => {
            return Ok(Flow::Handled)
        }
        _ => {}
    }
    match mouse.kind {
        MouseEventKind::ScrollDown if commands::history::on_wheel(cx, col, row, 1) => {
            return Ok(Flow::Handled)
        }
        MouseEventKind::ScrollUp if commands::history::on_wheel(cx, col, row, -1) => {
            return Ok(Flow::Handled)
        }
        _ => {}
    }
    match mouse.kind {
        // Shift+wheel scrolls the diff horizontally. Crossterm reports it as
        // `ScrollLeft`/`ScrollRight` or as plain `ScrollDown`/`ScrollUp` with
        // `SHIFT` set depending on the terminal, so both are handled (as in the
        // File Insight overlay).
        MouseEventKind::ScrollLeft if in_diff => {
            if let Some(st) = cx.diff_view_state.as_mut() {
                st.scroll_h(-4);
            }
        }
        MouseEventKind::ScrollRight if in_diff => {
            if let Some(st) = cx.diff_view_state.as_mut() {
                st.scroll_h(4);
            }
        }
        MouseEventKind::ScrollDown if in_diff && mouse.modifiers.contains(KeyModifiers::SHIFT) => {
            if let Some(st) = cx.diff_view_state.as_mut() {
                st.scroll_h(4);
            }
        }
        MouseEventKind::ScrollUp if in_diff && mouse.modifiers.contains(KeyModifiers::SHIFT) => {
            if let Some(st) = cx.diff_view_state.as_mut() {
                st.scroll_h(-4);
            }
        }
        // The wheel over the focused composer Description (the
        // keyboard twin is the caret's own Up/Down/PageUp/PageDown).
        MouseEventKind::ScrollDown | MouseEventKind::ScrollUp
            if *cx.composer_focus == Some(ComposerField::Description)
                && cx.hit_targets.iter().any(|(r, t)| {
                    *t == HitTarget::CommitDescriptionField && r.contains((col, row).into())
                }) =>
        {
            let rows = list_bar_regions(cx)
                .iter()
                .find(|r| r.bar == ListBar::CommitDescription)
                .map_or(1, |r| r.visible);
            let top = list_bar_offset(cx, ListBar::CommitDescription) as isize
                + if mouse.kind == MouseEventKind::ScrollDown {
                    3
                } else {
                    -3
                };
            cx.field_editor
                .caret_mut(MainField::Description)
                .scroll_lines_to(&*cx.commit_description, top.max(0) as usize, rows);
        }
        MouseEventKind::ScrollDown
            if *cx.right_panel == RightPanel::CommitDetail
                && cx.commit_message_area.contains((col, row).into()) =>
        {
            commands::scroll_commit_message(cx, 3);
        }
        MouseEventKind::ScrollUp
            if *cx.right_panel == RightPanel::CommitDetail
                && cx.commit_message_area.contains((col, row).into()) =>
        {
            commands::scroll_commit_message(cx, -3);
        }
        MouseEventKind::ScrollDown => {
            if in_refs {
                let sections = cx.sidebar_section_areas.clone();
                if let Some((section, area)) = sections
                    .into_iter()
                    .find(|(_, a)| a.contains((col, row).into()))
                {
                    *cx.active_ref_section = section;
                    let len = ref_section_row_count(
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
                    );
                    scroll_sidebar_section(&mut *cx.sidebar_states, section, len, area, 1);
                }
            } else if in_diff {
                if let Some(st) = cx.diff_view_state.as_mut() {
                    st.scroll = st.scroll.saturating_add(3);
                }
            } else if in_commit_files {
                let new_offset = scroll_offset(
                    cx.commit_file_state.offset(),
                    *cx.commit_files_len,
                    *cx.commit_files_visible_rows,
                    1,
                );
                *cx.commit_file_state.offset_mut() = new_offset;
                *cx.commit_files_follow = false;
            } else if in_log {
                let new_offset = scroll_offset(
                    cx.log_state.offset(),
                    *cx.log_len,
                    log_visible_rows(*cx.log_area),
                    *cx.log_scroll_lines,
                );
                *cx.log_state.offset_mut() = new_offset;
                *cx.log_follow = false;
            } else if cx.wip_all.shown() && cx.wip_all.area.contains((col, row).into()) {
                cx.wip_all.scroll_by(1);
            } else if in_status {
                scroll_status_pane(
                    &*cx.status_section_areas,
                    &mut *cx.status_states,
                    &*cx.status_section_lines,
                    col,
                    row,
                    1,
                );
                *cx.right_panel = RightPanel::Status;
            }
        }
        MouseEventKind::ScrollUp => {
            if in_refs {
                let sections = cx.sidebar_section_areas.clone();
                if let Some((section, area)) = sections
                    .into_iter()
                    .find(|(_, a)| a.contains((col, row).into()))
                {
                    *cx.active_ref_section = section;
                    let len = ref_section_row_count(
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
                    );
                    scroll_sidebar_section(&mut *cx.sidebar_states, section, len, area, -1);
                }
            } else if in_diff {
                if let Some(st) = cx.diff_view_state.as_mut() {
                    st.scroll = st.scroll.saturating_sub(3);
                }
            } else if in_commit_files {
                let new_offset = scroll_offset(
                    cx.commit_file_state.offset(),
                    *cx.commit_files_len,
                    *cx.commit_files_visible_rows,
                    -1,
                );
                *cx.commit_file_state.offset_mut() = new_offset;
                *cx.commit_files_follow = false;
            } else if in_log {
                let new_offset = scroll_offset(
                    cx.log_state.offset(),
                    *cx.log_len,
                    log_visible_rows(*cx.log_area),
                    -*cx.log_scroll_lines,
                );
                *cx.log_state.offset_mut() = new_offset;
                *cx.log_follow = false;
            } else if cx.wip_all.shown() && cx.wip_all.area.contains((col, row).into()) {
                cx.wip_all.scroll_by(-1);
            } else if in_status {
                scroll_status_pane(
                    &*cx.status_section_areas,
                    &mut *cx.status_states,
                    &*cx.status_section_lines,
                    col,
                    row,
                    -1,
                );
                *cx.right_panel = RightPanel::Status;
            }
        }
        _ => {}
    }
    Ok(Flow::Continue)
}
