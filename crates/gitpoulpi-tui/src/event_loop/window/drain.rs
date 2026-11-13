//! The input drain: reads every already-queued terminal event before the
//! next redraw and routes it to the paste, key and mouse handlers.

use super::{poll_timeout, FrameInputs};
use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    /// Drains the event queue. The first poll may block briefly
    /// ([`poll_timeout`]); later ones do not.
    pub(in crate::event_loop) fn drain_input(&mut self, fi: &FrameInputs) -> io::Result<()> {
        let FrameInputs {
            palette_ctx,
            ref unpushed_tags,
            ..
        } = *fi;
        // Drains every already-queued input event before the next redraw, so a
        // burst of keypresses does not wait one draw cycle each. The first `poll`
        // blocks briefly so an idle loop does not spin; later ones use a zero
        // timeout.
        //
        // **`continue` inside this loop is safe, by construction.** The poll that
        // decides whether to read again is the *first* thing each iteration does,
        // so skipping the rest of the body can never leave the loop believing an
        // event is queued. Do not move the poll to the bottom: a `continue` then
        // jumps into a blocking `event::read()` and the frame is not redrawn until
        // another key arrives (clicks on the tab bar and Preferences looked inert).
        self.should_quit = false;
        self.repeat_held_arrow(palette_ctx, unpushed_tags)?;
        self.scroll_dragged_tab();
        let mut first_poll = true;
        loop {
            // A press on a `Ctrl+N` badge asked for a click: it is read here as
            // if the pointer had made it.
            let mut event = if let Some(click) = crate::dialog_kit::frame::take_injected() {
                Event::Mouse(click)
            } else if let Some(queued) = self.queued_events.pop_front() {
                // Read ahead by the paste check below.
                queued
            } else {
                let timeout = poll_timeout(first_poll, self.animating());
                first_poll = false;
                // A failed console read is skipped rather than fatal, up to
                // `MAX_CONSECUTIVE_INPUT_ERRORS` in a row: on Windows every mouse event
                // re-queries the console screen buffer, and one transient failure there
                // must not end the session.
                let ready = match event::poll(timeout) {
                    Ok(ready) => ready,
                    Err(err) => {
                        exit_log::tolerate_input_error(err, &mut self.input_errors)
                            .map_err(exit_log::io_context("waiting for terminal input"))?;
                        break;
                    }
                };
                if !ready {
                    break;
                }
                match event::read() {
                    // Windows has no paste brackets: text that arrives as a
                    // run of keys with no gap between them is a paste
                    // (`paste_burst`). ConPTY turns a paste into key records
                    // a little after the first one is readable, so each next
                    // key is awaited for `paste_burst::GAP`: no person types
                    // that fast.
                    Ok(event) if cfg!(windows) => {
                        let mut ahead = || match event::poll(crate::paste_burst::GAP) {
                            Ok(true) => event::read().ok(),
                            _ => None,
                        };
                        let (now, after) = crate::paste_burst::take(event, &mut ahead);
                        self.queued_events.extend(after);
                        now
                    }
                    Ok(event) => event,
                    Err(err) => {
                        exit_log::tolerate_input_error(err, &mut self.input_errors)
                            .map_err(exit_log::io_context("reading terminal input"))?;
                        continue;
                    }
                }
            };
            self.input_errors = 0;
            // Windows reports AltGr as Ctrl+Alt: every handler sees the plain
            // character (`keymap::normalize_event`).
            event = keymap::normalize_event(event);
            // A tab opened from outside while the loop idled in the poll
            // above: take it before the key acts on the tab list (the
            // workspace search scope reads it), not a frame later.
            if matches!(
                &event,
                Event::Key(_)
                    | Event::Mouse(crossterm::event::MouseEvent {
                        kind: MouseEventKind::Down(_),
                        ..
                    })
            ) {
                self.sync_external_tabs();
            }
            self.redraw.input_handled = true;
            exit_log::note_input(&event);
            self.track_arrow_hold(&event);
            if self.ui_viewer.handle_event(&event) {
                continue;
            }
            // One dismissal rule for every tooltip (`tooltip.rs`): anything
            // but a plain pointer move hides it until the pointer moves.
            match &event {
                Event::Mouse(m) if m.kind == MouseEventKind::Moved => {}
                Event::Key(_) | Event::Mouse(_) | Event::Resize(..) | Event::Paste(_) => {
                    self.tooltip.dismiss()
                }
                Event::FocusLost => {
                    self.tooltip.reset();
                    self.hovered_commit = None;
                }
                _ => {}
            }

            // The mouse half of editing a main-window
            // text field, and pasting into one. Only when nothing modal
            // sits over the fields -- a click inside a dialog must not
            // move the caret of the field drawn underneath it.
            self.sync_fields();
            // A popup opened over Settings (palette, pickers, workspace
            // search) has its own text field, which then is uncovered.
            let fields_uncovered = (self.preferences.is_none()
                || matches!(
                    self.input_mode,
                    InputMode::CommandPalette { .. }
                        | InputMode::ThemePicker(_)
                        | InputMode::LanguagePicker(_)
                        | InputMode::WorkspaceSearch(_)
                ))
                && !self.repo_dialogs.is_open()
                && !self.remote_ui.is_open()
                && !self.show_help
                && !self.ai_ui.popup_open()
                && !crate::ai_sent_dialog::is_open()
                && !crate::ai_limit_dialog::is_open()
                && !crate::link_confirm::is_open()
                && !crate::stage_guard_dialog::is_open()
                && !crate::activity_log::is_open()
                && !crate::trust_dialog::is_open()
                && !crate::jobs_ui::panel_open()
                && !crate::updates::panel::is_open()
                && !matches!(
                    self.input_mode,
                    InputMode::Confirm { .. }
                        | InputMode::EditingCommitMessage(_)
                        | InputMode::InteractiveRebase(_)
                        | InputMode::ComposeCommits(_)
                        | InputMode::SplitCommit(_)
                        | InputMode::Rewrite(_)
                );
            if input::main_field_event(&mut self.command_ctx(), &event, fields_uncovered)
                == input::Flow::Handled
            {
                continue;
            }
            // An event earlier in this drain may have opened or closed
            // the File View; the next one must see the column it left.
            sync_central_view(
                &mut self.central_view,
                self.merge.is_some(),
                self.editor.is_some(),
                self.file_insight.is_some(),
                self.diff_view_state.is_some(),
            );

            // An open overflow menu owns the keyboard and every mouse press; a row
            // it runs comes back as a left click with `forced_hit` set.
            let mut forced_hit: Option<HitTarget> = None;
            if input::overflow_menu_event(&mut self.command_ctx(), &mut event, &mut forced_hit)
                == input::Flow::Handled
            {
                continue;
            }

            // The command palette and the tab picker own the mouse while open.
            if let Event::Mouse(m) = &event {
                let (palette_popup, tab_picker_popup) = (self.palette_popup, self.tab_picker_popup);
                match input::popup_list_mouse(
                    &mut self.command_ctx(),
                    m,
                    palette_popup,
                    tab_picker_popup,
                ) {
                    input::ListMouse::Pass => {}
                    input::ListMouse::Swallow => continue,
                    input::ListMouse::Enter => {
                        event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
                    }
                    input::ListMouse::ConfigureKey => {
                        event = Event::Key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::ALT));
                    }
                }
            }

            // Vim-style count prefix (`4j`, `10k`, ...). `count` is *peeked* here,
            // before `pending_count` is touched, so every arm below can read this one
            // local. A continuing digit is swallowed (`continue`), never also
            // dispatched as a bare `Char('4')`. Guards live in
            // `chrome::count_prefix_may_apply` (unit-tested): `Press` only (Windows
            // also reports `Release`), `InputMode::Normal` only, no context menu, no
            // composer field focused, Preferences closed (its numeric fields need
            // digits verbatim). CONTROL/ALT are excluded; SHIFT is not, since
            // terminals routinely set it. `is_count_digit`/`accumulate_count`
            // (`chrome.rs`) are the pure halves; `0` alone never starts a count.
            //
            // Every other `Press` key clears `pending_count`, so "Esc or any
            // non-motion key clears it" needs no separate cancel path and a count can
            // never leak into a later `j`.
            let count: i32 = self.pending_count.unwrap_or(1).max(1) as i32;
            if let Event::Key(key) = &event {
                if key.kind == KeyEventKind::Press
                    && count_prefix_may_apply(
                        matches!(self.input_mode, InputMode::Normal),
                        self.open_menu.is_log_menu() || self.open_menu.is_wip_menu(),
                        self.open_menu.is_ref_menu(),
                        self.composer_focus.is_some() || self.commit_filter_focus,
                        self.preferences.is_some(),
                        self.repo_dialogs.is_open()
                            // The Activity Log is modal: 1-5 are its level boxes
                            // and the filter field types digits.
                            || crate::activity_log::is_open()
                            || crate::jobs_ui::panel_open()
                            || crate::updates::panel::is_open()
                            // The tag delete dialog's rows are picked by their digit.
                            || crate::tag_delete_dialog::is_open()
                            // The security check's answers are digits too.
                            || crate::trust_dialog::is_open()
                            // The sensitive-files question: 1 and 2 answer it.
                            || crate::stage_guard_dialog::is_open()
                            || self.remote_ui.is_open()
                            // The encoding dropdown's search line types digits
                            // (`1252`), never a count.
                            || crate::diff_encoding_menu::is_open()
                            // The Home tab has no counts: its digits are the
                            // pages (`1` / `2` / `3`), the search field's text.
                            || self.management.is_some()
                            // A text editor owns the digits too: typed into the
                            // editor or the merge editor's Output (or its find bar)
                            // they are text, never a count.
                            || (self.central_view == CentralView::Editor
                                && self.editor.is_some())
                            // `0` and `1` are the image view's Fit and 1:1.
                            || (self.central_view == CentralView::File
                                && crate::image_view::pane::active())
                            || (self.central_view == CentralView::Diff
                                && crate::image_view::split::active())
                            || (self.central_view == CentralView::Merge
                                && self.merge.as_ref().is_some_and(|m| m.types_text())),
                    )
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && matches!(key.code, KeyCode::Char(c) if is_count_digit(c, self.pending_count))
                {
                    if let KeyCode::Char(c) = key.code {
                        self.pending_count =
                            accumulate_count(self.pending_count, c.to_digit(10).unwrap());
                    }
                    continue;
                } else if key.kind == KeyEventKind::Press {
                    self.pending_count = None;
                }
            }

            // The effective keymap, taken once per event.
            let km = keymap::current();
            let env = input::KeyEnv {
                km: &km,
                count,
                palette_ctx,
                unpushed_tags,
            };
            match event {
                // Windows (ConPTY/Windows Terminal) reports both a Press
                // and a Release event per physical keystroke; without this
                // guard every key handled below fires twice (e.g. one
                // arrow-key tap moving the selection by two rows).
                Event::Key(key) if key.kind != KeyEventKind::Press => {}
                // The two question dialogs have no text field: a paste goes nowhere.
                Event::Paste(_)
                    if crate::ai_sent_dialog::is_open()
                        || crate::ai_limit_dialog::is_open()
                        || crate::link_confirm::is_open()
                        || crate::stage_guard_dialog::is_open() => {}
                // A pasted token goes into Preferences' Integrations form.
                Event::Paste(text) if self.preferences.is_some() => {
                    if let Some(prefs) = self.preferences.as_mut() {
                        prefs.paste(&text);
                    }
                }
                Event::Paste(text) if self.repo_dialogs.is_open() => {
                    self.repo_dialogs.on_paste(&text)
                }
                // The review hosted over the Radar types like any dialog.
                Event::Paste(text)
                    if matches!(self.input_mode, InputMode::Normal)
                        && self
                            .management
                            .as_ref()
                            .is_some_and(|m| m.radar.hosts_review()) =>
                {
                    if let Some(manager) = self.management.as_mut() {
                        manager.radar.paste(&text);
                    }
                }
                Event::Paste(text)
                    if self.central_view == CentralView::Merge
                        && matches!(self.input_mode, InputMode::Normal)
                        && self.merge.is_some() =>
                {
                    if let Some(view) = self.merge.as_mut() {
                        view.paste(&text);
                    }
                }
                Event::Paste(text)
                    if self.central_view == CentralView::Editor
                        && matches!(self.input_mode, InputMode::Normal)
                        && self.composer_focus.is_none()
                        && !self.commit_filter_focus
                        && self.editor.is_some() =>
                {
                    if let Some(ed) = self.editor.as_mut() {
                        ed.paste(&text);
                    }
                }
                Event::Paste(text)
                    if matches!(self.input_mode, InputMode::EditingCommitMessage(_)) =>
                {
                    if let InputMode::EditingCommitMessage(form) = &mut self.input_mode {
                        form.on_paste(&text);
                    }
                }
                Event::Paste(text)
                    if matches!(self.input_mode, InputMode::InteractiveRebase(_)) =>
                {
                    if let InputMode::InteractiveRebase(form) = &mut self.input_mode {
                        form.on_paste(&text);
                    }
                }
                Event::Paste(text) if matches!(self.input_mode, InputMode::Rewrite(_)) => {
                    if let InputMode::Rewrite(view) = &mut self.input_mode {
                        view.paste(&text);
                    }
                }
                Event::Paste(text) if matches!(self.input_mode, InputMode::SplitCommit(_)) => {
                    if let InputMode::SplitCommit(form) = &mut self.input_mode {
                        form.on_paste(&text);
                    }
                }
                Event::Paste(text) if matches!(self.input_mode, InputMode::ComposeCommits(_)) => {
                    if let InputMode::ComposeCommits(form) = &mut self.input_mode {
                        form.on_paste(&text);
                    }
                }
                Event::Paste(text) if crate::open_with::popup::is_open() => {
                    crate::open_with::popup::paste(&text);
                }
                Event::Paste(text) if crate::diff_encoding_menu::is_open() => {
                    crate::diff_encoding_menu::paste(&text);
                }
                Event::Paste(text) if crate::image_view::edit_in::is_open() => {
                    crate::image_view::edit_in::paste(&text);
                }
                Event::Paste(text) if crate::workspace_ui::chip::is_open() => {
                    crate::workspace_ui::chip::paste(&text);
                }
                Event::Paste(text) if crate::profile_bar::is_open() => {
                    crate::profile_bar::paste(&text);
                }
                Event::Paste(text) if crate::open_in_bar::is_open() => {
                    crate::open_in_bar::paste(&text);
                }
                Event::Paste(text) if self.remote_ui.form.is_some() => {
                    if let Some(form) = self.remote_ui.form.as_mut() {
                        form.on_paste(&text);
                    }
                }
                Event::Paste(text)
                    if matches!(self.input_mode, InputMode::Normal)
                        && (self.hosting.picker.is_some() || self.hosting.editing.is_some()) =>
                {
                    self.hosting.paste(&text);
                }
                Event::Key(key) => {
                    input::handle_key(&mut self.command_ctx(), &env, key)?;
                }
                Event::Mouse(mouse) => {
                    input::handle_mouse(&mut self.command_ctx(), &env, mouse, &mut forced_hit)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl WindowState {
    /// Follows the left button for the arrow auto-repeat: a press on a
    /// scrollbar arrow arms it, moving off/on the arrow pauses/resumes it, and
    /// a release, a key, a focus loss or a resize ends it.
    fn track_arrow_hold(&mut self, event: &Event) {
        let now = std::time::Instant::now();
        match event {
            Event::Mouse(m) => {
                let cell = (m.column, m.row);
                match m.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if crate::scrollbar::is_arrow_cell(cell.0, cell.1) {
                            self.arrow_repeat.press(cell, now);
                        } else {
                            self.arrow_repeat.stop();
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => self.arrow_repeat.pointer(cell, now),
                    MouseEventKind::Down(_) | MouseEventKind::Up(_) => self.arrow_repeat.stop(),
                    _ => {}
                }
            }
            Event::Key(k) if k.kind == KeyEventKind::Press => self.arrow_repeat.stop(),
            Event::FocusLost | Event::Resize(..) => self.arrow_repeat.stop(),
            _ => {}
        }
    }

    /// A tab dragged onto one of the bar's scroll arrows and held there keeps
    /// the bar scrolling (`tab_scroll::drag_due`); any other way the drag
    /// ended drops the hold.
    fn scroll_dragged_tab(&mut self) {
        let dragging = self
            .tab_drag
            .as_ref()
            .is_some_and(|d| d.started && d.origin_kind == crate::tab_drag::DragFrom::Bar);
        if !dragging || crate::tab_column::area().is_some() {
            crate::tab_scroll::drag_end();
            return;
        }
        // The marker and the dimmed source follow the last painted frame,
        // which a scroll step changed under a pointer that did not move.
        if let Some(drag) = self.tab_drag.as_mut() {
            drag.source_rect = self
                .tab_hits
                .iter()
                .find(|h| h.index == drag.from)
                .map(|h| h.tab);
            let at = drag.at;
            crate::tab_drag::update_bar_target(
                drag,
                &self.tab_hits,
                self.repo_tabs.len(),
                at,
                false,
            );
        }
        if crate::tab_scroll::drag_due(std::time::Instant::now()) {
            self.redraw.input_handled = true;
        }
    }

    /// One more press of the held arrow when its repeat is due.
    fn repeat_held_arrow(
        &mut self,
        palette_ctx: PaletteContext,
        unpushed_tags: &HashSet<String>,
    ) -> io::Result<()> {
        let Some((column, row)) = self.arrow_repeat.due(std::time::Instant::now()) else {
            return Ok(());
        };
        self.redraw.input_handled = true;
        let km = keymap::current();
        let env = input::KeyEnv {
            km: &km,
            count: 1,
            palette_ctx,
            unpushed_tags,
        };
        let press = crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        };
        input::handle_mouse(&mut self.command_ctx(), &env, press, &mut None)?;
        Ok(())
    }
}
