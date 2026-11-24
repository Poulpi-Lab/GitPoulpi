//! Right press: the context menus.

use super::commands::{CommandCtx, Flow};
use super::mouse_prelude::Geo;
use super::KeyEnv;
use crate::event_loop::*;
use crate::history_view::HistoryHit;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// A press of the right button opens the context menu of what is under it.
pub(super) fn handle(
    cx: &mut CommandCtx<'_>,
    env: &KeyEnv<'_>,
    g: &Geo,
    _mouse: MouseEvent,
) -> io::Result<Flow> {
    let unpushed_tags = env.unpushed_tags;
    let Geo {
        col, row, in_log, ..
    } = *g;
    // A right-click on a conflict of the merge editor opens its menu.
    if let Some(HitTarget::Merge(h)) = find_hit_target(&*cx.hit_targets, col, row) {
        if let Some(view) = cx.merge.as_mut() {
            let out = view.right_click(h, col, row);
            commands::apply_merge_outcome(cx, out);
        }
        return Ok(Flow::Handled);
    }
    // Popups own the pointer: a right-click on a row of an open menu is
    // not a right-click on the pane behind it (one menu at a time, so it
    // must not open a second one under or over the first).
    if matches!(
        find_hit_target(&*cx.hit_targets, col, row),
        Some(
            HitTarget::RemoteMenuItem(_)
                | HitTarget::RefContextMenuItem(_)
                | HitTarget::LogContextMenuItem(_)
                | HitTarget::WipContextMenuItem(_)
                | HitTarget::PaneContextMenuItem(_)
                | HitTarget::DiffHunkContextMenuItem(_)
                | HitTarget::HistoryContextMenuItem(_)
                | HitTarget::CommitColumn(crate::commit_column::ColumnHit::ColumnsMenuItem(_))
                | HitTarget::FileContextMenuItem(_)
                | HitTarget::FolderContextMenuItem(_)
                | HitTarget::BarOverflowMenuItem(_)
                | HitTarget::Hosting(HostingHit::MenuItem(_))
        )
    ) {
        return Ok(Flow::Handled);
    }
    // A right-click on the commit-detail pane's message, description or
    // author opens that property's Amend menu.
    if *cx.right_panel == RightPanel::CommitDetail && cx.compare.is_none() {
        let scope = match find_hit_target(&*cx.hit_targets, col, row) {
            Some(HitTarget::CommitDetailPart(scope)) => Some(scope),
            Some(HitTarget::CommitAuthorName) => Some(AmendScope::Author),
            _ => None,
        };
        let commit = scope
            .and_then(|_| selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order));
        if let (Some(scope), Some(commit)) = (scope, commit) {
            *cx.log_menu_refs = Default::default();
            *cx.open_menu = OpenMenu::Log((
                commit,
                ContextMenu::new(
                    &commit.to_hex()[..SHORT_SHA_LEN],
                    detail_part_menu_entries(scope),
                    col,
                    row,
                ),
            ));
            return Ok(Flow::Handled);
        }
    }
    // A right-click on the commit column's header row opens the menu of the
    // parts shown (the Log header's way; the palette's `Choose the Columns`
    // and its per-part toggles are the keyboard twins).
    if cx.commit_col.header_contains(col, row) {
        let part = cx.commit_col.part_at(col);
        commands::commit_column::open_columns_menu(cx, Some((col, row)), part);
        return Ok(Flow::Handled);
    }
    // A right-click on a row of the commit column (its Graph) opens the
    // commit's menu: the views of the open file at that commit, then the
    // Log's own rows. The menu key is the keyboard twin.
    if let Some(HitTarget::CommitColumn(crate::commit_column::ColumnHit::Row(id))) =
        find_hit_target(&*cx.hit_targets, col, row)
    {
        commands::commit_column::open_menu_for(cx, id, col, row, unpushed_tags);
        return Ok(Flow::Handled);
    }
    // A right-click on a File History row opens its menu.
    if let Some(HitTarget::History(HistoryHit::Row(i))) =
        find_hit_target(&*cx.hit_targets, col, row)
    {
        commands::history::open_menu(cx, i, col, row);
        return Ok(Flow::Handled);
    }
    // A right-click on a pull request / issue row or
    // on the toolbar chip opens the hosting menu.
    cx.hosting.menu = None;
    if let Some(HitTarget::Hosting(target)) = find_hit_target(&*cx.hit_targets, col, row) {
        match target {
            HostingHit::Row(list, index) => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = list.section();
                cx.sidebar_states.select(list.section(), Some(index));
                cx.hosting.open_row_menu(list, index, col, row);
            }
            // The pencil is part of its row: same menu.
            HostingHit::EditRow(index) => {
                let list = HostingList::Pulls;
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = list.section();
                cx.sidebar_states.select(list.section(), Some(index));
                cx.hosting.open_row_menu(list, index, col, row);
            }
            HostingHit::LinkedPrChip | HostingHit::NewPrChip => {
                cx.hosting.open_chip_menu(col, row);
            }
            HostingHit::NewPr
            | HostingHit::NewIssue
            | HostingHit::Board
            | HostingHit::MenuItem(_)
            | HostingHit::PickerItem(_) => {}
        }
        return Ok(Flow::Handled);
    }
    // Right-click anywhere on the Log header row
    // opens the column show/hide menu -- doesn't
    // need a dedicated hit target since the whole
    // header row qualifies, not just a specific
    // cell.
    // `in_log` is false while the Diff view owns the column, so this can't
    // open a column menu over a diff.
    // A right-click on a lit funnel drops the author filter (the header's
    // column menu stays for the rest of the row).
    if in_log
        && !cx.log_author_filter.is_empty()
        && matches!(
            find_hit_target(&*cx.hit_targets, col, row),
            Some(HitTarget::LogAuthorFilter)
        )
    {
        commands::clear_author_filter(cx);
        return Ok(Flow::Handled);
    }
    if in_log && row == cx.log_area.y + 1 {
        // The "fit" row acts on the column right-clicked.
        let target = crate::log_table::log_hover_column(&cx.log_geo().offsets, col);
        *cx.open_menu = OpenMenu::Column(crate::log_table::ColumnMenu::new(target));
    } else {
        cx.open_menu.close_column_menu();
    }
    // A ref chip in the Log's Ref column opens the *ref* menu, not the commit
    // menu. Checked before the commit-row branch below because the chip sits
    // inside that row's band (the inner-first rule). The menu is the ref's
    // rows and the commit's in one list (`pill_menu`), and the rows of the
    // `+N` reveal popup (`TooltipRefPill`) open the same menu for the ref they
    // name.
    let pill_hit = find_hit_target(&*cx.hit_targets, col, row).and_then(|h| match h {
        HitTarget::LogRefPill { name, kind } => Some((name, kind, None)),
        HitTarget::TooltipRefPill { name, kind } => {
            Some((name, kind, cx.ref_popup.as_ref().map(|p| p.commit_id)))
        }
        _ => None,
    });
    let mut opened_pill_menu = false;
    if in_log && !cx.open_menu.is_column_menu() {
        if let Some((name, kind, popup_commit)) = pill_hit {
            let commit = popup_commit.or_else(|| {
                log_commit_at_row(
                    &*cx.log_result,
                    &*cx.log_order,
                    *cx.log_area,
                    cx.log_state.offset(),
                    *cx.log_len,
                    row,
                )
            });
            let identity = commit
                .zip(cx.refs_result.as_deref().ok())
                .and_then(|(commit, refs)| pill_identity(refs, commit, &name, kind));
            if let (Some(commit), Some(identity)) = (commit, identity) {
                let caption = identity.1.clone();
                let built = pill_menu(
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
                    identity,
                );
                *cx.focus = Focus::Log;
                *cx.ref_popup = None;
                *cx.log_menu_refs = built.table;
                *cx.open_menu =
                    OpenMenu::Log((commit, ContextMenu::new(caption, built.entries, col, row)));
                opened_pill_menu = true;
            }
        }
    }
    // A stash row opens the stash's menu -- the
    // STASHES section's own Pop / Apply / Drop.
    if in_log && !cx.open_menu.is_column_menu() && !cx.open_menu.is_ref_menu() {
        if let Some(stash) = log_row_to_index(*cx.log_area, cx.log_state.offset(), row, *cx.log_len)
            .and_then(|i| log_stash_at(&*cx.stashes_result, &*cx.log_order, i))
        {
            *cx.focus = Focus::Log;
            *cx.open_menu = OpenMenu::Ref(stash_context_menu(&stash, col, row));
        }
    }
    // A right-click on the WIP row opens its own menu, checked before the
    // commit row below so the WIP row gets its own row caption instead of
    // falling through to "nothing here".
    if in_log && !cx.open_menu.is_column_menu() && !cx.open_menu.is_ref_menu() {
        let is_wip_row = log_row_to_index(*cx.log_area, cx.log_state.offset(), row, *cx.log_len)
            .is_some_and(|i| matches!(cx.log_order.entry(i), graph::LogEntry::Wip));
        if is_wip_row {
            *cx.focus = Focus::Log;
            *cx.open_menu = OpenMenu::Wip(ContextMenu::new(
                msg::input::WIP_MENU_TITLE.get(),
                wip_menu_entries(has_wip_row(&*cx.status_result), cx.ai_ui.enabled()),
                col,
                row,
            ));
        }
    }
    if in_log
        && !cx.open_menu.is_column_menu()
        && !opened_pill_menu
        && !cx.open_menu.is_ref_menu()
        && !cx.open_menu.is_wip_menu()
    {
        if let Some(commit) = log_commit_at_row(
            &*cx.log_result,
            &*cx.log_order,
            *cx.log_area,
            cx.log_state.offset(),
            *cx.log_len,
            row,
        ) {
            *cx.focus = Focus::Log;
            // Right-click inside the selection keeps it (the
            // menu then acts on all of it); outside, it is
            // just that one commit.
            if !cx.log_multi.contains_id(&commit) {
                cx.log_multi.clear();
            }
            // The commit's rows, plus one submenu per
            // ref on it (`pill_menu::commit_menu`).
            let built = commit_menu(
                &MenuSources {
                    refs_result: &*cx.refs_result,
                    log_result: &*cx.log_result,
                    remotes: &cx.remote_ui.remotes,
                    hidden: &*cx.hidden_from_graph,
                    solo: &*cx.solo_refs,
                    unpushed_tags,
                    // Only commits can be cherry-picked: the WIP row and
                    // stashes of a comparison are not counted.
                    selected_count: cx.log_multi.newest_first(&*cx.log_result).len(),
                    ai_enabled: cx.ai_ui.enabled(),
                    merge_ff: cx.commit_settings.merge_ff,
                },
                commit,
            );
            *cx.log_menu_refs = built.table;
            *cx.open_menu = OpenMenu::Log((
                commit,
                ContextMenu::new(&commit.to_hex()[..SHORT_SHA_LEN], built.entries, col, row),
            ));
        }
    }
    // Right-click a file row in the working-changes
    // pane opens its per-file menu; a folder row
    // in the Unstaged tree opens its own -- a section header still offers
    // neither.
    if *cx.right_panel == RightPanel::Status {
        if let Some((section, area)) = cx
            .status_section_areas
            .iter()
            .copied()
            .find(|(_, a)| a.contains((col, row).into()))
        {
            let lines = section_lines(&*cx.status_section_lines, section).to_vec();
            let offset = cx.status_states.state(section).offset();
            let list = match section {
                StatusSection::Unstaged => FileList::Unstaged,
                StatusSection::Staged => FileList::Staged,
            };
            let at = row_to_index(area, offset, 1, row, lines.len());
            // A right-click inside a multi-selection acts on all of it; on
            // a row outside it, the selection starts over with that row.
            if let Some(menu) = commands::selection_menu(cx, list, at, col, row) {
                *cx.focus = Focus::Status;
                *cx.active_status_section = section;
                *cx.open_menu = menu;
                return Ok(Flow::Continue);
            }
            if at.is_some() {
                cx.file_sel.clear();
            }
            if let Some(i) = at {
                match lines.get(i) {
                    Some(StatusPaneLine::File { .. }) => {
                        if let Some((sec, path)) = lines[i].file() {
                            cx.status_states.select(section, Some(i));
                            *cx.active_status_section = section;
                            *cx.open_menu = OpenMenu::File((
                                sec,
                                path.clone(),
                                open_file_menu(
                                    sec,
                                    path,
                                    &*cx.status_result,
                                    &*cx.conflicts,
                                    &*cx.lfs,
                                    cx.repo_root,
                                    col,
                                    row,
                                ),
                            ));
                        }
                    }
                    Some(StatusPaneLine::Folder { path, .. }) => {
                        let path = path.clone();
                        cx.status_states.select(section, Some(i));
                        *cx.active_status_section = section;
                        let menu = open_folder_menu(section, &path, col, row);
                        *cx.open_menu = OpenMenu::Folder((section, path, menu));
                    }
                    _ => {}
                }
            } else {
                // Outside any item: the pane's own menu (Create File, the
                // bulk buttons' twins, the repository folder).
                *cx.open_menu = OpenMenu::Folder((
                    section,
                    PathBuf::new(),
                    open_empty_area_menu(section, col, row),
                ));
            }
        }
    }
    // Right-click on the commit's changed-files list (a file, a folder, the
    // empty area, or a multi-selection) or on the working tree's View all
    // files list: the mouse twin of the menu key.
    if *cx.right_panel == RightPanel::CommitDetail
        && cx.compare.is_none()
        && cx.commit_files_area.contains((col, row).into())
    {
        let at = row_to_index(
            *cx.commit_files_area,
            cx.commit_file_state.offset(),
            1 + commit_detail::FILES_HEADER_ROWS,
            row,
            *cx.commit_files_len,
        );
        let menu = commands::selection_menu(cx, FileList::Commit, at, col, row).or_else(|| {
            if at.is_some() {
                cx.file_sel.clear();
            }
            commands::list_row_menu(cx, FileList::Commit, at, col, row)
        });
        if let Some(menu) = menu {
            if let Some(i) = at {
                cx.commit_file_state.select(Some(i));
            }
            *cx.commit_files_follow = true;
            *cx.commit_file_focus = true;
            *cx.open_menu = menu;
        }
    }
    if cx.wip_all.shown() && cx.wip_all.area.contains((col, row).into()) {
        let at = cx.wip_all.row_at(row);
        let menu = commands::selection_menu(cx, FileList::WipAll, at, col, row).or_else(|| {
            if at.is_some() {
                cx.file_sel.clear();
            }
            commands::list_row_menu(cx, FileList::WipAll, at, col, row)
        });
        if let Some(menu) = menu {
            if let Some(i) = at {
                cx.wip_all.state.select(Some(i));
                cx.wip_all.follow = true;
            }
            *cx.focus = Focus::Status;
            *cx.open_menu = menu;
        }
    }
    // Right-click a diff row opens the hunk-under-
    // the-click menu -- the
    // mouse twin of the keyboard path in the
    // `InputMode::Normal` key match.
    if *cx.central_view == CentralView::Diff {
        // A row's band, or one of the buttons drawn
        // over it (a hovered changed row always
        // shows them): either way the row under
        // the pointer, by geometry.
        let on_diff_row = matches!(
            find_hit_target(&*cx.hit_targets, col, row),
            Some(HitTarget::DiffRow(_) | HitTarget::DiffBodyButton(_))
        );
        if let (true, Some(s)) = (on_diff_row, cx.diff_view_state.as_ref()) {
            if let Some(i) =
                diff_view::diff_row_at(*cx.diff_body_area, s.scroll, *cx.diff_rendered_len, row)
            {
                if let Some(menu) = open_diff_hunk_menu(s, &*cx.diff_rows, i, col, row) {
                    *cx.open_menu = OpenMenu::DiffHunk(menu);
                }
            }
        }
    }
    // Right-click on a leaf ref row in the sidebar
    // opens its "hide/show in graph" context menu
    // (the per-branch eye-icon toggle)
    // -- a folder row has no such action, so only
    // resolves for RefTreeRow::Leaf.
    let sections = cx.sidebar_section_areas.clone();
    // The left pane's show/hide checklist: right-click a section's header
    // row, or the pane's own area outside every section (the filter box, the
    // gap under the last section). Palette twin: "Left pane: Show/Hide
    // Sections...".
    // A refs section that has folders: its header or the empty space under
    // its rows opens the fold menu (Expand all / Collapse all), which leads
    // on to the checklist. Without folders the checklist opens as before.
    if matches!(*cx.input_mode, InputMode::Normal) && !*cx.left_pane_collapsed {
        if let Some((section, area)) = sections
            .iter()
            .copied()
            .find(|(_, a)| a.contains((col, row).into()))
        {
            if let (Some(kind), Ok(refs)) = (section.ref_kind(), cx.refs_result.as_ref()) {
                let names = group_refs(refs, &*cx.refs_filter).names(kind).to_vec();
                let has_folders = ref_display_rows(
                    &names,
                    kind,
                    *cx.hierarchical_refs,
                    &CollapsedFolders::new(),
                    &*cx.remote_names,
                )
                .iter()
                .any(|r| r.folder_path().is_some());
                let shown = ref_display_rows(
                    &names,
                    kind,
                    *cx.hierarchical_refs,
                    &*cx.collapsed_ref_folders,
                    &*cx.remote_names,
                )
                .len();
                let offset = cx.sidebar_states.state(section).offset();
                let on_row = row != area.y && row_to_index(area, offset, 1, row, shown).is_some();
                if has_folders && !on_row {
                    *cx.focus = Focus::Refs;
                    *cx.active_ref_section = section;
                    *cx.open_menu = OpenMenu::Ref((
                        section,
                        crate::ref_menu::SECTION_MENU_KEY.to_string(),
                        ContextMenu::new(
                            section.display_title(),
                            crate::ref_menu::section_menu_entries(),
                            col,
                            row,
                        ),
                    ));
                    return Ok(Flow::Handled);
                }
            }
        }
    }
    let in_left_pane = !*cx.left_pane_collapsed
        && cx.refs_area.contains((col, row).into())
        && !cx
            .sidebar_section_areas
            .iter()
            .any(|(_, a)| a.contains((col, row).into()) && row != a.y);
    if in_left_pane && matches!(*cx.input_mode, InputMode::Normal) {
        commands::open_sidebar_sections_menu(cx, col, row);
        return Ok(Flow::Handled);
    }
    if let Some((section, area)) = sections
        .into_iter()
        .find(|(_, a)| a.contains((col, row).into()))
    {
        match section.ref_kind() {
            Some(kind) => {
                let rows_owned = cx
                    .refs_result
                    .as_ref()
                    .map(|refs| {
                        let groups = group_refs(refs, &*cx.refs_filter);
                        ref_display_rows(
                            groups.names(kind),
                            kind,
                            *cx.hierarchical_refs,
                            &*cx.collapsed_ref_folders,
                            &*cx.remote_names,
                        )
                    })
                    .unwrap_or_default();
                let state = cx.sidebar_states.state_mut(section);
                let clicked = row_to_index(area, state.offset(), 1, row, rows_owned.len())
                    .and_then(|i| rows_owned.get(i));
                // A remote's own row (the REMOTE
                // tree's top level) opens the
                // remote's menu: copy name/link,
                // fetch, set as default, edit,
                // remove.
                if let Some(RefTreeRow::Folder {
                    path,
                    is_remote_root: true,
                    ..
                }) = clicked
                {
                    *cx.focus = Focus::Refs;
                    let hidden = crate::refs_tree::remote_hidden(&*cx.hidden_from_graph, path);
                    cx.remote_ui.open_remote_menu(
                        path,
                        hidden,
                        cx.refs_result.as_deref().unwrap_or(&[]),
                        col,
                        row,
                    );
                }
                // Any other folder row (a `/`-
                // derived one, in LOCAL or nested
                // under a remote): the bulk hide/
                // solo menu, its context menu.
                if let Some(RefTreeRow::Folder {
                    path,
                    is_remote_root: false,
                    ..
                }) = clicked
                {
                    let names: Vec<String> = cx
                        .refs_result
                        .as_ref()
                        .map(|refs| group_refs(refs, "").names(kind).to_vec())
                        .unwrap_or_default();
                    let under = refs_under_folder(&names, path);
                    *cx.focus = Focus::Refs;
                    *cx.open_menu = OpenMenu::Ref((
                        section,
                        path.clone(),
                        ContextMenu::new(
                            path.clone(),
                            folder_menu_entries(
                                folder_fully_in(&*cx.hidden_from_graph, &under)
                                    || (section == SidebarSection::Remote
                                        && refs_tree::hidden_through_remote(
                                            &*cx.hidden_from_graph,
                                            path,
                                        )),
                                folder_fully_in(&*cx.solo_refs, &under),
                            ),
                            col,
                            row,
                        ),
                    ));
                }
                if let Some(RefTreeRow::Leaf { full_name, .. }) = clicked {
                    *cx.open_menu = OpenMenu::Ref((
                        section,
                        full_name.clone(),
                        ContextMenu::new(
                            full_name.clone(),
                            ref_menu_entries(
                                section,
                                full_name,
                                &menu_context_for_ref(
                                    &*cx.refs_result,
                                    &*cx.log_result,
                                    section,
                                    full_name,
                                    &cx.remote_ui.remotes,
                                    cx.commit_settings.merge_ff,
                                ),
                                refs_tree::hidden_in_graph(
                                    &*cx.hidden_from_graph,
                                    full_name,
                                    section == SidebarSection::Remote,
                                ),
                                cx.solo_refs.contains(full_name.as_str()),
                                unpushed_tags.contains(full_name.as_str()),
                            ),
                            col,
                            row,
                        ),
                    ));
                }
            }
            // A submodule's menu: its path is the row's
            // identity.
            None if section == SidebarSection::Submodules => {
                let state = cx.sidebar_states.state_mut(section);
                if let Some(sub) =
                    row_to_index(area, state.offset(), 1, row, cx.submodules_result.len())
                        .and_then(|i| cx.submodules_result.get(i))
                {
                    *cx.open_menu = OpenMenu::Ref((
                        section,
                        sub.path.clone(),
                        ContextMenu::new(
                            sub.path.clone(),
                            ref_menu_entries(
                                section,
                                &sub.path,
                                &MenuContext::default(),
                                false,
                                false,
                                false,
                            ),
                            col,
                            row,
                        ),
                    ));
                }
            }
            // A worktree row: Open in a new tab / Open in / Merge /
            // Remove. Its identity is its folder.
            #[cfg(feature = "worktree")]
            None if section == SidebarSection::Worktrees => {
                let list = cx.worktrees_result.as_deref().unwrap_or(&[]).to_vec();
                if linked_worktree_count(&list) > 0 {
                    let state = cx.sidebar_states.state_mut(section);
                    if let Some(i) = row_to_index(area, state.offset(), 1, row, list.len()) {
                        cx.sidebar_states.select(section, Some(i));
                        let wt = &list[i];
                        *cx.open_menu = OpenMenu::Ref((
                            section,
                            wt.path.display().to_string(),
                            ContextMenu::new(
                                crate::worktree_remove_dialog::dir_name(&wt.path),
                                crate::ref_menu::worktree_menu_entries(wt),
                                col,
                                row,
                            ),
                        ));
                    }
                }
            }
            // Stashes carry their id as the row
            // identity; worktrees are read-only
            // and get no menu at all.
            None if section == SidebarSection::Stashes => {
                let entries = cx.stashes_result.as_deref().unwrap_or(&[]).to_vec();
                let state = cx.sidebar_states.state_mut(section);
                if let Some(st) = row_to_index(area, state.offset(), 1, row, entries.len())
                    .and_then(|i| entries.get(i))
                {
                    // The same menu a stash row in
                    // the Log opens.
                    *cx.open_menu = OpenMenu::Ref(stash_context_menu(st, col, row));
                }
            }
            None => {}
        }
    }
    Ok(Flow::Continue)
}
