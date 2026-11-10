//! The menus, tooltips, drags and help of the draw closure, painted over everything else.

use super::super::*;

/// A carried commit column header: its title, the pointer, the bar's cell.
pub(in crate::event_loop) type CommitHeaderDrag = (&'static str, (u16, u16), Option<(u16, u16)>);

pub(in crate::event_loop) struct OverlayMenusDrawCtx<'a> {
    pub(in crate::event_loop) ai_ui: &'a AiUi,
    pub(in crate::event_loop) commit_changes:
        &'a std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
    pub(in crate::event_loop) commit_lane_colors: &'a std::collections::HashMap<Oid, Color>,
    pub(in crate::event_loop) diff_line_drag: Option<usize>,
    pub(in crate::event_loop) dragging_commit_message: bool,
    pub(in crate::event_loop) dragging_divider: Option<u8>,
    pub(in crate::event_loop) dragging_list_bar: Option<(ListBar, i32)>,
    pub(in crate::event_loop) dragging_log_boundary: Option<LogBoundaryDrag>,
    pub(in crate::event_loop) dragging_log_header: Option<LogHeaderDrag>,
    /// The commit column's carried header: its title, the pointer and the
    /// insertion bar's cell.
    pub(in crate::event_loop) commit_header_drag: Option<CommitHeaderDrag>,
    pub(in crate::event_loop) dragging_scrollbar: Option<(ScrollBar, i32)>,
    pub(in crate::event_loop) dragging_sidebar_section: Option<SidebarSection>,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) help_filter: &'a String,
    pub(in crate::event_loop) help_popup_inner_height: &'a mut u16,
    pub(in crate::event_loop) help_scroll: &'a mut u16,
    pub(in crate::event_loop) hosting: &'a mut HostingUi,
    pub(in crate::event_loop) hovered_commit:
        Option<(gitpoulpi_core::domain::Oid, LogColumn, u16, u16)>,
    pub(in crate::event_loop) hovered_repo_menu_row: Option<usize>,
    pub(in crate::event_loop) hovered_resize_strip: Option<Rect>,
    pub(in crate::event_loop) hovered_target: &'a Option<HitTarget>,
    pub(in crate::event_loop) input_mode: &'a InputMode,
    pub(in crate::event_loop) journal_state: &'a gitpoulpi_core::domain::JournalState,
    pub(in crate::event_loop) log_area: Rect,
    pub(in crate::event_loop) log_column_hidden: &'a std::collections::HashSet<LogColumn>,
    pub(in crate::event_loop) log_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(in crate::event_loop) mode_hint: &'a Option<String>,
    pub(in crate::event_loop) nav_badge_targets: &'a mut Vec<(char, FocusTarget)>,
    pub(in crate::event_loop) log_widths: &'a LogColumnWidths,
    pub(in crate::event_loop) open_menu: &'a OpenMenu,
    pub(in crate::event_loop) palette_ctx: PaletteContext,
    pub(in crate::event_loop) palette_popup: &'a mut Option<PalettePopup>,
    pub(in crate::event_loop) field_editor: &'a crate::text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) preferences: &'a mut Option<PreferencesState>,
    pub(in crate::event_loop) tab_picker_popup: &'a mut Option<PalettePopup>,
    pub(in crate::event_loop) ref_drag: &'a Option<RefDrag>,
    pub(in crate::event_loop) ref_drop_mark: &'a Option<(String, RefRowKind)>,
    pub(in crate::event_loop) ref_popup: &'a Option<RefPopupState>,
    pub(in crate::event_loop) ref_popup_rect: &'a mut Option<Rect>,
    pub(in crate::event_loop) refs_for_pills: &'a Vec<gitpoulpi_core::domain::Ref>,
    pub(in crate::event_loop) log_layout: &'a crate::graph::LogLayout,
    pub(in crate::event_loop) remote_ui: &'a RemoteUi,
    pub(in crate::event_loop) repo_dialogs: &'a mut RepoDialogs,
    pub(in crate::event_loop) repo_menu: &'a Option<RepoMenuState>,
    pub(in crate::event_loop) repo_menu_rects: &'a mut Vec<Rect>,
    pub(in crate::event_loop) show_help: bool,
    pub(in crate::event_loop) text_drag: &'a Option<text_select::TextDrag>,
    pub(in crate::event_loop) tooltip: &'a mut tooltip::TooltipLifecycle,
    pub(in crate::event_loop) undo_busy: bool,
}

