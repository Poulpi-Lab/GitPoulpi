use super::*;

impl PullRequestDialog {
    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.edit.sync(match self.focus {
            PrFocus::Title | PrFocus::Body => Some(self.focus),
            _ => None,
        });
        self.edit.begin_frame();
        self.sync_meta_scope();
        self.guard.panel(buf, rect, msg::pull_request::TITLE.get());
        let mut hits: Vec<(Rect, PrHit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 15 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = inner.y;

        // Provider tabs: only the supported ones.
        let mut x = inner.x;
        let mut tab_rects = Vec::new();
        for (i, tab) in self.seed.tabs.iter().enumerate() {
            let selected = i == self.tab;
            let hovered = self.hover == Some(PrHit::Tab(i));
            let focused = self.focus == PrFocus::Tabs && selected;
            let text = format!(" {} ", tab.label);
            let style = Style::default()
                .fg(if selected {
                    th().ui.accent
                } else {
                    th().ui.text
                })
                .bg(if focused || hovered {
                    crate::lighten(th().ui.hover, 10)
                } else {
                    th().ui.panel
                })
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            let r = dialog_kit::put(buf, x, y, &text, inner.right().saturating_sub(x), style);
            if r.width == 0 {
                break;
            }
            hits.push((r, PrHit::Tab(i)));
            tab_rects.push(r);
            x = r.right() + 1;
        }
        // "Create on GitHub": the provider's own form, right end of the tab
        // row (still there after an API error); first in the hit list.
        let web_label = msg::issue_form::create_on_host(self.provider_name());
        let web_w = dialog_kit::button_width(&web_label);
        let web = dialog_kit::button(
            buf,
            inner.right().saturating_sub(web_w),
            y,
            &web_label,
            ButtonState::Normal {
                hovered: self.hover == Some(PrHit::Part(PrFocus::Web)),
                focused: self.focus == PrFocus::Web,
            },
        );
        hits.insert(0, (web, PrHit::Part(PrFocus::Web)));
        y += 1;
        dialog_kit::put(
            buf,
            inner.x,
            y,
            &"\u{2500}".repeat(inner.width as usize),
            inner.width,
            Style::default().fg(th().ui.hover).bg(th().ui.panel),
        );
        if let Some(r) = tab_rects.get(self.tab) {
            dialog_kit::put(
                buf,
                r.x,
                y,
                &"\u{2501}".repeat(r.width as usize),
                r.width,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        }
        y += 2;

        // From -> To.
        let arrow_w = 3u16;
        let col_w = inner.width.saturating_sub(arrow_w) / 2;
        let left_x = inner.x;
        let right_x = inner.x + col_w + arrow_w;
        dialog_kit::put(
            buf,
            left_x,
            y,
            msg::pull_request::FROM_REPO.get(),
            col_w,
            muted,
        );
        dialog_kit::put(
            buf,
            right_x,
            y,
            msg::pull_request::TO_REPO.get(),
            col_w,
            muted,
        );
        y += 1;
        // Both columns share one start and the right one runs to the
        // form's right edge (`col_w` alone loses a column to the
        // integer division), so the fields end where the Title does.
        let right_w = inner.right().saturating_sub(right_x);
        let cell = |x: u16, y: u16| Rect::new(x, y, if x == left_x { col_w } else { right_w }, 1);
        for (which, x) in [(PrFocus::FromRepo, left_x), (PrFocus::ToRepo, right_x)] {
            let rect = cell(x, y);
            let focused = self.focus == which;
            let hovered = self.hover == Some(PrHit::Part(which));
            if let Some(c) = self.combo_mut(which) {
                c.render_field(buf, rect, focused, hovered);
            }
            hits.push((rect, PrHit::Part(which)));
        }
        dialog_kit::put(
            buf,
            left_x + col_w + 1,
            y,
            "\u{2192}",
            1,
            Style::default().fg(th().ui.text).bg(th().ui.panel),
        );
        y += 1;
        dialog_kit::put(
            buf,
            left_x,
            y,
            msg::pull_request::BRANCH.get(),
            col_w,
            muted,
        );
        dialog_kit::put(
            buf,
            right_x,
            y,
            msg::pull_request::BRANCH.get(),
            col_w,
            muted,
        );
        y += 1;
        for (which, x) in [(PrFocus::FromBranch, left_x), (PrFocus::ToBranch, right_x)] {
            let rect = cell(x, y);
            let focused = self.focus == which;
            let hovered = self.hover == Some(PrHit::Part(which));
            if let Some(c) = self.combo_mut(which) {
                c.render_field(buf, rect, focused, hovered);
            }
            hits.push((rect, PrHit::Part(which)));
        }
        y += 2;

        // Title (+ the hidden AI hook).
        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::pull_request::FIELD_TITLE.get(),
            inner.width,
            muted,
        );
        if self.ai_generate {
            let label = if self.generating {
                format!("{AI_SPARK} {}", msg::pull_request::GENERATING.get())
            } else {
                format!("{AI_SPARK} {}", GENERATE_LABEL.get())
            };
            let label = label.as_str();
            let w = dialog_kit::button_width(label);
            let r = dialog_kit::button(
                buf,
                inner.right().saturating_sub(w),
                y,
                label,
                if self.generating {
                    ButtonState::Disabled
                } else {
                    ButtonState::Normal {
                        hovered: self.hover == Some(PrHit::Part(PrFocus::Generate)),
                        focused: self.focus == PrFocus::Generate,
                    }
                },
            );
            // The sparkle keeps the AI purple whatever the button's state.
            if !self.generating {
                buf[(r.x + 1, r.y)].set_fg(th().status.ai);
            }
            hits.push((r, PrHit::Part(PrFocus::Generate)));
        }
        y += 1;
        let title = Rect::new(inner.x, y, inner.width, 1);
        dialog_kit::text_input(
            buf,
            title,
            &self.title,
            msg::pull_request::TITLE_PLACEHOLDER.get(),
            self.focus == PrFocus::Title,
            self.hover == Some(PrHit::Part(PrFocus::Title)),
            (&self.edit, PrFocus::Title),
        );
        hits.push((title, PrHit::Part(PrFocus::Title)));
        y += 2;

        // Description: as tall as the room allows.
        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::pull_request::FIELD_DESCRIPTION.get(),
            inner.width,
            muted,
        );
        y += 1;
        let meta_rows = if self.uses_meta() { 2 } else { 0 };
        let footer_rows = 4 + meta_rows; // metadata, draft, message, buttons, breathing room
        let body_h = (inner.bottom().saturating_sub(y + footer_rows)).clamp(2, 8);
        let body = Rect::new(inner.x, y, inner.width, body_h);
        self.body_bar = dialog_kit::text_area(
            buf,
            body,
            &self.body,
            msg::pull_request::DESCRIPTION_PLACEHOLDER.get(),
            self.focus == PrFocus::Body,
            self.hover == Some(PrHit::Part(PrFocus::Body)),
            (&self.edit, PrFocus::Body),
        );
        hits.push((body, PrHit::Part(PrFocus::Body)));
        y += body_h + 1;

