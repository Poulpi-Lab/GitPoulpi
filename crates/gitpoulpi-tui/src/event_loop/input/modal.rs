//! The modal guard chain: the popups, dialogs and overlays that consume a
//! key before any input mode or the Normal-mode keymap sees it. The order of
//! the checks below IS the precedence between surfaces.

use super::commands::{run_blame_op, CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use gitpoulpi_i18n::msg;

/// `Flow::Handled` when a surface consumed `key`, `Flow::Continue` to pass it on.
/// Settles what Copy as Mermaid gitGraph asked for: the text goes on the
/// clipboard through the event loop's own handle.
pub(super) fn copy_mermaid_outcome(
    cx: &mut CommandCtx<'_>,
    outcome: crate::mermaid_popup::Outcome,
) {
    match outcome {
        crate::mermaid_popup::Outcome::Copy(text) => {
            *cx.last_message = Some(
                cx.clipboard
                    .copy(gitpoulpi_i18n::msg::mermaid::COPIED_WHAT.get(), &text),
            );
        }
        crate::mermaid_popup::Outcome::CopyDiagram(text) => {
            *cx.last_message = Some(cx.clipboard.copy(
                gitpoulpi_i18n::msg::mermaid::DIAGRAM_COPIED_WHAT.get(),
                &text,
            ));
        }
        crate::mermaid_popup::Outcome::None => {}
    }
}

/// The Settings screen was closed: drop it and apply what it changed.
pub(super) fn settings_closed(cx: &mut CommandCtx<'_>) {
    *cx.preferences = None;
    // `general.undo_enabled` may have flipped: the journal state (and the
    // toolbar) follows.
    cx.link.request_refresh();
    if let Some(load) = apply_live_settings(
        &mut *cx.last_message,
        &mut *cx.hierarchical_refs,
        &mut *cx.graph_show_stashes,
        &mut *cx.ref_connector,
        &mut *cx.avatar_placement,
        &mut *cx.avatar_live,
        &mut *cx.ui_lane_background,
        &mut *cx.ui_graph_theme,
        &mut *cx.ui_tint_strength,
    ) {
        cx.lazy_log.apply_settings(load);
    }
}

pub(super) fn handle(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    // "What is sent" before an AI request owns the keyboard, ahead of every
    // mode: the request that asked can come from a form (the commit message
    // box) or the Explain popup, which are not `Normal`.
    if key.kind == KeyEventKind::Press && crate::ai_sent_dialog::is_open() {
        crate::ai_sent_dialog::on_key(key);
        return Ok(Flow::Handled);
    }
    // "Over the AI limit" (an input would be cut): the same, asked first.
    if key.kind == KeyEventKind::Press && crate::ai_limit_dialog::is_open() {
        crate::ai_limit_dialog::on_key(key);
        super::commands::open_limit_setting(cx);
        return Ok(Flow::Handled);
    }

    // The steps of the repository's own sequences, before they are trusted.
    if key.kind == KeyEventKind::Press && crate::sequence_run::repo_review::is_open() {
        crate::sequence_run::repo_review::on_key(key);
        if let Some(said) = crate::sequence_run::repo_review::take_result() {
            *cx.last_message = Some(said);
        }
        return Ok(Flow::Handled);
    }

    // "Open this link?" (an address of a commit message): the same.
    if key.kind == KeyEventKind::Press && crate::link_confirm::is_open() {
        crate::link_confirm::on_key(key);
        take_link_result(cx);
        return Ok(Flow::Handled);
    }

    let km = env.km;
    // The integration security check owns the keyboard above everything: it
    // answers a change that was refused before anything was sent.
    if key.kind == KeyEventKind::Press && crate::trust_dialog::is_open() {
        if let Some(outcome) = crate::trust_dialog::on_key(key) {
            crate::trust_dialog::apply(
                outcome,
                cx.repo_root,
                &mut *cx.preferences,
                &mut *cx.last_message,
            );
        }
        return Ok(Flow::Handled);
    }
    // A ref drag in flight owns `Esc`, ahead of every mode and overlay below: it
    // is a gesture, not a mode, and backing out of one must not also close
    // whatever is behind it. Its own guarded arm rather than a branch in the
    // dispatch, like the help and File Insight arms: precedence between
    // surfaces is the order of these arms, where it can be read off.
    // Same for a tab being carried to a new place: Esc drops the gesture and
    // leaves the order as it was (the picker stays open under a picker drag).
    // A column header being carried (the Log's, the commit column's): Esc
    // lets it go where it was and moves nothing.
    if key.kind == KeyEventKind::Press
        && key.code == KeyCode::Esc
        && (cx.dragging_log_header.is_some_and(|d| d.started)
            || cx.commit_col.header_drag_at.is_some())
    {
        let title = match (*cx.dragging_log_header, cx.commit_col.dragging_header) {
            (Some(d), _) => d.col.title(),
            (None, Some(part)) => part.title(),
            (None, None) => "",
        };
        *cx.last_message = Some(msg::input::drag_canceled(title));
        *cx.dragging_log_header = None;
        cx.commit_col.cancel_header_drag();
        if *cx.last_cursor_shape == CursorShape::Grab {
            cursor::emit_cursor_shape(cx.terminal.backend_mut(), CursorShape::Default)
                .map_err(exit_log::io_context("setting the pointer shape"))?;
            *cx.last_cursor_shape = CursorShape::Default;
        }
        return Ok(Flow::Handled);
    }
    if key.kind == KeyEventKind::Press
        && key.code == KeyCode::Esc
        && cx.tab_drag.as_ref().is_some_and(|d| d.started)
    {
        if let Some(drag) = cx.tab_drag.take() {
            *cx.last_message = Some(msg::input::drag_canceled(&drag.name));
        }
        return Ok(Flow::Handled);
    }
    if key.kind == KeyEventKind::Press
        && key.code == KeyCode::Esc
        && cx.ref_drag.as_ref().is_some_and(|d| d.started)
    {
        let Some(drag) = cx.ref_drag.take() else {
            return Ok(Flow::Handled);
        };
        *cx.last_message = Some(msg::input::drag_canceled(&drag.source));
        // A keystroke carries no mouse position and the mouse
        // will report no motion of its own for this, so the
        // shape is restored from where the drag last saw the
        // pointer -- otherwise the open hand stays up over a
        // ref nobody is carrying until the user jiggles the
        // mouse.
        let (dcol, drow) = drag.at;
        let (divider1_x, divider2_x) = pane_divider_xs(
            *cx.panes_area,
            *cx.divider1_pct,
            *cx.divider2_pct,
            *cx.left_pane_collapsed,
            *cx.right_pane_collapsed,
        );
        *cx.hovered_ref_row = sidebar_ref_at(
            &*cx.sidebar_section_areas,
            cx.refs_result.as_deref().unwrap_or(&[]),
            &*cx.refs_filter,
            *cx.hierarchical_refs,
            &*cx.collapsed_ref_folders,
            &*cx.remote_names,
            |s| cx.sidebar_states.state(s).offset(),
            dcol,
            drow,
        )
        .map(|h| h.name);
        let shape = desired_cursor_shape(
            find_hit_target(&*cx.hit_targets, dcol, drow).as_ref(),
            cx.hovered_ref_row.as_deref(),
            None,
            None,
            false,
            resize_grab_at(
                &*cx.hit_targets,
                dcol,
                drow,
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

        return Ok(Flow::Handled);
    }

    // The Preferences screen owns the keyboard outright while it is open,
    // ahead of the help overlay and of `input_mode`: it is the topmost surface
    // and a key reaching what is underneath would act on a pane the user cannot
    // see. It reports `false` when it wants to close (Esc / q). `?` is let
    // through (`PreferencesState::passes_through`, unless a field is being typed
    // into) so the help overlay opens on top of it, as on the Repository
    // Management tab. `&& !show_help`: once the overlay is open it is the topmost
    // surface and any key must close it (the arm below), not drive Preferences
    // beneath it.
    // `&& Normal && !about`: a popup opened over Settings (the palette, a
    // picker, the workspace search, About) owns the keys while it is open.
    if key.kind == KeyEventKind::Press
        && !*cx.show_help
        && matches!(*cx.input_mode, InputMode::Normal)
        && !crate::about::is_open()
        && cx
            .preferences
            .as_ref()
            .is_some_and(|prefs| !prefs.passes_through(&key))
    {
        let Some(prefs) = cx.preferences.as_mut() else {
            return Ok(Flow::Handled);
        };
        let stay_open = prefs.on_key(key);
        if let Some(message) = prefs.take_message() {
            *cx.last_message = Some(message);
        }
        if !stay_open {
            // Settings the TUI itself reads are republished
            // on close as well as on commit, so a value typed
            // and then navigated away from still applies.
            settings_closed(cx);
        }

        return Ok(Flow::Handled);
    }

    // A binary file's "Open in" popup, the same way.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::open_with::popup::is_open()
    {
        if let Some((pick, subject)) = crate::open_with::popup::on_key(key) {
            *cx.last_message = Some(crate::open_with::popup::apply(
                pick,
                &subject,
                cx.repo_root,
                cx.link,
            ));
        }

        return Ok(Flow::Handled);
    }

    // The Diff view's encoding dropdown, the same way.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::diff_encoding_menu::is_open()
    {
        if let Some(pick) = crate::diff_encoding_menu::on_key(key) {
            commands::apply_diff_encoding_pick(cx, pick);
        }

        return Ok(Flow::Handled);
    }

    // The image header's "Edit in" popup, the same way.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::image_view::edit_in::is_open()
    {
        if let Some(pick) = crate::image_view::edit_in::on_key(key) {
            commands::apply_image_edit_pick(cx, pick);
        }

        return Ok(Flow::Handled);
    }

    // The About window and the profile popup own the keyboard while open.
    if key.kind == KeyEventKind::Press && matches!(*cx.input_mode, InputMode::Normal) {
        if crate::about::is_open() {
            crate::about::on_key(key);
            return Ok(Flow::Handled);
        }
        if crate::profile_bar::is_open() {
            if let Some(id) = crate::profile_bar::on_key(key) {
                commands::apply_profile_pick(cx, id);
            }
            return Ok(Flow::Handled);
        }
        if crate::workspace_ui::chip::is_open() {
            if let Some(pick) = crate::workspace_ui::chip::on_key(key) {
                commands::apply_workspace_pick(cx, pick);
            }
            return Ok(Flow::Handled);
        }
    }

    // The tab bar's "Open in" popup owns the keyboard while open (Up / Down /
    // Enter / Esc, typing searches a long list).
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::open_in_bar::is_open()
    {
        if let Some(pick) = crate::open_in_bar::on_key(key) {
            commands::apply_open_in_pick(cx, pick);
        }

        return Ok(Flow::Handled);
    }

    // Copy as Mermaid gitGraph owns the keyboard while open.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::mermaid_popup::is_open()
    {
        let outcome = crate::mermaid_popup::on_key(key, cx.diff_config_path);
        copy_mermaid_outcome(cx, outcome);
        return Ok(Flow::Handled);
    }

    // The predicted conflicts popup owns the keyboard while open.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::conflict_popup::is_open()
    {
        if let Some(action) = crate::conflict_popup::on_key(key) {
            crate::conflict_badge::apply_popup_action(
                action,
                &*cx.refs_result,
                cx.link,
                &mut *cx.last_message,
            );
        }
        return Ok(Flow::Handled);
    }

    // The merge-revert parent dropdown owns the keyboard while open.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::revert_mainline::is_open()
    {
        if let Some(pick) = crate::revert_mainline::on_key(key) {
            crate::log_menu::apply_revert_mainline(pick, &mut *cx.input_mode);
        }
        return Ok(Flow::Handled);
    }

    // The "Pull all branches" worktree question owns the keyboard while open.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::branch_sync_dialog::is_open()
    {
        crate::branch_sync_dialog::on_key(key, cx.link);
        return Ok(Flow::Handled);
    }

    // The tag delete scope dialog owns the keyboard while open.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::tag_delete_dialog::is_open()
    {
        if let Some(pick) = crate::tag_delete_dialog::on_key(key) {
            crate::tag_delete_dialog::apply(
                pick,
                cx.link,
                &mut *cx.input_mode,
                &mut *cx.last_message,
            );
        }
        return Ok(Flow::Handled);
    }

    // The sensitive-files question hands the keyboard to the palette (its
    // "Show in File" is a palette row), which then owns it until it closes.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::stage_guard_dialog::is_open()
        && km.is(Bind::GlobalPalette, &key)
    {
        *cx.input_mode = InputMode::CommandPalette {
            draft: String::new(),
            selected: 0,
        };
        return Ok(Flow::Handled);
    }
    // The sensitive-files question of a refused stage owns the keyboard.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::stage_guard_dialog::is_open()
    {
        if let Some(staged) = crate::stage_guard_dialog::on_key(key) {
            crate::stage_guard_dialog::apply(staged, cx.link, cx.repo_root);
        }
        return Ok(Flow::Handled);
    }

    // The grouped push dialog owns the keyboard.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && crate::push_group_ui::is_open()
    {
        let effect = crate::push_group_ui::on_key(key);
        crate::push_group_ui::apply(effect, cx.link, &mut *cx.input_mode);
        return Ok(Flow::Handled);
    }

    // The repository menu (tab menu, management row menu)
    // owns the keyboard while open, swallowing what it does
    // not use -- the rule every context menu here follows.
    if key.kind == KeyEventKind::Press
        && cx.repo_menu.is_some()
        && matches!(*cx.input_mode, InputMode::Normal)
    {
        let outcome = cx
            .repo_menu
            .as_mut()
            .map(|state| state.menu.on_key(key.code));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                if let Some(state) = cx.repo_menu.take() {
                    if let Some(message) =
                        apply_repo_menu_action(action, state.target, &mut tab_ctx!(cx))
                    {
                        *cx.last_message = Some(message.clone());
                        if let Some(m) = cx.management.as_mut() {
                            m.message = Some(message);
                        }
                    }
                }
            }
            Some(MenuOutcome::Close) => *cx.repo_menu = None,
            _ => {}
        }
        *cx.should_quit |=
            cx.switch_to.is_some() || *cx.close_window || cx.show_management.is_some();

        return Ok(Flow::Handled);
    }

    // The repository overlay owns the keyboard while it is open, ahead of every
    // mode and every other overlay: it is the only one that types free text *and*
    // can change which repository the window shows, so anything reaching past it
    // would steal a keystroke out of a URL or act on a repository the user is
    // leaving.
    // The Undo history panel hands the keyboard to the palette (its row actions
    // are palette rows), which then owns it until it closes.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && (cx.repo_dialogs.safety_wants_palette()
            || cx.repo_dialogs.review_wants_palette()
            || cx.repo_dialogs.board_wants_palette()
            || cx.repo_dialogs.item_wants_palette()
            || cx.repo_dialogs.composer_avail().is_some())
        && km.is(Bind::GlobalPalette, &key)
    {
        *cx.input_mode = InputMode::CommandPalette {
            draft: String::new(),
            selected: 0,
        };

        return Ok(Flow::Handled);
    }
    // `Ctrl+N` over a dialog that offers badges (its controls, see
    // `RepoDialogs::nav_targets`) starts the prefix; the key after it is the
    // badge (`keys_prefix::navigation`). A dialog without badges keeps the key.
    if key.kind == KeyEventKind::Press
        && km.is(Bind::GlobalNavPrefix, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.repo_dialogs.is_open()
        && !cx.repo_dialogs.nav_targets().is_empty()
    {
        *cx.input_mode = InputMode::Navigation;
        return Ok(Flow::Handled);
    }
    if key.kind == KeyEventKind::Press
        && cx.repo_dialogs.is_open()
        && !matches!(
            *cx.input_mode,
            InputMode::CommandPalette { .. } | InputMode::Navigation
        )
    {
        if let Some(effect) = cx.repo_dialogs.on_key(key) {
            super::menu_actions::apply_effect(cx, effect);
        }
        // Closing the form cancels a generation still running.
        if !cx.repo_dialogs.pull_request_open() && cx.ai_ui.running_pull_request() {
            cx.ai_ui.cancel();
        }
        // Closing the dialog stops a naming still running.
        if !cx.repo_dialogs.worktree_open() && cx.ai_ui.running_worktree_name() {
            cx.ai_ui.cancel();
        }
        if !cx.repo_dialogs.is_open() {
            refresh_management(cx.session, &mut *cx.management);
        }
        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;

        return Ok(Flow::Handled);
    }

    // The bindable keyboard twins of the tab bar's "Open in" button: the list
    // (focused) and the default entry, run straight away. Default first and both before
    // Home: `Alt+O` and `Ctrl+O` also match `Ctrl+Alt+O` (lenient modifiers).
    if matches!(*cx.input_mode, InputMode::Normal) && cx.management.is_none() {
        if km.is(Bind::TabOpenInDefault, &key) {
            let active = cx.repo_tabs.get(*cx.active_tab).map(|t| t.path.clone());
            *cx.last_message = Some(match active {
                Some(path) => crate::open_in_bar::run_default(&path),
                None => msg::tab_dir::NO_TAB.get().into(),
            });

            return Ok(Flow::Handled);
        }
        if km.is(Bind::TabOpenInPopup, &key) {
            crate::open_in_bar::open();

            return Ok(Flow::Handled);
        }
    }

    // The Repository Management tab, from anywhere that is
    // not already typing into something. `Ctrl+O` is *open*
    // in every editor there is; `Ctrl+Shift+N` goes there
    // with the clone form open. Both also exist as palette
    // entries, which is architecture principle 5's escape
    // hatch. On the management tab itself, `Ctrl+O` puts
    // the cursor in the list's search; with the Radar or the
    // release notes open it first closes them (the Radar has
    // its own `/` and `Ctrl+F`, so `Ctrl+O` never duplicates them).
    if km.is(Bind::GlobalHome, &key) && matches!(*cx.input_mode, InputMode::Normal) {
        match cx.management.as_mut() {
            // Same as the Radar's own Esc (`Outcome::Close`): dropping the
            // view drops its popups too.
            Some(manager) if manager.radar.is_open() => {
                manager.radar.close();
                manager.search_focused = true;
            }
            // The search box is on the list: leave the notes for it.
            Some(manager) if manager.notes.is_open() => {
                manager.notes.close();
                manager.search_focused = true;
            }
            Some(manager) => manager.search_focused = true,
            None => go_to_management(ManagementEntry::Show, &mut tab_ctx!(cx)),
        }

        return Ok(Flow::Handled);
    }

    // `Alt+W`: the workspace combo (the tab bar's chip).
    if km.is(Bind::GlobalWorkspaceSwitch, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
    {
        if crate::workspace_ui::chip::visible() {
            crate::workspace_ui::chip::open();
        } else {
            *cx.last_message = Some(gitpoulpi_i18n::msg::workspaces::WHY_NONE_YET.get().into());
        }
        return Ok(Flow::Handled);
    }

    // `Alt+T`: a transient New tab (the tab bar's `+`).
    if km.is(Bind::GlobalNewTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
    {
        go_to_management(ManagementEntry::NewTab, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();

        return Ok(Flow::Handled);
    }

    // `Ctrl+W` on a New tab closes it, with no question: nothing is open in it.
    if km.is(Bind::GlobalCloseTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        && cx.management.as_ref().is_some_and(|m| m.new_tab)
    {
        if let Some(message) = close_new_tab(&mut tab_ctx!(cx)) {
            *cx.last_message = Some(message);
        }
        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;

        return Ok(Flow::Handled);
    }

    // `Alt+R`: the Radar. Opens in place on the Home tab; from a
    // repository window the window ends and the Home window starts on it.
    if km.is(Bind::GlobalRadar, &key)
        && crate::build_features::offered(Some(crate::build_features::Feature::Radar))
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        // `Alt+R` is also `merge.reset_all` and `composer.insert_issue_ref`:
        // inside the merge editor and the commit message composer the
        // context wins (the Radar stays in the palette there).
        && cx.composer_focus.is_none()
        && !cx.merge.as_ref().is_some_and(|m| !m.passes_through(&key))
    {
        go_to_management(ManagementEntry::Radar, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();

        return Ok(Flow::Handled);
    }

    // `Alt+J`: the Workspaces tab. Opens in place on the Home window; from
    // a repository window the window ends and the Home window starts on it.
    if km.is(Bind::GlobalWorkspaces, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
    {
        go_to_management(ManagementEntry::Workspaces, &mut tab_ctx!(cx));
        *cx.should_quit |= cx.show_management.is_some();

        return Ok(Flow::Handled);
    }

    if km.is(Bind::GlobalClone, &key) && matches!(*cx.input_mode, InputMode::Normal) {
        go_to_management(ManagementEntry::Clone, &mut tab_ctx!(cx));

        return Ok(Flow::Handled);
    }

    // `Ctrl+T` opens the current tab's menu, the keyboard half of
    // right-clicking a tab, anchored under the tab itself.
    if km.is(Bind::GlobalTabMenu, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.management.is_none()
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

        return Ok(Flow::Handled);
    }

    // `Ctrl+W` on the Release notes tab closes it, with no question.
    if km.is(Bind::GlobalCloseTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        && cx.management.as_ref().is_some_and(|m| m.notes.is_open())
    {
        close_notes_tab(&mut tab_ctx!(cx));

        return Ok(Flow::Handled);
    }

    // `Ctrl+W` on the Workspaces tab closes it, with no question.
    if km.is(Bind::GlobalCloseTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        && cx
            .management
            .as_ref()
            .is_some_and(|m| m.page == crate::repo_manager::HomePage::Workspaces)
    {
        close_workspaces_tab(&mut tab_ctx!(cx));

        return Ok(Flow::Handled);
    }

    // `Ctrl+W` on the Radar tab closes it, with no question.
    if km.is(Bind::GlobalCloseTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        && cx.management.as_ref().is_some_and(|m| m.radar.is_open())
    {
        close_radar_tab(&mut tab_ctx!(cx));

        return Ok(Flow::Handled);
    }

    // `Ctrl+W` closes the current tab, asking first like every other way to
    // close one. The palette advertises this shortcut.
    if km.is(Bind::GlobalCloseTab, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
        && cx.management.is_none()
    {
        match close_tab_prompt(&*cx.repo_tabs, *cx.active_tab) {
            Some(prompt) => {
                *cx.input_mode = InputMode::Confirm {
                    prompt,
                    action: ConfirmAction::CloseTab(*cx.active_tab),
                }
            }
            None => *cx.last_message = Some(msg::input::NO_REPO_TAB.get().into()),
        }

        return Ok(Flow::Handled);
    }

    // `Ctrl+PageDown`/`Ctrl+PageUp` move to the next/previous repository tab, through
    // `cycle_repository_tab` (the palette entry's own function, so the two can't
    // disagree). Guarded off while the overlay owns the keyboard, like every
    // other top-level chord here.
    if repo_tab_cycle_chord(&key).is_some()
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
    {
        let Some(forward) = repo_tab_cycle_chord(&key) else {
            return Ok(Flow::Handled);
        };
        // From the management tab (the leftmost), forward is
        // the first repository tab and backward the last.
        if cx.management.is_some() {
            if !cx.repo_tabs.is_empty() {
                let index = if forward { 0 } else { cx.repo_tabs.len() - 1 };
                *cx.last_message =
                    tab_ctx!(cx).switcher_action(repo_switcher::SwitcherAction::Focus(index));
            }
        } else if let Some(message) = cycle_repository_tab(
            forward,
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

        return Ok(Flow::Handled);
    }

    // `Ctrl+N` enters the navigation prefix (`InputMode::Navigation`): the next
    // key is looked up in `navigation_key_action` (`chrome.rs`): `h`/`l`
    // collapse/restore the Refs/Status side panes, `1`/`2`/`3` focus a sub-pane
    // (number badges are drawn while the mode is open), `,` opens settings, `?`
    // toggles help, anything else cancels.
    //
    // Why `Ctrl+N`: the other chords were confirmed dead against the pty probe
    // (`xtask::pty::keys::control`, `docs/developer/visual-verification.md`).
    // `Ctrl+[` is byte `0x1B`, identical to `Escape`; `Ctrl+]` arrives as
    // `Ctrl+5` (xterm's C0 table maps `0x1C..=0x1F` to `Char('4'..='7')`);
    // `Ctrl+H` is `0x08`, the same as `Backspace` (`Ctrl+I`/`J`/`M` likewise
    // twin Tab/Enter). `Ctrl+N` collides with nothing a terminal intercepts, and
    // its one other use (moving the palette selection down) lives in
    // `InputMode::CommandPalette`, guarded off here by
    // `matches!(input_mode, InputMode::Normal)`.
    if km.is(Bind::GlobalNavPrefix, &key)
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.repo_dialogs.is_open()
    {
        *cx.input_mode = InputMode::Navigation;

        return Ok(Flow::Handled);
    }

    // The AI explanation popup: read-only, on top of everything
    // (like help). Esc cancels a request still running.
    if key.kind == KeyEventKind::Press && cx.ai_ui.popup_open() {
        let Some(popup) = cx.ai_ui.popup_mut() else {
            return Ok(Flow::Handled);
        };
        let working = popup.is_working();
        match popup.on_key(&key) {
            PopupKey::Close => {
                if working {
                    cx.ai_ui.cancel();
                } else {
                    cx.ai_ui.close_popup();
                }
            }
            PopupKey::Copy(text) => {
                *cx.last_message =
                    Some(cx.clipboard.copy(msg::input::WHAT_EXPLANATION.get(), &text));
            }
            PopupKey::Nothing => {}
        }

        return Ok(Flow::Handled);
    }

    // The help overlay owns the keyboard while it is open, ahead of every mode and
    // every other overlay. It is filterable, typed like the palette's filter
    // (`j`/`q`/any letter included), so `Esc` closes it only once the filter is
    // empty (clear, then leave: the Refs filter's two-step); `?` always closes
    // outright.
    if key.kind == KeyEventKind::Press && *cx.show_help {
        let page = (*cx.help_popup_inner_height).max(1);
        let help_nav = crate::popup_list::nav(
            &key,
            usize::from(*cx.help_scroll),
            usize::from(u16::MAX),
            usize::from(page) + 1,
        );
        match key.code {
            // The dragged text: copied, or dropped first (the popup stays).
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(text) = crate::help_view::selected_text() {
                    *cx.last_message =
                        Some(cx.clipboard.copy(msg::input::WHAT_SELECTION.get(), &text));
                }
            }
            KeyCode::Esc if crate::help_view::text_clear() => {}
            _ if km.is(Bind::GlobalHelp, &key) => {
                *cx.show_help = false;
                cx.help_filter.clear();
                *cx.help_scroll = 0;
            }
            KeyCode::Esc => {
                if cx.help_filter.is_empty() {
                    *cx.show_help = false;
                } else {
                    cx.help_filter.clear();
                    *cx.help_scroll = 0;
                }
            }
            // Tabs: `Tab` / `Shift+Tab` and `Left` / `Right` step (wrapping),
            // `Alt+1`..`Alt+8` jump. The filter is typed text, so bare digits
            // and letters stay the filter's.
            KeyCode::Tab | KeyCode::Right if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                crate::help_view::step_tab(1);
                *cx.help_scroll = 0;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Tab => {
                crate::help_view::step_tab(-1);
                *cx.help_scroll = 0;
            }
            KeyCode::Char(c @ '1'..='9') if key.modifiers.contains(KeyModifiers::ALT) => {
                crate::help_view::set_tab(c as usize - '1' as usize);
                *cx.help_scroll = 0;
            }
            // Scrolling takes the popup lists' keys (`popup_list::nav`): the
            // scroll offset plays the highlight, the length is unbounded and the
            // draw clamps it.
            _ if help_nav.is_some() => *cx.help_scroll = help_nav.unwrap_or(0) as u16,
            // `Ctrl+D` (docs), the keyboard twin of clicking the docs link.
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                *cx.last_message = Some(crate::help_view::open_docs());
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                cx.help_filter.clear();
                *cx.help_scroll = 0;
            }
            KeyCode::Backspace => {
                cx.help_filter.pop();
                *cx.help_scroll = 0;
            }
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                cx.help_filter.push(c);
                *cx.help_scroll = 0;
            }
            _ => {}
        }

        return Ok(Flow::Handled);
    }

    // The Repository Management tab owns the keyboard, but
    // for the few window-level keys it passes on
    // (`repo_manager::passes_through`: the palette,
    // Preferences, help) -- which then reach the ordinary
    // dispatch below. Nothing it swallows can act on the
    // empty panes behind it.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx
            .management
            .as_ref()
            .is_some_and(|m| !repo_manager::passes_through(m, &key))
    {
        let Some(manager) = cx.management.as_mut() else {
            return Ok(Flow::Handled);
        };
        // The Release notes tab, when it shows over the list, owns every key.
        if manager.notes.is_open() {
            let event = manager.notes.on_key(key);
            commands::apply_notes_event(cx, event);
            return Ok(Flow::Handled);
        }
        // The Radar, when it shows over the list, owns every key.
        if manager.radar.is_open() {
            // The window's refresh key (F5 by default, rebindable) reloads
            // every Radar source, like `r` and the palette's Radar: Refresh.
            let outcome = if km.is(Bind::GlobalRefreshAll, &key) {
                manager.radar.command(crate::radar::RadarCommand::Refresh)
            } else {
                manager.radar.on_key(key)
            };
            commands::apply_radar_outcome(cx, outcome);
            return Ok(Flow::Handled);
        }
        // The search box's editing keys first:
        // caret, selection, Ctrl+A/U/K, Ctrl+C/X.
        let action = match manager.edit_search(&key) {
            Some(edit) => {
                if let text_edit::KeyEdit::Copy(s) | text_edit::KeyEdit::Cut(s) = &edit {
                    *cx.last_message = Some(cx.clipboard.copy(msg::input::WHAT_SELECTION.get(), s));
                }
                repo_manager::ManagerAction::Handled
            }
            None => repo_manager::handle_event(manager, &key),
        };
        if action == repo_manager::ManagerAction::Quit {
            *cx.should_quit = true;
            *cx.quit_how = Some("q key on the Repository Management tab");
        } else {
            // A menu opened from the keyboard hangs under
            // the selected row.
            let anchor = cx
                .management_panel
                .as_ref()
                .and_then(|(_, render)| {
                    let selected = cx.management.as_ref()?.selected_row_hit();
                    render.hits.iter().find_map(|(rect, hit)| {
                        (*hit == selected).then_some((rect.x + 4, rect.y + 1))
                    })
                })
                .unwrap_or((2, 2));
            if let Some(message) = apply_manager_action(action, anchor, &mut tab_ctx!(cx)) {
                *cx.last_message = Some(message.clone());
                if let Some(m) = cx.management.as_mut() {
                    m.message = Some(message);
                }
            }
        }
        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;

        return Ok(Flow::Handled);
    }

    // The commit column has the keyboard (a click on it, its badge, the
    // palette's Focus command): the arrows pick a commit, ahead of the
    // views that otherwise own every key (the editor, the File View).
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.commit_col.keys
        && cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_open()
        && commands::commit_column::on_key(cx, km, key, env.unpushed_tags)
    {
        return Ok(Flow::Handled);
    }

    // A Log column is picked (`Shift+C`): its keys come before the Log's own.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.log_col_pick.is_some()
        && cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_open()
        && commands::log_columns::on_key(cx, km, key)
    {
        return Ok(Flow::Handled);
    }

    // Esc stops a running AI suggestion before the editor reads it as
    // "close" (the editor would otherwise ask about its unsaved work).
    if key.kind == KeyEventKind::Press
        && key.code == KeyCode::Esc
        && matches!(*cx.input_mode, InputMode::Normal)
        && *cx.central_view == CentralView::Merge
        && cx.ai_ui.running_conflict()
    {
        cx.ai_ui.cancel();
        return Ok(Flow::Handled);
    }

    // The merge conflict editor owns the keyboard while it is the
    // central view (on top of everything, the editor included), all but
    // the chords it leaves to the window (the palette, the prefixes).
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && *cx.central_view == CentralView::Merge
        && cx.merge.as_ref().is_some_and(|m| !m.passes_through(&key))
        && cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_log_menu()
        && !cx.open_menu.is_wip_menu()
        && !cx.open_menu.is_ref_menu()
        && !cx.open_menu.is_file_menu()
        && !cx.open_menu.is_folder_menu()
        && !cx.open_menu.is_pane_menu()
    {
        let Some(view) = cx.merge.as_mut() else {
            return Ok(Flow::Handled);
        };
        let out = view.on_key_counted(&key, &mut *cx.clipboard, env.count.max(1) as usize);
        commands::apply_merge_outcome(cx, out);
        return Ok(Flow::Handled);
    }

    // The editor owns the keyboard while it is
    // the central view, ahead of the File View under it -- all
    // but the `Ctrl` chords it has no use for
    // (`Editor::passes_through`: the palette, the `Ctrl+N`
    // prefix). Tab is its own (indent), so focus does not move
    // with Tab while editing; the mouse and `Ctrl+N` still do.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && *cx.central_view == CentralView::Editor
        && cx.editor.as_ref().is_some_and(|e| !e.passes_through(&key))
        && cx.composer_focus.is_none()
        && !*cx.commit_filter_focus
        && !cx.open_menu.is_log_menu()
        && !cx.open_menu.is_wip_menu()
        && !cx.open_menu.is_ref_menu()
        && !cx.open_menu.is_file_menu()
        && !cx.open_menu.is_folder_menu()
        && !cx.open_menu.is_pane_menu()
    {
        let Some(ed) = cx.editor.as_mut() else {
            return Ok(Flow::Handled);
        };
        let out = ed.on_key(&key, &mut *cx.clipboard);
        apply_editor_outcome(
            out,
            &mut *cx.editor,
            &mut *cx.file_insight,
            cx.link,
            &mut *cx.last_message,
            cx.diff_config_path,
        );

        return Ok(Flow::Handled);
    }

    // The File History list keeps its keys while its right side is a File
    // View or Blame (those are the overlay's arm below): rows, modes, range.
    if key.kind == KeyEventKind::Press
        && cx.file_insight.is_some()
        && matches!(*cx.input_mode, InputMode::Normal)
        && !cx.open_menu.is_open()
        && commands::history::on_key(cx, km, key)
    {
        return Ok(Flow::Handled);
    }

    // The File Insight overlay owns the keyboard while it is
    // open, ahead of `input_mode` dispatch -- same shape as
    // the help arm above, and for the same reason it is an
    // arm rather than a branch inside the one below.
    //
    // This arm's position is load-bearing: the Diff view's
    // own `Enter` stages the selected lines, and the
    // overlay's `Enter` goes to a blamed line's commit. Only
    // the ordering keeps one keystroke from doing both. See
    // `file_insight_owns_keys`.
    //
    // Every `Ctrl` chord and Tab are let through
    // (`file_panel_passes_through`) so the navigation prefix,
    // the palette and pane focus stay reachable while the
    // File View is open, instead of this arm swallowing them
    // before `InputMode::Normal` ever sees them.
    if key.kind == KeyEventKind::Press
                        && file_insight_owns_keys(cx.file_insight.as_ref(), &*cx.input_mode)
                        && !file_panel_passes_through(&key)
                        // A pane, not a modal: a field being typed into or a menu open elsewhere
                        // in the window keeps its keys.
                        && cx.composer_focus.is_none()
                        && !*cx.commit_filter_focus
                        && !cx.open_menu.is_log_menu()
                        && !cx.open_menu.is_wip_menu()
                        && !cx.open_menu.is_ref_menu()
                        && !cx.open_menu.is_file_menu()
                        && !cx.open_menu.is_folder_menu()
                        && !cx.open_menu.is_pane_menu()
    {
        let viewport = cx.file_insight_area.height.saturating_sub(2) as usize;
        let Some(insight) = cx.file_insight.as_mut() else {
            return Ok(Flow::Handled);
        };
        // The Blame view's own keys first: hunk moves, the card, its
        // actions, and the "go to line" prompt that swallows the rest.
        match insight.blame_key(km, &key, viewport) {
            crate::blame_view::KeyOutcome::Handled => return Ok(Flow::Handled),
            crate::blame_view::KeyOutcome::Run(op) => {
                run_blame_op(cx, op);
                return Ok(Flow::Handled);
            }
            crate::blame_view::KeyOutcome::Pass => {}
        }
        match key.code {
            // Back to the graph, as the Diff view's Esc does
            // -- the bar's Diff View is the
            // way back to a diff it was opened over.
            // An open text selection is
            // the nearer thing to cancel.
            KeyCode::Esc if insight.text_sel.is_some() => {
                insight.text_sel = None;
                *cx.last_message = Some(msg::input::TEXT_SELECTION_CLEARED.get().into());
            }
            _ if key.code == KeyCode::Esc || km.is(Bind::FileInsightClose, &key) => {
                close_file_panel(&mut *cx.file_insight, &mut *cx.diff_view_state)
            }
            // Edit in Working Directory.
            _ if km.is(Bind::FileInsightEdit, &key) => {
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
            }
            // Copy: the text selection, else the selected
            // row's text (`Ctrl+C` is kept here rather than
            // passed through -- `file_panel_passes_through`).
            _ if km.is(Bind::FileInsightCopy, &key) => {
                *cx.last_message = Some(match insight.yank() {
                    Ok((what, text)) => cx.clipboard.copy(what, &text),
                    Err(why) => why.to_string(),
                });
            }
            // A picture: "Edit in" (the list, or the default editor).
            _ if crate::image_view::pane::shows(insight) && km.is(Bind::ImageEditIn, &key) => {
                crate::image_view::edit_in::open(crate::image_view::edit_in::Target::Pane);
            }
            _ if crate::image_view::pane::shows(insight) && km.is(Bind::ImageEditDefault, &key) => {
                *cx.last_message = Some(crate::image_view::edit_in::run_default(
                    &crate::image_view::edit_in::Target::Pane,
                    cx.repo_root,
                ));
            }
            // A binary file with OS apps: the same keys open its "Open in" list
            // or run the default app.
            _ if crate::open_with::view::shown().is_some() && km.is(Bind::ImageEditIn, &key) => {
                if let Some(subject) = crate::open_with::view::shown() {
                    crate::open_with::popup::open(subject);
                }
            }
            _ if crate::open_with::view::shown().is_some()
                && km.is(Bind::ImageEditDefault, &key) =>
            {
                if let Some(subject) = crate::open_with::view::shown() {
                    *cx.last_message = Some(crate::open_with::popup::run_default(
                        &subject,
                        cx.repo_root,
                        cx.link,
                    ));
                }
            }
            // A picture: zoom keys and panning, ahead of the row moves.
            _ if crate::image_view::pane::shows(insight)
                && crate::image_view::pane::on_key(km, &key) => {}
            KeyCode::Char('j') | KeyCode::Down => insight.move_selection(1, viewport),
            KeyCode::Char('k') | KeyCode::Up => insight.move_selection(-1, viewport),
            KeyCode::PageDown => insight.move_selection(viewport as isize, viewport),
            KeyCode::PageUp => insight.move_selection(-(viewport as isize), viewport),
            KeyCode::Home => insight.move_selection(isize::MIN / 2, viewport),
            // `G` joins `End` -- the vim spelling the maintainer's
            // muscle memory already uses everywhere else the
            // Log's paging semantics apply.
            KeyCode::End => insight.move_selection(isize::MAX / 2, viewport),
            _ if km.is(Bind::FileInsightLastRow, &key) => {
                insight.move_selection(isize::MAX / 2, viewport)
            }
            // Horizontal scroll: `h`/`l` and the arrow keys.
            // Independent of the selection, same as the wheel.
            KeyCode::Char('h') | KeyCode::Left => insight.scroll_h(-4),
            KeyCode::Char('l') | KeyCode::Right => insight.scroll_h(4),
            _ if km.is(Bind::FileInsightOpenCommit, &key) => {
                // Blame and File History both answer "which
                // commit is this?", so Enter closes the
                // overlay and puts the Log on that commit --
                // the click-through the brief asks for, with
                // its keyboard equivalent.
                if let Some(commit) = insight.commit_at(insight.selected) {
                    if select_log_commit(
                        &*cx.log_result,
                        &*cx.log_order,
                        &mut *cx.log_state,
                        commit,
                    ) {
                        close_file_panel(&mut *cx.file_insight, &mut *cx.diff_view_state);
                        *cx.focus = Focus::Log;
                        *cx.right_panel = RightPanel::CommitDetail;
                    } else {
                        *cx.last_message =
                            Some(msg::input::not_in_loaded_log(&commit.to_hex()[..8]));
                    }
                }
            }
            _ => {}
        }

        return Ok(Flow::Handled);
    }

    // The Add / Edit remote popup, then the remote menu, own
    // the keyboard while open -- the same rule as the context
    // menus below, and for the same reason: keys the popup
    // does not use are swallowed, never passed to the panes.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.remote_ui.is_open()
    {
        if let Some(form) = cx.remote_ui.form.as_mut() {
            match form.on_key(key) {
                FormOutcome::Close => cx.remote_ui.form = None,
                FormOutcome::Submit(submit) => {
                    cx.remote_ui.form = None;
                    *cx.last_message = Some(submit_remote_form(
                        submit,
                        &mut *cx.remote_ui,
                        cx.link,
                        &mut *cx.hidden_from_graph,
                        &*cx.solo_refs,
                    ));
                }
                FormOutcome::Stay => {}
            }
        } else if let Some(menu) = cx.remote_ui.menu.as_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
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
                MenuOutcome::Close => cx.remote_ui.menu = None,
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        }

        return Ok(Flow::Handled);
    }

    // A pull-request / issue search box with the keyboard: it
    // types, Esc / Enter / Tab leave it.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.hosting.editing.is_some()
        && cx.hosting.picker.is_none()
        && cx.hosting.menu.is_none()
    {
        cx.hosting.on_search_key(key);

        return Ok(Flow::Handled);
    }

    // An open sidebar dropdown (filter, provider, repository)
    // is modal: it owns the keyboard.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.hosting.picker.is_some()
    {
        let effect = cx.hosting.on_picker_key(key);
        commands::apply_hosting_effect(cx, effect);

        return Ok(Flow::Handled);
    }

    // The hosting menu (a pull-request / issue row, the toolbar
    // chip, a palette chooser) owns the keyboard while open,
    // and swallows what it does not claim, like the others.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.hosting.menu.is_some()
    {
        let outcome = cx.hosting.menu.as_mut().map(|menu| menu.on_key(key.code));
        match outcome {
            Some(MenuOutcome::Run(action)) => {
                let effect = cx.hosting.run(action, &mut *cx.clipboard);
                commands::apply_hosting_effect(cx, effect);
            }
            Some(MenuOutcome::Close) => cx.hosting.menu = None,
            _ => {}
        }

        return Ok(Flow::Handled);
    }

    // A context menu owns the keyboard while it is open --
    // ahead of `input_mode` dispatch, the same shape (and for
    // the same reason) as the help and File Insight arms
    // above. Without this the menus opened by `.`/right-click
    // could only be dismissed, never operated, and `j`/`k`
    // would move the Log selection out from under a menu
    // captioned with a commit.
    //
    // The Log's columns chooser: the arrows (or `j` / `k`) move the cursor
    // over the columns that can be hidden, Space or Enter toggle the one
    // under it (the popup stays open, as a click does), Esc closes. Every
    // other key is swallowed.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.open_menu.is_column_menu()
    {
        let menu = cx.open_menu.column_menu().unwrap_or(ColumnMenu::new(None));
        let rows = crate::log_table::chooser_rows(menu.target);
        let layout = crate::live_settings::log_columns_layout();
        match key.code {
            KeyCode::Esc => cx.open_menu.close_column_menu(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                if let Some(m) = cx.open_menu.column_menu_mut() {
                    m.cursor = crate::log_table::step_chooser(&rows, layout, menu.cursor, true);
                }
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                if let Some(m) = cx.open_menu.column_menu_mut() {
                    m.cursor = crate::log_table::step_chooser(&rows, layout, menu.cursor, false);
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(row) = rows.get(menu.cursor).copied() {
                    commands::log_columns::run_chooser_row(cx, row);
                }
            }
            _ => {}
        }
        return Ok(Flow::Handled);
    }
    // Keys the menu does not claim are *swallowed*, not
    // passed on: a popup that quietly lets `q` quit the app
    // underneath it is the click-through bug this project
    // has already fixed for the mouse.
    if key.kind == KeyEventKind::Press
        && matches!(*cx.input_mode, InputMode::Normal)
        && cx.open_menu.is_context_menu()
    {
        if let Some((_, menu)) = cx.open_menu.log_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    if let Some((commit, _)) = cx.open_menu.take_log_menu() {
                        super::menu_actions::run_log_menu_action(cx, action, commit);
                    }
                }
                MenuOutcome::Close => cx.open_menu.close_log_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some(menu) = cx.open_menu.wip_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    cx.open_menu.close_wip_menu();
                    super::menu_actions::apply_wip_action(cx, action);
                }
                MenuOutcome::Close => cx.open_menu.close_wip_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((_, _, menu)) = cx.open_menu.ref_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) if crate::sections_menu::is_sections_action(action) => {
                    commands::apply_sidebar_sections_action(cx, action);
                }
                MenuOutcome::Run(action) if commands::is_ref_fold_action(action) => {
                    commands::apply_ref_fold_action(cx, action);
                }
                MenuOutcome::Run(action) => {
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
                MenuOutcome::Close => cx.open_menu.close_ref_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((_, _, menu)) = cx.open_menu.file_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    if let Some((section, path, _)) = cx.open_menu.take_file_menu() {
                        commands::apply_file_menu_action(cx, section, path, action);
                    }
                }
                MenuOutcome::Close => cx.open_menu.close_file_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((_, _, menu)) = cx.open_menu.folder_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    if let Some((section, path, _)) = cx.open_menu.take_folder_menu() {
                        super::menu_actions::apply_folder_menu_action(cx, section, path, action);
                    }
                }
                MenuOutcome::Close => cx.open_menu.close_folder_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((_, menu)) = cx.open_menu.pane_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    if let Some((target, _)) = cx.open_menu.take_pane_menu() {
                        commands::apply_pane_action(cx, target, action);
                    }
                }
                MenuOutcome::Close => cx.open_menu.close_pane_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((_, menu)) = cx.open_menu.history_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(action) => {
                    cx.open_menu.close_history_menu();
                    commands::history::run_action(cx, action);
                }
                MenuOutcome::Close => cx.open_menu.close_history_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some(menu) = cx.open_menu.commit_columns_menu_mut() {
            match menu.on_key(key.code) {
                MenuOutcome::Run(act) => {
                    cx.open_menu.close_commit_columns_menu();
                    commands::commit_column::run_columns_menu(cx, act);
                }
                MenuOutcome::Close => cx.open_menu.close_commit_columns_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        } else if let Some((target, menu)) = cx.open_menu.diff_hunk_menu_mut() {
            let target = *target;
            match menu.on_key(key.code) {
                MenuOutcome::Run(DiffHunkContextAction::Copy(what)) => {
                    cx.open_menu.close_diff_hunk_menu();
                    *cx.last_message = copy_from_diff(
                        &*cx.diff_view_state,
                        &*cx.diff_render,
                        what,
                        &mut *cx.clipboard,
                    );
                }
                MenuOutcome::Run(action) => {
                    cx.open_menu.close_diff_hunk_menu();
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
                MenuOutcome::Close => cx.open_menu.close_diff_hunk_menu(),
                MenuOutcome::Stay | MenuOutcome::NotHandled => {}
            }
        }

        return Ok(Flow::Handled);
    }

    Ok(Flow::Continue)
}

/// What "Open this link?" just did (opened, or the browser's refusal) goes to
/// the status line.
pub(super) fn take_link_result(cx: &mut CommandCtx<'_>) {
    if let Some(message) = crate::link_confirm::take_result() {
        *cx.last_message = Some(message);
    }
}