pub(in crate::event_loop) fn draw_overlay_menus(
    frame: &mut Frame,
    ctx: &mut OverlayMenusDrawCtx<'_>,
) {
    let ai_ui = ctx.ai_ui;
    let commit_changes = ctx.commit_changes;
    let commit_lane_colors = ctx.commit_lane_colors;
    let diff_line_drag = ctx.diff_line_drag;
    let dragging_commit_message = ctx.dragging_commit_message;
    let dragging_divider = ctx.dragging_divider;
    let dragging_list_bar = ctx.dragging_list_bar;
    let dragging_log_boundary = ctx.dragging_log_boundary;
    let dragging_log_header = ctx.dragging_log_header;
    let dragging_scrollbar = ctx.dragging_scrollbar;
    let dragging_sidebar_section = ctx.dragging_sidebar_section;
    let frame_hits = &mut *ctx.frame_hits;
    let help_filter = ctx.help_filter;
    let mut help_popup_inner_height = *ctx.help_popup_inner_height;
    let mut help_scroll = *ctx.help_scroll;
    let hosting = &mut *ctx.hosting;
    let hovered_commit = ctx.hovered_commit;
    let hovered_repo_menu_row = ctx.hovered_repo_menu_row;
    let hovered_resize_strip = ctx.hovered_resize_strip;
    let hovered_target = ctx.hovered_target;
    let input_mode = ctx.input_mode;
    let journal_state = ctx.journal_state;
    let log_area = ctx.log_area;
    let log_column_hidden = ctx.log_column_hidden;
    let log_result = ctx.log_result;
    let log_widths = ctx.log_widths;
    let open_menu = ctx.open_menu;
    let preferences = &mut *ctx.preferences;
    let ref_drag = ctx.ref_drag;
    let ref_drop_mark = ctx.ref_drop_mark;
    let ref_popup = ctx.ref_popup;
    let mut ref_popup_rect: Option<Rect>;
    let refs_for_pills = ctx.refs_for_pills;
    let log_layout = ctx.log_layout;
    let remote_ui = ctx.remote_ui;
    let repo_dialogs = &mut *ctx.repo_dialogs;
    let repo_menu = ctx.repo_menu;
    let repo_menu_rects = &mut *ctx.repo_menu_rects;
    let show_help = ctx.show_help;
    let text_drag = ctx.text_drag;
    let tooltip = &mut *ctx.tooltip;
    let undo_busy = ctx.undo_busy;
    // The ref marked for a keyboard drop: every Log pill carrying it is
    // underlined (the sidebar row does its own, `draw/sidebar.rs`).
    if let Some((mark, mark_kind)) = ref_drop_mark.as_ref() {
        for (rect, target) in frame_hits.iter() {
            if let HitTarget::LogRefPill { name, kind } = target {
                if name == mark && kind == mark_kind {
                    frame.buffer_mut().set_style(
                        *rect,
                        Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
                    );
                }
            }
        }
    }
    // The hover-revealed row buttons (Stage File / Stage Folder) sit on a
    // row that is itself lit, so their own hover is a fill, painted from
    // the rect the row walk registered.
    if let Some(
        target @ (HitTarget::StatusRowButton { .. } | HitTarget::StatusFolderButton { .. }),
    ) = hovered_target.as_ref()
    {
        if !show_help
            && preferences.is_none()
            && !repo_dialogs.is_open()
            && matches!(input_mode, InputMode::Normal)
        {
            if let Some((rect, _)) = frame_hits.iter().find(|(_, t)| t == target) {
                frame.buffer_mut().set_style(
                    *rect,
                    crate::theme::filled_hover_style(Style::default().fg(th().ui.accent)),
                );
            }
        }
    }

    // Column show/hide menu (right-click the Log header row): a checklist
    // popup over the Log pane, one row per column, toggled by a click.
    // `Message` is always shown (`LogColumn::adjustable`), so it is listed
    // dimmed with no hit target: the menu still reads as all six columns.
    if open_menu.is_column_menu() {
        super::column_chooser::paint(
            frame,
            log_area,
            log_column_hidden,
            open_menu.column_menu(),
            hovered_target.as_ref(),
            frame_hits,
        );
    }

    // The hosting menu (a PR / issue row, the toolbar chip, a
    // palette chooser), its rows spliced to the front of the hit
    // list: it is drawn over the panes.
    hosting.render_menu(frame, hover(hovered_target).menu_row(), frame_hits);
    // The sidebar dropdown (filter, provider, repository), over
    // everything.
    hosting.render_picker(frame, hover(hovered_target).menu_row(), frame_hits);
    // The tab bar's "Open in" popup, last: it hangs over the panes, and its
    // rows lead the hit list so nothing behind it hovers.
    crate::open_in_bar::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);
    // The merge-revert parent dropdown, the same way.
    crate::revert_mainline::render(frame);
    // The tag delete scope dialog, the same way.
    crate::tag_delete_dialog::render(frame);
    // The sensitive-files question of a refused stage, the same way.
    crate::stage_guard_dialog::render(frame);
    // The predicted conflicts popup, the same way.
    crate::conflict_popup::render(frame);
    // Copy as Mermaid gitGraph, the same way.
    crate::mermaid_popup::render(frame);
    // The toolbar's profile popup, the same way.
    crate::profile_bar::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);
    // The tab bar's workspace combo, the same way.
    crate::workspace_ui::chip::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);
    // A binary file's "Open in" popup, the same way.
    crate::open_with::popup::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);
    // The image header's "Edit in" popup, the same way.
    crate::image_view::edit_in::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);
    // The Diff view's encoding dropdown, the same way.
    crate::diff_encoding_menu::render_popup(frame, hover(hovered_target).menu_row(), frame_hits);

    // The remote menu (a remote's own menu, a toolbar dropdown,
    // Set upstream, the diverged-push dialog) and the Add / Edit
    // remote popup, registered from the walk that drew them.
    if let Some(menu) = &remote_ui.menu {
        for (i, rect) in menu
            .render(frame, hover(hovered_target).menu_row())
            .into_iter()
            .enumerate()
        {
            frame_hits.insert(0, (rect, HitTarget::RemoteMenuItem(i)));
        }
    }
    if let Some(form) = &remote_ui.form {
        let hovered_form_part = match &hovered_target {
            Some(HitTarget::RemoteForm(h)) => Some(*h),
            _ => None,
        };
        let front: Vec<(Rect, HitTarget)> = form
            .render(frame, frame.area(), hovered_form_part)
            .into_iter()
            .map(|(rect, h)| (rect, HitTarget::RemoteForm(h)))
            .collect();
        frame_hits.splice(0..0, front);
    }

    // The tab menu / a management row's menu. Its rects are kept
    // apart from `frame_hits`: while it is open it owns every
    // click (a row, or a dismiss), like the overlay does.
    *repo_menu_rects = match &repo_menu {
        Some(state) => state.menu.render(frame, hovered_repo_menu_row),
        None => Vec::new(),
    };

    // The one open popup menu (a context menu or the `⋯` overflow
    // menu) draws itself and hands back one rect per row from the
    // same walk, so what is registered as a hit target is exactly
    // what was painted, including the `Back` row of an open
    // submenu. Its rows go to the *front* of the hit list: it is
    // drawn over the panes, and `find_hit_target` returns the
    // first match.
    let menu_rows = open_menu.render(frame, hover(hovered_target).menu_row());
    frame_hits.splice(0..0, menu_rows);

    // Suppressed while any context menu is open: this tooltip follows the
    // mouse position over the Log and would paint over a menu open on the
    // same row.
    let any_context_menu_open = open_menu.is_open()
        // The remote menu alone (a Pull / Push dropdown) does not block:
        // its own rows carry tooltips, and the pointer events derive no
        // other tooltip while it is open (`mouse_moved`).
        || remote_ui.form.is_some()
        || matches!(input_mode, InputMode::EditingCommitMessage(_))
        || matches!(input_mode, InputMode::InteractiveRebase(_))
        || matches!(input_mode, InputMode::ComposeCommits(_))
        || matches!(input_mode, InputMode::SplitCommit(_))
        || hosting.modal_open()
        || crate::open_in_bar::is_open()
        || crate::revert_mainline::is_open()
        || crate::tag_delete_dialog::is_open()
        || crate::stage_guard_dialog::is_open()
        || crate::ai_sent_dialog::is_open()
        || crate::ai_limit_dialog::is_open()
        || crate::conflict_popup::is_open()
        || crate::mermaid_popup::is_open()
        || crate::profile_bar::is_open()
        || crate::workspace_ui::chip::is_open()
        || crate::about::is_open()
        || crate::trust_dialog::is_open()
        || crate::activity_log::is_open()
        || crate::jobs_ui::panel_open()
        || crate::updates::panel::is_open()
        || crate::open_with::popup::is_open()
        || crate::image_view::edit_in::is_open()
        || crate::diff_encoding_menu::is_open();
    // Every tooltip is gated by ONE lifecycle owner (`tooltip.rs`): never
    // while a modal, menu, popup or drag is up (the palette, the tab picker,
    // Confirm dialogs, help, Preferences and the dialogs), and only after a
    // real dwell on one target.
    let tooltip_blocked = any_context_menu_open
        || show_help
        || ai_ui.popup_open()
        || preferences.is_some()
        || repo_dialogs.is_open()
        || crate::tooltip::blocked_by_mode(input_mode)
        || ref_popup.is_some()
        || repo_menu.is_some()
        || ref_drag.is_some()
        || dragging_divider.is_some()
        || dragging_sidebar_section.is_some()
        || dragging_commit_message
        || dragging_log_boundary.is_some()
        || dragging_scrollbar.is_some()
        || dragging_log_header.is_some()
        || dragging_list_bar.is_some()
        || diff_line_drag.is_some()
        || text_drag.is_some();
    let now = Instant::now();
    let mut tooltip_painted: Option<Rect> = None;
    // The linked-PR chip's tooltip: title, state, source -> target.
    if tooltip.visible(
        crate::tooltip::TooltipKey::LinkedPrChip,
        now,
        tooltip_blocked,
    ) {
        if let Some(chip @ HitTarget::Hosting(HostingHit::LinkedPrChip | HostingHit::NewPrChip)) =
            hovered_target.as_ref()
        {
            if let Some((rect, _)) = frame_hits.iter().find(|(_, t)| t == chip) {
                tooltip_painted = Some(hosting.render_chip_tooltip(frame, *rect));
            }
        }
    }
    // A Pull / Push dropdown row: the tooltip opens to the side of the
    // menu so it never covers the rows.
    if let Some(HitTarget::RemoteMenuItem(i)) = hovered_target.as_ref() {
        if let Some(text) = remote_ui.row_tooltip(*i) {
            if tooltip.visible(
                crate::tooltip::TooltipKey::RemoteMenuRow(*i),
                now,
                tooltip_blocked,
            ) {
                if let Some((rect, _)) = frame_hits
                    .iter()
                    .find(|(_, t)| Some(t) == hovered_target.as_ref())
                {
                    let w = crate::ui_kit::display_width(&text) as u16 + 4;
                    let area = frame.area();
                    let x = if rect.right() + 1 + w <= area.right() {
                        rect.right() + 1
                    } else {
                        rect.x.saturating_sub(w + 1)
                    };
                    tooltip_painted = Some(crate::tooltip::paint_box(
                        frame,
                        area,
                        x,
                        rect.y.saturating_sub(1),
                        vec![Line::from(text)],
                    ));
                }
            }
        }
    }
    if let Some(text) = hovered_target.as_ref().and_then(crate::tooltip::hint_for) {
        let text_shown = hovered_target
            .as_ref()
            .and_then(crate::tooltip::hint_text)
            .unwrap_or_else(|| text.to_string());
        if tooltip.visible(crate::tooltip::TooltipKey::Hint(text), now, tooltip_blocked) {
            if let Some((rect, _)) = frame_hits
                .iter()
                .find(|(_, t)| Some(t) == hovered_target.as_ref())
            {
                tooltip_painted = Some(crate::tooltip::paint_box(
                    frame,
                    frame.area(),
                    rect.x,
                    rect.y + 1,
                    vec![Line::from(text_shown)],
                ));
            }
        }
    }
    // The commit-detail author name: the author's email.
    if let Some(HitTarget::CommitAuthorName) = hovered_target.as_ref() {
        if let Some(email) = crate::tooltip::author_email() {
            if tooltip.visible(
                crate::tooltip::TooltipKey::CommitAuthor,
                now,
                tooltip_blocked,
            ) {
                if let Some((rect, _)) = frame_hits
                    .iter()
                    .find(|(_, t)| *t == HitTarget::CommitAuthorName)
                {
                    tooltip_painted = Some(crate::tooltip::paint_box(
                        frame,
                        frame.area(),
                        rect.x,
                        rect.y + 1,
                        vec![Line::from(email)],
                    ));
                }
            }
        }
    }
    // A sidebar branch under a workflow: type, parent, how far behind it.
    if let Some((key, lines, col, row)) = crate::flow::view::hover_tooltip() {
        if tooltip.visible(
            crate::tooltip::TooltipKey::FlowBranch(key),
            now,
            tooltip_blocked,
        ) {
            tooltip_painted = Some(crate::tooltip::paint_box(
                frame,
                frame.area(),
                col,
                row + 1,
                lines.into_iter().map(Line::from).collect(),
            ));
        }
    }
    // A sidebar tag: where it lives, in words.
    if let Some((key, lines, col, row)) = crate::tag_where_ui::hover_tooltip() {
        if tooltip.visible(
            crate::tooltip::TooltipKey::TagWhere(key),
            now,
            tooltip_blocked,
        ) {
            tooltip_painted = Some(crate::tooltip::paint_box(
                frame,
                frame.area(),
                col,
                row + 1,
                lines.into_iter().map(Line::from).collect(),
            ));
        }
    }
    if let Some(HitTarget::Toolbar(button @ (ToolbarButton::Undo | ToolbarButton::Redo))) =
        hovered_target.as_ref()
    {
        let direction = undo_redo_direction(*button);
        if tooltip.visible(
            crate::tooltip::TooltipKey::UndoRedo(direction),
            now,
            tooltip_blocked,
        ) {
            if let Some((rect, _)) = frame_hits
                .iter()
                .find(|(_, t)| Some(t) == hovered_target.as_ref())
            {
                tooltip_painted = Some(crate::tooltip::paint_box(
                    frame,
                    frame.area(),
                    rect.x,
                    rect.y + 1,
                    vec![Line::from(crate::undo_ui::tooltip(
                        journal_state,
                        direction,
                        undo_busy,
                    ))],
                ));
            }
        }
    }
    if let Some(HitTarget::DiffBarButton(b)) = hovered_target.as_ref() {
        if tooltip.visible(
            crate::tooltip::TooltipKey::DiffBar(*b),
            now,
            tooltip_blocked,
        ) {
            if let Some((rect, _)) = frame_hits
                .iter()
                .find(|(_, t)| t == hovered_target.as_ref().unwrap())
            {
                tooltip_painted = Some(crate::tooltip::paint_box(
                    frame,
                    frame.area(),
                    rect.x,
                    rect.y,
                    vec![Line::from(b.tooltip())],
                ));
            }
        }
    }
    // Per-column hover tooltip: the content depends on which cell of the Log
    // row the mouse is over (`hovered_commit`'s `LogColumn`, computed in the
    // `Moved` arm), not one blanket tooltip. Message -> full message (summary
    // + body); Author -> name + email; Date -> full timestamp with seconds;
    // SHA -> the full hex; Changes -> the stat line; Ref -> the full ref name,
    // but ONLY for a single ref (a commit with more gets `ref_popup` below, a
    // real menu; the `pills.len() == 1` guard keeps the two exclusive). Graph
    // has no tooltip.
    if let Some((commit_id, hover_col, col, row)) = hovered_commit {
        if hover_col != LogColumn::Graph
            && tooltip.visible(
                crate::tooltip::TooltipKey::Commit(commit_id, hover_col),
                now,
                tooltip_blocked,
            )
        {
            if let Some(commit) = log_result
                .as_ref()
                .ok()
                .and_then(|g| g.nodes.iter().find(|c| c.id == commit_id))
            {
                let lines: Vec<Line> = match hover_col {
                    LogColumn::Message => commit
                        .message
                        .lines()
                        .map(|l| Line::from(l.to_string()))
                        .collect(),
                    LogColumn::Author => vec![Line::from(format!(
                        "{} <{}>",
                        commit.author_name, commit.author_email
                    ))],
                    // Only the full date + time.
                    LogColumn::Date => {
                        vec![Line::from(format_timestamp_full(commit.timestamp))]
                    }
                    LogColumn::Sha => vec![Line::from(commit.id.to_hex())],
                    LogColumn::Changes => commit_changes
                        .get(&commit_id)
                        .map(|s| {
                            vec![Line::from(changes_tooltip(
                                s.files,
                                s.insertions,
                                s.deletions,
                            ))]
                        })
                        .unwrap_or_default(),
                    LogColumn::Ref => {
                        let pills = ref_pills_for_commit(commit_id, refs_for_pills);
                        match pills.as_slice() {
                            [only] => vec![Line::from(only.name.clone())],
                            // The faded ghost pill's tooltip: the deduced ref and
                            // its kind (`ui.graph_ghost_ref`).
                            [] if crate::live_settings::graph_ghost_ref() => log_layout
                                .ghost_for_commit(commit_id)
                                .map(|g| vec![Line::from(g.tooltip())])
                                .unwrap_or_default(),
                            _ => Vec::new(),
                        }
                    }
                    LogColumn::Graph => unreachable!(),
                };
                if !lines.is_empty() {
                    // Bounded by `log_area`, not `frame.area()`: a per-column tooltip never
                    // needs to leave the pane whose column it describes.
                    tooltip_painted =
                        Some(crate::tooltip::paint_box(frame, log_area, col, row, lines));
                }
            }
        }
    }
    tooltip.end_frame(tooltip_painted);

    // The "other refs on this commit" reveal popup: a real menu, not a
    // tooltip -- positioned once at `anchor_col`/`anchor_row` (captured when it
    // opened) and never repositioned from later mouse movement, so moving the
    // cursor into it does not make it jump or vanish. See `RefPopupState` and
    // the `Moved`/keyboard-open handlers for the open/retarget/close rules.
    ref_popup_rect = None;
    if let Some(state) = &ref_popup {
        let commit_exists = log_result
            .as_ref()
            .is_ok_and(|g| g.nodes.iter().any(|c| c.id == state.commit_id));
        if commit_exists {
            let pills = popup_pills(state.commit_id, refs_for_pills);
            let lane_color = commit_lane_colors
                .get(&state.commit_id)
                .copied()
                .unwrap_or(th().ui.muted);
            if !pills.is_empty() {
                // Minimal: the pills stack directly, no title line or chrome box, but it
                // is still a double-click/keyboard-navigable menu. Two passes: build each
                // row's label + style, then pad every label to the widest one's display
                // width, so each row is a solid full-width bar. The budget is the Ref
                // column's full width (the anchor row pads its own pill to it), so the
                // row and the rows expanded under it share one right edge.
                let max_label_width = ref_expand_block_width(log_widths);
                let raw: Vec<(String, Style)> = pills
                    .iter()
                    .enumerate()
                    .map(|(i, pill)| {
                        let bg = muted_pill_bg(lane_color);
                        let mut style = Style::default().bg(bg).fg(contrasting_fg(bg));
                        if pill.checked_out {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if state.opened_via_keyboard && i == state.selected {
                            style = style.bg(th().ui.selection).add_modifier(Modifier::BOLD);
                        }
                        (popup_row_label_within(pill, max_label_width), style)
                    })
                    .collect();
                let lines: Vec<Line> = raw
                    .into_iter()
                    .map(|(label, style)| {
                        let pad = max_label_width.saturating_sub(display_width(&label));
                        let padded = format!("{label}{}", " ".repeat(pad));
                        Line::from(Span::styled(padded, style))
                    })
                    .collect();
                // Every line was padded to `max_label_width` above, so this is the Ref
                // column's width; taken as a max() over the rendered lines so the rect
                // never claims more than was drawn, and capped to the frame.
                let width = lines
                    .iter()
                    .map(|l| l.width() as u16)
                    .max()
                    .unwrap_or(max_label_width as u16)
                    .min(log_area.width.saturating_sub(2));
                let height = (lines.len() as u16).min(log_area.height.saturating_sub(1));
                // Anchored to `log_area`, not `frame.area()`: the popup belongs to a
                // commit row in the Log pane, and `anchored_popup` clamps to the area it
                // is given, so it stays inside both the pane and the frame (like the
                // Status pane's file/folder menus above and the Log column tooltip).
                let popup =
                    anchored_popup(log_area, state.anchor_col, state.anchor_row, width, height);
                ref_popup_rect = Some(popup);
                frame.render_clipped(Clear, popup);
                frame.render_clipped(
                    Paragraph::new(lines).style(Style::default().bg(th().ui.panel)),
                    popup,
                );
                for (i, pill) in pills.iter().enumerate() {
                    let hit_row = popup.y + i as u16;
                    if hit_row >= popup.y + popup.height {
                        break; // popup got clamped shorter than the list
                    }
                    frame_hits.push((
                        Rect::new(popup.x, hit_row, popup.width, 1),
                        HitTarget::TooltipRefPill {
                            name: pill.name.clone(),
                            kind: pill.kind,
                        },
                    ));
                }
            }
        }
    }

    // Painted last, over the panes but under nothing: a highlighted drag
    // strip must not sit on top of an open menu. It styles the existing cells
    // rather than writing a glyph, keeping the border character already
    // there, so the strip reads as "this edge is live".
    // Overlays always win: the strip is styled over what the panes drew, and
    // the palette, menus and dialogs are already in the buffer by now, so
    // while any of them is up the hover is not shown at all.
    let overlay_up = any_context_menu_open
        || show_help
        || ai_ui.popup_open()
        || preferences.is_some()
        || repo_dialogs.is_open()
        || crate::tooltip::blocked_by_mode(input_mode)
        || ref_popup.is_some()
        || repo_menu.is_some();
    if let Some(strip) = hovered_resize_strip.filter(|_| !overlay_up) {
        // A column boundary of a table lights as `ui.column_resize` says (a
        // thin line, or the header block); every other strip is the block.
        let is_column_boundary = frame_hits.iter().any(|(rect, target)| {
            *rect == strip
                && matches!(
                    target,
                    HitTarget::LogColumnBoundary(_)
                        | HitTarget::CommitColumn(crate::commit_column::ColumnHit::Boundary(_))
                )
        });
        if is_column_boundary {
            crate::column_resize::paint_grab(
                frame.buffer_mut(),
                crate::column_resize::mode(),
                strip,
            );
        } else {
            paint_resize_strip(frame.buffer_mut(), strip);
        }
    }

    // The ref-drag drop target, painted in the same last pass and for the
    // same reason: `OSC 22` is a bonus, so the cells are what tells every
    // terminal's user where this ref would land.
    //
    // Only a *valid* target lights; an invalid one staying dark says "not
    // here" without a second color to learn (the footer banner names the
    // reason in words).
    if let Some(candidate) = ref_drag
        .as_ref()
        .filter(|d| d.started)
        .and_then(|d| d.over.as_ref())
        .filter(|c| c.valid)
    {
        let buf = frame.buffer_mut();
        let area = buf.area;
        for y in candidate.rect.y..candidate.rect.y.saturating_add(candidate.rect.height) {
            for x in candidate.rect.x..candidate.rect.x.saturating_add(candidate.rect.width) {
                if x < area.x + area.width && y < area.y + area.height {
                    // The house hover treatment (the strong
                    // `plain_button_style(true)` every plain
                    // control lights with): a ref pill already
                    // carries its own tinted background, so a
                    // subtler `HOVER_BG` wash would be invisible
                    // exactly where it matters most.
                    buf[(x, y)].set_style(plain_button_style(true));
                }
            }
        }
    }
    // Ghost label, dimmed source and refused target: last, so the
    // ghost sits over everything drawn above.
    if let Some(drag) = ref_drag.as_ref() {
        paint_ref_drag(frame.buffer_mut(), drag);
    }
    // A carried column header: its ghost, where it came from and where it
    // would land (the same visual language as a carried ref).
    if let Some(drag) = dragging_log_header.filter(|d| d.started) {
        let buf = frame.buffer_mut();
        if let Some(source) = drag.source {
            crate::column_drag::dim(buf, source);
        }
        if let Some((x, y)) = drag.marker {
            crate::column_drag::paint_marker(buf, x, y);
        }
        crate::column_drag::paint_ghost(buf, drag.col.title(), drag.at);
    }
    if let Some((title, at, marker)) = ctx.commit_header_drag {
        let buf = frame.buffer_mut();
        if let Some((x, y)) = marker {
            crate::column_drag::paint_marker(buf, x, y);
        }
        crate::column_drag::paint_ghost(buf, title, at);
    }

    // The management tab's Browse / Init / Clone dialog, painted
    // over every other popup. It records its own hit rects while
    // drawing; while it is open it owns every key and mouse event.
    repo_dialogs.render(frame.area(), frame.buffer_mut());
    // `Ctrl+N` badges over the dialog just painted, from the rects it captured.
    if matches!(input_mode, InputMode::Navigation) && repo_dialogs.is_open() {
        let pairs: Vec<(Rect, FocusTarget)> = repo_dialogs
            .nav_targets()
            .into_iter()
            .map(|(rect, (col, row))| (rect, FocusTarget::Dialog(col, row)))
            .collect();
        *ctx.nav_badge_targets = badge_pairs(frame, &pairs);
    }
    // The amend dialog and the history workbench keep their form in the input
    // mode, so their `Ctrl+N` is a flag on the form (`badges`), not a mode.
    {
        let cells = match input_mode {
            InputMode::EditingCommitMessage(form) if form.badges && form.badge_ready() => {
                let hits: Vec<_> = frame_hits
                    .iter()
                    .filter_map(|(r, t)| match t {
                        HitTarget::RewordForm(part) => Some((*r, *part)),
                        _ => None,
                    })
                    .collect();
                Some(crate::reword_ui::RewordForm::nav_targets(&hits))
            }
            InputMode::Rewrite(view) if view.badges && view.badge_ready() => {
                let hits: Vec<_> = frame_hits
                    .iter()
                    .filter_map(|(r, t)| match t {
                        HitTarget::Rewrite(hit) => Some((*r, *hit)),
                        _ => None,
                    })
                    .collect();
                Some(crate::rewrite_view::RewriteView::nav_targets(&hits))
            }
            _ => None,
        };
        if let Some(cells) = cells {
            let pairs: Vec<(Rect, FocusTarget)> = cells
                .into_iter()
                .map(|(rect, (col, row))| (rect, FocusTarget::Dialog(col, row)))
                .collect();
            *ctx.nav_badge_targets = badge_pairs(frame, &pairs);
        }
    }
    // The palette opened from a dialog (the Undo history panel's row actions)
    // paints over it.
    if let (InputMode::CommandPalette { draft, selected }, true) =
        (input_mode, repo_dialogs.is_open())
    {
        super::palette_overlay::render_command_palette(
            frame,
            draft,
            *selected,
            ctx.palette_ctx,
            ctx.field_editor,
            journal_state,
            undo_busy,
            &mut *ctx.palette_popup,
        );
    }
    // The Preferences screen, drawn last so it is over everything: it is a
    // *screen*, not an overlay. It hit-tests itself from rects it captures
    // during this call, so nothing under it needs removing from
    // `frame_hits`; the mouse handler never consults them while it is open.
    if let Some(prefs) = preferences.as_mut() {
        // The pending `Ctrl+N` prefix's hint shows on the screen's bottom
        // border (its footer is under Settings).
        prefs.set_mode_hint(ctx.mode_hint.clone());
        prefs.render(frame, frame.area());
        // The status bar is under Settings: its Activity button's stale rect
        // (now the line's right end, where the bottom scroll arrow is) must
        // not take a click.
        crate::activity_log::clear_status_button();
        // ... and so is the status line: a failure is painted on the
        // screen's bottom row instead (the last pass of the draw).
        let area = frame.area();
        crate::activity_log::status_error::note_covered(Rect::new(
            area.x,
            area.bottom().saturating_sub(1),
            area.width,
            1,
        ));
        // `Ctrl+N` badges over the screen just painted, from the rects it
        // captured; the key handler reads them back from `nav_badge_targets`.
        if matches!(input_mode, InputMode::Navigation) {
            let pairs: Vec<(Rect, FocusTarget)> = prefs
                .nav_targets()
                .into_iter()
                .map(|(rect, (col, row))| (rect, FocusTarget::Settings(col, row)))
                .collect();
            let targets: Vec<FocusTarget> = pairs.iter().map(|(_, t)| *t).collect();
            let assigned = assign_navigation_badges(&targets);
            let panes: Vec<(Rect, char)> = pairs
                .iter()
                .zip(assigned.iter())
                .map(|((area, _), (c, _))| (*area, *c))
                .collect();
            render_navigation_badges(frame, &panes);
            *ctx.nav_badge_targets = assigned;
        }
        // The palette and the pickers open over Settings: painted after it
        // (`draw_overlays` skips them while it is open), so a theme preview
        // is seen on Settings itself.
        super::overlays::draw_command_palette(
            frame,
            input_mode,
            false,
            ctx.palette_ctx,
            ctx.field_editor,
            journal_state,
            undo_busy,
            &mut *ctx.palette_popup,
        );
        super::overlays::draw_picker_popups(
            frame,
            input_mode,
            ctx.field_editor,
            &mut *ctx.tab_picker_popup,
        );
    }

    // The help overlay ('?'), drawn **last of all the popups**, after even
    // the Preferences screen: `?` passes through Preferences and the
    // Repository Management tab, so it has to paint on top of both.
    // Scrollable and filterable; `chrome::help_body` lays out and
    // filters, unit-tested there since this closure isn't buffer-tested
    // (see the module doc note on `event_loop.rs`).
    if show_help {
        crate::help_view::render(
            frame,
            help_filter,
            &mut help_scroll,
            &mut help_popup_inner_height,
        );
    }

    ai_ui.render_popup(frame, frame.area());
    // "What is sent" before an AI request: over the explanation popup too
    // (an Explain shows its popup while it works).
    crate::ai_sent_dialog::render(frame);
    // "Over the AI limit": asked before the request is built, so before
    // "What is sent".
    crate::ai_limit_dialog::render(frame);
    crate::about::render(frame);
    // The integration security check, over everything but the notifications.
    crate::trust_dialog::render(frame);
    // The notification card and the Activity Log, over everything.
    crate::activity_log::render(frame);
    if matches!(input_mode, InputMode::Navigation) && crate::activity_log::is_open() {
        let pairs: Vec<(Rect, FocusTarget)> = crate::activity_log::nav_targets()
            .into_iter()
            .map(|(rect, (col, row))| (rect, FocusTarget::Activity(col, row)))
            .collect();
        *ctx.nav_badge_targets = badge_pairs(frame, &pairs);
    }
    crate::jobs_ui::render_panel(frame);
    crate::updates::panel::render(frame);
    *ctx.help_popup_inner_height = help_popup_inner_height;
    *ctx.help_scroll = help_scroll;
    *ctx.ref_popup_rect = ref_popup_rect;
}

