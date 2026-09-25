use super::*;
use crate::th;

impl IntegrationsPage {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_form(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        focused: bool,
        current: Option<Item>,
    ) {
        let kind = self.kind();
        let caps = kind.capabilities();
        if caps.local_folder {
            self.render_local_form(frame, body, cfg, focused, current);
            return;
        }
        let field_w = body.width.saturating_sub(2).clamp(8, 44);
        let mut y = body.y;
        let field = |this: &mut Self,
                     frame: &mut Canvas,
                     y: &mut u16,
                     label_text: &str,
                     item: Item,
                     hint: &str,
                     example: Option<&str>| {
            put(frame, body, body.x, *y, label(label_text));
            *y += 1;
            this.text_field(
                frame, body, body.x, *y, field_w, item, current, focused, hint,
            );
            *y += 1;
            if let Some(example) = example {
                put(frame, body, body.x, *y, muted(example));
                *y += 1;
            }
            *y += 1;
        };
        if caps.needs_host {
            let (name, example) = match kind {
                IntegrationKind::JiraCloud => (msg2::HOST_SITE, msg2::HOST_SITE_EXAMPLE),
                IntegrationKind::AzureDevopsServer => {
                    (msg2::HOST_COLLECTION, msg2::HOST_COLLECTION_EXAMPLE)
                }
                IntegrationKind::JiraDataCenter => (msg2::HOST_SERVER, msg2::HOST_SERVER_EXAMPLE),
                IntegrationKind::Youtrack => (msg2::HOST_SITE, msg2::HOST_YOUTRACK_EXAMPLE),
                IntegrationKind::Redmine => (msg2::HOST_SITE, msg2::HOST_REDMINE_EXAMPLE),
                IntegrationKind::Gerrit => (msg2::HOST_SERVER, msg2::HOST_GERRIT_EXAMPLE),
                _ => (msg::HOST_DOMAIN, msg::HOST_EXAMPLE),
            };
            field(
                self,
                frame,
                &mut y,
                name.get(),
                Item::Host,
                match kind {
                    IntegrationKind::Forgejo => "codeberg.org",
                    IntegrationKind::Gitea => "gitea.example.com",
                    IntegrationKind::Youtrack => "acme.youtrack.cloud",
                    IntegrationKind::Redmine => "redmine.example.com",
                    IntegrationKind::Gerrit => "gerrit.example.com",
                    _ => "gitlab.example.com",
                },
                Some(example.get()),
            );
        }
        if caps.org {
            field(
                self,
                frame,
                &mut y,
                msg2::ORG_LABEL.get(),
                Item::Org,
                "contoso",
                Some(msg2::ORG_EXAMPLE.get()),
            );
        }
        if let Some(account) = caps.account {
            let (name, example) = match account {
                AccountField::Email => (msg2::ACCOUNT_EMAIL, msg2::ACCOUNT_EMAIL_EXAMPLE),
                AccountField::ApiKey => (msg2::ACCOUNT_KEY, msg2::ACCOUNT_KEY_EXAMPLE),
                AccountField::Username => (msg2::ACCOUNT_USERNAME, msg2::ACCOUNT_USERNAME_EXAMPLE),
            };
            field(
                self,
                frame,
                &mut y,
                name.get(),
                Item::Account,
                "",
                Some(example.get()),
            );
        }
        put(frame, body, body.x, y, label(msg::TOKEN_LABEL.get()));
        y += 1;
        self.text_field(
            frame,
            body,
            body.x,
            y,
            field_w,
            Item::Token,
            current,
            focused,
            msg::TOKEN_PLACEHOLDER.get(),
        );
        y += 1;
        if self.stored_token(cfg, presence) {
            put(frame, body, body.x, y, muted(msg::TOKEN_KEPT.get()));
        }
        y += 2;

        // The hint box: scopes + the link that pre-fills them.
        let scopes = kind
            .token_help()
            .map(|h| h.scopes.join(", "))
            .unwrap_or_default();
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            &format!("{}{scopes}", msg::SCOPES.get()),
            Style::default().fg(th().status.warning),
            3,
        );
        if caps.account == Some(AccountField::ApiKey) {
            let w = self.button(
                frame,
                body,
                body.x,
                y,
                msg2::GET_KEY.get(),
                Item::GetKey,
                current,
                focused,
                false,
            );
            let _ = w;
            y += 1;
        }
        let token_url = self.token_url();
        let host = token_url.as_ref().map(|u| {
            let rest = u.split_once("://").map_or(u.as_str(), |(_, r)| r);
            rest.split(['/', '?']).next().unwrap_or(rest).to_string()
        });
        let link = match &host {
            Some(h) => msg::generate_on(fit(h, 30)),
            None if caps.account == Some(AccountField::ApiKey) => {
                msg2::GENERATE_NEED_KEY.get().to_string()
            }
            None => msg::GENERATE_NEED_HOST.get().to_string(),
        };
        self.button(
            frame,
            body,
            body.x,
            y,
            &link,
            Item::Generate,
            current,
            focused,
            host.is_none(),
        );
        y += 2;

