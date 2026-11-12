use super::*;

impl IntegrationsPage {
    pub(super) fn activate(&mut self, item: Item, env: &mut Env) {
        match item {
            Item::SshDefault => self.ssh.mode = SshMode::Default,
            Item::SshGenerate => self.ssh.mode = SshMode::Generated,
            Item::SshCustom => self.ssh.mode = SshMode::Custom,
            Item::SshTitle | Item::SshPath => self.step(1, env.config),
            Item::SshUpload => self.ssh.upload = !self.ssh.upload,
            Item::SshGo => self.start_generate(env),
            Item::SshBrowse => self.open_key_picker(),
            Item::FolderBrowse => self.open_folder_picker(),
            Item::SshUse => self.use_custom_key(env),
            Item::SshRemove => self.start_remove_key(env, false),
            Item::SshDelete => {
                self.mode = Mode::ConfirmKeyDelete;
                self.item = 1;
            }
            Item::SshConfirmDelete => {
                self.mode = Mode::Browse;
                self.item = 0;
                self.start_remove_key(env, true);
            }
            Item::SshCancelDelete => {
                self.mode = Mode::Browse;
                self.item = 0;
            }
            Item::RadarReview
            | Item::RadarOpen
            | Item::RadarDrafts
            | Item::RadarIssues
            | Item::RadarNamespaces
            | Item::RadarRegex
            | Item::RadarMode => self.activate_radar(item, env),
            Item::Host | Item::Org | Item::Account | Item::Token => self.step(1, env.config),
            Item::StrictPin => self.strict = !self.strict,
            Item::AllowPlain => self.plain_http = !self.plain_http,
            Item::GetKey => self.open_url(TRELLO_KEY_URL, env),
            Item::Generate => self.open_token_page(env),
            Item::Connect => {
                if self.can_connect(env.config, env.presence) {
                    self.connect(env);
                } else {
                    self.error = Some(self.missing_input().unwrap_or(msg::ERR_TOKEN).get().into());
                }
            }
            Item::StartConnect => self.begin_form(env.config, None),
            Item::CancelForm => {
                self.reset_view(env.config);
                self.zone = Zone::Profiles;
            }
            Item::CancelConnect => {
                let editing = self.pending.take().and_then(|p| p.editing.clone());
                self.mode = Mode::Form(editing);
                self.item = 0;
            }
            Item::Edit => {
                if let Some(i) = self.selected_profile(env.config) {
                    self.begin_form(env.config, Some(i));
                    // Pre-fill the stored token, visible. A failed load just
                    // leaves the field empty (the stored token is reused).
                    let token_ref = env.config.integrations[i].token_ref.clone();
                    if let Ok(Some(t)) = env.tokens.load(&token_ref) {
                        self.token = t;
                    }
                }
            }
            Item::Disconnect => self.disconnect(env),
            Item::Remove => {
                self.mode = Mode::ConfirmRemove;
                self.item = 1;
            }
            Item::ConfirmRemove => self.remove(env),
            Item::CancelRemove => {
                // Back to whatever the profile shows: the card when
                // connected, its form when it is not.
                self.reset_view(env.config);
                self.item = 0;
            }
        }
    }

    /// Moves the body's focus by `delta`, spilling into the neighboring
    /// zone at either end.
    pub(super) fn step(&mut self, delta: isize, cfg: &Config) {
        let len = self.items(cfg).len() as isize;
        let next = self.item as isize + delta;
        if next < 0 {
            self.zone = Zone::Profiles;
        } else if next >= len {
            self.zone = Zone::Types;
        } else {
            self.item = next as usize;
        }
    }

    pub(crate) fn on_key(&mut self, key: &KeyEvent, env: &mut Env) -> Nav {
        self.sync_radar(env.config);
        if self.radar_combo.is_open() {
            self.combo_key(*key, env);
            return Nav::Stay;
        }
        let nav = self.on_key_inner(key, env);
        self.flush_radar(env);
        nav
    }

