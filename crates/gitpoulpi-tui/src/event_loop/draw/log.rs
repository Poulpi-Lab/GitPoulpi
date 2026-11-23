//! The Log pane: table rows, graph picture, avatars, scrollbar, the inline
//! ref-name box, the find bar and the hit rects they register.

use super::super::*;

pub(in crate::event_loop) struct LogDrawCtx<'a> {
    pub(in crate::event_loop) log_area: Rect,
    pub(in crate::event_loop) central_view: CentralView,
    pub(in crate::event_loop) focus: Focus,
    pub(in crate::event_loop) has_wip: bool,
    pub(in crate::event_loop) no_repository: bool,
    pub(in crate::event_loop) log_len: usize,
    pub(in crate::event_loop) log_follow: bool,
    /// The display row Reveal in graph just brought into view, drawn like a hover.
    pub(in crate::event_loop) reveal_flash: Option<usize>,
    pub(in crate::event_loop) ref_connector: bool,
    pub(in crate::event_loop) searching: bool,
    pub(in crate::event_loop) hovered_wip_row: bool,
    pub(in crate::event_loop) hovered_search_button: Option<SearchBarButton>,
    pub(in crate::event_loop) hovered_commit: Option<(Oid, LogColumn, u16, u16)>,
    pub(in crate::event_loop) inline_ref: Option<(InlineRefKind, Oid)>,
    pub(in crate::event_loop) composer_focus: Option<ComposerField>,
    pub(in crate::event_loop) avatar_placement: gitpoulpi_config::AvatarPlacement,
    pub(in crate::event_loop) avatar_capability: terminal_image::Capability,
    pub(in crate::event_loop) ui_lane_background: graph::LaneBackground,
    pub(in crate::event_loop) ui_tint_strength: gitpoulpi_config::GraphRowTintStrength,
    pub(in crate::event_loop) graph_settings: graph::GraphSettings,
    pub(in crate::event_loop) log_widths: &'a LogColumnWidths,
    pub(in crate::event_loop) log_visible_order: &'a [LogColumn],
    pub(in crate::event_loop) log_col_pick: Option<LogColumn>,
    pub(in crate::event_loop) log_offsets: &'a [(LogColumn, u16, u16)],
    /// The whole geometry: the clipped offsets above are its on-screen part.
    pub(in crate::event_loop) log_geo: &'a LogColumnsGeo,
    pub(in crate::event_loop) layout: &'a graph::LogLayout,
    pub(in crate::event_loop) search_matches: &'a [usize],
    pub(in crate::event_loop) search_match_set: &'a std::collections::HashSet<usize>,
    pub(in crate::event_loop) lazy_log: &'a graph::LazyLog,
    pub(in crate::event_loop) input_mode: &'a InputMode,
    pub(in crate::event_loop) status_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::WorkingTree>,
    pub(in crate::event_loop) log_result:
        &'a gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(in crate::event_loop) pane_notice: &'a Option<(Vec<String>, Style)>,
    pub(in crate::event_loop) ref_popup: &'a Option<RefPopupState>,
    pub(in crate::event_loop) ref_name_error: &'a Option<String>,
    pub(in crate::event_loop) log_multi: &'a LogSelection,
    pub(in crate::event_loop) commit_summary: &'a String,
    pub(in crate::event_loop) commit_changes:
        &'a std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
    pub(in crate::event_loop) avatar_live: &'a avatar::LiveSettings,
    pub(in crate::event_loop) log_state: &'a mut TableState,
    pub(in crate::event_loop) field_editor: &'a mut text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) frame_hits: &'a mut Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) graph_image: &'a mut graph::GraphImage,
    pub(in crate::event_loop) log_avatar_images: &'a mut avatar::LogAvatarImages,
    pub(in crate::event_loop) avatar_service: &'a mut avatar::AvatarService,
    pub(in crate::event_loop) visible_commit_ids: &'a mut Vec<Oid>,
    pub(in crate::event_loop) log_search_bar_area: &'a mut Option<Rect>,
    pub(in crate::event_loop) inline_ref_row: &'a mut Option<u16>,
    pub(in crate::event_loop) name_slot: &'a mut NameSlot,
    pub(in crate::event_loop) commit_lane_colors: &'a mut std::collections::HashMap<Oid, Color>,
    /// Filled by this paint: what each column needs to show its content.
    pub(in crate::event_loop) log_natural: &'a mut NaturalWidths,
}

