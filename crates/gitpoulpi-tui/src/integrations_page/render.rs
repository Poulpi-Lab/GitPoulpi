use super::*;
use crate::th;

impl IntegrationsPage {
    fn profile_label(integration: &Integration) -> String {
        let base = match (&integration.user, integration.name.is_empty()) {
            (Some(u), true) => format!("@{}", u.login),
            (Some(u), false) => format!("{} (@{})", integration.name, u.login),
            (None, true) => integration.host().unwrap_or_else(|| integration.id.clone()),
            (None, false) => integration.name.clone(),
        };
        fit(&base, 22)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        focused: bool,
        mouse: Option<(u16, u16)>,
    ) {
        self.hits.clear();
        self.scroll.bar = None;
        self.ns_bar = None;
        self.editor.begin_frame();
        self.sync_radar(cfg);
        frame.render_widget(Clear, area);
        if area.width < 24 || area.height < 6 {
            return;
        }
        let items = self.items(cfg);
        if self.item >= items.len() {
            self.item = items.len().saturating_sub(1);
        }
        let current = items.get(self.item).copied();
        let text_focus = (focused && self.zone == Zone::Body)
            .then_some(current)
            .flatten()
            .filter(|i| i.is_text());
        self.editor.sync(text_focus);

        let types_w: u16 = if area.width >= 64 { 28 } else { 18 };
        let mut canvas = Canvas::new(frame.buffer_mut());
        let frame_canvas = &mut canvas;
        self.render_types(
            frame_canvas,
            Rect::new(area.x, area.y, types_w, area.height),
            cfg,
            focused,
        );
        for y in area.y..area.bottom() {
            put(
                frame_canvas,
                area,
                area.x + types_w,
                y,
                vec![Span::styled("\u{2502}", Style::default().fg(th().ui.muted))],
            );
        }
        let right = Rect::new(
            area.x + types_w + 2,
            area.y,
            area.width.saturating_sub(types_w + 2),
            area.height,
        );
        self.render_right(frame_canvas, right, cfg, presence, focused, current);
        if self.ssh.picker.is_none() {
            self.register_pointer();
        }
        // The combobox popup hangs over everything painted so far.
        self.radar_combo.render_popup(frame.buffer_mut(), area);

        // The key-file picker, over everything; while it is up the page
        // underneath takes no clicks.
        if self.ssh.picker.is_some() {
            self.pump_picker();
            self.hits.clear();
            if let Some(picker) = self.ssh.picker.as_mut() {
                picker.render(picker_rect(area), frame.buffer_mut());
            }
            return;
        }

        // The strong hover, over whatever was drawn.
        if let Some((mx, my)) = mouse {
            let buf = frame.buffer_mut();
            for (rect, _, hoverable) in &self.hits {
                if *hoverable && rect.contains((mx, my).into()) {
                    buf.set_style(*rect, plain_button_style(true));
                }
            }
        }
    }

    /// Tells the pointer map what this paint put under the mouse: the hand on
    /// every button, checkbox, radio and the Add profile button, the I-beam
    /// on the text fields, the hand on the Namespaces scrollbar. The hit list
    /// is in click order (first wins), the map in paint order (last wins), so
    /// it goes in reversed.
    fn register_pointer(&self) {
        for (rect, hit, _) in self.hits.iter().rev() {
            match hit {
                Hit::Item(item) if item.is_text() => crate::pointer_map::field(*rect),
                Hit::Item(_) | Hit::AddProfile => crate::pointer_map::hand(*rect),
                Hit::Type(_) | Hit::Profile(_) => {}
            }
        }
        if let Some(bar) = self.ns_bar {
            let top = self.scroll.top as i32;
            let y0 = (i32::from(bar.rect.y) - top).max(i32::from(self.scroll.window.y));
            let y1 = (i32::from(bar.rect.bottom()) - top).min(i32::from(self.scroll.window.bottom()));
            if y1 > y0 {
                crate::pointer_map::hand(Rect::new(bar.rect.x, y0 as u16, 1, (y1 - y0) as u16));
            }
        }
    }

