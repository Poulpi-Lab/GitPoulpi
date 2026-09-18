//! Pointer motion: the hover state, tooltips, the ref popup and the pointer shape.

use super::commands::{CommandCtx, Flow};
use super::mouse_prelude::Geo;
use crate::event_loop::*;
use crossterm::event::MouseEvent;

/// Pointer motion. `hovered_target` is the single source of truth: everything
/// else the draw pass highlights is derived from this one hit test.
pub(super) fn handle(cx: &mut CommandCtx<'_>, g: &Geo, _mouse: MouseEvent) -> io::Result<Flow> {
    let Geo {
        col,
        row,
        in_log,
        in_diff,
        divider1_x,
        divider2_x,
        ..
    } = *g;
    // Tracks which Log row+column (if any) the mouse is over, for the
    // per-column hover tooltip in the draw pass. Recomputes the column layout
    // from the persisted `log_area` (the same pure function the draw pass
    // uses): only geometry, not the widths, survives between frames.
    let moved_hit = find_hit_target(&*cx.hit_targets, col, row);
    // The single source of truth -- everything below is a
    // thin derived view of this one hit test.
    *cx.hovered_target = moved_hit.clone();
    *cx.hovered_tab_index = repo_switcher::tab_at(&*cx.tab_hits, col, row);
    *cx.hovered_close_tab = cx
        .tab_hits
        .iter()
        .find(|h| h.close.contains((col, row).into()))
        .map(|h| h.index);
    *cx.hovered_repo_menu_row = cx
        .repo_menu_rects
        .iter()
        .position(|r| r.contains((col, row).into()));
    // Same lighten-on-hover affordance the toolbar
    // uses, for the Diff view's own bar.
    // Which working-changes row the cursor is over,
    // so that row alone reveals its Stage/Unstage
    // button on hover.
    *cx.hovered_status_row = if *cx.right_panel == RightPanel::Status {
        cx.status_section_areas
            .iter()
            .copied()
            .find(|(_, a)| a.contains((col, row).into()))
            .and_then(|(section, area)| {
                let lines = section_lines(&*cx.status_section_lines, section);
                row_to_index(
                    area,
                    cx.status_states.state(section).offset(),
                    1,
                    row,
                    lines.len(),
                )
                .map(|i| (section, i))
            })
    } else {
        None
    };
    // Which diff row the cursor is over, so that
    // hunk's header reveals its Stage/Unstage
    // button -- the third pane's hover affordance,
    // one granularity down.
    *cx.hovered_diff_row = cx.diff_view_state.as_ref().and_then(|s| {
        in_diff
            .then(|| {
                diff_view::diff_row_at(*cx.diff_body_area, s.scroll, *cx.diff_rendered_len, row)
            })
            .flatten()
    });
    let ref_hit = sidebar_ref_at(
        &*cx.sidebar_section_areas,
        cx.refs_result.as_deref().unwrap_or(&[]),
        &*cx.refs_filter,
        *cx.hierarchical_refs,
        &*cx.collapsed_ref_folders,
        &*cx.remote_names,
        |s| cx.sidebar_states.state(s).offset(),
        col,
        row,
    );
    *cx.hovered_ref_row = ref_hit.as_ref().map(|h| h.name.clone());
    crate::flow::view::set_hover(cx.hovered_ref_row.clone().map(|n| (n, col, row)));
    // A tag row's tooltip says where the tag lives (`tag_where_ui`).
    crate::tag_where_ui::set_hover(
        ref_hit
            .filter(|h| h.kind == RefRowKind::Tag)
            .map(|h| (h.name, col, row)),
    );
    *cx.hovered_ref_folder = sidebar_folder_at(
        &*cx.sidebar_section_areas,
        cx.refs_result.as_deref().unwrap_or(&[]),
        &*cx.refs_filter,
        *cx.hierarchical_refs,
        &*cx.collapsed_ref_folders,
        &*cx.remote_names,
        |s| cx.sidebar_states.state(s).offset(),
        col,
        row,
    );
    let widths = LogColumnWidths::for_area_width(
        cx.log_area.width,
        &*cx.log_column_hidden,
        &*cx.log_column_overrides,
    );
    let visible_order: Vec<LogColumn> = cx
        .log_column_order
        .iter()
        .copied()
        .filter(|c| !cx.log_column_hidden.contains(c))
        .collect();
    let offsets = column_x_offsets(cx.log_area.x, &visible_order, &widths);
    let has_wip = has_wip_row(&*cx.status_result);
    let hovered_index = in_log
        .then(|| log_row_to_index(*cx.log_area, cx.log_state.offset(), row, *cx.log_len))
        .flatten();
    // The WIP row lights on hover like a commit row
    // does -- it is one now, as far as the eye goes.
    *cx.hovered_wip_row = has_wip && hovered_index == Some(0);
    // A stash row is "hovered" by its own commit id:
    // it lights like a commit row, and every
    // per-commit lookup (tooltip, pills) finds no
    // commit behind it and stays quiet.
    *cx.hovered_commit = hovered_index
        .and_then(|i| match cx.log_order.entry(i) {
            graph::LogEntry::Commit(commit_idx) => cx
                .log_result
                .as_ref()
                .ok()
                .and_then(|g| g.nodes.get(commit_idx))
                .map(|c| c.id),
            graph::LogEntry::Stash(_) => {
                log_stash_at(&*cx.stashes_result, &*cx.log_order, i).and_then(|s| s.commit)
            }
            graph::LogEntry::Wip => None,
        })
        .and_then(|commit_id| {
            log_hover_column(&offsets, col).map(|hover_col| (commit_id, hover_col, col, row))
        });

    // A commit row of the commit column hovers like a row of the Log: the
    // same tooltip (`overlay_menus`), by the part under the pointer (the
    // title is the message, then the author and the date; the lanes have
    // none).
    if let Some(HitTarget::CommitColumn(crate::commit_column::ColumnHit::Row(id))) = &moved_hit {
        use crate::commit_column::Part;
        let column = match cx.commit_col.part_at(col) {
            Some(Part::Title) => LogColumn::Message,
            Some(Part::Author) => LogColumn::Author,
            Some(Part::Date) => LogColumn::Date,
            Some(Part::Changes) => LogColumn::Changes,
            Some(Part::Sha) => LogColumn::Sha,
            // The lanes and the refs have no tooltip.
            Some(Part::Graph | Part::Refs) | None => LogColumn::Graph,
        };
        *cx.hovered_commit = Some((*id, column, col, row));
    }
    // And a row of the File History's list: its first line is the message
    // (the Log's tooltip for it), the second the author's.
    if let Some(HitTarget::History(crate::history_view::HistoryHit::Row(i))) = &moved_hit {
        if let Some(h) = cx.history.as_ref() {
            if let Some(entry) = h.row_at(*i) {
                let line = row.saturating_sub(h.rows_area.y) % crate::history_view::ROW_H;
                let column = if line == 0 {
                    LogColumn::Message
                } else {
                    LogColumn::Author
                };
                *cx.hovered_commit = Some((entry.commit, column, col, row));
            }
        }
    }

    crate::refs_sidebar::set_row_pointer(Some((col, row)));
    // Over an open popup's row, nothing behind it is hovered: the row hovers
    // above derive from geometry, not from `moved_hit`, so a row under a
    // context menu would otherwise light up along with the menu row.
    if hover(&*cx.hovered_target).menu_row().is_some()
        || hover(&*cx.hovered_target).column_menu_item().is_some()
        || cx.hovered_repo_menu_row.is_some()
        || hover(&*cx.hovered_target).confirm().is_some()
    {
        *cx.hovered_status_row = None;
        *cx.hovered_diff_row = None;
        *cx.hovered_ref_row = None;
        crate::flow::view::set_hover(None);
        crate::tag_where_ui::set_hover(None);
        *cx.hovered_ref_folder = None;
        crate::refs_sidebar::set_row_pointer(None);
        *cx.hovered_wip_row = false;
        *cx.hovered_commit = None;
    }

    // Scrollbars light under the pointer (arrows, thumb, track) unless a
    // popup, menu or modal owns it.
    crate::scrollbar::set_pointer(
        (matches!(
            moved_hit,
            None | Some(HitTarget::ScrollbarThumb(_))
                | Some(HitTarget::ScrollbarTrack(_))
                | Some(HitTarget::Merge(
                    crate::merge_view::MergeHit::Thumb(_) | crate::merge_view::MergeHit::Bar(_)
                ))
        ) && !crate::tooltip::blocked_by_mode(&*cx.input_mode)
            && !*cx.show_help
            && cx.preferences.is_none()
            && cx.hovered_repo_menu_row.is_none())
        .then_some((col, row)),
    );

    // While a remote menu is open only its own rows carry tooltips.
    let remote_menu_open = cx.remote_ui.menu.is_some();
    cx.tooltip.pointer_moved(
        match (&moved_hit, *cx.hovered_commit) {
            (Some(HitTarget::RemoteMenuItem(i)), _) => {
                Some(crate::tooltip::TooltipKey::RemoteMenuRow(*i))
            }
            _ if remote_menu_open => None,
            (Some(HitTarget::Hosting(HostingHit::LinkedPrChip | HostingHit::NewPrChip)), _) => {
                Some(crate::tooltip::TooltipKey::LinkedPrChip)
            }
            (Some(HitTarget::DiffBarButton(b)), _) => Some(crate::tooltip::TooltipKey::DiffBar(*b)),
            (Some(HitTarget::CommitAuthorName), _) => {
                Some(crate::tooltip::TooltipKey::CommitAuthor)
            }
            (Some(HitTarget::Toolbar(b @ (ToolbarButton::Undo | ToolbarButton::Redo))), _) => Some(
                crate::tooltip::TooltipKey::UndoRedo(undo_redo_direction(*b)),
            ),
            (Some(t), _) if crate::tooltip::hint_for(t).is_some() => {
                crate::tooltip::hint_for(t).map(crate::tooltip::TooltipKey::Hint)
            }
            _ if crate::flow::view::hover_key().is_some() => {
                crate::flow::view::hover_key().map(crate::tooltip::TooltipKey::FlowBranch)
            }
            _ if crate::tag_where_ui::hover_key().is_some() => {
                crate::tag_where_ui::hover_key().map(crate::tooltip::TooltipKey::TagWhere)
            }
            (_, Some((id, c, ..))) if c != LogColumn::Graph => {
                Some(crate::tooltip::TooltipKey::Commit(id, c))
            }
            _ => None,
        },
        (col, row),
        Instant::now(),
    );

    // Ref-popup open/retarget/close: deliberately NOT a mirror of
    // `hovered_commit` every frame like the other per-column tooltips (see
    // `RefPopupState`). A keyboard-opened popup ignores the mouse; a
    // hover-opened one stays open while the cursor is over the triggering row
    // or the popup, and closes once it is neither.
    //
    // `inside_popup` is checked against `ref_popup_rect`, which only the
    // *next* draw refreshes, but the drain loop handles every queued event
    // before that draw: a burst of `Moved` events would see a stale or absent
    // rect and close the popup before it ever renders. So `ref_popup_rect` is
    // recomputed immediately below whenever the popup opens or retargets; draw
    // overwrites it with the exact rect.
    let inside_popup = cx
        .ref_popup_rect
        .is_some_and(|r| r.contains((col, row).into()));
    let hovering_overflow_ref =
        cx.hovered_commit
            .and_then(|(commit_id, hover_col, _hcol, hrow)| {
                if hover_col != LogColumn::Ref {
                    return None;
                }
                let pills = cx
                    .refs_result
                    .as_ref()
                    .map(|refs| ref_pills_for_commit(commit_id, refs))
                    .unwrap_or_default();
                // Anchored to the Ref column's own left edge, not the raw mouse x: a
                // dropdown pinned under the item that opened it, not wherever the cursor
                // sat inside a possibly-wide cell.
                let ref_col_x = offsets
                    .iter()
                    .find(|(c, _, _)| *c == LogColumn::Ref)
                    .map(|(_, x, _)| *x)?;
                (pills.len() > 1).then_some((commit_id, ref_col_x, hrow, pills.len()))
            });
    let mut opened_or_retargeted: Option<(u16, u16, usize)> = None;
    match &*cx.ref_popup {
        Some(state) if state.opened_via_keyboard => {}
        Some(state) => {
            if inside_popup {
                // Stay exactly as-is.
            } else if let Some((commit_id, acol, arow, count)) = hovering_overflow_ref {
                if commit_id != state.commit_id {
                    *cx.ref_popup = Some(RefPopupState {
                        commit_id,
                        selected: 0,
                        opened_via_keyboard: false,
                        anchor_col: acol,
                        anchor_row: arow,
                    });
                    opened_or_retargeted = Some((acol, arow, count));
                }
            } else {
                *cx.ref_popup = None;
            }
        }
        None => {
            if let Some((commit_id, acol, arow, count)) = hovering_overflow_ref {
                *cx.ref_popup = Some(RefPopupState {
                    commit_id,
                    selected: 0,
                    opened_via_keyboard: false,
                    anchor_col: acol,
                    anchor_row: arow,
                });
                opened_or_retargeted = Some((acol, arow, count));
            }
        }
    }
    if let Some((acol, arow, count)) = opened_or_retargeted {
        // A generous, unclamped estimate of where the popup will appear, so
        // same-batch events recognize "inside" the instant it opens; draw
        // overwrites it with the exact rect next frame.
        *cx.ref_popup_rect = Some(Rect::new(acol, arow + 1, 60, (count as u16).max(1)));
    }

    // One predicate answers "does this light up?"
    // and "does a press here start a resize?" (see
    // `resize_grab_at`), and the shape below is
    // derived from the same answer.
    let resize_grab = resize_grab_at(
        &*cx.hit_targets,
        col,
        row,
        *cx.panes_area,
        divider1_x,
        divider2_x,
        *cx.left_pane_collapsed,
        *cx.right_pane_collapsed,
    );
    *cx.hovered_resize_strip = resize_grab.map(|(_, rect)| rect);

    // What the pointer is over that is clickable but not in `hit_targets`:
    // a repository tab (or its cross), then a list row found by geometry.
    // Folded into the one target the shape rule reads, so a tab and a button
    // are answered by the same arm.
    let virtual_hit = cx
        .hovered_tab_index
        .or(*cx.hovered_close_tab)
        .map(HitTarget::RepoTab)
        .or_else(|| {
            // A Refs folder row folds on a click: a control, the hand.
            cx.hovered_ref_folder
                .is_some()
                .then_some(HitTarget::VirtualHand)
        })
        .or_else(|| {
            // A changed-files row, not the Log's rows: the arrow, lit.
            cx.hovered_status_row
                .is_some()
                .then_some(HitTarget::ListRow)
        })
        .or_else(|| super::pointer::status_bar_hand(col, row).then_some(HitTarget::VirtualHand));
    let shape_hit = moved_hit.clone().or(virtual_hit);
    let shape = desired_cursor_shape(
        shape_hit.as_ref(),
        cx.hovered_ref_row.as_deref(),
        *cx.dragging_divider,
        *cx.dragging_log_boundary,
        cx.ref_drag.as_ref().is_some_and(|d| d.started),
        resize_grab.map(|(grab, _)| grab),
    );
    // What a widget registered when it painted (an input box: the I-beam; a
    // combobox, a button, a checkbox: the hand), below every resize, carry
    // and hit-target answer.
    let shape = crate::pointer_map::over(shape, col, row);
    if shape != *cx.last_cursor_shape {
        cursor::emit_cursor_shape(cx.terminal.backend_mut(), shape)
            .map_err(exit_log::io_context("setting the pointer shape"))?;
        *cx.last_cursor_shape = shape;
    }
    Ok(Flow::Continue)
}