pub(in crate::event_loop) fn draw_log(frame: &mut Frame, ctx: &mut LogDrawCtx<'_>) {
    let log_area = ctx.log_area;
    let central_view = ctx.central_view;
    let focus = ctx.focus;
    let has_wip = ctx.has_wip;
    let no_repository = ctx.no_repository;
    let log_len = ctx.log_len;
    // The Graph column at its least width: every commit on one lane, no
    // branch or merge lines (`graph::compact`). A function of the width.
    let compact_graph = graph::is_compact(usize::from(ctx.log_widths.width_of(LogColumn::Graph)));
    let log_follow = ctx.log_follow;
    let reveal_flash = ctx.reveal_flash;
    let ref_connector = ctx.ref_connector;
    let searching = ctx.searching;
    let hovered_wip_row = ctx.hovered_wip_row;
    let hovered_search_button = ctx.hovered_search_button;
    let hovered_commit = ctx.hovered_commit;
    let inline_ref = ctx.inline_ref;
    let composer_focus = ctx.composer_focus;
    let avatar_placement = ctx.avatar_placement;
    let avatar_capability = ctx.avatar_capability;
    let ui_lane_background = ctx.ui_lane_background;
    let ui_tint_strength = ctx.ui_tint_strength;
    let graph_settings = ctx.graph_settings;
    let log_widths = ctx.log_widths;
    let log_visible_order = ctx.log_visible_order;
    let log_offsets = ctx.log_offsets;
    let log_geo = ctx.log_geo;
    let layout = ctx.layout;
    let search_matches = ctx.search_matches;
    let search_match_set = ctx.search_match_set;
    let lazy_log = ctx.lazy_log;
    let input_mode = ctx.input_mode;
    let status_result = ctx.status_result;
    let log_result = ctx.log_result;
    let pane_notice = ctx.pane_notice;
    let ref_popup = ctx.ref_popup;
    let ref_name_error = ctx.ref_name_error;
    let log_multi = ctx.log_multi;
    let commit_summary = ctx.commit_summary;
    let commit_changes = ctx.commit_changes;
    let avatar_live = ctx.avatar_live;
    let log_state = &mut *ctx.log_state;
    let field_editor = &mut *ctx.field_editor;
    let frame_hits = &mut *ctx.frame_hits;
    let graph_image = &mut *ctx.graph_image;
    let log_avatar_images = &mut *ctx.log_avatar_images;
    let avatar_service = &mut *ctx.avatar_service;
    let visible_commit_ids = &mut *ctx.visible_commit_ids;
    let log_search_bar_area = &mut *ctx.log_search_bar_area;
    let inline_ref_row = &mut *ctx.inline_ref_row;
    let name_slot = &mut *ctx.name_slot;
    let commit_lane_colors = &mut *ctx.commit_lane_colors;
    let log_natural = &mut *ctx.log_natural;
    *log_natural = NaturalWidths::new();
    let solo_nodes: &Option<Vec<CommitNode>> = &layout.nodes;
    let graph_order = &layout.order;
    let wip_graph_line = &layout.wip_line;
    let wip_graph_row = &layout.wip_row;
    let graph_grid = &layout.grid;
    let stash_graph_rows = &layout.stash_rows;
    let stash_row_stashes = &layout.stash_row_stashes;

    // `ui.ref_connector`: the thin line from a
    // ref pill to its node, only while the Ref column sits right
    // before the Graph column. Read by the commit rows below and
    // by the picture's rows further down.
    let connector_on = ref_connector_applies(ref_connector, log_visible_order);
    let mut log_rows: Vec<Row> = Vec::new();
    // Set below, in the no-graph arm, when the Log has nothing to
    // draw as rows and instead needs a message centered over the
    // whole pane -- loading, or a genuine error. `None` draws
    // nothing extra: the graph (or the WIP-only row) speaks for
    // itself.
    let mut log_center_overlay: Option<(Vec<String>, Style)> = None;
    // Only the rows on screen are built (see `log_window`): the display
    // rows `window.start..window.end`, with `window.start` the scroll
    // offset this frame settles on. `None` in the no-graph arm, whose few
    // rows are all built and handed to the table as they are.
    let visible_rows = log_visible_rows(log_area);
    let window = solo_nodes.as_ref().map(|nodes| {
        if log_area.height <= 2 {
            // Border and header only: `Table` returns before it touches
            // the offset, and so does this.
            let start = log_state.offset();
            return LogWindow {
                start,
                end: start,
                selected: None,
            };
        }
        let render = table_render_state(log_state, log_follow, visible_rows);
        log_window(
            render.offset(),
            render.selected(),
            visible_rows,
            graph_order.len(nodes.len()),
        )
    });
    let offset = window.map_or_else(|| log_state.offset(), |w| w.start);
    // The WIP row is built only when the table is on screen and display
    // index 0 is inside the window.
    let wip_built = (has_wip
        && central_view == CentralView::Log
        && window.is_none_or(|w| w.start == 0 && w.end > 0))
    .then(|| {
        // The WIP row: one line like a commit. Its Graph
        // cell is the dotted marker on HEAD's lane (the dashed
        // connector down to HEAD is drawn in the commit rows it
        // passes, by the layout); its Message cell is the commit
        // composer's Summary as an inline field, then the change
        // counters. See `log_table::wip_row`.
        let counts = status_result
            .as_ref()
            .map(wip_change_counts)
            .unwrap_or_default();
        let counts_at = wip_counts_at(
            log_visible_order,
            live_settings::log_wip_counts_in_message(),
        );
        let row = wip_row(
            match (compact_graph, wip_graph_row.as_ref()) {
                (true, Some(r)) => graph::compact_line(&r.cells, graph_settings.node_set),
                _ => wip_graph_line.clone().unwrap_or_default(),
            },
            wip_graph_row.as_ref().and_then(|r| {
                if compact_graph {
                    Some(0)
                } else {
                    graph::node_column(&r.cells)
                }
            }),
            log_widths,
            log_visible_order,
            commit_summary,
            composer_focus == Some(ComposerField::WipSummary),
            field_editor.caret(MainField::Summary),
            counts,
            counts_at,
            log_state.selected() == Some(0) || log_multi.contains(&RowKey::Wip),
            hovered_wip_row,
        );
        let counts_in_message = wip_message_counts(counts, counts_at);
        log_natural.wip_row(
            commit_summary,
            counts_in_message,
            wip_graph_row.as_ref().map_or(0, |r| r.cells.len()),
        );
        (row, counts_in_message)
    });
    if let Some(counts) = wip_built.as_ref().map(|(_, counts)| *counts) {
        // The field's hit target: the Message cell of the WIP row,
        // while that row is on screen and the Log -- not a diff --
        // owns the central column.
        let message_col = log_offsets
            .iter()
            .find(|(c, _, _)| *c == LogColumn::Message)
            .map(|(_, x, w)| (*x, *w));
        let row = log_index_to_row(log_top_y(log_area), offset, 0);
        if let (Some(row), Some((x, w))) = (row, message_col) {
            if row < log_area.y + log_area.height.saturating_sub(1) {
                frame_hits.push((Rect::new(x, row, w, 1), HitTarget::WipMessageField));
                field_editor.drawn(
                    MainField::Summary,
                    Rect::new(x, row, wip_field_width(counts, w as usize) as u16, 1),
                    false,
                );
            }
        }
    }
    let mut wip_row_value = wip_built.map(|(row, _)| row);
    if window.is_none() {
        // No graph: the WIP row, if any, stands first and alone.
        log_rows.extend(wip_row_value.take());
    }
    match solo_nodes.as_ref() {
        Some(solo_nodes) => {
            // Registers each visible row's primary ref pill as a
            // hit target, at wherever the Ref column currently sits
            // in `log_offsets` (it's user-reorderable, so this
            // can't assume it's the first column) -- so a
            // double-click checks it out the same way a
            // Refs-sidebar row does. Skipped entirely if the Ref
            // column is hidden.
            let top_y = log_top_y(log_area);
            let visible_bottom = log_area.y + log_area.height.saturating_sub(1);
            let ref_col = log_offsets
                .iter()
                .find(|(c, _, _)| *c == LogColumn::Ref)
                .map(|(_, x, w)| (*x, *w));
            visible_commit_ids.clear();
            let changes_shown = log_widths.changes_w > 0;
            // The ref-reveal popup looks its commit's lane color up in
            // `commit_lane_colors`, which holds only the rows built below: seed the
            // popup's commit, wherever it is.
            if let Some(popup) = ref_popup.as_ref() {
                if let Some(i) = solo_nodes.iter().position(|c| c.id == popup.commit_id) {
                    if let Some((_, lane_color)) = layout.rows.get(i) {
                        commit_lane_colors.insert(popup.commit_id, *lane_color);
                    }
                }
            }
            // Skipped entirely while the Diff view owns the central
            // column -- the table isn't on screen, so building its rows
            // and registering its pills would leave invisible click
            // targets sitting on top of the diff.
            if let (Some(window), CentralView::Log) = (window, central_view) {
                log_rows = window_rows(graph_order, solo_nodes.len(), &window, |display, entry| {
                    match entry {
                        graph::LogEntry::Wip => wip_row_value.take().unwrap_or_default(),
                        graph::LogEntry::Stash(k) => {
                            let line = layout.stash_lines.get(k).cloned().unwrap_or_default();
                            match stash_row_stashes.get(k) {
                                Some(Some(stash)) => {
                                    log_natural.stash_row(
                                        stash,
                                        stash_graph_rows.get(k).map_or(0, |r| r.cells.len()),
                                    );
                                    stash_log_row(
                                        stash,
                                        match (compact_graph, stash_graph_rows.get(k)) {
                                            (true, Some(r)) => graph::compact_line(
                                                &r.cells,
                                                graph_settings.node_set,
                                            ),
                                            _ => line,
                                        },
                                        stash_graph_rows.get(k).and_then(|r| {
                                            if compact_graph {
                                                Some(0)
                                            } else {
                                                graph::node_column(&r.cells)
                                            }
                                        }),
                                        log_widths,
                                        log_visible_order,
                                        // A revealed row flashes in the hover color even
                                        // when selected, or revealing HEAD shows nothing.
                                        (log_state.selected() == Some(display)
                                            || stash
                                                .commit
                                                .is_some_and(|c| log_multi.contains_id(&c)))
                                            && reveal_flash != Some(display),
                                        hovered_commit
                                            .is_some_and(|(id, ..)| Some(id) == stash.commit)
                                            || reveal_flash == Some(display),
                                    )
                                }
                                _ => Row::new(vec![Cell::from(line)]),
                            }
                        }
                        graph::LogEntry::Commit(i) => {
                            let (Some(c), Some((line, lane_color))) =
                                (solo_nodes.get(i), layout.rows.get(i))
                            else {
                                return Row::default();
                            };
                            let lane_color = *lane_color;
                            let row_selected = (log_state.selected() == Some(display)
                                || log_multi.contains_id(&c.id))
                                && reveal_flash != Some(display);
                            let row_hovered = hovered_commit.is_some_and(|(id, ..)| id == c.id)
                                || reveal_flash == Some(display);
                            if let Some(screen_row) = log_index_to_row(top_y, offset, display) {
                                if screen_row < visible_bottom {
                                    if changes_shown {
                                        visible_commit_ids.push(c.id);
                                    }
                                    if inline_ref.is_some_and(|(_, at)| at == c.id) {
                                        *inline_ref_row = Some(screen_row);
                                    }
                                    if let (Some((x, w)), Some(pill)) =
                                        (ref_col, layout.pills_for(c.id).first())
                                    {
                                        frame_hits.push((
                                            Rect::new(x, screen_row, w, 1),
                                            HitTarget::LogRefPill {
                                                name: pill.name.clone(),
                                                kind: pill.kind,
                                            },
                                        ));
                                    }
                                }
                            }
                            commit_lane_colors.insert(c.id, lane_color);
                            log_natural.commit_row(
                                c,
                                layout.pills_for(c.id),
                                graph_grid.get(i).map_or(0, |r| r.cells.len()),
                                avatar::log_avatar_mode(avatar_placement, avatar_capability),
                                avatar_live.fallback,
                                commit_changes.get(&c.id),
                            );
                            // `ui.graph_ghost_ref`: the hovered row's deduced ref,
                            // faded, where the commit has no pill. Selection does
                            // not show it: only the pointer does.
                            let ghost = (row_hovered
                                && live_settings::graph_ghost_ref()
                                && inline_ref.is_none()
                                && layout.pills_for(c.id).is_empty())
                            .then(|| layout.ghost_for(i))
                            .flatten();
                            commit_row_with_changes(RowCtx {
                                commit: c,
                                pills: layout.pills_for(c.id),
                                graph: match (compact_graph, graph_grid.get(i)) {
                                    (true, Some(r)) => {
                                        graph::compact_line(&r.cells, graph_settings.node_set)
                                    }
                                    _ => line.clone(),
                                },
                                graph_node: graph_grid.get(i).and_then(|r| {
                                    if compact_graph {
                                        Some(0)
                                    } else {
                                        graph::node_column(&r.cells)
                                    }
                                }),
                                lane_color,
                                widths: log_widths,
                                visible_order: log_visible_order,
                                // A revealed row flashes in the hover color even
                                // when selected, or revealing HEAD shows nothing.
                                is_selected: row_selected,
                                is_hovered: row_hovered,
                                popup_fill_width: ref_popup
                                    .as_ref()
                                    .filter(|s| s.commit_id == c.id)
                                    .map(|_| ref_expand_block_width(log_widths)),
                                // Only dim once a query actually has hits --
                                // an empty query would otherwise gray out the
                                // entire table the instant the bar opens.
                                is_dimmed: searching
                                    && !search_match_set.is_empty()
                                    && !search_match_set.contains(&i),
                                avatar_mode: avatar::log_avatar_mode(
                                    avatar_placement,
                                    avatar_capability,
                                ),
                                avatar_fallback: avatar_live.fallback,
                                // `ui.ref_connector`: the Graph column's half
                                // only in text mode -- under the image
                                // renderer the picture draws its own line.
                                ref_connector: connector_on.then(|| RefConnector {
                                    graph_row: match graph_settings.render {
                                        graph::GraphRender::Text if !compact_graph => {
                                            graph_grid.get(i)
                                        }
                                        _ => None,
                                    },
                                }),
                                changes: commit_changes.get(&c.id),
                                ghost,
                            })
                        }
                    }
                });
            }
        }
        // No graph to draw: either the log is genuinely empty or it
        // failed to load. The WIP row, if any, has already been
        // pushed above and stands alone.
        None => match &log_result {
            Ok(_) if !has_wip => {
                log_rows.push(Row::new(vec![Cell::from(
                    gitpoulpi_i18n::msg::log_draw::NO_COMMITS.get(),
                )]));
            }
            Ok(_) => {}
            // A window with no repository has nothing to log, and the backend error
            // it would report is about a path that was never a repository: saying
            // `log error:` there would be the window blaming itself.
            Err(_) if no_repository => log_rows.push(Row::new(vec![Cell::from(
                match keymap::current().shown(Bind::GlobalHome) {
                    k if k.is_empty() => gitpoulpi_i18n::msg::log_draw::NO_REPO.get().to_string(),
                    k => gitpoulpi_i18n::msg::log_draw::no_repo_key(k),
                },
            )])),
            // The daemon round trip hasn't landed yet: not a failure, so never
            // worded as one. Centered over the whole pane rather than pushed into a
            // one-cell table row, which clips the message.
            // ... or the link stopped (`pane_notice` puts that
            // first), which is shown as the failure it is.
            Err(_) if pane_notice.is_some() => {
                log_center_overlay = pane_notice.clone();
            }
            // A genuine error, once the daemon has actually
            // answered: the real message, not truncated.
            Err(e) => {
                log_center_overlay = Some((
                    vec![gitpoulpi_i18n::msg::log_draw::log_error(e)],
                    Style::default().fg(th().chrome.notice),
                ));
            }
        },
    };
    let log_border_style = if focus == Focus::Log {
        Style::default().fg(th().ui.border_focused)
    } else {
        Style::default()
    };
    // Two hit targets per visible column, both on the header row:
    // a thin boundary strip at its right edge (drag-to-resize) and
    // the header cell itself (drag-to-reorder).
    // The header cells themselves take no hover fill: they have
    // no click action (only drag-to-reorder), so a lit-up cell
    // would advertise a button that isn't there. Only the resize
    // handle's own highlight (`hovered_resize_strip`, applied in
    // `log_table`) lights up.
    let header_cells: Vec<Cell> =
        crate::log_table::log_header_cells(log_visible_order, log_widths, ctx.log_col_pick);
    if central_view == CentralView::Log {
        frame_hits.extend(log_header_hit_targets(log_area, log_geo));
        // render through a throwaway clone so a
        // wheel-scrolled offset can't be dragged back to the
        // selection by ratatui's own `Table::render` -- see
        // `ui_kit::table_render_state`. `log_state.selected()` itself
        // is never touched here, so the graph column below (which
        // reads it) and every other consumer still see the real
        // selection regardless of whether it is on screen.
        let mut log_render_state = match window {
            // The rows are the window already: offset 0 over them, the
            // selection relative to the first. The offset this frame
            // settled on is written back below.
            Some(w) => {
                let mut state = TableState::default();
                *state.selected_mut() = w.selected;
                state
            }
            None => table_render_state(log_state, log_follow, visible_rows),
        };
        match log_geo.layout {
            gitpoulpi_config::LogColumnsLayout::Fit => frame.render_stateful_clipped(
                log_table(
                    log_rows,
                    header_cells,
                    log_visible_order,
                    log_widths,
                    log_border_style,
                ),
                log_area,
                &mut log_render_state,
            ),
            gitpoulpi_config::LogColumnsLayout::Scroll => super::log_scroll::render(
                frame,
                log_area,
                log_geo,
                super::log_scroll::ScrolledTable {
                    rows: log_rows,
                    header: header_cells,
                    order: log_visible_order,
                    widths: log_widths,
                    border_style: log_border_style,
                },
                &mut log_render_state,
            ),
        }
        *log_state.offset_mut() = window.map_or_else(|| log_render_state.offset(), |w| w.start);
        paint_log_ticks(frame.buffer_mut(), log_area, log_geo);

        // a scrollbar wherever a list
        // can genuinely scroll, over the block's own right border --
        // `scrollbar::Track::vertical` already returns `None` (drawing
        // nothing) once the content fits. Rendering-only for now: no
        // hover/drag wiring, so `hovered` is always `false` and a
        // click here still just falls through as a Log click (see
        // `docs/user/mouse-and-gestures.md`).
        if let Some(track) = scrollbar::Track::vertical(log_area, log_len, log_state.offset()) {
            scrollbar::render(frame, track, true, false);
        }
        // The sideways bar, on the bottom border, only while the columns are
        // wider than the pane (`scroll` layout). Pressable like the diff
        // view's: the thumb drags, the bare track brings the thumb under the
        // pointer, the end arrows step.
        if log_geo.scrolls() {
            if let Some(track) = scrollbar::Track::horizontal(
                log_area,
                usize::from(log_geo.content_w),
                usize::from(log_geo.h_offset),
            ) {
                scrollbar::render(frame, track, false, false);
                let thumb = Rect::new(
                    track.rect.x + track.thumb_start,
                    track.rect.y,
                    track.thumb_len.max(1),
                    track.rect.height,
                );
                frame_hits.push((thumb, HitTarget::ScrollbarThumb(ScrollBar::LogHorizontal)));
                frame_hits.push((
                    track.rect,
                    HitTarget::ScrollbarTrack(ScrollBar::LogHorizontal),
                ));
            }
        }

        // The inline name box (new branch/tag, a rename from the pill), over
        // the Ref cell of the target commit's row (over its pills, if it has
        // any). If the row is off screen or the Ref column is hidden/too
        // narrow, or another spot already holds the field, nothing is drawn
        // here and the next spot takes over (`ref_name_box.rs`).
        if let (Some((kind, _)), Some(row), InputMode::InlineName { value, .. }) =
            (inline_ref, *inline_ref_row, input_mode)
        {
            // A Ref column cut by the viewport counts as too narrow.
            let ref_whole: Vec<(LogColumn, u16, u16)> = log_offsets
                .iter()
                .copied()
                .filter(|(c, _, _)| *c != LogColumn::Ref || log_geo.fully_visible(*c))
                .collect();
            if let Some(box_rect) =
                inline_box_rect(&ref_whole, row).filter(|_| name_slot.may_draw(NameSpot::Log))
            {
                let text_rect = render_inline_box(
                    frame,
                    box_rect,
                    inline_error_rect(log_offsets, row),
                    kind,
                    value,
                    field_editor.caret(MainField::RefName),
                    ref_name_error.as_deref(),
                );
                field_editor.drawn(MainField::RefName, text_rect, false);
                name_slot.mark(NameSpot::Log);
            }
        }

        // The Graph column, as pixels, over the blanks the table just
        // drew for it.
        //
        // **After the table, deliberately, and in this order.** The
        // scroll offset is only settled once ratatui has rendered
        // with it, and this picture has to sit on exactly the rows
        // the table put commits on. The row arithmetic is not
        // re-derived here -- `graph::visible_raster_rows` is handed
        // the same offset / selected pair the hit-test side uses, and
        // the WIP row the table drew as display index 0.
        //
        // No-op on a terminal with no image protocol: `graph_settings`
        // was `Text` in that case, the table has already drawn real
        // glyphs, and `GraphImage::render` returns without touching
        // the buffer.
        let mut avatar_slots = Vec::new();
        if graph_image.capability().draws_graph() {
            if let Some((gx, gw)) = log_offsets
                .iter()
                .find(|(c, _, _)| *c == LogColumn::Graph)
                .map(|(_, x, w)| (*x, *w))
            {
                let top = log_top_y(log_area);
                let bottom = log_area.y + log_area.height.saturating_sub(1);
                let rows = bottom.saturating_sub(top);
                if gw > 0 && rows > 0 {
                    let window = graph::ColumnWindow {
                        rows,
                        offset: log_state.offset(),
                        selected: log_state.selected(),
                    };
                    let mut raster_rows = graph::visible_raster_rows(
                        graph_grid,
                        wip_graph_row.as_ref(),
                        stash_graph_rows,
                        graph_order,
                        &window,
                    );
                    // `ui.avatars = "graph"`: each commit's author
                    // picture goes into its circle, once fetched. The
                    // commit rows come from `graph::visible_commit_rows`
                    // -- the same walk of `graph_order` as the raster
                    // rows, so `v.row` indexes `raster_rows` exactly.
                    // Memory lookups only (`AvatarService::pictures`).
                    // `ui.ref_connector`: the picture's one-pixel line
                    // on each visible commit row that carries a pill.
                    // Walked with `graph::visible_commit_rows`, the
                    // same walk of `graph_order` as the raster rows,
                    // so `v.row` indexes `raster_rows` exactly.
                    if let (true, Some(nodes)) = (connector_on, solo_nodes.as_ref()) {
                        for v in graph::visible_commit_rows(graph_order, nodes.len(), &window) {
                            if let Some(row) = raster_rows.get_mut(usize::from(v.row)) {
                                row.ref_connector =
                                    !layout.pills_for(nodes[v.commit].id).is_empty();
                            }
                        }
                    }
                    let in_graph = avatar::in_graph(avatar_placement, avatar_capability);
                    graph_image.set_avatars(in_graph);
                    graph::set_avatars_in_graph(in_graph);
                    graph_image.set_lane_background(ui_lane_background);
                    graph_image.set_tint_strength(ui_tint_strength);
                    if let (true, Some(nodes)) = (in_graph, solo_nodes.as_ref()) {
                        let visible = graph::visible_commit_rows(graph_order, nodes.len(), &window);
                        graph::attach_node_avatars(
                            &mut raster_rows,
                            nodes,
                            &visible,
                            avatar_service,
                        );
                    }
                    // Clipped for the same reason `render_clipped`
                    // exists: an out-of-bounds rect indexes the
                    // buffer and panics the process, and this rect is
                    // built out of a resizable column's offset.
                    let area = clip_to_frame(frame.area(), Rect::new(gx, top, gw, rows));
                    if compact_graph {
                        graph::compact_raster_rows(&mut raster_rows);
                    } else {
                        // Cells scrolled off at the left are not drawn.
                        let cut = log_geo
                            .span_of(LogColumn::Graph)
                            .map_or(0, |(x, _)| usize::from(log_geo.h_offset.saturating_sub(x)));
                        if cut > 0 {
                            for row in &mut raster_rows {
                                row.cells.drain(..cut.min(row.cells.len()));
                            }
                        }
                    }
                    // Where each picture landed, for the hover tooltip
                    // (`graph::avatar_hover`): the rows that got one, after
                    // the compact / scrolled-off reductions above.
                    if let (true, Some(nodes)) = (in_graph, solo_nodes.as_ref()) {
                        let cell = graph_image.capability().cell;
                        avatar_slots =
                            graph::visible_commit_rows(graph_order, nodes.len(), &window)
                                .into_iter()
                                .filter_map(|v| {
                                    let row = raster_rows.get(usize::from(v.row))?;
                                    row.avatar.as_ref()?;
                                    let node =
                                        u16::try_from(graph::node_column(&row.cells)?).ok()?;
                                    let (first, count) = graph::picture_cells(cell, node);
                                    let slot = Rect::new(gx + first, top + v.row, count, 1)
                                        .intersection(area);
                                    (!slot.is_empty()).then_some((slot, nodes[v.commit].id))
                                })
                                .collect();
                    }
                    graph_image.render(frame.buffer_mut(), area, &raster_rows);
                }
            }
        }
        graph::set_avatar_slots(avatar_slots);

        // The Author column's avatars, as pixels, over the two-letter
        // initials the table just drew in each row's slot -- same
        // place in the frame as the Graph picture and for the same
        // reason (the offset is settled now). Rows are walked with
        // `graph::visible_commit_rows`, i.e. through `graph_order`;
        // a row whose picture is not fetched yet is left alone and
        // keeps its initials. Memory lookups only: no disk, no
        // network on this thread (`AvatarService::pictures`).
        if avatar::log_avatar_mode(avatar_placement, avatar_capability)
            == avatar::LogAvatarMode::Picture
        {
            if let (Some(nodes), Some((ax, aw))) = (
                solo_nodes.as_ref(),
                log_offsets
                    .iter()
                    .find(|(c, _, _)| *c == LogColumn::Author)
                    .filter(|_| {
                        // The slot is the column's two leading cells: a column
                        // cut at the left shows initials only.
                        log_geo
                            .span_of(LogColumn::Author)
                            .is_some_and(|(x, _)| x >= log_geo.h_offset)
                    })
                    .map(|(_, x, w)| (*x, *w)),
            ) {
                let top = log_top_y(log_area);
                let bottom = log_area.y + log_area.height.saturating_sub(1);
                let right = (log_area.x + log_area.width).saturating_sub(1);
                let window = graph::ColumnWindow {
                    rows: bottom.saturating_sub(top),
                    offset: log_state.offset(),
                    selected: log_state.selected(),
                };
                let visible = graph::visible_commit_rows(graph_order, nodes.len(), &window);
                let shas: Vec<String> = visible
                    .iter()
                    .map(|v| nodes[v.commit].id.to_hex())
                    .collect();
                let queries: Vec<avatar::AvatarQuery> = visible
                    .iter()
                    .zip(&shas)
                    .map(|(v, sha)| avatar::AvatarQuery {
                        email: nodes[v.commit].author_email.as_str(),
                        sha,
                    })
                    .collect();
                let pictures = avatar_service.pictures(&queries);
                let slots: Vec<avatar::AvatarSlot> = visible
                    .iter()
                    .zip(pictures)
                    .filter_map(|(v, picture)| {
                        let (author, bitmap) = picture?;
                        let node = &nodes[v.commit];
                        Some(avatar::AvatarSlot {
                            area: avatar::slot_area(ax, aw, top + v.row, right)?,
                            key: avatar::PictureKey {
                                author: author.to_string(),
                                tint: Some(avatar::RowTint::of(
                                    window.selected == Some(v.display),
                                    hovered_commit.is_some_and(|(id, ..)| id == node.id),
                                )),
                                dimmed: searching
                                    && !search_match_set.is_empty()
                                    && !search_match_set.contains(&v.commit),
                            },
                            bitmap,
                        })
                    })
                    .collect();
                log_avatar_images.render(frame.buffer_mut(), &slots);
            }
        }

        // Loading or error message, centered over the whole Log
        // pane -- set above, in the no-graph arm of `log_result`'s
        // match. Drawn after the (empty) table so it sits on top of
        // the blank rows the table drew underneath it.
        if let Some((lines, style)) = &log_center_overlay {
            render_centered_message(frame, log_area, lines, *style);
        }

        // "loading older commits", painted over the Log's *bottom* border (where
        // the loading happens) by the same overlay technique as the find bar on
        // the top border. An overlay, not a second block title, so the note can
        // never change the table's layout and shift a row under a click
        // mid-fetch.
        //
        // Three states, two of which say anything: a page in flight, and stopping
        // at the configured ceiling (else the graph looks simply truncated). The
        // root commit says nothing: the bottom of history is a quiet end.
        if central_view == CentralView::Log {
            let note = if lazy_log.is_loading() {
                Some(format!(
                    " {} ",
                    gitpoulpi_i18n::msg::log_draw::LOADING_OLDER.get()
                ))
            } else if lazy_log.is_at_ceiling() {
                Some(format!(
                    " {} ",
                    gitpoulpi_i18n::msg::log_draw::graph_max_reached(lazy_log.loaded())
                ))
            } else {
                None
            };
            if let Some(note) = note {
                let note_w = display_width(&note) as u16;
                if log_area.width > note_w + 4 && log_area.height > 1 {
                    let note_area = Rect::new(
                        log_area.x + log_area.width - note_w - 2,
                        log_area.y + log_area.height - 1,
                        note_w,
                        1,
                    );
                    frame.render_clipped(
                        Paragraph::new(Span::styled(
                            note,
                            Style::default()
                                .fg(th().ui.muted)
                                .add_modifier(Modifier::ITALIC),
                        )),
                        note_area,
                    );
                }
            }
        }

        // The find bar is painted *over* the table's top border row,
        // right-aligned, after the table itself -- an overlay.
        // Drawn last so it wins the cells; sized to what
        // it needs rather than the full width, so the Log's block
        // title stays readable to its left.
        *log_search_bar_area = None;
        if let InputMode::SearchingLog(search) = &input_mode {
            let bar_w = log_area.width.saturating_sub(4).min(46);
            if bar_w > 12 {
                let bar_area = Rect::new(
                    log_area.x + log_area.width.saturating_sub(bar_w + 1),
                    log_area.y,
                    bar_w,
                    1,
                );
                *log_search_bar_area = Some(bar_area);
                let (bar_line, bar_hits, (query_dx, query_w)) = log_search::search_bar_edit(
                    search,
                    field_editor.caret(MainField::LogFind),
                    search_matches.len(),
                    // The counter's slot is sized from the number of
                    // commits the search runs over, not from the live
                    // hit count -- see `search_bar`: a slot that
                    // tracked the hit count would still shift the bar
                    // sideways while typing.
                    solo_nodes.as_ref().map_or(0, |nodes| nodes.len()),
                    bar_w,
                );
                field_editor.drawn(
                    MainField::LogFind,
                    Rect::new(bar_area.x + query_dx, bar_area.y, query_w, 1),
                    false,
                );
                frame.render_clipped(Clear, bar_area);
                frame.render_clipped(
                    // Same popup background every other overlay in
                    // this file uses, so the bar reads as floating
                    // above the table rather than painted into it.
                    Paragraph::new(bar_line).style(Style::default().bg(th().ui.panel)),
                    bar_area,
                );
                for (dx, w, button) in bar_hits {
                    let rect = Rect::new(bar_area.x + dx, bar_area.y, w, 1);
                    if hovered_search_button == Some(button) {
                        frame
                            .buffer_mut()
                            .set_style(rect, crate::theme::plain_button_style(true));
                    }
                    frame_hits.push((rect, HitTarget::SearchBarButton(button)));
                }
            }
        }
    }
}

