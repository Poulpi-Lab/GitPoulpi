//! A press of the left button. `handle` resolves the target and the modal layers;
//! the menus and the panes underneath live in `mouse_left_menus` and
//! `mouse_left_panes`.

use super::commands::{self, CommandCtx, Flow};
use super::menu_actions;
use super::mouse_left_menus;
use super::mouse_left_panes;
use super::mouse_prelude::{list_bar_offset, list_bar_regions, set_list_bar_offset, Geo};
use crate::event_loop::*;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// What the press resolved to, handed down the click handlers.
pub(super) struct Click {
    pub(super) hit: Option<HitTarget>,
    pub(super) resize_grab: Option<ResizeGrab>,
    pub(super) inside_ref_popup: bool,
}

/// Whether the target is a row of a popup menu (`OpenMenu`): what a press
/// may hit while one is open without closing it.
fn is_open_menu_row(hit: Option<&HitTarget>) -> bool {
    matches!(
        hit,
        Some(
            HitTarget::RefContextMenuItem(_)
                | HitTarget::LogContextMenuItem(_)
                | HitTarget::WipContextMenuItem(_)
                | HitTarget::FileContextMenuItem(_)
                | HitTarget::FolderContextMenuItem(_)
                | HitTarget::PaneContextMenuItem(_)
                | HitTarget::DiffHunkContextMenuItem(_)
                | HitTarget::HistoryContextMenuItem(_)
                | HitTarget::ColumnMenuItem(_)
                | HitTarget::ColumnMenuAction(_)
                | HitTarget::CommitColumn(
                    crate::commit_column::ColumnHit::ColumnsMenuItem(_)
                        | crate::commit_column::ColumnHit::ColumnsMenuClose
                ),
        )
    )
}

