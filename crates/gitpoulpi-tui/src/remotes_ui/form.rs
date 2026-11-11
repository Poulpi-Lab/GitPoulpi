use super::*;

// ---------------------------------------------------------------------
// The Add / Edit remote popup
// ---------------------------------------------------------------------

/// Which way the popup fills its fields; every tab yields the same
/// name + URLs the `Url` tab types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteFormTab {
    Url,
}

impl RemoteFormTab {
    pub(crate) const ALL: &'static [RemoteFormTab] = &[RemoteFormTab::Url];

    fn label(self) -> &'static str {
        match self {
            RemoteFormTab::Url => "URL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemoteFormMode {
    Add,
    /// Editing an existing remote, with what it had when the popup
    /// opened - so only what changed is sent.
    Edit {
        original: String,
        fetch_url: String,
        /// The push URL a push used, which equals `fetch_url` when the
        /// remote had no separate one.
        push_url: String,
    },
}

/// The popup's focusable parts, in Tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteFormPart {
    Name,
    FetchUrl,
    PushUrl,
    Cancel,
    Submit,
}

impl RemoteFormPart {
    const ORDER: [RemoteFormPart; 5] = [
        RemoteFormPart::Name,
        RemoteFormPart::FetchUrl,
        RemoteFormPart::PushUrl,
        RemoteFormPart::Cancel,
        RemoteFormPart::Submit,
    ];

    fn is_field(self) -> bool {
        matches!(
            self,
            RemoteFormPart::Name | RemoteFormPart::FetchUrl | RemoteFormPart::PushUrl
        )
    }
}

/// A click target inside the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteFormHit {
    Tab(RemoteFormTab),
    Part(RemoteFormPart),
    /// The close cross on the frame.
    Close,
}

/// What a key or click in the popup asks of the caller.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FormOutcome {
    Stay,
    Close,
    Submit(FormSubmit),
}

/// A validated popup: the actions to send, in order, and a rename the
/// default-remote setting must follow.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormSubmit {
    pub(crate) actions: Vec<Action>,
    pub(crate) renamed: Option<(String, String)>,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoteForm {
    pub(crate) mode: RemoteFormMode,
    pub(crate) tab: RemoteFormTab,
    pub(crate) name: String,
    pub(crate) fetch_url: String,
    pub(crate) push_url: String,
    pub(crate) focus: RemoteFormPart,
    pub(crate) error: Option<String>,
    taken: Vec<String>,
    /// The popup's frame as last painted: where "outside" starts.
    popup: std::cell::Cell<Rect>,
    /// An outside click on a dirty form was swallowed: the cross blinks.
    pub(super) flash: std::cell::Cell<bool>,
    /// The three fields' caret and selection.
    edit: crate::text_edit::FieldEditor<RemoteFormPart>,
}

impl RemoteForm {
    pub(crate) fn add(taken: Vec<String>) -> Self {
        Self {
            mode: RemoteFormMode::Add,
            tab: RemoteFormTab::Url,
            name: String::new(),
            fetch_url: String::new(),
            push_url: String::new(),
            focus: RemoteFormPart::Name,
            error: None,
            taken,
            popup: Default::default(),
            flash: Default::default(),
            edit: Default::default(),
        }
    }

    /// The push field starts empty when the remote pushes where it
    /// fetches: empty means "same as fetch", which is what keeps a later
    /// fetch-URL change from leaving a stale push URL behind.
    pub(crate) fn edit(remote: &Remote, taken: Vec<String>) -> Self {
        let push = remote
            .push_url
            .clone()
            .unwrap_or_else(|| remote.fetch_url.clone());
        Self {
            mode: RemoteFormMode::Edit {
                original: remote.name.clone(),
                fetch_url: remote.fetch_url.clone(),
                push_url: push.clone(),
            },
            tab: RemoteFormTab::Url,
            name: remote.name.clone(),
            fetch_url: remote.fetch_url.clone(),
            push_url: if push == remote.fetch_url {
                String::new()
            } else {
                push
            },
            focus: RemoteFormPart::FetchUrl,
            error: None,
            taken,
            popup: Default::default(),
            flash: Default::default(),
            edit: Default::default(),
        }
    }