    fn on_key_inner(&mut self, key: &KeyEvent, env: &mut Env) -> Nav {
        if let Some(picker) = self.ssh.picker.as_mut() {
            let outcome = picker.on_key(*key);
            self.picker_outcome(outcome);
            return Nav::Stay;
        }
        if self.text_focused(env.config) {
            let item = self.current_item(env.config).unwrap_or(Item::Host);
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let copies = matches!(key.code, KeyCode::Char('\u{3}' | '\u{18}'))
                || (ctrl && matches!(key.code, KeyCode::Char('c' | 'C' | 'x' | 'X')));
            if item == Item::Token && copies {
                return Nav::Stay; // a token is never copied or cut
            }
            if item == Item::RadarNamespaces && self.namespaces_key(key, env) {
                return Nav::Stay;
            }
            let text = field_mut!(self, item);
            if item != Item::RadarNamespaces {
                if let Some((_, copied)) = self.editor.dialog_key(item, text, key) {
                    if copied.is_some() {
                        *env.message = copied;
                    }
                    self.error = None;
                    return Nav::Stay;
                }
            }
        }
        let typing = self.text_focused(env.config);
        let km = crate::keymap::current();
        match key.code {
            KeyCode::Esc => return self.back(env),
            KeyCode::Tab => self.tab(1, env.config),
            KeyCode::BackTab => self.tab(-1, env.config),
            _ if !typing && km.is(Bind::IntegrationsAdd, key) => self.add_profile(env.config),
            _ if !typing && km.is(Bind::IntegrationsEdit, key) => {
                if let Some(i) = self.selected_profile(env.config) {
                    self.begin_form(env.config, Some(i));
                    self.zone = Zone::Body;
                }
            }
            _ if !typing && km.is(Bind::IntegrationsOpenToken, key) => self.open_token_page(env),
            _ if !typing && km.is(Bind::IntegrationsRemove, key) => {
                if self.view(env.config) != View::Coming
                    && self.selected_profile(env.config).is_some()
                {
                    self.mode = Mode::ConfirmRemove;
                    self.item = 1;
                    self.zone = Zone::Body;
                }
            }
            _ => match self.zone {
                Zone::Types => return self.types_key(key, env),
                Zone::Profiles => self.profiles_key(key, env),
                Zone::Body => self.body_key(key, env),
            },
        }
        Nav::Stay
    }

    pub(super) fn add_profile(&mut self, cfg: &Config) {
        self.sel = self.profile_ids(cfg).len();
        self.begin_form(cfg, None);
        self.zone = Zone::Body;
    }

    /// Esc: one zone back; from the type list back to the sidebar.
    fn back(&mut self, env: &mut Env) -> Nav {
        match (self.zone, &self.mode) {
            (Zone::Body, Mode::Connecting(_)) => self.activate(Item::CancelConnect, env),
            (Zone::Body, Mode::ConfirmRemove) => self.activate(Item::CancelRemove, env),
            (Zone::Body, Mode::ConfirmKeyDelete) => self.activate(Item::SshCancelDelete, env),
            (Zone::Body, _) => self.zone = Zone::Profiles,
            (Zone::Profiles, _) => self.zone = Zone::Types,
            (Zone::Types, _) => return Nav::Sections,
        }
        Nav::Stay
    }

    fn tab(&mut self, dir: isize, cfg: &Config) {
        match (self.zone, dir > 0) {
            (Zone::Types, true) => self.zone = Zone::Profiles,
            (Zone::Profiles, true) => {
                self.zone = Zone::Body;
                self.item = 0;
                if self.items(cfg).is_empty() {
                    self.zone = Zone::Types;
                }
            }
            (Zone::Body, true) => self.step(1, cfg),
            (Zone::Types, false) => {
                let last = self.items(cfg).len().saturating_sub(1);
                self.zone = if last == 0 && self.items(cfg).is_empty() {
                    Zone::Profiles
                } else {
                    self.item = last;
                    Zone::Body
                };
            }
            (Zone::Profiles, false) => self.zone = Zone::Types,
            (Zone::Body, false) => self.step(-1, cfg),
        }
    }