/// Builds the rows of `window` - one per display index in
/// `window.start..window.end` - through `build`, in display order.
/// The only way the Log makes rows, so "rows built == rows on screen" is
/// pinned in one place (`ROWS_BUILT` counts them under `cfg(test)`).
pub(in crate::event_loop) fn window_rows<'a>(
    order: &graph::LogRows,
    commits: usize,
    window: &LogWindow,
    mut build: impl FnMut(usize, graph::LogEntry) -> Row<'a>,
) -> Vec<Row<'a>> {
    let mut rows = Vec::with_capacity(window.end.saturating_sub(window.start));
    for display in window.start..window.end {
        let entry = order.entry(display);
        if matches!(entry, graph::LogEntry::Commit(i) if i >= commits) {
            break;
        }
        #[cfg(test)]
        ROWS_BUILT.with(|n| n.set(n.get() + 1));
        rows.push(build(display, entry));
    }
    rows
}

#[cfg(test)]
thread_local! {
    /// How many rows `window_rows` has built on this thread.
    static ROWS_BUILT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(befores: &[usize]) -> Vec<graph::StashSlot> {
        befores
            .iter()
            .enumerate()
            .map(|(stash, &before)| graph::StashSlot { before, stash })
            .collect()
    }

    #[test]
    fn rows_built_equal_the_visible_rows_not_the_loaded_commits() {
        let commits = 5_000;
        let order = graph::LogRows::new(true, slots(&[0, 3, 40, 4_990]));
        let total = order.len(commits);
        for (offset, selected, visible) in [
            (0, Some(0), 30),
            (0, None, 30),
            (1_200, Some(1_210), 30),
            (1_200, Some(4_000), 30),
            (total - 5, None, 30),
            (total - 1, Some(total - 1), 1),
            (10, Some(0), 0),
        ] {
            let window = log_window(offset, selected, visible, total);
            ROWS_BUILT.with(|n| n.set(0));
            let mut seen = Vec::new();
            let rows = window_rows(&order, commits, &window, |display, entry| {
                seen.push((display, entry));
                Row::default()
            });
            let built = ROWS_BUILT.with(|n| n.get());
            let expected = visible.min(total - window.start);
            assert_eq!(built, expected, "offset={offset} sel={selected:?}");
            assert_eq!(rows.len(), built);
            // In display order, each through `LogRows::entry`.
            for (k, (display, entry)) in seen.iter().enumerate() {
                assert_eq!(*display, window.start + k);
                assert_eq!(*entry, order.entry(*display));
            }
        }
    }
}
