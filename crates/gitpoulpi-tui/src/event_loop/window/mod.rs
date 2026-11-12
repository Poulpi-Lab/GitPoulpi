//! One repository window's state and its per-iteration steps: [`WindowState`]
//! owns the window's state, and the loop is `tick` (timers, polls,
//! normalisation), `draw_frame`, `drain_input`, `after_input` and
//! `apply_outcomes`, in that order.

use crate::event_loop::*;
use init::WindowState;

mod agent_conflict;
mod agent_look;
mod agent_propose;
mod agent_show;
mod agent_view;
mod apply;
mod commit_follow;
mod commit_prefill;
mod conflict_prevention;
mod detail_reads;
mod drain;
mod external_tabs;
mod flow;
mod follow;
mod frame;
mod history;
mod idle;
mod merge_sync;
mod notices;
#[cfg(feature = "hosting")]
mod radar_pending;
mod refs_update;
mod reveal;
mod sizes;
mod tick;

pub(in crate::event_loop) use follow::HostSnapshot;
pub(super) use idle::RedrawState;
pub(super) use tick::{poll_timeout, FrameInputs};

impl WindowState {
    /// Every field a command may touch, borrowed for one `run_command` call
    /// (and for the helpers around it).
    pub(in crate::event_loop) fn command_ctx(&mut self) -> CommandCtx<'_> {
        CommandCtx {
            active_ref_section: &mut self.active_ref_section,
            active_status_section: &mut self.active_status_section,
            active_tab: &mut self.active_tab,
            ai_ui: &mut self.ai_ui,
            avatar_live: &mut self.avatar_live,
            avatar_placement: &mut self.avatar_placement,
            central_view: &mut self.central_view,
            clipboard: &mut self.clipboard,
            close_window: &mut self.close_window,
            collapsed_commit_folders: &mut self.collapsed_commit_folders,
            collapsed_ref_folders: &mut self.collapsed_ref_folders,
            collapsed_ref_sections: &mut self.collapsed_ref_sections,
            collapsed_status_folders: &mut self.collapsed_status_folders,
            collapsed_status_sections: &mut self.collapsed_status_sections,
            commit_amend: &mut self.commit_amend,
            commit_settings: &self.commit_settings,
            commit_skip_hooks: &mut self.commit_skip_hooks,
            commit_template_applied: &mut self.commit_template_applied,
            conflict_badge: &self.conflict_badge,
            commit_description: &mut self.commit_description,
            commit_detail_status_height: &mut self.commit_detail_status_height,
            commit_file_focus: &mut self.commit_file_focus,
            commit_file_rows: &mut self.commit_file_rows,
            commit_file_state: &mut self.commit_file_state,
            commit_files: &mut self.commit_files,
            commit_files_area: &mut self.commit_files_area,
            commit_files_filter: &mut self.commit_files_filter,
            commit_files_follow: &mut self.commit_files_follow,
            commit_files_len: &mut self.commit_files_len,
            commit_files_visible_rows: &mut self.commit_files_visible_rows,
            commit_filter_focus: &mut self.commit_filter_focus,
            commit_list_mode: &mut self.commit_list_mode,
            commit_message_area: &mut self.commit_message_area,
            commit_message_rows_now: &mut self.commit_message_rows_now,
            commit_message_scroll: &mut self.commit_message_scroll,
            commit_message_total: &mut self.commit_message_total,
            commit_message_user_rows: &mut self.commit_message_user_rows,
            commit_meta_area: &mut self.commit_meta_area,
            commit_paths_cache: &mut self.commit_paths_cache,
            commit_ref_urls: &mut self.commit_ref_urls,
            commit_sort_desc: &mut self.commit_sort_desc,
            commit_summary: &mut self.commit_summary,
            compose_move_requested: &mut self.compose_move_requested,
            compose_stash: &mut self.compose_stash,
            composer_focus: &mut self.composer_focus,
            conflicts: &mut self.conflicts,
            merge: &mut self.merge,
            merge_flow: &mut self.merge_flow,
            operation_state: &self.operation_state,
            index_lock: &self.index_lock,
            cwd: &self.cwd,
            diff_body_area: &mut self.diff_body_area,
            diff_config_path: &self.diff_config_path,
            diff_hunk_starts: &mut self.diff_hunk_starts,
            diff_line_drag: &mut self.diff_line_drag,
            diff_longest_line: &mut self.diff_longest_line,
            diff_render: &mut self.diff_render,
            diff_rendered_len: &mut self.diff_rendered_len,
            diff_rows: &mut self.diff_rows,
            diff_view_state: &mut self.diff_view_state,
            divider1_pct: &mut self.divider1_pct,
            divider2_pct: &mut self.divider2_pct,
            dragging_commit_message: &mut self.dragging_commit_message,
            dragging_divider: &mut self.dragging_divider,
            dragging_list_bar: &mut self.dragging_list_bar,
            dragging_log_boundary: &mut self.dragging_log_boundary,
            log_natural_widths: &self.log_natural_widths,
            dragging_log_header: &mut self.dragging_log_header,
            dragging_scrollbar: &mut self.dragging_scrollbar,
            dragging_sidebar_section: &mut self.dragging_sidebar_section,
            editor: &mut self.editor,
            field_editor: &mut self.field_editor,
            file_insight: &mut self.file_insight,
            history: &mut self.history,
            commit_col: &mut self.commit_col,
            file_insight_area: &mut self.file_insight_area,
            focus: &mut self.focus,
            graph_show_stashes: &mut self.graph_show_stashes,
            help_filter: &mut self.help_filter,
            help_popup_inner_height: &mut self.help_popup_inner_height,
            help_scroll: &mut self.help_scroll,
            hidden_from_graph: &mut self.hidden_from_graph,
            hierarchical_refs: &mut self.hierarchical_refs,
            hit_targets: &mut self.hit_targets,
            hosting: &mut self.hosting,
            hovered_close_tab: &mut self.hovered_close_tab,
            hovered_commit: &mut self.hovered_commit,
            hovered_diff_row: &mut self.hovered_diff_row,
            hovered_ref_folder: &mut self.hovered_ref_folder,
            hovered_ref_row: &mut self.hovered_ref_row,
            hovered_repo_menu_row: &mut self.hovered_repo_menu_row,
            hovered_resize_strip: &mut self.hovered_resize_strip,
            hovered_status_row: &mut self.hovered_status_row,
            hovered_tab_index: &mut self.hovered_tab_index,
            hovered_target: &mut self.hovered_target,
            hovered_wip_row: &mut self.hovered_wip_row,
            input_mode: &mut self.input_mode,
            last_click: &mut self.last_click,
            last_cursor_shape: &mut self.last_cursor_shape,
            last_message: &mut self.last_message,
            lazy_log: &mut self.lazy_log,
            left_auto_collapse: &mut self.left_auto_collapse,
            left_pane_collapsed: &mut self.left_pane_collapsed,
            right_auto_collapse: &mut self.right_auto_collapse,
            lfs: &mut self.lfs,
            link: &self.link,
            link_failure: &mut self.link_failure,
            log_area: &mut self.log_area,
            log_column_hidden: &mut self.log_column_hidden,
            hidden_sidebar_sections: &mut self.hidden_sidebar_sections,
            sidebar_scope: &mut self.sidebar_scope,
            log_column_order: &mut self.log_column_order,
            log_col_pick: &mut self.log_col_pick,
            log_column_overrides: &mut self.log_column_overrides,
            log_column_scroll_widths: &mut self.log_column_scroll_widths,
            log_h_offset: &mut self.log_h_offset,
            log_follow: &mut self.log_follow,
            log_len: &mut self.log_len,
            log_menu_refs: &mut self.log_menu_refs,
            log_multi: &mut self.log_multi,
            file_sel: &mut self.file_sel,
            log_order: &mut self.log_order,
            log_result: &mut self.log_result,
            log_scroll_lines: &self.log_scroll_lines,
            log_search_bar_area: &mut self.log_search_bar_area,
            log_state: &mut self.log_state,
            management: &mut self.management,
            management_panel: &mut self.management_panel,
            nav_badge_targets: &mut self.nav_badge_targets,
            new_file_error: &mut self.new_file_error,
            open_menu: &mut self.open_menu,
            overflow_slots: &mut self.overflow_slots,
            panes_area: &mut self.panes_area,
            patch_seed: &mut self.patch_seed,
            pending_field_click: &mut self.pending_field_click,
            pending_worktree_tab: &mut self.pending_worktree_tab,
            preferences: &mut self.preferences,
            pull_request_seed: &mut self.pull_request_seed,
            quit_how: &mut self.quit_how,
            ref_connector: &mut self.ref_connector,
            ref_drag: &mut self.ref_drag,
            tab_drag: &mut self.tab_drag,
            reveal: &mut self.reveal,
            ref_drop_source: &mut self.ref_drop_source,
            ref_drop_mark: &mut self.ref_drop_mark,
            ref_name_error: &mut self.ref_name_error,
            ref_popup: &mut self.ref_popup,
            ref_popup_rect: &mut self.ref_popup_rect,
            refs_area: &mut self.refs_area,
            refs_filter: &mut self.refs_filter,
            refs_filter_area: &mut self.refs_filter_area,
            refs_result: &mut self.refs_result,
            remote_names: &mut self.remote_names,
            remote_ui: &mut self.remote_ui,
            repo: &self.repo,
            repo_dialogs: &mut self.repo_dialogs,
            repo_menu: &mut self.repo_menu,
            repo_menu_rects: &mut self.repo_menu_rects,
            repo_root: &self.repo_root,
            repo_tabs: &mut self.repo_tabs,
            retry_link_requested: &mut self.retry_link_requested,
            right_pane_collapsed: &mut self.right_pane_collapsed,
            right_panel: &mut self.right_panel,
            session: &self.session,
            should_quit: &mut self.should_quit,
            show_help: &mut self.show_help,
            show_management: &mut self.show_management,
            sidebar_heights: &mut self.sidebar_heights,
            sidebar_layout_avail: &mut self.sidebar_layout_avail,
            sidebar_layout_specs: &mut self.sidebar_layout_specs,
            sidebar_section_areas: &mut self.sidebar_section_areas,
            sidebar_states: &mut self.sidebar_states,
            solo_refs: &mut self.solo_refs,
            stashes_result: &mut self.stashes_result,
            pending_auto_stash: &self.pending_auto_stash,
            status_area: &mut self.status_area,
            status_list_mode: &mut self.status_list_mode,
            status_result: &mut self.status_result,
            status_section_areas: &mut self.status_section_areas,
            status_section_lines: &mut self.status_section_lines,
            status_states: &mut self.status_states,
            submodule_open: &mut self.submodule_open,
            submodules_result: &mut self.submodules_result,
            switch_to: &mut self.switch_to,
            tab_hits: &mut self.tab_hits,
            terminal: &mut self.terminal,
            text_drag: &mut self.text_drag,
            tooltip: &mut self.tooltip,
            ui_viewer: &mut self.ui_viewer,
            ui_graph_theme: &mut self.ui_graph_theme,
            ui_lane_background: &mut self.ui_lane_background,
            ui_tint_strength: &mut self.ui_tint_strength,
            view_all_files: &mut self.view_all_files,
            compare: &self.compare,
            compare_swapped: &mut self.compare_swapped,
            visible_sidebar: &self.visible_sidebar,
            wip_all: &mut self.wip_all,
            worktree_seed: &mut self.worktree_seed,
            worktrees_result: &mut self.worktrees_result,
        }
    }

    /// The tab-level actions' borrow of this window (`tabs::TabCtx`): the
    /// management tab, the tab menu and the palette all build one.
    pub(in crate::event_loop) fn tab_ctx(&mut self) -> TabCtx<'_> {
        TabCtx {
            session: &self.session,
            link: &self.link,
            current: &self.repo,
            cwd: &self.cwd,
            dialogs: &mut self.repo_dialogs,
            tabs: &mut self.repo_tabs,
            active_tab: &mut self.active_tab,
            switch_to: &mut self.switch_to,
            close_window: &mut self.close_window,
            show_management: &mut self.show_management,
            management: &mut self.management,
            repo_menu: &mut self.repo_menu,
            input_mode: &mut self.input_mode,
            clipboard: &mut self.clipboard,
            pending_worktree_tab: &mut self.pending_worktree_tab,
            remote_ui: &mut self.remote_ui,
        }
    }

    /// Keeps the field editor on the focused main-window text field and starts a
    /// caret placement the last mouse press asked for.
    pub(in crate::event_loop) fn sync_fields(&mut self) {
        if let Some(manager) = self.management.as_mut() {
            manager.sync_search();
        }
        self.field_editor.sync(MainField::active(
            &self.input_mode,
            self.composer_focus,
            self.commit_filter_focus,
        ));
        if let Some((id, col, row, at)) = self.pending_field_click.take() {
            if self.field_editor.caret(id).is_some() {
                if let Some(text) = id.text(
                    &mut self.input_mode,
                    &mut self.commit_summary,
                    &mut self.commit_description,
                    &mut self.commit_files_filter,
                ) {
                    self.field_editor.mouse_down(id, text, col, row, at);
                }
            }
        }
    }

    /// Leaves the alternate screen and says why the window ended.
    pub(in crate::event_loop) fn finish(&mut self) -> io::Result<NextWindow> {
        let next = match (self.switch_to.take(), self.show_management.take()) {
            (Some(next), _) => NextWindow::Repository(next),
            (None, Some(entry)) => NextWindow::Management(entry),
            (None, None) if self.close_window => NextWindow::None,
            (None, None) => NextWindow::Quit,
        };
        // The process is about to end -- say why, before
        // anything below can fail and turn a quit into an error.
        if matches!(next, NextWindow::Quit) {
            if self.quit_how == Some(crate::tab_dir::QUIT_CD_HOW) {
                crate::tab_dir::request_quit_cd();
            }
            let context = format!(
                "{:?} in the center, {:?} on the right, focus {:?}{}",
                self.central_view,
                self.right_panel,
                self.focus,
                if self.view_all_files {
                    ", View all files on"
                } else {
                    ""
                }
            );
            exit_log::record_exit(
                &exit_log::ExitReason::UserQuit {
                    how: self
                        .quit_how
                        .unwrap_or("no quit gesture recorded")
                        .to_string(),
                    context,
                },
                &self.session,
                &exit_log::recent_inputs(),
            );
        }
        if self.last_cursor_shape != CursorShape::Default {
            cursor::emit_cursor_shape(self.terminal.backend_mut(), CursorShape::Default)
                .map_err(exit_log::io_context("resetting the cursor shape"))?;
        }
        // Every Kitty picture still stored in the terminal: leaving the
        // alternate screen removes the placeholder cells, not the pixels.
        terminal_image::release_all(self.terminal.backend_mut())
            .map_err(exit_log::io_context("deleting the pictures"))?;
        crate::kitty_keyboard::disable(self.terminal.backend_mut());
        crate::cursor::reset_pointer(self.terminal.backend_mut());
        crate::text_cursor::restore_style(self.terminal.backend_mut());
        crate::term_modes::disable_bracketed_paste(self.terminal.backend_mut());
        disable_raw_mode().map_err(exit_log::io_context("leaving raw mode"))?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            crossterm::event::DisableFocusChange
        )
        .map_err(exit_log::io_context("leaving the alternate screen"))?;
        Ok(next)
    }
}
