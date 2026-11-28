//! Normal-mode keys, last third: quit, help, focus cycling, refresh, the
//! Refs sidebar, and list movement. Anything unmatched is simply ignored.

use super::commands::{CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use crate::keymap::term::TermKeymap;
use gitpoulpi_i18n::msg;

/// Handles the keys no earlier part claimed.
pub(super) fn handle(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    let km = env.km;
    let count = env.count;
    // A key the user gave a sequence (`[keybindings.sequence]`): the same
    // availability check as its palette row, then the run.
    if let Some(keymap::Action::Sequence(slot)) = km.action_in(keymap::Ctx::Sequence, &key) {
        let command = PaletteCommand::Sequence(palette::SeqOp::Run(slot));
        match palette::unavailable_reason(command, env.palette_ctx) {
            Some(reason) => {
                let name =
                    crate::sequence_run::registry::info(slot).map_or_else(String::new, |i| i.name);
                *cx.last_message = Some(msg::input::command_unavailable(&name, reason));
            }
            None => crate::sequence_run::start(slot, cx.repo_root, cx.last_message),
        }
        return Ok(Flow::Handled);
    }
    // The runner's own two keys (`sequence.cancel`, `sequence.continue`).
    if let Some(action @ (keymap::Action::SequenceCancel | keymap::Action::SequenceContinue)) =
        km.action_in(keymap::Ctx::Sequence, &key)
    {
        let command = if action == keymap::Action::SequenceCancel {
            PaletteCommand::SequenceCancel
        } else {
            PaletteCommand::SequenceContinue
        };
        match palette::unavailable_reason(command, env.palette_ctx) {
            Some(reason) => *cx.last_message = Some(reason.to_string()),
            None => {
                commands::run_command(cx, command);
            }
        }
        return Ok(Flow::Handled);
    }
    // A key the user gave a palette command (`[keybindings.palette]`): the
    // same availability check and the same `run_command` as the palette.
    if let Some(entry) = km
        .action_in(keymap::Ctx::Palette, &key)
        .and_then(palette::palette_entry)
    {
        let name = entry.label.get();
        match palette::unavailable_reason(entry.command, env.palette_ctx) {
            Some(reason) => {
                *cx.last_message = Some(msg::input::command_unavailable(name, reason));
            }
            None => {
                *cx.last_message = Some(msg::input::ran(name));
                commands::run_command(cx, entry.command);
            }
        }
        return Ok(Flow::Handled);
    }
    match key.code {
        // `q` quits only from the graph: over a diff or the File View an earlier
        // arm takes it and closes that view.
        _ if km.is(Bind::GlobalQuit, &key) => {
            *cx.should_quit = true;
            *cx.quit_how = Some("q key");
        }
        _ if km.is(Bind::GlobalHelp, &key) => *cx.show_help = true,
        // Bare `1`/`2`/`3` are not bound: plain digits are the vim-style count
        // prefix for j/k/Up/Down, and pane focus is behind `Ctrl+N`.
        KeyCode::Tab | KeyCode::BackTab => {
            *cx.focus = cycle_focus_skipping_collapsed(
                *cx.focus,
                key.code == KeyCode::BackTab,
                *cx.left_pane_collapsed,
                *cx.right_pane_collapsed,
            );
            if *cx.focus == Focus::Log {
                *cx.right_panel = RightPanel::CommitDetail;
                commands::graph_takes_keys(cx);
            }
            if *cx.focus == Focus::Refs {
                land_on_open_ref_section(
                    &mut *cx.active_ref_section,
                    &*cx.collapsed_ref_sections,
                    cx.visible_sidebar,
                );
            }
        }
        _ if km.is(Bind::GlobalRefreshAll, &key) => {
            commands::run_command(cx, PaletteCommand::RefreshAll);
        }
        // Retry, while the link has stopped: a refresh request would go to a
        // thread that has ended.
        _ if km.is(Bind::GlobalRefresh, &key) && cx.link_failure.is_some() => {
            *cx.retry_link_requested = true;
        }
        _ if km.is(Bind::GlobalRefresh, &key) => {
            cx.link.request_refresh();
            cx.hosting.demand();
        }
        // Search the commit graph. Plain `f` is free in
        // Normal mode -- the only other `f` is `Ctrl+G f`
        // (fetch), which lives behind the leader chord.
        // Searching the graph while the Diff view owns the
        // center makes no sense, so this returns to the Log
        // first rather than opening a bar over a diff.
        _ if km.is(Bind::GlobalFindInLog, &key) => {
            commands::run_command(cx, PaletteCommand::SearchLog);
        }
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsFilter, &key) => {
            *cx.input_mode = InputMode::FilteringRefs(cx.refs_filter.clone());
        }
        // `/` in the Log (no text field has the keyboard, the changed files
        // are not the focus): the Refs filter, as `/` is when the Refs pane
        // has the focus. The Log's own search stays on `f` (`Ctrl+F` too).
        _ if *cx.focus == Focus::Log
            && !*cx.commit_file_focus
            && cx.composer_focus.is_none()
            && !*cx.commit_filter_focus
            && km.is(Bind::RefsFilter, &key) =>
        {
            *cx.left_pane_collapsed = false;
            *cx.focus = Focus::Refs;
            *cx.input_mode = InputMode::FilteringRefs(cx.refs_filter.clone());
        }
        KeyCode::Esc if *cx.focus == Focus::Refs && !cx.refs_filter.is_empty() => {
            cx.refs_filter.clear();
        }
        // `h` is a vim-tree-style alias for "back out", alongside `Esc`.
        KeyCode::Esc | KeyCode::Char('h') if *cx.focus == Focus::Log && *cx.commit_file_focus => {
            *cx.commit_file_focus = false;
        }
        // The tab picker without the `Ctrl+N` leader (owner: "just `t`").
        _ if km.is(Bind::GlobalSelectTab, &key) => {
            commands::run_command(cx, PaletteCommand::SelectTab);
        }
        // `Enter` on a worktree row is the keyboard double-
        // click: open it as its own tab.
        KeyCode::Enter
            if *cx.focus == Focus::Refs
                && *cx.active_ref_section == SidebarSection::Worktrees
                && !cx
                    .collapsed_ref_sections
                    .contains(SidebarSection::Worktrees.title())
                && cx
                    .sidebar_states
                    .state(SidebarSection::Worktrees)
                    .selected()
                    .is_some() =>
        {
            let selected = cx
                .sidebar_states
                .state(SidebarSection::Worktrees)
                .selected();
            let list = cx.worktrees_result.as_deref().unwrap_or(&[]).to_vec();
            if let Some(message) = open_worktree_in_tab(&list, selected, &mut tab_ctx!(cx)) {
                *cx.last_message = Some(message);
            }
        }
        // Enter on a submodule row: open it as its own tab.
        KeyCode::Enter
            if *cx.focus == Focus::Refs
                && *cx.active_ref_section == SidebarSection::Submodules
                && !cx
                    .collapsed_ref_sections
                    .contains(SidebarSection::Submodules.title())
                && cx
                    .sidebar_states
                    .state(SidebarSection::Submodules)
                    .selected()
                    .is_some() =>
        {
            let selected = cx
                .sidebar_states
                .state(SidebarSection::Submodules)
                .selected();
            if let Some(message) = open_submodule_in_tab(
                &*cx.submodules_result,
                selected,
                &cx.repo.root,
                &mut tab_ctx!(cx),
            ) {
                *cx.last_message = Some(message);
            }
        }
        // Section collapse toggle: Enter/Space toggles whichever section
        // `Left`/`Right` made active, unless the hierarchical tree view is on and
        // the current row is a folder, which it targets instead (the mouse
        // click's own folder-vs-section precedence).
        KeyCode::Enter | KeyCode::Char(' ') if *cx.focus == Focus::Refs => {
            // The row the selection sits on -- but only
            // while its section is open: a collapsed
            // section still remembers its `ListState`,
            // and `Enter` must not check out a row the
            // user cannot see.
            let section_open = !cx
                .collapsed_ref_sections
                .contains(cx.active_ref_section.title());
            let selected_row = section_open
                .then(|| {
                    let state = cx.sidebar_states.state_mut(*cx.active_ref_section);
                    let i = state.selected()?;
                    let groups = group_refs(cx.refs_result.as_ref().ok()?, &*cx.refs_filter);
                    let kind = cx.active_ref_section.ref_kind()?;
                    let rows = ref_display_rows(
                        groups.names(kind),
                        kind,
                        *cx.hierarchical_refs,
                        &*cx.collapsed_ref_folders,
                        &*cx.remote_names,
                    );
                    rows.get(i).cloned().map(|row| (kind, row))
                })
                .flatten();
            match selected_row {
                Some((kind, RefTreeRow::Folder { path, .. })) => {
                    toggle_in(&mut *cx.collapsed_ref_folders, (kind, path));
                }
                // `Enter` on a branch is the keyboard
                // double-click: the same `checkout_ref_now`
                // the mouse calls, remote branches
                // included. `Space` keeps folding the
                // section, as it always has.
                Some((_, RefTreeRow::Leaf { full_name, .. })) if key.code == KeyCode::Enter => {
                    let msg = checkout_ref_now(
                        &full_name,
                        &*cx.refs_result,
                        &*cx.remote_names,
                        cx.link,
                        &mut *cx.input_mode,
                    );
                    if !msg.is_empty() {
                        *cx.last_message = Some(msg);
                    }
                }
                _ => {
                    let name = cx.active_ref_section.title();
                    if !cx.collapsed_ref_sections.remove(name) {
                        cx.collapsed_ref_sections.insert(name.to_string());
                    }
                }
            }
        }
        // The branch-hierarchy toggle: splits
        // LOCAL/REMOTE branch names on `/` into an
        // expandable folder tree instead of one row per
        // branch. On by default (`ui.ref_tree`); Tags never go
        // hierarchical (see `ref_display_rows`).
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsToggleHierarchy, &key) => {
            *cx.last_message = Some(toggle_ref_tree(&mut *cx.hierarchical_refs));
        }
        KeyCode::Left | KeyCode::Char('h') if *cx.focus == Focus::Refs => {
            step_sidebar_section(&mut *cx.active_ref_section, cx.visible_sidebar, -1);
        }
        KeyCode::Right | KeyCode::Char('l') if *cx.focus == Focus::Refs => {
            step_sidebar_section(&mut *cx.active_ref_section, cx.visible_sidebar, 1);
        }
        // Keyboard equivalent of the ref context menu's
        // Solo/Clear-solo entry -- a pure display filter
        // (no git mutation), so unlike delete it's a bare
        // key, same as the sidebar's other navigation/view
        // toggles (h/l/y).
        // On a pull request / issue row the same key opens the page in the
        // browser (Enter opens the conversation).
        _ if *cx.focus == Focus::Refs
            && km.is(Bind::RefsSolo, &key)
            && HostingList::from_section(*cx.active_ref_section).is_some() =>
        {
            if let (Some(list), Some(index)) = (
                HostingList::from_section(*cx.active_ref_section),
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
            ) {
                let effect = cx
                    .hosting
                    .open_row_in_browser(list, index, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
        }
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsSolo, &key) => {
            // Solo only means anything for refs; the
            // worktree/stash panes have nothing to solo.
            if let (Ok(refs), Some(kind)) =
                (cx.refs_result.as_ref(), cx.active_ref_section.ref_kind())
            {
                let groups = group_refs(refs, &*cx.refs_filter);
                let state = cx.sidebar_states.state_mut(*cx.active_ref_section);
                let rows = ref_display_rows(
                    groups.names(kind),
                    kind,
                    *cx.hierarchical_refs,
                    &*cx.collapsed_ref_folders,
                    &*cx.remote_names,
                );
                let selected_row = state.selected().and_then(|i| rows.get(i));
                let mut soloed_something = false;
                match selected_row {
                    Some(RefTreeRow::Leaf { full_name, .. }) => {
                        let full_name = full_name.clone();
                        if cx.solo_refs.remove(&full_name) {
                            *cx.last_message = Some(msg::input::solo_cleared_ref(&full_name));
                        } else {
                            *cx.last_message = Some(msg::input::solo_ref(&full_name));
                            cx.solo_refs.insert(full_name);
                        }
                        soloed_something = true;
                    }
                    // A folder row solos (or un-solos) every
                    // ref beneath it at once -- the same
                    // bulk action the eye gutter and the
                    // folder's context menu offer,
                    // reachable from the keyboard too.
                    Some(RefTreeRow::Folder { path, .. }) => {
                        let under = refs_under_folder(groups.names(kind), path);
                        if !under.is_empty() {
                            let now_soloed = !folder_fully_in(&*cx.solo_refs, &under);
                            toggle_folder_membership(&mut *cx.solo_refs, &under);
                            *cx.last_message = Some(if now_soloed {
                                msg::input::solo_folder(under.len() as u64, path)
                            } else {
                                msg::input::solo_cleared_ref(path)
                            });
                            soloed_something = true;
                        }
                    }
                    None => {}
                }
                if soloed_something {
                    refs_tree::submit_ref_visibility(
                        cx.link,
                        &*cx.hidden_from_graph,
                        &*cx.solo_refs,
                    );
                    // Same immediate recompute as the context-menu path.
                    *cx.log_order = log_rows_now(
                        &*cx.log_result,
                        &*cx.status_result,
                        &*cx.stashes_result,
                        stash_rows_shown(*cx.graph_show_stashes, &*cx.solo_refs),
                        &*cx.solo_refs,
                        &*cx.refs_result,
                    );
                    *cx.log_len = log_len_now(
                        &*cx.log_result,
                        &*cx.solo_refs,
                        &*cx.refs_result,
                        &*cx.log_order,
                    );
                    move_table_selection(&mut *cx.log_state, *cx.log_len, 0);
                    *cx.log_follow = true;
                }
            }
        }
        // Keyboard equivalent of the sidebar header's "Stop
        // Soloing" button -- clears the whole solo set at
        // once, mirroring the mouse path.
        // Reveal in graph: the keyboard twin of the ref menu's first row.
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsMarkDrop, &key) => {
            commands::run_command(cx, PaletteCommand::MarkRefAsDropSource);
        }
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsDropMarked, &key) => {
            commands::run_command(cx, PaletteCommand::DropMarkedRefHere);
        }
        // `Esc` drops the mark before it clears the name filter.
        KeyCode::Esc if *cx.focus == Focus::Refs && cx.ref_drop_mark.is_some() => {
            if let Some((n, _)) = cx.ref_drop_mark.take() {
                *cx.last_message = Some(msg::input::ref_drop_mark_cleared(&n));
            }
        }
        _ if *cx.focus == Focus::Refs && km.is(Bind::RefsReveal, &key) => {
            commands::run_command(cx, PaletteCommand::RevealSelectedRefInGraph);
        }
        _ if *cx.focus == Focus::Refs
            && !cx.solo_refs.is_empty()
            && km.is(Bind::RefsClearSolo, &key) =>
        {
            cx.solo_refs.clear();
            refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            *cx.last_message = Some(msg::input::SOLO_CLEARED.get().to_string());
            *cx.log_order = log_rows_now(
                &*cx.log_result,
                &*cx.status_result,
                &*cx.stashes_result,
                stash_rows_shown(*cx.graph_show_stashes, &*cx.solo_refs),
                &*cx.solo_refs,
                &*cx.refs_result,
            );
            *cx.log_len = log_len_now(
                &*cx.log_result,
                &*cx.solo_refs,
                &*cx.refs_result,
                &*cx.log_order,
            );
            move_table_selection(&mut *cx.log_state, *cx.log_len, 0);
            *cx.log_follow = true;
        }
        // Keyboard equivalent of the sidebar header's "Show
        // All" button -- unhides every ref at once, the
        // bulk complement of `Shift+O`'s solo-clear.
        _ if *cx.focus == Focus::Refs
            && !cx.hidden_from_graph.is_empty()
            && km.is(Bind::RefsShowAll, &key) =>
        {
            cx.hidden_from_graph.clear();
            refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            *cx.last_message = Some(msg::input::HIDDEN_REFS_SHOWN.get().to_string());
        }
        // Moves selection *into* the selected commit's changed-files list (its
        // own `ListState`) instead of flipping the right panel to the Status
        // pane. `l` is a vim-tree-style alias for "go deeper", alongside `Enter`.
        KeyCode::Enter | KeyCode::Char('l')
            if *cx.focus == Focus::Log && !*cx.commit_file_focus && *cx.commit_files_len > 0 =>
        {
            *cx.commit_file_focus = true;
            move_selection(&mut *cx.commit_file_state, *cx.commit_files_len, 0);
            *cx.commit_files_follow = true;
        }
        // Enter *again*, with a file row selected, opens that file's diff in the
        // central column: the keyboard equivalent of double-clicking the row.
        KeyCode::Enter | KeyCode::Char('l')
            if *cx.focus == Focus::Log && *cx.commit_file_focus && *cx.commit_files_len > 0 =>
        {
            // Only openable when we actually know which
            // commit the file list belongs to -- the diff
            // view pins that oid so a later refresh can't
            // slide a different commit's files underneath it.
            // In Tree mode the selected ROW is not the
            // selected FILE -- folder rows sit in between --
            // so the row has to be resolved back to an index
            // into the commit's `Vec<FileDiff>` before the
            // diff view pins it. Enter on a folder row
            // collapses it instead.
            match cx
                .commit_file_state
                .selected()
                .and_then(|i| cx.commit_file_rows.get(i))
            {
                Some(TreeRow::Folder { path, .. }) => {
                    let path = path.clone();
                    toggle_in(&mut *cx.collapsed_commit_folders, path);
                }
                Some(row) => {
                    // The row's index is into the list
                    // the rows were built from -- the
                    // whole tree under View all files --
                    // so it is resolved by path: a diff
                    // for a changed file, the File View
                    // for an unchanged one.
                    let open = match (row.file_index(), cx.commit_files.as_ref()) {
                        (Some(pi), Some(outcome)) => {
                            commit_file_open(outcome, &*cx.commit_paths_cache, pi)
                                .map(|open| (open, outcome.commit))
                        }
                        _ => None,
                    };
                    if let Some((open, commit)) = open {
                        open_commit_file(
                            open,
                            commit,
                            cx.link,
                            &mut *cx.file_insight,
                            &mut *cx.diff_view_state,
                            &mut *cx.central_view,
                            cx.compare.as_ref(),
                        );
                        // The log table is gone from under
                        // them -- anything anchored to it
                        // has to go too.
                        *cx.ref_popup = None;
                        cx.open_menu.close_column_menu();
                    }
                }
                None => {}
            }
        }
        // The commit-detail changed-files list's own
        // menu -- opened anchored on
        // the selected file row, exactly as the Log's,
        // the Refs sidebar's and the third pane's own
        // `.` are. A folder row (Tree mode) has no
        // file-specific menu, so it explains itself
        // instead of doing nothing.
        _ if km.is(Bind::GlobalContextMenu, &key)
            && *cx.focus == Focus::Log
            && cx.compare.is_none()
            && *cx.commit_file_focus =>
        {
            super::commands::open_menu_from_keyboard(cx);
        }
        // `count` is the vim-style prefix accumulated
        // above (`4j` etc.) -- 1 when none was typed.
        // Every one of these motions threads it through
        // as the delta instead of a hard-coded `±1`.
        KeyCode::Up | KeyCode::Char('k') => match *cx.focus {
            Focus::Refs => {
                let wt_len = {
                    let list = cx.worktrees_result.as_deref().unwrap_or(&[]);
                    if linked_worktree_count(list) == 0 {
                        0
                    } else {
                        list.len()
                    }
                };
                let st_len = cx.stashes_result.as_deref().unwrap_or(&[]).len();
                move_sidebar_selection(
                    &mut *cx.active_ref_section,
                    &mut *cx.sidebar_states,
                    cx.visible_sidebar,
                    &*cx.collapsed_ref_sections,
                    -count,
                    |section| {
                        ref_section_row_count(
                            &*cx.refs_result,
                            &*cx.refs_filter,
                            section,
                            *cx.hierarchical_refs,
                            &*cx.collapsed_ref_folders,
                            &*cx.remote_names,
                            wt_len,
                            st_len,
                            cx.hosting.row_counts(),
                            cx.submodules_result.len(),
                        )
                    },
                );
            }
            Focus::Log if *cx.commit_file_focus => {
                move_selection(&mut *cx.commit_file_state, *cx.commit_files_len, -count);
                *cx.commit_files_follow = true;
            }
            Focus::Log => {
                if !land_unselected_log(cx) && !step_lit_rows(cx, -count, false) {
                    move_table_selection(&mut *cx.log_state, *cx.log_len, -count);
                }
                *cx.log_follow = true;
                *cx.right_panel = RightPanel::CommitDetail;
            }
            Focus::Status if cx.wip_all.shown() => cx.wip_all.move_by(-count),
            Focus::Status => {
                move_status_selection(
                    &mut *cx.status_states,
                    &*cx.status_section_lines,
                    &*cx.collapsed_status_sections,
                    &mut *cx.active_status_section,
                    -count,
                );
                *cx.right_panel = RightPanel::Status;
            }
        },
        KeyCode::Down | KeyCode::Char('j') => match *cx.focus {
            Focus::Refs => {
                let wt_len = {
                    let list = cx.worktrees_result.as_deref().unwrap_or(&[]);
                    if linked_worktree_count(list) == 0 {
                        0
                    } else {
                        list.len()
                    }
                };
                let st_len = cx.stashes_result.as_deref().unwrap_or(&[]).len();
                move_sidebar_selection(
                    &mut *cx.active_ref_section,
                    &mut *cx.sidebar_states,
                    cx.visible_sidebar,
                    &*cx.collapsed_ref_sections,
                    count,
                    |section| {
                        ref_section_row_count(
                            &*cx.refs_result,
                            &*cx.refs_filter,
                            section,
                            *cx.hierarchical_refs,
                            &*cx.collapsed_ref_folders,
                            &*cx.remote_names,
                            wt_len,
                            st_len,
                            cx.hosting.row_counts(),
                            cx.submodules_result.len(),
                        )
                    },
                );
            }
            Focus::Log if *cx.commit_file_focus => {
                move_selection(&mut *cx.commit_file_state, *cx.commit_files_len, count);
                *cx.commit_files_follow = true;
            }
            Focus::Log => {
                if !land_unselected_log(cx) && !step_lit_rows(cx, count, false) {
                    move_table_selection(&mut *cx.log_state, *cx.log_len, count);
                }
                *cx.log_follow = true;
                *cx.right_panel = RightPanel::CommitDetail;
            }
            Focus::Status if cx.wip_all.shown() => cx.wip_all.move_by(count),
            Focus::Status => {
                move_status_selection(
                    &mut *cx.status_states,
                    &*cx.status_section_lines,
                    &*cx.collapsed_status_sections,
                    &mut *cx.active_status_section,
                    count,
                );
                *cx.right_panel = RightPanel::Status;
            }
        },
        // A page is one visible screen of rows, the same `log_visible_rows` the
        // lazy loader measures the window by, so a PageDown past what is loaded
        // triggers the next page like a manual scroll. Diff view and File Insight
        // have their own PageUp/PageDown arms earlier; this one is the commit
        // graph only. Ctrl+PageUp/PageDown mean previous/next repository tab, so
        // these arms are guarded off while Ctrl is held.
        KeyCode::PageUp
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            // A count on PageUp/PageDown means that many
            // *screens*, not that many rows -- `4 PageDown`
            // scrolls four screens down.
            let delta = -log_page_delta(*cx.log_area).saturating_mul(count);
            if !step_lit_rows(cx, delta, true) {
                move_table_selection(&mut *cx.log_state, *cx.log_len, delta);
            }
            *cx.right_panel = RightPanel::CommitDetail;
        }
        KeyCode::PageDown
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let delta = log_page_delta(*cx.log_area).saturating_mul(count);
            if !step_lit_rows(cx, delta, true) {
                move_table_selection(&mut *cx.log_state, *cx.log_len, delta);
            }
            *cx.right_panel = RightPanel::CommitDetail;
        }
        // The first/last *loaded* commit -- not the
        // repository's actual first/last, which lazy
        // loading may not have fetched yet. Landing on
        // the last loaded row is exactly what makes
        // `End` page further: the same post-draw check
        // that answers a manual scroll sees the
        // selection (and the offset that follows it)
        // sitting at the loaded ceiling and asks for
        // the next page.
        KeyCode::Home
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && *cx.log_len > 0 =>
        {
            let first = lit_rows(cx).map_or(Some(0), |h| h.first().copied());
            if first.is_some() {
                cx.log_state.select(first);
            }
            *cx.log_follow = true;
            *cx.right_panel = RightPanel::CommitDetail;
        }
        KeyCode::End
            if *cx.focus == Focus::Log
                && !*cx.commit_file_focus
                && *cx.central_view == CentralView::Log
                && *cx.log_len > 0 =>
        {
            let last = lit_rows(cx).map_or(Some(*cx.log_len - 1), |h| h.last().copied());
            if last.is_some() {
                cx.log_state.select(last);
            }
            *cx.log_follow = true;
            *cx.right_panel = RightPanel::CommitDetail;
        }
        // `Shift+F10`/the Menu key fall back to opening the current repository
        // tab's menu (the keyboard twin of right-clicking a tab, like `Ctrl+T`)
        // when no more specific menu arm above claimed the key. `.` deliberately
        // does NOT fall back: it has no meaning outside the panes that already
        // bind it, and letting it open a tab menu would make one key mean two
        // things.
        _ if km.is(Bind::GlobalContextMenu, &key)
            && key.code != KeyCode::Char('.')
            && cx.management.is_none() =>
        {
            let anchor = cx
                .tab_hits
                .iter()
                .find(|h| h.index == *cx.active_tab)
                .map(|h| (h.tab.x, h.tab.y + 1))
                .unwrap_or((0, 1));
            if let Some(message) = open_current_tab_menu(anchor, &mut tab_ctx!(cx)) {
                *cx.last_message = Some(message);
            }
        }
        // Last Esc owner of all: a running sequence stops only when nothing
        // above wanted the key (a dialog, a field, a menu, a view).
        KeyCode::Esc if crate::sequence_run::active() => crate::sequence_run::request_cancel(),
        _ => {}
    }
    Ok(Flow::Handled)
}