        if self.uses_meta() {
            // Assignees, reviewers, labels and milestone: one row of
            // buttons, each opening the picker.
            let fields = [
                MetaField::Assignees,
                MetaField::Reviewers,
                MetaField::Labels,
                MetaField::Milestone,
            ];
            let focus = self.focus.meta_field();
            let hover = match self.hover {
                Some(PrHit::Part(part)) => part.meta_field(),
                _ => None,
            };
            for (r, field) in
                self.meta
                    .render_chips(buf, (inner.x, y), inner.width, &fields, focus, hover)
            {
                hits.push((r, PrHit::Part(PrFocus::of_meta(field))));
            }
            y += 2;
        }

        let draft = dialog_kit::checkbox(
            buf,
            inner.x,
            y,
            inner.width,
            msg::pull_request::DRAFT.get(),
            self.draft,
            CheckState::Enabled {
                hovered: self.hover == Some(PrHit::Part(PrFocus::Draft)),
                focused: self.focus == PrFocus::Draft,
            },
        );
        hits.push((draft, PrHit::Part(PrFocus::Draft)));

        let bottom = inner.y + inner.height;
        let msg_y = bottom.saturating_sub(2);
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        } else if self.busy {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                msg::pull_request::OPENING.get(),
                inner.width,
                muted,
            );
        } else if self.hover == Some(PrHit::Part(PrFocus::Web)) {
            // The hover tooltip (the global one is off under a dialog).
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                &msg::pull_request::tip_web(self.provider_name()),
                inner.width,
                muted,
            );
        } else if self.is_loading() {
            // The host's repository list is still on its way: say so, with
            // a glyph that turns (the loop redraws every ~33 ms).
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                &format!("{} {}", spinner_glyph(), LOADING_TEXT.get()),
                inner.width,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        } else if let Err(reason) = self.plan() {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                &reason,
                inner.width,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        }

        let button_y = bottom.saturating_sub(1);
        let cancel_w = dialog_kit::button_width(msg::pull_request::CANCEL.get());
        let create_label = msg::pull_request::TITLE.get();
        let create_w = dialog_kit::button_width(create_label);
        let create_x = inner.right().saturating_sub(create_w);
        let cancel_x = create_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            msg::pull_request::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(PrHit::Part(PrFocus::Cancel)),
                focused: self.focus == PrFocus::Cancel,
            },
        );
        hits.push((cancel, PrHit::Part(PrFocus::Cancel)));
        let create = dialog_kit::button(
            buf,
            create_x,
            button_y,
            create_label,
            if self.busy || self.plan().is_err() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(PrHit::Part(PrFocus::Create)),
                    focused: self.focus == PrFocus::Create,
                }
            },
        );
        hits.push((create, PrHit::Part(PrFocus::Create)));
        let create_key = dialog_kit::key_label(Bind::DialogSubmit);
        let web_key = dialog_kit::key_label(Bind::DialogWeb);
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (
                    msg::pull_request::KEY_TAB.get(),
                    msg::pull_request::HINT_NEXT.get(),
                ),
                (create_key.as_str(), msg::pull_request::HINT_CREATE.get()),
                (web_key.as_str(), msg::issue_form::HINT_WEB.get()),
            ],
        );

        // The open popup last, over everything else.
        let bounds = buf.area;
        for which in [
            PrFocus::FromRepo,
            PrFocus::FromBranch,
            PrFocus::ToRepo,
            PrFocus::ToBranch,
        ] {
            if let Some(c) = self.combo_mut(which) {
                c.render_popup(buf, bounds);
            }
        }
        self.meta.render_popup(buf);
        self.guard.render(buf);
        crate::pointer_map::from_hits(&hits, |h| match h {
            PrHit::Tab(_) => Some(crate::pointer_map::Kind::Hand),
            PrHit::Part(_) => None,
        });
        self.hits = hits;
    }

    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    pub(super) fn hit_rect(&self, hit: PrHit) -> Rect {
        self.hits
            .iter()
            .find(|(_, h)| *h == hit)
            .map(|(r, _)| *r)
            .unwrap_or_else(|| panic!("{hit:?} not drawn"))
    }
}

