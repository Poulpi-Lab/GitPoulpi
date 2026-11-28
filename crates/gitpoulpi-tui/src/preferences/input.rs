//! Keyboard and mouse input for the Preferences screen, and the actions its buttons run.

use super::*;
use crate::keymap::term::TermKeymap;

pub(super) fn flash_unselected(flashing: bool, selected: bool) -> bool {
    flashing && !selected
}

impl PreferencesState {
    // ---------------------------------------------------------------
    // Input
    // ---------------------------------------------------------------

    /// The window-level keys this screen lets through to the ordinary
    /// dispatch instead of swallowing, mirroring
    /// `repo_manager::passes_through`: `?` for the help overlay (without it `?`
    /// did nothing while Preferences was open). Held back while a field is being
    /// typed into (`self.edit.is_some()`), the same as `?` being an
    /// ordinary character everywhere else text is being entered.
    pub(crate) fn passes_through(&self, key: &KeyEvent) -> bool {
        let typing_token = self.current_section() == PrefSection::Integrations
            && self.focus == PrefFocus::Fields
            && !self.strip
            && self.integrations.text_focused(&self.config);
        self.edit.is_none()
            && self.choice.is_none()
            && self.search.is_none()
            && !self.ai.prompt_open()
            && !self.ai.browse_open()
            && !self.wf_editor_open()
            && !self.rp_editor_open()
            && !self.ra_overlay_open()
            && !self.seq_overlay_open()
            && !typing_token
            && !(self.keys_active() && self.keys.text_focused())
            && {
                // The window-level chords that open a popup over Settings:
                // help, the command palette, the workspace search and the
                // `Ctrl+N` prefix (theme, language, About).
                use crate::keymap::Action as A;
                let km = crate::keymap::current();
                [
                    A::GlobalHelp,
                    A::GlobalPalette,
                    A::GlobalWorkspaceSearch,
                    A::GlobalNavPrefix,
                ]
                .iter()
                .any(|a| km.is(*a, key))
            }
    }

    /// `false` means "close the screen".
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> bool {
        let stay = self.on_key_inner(key);
        self.sync_theme_preview();
        stay
    }

    fn on_key_inner(&mut self, key: KeyEvent) -> bool {
        self.flash = None;
        self.notice = None;
        if self.confirm_action.is_some() {
            self.confirm_action_key(key);
            return true;
        }
        if self.ra_overlay_open() {
            self.ra_overlay_key(key);
            return true;
        }
        if self.close_ask.is_some() {
            return self.close_ask_key(key);
        }
        if self.confirm_undo_off.is_some() {
            return self.undo_off_key(key);
        }
        if self.ai.browse_open() {
            self.review_browse_key(key);
            return true;
        }
        if self.search.is_some() {
            return self.search_key(key);
        }
        if self.ai.prompt_open() {
            return self.ai_prompt_key(key);
        }
        if self.wf_editor_open() {
            return self.wf_editor_key(key);
        }
        if self.rp_editor_open() {
            return self.rp_editor_key(key);
        }
        if self.seq_overlay_open() {
            return self.seq_overlay_key(key);
        }
        if self.opens_search(&key) {
            self.open_search();
            return true;
        }
        if let Some((_, combo)) = self.choice.as_mut() {
            let outcome = combo.on_key(key);
            self.finish_choice(outcome);
            return true;
        }
        if self.edit.is_some() {
            return self.edit_key(key);
        }
        if self.sub_tab_key(&key) {
            return true;
        }
        if self.enter_strip_on(&key) {
            return true;
        }
        if self.keys_active() {
            return self.keys_key(&key);
        }
        if self.current_section() == PrefSection::Integrations
            && self.focus == PrefFocus::Fields
            && !self.strip
        {
            return self.integrations_key(&key);
        }
        if self.seq_steps_back_key(&key) {
            return true;
        }
        if self.focus == PrefFocus::List && self.list_shown() {
            if let Some(stay) = self.list_key(key) {
                return stay;
            }
        }
        if self.focus == PrefFocus::Fields && self.buttons_key(&key) {
            return true;
        }
        if self.focus == PrefFocus::Fields && self.quantity_key(&key) {
            return true;
        }
        let was_sections = self.focus == PrefFocus::Sections;
        let stay = self.on_key_generic(key);
        if was_sections && self.focus == PrefFocus::Fields {
            self.integrations.zone = crate::integrations_page::Zone::Types;
            self.strip = false;
        }
        stay
    }