/// The Changes column's hover text: three counted phrases, each a plural
/// message of its own so every language inflects its own noun, joined by a
/// message that can reorder them.
pub(crate) fn changes_tooltip(files: u32, insertions: u32, deletions: u32) -> String {
    use gitpoulpi_i18n::msg::log_draw as m;
    m::changes_tooltip(
        m::changes_files(u64::from(files)),
        m::changes_insertions(u64::from(insertions)),
        m::changes_deletions(u64::from(deletions)),
    )
}

/// Paints one badge on each of `pairs` (the next free character each, by
/// `assign_navigation_badges`) and returns the assignment for the key handler.
fn badge_pairs(frame: &mut Frame, pairs: &[(Rect, FocusTarget)]) -> Vec<(char, FocusTarget)> {
    let targets: Vec<FocusTarget> = pairs.iter().map(|(_, t)| *t).collect();
    let assigned = assign_navigation_badges(&targets);
    let panes: Vec<(Rect, char)> = pairs
        .iter()
        .zip(assigned.iter())
        .map(|((area, _), (c, _))| (*area, *c))
        .collect();
    render_navigation_badges(frame, &panes);
    assigned
}

#[cfg(test)]
mod tooltip_tests {
    use super::changes_tooltip;

    #[test]
    fn the_changes_tooltip_counts_with_real_plurals() {
        assert_eq!(
            changes_tooltip(1, 1, 1),
            "1 file changed, 1 insertion, 1 deletion"
        );
        assert_eq!(
            changes_tooltip(12, 0, 3),
            "12 files changed, 0 insertions, 3 deletions"
        );
        gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::Ru, || {
            assert_eq!(
                changes_tooltip(2, 1, 5),
                "2 изменённых файла, 1 добавление, 5 удалений"
            );
        });
    }
}