    /// The controls the last paint put on screen, for the `Ctrl+N` badges:
    /// each with where a click on it lands. Buttons, checkboxes, radios, the
    /// Add profile button and the text fields (the Namespaces area too), in
    /// reading order.
    pub(crate) fn nav_hits(&self) -> Vec<(Rect, (u16, u16))> {
        if self.ssh.picker.is_some() || self.radar_combo.is_open() {
            return Vec::new();
        }
        let mut out: Vec<(Rect, (u16, u16))> = self
            .hits
            .iter()
            .filter(|(_, hit, _)| matches!(hit, Hit::Item(_) | Hit::AddProfile))
            .map(|(r, _, _)| {
                let before = Rect::new(r.x.saturating_sub(1), r.y, r.width + 1, 1);
                (before, (r.x + 1.min(r.width.saturating_sub(1)), r.y))
            })
            .collect();
        out.sort_by_key(|(r, _)| (r.y, r.x));
        out
    }

    fn render_types(&mut self, frame: &mut Canvas, area: Rect, cfg: &Config, focused: bool) {
        let mut y = area.y;
        let mut prev: Option<IntegrationCategory> = None;
        for (index, kind) in self.kinds.clone().iter().enumerate() {
            let cat = kind.category();
            if prev != Some(cat) {
                if prev.is_some() {
                    y += 1;
                }
                let head = match cat {
                    IntegrationCategory::Hosting => msg::HOSTING_HEAD,
                    IntegrationCategory::IssueTracker => msg::ISSUE_TRACKERS_HEAD,
                }
                .get();
                put(
                    frame,
                    area,
                    area.x,
                    y,
                    vec![Span::styled(
                        head.to_string(),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::BOLD),
                    )],
                );
                y += 1;
                prev = Some(cat);
            }
            if y >= area.bottom() {
                return;
            }
            let profiles: Vec<&Integration> = cfg
                .integrations
                .iter()
                .filter(|i| i.kind == *kind)
                .collect();
            let connected = profiles.iter().any(|i| i.user.is_some());
            let implemented = kind.capabilities().implemented;
            let selected = index == self.kind;
            let badge = if !implemented {
                msg::SOON.get().to_string()
            } else if profiles.is_empty() {
                String::new()
            } else {
                profiles.len().to_string()
            };
            let name_w = (area.width as usize).saturating_sub(9);
            let label = format!("{:<name_w$}", fit(kind.label(), name_w));
            let base = if !implemented {
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(th().ui.text)
            };
            let row_bg = if selected && focused && self.zone == Zone::Types {
                base.bg(th().ui.selection)
            } else if selected {
                base.bg(th().ui.hover)
            } else {
                base
            };
            let dot = if connected {
                Span::styled("\u{25CF}", row_bg.fg(th().refs.local))
            } else {
                Span::styled("\u{25CB}", row_bg.fg(th().ui.muted))
            };
            let line = vec![
                Span::styled(" ", row_bg),
                dot,
                Span::styled(format!(" {label} "), row_bg),
                Span::styled(format!("{badge:>4}"), row_bg.fg(th().ui.muted)),
            ];
            put(frame, area, area.x, y, line);
            let rect = Rect::new(area.x, y, area.width, 1);
            self.hits.push((rect, Hit::Type(index), true));
            y += 1;
        }
    }

    fn render_right(
        &mut self,
        frame: &mut Canvas,
        area: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        focused: bool,
        current: Option<Item>,
    ) {
        if area.width < 10 {
            return;
        }
        let kind = self.kind();
        let caps = kind.capabilities();
        let mut y = area.y;
        let head = match kind.category() {
            IntegrationCategory::Hosting => msg::HOSTING_BLURB,
            IntegrationCategory::IssueTracker => msg::ISSUE_TRACKING_BLURB,
        }
        .get();
        put(
            frame,
            area,
            area.x,
            y,
            vec![
                Span::styled(
                    kind.label().to_string(),
                    Style::default()
                        .fg(th().ui.text)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("  {head}"), Style::default().fg(th().ui.muted)),
            ],
        );
        y += 1;

        // The profile strip.
        if caps.implemented {
            let ids = self.profile_ids(cfg);
            let mut x = area.x;
            for (n, i) in ids.iter().enumerate() {
                let text = format!(" {} ", Self::profile_label(&cfg.integrations[*i]));
                let w = display_width(&text) as u16;
                if x + w > area.right() {
                    break;
                }
                let selected = n == self.sel;
                let style = if selected && focused && self.zone == Zone::Profiles {
                    Style::default().fg(th().ui.text).bg(th().ui.selection)
                } else if selected {
                    Style::default().fg(th().ui.text).bg(th().ui.hover)
                } else {
                    Style::default().fg(th().ui.muted)
                };
                put(frame, area, x, y, vec![Span::styled(text, style)]);
                self.hits
                    .push((Rect::new(x, y, w, 1), Hit::Profile(n), true));
                x += w + 1;
            }
            let add = format!(" {} ", msg::ADD_PROFILE.get());
            let w = display_width(&add) as u16;
            if x + w <= area.right() {
                let selected = self.sel >= ids.len();
                let style = if selected && focused && self.zone == Zone::Profiles {
                    Style::default().fg(th().ui.accent).bg(th().ui.selection)
                } else {
                    Style::default().fg(th().ui.accent)
                };
                put(frame, area, x, y, vec![Span::styled(add, style)]);
                self.hits
                    .push((Rect::new(x, y, w, 1), Hit::AddProfile, true));
            }
        }
        y += 2;

        let width = area.width;
        let window = Rect::new(area.x, y, width, area.bottom().saturating_sub(y));
        self.render_scrolled(frame, window, cfg, presence, focused, current);
    }

    /// Paints the selected profile's view into `body`, a rect as tall as
    /// the content may get (the caller windows it).
    pub(super) fn render_view(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        focused: bool,
        current: Option<Item>,
    ) {
        let kind = self.kind();
        match self.view(cfg) {
            View::Coming => self.render_coming(frame, body),
            View::Empty => {
                let mut y = body.y;
                put(
                    frame,
                    body,
                    body.x,
                    y,
                    muted(&msg::no_profile_yet(kind.label())),
                );
                y += 1;
                y += put_wrapped(
                    frame,
                    body,
                    body.x,
                    y,
                    msg::CONNECT_HINT.get(),
                    Style::default().fg(th().ui.muted),
                    3,
                ) + 1;
                self.button(
                    frame,
                    body,
                    body.x,
                    y,
                    msg::CONNECT.get(),
                    Item::StartConnect,
                    current,
                    focused,
                    false,
                );
            }
            View::Form => self.render_form(frame, body, cfg, presence, focused, current),
            View::Connecting => {
                let spin =
                    ["|", "/", "-", "\\"][(self.started.elapsed().as_millis() / 150 % 4) as usize];
                let host = self
                    .pending
                    .as_ref()
                    .and_then(|p| p.integration.host())
                    .unwrap_or_default();
                put(
                    frame,
                    body,
                    body.x,
                    body.y,
                    vec![Span::styled(
                        format!("{spin} {}", msg::connecting_to(host)),
                        Style::default().fg(th().status.warning),
                    )],
                );
                put(
                    frame,
                    body,
                    body.x,
                    body.y + 1,
                    muted(msg::CHECKING_TOKEN.get()),
                );
                self.button(
                    frame,
                    body,
                    body.x,
                    body.y + 3,
                    gitpoulpi_i18n::msg::dialog::CANCEL.get(),
                    Item::CancelConnect,
                    current,
                    focused,
                    false,
                );
            }
            View::Connected(i) => {
                self.render_connected(frame, body, cfg, presence, i, focused, current)
            }
            View::ConfirmKey(i) => {
                self.render_confirm_key(frame, body, &cfg.integrations[i], focused, current)
            }
            View::Confirm(i) => {
                let e = &cfg.integrations[i];
                put(
                    frame,
                    body,
                    body.x,
                    body.y,
                    vec![Span::styled(
                        msg::remove_profile_q(Self::profile_label(e)),
                        Style::default()
                            .fg(th().status.danger)
                            .add_modifier(Modifier::BOLD),
                    )],
                );
                put(
                    frame,
                    body,
                    body.x,
                    body.y + 1,
                    muted(msg::REMOVE_NOTE.get()),
                );
                let x = body.x;
                let w = self.button(
                    frame,
                    body,
                    x,
                    body.y + 3,
                    msg::REMOVE.get(),
                    Item::ConfirmRemove,
                    current,
                    focused,
                    false,
                );
                self.button(
                    frame,
                    body,
                    x + w + 2,
                    body.y + 3,
                    msg::KEEP.get(),
                    Item::CancelRemove,
                    current,
                    focused,
                    false,
                );
            }
        }
    }

    /// "Delete the key files?" - names every path that will go.
    fn render_confirm_key(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        e: &Integration,
        focused: bool,
        current: Option<Item>,
    ) {
        let ssh = e.ssh.clone().unwrap_or_default();
        let width = body.width.saturating_sub(10) as usize;
        let mut y = body.y;
        put(
            frame,
            body,
            body.x,
            y,
            vec![Span::styled(
                msg::DELETE_KEY_Q.get(),
                Style::default()
                    .fg(th().status.danger)
                    .add_modifier(Modifier::BOLD),
            )],
        );
        y += 2;
        for (label, path) in [
            (msg::KEY_PRIVATE.get(), &ssh.private_key_path),
            (msg::KEY_PUBLIC.get(), &ssh.public_key_path),
        ] {
            if let Some(path) = path {
                put(
                    frame,
                    body,
                    body.x,
                    y,
                    vec![
                        Span::styled(format!("{label:<8}"), Style::default().fg(th().ui.muted)),
                        Span::styled(tail_fit(path, width), Style::default().fg(th().ui.text)),
                    ],
                );
                y += 1;
            }
        }
        y += 1;
        let note = if ssh.remote_key_id.is_some() {
            msg::delete_note_remote(e.host().unwrap_or_default())
        } else {
            msg::DELETE_NOTE.get().to_string()
        };
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            &note,
            Style::default().fg(th().ui.muted),
            4,
        ) + 1;
        let w = self.button(
            frame,
            body,
            body.x,
            y,
            msg::DELETE_FILES.get(),
            Item::SshConfirmDelete,
            current,
            focused,
            false,
        );
        self.button(
            frame,
            body,
            body.x + w + 2,
            y,
            msg::KEEP.get(),
            Item::SshCancelDelete,
            current,
            focused,
            false,
        );
    }

    fn render_coming(&mut self, frame: &mut Canvas, body: Rect) {
        let kind = self.kind();
        put(
            frame,
            body,
            body.x,
            body.y,
            vec![Span::styled(
                msg::coming_soon(kind.label()),
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::ITALIC),
            )],
        );
        put(
            frame,
            body,
            body.x,
            body.y + 2,
            muted(msg::HOST_DISABLED.get()),
        );
        put(
            frame,
            body,
            body.x,
            body.y + 3,
            muted(msg::TOKEN_DISABLED.get()),
        );
        put_wrapped(
            frame,
            body,
            body.x,
            body.y + 5,
            msg::COMING_NOTE.get(),
            Style::default().fg(th().ui.muted),
            3,
        );
    }
}

pub(super) fn put(frame: &mut Canvas, area: Rect, x: u16, y: u16, spans: Vec<Span<'static>>) {
    if y >= area.bottom() || x >= area.right() || y < area.y {
        return;
    }
    frame
        .buffer_mut()
        .set_line(x, y, &Line::from(spans), area.right() - x);
}

pub(super) fn muted(text: &str) -> Vec<Span<'static>> {
    vec![Span::styled(
        text.to_string(),
        Style::default().fg(th().ui.muted),
    )]
}

pub(super) fn label(text: &str) -> Vec<Span<'static>> {
    vec![Span::styled(
        text.to_string(),
        Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD),
    )]
}

pub(super) fn kv(key: &str, value: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{key:<7}"), Style::default().fg(th().ui.muted)),
        Span::styled(value.to_string(), Style::default().fg(th().ui.text)),
    ]
}