        let mut used = 0;
        if let Some(err) = &self.error {
            used = put_wrapped(
                frame,
                body,
                body.x,
                y,
                &format!("\u{26A0} {err}"),
                Style::default().fg(th().status.danger),
                3,
            );
        }
        y += used.max(1) + 1;
        let enabled = self.can_connect(cfg, presence);
        let w = self.button(
            frame,
            body,
            body.x,
            y,
            msg::CONNECT.get(),
            Item::Connect,
            current,
            focused,
            !enabled,
        );
        let w2 = self.button(
            frame,
            body,
            body.x + w + 2,
            y,
            gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            Item::CancelForm,
            current,
            focused,
            false,
        );
        if matches!(self.mode, Mode::Form(Some(_))) {
            self.button(
                frame,
                body,
                body.x + w + 2 + w2 + 2,
                y,
                msg::REMOVE.get(),
                Item::Remove,
                current,
                focused,
                false,
            );
        }
    }

    /// The local Markdown profile's form: a folder, a Browse button, no token.
    fn render_local_form(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        cfg: &Config,
        focused: bool,
        current: Option<Item>,
    ) {
        use gitpoulpi_i18n::msg::local_issues as lmsg;
        let field_w = body.width.saturating_sub(2).clamp(8, 60);
        let mut y = body.y;
        put(frame, body, body.x, y, label(lmsg::FOLDER_LABEL.get()));
        y += 1;
        self.text_field(
            frame,
            body,
            body.x,
            y,
            field_w,
            Item::Host,
            current,
            focused,
            "docs/issues",
        );
        y += 1;
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            lmsg::FOLDER_EXAMPLE.get(),
            Style::default().fg(th().ui.muted),
            2,
        );
        y += 1;
        self.button(
            frame,
            body,
            body.x,
            y,
            lmsg::BROWSE.get(),
            Item::FolderBrowse,
            current,
            focused,
            false,
        );
        y += 2;
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            lmsg::LOCAL_NOTE.get(),
            Style::default().fg(th().ui.muted),
            3,
        );
        y += 1;
        if let Some(err) = &self.error {
            y += put_wrapped(
                frame,
                body,
                body.x,
                y,
                &format!("\u{26A0} {err}"),
                Style::default().fg(th().status.danger),
                3,
            );
        }
        y += 1;
        let enabled = self.missing_input().is_none();
        let w = self.button(
            frame,
            body,
            body.x,
            y,
            lmsg::ADD_FOLDER.get(),
            Item::Connect,
            current,
            focused,
            !enabled,
        );
        let w2 = self.button(
            frame,
            body,
            body.x + w + 2,
            y,
            gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            Item::CancelForm,
            current,
            focused,
            false,
        );
        if matches!(self.mode, Mode::Form(Some(_))) {
            self.button(
                frame,
                body,
                body.x + w + 2 + w2 + 2,
                y,
                msg::REMOVE.get(),
                Item::Remove,
                current,
                focused,
                false,
            );
        }
        let _ = cfg;
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_connected(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        index: usize,
        focused: bool,
        current: Option<Item>,
    ) {
        let e = &cfg.integrations[index];
        if e.kind.capabilities().local_folder {
            self.render_local_connected(frame, body, e, focused, current);
            return;
        }
        let user = e.user.clone().unwrap_or_default();
        let mut y = body.y;
        let who = if user.name.is_empty() {
            format!("@{}", user.login)
        } else {
            format!("{} (@{})", user.name, user.login)
        };
        put(
            frame,
            body,
            body.x,
            y,
            vec![
                Span::styled("\u{25CF} ", Style::default().fg(th().refs.local)),
                Span::styled(
                    who,
                    Style::default()
                        .fg(th().ui.text)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", msg::CONNECTED.get()),
                    Style::default().fg(th().refs.local),
                ),
            ],
        );
        y += 1;
        put(
            frame,
            body,
            body.x,
            y,
            kv(msg::KV_HOST.get(), &e.host().unwrap_or_default()),
        );
        y += 1;
        if let Some(org) = e.azure_org() {
            put(frame, body, body.x, y, kv(msg2::KV_ORG.get(), &org));
            y += 1;
        }
        if !e.account.is_empty() {
            put(
                frame,
                body,
                body.x,
                y,
                kv(msg2::KV_ACCOUNT.get(), &e.account),
            );
            y += 1;
        }
        let token = match presence.get(&e.token_ref).copied().flatten() {
            Some(true) => format!(
                "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022} {}",
                msg::TOKEN_STORED.get()
            ),
            Some(false) => msg::TOKEN_MISSING.get().to_string(),
            None => msg::TOKEN_CHECKING.get().to_string(),
        };
        put(frame, body, body.x, y, kv(msg::KV_TOKEN.get(), &token));
        y += 1;
        put(
            frame,
            body,
            body.x,
            y,
            kv(msg::KV_AUTH.get(), msg::AUTH_PAT.get()),
        );
        y += 1;
        if let Some(missing) = self.warnings.get(&e.id) {
            put(
                frame,
                body,
                body.x,
                y,
                vec![Span::styled(
                    format!("\u{26A0} {}", msg::token_lacks(missing.join(", "))),
                    Style::default().fg(th().status.danger),
                )],
            );
            y += 1;
        }
        y += 1;
        let mut x = body.x;
        for (label, item) in [
            (msg::EDIT.get(), Item::Edit),
            (msg::DISCONNECT.get(), Item::Disconnect),
            (msg::REMOVE.get(), Item::Remove),
        ] {
            x += self.button(frame, body, x, y, label, item, current, focused, false) + 2;
        }
        y += 2;

        if e.kind.capabilities().ssh_keys {
            y = self.render_ssh(frame, body, y, e, focused, current);
        }
        self.render_radar(frame, body, y, e, focused, current);
    }

    /// A local Markdown profile: its folder and the edit / remove buttons.
    fn render_local_connected(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        e: &Integration,
        focused: bool,
        current: Option<Item>,
    ) {
        use gitpoulpi_i18n::msg::local_issues as lmsg;
        let mut y = body.y;
        put(
            frame,
            body,
            body.x,
            y,
            vec![
                Span::styled("\u{25CF} ", Style::default().fg(th().refs.local)),
                Span::styled(
                    e.display_name().to_string(),
                    Style::default()
                        .fg(th().ui.text)
                        .add_modifier(Modifier::BOLD),
                ),
            ],
        );
        y += 1;
        put(frame, body, body.x, y, kv(lmsg::KV_FOLDER.get(), &e.url));
        y += 2;
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            lmsg::LOCAL_NOTE.get(),
            Style::default().fg(th().ui.muted),
            3,
        );
        y += 1;
        let mut x = body.x;
        for (text, item) in [
            (msg::EDIT.get(), Item::Edit),
            (msg::REMOVE.get(), Item::Remove),
        ] {
            x += self.button(frame, body, x, y, text, item, current, focused, false) + 2;
        }
    }

    /// The per-profile SSH section: the attached key's line, or the
    /// default / generate / custom choice.
    fn render_ssh(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        mut y: u16,
        e: &Integration,
        focused: bool,
        current: Option<Item>,
    ) -> u16 {
        let field_w = body.width.saturating_sub(2).clamp(8, 44);
        let host = e.host().unwrap_or_default();
        put(
            frame,
            body,
            body.x,
            y,
            vec![Span::styled(
                msg::SSH_HEADING.get(),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            )],
        );
        y += 1;
        if let Some(job) = &self.ssh.job {
            let spin =
                ["|", "/", "-", "\\"][(self.started.elapsed().as_millis() / 150 % 4) as usize];
            put(
                frame,
                body,
                body.x,
                y,
                vec![Span::styled(
                    format!("{spin} {}\u{2026}", job.what.get()),
                    Style::default().fg(th().status.warning),
                )],
            );
            y += 1;
        }
        if has_key(e) {
            let ssh = e.ssh.clone().unwrap_or_default();
            let private = ssh.private_key_path.clone().unwrap_or_default();
            let title = ssh.title.clone().unwrap_or_else(|| {
                std::path::Path::new(&private)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
            let fingerprint = ssh.public_key_path.as_ref().and_then(|p| {
                self.ssh
                    .fingerprints
                    .entry(p.clone())
                    .or_insert_with(|| ssh_io::fingerprint_of_file(p))
                    .clone()
            });
            // The fingerprint is what identifies the key: clip the title
            // to make room for it, never the other way round.
            let room = (body.width as usize).saturating_sub(16);
            let line = match fingerprint {
                Some(fp) => format!(
                    "{} ({fp})",
                    fit(&title, room.saturating_sub(display_width(&fp) + 3).max(8))
                ),
                None => title,
            };
            let kind = match ssh.mode {
                SshMode::Custom => msg::SSH_CUSTOM_KEY.get(),
                _ => msg::SSH_GENERATED_KEY.get(),
            };
            put(
                frame,
                body,
                body.x,
                y,
                vec![
                    Span::styled(format!("{kind}  "), Style::default().fg(th().ui.muted)),
                    Span::styled(
                        fit(&line, (body.width as usize).saturating_sub(16)),
                        Style::default().fg(th().ui.text),
                    ),
                ],
            );
            y += 1;
            put(
                frame,
                body,
                body.x,
                y,
                muted(&tail_fit(&private, (body.width as usize).saturating_sub(2))),
            );
            y += 1;
            let remote = match &ssh.remote_key_id {
                Some(id) => msg::key_added_to(&host, id),
                None => msg::key_not_added(&host),
            };
            put(frame, body, body.x, y, muted(&remote));
            y += 2;
            let w = self.button(
                frame,
                body,
                body.x,
                y,
                msg::SSH_REMOVE.get(),
                Item::SshRemove,
                current,
                focused,
                self.ssh.job.is_some(),
            );
            self.button(
                frame,
                body,
                body.x + w + 2,
                y,
                msg::DELETE.get(),
                Item::SshDelete,
                current,
                focused,
                self.ssh.job.is_some(),
            );
            put_wrapped(
                frame,
                body,
                body.x,
                y + 1,
                msg::KEY_BUTTONS_NOTE.get(),
                Style::default().fg(th().ui.muted),
                2,
            );
            self.render_ssh_error(frame, body, y + 4);
            return y + 6;
        }

        // No key yet: the choice.
        let mut x = body.x;
        for (label, item, mode) in [
            (msg::RADIO_DEFAULT.get(), Item::SshDefault, SshMode::Default),
            (
                msg::RADIO_GENERATE.get(),
                Item::SshGenerate,
                SshMode::Generated,
            ),
            (msg::RADIO_CUSTOM.get(), Item::SshCustom, SshMode::Custom),
        ] {
            let mark = if self.ssh.mode == mode {
                "\u{25CF}"
            } else {
                "\u{25CB}"
            };
            x += self.control(
                frame,
                body,
                x,
                y,
                format!("{mark} {label}"),
                item,
                current,
                focused,
                false,
            ) + 2;
        }
        y += 2;
        match self.ssh.mode {
            SshMode::Default => {
                put_wrapped(
                    frame,
                    body,
                    body.x,
                    y,
                    msg::SSH_DEFAULT_NOTE.get(),
                    Style::default().fg(th().ui.muted),
                    2,
                );
                y += 2;
            }
            SshMode::Generated => {
                put(frame, body, body.x, y, label(msg::KEY_TITLE.get()));
                y += 1;
                let hint = gitpoulpi_hosting::ssh_key::profile_key_title(&host);
                self.text_field(
                    frame,
                    body,
                    body.x,
                    y,
                    field_w,
                    Item::SshTitle,
                    current,
                    focused,
                    &hint,
                );
                y += 1;
                let mark = if self.ssh.upload { "[x]" } else { "[ ]" };
                self.control(
                    frame,
                    body,
                    body.x,
                    y,
                    format!("{mark} {}", msg::add_public_key(&host)),
                    Item::SshUpload,
                    current,
                    focused,
                    false,
                );
                y += 2;
                self.button(
                    frame,
                    body,
                    body.x,
                    y,
                    msg::GENERATE_KEY.get(),
                    Item::SshGo,
                    current,
                    focused,
                    self.ssh.job.is_some(),
                );
                y += 1;
            }
            SshMode::Custom => {
                put(frame, body, body.x, y, label(msg::PRIVATE_KEY_FILE.get()));
                y += 1;
                self.text_field(
                    frame,
                    body,
                    body.x,
                    y,
                    field_w,
                    Item::SshPath,
                    current,
                    focused,
                    "~/.ssh/id_work",
                );
                y += 2;
                let w = self.button(
                    frame,
                    body,
                    body.x,
                    y,
                    msg::BROWSE.get(),
                    Item::SshBrowse,
                    current,
                    focused,
                    false,
                );
                self.button(
                    frame,
                    body,
                    body.x + w + 2,
                    y,
                    msg::USE_THIS_KEY.get(),
                    Item::SshUse,
                    current,
                    focused,
                    self.ssh.path.trim().is_empty(),
                );
                y += 1;
            }
        }
        self.render_ssh_error(frame, body, y + 1);
        y + 3
    }

    fn render_ssh_error(&self, frame: &mut Canvas, body: Rect, y: u16) {
        if let Some(err) = &self.ssh.error {
            put_wrapped(
                frame,
                body,
                body.x,
                y,
                &format!("\u{26A0} {err}"),
                Style::default().fg(th().status.danger),
                3,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn button(
        &mut self,
        frame: &mut Canvas,
        area: Rect,
        x: u16,
        y: u16,
        text: &str,
        item: Item,
        current: Option<Item>,
        focused: bool,
        disabled: bool,
    ) -> u16 {
        self.control(
            frame,
            area,
            x,
            y,
            format!("[ {text} ]"),
            item,
            current,
            focused,
            disabled,
        )
    }

    /// Any clickable label (button, radio, checkbox): drawn, focusable,
    /// and registered for the strong hover.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn control(
        &mut self,
        frame: &mut Canvas,
        area: Rect,
        x: u16,
        y: u16,
        label: String,
        item: Item,
        current: Option<Item>,
        focused: bool,
        disabled: bool,
    ) -> u16 {
        let w = display_width(&label) as u16;
        if y >= area.bottom() || x >= area.right() {
            return w;
        }
        let is_focus = focused && self.zone == Zone::Body && current == Some(item);
        let style = if disabled {
            Style::default().fg(th().ui.muted)
        } else if is_focus {
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th().ui.accent)
        };
        put(frame, area, x, y, vec![Span::styled(label, style)]);
        let rect = Rect::new(x, y, w.min(area.right() - x), 1);
        // A disabled control still takes the click (it explains itself),
        // but never lights up as if it would act.
        self.hits.push((rect, Hit::Item(item), !disabled));
        w
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn text_field(
        &mut self,
        frame: &mut Canvas,
        area: Rect,
        x: u16,
        y: u16,
        width: u16,
        item: Item,
        current: Option<Item>,
        focused: bool,
        hint: &str,
    ) {
        if y >= area.bottom() {
            return;
        }
        let is_focus = focused && self.zone == Zone::Body && current == Some(item);
        let text = self.field(item);
        let field_bg = crate::input_bg(is_focus, false);
        let bg = Style::default().fg(th().ui.text).bg(field_bg);
        let rect = Rect::new(x, y, width.min(area.right().saturating_sub(x)), 1);
        let inner_w = rect.width.saturating_sub(2) as usize;
        let mut spans = vec![Span::styled(" ", bg)];
        if text.is_empty() && !is_focus {
            spans.push(Span::styled(
                format!("{:<inner_w$}", fit(hint, inner_w)),
                Style::default().fg(th().ui.muted).bg(field_bg),
            ));
        } else {
            let caret = self.editor.caret(item).cloned().unwrap_or_default();
            // The token is drawn in clear.
            let shown = text.clone();
            let look = crate::text_field::EditLook::new(
                Style::default()
                    .fg(if is_focus {
                        th().status.warning
                    } else {
                        th().ui.text
                    })
                    .bg(field_bg),
                "\u{2588}",
            );
            let line = crate::text_field::edit_line(&shown, Some(&caret), inner_w, is_focus, look);
            if let Some(real) = self.editor.caret(item) {
                real.scroll.set(caret.scroll.get());
            }
            spans.extend(line.spans);
        }
        spans.push(Span::styled(" ", bg));
        put(frame, area, x, y, spans);
        self.editor.drawn(
            item,
            Rect::new(rect.x + 1, y, rect.width.saturating_sub(2), 1),
            false,
        );
        self.hits.push((rect, Hit::Item(item), !is_focus));
    }
}

/// `text` word-wrapped to the area's width from `(x, y)`, at most
/// `max_lines` lines; returns how many lines were drawn.
pub(super) fn put_wrapped(
    frame: &mut Canvas,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    style: Style,
    max_lines: u16,
) -> u16 {
    let width = area.right().saturating_sub(x).max(1) as usize;
    let mut lines = wrap_words(text, width);
    if lines.is_empty() {
        lines.push(String::new());
    }
    let mut drawn = 0;
    for line in lines.into_iter().take(max_lines as usize) {
        put(
            frame,
            area,
            x,
            y + drawn,
            vec![Span::styled(fit(&line, width), style)],
        );
        drawn += 1;
    }
    drawn
}