/// The rectangle the form dialog draws in, centered like every other
/// repo dialog.
pub(crate) fn pull_request_rect(area: Rect) -> Rect {
    dialog_kit::centered(area, area.width.saturating_sub(4).min(86), 30)
}

/// What a running `create_pull_request` call reports.
pub(crate) enum PullRequestEvent {
    Finished(Result<CreatedPullRequest, String>),
}

pub(crate) struct PullRequestReceiver {
    rx: std_mpsc::Receiver<PullRequestEvent>,
}

impl PullRequestReceiver {
    pub(crate) fn try_recv(&self) -> Option<PullRequestEvent> {
        self.rx.try_recv().ok()
    }
}

/// Starts the API call on its own thread - the same shape as
/// `clone_task::start`, fire-and-forget, reporting on a channel the
/// render loop drains with `try_recv` and never blocking the input loop.
pub(crate) fn start_create(integration: Integration, req: NewPullRequest) -> PullRequestReceiver {
    let (tx, rx) = std_mpsc::channel();
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = tx.send_notify(PullRequestEvent::Finished(Err(
                    crate::hosting_task::request_task_failed(e),
                )));
                return;
            }
        };
        runtime.block_on(async move {
            let result = create(integration, req).await;
            let _ = tx.send_notify(PullRequestEvent::Finished(result));
        });
    });
    PullRequestReceiver { rx }
}

async fn create(
    integration: Integration,
    req: NewPullRequest,
) -> Result<CreatedPullRequest, String> {
    gitpoulpi_hosting::TrustGate::ambient()
        .preflight(&integration)
        .map_err(|e| crate::hosting_task::error_text(&e))?;
    let token = DefaultTokenSource::for_integrations(&load_integrations())
        .token(&integration)
        .map_err(|e| crate::hosting_task::error_text(&e))?;
    let transport = std::sync::Arc::new(
        gitpoulpi_hosting::ReqwestTransport::new()
            .map_err(|e| crate::hosting_task::error_text(&e))?,
    );
    let client = client_for(
        integration,
        token,
        transport,
        &gitpoulpi_hosting::TrustGate::ambient(),
    )
    .map_err(|e| crate::hosting_task::error_text(&e))?;
    client
        .create_pull_request(&req)
        .await
        .map_err(|e| crate::hosting_task::error_text(&e))
}
