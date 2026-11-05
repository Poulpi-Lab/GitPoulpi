//! Left drag and release: resizes, scrollbar thumbs, selection and drag & drop.

use super::commands::{CommandCtx, Flow};
use super::mouse_prelude::{list_bar_offset, list_bar_regions, set_list_bar_offset, Geo};
use crate::event_loop::*;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// The left button held while the pointer moves: resizes, list bars, the diff
/// view, text selection and a ref being dragged.
pub(super) fn drag(cx: &mut CommandCtx<'_>, g: &Geo, _mouse: MouseEvent) -> io::Result<Flow> {
    let Geo { col, row, .. } = *g;
    // A scrollbar drag, started by a mouse-down on the thumb (see the
    // `ScrollbarThumb` arm above). The track is recomputed every event, not
    // reused from mouse-down: the content can reflow under a long drag (a
    // hunk staged mid-drag shortens the diff). A list's bar works the same
    // way: recompute the region live and move the viewport only.
    if let Some((bar, grab)) = *cx.dragging_list_bar {
        if let Some(region) = list_bar_regions(cx).into_iter().find(|r| r.bar == bar) {
            let offset = list_bar_offset(cx, bar);
            let new = list_bar_drag(region, offset, grab, row);
            set_list_bar_offset(cx, bar, new);
        }
        return Ok(Flow::Handled);
    }
    if let Some((ScrollBar::LogHorizontal, grab)) = *cx.dragging_scrollbar {
        let geo = cx.log_geo();
        if let Some(track) = scrollbar::Track::horizontal(
            *cx.log_area,
            usize::from(geo.content_w),
            usize::from(geo.h_offset),
        ) {
            let pos = col as i32 - track.rect.x as i32;
            let to = track.drag_offset(
                usize::from(geo.content_w),
                track.rect.width as usize,
                grab,
                pos,
            );
            *cx.log_h_offset = u16::try_from(to)
                .unwrap_or(u16::MAX)
                .min(geo.max_h_offset());
        }
        return Ok(Flow::Handled);
    }
    if let Some((kind, grab)) = *cx.dragging_scrollbar {
        match (kind, cx.diff_view_state.as_mut()) {
            (ScrollBar::DiffVertical, Some(s)) => {
                if let Some(track) =
                    scrollbar::Track::vertical(*cx.diff_body_area, *cx.diff_rendered_len, s.scroll)
                {
                    let viewport = track.rect.height as usize;
                    let pos = row as i32 - track.rect.y as i32;
                    s.scroll = track.drag_offset(*cx.diff_rendered_len, viewport, grab, pos);
                }
            }
            (ScrollBar::DiffHorizontal, Some(s)) => {
                if let Some(track) = scrollbar::Track::horizontal(
                    *cx.diff_body_area,
                    *cx.diff_longest_line,
                    s.h_offset,
                ) {
                    let viewport = track.rect.width as usize;
                    let pos = col as i32 - track.rect.x as i32;
                    s.h_offset = track.drag_offset(*cx.diff_longest_line, viewport, grab, pos);
                }
            }
            (ScrollBar::DiffMinimap, Some(s)) => {
                if let Some(rect) = minimap::rect_in(*cx.diff_body_area) {
                    let viewport = cx.diff_body_area.height.saturating_sub(2) as usize;
                    let line = minimap::line_at(
                        rect,
                        *cx.diff_rendered_len,
                        row.clamp(rect.y, rect.bottom().saturating_sub(1)),
                    );
                    s.scroll = minimap::top_for(line, *cx.diff_rendered_len, viewport);
                }
            }
            (
                ScrollBar::FileInsightVertical
                | ScrollBar::FileInsightHorizontal
                | ScrollBar::LogHorizontal
                | ScrollBar::BlameMinimap,
                _,
            )
            | (ScrollBar::CommitDescription, _)
            | (_, None) => {}
        }
        return Ok(Flow::Handled);
    }
    // Crossterm reports a *held* button as `Drag`, never `Moved`, so this arm
    // is the only place a carried ref learns that it has moved, what it is
    // over, and which pointer shape to ask for (the pane divider's highlight
    // froze mid-drag for the same reason, 81574de).
    if let Some(drag) = cx.ref_drag.as_mut() {
        if !drag.started && (col, row) != drag.origin {
            drag.started = true;
            let (oc, orow) = drag.origin;
            drag.source_rect = find_hit_entry(&*cx.hit_targets, oc, orow)
                .and_then(|(rect, target)| match target {
                    HitTarget::LogRefPill { .. } | HitTarget::TooltipRefPill { .. } => Some(rect),
                    _ => None,
                })
                .or_else(|| {
                    sidebar_ref_at(
                        &*cx.sidebar_section_areas,
                        cx.refs_result.as_deref().unwrap_or(&[]),
                        &*cx.refs_filter,
                        *cx.hierarchical_refs,
                        &*cx.collapsed_ref_folders,
                        &*cx.remote_names,
                        |s| cx.sidebar_states.state(s).offset(),
                        oc,
                        orow,
                    )
                    .map(|h| h.rect)
                });
        }
        drag.at = (col, row);
        if drag.started {
            // Pills first, sidebar second, which
            // is also the order they overlap in:
            // the reveal popup's rows are
            // registered hit targets drawn over
            // whatever pane is underneath.
            let under = find_hit_entry(&*cx.hit_targets, col, row)
                .and_then(|(rect, target)| match target {
                    HitTarget::LogRefPill { name, kind }
                    | HitTarget::TooltipRefPill { name, kind } => Some((name, kind, rect)),
                    _ => None,
                })
                .or_else(|| {
                    sidebar_ref_at(
                        &*cx.sidebar_section_areas,
                        cx.refs_result.as_deref().unwrap_or(&[]),
                        &*cx.refs_filter,
                        *cx.hierarchical_refs,
                        &*cx.collapsed_ref_folders,
                        &*cx.remote_names,
                        |s| cx.sidebar_states.state(s).offset(),
                        col,
                        row,
                    )
                    .map(|h| (h.name, h.kind, h.rect))
                });
            drag.over = match under {
                Some((name, kind, rect)) => Some(RefDropCandidate {
                    valid: ref_drop_is_valid(&drag.source, drag.source_kind, &name, kind),
                    name,
                    kind,
                    rect,
                }),
                None => None,
            };
            if *cx.last_cursor_shape != CursorShape::Grab {
                cursor::emit_cursor_shape(cx.terminal.backend_mut(), CursorShape::Grab)?;
                *cx.last_cursor_shape = CursorShape::Grab;
            }
        }
    }
    // Dragging inside the diff sweeps a line selection from where the button
    // went down. The row comes from the body rect and the live scroll, not a
    // hit target, because a drag past the bottom edge must still extend the
    // selection to the last visible row.
    if let (Some(anchor), Some(s)) = (*cx.diff_line_drag, cx.diff_view_state.as_mut()) {
        if let Some(index) =
            diff_view::diff_row_at(*cx.diff_body_area, s.scroll, *cx.diff_rendered_len, row)
        {
            s.selection_anchor = Some(anchor);
            s.cursor = index;
        }
    }
    // ...and the same drag sweeps the text
    // selection, on the side it
    // started on. The File View's twin lives in
    // its own body handler; this one also catches
    // a File View drag that left the body.
    match *cx.text_drag {
        Some(drag) if drag.view == text_select::TextView::Diff => {
            if let (Some(s), Some(render)) =
                (cx.diff_view_state.as_mut(), cx.diff_render.as_deref())
            {
                let inner = cx.diff_body_area.inner(Margin::new(1, 1));
                if let Some(head) =
                    diff_view::text_pos_at(render, inner, s.scroll, drag.side, col, row)
                {
                    s.text_sel = Some(drag.to(head));
                }
            }
        }
        Some(drag) => {
            if let Some(insight) = cx.file_insight.as_mut() {
                if let Some(head) = file_view::text_pos_at(insight, *cx.file_insight_area, col, row)
                {
                    insight.text_sel = Some(drag.to(head));
                }
            }
        }
        None => {}
    }
    if let Some(h) = cx.history.as_mut().filter(|h| h.dragging_split) {
        let pct = h.pct_at(col);
        h.set_split(pct);
        // Re-light the strip where the splitter now is: a held button
        // reports `Drag`, never `Moved`, so the hover path that paints the
        // strip does not run mid-drag, and the rect from the press stays
        // behind as a stale bar inside the view.
        *cx.hovered_resize_strip = Some(h.divider_rect(h.column));
    }
    commands::commit_column::drag(cx, col, row);
    if cx.commit_col.dragging {
        *cx.hovered_resize_strip = cx.commit_col.divider_rect();
    }
    // The header's boundary follows the pointer: its highlight is lit where
    // the widths just written put it (a held button reports `Drag`, never
    // `Moved`, so the hover path does not run).
    if let Some(boundary) = cx.commit_col.dragging_part {
        *cx.hovered_resize_strip = cx.commit_col.boundary_rect(boundary);
    }
    if *cx.dragging_commit_message {
        let a = *cx.commit_message_area;
        let rows = row.saturating_sub(a.y).saturating_sub(1).max(1);
        *cx.commit_message_user_rows = rows.min(commit_detail::max_message_rows(
            *cx.commit_detail_status_height,
        ));
        *cx.hovered_resize_strip = Some(Rect::new(
            a.x + 1,
            a.y + *cx.commit_message_user_rows + 1,
            a.width.saturating_sub(2),
            1,
        ));
    }
    if let Some(section) = *cx.dragging_sidebar_section {
        // The pointer is on the section's new bottom
        // border row: height = distance from its top.
        if let Some((_, area)) = cx.sidebar_section_areas.iter().find(|(s, _)| *s == section) {
            let wanted = row.saturating_sub(area.y).saturating_add(1);
            if let Some(index) = cx
                .sidebar_section_areas
                .iter()
                .position(|(s, _)| *s == section)
            {
                let height = sidebar_layout::clamp_section_height(
                    wanted,
                    *cx.sidebar_layout_avail,
                    &*cx.sidebar_layout_specs,
                    index,
                );
                sidebar_layout::store_height(&mut *cx.sidebar_heights, section.index(), height);
                // Re-light the strip where the border
                // will be next frame: a held button
                // reports `Drag`, not `Moved`.
                *cx.hovered_resize_strip = Some(Rect::new(
                    area.x + 1,
                    area.y + height - 1,
                    area.width.saturating_sub(2),
                    1,
                ));
            }
        }
    }
    match *cx.dragging_divider {
        // Dragged well past `MIN_PANE_PERCENT`
        // rather than merely pinned there: finish
        // the gesture the user was making and
        // collapse the pane instead of leaving a
        // sliver at the floor. The drag ends here
        // -- there is no divider left to keep
        // dragging, only the restore strip.
        Some(1)
            if raw_divider_pct(col, cx.panes_area.x, cx.panes_area.width)
                < DRAG_COLLAPSE_PERCENT =>
        {
            *cx.left_pane_collapsed = true;
            *cx.dragging_divider = None;
            *cx.hovered_resize_strip = None;
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        Some(1) => {
            *cx.divider1_pct =
                compute_divider1_pct(col, cx.panes_area.x, cx.panes_area.width, *cx.divider2_pct);
        }
        Some(2)
            if raw_divider_pct(col, cx.panes_area.x, cx.panes_area.width)
                > 100u16.saturating_sub(DRAG_COLLAPSE_PERCENT) =>
        {
            *cx.right_pane_collapsed = true;
            *cx.dragging_divider = None;
            *cx.hovered_resize_strip = None;
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        Some(2) => {
            *cx.divider2_pct =
                compute_divider2_pct(col, cx.panes_area.x, cx.panes_area.width, *cx.divider1_pct);
        }
        _ => {}
    }
    if let Some(which) = *cx.dragging_divider {
        // Re-light the strip from the percentage just written, at the divider's
        // own column rather than the pointer's (they part company once the drag
        // hits `MIN_PANE_PERCENT`). A held button reports `Drag`, never `Moved`,
        // so the hover path that normally paints this never runs mid-drag.
        *cx.hovered_resize_strip = dragged_divider_strip(
            *cx.panes_area,
            which,
            *cx.divider1_pct,
            *cx.divider2_pct,
            *cx.left_pane_collapsed,
            *cx.right_pane_collapsed,
        );
    }
    if let Some(grip) = *cx.dragging_log_boundary {
        let geo = cx.log_geo();
        if geo.layout == gitpoulpi_config::LogColumnsLayout::Scroll {
            // Only this column changes (the others were pinned at the press)
            // and the table grows or shrinks: the width is the pointer's place
            // in the content less where the column starts.
            cx.log_column_scroll_widths
                .insert(grip.col, scroll_drag_width(&geo, grip.col, col));
        } else if grip.last {
            // The grip after the last column: by how far the pointer travelled
            // since the press, `Message` taking up the difference, never past
            // what the pane has room for (`LogColumnWidths::fit_max`).
            let room = geo.widths.fit_max(
                cx.log_area.width,
                &geo.order,
                &*cx.log_column_overrides,
                grip.col,
            );
            cx.log_column_overrides
                .insert(grip.col, last_grip_width(&grip, col, room));
        } else if let Some(pair) = {
            // Both sides of the boundary, never just the left one (see
            // `boundary_drag_widths`). Each width is already clamped there to
            // its column's floor and to what the pair has between them, so no
            // phantom override wider than the pane can land in the map. The
            // pointer holds the gap cell in `line` mode, one cell before the
            // edge it moves (`column_resize::pointer_to_edge`).
            let edge =
                col + crate::column_resize::pointer_to_edge(crate::column_resize::mode(), true);
            boundary_drag_widths(&geo.offsets, grip.col, edge)
        } {
            for (c, w) in pair {
                cx.log_column_overrides.insert(c, w);
            }
        }
        // Re-light the strip from the widths just written. A held button
        // reports as `Drag`, not `Moved`, so this is the only place the
        // highlight can follow a boundary that is being dragged.
        *cx.hovered_resize_strip = cx.log_geo().grip_rect(*cx.log_area, grip.col);
    }
    if let Some(mut drag) = *cx.dragging_log_header {
        drag.at = (col, row);
        if !drag.started && (col, row) != drag.origin {
            drag.started = true;
        }
        if drag.started {
            let offsets = cx.log_geo().offsets;
            let slot = drop_slot(&offsets, col);
            drag.slot = Some(slot);
            let header_row = cx.log_area.y + 1;
            drag.marker = marker_x(&offsets, slot).map(|x| (x, header_row));
            drag.source = offsets
                .iter()
                .find(|(c, _, _)| *c == drag.col)
                .map(|(_, x, w)| Rect::new(*x, header_row, *w, 1));
            if *cx.last_cursor_shape != CursorShape::Grab {
                cursor::emit_cursor_shape(cx.terminal.backend_mut(), CursorShape::Grab)?;
                *cx.last_cursor_shape = CursorShape::Grab;
            }
        }
        *cx.dragging_log_header = Some(drag);
    }
    Ok(Flow::Continue)
}

/// The left button released: ends a drag and drops what it carried.
pub(super) fn up(cx: &mut CommandCtx<'_>, g: &Geo, _mouse: MouseEvent) -> io::Result<Flow> {
    let Geo {
        col,
        row,
        divider1_x,
        divider2_x,
        ..
    } = *g;
    if let Some(drag) = cx.ref_drag.take() {
        // A press that never moved is a click, and
        // the click handlers already did their
        // work on the way down -- say nothing.
        if drag.started {
            match &drag.over {
                // The drop
                // opens a menu of what to do with
                // the pair, at the pointer (see
                // `ref_drop`). Picking a row runs
                // it through the ref menu's own
                // `apply_ref_context_action`.
                Some(candidate) if candidate.valid => {
                    let entries = drop_menu_entries(
                        &drag.source,
                        drag.source_kind,
                        &candidate.name,
                        candidate.kind,
                        cx.refs_result.as_deref().unwrap_or(&[]),
                        cx.log_result.as_ref().ok(),
                    );
                    if entries.is_empty() {
                        *cx.last_message =
                            Some(msg::input::nothing_to_do(&drag.source, &candidate.name));
                    } else {
                        *cx.ref_drop_source = Some((drag.source.clone(), drag.source_kind));
                        *cx.open_menu = OpenMenu::Ref((
                            SidebarSection::from_ref_kind(candidate.kind),
                            candidate.name.clone(),
                            ContextMenu::new(
                                drop_menu_title(&drag.source, &candidate.name),
                                entries,
                                col,
                                row,
                            ),
                        ));
                    }
                }
                // Released on nothing that can
                // take it. Said out loud, because
                // a gesture that ends in silence
                // reads as a bug.
                _ => {
                    *cx.last_message = Some(msg::input::drag_canceled(&drag.source));
                }
            }
            // A release produces no `Moved`, so
            // nothing else would ever take the
            // open hand back down: restore the
            // shape for where the pointer is now.
            *cx.hovered_ref_row = sidebar_ref_at(
                &*cx.sidebar_section_areas,
                cx.refs_result.as_deref().unwrap_or(&[]),
                &*cx.refs_filter,
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
                |s| cx.sidebar_states.state(s).offset(),
                col,
                row,
            )
            .map(|h| h.name);
            let shape = desired_cursor_shape(
                find_hit_target(&*cx.hit_targets, col, row).as_ref(),
                cx.hovered_ref_row.as_deref(),
                None,
                None,
                false,
                resize_grab_at(
                    &*cx.hit_targets,
                    col,
                    row,
                    *cx.panes_area,
                    divider1_x,
                    divider2_x,
                    *cx.left_pane_collapsed,
                    *cx.right_pane_collapsed,
                )
                .map(|(grab, _)| grab),
            );
            if shape != *cx.last_cursor_shape {
                cursor::emit_cursor_shape(cx.terminal.backend_mut(), shape)
                    .map_err(exit_log::io_context("setting the pointer shape"))?;
                *cx.last_cursor_shape = shape;
            }
        }
    }
    // A drag that ended by an ordinary release
    // (not the collapse-on-drag branches above,
    // which already persisted and cleared this)
    // writes the new width out once, here, rather
    // than on every intermediate `Drag` frame.
    if cx.dragging_divider.is_some() {
        persist_pane_layout(
            cx.session,
            *cx.divider1_pct,
            *cx.divider2_pct,
            cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
            cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
        );
    }
    if let Some(h) = cx.history.as_mut().filter(|h| h.dragging_split) {
        h.dragging_split = false;
        session_store::save_history_split(cx.session, h.split_pct);
    }
    commands::commit_column::end_drag(cx);
    if *cx.dragging_commit_message {
        *cx.dragging_commit_message = false;
        *cx.hovered_resize_strip = None;
        session_store::save_commit_message_rows(cx.session, *cx.commit_message_user_rows);
    }
    if cx.dragging_sidebar_section.take().is_some() {
        session_store::save_sidebar_heights(cx.session, &*cx.sidebar_heights);
        *cx.hovered_resize_strip = None;
    }
    *cx.dragging_divider = None;
    *cx.dragging_log_boundary = None;
    if let Some(drag) = cx.dragging_log_header.take() {
        if let (true, Some(slot)) = (drag.started, drag.slot) {
            crate::log_table::drop_column(
                &mut *cx.log_column_order,
                &*cx.log_column_hidden,
                drag.col,
                slot,
            );
        }
    }
    *cx.diff_line_drag = None;
    *cx.text_drag = None;
    *cx.dragging_scrollbar = None;
    *cx.dragging_list_bar = None;
    // A release produces no `Moved` either, so the highlight a drag left on
    // its boundary is recomputed here for where the pointer ended, with the
    // same predicate hover uses: lit only while the pointer is still on a
    // boundary, whichever surface was dragged.
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
    // A release produces no `Moved`: the resize pointer a drag showed is
    // taken down here for where the pointer ended (it can be far from the
    // strip once the drag hit its limit), not at the next movement.
    let shape = desired_cursor_shape(
        find_hit_target(&*cx.hit_targets, col, row).as_ref(),
        cx.hovered_ref_row.as_deref(),
        None,
        None,
        false,
        resize_grab_at(
            &*cx.hit_targets,
            col,
            row,
            *cx.panes_area,
            divider1_x,
            divider2_x,
            *cx.left_pane_collapsed,
            *cx.right_pane_collapsed,
        )
        .map(|(grab, _)| grab),
    );
    if shape != *cx.last_cursor_shape {
        cursor::emit_cursor_shape(cx.terminal.backend_mut(), shape)
            .map_err(exit_log::io_context("setting the pointer shape"))?;
        *cx.last_cursor_shape = shape;
    }
    Ok(Flow::Continue)
}