/// With no graph row selected, the first move selects one instead of
/// stepping from a phantom row 0: HEAD's row when it is shown, else the
/// first visible row. Returns whether it selected (the caller then skips
/// the step).
fn land_unselected_log(cx: &mut CommandCtx<'_>) -> bool {
    if cx.log_state.selected().is_some() || *cx.log_len == 0 {
        return false;
    }
    let head = cx.refs_result.as_deref().ok().and_then(head_oid);
    let on_head = head.is_some_and(|oid| {
        select_log_commit(&*cx.log_result, &*cx.log_order, &mut *cx.log_state, oid)
    });
    if !on_head {
        let first = cx.log_state.offset().min(*cx.log_len - 1);
        cx.log_state.select(Some(first));
    }
    true
}

/// The display rows that stay lit while the author filter is on, ascending;
/// `None` with no filter, so plain navigation is untouched. (A text search
/// steps through its hits itself, in `InputMode::SearchingLog`.) The WIP and
/// stash rows are never hits, exactly as the paint dims them.
fn lit_rows(cx: &CommandCtx<'_>) -> Option<Vec<usize>> {
    if cx.log_author_filter.is_empty() {
        return None;
    }
    let hits = crate::log_table::log_search_hits(
        &*cx.log_result,
        &*cx.refs_result,
        &*cx.solo_refs,
        "",
        &*cx.log_author_filter,
    );
    Some(
        hits.into_iter()
            .map(|i| cx.log_order.shown_display_index(i))
            .collect(),
    )
}

/// Moves the Log selection between lit rows when a filter is on. Returns
/// whether a filter handled the key (the selection may stay put).
fn step_lit_rows(cx: &mut CommandCtx<'_>, delta: i32, page: bool) -> bool {
    let Some(rows) = lit_rows(cx) else {
        return false;
    };
    let selected = cx.log_state.selected();
    let to = if page {
        crate::log_search::page_among_hits(&rows, selected, delta)
    } else {
        crate::log_search::step_among_hits(&rows, selected, delta)
    };
    if let Some(row) = to {
        cx.log_state.select(Some(row));
    }
    true
}