/// A press of the left button: the modal layers first, then the targets the paint
/// walk registered, then the panes underneath.
pub(super) fn handle(
    cx: &mut CommandCtx<'_>,
    g: &Geo,
    mouse: MouseEvent,
    forced_hit: &mut Option<HitTarget>,
) -> io::Result<Flow> {
    let Geo {
        col,
        row,
        divider1_x,
        divider2_x,
        ..
    } = *g;
    // `forced_hit` is an overflow-menu choice
    // (see the top of this drain loop): the
    // hidden button this click stands for.
    let hit = forced_hit
        .take()
        .or_else(|| find_hit_target(&*cx.hit_targets, col, row));
    // The same predicate the hover highlight and
    // the resize pointer use: a press arms a drag
    // exactly where -- and only where -- the strip
    // lit up. See `resize_grab_at`.
    let resize_grab = resize_grab_at(
        &*cx.hit_targets,
        col,
        row,
        *cx.panes_area,
        divider1_x,
        divider2_x,
        *cx.left_pane_collapsed,
        *cx.right_pane_collapsed,
    )
    .map(|(grab, _)| grab);
    // Any new press begins a new gesture: a drag armed by an earlier press is
    // stale. The arming below re-sets it when this press is on a ref.
    *cx.ref_drag = None;
    *cx.dragging_list_bar = None;
    // The hosting menu is modal while open: a
    // click outside it closes it and is swallowed.
    if cx.hosting.modal_open()
        && !matches!(
            hit,
            Some(HitTarget::Hosting(
                HostingHit::MenuItem(_) | HostingHit::PickerItem(_)
            ))
        )
    {
        cx.hosting.menu = None;
        cx.hosting.close_picker();
        return Ok(Flow::Handled);
    }
    // One rule for every popup menu of `OpenMenu`: a press that is not on one
    // of its own rows closes it and does nothing else. This runs before every
    // target the panes and the diff view register (their buttons, lines,
    // scrollbars...), which would otherwise act with the menu left open.
    if cx.open_menu.closes_on_press_outside() && !is_open_menu_row(hit.as_ref()) {
        cx.open_menu.close();
        return Ok(Flow::Handled);
    }
    // Any click leaves a search box.
    if !matches!(hit, Some(HitTarget::Hosting(HostingHit::Row(..)))) {
        cx.hosting.editing = None;
    }
    // A list's scrollbar (Log, Refs sections,
    // commit files, Status sections, WIP all-files):
    // a press on the thumb grabs it, a press on the
    // track jumps there and keeps dragging. Only
    // with nothing modal or popped up over it, and
    // never on a divider's grab strip.
    if matches!(*cx.input_mode, InputMode::Normal)
        && resize_grab.is_none()
        && !cx.open_menu.closes_on_press_outside()
    {
        let pressed = list_bar_regions(cx).into_iter().find_map(|region| {
            let offset = list_bar_offset(cx, region.bar);
            list_bar_press(region, offset, col, row).map(|press| (region.bar, press))
        });
        if let Some((bar, press)) = pressed {
            if let Some(offset) = press.jump_to {
                set_list_bar_offset(cx, bar, offset);
            }
            if !press.arrow {
                *cx.dragging_list_bar = Some((bar, press.grab));
            }
            return Ok(Flow::Handled);
        }
    }
    // The commit column: a click on a row picks that commit, the cross
    // closes it; a press anywhere else gives the keyboard back to the view.
    if matches!(*cx.input_mode, InputMode::Normal) && !cx.open_menu.is_open() {
        if let Some(flow) = commands::commit_column::on_press(cx, hit.as_ref(), col, row) {
            return Ok(flow);
        }
    }
    // The File History split: which half has the keyboard, and a row click.
    if matches!(*cx.input_mode, InputMode::Normal) && !cx.open_menu.is_open() {
        let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
        if let Some(flow) = commands::history::on_press(cx, hit.as_ref(), col, row, shift) {
            return Ok(flow);
        }
    }
    // A click anywhere outside a plain single-line field releases its typing
    // focus (as the composer's Summary/Description fields do), checked once up
    // front so no click site added later can forget it: otherwise the field
    // keeps the keyboard and a shortcut like `j` or `3` is typed into it.
    //
    // The Refs filter's draft is committed like Enter, not discarded like Esc:
    // the filter stays applied, only the *typing* focus is lost. The find bar
    // and the commit-file filter type straight into their own persistent
    // String (the find bar's query is discarded on its own Esc, as on an
    // outside click here), so there is nothing to commit.
    if let InputMode::FilteringRefs(draft) = &*cx.input_mode {
        if !cx.refs_filter_area.contains((col, row).into()) {
            *cx.refs_filter = draft.clone();
            *cx.input_mode = InputMode::Normal;
        }
    }
    if matches!(*cx.input_mode, InputMode::SearchingLog(_))
        && !cx
            .log_search_bar_area
            .is_some_and(|r| r.contains((col, row).into()))
    {
        *cx.input_mode = InputMode::Normal;
    }
    if *cx.commit_filter_focus && !matches!(hit, Some(HitTarget::CommitFilterField)) {
        *cx.commit_filter_focus = false;
    }
    // The Confirm popup is modal: while it's open,
    // every left click either hits one of its two
    // buttons or is swallowed (no click-through to
    // whatever's underneath, same as the palette).
    if let InputMode::Confirm { action, .. } = &*cx.input_mode {
        match hit {
            Some(HitTarget::ConfirmButton(true)) => {
                let action = action.clone();
                *cx.input_mode = InputMode::Normal;
                // One place decides what a
                // confirmed action does; the
                // keyboard arm above calls the
                // same one, so `y` and the button
                // cannot drift apart.
                if let Some(message) = confirm_now(
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
            }
            _ => {
                *cx.input_mode = InputMode::Normal;
                *cx.last_message = Some(msg::input::CANCELED.get().into());
            }
        }
        return Ok(Flow::Handled);
    }
    // The remote popup and menus (`remotes_ui.rs`).
    // The popup is modal: a click lands on it or
    // nowhere. A menu runs the row clicked, and a
    // click anywhere else dismisses it without
    // falling through.
    if cx.remote_ui.form.is_some() {
        // A click outside the popup closes a pristine form; a dirty one
        // swallows it and the cross blinks.
        if remote_hit_at(&*cx.hit_targets, col, row).is_none() {
            if let Some(FormOutcome::Close) = cx
                .remote_ui
                .form
                .as_ref()
                .and_then(|f| f.on_outside_click(col, row))
            {
                cx.remote_ui.form = None;
            }
            return Ok(Flow::Handled);
        }
        if let Some(HitTarget::RemoteForm(h)) = remote_hit_at(&*cx.hit_targets, col, row).as_ref() {
            let outcome = cx.remote_ui.form.as_mut().map(|f| {
                let outcome = f.on_click(*h);
                f.field_press(col, row);
                outcome
            });
            match outcome {
                Some(FormOutcome::Close) => cx.remote_ui.form = None,
                Some(FormOutcome::Submit(submit)) => {
                    cx.remote_ui.form = None;
                    *cx.last_message = Some(submit_remote_form(
                        submit,
                        &mut *cx.remote_ui,
                        cx.link,
                        &mut *cx.hidden_from_graph,
                        &*cx.solo_refs,
                    ));
                }
                _ => {}
            }
        }
        return Ok(Flow::Handled);
    }
    if cx.remote_ui.menu.is_some() {
        if let Some(HitTarget::RemoteMenuItem(idx)) =
            remote_hit_at(&*cx.hit_targets, col, row).as_ref()
        {
            let outcome = cx.remote_ui.menu.as_mut().map(|m| m.activate(*idx));
            match outcome {
                Some(MenuOutcome::Run(action)) => {
                    cx.remote_ui.menu = None;
                    apply_remote_ui_action(
                        action,
                        RemoteUiCtx {
                            ui: &mut *cx.remote_ui,
                            refs: cx.refs_result.as_deref().unwrap_or(&[]),
                            link: cx.link,
                            clipboard: &mut *cx.clipboard,
                            input_mode: &mut *cx.input_mode,
                            last_message: &mut *cx.last_message,
                            hidden_from_graph: &mut *cx.hidden_from_graph,
                            solo_refs: &*cx.solo_refs,
                        },
                    );
                }
                Some(MenuOutcome::Close) => cx.remote_ui.menu = None,
                _ => {}
            }
        } else {
            cx.remote_ui.menu = None;
        }
        return Ok(Flow::Handled);
    }
    match &hit {
        // A `⋯`: the menu of the
        // buttons its bar had no room for, from
        // the list the last frame drew.
        Some(HitTarget::BarOverflow(bar)) => {
            if let Some((_, rect, items)) = cx.overflow_slots.iter().find(|(b, _, _)| b == bar) {
                *cx.open_menu = OpenMenu::Overflow(OverflowMenuState {
                    button: *rect,
                    menu: overflow_menu_for(items, *rect),
                });
            }
            return Ok(Flow::Handled);
        }
        Some(HitTarget::ToolbarDropdown(ToolbarButton::Push)) => {
            let (c, r) = cx.remote_ui.push_anchor();
            *cx.last_message =
                cx.remote_ui
                    .open_push_menu(cx.refs_result.as_deref().unwrap_or(&[]), c, r);
            return Ok(Flow::Handled);
        }
        Some(HitTarget::ToolbarDropdown(ToolbarButton::Undo)) => {
            cx.repo_dialogs
                .open_safety(cx.link, crate::safety_panel::Tab::History, None);
            return Ok(Flow::Handled);
        }
        Some(HitTarget::ToolbarDropdown(_)) => {
            let (c, r) = cx.remote_ui.pull_anchor();
            cx.remote_ui
                .open_pull_menu(cx.refs_result.as_deref().unwrap_or(&[]), c, r);
            return Ok(Flow::Handled);
        }
        Some(HitTarget::RefRemotesBadge(branch)) => {
            *cx.focus = Focus::Refs;
            *cx.active_ref_section = SidebarSection::Local;
            *cx.last_message = crate::multi_remote_badge::open_branch_remotes(
                &*cx.remote_ui,
                cx.refs_result.as_deref().unwrap_or(&[]),
                branch,
                &cx.link.activity_handle().raw.requester(),
                &mut *cx.input_mode,
            );
            return Ok(Flow::Handled);
        }
        Some(HitTarget::AddRemoteButton) => {
            cx.remote_ui.form = Some(RemoteForm::add(cx.remote_ui.names()));
            return Ok(Flow::Handled);
        }
        Some(HitTarget::LfsInstallLink) => {
            *cx.last_message = Some(open_lfs_install_page());
            return Ok(Flow::Handled);
        }
        _ => {}
    }
    // A click outside the ref-reveal popup closes it (click-away) but still
    // falls through to the pane underneath. A click *inside* it (on a
    // `TooltipRefPill` row or not) is consumed and never reaches the pane.
    let inside_ref_popup = cx
        .ref_popup_rect
        .is_some_and(|r| r.contains((col, row).into()));
    if cx.ref_popup.is_some() && !inside_ref_popup {
        *cx.ref_popup = None;
    }

    let click = Click {
        hit,
        resize_grab,
        inside_ref_popup,
    };
    if diff_targets(cx, g, mouse, &click)? == Flow::Handled
        || third_pane_controls(cx, g, mouse, &click)? == Flow::Handled
        || mouse_left_menus::handle(cx, g, mouse, &click)? == Flow::Handled
    {
        return Ok(Flow::Handled);
    }
    mouse_left_panes::handle(cx, g, mouse, &click)
}

/// The diff view's own targets: the search bar, the body and bar buttons,
/// scrollbars and the hunk rows.
pub(super) fn diff_targets(
    cx: &mut CommandCtx<'_>,
    g: &Geo,
    _mouse: MouseEvent,
    click: &Click,
) -> io::Result<Flow> {
    let Geo { col, row, .. } = *g;
    let hit = click.hit.clone();
    if let Some(HitTarget::SearchBarButton(button)) = hit {
        // Routed through the same `step`/`clamp`
        // helpers the keyboard uses, so ↑/↓ and
        // Enter/Shift+Enter can't disagree.
        if let InputMode::SearchingLog(search) = &mut *cx.input_mode {
            match button {
                SearchBarButton::Close => *cx.input_mode = InputMode::Normal,
                SearchBarButton::Prev | SearchBarButton::Next => {
                    let hits = log_search_hits(
                        &*cx.log_result,
                        &*cx.refs_result,
                        &*cx.solo_refs,
                        &search.query,
                    );
                    search.current = log_search::step(
                        search.current,
                        hits.len(),
                        button == SearchBarButton::Next,
                    );
                    if let Some(&node_index) = hits.get(search.current) {
                        cx.log_state
                            .select(Some(cx.log_order.shown_display_index(node_index)));
                        *cx.log_follow = true;
                    }
                }
            }
        }
    } else if let Some(HitTarget::DiffBodyButton(button)) = hit {
        if let Some(s) = cx.diff_view_state.as_mut() {
            let (action, hunk, range) = button.action();
            *cx.last_message = Some(apply_diff_stage_at(
                action,
                s,
                &*cx.diff_rows,
                hunk,
                range,
                cx.link,
                &mut *cx.input_mode,
            ));
        }
        return Ok(Flow::Handled);
    } else if let Some(HitTarget::ScrollbarThumb(kind)) = hit {
        // Only start a drag -- the thumb itself
        // never moves on a plain mouse-down, only
        // once the drag actually moves (below).
        let grab = match (kind, cx.diff_view_state.as_ref()) {
            (ScrollBar::LogHorizontal, _) => {
                let geo = cx.log_geo();
                scrollbar::Track::horizontal(
                    *cx.log_area,
                    usize::from(geo.content_w),
                    usize::from(geo.h_offset),
                )
                .map(|t| col as i32 - t.rect.x as i32 - t.thumb_start as i32)
            }
            (ScrollBar::DiffVertical, Some(s)) => {
                scrollbar::Track::vertical(*cx.diff_body_area, *cx.diff_rendered_len, s.scroll)
                    .map(|t| row as i32 - t.rect.y as i32 - t.thumb_start as i32)
            }
            (ScrollBar::DiffHorizontal, Some(s)) => {
                scrollbar::Track::horizontal(*cx.diff_body_area, *cx.diff_longest_line, s.h_offset)
                    .map(|t| col as i32 - t.rect.x as i32 - t.thumb_start as i32)
            }
            _ => None,
        };
        if let Some(grab) = grab {
            *cx.dragging_scrollbar = Some((kind, grab));
        }
        return Ok(Flow::Handled);
    } else if let Some(HitTarget::ScrollbarTrack(kind)) = hit {
        match kind {
            ScrollBar::DiffVertical => {
                let track = scrollbar::Track::vertical(
                    *cx.diff_body_area,
                    *cx.diff_rendered_len,
                    cx.diff_view_state.as_ref().map_or(0, |s| s.scroll),
                );
                if let (Some(track), Some(s)) = (track, cx.diff_view_state.as_mut()) {
                    let pos = (row as i32 - track.rect.y as i32).max(0) as u16;
                    let viewport = track.rect.height as usize;
                    match track.press(pos) {
                        scrollbar::TrackPress::Arrow(step) => {
                            let max_scroll = cx.diff_rendered_len.saturating_sub(viewport);
                            s.scroll =
                                (s.scroll as isize + step).clamp(0, max_scroll as isize) as usize;
                        }
                        scrollbar::TrackPress::Grab { grab, .. } => {
                            s.scroll = track.drag_offset(
                                *cx.diff_rendered_len,
                                viewport,
                                grab,
                                i32::from(pos),
                            );
                            *cx.dragging_scrollbar = Some((ScrollBar::DiffVertical, grab));
                        }
                    }
                }
            }
            ScrollBar::DiffHorizontal => {
                let track = scrollbar::Track::horizontal(
                    *cx.diff_body_area,
                    *cx.diff_longest_line,
                    cx.diff_view_state.as_ref().map_or(0, |s| s.h_offset),
                );
                if let (Some(track), Some(s)) = (track, cx.diff_view_state.as_mut()) {
                    let pos = (col as i32 - track.rect.x as i32).max(0) as u16;
                    match track.press(pos) {
                        scrollbar::TrackPress::Arrow(step) => s.scroll_h(step),
                        scrollbar::TrackPress::Grab { grab, .. } => {
                            s.h_offset = track.drag_offset(
                                *cx.diff_longest_line,
                                track.rect.width as usize,
                                grab,
                                i32::from(pos),
                            );
                            *cx.dragging_scrollbar = Some((ScrollBar::DiffHorizontal, grab));
                        }
                    }
                }
            }
            ScrollBar::LogHorizontal => {
                let geo = cx.log_geo();
                let track = scrollbar::Track::horizontal(
                    *cx.log_area,
                    usize::from(geo.content_w),
                    usize::from(geo.h_offset),
                );
                if let Some(track) = track {
                    let pos = (col as i32 - track.rect.x as i32).max(0) as u16;
                    match track.press(pos) {
                        scrollbar::TrackPress::Arrow(step) => {
                            *cx.log_h_offset =
                                crate::log_table::scrolled(&geo, i32::try_from(step).unwrap_or(0));
                        }
                        scrollbar::TrackPress::Grab { grab, .. } => {
                            let to = track.drag_offset(
                                usize::from(geo.content_w),
                                track.rect.width as usize,
                                grab,
                                i32::from(pos),
                            );
                            *cx.log_h_offset = u16::try_from(to)
                                .unwrap_or(u16::MAX)
                                .min(geo.max_h_offset());
                            *cx.dragging_scrollbar = Some((ScrollBar::LogHorizontal, grab));
                        }
                    }
                }
            }
            ScrollBar::DiffMinimap => {
                // A click on the minimap centers the
                // view on the line it shows there.
                if let (Some(rect), Some(s)) = (
                    minimap::rect_in(*cx.diff_body_area),
                    cx.diff_view_state.as_mut(),
                ) {
                    let viewport = cx.diff_body_area.height.saturating_sub(2) as usize;
                    let line = minimap::line_at(rect, *cx.diff_rendered_len, row);
                    s.scroll = minimap::top_for(line, *cx.diff_rendered_len, viewport);
                    // Dragging along it keeps
                    // scrubbing.
                    *cx.dragging_scrollbar = Some((ScrollBar::DiffMinimap, 0));
                }
            }
            ScrollBar::FileInsightVertical
            | ScrollBar::FileInsightHorizontal
            | ScrollBar::BlameMinimap
            | ScrollBar::CommitDescription => {}
        }
        return Ok(Flow::Handled);
    } else if let Some(HitTarget::DiffRow(index)) = hit {
        // A plain click only lands the cursor and
        // drops any open selection; it is the drag
        // (below) that sweeps a new one, so a
        // mis-click can't quietly leave a
        // one-line selection behind.
        if let Some(s) = cx.diff_view_state.as_mut() {
            s.cursor = index;
            s.selection_anchor = None;
            // The same press may
            // start a text selection (a drag) or,
            // doubled, select the word under it.
            let now = Instant::now();
            let double = cx.last_click.is_some_and(|(t, c, r)| {
                now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
            });
            *cx.last_click = Some((now, col, row));
            *cx.text_drag = None;
            if let Some(render) = cx.diff_render.as_deref() {
                let inner = cx.diff_body_area.inner(Margin::new(1, 1));
                let side = diff_view::text_side_at(render, inner, col);
                let pos = diff_view::text_pos_at(render, inner, s.scroll, side, col, row);
                *cx.text_drag = text_select::press(
                    text_select::TextView::Diff,
                    side,
                    pos,
                    double,
                    &render.text.side(side),
                    &mut s.text_sel,
                );
            }
        }
        *cx.diff_line_drag = Some(index);
        return Ok(Flow::Handled);
    } else if let Some(HitTarget::DiffBarButton(button)) = hit {
        if let Some(reason) = button.pending_reason() {
            // Disabled, but never silent: say what
            // it needs, rather than eating the click.
            *cx.last_message = Some(reason.to_string());
        } else if matches!(
            button,
            DiffBarButton::History | DiffBarButton::ColumnFile | DiffBarButton::ColumnAll
        ) {
            // The `File | All` selector of the column left of the view: a
            // click on a segment picks that list and shows the column; a
            // click on the segment already selected while the column is
            // shown hides it (modifiers change nothing). The palette's
            // `View: Commit Column ...` rows and `Alt+C` are the keyboard
            // twins.
            match button {
                DiffBarButton::ColumnFile => {
                    commands::commit_column::select(cx, crate::commit_column::ColumnMode::File)
                }
                DiffBarButton::ColumnAll => {
                    commands::commit_column::select(cx, crate::commit_column::ColumnMode::All)
                }
                _ => commands::commit_column::toggle(cx),
            }
        } else if commands::history::is_open(cx)
            && matches!(
                button,
                DiffBarButton::FileView | DiffBarButton::DiffView | DiffBarButton::Blame
            )
        {
            // In the File History these choose what the right side shows of
            // the selected commit, and stay in the History.
            let mode = match button {
                DiffBarButton::FileView => crate::history_view::ViewMode::File,
                DiffBarButton::Blame => crate::history_view::ViewMode::Blame,
                _ => crate::history_view::ViewMode::Diff,
            };
            commands::history::set_mode(cx, mode);
        } else if button == DiffBarButton::EditWorkingFile {
            // From either bar.
            match edit_target(
                *cx.central_view,
                cx.file_insight.as_ref(),
                cx.diff_view_state.as_ref(),
            ) {
                Ok(path) => {
                    if let Err(why) = open_editor(
                        cx.repo_root,
                        &path,
                        head_oid(cx.refs_result.as_deref().unwrap_or(&[])),
                        cx.link,
                        &mut *cx.editor,
                        &mut *cx.file_insight,
                        &*cx.diff_view_state,
                    ) {
                        *cx.last_message = Some(why);
                    }
                }
                Err(why) => *cx.last_message = Some(why.to_string()),
            }
        } else if *cx.central_view == CentralView::File {
            // The File View's bar --
            // same buttons, its own meaning.
            if let Some(message) = apply_file_bar_button(
                button,
                &mut *cx.file_insight,
                &mut *cx.diff_view_state,
                cx.link,
                cx.diff_config_path,
            ) {
                *cx.last_message = Some(message);
            }
        } else if let Some(s) = cx.diff_view_state.as_mut() {
            match button {
                // Handled above, from either bar.
                DiffBarButton::DiffView
                | DiffBarButton::EditWorkingFile
                | DiffBarButton::ColumnFile
                | DiffBarButton::ColumnAll => {}
                DiffBarButton::Mode(m) => {
                    persist_diff_view_toggle(DiffViewToggle::Mode(m), s, cx.diff_config_path);
                }
                DiffBarButton::ToggleMinimap => {
                    *cx.last_message = Some(toggle_minimap(cx.diff_config_path));
                }
                DiffBarButton::Encoding => {
                    crate::diff_encoding_menu::open(s.encoding);
                }
                DiffBarButton::ToggleWordWrap => {
                    persist_diff_view_toggle(
                        DiffViewToggle::ToggleWordWrap,
                        s,
                        cx.diff_config_path,
                    );
                }
                DiffBarButton::ToggleWhitespace => {
                    persist_diff_view_toggle(
                        DiffViewToggle::ToggleWhitespace,
                        s,
                        cx.diff_config_path,
                    );
                }
                DiffBarButton::ToggleHunksOnly => {
                    persist_diff_view_toggle(
                        DiffViewToggle::ToggleHunksOnly,
                        s,
                        cx.diff_config_path,
                    );
                }
                DiffBarButton::NextHunk | DiffBarButton::PrevHunk => {
                    if let Err(why) = diff_view::jump_to_hunk(
                        s,
                        &*cx.diff_hunk_starts,
                        button == DiffBarButton::NextHunk,
                        cx.diff_body_area.height.saturating_sub(2),
                    ) {
                        *cx.last_message = Some(why.into());
                    }
                }
                DiffBarButton::FileView | DiffBarButton::Blame | DiffBarButton::History => {
                    let kind = match button {
                        DiffBarButton::Blame => FileInsightKind::Blame,
                        DiffBarButton::History => FileInsightKind::History,
                        _ => FileInsightKind::Content,
                    };
                    // `s.path`, not a lookup into
                    // `commit_files`: that list is only the
                    // selected *commit*'s files, so indexing
                    // it while a working-tree half is open
                    // would blame a different file entirely.
                    let request =
                        diff_insight_request(s, kind, cx.refs_result.as_deref().unwrap_or(&[]));
                    open_insight_request(request, cx.link, &mut *cx.file_insight);
                }
            }
        }
        return Ok(Flow::Handled);
    }
    Ok(Flow::Continue)
}

/// A press on a property of the commit-detail pane: the second press of a
/// double click opens the Amend popup over that property (the palette's
/// `Commit: Edit ...` entries are the keyboard twin).
fn detail_part_click(cx: &mut CommandCtx<'_>, scope: AmendScope, col: u16, row: u16) {
    let now = Instant::now();
    let double = cx.last_click.is_some_and(|(t, c, r)| {
        now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
    });
    *cx.last_click = Some((now, col, row));
    if double {
        *cx.last_click = None;
        commands::run_command(cx, PaletteCommand::AmendSelectedCommit(scope));
    }
}

/// The third pane's controls, handled before the generic `in_status` fallthrough
/// (which would otherwise swallow these clicks and flip the pane back to
/// working-changes mode).
pub(super) fn third_pane_controls(
    cx: &mut CommandCtx<'_>,
    g: &Geo,
    _mouse: MouseEvent,
    click: &Click,
) -> io::Result<Flow> {
    let hit = click.hit.clone();
    if let Some(target) = hit.clone() {
        let mut handled = true;
        match target {
            // A double click on the commit's message, description or author
            // opens the Amend popup on that property (the single click only
            // selects, as before).
            HitTarget::CommitDetailPart(scope) => detail_part_click(cx, scope, g.col, g.row),
            HitTarget::CommitAuthorName => detail_part_click(cx, AmendScope::Author, g.col, g.row),
            HitTarget::CommitIdLine => {
                commands::run_command(
                    cx,
                    PaletteCommand::CopyCommitField(crate::log_menu::CommitField::ShortSha),
                );
            }
            HitTarget::FileListModeButton(mode) => {
                if *cx.right_panel == RightPanel::CommitDetail {
                    *cx.commit_list_mode = mode;
                } else {
                    *cx.status_list_mode = mode;
                }
            }
            HitTarget::CollapseAllButton { collapse } => {
                if *cx.right_panel == RightPanel::CommitDetail {
                    *cx.collapsed_commit_folders = if collapse {
                        all_folders(&*cx.commit_paths_cache)
                    } else {
                        HashSet::new()
                    };
                } else if collapse {
                    *cx.collapsed_status_folders = all_status_fold_keys(&*cx.status_section_lines);
                } else {
                    cx.collapsed_status_folders.clear();
                }
            }
            HitTarget::StatusSectionHeader(section) => {
                toggle_in(&mut *cx.collapsed_status_sections, section);
            }
            HitTarget::StatusFolderRow(path) => {
                toggle_in(&mut *cx.collapsed_status_folders, path);
            }
            HitTarget::StatusBulkButton(section) => {
                cx.link.submit_action(match section {
                    StatusSection::Unstaged => Action::StageAll,
                    StatusSection::Staged => Action::UnstageAll,
                });
            }
            HitTarget::StatusDiscardAllButton => {
                discard_all_changes(
                    cx.status_result.as_ref().ok(),
                    &mut *cx.input_mode,
                    &mut *cx.last_message,
                );
            }
            HitTarget::StatusFolderButton { section, path } => {
                cx.link.submit_action(match section {
                    StatusSection::Unstaged => Action::StageFolder(path.clone()),
                    StatusSection::Staged => Action::UnstageFolder(path.clone()),
                });
            }
            HitTarget::StatusRowButton { section, path } => {
                cx.link.submit_action(match section {
                    StatusSection::Unstaged => Action::StageFile(path.clone()),
                    StatusSection::Staged => Action::UnstageFile(path.clone()),
                });
            }
            HitTarget::AmendCheckbox => {
                *cx.commit_amend = !*cx.commit_amend;
            }
            HitTarget::SkipHooksCheckbox => {
                *cx.commit_skip_hooks = !*cx.commit_skip_hooks;
            }
            HitTarget::AiGenerate => {
                commands::generate_commit_message(cx);
            }
            HitTarget::AiCompose => menu_actions::compose_commits(cx),
            HitTarget::ComposerIssueRef => {
                // Grayed with nothing linked: the message says why.
                let effect = cx.hosting.run_command(
                    HostingCommand::InsertIssueRef,
                    (2, 2),
                    &mut *cx.clipboard,
                );
                commands::apply_hosting_effect(cx, effect);
            }
            HitTarget::StatusExplainButton => commands::explain_wip(cx),
            HitTarget::CommitSummaryField => {
                *cx.composer_focus = Some(ComposerField::Summary);
                *cx.focus = Focus::Status;
            }
            HitTarget::CommitDescriptionField => {
                *cx.composer_focus = Some(ComposerField::Description);
                *cx.focus = Focus::Status;
            }
            HitTarget::CommitFilterField => {
                *cx.commit_filter_focus = true;
            }
            HitTarget::CommitButton => {
                // Unresolved conflicts: guide to them instead of an error.
                if commands::block_on_conflicts(cx, crate::guide_dialog::Blocked::Commit) {
                    return Ok(Flow::Handled);
                }
                let staged_count = cx
                    .status_result
                    .as_ref()
                    .map(|t| section_paths(t, StatusSection::Staged).len())
                    .unwrap_or(0);
                if let Err(reason) = commit_readiness(
                    &*cx.commit_summary,
                    *cx.commit_amend,
                    staged_count,
                    matches!(
                        cx.operation_state,
                        Some(gitpoulpi_core::domain::OperationState::Merge { .. })
                    ),
                    0,
                ) {
                    // Disabled, but never silent --
                    // the same convention as
                    // `DiffBarButton::pending_reason`.
                    *cx.last_message = Some(reason.get().to_string());
                } else {
                    commands::submit_composer(cx);
                }
            }
            HitTarget::ViewAllFilesCheckbox => {
                commands::toggle_view_all_files(cx);
            }
            HitTarget::CommitSortButton => {
                commands::toggle_commit_sort(cx);
            }
            HitTarget::CompareSwap => {
                commands::compare_swap(cx);
            }
            HitTarget::CompareCard(i) => {
                commands::compare_goto_card(cx, i);
            }
            HitTarget::CommitReference(i) => {
                if let Some((url, true)) = cx.commit_ref_urls.get(i) {
                    // Text of the message: ask before opening it.
                    crate::link_confirm::ask(vec![url.clone()]);
                } else if let Some((url, _)) = cx.commit_ref_urls.get(i) {
                    *cx.last_message = Some(match crate::repo_menu::open_url(url) {
                        Ok(()) => msg::input::opened_in_browser(url),
                        Err(err) => crate::activity_log::report_problem_clear(
                            crate::activity_log::ProblemSource::Tool,
                            &err,
                        ),
                    });
                }
            }
            HitTarget::FileContextMenuItem(idx) => {
                // Same `activate` the keyboard's Enter
                // runs: a submenu row opens, `Back`
                // goes back, a leaf runs and closes.
                let outcome = cx
                    .open_menu
                    .file_menu_mut()
                    .map(|(_, _, menu)| menu.activate(idx));
                match outcome {
                    Some(MenuOutcome::Run(action)) => {
                        if let Some((section, path, _)) = cx.open_menu.take_file_menu() {
                            commands::apply_file_menu_action(cx, section, path, action);
                        }
                    }
                    Some(MenuOutcome::Close) => cx.open_menu.close_file_menu(),
                    _ => {}
                }
            }
            HitTarget::FolderContextMenuItem(idx) => {
                let outcome = cx
                    .open_menu
                    .folder_menu_mut()
                    .map(|(_, _, menu)| menu.activate(idx));
                match outcome {
                    Some(MenuOutcome::Run(action)) => {
                        if let Some((section, path, _)) = cx.open_menu.take_folder_menu() {
                            menu_actions::apply_folder_menu_action(cx, section, path, action);
                        }
                    }
                    Some(MenuOutcome::Close) => cx.open_menu.close_folder_menu(),
                    _ => {}
                }
            }
            _ => handled = false,
        }
        if handled {
            return Ok(Flow::Handled);
        }
    }
    Ok(Flow::Continue)
}