    fn types_key(&mut self, key: &KeyEvent, env: &mut Env) -> Nav {
        let last = self.kinds.len() - 1;
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_kind((self.kind + 1).min(last), env.config)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_kind(self.kind.saturating_sub(1), env.config)
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => {
                self.zone = Zone::Profiles
            }
            KeyCode::Left | KeyCode::Char('h') => return Nav::Sections,
            _ => {}
        }
        Nav::Stay
    }

    fn profiles_key(&mut self, key: &KeyEvent, env: &mut Env) {
        let count = self.profile_ids(env.config).len();
        match key.code {
            KeyCode::Right | KeyCode::Char('l') => {
                self.select_profile((self.sel + 1).min(count), env.config)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.select_profile(self.sel.saturating_sub(1), env.config)
            }
            KeyCode::Up | KeyCode::Char('k') => self.zone = Zone::Types,
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.items(env.config).is_empty() {
                    self.zone = Zone::Body;
                    self.item = 0;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.sel >= count {
                    self.add_profile(env.config);
                } else if !self.items(env.config).is_empty() {
                    self.zone = Zone::Body;
                    self.item = 0;
                }
            }
            _ => {}
        }
    }

    fn body_key(&mut self, key: &KeyEvent, env: &mut Env) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.step(1, env.config),
            KeyCode::Up | KeyCode::Char('k') => self.step(-1, env.config),
            KeyCode::Right | KeyCode::Char('l')
                if !self.current_item(env.config).is_some_and(Item::is_text) =>
            {
                self.step(1, env.config)
            }
            KeyCode::Left | KeyCode::Char('h')
                if !self.current_item(env.config).is_some_and(Item::is_text) =>
            {
                self.step(-1, env.config)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(item) = self.current_item(env.config) {
                    self.activate(item, env);
                }
            }
            _ => {}
        }
        // Up/Down inside a text field were not claimed by the editor
        // (single line), so they arrive here and move between fields.
    }

    /// A bracketed paste: into the focused text field (control characters
    /// dropped, so a pasted token with a trailing newline stays one line).
    pub(crate) fn paste(&mut self, text: &str, cfg: &Config) -> bool {
        if let Some(picker) = self.ssh.picker.as_mut() {
            picker.paste(text);
            return true;
        }
        if !self.text_focused(cfg) {
            return false;
        }
        let item = self.current_item(cfg).unwrap_or(Item::Host);
        if item == Item::RadarNamespaces {
            // The area keeps the pasted lines.
            let lines = text.replace("\r\n", "\n").replace('\r', "\n");
            let clean: String = lines
                .chars()
                .filter(|c| *c == '\n' || !c.is_control())
                .collect();
            let field = field_mut!(self, item);
            self.editor.caret_mut(item).insert(field, &clean);
            return true;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        let field = field_mut!(self, item);
        self.editor.caret_mut(item).insert(field, clean.trim());
        self.error = None;
        true
    }

    pub(super) fn field(&self, item: Item) -> &String {
        match item {
            Item::Token => &self.token,
            Item::Org => &self.org,
            Item::Account => &self.account,
            Item::SshTitle => &self.ssh.title,
            Item::SshPath => &self.ssh.path,
            Item::RadarNamespaces => &self.radar_ns,
            _ => &self.host,
        }
    }

    /// A left-button event inside the page's area. Returns whether the
    /// page used it.
    pub(crate) fn on_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        env: &mut Env,
    ) -> bool {
        if self.radar_combo.is_open() {
            self.combo_mouse(kind, col, row, env);
            return true;
        }
        if let Some(picker) = self.ssh.picker.as_mut() {
            let event = crossterm::event::MouseEvent {
                kind,
                column: col,
                row,
                modifiers: KeyModifiers::NONE,
            };
            let outcome = picker.on_mouse(event, false);
            self.picker_outcome(outcome);
            return true;
        }
        if self.namespaces_wheel(kind, col, row) {
            return true;
        }
        if self.namespaces_bar_mouse(kind, col, row, env) {
            return true;
        }
        if self.scroll_mouse(kind, col, row) {
            return true;
        }
        let vrow = self.editor_row(row);
        if let Some(item) = self.current_item(env.config).filter(|i| i.is_text()) {
            let text = self.field(item).clone();
            if !matches!(kind, MouseEventKind::Down(_))
                && self.editor.dialog_mouse(Some(item), &text, kind, col, vrow)
            {
                return true;
            }
        }
        if kind != MouseEventKind::Down(MouseButton::Left) {
            return false;
        }
        let Some(hit) = self
            .hits
            .iter()
            .find(|(r, _, _)| r.contains((col, row).into()))
            .map(|(_, h, _)| *h)
        else {
            return false;
        };
        match hit {
            Hit::Type(i) => {
                self.zone = Zone::Types;
                self.select_kind(i, env.config);
            }
            Hit::Profile(i) => {
                self.zone = Zone::Profiles;
                self.select_profile(i, env.config);
            }
            Hit::AddProfile => self.add_profile(env.config),
            Hit::Item(item) => {
                let items = self.items(env.config);
                if let Some(at) = items.iter().position(|i| *i == item) {
                    self.item = at;
                    self.zone = Zone::Body;
                }
                if item.is_text() {
                    let text = self.field(item).clone();
                    self.editor.sync(Some(item));
                    self.editor.dialog_mouse(Some(item), &text, kind, col, vrow);
                } else {
                    self.activate(item, env);
                }
            }
        }
        true
    }
}