    fn title(&self) -> String {
        match &self.mode {
            RemoteFormMode::Add => format!(" {} ", msg::FORM_ADD_TITLE.get()),
            RemoteFormMode::Edit { original, .. } => {
                format!(" {} ", msg::form_edit_title(original))
            }
        }
    }

    fn submit_label(&self) -> String {
        let text = match self.mode {
            RemoteFormMode::Add => msg::FORM_ADD_BUTTON.get(),
            RemoteFormMode::Edit { .. } => msg::FORM_SAVE.get(),
        };
        format!(" {text} ")
    }

    /// The shared editor, and the focused text field with its text when
    /// a text field has the keyboard - split so both can be borrowed.
    fn field_parts(
        &mut self,
    ) -> (
        &mut crate::text_edit::FieldEditor<RemoteFormPart>,
        Option<(RemoteFormPart, &mut String)>,
    ) {
        let focus = self.focus;
        let field = match focus {
            RemoteFormPart::Name => Some(&mut self.name),
            RemoteFormPart::FetchUrl => Some(&mut self.fetch_url),
            RemoteFormPart::PushUrl => Some(&mut self.push_url),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    /// A left press at (`col`, `row`) after [`Self::on_click`] focused a
    /// field: places its caret there (a double-click selects a word).
    pub(crate) fn field_press(&mut self, col: u16, row: u16) {
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.dialog_mouse(
                Some(id),
                field,
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
                col,
                row,
            );
        }
    }

    /// Whether a press in a field is being dragged.
    pub(crate) fn field_dragging(&self) -> bool {
        self.edit.is_dragging()
    }

    /// A drag (or the release ending it) of a press in a field.
    pub(crate) fn field_drag(
        &mut self,
        kind: crossterm::event::MouseEventKind,
        col: u16,
        row: u16,
    ) {
        match self.field_parts() {
            (edit, Some((_, field))) => edit.dialog_mouse(None, field, kind, col, row),
            (edit, None) => edit.dialog_mouse(None, "", kind, col, row),
        };
    }

    fn step_focus(&mut self, delta: isize) {
        let order = RemoteFormPart::ORDER;
        let at = order.iter().position(|p| *p == self.focus).unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(order.len() as isize) as usize;
        self.focus = order[next];
    }

    /// Validates and turns the fields into the actions that make them so.
    ///
    /// Edit sends only what changed, rename first (the URL setters then
    /// address the remote by its new name). An empty push URL, or one
    /// equal to the fetch URL, means "push where you fetch": a separate
    /// push URL the remote had is then *removed*, not rewritten to a
    /// copy of the fetch URL that a later fetch-URL change would orphan.
    pub(crate) fn submit(&self) -> Result<FormSubmit, String> {
        let name = self.name.trim().to_string();
        let fetch = self.fetch_url.trim().to_string();
        let push = self.push_url.trim().to_string();
        let keeping = match &self.mode {
            RemoteFormMode::Edit { original, .. } => Some(original.as_str()),
            RemoteFormMode::Add => None,
        };
        validate_remote_name(&name, &self.taken, keeping)?;
        // Only a URL the user typed is judged: an edit that leaves a
        // remote's existing URL alone -- a local path, a `file://`, an
        // `insteadOf` alias this check does not know -- must not be
        // refused for a value git already accepts.
        let (old_fetch, old_push) = match &self.mode {
            RemoteFormMode::Edit {
                fetch_url,
                push_url,
                ..
            } => (Some(fetch_url.as_str()), Some(push_url.as_str())),
            RemoteFormMode::Add => (None, None),
        };
        if old_fetch != Some(fetch.as_str()) {
            validate_remote_url(&fetch).map_err(msg::fetch_url_error)?;
        }
        let separate_push = !push.is_empty() && push != fetch;
        if separate_push && old_push != Some(push.as_str()) {
            validate_remote_url(&push).map_err(msg::push_url_error)?;
        }

        let mut actions = Vec::new();
        match &self.mode {
            RemoteFormMode::Add => {
                actions.push(Action::RemoteAdd {
                    name: name.clone(),
                    url: fetch.clone(),
                });
                if separate_push {
                    actions.push(Action::RemoteSetPushUrl {
                        name: name.clone(),
                        url: Some(push),
                    });
                }
                Ok(FormSubmit {
                    actions,
                    renamed: None,
                    summary: msg::adding_remote(&name),
                })
            }
            RemoteFormMode::Edit {
                original,
                fetch_url,
                push_url,
            } => {
                let renamed = (name != *original).then(|| (original.clone(), name.clone()));
                if renamed.is_some() {
                    actions.push(Action::RemoteRename {
                        old_name: original.clone(),
                        new_name: name.clone(),
                    });
                }
                if fetch != *fetch_url {
                    actions.push(Action::RemoteSetUrl {
                        name: name.clone(),
                        url: fetch.clone(),
                    });
                }
                let had_separate_push = push_url != fetch_url;
                if separate_push {
                    if push != *push_url || !had_separate_push {
                        actions.push(Action::RemoteSetPushUrl {
                            name: name.clone(),
                            url: Some(push),
                        });
                    }
                } else if had_separate_push {
                    actions.push(Action::RemoteSetPushUrl {
                        name: name.clone(),
                        url: None,
                    });
                }
                let summary = if actions.is_empty() {
                    msg::nothing_changed(&name)
                } else {
                    msg::saving_remote(&name)
                };
                Ok(FormSubmit {
                    actions,
                    renamed,
                    summary,
                })
            }
        }
    }

    fn try_submit(&mut self) -> FormOutcome {
        match self.submit() {
            Ok(submit) => FormOutcome::Submit(submit),
            Err(e) => {
                self.error = Some(e);
                FormOutcome::Stay
            }
        }
    }

    /// The popup's whole keyboard model. Tab / Shift+Tab (and Up/Down)
    /// move between fields and buttons, Enter moves on from a field and
    /// submits from the last one or the submit button, Esc cancels.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> FormOutcome {
        // The fields' editing keys, through the shared core: caret,
        // selection, Ctrl+A/U/K, Ctrl+C/X.
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, _copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.error = None;
                }
                return FormOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => FormOutcome::Close,
            KeyCode::Tab | KeyCode::Down => {
                self.step_focus(1);
                FormOutcome::Stay
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.step_focus(-1);
                FormOutcome::Stay
            }
            KeyCode::Left if !self.focus.is_field() => {
                self.step_focus(-1);
                FormOutcome::Stay
            }
            KeyCode::Right if !self.focus.is_field() => {
                self.step_focus(1);
                FormOutcome::Stay
            }
            KeyCode::Enter => match self.focus {
                RemoteFormPart::Cancel => FormOutcome::Close,
                RemoteFormPart::Submit | RemoteFormPart::PushUrl => self.try_submit(),
                _ => {
                    self.step_focus(1);
                    FormOutcome::Stay
                }
            },
            _ => FormOutcome::Stay,
        }
    }

    /// Pasted text goes into the focused field, newlines dropped.
    pub(crate) fn on_paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
            self.error = None;
        }
    }

    /// The form holds something typed or changed.
    fn is_dirty(&self) -> bool {
        match &self.mode {
            RemoteFormMode::Add => {
                !self.name.is_empty() || !self.fetch_url.is_empty() || !self.push_url.is_empty()
            }
            RemoteFormMode::Edit {
                original,
                fetch_url,
                push_url,
            } => {
                let push = if self.push_url.is_empty() {
                    &self.fetch_url
                } else {
                    &self.push_url
                };
                self.name != *original || self.fetch_url != *fetch_url || push != push_url
            }
        }
    }

    /// A click that landed outside the popup (`col`, `row`): closes a
    /// pristine form, is swallowed (the cross blinks) when it holds typed
    /// work. `None` when the click is inside.
    pub(crate) fn on_outside_click(&self, col: u16, row: u16) -> Option<FormOutcome> {
        if self.popup.get().contains((col, row).into()) {
            return None;
        }
        if self.is_dirty() {
            self.flash.set(true);
            Some(FormOutcome::Stay)
        } else {
            Some(FormOutcome::Close)
        }
    }

    pub(crate) fn on_click(&mut self, hit: RemoteFormHit) -> FormOutcome {
        match hit {
            RemoteFormHit::Tab(tab) => {
                self.tab = tab;
                FormOutcome::Stay
            }
            RemoteFormHit::Part(RemoteFormPart::Cancel) | RemoteFormHit::Close => {
                FormOutcome::Close
            }
            RemoteFormHit::Part(RemoteFormPart::Submit) => {
                self.focus = RemoteFormPart::Submit;
                self.try_submit()
            }
            RemoteFormHit::Part(part) => {
                self.focus = part;
                FormOutcome::Stay
            }
        }
    }

    /// Draws the popup centered in `area` and returns its click targets,
    /// from the same walk that painted them.
    /// `hovered` is the Add/Edit Remote popup's own part the mouse is
    /// currently over: its tabs and buttons had no
    /// hover feedback (only the focused/active one painted anything, and
    /// focus only follows a click or Tab, never the mouse resting on a
    /// row it hasn't clicked yet).
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<RemoteFormHit>,
    ) -> Vec<(Rect, RemoteFormHit)> {
        self.edit.begin_frame();
        let width = area.width.saturating_sub(4).min(72);
        let height = 12u16.min(area.height);
        let popup = WIN.place(
            area,
            Rect::new(
                area.x + area.width.saturating_sub(width) / 2,
                area.y + area.height.saturating_sub(height) / 3,
                width,
                height,
            ),
        );
        frame.render_clipped(Clear, popup);
        frame.render_clipped(
            Block::default()
                .title(self.title())
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent))
                .style(Style::default().bg(th().ui.panel)),
            popup,
        );
        self.popup.set(popup);
        let mut hits = Vec::new();
        let buf = frame.buffer_mut();
        let cross = if self.flash.take() {
            crate::chrome::close_cross::flash(buf, popup)
        } else {
            crate::chrome::close_cross::paint(buf, popup, hovered == Some(RemoteFormHit::Close))
        };
        if let Some(cross) = cross {
            hits.push((cross, RemoteFormHit::Close));
        }
        WIN.decorate(buf, popup, cross, true);
        let inner = Rect::new(
            popup.x + 2,
            popup.y + 1,
            popup.width.saturating_sub(4),
            popup.height.saturating_sub(2),
        );
        if inner.width < 10 || inner.height < 8 {
            return hits;
        }

        // Tabs row.
        let mut x = inner.x;
        for tab in RemoteFormTab::ALL {
            let label = format!(" {} ", tab.label());
            let w = display_width(&label) as u16;
            let style = if *tab == self.tab {
                Style::default()
                    .fg(th().chrome.on_bright)
                    .bg(th().ui.accent)
            } else if hovered == Some(RemoteFormHit::Tab(*tab)) {
                Style::default()
                    .fg(th().ui.muted)
                    .patch(plain_button_style(true))
            } else {
                Style::default().fg(th().ui.muted)
            };
            let rect = Rect::new(x, inner.y, w.min(inner.width), 1);
            frame.render_clipped(Paragraph::new(Span::styled(label, style)), rect);
            hits.push((rect, RemoteFormHit::Tab(*tab)));
            x += w + 1;
        }

        // Fields.
        let rows = [
            (RemoteFormPart::Name, msg::FORM_NAME.get(), &self.name, ""),
            (
                RemoteFormPart::FetchUrl,
                msg::FORM_FETCH_URL.get(),
                &self.fetch_url,
                "https://host/owner/repo.git",
            ),
            (
                RemoteFormPart::PushUrl,
                msg::FORM_PUSH_URL.get(),
                &self.push_url,
                msg::FORM_PUSH_PLACEHOLDER.get(),
            ),
        ];
        // The label column fits the longest label plus two spaces (11 in
        // English), but never more than a third of the popup.
        let label_w = rows
            .iter()
            .map(|r| display_width(r.1) as u16 + 2)
            .max()
            .unwrap_or(11)
            .max(11)
            .min(inner.width / 3);
        let field_w = inner.width.saturating_sub(label_w);
        for (i, (part, label, value, placeholder)) in rows.iter().enumerate() {
            let y = inner.y + 2 + i as u16 * 2;
            frame.render_clipped(
                Paragraph::new(Span::styled(*label, Style::default().fg(th().ui.text))),
                Rect::new(inner.x, y, label_w, 1),
            );
            let focused = self.focus == *part;
            let text_rect = Rect::new(inner.x + label_w, y, field_w, 1);
            self.edit.drawn(*part, text_rect, false);
            let mut spans = Vec::new();
            if value.is_empty() && !focused {
                spans.push(Span::styled(
                    truncate_with_ellipsis(placeholder, field_w as usize),
                    Style::default().fg(th().ui.muted),
                ));
            } else {
                // The caret at the end is a block of PRIMARY; anywhere
                // else, the char under it reversed.
                let look = text_field::EditLook::new(Style::default(), " ")
                    .caret_style(Style::default().bg(th().ui.text));
                spans.extend(
                    text_field::edit_line(
                        value,
                        self.edit.caret(*part),
                        field_w as usize,
                        focused,
                        look,
                    )
                    .spans,
                );
            }
            let bg = crate::input_bg(focused, hovered == Some(RemoteFormHit::Part(*part)));
            let rect = Rect::new(inner.x + label_w, y, field_w, 1);
            frame.render_clipped(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
                rect,
            );
            hits.push((rect, RemoteFormHit::Part(*part)));
        }

        // Error line.
        if let Some(error) = &self.error {
            frame.render_clipped(
                Paragraph::new(Span::styled(
                    truncate_with_ellipsis(error, inner.width as usize),
                    Style::default().fg(th().status.danger),
                )),
                Rect::new(inner.x, inner.y + 8, inner.width, 1),
            );
        }

        // Buttons, right-aligned.
        let buttons = [
            (
                RemoteFormPart::Cancel,
                format!(" {} ", gitpoulpi_i18n::msg::dialog::CANCEL.get()),
            ),
            (RemoteFormPart::Submit, self.submit_label()),
        ];
        let total: u16 = buttons
            .iter()
            .map(|(_, l)| display_width(l) as u16 + 1)
            .sum();
        let mut x = inner.x + inner.width.saturating_sub(total);
        let y = inner.y + inner.height.saturating_sub(1);
        for (part, label) in buttons {
            let w = display_width(&label) as u16;
            let mut style = Style::default().fg(th().ui.on_button).bg(th().ui.button);
            if self.focus == part {
                style = style.bg(th().ui.accent).fg(th().chrome.on_bright);
            } else if hovered == Some(RemoteFormHit::Part(part)) {
                style = style.bg(crate::hover_fill(th().ui.button, th().ui.on_button, 24));
            }
            let rect = Rect::new(x, y, w, 1);
            frame.render_clipped(Paragraph::new(Span::styled(label, style)), rect);
            hits.push((rect, RemoteFormHit::Part(part)));
            x += w + 1;
        }
        hits
    }
}

/// The Add / Edit Remote popup moves (its size follows its tabs).
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("remote_form");
