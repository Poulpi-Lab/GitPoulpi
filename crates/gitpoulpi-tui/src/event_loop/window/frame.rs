//! Draws one frame: clears the per-frame popup rects, drives the chrome,
//! sidebar, central column, right panel, footer and overlays through their
//! draw contexts, then flushes the terminal-image writes.

use super::FrameInputs;
use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// A popup or dialog is over the live main window and is not a full page:
    /// the Log's pictures (graph, avatars) stay under it. The cells the dialog
    /// painted cover them, so a picture never crosses its box. Settings, Help
    /// and the other full pages are left out on purpose: they replace the window.
    fn overlay_keeps_the_log(
        input_mode: &InputMode,
        repo_dialogs: &crate::repo_dialogs::RepoDialogs,
    ) -> bool {
        matches!(
            input_mode,
            InputMode::ThemePicker(_)
                | InputMode::LanguagePicker(_)
                | InputMode::AuthorFilter(_)
                | InputMode::WorkspaceSearch(_)
                | InputMode::TabPicker { .. }
                | InputMode::CommandPalette { .. }
        ) || crate::activity_log::is_open()
            || repo_dialogs.is_open()
            || crate::jobs_ui::panel_open()
            || crate::updates::panel::is_open()
            || crate::about::is_open()
            || crate::trust_dialog::is_open()
            || crate::conflict_popup::is_open()
            || crate::ai_sent_dialog::is_open()
            || crate::ai_limit_dialog::is_open()
            || crate::sequence_run::repo_review::is_open()
            || crate::link_confirm::is_open()
    }

    /// Polls the status bar's total and asks for a new count when the refs
    /// moved (`graph::total`). Nothing here waits: the request is a channel
    /// send and the answer is read on a later frame.
    fn tick_commit_total(&mut self) -> graph::TotalShown {
        let exhausted = self.lazy_log.is_exhausted();
        // The key also moves with the hidden-refs filter: another walk,
        // another total.
        let (filter_gen, filter) = self.link.log_filter();
        let key = match &self.refs_result {
            Ok(refs) if !exhausted && !self.still_loading => {
                Some(graph::refs_key(refs) ^ filter_gen.wrapping_mul(0x9E37_79B9_7F4A_7C15))
            }
            _ => None,
        };
        // No refs yet (or a failed read): nothing to key a count on, and
        // nothing is started - but an answer already in flight is still read.
        let requester = self.link.activity_handle().raw.requester();
        // `log.commit_count`, live: a change drops the old total and recounts.
        let method = crate::live_settings::commit_count();
        self.commit_total
            .tick(method, key.unwrap_or(0), exhausted || key.is_none(), || {
                requester.call(
                    gitpoulpi_core::domain::OperationKind::CountCommits,
                    serde_json::json!({
                        // The same walk as the graph (`all_refs`, the same
                        // hidden refs), or the total would not be what
                        // scrolling to the end loads.
                        "opts": gitpoulpi_core::domain::LogOptions {
                            all_refs: true,
                            hidden: filter,
                            ..Default::default()
                        },
                        "method": method.as_str(),
                    }),
                )
            });
        self.commit_total.shown(self.lazy_log.loaded(), exhausted)
    }

    /// The one place a frame is drawn and its image writes are flushed.
    pub(in crate::event_loop) fn draw_frame(&mut self, fi: &FrameInputs) -> io::Result<()> {
        let &FrameInputs {
            ref journal_state,
            undo_busy,
            undo_avail,
            redo_avail,
            ref unpushed_tags,
            palette_ctx,
        } = fi;
        let commit_total = self.tick_commit_total();
        let conflict_prediction = self.tick_conflict_badge();

        // Cleared before the draw, not after the palette closes: the
        // rects are only ever valid for the frame that produced them,
        // and a stale one is a click landing on a popup that is no
        // longer there.
        crate::scrollbar::begin_frame();
        crate::hscroll::begin_frame(&self.repo.root);
        crate::context_menu::begin_frame();
        crate::popup_list::View::begin_frame();
        crate::help_view::forget();
        self.palette_popup = None;
        self.tab_picker_popup = None;
        // What the Log, the right pane and the Refs sidebar show instead
        // of their content: a stopped link (with its reason and Retry) or
        // the first load still in flight. Only while nothing real is
        // loaded -- a link that stops *after* a good load leaves the last
        // answer on screen and says so in the footer instead.
        let pane_notice: Option<(Vec<String>, Style)> =
            if self.still_loading || (self.link_failure.is_some() && self.log_result.is_err()) {
                ui_kit::pane_notice(
                    self.link_failure.as_deref(),
                    self.still_loading,
                    &self.repo.root,
                    self.repo_open_started_at.elapsed(),
                )
                .map(|(lines, is_error)| {
                    let style = if is_error {
                        Style::default().fg(th().chrome.notice)
                    } else {
                        Style::default().fg(th().ui.muted)
                    };
                    (lines, style)
                })
            } else {
                None
            };
        // On the image path the frame goes out in two writes -- ratatui's
        // diff, then the Graph column's picture and whatever popup cells
        // sit over it (`GraphImage::write_pending`). A synchronized update
        // makes the terminal show them as one, so a picture re-sent under
        // an open popup never flashes over it for a frame. Terminals that
        // do not know mode 2026 ignore it; the text path does not need it
        // and does not send it.
        let synchronized = self.graph_image.capability().draws_images();
        if synchronized {
            // Held back until the frame writes something: an idle frame
            // must not write even this bracket.
            self.terminal.backend_mut().begin_group(b"\x1b[?2026h");
        }
        let mut diff_content_request: Option<FileInsightRequest> = None;
        // Inline branch/tag naming: which commit the box belongs on. The
        // first frame of the prompt brings that commit into view (and
        // selects it), so the box is never typed into off screen.
        let inline_ref: Option<(InlineRefKind, Oid)> = match &self.input_mode {
            InputMode::InlineName { op, .. } => {
                inline_target(op, self.refs_result.as_deref().unwrap_or(&[]))
            }
            _ => None,
        };
        // A palette rename (`Auto`) takes whichever spot is already on
        // screen; it never scrolls the Log to its tip.
        let reveals = !matches!(
            &self.input_mode,
            InputMode::InlineName {
                op: RefNameOp::RenameBranch {
                    spot: RenameSpot::Auto,
                    ..
                },
                ..
            }
        );
        match inline_ref {
            Some((_, at)) if !self.ref_name_revealed => {
                if reveals
                    && self.solo_refs.is_empty()
                    && self.central_view == CentralView::Log
                    && select_log_commit(&self.log_result, &self.log_order, &mut self.log_state, at)
                {
                    self.log_follow = true;
                }
                self.ref_name_revealed = true;
            }
            Some(_) => {}
            // A new file keeps its own first-frame flag below.
            None if matches!(
                &self.input_mode,
                InputMode::InlineName {
                    op: RefNameOp::CreateFile(_),
                    ..
                }
            ) => {}
            None => self.ref_name_revealed = false,
        }
        // A new file is named in the Status list: on the first frame the
        // list is brought on screen (the working changes, not a commit's
        // detail) and the folder it goes in is opened.
        if let InputMode::InlineName {
            op: RefNameOp::CreateFile(folder),
            ..
        } = &self.input_mode
        {
            if !self.ref_name_revealed {
                self.right_panel = RightPanel::Status;
                self.right_pane_collapsed = false;
                if !folder.as_os_str().is_empty() {
                    self.collapsed_status_folders
                        .remove(&crate::status_list::fold_key(
                            StatusSection::Unstaged,
                            folder,
                        ));
                }
                self.ref_name_revealed = true;
            }
        }
        // The prompts named inline (or in the name box) keep their error
        // for as long as the mode lasts; every other prompt has none.
        if !matches!(&self.input_mode, InputMode::InlineName { .. }) {
            self.ref_name_error = None;
            self.ref_name_slot.reset();
            self.ref_name_revealed = false;
        }
        let mut name_slot = self.ref_name_slot;
        let mut inline_ref_row: Option<u16> = None;
        self.sync_fields();
        self.field_editor.begin_frame();
        crate::pointer_map::begin_frame();
        crate::dialog_kit::frame::begin_frame();
        // What the link is busy with, once it has taken a moment (the
        // spinner animates because the loop redraws on every poll).
        let in_flight_text = self.link.in_flight_text();
        let agent_busy = self.agent_input_busy();
        let completed = self
            .terminal
            .draw(|frame| {
                // A theme that paints its own background (`ui.bg` is not the
                // terminal's) fills the whole frame first; Dark leaves it alone.
                if th().ui.bg != Color::Reset {
                    frame.render_widget(
                        ratatui::widgets::Block::default().style(Style::default().bg(th().ui.bg)),
                        frame.area(),
                    );
                }
                // Set again below only if the WIP View all files list is
                // drawn this frame; otherwise it must not catch the mouse.
                self.wip_all.area = Rect::default();
                // Declared ahead of the title bar block so the close button, painted
                // into the outer title bar, registers its hit target in the same list as
                // everything else.
                let mut frame_hits: Vec<(Rect, HitTarget)> = Vec::new();
                self.overflow_slots.clear();
                let tabs_col = crate::live_settings::chrome_layout()
                    .effective(frame.area().width, self.management.is_some())
                    .tabs_column_width();
                let inner_width = frame
                    .area()
                    .width
                    .saturating_sub(2)
                    .saturating_sub(tabs_col);
                let footer_rows =
                    prefix_hint_rows(&self.input_mode, self.ai_ui.enabled(), inner_width).len()
                        as u16;
                let ChromeAreas {
                    inner,
                    tab_bar_area,
                    toolbar_area,
                    footer_area,
                } = draw_chrome(
                    frame,
                    &mut ChromeDrawCtx {
                        footer_rows,
                        active_tab: self.active_tab,
                        divider1_pct: self.divider1_pct,
                        divider2_pct: self.divider2_pct,
                        left_pane_collapsed: self.left_pane_collapsed,
                        right_pane_collapsed: self.right_pane_collapsed,
                        undo_avail,
                        redo_avail,
                        auto_stash_pending: self.pending_auto_stash.is_some(),
                        lfs_hidden: self.lfs.toolbar_hidden(),
                        hovered_close_tab: self.hovered_close_tab,
                        hovered_tab_index: self.hovered_tab_index,
                        title: &self.title,
                        repo_tabs: &self.repo_tabs,
                        hovered_target: &self.hovered_target,
                        management: &self.management,
                        hosting: &self.hosting,
                        stashes_result: &self.stashes_result,
                        frame_hits: &mut frame_hits,
                        overflow_slots: &mut self.overflow_slots,
                        remote_ui: &mut self.remote_ui,
                        tab_hits: &mut self.tab_hits,
                        panes_area: &mut self.panes_area,
                        refs_area: &mut self.refs_area,
                        log_area: &mut self.log_area,
                        status_area: &mut self.status_area,
                    },
                );

                // Color only: the toolbar row and the selected tab share
                // `ui.toolbar_bg`.
                crate::theme::paint_toolbar_bg(frame.buffer_mut(), toolbar_area);
                let leftover_remotes = crate::remotes_ui::leftover_names(
                    self.refs_result.as_deref().unwrap_or(&[]),
                    &self.remote_names,
                    self.remote_ui.loaded,
                );
                draw_sidebar(
                    frame,
                    &mut SidebarDrawCtx {
                        focus: self.focus,
                        left_pane_collapsed: self.left_pane_collapsed,
                        hierarchical_refs: self.hierarchical_refs,
                        active_ref_section: self.active_ref_section,
                        refs_area: self.refs_area,
                        hovered_target: &self.hovered_target,
                        hidden_from_graph: &self.hidden_from_graph,
                        collapsed_ref_sections: &self.collapsed_ref_sections,
                        collapsed_ref_folders: &self.collapsed_ref_folders,
                        hovered_ref_row: &self.hovered_ref_row,
                        hovered_ref_folder: &self.hovered_ref_folder,
                        input_mode: &self.input_mode,
                        ref_drag: &self.ref_drag,
                        ref_drop_mark: &self.ref_drop_mark,
                        refs_filter: &self.refs_filter,
                        refs_result: &self.refs_result,
                        remote_names: &self.remote_names,
                        leftover_remotes: &leftover_remotes,
                        sidebar_states: &mut self.sidebar_states,
                        solo_refs: &self.solo_refs,
                        submodules_result: &self.submodules_result,
                        unpushed_tags,
                        worktrees_result: &self.worktrees_result,
                        stashes_result: &self.stashes_result,
                        pane_notice: &pane_notice,
                        sidebar_heights: &self.sidebar_heights,
                        hidden_sidebar_sections: &self.hidden_sidebar_sections,
                        hosting: &self.hosting,
                        frame_hits: &mut frame_hits,
                        field_editor: &mut self.field_editor,
                        refs_filter_area: &mut self.refs_filter_area,
                        sidebar_layout_avail: &mut self.sidebar_layout_avail,
                        sidebar_layout_specs: &mut self.sidebar_layout_specs,
                        sidebar_section_areas: &mut self.sidebar_section_areas,
                        ref_name_error: &self.ref_name_error,
                        name_slot: &mut name_slot,
                    },
                );
                crate::theme::paint_side_bg(frame.buffer_mut(), self.refs_area);
                // A pinned "WIP" row for uncommitted work,
                // ahead of real commits when the working tree isn't clean.
                let has_wip = matches!(&self.status_result, Ok(tree) if !tree.files.is_empty());
                // Refs the user hid from the graph (right-click a sidebar row
                // -> "Hide in graph") are filtered out here -- everything
                // downstream of `refs_for_pills` (the graph's ● vs ○ glyph
                // choice, the Log column's pills, the ref-reveal popup) only
                // ever sees the non-hidden set. The sidebar itself still shows
                // every ref (dimmed when hidden) since it reads `refs_result`
                // directly, not this filtered copy.
                //
                // Solo also filters which ref pills render on the remaining rows: only
                // pills for currently-soloed refs show (branches and tags alike), even on
                // a visible ancestor of a soloed branch.
                let log_layout = live_settings::log_columns_layout();
                let mut log_geo = LogColumnsGeo::compute(
                    self.log_area,
                    &self.log_column_order,
                    &self.log_column_hidden,
                    &LogWidthMaps {
                        fit: &self.log_column_overrides,
                        scroll: &self.log_column_scroll_widths,
                    },
                    (log_layout, self.log_h_offset),
                );
                // While the WIP summary is being typed its cell must stay in
                // view: the scroll never passes the start of Message.
                if self.composer_focus == Some(ComposerField::WipSummary) {
                    if let Some((x, _)) = log_geo.span_of(LogColumn::Message) {
                        if log_geo.h_offset > x {
                            log_geo = LogColumnsGeo::compute(
                                self.log_area,
                                &self.log_column_order,
                                &self.log_column_hidden,
                                &LogWidthMaps {
                                    fit: &self.log_column_overrides,
                                    scroll: &self.log_column_scroll_widths,
                                },
                                (log_layout, x),
                            );
                        }
                    }
                }
                // The settled offset is the state: a window that grew, a
                // column that shrank or a layout that changed pulls it back.
                self.log_h_offset = log_geo.h_offset;
                let log_draw_order: Vec<LogColumn> =
                    log_geo.offsets.iter().map(|(c, _, _)| *c).collect();
                let (log_widths, log_visible_order, log_offsets) =
                    (&log_geo.widths, &log_draw_order, &log_geo.offsets);
                // The solo filter, the lane walk, the stash placement and the
                // search matches are functions of the inputs below only, so
                // they are rebuilt when one of those changes -- not on every
                // poll-tick redraw (`graph::LogLayoutCache`).
                // `render` is the only thing the terminal's capabilities
                // change about the graph: under `Image` the Graph cells are
                // produced as blanks of exactly the same width, the table
                // lays out identically, and the picture is composited over
                // them after the table is drawn.
                let graph_settings = graph::GraphSettings {
                    render: if self.graph_image.capability().draws_graph() {
                        graph::GraphRender::Image
                    } else {
                        graph::GraphRender::Text
                    },
                    lane_background: self.ui_lane_background,
                    tint_strength: self.ui_tint_strength,
                    theme: self.ui_graph_theme,
                    ..Default::default()
                };
                // Matches are over the solo-filtered nodes: searching rows
                // that aren't on screen, and counting them in "n of N", would
                // be a lie. Indices are into `solo_nodes`; add `wip_offset` to
                // get a Log display index.
                let search_query = match &self.input_mode {
                    InputMode::SearchingLog(search) => Some(search.query.as_str()),
                    _ => None,
                };
                self.log_layout.update(
                    &graph::LayoutInputs {
                        log: &self.log_result,
                        log_gen: self.log_gen,
                        refs: &self.refs_result,
                        refs_gen: self.refs_gen,
                        stashes: &self.stashes_result,
                        stashes_gen: self.stashes_gen,
                        solo: &self.solo_refs,
                        hidden: &self.hidden_from_graph,
                        has_wip,
                        show_stashes: self.graph_show_stashes,
                        settings: graph_settings,
                    },
                    search_query,
                    &self.log_author_filter,
                );
                let author_names = self.log_layout.author_names(&self.log_author_filter);
                let author_filter_note = (!author_names.is_empty())
                    .then(|| crate::author_filter::note_text(&author_names));
                crate::author_filter::set_tip(author_names);
                let layout = self.log_layout.layout();
                let refs_for_pills = &layout.refs_for_pills;
                let search_matches = self.log_layout.matches();
                let search_match_set = self.log_layout.match_set();
                let searching = search_query.is_some() || !self.log_author_filter.is_empty();

                // Looked up again later when rendering the ref-reveal popup
                // (`ref_popup`), so its pills get the same lane-colored
                // background as the row they were opened from, instead of a
                // flat/unstyled background that doesn't visually tie back to
                // the commit.
                let mut commit_lane_colors: std::collections::HashMap<Oid, Color> =
                    std::collections::HashMap::new();
                draw_log(
                    frame,
                    &mut LogDrawCtx {
                        log_area: self.log_area,
                        central_view: self.central_view,
                        focus: self.focus,
                        has_wip,
                        no_repository: self.no_repository,
                        log_len: self.log_len,
                        log_follow: self.log_follow,
                        reveal_flash: self.reveal.flash_row(),
                        ref_connector: self.ref_connector,
                        searching,
                        author_filter_on: !self.log_author_filter.is_empty(),
                        author_filter_note,
                        hovered_wip_row: self.hovered_wip_row,
                        hovered_search_button: match &self.hovered_target {
                            Some(HitTarget::SearchBarButton(b)) => Some(*b),
                            _ => None,
                        },
                        hovered_commit: self.hovered_commit,
                        inline_ref,
                        composer_focus: self.composer_focus,
                        avatar_placement: self.avatar_placement,
                        avatar_capability: self.avatar_capability,
                        ui_lane_background: self.ui_lane_background,
                        ui_tint_strength: self.ui_tint_strength,
                        graph_settings,
                        log_widths,
                        log_visible_order,
                        log_natural: &mut self.log_natural_widths,
                        log_col_pick: self.log_col_pick,
                        log_offsets,
                        log_geo: &log_geo,
                        layout,
                        search_matches,
                        search_match_set,
                        lazy_log: &self.lazy_log,
                        input_mode: &self.input_mode,
                        status_result: &self.status_result,
                        log_result: &self.log_result,
                        pane_notice: &pane_notice,
                        ref_popup: &self.ref_popup,
                        ref_name_error: &self.ref_name_error,
                        log_multi: &self.log_multi,
                        commit_summary: &self.commit_summary,
                        commit_changes: &self.commit_changes,
                        avatar_live: &self.avatar_live,
                        log_state: &mut self.log_state,
                        field_editor: &mut self.field_editor,
                        frame_hits: &mut frame_hits,
                        graph_image: &mut self.graph_image,
                        log_avatar_images: &mut self.log_avatar_images,
                        avatar_service: &mut self.avatar_service,
                        visible_commit_ids: &mut self.visible_commit_ids,
                        log_search_bar_area: &mut self.log_search_bar_area,
                        inline_ref_row: &mut inline_ref_row,
                        name_slot: &mut name_slot,
                        commit_lane_colors: &mut commit_lane_colors,
                    },
                );
                draw_central(
                    frame,
                    &mut CentralDrawCtx {
                        central_view: self.central_view,
                        log_area: self.log_area,
                        hovered_diff_row: self.hovered_diff_row,
                        hovered_target: &self.hovered_target,
                        commit_files: &self.commit_files,
                        working_diff: &self.working_diff,
                        lfs: &self.lfs,
                        frame_hits: &mut frame_hits,
                        overflow_slots: &mut self.overflow_slots,
                        diff_bar_area: &mut self.diff_bar_area,
                        diff_body_area: &mut self.diff_body_area,
                        diff_longest_line: &mut self.diff_longest_line,
                        diff_rendered_len: &mut self.diff_rendered_len,
                        diff_hunk_starts: &mut self.diff_hunk_starts,
                        diff_rows: &mut self.diff_rows,
                        diff_render: &mut self.diff_render,
                        diff_content_request: &mut diff_content_request,
                        file_insight_area: &mut self.file_insight_area,
                        file_insight: &mut self.file_insight,
                        history: &mut self.history,
                        avatar_fallback: self.avatar_live.fallback,
                        diff_view_state: &mut self.diff_view_state,
                        editor: &mut self.editor,
                        merge: &mut self.merge,
                        conflicts: &self.conflicts,
                        diff_minimap: &mut self.diff_minimap,
                        diff_content: &mut self.diff_content,
                        avatar_placement: self.avatar_placement,
                        avatar_capability: self.avatar_capability,
                        avatar_live: &self.avatar_live,
                        avatar_service: &mut self.avatar_service,
                        log_avatar_images: &mut self.log_avatar_images,
                        commit_col: &mut self.commit_col,
                        commit_changes: &self.commit_changes,
                        layout,
                        selected_commit: selected_log_commit(
                            &self.log_result,
                            &self.log_state,
                            &self.log_order,
                        ),
                        more_commits: !self.lazy_log.is_exhausted(),
                        node_set: graph_settings.node_set,
                        commit_graph_image: &mut self.commit_graph_image,
                        ui_lane_background: self.ui_lane_background,
                        ui_tint_strength: self.ui_tint_strength,
                    },
                );

                draw_right_panel(
                    frame,
                    &mut RightPanelDrawCtx {
                        active_status_section: self.active_status_section,
                        ai_ui: &self.ai_ui,
                        avatar_placement: self.avatar_placement,
                        avatar_capability: self.avatar_capability,
                        avatar_live: &self.avatar_live,
                        avatar_service: &mut self.avatar_service,
                        log_avatar_images: &mut self.log_avatar_images,
                        collapsed_commit_folders: &self.collapsed_commit_folders,
                        collapsed_status_folders: &self.collapsed_status_folders,
                        collapsed_status_sections: &self.collapsed_status_sections,
                        commit_amend: self.commit_amend,
                        commit_skip_hooks: self.commit_skip_hooks,
                        merging: matches!(
                            self.operation_state,
                            Some(gitpoulpi_core::domain::OperationState::Merge { .. })
                        ),
                        commit_description: &self.commit_description,
                        commit_detail_status_height: &mut self.commit_detail_status_height,
                        commit_file_focus: self.commit_file_focus,
                        commit_file_rows: &mut self.commit_file_rows,
                        commit_file_state: &mut self.commit_file_state,
                        commit_files: &self.commit_files,
                        commit_files_area: &mut self.commit_files_area,
                        commit_files_filter: &self.commit_files_filter,
                        commit_files_follow: self.commit_files_follow,
                        commit_files_len: &mut self.commit_files_len,
                        commit_files_visible_rows: &mut self.commit_files_visible_rows,
                        commit_filter_focus: self.commit_filter_focus,
                        commit_list_mode: self.commit_list_mode,
                        commit_message_area: &mut self.commit_message_area,
                        commit_message_text: &mut self.commit_text,
                        commit_message_for: &mut self.commit_message_for,
                        commit_message_rows_now: &mut self.commit_message_rows_now,
                        commit_message_scroll: &mut self.commit_message_scroll,
                        commit_message_total: &mut self.commit_message_total,
                        commit_message_user_rows: self.commit_message_user_rows,
                        commit_meta_area: &mut self.commit_meta_area,
                        commit_paths_cache: &mut self.commit_paths_cache,
                        commit_ref_urls: &mut self.commit_ref_urls,
                        commit_sort_desc: self.commit_sort_desc,
                        commit_summary: &self.commit_summary,
                        composer_focus: self.composer_focus,
                        conflicts: &self.conflicts,
                        detail_avatar_cols: self.detail_avatar_cols,
                        detail_avatar_image: &mut self.detail_avatar_image,
                        detail_avatar_rows: self.detail_avatar_rows,
                        field_editor: &self.field_editor,
                        focus: self.focus,
                        frame_hits: &mut frame_hits,
                        hosting: &mut self.hosting,
                        hovered_status_row: self.hovered_status_row,
                        hovered_target: &self.hovered_target,
                        lfs: &self.lfs,
                        link_failure: &self.link_failure,
                        log_order: &self.log_order,
                        log_result: &self.log_result,
                        log_state: &self.log_state,
                        overflow_slots: &mut self.overflow_slots,
                        pane_notice: &pane_notice,
                        refs_result: &self.refs_result,
                        right_panel: self.right_panel,
                        stashes_result: &self.stashes_result,
                        status_area: self.status_area,
                        status_list_mode: self.status_list_mode,
                        status_result: &self.status_result,
                        status_section_areas: &mut self.status_section_areas,
                        status_section_lines: &mut self.status_section_lines,
                        status_states: &mut self.status_states,
                        view_all_files: self.view_all_files,
                        compare: &self.compare,
                        wip_all: &mut self.wip_all,
                        file_sel: &self.file_sel,
                        input_mode: &self.input_mode,
                        ref_name_error: &self.ref_name_error,
                        name_slot: &mut name_slot,
                    },
                );

                crate::theme::paint_side_bg(frame.buffer_mut(), self.status_area);
                let home_mode_hint = draw_footer(
                    frame,
                    &mut FooterDrawCtx {
                        ai_ui: &self.ai_ui,
                        footer_area,
                        in_flight_text: &in_flight_text,
                        input_mode: &self.input_mode,
                        last_message: &self.last_message,
                        commit_total,
                        log_result: &self.log_result,
                        pending_count: self.pending_count,
                        ref_drag: &self.ref_drag,
                        refs_result: &self.refs_result,
                        repo_open_started_at: self.repo_open_started_at,
                        session: &self.session,
                        sparse: self.sparse.enabled,
                        status_result: &self.status_result,
                    },
                );
                crate::theme::paint_statusbar(frame.buffer_mut(), footer_area);

                if let Some(prefs) = self.preferences.as_mut() {
                    prefs.note_picker(matches!(
                        self.input_mode,
                        InputMode::ThemePicker(_) | InputMode::LanguagePicker(_)
                    ));
                }
                draw_overlays(
                    frame,
                    &mut OverlaysDrawCtx {
                        active_tab: self.active_tab,
                        central_view: self.central_view,
                        commit_files_area: self.commit_files_area,
                        editor: &self.editor,
                        merge: &self.merge,
                        field_editor: &self.field_editor,
                        file_insight: &self.file_insight,
                        file_insight_area: &mut self.file_insight_area,
                        frame_hits: &mut frame_hits,
                        home_mode_hint: &home_mode_hint,
                        hovered_target: &self.hovered_target,
                        inner,
                        input_mode: &self.input_mode,
                        journal_state,
                        left_pane_collapsed: self.left_pane_collapsed,
                        log_area: self.log_area,
                        commit_column_area: self.commit_col.area,
                        view_area: self.commit_col.view,
                        management: &mut self.management,
                        management_panel: &mut self.management_panel,
                        nav_badge_targets: &mut self.nav_badge_targets,
                        palette_ctx,
                        palette_over_dialog: self.repo_dialogs.is_open(),
                        settings_open: self.preferences.is_some(),
                        palette_popup: &mut self.palette_popup,
                        rebase_state: &self.rebase_state,
                        operation_state: &self.operation_state,
                        conflict_count: self.conflicts.len(),
                        index_lock: &self.index_lock,
                        pending_auto_stash: &self.pending_auto_stash,
                        repo_root: &self.repo_root,
                        conflict_prediction: &conflict_prediction,
                        refs_filter_area: self.refs_filter_area,
                        repo_tabs: &self.repo_tabs,
                        right_pane_collapsed: self.right_pane_collapsed,
                        right_panel: self.right_panel,
                        sidebar_section_areas: &self.sidebar_section_areas,
                        ref_name_error: &self.ref_name_error,
                        name_slot: &mut name_slot,
                        status_area: self.status_area,
                        status_section_areas: &self.status_section_areas,
                        tab_bar_area,
                        tab_picker_popup: &mut self.tab_picker_popup,
                        ui_graph_theme: self.ui_graph_theme,
                        undo_busy,
                    },
                );

                draw_overlay_menus(
                    frame,
                    &mut OverlayMenusDrawCtx {
                        mode_hint: &home_mode_hint,
                        nav_badge_targets: &mut self.nav_badge_targets,
                        ai_ui: &self.ai_ui,
                        commit_changes: &self.commit_changes,
                        commit_lane_colors: &commit_lane_colors,
                        diff_line_drag: self.diff_line_drag,
                        dragging_commit_message: self.dragging_commit_message,
                        dragging_divider: self.dragging_divider,
                        dragging_list_bar: self.dragging_list_bar,
                        dragging_log_boundary: self.dragging_log_boundary,
                        dragging_log_header: self.dragging_log_header,
                        commit_header_drag: self.commit_col.dragging_header.and_then(|part| {
                            self.commit_col.header_drag_at.map(|at| {
                                (
                                    part.title(),
                                    at,
                                    self.commit_col.header_drop.map(|(_, m)| m),
                                )
                            })
                        }),
                        dragging_scrollbar: self.dragging_scrollbar,
                        dragging_sidebar_section: self.dragging_sidebar_section,
                        frame_hits: &mut frame_hits,
                        help_filter: &self.help_filter,
                        help_popup_inner_height: &mut self.help_popup_inner_height,
                        help_scroll: &mut self.help_scroll,
                        hosting: &mut self.hosting,
                        hovered_commit: self.hovered_commit,
                        hovered_repo_menu_row: self.hovered_repo_menu_row,
                        hovered_resize_strip: self.hovered_resize_strip,
                        hovered_target: &self.hovered_target,
                        input_mode: &self.input_mode,
                        journal_state,
                        log_area: self.log_area,
                        log_column_hidden: &self.log_column_hidden,
                        log_result: &self.log_result,
                        log_widths,
                        open_menu: &self.open_menu,
                        palette_ctx,
                        palette_popup: &mut self.palette_popup,
                        field_editor: &self.field_editor,
                        preferences: &mut self.preferences,
                        tab_picker_popup: &mut self.tab_picker_popup,
                        ref_drag: &self.ref_drag,
                        ref_drop_mark: &self.ref_drop_mark,
                        ref_popup: &self.ref_popup,
                        ref_popup_rect: &mut self.ref_popup_rect,
                        refs_for_pills,
                        log_layout: layout,
                        remote_ui: &self.remote_ui,
                        repo_dialogs: &mut self.repo_dialogs,
                        repo_menu: &self.repo_menu,
                        repo_menu_rects: &mut self.repo_menu_rects,
                        show_help: self.show_help,
                        text_drag: &self.text_drag,
                        tooltip: &mut self.tooltip,
                        undo_busy,
                    },
                );

                // A carried tab's ghost, marker and dimmed source: over every
                // popup (the tab picker's rows can be the thing dragged).
                if let Some(drag) = self.tab_drag.as_ref() {
                    crate::tab_drag::paint(frame.buffer_mut(), drag);
                }

                // The author popup's real avatars, over the badges it just
                // drew; `finish_frame` below clears or re-sends them as the
                // list scrolls, filters or closes.
                if let (InputMode::AuthorFilter(popup), Some(rects)) =
                    (&self.input_mode, self.tab_picker_popup.as_ref())
                {
                    crate::author_filter::paint_pictures(
                        frame.buffer_mut(),
                        popup,
                        rects,
                        &mut self.avatar_service,
                        &mut self.log_avatar_images,
                        avatar::card_avatar_mode(self.avatar_placement, self.avatar_capability),
                    );
                }

                // **Last, after every popup and screen above.** It decides
                // whether the Graph column's picture has to be sent again by
                // looking at what ended up drawn over it -- a tooltip that has
                // just closed leaves its text printed over the image unless
                // the picture is re-sent. Anything painted after this line is
                // invisible to that check.
                self.ui_viewer.render(frame);
                // The palette and its pickers paint over the live app (the
                // theme / language / graph color pickers preview on it): the
                // Log's graph picture stays under them.
                let keep = Self::overlay_keeps_the_log(&self.input_mode, &self.repo_dialogs);
                self.graph_image.keep_under_overlay(keep);
                self.graph_image.finish_frame(frame.buffer_mut());
                // The commit column's picture and the Log's avatars, the
                // same overlay rule.
                self.commit_graph_image.keep_under_overlay(keep);
                self.log_avatar_images.keep_under_overlay(keep);
                self.commit_graph_image.finish_frame(frame.buffer_mut());
                self.detail_avatar_image.finish_frame(frame.buffer_mut());
                minimap::finish_frame(frame.buffer_mut());
                crate::about::finish_frame(frame.buffer_mut());
                crate::title_logo::finish_frame(frame.buffer_mut());
                crate::image_view::pane::finish_frame(frame.buffer_mut());
                crate::image_view::split::finish_frame(frame.buffer_mut());
                self.log_avatar_images.finish_frame(frame.buffer_mut());

                self.hit_targets = frame_hits;

                // A screen that starts with `Clear` (Settings, the full-page
                // views) wipes the root fill above: whatever nothing painted
                // takes the theme's own background, so a light theme never
                // shows the terminal's dark one through, and text with no
                // color of its own takes the theme's text color (the
                // terminal's default foreground may be pale too). Dark
                // (`Reset`) skips this. The kitty placeholder cells carry
                // their image id in the foreground and are left alone.
                if th().ui.bg != Color::Reset {
                    for cell in frame.buffer_mut().content.iter_mut() {
                        crate::terminal_image::paint_cell(cell);
                    }
                }
                // `ui.color_depth`: a reduced-color terminal gets the nearest
                // 256/16 entries (a no-op at truecolor).
                crate::color_depth::apply(frame.buffer_mut());
                // `ui.text_cursor = terminal`: the real cursor goes to the
                // focused field's caret (and is hidden when there is none).
                if let Some(at) = crate::text_cursor::take_from_buffer(frame.buffer_mut()) {
                    frame.set_cursor_position(at);
                }
                crate::scrollbar::retain_visible_arrows(frame.buffer_mut());
            })
            .map_err(exit_log::io_context("drawing a frame"))?;
        name_slot.end_frame();
        self.ref_name_slot = name_slot;
        let agent_diff = super::agent_look::diff_state_json(
            self.central_view,
            &self.diff_view_state,
            &self.diff_render,
            self.diff_body_area,
        );
        let agent_file = super::agent_look::file_state_json(
            self.central_view,
            &self.file_insight,
            self.file_insight_area,
            self.history.as_ref(),
        );
        self.ui_viewer.frame(
            self.link.ui(),
            &crate::ui_viewer::Facts {
                active_repo: self.repo.id.clone(),
                tabs: (self.repo_tabs.iter().enumerate())
                    .map(|(i, t)| (RepoId::from_path(&t.path), i == self.active_tab))
                    .collect(),
                central: match (self.management.is_some(), self.central_view) {
                    (true, _) => "home",
                    (_, CentralView::Log) => "log",
                    (_, CentralView::Diff) => "diff",
                    (_, CentralView::File) => "file",
                    (_, CentralView::Editor) => "editor",
                    (_, CentralView::Merge) => "merge",
                },
                focus: match self.focus {
                    Focus::Refs => "refs",
                    Focus::Log => "log",
                    Focus::Status => "status",
                },
                right_panel: match self.right_panel {
                    RightPanel::Status => "status",
                    RightPanel::CommitDetail => "commit_detail",
                },
                overlay: if self.preferences.is_some() {
                    Some("settings")
                } else if self.show_help {
                    Some("help")
                } else if matches!(self.input_mode, InputMode::CommandPalette { .. }) {
                    Some("palette")
                } else if matches!(self.input_mode, InputMode::TabPicker { .. }) {
                    Some("tab_picker")
                } else if matches!(self.input_mode, InputMode::ThemePicker(_)) {
                    Some("theme_picker")
                } else if matches!(self.input_mode, InputMode::LanguagePicker(_)) {
                    Some("language_picker")
                } else if matches!(self.input_mode, InputMode::AuthorFilter(_)) {
                    Some("author_filter")
                } else if matches!(self.input_mode, InputMode::WorkspaceSearch(_)) {
                    Some("workspace_search")
                } else if self.repo_dialogs.is_open()
                    || self.remote_ui.is_open()
                    || self.ai_ui.popup_open()
                {
                    Some("dialog")
                } else {
                    None
                },
                input_busy: agent_busy,
                settings: self.preferences.as_ref().map(|p| p.view_state()),
                log_selected: self.log_state.selected(),
                selected_commit: selected_log_commit(
                    &self.log_result,
                    &self.log_state,
                    &self.log_order,
                )
                .map(|oid| oid.to_hex()),
                central_area: self.log_area,
                diff: agent_diff,
                file: agent_file,
                merge: self.merge.as_ref().map(|v| v.agent_state()),
            },
            completed.buffer,
        );
        // A tooltip that just disappeared: force the terminal to rewrite
        // every cell it covered from the finished frame, so nothing the
        // ratatui diff wrongly believes is already gone (wide glyphs,
        // graphics) survives as a ghost. Image-owned (`skip`) cells are left
        // to their own protocol.
        if let Some(rect) = self.tooltip.take_repaint() {
            let rect = rect.intersection(completed.area);
            let mut cells = Vec::new();
            for y in rect.top()..rect.bottom() {
                for x in rect.left()..rect.right() {
                    let cell = &completed.buffer[(x, y)];
                    if cell.diff_option != ratatui::buffer::CellDiffOption::Skip {
                        cells.push((x, y, cell.clone()));
                    }
                }
            }
            use ratatui::backend::Backend as _;
            self.terminal
                .backend_mut()
                .draw(cells.iter().map(|(x, y, c)| (*x, *y, c)))
                .and_then(|()| self.terminal.backend_mut().flush())
                .map_err(exit_log::io_context("repainting a closed tooltip"))?;
        }
        if !self.open_first_frame_logged {
            self.open_first_frame_logged = true;
            open_trace("first frame drawn", self.open_t0);
        }
        self.graph_image
            .write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the graph picture"))?;
        let column_picture = "sending the column's picture"; // i18n-english
        self.commit_graph_image
            .write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context(column_picture))?;
        self.detail_avatar_image
            .write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the commit-detail avatar"))?;
        self.log_avatar_images
            .write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the Log avatars"))?;
        minimap::write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the minimap"))?;
        crate::about::write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the About logo"))?;
        crate::title_logo::write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the title-bar logo"))?;
        crate::image_view::pane::write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the image view"))?;
        crate::image_view::split::write_pending(self.terminal.backend_mut())
            .map_err(exit_log::io_context("sending the image diff"))?;
        // Kitty pictures dropped this frame (a changed graph, an evicted
        // avatar): ratatui has overwritten their cells, so free them.
        terminal_image::write_retired(self.terminal.backend_mut())
            .map_err(exit_log::io_context("deleting retired pictures"))?;
        // The pictures above wrote raw bytes, which move the cursor: put it
        // back on the caret (the backend skips this when nothing moved it).
        if let Some(at) = crate::text_cursor::last() {
            use ratatui::backend::Backend as _;
            let backend = self.terminal.backend_mut();
            backend
                .set_cursor_position(at)
                .and_then(|()| backend.show_cursor())
                .map_err(exit_log::io_context("placing the text cursor"))?;
        }
        crate::text_cursor::sync_style(self.terminal.backend_mut());
        if synchronized {
            self.terminal
                .backend_mut()
                .end_group(b"\x1b[?2026l")
                .map_err(exit_log::io_context("ending a synchronized update"))?;
        }
        // The whole-file read the Diff view's draw asked
        // for (sent here, off the closure; the answer comes back on
        // `file_insight_rx` and `diff_content.offer` takes it).
        if let Some(request) = diff_content_request.take() {
            self.link.request_file_insight(request);
        }
        // The re-reads in a chosen encoding the Diff view (or the review) asked for.
        for request in crate::diff_recode::take_requests() {
            self.link.request_file_insight(request);
        }
        // An unstaged picture changed on disk since it was read.
        crate::image_view::split::refresh_worktree(&self.repo_root);
        // The image diff's reads (its before and after pictures).
        for request in crate::image_view::split::take_requests() {
            self.link.request_file_insight(request);
        }

        // Lazy loading of older history, decided against what was just
        // *drawn* rather than against a guess: `log_area` and
        // `log_state`'s offset are both settled by the render pass, the
        // same discipline the rest of this loop follows. Asking here also
        // means the question is answered before the input drain below, so
        // the fetch is already on its way while the user is still holding
        // the key that got them here.
        //
        // Nothing blocks: `request_log_page` is a channel send onto the
        // link thread and the answer is picked up at the top of a later
        // iteration.
        if self.central_view == CentralView::Log {
            let visible_rows = log_visible_rows(self.log_area);
            let rows_below_view = self
                .log_len
                .saturating_sub(self.log_state.offset() + visible_rows);
            if let Some(request) = self.lazy_log.next_page(rows_below_view, visible_rows) {
                self.lazy_log.mark_requested(request);
                self.link.request_log_page(request);
            }
        } else if self.commit_col.area.width > 0 {
            // The commit column scrolls the same history: the same loader,
            // measured against its own window.
            let visible_rows = self.commit_col.viewport();
            let rows_below_view = self
                .commit_col
                .len
                .saturating_sub(self.commit_col.offset + visible_rows);
            if let Some(request) = self.lazy_log.next_page(rows_below_view, visible_rows) {
                self.lazy_log.mark_requested(request);
                self.link.request_log_page(request);
            }
        }

        // The sections actually drawn this frame, in order -- the single
        // source of truth for keyboard section navigation, so it can
        // never offer a section the user can't see.
        self.visible_sidebar = self.sidebar_section_areas.iter().map(|(s, _)| *s).collect();
        Ok(())
    }
}