    /// Keys while the "turn Undo off" question is up: Left/Right/Tab
    /// choose, Enter confirms the chosen button, Esc cancels.
    fn undo_off_key(&mut self, key: KeyEvent) -> bool {
        if let Some(confirm) = self.confirm_undo_off.as_mut() {
            let event = confirm.on_key(key);
            self.finish_undo_off(event);
        }
        true
    }

    /// Mouse while the question is up: a button answers it, the cross or a
    /// click outside cancels, anything else is swallowed.
    fn undo_off_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) {
        if let Some(confirm) = self.confirm_undo_off.as_mut() {
            let event = confirm.on_mouse(crossterm::event::MouseEvent {
                kind,
                column: col,
                row,
                modifiers: KeyModifiers::NONE,
            });
            self.finish_undo_off(event);
        }
    }

    fn finish_undo_off(&mut self, event: crate::dialog_kit::ConfirmEvent<bool>) {
        if let crate::dialog_kit::ConfirmEvent::Chose(turn_off) = event {
            self.confirm_undo_off = None;
            if turn_off {
                self.write_global("general.undo_enabled", "false");
            }
        }
    }

    /// `Ctrl+F`, or `/` where it is not a character being typed.
    fn opens_search(&self, key: &KeyEvent) -> bool {
        if self.edit.is_some() || self.choice.is_some() || self.keys_active() {
            return false;
        }
        let typing_token = self.current_section() == PrefSection::Integrations
            && self.focus == PrefFocus::Fields
            && !self.strip
            && self.integrations.text_focused(&self.config);
        // Both of `preferences.search`'s chords; a bare printable one is
        // not taken while a token is being typed.
        if !crate::keymap::current().is(crate::keymap::Action::PreferencesSearch, key) {
            return false;
        }
        let bare = !key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT);
        !(bare && typing_token)
    }

    fn on_key_generic(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return self.request_close(),
            KeyCode::Tab | KeyCode::BackTab => {
                let back = key.code == KeyCode::BackTab;
                self.focus = match (self.focus, self.list_shown(), back) {
                    (PrefFocus::Sections, true, false) => PrefFocus::List,
                    (PrefFocus::Sections, true, true) => PrefFocus::Fields,
                    (PrefFocus::List, _, false) => PrefFocus::Fields,
                    (PrefFocus::List, _, true) => PrefFocus::Sections,
                    (PrefFocus::Fields, true, false) => PrefFocus::Sections,
                    (PrefFocus::Fields, true, true) => PrefFocus::List,
                    (PrefFocus::Sections, false, _) => PrefFocus::Fields,
                    (PrefFocus::Fields, false, _) => PrefFocus::Sections,
                };
            }
            KeyCode::Right | KeyCode::Char('l') if self.focus == PrefFocus::Sections => {
                self.focus = if self.list_shown() {
                    PrefFocus::List
                } else {
                    PrefFocus::Fields
                };
            }
            KeyCode::Left | KeyCode::Char('h') if self.focus == PrefFocus::Fields => {
                self.focus = if self.list_shown() {
                    PrefFocus::List
                } else {
                    PrefFocus::Sections
                };
            }
            KeyCode::Left | KeyCode::Char('h') if self.focus == PrefFocus::List => {
                self.focus = PrefFocus::Sections;
            }
            KeyCode::Right | KeyCode::Char('l') if self.focus == PrefFocus::List => {
                self.apply_list_op(super::list_editor::ListOp::FocusDetail);
            }
            // The list answers its own keys; the rest is not a row's to act on.
            KeyCode::Down | KeyCode::Up | KeyCode::Enter | KeyCode::Char(' ' | 'j' | 'k')
                if self.focus == PrefFocus::List => {}
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(),
            _ => {}
        }
        true
    }

    pub(super) fn edit_key(&mut self, key: KeyEvent) -> bool {
        let Some((index, draft, masked)) = self.edit.as_mut() else {
            return true;
        };
        // Typing again lifts the red mark; a blur that is refused sets it.
        self.edit_invalid = None;
        self.inline_error = None;
        // The shared editing core: caret, selection,
        // Ctrl+A/U/K, Ctrl+C/X -- except that a secret is never copied or
        // cut to the clipboard.
        self.caret_edit.sync(Some(*index));
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let copies = matches!(key.code, KeyCode::Char('\u{3}' | '\u{18}'))
            || (ctrl && matches!(key.code, KeyCode::Char('c' | 'C' | 'x' | 'X')));
        if *masked && copies {
            return true;
        }
        if let Some((_, copied)) = self.caret_edit.dialog_key(*index, draft, &key) {
            if copied.is_some() {
                self.message = copied;
            }
            return true;
        }
        match key.code {
            // Esc cancels this edit alone.
            KeyCode::Esc => {
                self.edit = None;
                self.edit_invalid = None;
            }
            // Enter, and leaving the row, commit; a refused value keeps
            // the field open, red, with its text.
            KeyCode::Enter => {
                self.blur_edit();
            }
            // A quantity's number: Up/Down step it, Tab commits and moves to
            // its unit instead of the next row.
            KeyCode::Up | KeyCode::Down if self.editing_quantity() => {
                self.quantity_step(if key.code == KeyCode::Up { 1 } else { -1 });
            }
            KeyCode::Tab if self.editing_quantity() => {
                if self.blur_edit() {
                    self.ai.chip = quantity::PART_UNIT;
                }
            }
            KeyCode::Down | KeyCode::Tab if self.blur_edit() => self.move_selection(1),
            KeyCode::Up | KeyCode::BackTab if self.blur_edit() => self.move_selection(-1),
            _ => {}
        }
        true
    }

    /// Leaving the field being typed in (a click elsewhere, Tab, Up/Down,
    /// Enter) commits it. `false`: the value was refused, the field stays
    /// open and red with its text, and the caller must not go on.
    pub(super) fn blur_edit(&mut self) -> bool {
        let Some((index, draft, _)) = self.edit.clone() else {
            return true;
        };
        if self.edit_is_unchanged(index, &draft) || self.commit_text(index, &draft) {
            self.edit = None;
            self.edit_invalid = None;
            true
        } else {
            self.edit_invalid = Some(index);
            false
        }
    }

    /// Whether `draft` is what the field already holds (a blur then writes
    /// nothing).
    fn edit_is_unchanged(&self, index: usize, draft: &str) -> bool {
        let fields = self.fields();
        let Some(field) = fields.get(index) else {
            return true;
        };
        match (&field.control, &field.target) {
            // A new key is typed from scratch: empty means untouched.
            (_, Target::Ai(ai_page::AiTarget::ProfileKey)) => draft.trim().is_empty(),
            // A hidden environment value is typed from scratch.
            (Control::Text(v), Target::Ai(ai_page::AiTarget::ProfileEnv(_)))
                if v.starts_with('\u{2022}') =>
            {
                draft.is_empty()
            }
            (Control::Text(v), _) => v == draft,
            (Control::Number { value, .. } | Control::Quantity { value, .. }, _) => {
                value.to_string() == draft.trim()
            }
            _ => false,
        }
    }

    pub(super) fn move_selection(&mut self, delta: isize) {
        match self.focus {
            PrefFocus::Sections => {
                // Step over pages this build lacks.
                let next = PrefSection::step(self.section, delta, PrefSection::is_built) as isize;
                self.switch_section(next as usize);
            }
            PrefFocus::List => {}
            PrefFocus::Fields => {
                let fields = self.fields();
                let selectable: Vec<usize> = (0..fields.len())
                    .filter(|i| fields[*i].selectable())
                    .collect();
                if selectable.is_empty() {
                    return;
                }
                let at = selectable
                    .iter()
                    .position(|i| *i == self.field)
                    .unwrap_or(0) as isize;
                if self.strip_active() && at + delta >= selectable.len() as isize {
                    self.leave_strip();
                    return;
                }
                let next = (at + delta).clamp(0, (selectable.len() as isize - 1).max(0)) as usize;
                self.field = selectable[next];
                self.quantity_focus_reset();
            }
        }
    }

    pub(super) fn first_selectable(&self) -> usize {
        self.fields()
            .iter()
            .position(Field::selectable)
            .unwrap_or(0)
    }

    /// A key or click result of the open dropdown: a pick commits the
    /// option index it carries (a `Combo` row: its value, or the typed
    /// text).
    fn finish_choice(&mut self, outcome: crate::combo::ComboOutcome) {
        use crate::combo::ComboOutcome;
        // A quantity's unit: the pick is the unit's index.
        let unit_row = self.choice.as_ref().and_then(|(field, _)| {
            matches!(
                self.fields().get(*field).map(|f| &f.control),
                Some(Control::Quantity { .. })
            )
            .then_some(*field)
        });
        if let Some(field) = unit_row {
            if let ComboOutcome::Picked(value) = outcome {
                self.choice = None;
                self.field = field;
                if let Ok(pick) = value.parse::<usize>() {
                    self.commit_quantity_unit(field, pick);
                }
            } else if self.choice.as_ref().is_some_and(|(_, c)| !c.is_open()) {
                self.choice = None;
            }
            return;
        }
        let combo_row = self.choice.as_ref().and_then(|(field, _)| {
            match self
                .fields()
                .get(*field)
                .map(|f| (&f.control, f.target.clone()))
            {
                Some((Control::Combo { .. }, target)) => Some((*field, target)),
                _ => None,
            }
        });
        if let Some((field, target)) = combo_row {
            match outcome {
                ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => {
                    let picked = self.choice.take().is_some();
                    self.field = field;
                    if picked {
                        self.commit_combo(&target, &value);
                    }
                }
                _ => {
                    if self.choice.as_ref().is_some_and(|(_, c)| !c.is_open()) {
                        self.choice = None;
                    }
                }
            }
            return;
        }
        if let ComboOutcome::Picked(value) = outcome {
            if let (Some((field, _)), Ok(next)) = (self.choice.take(), value.parse::<usize>()) {
                self.field = field;
                let target = self.fields().get(field).map(|f| f.target.clone());
                if let Some(target) = target {
                    self.commit_choice(&target, next);
                }
            }
        } else if self.choice.as_ref().is_some_and(|(_, c)| !c.is_open()) {
            self.choice = None;
        }
    }

    /// A `Combo` row's pick: `value` is an option's value or typed text.
    pub(super) fn commit_combo(&mut self, target: &Target, value: &str) {
        match target {
            Target::Ai(t) => self.ai_commit_combo(*t, value),
            Target::Workflow(t) => self.wf_commit_combo(*t, value),
            Target::Sequence(t) => self.seq_commit_combo(*t, value),
            Target::RepoAuth(t) => self.ra_commit_combo(*t, value),
            Target::Global(key) => {
                self.write_global(key, value);
            }
            _ => {}
        }
    }

    /// Test helper: open the selected choice's dropdown and pick the option
    /// after the current one (wrapping), through the keyboard.
    #[cfg(test)]
    pub(super) fn choose_next(&mut self) {
        self.activate();
        let Some((_, combo)) = self.choice.as_mut() else {
            return;
        };
        let (mut at, len) = (0usize, combo.options.len());
        if let Some(cur) = combo.selected().and_then(|v| v.parse::<usize>().ok()) {
            at = cur;
        }
        let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
        if at + 1 >= len {
            self.on_key(key(KeyCode::Home));
        } else {
            self.on_key(key(KeyCode::Down));
        }
        self.on_key(key(KeyCode::Enter));
    }

    /// Enter / Space / a click on the selected row.
    pub(super) fn activate(&mut self) {
        if self.focus == PrefFocus::Sections {
            self.focus = if self.list_shown() {
                PrefFocus::List
            } else {
                PrefFocus::Fields
            };
            self.field = self.first_selectable();
            self.scroll = 0;
            return;
        }
        // The theme files may have changed since Settings opened.
        if self.fields().get(self.field).is_some_and(|f| {
            matches!(
                f.target,
                Target::Global("ui.theme" | "ui.graph_color_theme")
            )
        }) {
            self.rescan_user_themes();
        }
        let fields = self.fields();
        let Some(field) = fields.get(self.field) else {
            return;
        };
        match (&field.control, field.target.clone()) {
            (Control::Toggle(true), Target::Global("general.undo_enabled")) => {
                // Erases the history: ask first.
                self.confirm_undo_off = Some(
                    crate::dialog_kit::Confirm::new(
                        gitpoulpi_i18n::msg::settings_screen::UNDO_OFF_TITLE.get(),
                        gitpoulpi_i18n::msg::settings_screen::UNDO_OFF_QUESTION.get(),
                        vec![
                            (
                                gitpoulpi_i18n::msg::settings_screen::CANCEL
                                    .get()
                                    .to_string(),
                                false,
                            ),
                            (
                                gitpoulpi_i18n::msg::settings_screen::TURN_OFF
                                    .get()
                                    .to_string(),
                                true,
                            ),
                        ],
                        false,
                    )
                    .primary(true),
                );
            }
            (Control::Toggle(on), target) => {
                let next = !on;
                self.commit_bool(&target, next);
            }
            (Control::Choice { options, index }, target) => {
                let _ = target;
                if options.is_empty() {
                    return;
                }
                let mut combo =
                    crate::combo::Combo::new(gitpoulpi_i18n::msg::settings_screen::NONE.get());
                combo.set_options(
                    options
                        .iter()
                        .enumerate()
                        .map(|(i, o)| {
                            crate::combo::ComboOption::new(field.option_label(o), i.to_string())
                        })
                        .collect(),
                );
                combo.select(Some(index.to_string()));
                combo.open();
                self.choice = Some((self.field, combo));
            }
            (
                Control::Combo {
                    options,
                    value,
                    custom,
                },
                target,
            ) => {
                let mut combo =
                    crate::combo::Combo::new(gitpoulpi_i18n::msg::settings_screen::NONE.get());
                if *custom {
                    combo = combo.with_custom();
                }
                let lanes = target == Target::Global("ui.graph_color_theme");
                combo.set_options(
                    options
                        .iter()
                        .map(|(l, v)| {
                            let option = crate::combo::ComboOption::new(l.clone(), v.clone());
                            if lanes {
                                option.with_swatch(self.lane_swatch(v))
                            } else {
                                option
                            }
                        })
                        .collect(),
                );
                combo.select(Some(value.clone()));
                combo.open();
                self.choice = Some((self.field, combo));
            }
            (Control::Text(_) | Control::Area(_), Target::Ai(t)) if self.ai_activate(t) => {}
            (Control::Area(_), Target::Workflow(t)) => self.wf_open_editor(t),
            (Control::Area(_), Target::RepoPage(t)) => self.rp_open_editor(t),
            (Control::Chips(_), Target::ImageEditor(_)) => {
                let chip = self.ai.chip_focus(images_page::IMAGE_VARIABLES.len());
                self.images_insert_variable(chip);
            }
            (Control::Chips(labels), Target::OpenIn(_)) => {
                let chip = self.ai.chip_focus(labels.len());
                self.open_in_insert_variable(chip);
            }
            (Control::Text(value), _) => {
                let draft = if value.is_empty() {
                    String::new()
                } else {
                    value.clone()
                };
                self.caret_edit.reset();
                self.edit = Some((self.field, draft, false));
            }
            (Control::Number { value, .. }, _) => {
                self.caret_edit.reset();
                self.edit = Some((self.field, value.to_string(), false));
            }
            (Control::Quantity { .. }, _) => {
                self.activate_quantity(field);
            }
            (Control::Buttons(buttons), _) => {
                let at = self.ai.chip_focus(buttons.len());
                if let Some(button) = buttons.get(at) {
                    self.press_button(button);
                }
            }
            _ => {}
        }
    }

    pub(super) fn run_action(&mut self, action: PrefAction) {
        match action {
            PrefAction::InitializeLfs => {
                self.lfs_init_requested = true;
                self.message = Some(gitpoulpi_i18n::msg::settings_screen::LFS_SETUP.get().into());
            }
            PrefAction::Ai(button) => self.ai_run_button(button),
            PrefAction::OpenIn(button) => self.open_in_button(button),
            PrefAction::ImageEditor(button) => self.images_button(button),
            PrefAction::Launcher(button) => self.launcher_button(button),
            PrefAction::Installation(button) => self.install_button(button),
            PrefAction::Tools(button) => self.tools_button(button),
            PrefAction::FolderMenu(button) => self.folder_menu_button(button),
            #[cfg(feature = "mcp")]
            PrefAction::McpSecurity(button) => self.mcp_button(button),
            PrefAction::Workflow(button) => self.wf_button(button),
            PrefAction::Sequence(button) => self.seq_button(button),
            PrefAction::RepoPage(button) => self.rp_button(button),
            PrefAction::RepoAuth(button) => self.ra_button(button),
            PrefAction::ForgetCredentials => {
                let refs: Vec<String> = self
                    .config
                    .integrations
                    .iter()
                    .map(|e| e.token_ref.clone())
                    .collect();
                let mut cleared = 0;
                for token_ref in refs {
                    if self.tokens.delete(&token_ref).is_ok() {
                        cleared += 1;
                    }
                    // Optimistic: the delete already answers presence, and a
                    // `has` call would cost ~2.4s per integration, serially.
                    self.token_presence.insert(token_ref, Some(false));
                }
                self.message = Some(gitpoulpi_i18n::msg::settings_screen::tokens_cleared(
                    cleared,
                    self.config.integrations.len(),
                ));
            }
            PrefAction::GenerateSshKey => self.generate_ssh_key(),
        }
    }

    /// Generates the global key in process (`gitpoulpi_hosting::ssh_key`): no `ssh-keygen` binary, no passphrase prompt. Always
    /// of the `ssh.key_type` setting; an existing file is never overwritten.
    fn generate_ssh_key(&mut self) {
        let Some(home) = gitpoulpi_config::home_dir() else {
            self.message = Some(gitpoulpi_i18n::msg::settings_screen::NO_HOME.get().into());
            return;
        };
        self.generate_ssh_key_in(&home.join(".ssh"));
    }

    pub(super) fn generate_ssh_key_in(&mut self, ssh_dir: &Path) {
        let key_type = gitpoulpi_hosting::ssh_key::KeyType::from_setting(&self.config.ssh.key_type);
        let path = ssh_dir.join(format!("gitpoulpi_{}", key_type.name()));
        match gitpoulpi_hosting::ssh_key::generate_typed_and_write(&path, "gitpoulpi", key_type) {
            Ok(key) => {
                self.write_global(
                    "ssh.private_key_path",
                    &key.private_key_path.display().to_string(),
                );
                self.write_global(
                    "ssh.public_key_path",
                    &key.public_key_path.display().to_string(),
                );
                self.message = Some(gitpoulpi_i18n::msg::settings_screen::key_generated(
                    key.private_key_path.display(),
                    &key.fingerprint,
                ));
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    // ---------------------------------------------------------------
    // Mouse
    // ---------------------------------------------------------------

    /// `false` means "close the screen" - a click outside it, like every
    /// other modal in this app.
    #[cfg(test)]
    pub(crate) fn on_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.mouse_shift = false;
        self.on_mouse_shift(kind, col, row, false)
    }

    /// [`Self::on_mouse`] with the Shift state of the event (Shift+click
    /// extends a selection in the text editors).
    pub(crate) fn on_mouse_shift(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        shift: bool,
    ) -> bool {
        self.mouse_shift = shift;
        let handled = self.on_mouse_inner(kind, col, row);
        self.sync_theme_preview();
        handled
    }

    fn on_mouse_inner(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.mouse_pos = Some((col, row));
        self.notice = None;
        // The octopus in the title line opens About, when nothing modal of
        // Settings' own is open over it.
        if kind == MouseEventKind::Down(MouseButton::Left)
            && self
                .logo_rect
                .is_some_and(|r| r.contains((col, row).into()))
            && self.confirm_action.is_none()
            && self.close_ask.is_none()
            && self.confirm_undo_off.is_none()
            && self.search.is_none()
            && self.choice.is_none()
        {
            self.open_about = true;
            return true;
        }
        if self.confirm_action.is_some() {
            self.confirm_action_mouse(kind, col, row);
            return true;
        }
        if self.ra_overlay_open() {
            self.ra_overlay_mouse(crossterm::event::MouseEvent {
                kind,
                column: col,
                row,
                modifiers: KeyModifiers::NONE,
            });
            return true;
        }
        if self.close_ask.is_some() {
            return self.close_ask_mouse(crossterm::event::MouseEvent {
                kind,
                column: col,
                row,
                modifiers: KeyModifiers::NONE,
            });
        }
        if self.confirm_undo_off.is_some() {
            self.undo_off_mouse(kind, col, row);
            return true;
        }
        // The extra file picker is modal.
        if self.ai.browse_open() {
            self.review_browse_mouse(kind, col, row);
            return true;
        }
        // The search popup is modal and its hit rects come first.
        if self.search.is_some() {
            self.search_mouse(kind, col, row);
            return true;
        }
        // The system-prompt editor is modal.
        if self.ai.prompt_open() {
            self.ai_prompt_mouse(kind, col, row, self.mouse_shift);
            return true;
        }
        if self.wf_editor_open() {
            self.wf_editor_mouse(kind, col, row, self.mouse_shift);
            return true;
        }
        if self.rp_editor_open() {
            self.rp_editor_mouse(kind, col, row, self.mouse_shift);
            return true;
        }
        if self.seq_overlay_open() {
            return self.seq_overlay_mouse(kind, col, row);
        }
        // An open dropdown is modal: it takes every mouse event.
        if let Some((_, combo)) = self.choice.as_mut() {
            let outcome = combo.on_mouse(crossterm::event::MouseEvent {
                kind,
                column: col,
                row,
                modifiers: crossterm::event::KeyModifiers::NONE,
            });
            self.finish_choice(outcome);
            return true;
        }
        if self.body_scrollbar_mouse(kind, col, row) {
            return true;
        }
        // The value being edited is a text field: a press in it places
        // the caret (a double-click selects a word), a drag selects.
        if let Some((index, draft, _)) = self.edit.as_ref() {
            let on_value = self.caret_edit.field_at(col, row) == Some(*index);
            let pressed = kind == MouseEventKind::Down(MouseButton::Left);
            if (pressed && on_value) || (!pressed && self.caret_edit.is_dragging()) {
                let (index, draft) = (*index, draft.clone());
                self.caret_edit.sync(Some(index));
                self.caret_edit
                    .dialog_mouse(Some(index), &draft, kind, col, row);
                return true;
            }
            // A press anywhere else leaves the field: it commits first,
            // and a refused value keeps the field (the click is spent).
            if pressed && !self.blur_edit() {
                return true;
            }
        }
        self.flash = None;
        if self.keys_shown()
            && self.keys_wants_mouse(col, row)
            && matches!(
                kind,
                MouseEventKind::Down(MouseButton::Left)
                    | MouseEventKind::ScrollDown
                    | MouseEventKind::ScrollUp
                    | MouseEventKind::Moved
            )
        {
            if kind == MouseEventKind::Down(MouseButton::Left) {
                self.focus = PrefFocus::Fields;
                self.strip = false;
            }
            return self.keys_mouse(kind, col, row);
        }
        if self.list_shown()
            && matches!(
                kind,
                MouseEventKind::Down(MouseButton::Left)
                    | MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left)
                    | MouseEventKind::Moved
                    | MouseEventKind::ScrollDown
                    | MouseEventKind::ScrollUp
            )
            && self.list_mouse(kind, col, row)
        {
            return true;
        }
        if kind == MouseEventKind::Down(MouseButton::Left)
            && self
                .search_box
                .is_some_and(|r| r.contains((col, row).into()))
        {
            self.open_search();
            return true;
        }
        if self.current_section() == PrefSection::Integrations
            && ((self.body_area.contains((col, row).into())
                && !self.strip_rect.contains((col, row).into()))
                || self.integrations_page_owns_drag(kind))
            && matches!(
                kind,
                MouseEventKind::Down(MouseButton::Left)
                    | MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left)
                    | MouseEventKind::ScrollUp
                    | MouseEventKind::ScrollDown
                    | MouseEventKind::Moved
            )
            && self.integrations_mouse(kind, col, row)
        {
            self.focus = PrefFocus::Fields;
            self.strip = false;
            return true;
        }
        let inside = self.area.contains((col, row).into());

        match kind {
            MouseEventKind::Down(MouseButton::Left) if !inside => return self.request_close(),
            MouseEventKind::Down(MouseButton::Left) => {
                // Which side was clicked, before which row: the sidebar and the
                // settings list number their rows independently.
                // The close control first: it sits on the border, which
                // belongs to neither half below.
                if self
                    .close_rect
                    .is_some_and(|r| r.contains((col, row).into()))
                {
                    return self.request_close();
                }
                let in_sidebar = self.sidebar_area.contains((col, row).into());
                let in_body = self.body_area.contains((col, row).into());

                if in_sidebar {
                    if let Some((_, index)) = self.sidebar_rows.iter().find(|(y, _)| *y == row) {
                        let index = *index;
                        self.focus = PrefFocus::Sections;
                        self.switch_section(index);
                    }
                    return true;
                }

                if !in_body {
                    return true;
                }

                if self.sub_tab_click(col, row) {
                    return true;
                }
                if let Some((_, field, chip)) = self
                    .chip_rects
                    .iter()
                    .find(|(r, _, _)| r.contains((col, row).into()))
                    .copied()
                {
                    self.focus = PrefFocus::Fields;
                    self.field = field;
                    self.ai.chip = chip;
                    self.activate();
                    return true;
                }
                if let Some((_, field, button)) = self
                    .btn_rects
                    .iter()
                    .find(|(r, _, _)| r.contains((col, row).into()))
                    .copied()
                {
                    self.focus = PrefFocus::Fields;
                    self.field = field;
                    self.ai.chip = button;
                    self.activate();
                    return true;
                }
                if let Some((_, index)) = self.field_rows.iter().find(|(y, _)| *y == row) {
                    let index = *index;
                    self.focus = PrefFocus::Fields;
                    self.strip = self.current_section().has_strip();
                    self.field = index;
                    // A click acts: it ticks a checkbox, opens a dropdown or
                    // opens a text field. A buttons row is the exception: a
                    // click on the row only selects it, a click on one of
                    // its buttons (above) presses that button.
                    let is_action = matches!(
                        self.fields().get(index).map(|f| &f.control),
                        Some(Control::Buttons(_))
                    );
                    if !is_action {
                        self.activate();
                    }
                }
            }
            MouseEventKind::ScrollDown if inside => {
                self.scroll = self.scroll.saturating_add(3);
            }
            MouseEventKind::ScrollUp if inside => {
                self.scroll = self.scroll.saturating_sub(3);
            }
            _ => {}
        }
        true
    }
}

impl PreferencesState {
    /// A press, drag or release on the body's scrollbar (the generic page's,
    /// or the Keyboard page's list): the thumb drags, the track jumps, the
    /// arrows step. Whether the event was the bar's.
    fn body_scrollbar_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let top = if self.keys_shown() {
            self.keys.bar_mouse(event)
        } else {
            let (track, lines, scroll) = (self.body_bar, self.body_lines, self.scroll);
            self.body_drag.on_mouse(track, event, lines, scroll)
        };
        match top {
            Some(top) => {
                if !self.keys_shown() {
                    self.scroll = top;
                }
                true
            }
            None => false,
        }
    }
}
