//! Left press: the open context menus, hosting rows and the chrome controls.

use super::commands::{self, CommandCtx, Flow};
use super::menu_actions;
use super::mouse_left::Click;
use super::mouse_prelude::Geo;
use crate::event_loop::*;
use crossterm::event::MouseEvent;
use gitpoulpi_i18n::msg;

/// The open menus, the hosting rows and the chrome buttons: everything a press can
/// hit before the pane under it.
pub(super) fn handle(
    cx: &mut CommandCtx<'_>,
    g: &Geo,
    _mouse: MouseEvent,
    click: &Click,
) -> io::Result<Flow> {
    let Geo { col, row, .. } = *g;
    let hit = click.hit.clone();
    let resize_grab = click.resize_grab;
    let inside_ref_popup = click.inside_ref_popup;
    if let Some(HitTarget::RefContextMenuItem(idx)) = hit {
        // The click runs the *same* `activate` the
        // keyboard's Enter runs, so a submenu row
        // opens (rather than firing something) and
        // the `Back` row goes back, by either
        // input. The menu only closes on `Run` or
        // `Close`.
        let outcome = cx
            .open_menu
            .ref_menu_mut()
            .map(|(_, _, menu)| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(action)) if crate::sections_menu::is_sections_action(action) => {
                commands::apply_sidebar_sections_action(cx, action);
            }
            Some(MenuOutcome::Run(action)) if commands::is_ref_fold_action(action) => {
                commands::apply_ref_fold_action(cx, action);
            }
            Some(MenuOutcome::Run(action)) => {
                if let Some((section, full_name, _)) = cx.open_menu.take_ref_menu() {
                    apply_ref_context_action(
                        action,
                        section,
                        &full_name,
                        RefActionCtx {
                            refs_result: &*cx.refs_result,
                            log_result: &*cx.log_result,
                            status_result: &*cx.status_result,
                            hidden_from_graph: &mut *cx.hidden_from_graph,
                            solo_refs: &mut *cx.solo_refs,
                            log_len: &mut *cx.log_len,
                            log_order: &mut *cx.log_order,
                            stashes_result: &*cx.stashes_result,
                            graph_show_stashes: *cx.graph_show_stashes,
                            log_state: &mut *cx.log_state,
                            link: cx.link,
                            clipboard: &mut *cx.clipboard,
                            input_mode: &mut *cx.input_mode,
                            last_message: &mut *cx.last_message,
                            remotes: &*cx.remote_names,
                            remote_ui: &mut *cx.remote_ui,
                            worktrees: cx.worktrees_result.as_deref().unwrap_or(&[]),
                            worktree_seed: &mut *cx.worktree_seed,
                            pull_request_seed: &mut *cx.pull_request_seed,
                            drop_source: cx.ref_drop_source.as_ref(),
                            submodules: &*cx.submodules_result,
                            repo_root: &cx.repo.root,
                            open_in_tab: &mut *cx.submodule_open,
                            reveal: &mut *cx.reveal,
                        },
                    );
                    refs_tree::submit_ref_visibility(
                        cx.link,
                        &*cx.hidden_from_graph,
                        &*cx.solo_refs,
                    );
                }
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_ref_menu(),
            _ => {}
        }
    } else if let (Some(HitTarget::LogContextMenuItem(idx)), Some(_)) =
        (&hit, cx.open_menu.log_menu())
    {
        let idx = *idx;
        let outcome = cx
            .open_menu
            .log_menu_mut()
            .map(|(_, menu)| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                if let Some((commit, _)) = cx.open_menu.take_log_menu() {
                    menu_actions::run_log_menu_action(cx, action, commit);
                }
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_log_menu(),
            _ => {}
        }
    } else if let (Some(HitTarget::WipContextMenuItem(idx)), Some(_)) =
        (&hit, cx.open_menu.wip_menu())
    {
        let idx = *idx;
        let outcome = cx.open_menu.wip_menu_mut().map(|menu| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                cx.open_menu.close_wip_menu();
                menu_actions::apply_wip_action(cx, action);
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_wip_menu(),
            _ => {}
        }
    } else if let (Some(HitTarget::PaneContextMenuItem(idx)), Some(_)) =
        (&hit, cx.open_menu.pane_menu())
    {
        let idx = *idx;
        let outcome = cx
            .open_menu
            .pane_menu_mut()
            .map(|(_, menu)| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                if let Some((target, _)) = cx.open_menu.take_pane_menu() {
                    commands::apply_pane_action(cx, target, action);
                }
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_pane_menu(),
            _ => {}
        }
    } else if let (Some(HitTarget::DiffHunkContextMenuItem(idx)), Some(_)) =
        (&hit, cx.open_menu.diff_hunk_menu())
    {
        let idx = *idx;
        let outcome = cx
            .open_menu
            .diff_hunk_menu_mut()
            .map(|(_, menu)| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(DiffHunkContextAction::Copy(what))) => {
                cx.open_menu.close_diff_hunk_menu();
                *cx.last_message = copy_from_diff(
                    &*cx.diff_view_state,
                    &*cx.diff_render,
                    what,
                    &mut *cx.clipboard,
                );
            }
            Some(MenuOutcome::Run(action)) => {
                if let Some((target, _)) = cx.open_menu.take_diff_hunk_menu() {
                    if let Some(s) = cx.diff_view_state.as_mut() {
                        apply_diff_hunk_context_action(
                            action,
                            target,
                            s,
                            &*cx.diff_rows,
                            cx.refs_result.as_deref().unwrap_or(&[]),
                            cx.link,
                            &mut *cx.file_insight,
                            &mut *cx.last_message,
                            &mut *cx.input_mode,
                        );
                    }
                }
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_diff_hunk_menu(),
            _ => {}
        }
    } else if let (Some(HitTarget::HistoryContextMenuItem(idx)), Some(_)) =
        (&hit, cx.open_menu.history_menu())
    {
        let idx = *idx;
        let outcome = cx
            .open_menu
            .history_menu_mut()
            .map(|(_, menu)| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                cx.open_menu.close_history_menu();
                commands::history::run_action(cx, action);
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_history_menu(),
            _ => {}
        }
    } else if let (
        Some(HitTarget::CommitColumn(crate::commit_column::ColumnHit::ColumnsMenuItem(idx))),
        true,
    ) = (&hit, cx.open_menu.is_commit_columns_menu())
    {
        let idx = *idx;
        let outcome = cx
            .open_menu
            .commit_columns_menu_mut()
            .map(|menu| menu.activate(idx));
        match outcome {
            Some(MenuOutcome::Run(act)) => {
                cx.open_menu.close_commit_columns_menu();
                commands::commit_column::run_columns_menu(cx, act);
            }
            Some(MenuOutcome::Close) => cx.open_menu.close_commit_columns_menu(),
            _ => {}
        }
    } else if matches!(
        (&hit, cx.open_menu.is_commit_columns_menu()),
        (
            Some(HitTarget::CommitColumn(
                crate::commit_column::ColumnHit::ColumnsMenuClose
            )),
            true
        )
    ) {
        cx.open_menu.close_commit_columns_menu();
    } else if let Some(HitTarget::Hosting(target)) = hit.clone() {
        match target {
            HostingHit::MenuItem(index) => {
                // A grayed-out row says why, in full, on the status line.
                if let Some(note) = cx
                    .hosting
                    .menu
                    .as_ref()
                    .and_then(|menu| menu.disabled_note(index))
                    .map(str::to_string)
                {
                    commands::apply_hosting_effect(cx, hosting_ui::HostingEffect::Message(note));
                } else {
                    let outcome = cx.hosting.menu.as_mut().map(|menu| menu.activate(index));
                    match outcome {
                        Some(MenuOutcome::Run(action)) => {
                            let effect = cx.hosting.run(action, &mut *cx.clipboard);
                            commands::apply_hosting_effect(cx, effect);
                        }
                        Some(MenuOutcome::Close) => cx.hosting.menu = None,
                        _ => {}
                    }
                }
            }
            HostingHit::PickerItem(index) => {
                let effect = cx.hosting.picker_click(index);
                commands::apply_hosting_effect(cx, effect);
            }
            HostingHit::EditRow(index) => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = SidebarSection::PullRequests;
                cx.sidebar_states
                    .select(SidebarSection::PullRequests, Some(index));
                let effect = cx.hosting.edit_row(index, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
            HostingHit::NewPr => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = SidebarSection::PullRequests;
                let effect = cx.hosting.run_command(
                    HostingCommand::CreatePullRequest,
                    (0, 0),
                    &mut *cx.clipboard,
                );
                commands::apply_hosting_effect(cx, effect);
            }
            HostingHit::NewIssue => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = SidebarSection::Issues;
                let effect =
                    cx.hosting
                        .run_command(HostingCommand::CreateIssue, (0, 0), &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
            HostingHit::Board => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = SidebarSection::Issues;
                let effect =
                    cx.hosting
                        .run_command(HostingCommand::OpenBoard, (0, 0), &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
            HostingHit::Row(list, index) => {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = list.section();
                cx.sidebar_states.select(list.section(), Some(index));
                cx.hosting.editing = None;
                let now = Instant::now();
                let is_double_click = cx.last_click.is_some_and(|(t, c, r)| {
                    now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
                });
                *cx.last_click = Some((now, col, row));
                // The controls (Reconnect, the search box,
                // the dropdowns, a group's header) act on
                // one click; the items open on a double.
                let is_button = cx
                    .hosting
                    .rows(list)
                    .get(index)
                    .is_some_and(hosting_ui::HostingRow::is_control);
                if is_double_click || is_button {
                    *cx.last_click = None;
                    let effect = cx.hosting.activate_row(list, index, &mut *cx.clipboard);
                    commands::apply_hosting_effect(cx, effect);
                }
            }
            HostingHit::LinkedPrChip | HostingHit::NewPrChip => {
                let effect = cx.hosting.click_chip(&target, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
        }
    } else if cx.open_menu.is_context_menu() {
        // Any other click closes the menu without
        // toggling -- click-away dismiss, same
        // convention as the ref-reveal popup.
        cx.open_menu.close();
    } else if cx.open_menu.is_column_menu() {
        // The menu is modal-ish while open: a click
        // on one of its rows toggles that column
        // (menu stays open, checklist convention);
        // a click anywhere else closes it instead
        // of falling through to whatever's
        // underneath.
        match &hit {
            Some(HitTarget::ColumnMenuItem(col)) => {
                if let Some(i) = LogColumn::ALL.iter().position(|c| c == col) {
                    if let Some(menu) = cx.open_menu.column_menu_mut() {
                        menu.cursor = i;
                    }
                }
                commands::log_columns::run_chooser_row(cx, ChooserRow::Column(*col));
            }
            Some(HitTarget::ColumnMenuAction(row)) => {
                commands::log_columns::run_chooser_row(cx, *row);
            }
            _ => cx.open_menu.close_column_menu(),
        }
    } else if let Some(ResizeGrab::LogColumn(c)) = resize_grab {
        let now = Instant::now();
        let double = cx.last_click.is_some_and(|(t, lc, lr)| {
            now.duration_since(t) < DOUBLE_CLICK_WINDOW && lc == col && lr == row
        });
        if double {
            // A second press on the same grip fits the column to its content
            // and starts no drag: the first press's release already ended its
            // own. The light follows the boundary's new place, and only when
            // the pointer is still on it.
            *cx.dragging_log_boundary = None;
            *cx.last_click = None;
            *cx.last_message = Some(commands::log_columns::fit_one(cx, c));
            *cx.hovered_resize_strip = cx
                .log_geo()
                .grip_rect(*cx.log_area, c)
                .filter(|r| r.contains((col, row).into()));
        } else {
            *cx.last_click = Some((now, col, row));
            // The `scroll` layout: pin every column where it is, so the drag
            // changes this one and nothing else moves.
            commands::log_columns::pin_scroll(cx);
            let geo = cx.log_geo();
            *cx.dragging_log_boundary = Some(LogBoundaryDrag {
                col: c,
                press_x: col,
                start_w: geo.widths.width_of(c),
                last: geo.order.last() == Some(&c),
            });
        }
    } else if let Some(HitTarget::LogAuthorFilter) = &hit {
        commands::run_command(cx, PaletteCommand::FilterLogByAuthor);
    } else if let Some(HitTarget::LogColumnHeader(header)) = &hit {
        *cx.dragging_log_header = Some(LogHeaderDrag::armed(*header, (col, row)));
    } else if let Some(HitTarget::TooltipRefPill { name, kind }) = &hit {
        // Checked first: the popup overlays
        // whatever pane is underneath it, so its
        // hit targets must win over Log/Refs row
        // clicks at the same screen position.
        let now = Instant::now();
        let is_double_click = cx.last_click.is_some_and(|(t, c, r)| {
            now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
        });
        *cx.last_click = Some((now, col, row));
        *cx.ref_drag = Some(RefDrag::armed_at(name.clone(), *kind, col, row));
        if is_double_click {
            match kind {
                RefRowKind::Local | RefRowKind::Tag => {
                    cx.link.submit_action(Action::CheckoutBranch(name.clone()));
                    *cx.last_message = Some(msg::input::checking_out(name));
                }
                RefRowKind::Remote => {
                    let msg = checkout_remote_pill(
                        name,
                        cx.ref_popup.as_ref().map(|p| p.commit_id),
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
            *cx.ref_popup = None;
        }
    } else if inside_ref_popup {
        // Landed inside the popup but not on a
        // specific ref row (its header line, or
        // padding) -- consume the click silently
        // rather than falling through to whatever
        // pane is underneath.
    } else if let Some(HitTarget::Merge(h)) = &hit {
        if let Some(view) = cx.merge.as_mut() {
            let out = view.click(*h, col, row);
            commands::apply_merge_outcome(cx, out);
        }
    } else if let Some(HitTarget::Editor(h)) = &hit {
        if let Some(ed) = cx.editor.as_mut() {
            let out = ed.click(*h);
            apply_editor_outcome(
                out,
                &mut *cx.editor,
                &mut *cx.file_insight,
                cx.link,
                &mut *cx.last_message,
                cx.diff_config_path,
            );
        }
    } else if let Some(HitTarget::CloseDiff) = &hit {
        // Exactly what Esc does, minus the
        // selection-clearing step: a click on a
        // close button means close, not "undo one
        // level first". The File View's own ✕ is
        // the same target: both
        // go back to the graph.
        *cx.central_view = CentralView::Log;
        *cx.diff_view_state = None;
        *cx.file_insight = None;
    } else if let Some(HitTarget::OpenRepositoryButton) = &hit {
        // The Repository Management tab. The same
        // call `Ctrl+O` and the palette's
        // `Repository: Open...` make, so the
        // three ways in cannot drift apart.
        // Already on it, a click is a no-op.
        go_to_management(ManagementEntry::Show, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();
    } else if let Some(HitTarget::NewTabButton) = &hit {
        // The `+` menu (New tab, Radar, Workspaces), under the `+`; the
        // palette's `Tab: New...` opens the same one.
        let at = cx
            .hit_targets
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::NewTabButton))
            .map_or((col, row + 1), |(r, _)| (r.x, r.y + 1));
        open_new_menu(at, &mut tab_ctx!(cx));
    } else if let Some(HitTarget::TabOverflow) = &hit {
        // The same call `t` and `Repository: Select Tab...` make.
        commands::run_command(cx, PaletteCommand::SelectTab);
    } else if let Some(HitTarget::TabScrollLeft) = &hit {
        // One tab per press; a held press repeats (`scrollbar_repeat`).
        crate::tab_scroll::scroll_by(-1);
    } else if let Some(HitTarget::TabScrollRight) = &hit {
        crate::tab_scroll::scroll_by(1);
    } else if let Some(HitTarget::RadarClose) = &hit {
        close_radar_tab(&mut tab_ctx!(cx));
    } else if let Some(HitTarget::WorkspacesClose) = &hit {
        close_workspaces_tab(&mut tab_ctx!(cx));
    } else if let Some(HitTarget::WorkspacesTab) = &hit {
        // The same call `Alt+J` and `Workspace: Manage...` make.
        go_to_management(ManagementEntry::Workspaces, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();
    } else if let Some(HitTarget::NotesClose) = &hit {
        close_notes_tab(&mut tab_ctx!(cx));
    } else if let Some(HitTarget::RadarTab) = &hit {
        // The same call `Alt+R` and `Radar: Open` make.
        go_to_management(ManagementEntry::Radar, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();
    } else if let Some(HitTarget::GlobalSearchButton) = &hit {
        // The same call `Ctrl+Alt+F` and `Search: Workspace...` make.
        commands::run_command(cx, PaletteCommand::WorkspaceSearch);
    } else if let Some(HitTarget::TabDir(crate::tab_dir::TabDirButton::Terminal)) = &hit {
        // "Open in": a click always opens the list (the palette's
        // `Tab: Open in...` does the same).
        crate::open_in_bar::open();
    } else if let Some(HitTarget::TabDir(button)) = &hit {
        // The same call the palette's `Tab: Quit and cd here` makes.
        let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
        match crate::tab_dir::run(*button, active.as_deref()) {
            crate::tab_dir::Outcome::Message(message) => *cx.last_message = Some(message),
            crate::tab_dir::Outcome::QuitNow => {
                *cx.should_quit = true;
                *cx.quit_how = Some(crate::tab_dir::QUIT_CD_HOW);
            }
        }
    } else if let Some(HitTarget::About) = &hit {
        crate::about::open();
    } else if let Some(HitTarget::ProfileField) = &hit {
        crate::profile_bar::open();
    } else if let Some(HitTarget::WorkspaceChip) = &hit {
        // The same combo `Alt+W` and `Workspace: Switch...` open.
        crate::workspace_ui::chip::open();
    } else if let Some(HitTarget::Gear) = &hit {
        // The gear. Same one line the palette's
        // `App: Settings...` and `Ctrl+,`
        // run, so the three ways in cannot
        // drift apart.
        *cx.preferences = Some(PreferencesState::open(cx.repo_root));
    } else if let Some(HitTarget::CloseApp) = &hit {
        // Exactly what `q` does: no separate confirmation path.
        *cx.should_quit = true;
        *cx.quit_how = Some("the window's close button");
    } else if let Some(HitTarget::RebaseBanner(button)) = &hit {
        match button {
            RebaseBannerButton::Continue => {
                cx.link.submit_action(Action::RebaseContinue);
                *cx.last_message = Some(msg::input::CONTINUING_REBASE.get().into());
            }
            RebaseBannerButton::Skip => {
                cx.link.submit_action(Action::RebaseSkip);
                *cx.last_message = Some(msg::input::SKIPPING.get().into());
            }
            RebaseBannerButton::Resolve => {
                commands::run_command(cx, PaletteCommand::OpenMergeEditor);
            }
            RebaseBannerButton::Abort => {
                *cx.input_mode = InputMode::Confirm {
                    prompt: msg::input::ABORT_REBASE_CONFIRM.get().to_string(),
                    action: ConfirmAction::Destructive(PendingDestructive::AbortRebase),
                };
            }
        }
    } else if let Some(HitTarget::MergeEditorButton) = &hit {
        commands::run_command(cx, PaletteCommand::OpenMergeEditor);
    } else if let Some(HitTarget::OperationBanner(button)) = &hit {
        use crate::operation_banner::OperationBannerButton as B;
        match button {
            B::Resolve => {
                commands::run_command(cx, PaletteCommand::OpenMergeEditor);
            }
            // Committing concludes a merge, a cherry-pick and a revert
            // alike: the commit message field takes it from here.
            B::Commit => {
                commands::run_command(cx, PaletteCommand::EditWipMessage);
            }
            B::Skip => {
                commands::run_command(cx, PaletteCommand::SkipOperation);
            }
            B::Abort => {
                commands::run_command(cx, PaletteCommand::AbortOperation);
            }
        }
    } else if let Some(HitTarget::StuckBanner(button)) = &hit {
        use crate::stuck_banner::StuckBannerButton as B;
        // Every button is its palette command's twin.
        let cmd = match button {
            B::AmResolve => PaletteCommand::OpenMergeEditor,
            B::AmContinue => PaletteCommand::AmContinue,
            B::AmSkip => PaletteCommand::AmSkip,
            B::AmAbort => PaletteCommand::AmAbort,
            B::RemoveLock => PaletteCommand::IndexLockRemove,
        };
        commands::run_command(cx, cmd);
    } else if let Some(HitTarget::AutoStashBanner(button)) = &hit {
        let cmd = match button {
            AutoStashBannerButton::Resolve => PaletteCommand::OpenMergeEditor,
            AutoStashBannerButton::Pop => PaletteCommand::AutoStashPop,
            AutoStashBannerButton::Finish => PaletteCommand::AutoStashFinish,
            AutoStashBannerButton::Show => PaletteCommand::AutoStashShow,
            AutoStashBannerButton::Drop => PaletteCommand::AutoStashDrop,
            AutoStashBannerButton::StartOver => PaletteCommand::AutoStashStartOver,
        };
        commands::run_command(cx, cmd);
    } else if let Some(HitTarget::PushGroupStrip(button)) = &hit {
        let effect = crate::push_group_ui::strip_click(cx.repo_root, *button);
        crate::push_group_ui::apply(effect, cx.link, &mut *cx.input_mode);
    } else if let Some(HitTarget::SequenceStrip(button)) = &hit {
        use crate::sequence_run::strip::StripButton as B;
        // Every button is its palette command's twin.
        match button {
            B::Continue => {
                commands::run_command(cx, PaletteCommand::SequenceContinue);
            }
            B::Close => {
                commands::run_command(cx, PaletteCommand::SequenceCancel);
            }
            B::GoThere => crate::sequence_run::request_go_there(),
            B::Details => {
                commands::run_command(cx, PaletteCommand::OpenActivityLog);
            }
        }
    } else if let Some(HitTarget::ConflictBadge) = &hit {
        commands::run_command(cx, PaletteCommand::ConflictsShow);
    } else if let Some(HitTarget::Toolbar(button)) = &hit {
        // The button's palette command: the button, the
        // chord and the palette cannot drift.
        let cmd = button.command();
        commands::run_command(cx, cmd);
    } else if matches!(resize_grab, Some(ResizeGrab::CommitColumn)) {
        cx.commit_col.dragging = true;
    } else if let Some(ResizeGrab::CommitColumnPart(boundary)) = resize_grab {
        let now = Instant::now();
        let double = cx.last_click.is_some_and(|(t, lc, lr)| {
            now.duration_since(t) < DOUBLE_CLICK_WINDOW && lc == col && lr == row
        });
        if double {
            // The Log's rule: a second press on the same grip fits the part
            // to its content and starts no drag.
            cx.commit_col.release_for_fit();
            *cx.last_click = None;
            commands::commit_column::fit_one(cx, boundary);
        } else {
            *cx.last_click = Some((now, col, row));
            cx.commit_col.grab_boundary(boundary);
        }
    } else if matches!(resize_grab, Some(ResizeGrab::HistorySplit)) {
        if let Some(h) = cx.history.as_mut() {
            h.dragging_split = true;
        }
    } else if matches!(resize_grab, Some(ResizeGrab::CommitMessage)) {
        *cx.dragging_commit_message = true;
    } else if let Some(ResizeGrab::SidebarSection(section)) = resize_grab {
        *cx.dragging_sidebar_section = Some(section);
        *cx.focus = Focus::Refs;
        *cx.active_ref_section = section;
    } else if let Some(ResizeGrab::PaneDivider(which)) = resize_grab {
        *cx.dragging_divider = Some(which);
    } else if matches!(&hit, Some(HitTarget::CollapsePane(PaneSide::Refs))) {
        *cx.left_pane_collapsed = true;
        if *cx.focus == Focus::Refs {
            *cx.focus = Focus::Log;
        }
        persist_pane_layout(
            cx.session,
            *cx.divider1_pct,
            *cx.divider2_pct,
            cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
            cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
        );
    } else if matches!(&hit, Some(HitTarget::CollapsePane(PaneSide::Status))) {
        *cx.right_pane_collapsed = true;
        if *cx.focus == Focus::Status {
            *cx.focus = Focus::Log;
        }
        persist_pane_layout(
            cx.session,
            *cx.divider1_pct,
            *cx.divider2_pct,
            cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
            cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
        );
    } else if matches!(&hit, Some(HitTarget::RestorePane(PaneSide::Refs))) {
        *cx.left_pane_collapsed = false;
        persist_pane_layout(
            cx.session,
            *cx.divider1_pct,
            *cx.divider2_pct,
            cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
            cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
        );
    } else if matches!(&hit, Some(HitTarget::RestorePane(PaneSide::Status))) {
        *cx.right_pane_collapsed = false;
        persist_pane_layout(
            cx.session,
            *cx.divider1_pct,
            *cx.divider2_pct,
            cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
            cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
        );
    } else if matches!(&hit, Some(HitTarget::StopSoloing)) {
        *cx.focus = Focus::Refs;
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
    } else if matches!(&hit, Some(HitTarget::ShowAllHiddenRefs)) {
        *cx.focus = Focus::Refs;
        cx.hidden_from_graph.clear();
        refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
        *cx.last_message = Some(msg::input::HIDDEN_REFS_SHOWN.get().to_string());
    } else if let Some(HitTarget::RefsSectionHeader(kind)) = &hit {
        // Mouse equivalent of Enter/Space on the
        // active section -- clicking a header both
        // makes that section active and toggles
        // its collapsed state.
        *cx.focus = Focus::Refs;
        *cx.active_ref_section = *kind;
        let name = kind.title();
        if !cx.collapsed_ref_sections.remove(name) {
            cx.collapsed_ref_sections.insert(name.to_string());
        }
    } else {
        return Ok(Flow::Continue);
    }
    Ok(Flow::Handled)
}
