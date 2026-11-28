//! Normal-mode keys, second third: the ref popup, the Log, context menus
//! opened from the keyboard, and the Status list.

use super::commands::{CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use crate::keymap::term::TermKeymap;
use gitpoulpi_i18n::msg;

/// `Flow::Handled` when an arm matched, `Flow::Continue` to try the next part.
pub(super) fn handle(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    let km = env.km;
    let unpushed_tags = env.unpushed_tags;
    match key.code {
        // The ref-reveal popup's own keyboard handling
        // takes priority over everything below it while
        // it's open -- Esc/Up-k/Down-j/Enter all target
        // the popup instead of whatever pane has focus
        // underneath, the same "modal-ish while open"
        // convention the column show/hide menu already
        // uses.
        KeyCode::Esc if cx.ref_popup.is_some() => *cx.ref_popup = None,
        KeyCode::Up | KeyCode::Char('k') if cx.ref_popup.is_some() => {
            if let Some(state) = &mut *cx.ref_popup {
                state.selected = state.selected.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') if cx.ref_popup.is_some() => {
            if let Some(state) = &mut *cx.ref_popup {
                let count = cx
                    .refs_result
                    .as_ref()
                    .map(|refs| popup_pills(state.commit_id, refs).len())
                    .unwrap_or(0);
                if count > 0 {
                    state.selected = (state.selected + 1).min(count - 1);
                }
            }
        }
        KeyCode::Enter if cx.ref_popup.is_some() => {
            if let Some(state) = cx.ref_popup.take() {
                let pills = cx
                    .refs_result
                    .as_ref()
                    .map(|refs| popup_pills(state.commit_id, refs))
                    .unwrap_or_default();
                if let Some(pill) = pills.get(state.selected) {
                    match pill.kind {
                        RefRowKind::Local | RefRowKind::Tag => {
                            cx.link
                                .submit_action(Action::CheckoutBranch(pill.name.clone()));
                            *cx.last_message = Some(msg::input::checking_out(&pill.name));
                        }
                        RefRowKind::Remote => {
                            let msg = checkout_remote_pill(
                                &pill.name,
                                Some(state.commit_id),
                                &*cx.refs_result,
                                &*cx.remote_names,
                                cx.link,
                                &mut *cx.input_mode,
                            );
                            if !msg.is_empty() {
                                *cx.last_message = Some(msg);
                            }
                        }
                    }
                }
            }
        }
        // Opens the same popup the mouse-hover path opens
        // ("au survol on affiche les autres [...] double
        // clicable" plus the later keyboard-parity ask) --
        // reuses `RefPopupState`, doesn't build a parallel
        // mechanism. Only meaningful when the selected
        // commit actually has more than one ref to show.
        // Multi-select in the commit tree, keyboard half of
        // Ctrl/Shift+click: Space toggles the current commit,
        // Shift+Up/Down extends a range from the anchor.
        KeyCode::Char(' ') if *cx.focus == Focus::Log && !*cx.commit_file_focus => {
            let display = display_commits(
                &*cx.log_result,
                &*cx.stashes_result,
                &*cx.log_order,
                *cx.log_len,
            );
            let cursor_key = cx
                .log_state
                .selected()
                .and_then(|c| display.get(c).copied().flatten());
            if let Some(key) = cursor_key {
                cx.log_multi.toggle(key, cursor_key);
                *cx.last_message = Some(msg::compare::rows_selected(cx.log_multi.len() as u64));
            }
        }
        KeyCode::Up | KeyCode::Down
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                // Alt+arrow is the twin of Alt+click.
                && key
                    .modifiers
                    .intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) =>
        {
            let display = display_commits(
                &*cx.log_result,
                &*cx.stashes_result,
                &*cx.log_order,
                *cx.log_len,
            );
            let before = cx.log_state.selected();
            let delta = if key.code == KeyCode::Up { -1 } else { 1 };
            move_table_selection(&mut *cx.log_state, *cx.log_len, delta);
            *cx.log_follow = true;
            *cx.right_panel = RightPanel::CommitDetail;
            if let Some(target) = cx.log_state.selected() {
                cx.log_multi.extend_to(&display, target, before);
                *cx.last_message = Some(msg::compare::rows_selected(cx.log_multi.len() as u64));
            }
        }
        KeyCode::Esc if *cx.focus == Focus::Log && !cx.log_multi.is_empty() => {
            cx.log_multi.clear();
            *cx.last_message = Some(msg::input::SELECTION_CLEARED.get().into());
        }
        // With nothing nearer to drop (the selection above, a menu, a field, the
        // file focus), Esc in the Log drops the author filter: the same path as
        // `Log: Clear Author Filter`.
        KeyCode::Esc if esc_clears_author_filter(cx) => {
            commands::clear_author_filter(cx);
        }
        // The columns scroll sideways (`scroll` layout). Before the bare `Right`
        // arm below, which ignores modifiers and would open the ref popup on
        // `Shift+Right`; `LogFiles` keeps its own `Shift+Left/Right`.
        _ if *cx.focus == Focus::Log
            && !*cx.commit_file_focus
            && (km.is(Bind::LogScrollLeft, &key) || km.is(Bind::LogScrollRight, &key)) =>
        {
            commands::log_columns::run(
                cx,
                if km.is(Bind::LogScrollLeft, &key) {
                    crate::log_table::LogColumnCmd::ScrollLeft
                } else {
                    crate::log_table::LogColumnCmd::ScrollRight
                },
            );
        }
        KeyCode::Right | KeyCode::Char(' ')
            if *cx.focus == Focus::Log && !*cx.commit_file_focus =>
        {
            if let Some(commit_id) =
                selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order)
            {
                let count = cx
                    .refs_result
                    .as_ref()
                    .map(|refs| ref_pills_for_commit(commit_id, refs).len())
                    .unwrap_or(0);
                if count > 1 {
                    // Through `log_index_to_row` (not a hand-rolled `top_y + i - offset`) so
                    // it stays in step with the Log's one row arithmetic: `log_top_y` is the
                    // border + header row, without which the popup's render position
                    // (`anchor_row + 1`) lands ON the triggering row instead of below it.
                    let top_y = log_top_y(*cx.log_area);
                    let anchor_row = cx
                        .log_state
                        .selected()
                        .and_then(|i| log_index_to_row(top_y, cx.log_state.offset(), i))
                        .unwrap_or(top_y);
                    // The Ref column's own left edge, the same anchor the mouse path uses, so
                    // the popup lines up with the pill it expands (a hard-coded
                    // `log_area.x + 2` only coincides when Ref starts there).
                    let anchor_col = cx
                        .log_geo()
                        .offsets
                        .iter()
                        .find(|(c, _, _)| *c == LogColumn::Ref)
                        .map(|(_, x, _)| *x)
                        .unwrap_or(cx.log_area.x);
                    *cx.ref_popup = Some(RefPopupState {
                        commit_id,
                        selected: 0,
                        opened_via_keyboard: true,
                        anchor_col,
                        anchor_row,
                    });
                } else {
                    *cx.last_message = Some(msg::input::NO_OTHER_REFS.get().into());
                }
            }
        }
        // --- third pane: keyboard equivalents of every
        // control the mouse can reach there (principle 5).
        // Placed before the generic keys below so they win
        // while the Status pane has focus.
        // The keyboard twin of right-clicking a commit,
        // anchored on the selected row exactly the way the
        // third pane's own `.` is. `log_index_to_row` --
        // never `log_area.y + n` -- because anchoring by
        // hand is how the menu ends up floating over the
        // wrong commit.
        // On a stash row it is the stash's menu instead --
        // the STASHES section's own Pop / Apply / Drop.
        // `Shift+F10`/the Menu key are standard aliases of
        // `.` here too -- see `is_menu_key`.
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Log
            && !*cx.commit_file_focus
            && selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
                .is_some() =>
        {
            let Some(stash) =
                selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
            else {
                return Ok(Flow::Handled);
            };
            let row = cx
                .log_state
                .selected()
                .and_then(|i| log_index_to_row(log_top_y(*cx.log_area), cx.log_state.offset(), i))
                .unwrap_or(log_top_y(*cx.log_area));
            *cx.open_menu = OpenMenu::Ref(stash_context_menu(&stash, cx.log_area.x + 2, row));
        }
        // The keyboard twin of right-clicking the WIP row:
        // same menu key, anchored on row 0 (the WIP row is
        // always the top of the Log while it exists), the
        // WIP row's own menu instead of a commit's.
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Log
            && !*cx.commit_file_focus
            && selected_log_is_wip(&*cx.log_state, &*cx.log_order) =>
        {
            let row = cx
                .log_state
                .selected()
                .and_then(|i| log_index_to_row(log_top_y(*cx.log_area), cx.log_state.offset(), i))
                .unwrap_or(log_top_y(*cx.log_area));
            *cx.open_menu = OpenMenu::Wip(ContextMenu::new(
                msg::input::WIP_MENU_TITLE.get(),
                wip_menu_entries(has_wip_row(&*cx.status_result), cx.ai_ui.enabled()),
                cx.log_area.x + 2,
                row,
            ));
        }
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Log
            && !*cx.commit_file_focus =>
        {
            match selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order) {
                Some(commit) => {
                    let row = cx
                        .log_state
                        .selected()
                        .and_then(|i| {
                            log_index_to_row(log_top_y(*cx.log_area), cx.log_state.offset(), i)
                        })
                        .unwrap_or(log_top_y(*cx.log_area));
                    let built = commit_menu(
                        &MenuSources {
                            refs_result: &*cx.refs_result,
                            log_result: &*cx.log_result,
                            remotes: &cx.remote_ui.remotes,
                            hidden: &*cx.hidden_from_graph,
                            solo: &*cx.solo_refs,
                            unpushed_tags,
                            selected_count: 0,
                            ai_enabled: cx.ai_ui.enabled(),
                            merge_ff: cx.commit_settings.merge_ff,
                        },
                        commit,
                    );
                    *cx.log_menu_refs = built.table;
                    *cx.open_menu = OpenMenu::Log((
                        commit,
                        ContextMenu::new(
                            &commit.to_hex()[..SHORT_SHA_LEN],
                            built.entries,
                            cx.log_area.x + 2,
                            row,
                        ),
                    ));
                }
                None => *cx.last_message = Some(msg::input::SELECT_COMMIT_NOT_WIP.get().into()),
            }
        }
        KeyCode::Esc if cx.open_menu.esc_closes_here() => cx.open_menu.close(),
        KeyCode::Esc if *cx.commit_filter_focus => *cx.commit_filter_focus = false,
        // Typing into the commit file filter goes through
        // the shared editing arm near the top of this
        // match, with the composer's: a field with the
        // keyboard wins over every pane shortcut.
        KeyCode::Enter if *cx.commit_filter_focus => *cx.commit_filter_focus = false,
        // `c` focuses the composer, `m` toggles Path/Tree,
        // `A` toggles amend, `z` collapses/expands all.
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusCompose, &key) => {
            *cx.composer_focus = Some(ComposerField::Summary);
        }
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusToggleFileMode, &key) => {
            if *cx.right_panel == RightPanel::CommitDetail {
                *cx.commit_list_mode = cx.commit_list_mode.toggled();
            } else {
                *cx.status_list_mode = cx.status_list_mode.toggled();
            }
        }
        // `m` / `a` also work with the Log focused (whatever
        // the right pane lists: the selected commit's files,
        // or the WIP row's) -- the Log and the commit-detail
        // pane are one "commit view" to the keyboard, and
        // neither key means anything else there.
        _ if *cx.focus == Focus::Log && km.is(Bind::LogToggleFileMode, &key) => {
            commands::toggle_file_list_mode(cx);
        }
        // `a`: View all files, from either file list (the
        // Status pane / WIP row, or a commit's changed
        // files), like the toolbar checkbox.
        // `v` compares the Log's selected rows, `x` swaps its two ends.
        _ if *cx.focus == Focus::Log && km.is(Bind::LogCompareSelection, &key) => {
            commands::compare_selection(cx);
        }
        // `Shift+Y` (yank) opens Copy as Mermaid gitGraph on the selection.
        _ if *cx.focus == Focus::Log && km.is(Bind::LogCopyMermaid, &key) => {
            if selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order).is_some() {
                commands::run_command(cx, PaletteCommand::CopySelectedAsMermaid);
            } else {
                *cx.last_message = Some(msg::input::SELECT_LOG_COMMIT_NOT_WIP.get().into());
            }
        }
        _ if *cx.focus == Focus::Log && km.is(Bind::LogFilterAuthor, &key) => {
            commands::run_command(cx, PaletteCommand::FilterLogByAuthor);
        }
        _ if *cx.focus == Focus::Log && km.is(Bind::LogPickColumn, &key) => {
            commands::log_columns::run(cx, crate::log_table::LogColumnCmd::Pick);
        }
        _ if *cx.focus == Focus::Log && km.is(Bind::LogCompareSwap, &key) => {
            commands::compare_swap(cx);
        }
        _ if *cx.focus == Focus::Log && km.is(Bind::LogCompareNext, &key) => {
            commands::compare_goto_next(cx);
        }
        _ if (*cx.focus == Focus::Status && km.is(Bind::StatusViewAllFiles, &key))
            || (*cx.focus == Focus::Log && km.is(Bind::LogViewAllFiles, &key)) =>
        {
            commands::toggle_view_all_files(cx);
        }
        _ if (*cx.focus == Focus::Status && km.is(Bind::StatusLfsOnly, &key))
            || (*cx.focus == Focus::Log && km.is(Bind::LogLfsOnly, &key)) =>
        {
            commands::toggle_lfs_only(cx);
        }
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusAmend, &key) => {
            *cx.commit_amend = !*cx.commit_amend;
        }
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusSkipHooks, &key) => {
            *cx.commit_skip_hooks = !*cx.commit_skip_hooks;
        }
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusCollapseAll, &key) => {
            if cx.collapsed_status_folders.is_empty() {
                *cx.collapsed_status_folders = all_status_fold_keys(&*cx.status_section_lines);
            } else {
                cx.collapsed_status_folders.clear();
            }
        }
        _ if *cx.focus == Focus::Log
            && *cx.commit_file_focus
            && km.is(Bind::LogFilesCollapseAll, &key) =>
        {
            if cx.collapsed_commit_folders.is_empty() {
                *cx.collapsed_commit_folders = all_folders(&*cx.commit_paths_cache);
            } else {
                cx.collapsed_commit_folders.clear();
            }
        }
        // `/` filters the commit's changed files, matching
        // the Refs sidebar's own filter key.
        _ if *cx.focus == Focus::Log
            && *cx.commit_file_focus
            && km.is(Bind::LogFilesFilter, &key) =>
        {
            *cx.commit_filter_focus = true;
        }
        // The pull-request / issue lists: Enter opens the row
        // (the double-click), `.` its menu (the right-click),
        // `v` cycles the header chip's filter.
        KeyCode::Enter
            if *cx.focus == Focus::Refs
                && HostingList::from_section(*cx.active_ref_section).is_some()
                && !cx
                    .collapsed_ref_sections
                    .contains(cx.active_ref_section.title())
                && cx
                    .sidebar_states
                    .state(*cx.active_ref_section)
                    .selected()
                    .is_some() =>
        {
            if let (Some(list), Some(index)) = (
                HostingList::from_section(*cx.active_ref_section),
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
            ) {
                let effect = cx.hosting.activate_row(list, index, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
        }
        _ if km.is(Bind::RefsHostingFilter, &key)
            && *cx.focus == Focus::Refs
            && HostingList::from_section(*cx.active_ref_section).is_some() =>
        {
            if let Some(list) = HostingList::from_section(*cx.active_ref_section) {
                let command = match list {
                    HostingList::Pulls => HostingCommand::ChoosePullRequestFilter,
                    HostingList::Issues => HostingCommand::ChooseIssueFilter,
                };
                let anchor = (cx.refs_area.x + 2, cx.refs_area.y + 2);
                let effect = cx.hosting.run_command(command, anchor, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
        }
        // `e` on a pull request row: Edit on GitHub / GitLab
        // (the row pencil's keyboard twin).
        _ if km.is(Bind::RefsHostingEdit, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::PullRequests
            && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let effect = match cx
                .sidebar_states
                .state(SidebarSection::PullRequests)
                .selected()
            {
                Some(sel) => cx.hosting.edit_row(sel, &mut *cx.clipboard),
                None => HostingEffect::Message(msg::input::SELECT_ROW_FIRST.get().into()),
            };
            commands::apply_hosting_effect(cx, effect);
        }
        // `n` on the PULL REQUESTS section: the header's `+`.
        _ if km.is(Bind::RefsNewPullRequest, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::PullRequests
            && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let effect = cx.hosting.run_command(
                HostingCommand::CreatePullRequest,
                (0, 0),
                &mut *cx.clipboard,
            );
            commands::apply_hosting_effect(cx, effect);
        }
        // `n` on the ISSUES section: the header's `+` (New issue).
        _ if km.is(Bind::RefsNewPullRequest, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Issues
            && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let effect =
                cx.hosting
                    .run_command(HostingCommand::CreateIssue, (0, 0), &mut *cx.clipboard);
            commands::apply_hosting_effect(cx, effect);
        }
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Refs
            && HostingList::from_section(*cx.active_ref_section).is_some() =>
        {
            let Some(list) = HostingList::from_section(*cx.active_ref_section) else {
                return Ok(Flow::Handled);
            };
            let area = cx
                .sidebar_section_areas
                .iter()
                .find(|(s, _)| *s == *cx.active_ref_section)
                .map(|(_, a)| *a)
                .unwrap_or_default();
            let state = cx.sidebar_states.state(*cx.active_ref_section);
            match state.selected() {
                Some(sel) => {
                    let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
                    if !cx.hosting.open_row_menu(list, sel, area.x + 2, row) {
                        *cx.last_message = Some(msg::input::NOTHING_FOR_MENU.get().into());
                    }
                }
                None => *cx.last_message = Some(msg::input::SELECT_ROW_FIRST.get().into()),
            }
        }
        // The keyboard twin of right-clicking a ref row,
        // anchored on the selected row exactly the way the
        // Log's and the third pane's `.` are. Stashes are
        // deliberately not reachable here yet: their row
        // identity is a numeric id rather than a name, and
        // `selected_ref_leaf` only resolves ref sections.
        // A submodule's menu (Update / Init / Open), anchored
        // on its row: the keyboard twin of its right-click.
        #[cfg(feature = "worktree")]
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Worktrees =>
        {
            commands::worktrees::open_row_menu(cx);
        }
        // `+` in WORKTREES: the header `+`'s keyboard twin, the create menu.
        #[cfg(feature = "worktree")]
        _ if km.is(Bind::RefsAddRemote, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Worktrees =>
        {
            commands::run_command(cx, PaletteCommand::CreateWorktreeMenu);
        }
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Submodules =>
        {
            let state = cx.sidebar_states.state(SidebarSection::Submodules);
            match state.selected().and_then(|i| cx.submodules_result.get(i)) {
                Some(sub) => {
                    let area = cx
                        .sidebar_section_areas
                        .iter()
                        .find(|(s, _)| *s == SidebarSection::Submodules)
                        .map(|(_, a)| *a)
                        .unwrap_or_default();
                    let sel = state.selected().unwrap_or(0);
                    let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
                    *cx.open_menu = OpenMenu::Ref((
                        SidebarSection::Submodules,
                        sub.path.clone(),
                        ContextMenu::new(
                            sub.path.clone(),
                            ref_menu_entries(
                                SidebarSection::Submodules,
                                &sub.path,
                                &MenuContext::default(),
                                false,
                                false,
                                false,
                            ),
                            area.x + 2,
                            row,
                        ),
                    ));
                }
                None => *cx.last_message = Some(msg::input::SELECT_SUBMODULE_FIRST.get().into()),
            }
        }
        _ if km.is(Bind::GlobalContextMenu, &key) && *cx.focus == Focus::Refs => {
            match selected_ref_leaf(
                &*cx.refs_result,
                &*cx.refs_filter,
                *cx.active_ref_section,
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
            ) {
                Some(full_name) => {
                    let area = cx
                        .sidebar_section_areas
                        .iter()
                        .find(|(s, _)| *s == *cx.active_ref_section)
                        .map(|(_, a)| *a)
                        .unwrap_or_default();
                    let state = cx.sidebar_states.state(*cx.active_ref_section);
                    let sel = state.selected().unwrap_or(0);
                    // +1 for the sub-pane's own top border.
                    let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
                    *cx.open_menu = OpenMenu::Ref((
                        *cx.active_ref_section,
                        full_name.clone(),
                        ContextMenu::new(
                            full_name.clone(),
                            ref_menu_entries(
                                *cx.active_ref_section,
                                &full_name,
                                &menu_context_for_ref(
                                    &*cx.refs_result,
                                    &*cx.log_result,
                                    *cx.active_ref_section,
                                    &full_name,
                                    &cx.remote_ui.remotes,
                                    cx.commit_settings.merge_ff,
                                ),
                                refs_tree::hidden_in_graph(
                                    &*cx.hidden_from_graph,
                                    &full_name,
                                    *cx.active_ref_section == SidebarSection::Remote,
                                ),
                                cx.solo_refs.contains(full_name.as_str()),
                                unpushed_tags.contains(full_name.as_str()),
                            ),
                            area.x + 2,
                            row,
                        ),
                    ));
                }
                // Not a ref: a folder row. A remote's own
                // root row opens the remote's menu, exactly
                // as right-clicking it does; any other
                // folder (a `/`-derived one, in LOCAL or
                // nested under a remote in REMOTE) opens
                // its own hide-all/solo-all menu instead.
                None => match selected_ref_folder(
                    &*cx.refs_result,
                    &*cx.refs_filter,
                    *cx.active_ref_section,
                    cx.sidebar_states.state(*cx.active_ref_section).selected(),
                    *cx.hierarchical_refs,
                    &*cx.collapsed_ref_folders,
                    &*cx.remote_names,
                ) {
                    Some((path, true)) => {
                        let area = cx
                            .sidebar_section_areas
                            .iter()
                            .find(|(s, _)| *s == *cx.active_ref_section)
                            .map(|(_, a)| *a)
                            .unwrap_or_default();
                        let state = cx.sidebar_states.state(*cx.active_ref_section);
                        let sel = state.selected().unwrap_or(0);
                        let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
                        let hidden = crate::refs_tree::remote_hidden(&*cx.hidden_from_graph, &path);
                        cx.remote_ui.open_remote_menu(
                            &path,
                            hidden,
                            cx.refs_result.as_deref().unwrap_or(&[]),
                            area.x + 2,
                            row,
                        );
                    }
                    Some((path, false)) => {
                        if let (Ok(refs), Some(kind)) =
                            (cx.refs_result.as_ref(), cx.active_ref_section.ref_kind())
                        {
                            let names = group_refs(refs, "").names(kind).to_vec();
                            let under = refs_under_folder(&names, &path);
                            let area = cx
                                .sidebar_section_areas
                                .iter()
                                .find(|(s, _)| *s == *cx.active_ref_section)
                                .map(|(_, a)| *a)
                                .unwrap_or_default();
                            let state = cx.sidebar_states.state(*cx.active_ref_section);
                            let sel = state.selected().unwrap_or(0);
                            let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
                            *cx.open_menu = OpenMenu::Ref((
                                *cx.active_ref_section,
                                path.clone(),
                                ContextMenu::new(
                                    path.clone(),
                                    folder_menu_entries(
                                        folder_fully_in(&*cx.hidden_from_graph, &under)
                                            || (*cx.active_ref_section == SidebarSection::Remote
                                                && refs_tree::hidden_through_remote(
                                                    &*cx.hidden_from_graph,
                                                    &path,
                                                )),
                                        folder_fully_in(&*cx.solo_refs, &under),
                                    ),
                                    area.x + 2,
                                    row,
                                ),
                            ));
                        }
                    }
                    None => {
                        *cx.last_message = Some(msg::input::SELECT_BRANCH_TAG_REMOTE.get().into())
                    }
                },
            }
        }
        // `+` in the REMOTE section: Add remote -- the
        // keyboard twin of the section header's `+`.
        _ if km.is(Bind::RefsAddRemote, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Remote =>
        {
            cx.remote_ui.form = Some(RemoteForm::add(cx.remote_ui.names()));
        }
        // `u` on a local branch: its remotes (upstream and extra push
        // targets), the keyboard twin of clicking the remotes badge.
        _ if km.is(Bind::RefsRemotes, &key)
            && *cx.focus == Focus::Refs
            && *cx.active_ref_section == SidebarSection::Local =>
        {
            let leaf = selected_ref_leaf(
                &*cx.refs_result,
                &*cx.refs_filter,
                *cx.active_ref_section,
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
            );
            *cx.last_message = match leaf {
                Some(branch) => crate::multi_remote_badge::open_branch_remotes(
                    &*cx.remote_ui,
                    cx.refs_result.as_deref().unwrap_or(&[]),
                    &branch,
                    &cx.link.activity_handle().raw.requester(),
                    &mut *cx.input_mode,
                ),
                None => Some(msg::input::SELECT_BRANCH_TAG_REMOTE.get().into()),
            };
        }
        // Opens the selected file's (or, in the Unstaged
        // tree, folder's) context menu
        // from the keyboard, anchored on its own row.
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Status
            && super::commands::open_menu_from_keyboard(cx) => {}
        _ if km.is(Bind::GlobalContextMenu, &key) && *cx.focus == Focus::Status => {
            super::commands::open_status_row_menu(cx);
        }
        // Stage/unstage the selected row without reaching
        // for its hover button.
        // With several files selected in this list: all of them.
        _ if *cx.focus == Focus::Status
            && km.is(Bind::StatusStageToggle, &key)
            && stage_selection(cx) => {}
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusStageToggle, &key) => {
            let active = *cx.active_status_section;
            let selected = cx.status_states.state(active).selected();
            let line = selected
                .and_then(|i| section_lines(&*cx.status_section_lines, active).get(i))
                .cloned();
            match line {
                Some(StatusPaneLine::File { section, path, .. }) => {
                    cx.link.submit_action(match section {
                        StatusSection::Unstaged => Action::StageFile(path),
                        StatusSection::Staged => Action::UnstageFile(path),
                    });
                }
                Some(StatusPaneLine::Folder { fold_key, .. }) => {
                    toggle_in(&mut *cx.collapsed_status_folders, fold_key);
                }
                None => {}
            }
        }
        // Open the selected file's diff in the central
        // column -- the keyboard half of double-clicking
        // the row, and the only way to reach hunk- and
        // line-level staging, which needs a working-tree
        // diff rather than a commit's.
        _ if *cx.focus == Focus::Status && km.is(Bind::StatusOpenDiff, &key) => {
            let active = *cx.active_status_section;
            let selected = cx.status_states.state(active).selected();
            match selected.and_then(|i| section_lines(&*cx.status_section_lines, active).get(i)) {
                Some(StatusPaneLine::File { section, path, .. }) => {
                    *cx.central_view = CentralView::Diff;
                    *cx.diff_view_state = Some(working_diff_state(*section, path.clone()));
                    *cx.ref_popup = None;
                    cx.open_menu.close_column_menu();
                }
                _ => {
                    *cx.last_message = Some(msg::input::SELECT_FILE_STATUS.get().into());
                }
            }
        }
        // Collapse/expand the active sub-pane, the keyboard
        // equivalent of clicking its header.
        // The WIP View all files list: Enter opens the selected row, as a click
        // does.
        KeyCode::Enter if *cx.focus == Focus::Status && cx.wip_all.shown() => {
            if let Some(i) = cx.wip_all.state.selected() {
                activate_wip_row(
                    &mut *cx.wip_all,
                    i,
                    &*cx.status_result,
                    head_oid(cx.refs_result.as_deref().unwrap_or(&[])),
                    cx.link,
                    &mut *cx.file_insight,
                    &mut *cx.diff_view_state,
                    &mut *cx.central_view,
                    &mut *cx.collapsed_status_folders,
                );
            } else {
                cx.wip_all.move_by(0);
            }
        }
        // Enter on a conflicted row opens the merge editor on it (the
        // keyboard twin of double-clicking it); any other row keeps the
        // section toggle.
        KeyCode::Enter
            if *cx.focus == Focus::Status
                && *cx.active_status_section == StatusSection::Unstaged
                && selected_conflicted_path(
                    &*cx.status_result,
                    &*cx.status_states,
                    *cx.status_list_mode,
                    &*cx.collapsed_status_folders,
                    &*cx.conflicts,
                )
                .is_some() =>
        {
            commands::open_selected(cx);
        }
        KeyCode::Enter if *cx.focus == Focus::Status => {
            toggle_in(
                &mut *cx.collapsed_status_sections,
                *cx.active_status_section,
            );
        }
        // Move between the two sub-panes, the same way
        // h/l moves between the sidebar's sections.
        KeyCode::Left | KeyCode::Char('h') if *cx.focus == Focus::Status => {
            *cx.active_status_section = StatusSection::Unstaged;
        }
        KeyCode::Right | KeyCode::Char('l') if *cx.focus == Focus::Status => {
            *cx.active_status_section = StatusSection::Staged;
        }
        _ => return Ok(Flow::Continue),
    }
    Ok(Flow::Handled)
}

/// Ctrl+Space on a multi-selection of Unstaged (Staged): stages (unstages)
/// every selected file. `false` when the active list holds no selection.
fn stage_selection(cx: &mut CommandCtx<'_>) -> bool {
    use crate::file_selection::FileList;
    let list = match *cx.active_status_section {
        StatusSection::Unstaged => FileList::Unstaged,
        StatusSection::Staged => FileList::Staged,
    };
    if cx.file_sel.list() != Some(list) || cx.file_sel.is_empty() {
        return false;
    }
    let paths = cx
        .file_sel
        .files_in_order(&super::commands::list_rows(cx, list));
    if paths.is_empty() {
        return false;
    }
    for path in paths {
        cx.link.submit_action(match list {
            FileList::Unstaged => Action::StageFile(path),
            _ => Action::UnstageFile(path),
        });
    }
    cx.file_sel.clear();
    true
}

/// Whether a bare `Esc` in the Log drops the author filter: the Log has the
/// keyboard, a filter is on, and no field, menu, popup or file focus owns the key.
fn esc_clears_author_filter(cx: &CommandCtx<'_>) -> bool {
    esc_drops_author_filter(
        *cx.focus == Focus::Log,
        !cx.log_author_filter.is_empty(),
        *cx.commit_file_focus
            || *cx.commit_filter_focus
            || cx.composer_focus.is_some()
            || cx.ref_popup.is_some()
            || cx.open_menu.is_open(),
    )
}

/// The pure rule behind [`esc_clears_author_filter`]; `owned` is "something
/// nearer already owns Esc". (The Log's own selection clears first, by its arm
/// sitting above this one; a pending chord, the search bar and a running
/// sequence are other input modes or later arms.)
pub(super) fn esc_drops_author_filter(in_log: bool, filter_on: bool, owned: bool) -> bool {
    in_log && filter_on && !owned
}

#[cfg(test)]
mod esc_tests {
    use super::esc_drops_author_filter;

    #[test]
    fn esc_drops_the_filter_only_in_the_log_with_nothing_nearer() {
        assert!(esc_drops_author_filter(true, true, false));
        assert!(!esc_drops_author_filter(false, true, false), "not the Log");
        assert!(!esc_drops_author_filter(true, false, false), "no filter");
        assert!(!esc_drops_author_filter(true, true, true), "a nearer owner");
    }
}
